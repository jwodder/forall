use crate::logging::{logfailures, logproject};
use crate::project::Project;
use crate::util::Options;
use clap::Args;

/// Run `git push` on each project
///
/// Only projects that have GitHub remotes and for which `HEAD` is ahead of
/// `@{upstream}` are considered.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Push;

impl Push {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            if !p.has_github() {
                continue;
            }
            // TODO: If this fails, emit "{BOLD:name}\n{ERROR:[1]}" and handle
            // with keep_going:
            let ahead = p.readcmd(
                "git",
                ["rev-list", "--count", "--right-only", "@{upstream}...HEAD"],
            )?;
            if ahead.parse::<usize>().unwrap_or_default() > 0 {
                logproject(&p);
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
        logfailures(failures);
        Ok(())
    }
}
