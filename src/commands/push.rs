use super::ForAll;
use crate::logging::logproject;
use crate::project::Project;
use clap::Args;

/// Run `git push` on each project
///
/// Only projects that have GitHub remotes and for which `HEAD` is ahead of
/// `@{upstream}` are considered.
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Push;

impl ForAll for Push {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        if !p.has_github() {
            debug!("{} does not have a GitHub repository; skipping", p.name());
        } else {
            let ahead = p.readcmd(
                "git",
                ["rev-list", "--count", "--right-only", "@{upstream}...HEAD"],
            )?;
            if ahead.parse::<usize>().unwrap_or_default() > 0 {
                logproject(p);
                p.runcmd("git").arg("push").run()?;
            }
        }
        Ok(())
    }
}
