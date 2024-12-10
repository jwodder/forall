mod clean;
mod gc;
mod list;
use self::clean::Clean;
use self::gc::Gc;
use self::list::List;
use crate::project::Project;
use clap::Subcommand;

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    List(List),
    Clean(Clean),
    Gc(Gc),
}

impl Command {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        match self {
            Command::List(c) => c.run(projects),
            Command::Clean(c) => c.run(projects),
            Command::Gc(c) => c.run(projects),
        }
    }
}
