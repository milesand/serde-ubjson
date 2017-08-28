use std;
use std::io;
use std::fmt::{self, Display};

use serde::{ser, de};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(io::Error),
    KeyMustBeAString,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::Io(ref err) => err.description(),
            Error::KeyMustBeAString => "key must be a string",
        }
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
        where T: Display
    {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
        where T: Display
    {
        Error::Message(msg.to_string())
    }
}
