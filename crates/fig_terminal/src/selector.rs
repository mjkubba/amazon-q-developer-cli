use std::time::Duration;

use crate::{Color, KeyEvent, Result, Terminal};

// A simple selector implementation that works across platforms
pub struct Selector {
    terminal: Box<dyn Terminal>,
    items: Vec<String>,
    selected_index: usize,
    prompt: String,
}

impl Selector {
    pub fn new(items: Vec<String>, prompt: &str) -> Result<Self> {
        let terminal = crate::create_terminal()?;
        
        Ok(Self {
            terminal,
            items,
            selected_index: 0,
            prompt: prompt.to_string(),
        })
    }
    
    pub fn run(&mut self) -> Result<Option<String>> {
        // Set up terminal
        self.terminal.set_raw_mode()?;
        self.terminal.clear_screen()?;
        self.terminal.hide_cursor()?;
        
        // Initial render
        self.render()?;
        
        // Input loop
        loop {
            match self.terminal.read_key(100)? {
                KeyEvent::Esc => {
                    // Clean up and exit with no selection
                    self.cleanup()?;
                    return Ok(None);
                },
                KeyEvent::Enter => {
                    // Clean up and return the selected item
                    let selected = self.items.get(self.selected_index).cloned();
                    self.cleanup()?;
                    return Ok(selected);
                },
                KeyEvent::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        self.render()?;
                    }
                },
                KeyEvent::Down => {
                    if self.selected_index < self.items.len() - 1 {
                        self.selected_index += 1;
                        self.render()?;
                    }
                },
                _ => {}
            }
        }
    }
    
    fn render(&mut self) -> Result<()> {
        // Clear the screen
        self.terminal.clear_screen()?;
        
        // Draw the prompt
        self.terminal.move_cursor(0, 0)?;
        self.terminal.write(&self.prompt)?;
        
        // Draw the items
        for (i, item) in self.items.iter().enumerate() {
            // Position for this item
            self.terminal.move_cursor(2, (i + 2) as u16)?;
            
            // Highlight if selected
            if i == self.selected_index {
                self.terminal.set_fg_color(Color::BrightGreen)?;
                self.terminal.write(&format!("> {}", item))?;
                self.terminal.reset_colors()?;
            } else {
                self.terminal.write(&format!("  {}", item))?;
            }
        }
        
        // Instructions
        self.terminal.move_cursor(0, (self.items.len() + 3) as u16)?;
        self.terminal.write("Use arrow keys to navigate, Enter to select, Esc to cancel")?;
        
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<()> {
        self.terminal.reset_raw_mode()?;
        self.terminal.clear_screen()?;
        self.terminal.show_cursor()?;
        self.terminal.move_cursor(0, 0)?;
        Ok(())
    }
}
