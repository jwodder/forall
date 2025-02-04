use super::ForAll;
use crate::logging::logproject;
use crate::project::Project;
use clap::Args;

/// Run `git pull` on each project
///
/// Only projects that have GitHub remotes are considered.
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Pull;

impl ForAll for Pull {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        if !p.has_github() {
            debug!("{} does not have a GitHub repository; skipping", p.name());
        } else {
            logproject(p);
            p.runcmd("git").arg("pull").run()?;
        }
        Ok(())
    }
}
