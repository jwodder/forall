use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;
use std::path::PathBuf;

/// Run a script on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Script {
    /// Don't exit on errors
    #[arg(short, long)]
    keep_going: bool,

    /// Suppress command output
    #[arg(short, long)]
    quiet: bool,

    #[arg(short = 'F', long)]
    show_failures: bool,

    scriptfile: PathBuf,
}

impl Script {
    pub(crate) fn run(mut self, projects: Vec<Project>) -> anyhow::Result<()> {
        let scriptfile = fs_err::canonicalize(self.scriptfile)?;
        if self.show_failures {
            self.keep_going = true;
        }
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
                failures.push(p.name().to_owned());
            }
        }
        if !failures.is_empty() && self.show_failures {
            printlnbold("\nFailures:");
            for name in failures {
                println!("{name}");
            }
        }
        Ok(())
    }
}
