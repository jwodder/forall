use crate::cmd::CommandOutputError;
use crate::logging::logfailures;
use crate::project::{Language, Project};
use crate::util::Options;
use anyhow::Context;
use clap::Args;
use serde::Deserialize;

/// Count effective lines of code in each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Cloc;

impl Cloc {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let mut failures = Vec::new();
        for p in projects {
            let srcs = p.source_paths()?;
            if srcs.is_empty() {
                if opts.keep_going {
                    log::error!("{}: Could not identify source files", p.name());
                    failures.push(p);
                    continue;
                } else {
                    anyhow::bail!("{}: Could not identify source files", p.name());
                }
            }
            let r = p
                .runcmd("cloc")
                .arg(format!("--include-ext={}", p.language().ext()))
                .arg("--json")
                .args(srcs)
                .check_output();
            let output = match r {
                Ok(output) => output,
                Err(CommandOutputError::Exit { .. }) if opts.keep_going => {
                    failures.push(p);
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
            let data = serde_json::from_str::<ClocJson>(&output)
                .context("failed to deserialize `cloc` output")?;
            let lines = data.for_language(p.language()).unwrap_or_default().code;
            println!("{lines:6} {}", p.name());
        }
        logfailures(failures);
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct ClocJson {
    python: Option<Stats>,
    rust: Option<Stats>,
}

impl ClocJson {
    fn for_language(&self, language: Language) -> Option<Stats> {
        match language {
            Language::Python => self.python,
            Language::Rust => self.rust,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
struct Stats {
    code: usize,
}
