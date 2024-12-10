use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct CommandPlus {
    cmdline: String,
    cmd: Command,
}

impl CommandPlus {
    pub(crate) fn new<S: AsRef<OsStr>>(arg0: S) -> Self {
        let arg0 = arg0.as_ref();
        CommandPlus {
            cmdline: quote_osstr(arg0),
            cmd: Command::new(arg0),
        }
    }

    pub(crate) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        let arg = arg.as_ref();
        self.cmdline.push(' ');
        self.cmdline.push_str(&quote_osstr(arg));
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
            self.cmdline.push(' ');
            self.cmdline.push_str(&quote_osstr(arg));
            self.cmd.arg(arg);
        }
        self
    }

    pub(crate) fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.cmd.current_dir(dir);
        self
    }

    pub(crate) fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.cmd.stdout(cfg);
        self
    }

    pub(crate) fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.cmd.stderr(cfg);
        self
    }

    pub(crate) fn status(&mut self) -> Result<(), CommandError> {
        match self.cmd.status() {
            Ok(rc) if rc.success() => Ok(()),
            Ok(rc) => Err(CommandError::Exit {
                cmdline: self.cmdline.clone(),
                rc,
            }),
            Err(source) => Err(CommandError::Startup {
                cmdline: self.cmdline.clone(),
                source,
            }),
        }
    }

    pub(crate) fn check_output(&mut self) -> Result<String, CommandOutputError> {
        match self.cmd.stderr(Stdio::inherit()).output() {
            Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
                Ok(s) => Ok(s),
                Err(e) => Err(CommandOutputError::Decode {
                    cmdline: self.cmdline.clone(),
                    source: e.utf8_error(),
                }),
            },
            Ok(output) => Err(CommandOutputError::Exit {
                cmdline: self.cmdline.clone(),
                rc: output.status,
            }),
            Err(source) => Err(CommandOutputError::Startup {
                cmdline: self.cmdline.clone(),
                source,
            }),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum CommandError {
    #[error("failed to run `{cmdline}`")]
    Startup {
        cmdline: String,
        source: std::io::Error,
    },
    #[error("command `{cmdline}` failed: {rc}")]
    Exit { cmdline: String, rc: ExitStatus },
}

#[derive(Debug, Error)]
pub(crate) enum CommandOutputError {
    #[error("failed to run `{cmdline}`")]
    Startup {
        cmdline: String,
        source: std::io::Error,
    },
    #[error("command `{cmdline}` failed: {rc}")]
    Exit { cmdline: String, rc: ExitStatus },
    #[error("could not decode `{cmdline}` output")]
    Decode {
        cmdline: String,
        source: std::str::Utf8Error,
    },
}

fn quote_osstr(s: &OsStr) -> String {
    shell_words::quote(&s.to_string_lossy()).to_string()
}
