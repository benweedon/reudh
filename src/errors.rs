use std::any;
use std::error;
use std::fmt;
use std::io;
use std::num;

use hyper;
use hyper::error::UriError;
use native_tls;

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

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error {
            s: From::from(error::Error::description(&err)),
        }
    }
}

impl From<native_tls::Error> for Error {
    fn from(err: native_tls::Error) -> Error {
        Error {
            s: From::from(error::Error::description(&err)),
        }
    }
}

impl From<Box<any::Any + Send>> for Error {
    fn from(_: Box<any::Any + Send>) -> Error {
        Error {
            s: From::from("Thread panicked"),
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error {
            s: From::from(error::Error::description(&err)),
        }
    }
}
