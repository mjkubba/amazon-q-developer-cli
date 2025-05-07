use std::io;
use thiserror::Error;

mod selector;
pub use selector::Selector;

#[cfg(not(target_os = "windows"))]
mod termwiz_terminal;
#[cfg(not(target_os = "windows"))]
pub use termwiz_terminal::TermwizTerminal;

#[cfg(feature = "minimal")]
mod minimal;

#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Terminal error: {0}")]
    Terminal(String),
    
    #[cfg(not(target_os = "windows"))]
    #[error("Termwiz error: {0}")]
    Termwiz(#[from] termwiz::Error),
    
    #[error("Unsupported platform")]
    UnsupportedPlatform,
}

pub type Result<T> = std::result::Result<T, TerminalError>;

/// Common terminal interface
pub trait Terminal {
    /// Get the terminal size (width, height)
    fn get_size(&self) -> Result<(u16, u16)>;
    
    /// Clear the screen
    fn clear_screen(&mut self) -> Result<()>;
    
    /// Move cursor to position
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()>;
    
    /// Hide cursor
    fn hide_cursor(&mut self) -> Result<()>;
    
    /// Show cursor
    fn show_cursor(&mut self) -> Result<()>;
    
    /// Set raw mode
    fn set_raw_mode(&mut self) -> Result<()>;
    
    /// Reset raw mode
    fn reset_raw_mode(&mut self) -> Result<()>;
    
    /// Write text at current cursor position
    fn write(&mut self, text: &str) -> Result<()>;
    
    /// Set foreground color
    fn set_fg_color(&mut self, color: Color) -> Result<()>;
    
    /// Set background color
    fn set_bg_color(&mut self, color: Color) -> Result<()>;
    
    /// Reset colors
    fn reset_colors(&mut self) -> Result<()>;
    
    /// Read a key from the terminal
    fn read_key(&mut self, timeout_ms: u64) -> Result<KeyEvent>;
}

/// Common color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
}

/// Key event representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Ctrl(char),
    Alt(char),
    F(u8),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Insert,
    Enter,
    Tab,
    BackTab,
    Esc,
    Unknown,
}

/// Create a terminal instance
pub fn create_terminal() -> Result<Box<dyn Terminal>> {
    #[cfg(feature = "minimal")]
    {
        return Ok(Box::new(minimal::MinimalTerminal::new()?));
    }
    
    #[cfg(all(not(feature = "minimal"), not(target_os = "windows")))]
    {
        return Ok(Box::new(termwiz_terminal::TermwizTerminal::new()?));
    }
    
    #[cfg(all(not(feature = "minimal"), target_os = "windows"))]
    {
        // For now, return an error on Windows
        return Err(TerminalError::UnsupportedPlatform);
    }
}
