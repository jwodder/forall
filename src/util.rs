pub(crate) fn printlnbold(s: &str) {
    println!("\x1B[1m{s}\x1B[m");
}

pub(crate) fn printlnerror(s: &str) {
    println!("\x1B[1;31m{s}\x1B[m");
}
