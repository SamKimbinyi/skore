use std::io;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Returned when trying to read or write to storage and an I/O error occurs
    ///
    /// Example: When file operations fail (disk full, permissions, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Returned when a key is invalid.
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Returned when a value is invalid
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    ///Returned when a ket is not found in the Skore
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },

    /// Store is corrupted, ie. Invalid file format
    #[error("Store is corrupted: {reason}")]
    Corruption { reason: String },

    ///Store is closed,
    #[error("Store is closed")]
    StoreClosed,

    #[error("UTF-8 error: {0}")]
    UTF8Error(#[from] FromUtf8Error),

    ///Generic Internal error for other cases
    #[error("Internal Error: {0}")]
    Internal(String),
}

impl Error {
    /// Creates a new InvalidKey error.
    ///
    /// This is a convenience method to avoid having to write
    /// Error::InvalidKey("message".to_string()) everywhere.
    pub fn invalid_key<S: Into<String>>(msg: S) -> Self {
        Error::InvalidKey(msg.into())
    }

    /// Creates a new InvalidValue error.
    pub fn invalid_value<S: Into<String>>(msg: S) -> Self {
        Error::InvalidValue(msg.into())
    }

    /// Creates a new KeyNotFound error.
    pub fn key_not_found<S: Into<String>>(key: S) -> Self {
        Error::KeyNotFound { key: key.into() }
    }

    /// Creates a new Corruption error.
    pub fn corruption<S: Into<String>>(reason: S) -> Self {
        Error::Corruption {
            reason: reason.into(),
        }
    }

    /// Creates a new Internal error.
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Error::Internal(msg.into())
    }

    /// Returns true if this error is related to I/O operations.
    pub fn is_io_error(&self) -> bool {
        matches!(self, Error::Io(_))
    }

    /// Returns true if this error indicates data corruption.
    pub fn is_corruption(&self) -> bool {
        matches!(self, Error::Corruption { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = Error::invalid_key("key is empty");
        assert_eq!(err.to_string(), "Invalid key: key is empty");

        let err = Error::key_not_found("user:123");
        assert_eq!(err.to_string(), "Key not found: user:123");

        let err = Error::corruption("unexpected EOF");
        assert_eq!(err.to_string(), "Store is corrupted: unexpected EOF");
    }

    #[test]
    fn test_io_error_conversion() {
        // The #[from] attribute allows automatic conversion
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();

        assert!(matches!(err, Error::Io(_)));
        assert!(err.is_io_error());
    }

    #[test]
    fn test_result_with_question_mark() {
        // This function demonstrates using ? operator with our Result type
        fn read_file() -> Result<String> {
            // If this fails, io::Error is automatically converted to Error::Io
            let contents = std::fs::read_to_string("test.txt")?;
            Ok(contents)
        }

        // The error should be an Error::Io variant
        let result = read_file();
        assert!(result.is_err());
    }

    #[test]
    fn test_helper_methods() {
        let err = Error::invalid_key("empty");
        assert_eq!(err.to_string(), "Invalid key: empty");

        let err = Error::corruption("bad magic number");
        assert!(err.is_corruption());
    }
}
