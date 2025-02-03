use crate::cmd::CommandPlus;
use crate::project::Project;
use anstream::AutoStream;
use anstyle::{AnsiColor, Style};
use log::{Level, LevelFilter};
use std::io;

static PROJECT_TARGET: &str = "forall::class::project";
static COMMAND_TARGET: &str = "forall::class::command";
static REQUEST_TARGET: &str = "forall::class::request";

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Verbosity {
    Quiet2,
    Quiet,
    #[default]
    Normal,
    Verbose,
}

impl Verbosity {
    fn level_filter(&self) -> LevelFilter {
        match self {
            Verbosity::Quiet2 => LevelFilter::Info,
            Verbosity::Quiet => LevelFilter::Info,
            Verbosity::Normal => LevelFilter::Info,
            Verbosity::Verbose => LevelFilter::Debug,
        }
    }

    fn command_filter(&self) -> LevelFilter {
        match self {
            Verbosity::Quiet2 => LevelFilter::Info,
            Verbosity::Quiet => LevelFilter::Info,
            Verbosity::Normal => LevelFilter::Debug,
            Verbosity::Verbose => LevelFilter::Trace,
        }
    }

    fn request_filter(&self) -> LevelFilter {
        match self {
            Verbosity::Quiet2 => LevelFilter::Info,
            Verbosity::Quiet => LevelFilter::Info,
            Verbosity::Normal => LevelFilter::Debug,
            Verbosity::Verbose => LevelFilter::Trace,
        }
    }
}

pub(crate) fn init_logging(verbosity: Verbosity) {
    let outstream: Box<dyn io::Write + Send> = Box::new(AutoStream::auto(io::stdout()));
    fern::Dispatch::new()
        .format(|out, message, record| {
            use AnsiColor::*;
            let style = match (record.level(), record.target()) {
                (_, t) if t == COMMAND_TARGET => Style::new().fg_color(Some(Cyan.into())),
                (Level::Error, _) => Style::new().fg_color(Some(Red.into())),
                (Level::Warn, _) => Style::new().fg_color(Some(Yellow.into())),
                (Level::Info, t) if t == PROJECT_TARGET => Style::new().bold(),
                (Level::Info, _) => Style::new().fg_color(Some(Blue.into())),
                (Level::Debug, _) => Style::new().fg_color(Some(Cyan.into())),
                (Level::Trace, _) => Style::new().fg_color(Some(Green.into())),
            };
            out.finish(format_args!("{style}{message}{style:#}"));
        })
        .level(LevelFilter::Info)
        .level_for("forall", verbosity.level_filter())
        .level_for(COMMAND_TARGET, verbosity.command_filter())
        .level_for(REQUEST_TARGET, verbosity.request_filter())
        .chain(outstream)
        .apply()
        .expect("no other logger should have been previously initialized");
}

pub(crate) fn logproject(p: &Project) {
    log::info!(target: PROJECT_TARGET, "{}", p.name());
}

pub(crate) fn logcmd(cmd: &CommandPlus, trace: bool) {
    if trace {
        log::trace!(target: COMMAND_TARGET, "+{}", cmd.cmdline());
    } else {
        log::debug!(target: COMMAND_TARGET, "+{}", cmd.cmdline());
    }
}

pub(crate) fn logrequest(method: &str, url: &url::Url) {
    log::debug!(target: REQUEST_TARGET, "{method} {url}");
}

pub(crate) fn logfailures(failures: Vec<Project>) {
    if !failures.is_empty() {
        anstream::println!("\n{bold}Failures:{bold:#}", bold = Style::new().bold());
        for p in failures {
            println!("{}", p.name());
        }
    }
}
