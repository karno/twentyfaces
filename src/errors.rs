use std::error;
use std::fmt;

use thiserror::Error;

use crate::twitter_api::TwitterError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Twitter failed: {0}")]
    Twitter(#[from] TwitterError),
    #[error("Invalid configuration: {0}")]
    Configuration(#[from] ConfigurationError),
}

#[derive(Debug)]
pub struct ConfigurationError {
    message: String,
}
impl ConfigurationError {
    pub fn new<T: Into<String>>(message: T) -> Self {
        ConfigurationError {
            message: message.into(),
        }
    }
}

impl error::Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Configuration error: {}", self.message)
    }
}
