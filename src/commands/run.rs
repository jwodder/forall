use crate::project::Project;
use crate::util::{Options, RunOpts, Runner};
use clap::Args;

/// Run a command on each project.
///
/// The command is run with the current working directory set to each
/// respective project's directory.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Run {
    #[command(flatten)]
    pub(crate) run_opts: RunOpts,
}

impl Run {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let runner = Runner::try_from(self.run_opts)?;
        let mut failures = Vec::new();
        for p in projects {
            boldln!("{}", p.name());
            if !runner.run(&p, opts)? {
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
