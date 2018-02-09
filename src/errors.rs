use std::error;
use std::fmt;

use hyper;
use hyper::error::UriError;

#[derive(Debug)]
pub struct Error {
    s: String,
}

impl Error {
    pub fn new(s: &str) -> Error {
        Error { s: From::from(s) }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.s.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error {
            s: From::from("Unknown error"),
        }
    }
}

impl From<UriError> for Error {
    fn from(err: UriError) -> Error {
        Error {
            s: From::from(error::Error::description(&err)),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error {
            s: From::from(error::Error::description(&err)),
        }
    }
}