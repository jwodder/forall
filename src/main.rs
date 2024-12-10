mod cmd;
mod commands;
mod finder;
mod project;
mod util;
use crate::commands::Command;
use crate::finder::Finder;
use clap::Parser;

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
#[command(version = env!("VERSION_WITH_GIT"))]
struct Arguments {
    #[command(flatten)]
    finder: Finder,

    #[command(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let Arguments { finder, command } = Arguments::parse();
    let projects = finder.findall()?;
    command.run(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Run;
    use std::ffi::OsString;

    #[test]
    fn test_run_known_opt() {
        let args = Arguments::try_parse_from(["arg0", "run", "cmd", "-q"]).unwrap();
        assert_eq!(
            args.command,
            Command::Run(Run {
                quiet: false,
                keep_going: false,
                shell: false,
                show_failures: false,
                command: vec![OsString::from("cmd"), OsString::from("-q")],
            })
        );
    }

    #[test]
    fn test_run_unknown_opt() {
        let args = Arguments::try_parse_from(["arg0", "run", "cmd", "-x"]).unwrap();
        assert_eq!(
            args.command,
            Command::Run(Run {
                quiet: false,
                keep_going: false,
                shell: false,
                show_failures: false,
                command: vec![OsString::from("cmd"), OsString::from("-x")],
            })
        );
    }
}
