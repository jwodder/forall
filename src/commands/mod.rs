mod list;
use self::list::List;
use crate::finder::Finder;
use clap::Subcommand;

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    List(List),
}

impl Command {
    pub(crate) fn run(self, finder: Finder) -> anyhow::Result<()> {
        match self {
            Command::List(lst) => lst.run(finder),
        }
    }
}
