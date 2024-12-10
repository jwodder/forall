use crate::project::Project;
use crate::util::printbold;
use clap::Args;

/// Run `git clean -dX` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Clean {
    /// Suppress command output
    #[arg(short, long)]
    quiet: bool,
}

impl Clean {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            if !p.readcmd("git", ["clean", "-dXn"])?.is_empty() {
                printbold(p.name());
                p.runcmd("git")
                    .args(["clean", "-dX"])
                    .quiet(self.quiet)
                    .run()?;
            }
        }
        Ok(())
    }
}
