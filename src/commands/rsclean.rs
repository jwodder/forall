use crate::project::{Language, Project};
use crate::util::Options;
use clap::Args;
use fs_err::PathExt;

/// Run `cargo clean` on Rust projects with `target/` directories
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Rsclean;

impl Rsclean {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            if p.language() != Language::Rust || !p.dirpath().join("target").fs_err_try_exists()? {
                continue;
            }
            boldln!("{}", p.name());
            if !p
                .runcmd("cargo")
                .arg("clean")
                .quiet(opts.quiet)
                .keep_going(opts.keep_going)
                .run()?
            {
                failures.push(p);
            }
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
