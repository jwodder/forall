mod clean;
mod gc;
mod list;
mod preupdate;
mod pull;
mod push;
mod run;
mod script;
use self::clean::Clean;
use self::gc::Gc;
use self::list::List;
use self::preupdate::PreUpdate;
use self::pull::Pull;
use self::push::Push;
pub(crate) use self::run::Run;
use self::script::Script;
use crate::project::Project;
use clap::Subcommand;

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    List(List),
    Clean(Clean),
    Gc(Gc),
    Pull(Pull),
    Push(Push),
    PreUpdate(PreUpdate),
    Script(Script),
    Run(Run),
}

impl Command {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        match self {
            Command::List(c) => c.run(projects),
            Command::Clean(c) => c.run(projects),
            Command::Gc(c) => c.run(projects),
            Command::Pull(c) => c.run(projects),
            Command::Push(c) => c.run(projects),
            Command::PreUpdate(c) => c.run(projects),
            Command::Script(c) => c.run(projects),
            Command::Run(c) => c.run(projects),
        }
    }
}
