use crate::cmd::{CommandOutputError, CommandPlus};
use anstyle::Style;
use clap::Args;
use ghrepo::GHRepo;
use std::path::Path;

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

pub(crate) fn get_ghrepo(p: &Path) -> anyhow::Result<Option<GHRepo>> {
    // Don't use ghrepo::LocalRepo::github_remote(), as that doesn't suppress
    // stderr from Git.
    match CommandPlus::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(p)
        .quiet(true)
        .stderr(std::process::Stdio::null())
        .check_output()
    {
        Ok(s) => Ok(GHRepo::from_url(s.trim()).ok()),
        Err(CommandOutputError::Decode { .. }) => Ok(None),
        Err(CommandOutputError::Exit { rc, .. }) if rc.code() == Some(2) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
