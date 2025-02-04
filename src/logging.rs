use crate::cmd::{CommandError, CommandPlus};
use crate::http_util::StatusError;
use crate::project::Project;
use anstyle::{AnsiColor, Style};
use indenter::indented;
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
    if let Some(output) = e
        .downcast_ref::<CommandError>()
        .and_then(|src| src.output())
    {
        if !output.is_empty() {
            anstream::eprint!(
                "{style}{text}{style:#}",
                text = Indented(output, "[Output] ")
            );
        }
    } else if let Some(body) = e.downcast_ref::<StatusError>().and_then(|src| src.body()) {
        if !body.is_empty() {
            anstream::eprint!(
                "{style}{text}{style:#}",
                text = Indented(body, "[Response] ")
            );
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Indented<'a>(&'a str, &'static str);

impl fmt::Display for Indented<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(indented(f).with_str(self.1), "{}", self.0)
    }
}
