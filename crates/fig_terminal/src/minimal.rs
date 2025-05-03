use std::io::{self, Write};

use crate::{Result, Terminal, TerminalError};

// A minimal terminal implementation that just writes to stdout
// This is used when neither unix-terminal nor windows-terminal features are enabled
pub struct MinimalTerminal {
    stdout: io::Stdout,
}

impl MinimalTerminal {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stdout: io::stdout(),
        })
    }
}

impl Terminal for MinimalTerminal {
    fn get_size(&self) -> Result<(u16, u16)> {
        // Default size for minimal terminal
        Ok((80, 24))
    }

    fn clear_screen(&mut self) -> Result<()> {
        // Can't clear screen in minimal mode
        println!("\n\n");
        Ok(())
    }

    fn move_cursor(&mut self, _x: u16, _y: u16) -> Result<()> {
        // Can't move cursor in minimal mode
        Err(TerminalError::FeatureNotAvailable)
    }

    fn hide_cursor(&mut self) -> Result<()> {
        // Can't hide cursor in minimal mode
        Err(TerminalError::FeatureNotAvailable)
    }

    fn show_cursor(&mut self) -> Result<()> {
        // Can't show cursor in minimal mode
        Err(TerminalError::FeatureNotAvailable)
    }

    fn set_raw_mode(&mut self) -> Result<()> {
        // Can't set raw mode in minimal mode
        Ok(())
    }

    fn reset_mode(&mut self) -> Result<()> {
        // Nothing to reset in minimal mode
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

pub fn create_minimal_terminal() -> Result<Box<dyn Terminal>> {
    Ok(Box::new(MinimalTerminal::new()?))
}
