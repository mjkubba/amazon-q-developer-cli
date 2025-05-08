#![cfg(any(windows, feature = "termwiz-terminal"))]

use std::time::Duration;
use eyre::Result;
use termwiz::caps::Capabilities;
use termwiz::cell::AttributeChange;
use termwiz::input::{InputEvent, KeyCode};
use termwiz::surface::{Change, Position, Surface};
use termwiz::terminal::{new_terminal, Terminal as TermwizTerm, ScreenSize};

pub struct WindowsSelector {
    terminal: Box<dyn TermwizTerm>,
    surface: Surface,
    items: Vec<String>,
    selected_index: usize,
    prompt: String,
    width: usize,
    height: usize,
}

impl WindowsSelector {
    pub fn new(items: Vec<String>, prompt: &str) -> Result<Self> {
        let mut terminal = Box::new(new_terminal(Capabilities::new_from_env()?)?);
        let screen_size = terminal.get_screen_size()?;
        let width = screen_size.cols;
        let height = screen_size.rows;
        let surface = Surface::new(width, height);
        
        Ok(Self {
            terminal,
            surface,
            items,
            selected_index: 0,
            prompt: prompt.to_string(),
            width,
            height,
        })
    }

    pub fn run(&mut self) -> Result<Option<String>> {
        // Set raw mode
        self.terminal.enter_alternate_screen()?;
        
        // Render initial state
        self.render()?;
        
        loop {
            if let Some(input) = self.terminal.poll_input(Some(Duration::from_millis(100)))? {
                match input {
                    InputEvent::Key(key) => {
                        match key.key {
                            KeyCode::Escape => {
                                self.cleanup()?;
                                return Ok(None);
                            }
                            KeyCode::Enter => {
                                let selected = self.items.get(self.selected_index).cloned();
                                self.cleanup()?;
                                return Ok(selected);
                            }
                            KeyCode::UpArrow => {
                                if self.selected_index > 0 {
                                    self.selected_index -= 1;
                                }
                                self.render()?;
                            }
                            KeyCode::DownArrow => {
                                if self.selected_index < self.items.len() - 1 {
                                    self.selected_index += 1;
                                }
                                self.render()?;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        // Clear the surface manually
        for y in 0..self.height {
            for x in 0..self.width {
                self.surface.set_cell(x, y, Default::default());
            }
        }
        
        // Render prompt
        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(0),
        });
        self.surface.add_change(Change::Text(self.prompt.clone()));
        
        // Render items
        for (i, item) in self.items.iter().enumerate().take(self.height - 2) {
            self.surface.add_change(Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(i + 1),
            });
            
            if i == self.selected_index {
                self.surface.add_change(Change::Attribute(AttributeChange::Reverse(true)));
            }
            
            self.surface.add_change(Change::Text(item.clone()));
            
            if i == self.selected_index {
                self.surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
            }
        }
        
        // Render changes
        let (_, changes) = self.surface.get_changes(0);
        self.terminal.render(&changes)?;
        
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        self.terminal.exit_alternate_screen()?;
        Ok(())
    }
}
