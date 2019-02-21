use atty::{self, Stream};
use std::env;
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

static NO_COLOR: &str = "NO_COLOR";

/// Returns true if the `NO_COLOR` environment variable is set.
pub fn env_no_color() -> bool {
    env::var(NO_COLOR).is_ok()
}

#[allow(dead_code)]
pub fn init_color_stdout(no_color: bool) -> StandardStream {
    if no_color || env_no_color() || atty::isnt(Stream::Stdout) {
        return StandardStream::stdout(ColorChoice::Never);
    }

    StandardStream::stdout(ColorChoice::Auto)
}

pub fn init_color_stderr(no_color: bool) -> StandardStream {
    if no_color || env_no_color() || atty::isnt(Stream::Stderr) {
        return StandardStream::stderr(ColorChoice::Never);
    }

    StandardStream::stderr(ColorChoice::Auto)
}

pub fn set_and_unset_color(stream: &mut StandardStream, s: &str, color: &ColorSpec) {
    stream.set_color(color).unwrap();
    write!(stream, "{}", s).unwrap();
    stream.reset().unwrap();
}
