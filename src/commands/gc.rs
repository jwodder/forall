use super::ForAll;
use crate::logging::logproject;
use crate::project::Project;
use clap::Args;

/// Run `git gc` on each project
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Gc;

impl ForAll for Gc {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        logproject(p);
        p.runcmd("git").arg("gc").run()?;
        Ok(())
    }
}
