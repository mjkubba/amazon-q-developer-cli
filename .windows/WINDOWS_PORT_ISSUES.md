# Windows Port Issues

This document outlines the issues encountered during the Windows port of Amazon Q CLI and their solutions.

## Stack Overflow in Chat Command

### Issue
When running `q chat` on Windows, the application experiences a stack overflow. This occurs in the terminal handling code, specifically when using `termion::cursor::DetectCursorPos()` which causes unbounded recursion on Windows.

### Solution
1. Increased stack size for Windows builds by adding a `.cargo/config.toml` file:
   ```toml
   [target.'cfg(windows)']
   rustflags = ["-C", "link-args=/STACK:8388608"]  # Increase stack size to 8MB for Windows
   ```

2. Fixed the recursive cursor positioning in the terminal handler:
   ```rust
   // Before (causes recursion):
   fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
       write!(self.stdout, "{}", termion::cursor::Goto(column, termion::cursor::DetectCursorPos().unwrap_or((0, 0)).1))?;
       Ok(())
   }

   // After (safe implementation):
   fn move_cursor_to_column(&mut self, column: u16) -> Result<()> {
       // Use a safer approach that doesn't rely on DetectCursorPos which can cause recursion
       write!(self.stdout, "\r")?;
       if column > 0 {
           write!(self.stdout, "{}", termion::cursor::Right(column))?;
       }
       Ok(())
   }
   ```

3. Added panic handling to the input handler to prevent crashes:
   ```rust
   fn read_line(&mut self, prompt: Option<&str>) -> Result<Option<String>> {
       // Use a try-catch pattern to avoid potential recursion issues
       let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
           self.editor.readline(prompt.unwrap_or("> "))
       }));
       
       // Handle the result safely with fallback to simpler implementation
       match result {
           Ok(Ok(line)) => {
               // Process line normally
               Ok(Some(line))
           },
           Err(_) => {
               // If we panic, fall back to a simpler implementation
               // ...
           }
       }
   }
   ```

4. Added safety mechanisms to the chat loop:
   - Maximum iteration count to detect infinite loops
   - Timeout protection for input operations
   - Debug command to help diagnose issues

## Other Windows-Specific Issues

1. **Path Handling**: Windows uses backslashes for paths, which required special handling in file operations.

2. **Terminal Control**: Windows console API differs significantly from Unix terminal handling, requiring platform-specific implementations.

3. **Process Management**: Process termination on Windows required using the Windows API instead of Unix signals.

4. **Build System**: Windows builds required special handling for tools like protoc.

## Remaining Work

1. Complete the implementation of the chat command with proper error handling
2. Add more comprehensive Windows-specific tests
3. Create a proper Windows installer package
4. Fix the remaining issues with the full workspace build
