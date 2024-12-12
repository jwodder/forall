use crate::project::Project;
use crate::util::{printlnbold, Options};
use clap::Args;

/// Run `git pull` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Pull;

impl Pull {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            printlnbold(p.name());
            if !p
                .runcmd("git")
                .arg("pull")
                .quiet(opts.quiet)
                .keep_going(opts.keep_going)
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
