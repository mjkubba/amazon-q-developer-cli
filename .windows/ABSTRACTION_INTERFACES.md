# Platform Abstraction Interfaces

This document outlines the key abstraction interfaces needed to make the Amazon Q CLI cross-platform compatible.

## Terminal Handling Abstraction

```rust
/// Trait for platform-agnostic terminal handling
pub trait TerminalHandler {
    /// Get the current terminal width
    fn terminal_width(&self) -> Option<usize>;
    
    /// Clear the current line
    fn clear_line(&mut self) -> Result<()>;
    
    /// Move cursor to a specific column
    fn move_cursor_to_column(&mut self, column: u16) -> Result<()>;
    
    /// Show the cursor
    fn show_cursor(&mut self) -> Result<()>;
    
    /// Hide the cursor
    fn hide_cursor(&mut self) -> Result<()>;
    
    /// Set foreground color
    fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()>;
    
    /// Reset colors to default
    fn reset_color(&mut self) -> Result<()>;
    
    /// Set text attribute
    fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()>;
    
    /// Reset all attributes
    fn reset_attributes(&mut self) -> Result<()>;
    
    /// Print text to the terminal
    fn print(&mut self, text: &str) -> Result<()>;
    
    /// Flush output
    fn flush(&mut self) -> Result<()>;
}
```

## Input Handling Abstraction

```rust
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
    
    /// Set up fuzzy search functionality (replacement for skim on Windows)
    fn setup_fuzzy_search(&mut self, items: Vec<String>) -> Result<()>;
    
    /// Get fuzzy search result
    fn get_fuzzy_search_result(&mut self) -> Result<Option<Vec<String>>>;
}
```

## File System Abstraction

The project already has a good `Fs` abstraction in the `platform` module, but it may need extensions:

```rust
/// Extensions to the existing Fs trait for better cross-platform support
pub trait FsExt {
    /// Convert a path to the platform-specific format
    fn platform_path(&self, path: &str) -> String;
    
    /// Get the platform-specific path separator
    fn path_separator(&self) -> &str;
    
    /// Check if a path is absolute in a platform-agnostic way
    fn is_absolute_path(&self, path: &str) -> bool;
    
    /// Normalize a path for the current platform
    fn normalize_path(&self, path: &str) -> String;
}
```

## Process Execution Abstraction

```rust
/// Trait for platform-agnostic process execution
pub trait ProcessExecutor {
    /// Execute a command and return the output
    fn execute_command(&self, command: &str) -> Result<(String, String, i32)>;
    
    /// Execute a command in the background
    fn execute_background_command(&self, command: &str) -> Result<u32>;
    
    /// Check if a process is running
    fn is_process_running(&self, pid: u32) -> Result<bool>;
    
    /// Kill a process
    fn kill_process(&self, pid: u32) -> Result<()>;
    
    /// Get the platform-specific shell
    fn shell_command(&self) -> &str;
    
    /// Get the platform-specific shell arguments for executing a command
    fn shell_args(&self, command: &str) -> Vec<String>;
}
```

## Signal Handling Abstraction

```rust
/// Trait for platform-agnostic signal handling
pub trait SignalHandler {
    /// Set up a handler for Ctrl+C
    fn handle_ctrl_c<F>(&self, handler: F) -> Result<()>
    where
        F: FnMut() + Send + 'static;
    
    /// Set up a handler for terminal resize events
    fn handle_resize<F>(&self, handler: F) -> Result<()>
    where
        F: FnMut(u16, u16) + Send + 'static;
    
    /// Check if a signal has been received
    fn check_signal(&self) -> Result<Option<Signal>>;
}
```

## Notification Abstraction

```rust
/// Trait for platform-agnostic notifications
pub trait NotificationHandler {
    /// Play a notification sound
    fn play_notification_sound(&self, urgent: bool) -> Result<()>;
    
    /// Show a desktop notification
    fn show_notification(&self, title: &str, message: &str) -> Result<()>;
}
```

These abstractions will form the foundation for separating platform-specific code from the core agent loop logic.
