/// Module for server error handling.
use std::io;
use std::{error::Error, fmt::Display, num::ParseIntError};

/// A specialized [`Result`] type for server operations.
pub type Result<T> = std::result::Result<T, ServerError>;

use InvalidResponse::{InvalidValue, MissingValue};

/// Errors that can occur while communicating with the server.
#[derive(Debug)]
pub enum ServerError {
    /// An IO error.
    Io(io::Error),
    /// An invalid response from the server.
    InvalidResponse(InvalidResponse),
}

impl Error for ServerError {}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::Io(err) => write!(f, "IO error: {}", err),
            ServerError::InvalidResponse(err) => write!(f, "Invalid response: {}", err),
        }
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<InvalidResponse> for ServerError {
    fn from(err: InvalidResponse) -> Self {
        Self::InvalidResponse(err)
    }
}

impl From<ParseIntError> for ServerError {
    fn from(err: ParseIntError) -> Self {
        Self::InvalidResponse(err.into())
    }
}

/// Error type specifying the kind of invalid response.
#[derive(Debug)]
pub enum InvalidResponse {
    /// A missing value.
    MissingValue,
    /// An invalid value.
    InvalidValue(ParseIntError),
}

impl Error for InvalidResponse {}

impl Display for InvalidResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingValue => write!(f, "missing value"),
            InvalidValue(err) => write!(f, "invalid value: {}", err),
        }
    }
}

impl From<ParseIntError> for InvalidResponse {
    fn from(err: ParseIntError) -> Self {
        Self::InvalidValue(err)
    }
}
