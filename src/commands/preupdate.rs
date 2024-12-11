use crate::cmd::CommandError;
use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;
use fs_err::PathExt;

static PRE_COMMIT_FILE: &str = ".pre-commit-config.yaml";

/// Update .pre-commit-config.yaml files
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreUpdate {
    /// Don't exit on errors
    #[arg(
        short,
        long,
        default_value_if("show_failures", clap::builder::ArgPredicate::IsPresent, "true")
    )]
    keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long)]
    quiet: bool,

    #[arg(short = 'F', long)]
    show_failures: bool,
}

impl PreUpdate {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            if !p.dirpath().join(PRE_COMMIT_FILE).fs_err_try_exists()? {
                continue;
            }
            printlnbold(p.name());
            p.stash()?;
            if !p
                .runcmd("pre-commit")
                .arg("autoupdate")
                .quiet(self.quiet)
                .keep_going(self.keep_going)
                .run()?
            {
                failures.push(p.name().to_owned());
                continue;
            }
            p.runcmd("git").args(["add", PRE_COMMIT_FILE]).run()?;
            p.runcmd("pre-commit")
                .args(["run", "-a"])
                .keep_going(true)
                .run()?;
            p.runcmd("git").args(["add", "-a"]).run()?;
            // Run pre-commit again in order to check for linting & similar
            // errors without the rewriting of files (which also causes
            // pre-commit to exit nonzero) causing false positives:
            if !p
                .runcmd("pre-commit")
                .args(["run", "-a"])
                .keep_going(self.keep_going)
                .run()?
            {
                failures.push(p.name().to_owned());
                continue;
            }
            match p.runcmd("git").args(["diff", "--cached", "--quiet"]).run() {
                Ok(_) => (),
                Err(CommandError::Exit { .. }) => {
                    p.runcmd("git")
                        .args(["commit", "-m"])
                        .arg(format!("Autoupdate {PRE_COMMIT_FILE}"))
                        .quiet(self.quiet)
                        .keep_going(self.keep_going)
                        .run()?;
                }
                Err(e) => return Err(e.into()),
            }
        }
        if !failures.is_empty() && self.show_failures {
            printlnbold("\nFailures:");
            for name in failures {
                println!("{name}");
            }
        }
        Ok(())
    }
}
