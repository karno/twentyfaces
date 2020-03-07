// this code derived from https://qiita.com/hppRC/items/05a81b56d12d663d03e0

pub mod auth;
pub mod misc;
pub mod models;

use reqwest;
use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Network(reqwest::Error),
    Http(u16, String),
    Twitter(TwitterError),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &*self {
            Error::Io(ref err) => Some(err),
            Error::Network(ref err) => Some(err),
            Error::Twitter(ref err) => Some(err),
            Error::Http(_, __) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "Error: IO: {}", err),
            Error::Network(ref err) => write!(f, "Error: Network: {}", err),
            Error::Twitter(ref err) => write!(f, "Error: Twitter: {}", err),
            Error::Http(ref code, ref msg) => write!(f, "Error: HTTP {}: {}", code, msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Network(err)
    }
}

#[derive(Debug)]
pub struct TwitterError {
    pub code: u32,
    pub message: String,
}

impl error::Error for TwitterError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for TwitterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error Code: {}: {}", self.code, self.message)
    }
}
