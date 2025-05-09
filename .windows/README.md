# Windows Port for Amazon Q CLI

This directory contains planning documents and resources for porting the Amazon Q CLI to Windows.

## Overview

The Amazon Q CLI is currently designed primarily for Unix-based systems. This project aims to refactor the codebase to support Windows while maintaining compatibility with existing Unix platforms.

## Documents

- [**REFACTORING_PLAN.md**](./REFACTORING_PLAN.md): Overall phased approach for the Windows port
- [**PLATFORM_SPECIFIC_CODE.md**](./PLATFORM_SPECIFIC_CODE.md): Analysis of platform-specific code in the codebase
- [**ABSTRACTION_INTERFACES.md**](./ABSTRACTION_INTERFACES.md): Proposed abstraction interfaces for platform-agnostic code
- [**IMPLEMENTATION_PHASES.md**](./IMPLEMENTATION_PHASES.md): Detailed implementation steps for each phase
- [**WINDOWS_SPECIFIC_CONSIDERATIONS.md**](./WINDOWS_SPECIFIC_CONSIDERATIONS.md): Special considerations for Windows implementation

## Getting Started

1. Review the refactoring plan to understand the overall approach
2. Examine the platform-specific code analysis to identify areas needing changes
3. Follow the implementation phases document for step-by-step guidance
4. Refer to Windows-specific considerations when implementing Windows support

## Development Environment Setup

### Windows Native Development

1. Install Rust for Windows: https://www.rust-lang.org/tools/install
2. Install Visual Studio Build Tools with C++ support
3. Clone the repository and switch to the `mjkubba-win` branch
4. Run `cargo build` to verify the build environment

### WSL Development

1. Install WSL2 with a Linux distribution
2. Set up Rust in the WSL environment
3. Clone the repository and switch to the `mjkubba-win` branch
4. Run `cargo build` to verify the build environment
5. Test Windows compatibility by building for the Windows target:
   ```
   rustup target add x86_64-pc-windows-gnu
   cargo build --target x86_64-pc-windows-gnu
   ```

## Testing

- Unit tests should be written for all platform abstractions
- Integration tests should verify functionality on both Unix and Windows
- Manual testing on various Windows versions and terminal emulators is recommended

## Development Workflow

1. Follow the implementation phases outlined in the documents
2. After each significant change, run `powershell.exe cargo build` to test Windows compatibility
3. Create small, focused commits with clear messages
4. Document any Windows-specific considerations or workarounds
5. Ensure all tests pass on both Unix and Windows platforms
