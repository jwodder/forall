use crate::cmd::{CommandError, CommandPlus};
use crate::project::Project;
use anstyle::{AnsiColor, Style};
use indenter::indented;
use log::{Log, Metadata, Record};
use std::fmt::{self, Write};
use std::sync::OnceLock;

static VERBOSITY: OnceLock<Verbosity> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Verbosity {
    //On,
    Quiet2,
    Quiet,
    #[default]
    Normal,
    Verbose,
    Off,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        let top = match metadata.target().split_once("::") {
            Some((pre, _)) => pre,
            None => metadata.target(),
        };
        top == "minigh"
    }

    fn log(&self, record: &Record<'_>) {
        anstream::eprintln!(
            // TODO: Add a prefix?
            "{style}{msg}{style:#}",
            style = Style::new().fg_color(Some(AnsiColor::Cyan.into())),
            msg = record.args(),
        );
    }

    fn flush(&self) {}
}

pub(crate) fn init_logging(level: Verbosity) {
    let _ = VERBOSITY.set(level);
    log::set_logger(&Logger).expect("no other logger should have been previously initialized");
    log::set_max_level(if level >= Verbosity::Verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Off
    });
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

#[clippy::format_args]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::logging::logln(
            $crate::logging::Verbosity::Normal,
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into())),
            format_args!($($arg)*)
        );
    }};
}

#[clippy::format_args]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::logging::logln(
            $crate::logging::Verbosity::Verbose,
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into())),
            format_args!($($arg)*)
        );
    }};
}

pub(crate) fn logln(level: Verbosity, style: Style, fmtargs: fmt::Arguments<'_>) {
    if is_active(level) {
        anstream::eprintln!("{style}[·] {fmtargs}{style:#}");
    }
}

pub(crate) fn logerror(e: anyhow::Error) {
    let style = Style::new().fg_color(Some(AnsiColor::BrightRed.into()));
    anstream::eprintln!("{style}[!] {e}{style:#}");
    for src in e.chain().skip(1) {
        anstream::eprintln!("{style}[!] ⤷ {src}{style:#}");
    }
    if let Some(src) = e.downcast_ref::<CommandError>() {
        if let Some(out) = src.stdout().filter(|s| !s.is_empty()) {
            anstream::eprint!("{style}{text}{style:#}", text = Indented(out, "[stdout] "));
        }
        if let Some(err) = src.stderr().filter(|s| !s.is_empty()) {
            anstream::eprint!("{style}{text}{style:#}", text = Indented(err, "[stderr] "));
        }
    } else if let Some(body) = e
        .downcast_ref::<minigh::RequestError>()
        .and_then(|src| src.body())
    {
        anstream::eprint!(
            "{style}{text}{style:#}",
            text = Indented(body, "[Response] ")
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Indented<'a>(&'a str, &'static str);

impl fmt::Display for Indented<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(indented(f).with_str(self.1), "{}", self.0)
    }
}
