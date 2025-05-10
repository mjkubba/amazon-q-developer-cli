# Windows Stack Overflow Fix Plan

## Problem Statement

When running `q chat` on Windows, the application experiences a stack overflow error. Despite our initial fixes:
1. Increasing the stack size
2. Fixing recursive cursor positioning
3. Adding panic handling to the input handler
4. Adding safety mechanisms to the chat loop

The issue persists when entering "hi there" as input, suggesting there are deeper issues in the Windows-specific code.

## Root Cause Analysis

The stack overflow is likely occurring due to one or more of these issues:

1. **Recursive Terminal Handling**: The current terminal handling code may still contain recursive calls that aren't properly bounded on Windows.

2. **Cross-Platform Library Incompatibilities**: Libraries like `termion` are primarily designed for Unix systems and may have unexpected behavior on Windows.

3. **Deep Call Stacks**: The combination of async code, error handling, and terminal operations may be creating excessively deep call stacks.

4. **Input Processing**: The way user input is processed might be causing infinite recursion in certain edge cases.

## Comprehensive Solution Plan

### Phase 1: Create a Pure Windows Terminal Implementation

1. **Create a new Windows-specific terminal handler**:
   - Implement a terminal handler that uses the Windows Console API directly
   - Avoid using termion or other Unix-centric libraries for Windows
   - Use the `windows` crate for direct Windows API access

2. **Implement core terminal functions**:
   ```rust
   pub struct WindowsTerminalHandler {
       handle: HANDLE,
   }
   
   impl WindowsTerminalHandler {
       pub fn new() -> Self {
           let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }.unwrap();
           Self { handle }
       }
       
       // Implement terminal functions using Windows Console API
       fn clear_line(&mut self) -> Result<()> {
           // Use Windows API directly
       }
       
       fn move_cursor(&mut self, x: i16, y: i16) -> Result<()> {
           // Use SetConsoleCursorPosition instead of termion
       }
       
       // Other terminal functions...
   }
   ```

### Phase 2: Implement a Robust Input Handler for Windows

1. **Create a Windows-specific input handler**:
   - Use the Windows Console API for input handling
   - Implement a non-recursive approach to reading input
   - Add proper error handling and timeouts

2. **Implement a simplified input reading method**:
   ```rust
   pub struct WindowsInputHandler {
       handle: HANDLE,
   }
   
   impl WindowsInputHandler {
       pub fn new() -> Self {
           let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) }.unwrap();
           Self { handle }
       }
       
       fn read_line(&mut self, prompt: Option<&str>) -> Result<Option<String>> {
           // Display prompt if provided
           if let Some(p) = prompt {
               print!("{}", p);
               std::io::stdout().flush()?;
           }
           
           // Use a simple approach that won't cause recursion
           let mut input = String::new();
           match std::io::stdin().read_line(&mut input) {
               Ok(_) => {
                   input = input.trim_end().to_string();
                   Ok(Some(input))
               },
               Err(e) => Err(eyre!("Error reading input: {}", e)),
           }
       }
   }
   ```

### Phase 3: Add Comprehensive Debug Logging

1. **Add detailed logging**:
   - Log function entry/exit for key functions
   - Track call stack depth
   - Add memory usage monitoring

2. **Implement a call stack depth tracker**:
   ```rust
   thread_local! {
       static CALL_DEPTH: RefCell<usize> = RefCell::new(0);
   }
   
   fn log_function_entry(name: &str) {
       CALL_DEPTH.with(|depth| {
           let current = *depth.borrow();
           *depth.borrow_mut() = current + 1;
           debug!("{}{} - ENTER", " ".repeat(current), name);
       });
   }
   
   fn log_function_exit(name: &str) {
       CALL_DEPTH.with(|depth| {
           let current = *depth.borrow();
           debug!("{}{} - EXIT", " ".repeat(current.saturating_sub(1)), name);
           *depth.borrow_mut() = current.saturating_sub(1);
       });
   }
   ```

### Phase 4: Implement Platform-Specific Feature Flags

1. **Use conditional compilation**:
   - Add feature flags for Windows-specific code
   - Create simplified implementations for problematic features on Windows

2. **Example implementation**:
   ```rust
   #[cfg(windows)]
   mod windows_impl {
       // Windows-specific implementation
   }
   
   #[cfg(not(windows))]
   mod unix_impl {
       // Unix implementation
   }
   
   #[cfg(windows)]
   pub use windows_impl::*;
   
   #[cfg(not(windows))]
   pub use unix_impl::*;
   ```

### Phase 5: Testing and Validation

1. **Create Windows-specific tests**:
   - Test terminal handling
   - Test input processing
   - Test error handling

2. **Implement stress tests**:
   - Test with large inputs
   - Test with rapid input sequences
   - Test memory usage under load

3. **Create a test harness**:
   ```rust
   #[test]
   #[cfg(windows)]
   fn test_windows_terminal_handler() {
       let mut handler = WindowsTerminalHandler::new();
       // Test terminal operations
   }
   
   #[test]
   #[cfg(windows)]
   fn test_windows_input_handler() {
       let mut handler = WindowsInputHandler::new();
       // Test input operations
   }
   ```

## Implementation Timeline

1. **Week 1**: Implement the pure Windows terminal handler
2. **Week 2**: Implement the robust Windows input handler
3. **Week 3**: Add comprehensive debug logging
4. **Week 4**: Implement platform-specific feature flags
5. **Week 5**: Testing and validation

## Success Criteria

1. The `q chat` command runs successfully on Windows without stack overflow
2. User can enter "hi there" and other inputs without crashes
3. All terminal features work correctly on Windows
4. Memory usage remains stable during extended chat sessions

## Fallback Options

If the comprehensive solution proves too complex or time-consuming, we can implement these fallback options:

1. **Simplified Mode for Windows**: Create a simplified chat mode for Windows that uses basic terminal I/O
2. **External Terminal Process**: Launch a separate process for terminal handling to isolate potential issues
3. **WSL Recommendation**: Recommend using WSL for Windows users until native support is fully implemented

## Conclusion

This comprehensive plan addresses the Windows stack overflow issue by creating platform-specific implementations that avoid the problematic recursive calls. By implementing a pure Windows terminal handler and robust input processing, we can ensure that the Amazon Q CLI works reliably on Windows platforms.
