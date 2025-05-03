use std::io;
use std::time::Duration;

use crate::{Result, Terminal, TerminalError};

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
        
        // Event loop
        let result = self.event_loop();
        
        // Clean up terminal
        self.terminal.reset_mode()?;
        self.terminal.show_cursor()?;
        
        result
    }
    
    fn render(&mut self) -> Result<()> {
        self.terminal.clear_screen()?;
        self.terminal.move_cursor(0, 0)?;
        
        // Render prompt
        self.terminal.write(&format!("{}\n\n", self.prompt))?;
        
        // Render items
        for (i, item) in self.items.iter().enumerate() {
            if i == self.selected_index {
                self.terminal.write(&format!("> {}\n", item))?;
            } else {
                self.terminal.write(&format!("  {}\n", item))?;
            }
        }
        
        self.terminal.flush()?;
        Ok(())
    }
    
    fn event_loop(&mut self) -> Result<Option<String>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "unix-terminal")] {
                self.unix_event_loop()
            } else if #[cfg(feature = "windows-terminal")] {
                self.windows_event_loop()
            } else {
                self.minimal_event_loop()
            }
        }
    }
    
    #[cfg(feature = "unix-terminal")]
    fn unix_event_loop(&mut self) -> Result<Option<String>> {
        use tuikit::event::{Event, Key};
        
        let term = unsafe { &*(self.terminal.as_ref() as *const _ as *const crate::unix::UnixTerminal) };
        
        loop {
            if let Ok(evt) = term.term.poll_event() {
                match evt {
                    Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                        return Ok(None);
                    }
                    Event::Key(Key::Char('\n')) => {
                        if self.selected_index < self.items.len() {
                            return Ok(Some(self.items[self.selected_index].clone()));
                        }
                    }
                    Event::Key(Key::Up) => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                            self.render()?;
                        }
                    }
                    Event::Key(Key::Down) => {
                        if self.selected_index < self.items.len() - 1 {
                            self.selected_index += 1;
                            self.render()?;
                        }
                    }
                    _ => {}
                }
            }
            
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    
    #[cfg(feature = "windows-terminal")]
    fn windows_event_loop(&mut self) -> Result<Option<String>> {
        use crossterm::event::{self, Event, KeyCode, KeyEvent};
        
        loop {
            if event::poll(Duration::from_millis(100)).map_err(TerminalError::Io)? {
                if let Event::Key(KeyEvent { code, .. }) = event::read().map_err(TerminalError::Io)? {
                    match code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(None);
                        }
                        KeyCode::Enter => {
                            if self.selected_index < self.items.len() {
                                return Ok(Some(self.items[self.selected_index].clone()));
                            }
                        }
                        KeyCode::Up => {
                            if self.selected_index > 0 {
                                self.selected_index -= 1;
                                self.render()?;
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_index < self.items.len() - 1 {
                                self.selected_index += 1;
                                self.render()?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    #[cfg(not(any(feature = "unix-terminal", feature = "windows-terminal")))]
    fn minimal_event_loop(&mut self) -> Result<Option<String>> {
        // In minimal mode, just print the items and ask for input
        println!("\n{}", self.prompt);
        
        for (i, item) in self.items.iter().enumerate() {
            println!("{}. {}", i + 1, item);
        }
        
        println!("\nEnter number (or q to quit): ");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(TerminalError::Io)?;
        
        let input = input.trim();
        
        if input == "q" {
            return Ok(None);
        }
        
        if let Ok(index) = input.parse::<usize>() {
            if index > 0 && index <= self.items.len() {
                return Ok(Some(self.items[index - 1].clone()));
            }
        }
        
        Ok(None)
    }
}
