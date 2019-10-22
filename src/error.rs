use std::{
    fmt::{self, Display, Formatter},
    io,
};

/// Result type for the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the crate.
#[derive(Debug)]
pub enum Error {
    /// Errors originating from the toml crate.
    Toml(String),
    /// IO errors.
    Io(io::Error),
    /// All other errors.
    Generic(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::Generic(ref e) => write!(f, "{}", e),
            Self::Toml(ref e) => write!(f, "Could not parse toml file: {}", e),
            Self::Io(ref e) => write!(f, "{}", e),
        }
    }
}

impl Error {
    /// Print this error and immediately exit the program.
    pub fn exit(&self) -> ! {
        eprintln!("error: {}", self);
        ::std::process::exit(1)
    }
}

impl From<io::Error> for Error {
    fn from(ioe: io::Error) -> Self {
        Self::Io(ioe)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(format!("Could not parse input as TOML: {}", err))
    }
}
