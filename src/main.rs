#[macro_use]
mod logging;

mod cmd;
mod commands;
mod finder;
mod github;
mod http_util;
mod project;
mod util;
use crate::commands::Command;
use crate::finder::Finder;
use crate::logging::{init_logging, logerror};
use crate::util::Options;
use clap::Parser;
use std::process::ExitCode;

/// Operate on each project in a directory
#[derive(Clone, Debug, Eq, Parser, PartialEq)]
#[command(version = env!("VERSION_WITH_GIT"))]
struct Arguments {
    #[command(flatten)]
    opts: Options,

    #[command(flatten)]
    finder: Finder,

    #[command(subcommand)]
    command: Command,
}

fn main() -> ExitCode {
    let Arguments {
        opts,
        finder,
        command,
    } = Arguments::parse();
    init_logging(opts.verbosity());
    let projects = match finder.findall() {
        Ok(projects) => projects,
        Err(e) => {
            logerror(e.context("Failed to list projects"));
            return ExitCode::FAILURE;
        }
    };
    command.run(opts, projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Run;
    use crate::util::RunOpts;
    use std::ffi::OsString;

    #[test]
    fn test_run_known_opt() {
        let args = Arguments::try_parse_from(["arg0", "run", "cmd", "--shell"]).unwrap();
        assert_eq!(
            args.command,
            Command::Run(Run {
                run_opts: RunOpts {
                    script: false,
                    shell: false,
                    command: vec![OsString::from("cmd"), OsString::from("--shell")],
                },
                stash: false
            })
        );
    }

    #[test]
    fn test_run_unknown_opt() {
        let args = Arguments::try_parse_from(["arg0", "run", "cmd", "-x"]).unwrap();
        assert_eq!(
            args.command,
            Command::Run(Run {
                run_opts: RunOpts {
                    script: false,
                    shell: false,
                    command: vec![OsString::from("cmd"), OsString::from("-x")],
                },
                stash: false
            })
        );
    }

    #[test]
    fn test_run_script_shell() {
        let r = Arguments::try_parse_from(["arg0", "run", "--script", "--shell", "foo.sh"]);
        assert!(r.is_err());
    }
}
