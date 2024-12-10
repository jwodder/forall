use crate::project::Project;
use crate::util::printlnbold;
use clap::Args;
use std::ffi::OsString;

/// Run a command on each project
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Run {
    /// Don't exit on errors
    #[arg(short, long)]
    pub(crate) keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long)]
    pub(crate) quiet: bool,

    #[arg(long)]
    pub(crate) shell: bool,

    #[arg(short = 'F', long)]
    pub(crate) show_failures: bool,

    #[arg(allow_hyphen_values = true, trailing_var_arg = true, required = true)]
    pub(crate) command: Vec<OsString>,
}

impl Run {
    pub(crate) fn run(mut self, projects: Vec<Project>) -> anyhow::Result<()> {
        let (cmd, args) = if self.shell {
            let cmd = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("sh"));
            let mut args = Vec::with_capacity(self.command.len().saturating_add(1));
            args.push(OsString::from("-c"));
            args.extend(self.command);
            (cmd, args)
        } else {
            let mut iter = self.command.into_iter();
            let cmd = iter.next().expect("command should be nonempty");
            (cmd, iter.collect())
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
