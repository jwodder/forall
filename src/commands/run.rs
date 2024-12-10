use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;
use std::ffi::OsString;

/// Run a command on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Run {
    /// Don't exit on errors
    #[arg(short, long)]
    keep_going: bool,

    /// Suppress command output
    #[arg(short, long)]
    quiet: bool,

    #[arg(long)]
    shell: bool,

    #[arg(short = 'F', long)]
    show_failures: bool,

    command: OsString,

    #[arg(allow_hyphen_values = true)]
    args: Vec<OsString>,
}

impl Run {
    pub(crate) fn run(mut self, projects: Vec<Project>) -> anyhow::Result<()> {
        let (cmd, args) = if self.shell {
            let cmd = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("sh"));
            let mut args = Vec::with_capacity(self.args.len().saturating_add(2));
            args.push(OsString::from("-c"));
            args.push(self.command);
            args.extend(self.args);
            (cmd, args)
        } else {
            (self.command, self.args)
        };
        if self.show_failures {
            self.keep_going = true;
        }
        let mut failures = Vec::new();
        for p in projects {
            printlnbold(p.name());
            if !p
                .runcmd(&cmd)
                .args(args.iter())
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
