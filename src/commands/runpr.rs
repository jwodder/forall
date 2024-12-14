use crate::project::Project;
use crate::util::{Options, RunOpts, Runner};
use clap::Args;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

static DEFAULT_BRANCH_FORMAT: &[FormatItem<'_>] =
    format_description!("forall-runpr-[year][month][day][hour][minute][second]");

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunPr {
    #[arg(short, long)]
    branch: Option<String>,

    #[command(flatten)]
    pub(crate) run_opts: RunOpts,
}

impl RunPr {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let branch = match self.branch {
            Some(b) => b,
            None => OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc())
                .format(&DEFAULT_BRANCH_FORMAT)
                .expect("formatting a datetime should not fail"),
        };
        let runner = Runner::try_from(self.run_opts)?;
        let mut failures = Vec::new();
        for p in projects {
            let Some(ghrepo) = p.ghrepo() else {
                // TODO: Log a message
                continue;
            };
            boldln!("{}", p.name());
            let defbranch = p.default_branch()?;
            p.stash()?;
            p.runcmd("git")
                .arg("checkout")
                .arg("-b")
                .arg(&branch)
                .arg(defbranch)
                .quiet(opts.quiet)
                .run()?;
            if !runner.run(&p, opts)? {
                failures.push(p);
                continue;
            }
            p.runcmd("git")
                .args(["push", "--set-upstream", "origin"])
                .arg(&branch)
                .quiet(opts.quiet)
                .run()?;
            todo!("Create PR");
        }
        if !failures.is_empty() {
            boldln!("\nFailures:");
            for p in failures {
                println!("{}", p.name());
            }
        }
        Ok(())
    }
}
