use std::io::{
    BufReader,
    Read,
    Write,
};

use async_trait::async_trait;
use tokio::io::{
    AsyncBufReadExt,
    AsyncReadExt,
    AsyncWriteExt,
    BufReader as TokioBufReader,
};

use crate::client::{
    ClientError,
    Transport,
};

pub struct StdioTransport {
    stdin: tokio::process::ChildStdin,
    stdout: TokioBufReader<tokio::process::ChildStdout>,
}

impl From<tokio::process::ChildStdout> for StdioTransport {
    fn from(stdout: tokio::process::ChildStdout) -> Self {
        Self {
            stdin: tokio::process::ChildStdin::from_std(std::io::stdin()).unwrap(),
            stdout: TokioBufReader::new(stdout),
        }
    }
}

impl From<tokio::process::ChildStdin> for StdioTransport {
    fn from(stdin: tokio::process::ChildStdin) -> Self {
        Self {
            stdin,
            stdout: TokioBufReader::new(tokio::process::ChildStdout::from_std(std::io::stdout()).unwrap()),
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn connect(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), ClientError> {
        self.stdin.write_all(data).await.map_err(|e| ClientError::SendFailed(e.to_string()))?;
        self.stdin.write_all(b"\n").await.map_err(|e| ClientError::SendFailed(e.to_string()))?;
        self.stdin.flush().await.map_err(|e| ClientError::SendFailed(e.to_string()))?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, ClientError> {
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .await
            .map_err(|e| ClientError::ReceiveFailed(e.to_string()))?;
        Ok(line.into_bytes())
    }

    async fn close(&mut self) -> Result<(), ClientError> {
        Ok(())
    }
}
