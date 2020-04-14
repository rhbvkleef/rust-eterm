use std::error;
use std::fmt::{ self, Debug, Display };
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Message(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref msg) => f.write_str(msg),
            Error::Io(ref err) => Display::fmt(err, f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}
