mod clean;
mod cloc;
mod gc;
mod list;
mod preupdate;
mod pull;
mod push;
mod rsclean;
mod run;
mod runpr;
use self::clean::Clean;
use self::cloc::Cloc;
use self::gc::Gc;
use self::list::List;
use self::preupdate::PreUpdate;
use self::pull::Pull;
use self::push::Push;
use self::rsclean::Rsclean;
pub(crate) use self::run::Run;
use self::runpr::RunPr;
use crate::logging::logerror;
use crate::project::Project;
use crate::util::Options;
use clap::Subcommand;
use std::process::ExitCode;

trait ForAll {
    fn run(&mut self, p: &Project) -> anyhow::Result<()>;
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum Command {
    List(List),
    Clean(Clean),
    Cloc(Cloc),
    Gc(Gc),
    PreUpdate(PreUpdate),
    Pull(Pull),
    Push(Push),
    Rsclean(Rsclean),
    Run(Run),
    RunPr(RunPr),
}

impl Command {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> ExitCode {
        let mut cmd: Box<dyn ForAll> = match self {
            Command::List(c) => Box::new(c),
            Command::Clean(c) => Box::new(c),
            Command::Cloc(c) => Box::new(c),
            Command::Gc(c) => Box::new(c),
            Command::PreUpdate(c) => Box::new(c),
            Command::Pull(c) => Box::new(c),
            Command::Push(c) => Box::new(c),
            Command::Rsclean(c) => Box::new(c),
            Command::Run(c) => match c.into_forall() {
                Ok(cmd) => cmd,
                Err(e) => {
                    logerror(e.context("Failed to initialize command"));
                    return ExitCode::FAILURE;
                }
            },
            Command::RunPr(c) => match c.into_forall() {
                Ok(cmd) => cmd,
                Err(e) => {
                    logerror(e.context("Failed to initialize command"));
                    return ExitCode::FAILURE;
                }
            },
        };
        let mut failures = Vec::new();
        for p in projects {
            if let Err(e) = cmd.run(&p) {
                logerror(e);
                if opts.keep_going {
                    failures.push(p);
                } else {
                    return ExitCode::FAILURE;
                }
            }
        }
        if failures.is_empty() {
            ExitCode::SUCCESS
        } else {
            anstream::println!(
                "\n{bold}Failures:{bold:#}",
                bold = anstyle::Style::new().bold()
            );
            for p in failures {
                println!("{}", p.name());
            }
            ExitCode::FAILURE
        }
    }
}
