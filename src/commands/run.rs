use crate::project::Project;
use crate::util::{printlnbold, Options};
use clap::Args;
use std::ffi::OsString;

/// Run a command on each project.
///
/// The command is run with the current working directory set to each
/// respective project's directory.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Run {
    /// Run command in a shell
    #[arg(long)]
    pub(crate) shell: bool,

    #[arg(allow_hyphen_values = true, trailing_var_arg = true, required = true)]
    pub(crate) command: Vec<OsString>,
}

impl Run {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
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
        let mut failures = Vec::new();
        for p in projects {
            printlnbold(p.name());
            if !p
                .runcmd(&cmd)
                .args(args.iter())
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
