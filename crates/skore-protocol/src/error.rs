#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("Incomplete Data")]
    Incomplete,

    #[error("Invalid format {0}")]
    InvalidFormat(String),

    #[error("Invalid UTF-8")]
    InvalidUtf8,

    #[error("Invalid Integer")]
    InvalidInteger,
}
