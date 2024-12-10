mod list;
use self::list::List;
use crate::project::Project;
use clap::Subcommand;

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    List(List),
}

impl Command {
    #[expect(clippy::unnecessary_wraps)]
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        match self {
            Command::List(lst) => lst.run(projects),
        }
        Ok(())
    }
}
