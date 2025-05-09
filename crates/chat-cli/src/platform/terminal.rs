use eyre::Result;
use std::io::Write;

/// Trait for platform-agnostic terminal handling
pub trait TerminalHandler {
    /// Get the current terminal width
    fn terminal_width(&self) -> Option<usize>;
    
    /// Clear the current line
    fn clear_line(&mut self) -> Result<()>;
    
    /// Move cursor to a specific column
    fn move_cursor_to_column(&mut self, column: u16) -> Result<()>;
    
    /// Show the cursor
    fn show_cursor(&mut self) -> Result<()>;
    
    /// Hide the cursor
    fn hide_cursor(&mut self) -> Result<()>;
    
    /// Set foreground color
    fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()>;
    
    /// Reset colors to default
    fn reset_color(&mut self) -> Result<()>;
    
    /// Set text attribute
    fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()>;
    
    /// Reset all attributes
    fn reset_attributes(&mut self) -> Result<()>;
    
    /// Print text to the terminal
    fn print(&mut self, text: &str) -> Result<()>;
    
    /// Flush output
    fn flush(&mut self) -> Result<()>;
}

/// Terminal colors abstraction
#[derive(Debug, Clone, Copy)]
pub enum TerminalColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    DarkGrey,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    Grey,
    Reset,
}

/// Terminal text attributes
#[derive(Debug, Clone, Copy)]
pub enum TerminalAttribute {
    Bold,
    Dim,
    Italic,
    Underlined,
    SlowBlink,
    RapidBlink,
    Reverse,
    Hidden,
    CrossedOut,
    Reset,
}

#[cfg(unix)]
pub mod unix {
    use super::*;
    use crossterm::{
        cursor,
        style::{self, Attribute, Color},
        terminal,
        QueueableCommand,
    };

    /// Unix implementation of TerminalHandler using crossterm
    pub struct CrosstermHandler<W: Write> {
        writer: W,
    }

    impl<W: Write> CrosstermHandler<W> {
        pub fn new(writer: W) -> Self {
            Self { writer }
        }
    }

    impl<W: Write> TerminalHandler for CrosstermHandler<W> {
        fn terminal_width(&self) -> Option<usize> {
            terminal::size().ok().map(|(w, _)| w as usize)
        }

        fn clear_line(&mut self) -> Result<()> {
            self.writer.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            Ok(())
        }

        fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
            self.writer.queue(cursor::MoveToColumn(column))?;
            Ok(())
        }

        fn show_cursor(&mut self) -> Result<()> {
            self.writer.queue(cursor::Show)?;
            Ok(())
        }

        fn hide_cursor(&mut self) -> Result<()> {
            self.writer.queue(cursor::Hide)?;
            Ok(())
        }

        fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()> {
            let color = match color {
                TerminalColor::Black => Color::Black,
                TerminalColor::Red => Color::Red,
                TerminalColor::Green => Color::Green,
                TerminalColor::Yellow => Color::Yellow,
                TerminalColor::Blue => Color::Blue,
                TerminalColor::Magenta => Color::Magenta,
                TerminalColor::Cyan => Color::Cyan,
                TerminalColor::White => Color::White,
                TerminalColor::DarkGrey => Color::DarkGrey,
                TerminalColor::DarkRed => Color::DarkRed,
                TerminalColor::DarkGreen => Color::DarkGreen,
                TerminalColor::DarkYellow => Color::DarkYellow,
                TerminalColor::DarkBlue => Color::DarkBlue,
                TerminalColor::DarkMagenta => Color::DarkMagenta,
                TerminalColor::DarkCyan => Color::DarkCyan,
                TerminalColor::Grey => Color::Grey,
                TerminalColor::Reset => Color::Reset,
            };
            self.writer.queue(style::SetForegroundColor(color))?;
            Ok(())
        }

        fn reset_color(&mut self) -> Result<()> {
            self.writer.queue(style::ResetColor)?;
            Ok(())
        }

        fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()> {
            let attr = match attribute {
                TerminalAttribute::Bold => Attribute::Bold,
                TerminalAttribute::Dim => Attribute::Dim,
                TerminalAttribute::Italic => Attribute::Italic,
                TerminalAttribute::Underlined => Attribute::Underlined,
                TerminalAttribute::SlowBlink => Attribute::SlowBlink,
                TerminalAttribute::RapidBlink => Attribute::RapidBlink,
                TerminalAttribute::Reverse => Attribute::Reverse,
                TerminalAttribute::Hidden => Attribute::Hidden,
                TerminalAttribute::CrossedOut => Attribute::CrossedOut,
                TerminalAttribute::Reset => Attribute::Reset,
            };
            self.writer.queue(style::SetAttribute(attr))?;
            Ok(())
        }

        fn reset_attributes(&mut self) -> Result<()> {
            self.writer.queue(style::SetAttribute(Attribute::Reset))?;
            Ok(())
        }

        fn print(&mut self, text: &str) -> Result<()> {
            self.writer.queue(style::Print(text))?;
            Ok(())
        }

        fn flush(&mut self) -> Result<()> {
            self.writer.flush()?;
            Ok(())
        }
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use crossterm::{
        cursor,
        style::{self, Attribute, Color},
        terminal,
        QueueableCommand,
    };

    /// Windows implementation of TerminalHandler
    /// 
    /// Note: This is currently using crossterm which has Windows support,
    /// but we might need to add Windows-specific optimizations or workarounds.
    pub struct WindowsTerminalHandler<W: Write> {
        writer: W,
    }

    impl<W: Write> WindowsTerminalHandler<W> {
        pub fn new(writer: W) -> Self {
            Self { writer }
        }
    }

    impl<W: Write> TerminalHandler for WindowsTerminalHandler<W> {
        fn terminal_width(&self) -> Option<usize> {
            terminal::size().ok().map(|(w, _)| w as usize)
        }

        fn clear_line(&mut self) -> Result<()> {
            self.writer.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            Ok(())
        }

        fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
            self.writer.queue(cursor::MoveToColumn(column))?;
            Ok(())
        }

        fn show_cursor(&mut self) -> Result<()> {
            self.writer.queue(cursor::Show)?;
            Ok(())
        }

        fn hide_cursor(&mut self) -> Result<()> {
            self.writer.queue(cursor::Hide)?;
            Ok(())
        }

        fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()> {
            let color = match color {
                TerminalColor::Black => Color::Black,
                TerminalColor::Red => Color::Red,
                TerminalColor::Green => Color::Green,
                TerminalColor::Yellow => Color::Yellow,
                TerminalColor::Blue => Color::Blue,
                TerminalColor::Magenta => Color::Magenta,
                TerminalColor::Cyan => Color::Cyan,
                TerminalColor::White => Color::White,
                TerminalColor::DarkGrey => Color::DarkGrey,
                TerminalColor::DarkRed => Color::DarkRed,
                TerminalColor::DarkGreen => Color::DarkGreen,
                TerminalColor::DarkYellow => Color::DarkYellow,
                TerminalColor::DarkBlue => Color::DarkBlue,
                TerminalColor::DarkMagenta => Color::DarkMagenta,
                TerminalColor::DarkCyan => Color::DarkCyan,
                TerminalColor::Grey => Color::Grey,
                TerminalColor::Reset => Color::Reset,
            };
            self.writer.queue(style::SetForegroundColor(color))?;
            Ok(())
        }

        fn reset_color(&mut self) -> Result<()> {
            self.writer.queue(style::ResetColor)?;
            Ok(())
        }

        fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()> {
            let attr = match attribute {
                TerminalAttribute::Bold => Attribute::Bold,
                TerminalAttribute::Dim => Attribute::Dim,
                TerminalAttribute::Italic => Attribute::Italic,
                TerminalAttribute::Underlined => Attribute::Underlined,
                TerminalAttribute::SlowBlink => Attribute::SlowBlink,
                TerminalAttribute::RapidBlink => Attribute::RapidBlink,
                TerminalAttribute::Reverse => Attribute::Reverse,
                TerminalAttribute::Hidden => Attribute::Hidden,
                TerminalAttribute::CrossedOut => Attribute::CrossedOut,
                TerminalAttribute::Reset => Attribute::Reset,
            };
            self.writer.queue(style::SetAttribute(attr))?;
            Ok(())
        }

        fn reset_attributes(&mut self) -> Result<()> {
            self.writer.queue(style::SetAttribute(Attribute::Reset))?;
            Ok(())
        }

        fn print(&mut self, text: &str) -> Result<()> {
            self.writer.queue(style::Print(text))?;
            Ok(())
        }

        fn flush(&mut self) -> Result<()> {
            self.writer.flush()?;
            Ok(())
        }
    }
}

/// Factory function to create the appropriate terminal handler for the current platform
pub fn create_terminal_handler<W: Write + 'static>(writer: W) -> Box<dyn TerminalHandler> {
    #[cfg(unix)]
    {
        Box::new(unix::CrosstermHandler::new(writer))
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsTerminalHandler::new(writer))
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        Box::new(unix::CrosstermHandler::new(writer))
    }
}
