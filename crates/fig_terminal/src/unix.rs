use std::io;
use tuikit::term::Term;

use crate::{Result, Terminal, TerminalError};

pub struct UnixTerminal {
    term: Term,
}

impl UnixTerminal {
    pub fn new() -> Result<Self> {
        let term = Term::new().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(Self { term })
    }
}

impl Terminal for UnixTerminal {
    fn get_size(&self) -> Result<(u16, u16)> {
        let (width, height) = self.term.term_size().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok((width as u16, height as u16))
    }

    fn clear_screen(&mut self) -> Result<()> {
        self.term.clear_screen().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        self.term.move_cursor_to(x as usize, y as usize).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.term.hide_cursor().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.term.show_cursor().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn set_raw_mode(&mut self) -> Result<()> {
        self.term.enter_alternate_screen().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn reset_mode(&mut self) -> Result<()> {
        self.term.leave_alternate_screen().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn write(&mut self, text: &str) -> Result<()> {
        self.term.write_str(text).map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.term.present().map_err(|e| TerminalError::Terminal(e.to_string()))?;
        Ok(())
    }
}

pub fn create_unix_terminal() -> Result<Box<dyn Terminal>> {
    Ok(Box::new(UnixTerminal::new()?))
}
