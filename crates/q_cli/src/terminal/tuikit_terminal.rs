#![cfg(unix)]

use anyhow::{Context, Result};
use std::time::Duration;
use tuikit::prelude::*;
use tuikit::term::Term;

use super::{Color, Input, Terminal, TerminalSize};

pub struct TuikitTerminal {
    term: Term,
}

impl Terminal for TuikitTerminal {
    fn init() -> Result<Self> {
        let term = Term::new()
            .context("Failed to initialize tuikit terminal")?;
        
        Ok(Self { term })
    }
    
    fn cleanup(&mut self) -> Result<()> {
        self.term.clear()?;
        self.term.present()?;
        Ok(())
    }
    
    fn read_input(&mut self) -> Result<Input> {
        // Poll for input with a timeout
        match self.term.poll_event(Duration::from_millis(100))? {
            Some(event) => Ok(convert_event(event)),
            None => Ok(Input::Unknown),
        }
    }
    
    fn write_output(&mut self, output: &str) -> Result<()> {
        let (x, y) = self.term.cursor_pos()?;
        self.term.print(x, y, output)?;
        self.term.present()?;
        Ok(())
    }
    
    fn set_color(&mut self, fg: Color, bg: Color) -> Result<()> {
        let attr = Attr {
            fg: convert_color(fg),
            bg: convert_color(bg),
            ..Attr::default()
        };
        
        self.term.set_attr(attr)?;
        Ok(())
    }
    
    fn reset_color(&mut self) -> Result<()> {
        self.term.set_attr(Attr::default())?;
        Ok(())
    }
    
    fn clear_screen(&mut self) -> Result<()> {
        self.term.clear()?;
        self.term.present()?;
        Ok(())
    }
    
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        self.term.move_cursor(x as i32, y as i32)?;
        self.term.present()?;
        Ok(())
    }
    
    fn hide_cursor(&mut self) -> Result<()> {
        self.term.hide_cursor()?;
        self.term.present()?;
        Ok(())
    }
    
    fn show_cursor(&mut self) -> Result<()> {
        self.term.show_cursor()?;
        self.term.present()?;
        Ok(())
    }
    
    fn get_size(&self) -> Result<TerminalSize> {
        let (width, height) = self.term.term_size()
            .context("Failed to get terminal size")?;
        
        Ok(TerminalSize {
            width: width as u16,
            height: height as u16,
        })
    }
}

// Convert our Color enum to tuikit's Color
fn convert_color(color: Color) -> tuikit::attr::Color {
    match color {
        Color::Reset => tuikit::attr::Color::Default,
        Color::Black => tuikit::attr::Color::Black,
        Color::Red => tuikit::attr::Color::Red,
        Color::Green => tuikit::attr::Color::Green,
        Color::Yellow => tuikit::attr::Color::Yellow,
        Color::Blue => tuikit::attr::Color::Blue,
        Color::Magenta => tuikit::attr::Color::Magenta,
        Color::Cyan => tuikit::attr::Color::Cyan,
        Color::White => tuikit::attr::Color::White,
        Color::BrightBlack => tuikit::attr::Color::LightBlack,
        Color::BrightRed => tuikit::attr::Color::LightRed,
        Color::BrightGreen => tuikit::attr::Color::LightGreen,
        Color::BrightYellow => tuikit::attr::Color::LightYellow,
        Color::BrightBlue => tuikit::attr::Color::LightBlue,
        Color::BrightMagenta => tuikit::attr::Color::LightMagenta,
        Color::BrightCyan => tuikit::attr::Color::LightCyan,
        Color::BrightWhite => tuikit::attr::Color::LightWhite,
        Color::Rgb(r, g, b) => tuikit::attr::Color::Rgb(r, g, b),
    }
}

// Convert tuikit's Event to our Input enum
fn convert_event(event: Event) -> Input {
    match event {
        Event::Key(key) => match key {
            Key::Char(c) => Input::Char(c),
            Key::Ctrl(c) => Input::Ctrl(c),
            Key::Alt(c) => Input::Alt(c),
            Key::F(n) => Input::F(n),
            Key::Up => Input::Up,
            Key::Down => Input::Down,
            Key::Left => Input::Left,
            Key::Right => Input::Right,
            Key::Home => Input::Home,
            Key::End => Input::End,
            Key::PageUp => Input::PageUp,
            Key::PageDown => Input::PageDown,
            Key::Backspace => Input::Backspace,
            Key::Delete => Input::Delete,
            Key::Insert => Input::Insert,
            Key::Enter => Input::Enter,
            Key::Tab => Input::Tab,
            Key::BackTab => Input::BackTab,
            Key::Esc => Input::Esc,
            _ => Input::Unknown,
        },
        _ => Input::Unknown,
    }
}
