use std::io;
use thiserror::Error;

mod selector;
pub use selector::Selector;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Terminal feature not available")]
    FeatureNotAvailable,
}

pub type Result<T> = std::result::Result<T, TerminalError>;

// Re-export platform-specific modules based on features
cfg_if::cfg_if! {
    if #[cfg(feature = "unix-terminal")] {
        mod unix;
        pub use unix::*;
    } else if #[cfg(feature = "windows-terminal")] {
        mod windows;
        pub use windows::*;
    } else {
        mod minimal;
        pub use minimal::*;
    }
}

// Common terminal interface
pub trait Terminal {
    fn get_size(&self) -> Result<(u16, u16)>;
    fn clear_screen(&mut self) -> Result<()>;
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()>;
    fn hide_cursor(&mut self) -> Result<()>;
    fn show_cursor(&mut self) -> Result<()>;
    fn set_raw_mode(&mut self) -> Result<()>;
    fn reset_mode(&mut self) -> Result<()>;
    fn write(&mut self, text: &str) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

// Factory function to create a terminal instance
pub fn create_terminal() -> Result<Box<dyn Terminal>> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "unix-terminal")] {
            unix::create_unix_terminal()
        } else if #[cfg(feature = "windows-terminal")] {
            windows::create_windows_terminal()
        } else {
            minimal::create_minimal_terminal()
        }
    }
}
