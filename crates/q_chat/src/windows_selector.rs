#![cfg(any(windows, feature = "termwiz-terminal"))]

use std::io;
use std::time::Duration;
use eyre::Result;
use termwiz::caps::Capabilities;
use termwiz::cell::{AttributeChange, CellAttributes, Color};
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::{Change, Position, Surface};
use termwiz::terminal::{new_terminal, Terminal as TermwizTerm};

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
        let terminal = new_terminal(Capabilities::new_from_env()?)?;
        let (width, height) = terminal.get_screen_size()?;
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
        // Set up terminal
        self.terminal.enter_alternate_screen()?;
        self.terminal.set_raw_mode(true)?;
        
        // Initial render
        self.render()?;
        
        // Input loop
        loop {
            match self.terminal.poll_input(Some(Duration::from_millis(100)))? {
                Some(InputEvent::Key(key)) => {
                    match key.key {
                        KeyCode::Escape => {
                            // Clean up and exit with no selection
                            self.cleanup()?;
                            return Ok(None);
                        },
                        KeyCode::Enter => {
                            // Clean up and return the selected item
                            let selected = self.items.get(self.selected_index).cloned();
                            self.cleanup()?;
                            return Ok(selected);
                        },
                        KeyCode::UpArrow => {
                            if self.selected_index > 0 {
                                self.selected_index -= 1;
                                self.render()?;
                            }
                        },
                        KeyCode::DownArrow => {
                            if self.selected_index < self.items.len() - 1 {
                                self.selected_index += 1;
                                self.render()?;
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
    
    fn render(&mut self) -> Result<()> {
        // Clear the screen
        self.surface.add_change(Change::ClearScreen(Color::Default));
        
        // Draw the prompt
        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(0),
        });
        self.surface.add_change(Change::Text(self.prompt.clone()));
        
        // Draw the items
        for (i, item) in self.items.iter().enumerate() {
            // Position for this item
            self.surface.add_change(Change::CursorPosition {
                x: Position::Absolute(2),
                y: Position::Absolute(i + 2),
            });
            
            // Highlight if selected
            if i == self.selected_index {
                self.surface.add_change(Change::Attribute(AttributeChange::Foreground(
                    Color::Rgb(0, 255, 0),
                )));
                self.surface.add_change(Change::Text(format!("> {}", item)));
                self.surface.add_change(Change::Attribute(AttributeChange::Foreground(
                    Color::Default,
                )));
            } else {
                self.surface.add_change(Change::Text(format!("  {}", item)));
            }
        }
        
        // Instructions
        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(self.items.len() + 3),
        });
        self.surface.add_change(Change::Text(
            "Use arrow keys to navigate, Enter to select, Esc to cancel".to_string(),
        ));
        
        // Render the changes
        self.terminal.render(&self.surface)?;
        
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<()> {
        self.terminal.set_raw_mode(false)?;
        self.terminal.exit_alternate_screen()?;
        Ok(())
    }
}
