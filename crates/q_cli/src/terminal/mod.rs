use anyhow::Result;
use std::fmt;

#[cfg(any(windows, feature = "termwiz-terminal"))]
pub mod termwiz_terminal;

#[cfg(all(unix, feature = "unix-terminal"))]
pub mod tuikit_terminal;

/// Common input events across terminal implementations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
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

/// Common color representation across terminal implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
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

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Reset => write!(f, "Reset"),
            Color::Black => write!(f, "Black"),
            Color::Red => write!(f, "Red"),
            Color::Green => write!(f, "Green"),
            Color::Yellow => write!(f, "Yellow"),
            Color::Blue => write!(f, "Blue"),
            Color::Magenta => write!(f, "Magenta"),
            Color::Cyan => write!(f, "Cyan"),
            Color::White => write!(f, "White"),
            Color::BrightBlack => write!(f, "BrightBlack"),
            Color::BrightRed => write!(f, "BrightRed"),
            Color::BrightGreen => write!(f, "BrightGreen"),
            Color::BrightYellow => write!(f, "BrightYellow"),
            Color::BrightBlue => write!(f, "BrightBlue"),
            Color::BrightMagenta => write!(f, "BrightMagenta"),
            Color::BrightCyan => write!(f, "BrightCyan"),
            Color::BrightWhite => write!(f, "BrightWhite"),
            Color::Rgb(r, g, b) => write!(f, "RGB({},{},{})", r, g, b),
        }
    }
}

/// Common terminal size representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

/// Common terminal interface for both tuikit and termwiz implementations
pub trait Terminal {
    /// Initialize the terminal
    fn init() -> Result<Self> where Self: Sized;
    
    /// Clean up the terminal (restore state)
    fn cleanup(&mut self) -> Result<()>;
    
    /// Read input from the terminal
    fn read_input(&mut self) -> Result<Input>;
    
    /// Write output to the terminal
    fn write_output(&mut self, output: &str) -> Result<()>;
    
    /// Set foreground and background colors
    fn set_color(&mut self, fg: Color, bg: Color) -> Result<()>;
    
    /// Reset colors to default
    fn reset_color(&mut self) -> Result<()>;
    
    /// Clear the screen
    fn clear_screen(&mut self) -> Result<()>;
    
    /// Move cursor to position
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()>;
    
    /// Hide cursor
    fn hide_cursor(&mut self) -> Result<()>;
    
    /// Show cursor
    fn show_cursor(&mut self) -> Result<()>;
    
    /// Get terminal size
    fn get_size(&self) -> Result<TerminalSize>;
}

// Select the appropriate terminal implementation based on platform and features
#[cfg(any(windows, feature = "termwiz-terminal"))]
pub use termwiz_terminal::TermwizTerminal as PlatformTerminal;

#[cfg(all(unix, feature = "unix-terminal", not(feature = "termwiz-terminal")))]
pub use tuikit_terminal::TuikitTerminal as PlatformTerminal;
