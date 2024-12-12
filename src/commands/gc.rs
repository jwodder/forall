use crate::project::Project;
use crate::util::{printlnbold, Options};
use clap::Args;

/// Run `git gc` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Gc;

impl Gc {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            printlnbold(p.name());
            p.runcmd("git").arg("gc").quiet(opts.quiet).run()?;
        }
        Ok(())
    }
}
