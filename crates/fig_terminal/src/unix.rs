#![cfg(all(unix, feature = "unix-terminal"))]

use std::io;
use tuikit::attr::{Attr, Color as TuikitColor};
use tuikit::term::Term;

use crate::{Color, Result, Terminal, TerminalError};

pub struct UnixTerminal {
    term: Term,
}

impl UnixTerminal {
    pub fn new() -> Result<Self> {
        let term = Term::new().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(Self { term })
    }
    
    fn convert_color(color: Color) -> TuikitColor {
        match color {
            Color::Default => TuikitColor::Default,
            Color::Black => TuikitColor::Black,
            Color::Red => TuikitColor::Red,
            Color::Green => TuikitColor::Green,
            Color::Yellow => TuikitColor::Yellow,
            Color::Blue => TuikitColor::Blue,
            Color::Magenta => TuikitColor::Magenta,
            Color::Cyan => TuikitColor::Cyan,
            Color::White => TuikitColor::White,
            Color::BrightBlack => TuikitColor::LightBlack,
            Color::BrightRed => TuikitColor::LightRed,
            Color::BrightGreen => TuikitColor::LightGreen,
            Color::BrightYellow => TuikitColor::LightYellow,
            Color::BrightBlue => TuikitColor::LightBlue,
            Color::BrightMagenta => TuikitColor::LightMagenta,
            Color::BrightCyan => TuikitColor::LightCyan,
            Color::BrightWhite => TuikitColor::LightWhite,
            Color::Rgb(r, g, b) => TuikitColor::Rgb(r, g, b),
        }
    }
}

impl Terminal for UnixTerminal {
    fn get_size(&self) -> Result<(u16, u16)> {
        let (width, height) = self.term.term_size().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok((width as u16, height as u16))
    }
    
    fn clear_screen(&mut self) -> Result<()> {
        self.term.clear().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        self.term.present().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        self.term.move_cursor_to(x as usize, y as usize).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        self.term.present().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn hide_cursor(&mut self) -> Result<()> {
        self.term.hide_cursor().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        self.term.present().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn show_cursor(&mut self) -> Result<()> {
        self.term.show_cursor().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        self.term.present().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn set_raw_mode(&mut self) -> Result<()> {
        // tuikit is already in raw mode by default
        Ok(())
    }
    
    fn reset_raw_mode(&mut self) -> Result<()> {
        // We'll handle this in cleanup
        Ok(())
    }
    
    fn write(&mut self, text: &str) -> Result<()> {
        let (x, y) = self.term.cursor_pos().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        self.term.print(x, y, text).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        self.term.present().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn set_fg_color(&mut self, color: Color) -> Result<()> {
        let mut attr = self.term.get_attr().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        attr.fg = Self::convert_color(color);
        self.term.set_attr(attr).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn set_bg_color(&mut self, color: Color) -> Result<()> {
        let mut attr = self.term.get_attr().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        attr.bg = Self::convert_color(color);
        self.term.set_attr(attr).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
    
    fn reset_colors(&mut self) -> Result<()> {
        self.term.set_attr(Attr::default()).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
}
