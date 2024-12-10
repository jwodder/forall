use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;

/// Run `git clean -dXf` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Clean {
    /// Suppress successful command output
    #[arg(short, long)]
    quiet: bool,
}

impl Clean {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            if !p.readcmd("git", ["clean", "-dXn"])?.is_empty() {
                printlnbold(p.name());
                p.runcmd("git")
                    .args(["clean", "-dXf"])
                    .quiet(self.quiet)
                    .run()?;
            }
        }
        Ok(())
    }
}
