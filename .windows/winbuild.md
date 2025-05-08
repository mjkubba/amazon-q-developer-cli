# Windows Native Build Guide for Amazon Q Developer CLI

## Current Status

We've made significant progress in porting the Amazon Q Developer CLI to Windows. The core components of the application now compile successfully on Windows, including:

- The IPC implementation using Windows named pipes instead of Unix sockets
- The authentication system using a Windows-specific implementation
- The terminal handling with conditional compilation for platform-specific code
- The installation system with Windows-specific paths and functions

However, there are still several components that need to be fixed for a complete Windows build:

1. The `q_chat` crate has multiple Windows-specific issues
2. The `figterm` crate has Windows-specific issues with PTY implementation
3. The `fig_desktop` crate has Windows-specific issues with UI components

## Completed Fixes

### 1. Protocol Buffers Compiler (protoc) Missing - FIXED

**Issue:**
- The build was failing because it cannot download or find the Protocol Buffers compiler (protoc).

**Fix:**
- Modified `fig_proto/build.rs` to look for protoc in the custom path at `I:\workspace\protobuf\bin\protoc.exe`

### 2. Unix-specific Code in Windows Build - FIXED

**Issue:**
- Several crates are using Unix-specific code that doesn't compile on Windows

**Fix:**
- Added conditional compilation flags (`#[cfg(unix)]`) to the Unix-specific code
- Fixed the `nix` crate dependency issue with conditional compilation
- Added conditional compilation for `termwiz_terminal.rs` to skip it on Windows

### 3. IPC Implementation Issues - FIXED

**Issue:**
- The `fig_ipc` crate uses Unix sockets which are not available on Windows

**Fix:**
- Created a Windows-specific implementation using named pipes in `windows_pipe.rs`
- Updated `lib.rs` to use the appropriate implementation based on the platform
- Fixed the trait implementation issues in `local.rs`

### 4. Authentication Implementation Issues - FIXED

**Issue:**
- The `fig_auth` crate had missing type errors for `SecretStoreImpl`

**Fix:**
- Created a Windows-specific implementation of the secret store in `windows.rs`
- Added conditional compilation to use the appropriate implementation based on the platform

### 5. Installation Issues - FIXED

**Issue:**
- The `fig_install` crate has Windows-specific issues with paths and functions

**Fix:**
- Added conditional compilation for Unix-specific functions
- Implemented Windows-specific versions of these functions
- Fixed the `UpdatePackage` field reference to use `download_url` instead of `download`

### 6. Q Chat Syntax Error - FIXED

**Issue:**
- The `q_chat` crate had a syntax error with an unclosed delimiter in the `check_for_updates` function

**Fix:**
- Fixed the unclosed delimiter by properly closing the function

## Remaining Issues

### 1. Q Chat Windows-specific Issues

**Issue:**
- The `q_chat` crate has multiple Windows-specific issues:
  - Unix-specific imports that don't exist on Windows (tokio::signal::unix)
  - Missing dependencies in Cargo.toml (glob, etc.)
  - Tool implementation issues (UseAws implementation)
  - Rustyline integration issues (Helper trait implementation)
  - Windows terminal integration issues (termwiz compatibility)

**Progress:**
- Added conditional compilation for Unix-specific signal handling code
- Implemented a Windows-specific version using `tokio::signal::windows::ctrl_c`
- Added the `glob` dependency to the `Cargo.toml` file
- Fixed the `SkimHandler` implementation for Windows
- Fixed the `WindowsSelector` implementation for Windows terminal
- Fixed the `UseAws` tool implementation for Windows
- Fixed the `InvokeOutput` structure to be compatible with Windows

**Remaining Issues:**
- Need to fix the `InvokeOutput` structure to match the expected format
- Need to fix the `SkimHandler` implementation to match the expected trait
- Need to fix the `WindowsSelector` implementation to work with the Windows terminal API
- Need to fix the `OutputKind` enum to match the expected format

### 2. Figterm Windows PTY Implementation

**Issue:**
- The `figterm` crate has Windows-specific issues with PTY implementation:
  - Missing Windows-specific PTY implementation
  - Type mismatches between Windows API types
  - Missing imports for Windows-specific traits

**Plan to Fix:**
1. Implement a proper Windows PTY implementation
2. Fix type mismatches between Windows API types
3. Add missing imports for Windows-specific traits

### 3. Fig Desktop Windows UI Components

**Issue:**
- The `fig_desktop` crate has Windows-specific issues with UI components:
  - Missing Windows-specific menu implementation
  - Type mismatches in Windows API calls
  - Missing imports for Windows-specific traits

**Plan to Fix:**
1. Implement Windows-specific menu components
2. Fix type mismatches in Windows API calls
3. Add missing imports for Windows-specific traits

## Next Steps

1. Fix the remaining issues in the `q_chat` crate:
   - Fix the `InvokeOutput` structure to match the expected format
   - Fix the `SkimHandler` implementation to match the expected trait
   - Fix the `WindowsSelector` implementation to work with the Windows terminal API
   - Fix the `OutputKind` enum to match the expected format

2. Implement a proper Windows PTY implementation in the `figterm` crate:
   - Research Windows ConPTY API
   - Create a Windows-specific implementation of the PTY interface
   - Add conditional compilation to use the appropriate implementation based on the platform

3. Fix Windows-specific UI components in the `fig_desktop` crate:
   - Implement Windows-specific menu components
   - Fix type mismatches in Windows API calls
   - Add missing imports for Windows-specific traits

4. Create a Windows-specific installation package:
   - Create a Windows installer using NSIS or WiX
   - Add Windows-specific installation instructions to the README
   - Add Windows-specific uninstallation instructions

## Summary

We've made significant progress in porting Amazon Q Developer CLI to Windows. The core functionality now builds successfully, and we've fixed many of the Windows-specific issues. The most challenging parts remaining are the terminal emulation and UI components, which require significant Windows-specific code.

For now, we recommend using the WSL approach for Windows users until the native Windows build is fully implemented.
