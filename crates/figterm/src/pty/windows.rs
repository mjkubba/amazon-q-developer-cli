use std::io::{Error as IoError, Result as IoResult};
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use filedescriptor::FileDescriptor;
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use winapi::um::wincon::COORD;
use windows::Win32::System::Console::{
    CreatePseudoConsole, ClosePseudoConsole, HPCON, PSEUDOCONSOLE_CREATION_FLAGS,
};
use windows::Win32::Foundation::{HANDLE, CloseHandle};

use crate::pty::cmdbuilder::CmdBuilder;
use crate::pty::PtySize;

/// Windows implementation of a pseudoterminal using ConPTY
pub struct Pty {
    conpty_handle: HPCON,
    input_write_fd: FileDescriptor,
    output_read_fd: FileDescriptor,
    child: Option<Child>,
    size: Arc<Mutex<PtySize>>,
}

impl Drop for Pty {
    fn drop(&mut self) {
        // Close the ConPTY handle
        unsafe {
            ClosePseudoConsole(self.conpty_handle);
        }
    }
}

/// Open a new PTY
pub fn open_pty(size: PtySize) -> Result<(Pty, FileDescriptor, FileDescriptor)> {
    // Create pipes for ConPTY
    let (input_read_fd, input_write_fd) = create_pipe()?;
    let (output_read_fd, output_write_fd) = create_pipe()?;

    // Convert to HANDLE types
    let input_read_handle = HANDLE(input_read_fd.as_raw_handle() as isize);
    let output_write_handle = HANDLE(output_write_fd.as_raw_handle() as isize);

    // Create the ConPTY
    let conpty_size = COORD {
        X: size.cols as i16,
        Y: size.rows as i16,
    };

    let mut conpty_handle = HPCON::default();
    let result = unsafe {
        CreatePseudoConsole(
            conpty_size,
            input_read_handle,
            output_write_handle,
            PSEUDOCONSOLE_CREATION_FLAGS(0),
            &mut conpty_handle,
        )
    };

    if let Err(err) = result {
        return Err(anyhow::anyhow!("Failed to create ConPTY: {:?}", err));
    }

    // Close the handles that were passed to CreatePseudoConsole
    unsafe {
        CloseHandle(input_read_handle);
        CloseHandle(output_write_handle);
    }

    let pty = Pty {
        conpty_handle,
        input_write_fd,
        output_read_fd,
        child: None,
        size: Arc::new(Mutex::new(size)),
    };

    Ok((pty, input_write_fd, output_read_fd))
}

/// Create a pipe for communication
fn create_pipe() -> IoResult<(FileDescriptor, FileDescriptor)> {
    let mut read_handle = std::ptr::null_mut();
    let mut write_handle = std::ptr::null_mut();
    
    let success = unsafe {
        winapi::um::namedpipeapi::CreatePipe(
            &mut read_handle,
            &mut write_handle,
            std::ptr::null_mut(),
            0,
        )
    };
    
    if success == 0 {
        return Err(IoError::last_os_error());
    }
    
    let read_fd = unsafe { FileDescriptor::from_raw_handle(read_handle) };
    let write_fd = unsafe { FileDescriptor::from_raw_handle(write_handle) };
    
    Ok((read_fd, write_fd))
}

impl Pty {
    /// Spawn a command in the PTY
    pub fn spawn(&mut self, cmd: CmdBuilder) -> Result<()> {
        // Create a process with the ConPTY as its console
        let mut command = Command::new(cmd.program);
        
        // Add arguments
        if let Some(args) = cmd.args {
            command.args(args);
        }
        
        // Set working directory
        if let Some(cwd) = cmd.cwd {
            command.current_dir(cwd);
        }
        
        // Set environment variables
        if let Some(env) = cmd.env {
            command.envs(env);
        }
        
        // TODO: Implement Windows-specific process creation with ConPTY
        // This requires using the Windows API to create a process with the ConPTY
        
        // For now, just spawn a regular process
        let child = command.spawn().context("Failed to spawn command")?;
        self.child = Some(child);
        
        Ok(())
    }
    
    /// Resize the PTY
    pub fn resize(&self, size: PtySize) -> Result<()> {
        let conpty_size = COORD {
            X: size.cols as i16,
            Y: size.rows as i16,
        };
        
        // Update the ConPTY size
        let result = unsafe {
            windows::Win32::System::Console::ResizePseudoConsole(
                self.conpty_handle,
                conpty_size,
            )
        };
        
        if let Err(err) = result {
            return Err(anyhow::anyhow!("Failed to resize ConPTY: {:?}", err));
        }
        
        // Update the stored size
        *self.size.lock() = size;
        
        Ok(())
    }
    
    /// Get the current size of the PTY
    pub fn get_size(&self) -> PtySize {
        *self.size.lock()
    }
    
    /// Create read/write streams for the PTY
    pub fn create_streams(&self) -> (Receiver<Vec<u8>>, Sender<Vec<u8>>) {
        let (write_tx, write_rx) = flume::unbounded();
        let (read_tx, read_rx) = flume::unbounded();
        
        // Clone file descriptors for the background tasks
        let input_write_fd = FileDescriptor::dup(&self.input_write_fd).unwrap();
        let output_read_fd = FileDescriptor::dup(&self.output_read_fd).unwrap();
        
        // Background task for writing to the PTY
        tokio::spawn(async move {
            let mut input_writer = tokio::fs::File::from_std(input_write_fd.into_std_file());
            
            while let Ok(data) = write_rx.recv_async().await {
                if let Err(e) = input_writer.write_all(&data).await {
                    error!("Failed to write to PTY: {}", e);
                    break;
                }
                
                if let Err(e) = input_writer.flush().await {
                    error!("Failed to flush PTY writer: {}", e);
                    break;
                }
            }
        });
        
        // Background task for reading from the PTY
        tokio::spawn(async move {
            let mut output_reader = tokio::fs::File::from_std(output_read_fd.into_std_file());
            let mut buffer = [0u8; 4096];
            
            loop {
                match output_reader.read(&mut buffer).await {
                    Ok(0) => {
                        debug!("PTY read returned 0 bytes, EOF reached");
                        break;
                    }
                    Ok(n) => {
                        if let Err(e) = read_tx.send_async(buffer[..n].to_vec()).await {
                            error!("Failed to send PTY output: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from PTY: {}", e);
                        break;
                    }
                }
                
                // Small delay to prevent CPU spinning
                sleep(Duration::from_millis(10)).await;
            }
        });
        
        (read_rx, write_tx)
    }
}
