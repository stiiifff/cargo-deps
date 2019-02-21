use crate::color;
use std::error::Error;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::io;
use termcolor::{Color, ColorSpec};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliErrorKind {
    TomlNoName,
    Io(io::Error),
    Generic(String),
}

impl CliErrorKind {
    fn description(&self) -> &str {
        match *self {
            CliErrorKind::Generic(ref e) => e,
            CliErrorKind::TomlNoName => "No name for package in toml file",
            CliErrorKind::Io(ref e) => e.description(),
        }
    }
}

impl From<CliErrorKind> for CliError {
    fn from(kind: CliErrorKind) -> Self {
        CliError {
            error: kind.description().into(),
            kind,
        }
    }
}

#[derive(Debug)]
pub struct CliError {
    /// The formatted error message
    pub error: String,
    /// The type of error
    pub kind: CliErrorKind,
}

// Copies clog::error::Error;
impl CliError {
    /// Print this error and immediately exit the program.
    pub fn exit(&self, no_color: bool) -> ! {
        let mut stderr = color::init_color_stderr(no_color);
        let mut color = ColorSpec::new();
        color.set_fg(Some(Color::Red)).set_bold(true);

        color::set_and_unset_color(&mut stderr, "error:", &color);
        eprintln!(" {}", self);
        ::std::process::exit(1)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.error)
    }
}

impl Error for CliError {
    fn description(&self) -> &str {
        self.kind.description()
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            CliErrorKind::Io(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(ioe: io::Error) -> Self {
        CliError {
            error: ioe.description().into(),
            kind: CliErrorKind::Io(ioe),
        }
    }
}

impl From<toml::ser::Error> for CliError {
    fn from(err: toml::ser::Error) -> Self {
        From::from(CliErrorKind::Generic(format!(
            "Could not parse input as TOML: {}",
            err
        )))
    }
}
