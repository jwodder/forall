use crate::logging::{Verbosity, is_active, logcmd};
use bstr::ByteVec; // into_string_lossy()
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CommandKind {
    Run,
    #[default]
    Operational,
    Filter,
}

impl CommandKind {
    fn cmdline_verbosity(&self) -> Verbosity {
        match self {
            CommandKind::Run => Verbosity::Normal,
            CommandKind::Operational => Verbosity::Normal,
            CommandKind::Filter => Verbosity::Verbose,
        }
    }

    fn output_verbosity(&self) -> Verbosity {
        match self {
            CommandKind::Run => Verbosity::Quiet,
            CommandKind::Operational => Verbosity::Normal,
            CommandKind::Filter => Verbosity::Off,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandPlus {
    cmdline: CommandLine,
    cmd: Command,
    kind: CommandKind,
}

impl CommandPlus {
    pub(crate) fn new<S: AsRef<OsStr>>(arg0: S) -> Self {
        let arg0 = arg0.as_ref();
        CommandPlus {
            cmdline: CommandLine::new(arg0),
            cmd: Command::new(arg0),
            kind: CommandKind::default(),
        }
    }

    pub(crate) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        let arg = arg.as_ref();
        self.cmdline.arg(arg);
        self.cmd.arg(arg);
        self
    }

    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            let arg = arg.as_ref();
            self.cmdline.arg(arg);
            self.cmd.arg(arg);
        }
        self
    }

    pub(crate) fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        let dir = dir.as_ref();
        self.cmdline.current_dir(dir);
        self.cmd.current_dir(dir);
        self
    }

    pub(crate) fn kind(&mut self, k: CommandKind) -> &mut Self {
        self.kind = k;
        self
    }

    pub(crate) fn cmdline(&self) -> &CommandLine {
        &self.cmdline
    }

    pub(crate) fn run(&mut self) -> Result<(), CommandError> {
        logcmd(self, self.kind.cmdline_verbosity());
        let (rc, stdout, stderr) = if !is_active(self.kind.output_verbosity()) {
            let output = self.cmd.output().map_err(|source| CommandError::Startup {
                cmdline: self.cmdline().clone(),
                source,
            })?;
            (
                output.status,
                Some(output.stdout.into_string_lossy()),
                Some(output.stderr.into_string_lossy()),
            )
        } else {
            (
                self.cmd.status().map_err(|source| CommandError::Startup {
                    cmdline: self.cmdline().clone(),
                    source,
                })?,
                None,
                None,
            )
        };
        if rc.success() {
            Ok(())
        } else {
            Err(CommandError::Exit {
                cmdline: self.cmdline().clone(),
                rc,
                stdout,
                stderr,
            })
        }
    }

    pub(crate) fn status(&mut self) -> Result<ExitStatus, CommandError> {
        logcmd(self, self.kind.cmdline_verbosity());
        self.cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|source| CommandError::Startup {
                cmdline: self.cmdline().clone(),
                source,
            })
    }

    pub(crate) fn check_output(&mut self) -> Result<String, CommandError> {
        logcmd(self, self.kind.cmdline_verbosity());
        match self.cmd.output() {
            Ok(output) if output.status.success() => Ok(output.stdout.into_string_lossy()),
            Ok(output) => Err(CommandError::Exit {
                cmdline: self.cmdline().clone(),
                rc: output.status,
                stdout: Some(output.stdout.into_string_lossy()),
                stderr: Some(output.stderr.into_string_lossy()),
            }),
            Err(source) => Err(CommandError::Startup {
                cmdline: self.cmdline().clone(),
                source,
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandLine {
    line: String,
    cwd: Option<PathBuf>,
}

impl CommandLine {
    fn new(arg0: &OsStr) -> CommandLine {
        CommandLine {
            line: quote_osstr(arg0),
            cwd: None,
        }
    }

    fn arg(&mut self, arg: &OsStr) {
        self.line.push(' ');
        self.line.push_str(&quote_osstr(arg));
    }

    fn current_dir(&mut self, cwd: &Path) {
        self.cwd = Some(cwd.to_owned());
    }
}

impl fmt::Display for CommandLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{tick}{line}{tick}",
            line = self.line,
            tick = if f.alternate() { "`" } else { "" }
        )?;
        if let Some(ref cwd) = self.cwd {
            write!(f, " [cwd={}]", cwd.display())?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub(crate) enum CommandError {
    #[error("failed to run {cmdline:#}")]
    Startup {
        cmdline: CommandLine,
        source: std::io::Error,
    },
    #[error("command {cmdline:#} failed: {rc}")]
    Exit {
        cmdline: CommandLine,
        rc: ExitStatus,
        stdout: Option<String>,
        stderr: Option<String>,
    },
}

impl CommandError {
    pub(crate) fn stdout(&self) -> Option<&str> {
        if let CommandError::Exit { stdout, .. } = self {
            stdout.as_deref()
        } else {
            None
        }
    }

    pub(crate) fn stderr(&self) -> Option<&str> {
        if let CommandError::Exit { stderr, .. } = self {
            stderr.as_deref()
        } else {
            None
        }
    }
}

fn quote_osstr(s: &OsStr) -> String {
    shell_words::quote(&s.to_string_lossy()).to_string()
}
