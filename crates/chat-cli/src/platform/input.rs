use eyre::Result;
use std::path::PathBuf;

/// Key event representation for platform-agnostic input handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Ctrl(char),
    Alt(char),
    F(u8),
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Insert,
    Esc,
    Tab,
    BackTab,
    Enter,
    Unknown,
}

/// Trait for platform-agnostic input handling
pub trait InputHandler {
    /// Read a line of input with an optional prompt
    fn read_line(&mut self, prompt: Option<&str>) -> Result<Option<String>>;
    
    /// Check if input is available
    fn has_input(&self) -> Result<bool>;
    
    /// Enable raw mode (character-by-character input)
    fn enable_raw_mode(&mut self) -> Result<()>;
    
    /// Disable raw mode
    fn disable_raw_mode(&mut self) -> Result<()>;
    
    /// Read a key press
    fn read_key(&mut self) -> Result<Option<KeyEvent>>;
    
    /// Set up fuzzy search functionality
    fn setup_fuzzy_search(&mut self, items: Vec<String>) -> Result<()>;
    
    /// Get fuzzy search result
    fn get_fuzzy_search_result(&mut self) -> Result<Option<Vec<String>>>;
    
    /// Add context files for fuzzy search
    fn add_context_files(&mut self, files: Vec<(PathBuf, String)>) -> Result<()>;
}

#[cfg(unix)]
pub mod unix {
    use super::*;
    use eyre::eyre;
    use rustyline::{
        error::ReadlineError,
        Editor,
        DefaultEditor,
    };
    
    /// Unix implementation of InputHandler using rustyline
    pub struct UnixInputHandler {
        editor: DefaultEditor,
        context_files: Vec<(PathBuf, String)>,
    }
    
    impl UnixInputHandler {
        pub fn new() -> Self {
            let mut editor = DefaultEditor::new().expect("Failed to create editor");
            editor.load_history(".q_history").ok(); // Ignore errors if history doesn't exist
            
            Self {
                editor,
                context_files: Vec::new(),
            }
        }
    }
    
    impl InputHandler for UnixInputHandler {
        fn read_line(&mut self, prompt: Option<&str>) -> Result<Option<String>> {
            let prompt = prompt.unwrap_or("> ");
            match self.editor.readline(prompt) {
                Ok(line) => {
                    self.editor.add_history_entry(&line)?;
                    self.editor.save_history(".q_history").ok();
                    Ok(Some(line))
                },
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Ok(None),
                Err(err) => Err(eyre!("Error reading line: {}", err)),
            }
        }
        
        fn has_input(&self) -> Result<bool> {
            // This is a simplified implementation
            Ok(true)
        }
        
        fn enable_raw_mode(&mut self) -> Result<()> {
            // Rustyline handles this internally
            Ok(())
        }
        
        fn disable_raw_mode(&mut self) -> Result<()> {
            // Rustyline handles this internally
            Ok(())
        }
        
        fn read_key(&mut self) -> Result<Option<KeyEvent>> {
            // This would need a more complex implementation using crossterm or similar
            // For now, we'll return a placeholder
            Err(eyre!("Not implemented"))
        }
        
        fn setup_fuzzy_search(&mut self, _items: Vec<String>) -> Result<()> {
            // This would need to be implemented with skim
            // For now, we'll return a placeholder
            Err(eyre!("Not implemented"))
        }
        
        fn get_fuzzy_search_result(&mut self) -> Result<Option<Vec<String>>> {
            // This would need to be implemented with skim
            // For now, we'll return a placeholder
            Err(eyre!("Not implemented"))
        }
        
        fn add_context_files(&mut self, files: Vec<(PathBuf, String)>) -> Result<()> {
            self.context_files = files;
            Ok(())
        }
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use eyre::eyre;
    use rustyline::{
        error::ReadlineError,
        DefaultEditor,
    };
    
    /// Windows implementation of InputHandler using rustyline
    /// 
    /// Note: This is a simplified implementation that doesn't include fuzzy search
    /// functionality, which will need to be implemented separately for Windows.
    pub struct WindowsInputHandler {
        editor: DefaultEditor,
        context_files: Vec<(PathBuf, String)>,
    }
    
    impl WindowsInputHandler {
        pub fn new() -> Self {
            let mut editor = DefaultEditor::new().expect("Failed to create editor");
            editor.load_history(".q_history").ok(); // Ignore errors if history doesn't exist
            
            Self {
                editor,
                context_files: Vec::new(),
            }
        }
    }
    
    impl InputHandler for WindowsInputHandler {
        fn read_line(&mut self, prompt: Option<&str>) -> Result<Option<String>> {
            let prompt = prompt.unwrap_or("> ");
            match self.editor.readline(prompt) {
                Ok(line) => {
                    self.editor.add_history_entry(&line)?;
                    self.editor.save_history(".q_history").ok();
                    Ok(Some(line))
                },
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Ok(None),
                Err(err) => Err(eyre!("Error reading line: {}", err)),
            }
        }
        
        fn has_input(&self) -> Result<bool> {
            // This is a simplified implementation
            Ok(true)
        }
        
        fn enable_raw_mode(&mut self) -> Result<()> {
            // Rustyline handles this internally
            Ok(())
        }
        
        fn disable_raw_mode(&mut self) -> Result<()> {
            // Rustyline handles this internally
            Ok(())
        }
        
        fn read_key(&mut self) -> Result<Option<KeyEvent>> {
            // This would need a more complex implementation
            // For now, we'll return a placeholder
            Err(eyre!("Not implemented"))
        }
        
        fn setup_fuzzy_search(&mut self, _items: Vec<String>) -> Result<()> {
            // Windows implementation will need a different approach than skim
            Err(eyre!("Fuzzy search not implemented for Windows yet"))
        }
        
        fn get_fuzzy_search_result(&mut self) -> Result<Option<Vec<String>>> {
            // Windows implementation will need a different approach than skim
            Err(eyre!("Fuzzy search not implemented for Windows yet"))
        }
        
        fn add_context_files(&mut self, files: Vec<(PathBuf, String)>) -> Result<()> {
            self.context_files = files;
            Ok(())
        }
    }
}

/// Factory function to create the appropriate input handler for the current platform
pub fn create_input_handler() -> Box<dyn InputHandler> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixInputHandler::new())
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsInputHandler::new())
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        Box::new(unix::UnixInputHandler::new())
    }
}
