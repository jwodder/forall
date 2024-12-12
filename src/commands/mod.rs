mod clean;
mod cloc;
mod gc;
mod list;
mod preupdate;
mod pull;
mod push;
mod run;
use self::clean::Clean;
use self::cloc::Cloc;
use self::gc::Gc;
use self::list::List;
use self::preupdate::PreUpdate;
use self::pull::Pull;
use self::push::Push;
pub(crate) use self::run::Run;
use crate::project::Project;
use crate::util::Options;
use clap::Subcommand;

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    List(List),
    Clean(Clean),
    Cloc(Cloc),
    Gc(Gc),
    PreUpdate(PreUpdate),
    Pull(Pull),
    Push(Push),
    Run(Run),
}

impl Command {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        match self {
            Command::List(c) => c.run(opts, projects),
            Command::Clean(c) => c.run(opts, projects),
            Command::Cloc(c) => c.run(opts, projects),
            Command::Gc(c) => c.run(opts, projects),
            Command::PreUpdate(c) => c.run(opts, projects),
            Command::Pull(c) => c.run(opts, projects),
            Command::Push(c) => c.run(opts, projects),
            Command::Run(c) => c.run(opts, projects),
        }
    }
}
