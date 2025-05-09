use eyre::Result;
use std::process::Output;

/// Trait for platform-agnostic process execution
pub trait ProcessExecutor {
    /// Execute a command and return the output
    fn execute_command(&self, command: &str) -> Result<Output>;
    
    /// Execute a command in the background
    fn execute_background_command(&self, command: &str) -> Result<u32>;
    
    /// Check if a process is running
    fn is_process_running(&self, pid: u32) -> Result<bool>;
    
    /// Kill a process
    fn kill_process(&self, pid: u32) -> Result<()>;
    
    /// Get the platform-specific shell
    fn shell_command(&self) -> &str;
    
    /// Get the platform-specific shell arguments for executing a command
    fn shell_args(&self, command: &str) -> Vec<String>;
}

#[cfg(unix)]
pub mod unix {
    use super::*;
    use eyre::eyre;
    use std::process::{Command, Stdio};
    
    /// Unix implementation of ProcessExecutor
    pub struct UnixProcessExecutor;
    
    impl UnixProcessExecutor {
        pub fn new() -> Self {
            Self
        }
    }
    
    impl ProcessExecutor for UnixProcessExecutor {
        fn execute_command(&self, command: &str) -> Result<Output> {
            let output = Command::new("bash")
                .arg("-c")
                .arg(command)
                .output()?;
            
            Ok(output)
        }
        
        fn execute_background_command(&self, command: &str) -> Result<u32> {
            let child = Command::new("bash")
                .arg("-c")
                .arg(command)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            
            Ok(child.id())
        }
        
        fn is_process_running(&self, pid: u32) -> Result<bool> {
            // On Unix, we can check if a process is running by sending signal 0
            // This doesn't actually send a signal, but performs error checking
            #[cfg(unix)]
            {
                use nix::sys::signal;
                use nix::unistd::Pid;
                
                let result = signal::kill(Pid::from_raw(pid as i32), None);
                Ok(result.is_ok())
            }
            
            #[cfg(not(unix))]
            {
                Err(eyre!("Not implemented on this platform"))
            }
        }
        
        fn kill_process(&self, pid: u32) -> Result<()> {
            #[cfg(unix)]
            {
                use nix::sys::signal;
                use nix::unistd::Pid;
                
                signal::kill(Pid::from_raw(pid as i32), signal::Signal::SIGTERM)?;
                Ok(())
            }
            
            #[cfg(not(unix))]
            {
                Err(eyre!("Not implemented on this platform"))
            }
        }
        
        fn shell_command(&self) -> &str {
            "bash"
        }
        
        fn shell_args(&self, command: &str) -> Vec<String> {
            vec!["-c".to_string(), command.to_string()]
        }
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use eyre::eyre;
    use std::process::{Command, Stdio};
    
    /// Windows implementation of ProcessExecutor
    pub struct WindowsProcessExecutor;
    
    impl WindowsProcessExecutor {
        pub fn new() -> Self {
            Self
        }
    }
    
    impl ProcessExecutor for WindowsProcessExecutor {
        fn execute_command(&self, command: &str) -> Result<Output> {
            let output = Command::new("cmd")
                .arg("/C")
                .arg(command)
                .output()?;
            
            Ok(output)
        }
        
        fn execute_background_command(&self, command: &str) -> Result<u32> {
            let child = Command::new("cmd")
                .arg("/C")
                .arg(command)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            
            Ok(child.id())
        }
        
        fn is_process_running(&self, pid: u32) -> Result<bool> {
            // On Windows, we can use tasklist to check if a process is running
            let output = Command::new("tasklist")
                .arg("/FI")
                .arg(format!("PID eq {}", pid))
                .arg("/NH")
                .output()?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(!output_str.trim().is_empty() && !output_str.contains("No tasks"))
        }
        
        fn kill_process(&self, pid: u32) -> Result<()> {
            let output = Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/F")
                .output()?;
            
            if output.status.success() {
                Ok(())
            } else {
                Err(eyre!(
                    "Failed to kill process: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
        
        fn shell_command(&self) -> &str {
            "cmd"
        }
        
        fn shell_args(&self, command: &str) -> Vec<String> {
            vec!["/C".to_string(), command.to_string()]
        }
    }
}

/// Factory function to create the appropriate process executor for the current platform
pub fn create_process_executor() -> Box<dyn ProcessExecutor> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixProcessExecutor::new())
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsProcessExecutor::new())
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        Box::new(unix::UnixProcessExecutor::new())
    }
}
