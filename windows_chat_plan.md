# Windows Native Build Plan for Amazon Q Chat Functionality

This document outlines a focused plan to build just the essential 'q chat' functionality natively on Windows without relying on WSL. The goal is to create a minimal working version that allows users to run the 'q' command and access the chat functionality.

## Dependency Analysis

Based on the codebase analysis, here are the essential crates needed for the 'q chat' functionality:

1. **q_cli** - Main entry point for the CLI
2. **q_chat** - Core chat functionality
3. **fig_api_client** - Client for communicating with Amazon Q service
4. **fig_os_shim** - OS abstraction layer
5. **fig_util** - Utility functions
6. **fig_settings** - Settings management
7. **fig_auth** - Authentication functionality
8. **fig_terminal** - Terminal handling (needs Windows compatibility)
9. **fig_log** - Logging functionality
10. **fig_telemetry** - Telemetry collection

## Phased Implementation Plan

### Phase 1: Setup and Initial Build Attempt

1. **Create a minimal feature set**:
   - Create a new feature flag `windows-chat-minimal` that only enables the essential components
   - Modify Cargo.toml files to use this feature flag

2. **Initial build attempt**:
   ```powershell
   powershell.exe cargo build --no-default-features --features windows-chat-minimal -p q_cli
   ```

3. **Identify and document build errors**:
   - Capture all build errors
   - Categorize them by crate and issue type

### Phase 2: Fix Core Dependencies

1. **Fix fig_os_shim for Windows**:
   - Implement Windows-specific versions of file system operations
   - Implement Windows-specific process info functions
   - Add proper conditional compilation with `#[cfg(windows)]` attributes

2. **Fix fig_terminal for Windows**:
   - Replace Unix-specific terminal code with Windows-compatible alternatives
   - Implement Windows console handling using termwiz
   - Create a cross-platform terminal interface

3. **Fix signal handling**:
   - Replace Unix-specific signal handling with Windows alternatives
   - Implement Ctrl+C handling for Windows

### Phase 3: Fix Chat-Specific Components

1. **Fix q_chat for Windows**:
   - Replace Unix-specific code with Windows alternatives
   - Add conditional compilation for platform-specific code
   - Implement Windows-compatible versions of any Unix-specific functionality

2. **Fix q_cli for Windows**:
   - Ensure the main entry point works on Windows
   - Fix any platform-specific code in command handling

### Phase 4: Testing and Refinement

1. **Build minimal version**:
   ```powershell
   powershell.exe cargo build --no-default-features --features windows-chat-minimal -p q_cli
   ```

2. **Test basic functionality**:
   - Test running the 'q' command
   - Test basic chat interaction
   - Test tool execution

3. **Fix remaining issues**:
   - Address any runtime issues discovered during testing
   - Fix any remaining compatibility issues

### Phase 5: Packaging

1. **Create Windows executable**:
   ```powershell
   powershell.exe cargo build --release --no-default-features --features windows-chat-minimal -p q_cli
   ```

2. **Create simple installer script**:
   - Create a PowerShell script to install the executable
   - Add necessary environment setup

## Implementation Details

### Windows-Specific Modifications

1. **Terminal Handling**:
   - Replace crossterm/tuikit with termwiz for Windows console support
   - Implement Windows-specific terminal size detection
   - Handle Windows console color and formatting

2. **File System Operations**:
   - Use platform-agnostic file system operations
   - Replace Unix-specific path handling with Windows-compatible alternatives

3. **Process Handling**:
   - Replace Unix-specific process handling with Windows alternatives
   - Implement Windows-compatible versions of process information functions

4. **Signal Handling**:
   - Replace Unix-specific signal handling with Windows event handling
   - Implement Ctrl+C handling using Windows APIs

### Feature Flag Strategy

Create a hierarchical feature flag system:

```toml
[features]
default = ["unix-full"]
unix-full = ["unix-terminal", "unix-process", "unix-fs"]
windows-full = ["windows-terminal", "windows-process", "windows-fs"]
windows-chat-minimal = ["windows-terminal", "windows-process", "windows-fs", "minimal-tools"]
unix-terminal = []
windows-terminal = []
unix-process = []
windows-process = []
unix-fs = []
windows-fs = []
minimal-tools = []
```

This allows for selective enabling of platform-specific components while maintaining a minimal feature set for the chat functionality.

## Success Criteria

1. The 'q' command runs natively on Windows without WSL
2. Users can interact with the chat functionality
3. Basic tool execution works
4. The application handles Windows-specific terminal behavior correctly
5. The application can be built with a simple cargo command

## Next Steps After Initial Success

1. Expand Windows support to other CLI commands
2. Improve Windows terminal handling for better user experience
3. Add more comprehensive Windows testing
4. Create a proper Windows installer
