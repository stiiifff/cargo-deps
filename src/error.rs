use crate::color;
use std::error::Error;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::io;
use termcolor::{Color, ColorSpec};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    TomlNoName,
    TomlNoPackage,
    Io(io::Error),
    Generic(String),
}

impl Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::Generic(ref e) => e,
            CliError::TomlNoName => "No name for package in toml file",
            CliError::TomlNoPackage => "No package in toml file",
            CliError::Io(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CliError::Io(ref e) => Some(e),
            _ => None,
        }
    }
}

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
        write!(f, "{}", self.description())
    }
}

impl From<io::Error> for CliError {
    fn from(ioe: io::Error) -> Self {
        CliError::Io(ioe)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError::Generic(format!("Could not parse input as TOML: {}", err))
    }
}
