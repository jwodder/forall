use crate::cmd::CommandOutputError;
use crate::project::{Language, Project};
use crate::util::printlnerror;
use anyhow::Context;
use clap::Args;
use serde::Deserialize;

/// Count effective lines of code in each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Cloc {
    /// Don't exit on errors
    #[arg(short, long)]
    keep_going: bool,
}

impl Cloc {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        for p in projects {
            let srcs = p.source_paths()?;
            if srcs.is_empty() {
                printlnerror(&format!("{}: Could not identify source files", p.name()));
                if self.keep_going {
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
                Err(CommandOutputError::Exit { .. }) if self.keep_going => continue,
                Err(e) => return Err(e.into()),
            };
            let data = serde_json::from_str::<ClocJson>(&output)
                .context("failed to deserialize `cloc` output")?;
            let lines = data.for_language(p.language()).unwrap_or_default().code;
            println!("{lines:6} {}", p.name());
        }
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
