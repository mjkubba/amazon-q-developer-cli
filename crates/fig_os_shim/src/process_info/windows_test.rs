#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::Context;
    use crate::process_info::{Pid, RawPid};
    use super::super::windows::{current, parent, exe, cmdline};

    #[test]
    fn test_current_process() {
        let ctx = Arc::new(Context::new());
        let pid = current(Arc::downgrade(&ctx));
        
        // Verify that the current process ID matches the one from std::process::id()
        assert_eq!(pid.as_u32(), std::process::id());
    }

    #[test]
    fn test_process_exe() {
        let ctx = Arc::new(Context::new());
        let pid = current(Arc::downgrade(&ctx));
        
        // Get the executable path
        let exe_path = exe(Arc::downgrade(&ctx), &RawPid(pid.as_u32()));
        
        // Verify that we got a path
        assert!(exe_path.is_some());
        
        // Verify that the path exists
        let path = exe_path.unwrap();
        assert!(path.exists());
        
        // Verify that the path is to an executable file
        assert!(path.is_file());
    }

    #[test]
    fn test_parent_process() {
        let ctx = Arc::new(Context::new());
        let pid = current(Arc::downgrade(&ctx));
        
        // Get the parent process
        let parent_pid = parent(Arc::downgrade(&ctx), &RawPid(pid.as_u32()));
        
        // Verify that we got a parent process
        assert!(parent_pid.is_some());
        
        // Verify that the parent process ID is different from the current process ID
        let parent = parent_pid.unwrap();
        assert_ne!(parent.as_u32(), pid.as_u32());
        
        // Verify that the parent process has an executable path
        let parent_exe = parent.exe();
        assert!(parent_exe.is_some());
    }

    #[test]
    fn test_process_cmdline() {
        let ctx = Arc::new(Context::new());
        let pid = current(Arc::downgrade(&ctx));
        
        // Get the command line
        let cmd = cmdline(Arc::downgrade(&ctx), &RawPid(pid.as_u32()));
        
        // Verify that we got a command line
        // Note: This test might be flaky if WMI access is restricted
        // So we'll just print the result for debugging purposes
        println!("Command line: {:?}", cmd);
        
        // The command line might be None in some environments, so we don't assert on it
        // But if it's Some, it should contain the executable name
        if let Some(cmd_str) = cmd {
            let exe_path = exe(Arc::downgrade(&ctx), &RawPid(pid.as_u32())).unwrap();
            let exe_name = exe_path.file_name().unwrap().to_string_lossy();
            
            // The command line should contain the executable name
            assert!(cmd_str.contains(&exe_name));
        }
    }
}
