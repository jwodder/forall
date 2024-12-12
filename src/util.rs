use anstyle::{AnsiColor, Style};
use clap::Args;

#[derive(Args, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Options {
    /// Don't exit on errors
    #[arg(short, long, global = true)]
    pub(crate) keep_going: bool,

    /// Suppress successful command output
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,
}

pub(crate) fn printlnbold(s: &str) {
    printlnstyled(s, Style::new().bold());
}

pub(crate) fn printlnerror(s: &str) {
    printlnstyled(s, Style::new().fg_color(Some(AnsiColor::Red.into())).bold());
}

fn printlnstyled(s: &str, style: Style) {
    anstream::println!("{style}{s}{style:#}");
}
