use std;
use std::io;
use std::fmt::{self, Display};

use serde::{ser, de};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    err: ErrorImpl,
}

#[derive(Debug)]
enum ErrorImpl {
    Message(String),
    Io(io::Error),
}

impl Error {
    pub fn io(err: io::Error) -> Self {
        Error { err: ErrorImpl::Io(err) }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self.err {
            ErrorImpl::Message(ref msg) => msg,
            ErrorImpl::Io(ref err) => err.description(),
        }
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
        where T: Display
    {
        Error { err: ErrorImpl::Message(msg.to_string()) }
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
        where T: Display
    {
        Error { err: ErrorImpl::Message(msg.to_string()) }
    }
}
