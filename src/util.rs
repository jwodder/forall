use crate::cmd::{CommandError, CommandOutputError, CommandPlus};
use crate::logging::Verbosity;
use crate::project::Project;
use clap::{ArgAction, Args};
use ghrepo::GHRepo;
use std::ffi::OsString;
use std::path::Path;
use thiserror::Error;

#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    /// Don't exit on errors
    #[arg(short, long, global = true)]
    pub(crate) keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub(crate) quiet: u8,

    #[arg(short, long, global = true, conflicts_with = "quiet")]
    pub(crate) verbose: bool,
}

impl Options {
    pub(crate) fn verbosity(&self) -> Verbosity {
        match (self.quiet, self.verbose) {
            (0, false) => Verbosity::Normal,
            (1, false) => Verbosity::Quiet,
            (_, false) => Verbosity::Quiet2,
            (0, true) => Verbosity::Verbose,
            // Work around <https://github.com/clap-rs/clap/issues/5899>:
            (1, true) => Verbosity::Normal,
            (2, true) => Verbosity::Quiet,
            (_, true) => Verbosity::Quiet2,
        }
    }

    pub(crate) fn quiet(&self) -> bool {
        self.quiet - u8::from(self.verbose) > 0
    }

    pub(crate) fn quiet2(&self) -> bool {
        self.quiet - u8::from(self.verbose) > 1
    }
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunOpts {
    /// Treat the command as a path to a script file.
    ///
    /// The path is canonicalized, and it is run via `perl` for its shebang
    /// handling; thus, the script need not be executable, but it does need to
    /// have an appropriate shebang.
    #[arg(long, conflicts_with = "shell")]
    pub(crate) script: bool,

    /// Run command in a shell
    #[arg(long)]
    pub(crate) shell: bool,

    #[arg(allow_hyphen_values = true, trailing_var_arg = true, required = true)]
    pub(crate) command: Vec<OsString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Runner {
    command: OsString,
    args: Vec<OsString>,
}

impl Runner {
    pub(crate) fn run(&self, p: &Project, opts: Options) -> Result<bool, CommandError> {
        p.runcmd(&self.command)
            .args(self.args.iter())
            .quiet(opts.quiet2())
            .keep_going(opts.keep_going)
            .run()
    }
}

impl TryFrom<RunOpts> for Runner {
    type Error = RunOptsError;

    fn try_from(value: RunOpts) -> Result<Runner, RunOptsError> {
        let (command, args) = if value.shell {
            let cmd = get_shell();
            let mut args = Vec::with_capacity(value.command.len().saturating_add(1));
            args.push(OsString::from("-c"));
            args.extend(value.command);
            (cmd, args)
        } else if value.script {
            // Use perl to interpret the script's shebang, thereby supporting
            // non-executable scripts
            let cmd = OsString::from("perl");
            let mut args = Vec::with_capacity(value.command.len());
            let mut iter = value.command.into_iter();
            let scriptfile = iter.next().expect("command should be nonempty");
            args.push(
                fs_err::canonicalize(scriptfile)
                    .map_err(RunOptsError::Canonicalize)?
                    .into_os_string(),
            );
            args.extend(iter);
            (cmd, args)
        } else {
            let mut iter = value.command.into_iter();
            let cmd = iter.next().expect("command should be nonempty");
            (cmd, iter.collect())
        };
        Ok(Runner { command, args })
    }
}

#[derive(Debug, Error)]
pub(crate) enum RunOptsError {
    #[error("failed to canonicalize script path")]
    Canonicalize(#[from] std::io::Error),
}

pub(crate) fn get_ghrepo(p: &Path) -> anyhow::Result<Option<GHRepo>> {
    // Don't use ghrepo::LocalRepo::github_remote(), as that doesn't suppress
    // stderr from Git.
    match CommandPlus::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(p)
        .trace(true)
        .stderr(std::process::Stdio::null())
        .check_output()
    {
        Ok(s) => Ok(GHRepo::from_url(s.trim()).ok()),
        Err(CommandOutputError::Decode { .. }) => Ok(None),
        Err(CommandOutputError::Exit { rc, .. }) if rc.code() == Some(2) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn get_shell() -> OsString {
    std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("sh"))
}
