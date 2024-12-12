use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;

/// Run `git pull` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Pull {
    /// Don't exit on errors
    #[arg(short, long)]
    keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long)]
    quiet: bool,
}

impl Pull {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            printlnbold(p.name());
            if !p
                .runcmd("git")
                .arg("pull")
                .quiet(self.quiet)
                .keep_going(self.keep_going)
                .run()?
            {
                failures.push(p);
            }
        }
        if !failures.is_empty() {
            printlnbold("\nFailures:");
            for p in failures {
                println!("{}", p.name());
            }
        }
        Ok(())
    }
}
