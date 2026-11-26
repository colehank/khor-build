//! Real CLI -> target node -> detached worker, without terminal/session rows.
use serde_json::Value;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
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
    let stdout = home.join(format!("cli-{serial}.stdout.log"));
    let stderr = home.join(format!("cli-{serial}.stderr.log"));
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
    tasks: Vec<String>,
    install_source: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home =
            std::env::temp_dir().join(format!("khor-command-cli-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        Self {
            home,
            tasks: Vec::new(),
            install_source: env!("CARGO_BIN_EXE_khor").into(),
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_khor"));
        command
            .env("KHOR_HOME", &self.home)
            .env("KHOR_NAME", "command-test")
            .env("KHOR_RUNTIME_DIR", self.home.join("runtime"))
            .env("KHOR_RUNTIME_INSTALL_SOURCE", &self.install_source)
            .env("KHOR_TASK_LITERAL", "space value %KEEP_LITERAL%")
            .env("KHOR_TASK_INPUT", self.home.join("literal input 中文.txt"))
            .env_remove("KHOR_SESSION")
            .env_remove("KHOR_PATH");
        command
    }
    fn try_run(&self, args: &[&str]) -> io::Result<Output> {
        bounded_output(&self.home, self.command().args(args))
    }
    fn run(&self, args: &[&str]) -> Output {
        self.try_run(args)
            .unwrap_or_else(|error| panic!("CLI {args:?} did not finish: {error}"))
    }
    fn json(&self, args: &[&str]) -> Value {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "args={args:?}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "Invalid command JSON: {error}: {}",
                String::from_utf8_lossy(&output.stdout)
            )
        })
    }
    fn start(&mut self, token: &str, command: &str) -> Value {
        let reply = self.json(&[
            "exec",
            "--on",
            "command-test",
            "--request",
            token,
            "--command",
            command,
            "--cwd",
            self.home.to_str().unwrap(),
        ]);
        let id = reply["task"]["id"].as_str().unwrap().to_owned();
        if !self.tasks.contains(&id) {
            self.tasks.push(id);
        }
        reply
    }
    fn action(&self, action: &str, id: &str) -> Value {
        self.json(&["execution", action, "--on", "command-test", id])
    }
    fn terminal(&self, id: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            let reply = self.action("status", id);
            if !matches!(
                reply["task"]["phase"].as_str(),
                Some("starting" | "running")
            ) {
                return reply;
            }
            assert!(Instant::now() < deadline, "task did not finish: {reply}");
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    fn assert_no_sessions(&self) {
        let directory = self.home.join(".khor/sessions");
        if directory.exists() {
            assert_eq!(std::fs::read_dir(directory).unwrap().count(), 0);
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // A timed-out Start can admit work before its CLI reply arrives.
        if let Ok(entries) = fs::read_dir(self.home.join(".khor/command-tasks")) {
            for entry in entries.flatten() {
                let id = fs::read(entry.path().join("task.json"))
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .and_then(|record| record["status"]["id"].as_str().map(str::to_owned));
                if let Some(id) = id {
                    if !self.tasks.contains(&id) {
                        self.tasks.push(id);
                    }
                }
            }
        }
        let mut failures = Vec::new();
        for id in &self.tasks {
            match self.try_run(&["execution", "cancel", "--on", "command-test", id]) {
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
                    "cancel {id}: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )),
                Err(error) => failures.push(format!("cancel {id}: {error}")),
            }
        }
        if failures.is_empty() && !thread::panicking() {
            // The worker may still be releasing its runtime executable on
            // Windows after publishing the final task receipt.
            if let Err(error) = remove_fixture_home(&self.home) {
                failures.push(format!("fixture directory cleanup: {error}"));
            }
        } else if failures.is_empty() {
            eprintln!("Preserving failed command fixture at {:?}", self.home);
        }
        if !failures.is_empty() {
            let message = format!(
                "Command cleanup was not confirmed; preserving {:?}: {}",
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

#[test]
fn actual_worker_uses_its_running_image_after_the_installation_source_has_moved() {
    let mut fixture = Fixture::new();
    fixture.install_source = fixture.home.join("retired-installation-program");
    assert!(!fixture.install_source.exists());
    #[cfg(not(windows))]
    let command = "printf current-image";
    #[cfg(windows)]
    let command = r#"<nul set /p "=current-image"& exit /b 0"#;
    let started = fixture.start("cli-running-image-token", command);
    let id = started["task"]["id"].as_str().unwrap();
    let done = fixture.terminal(id);
    assert_eq!(done["task"]["phase"], "completed");
    assert_eq!(done["task"]["exit"], 0);
    assert_eq!(
        fixture.action("read", id)["stdout"]["text"],
        "current-image"
    );
    fixture.assert_no_sessions();
    assert!(!fixture.install_source.exists());
}

#[test]
fn actual_worker_keeps_receipts_not_sessions_and_supports_retry_cancel_and_forget() {
    let mut fixture = Fixture::new();
    #[cfg(not(windows))]
    let success_command = "printf task-ok; printf task-error >&2";
    #[cfg(windows)]
    let success_command = r#"<nul set /p "=task-ok"& <nul set /p "=task-error" >&2& exit /b 0"#;
    let started = fixture.start("cli-success-token-1234", success_command);
    let id = started["task"]["id"].as_str().unwrap().to_owned();
    let done = fixture.terminal(&id);
    assert_eq!(done["task"]["phase"], "completed");
    assert_eq!(done["task"]["exit"], 0);
    let output = fixture.action("read", &id);
    assert_eq!(output["stdout"]["text"], "task-ok");
    assert_eq!(output["stderr"]["text"], "task-error");
    fixture.assert_no_sessions();
    let retry = fixture.start("cli-success-token-1234", success_command);
    assert_eq!(retry["task"]["id"], id);
    assert_eq!(retry["task"]["phase"], "completed");
    let collision = fixture.run(&[
        "exec",
        "--on",
        "command-test",
        "--request",
        "cli-success-token-1234",
        "--command",
        "a different command",
        "--cwd",
        fixture.home.to_str().unwrap(),
    ]);
    assert!(!collision.status.success());

    #[cfg(not(windows))]
    let failure_command = "printf task-failed >&2; exit 7";
    #[cfg(windows)]
    let failure_command = r#"<nul set /p "=task-failed" >&2& exit /b 7"#;
    let failed = fixture.start("cli-failure-token-1234", failure_command);
    let failed_id = failed["task"]["id"].as_str().unwrap();
    let failed = fixture.terminal(failed_id);
    assert_eq!(failed["task"]["phase"], "failed");
    assert_eq!(failed["task"]["exit"], 7);
    assert_eq!(
        fixture.action("read", failed_id)["stderr"]["text"],
        "task-failed"
    );

    #[cfg(not(windows))]
    let slow_command = "printf ready; sleep 30";
    #[cfg(windows)]
    let slow_command = r#"<nul set /p "=ready"& ping -n 31 127.0.0.1 >nul"#;
    let active = fixture.start("cli-cancel-token-1234", slow_command);
    let active_id = active["task"]["id"].as_str().unwrap();
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        let reply = fixture.action("read", active_id);
        if reply["stdout"]["text"]
            .as_str()
            .is_some_and(|text| text.contains("ready"))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "command never produced its startup output: {reply}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let cancelled = fixture.action("cancel", active_id);
    assert_eq!(cancelled["task"]["phase"], "cancelled");
    assert_eq!(
        fixture.action("cancel", active_id)["task"]["phase"],
        "cancelled"
    );
    assert_eq!(fixture.action("read", active_id)["stdout"]["text"], "ready");

    // Removing output cannot turn an old uncertain Start retry into new work.
    assert_eq!(fixture.action("forget", &id)["stdout"]["text"], "task-ok");
    assert_eq!(fixture.action("status", &id)["task"]["forgotten"], true);
    assert_eq!(
        fixture.start("cli-success-token-1234", success_command)["task"]["forgotten"],
        true
    );
    fixture.assert_no_sessions();
    let listed = fixture.run(&["sessions", "--all"]);
    assert!(listed.status.success());
    assert!(!String::from_utf8_lossy(&listed.stdout).contains("command/"));

    #[cfg(windows)]
    {
        std::fs::write(
            fixture.home.join("literal input 中文.txt"),
            "unicode 中文\n",
        )
        .unwrap();
        let started = fixture.start(
            "cli-cmd-literal-1234",
            r#"echo "%KHOR_TASK_LITERAL%"& type "%KHOR_TASK_INPUT%""#,
        );
        let literal_id = started["task"]["id"].as_str().unwrap();
        assert_eq!(fixture.terminal(literal_id)["task"]["phase"], "completed");
        let output = fixture.action("read", literal_id);
        let text = output["stdout"]["text"].as_str().unwrap();
        assert!(
            text.starts_with("\"space value %KEEP_LITERAL%\"\r\n"),
            "{text:?}"
        );
        assert!(text.ends_with("unicode 中文\n"), "{text:?}");
    }

    let literals = [
        "space value",
        "中文路径",
        "$HOME",
        "%PATH%",
        "a&b",
        "quote\"value",
        "single'quote",
    ];
    let mut command = fixture.command();
    command.args(["run", "--"]);
    #[cfg(not(windows))]
    command.args(["/bin/sh", "-c", "printf '%s\\n' \"$@\"", "argument-probe"]);
    #[cfg(windows)]
    {
        let script = fixture.home.join("arguments 空格.ps1");
        std::fs::write(
            &script,
            "param([Parameter(ValueFromRemainingArguments=$true)][string[]] $Values)\n[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false\n[Console]::Out.Write((ConvertTo-Json -InputObject @($Values) -Compress))\n",
        ).unwrap();
        let powershell = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap()
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        command
            .arg(powershell)
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(script);
    }
    let output = bounded_output(&fixture.home, command.args(literals)).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    #[cfg(not(windows))]
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        literals
    );
    #[cfg(windows)]
    assert_eq!(
        serde_json::from_slice::<Vec<String>>(&output.stdout).unwrap(),
        literals
    );
    fixture.assert_no_sessions();
}
