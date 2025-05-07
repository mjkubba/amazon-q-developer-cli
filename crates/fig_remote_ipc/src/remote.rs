use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

use anyhow::{
    Context,
    Result,
};
use fig_ipc::{
    BufferedStream,
    RecvMessage,
    SendMessage,
};
use fig_proto::figterm::{
    InsertTextRequest,
    InterceptRequest,
    SetBufferRequest,
    intercept_request,
};
use fig_proto::local::ShellContext;
use fig_proto::remote::clientbound::request::Request;
use fig_proto::remote::clientbound::{
    self,
    HandshakeResponse,
};
use fig_proto::remote::{
    Clientbound,
    Hostbound,
    RunProcessRequest,
    hostbound,
};
use fig_util::PTY_BINARY_NAME;
use time::OffsetDateTime;
#[cfg(unix)]
use tokio::net::{
    UnixListener,
    UnixStream,
};
#[cfg(windows)]
use tokio::net::windows::named_pipe::{
    ClientOptions,
    ServerOptions,
    NamedPipeClient,
    NamedPipeServer,
};
use tokio::select;
use tokio::sync::Notify;
use tokio::time::{
    Duration,
    Instant,
    MissedTickBehavior,
};
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};
