use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event as CrosstermEvent, KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, ClearType},
};

use crate::error::{Result, TerminalError};

/// Represents a terminal key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Esc,
    Char(char),
    Ctrl(char),
    Alt(char),
    F(u8),
    Unknown,
}

/// Represents a key event
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub key: Key,
}

impl From<CrosstermKeyEvent> for KeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        let key = match event.code {
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Esc => Key::Esc,
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    Key::Ctrl(c)
                } else if event.modifiers.contains(KeyModifiers::ALT) {
                    Key::Alt(c)
                } else {
                    Key::Char(c)
                }
            }
            KeyCode::F(n) => Key::F(n),
            _ => Key::Unknown,
        };

        KeyEvent { key }
    }
}

/// A cross-platform terminal implementation
pub struct Term {
    raw_mode: bool,
    alternate_screen: bool,
}

impl Term {
    /// Create a new terminal instance
    pub fn new() -> Self {
        Self {
            raw_mode: false,
            alternate_screen: false,
        }
    }

    /// Enter raw mode
    pub fn enter_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode {
            terminal::enable_raw_mode()?;
            self.raw_mode = true;
        }
        Ok(())
    }

    /// Exit raw mode
    pub fn exit_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode {
            terminal::disable_raw_mode()?;
            self.raw_mode = false;
        }
        Ok(())
    }

    /// Enter alternate screen
    pub fn enter_alternate_screen(&mut self) -> Result<()> {
        if !self.alternate_screen {
            execute!(io::stdout(), terminal::EnterAlternateScreen)?;
            self.alternate_screen = true;
        }
        Ok(())
    }

    /// Exit alternate screen
    pub fn exit_alternate_screen(&mut self) -> Result<()> {
        if self.alternate_screen {
            execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }
        Ok(())
    }

    /// Get terminal size
    pub fn size(&self) -> Result<(u16, u16)> {
        let (cols, rows) = terminal::size()?;
        Ok((cols, rows))
    }

    /// Clear the screen
    pub fn clear_screen(&self) -> Result<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;
        Ok(())
    }

    /// Move cursor to position
    pub fn move_cursor_to(&self, x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), cursor::MoveTo(x, y))?;
        Ok(())
    }

    /// Hide cursor
    pub fn hide_cursor(&self) -> Result<()> {
        execute!(io::stdout(), cursor::Hide)?;
        Ok(())
    }

    /// Show cursor
    pub fn show_cursor(&self) -> Result<()> {
        execute!(io::stdout(), cursor::Show)?;
        Ok(())
    }

    /// Write text at position with color
    pub fn write_at(&self, x: u16, y: u16, text: &str, fg: Option<Color>, bg: Option<Color>) -> Result<()> {
        let mut stdout = io::stdout();
        queue!(stdout, cursor::MoveTo(x, y))?;

        match (fg, bg) {
            (Some(fg), Some(bg)) => queue!(stdout, style::PrintStyledContent(text.with(fg).on(bg)))?,
            (Some(fg), None) => queue!(stdout, style::PrintStyledContent(text.with(fg)))?,
            (None, Some(bg)) => queue!(stdout, style::PrintStyledContent(text.on(bg)))?,
            (None, None) => queue!(stdout, style::Print(text))?,
        }

        stdout.flush()?;
        Ok(())
    }

    /// Poll for a key event with timeout
    pub fn poll_key_event(&self, timeout: Duration) -> Result<Option<KeyEvent>> {
        if event::poll(timeout)? {
            if let CrosstermEvent::Key(key_event) = event::read()? {
                return Ok(Some(key_event.into()));
            }
        }
        Ok(None)
    }

    /// Read a key event (blocking)
    pub fn read_key_event(&self) -> Result<KeyEvent> {
        loop {
            if let CrosstermEvent::Key(key_event) = event::read()? {
                return Ok(key_event.into());
            }
        }
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        // Clean up terminal state when dropped
        let _ = self.exit_raw_mode();
        let _ = self.exit_alternate_screen();
    }
}

impl Default for Term {
    fn default() -> Self {
        Self::new()
    }
}