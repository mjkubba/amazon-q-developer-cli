# Windows Compatibility Plan for fig_os_shim

After analyzing the fig_os_shim crate, which is critical for the Amazon Q chat functionality, I've identified the following plan to make it fully Windows compatible.

## Current Status

The fig_os_shim crate already has some Windows support implemented:

1. **Platform Detection**: The `Os` enum in `platform.rs` already includes a Windows variant and detection logic.

2. **Process Info**: There's a `windows.rs` implementation for process information that uses the Windows API.

3. **File System Operations**: The `fs.rs` file has some Windows-specific code for symlinks and path handling.

However, there are several areas that need improvement to ensure full Windows compatibility.

## Implementation Plan

### 1. Fix File System Operations

The `fs.rs` module needs the following improvements:

- **Path Handling**: Ensure proper handling of Windows paths with drive letters (e.g., `C:\`)
- **Symlink Support**: Complete the Windows symlink implementation
- **File Permissions**: Implement Windows-specific file permission handling

```rust
// Example implementation for Windows path handling
#[cfg(windows)]
fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path_str = path.as_ref().to_string_lossy();
    
    // Handle UNC paths and drive letters
    if path_str.starts_with("\\\\") || (path_str.len() >= 2 && path_str.chars().nth(1) == Some(':')) {
        path.as_ref().to_path_buf()
    } else {
        // Relative path handling
        path.as_ref().to_path_buf()
    }
}
```

### 2. Improve Process Information

The `process_info/windows.rs` file needs:

- **Error Handling**: Better error handling for Windows API calls
- **Command Line Arguments**: Improve command line argument retrieval
- **Process Hierarchy**: Ensure reliable parent-child process relationship detection

```rust
// Example improvement for command line retrieval
pub fn cmdline(_ctx: Weak<Context>, pid: &RawPid) -> Option<String> {
    // Initialize COM library with proper error handling
    let com_lib = match COMLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return None,
    };
    
    // Connect to WMI with proper error handling
    let wmi_con = match WMIConnection::new(com_lib) {
        Ok(con) => con,
        Err(_) => return None,
    };
    
    // Rest of implementation...
}
```

### 3. Environment Variables

The `env.rs` module needs:

- **Windows-specific Environment Variables**: Handle Windows-specific environment variables like `%APPDATA%`, `%LOCALAPPDATA%`, etc.
- **Path Separators**: Handle Windows path separators in environment variables

```rust
#[cfg(windows)]
pub fn get_app_data_dir(&self) -> Option<PathBuf> {
    self.get_os("APPDATA").map(PathBuf::from)
}

#[cfg(windows)]
pub fn get_local_app_data_dir(&self) -> Option<PathBuf> {
    self.get_os("LOCALAPPDATA").map(PathBuf::from)
}
```

### 4. Signal Handling

Replace Unix-specific signal handling with Windows alternatives:

```rust
#[cfg(windows)]
pub async fn handle_ctrl_c() -> Result<(), std::io::Error> {
    use tokio::signal::windows::ctrl_c;
    ctrl_c().await
}

#[cfg(unix)]
pub async fn handle_ctrl_c() -> Result<(), std::io::Error> {
    use tokio::signal::unix::signal;
    use tokio::signal::unix::SignalKind;
    
    let mut sigint = signal(SignalKind::interrupt())?;
    sigint.recv().await;
    Ok(())
}
```

### 5. Feature Flags

Update the Cargo.toml to use appropriate feature flags:

```toml
[features]
default = ["auto-platform"]
auto-platform = []
unix-platform = []
windows-platform = []
```

## Testing Strategy

1. **Unit Tests**: Add Windows-specific unit tests for each component
2. **Integration Tests**: Create integration tests that verify Windows compatibility
3. **Manual Testing**: Test on Windows machines with different Windows versions

## Implementation Order

1. Fix file system operations first, as they're most critical for chat functionality
2. Improve process information handling
3. Update environment variable handling
4. Implement Windows-specific signal handling
5. Add comprehensive tests

## Compatibility Verification

After implementing these changes, verify compatibility by:

1. Building the minimal chat functionality on Windows
2. Testing basic chat operations
3. Testing file system operations used by tools
4. Testing process information retrieval

This plan focuses on making fig_os_shim fully Windows compatible while maintaining the existing Unix functionality, ensuring that the Amazon Q chat functionality works seamlessly on Windows.
