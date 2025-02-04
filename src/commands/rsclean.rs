use super::ForAll;
use crate::logging::logproject;
use crate::project::{Language, Project};
use clap::Args;
use fs_err::PathExt;

/// Run `cargo clean` on Rust projects with `target/` directories
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Rsclean;

impl ForAll for Rsclean {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        if p.language() != Language::Rust || !p.dirpath().join("target").fs_err_try_exists()? {
            debug!(
                "{} is not a Rust project with a target/ directory; skipping",
                p.name()
            );
        } else {
            logproject(p);
            p.runcmd("cargo").arg("clean").run()?;
        }
        Ok(())
    }
}
