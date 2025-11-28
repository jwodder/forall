use crate::cmd::{CommandError, CommandKind, CommandPlus};
use crate::logging::Verbosity;
use crate::project::Project;
use clap::{ArgAction, Args};
use ghrepo::GHRepo;
use std::ffi::OsString;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Args, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Options {
    /// Don't exit on errors
    #[arg(short, long, global = true)]
    pub(crate) keep_going: bool,

    /// Be less verbose
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub(crate) quiet: u8,

    /// Be more verbose
    #[arg(short, long, global = true)]
    pub(crate) verbose: bool,
}

impl Options {
    pub(crate) fn verbosity(&self) -> Verbosity {
        match i16::from(self.verbose) - i16::from(self.quiet) {
            1.. => Verbosity::Verbose,
            0 => Verbosity::Normal,
            -1 => Verbosity::Quiet,
            _ => Verbosity::Quiet2,
        }
    }
}

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunOpts {
    /// Treat the command as a path to a script file.
    ///
    /// The script file must either be executable or else start with a shebang
    /// line.
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
    pub(crate) fn run(&self, p: &Project) -> Result<(), CommandError> {
        p.runcmd(&self.command)
            .args(self.args.iter())
            .kind(CommandKind::Run)
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
            let mut iter = value.command.into_iter();
            let scriptfile = iter.next().expect("command should be nonempty");
            let cmd;
            let mut args = Vec::new();
            if is_executable(&scriptfile)? {
                cmd = fs_err::canonicalize(scriptfile)
                    .map_err(RunOptsError::Canonicalize)?
                    .into_os_string();
            } else {
                let mut line = String::new();
                {
                    let mut fp = BufReader::new(
                        fs_err::File::open(&scriptfile).map_err(RunOptsError::Open)?,
                    );
                    fp.read_line(&mut line).map_err(RunOptsError::Read)?;
                }
                let Some(line) = line.strip_prefix("#!") else {
                    return Err(RunOptsError::NoShebang);
                };
                let mut bangargs = line.split_whitespace();
                let Some(interpreter) = bangargs.next() else {
                    return Err(RunOptsError::NoShebang);
                };
                cmd = OsString::from(interpreter);
                args.extend(bangargs.map(OsString::from));
                args.push(
                    fs_err::canonicalize(scriptfile)
                        .map_err(RunOptsError::Canonicalize)?
                        .into_os_string(),
                );
            }
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
    #[error("failed to get filesystem metadata for script")]
    Metadata(#[source] std::io::Error),
    #[error("failed to open script for reading")]
    Open(#[source] std::io::Error),
    #[error("failed to read shebang line from script")]
    Read(#[source] std::io::Error),
    #[error("script does not start with shebang")]
    NoShebang,
    #[error("failed to canonicalize script path")]
    Canonicalize(#[source] std::io::Error),
}

pub(crate) fn get_ghrepo(p: &Path) -> anyhow::Result<Option<GHRepo>> {
    // Don't use ghrepo::LocalRepo::github_remote(), as that doesn't suppress
    // stderr from Git.
    match CommandPlus::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(p)
        .kind(CommandKind::Filter)
        .check_output()
    {
        Ok(s) => Ok(GHRepo::from_url(s.trim()).ok()),
        Err(CommandError::Exit { rc, .. }) if rc.code() == Some(2) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn get_shell() -> OsString {
    std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("sh"))
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(p: P) -> Result<bool, RunOptsError> {
    use std::os::unix::fs::MetadataExt;
    let md = fs_err::metadata(p).map_err(RunOptsError::Metadata)?;
    Ok(md.mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable<P: AsRef<Path>>(p: P) -> Result<bool, RunOptsError> {
    // TODO: How do you do this on Windows?
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, false, Verbosity::Normal)]
    #[case(1, false, Verbosity::Quiet)]
    #[case(2, false, Verbosity::Quiet2)]
    #[case(3, false, Verbosity::Quiet2)]
    #[case(0, true, Verbosity::Verbose)]
    #[case(1, true, Verbosity::Normal)]
    #[case(2, true, Verbosity::Quiet)]
    #[case(3, true, Verbosity::Quiet2)]
    fn test_verbosity(#[case] quiet: u8, #[case] verbose: bool, #[case] verbosity: Verbosity) {
        let opts = Options {
            quiet,
            verbose,
            ..Options::default()
        };
        assert_eq!(opts.verbosity(), verbosity);
    }
}
