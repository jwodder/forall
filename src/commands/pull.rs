use crate::logging::{logfailures, logproject};
use crate::project::Project;
use crate::util::Options;
use clap::Args;

/// Run `git pull` on each project
///
/// Only projects that have GitHub remotes are considered.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Pull;

impl Pull {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            if !p.has_github() {
                continue;
            }
            logproject(&p);
            if !p
                .runcmd("git")
                .arg("pull")
                .keep_going(opts.keep_going)
                .run()?
            {
                failures.push(p);
            }
        }
        logfailures(failures);
        Ok(())
    }
}
