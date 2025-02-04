use crate::logging::{is_active, logcmd, Verbosity};
use std::ffi::OsStr;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
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
    keep_going: bool,
    stderr_set: bool,
}

impl CommandPlus {
    pub(crate) fn new<S: AsRef<OsStr>>(arg0: S) -> Self {
        let arg0 = arg0.as_ref();
        CommandPlus {
            cmdline: CommandLine::new(arg0),
            cmd: Command::new(arg0),
            kind: CommandKind::default(),
            keep_going: false,
            stderr_set: false,
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

    pub(crate) fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.cmd.stdout(cfg);
        self
    }

    pub(crate) fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.cmd.stderr(cfg);
        self.stderr_set = true;
        self
    }

    pub(crate) fn kind(&mut self, k: CommandKind) -> &mut Self {
        self.kind = k;
        self
    }

    pub(crate) fn keep_going(&mut self, yes: bool) -> &mut Self {
        self.keep_going = yes;
        self
    }

    pub(crate) fn cmdline(&self) -> &CommandLine {
        &self.cmdline
    }

    pub(crate) fn run(&mut self) -> Result<bool, CommandError> {
        logcmd(self, self.kind.cmdline_verbosity());
        let rc = if !is_active(self.kind.output_verbosity()) {
            let (output, rc) = self.combine_stdout_stderr()?;
            if !rc.success() {
                // TODO: Better error handling here:
                let _ = std::io::stdout().write_all(output.as_bytes());
            }
            rc
        } else {
            self.cmd.status().map_err(|source| CommandError::Startup {
                cmdline: self.cmdline().clone(),
                source,
            })?
        };
        if rc.success() {
            Ok(true)
        } else if self.keep_going {
            if let Some(code) = rc.code() {
                error!("[{code}]");
            } else {
                error!("[{rc}]");
            }
            Ok(false)
        } else {
            Err(CommandError::Exit {
                cmdline: self.cmdline().clone(),
                rc,
            })
        }
    }

    pub(crate) fn status(&mut self) -> Result<ExitStatus, CommandError> {
        logcmd(self, self.kind.cmdline_verbosity());
        self.cmd.status().map_err(|source| CommandError::Startup {
            cmdline: self.cmdline().clone(),
            source,
        })
    }

    pub(crate) fn check_output(&mut self) -> Result<String, CommandOutputError> {
        logcmd(self, self.kind.cmdline_verbosity());
        if !self.stderr_set {
            self.cmd.stderr(Stdio::inherit());
        }
        match self.cmd.output() {
            Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
                Ok(s) => Ok(s),
                Err(e) => Err(CommandOutputError::Decode {
                    cmdline: self.cmdline().clone(),
                    source: e.utf8_error(),
                }),
            },
            Ok(output) => Err(CommandOutputError::Exit {
                cmdline: self.cmdline().clone(),
                rc: output.status,
            }),
            Err(source) => Err(CommandOutputError::Startup {
                cmdline: self.cmdline().clone(),
                source,
            }),
        }
    }

    fn combine_stdout_stderr(&mut self) -> Result<(String, ExitStatus), CombinedCommandError> {
        logcmd(self, self.kind.cmdline_verbosity());
        // <https://stackoverflow.com/a/72831067/744178>
        let mut child = self
            .cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| CombinedCommandError::Startup {
                cmdline: self.cmdline().clone(),
                source,
            })?;
        let child_stdout = child
            .stdout
            .take()
            .expect("child.stdout should be non-None");
        let child_stderr = child
            .stderr
            .take()
            .expect("child.stderr should be non-None");

        let (sender, receiver) = std::sync::mpsc::channel();

        let stdout_sender = sender.clone();
        let stdout_thread = std::thread::spawn(move || {
            let mut stdout = BufReader::new(child_stdout);
            loop {
                let mut line = String::new();
                if stdout.read_line(&mut line)? == 0 {
                    break;
                }
                if stdout_sender.send(line).is_err() {
                    break;
                }
            }
            Ok(())
        });

        let stderr_sender = sender.clone();
        let stderr_thread = std::thread::spawn(move || {
            let mut stderr = BufReader::new(child_stderr);
            loop {
                let mut line = String::new();
                if stderr.read_line(&mut line)? == 0 {
                    break;
                }
                if stderr_sender.send(line).is_err() {
                    break;
                }
            }
            Ok(())
        });

        drop(sender);

        let rc = child.wait().map_err(|source| CombinedCommandError::Wait {
            cmdline: self.cmdline().clone(),
            source,
        })?;

        match stdout_thread.join() {
            Ok(Ok(())) => (),
            Ok(Err(source)) => {
                return Err(CombinedCommandError::Read {
                    cmdline: self.cmdline().clone(),
                    source,
                })
            }
            Err(barf) => std::panic::resume_unwind(barf),
        }

        match stderr_thread.join() {
            Ok(Ok(())) => (),
            Ok(Err(source)) => {
                return Err(CombinedCommandError::Read {
                    cmdline: self.cmdline().clone(),
                    source,
                })
            }
            Err(barf) => std::panic::resume_unwind(barf),
        }

        let output = receiver.into_iter().collect::<String>();
        Ok((output, rc))
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
    },
    #[error(transparent)]
    Combined(#[from] CombinedCommandError),
}

#[derive(Debug, Error)]
pub(crate) enum CommandOutputError {
    #[error("failed to run {cmdline:#}")]
    Startup {
        cmdline: CommandLine,
        source: std::io::Error,
    },
    #[error("command {cmdline:#} failed: {rc}")]
    Exit {
        cmdline: CommandLine,
        rc: ExitStatus,
    },
    #[error("could not decode {cmdline:#} output")]
    Decode {
        cmdline: CommandLine,
        source: std::str::Utf8Error,
    },
}

#[derive(Debug, Error)]
pub(crate) enum CombinedCommandError {
    #[error("failed to run {cmdline:#}")]
    Startup {
        cmdline: CommandLine,
        source: std::io::Error,
    },
    #[error("error reading from {cmdline:#}")]
    Read {
        cmdline: CommandLine,
        source: std::io::Error,
    },
    #[error("error waiting for {cmdline:#} to terminate")]
    Wait {
        cmdline: CommandLine,
        source: std::io::Error,
    },
}

fn quote_osstr(s: &OsStr) -> String {
    shell_words::quote(&s.to_string_lossy()).to_string()
}
