use anstyle::Style;
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

macro_rules! boldln {
    ($($arg:tt)*) => {{
        $crate::util::styleln(anstyle::Style::new().bold(), format_args!($($arg)*));
    }};
}

macro_rules! errorln {
    ($($arg:tt)*) => {{
        $crate::util::styleln(
            anstyle::Style::new().bold().fg_color(Some(anstyle::AnsiColor::Red.into())),
            format_args!($($arg)*)
        );
    }};
}

pub(crate) fn styleln(style: Style, fmtargs: std::fmt::Arguments<'_>) {
    anstream::println!("{style}{fmtargs}{style:#}");
}
