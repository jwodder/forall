use crate::project::Project;
use crate::util::{printlnbold, Options};
use clap::Args;

/// Run `git clean -dXf` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Clean;

impl Clean {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            if !p.readcmd("git", ["clean", "-dXn"])?.is_empty() {
                printlnbold(p.name());
                p.runcmd("git")
                    .args(["clean", "-dXf"])
                    .quiet(opts.quiet)
                    .run()?;
            }
        }
        Ok(())
    }
}
