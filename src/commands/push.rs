use crate::project::Project;
use crate::util::{printlnbold, Options};
use clap::Args;

/// Run `git push` on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Push;

impl Push {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            // TODO: If this fails, emit "{BOLD:name}\n{ERROR:[1]}" and handle
            // with keep_going:
            let ahead = p.readcmd(
                "git",
                ["rev-list", "--count", "--right-only", "@{upstream}...HEAD"],
            )?;
            if ahead.parse::<usize>().unwrap_or_default() > 0 {
                printlnbold(p.name());
                if !p
                    .runcmd("git")
                    .arg("push")
                    .quiet(opts.quiet)
                    .keep_going(opts.keep_going)
                    .run()?
                {
                    failures.push(p);
                }
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
