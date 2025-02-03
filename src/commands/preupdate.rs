use crate::cmd::CommandError;
use crate::logging::{logfailures, logproject};
use crate::project::Project;
use crate::util::Options;
use clap::Args;
use fs_err::PathExt;

static PRE_COMMIT_FILE: &str = ".pre-commit-config.yaml";

/// Update .pre-commit-config.yaml files
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreUpdate;

impl PreUpdate {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            if !p.dirpath().join(PRE_COMMIT_FILE).fs_err_try_exists()? {
                continue;
            }
            logproject(&p);
            p.stash(opts.quiet())?;
            if !p
                .runcmd("pre-commit")
                .arg("autoupdate")
                .quiet(opts.quiet())
                .keep_going(opts.keep_going)
                .run()?
            {
                failures.push(p);
                continue;
            }
            p.runcmd("git").args(["add", PRE_COMMIT_FILE]).run()?;
            // TODO: Suppress the "[{returncode}]" output when this fails:
            // TODO: Shouldn't this honor --quiet?
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
                .keep_going(opts.keep_going)
                .run()?
            {
                failures.push(p);
                continue;
            }
            match p.runcmd("git").args(["diff", "--cached", "--quiet"]).run() {
                Ok(_) => (),
                Err(CommandError::Exit { .. }) => {
                    p.runcmd("git")
                        .args(["commit", "-m"])
                        .arg(format!("Autoupdate {PRE_COMMIT_FILE}"))
                        .quiet(opts.quiet())
                        .keep_going(opts.keep_going)
                        .run()?;
                }
                Err(e) => return Err(e.into()),
            }
        }
        logfailures(failures);
        Ok(())
    }
}
