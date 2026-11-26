//! Actual MCP JSON-RPC, durable hosts and normal viewer input in a private store.
use khor_node::host::{self, ClientOp};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, ExitStatus, Output, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn wait_for_exit(child: &mut Child, timeout: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "CLI exit deadline"));
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn bounded_output(home: &Path, command: &mut Command) -> io::Result<Output> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let stdout = home.join(format!("cleanup-{serial}.stdout.log"));
    let stderr = home.join(format!("cleanup-{serial}.stderr.log"));
    let mut child = command
        .stdin(Stdio::null())
        .stdout(File::create_new(&stdout)?)
        .stderr(File::create_new(&stderr)?)
        .spawn()?;
    let status = match wait_for_exit(&mut child, Duration::from_secs(25)) {
        Ok(status) => status,
        Err(error) => {
            let killed = child.kill();
            let reaped = wait_for_exit(&mut child, Duration::from_secs(5));
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "{error}; kill={killed:?}, reap={reaped:?}; output: {stdout:?}, {stderr:?}"
                ),
            ));
        }
    };
    // A descendant can retain an inherited file handle. Reading a regular
    // file does not wait for every writer to close it, unlike pipe EOF.
    Ok(Output {
        status,
        stdout: fs::read(stdout)?,
        stderr: fs::read(stderr)?,
    })
}

fn remove_fixture_home(home: &Path) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match fs::remove_dir_all(home) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound && !home.exists() => {
                return Ok(());
            }
            Err(error) if Instant::now() >= deadline => return Err(error),
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }
}

struct Fixture {
    home: PathBuf,
    server: Child,
    input: Option<ChildStdin>,
    replies: mpsc::Receiver<Result<Value, String>>,
    reader: Option<thread::JoinHandle<()>>,
    request: u64,
    sessions: Vec<String>,
    tasks: Vec<String>,
}

impl Fixture {
    fn new() -> Self {
        Self::new_with_shell(None)
    }

    fn new_with_shell(shell: Option<&Path>) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home =
            std::env::temp_dir().join(format!("khor-mcp-terminal-{}-{stamp}", std::process::id()));
        fs::create_dir(&home).unwrap();
        fs::create_dir(home.join(".khor")).unwrap();
        fs::create_dir(home.join("tmux")).unwrap();
        fs::write(
            home.join(".khor/relays.json"),
            br#"{"relays":[],"only_mine":true}"#,
        )
        .unwrap();
        let mut command = Self::command_at(&home);
        if let Some(shell) = shell {
            command.env("KHOR_SHELL", shell);
        }
        let mut server = command
            .args(["mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(home.join("mcp-stderr.log")).unwrap())
            .spawn()
            .unwrap();
        let input = server.stdin.take();
        let output = server.stdout.take().unwrap();
        let (send, replies) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let reply = line.map_err(|error| error.to_string()).and_then(|line| {
                    serde_json::from_str(&line)
                        .map_err(|error| format!("Invalid MCP reply: {error}: {line}"))
                });
                if send.send(reply).is_err() {
                    break;
                }
            }
        });
        let mut fixture = Self {
            home,
            server,
            input,
            replies,
            reader: Some(reader),
            request: 0,
            sessions: Vec::new(),
            tasks: Vec::new(),
        };
        let initialized = fixture.rpc(
            "initialize",
            json!({
                "protocolVersion":"2024-11-05","capabilities":{},
                "clientInfo":{"name":"terminal-regression","version":"1"},
            }),
        );
        assert_eq!(initialized["serverInfo"]["name"], "khor");
        fixture.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        fixture
    }

    fn command_at(home: &PathBuf) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_khor"));
        command
            .env("KHOR_HOME", home)
            .env("KHOR_NAME", "mcp-terminal-test")
            .env("KHOR_RUNTIME_DIR", home.join("runtime"))
            .env("KHOR_RUNTIME_INSTALL_SOURCE", env!("CARGO_BIN_EXE_khor"))
            .env("KHOR_SHELL_LOGIN", "never")
            .env("KHOR_NO_MDNS", "1")
            .env("TMUX_TMPDIR", home.join("tmux"))
            .env("KHOR_TMUX_SOCKET", "khor-mcp-test")
            .env_remove("TMUX")
            .env_remove("TMUX_PANE")
            .env_remove("KHOR_SESSION")
            .env_remove("KHOR_SHELL_INTEGRATION")
            .env_remove("ENV")
            .env_remove("BASH_ENV")
            .current_dir(home);
        if let Some(path) = std::env::var_os("PATH") {
            command.env("KHOR_PATH", path);
        }
        #[cfg(unix)]
        command.env("KHOR_SHELL", "/bin/sh");
        #[cfg(windows)]
        command.env(
            "KHOR_SHELL",
            PathBuf::from(std::env::var_os("SystemRoot").unwrap())
                .join("System32/WindowsPowerShell/v1.0/powershell.exe"),
        );
        command
    }

    fn send(&mut self, message: Value) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
    }

    fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.request += 1;
        let id = self.request;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            let reply = self
                .replies
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or_else(|error| {
                    panic!(
                        "MCP {method} did not answer: {error}; {}",
                        fs::read_to_string(self.home.join("mcp-stderr.log")).unwrap_or_default()
                    )
                })
                .unwrap();
            if reply["id"] != id {
                continue;
            }
            assert!(reply.get("error").is_none(), "MCP protocol error: {reply}");
            return reply["result"].clone();
        }
    }

    fn tool(&mut self, name: &str, arguments: Value) -> Result<Value, String> {
        let reply = self.rpc("tools/call", json!({"name":name,"arguments":arguments}));
        let text = reply["content"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|block| block["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if reply["isError"] == true {
            return Err(text);
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }

    fn open(&mut self, request: &str, command: &str) -> (String, Value) {
        let args = json!({"machine":"mcp-terminal-test","request":request,"command":command,
            "cwd":self.home,"title":"MCP persistent terminal"});
        let reply = self.tool("open", args.clone()).unwrap();
        let id = reply["session"].as_str().unwrap().to_owned();
        assert_eq!(reply["kind"], "terminal");
        if !self.sessions.contains(&id) {
            self.sessions.push(id.clone());
        }
        (id, args)
    }

    fn read_until(&mut self, id: &str, marker: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut cursor = None;
        let mut output = String::new();
        loop {
            let mut args = json!({"session":id,"limit":16384,"wait_ms":100});
            if let Some(value) = cursor {
                args["cursor"] = Value::from(value);
            }
            let reply = self.tool("terminal_read", args).unwrap();
            output.push_str(reply["output"]["text"].as_str().unwrap());
            if output.contains(marker) {
                return reply;
            }
            assert!(
                Instant::now() < deadline,
                "Terminal never produced {marker:?}: {output}"
            );
            cursor = reply["output"]["cursor"].as_u64();
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn read_revision_after(&mut self, id: &str, previous: u64) -> u64 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let reply = self
                .tool("terminal_read", json!({"session":id,"limit":1024}))
                .unwrap();
            let revision = reply["output"]["input_revision"].as_u64().unwrap();
            if revision > previous {
                return revision;
            }
            assert!(
                Instant::now() < deadline,
                "Viewer input never changed input revision"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn viewer(&self, id: &str) -> TcpStream {
        let directory = self.home.join(".khor/sessions").join(id.replace('/', "-"));
        // The official viewer handshake selects a raw replay/live byte stream.
        // It has no extra framed snapshot after Version.
        let stream = host::connect(&directory, 80, 24).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
    }

    fn cli(&self, args: &[&str]) -> io::Result<Output> {
        bounded_output(&self.home, Self::command_at(&self.home).args(args))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Stop admission before discovering cleanup identities: a timed-out
        // open may have created its host without delivering the response.
        self.input.take();
        let mut failures = Vec::new();
        if wait_for_exit(&mut self.server, Duration::from_secs(5)).is_err() {
            let _ = self.server.kill();
            if let Err(error) = wait_for_exit(&mut self.server, Duration::from_secs(5)) {
                failures.push(format!("MCP server did not stop: {error}"));
            }
        }
        for (directory, file, pointer, identities) in [
            ("sessions", "meta.json", "/id", &mut self.sessions),
            ("command-tasks", "task.json", "/status/id", &mut self.tasks),
        ] {
            if let Ok(entries) = fs::read_dir(self.home.join(".khor").join(directory)) {
                for entry in entries.flatten() {
                    let record = fs::read(entry.path().join(file))
                        .ok()
                        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
                    if let Some(id) = record
                        .as_ref()
                        .and_then(|v| v.pointer(pointer))
                        .and_then(Value::as_str)
                    {
                        if !identities.iter().any(|known| known == id) {
                            identities.push(id.into());
                        }
                    }
                }
            }
        }
        for session in &self.sessions {
            match self.cli(&["close", session]) {
                Ok(output) if output.status.success() => {}
                Ok(output) => failures.push(format!(
                    "close {session}: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
                Err(error) => failures.push(format!("close {session}: {error}")),
            }
        }
        for task in &self.tasks {
            match self.cli(&["execution", "cancel", task]) {
                Ok(output)
                    if output.status.success()
                        && serde_json::from_slice::<Value>(&output.stdout)
                            .ok()
                            .is_some_and(|reply| {
                                matches!(
                                    reply["task"]["phase"].as_str(),
                                    Some("completed" | "failed" | "cancelled" | "timed_out")
                                )
                            }) => {}
                Ok(output) => failures.push(format!(
                    "cancel {task}: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )),
                Err(error) => failures.push(format!("cancel {task}: {error}")),
            }
        }
        // Close all owned resources before awaiting RPC pipe EOF: their
        // descendants may still hold the server's inherited stdout handle.
        if let Some(reader) = self.reader.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !reader.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(25));
            }
            if !reader.is_finished() {
                failures.push("MCP reply reader did not finish after cleanup".into());
            } else if reader.join().is_err() {
                failures.push("MCP reply reader panicked".into());
            }
        }
        if failures.is_empty() && !thread::panicking() {
            // Windows may retain the executable handle briefly after the
            // final task receipt, until its worker has actually exited.
            if let Err(error) = remove_fixture_home(&self.home) {
                failures.push(format!("fixture directory cleanup: {error}"));
            }
        } else if failures.is_empty() {
            eprintln!("Preserving failed MCP fixture at {:?}", self.home);
        }
        if !failures.is_empty() {
            let message = format!(
                "MCP cleanup was not confirmed; preserving {:?}: {}",
                self.home,
                failures.join("; ")
            );
            eprintln!("{message}");
            if !thread::panicking() {
                panic!("{message}");
            }
        }
    }
}

#[cfg(unix)]
fn marker_command(first: &str, second: &str) -> String {
    format!("printf '%s-%s\\n' '{first}' '{second}'")
}
#[cfg(windows)]
fn marker_command(first: &str, second: &str) -> String {
    format!("Write-Output ('{first}-' + '{second}')")
}

#[cfg(unix)]
fn once_command(path: &std::path::Path) -> String {
    format!(
        "printf 'MCP-ONCE\\n' >> '{}'; {}",
        path.display().to_string().replace('\'', "'\\''"),
        marker_command("INPUT", "DONE")
    )
}
#[cfg(windows)]
fn once_command(path: &std::path::Path) -> String {
    format!(
        "Add-Content -LiteralPath '{}' -Value 'MCP-ONCE'; {}",
        path.display().to_string().replace('\'', "''"),
        marker_command("INPUT", "DONE")
    )
}

#[test]
fn actual_mcp_terminal_starts_without_a_viewer_and_coordinates_idempotent_input() {
    let mut fixture = Fixture::new();
    let tools = fixture.rpc("tools/list", json!({}));
    for tool in [
        "open",
        "terminal_read",
        "terminal_input",
        "terminal_ended",
        "execute",
    ] {
        assert!(
            tools["tools"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == tool)
        );
    }
    let (id, open_args) = fixture.open("mcp-open-terminal-1234", &marker_command("START", "READY"));
    let opened = fixture.read_until(&id, "START-READY");
    assert_eq!(opened["session"]["dead"], false);
    assert_eq!(opened["output"]["shell_ready"], true);
    assert_eq!(opened["output"]["startup_released"], true);
    assert_eq!(
        fixture.tool("open", open_args.clone()).unwrap()["session"],
        id
    );
    let mut conflict = open_args;
    conflict["command"] = Value::String(marker_command("OTHER", "COMMAND"));
    assert!(fixture.tool("open", conflict).is_err());
    let listed = fixture.tool("sessions", json!({})).unwrap();
    assert_eq!(
        listed
            .as_str()
            .unwrap()
            .lines()
            .filter(|line| line.starts_with(&format!("{id}\t")))
            .count(),
        1
    );

    let revision = opened["output"]["input_revision"].as_u64().unwrap();
    let generation = opened["output"]["generation"].as_str().unwrap().to_owned();
    let stale_generation = if generation == "0".repeat(64) {
        "1".repeat(64)
    } else {
        "0".repeat(64)
    };
    assert!(fixture.tool("terminal_input", json!({"session":id,"generation":stale_generation,"request":"mcp-input-old-host-1234","revision":revision,"text":marker_command("OLD","HOST"),"enter":true})).is_err());
    let counter = fixture.home.join("counter.txt");
    let input = json!({"session":id,"generation":generation,"request":"mcp-terminal-input-1234","revision":revision,"text":once_command(&counter),"enter":true});
    let accepted = fixture.tool("terminal_input", input.clone()).unwrap();
    assert_eq!(accepted["Input"]["generation"], generation);
    let current = accepted["Input"]["revision"].as_u64().unwrap();
    fixture.read_until(&id, "INPUT-DONE");
    assert_eq!(fixture.tool("terminal_input", input).unwrap(), accepted);
    assert!(fixture.tool("terminal_input",json!({"session":id,"generation":generation,"request":"mcp-input-stale-1234","revision":revision,"text":marker_command("STALE","INPUT"),"enter":true})).is_err());
    let barrier=fixture.tool("terminal_input",json!({"session":id,"generation":generation,"request":"mcp-input-barrier-1234","revision":current,"text":marker_command("BARRIER","DONE"),"enter":true})).unwrap();
    fixture.read_until(&id, "BARRIER-DONE");
    let recorded = fs::read_to_string(counter).unwrap();
    assert_eq!(recorded.lines().collect::<Vec<_>>(), ["MCP-ONCE"]);

    let mut viewer = fixture.viewer(&id);
    let revision = barrier["Input"]["revision"].as_u64().unwrap();
    host::write_frame(
        &mut viewer,
        &ClientOp::Input(b"unfinished-user-line".to_vec()),
    )
    .unwrap();
    let editing = fixture.read_revision_after(&id, revision);
    let blocked=fixture.tool("terminal_input",json!({"session":id,"generation":generation,"request":"mcp-input-editing-1234","revision":editing,"text":marker_command("BLOCKED","INPUT"),"enter":true})).unwrap_err();
    assert!(blocked.contains("editing"), "{blocked}");
    host::write_frame(&mut viewer, &ClientOp::Input(vec![3])).unwrap();
    let cleared = fixture.read_revision_after(&id, editing);
    drop(viewer);
    fixture.tool("terminal_input",json!({"session":id,"generation":generation,"request":"mcp-input-exit-1234","revision":cleared,"text":"exit 7","enter":true})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let ended = fixture
            .tool("terminal_ended", json!({"session":id}))
            .unwrap();
        if !ended["Ended"]["value"].is_null() {
            assert_eq!(ended["Ended"]["value"]["ending"]["Exit"], 7);
            assert!(ended["Ended"]["value"]["screen"].is_object());
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Ended terminal content was not published"
        );
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(any(unix, windows))]
#[test]
fn hosted_shell_projects_edited_drafts_and_clears_them_on_submission() {
    #[cfg(unix)]
    let mut fixture = Fixture::new_with_shell(Some(Path::new("/bin/bash")));
    #[cfg(windows)]
    let mut fixture = Fixture::new();
    let (id, _) = fixture.open(
        "mcp-shell-draft-test-1234",
        &marker_command("DRAFT", "READY"),
    );
    fixture.read_until(&id, "DRAFT-READY");
    let directory = fixture
        .home
        .join(".khor/sessions")
        .join(id.replace('/', "-"));
    let mut viewer = fixture.viewer(&id);
    let mut send = |keys: &[u8]| {
        host::write_frame(&mut viewer, &ClientOp::Input(keys.to_vec())).unwrap();
    };
    let wait = |condition: &dyn Fn(&khor_core::Doing) -> bool| {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let doing = khor_node::commands::read_doing(&directory);
            if doing.as_ref().is_some_and(condition) {
                return;
            }
            assert!(Instant::now() < deadline, "Last shell reading: {doing:?}");
            thread::sleep(Duration::from_millis(25));
        }
    };
    wait(&|doing| !doing.running);

    send(b"echo KHOR-DRFT");
    wait(&|doing| doing.editor.as_deref() == Some("echo KHOR-DRFT"));
    send(b"\x1b[D\x1b[DA");
    wait(&|doing| doing.editor.as_deref() == Some("echo KHOR-DRAFT"));
    send(b"\r");
    wait(&|doing| doing.editor.is_none());
    fixture.read_until(&id, "KHOR-DRAFT");
    wait(&|doing| doing.command.as_deref() == Some("echo KHOR-DRAFT") && !doing.running);

    #[cfg(windows)]
    {
        // Retrieve actual PSReadLine history: the bytes contain no draft text.
        send(b"\x1b[A");
        wait(&|doing| doing.editor.as_deref() == Some("echo KHOR-DRAFT"));
        send(b"\x03");
        wait(&|doing| doing.editor.is_none());
    }
}

#[test]
fn actual_mcp_one_shot_retains_output_without_a_terminal_list_entry() {
    let mut fixture = Fixture::new();
    #[cfg(unix)]
    let command = "printf one-shot-output";
    #[cfg(windows)]
    let command = r#"<nul set /p "=one-shot-output"& exit /b 0"#;
    let started=fixture.tool("execute",json!({"machine":"mcp-terminal-test","request":"mcp-one-shot-task-1234","command":command,"cwd":fixture.home})).unwrap();
    let id = started["task"]["id"].as_str().unwrap().to_owned();
    fixture.tasks.push(id.clone());
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let reply = fixture
            .tool(
                "execution_output",
                json!({"machine":"mcp-terminal-test","task":id,"wait_ms":100}),
            )
            .unwrap();
        if reply["task"]["phase"] == "completed" {
            assert_eq!(reply["task"]["exit"], 0);
            assert_eq!(reply["stdout"]["text"], "one-shot-output");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "One-shot MCP task did not complete: {reply}"
        );
        thread::sleep(Duration::from_millis(25));
    }
    let listed = fixture.tool("sessions", json!({})).unwrap();
    assert!(!listed.as_str().unwrap().contains(&id));
    let sessions = fixture.home.join(".khor/sessions");
    if sessions.exists() {
        assert_eq!(fs::read_dir(sessions).unwrap().count(), 0);
    }
}

#[test]
fn failed_terminal_launch_never_leaves_an_unbacked_live_session() {
    let missing = std::env::temp_dir().join(format!(
        "this-khor-test-shell-does-not-exist-{}",
        std::process::id()
    ));
    assert!(!missing.exists());
    let mut fixture = Fixture::new_with_shell(Some(&missing));
    let failed = fixture
        .tool(
            "open",
            json!({
                "machine":"mcp-terminal-test", "request":"mcp-failed-shell-1234",
                "command":marker_command("MUST", "NOT-RUN"), "cwd":fixture.home,
                "title":"MCP failed startup",
            }),
        )
        .unwrap_err();
    assert!(
        !failed.trim().is_empty(),
        "Launch failure must explain its cause"
    );
    // Validation may reject before admission. If a row was admitted, it must
    // publish a readable ending instead of staying pid=None/unbacked forever.
    let directory = fixture.home.join(".khor/sessions");
    let rows: Vec<_> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| fs::read(entry.path().join("meta.json")).ok())
        .map(|bytes| serde_json::from_slice::<Value>(&bytes).unwrap())
        .collect();
    for row in rows {
        let id = row["id"].as_str().unwrap().to_owned();
        fixture.sessions.push(id.clone());
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let ended = fixture
                .tool("terminal_ended", json!({"session":id}))
                .unwrap();
            if !ended["Ended"]["value"].is_null() {
                assert!(
                    ended["Ended"]["value"]["startup_error"]
                        .as_str()
                        .is_some_and(|error| !error.trim().is_empty()),
                    "{ended}"
                );
                break;
            }
            assert!(
                Instant::now() < deadline,
                "Launch returned error but the session never ended: {failed}; {id}"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
}
