use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use fig_proto::{FigProtobufEncodable, prost::Message, ReflectMessage};
use fig_util::directories::{
    sockets_dir,
    DirectoryError,
};
use tokio::io::{
    AsyncRead,
    AsyncWrite,
};
use tracing::{
    debug,
    trace,
};

use crate::{
    Error,
    RecvError,
    RecvMessage,
    SendError,
    SendMessage,
    SendRecvMessage,
    BufferedStream,
    connect,
    connect_timeout,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A local socket connection
pub struct LocalSocket<T> {
    socket: T,
    path: PathBuf,
}

impl<T> LocalSocket<T> {
    /// Create a new local socket
    pub fn new(socket: T, path: PathBuf) -> Self {
        Self { socket, path }
    }
}

/// Connect to a local socket
pub async fn connect_local(name: &str) -> Result<LocalSocket<BufferedStream>, Error> {
    let path = socket_path(name)?;
    debug!(?path, "Connecting to local socket");
    let socket = connect(&path).await?;
    let buffered_socket = BufferedStream::new(socket);
    trace!(?path, "Connected to local socket");
    Ok(LocalSocket::new(buffered_socket, path))
}

/// Connect to a local socket with a timeout
pub async fn connect_local_timeout(name: &str, timeout: Duration) -> Result<LocalSocket<BufferedStream>, Error> {
    let path = socket_path(name)?;
    debug!(?path, ?timeout, "Connecting to local socket with timeout");
    let socket = connect_timeout(&path, timeout).await?;
    let buffered_socket = BufferedStream::new(socket);
    trace!(?path, "Connected to local socket");
    Ok(LocalSocket::new(buffered_socket, path))
}

/// Connect to a local socket with a default timeout
pub async fn connect_local_with_default_timeout(name: &str) -> Result<LocalSocket<BufferedStream>, Error> {
    connect_local_timeout(name, DEFAULT_TIMEOUT).await
}

/// Get the socket path for a local socket
pub fn socket_path(name: &str) -> Result<PathBuf, DirectoryError> {
    let mut path = sockets_dir()?;
    path.push(name);
    Ok(path)
}

#[async_trait]
impl<T> SendMessage for LocalSocket<T>
where
    T: AsyncWrite + Unpin + Send + SendMessage,
{
    async fn send_message<M>(&mut self, message: M) -> Result<(), SendError>
    where
        M: FigProtobufEncodable + Send,
    {
        self.socket.send_message(message).await
    }
}

#[async_trait]
impl<T> RecvMessage for LocalSocket<T>
where
    T: AsyncRead + Unpin + Send + RecvMessage,
{
    async fn recv_message<R>(&mut self) -> Result<Option<R>, RecvError>
    where
        R: Message + ReflectMessage + Default + Send,
    {
        self.socket.recv_message().await
    }
}

impl<T> SendRecvMessage for LocalSocket<T> where T: AsyncRead + AsyncWrite + Unpin + Send + SendMessage + RecvMessage {}

impl<T> Drop for LocalSocket<T> {
    fn drop(&mut self) {
        trace!(?self.path, "Dropping local socket");
    }
}
