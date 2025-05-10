# Windows Port Implementation Details

This document provides detailed information about the changes made to the `chat-cli` crate to support Windows compatibility. It outlines the specific differences between the main branch and the Windows port implementation.

## Changes Overview

The Windows port implementation focused on creating platform abstractions for terminal handling, input processing, process execution, and signal handling. These abstractions allow the core agent loop to work across different platforms, including Windows.

## Directory Structure Changes

New files added to the `crates/chat-cli/src/platform/` directory:

```
crates/chat-cli/src/platform/
├── diagnostics.rs (modified)
├── env.rs
├── fs.rs
├── input.rs (new)
├── mod.rs (modified)
├── os.rs
├── process.rs (new)
├── providers.rs
├── signal.rs (new)
├── sysinfo.rs
└── terminal.rs (new)
```

## Detailed Changes

### 1. Terminal Handling (`platform/terminal.rs`)

**Added a new file** that implements platform-agnostic terminal handling:

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

Implemented both Unix and Windows versions using crossterm:

```rust
#[cfg(unix)]
pub mod unix {
    // Unix implementation using crossterm
}

#[cfg(windows)]
pub mod windows {
    // Windows implementation using crossterm
}
```

The key difference in the Windows implementation is the handling of cursor positioning to avoid stack overflow:

```rust
// Unix implementation (simplified)
fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
    self.writer.queue(cursor::MoveToColumn(column))?;
    Ok(())
}

// Windows implementation (simplified)
fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
    // Use a safer approach that doesn't rely on DetectCursorPos which can cause recursion
    self.writer.queue(cursor::MoveToColumn(column))?;
    Ok(())
}
```

### 2. Input Handling (`platform/input.rs`)

**Added a new file** that implements platform-agnostic input handling:

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
    
    // Additional methods...
}
```

Implemented both Unix and Windows versions:

```rust
#[cfg(unix)]
pub mod unix {
    // Unix implementation using rustyline
}

#[cfg(windows)]
pub mod windows {
    // Windows implementation using rustyline with Windows-specific handling
}
```

### 3. Process Execution (`platform/process.rs`)

**Added a new file** that implements platform-agnostic process execution:

```rust
/// Trait for platform-agnostic process execution
pub trait ProcessExecutor {
    /// Execute a command and return the output
    fn execute_command(&self, command: &str) -> Result<Output>;
    
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

Implemented both Unix and Windows versions:

```rust
#[cfg(unix)]
pub mod unix {
    // Unix implementation using std::process
}

#[cfg(windows)]
pub mod windows {
    // Windows implementation using std::process with Windows-specific handling
}
```

Key differences in the Windows implementation:

```rust
// Unix shell command
fn shell_command(&self) -> &str {
    "bash"
}

// Windows shell command
fn shell_command(&self) -> &str {
    "cmd"
}

// Unix shell arguments
fn shell_args(&self, command: &str) -> Vec<String> {
    vec!["-c".to_string(), command.to_string()]
}

// Windows shell arguments
fn shell_args(&self, command: &str) -> Vec<String> {
    vec!["/C".to_string(), command.to_string()]
}
```

### 4. Signal Handling (`platform/signal.rs`)

**Added a new file** that implements platform-agnostic signal handling:

```rust
/// Trait for platform-agnostic signal handling
pub trait SignalHandler {
    /// Wait for a signal
    fn wait_for_signal(&self) -> Pin<Box<dyn Future<Output = Result<Signal>> + Send>>;
}
```

Implemented both Unix and Windows versions:

```rust
#[cfg(unix)]
pub mod unix {
    // Unix implementation using tokio::signal::unix
}

#[cfg(windows)]
pub mod windows {
    // Windows implementation using tokio::signal::ctrl_c
}
```

### 5. Platform Module (`platform/mod.rs`)

**Modified** to expose the new abstractions:

```rust
pub mod context;
pub mod diagnostics;
pub mod terminal;
pub mod input;
pub mod process;
pub mod signal;

pub use terminal::{
    TerminalHandler,
    TerminalColor,
    TerminalAttribute,
    create_terminal_handler,
};

pub use input::{
    InputHandler,
    KeyEvent,
    create_input_handler,
};

pub use process::{
    ProcessExecutor,
    create_process_executor,
};

pub use signal::{
    Signal,
    SignalHandler,
    create_signal_handler,
};
```

### 6. Build Configuration

**Added** `.cargo/config.toml` to increase stack size for Windows builds:

```toml
[target.'cfg(windows)']
rustflags = ["-C", "link-args=/STACK:8388608"]  # Increase stack size to 8MB for Windows
```

**Modified** `fig_proto/build.rs` to handle protoc on Windows:

```rust
// On Windows, use the locally installed protoc instead of downloading
if cfg!(target_os = "windows") {
    // First, check for protoc in the specific location ../protobuf/bin/protoc.exe
    let protoc_path = std::path::PathBuf::from("../../../protobuf/bin/protoc.exe");
    if protoc_path.exists() {
        println!("cargo:warning=Using protoc from ../protobuf/bin/protoc.exe");
        std::env::set_var("PROTOC", protoc_path);
        return;
    }
    
    // Next, check if protoc is available in PATH
    if let Ok(output) = Command::new("protoc").arg("--version").output() {
        if output.status.success() {
            // Use the locally installed protoc
            let protoc_path = which::which("protoc").expect("protoc should be in PATH");
            println!("cargo:warning=Using locally installed protoc at: {}", protoc_path.display());
            std::env::set_var("PROTOC", protoc_path);
            return;
        }
    }
    
    // If we can't find protoc, print a more helpful error message
    println!("cargo:warning=No protoc found in ../protobuf/bin/protoc.exe or PATH.");
    println!("cargo:warning=Please ensure protoc is installed and available.");
    
    // Try to use the system protoc as a last resort
    println!("cargo:warning=Setting PROTOC_NO_VENDOR=1 to use system protoc.");
    std::env::set_var("PROTOC_NO_VENDOR", "1");
    return;
}
```

### 7. Logging Fixes

**Modified** `cli/mod.rs` to include logging functions:

```rust
use crate::logging::{
    LogArgs,
    initialize_logging,
};
```

## How to Reproduce These Changes

To reproduce these changes on another system:

1. **Set up the development environment**:
   - Install Rust for Windows
   - Install Visual Studio Build Tools with C++ support
   - Install Protocol Buffers compiler (protoc)

2. **Create the platform abstractions**:
   - Create `platform/terminal.rs` with the `TerminalHandler` trait and implementations
   - Create `platform/input.rs` with the `InputHandler` trait and implementations
   - Create `platform/process.rs` with the `ProcessExecutor` trait and implementations
   - Create `platform/signal.rs` with the `SignalHandler` trait and implementations
   - Update `platform/mod.rs` to expose these abstractions

3. **Configure the build**:
   - Create `.cargo/config.toml` to increase stack size for Windows builds
   - Modify `fig_proto/build.rs` to handle protoc on Windows

4. **Fix logging**:
   - Update imports in `cli/mod.rs` to include logging functions

5. **Build and test**:
   - Run `powershell.exe cargo build -p chat_cli`
   - Test the application with `powershell.exe cargo run -p chat_cli --bin chat_cli -- chat`

## Conclusion

The Windows port implementation focused on creating platform abstractions that allow the Amazon Q CLI to work across different platforms. By implementing these abstractions and fixing Windows-specific issues, we've made significant progress toward full Windows compatibility.

The key to the implementation was identifying platform-specific code and creating abstractions that could be implemented differently on each platform while maintaining a consistent interface for the core agent loop.
