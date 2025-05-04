#![cfg(feature = "minimal")]

use std::io::{self, Write};
use std::time::Duration;
use std::thread::sleep;

use crate::{Color, KeyEvent, Result, Terminal, TerminalError};

/// A minimal terminal implementation that doesn't use any platform-specific features
pub struct MinimalTerminal;

impl MinimalTerminal {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl Terminal for MinimalTerminal {
    fn get_size(&self) -> Result<(u16, u16)> {
        // Default size for minimal terminal
        Ok((80, 24))
    }
    
    fn clear_screen(&mut self) -> Result<()> {
        // Print ANSI clear screen sequence
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        // Print ANSI cursor position sequence
        print!("\x1B[{};{}H", y + 1, x + 1);
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn hide_cursor(&mut self) -> Result<()> {
        // Print ANSI hide cursor sequence
        print!("\x1B[?25l");
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn show_cursor(&mut self) -> Result<()> {
        // Print ANSI show cursor sequence
        print!("\x1B[?25h");
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn set_raw_mode(&mut self) -> Result<()> {
        // Not supported in minimal mode
        Ok(())
    }
    
    fn reset_raw_mode(&mut self) -> Result<()> {
        // Not supported in minimal mode
        Ok(())
    }
    
    fn write(&mut self, text: &str) -> Result<()> {
        print!("{}", text);
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn set_fg_color(&mut self, color: Color) -> Result<()> {
        // Print ANSI color sequence
        let code = match color {
            Color::Default => 39,
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
            Color::Rgb(r, g, b) => {
                print!("\x1B[38;2;{};{};{}m", r, g, b);
                io::stdout().flush().map_err(TerminalError::Io)?;
                return Ok(());
            }
        };
        
        print!("\x1B[{}m", code);
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn set_bg_color(&mut self, color: Color) -> Result<()> {
        // Print ANSI background color sequence
        let code = match color {
            Color::Default => 49,
            Color::Black => 40,
            Color::Red => 41,
            Color::Green => 42,
            Color::Yellow => 43,
            Color::Blue => 44,
            Color::Magenta => 45,
            Color::Cyan => 46,
            Color::White => 47,
            Color::BrightBlack => 100,
            Color::BrightRed => 101,
            Color::BrightGreen => 102,
            Color::BrightYellow => 103,
            Color::BrightBlue => 104,
            Color::BrightMagenta => 105,
            Color::BrightCyan => 106,
            Color::BrightWhite => 107,
            Color::Rgb(r, g, b) => {
                print!("\x1B[48;2;{};{};{}m", r, g, b);
                io::stdout().flush().map_err(TerminalError::Io)?;
                return Ok(());
            }
        };
        
        print!("\x1B[{}m", code);
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn reset_colors(&mut self) -> Result<()> {
        // Print ANSI reset sequence
        print!("\x1B[0m");
        io::stdout().flush().map_err(TerminalError::Io)?;
        Ok(())
    }
    
    fn read_key(&mut self, timeout_ms: u64) -> Result<KeyEvent> {
        // In minimal mode, we can't read keys without blocking
        // So we just sleep for the timeout and return Unknown
        sleep(Duration::from_millis(timeout_ms));
        Ok(KeyEvent::Unknown)
    }
}
