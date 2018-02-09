use std::error;
use std::fmt;

use hyper;
use hyper::error::UriError;

#[derive(Debug)]
pub enum Error {
    Err,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "This is an error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This is an error")
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::Err
    }
}

impl From<UriError> for Error {
    fn from(_: UriError) -> Error {
        Error::Err
    }
}

impl From<hyper::Error> for Error {
    fn from(_: hyper::Error) -> Error {
        Error::Err
    }
}
