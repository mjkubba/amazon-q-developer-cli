use std::path::Path;
use std::time::Duration;
use std::io;

use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tracing::{error, trace};

use crate::{BufferedReader, ConnectError};

/// Validates that the named pipe path is valid
pub async fn validate_pipe(_pipe_path: impl AsRef<Path>) -> Result<(), ConnectError> {
    // Windows named pipes have different security model than Unix sockets
    // For now, we'll just return Ok as Windows handles permissions differently
    Ok(())
}

/// Connects to a Windows named pipe
pub async fn pipe_connect(pipe_path: impl AsRef<Path>) -> Result<NamedPipeClient, ConnectError> {
    let pipe_path = pipe_path.as_ref();
    let pipe_name = format!(r"\\.\pipe\{}", pipe_path.to_string_lossy());

    validate_pipe(&pipe_path).await?;

    let client = match ClientOptions::new().open(&pipe_name) {
        Ok(client) => client,
        Err(err) => {
            error!(%err, ?pipe_path, "Failed to connect to named pipe");
            return Err(ConnectError::Io(err));
        },
    };

    trace!(?pipe_path, "Connected to named pipe");

    Ok(client)
}

/// Connects to a Windows named pipe with a timeout
pub async fn pipe_connect_timeout(pipe_path: impl AsRef<Path>, timeout: Duration) -> Result<NamedPipeClient, ConnectError> {
    let pipe_path = pipe_path.as_ref();
    match tokio::time::timeout(timeout, pipe_connect(&pipe_path)).await {
        Ok(Ok(conn)) => Ok(conn),
        Ok(Err(err)) => Err(err),
        Err(_) => {
            error!(?pipe_path, ?timeout, "Timeout while connecting to named pipe");
            Err(ConnectError::Timeout)
        },
    }
}

pub type BufferedNamedPipeClient = BufferedReader<NamedPipeClient>;

impl BufferedNamedPipeClient {
    /// Connect to a Windows named pipe
    pub async fn connect(pipe_path: impl AsRef<Path>) -> Result<Self, ConnectError> {
        Ok(Self::new(pipe_connect(pipe_path).await?))
    }

    /// Connect to a Windows named pipe with a timeout
    pub async fn connect_timeout(pipe_path: impl AsRef<Path>, timeout: Duration) -> Result<Self, ConnectError> {
        Ok(Self::new(pipe_connect_timeout(pipe_path, timeout).await?))
    }
}
