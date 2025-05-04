use anyhow::{Context, Result};
use std::time::Duration;
use termwiz::caps::Capabilities;
use termwiz::cell::{AttributeChange, CellAttributes, Color as TermwizColor};
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::{Change, Position, Surface};
use termwiz::terminal::{new_terminal, Terminal as TermwizTerm};

use super::{Color, Input, Terminal, TerminalSize};

pub struct TermwizTerminal {
    terminal: Box<dyn TermwizTerm>,
    surface: Surface,
    width: usize,
    height: usize,
}

impl Terminal for TermwizTerminal {
    fn init() -> Result<Self> {
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
    
    fn cleanup(&mut self) -> Result<()> {
        self.terminal.set_raw_mode(false)?;
        self.terminal.end()?;
        Ok(())
    }
    
    fn read_input(&mut self) -> Result<Input> {
        // Poll for input with a timeout
        match self.terminal.poll_input(Some(Duration::from_millis(100)))? {
            None => Ok(Input::Unknown),
            Some(input_event) => match input_event {
                InputEvent::Key(key_event) => Ok(convert_key_event(key_event)),
                _ => Ok(Input::Unknown),
            },
        }
    }
    
    fn write_output(&mut self, output: &str) -> Result<()> {
        self.surface.add_change(Change::Text(output.to_string()));
        self.terminal.render(&self.surface)?;
        Ok(())
    }
    
    fn set_color(&mut self, fg: Color, bg: Color) -> Result<()> {
        let mut attrs = CellAttributes::default();
        
        // Set foreground color
        attrs.set_foreground(convert_color(fg));
        
        // Set background color
        attrs.set_background(convert_color(bg));
        
        self.surface.add_change(Change::Attribute(AttributeChange::Foreground(convert_color(fg))));
        self.surface.add_change(Change::Attribute(AttributeChange::Background(convert_color(bg))));
        
        Ok(())
    }
    
    fn reset_color(&mut self) -> Result<()> {
        self.surface.add_change(Change::Attribute(AttributeChange::Foreground(
            TermwizColor::Default,
        )));
        self.surface.add_change(Change::Attribute(AttributeChange::Background(
            TermwizColor::Default,
        )));
        Ok(())
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
    
    fn get_size(&self) -> Result<TerminalSize> {
        let (width, height) = self.terminal.get_screen_size()
            .context("Failed to get terminal size")?;
        
        Ok(TerminalSize {
            width: width as u16,
            height: height as u16,
        })
    }
}

// Convert our Color enum to termwiz's Color
fn convert_color(color: Color) -> TermwizColor {
    match color {
        Color::Reset => TermwizColor::Default,
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

// Convert termwiz's KeyEvent to our Input enum
fn convert_key_event(key_event: KeyEvent) -> Input {
    match key_event.key {
        KeyCode::Char(c) => {
            if key_event.modifiers.contains(Modifiers::CTRL) {
                Input::Ctrl(c)
            } else if key_event.modifiers.contains(Modifiers::ALT) {
                Input::Alt(c)
            } else {
                Input::Char(c)
            }
        },
        KeyCode::Function(n) => Input::F(n as u8),
        KeyCode::UpArrow => Input::Up,
        KeyCode::DownArrow => Input::Down,
        KeyCode::LeftArrow => Input::Left,
        KeyCode::RightArrow => Input::Right,
        KeyCode::Home => Input::Home,
        KeyCode::End => Input::End,
        KeyCode::PageUp => Input::PageUp,
        KeyCode::PageDown => Input::PageDown,
        KeyCode::Backspace => Input::Backspace,
        KeyCode::Delete => Input::Delete,
        KeyCode::Insert => Input::Insert,
        KeyCode::Enter => Input::Enter,
        KeyCode::Tab => Input::Tab,
        KeyCode::BackTab => Input::BackTab,
        KeyCode::Escape => Input::Esc,
        _ => Input::Unknown,
    }
}
