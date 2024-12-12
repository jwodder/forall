use anstyle::{AnsiColor, Style};

pub(crate) fn printlnbold(s: &str) {
    printlnstyled(s, Style::new().bold());
}

pub(crate) fn printlnerror(s: &str) {
    printlnstyled(s, Style::new().fg_color(Some(AnsiColor::Red.into())).bold());
}

fn printlnstyled(s: &str, style: Style) {
    anstream::println!("{style}{s}{style:#}");
}
