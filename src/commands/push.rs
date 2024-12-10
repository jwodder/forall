use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;

/// Run `git push` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Push {
    /// Don't exit on errors
    #[arg(short, long)]
    keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long)]
    quiet: bool,
}

impl Push {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            let ahead = p.readcmd(
                "git",
                ["rev-list", "--count", "--right-only", "@{upstream}...HEAD"],
            )?;
            if ahead.parse::<usize>().unwrap_or_default() > 0 {
                printlnbold(p.name());
                p.runcmd("git")
                    .arg("push")
                    .quiet(self.quiet)
                    .keep_going(self.keep_going)
                    .run()?;
            }
        }
        Ok(())
    }
}
