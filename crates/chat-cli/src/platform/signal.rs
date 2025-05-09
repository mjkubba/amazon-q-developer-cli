use eyre::Result;
use std::future::Future;
use std::pin::Pin;

/// Signal types that can be handled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Interrupt signal (Ctrl+C)
    Interrupt,
    /// Terminal resize event
    Resize(u16, u16),
    /// Hangup signal
    Hangup,
    /// Termination signal
    Terminate,
}

/// Trait for platform-agnostic signal handling
pub trait SignalHandler {
    /// Wait for a signal
    fn wait_for_signal(&self) -> Pin<Box<dyn Future<Output = Result<Signal>> + Send>>;
}

/// Type for Ctrl+C handler function
pub type CtrlCHandlerFn = Box<dyn FnMut() + Send + 'static>;

/// Type for resize handler function
pub type ResizeHandlerFn = Box<dyn FnMut(u16, u16) + Send + 'static>;

#[cfg(unix)]
pub mod unix {
    use super::*;
    use tokio::signal::unix::{signal, SignalKind};
    
    /// Unix implementation of SignalHandler
    pub struct UnixSignalHandler;
    
    impl UnixSignalHandler {
        pub fn new() -> Self {
            Self
        }
        
        /// Set up a handler for Ctrl+C
        pub fn handle_ctrl_c(handler: impl FnMut() + Send + 'static) -> Result<()> {
            let _ = ctrlc::set_handler(handler)?;
            Ok(())
        }
        
        /// Set up a handler for terminal resize events
        pub fn handle_resize(handler: impl FnMut(u16, u16) + Send + 'static) -> Result<()> {
            // This is a simplified implementation
            // In a real implementation, we would need to set up a signal handler for SIGWINCH
            // and call the handler with the new terminal size
            
            Ok(())
        }
    }
    
    impl SignalHandler for UnixSignalHandler {
        fn wait_for_signal(&self) -> Pin<Box<dyn Future<Output = Result<Signal>> + Send>> {
            Box::pin(async {
                let mut sigint = signal(SignalKind::interrupt())?;
                let mut sigterm = signal(SignalKind::terminate())?;
                let mut sighup = signal(SignalKind::hangup())?;
                
                tokio::select! {
                    _ = sigint.recv() => Ok(Signal::Interrupt),
                    _ = sigterm.recv() => Ok(Signal::Terminate),
                    _ = sighup.recv() => Ok(Signal::Hangup),
                }
            })
        }
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    
    /// Windows implementation of SignalHandler
    pub struct WindowsSignalHandler;
    
    impl WindowsSignalHandler {
        pub fn new() -> Self {
            Self
        }
        
        /// Set up a handler for Ctrl+C
        pub fn handle_ctrl_c(handler: impl FnMut() + Send + 'static) -> Result<()> {
            let _ = ctrlc::set_handler(handler)?;
            Ok(())
        }
        
        /// Set up a handler for terminal resize events
        pub fn handle_resize(handler: impl FnMut(u16, u16) + Send + 'static) -> Result<()> {
            // Windows doesn't have SIGWINCH, so we would need a different approach
            // This is a simplified implementation
            
            Ok(())
        }
    }
    
    impl SignalHandler for WindowsSignalHandler {
        fn wait_for_signal(&self) -> Pin<Box<dyn Future<Output = Result<Signal>> + Send>> {
            Box::pin(async {
                // This is a simplified implementation
                // In a real implementation, we would need to set up a Windows-specific
                // way to wait for signals
                
                let ctrl_c = tokio::signal::ctrl_c().await?;
                Ok(Signal::Interrupt)
            })
        }
    }
}

/// Factory function to create the appropriate signal handler for the current platform
pub fn create_signal_handler() -> Box<dyn SignalHandler> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixSignalHandler::new())
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsSignalHandler::new())
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        Box::new(unix::UnixSignalHandler::new())
    }
}

/// Set up a handler for Ctrl+C signals
pub fn handle_ctrl_c(handler: impl FnMut() + Send + 'static) -> Result<()> {
    #[cfg(unix)]
    {
        unix::UnixSignalHandler::handle_ctrl_c(handler)
    }
    #[cfg(windows)]
    {
        windows::WindowsSignalHandler::handle_ctrl_c(handler)
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        unix::UnixSignalHandler::handle_ctrl_c(handler)
    }
}

/// Set up a handler for terminal resize events
pub fn handle_resize(handler: impl FnMut(u16, u16) + Send + 'static) -> Result<()> {
    #[cfg(unix)]
    {
        unix::UnixSignalHandler::handle_resize(handler)
    }
    #[cfg(windows)]
    {
        windows::WindowsSignalHandler::handle_resize(handler)
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback implementation for other platforms
        // For now, we'll use the Unix implementation
        unix::UnixSignalHandler::handle_resize(handler)
    }
}
