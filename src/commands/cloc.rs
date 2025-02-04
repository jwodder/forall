use super::ForAll;
use crate::cmd::CommandKind;
use crate::project::{Language, Project};
use anyhow::Context;
use clap::Args;
use serde::Deserialize;

/// Count effective lines of code in each project
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Cloc;

impl ForAll for Cloc {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        let srcs = p.source_paths()?;
        if srcs.is_empty() {
            anyhow::bail!("{}: Could not identify source files", p.name());
        }
        let output = p
            .runcmd("cloc")
            .arg(format!("--include-ext={}", p.language().ext()))
            .arg("--json")
            .args(srcs)
            .kind(CommandKind::Filter) // Don't fill up output with command logs
            .check_output()?;
        let data = serde_json::from_str::<ClocJson>(&output)
            .context("failed to deserialize `cloc` output")?;
        let lines = data.for_language(p.language()).unwrap_or_default().code;
        println!("{lines:6} {}", p.name());
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
