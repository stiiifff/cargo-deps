use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Toml(String),
    Io(io::Error),
    Generic(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Self::Generic(ref e) => write!(f, "{}", e),
            Self::Toml(ref e) => write!(f, "Could not parse toml file: {}", e),
            Self::Io(ref e) => write!(f, "{}", e),
        }
    }
}

impl CliError {
    /// Print this error and immediately exit the program.
    pub fn exit(&self) -> ! {
        eprintln!("error: {}", self);
        ::std::process::exit(1)
    }
}

impl From<io::Error> for CliError {
    fn from(ioe: io::Error) -> Self {
        Self::Io(ioe)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(format!("Could not parse input as TOML: {}", err))
    }
}
