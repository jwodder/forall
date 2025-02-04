use crate::cmd::CommandPlus;
use crate::project::Project;
use anstyle::{AnsiColor, Style};
use std::sync::OnceLock;

static VERBOSITY: OnceLock<Verbosity> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Verbosity {
    On,
    Quiet2,
    Quiet,
    #[default]
    Normal,
    Verbose,
    Off,
}

pub(crate) fn init_logging(level: Verbosity) {
    let _ = VERBOSITY.set(level);
}

pub(crate) fn is_active(level: Verbosity) -> bool {
    level <= VERBOSITY.get().copied().unwrap_or_default()
}

pub(crate) fn logproject(p: &Project) {
    anstream::println!(
        "{bold}{name}{bold:#}",
        name = p.name(),
        bold = Style::new().bold()
    );
}

pub(crate) fn logcmd(cmd: &CommandPlus, level: Verbosity) {
    if is_active(level) {
        anstream::eprintln!(
            "{style}+{line}{style:#}",
            line = cmd.cmdline(),
            style = Style::new().fg_color(Some(AnsiColor::Cyan.into()))
        );
    }
}

pub(crate) fn logrequest(method: &str, url: &url::Url) {
    if is_active(Verbosity::Verbose) {
        anstream::eprintln!(
            // TODO: Add a prefix?
            "{style}{method} {url}{style:#}",
            style = Style::new().fg_color(Some(AnsiColor::Cyan.into()))
        );
    }
}

pub(crate) fn logfailures(failures: Vec<Project>) {
    if !failures.is_empty() {
        anstream::println!("\n{bold}Failures:{bold:#}", bold = Style::new().bold());
        for p in failures {
            println!("{}", p.name());
        }
    }
}

macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::logging::logln(
            $crate::logging::Verbosity::On,
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Red.into())),
            format_args!($($arg)*)
        );
    }};
}

macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::logging::logln(
            $crate::logging::Verbosity::Normal,
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into())),
            format_args!($($arg)*)
        );
    }};
}

macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::logging::logln(
            $crate::logging::Verbosity::Verbose,
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into())),
            format_args!($($arg)*)
        );
    }};
}

pub(crate) fn logln(level: Verbosity, style: Style, fmtargs: std::fmt::Arguments<'_>) {
    if is_active(level) {
        anstream::eprintln!("{style}[·] {fmtargs}{style:#}");
    }
}
