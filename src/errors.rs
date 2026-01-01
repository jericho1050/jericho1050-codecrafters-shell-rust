use std::io;
use thiserror::Error;

/// Comprehensive error type for shell operations
#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Input error: {0}")]
    InputError(String),

    #[error("Redirection error: {0}")]
    RedirectionError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid directory: {0}")]
    InvalidDirectory(String),

    #[error("Invalid quoting in command")]
    InvalidQuoting,

    #[error("Interrupted")]
    Interrupted,
}

pub type ShellResult<T> = Result<T, ShellError>;
