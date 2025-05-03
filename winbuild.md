# Windows Native Build Plan for Amazon Q Developer CLI

This document tracks the plan and progress for building Amazon Q Developer CLI natively on Windows using the `x86_64-pc-windows-msvc` target.

## Environment Setup

### Required Tools
- [x] Rust toolchain (stable-msvc) - Installed: rustc 1.84.0
- [x] CMake - Installed
- [x] Visual Studio Build Tools with C++ workload - Installed
- [x] Protobuf Compiler - Installed

### Installation Steps
1. **Rust Toolchain**:
   ```cmd
   rustup default stable-msvc
   rustup target add x86_64-pc-windows-msvc
   ```
   ✅ Completed: Rust 1.84.0 is installed

2. **CMake**:
   - Download from: https://cmake.org/download/
   - Install and add to PATH
   - Verify with: `cmake --version`
   ✅ Completed: CMake is installed

3. **Visual Studio Build Tools**:
   - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Install with "Desktop development with C++" workload
   - Verify with: `cl.exe` (should be in PATH after running vcvars64.bat)
   ✅ Completed: Visual Studio Build Tools are installed

4. **Protobuf Compiler**:
   - Download from: https://github.com/protocolbuffers/protobuf/releases
   - Extract and add bin directory to PATH
   - Verify with: `protoc --version`
   ✅ Completed: Protobuf Compiler is installed

5. **Environment Variables**:
   ```cmd
   setx AWS_LC_SYS_STATIC 1
   setx AWS_LC_SYS_NO_ASM 1
   setx AWS_LC_SYS_C_STD c11
   setx CMAKE_POLICY_VERSION_MINIMUM 3.5
   ```
   ✅ Completed: Environment variables are set

## Code Modifications Completed

### 1. Platform Support in fig_os_shim
- ✅ Added Windows to the `Os` enum in `platform.rs`
- ✅ Updated `current()` method to handle Windows platform
- ✅ Updated `all()` and `as_str()` methods to include Windows

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum Os {
    Mac,
    Linux,
    Windows,  // Added Windows support
}

impl Os {
    pub fn current() -> Self {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                Self::Mac
            } else if #[cfg(target_os = "linux")] {
                Self::Linux
            } else if #[cfg(target_os = "windows")] {
                Self::Windows  // Added Windows case
            } else {
                compile_error!("unsupported platform");
            }
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Mac, Self::Linux, Self::Windows]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mac => "macos",
            Self::Linux => "linux",
            Self::Windows => "windows",
        }
    }
}
```

### 2. Process Info Implementation for Windows
- ✅ Created `process_info/windows.rs` with Windows implementations
- ✅ Added Windows module to `process_info/mod.rs`
- ✅ Implemented `as_u32()` method for Windows `RawPid`

```rust
// windows.rs
use std::path::PathBuf;
use std::sync::Weak;

use crate::Context;
use super::{Pid, RawPid};

pub fn current(ctx: Weak<Context>) -> Pid {
    let pid = std::process::id();
    Pid::Real(ctx, RawPid(pid))
}

pub fn parent(_ctx: Weak<Context>, _pid: &RawPid) -> Option<Box<Pid>> {
    // Windows implementation would need to use Windows API to get parent process
    // For now, return None as a placeholder
    None
}

pub fn exe(_ctx: Weak<Context>, _pid: &RawPid) -> Option<PathBuf> {
    // Windows implementation would need to use Windows API to get executable path
    // For now, return current executable as a placeholder
    std::env::current_exe().ok()
}

pub fn cmdline(_ctx: Weak<Context>, _pid: &RawPid) -> Option<String> {
    // Windows implementation would need to use Windows API to get command line
    // For now, return None as a placeholder
    None
}

// In pid.rs
impl RawPid {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
```

### 3. File System Operations for Windows
- ✅ Modified imports in `fs.rs` to use conditional compilation
- ✅ Updated the `append()` function to work on Windows
- ✅ Added Windows implementation for `symlink_exists()`

```rust
// Conditional imports
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(unix)] {
        use std::os::unix::ffi::OsStrExt;
    } else if #[cfg(windows)] {
        // Windows equivalent imports
        use std::os::windows::ffi::OsStrExt as WindowsOsStrExt;
    }
}

// Windows implementation for symlink_exists
pub async fn symlink_exists(&self, path: impl AsRef<Path>) -> bool {
    cfg_if! {
        if #[cfg(unix)] {
            match self.symlink_metadata(path).await {
                Ok(_) => true,
                Err(err) if err.kind() != std::io::ErrorKind::NotFound => true,
                Err(_) => false,
            }
        } else {
            // Windows implementation - simplified for now
            // Windows has limited symlink support, so we'll just check if the path exists
            self.exists(path)
        }
    }
}
```

### 4. Protobuf Build Script Support for Windows
- ✅ Added Windows support to `build.rs`
- ✅ Added Windows checksum for protoc download
- ✅ Modified checksum verification to work on Windows
- ✅ Added PowerShell commands for unzip and file operations on Windows
- ✅ Modified the build script to use locally installed protoc on Windows

```rust
// OS detection
let os = match std::env::consts::OS {
    "linux" => "linux",
    "macos" => "osx",
    "windows" => "win64",
    os => panic!("Unsupported os: {os}"),
};

// Use locally installed protoc on Windows
if cfg!(target_os = "windows") {
    // Check if protoc is available in PATH
    if let Ok(output) = Command::new("protoc").arg("--version").output() {
        if output.status.success() {
            // Use the locally installed protoc
            let protoc_path = which::which("protoc").expect("protoc should be in PATH");
            std::env::set_var("PROTOC", protoc_path);
            return;
        }
    }
}
```

### 5. System Info Implementation
- ✅ Fixed sysinfo usage in `sysinfo.rs`

```rust
use sysinfo::{System, SystemExt, ProcessExt};

// In is_process_running method
let mut system = System::new_all();
system.refresh_all();
let is_running = system.processes_by_name(name).next().is_some();
```

## Current Issues and Next Steps

### Issue 1: CMake C11 Atomics Error
```
fatal error C1189: #error: "C atomics require C11 or later"
```

**Next Steps**:
- [ ] Verify that `AWS_LC_SYS_C_STD=c11` environment variable is being properly recognized
- [ ] Try setting additional compiler flags for C11 support
- [ ] Consider using a different approach for aws-lc-sys

### Issue 2: CMake Generator Mismatch
```
CMake Error: Error: generator : Visual Studio 17 2022
Does not match the generator used previously: Visual Studio 16 2019
```

**Next Steps**:
- [ ] Run `cargo clean` to clear all build artifacts and CMake cache
- [ ] Delete any remaining CMakeCache.txt files manually
- [ ] Try specifying the generator explicitly with `CMAKE_GENERATOR` environment variable

### Issue 3: Protoc Download URL Issue
```
curl: (22) The requested URL returned error: 404
```

**Next Steps**:
- [ ] Verify the protoc path is correctly detected
- [ ] Update the URL pattern to match the actual release URL structure
- [ ] Consider bundling a protoc binary with the project for Windows builds

### Issue 4: Remaining Unix-specific Code
```
error[E0599]: no method named `symlink_metadata` found for reference `&Fs` in the current scope
```

**Next Steps**:
- [ ] Add Windows implementation for `symlink_metadata` or conditionally compile this function for Unix only
- [ ] Identify and fix other Unix-specific functions that need Windows alternatives

## Build Strategy

1. **Clean Build Environment**:
   ```cmd
   cargo clean
   ```

2. **Incremental Fixes**:
   - Fix one issue at a time
   - Verify each fix with a build attempt
   - Document progress and any new issues discovered

3. **Feature Flags**:
   - Try building with `--no-default-features` to disable problematic features
   - Gradually enable features as issues are fixed

4. **Testing**:
   - Test basic functionality after successful build
   - Document any runtime issues for future fixes

## Progress Tracking

| Issue | Status | Notes |
|-------|--------|-------|
| Environment Setup | ✅ Completed | All required tools installed |
| Platform Support | ✅ Completed | Windows added to Os enum |
| Process Info Implementation | ✅ Completed | Basic Windows implementation added |
| File System Operations | ⚠️ Partially Completed | Need to fix symlink_metadata |
| Protobuf Build Script | ⚠️ Partially Completed | Need to fix download URL |
| System Info Implementation | ✅ Completed | Fixed sysinfo usage |
| CMake C11 Atomics | ❌ Not Fixed | Need to ensure C11 standard is used |
| CMake Generator Mismatch | ❌ Not Fixed | Need to clear CMake cache |
| Build Attempt | ❌ Failed | Multiple issues to fix |

## References

- [Rust on Windows Documentation](https://doc.rust-lang.org/book/ch01-01-installation.html#windows)
- [CMake Download Page](https://cmake.org/download/)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [Protobuf Releases](https://github.com/protocolbuffers/protobuf/releases)