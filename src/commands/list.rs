use crate::project::Project;
use crate::util::Options;
use clap::Args;
use serde_jsonlines::JsonLinesWriter;

/// List all projects
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct List {
    /// Output JSON
    #[arg(short = 'J', long)]
    json: bool,
}

impl List {
    pub(crate) fn run(self, _opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        if self.json {
            let mut out = JsonLinesWriter::new(std::io::stdout());
            for p in projects {
                out.write(&p.to_details()?)?;
            }
        } else {
            for p in projects {
                println!("{}", p.name());
            }
        }
        Ok(())
    }
}
