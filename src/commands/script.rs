use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;
use std::path::PathBuf;

/// Run a script on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Script {
    /// Don't exit on errors
    #[arg(short, long)]
    pub(crate) keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long)]
    pub(crate) quiet: bool,

    pub(crate) scriptfile: PathBuf,
}

impl Script {
    pub(crate) fn run(self, projects: Vec<Project>) -> anyhow::Result<()> {
        let scriptfile = fs_err::canonicalize(self.scriptfile)?;
        let mut failures = Vec::new();
        for p in projects {
            printlnbold(p.name());
            // Use perl to interpret the script's shebang, thereby supporting
            // non-executable scripts
            if !p
                .runcmd("perl")
                .arg(&scriptfile)
                .quiet(self.quiet)
                .keep_going(self.keep_going)
                .run()?
            {
                failures.push(p);
            }
        }
        if !failures.is_empty() {
            printlnbold("\nFailures:");
            for p in failures {
                println!("{}", p.name());
            }
        }
        Ok(())
    }
}
