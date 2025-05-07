#![cfg(unix)] // Only compile this module on Unix platforms

use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{
    anyhow,
    Context,
    Result,
};
use fig_proto::figterm::{
    EditBufferRequest,
    HideRequest,
    InsertTextRequest,
    InterceptRequest,
    SetBufferRequest,
    ShowRequest,
    intercept_request,
};
use fig_proto::local::{
    LocalMessage,
    LocalRequest,
    LocalResponse,
    ShellContext,
    local_request,
    local_response,
};
use fig_proto::remote::{
    Clientbound,
    Hostbound,
    RunProcessRequest,
};
use fig_util::directories::fig_runtime_dir;
use fig_util::PTY_BINARY_NAME;
use tokio::io::{
    AsyncBufReadExt,
    AsyncWriteExt,
    BufReader,
};
use tokio::net::UnixListener;
use tokio::process::{
    Child,
    Command,
};
use tokio::sync::Mutex;
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

use crate::figterm_state::FigtermState;

pub struct Shell {
    pub child: Child,
    pub socket_path: PathBuf,
}

impl Shell {
    pub async fn new(figterm_state: Arc<Mutex<FigtermState>>) -> Result<Self> {
        let runtime_dir = fig_runtime_dir()?;
        tokio::fs::create_dir_all(&runtime_dir).await?;
        tokio::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700)).await?;

        let socket_path = runtime_dir.join("figterm.sock");
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        tokio::task::spawn(async move {
            let (stream, _) = match listener.accept().await {
                Ok(stream) => stream,
                Err(err) => {
                    error!(%err, "Failed to accept connection");
                    return;
                },
            };

            let (reader, mut writer) = tokio::io::split(stream);
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("Connection closed");
                        break;
                    },
                    Ok(_) => {
                        let message = match serde_json::from_str::<LocalMessage>(&line) {
                            Ok(message) => message,
                            Err(err) => {
                                error!(%err, ?line, "Failed to parse message");
                                continue;
                            },
                        };

                        let request = match message.request {
                            Some(request) => request,
                            None => {
                                error!("No request in message");
                                continue;
                            },
                        };

                        let response = match request.request {
                            Some(local_request::Request::Intercept(request)) => {
                                let mut state = figterm_state.lock().await;
                                state.intercept_request = Some(request.clone());
                                state.intercept_response = None;

                                let response = match request.request {
                                    Some(intercept_request::Request::Show(ShowRequest {})) => {
                                        state.visible = true;
                                        LocalResponse {
                                            response: Some(local_response::Response::Intercept(fig_proto::figterm::InterceptResponse {
                                                response: Some(fig_proto::figterm::intercept_response::Response::Show(fig_proto::figterm::ShowResponse {})),
                                            })),
                                        }
                                    },
                                    Some(intercept_request::Request::Hide(HideRequest {})) => {
                                        state.visible = false;
                                        LocalResponse {
                                            response: Some(local_response::Response::Intercept(fig_proto::figterm::InterceptResponse {
                                                response: Some(fig_proto::figterm::intercept_response::Response::Hide(fig_proto::figterm::HideResponse {})),
                                            })),
                                        }
                                    },
                                    Some(intercept_request::Request::SetBuffer(SetBufferRequest { buffer, cursor_position })) => {
                                        state.buffer = buffer.clone();
                                        state.cursor_position = cursor_position;
                                        LocalResponse {
                                            response: Some(local_response::Response::Intercept(fig_proto::figterm::InterceptResponse {
                                                response: Some(fig_proto::figterm::intercept_response::Response::SetBuffer(fig_proto::figterm::SetBufferResponse {})),
                                            })),
                                        }
                                    },
                                    Some(intercept_request::Request::EditBuffer(EditBufferRequest { buffer, cursor_position })) => {
                                        state.buffer = buffer.clone();
                                        state.cursor_position = cursor_position;
                                        LocalResponse {
                                            response: Some(local_response::Response::Intercept(fig_proto::figterm::InterceptResponse {
                                                response: Some(fig_proto::figterm::intercept_response::Response::EditBuffer(fig_proto::figterm::EditBufferResponse {})),
                                            })),
                                        }
                                    },
                                    Some(intercept_request::Request::InsertText(InsertTextRequest { text })) => {
                                        state.buffer.insert_str(state.cursor_position as usize, &text);
                                        state.cursor_position += text.len() as u32;
                                        LocalResponse {
                                            response: Some(local_response::Response::Intercept(fig_proto::figterm::InterceptResponse {
                                                response: Some(fig_proto::figterm::intercept_response::Response::InsertText(fig_proto::figterm::InsertTextResponse {})),
                                            })),
                                        }
                                    },
                                    None => {
                                        error!("No request in intercept request");
                                        LocalResponse {
                                            response: Some(local_response::Response::Error(fig_proto::local::ErrorResponse {
                                                message: "No request in intercept request".to_string(),
                                            })),
                                        }
                                    },
                                };

                                state.intercept_response = Some(response.clone());
                                response
                            },
                            Some(local_request::Request::Echo(request)) => LocalResponse {
                                response: Some(local_response::Response::Echo(fig_proto::local::EchoResponse {
                                    data: request.data,
                                })),
                            },
                            None => {
                                error!("No request in request");
                                LocalResponse {
                                    response: Some(local_response::Response::Error(fig_proto::local::ErrorResponse {
                                        message: "No request in request".to_string(),
                                    })),
                                }
                            },
                        };

                        let response = LocalMessage {
                            request: None,
                            response: Some(response),
                        };

                        let response = serde_json::to_string(&response).unwrap();
                        if let Err(err) = writer.write_all(response.as_bytes()).await {
                            error!(%err, "Failed to write response");
                            break;
                        }
                        if let Err(err) = writer.write_all(b"\n").await {
                            error!(%err, "Failed to write newline");
                            break;
                        }
                    },
                    Err(err) => {
                        error!(%err, "Failed to read line");
                        break;
                    },
                }
            }
        });

        let path = socket_path.clone();
        tokio::task::spawn(async move {
            struct RemoteHook {}

            #[async_trait::async_trait]
            impl fig_remote_ipc::remote::RemoteHook for RemoteHook {
                async fn on_run_process(&self, _request: &RunProcessRequest) -> Result<Clientbound> {
                    Ok(Clientbound::RunProcessResponse(fig_proto::remote::RunProcessResponse {}))
                }

                async fn on_hostbound(&self, _message: &Hostbound) -> Result<()> {
                    Ok(())
                }
            }

            if let Err(err) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                fig_remote_ipc::remote::start_remote_ipc(path, figterm_state.clone(), RemoteHook {}).await
            })
            .await
            {
                error!(%err, "Remote IPC timed out");
            }
        });

        let mut child = Command::new(PTY_BINARY_NAME)
            .arg("--socket")
            .arg(&socket_path)
            .arg("--")
            .arg("bash")
            .arg("-c")
            .arg("echo 'Hello, world!'")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn figterm")?;

        // Wait for the child to exit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Check if the child is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(anyhow!("figterm exited with status: {}", status));
            },
            Ok(None) => {},
            Err(err) => {
                return Err(anyhow!("Failed to check if figterm is running: {}", err));
            },
        }

        Ok(Self {
            child,
            socket_path,
        })
    }
}
