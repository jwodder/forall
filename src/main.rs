mod cmd;
mod commands;
mod finder;
mod project;
mod util;
use crate::commands::Command;
use crate::finder::Finder;
use clap::Parser;

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    #[command(flatten)]
    finder: Finder,

    #[command(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let Arguments { finder, command } = Arguments::parse();
    let projects = finder.findall(".".into())?;
    command.run(projects)
}
