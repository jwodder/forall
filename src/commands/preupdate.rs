use super::ForAll;
use crate::cmd::CommandError;
use crate::logging::logproject;
use crate::project::Project;
use clap::Args;
use fs_err::PathExt;

static PRE_COMMIT_FILE: &str = ".pre-commit-config.yaml";

/// Update .pre-commit-config.yaml files
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreUpdate;

impl ForAll for PreUpdate {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        if !p.dirpath().join(PRE_COMMIT_FILE).fs_err_try_exists()? {
            debug!("{} does not use pre-commit; skipping", p.name());
            return Ok(());
        }
        logproject(p);
        p.stash()?;
        p.runcmd("pre-commit").arg("autoupdate").run()?;
        p.runcmd("git").args(["add", PRE_COMMIT_FILE]).run()?;
        match p.runcmd("pre-commit").args(["run", "-a"]).run() {
            Ok(()) | Err(CommandError::Exit { .. }) => (),
            Err(e) => return Err(e.into()),
        }
        p.runcmd("git").args(["add", "-a"]).run()?;
        // Run pre-commit again in order to check for linting & similar
        // errors without the rewriting of files (which also causes
        // pre-commit to exit nonzero) causing false positives:
        p.runcmd("pre-commit").args(["run", "-a"]).run()?;
        match p.runcmd("git").args(["diff", "--cached", "--quiet"]).run() {
            Ok(()) => (),
            Err(CommandError::Exit { .. }) => {
                p.runcmd("git")
                    .args(["commit", "-m"])
                    .arg(format!("Autoupdate {PRE_COMMIT_FILE}"))
                    .run()?;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}
