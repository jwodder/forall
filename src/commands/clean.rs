use super::ForAll;
use crate::logging::logproject;
use crate::project::Project;
use clap::Args;

/// Run `git clean -dXf` on each project
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Clean;

impl ForAll for Clean {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        if !p.readcmd("git", ["clean", "-dXn"])?.is_empty() {
            logproject(p);
            p.runcmd("git").args(["clean", "-dXf"]).run()?;
        } else {
            debug!("{}: already clean", p.name());
        }
        Ok(())
    }
}
