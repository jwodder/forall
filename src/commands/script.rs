use crate::project::Project;
use crate::util::{printlnbold, Options};
use clap::Args;
use std::path::PathBuf;

/// Run a script on each project.
///
/// The command is run with the current working directory set to each
/// respective project's directory.
///
/// The script is run via `perl` for its shebang-handling, so the script need
/// not be executable, but it does need to have an appropriate shebang.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Script {
    pub(crate) scriptfile: PathBuf,
}

impl Script {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let scriptfile = fs_err::canonicalize(self.scriptfile)?;
        let mut failures = Vec::new();
        for p in projects {
            printlnbold(p.name());
            // Use perl to interpret the script's shebang, thereby supporting
            // non-executable scripts
            if !p
                .runcmd("perl")
                .arg(&scriptfile)
                .quiet(opts.quiet)
                .keep_going(opts.keep_going)
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
