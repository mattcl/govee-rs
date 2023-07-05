use reqwest;

use crate::schema::{Command, Device};

pub type Result<T> = std::result::Result<T, GoveeError>;

/// GoveeError enumerates all possible errors returned by this library
#[derive(Debug)]
pub enum GoveeError {
    Error(String),
    NoDevicesReturned(),
    Unsupported(Command, Box<Device>),

    /// Represents all other cases of IOError
    IOError(std::io::Error),

    /// Represents all other cases of reqwest::Error
    RequestError(reqwest::Error),
}

impl std::error::Error for GoveeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            GoveeError::Error(_) => None,
            GoveeError::NoDevicesReturned() => None,
            GoveeError::Unsupported(_, _) => None,
            GoveeError::IOError(ref err) => Some(err),
            GoveeError::RequestError(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for GoveeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GoveeError::Error(ref msg) => {
                write!(f, "{}", msg)
            }
            GoveeError::NoDevicesReturned() => {
                write!(f, "No devices were returned from the API")
            }
            GoveeError::Unsupported(ref cmd, ref device) => {
                write!(f, "Unsupported command '{}' for '{}'", cmd, device)
            }
            GoveeError::IOError(ref err) => err.fmt(f),
            GoveeError::RequestError(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for GoveeError {
    fn from(err: std::io::Error) -> GoveeError {
        GoveeError::IOError(err)
    }
}

impl From<reqwest::Error> for GoveeError {
    fn from(err: reqwest::Error) -> GoveeError {
        GoveeError::RequestError(err)
    }
}
