use crate::logging::{logfailures, logproject};
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

    /// Stash any uncommitted changes before running the command
    #[arg(short, long)]
    pub(crate) stash: bool,
}

impl Run {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let runner = Runner::try_from(self.run_opts)?;
        let mut failures = Vec::new();
        for p in projects {
            logproject(&p);
            if self.stash {
                p.stash(opts.quiet())?;
            }
            if !runner.run(&p, opts)? {
                failures.push(p);
            }
        }
        logfailures(failures);
        Ok(())
    }
}
