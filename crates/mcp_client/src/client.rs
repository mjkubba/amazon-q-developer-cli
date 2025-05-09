use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{
    AtomicBool,
    AtomicU64,
    Ordering,
};
use std::sync::{
    Arc,
    RwLock as SyncRwLock,
};
use std::time::Duration;

#[cfg(unix)]
use nix::sys::signal::Signal;
#[cfg(unix)]
use nix::unistd::Pid;

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Failed to connect to server: {0}")]
    ConnectionFailed(String),
    #[error("Failed to send message: {0}")]
    SendFailed(String),
    #[error("Failed to receive message: {0}")]
    ReceiveFailed(String),
    #[error("Failed to parse message: {0}")]
    ParseFailed(String),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Server process exited with code {0}")]
    ServerExited(i32),
    #[error("Server process was killed")]
    ServerKilled,
    #[error("Server process exited without a code")]
    ServerExitedWithoutCode,
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_name: String,
    pub bin_path: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub client_info: ClientInfo,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<ResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub method: String,
    pub params: serde_json::Value,
}

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self) -> Result<(), ClientError>;
    async fn send(&mut self, data: &[u8]) -> Result<(), ClientError>;
    async fn receive(&mut self) -> Result<Vec<u8>, ClientError>;
    async fn close(&mut self) -> Result<(), ClientError>;
}

// Define a platform-agnostic process ID type
#[cfg(unix)]
pub type ProcessId = Pid;
#[cfg(not(unix))]
pub type ProcessId = u32;

pub struct Client<T>
where
    T: Transport,
{
    transport: T,
    server_info: Option<ServerInfo>,
    request_id: AtomicU64,
    #[cfg(unix)]
    server_process_id: Option<ProcessId>,
    #[cfg(not(unix))]
    server_process_id: Option<ProcessId>,
    connected: AtomicBool,
    pending_requests: Arc<SyncRwLock<HashMap<String, tokio::sync::oneshot::Sender<Response>>>>,
}

impl<T> Client<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            server_info: None,
            request_id: AtomicU64::new(0),
            server_process_id: None,
            connected: AtomicBool::new(false),
            pending_requests: Arc::new(SyncRwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self) -> Result<ServerInfo, ClientError> {
        self.transport.connect().await?;
        self.connected.store(true, Ordering::SeqCst);
        let server_info = self.get_server_info().await?;
        self.server_info = Some(server_info.clone());
        Ok(server_info)
    }

    pub async fn start_server(config: ServerConfig) -> Result<(Self, ServerInfo), ClientError>
    where
        T: From<tokio::process::ChildStdout> + From<tokio::process::ChildStdin>,
    {
        let ServerConfig {
            server_name,
            bin_path,
            args,
            timeout,
            client_info,
            env,
        } = config;
        let child = {
            let mut command = tokio::process::Command::new(bin_path);
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
                
            #[cfg(unix)]
            command.process_group(0);
                
            command.envs(std::env::vars());
            if let Some(env) = env {
                for (env_name, env_value) in env {
                    command.env(env_name, env_value);
                }
            }
            command.args(args).spawn()?
        };

        let stdin = child.stdin.expect("Failed to open stdin");
        let stdout = child.stdout.expect("Failed to open stdout");

        // Create the server process ID based on the platform
        #[cfg(unix)]
        let server_process_id = Some(Pid::from_raw(child.id() as i32));
        
        // On Windows, we need to use a different approach
        #[cfg(not(unix))]
        let server_process_id = Some(child.id());

        let mut client = Self {
            transport: T::from(stdout),
            server_info: None,
            request_id: AtomicU64::new(0),
            server_process_id,
            connected: AtomicBool::new(false),
            pending_requests: Arc::new(SyncRwLock::new(HashMap::new())),
        };

        client.connect().await?;
        let server_info = client.server_info.clone().unwrap();

        Ok((client, server_info))
    }
}

impl<T> Drop for Client<T>
where
    T: Transport,
{
    // IF the servers are implemented well, they will shutdown once the pipe closes.
    // This drop trait is here as a fail safe to ensure we don't leave behind any orphans.
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(process_id) = self.server_process_id {
            let _ = nix::sys::signal::kill(process_id, Signal::SIGTERM);
        }
        
        #[cfg(windows)]
        if let Some(process_id) = self.server_process_id {
            // On Windows, we use the Windows API to terminate the process
            use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
            use windows::Win32::Foundation::{BOOL, CloseHandle};
            
            unsafe {
                // Open the process with termination rights
                let process_handle = OpenProcess(PROCESS_TERMINATE, false, process_id);
                if let Ok(handle) = process_handle {
                    if !handle.is_invalid() {
                        // Terminate the process with exit code 1
                        let _ = TerminateProcess(handle, 1);
                        let _ = CloseHandle(handle);
                    } else {
                        tracing::warn!("Failed to open process for termination: invalid handle");
                    }
                } else {
                    tracing::warn!("Failed to open process for termination: {:?}", process_handle.err());
                }
            }
        }
    }
}

impl<T> Client<T>
where
    T: Transport,
{
    pub fn get_server_name(&self) -> Option<&str> {
        self.server_info.as_ref().map(|info| info.name.as_str())
    }

    pub async fn get_server_info(&mut self) -> Result<ServerInfo, ClientError> {
        let request = Request {
            id: self.next_request_id(),
            method: "server.info".to_string(),
            params: serde_json::json!({}),
        };

        let response = self.send_request(request).await?;
        let result = response
            .result
            .ok_or_else(|| ClientError::ServerError("No result in response".to_string()))?;

        let server_info: ServerInfo = serde_json::from_value(result)?;
        Ok(server_info)
    }

    fn next_request_id(&self) -> String {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        id.to_string()
    }

    pub async fn send_request(&mut self, request: Request) -> Result<Response, ClientError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending_requests = self.pending_requests.write().unwrap();
            pending_requests.insert(request.id.clone(), tx);
        }

        let request_json = serde_json::to_string(&request)?;
        self.transport.send(request_json.as_bytes()).await?;

        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(ClientError::ReceiveFailed(
                "Response channel closed".to_string(),
            )),
            Err(_) => Err(ClientError::Timeout("Request timed out".to_string())),
        }
    }

    pub async fn send_notification(&mut self, notification: Notification) -> Result<(), ClientError> {
        let notification_json = serde_json::to_string(&notification)?;
        self.transport.send(notification_json.as_bytes()).await
    }

    pub async fn receive_message(&mut self) -> Result<(), ClientError> {
        let data = self.transport.receive().await?;
        let message_str = String::from_utf8(data)?;

        if let Ok(response) = serde_json::from_str::<Response>(&message_str) {
            let id = response.id.clone();
            let mut pending_requests = self.pending_requests.write().unwrap();
            if let Some(tx) = pending_requests.remove(&id) {
                let _ = tx.send(response);
            }
        } else if let Ok(_notification) = serde_json::from_str::<Notification>(&message_str) {
            // Handle notification if needed
        } else {
            return Err(ClientError::ParseFailed(format!(
                "Failed to parse message: {}",
                message_str
            )));
        }

        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), ClientError> {
        self.transport.close().await
    }
}
