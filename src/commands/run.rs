use super::ForAll;
use crate::logging::logproject;
use crate::project::Project;
use crate::util::{RunOpts, Runner};
use clap::Args;

/// Run a command on each project.
///
/// The command is run with the current working directory set to each
/// respective project's directory.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Run {
    #[command(flatten)]
    pub(crate) run_opts: RunOpts,

    /// Stash any uncommitted changes before running the command
    #[arg(short, long)]
    pub(crate) stash: bool,
}

impl Run {
    pub(super) fn into_forall(self) -> anyhow::Result<Box<dyn ForAll>> {
        let runner = Runner::try_from(self.run_opts)?;
        Ok(Box::new(RunForAll {
            runner,
            stash: self.stash,
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RunForAll {
    runner: Runner,
    stash: bool,
}

impl ForAll for RunForAll {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        logproject(p);
        if self.stash {
            p.stash()?;
        }
        self.runner.run(p)?;
        Ok(())
    }
}
