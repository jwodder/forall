mod cmd;
mod commands;
mod finder;
mod project;
mod util;
use crate::commands::Command;
use crate::finder::Finder;
use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
struct Arguments {
    #[arg(long)]
    root: Option<PathBuf>,

    #[command(flatten)]
    finder: Finder,

    #[command(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let Arguments {
        root,
        finder,
        command,
    } = Arguments::parse();
    let root = match root {
        Some(p) => p,
        None => std::env::current_dir().context("failed to determine current directory")?,
    };
    let projects = finder.findall(root)?;
    command.run(projects)
}
