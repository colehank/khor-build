//! Paired residents, local IPC and real QUIC resolve terminal data on its home.
//! The TCP host is an owned protocol fixture; MCP tests exercise the real PTY.
mod util;
use khor_node::{
    Node,
    host::{self, ClientOp, Hello, HostFile, RingFile, Welcome},
    live::{Ending, LiveKind},
    terminal_control::{HostOutput, HostReply, HostRequest, Reply, Request},
};
use std::{
    net::TcpListener,
    path::Path,
    thread,
    time::{Duration, Instant},
};
use tokio::time::timeout;

struct Resident(tokio::task::JoinHandle<Result<(), String>>);
impl Drop for Resident {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn resident(root: &Path, name: &str) -> Resident {
    let node = Node::open_as(root.to_path_buf(), name).unwrap();
    let resident = Resident(tokio::spawn(async move { node.serve().await }));
    timeout(Duration::from_secs(90), async {
        while !root.join(".khor/endpoint.json").exists() {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    resident
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn monitor_and_ended_content_cross_the_execution_home_instead_of_reading_local_disk() {
    let source_root = tempfile::tempdir().unwrap();
    let caller_root = tempfile::tempdir().unwrap();
    util::hermetic(source_root.path());
    util::hermetic(caller_root.path());
    let _source = resident(source_root.path(), "source").await;
    let source = Node::open_as(source_root.path().to_path_buf(), "source").unwrap();
    let caller = Node::open_as(caller_root.path().to_path_buf(), "caller").unwrap();
    timeout(
        Duration::from_secs(60),
        caller.pair(&source.invite().unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    let _caller = resident(caller_root.path(), "caller").await;
    // A visible label may collide with the source's canonical channel.
    // Home-ID requests must still authenticate and cache the actual source.
    caller
        .rename_machine(caller.device_str(), "source")
        .unwrap();
    let id = source
        .open_ephemeral("shell", "remote terminal fixture")
        .unwrap();
    let live = LiveKind::new(source_root.path().to_path_buf(), source.device());
    let directory = live.dir_of(&id).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    std::fs::write(
        host::host_file_path(&directory),
        serde_json::to_vec(&HostFile {
            port: listener.local_addr().unwrap().port(),
            cookie: "owned-wire-cookie".into(),
            host_pid: std::process::id(),
            child_pid: std::process::id(),
        })
        .unwrap(),
    )
    .unwrap();
    let peer = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("owned fixture listener failed: {error}"),
            }
            assert!(
                Instant::now() < deadline,
                "Remote monitor never reached the execution host"
            );
            thread::sleep(Duration::from_millis(20));
        };
        // BSD accept inherits the listener's O_NONBLOCK; Linux does not.
        // Framed IO below is blocking with explicit bounded timeouts on both.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let hello: Hello = host::read_frame(&mut stream).unwrap();
        assert_eq!(hello.cookie, "owned-wire-cookie");
        assert!(
            hello.control,
            "Observation must use the host's no-resize control channel"
        );
        assert_eq!((hello.cols, hello.rows), (0, 0));
        host::write_frame(
            &mut stream,
            &Welcome {
                ok: true,
                why: format!("khor-host/{}", host::CONTROL_VERSION),
            },
        )
        .unwrap();
        assert!(matches!(
            host::read_frame::<ClientOp>(&mut stream).unwrap(),
            ClientOp::Version(host::CONTROL_VERSION)
        ));
        assert!(matches!(
            host::read_frame::<HostRequest>(&mut stream).unwrap(),
            HostRequest::Read {
                limit: 4096,
                wait_ms: 0,
                ..
            }
        ));
        host::write_frame(
            &mut stream,
            &HostReply::Output(HostOutput {
                bytes: b"output on source machine".to_vec(),
                cursor: 24,
                end: 24,
                reset: true,
                input_revision: 9,
                generation: "1".repeat(64),
                shell_ready: true,
                startup_released: true,
            }),
        )
        .unwrap();
    });
    let answer = timeout(
        Duration::from_secs(60),
        caller.terminal_on(
            source.device_str(),
            Request::Read {
                session: id.0.clone(),
                cursor: None,
                limit: 4096,
                wait_ms: 0,
            },
        ),
    )
    .await
    .unwrap()
    .unwrap();
    let Reply::Read { session, output } = answer else {
        panic!("Expected monitor output")
    };
    assert_eq!(session.home, source.device());
    assert_eq!(output.bytes, b"output on source machine");
    assert_eq!(output.input_revision, 9);
    assert_eq!(output.generation, "1".repeat(64));
    peer.join().unwrap();

    let saved = b"saved final output on source\r\n".to_vec();
    host::write_ring_file(
        &directory,
        &RingFile {
            form: host::RING_FORM_SNAPSHOT,
            cols: 103,
            rows: 31,
            at_ms: 123,
            bytes: saved.clone(),
        },
    )
    .unwrap();
    std::fs::write(
        host::host_error_path(&directory),
        "Source startup diagnostic",
    )
    .unwrap();
    live.record_exit(&id, 7).unwrap();
    timeout(Duration::from_secs(60), caller.sync_now())
        .await
        .unwrap()
        .unwrap();
    assert!(
        !caller_root
            .path()
            .join(".khor/sessions")
            .join(id.0.replace('/', "-"))
            .exists(),
        "The observer must not acquire a fake local terminal registry"
    );
    let ended = timeout(Duration::from_secs(60), caller.ended_anywhere(&id))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(ended.ending, Ending::Exit(7)));
    assert_eq!(
        ended.startup_error.as_deref(),
        Some("Source startup diagnostic")
    );
    let screen = ended.screen.unwrap();
    assert_eq!((screen.cols, screen.rows), (103, 31));
    assert_eq!(screen.bytes, saved);
    // The fixture host PID is this test process: never call Node::close on it.
    std::fs::remove_file(host::host_file_path(&directory)).unwrap();
}
