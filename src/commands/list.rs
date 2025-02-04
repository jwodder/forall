use super::ForAll;
use crate::project::Project;
use clap::Args;

/// List all projects
#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct List {
    /// Output JSON
    #[arg(short = 'J', long)]
    json: bool,
}

impl ForAll for List {
    fn run(&mut self, p: &Project) -> anyhow::Result<()> {
        if self.json {
            println!(
                "{}",
                serde_json::to_string(&p.to_details()?).expect("JSONification should not fail")
            );
        } else {
            println!("{}", p.name());
        }
        Ok(())
    }
}
