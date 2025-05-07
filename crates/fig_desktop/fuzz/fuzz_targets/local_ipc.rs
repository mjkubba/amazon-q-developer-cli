#![no_main]
#![cfg(unix)] // Only run this fuzzing target on Unix platforms

use std::path::PathBuf;
use std::sync::Once;

use fig_ipc::{
    BufferedStream,
    connect,
};
use fig_proto::local::LocalMessage;
use libfuzzer_sys::fuzz_target;
use tokio::net::UnixListener;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

static RUNTIME: once_cell::sync::Lazy<Runtime> = once_cell::sync::Lazy::new(|| Runtime::new().unwrap());
static INIT: Once = Once::new();
static DIRSOCK: once_cell::sync::Lazy<(PathBuf, PathBuf)> = once_cell::sync::Lazy::new(|| {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("socket");
    (dir.into_path(), path)
});

fn init() {
    INIT.call_once(|| {
        let (tx, rx) = oneshot::channel();
        RUNTIME.spawn(async move {
            let listener = UnixListener::bind(&DIRSOCK.1).unwrap();
            tx.send(()).unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = BufferedStream::new(stream);
            let _: LocalMessage = stream.recv_message().await.unwrap().unwrap();
        });
        rx.blocking_recv().unwrap();
    });
}

fuzz_target!(|data: Vec<u8>| {
    if data.is_empty() {
        return;
    }

    init();

    RUNTIME.block_on(async {
        let mut stream = connect(&DIRSOCK.1).await.unwrap();
        let msg = LocalMessage {
            request: Some(fig_proto::local::LocalRequest {
                request: Some(fig_proto::local::local_request::Request::Echo(
                    fig_proto::local::EchoRequest { data },
                )),
            }),
            response: None,
        };
        stream.send_message(msg).await.unwrap();
    });
});
