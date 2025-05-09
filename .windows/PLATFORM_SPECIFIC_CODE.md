# Platform-Specific Code Analysis

This document identifies the platform-specific code in the Amazon Q CLI that needs to be abstracted for Windows compatibility.

## Core Agent Loop (`crates/chat-cli/src/cli/chat/mod.rs`)

### Terminal Handling

The core agent loop uses `crossterm` for terminal manipulation:

- **Terminal size detection**: Uses `terminal::window_size()` to get terminal dimensions
- **Cursor manipulation**: Uses `cursor::Show`, `cursor::Hide`, `cursor::MoveToColumn`
- **Terminal clearing**: Uses `terminal::Clear(terminal::ClearType::CurrentLine)`
- **Text styling**: Uses `style::SetForegroundColor`, `style::SetAttribute`, etc.

### Input Handling

- **Input source**: The `InputSource` class handles user input
- **Skim integration**: The `skim_integration` module is conditionally compiled with `#[cfg(unix)]`
- **Readline**: Uses `rustyline` for input handling which may have platform-specific behavior

### File System Operations

- **Path handling**: File paths use Unix-style separators
- **File operations**: Uses standard Rust file operations which should be cross-platform but may need path adjustments

### Process Handling

- **Command execution**: Uses `std::process::Command` to execute shell commands
- **Shell commands**: Assumes `bash` is available for command execution

### Unix-Specific Features

- **Terminal detection**: Uses `IsTerminal` trait which may have platform-specific implementations
- **Signal handling**: Uses `tokio::signal::ctrl_c()` for handling Ctrl+C
- **Bell notification**: Uses platform-specific code for audio notifications

## Dependencies with Platform-Specific Code

1. **crossterm**: Has Windows support but may need configuration
2. **rustyline**: Has Windows support but may need configuration
3. **skim**: Unix-only, needs alternative for Windows
4. **nix**: Unix-specific, needs Windows alternatives
5. **tokio**: Has Windows support but signal handling differs

## Platform-Specific Modules

1. `skim_integration.rs`: Conditionally compiled with `#[cfg(unix)]`
2. Parts of `input_source.rs`: May contain Unix-specific code
3. Parts of `util.rs`: May contain Unix-specific code for notifications and terminal handling

## Abstraction Needs

1. **Terminal Handling**: Create a platform-agnostic interface for terminal operations
2. **Input Methods**: Create a platform-agnostic interface for input handling
3. **File System**: Extend the existing `fs` module for better cross-platform support
4. **Process Execution**: Create a platform-agnostic interface for executing commands
5. **Signal Handling**: Create a platform-agnostic interface for handling signals
