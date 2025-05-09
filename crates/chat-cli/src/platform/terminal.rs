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
    fn reset_attribute(&mut self) -> Result<()>;
    
    /// Write text to the terminal
    fn write(&mut self, text: &str) -> Result<()>;
    
    /// Flush the terminal output
    fn flush(&mut self) -> Result<()>;
}

/// Terminal colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalColor {
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
}

/// Terminal attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalAttribute {
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
}

#[cfg(unix)]
pub mod unix {
    use super::*;
    use std::io::{stdout, Stdout};
    
    /// Unix implementation of TerminalHandler
    pub struct UnixTerminalHandler {
        stdout: Stdout,
    }
    
    impl UnixTerminalHandler {
        pub fn new() -> Self {
            Self {
                stdout: stdout(),
            }
        }
    }
    
    impl TerminalHandler for UnixTerminalHandler {
        fn terminal_width(&self) -> Option<usize> {
            // Use termion to get terminal size
            termion::terminal_size().ok().map(|(w, _)| w as usize)
        }
        
        fn clear_line(&mut self) -> Result<()> {
            write!(self.stdout, "{}", termion::clear::CurrentLine)?;
            Ok(())
        }
        
        fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
            write!(self.stdout, "{}", termion::cursor::Goto(column, termion::cursor::DetectCursorPos().unwrap_or((0, 0)).1))?;
            Ok(())
        }
        
        fn show_cursor(&mut self) -> Result<()> {
            write!(self.stdout, "{}", termion::cursor::Show)?;
            Ok(())
        }
        
        fn hide_cursor(&mut self) -> Result<()> {
            write!(self.stdout, "{}", termion::cursor::Hide)?;
            Ok(())
        }
        
        fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()> {
            match color {
                TerminalColor::Black => write!(self.stdout, "{}", termion::color::Fg(termion::color::Black))?,
                TerminalColor::Red => write!(self.stdout, "{}", termion::color::Fg(termion::color::Red))?,
                TerminalColor::Green => write!(self.stdout, "{}", termion::color::Fg(termion::color::Green))?,
                TerminalColor::Yellow => write!(self.stdout, "{}", termion::color::Fg(termion::color::Yellow))?,
                TerminalColor::Blue => write!(self.stdout, "{}", termion::color::Fg(termion::color::Blue))?,
                TerminalColor::Magenta => write!(self.stdout, "{}", termion::color::Fg(termion::color::Magenta))?,
                TerminalColor::Cyan => write!(self.stdout, "{}", termion::color::Fg(termion::color::Cyan))?,
                TerminalColor::White => write!(self.stdout, "{}", termion::color::Fg(termion::color::White))?,
                TerminalColor::BrightBlack => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightBlack))?,
                TerminalColor::BrightRed => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightRed))?,
                TerminalColor::BrightGreen => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightGreen))?,
                TerminalColor::BrightYellow => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightYellow))?,
                TerminalColor::BrightBlue => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightBlue))?,
                TerminalColor::BrightMagenta => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightMagenta))?,
                TerminalColor::BrightCyan => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightCyan))?,
                TerminalColor::BrightWhite => write!(self.stdout, "{}", termion::color::Fg(termion::color::LightWhite))?,
            }
            Ok(())
        }
        
        fn reset_color(&mut self) -> Result<()> {
            write!(self.stdout, "{}", termion::color::Fg(termion::color::Reset))?;
            Ok(())
        }
        
        fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()> {
            match attribute {
                TerminalAttribute::Bold => write!(self.stdout, "{}", termion::style::Bold)?,
                TerminalAttribute::Dim => {} // Not supported by termion
                TerminalAttribute::Italic => write!(self.stdout, "{}", termion::style::Italic)?,
                TerminalAttribute::Underline => write!(self.stdout, "{}", termion::style::Underline)?,
                TerminalAttribute::Blink => write!(self.stdout, "{}", termion::style::Blink)?,
                TerminalAttribute::Reverse => write!(self.stdout, "{}", termion::style::Invert)?,
                TerminalAttribute::Hidden => {} // Not supported by termion
            }
            Ok(())
        }
        
        fn reset_attribute(&mut self) -> Result<()> {
            write!(self.stdout, "{}", termion::style::Reset)?;
            Ok(())
        }
        
        fn write(&mut self, text: &str) -> Result<()> {
            write!(self.stdout, "{}", text)?;
            Ok(())
        }
        
        fn flush(&mut self) -> Result<()> {
            self.stdout.flush()?;
            Ok(())
        }
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use std::io::{stdout, Stdout};
    use ::windows::Win32::System::Console::{
        GetConsoleScreenBufferInfo, 
        SetConsoleCursorPosition, 
        FillConsoleOutputCharacterA, 
        SetConsoleTextAttribute,
        GetStdHandle,
        CONSOLE_SCREEN_BUFFER_INFO,
        COORD,
        STD_OUTPUT_HANDLE,
    };
    use ::windows::Win32::Foundation::HANDLE;
    
    /// Windows implementation of TerminalHandler
    pub struct WindowsTerminalHandler {
        stdout: Stdout,
    }
    
    impl WindowsTerminalHandler {
        pub fn new() -> Self {
            Self {
                stdout: stdout(),
            }
        }
        
        fn get_console_handle(&self) -> ::windows::core::Result<HANDLE> {
            unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }
        }
        
        fn get_console_info(&self) -> Option<CONSOLE_SCREEN_BUFFER_INFO> {
            let handle = match self.get_console_handle() {
                Ok(h) => h,
                Err(_) => return None,
            };
            
            let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
            match unsafe { GetConsoleScreenBufferInfo(handle, &mut info) } {
                Ok(_) => Some(info),
                Err(_) => None,
            }
        }
    }
    
    impl TerminalHandler for WindowsTerminalHandler {
        fn terminal_width(&self) -> Option<usize> {
            self.get_console_info().map(|info| info.dwSize.X as usize)
        }
        
        fn clear_line(&mut self) -> Result<()> {
            let handle = self.get_console_handle()?;
            let info = self.get_console_info().ok_or_else(|| eyre::eyre!("Failed to get console info"))?;
            
            let cursor_pos = info.dwCursorPosition;
            let width = info.dwSize.X;
            
            // Set cursor to beginning of line
            let new_pos = COORD { X: 0, Y: cursor_pos.Y };
            unsafe { SetConsoleCursorPosition(handle, new_pos)?; }
            
            // Fill line with spaces
            let mut chars_written = 0;
            unsafe { 
                FillConsoleOutputCharacterA(
                    handle, 
                    b' ' as u8, 
                    width as u32, 
                    new_pos, 
                    &mut chars_written
                )?;
            }
            
            // Reset cursor position
            unsafe { SetConsoleCursorPosition(handle, new_pos)?; }
            
            Ok(())
        }
        
        fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
            let handle = self.get_console_handle()?;
            let info = self.get_console_info().ok_or_else(|| eyre::eyre!("Failed to get console info"))?;
            
            let new_pos = COORD { X: column as i16, Y: info.dwCursorPosition.Y };
            unsafe { SetConsoleCursorPosition(handle, new_pos)?; }
            
            Ok(())
        }
        
        fn show_cursor(&mut self) -> Result<()> {
            // Windows doesn't have a direct API for this
            // We would need to use SetConsoleCursorInfo
            // For now, we'll just do nothing
            Ok(())
        }
        
        fn hide_cursor(&mut self) -> Result<()> {
            // Windows doesn't have a direct API for this
            // We would need to use SetConsoleCursorInfo
            // For now, we'll just do nothing
            Ok(())
        }
        
        fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()> {
            let handle = self.get_console_handle()?;
            let info = self.get_console_info().ok_or_else(|| eyre::eyre!("Failed to get console info"))?;
            
            // Get current background color and keep it
            let bg_color = info.wAttributes & 0xF0;
            
            // Set new foreground color
            let fg_color = match color {
                TerminalColor::Black => 0,
                TerminalColor::Red => 4,
                TerminalColor::Green => 2,
                TerminalColor::Yellow => 6,
                TerminalColor::Blue => 1,
                TerminalColor::Magenta => 5,
                TerminalColor::Cyan => 3,
                TerminalColor::White => 7,
                TerminalColor::BrightBlack => 8,
                TerminalColor::BrightRed => 12,
                TerminalColor::BrightGreen => 10,
                TerminalColor::BrightYellow => 14,
                TerminalColor::BrightBlue => 9,
                TerminalColor::BrightMagenta => 13,
                TerminalColor::BrightCyan => 11,
                TerminalColor::BrightWhite => 15,
            };
            
            unsafe { SetConsoleTextAttribute(handle, bg_color | fg_color)?; }
            
            Ok(())
        }
        
        fn reset_color(&mut self) -> Result<()> {
            let handle = self.get_console_handle()?;
            
            // Default color is white on black (7)
            unsafe { SetConsoleTextAttribute(handle, 7)?; }
            
            Ok(())
        }
        
        fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()> {
            let handle = self.get_console_handle()?;
            let info = self.get_console_info().ok_or_else(|| eyre::eyre!("Failed to get console info"))?;
            
            let current_attr = info.wAttributes;
            let new_attr = match attribute {
                TerminalAttribute::Bold => current_attr | 8, // FOREGROUND_INTENSITY
                TerminalAttribute::Dim => current_attr,      // Not supported
                TerminalAttribute::Italic => current_attr,   // Not supported
                TerminalAttribute::Underline => current_attr, // Not supported
                TerminalAttribute::Blink => current_attr,    // Not supported
                TerminalAttribute::Reverse => {
                    // Swap foreground and background colors
                    let fg = current_attr & 0x0F;
                    let bg = current_attr & 0xF0;
                    (fg << 4) | (bg >> 4)
                },
                TerminalAttribute::Hidden => current_attr,   // Not supported
            };
            
            unsafe { SetConsoleTextAttribute(handle, new_attr)?; }
            
            Ok(())
        }
        
        fn reset_attribute(&mut self) -> Result<()> {
            // Reset to default attributes (white on black)
            let handle = self.get_console_handle()?;
            unsafe { SetConsoleTextAttribute(handle, 7)?; }
            
            Ok(())
        }
        
        fn write(&mut self, text: &str) -> Result<()> {
            write!(self.stdout, "{}", text)?;
            Ok(())
        }
        
        fn flush(&mut self) -> Result<()> {
            self.stdout.flush()?;
            Ok(())
        }
    }
}

/// Factory function to create the appropriate terminal handler for the current platform
pub fn create_terminal_handler() -> Box<dyn TerminalHandler> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixTerminalHandler::new())
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsTerminalHandler::new())
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        Box::new(unix::UnixTerminalHandler::new())
    }
}
