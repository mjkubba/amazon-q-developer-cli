# Amazon Q CLI Windows Port Guide

This guide provides comprehensive instructions for porting the Amazon Q CLI to Windows. It consolidates the documentation from the `.windows/` directory and outlines the steps taken to implement Windows compatibility.

## Table of Contents

1. [Overview](#overview)
2. [Development Environment Setup](#development-environment-setup)
3. [Implementation Approach](#implementation-approach)
4. [Key Issues and Solutions](#key-issues-and-solutions)
5. [Implementation Steps](#implementation-steps)
6. [Testing and Validation](#testing-and-validation)
7. [Changes Made to chat-cli](#changes-made-to-chat-cli)
8. [Future Work](#future-work)

## Overview

The Amazon Q CLI was originally designed primarily for Unix-based systems. This project aims to refactor the codebase to support Windows while maintaining compatibility with existing Unix platforms. The core focus is on separating platform-specific code from the core agent loop logic.

## Development Environment Setup

### Windows Native Development

1. Install Rust for Windows: https://www.rust-lang.org/tools/install
2. Install Visual Studio Build Tools with C++ support
3. Clone the repository
4. Install Protocol Buffers compiler (protoc):
   - Download from https://github.com/protocolbuffers/protobuf/releases
   - Extract to a directory (e.g., `../protobuf/`)
   - Ensure `protoc.exe` is in the PATH or referenced in the build script

### WSL Development

1. Install WSL2 with a Linux distribution
2. Set up Rust in the WSL environment
3. Clone the repository
4. Test Windows compatibility by building for the Windows target:
   ```
   rustup target add x86_64-pc-windows-gnu
   cargo build --target x86_64-pc-windows-gnu
   ```

### Build Configuration

Create a `.cargo/config.toml` file with the following content to increase the stack size for Windows builds:

```toml
[target.'cfg(windows)']
rustflags = ["-C", "link-args=/STACK:8388608"]  # Increase stack size to 8MB for Windows
```

## Implementation Approach

The implementation follows a phased approach:

1. **Analysis and Preparation**: Identify platform-specific code and dependencies
2. **Platform Abstraction Layer**: Create platform-agnostic abstractions
3. **Core Agent Loop Refactoring**: Update code to use the abstractions
4. **Windows Implementation**: Implement Windows-specific versions
5. **Integration and Testing**: Ensure end-to-end functionality

## Key Issues and Solutions

### Stack Overflow in Chat Command

**Issue**: When running `q chat` on Windows, the application experiences a stack overflow in the terminal handling code, specifically when using `termion::cursor::DetectCursorPos()` which causes unbounded recursion.

**Solution**:
1. Increase stack size for Windows builds
2. Fix recursive cursor positioning in the terminal handler:
   ```rust
   // Before (causes recursion):
   fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
       write!(self.stdout, "{}", termion::cursor::Goto(column, termion::cursor::DetectCursorPos().unwrap_or((0, 0)).1))?;
       Ok(())
   }

   // After (safe implementation):
   fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
       // Use a safer approach that doesn't rely on DetectCursorPos
       write!(self.stdout, "\r")?;
       if column > 0 {
           write!(self.stdout, "{}", termion::cursor::Right(column))?;
       }
       Ok(())
   }
   ```
3. Add panic handling to the input handler
4. Add safety mechanisms to the chat loop

### Other Windows-Specific Issues

1. **Path Handling**: Windows uses backslashes for paths, requiring special handling
2. **Terminal Control**: Windows console API differs from Unix terminal handling
3. **Process Management**: Process termination on Windows requires Windows API
4. **Build System**: Windows builds require special handling for tools like protoc

## Implementation Steps

### Phase 1: Terminal Handling Abstraction

1. Create `platform/terminal.rs` with the `TerminalHandler` trait:
   ```rust
   pub trait TerminalHandler {
       fn terminal_width(&self) -> Option<usize>;
       fn clear_line(&mut self) -> Result<()>;
       fn move_cursor_to_column(&mut self, column: u16) -> Result<()>;
       fn show_cursor(&mut self) -> Result<()>;
       fn hide_cursor(&mut self) -> Result<()>;
       fn set_foreground_color(&mut self, color: TerminalColor) -> Result<()>;
       fn reset_color(&mut self) -> Result<()>;
       fn set_attribute(&mut self, attribute: TerminalAttribute) -> Result<()>;
       fn reset_attributes(&mut self) -> Result<()>;
       fn print(&mut self, text: &str) -> Result<()>;
       fn flush(&mut self) -> Result<()>;
   }
   ```

2. Implement Unix version using crossterm
3. Create Windows implementation using crossterm's Windows support
4. Update `platform/mod.rs` to expose the terminal abstraction

### Phase 2: Input Handling Abstraction

1. Create `platform/input.rs` with the `InputHandler` trait:
   ```rust
   pub trait InputHandler {
       fn read_line(&mut self, prompt: Option<&str>) -> Result<Option<String>>;
       fn has_input(&self) -> Result<bool>;
       fn enable_raw_mode(&mut self) -> Result<()>;
       fn disable_raw_mode(&mut self) -> Result<()>;
       fn read_key(&mut self) -> Result<Option<KeyEvent>>;
       // Additional methods...
   }
   ```

2. Implement Unix version using rustyline
3. Create Windows implementation using rustyline with Windows-specific handling

### Phase 3: Process Execution Abstraction

1. Create `platform/process.rs` with the `ProcessExecutor` trait:
   ```rust
   pub trait ProcessExecutor {
       fn execute_command(&self, command: &str) -> Result<Output>;
       fn execute_background_command(&self, command: &str) -> Result<u32>;
       fn is_process_running(&self, pid: u32) -> Result<bool>;
       fn kill_process(&self, pid: u32) -> Result<()>;
       fn shell_command(&self) -> &str;
       fn shell_args(&self, command: &str) -> Vec<String>;
   }
   ```

2. Implement Unix version using std::process
3. Create Windows implementation using appropriate Windows APIs

### Phase 4: Signal Handling Abstraction

1. Create `platform/signal.rs` with the `SignalHandler` trait:
   ```rust
   pub trait SignalHandler {
       fn wait_for_signal(&self) -> Pin<Box<dyn Future<Output = Result<Signal>> + Send>>;
   }
   ```

2. Implement Unix version using tokio::signal
3. Create Windows implementation using Windows-specific signal handling

## Testing and Validation

1. Run `powershell.exe cargo build` after each significant change
2. Test terminal handling on Windows
3. Test input processing on Windows
4. Test command execution on Windows
5. Run the full application on Windows with `powershell.exe cargo run -p chat_cli --bin chat_cli -- chat`

## Changes Made to chat-cli

The following changes were made to the `chat-cli` crate to support Windows:

1. **Added Platform Abstraction Layer**:
   - Created `platform/terminal.rs` with cross-platform terminal handling
   - Created `platform/input.rs` with cross-platform input handling
   - Created `platform/process.rs` with cross-platform process execution
   - Created `platform/signal.rs` with cross-platform signal handling
   - Updated `platform/mod.rs` to expose these abstractions

2. **Terminal Handling**:
   - Implemented `TerminalHandler` trait with Unix and Windows implementations
   - Fixed recursive cursor positioning that caused stack overflow on Windows
   - Used crossterm for both Unix and Windows terminal handling

3. **Build Configuration**:
   - Added `.cargo/config.toml` to increase stack size for Windows builds
   - Modified `fig_proto/build.rs` to handle protoc on Windows

4. **Fixed Logging Issues**:
   - Updated imports in `cli/mod.rs` to include logging functions

## Future Work

1. Complete the implementation of the chat command with proper error handling
2. Add more comprehensive Windows-specific tests
3. Create a proper Windows installer package
4. Fix the remaining issues with the full workspace build
5. Implement a Windows alternative to skim for fuzzy search functionality
6. Add Windows-specific documentation for users

## Conclusion

This guide provides a comprehensive overview of the Windows port implementation for the Amazon Q CLI. By following the steps outlined in this document, you can successfully port the application to Windows while maintaining compatibility with existing Unix platforms.

For more detailed information, refer to the individual documents in the `.windows/` directory.
