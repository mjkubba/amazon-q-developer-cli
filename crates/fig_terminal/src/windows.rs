use std::io::{self, Write};
use crossterm::{
    cursor,
    execute,
    terminal::{self, Clear, ClearType},
};

use crate::{Result, Terminal, TerminalError};

pub struct WindowsTerminal {
    stdout: io::Stdout,
    raw_mode_enabled: bool,
}

impl WindowsTerminal {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stdout: io::stdout(),
            raw_mode_enabled: false,
        })
    }
}

impl Terminal for WindowsTerminal {
    fn get_size(&self) -> Result<(u16, u16)> {
        let (width, height) = terminal::size().map_err(TerminalError::Io)?;
        Ok((width, height))
    }

    fn clear_screen(&mut self) -> Result<()> {
        execute!(self.stdout, Clear(ClearType::All)).map_err(TerminalError::Io)?;
        Ok(())
    }

    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        execute!(self.stdout, cursor::MoveTo(x, y)).map_err(TerminalError::Io)?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::Hide).map_err(TerminalError::Io)?;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::Show).map_err(TerminalError::Io)?;
        Ok(())
    }

    fn set_raw_mode(&mut self) -> Result<()> {
        terminal::enable_raw_mode().map_err(TerminalError::Io)?;
        execute!(self.stdout, terminal::EnterAlternateScreen).map_err(TerminalError::Io)?;
        self.raw_mode_enabled = true;
        Ok(())
    }

    fn reset_mode(&mut self) -> Result<()> {
        if self.raw_mode_enabled {
            execute!(self.stdout, terminal::LeaveAlternateScreen).map_err(TerminalError::Io)?;
            terminal::disable_raw_mode().map_err(TerminalError::Io)?;
            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    fn write(&mut self, text: &str) -> Result<()> {
        write!(self.stdout, "{}", text).map_err(TerminalError::Io)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.stdout.flush().map_err(TerminalError::Io)?;
        Ok(())
    }
}

impl Drop for WindowsTerminal {
    fn drop(&mut self) {
        // Make sure we restore the terminal state when the terminal is dropped
        let _ = self.reset_mode();
    }
}

pub fn create_windows_terminal() -> Result<Box<dyn Terminal>> {
    Ok(Box::new(WindowsTerminal::new()?))
}
