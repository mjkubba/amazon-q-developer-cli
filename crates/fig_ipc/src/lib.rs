pub mod local;

mod error;

mod buffered_reader;
mod codec;
mod recv_message;
mod send_message;
mod send_recv_message;

#[cfg(unix)]
mod unix_socket;
#[cfg(windows)]
mod windows_pipe;

pub use buffered_reader::BufferedReader;
pub use codec::Base64LineCodec;
pub use error::{
    ConnectError,
    Error,
    RecvError,
    SendError,
};
pub use recv_message::RecvMessage;
pub use send_message::SendMessage;
pub use send_recv_message::SendRecvMessage;

#[cfg(unix)]
pub use unix_socket::{
    BufferedUnixStream as BufferedStream,
    socket_connect as connect,
    socket_connect_timeout as connect_timeout,
    validate_socket as validate,
};

#[cfg(windows)]
pub use windows_pipe::{
    BufferedNamedPipeClient as BufferedStream,
    pipe_connect as connect,
    pipe_connect_timeout as connect_timeout,
    validate_pipe as validate,
};
