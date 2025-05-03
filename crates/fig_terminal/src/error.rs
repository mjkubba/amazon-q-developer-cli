use thiserror::Error;

pub type Result<T> = std::result::Result<T, TerminalError>;

#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Crossterm error: {0}")]
    Crossterm(#[from] crossterm::ErrorKind),

    #[error("Terminal operation failed: {0}")]
    Operation(String),
}