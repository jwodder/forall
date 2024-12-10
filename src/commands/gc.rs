use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;

/// Run `git gc` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Gc {
    /// Suppress successful command output
    #[arg(short, long)]
    quiet: bool,
}

impl Gc {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            printlnbold(p.name());
            p.runcmd("git").arg("gc").quiet(self.quiet).run()?;
        }
        Ok(())
    }
}
