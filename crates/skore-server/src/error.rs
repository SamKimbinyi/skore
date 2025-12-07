use thiserror::Error;
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Unexpected null bulk string")]
    NullBulkString,

    #[error("Invalid UTF-8 in command")]
    InvalidUtf8,

    #[error("Command requires exactly {expected} argument(s), got {got}")]
    WrongArgCount { expected: usize, got: usize },

    #[error("First element must be a bulk string")]
    FirstElementNotBulkString,

    #[error("Unknown command '{0}'")]
    UnknownCommand(String),
}
