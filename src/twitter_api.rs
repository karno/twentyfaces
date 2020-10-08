use thiserror::Error;

pub mod auth;
pub mod misc;
pub mod models;
pub mod statuses;

use reqwest;
use std::error;
use std::fmt;
use std::io;

pub type TwitterResult<T> = std::result::Result<T, TwitterError>;

#[derive(Error, Debug)]
pub enum TwitterError {
    #[error("IO failed: {0}")]
    Io(#[from] io::Error),
    #[error("OAuth failed: {0}")]
    Sign(#[from] reqwest_oauth1::Error),
    #[error("reqwest failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Twitter data read error: {0}")]
    Data(#[from] TwitterDataError),
    #[error("Twitter error: {0}")]
    Twitter(#[from] TwitterAccessError),
}

#[derive(Debug)]
pub struct TwitterAccessError {
    pub code: u32,
    pub message: String,
}

impl error::Error for TwitterAccessError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for TwitterAccessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error Code: {}: {}", self.code, self.message)
    }
}

#[derive(Debug)]
pub struct TwitterDataError {
    pub field: String,
    pub body: String,
}

impl TwitterDataError {
    pub fn new<TField: Into<String>, TBody: Into<String>>(field: TField, body: TBody) -> Self {
        TwitterDataError {
            field: field.into(),
            body: body.into(),
        }
    }
}

impl error::Error for TwitterDataError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
impl fmt::Display for TwitterDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "field {} is required but not found or not acceptable: {}",
            self.field, self.body
        )
    }
}
