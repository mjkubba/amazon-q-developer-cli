use std::io;
use std::time::Duration;
use termwiz::caps::Capabilities;
use termwiz::cell::{AttributeChange, CellAttributes, Color as TermwizColor};
use termwiz::input::{InputEvent, KeyCode, KeyEvent as TermwizKeyEvent, Modifiers};
use termwiz::surface::{Change, Position, Surface};
use termwiz::terminal::{new_terminal, Terminal as TermwizTerm};

use crate::{Color, KeyEvent, Result, Terminal, TerminalError};

pub struct TermwizTerminal {
    terminal: Box<dyn TermwizTerm>,
    surface: Surface,
    width: usize,
    height: usize,
}

impl TermwizTerminal {
    pub fn new() -> Result<Self> {
        let terminal = new_terminal(Capabilities::new_from_env()?)?;
        let (width, height) = terminal.get_screen_size()?;
        let surface = Surface::new(width, height);
        
        Ok(Self {
            terminal,
            surface,
            width,
            height,
        })
    }
    
    fn convert_color(color: Color) -> TermwizColor {
        match color {
            Color::Default => TermwizColor::Default,
            Color::Black => TermwizColor::PaletteIndex(0),
            Color::Red => TermwizColor::PaletteIndex(1),
            Color::Green => TermwizColor::PaletteIndex(2),
            Color::Yellow => TermwizColor::PaletteIndex(3),
            Color::Blue => TermwizColor::PaletteIndex(4),
            Color::Magenta => TermwizColor::PaletteIndex(5),
            Color::Cyan => TermwizColor::PaletteIndex(6),
            Color::White => TermwizColor::PaletteIndex(7),
            Color::BrightBlack => TermwizColor::PaletteIndex(8),
            Color::BrightRed => TermwizColor::PaletteIndex(9),
            Color::BrightGreen => TermwizColor::PaletteIndex(10),
            Color::BrightYellow => TermwizColor::PaletteIndex(11),
            Color::BrightBlue => TermwizColor::PaletteIndex(12),
            Color::BrightMagenta => TermwizColor::PaletteIndex(13),
            Color::BrightCyan => TermwizColor::PaletteIndex(14),
            Color::BrightWhite => TermwizColor::PaletteIndex(15),
            Color::Rgb(r, g, b) => TermwizColor::Rgb(r, g, b),
        }
    }
    
    fn convert_key_event(key_event: TermwizKeyEvent) -> KeyEvent {
        match key_event.key {
            KeyCode::Char(c) => {
                if key_event.modifiers.contains(Modifiers::CTRL) {
                    KeyEvent::Ctrl(c)
                } else if key_event.modifiers.contains(Modifiers::ALT) {
                    KeyEvent::Alt(c)
                } else {
                    KeyEvent::Char(c)
                }
            },
            KeyCode::Function(n) => KeyEvent::F(n as u8),
            KeyCode::UpArrow => KeyEvent::Up,
            KeyCode::DownArrow => KeyEvent::Down,
            KeyCode::LeftArrow => KeyEvent::Left,
            KeyCode::RightArrow => KeyEvent::Right,
            KeyCode::Home => KeyEvent::Home,
            KeyCode::End => KeyEvent::End,
            KeyCode::PageUp => KeyEvent::PageUp,
            KeyCode::PageDown => KeyEvent::PageDown,
            KeyCode::Backspace => KeyEvent::Backspace,
            KeyCode::Delete => KeyEvent::Delete,
            KeyCode::Insert => KeyEvent::Insert,
            KeyCode::Enter => KeyEvent::Enter,
            KeyCode::Tab => KeyEvent::Tab,
            KeyCode::BackTab => KeyEvent::BackTab,
            KeyCode::Escape => KeyEvent::Esc,
            _ => KeyEvent::Unknown,
        }
    }
}

impl Terminal for TermwizTerminal {
    fn get_size(&self) -> Result<(u16, u16)> {
        Ok((self.width as u16, self.height as u16))
    }
    
    fn clear_screen(&mut self) -> Result<()> {
        self.surface.add_change(Change::ClearScreen(TermwizColor::Default));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn move_cursor(&mut self, x: u16, y: u16) -> Result<()> {
        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(x as usize),
            y: Position::Absolute(y as usize),
        });
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn hide_cursor(&mut self) -> Result<()> {
        self.surface.add_change(Change::CursorVisibility(false));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn show_cursor(&mut self) -> Result<()> {
        self.surface.add_change(Change::CursorVisibility(true));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn set_raw_mode(&mut self) -> Result<()> {
        self.terminal.set_raw_mode(true)?;
        Ok(())
    }
    
    fn reset_raw_mode(&mut self) -> Result<()> {
        self.terminal.set_raw_mode(false)?;
        Ok(())
    }
    
    fn write(&mut self, text: &str) -> Result<()> {
        self.surface.add_change(Change::Text(text.to_string()));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn set_fg_color(&mut self, color: Color) -> Result<()> {
        self.surface.add_change(Change::Attribute(AttributeChange::Foreground(
            Self::convert_color(color),
        )));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn set_bg_color(&mut self, color: Color) -> Result<()> {
        self.surface.add_change(Change::Attribute(AttributeChange::Background(
            Self::convert_color(color),
        )));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn reset_colors(&mut self) -> Result<()> {
        self.surface.add_change(Change::Attribute(AttributeChange::Foreground(
            TermwizColor::Default,
        )));
        self.surface.add_change(Change::Attribute(AttributeChange::Background(
            TermwizColor::Default,
        )));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn read_key(&mut self, timeout_ms: u64) -> Result<KeyEvent> {
        // Poll for input with a timeout
        match self.terminal.poll_input(Some(Duration::from_millis(timeout_ms)))? {
            None => Ok(KeyEvent::Unknown),
            Some(input_event) => match input_event {
                InputEvent::Key(key_event) => Ok(Self::convert_key_event(key_event)),
                _ => Ok(KeyEvent::Unknown),
            },
        }
    }
}
