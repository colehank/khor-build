//! Detached command executions with retained results, independent of terminal sessions.
//!
//! The worker owns the process group / Job Object. Clients only address the
//! durable task identity; cancellation never signals a PID read from a file.
use crate::agent_process::{Control, Session};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

const RECORD_LIMIT: u64 = 128 * 1024;
const BLOCK_BYTES: u64 = 64 * 1024;
const KEPT_BLOCKS: u64 = 16;
const READ_LIMIT: usize = 64 * 1024;
const WAIT_LIMIT_MS: u64 = 10_000;
const MAX_TASKS: usize = 256;
const WORKER: &str = "_command_task";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Start {
        token: String,
        command: String,
        #[serde(default)]
        argv: Option<Vec<String>>,
        cwd: String,
        timeout_ms: Option<u64>,
    },
    Status {
        id: String,
    },
    Read {
        id: String,
        stdout_cursor: u64,
        stderr_cursor: u64,
        limit: usize,
        wait_ms: u64,
    },
    Cancel {
        id: String,
    },
    Forget {
        id: String,
    },
}

impl Request {
    pub fn is_reading(&self) -> bool {
        matches!(self, Self::Status { .. } | Self::Read { .. })
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Start {
                token,
                command,
                argv,
                cwd,
                timeout_ms,
            } => {
                if !crate::once::valid(token)
                    || command.len() > 32 * 1024
                    || command.contains('\0')
                    || cwd.len() > 32 * 1024
                    || cwd.contains('\0')
                    || *timeout_ms == Some(0)
                {
                    return Err("Invalid command task arguments".into());
                }
                match argv {
                    Some(words) => {
                        if !command.is_empty()
                            || words.first().is_none_or(|program| program.is_empty())
                            || words.len() > 1024
                            || words.iter().any(|word| word.contains('\0'))
                            || words.iter().map(String::len).sum::<usize>() > 32 * 1024
                        {
                            return Err("Invalid command task argument vector".into());
                        }
                    }
                    None if command.trim().is_empty() => {
                        return Err("Command task requires a command or argument vector".into());
                    }
                    None => {}
                }
            }
            Self::Status { id } | Self::Cancel { id } | Self::Forget { id } => {
                valid_id(id)?;
            }
            Self::Read {
                id, limit, wait_ms, ..
            } => {
                valid_id(id)?;
                if *limit == 0 || *limit > READ_LIMIT || *wait_ms > WAIT_LIMIT_MS {
                    return Err("Command output limit or wait exceeds its budget".into());
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Starting,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Interrupted,
}

impl Phase {
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Starting | Self::Running)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub id: String,
    pub command: String,
    pub cwd: String,
    pub phase: Phase,
    pub exit: Option<i32>,
    pub error: Option<String>,
    pub created_ms: u64,
    pub started_ms: Option<u64>,
    pub ended_ms: Option<u64>,
    #[serde(default)]
    pub forgotten: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Output {
    /// UTF-8 with invalid sequences replaced. Cursors always count source bytes.
    pub text: String,
    /// Exact source bytes, base64 encoded, including split UTF-8 sequences.
    pub bytes: String,
    pub next: u64,
    pub end: u64,
    /// Some bytes before the returned range have left the retained window.
    pub truncated: bool,
}

impl Output {
    pub fn raw_bytes(&self) -> Result<Vec<u8>, String> {
        B64.decode(&self.bytes).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub task: Status,
    pub stdout: Output,
    pub stderr: Output,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    fingerprint: String,
    timeout_ms: Option<u64>,
    #[serde(default)]
    argv: Option<Vec<String>>,
    worker_pid: Option<u32>,
    #[serde(default)]
    cleanup_confirmed: bool,
    status: Status,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Forgotten {
    fingerprint: String,
    status: Status,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Window {
    start: u64,
    end: u64,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn tasks(root: &Path) -> PathBuf {
    root.join(".khor").join("command-tasks")
}

fn task_dir(root: &Path, id: &str) -> Result<PathBuf, String> {
    Ok(tasks(root).join(valid_id(id)?))
}

fn valid_id(id: &str) -> Result<&str, String> {
    id.strip_prefix("command/")
        .filter(|token| crate::once::valid(token))
        .ok_or_else(|| "Invalid command task identity".into())
}

fn private_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn gate(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path).map_err(|error| error.to_string())?;
    file.lock().map_err(|error| error.to_string())?;
    Ok(file)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(RECORD_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > RECORD_LIMIT {
        return Err("Command task record exceeds its byte limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > RECORD_LIMIT {
        return Err("Command task record exceeds its byte limit".into());
    }
    // Every caller holds this record's cross-process lock.
    crate::live::write_whole(path, &bytes)
}

fn record(dir: &Path) -> Result<Record, String> {
    read_json(&dir.join("task.json"))
}

fn save(dir: &Path, record: &Record) -> Result<(), String> {
    write_json(&dir.join("task.json"), record)
}

fn forgotten(root: &Path, id: &str) -> Result<Option<Forgotten>, String> {
    let token = valid_id(id)?;
    let path = tasks(root).join(".forgotten").join(format!("{token}.json"));
    if !path.exists() {
        return Ok(None);
    }
    read_json(&path).map(Some)
}

fn forgotten_reply(saved: Forgotten) -> Reply {
    Reply {
        task: saved.status,
        stdout: Output::default(),
        stderr: Output::default(),
    }
}

/// Called by the target machine. This never registers a terminal/session row.
pub fn handle(root: &Path, request: Request) -> Result<Reply, String> {
    request.validate()?;
    match request {
        Request::Start {
            token,
            command,
            argv,
            cwd,
            timeout_ms,
        } => start(root, &token, &command, argv, &cwd, timeout_ms),
        Request::Status { id } => reply(root, &id, 0, 0, 0),
        Request::Read {
            id,
            stdout_cursor,
            stderr_cursor,
            limit,
            wait_ms,
        } => {
            let deadline = Instant::now() + Duration::from_millis(wait_ms);
            loop {
                let answer = reply(root, &id, stdout_cursor, stderr_cursor, limit)?;
                if answer.task.phase.terminal()
                    || answer.stdout.end != stdout_cursor
                    || answer.stderr.end != stderr_cursor
                    || Instant::now() >= deadline
                {
                    return Ok(answer);
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
        Request::Cancel { id } => {
            if let Some(saved) = forgotten(root, &id)? {
                return Ok(forgotten_reply(saved));
            }
            let dir = task_dir(root, &id)?;
            {
                let _held = gate(&dir.join("record.lock"))?;
                if !record(&dir)?.status.phase.terminal() {
                    crate::link::write_private(&dir.join("cancel"), b"cancel\n")
                        .map_err(|error| error.to_string())?;
                }
            }
            let deadline = Instant::now() + Duration::from_millis(WAIT_LIMIT_MS);
            loop {
                let answer = reply(root, &id, 0, 0, 0)?;
                if answer.task.phase.terminal() || Instant::now() >= deadline {
                    return Ok(answer);
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
        Request::Forget { id } => forget(root, &id),
    }
}

fn start(
    root: &Path,
    token: &str,
    command: &str,
    argv: Option<Vec<String>>,
    cwd: &str,
    timeout_ms: Option<u64>,
) -> Result<Reply, String> {
    let cwd =
        fs::canonicalize(crate::shell_start::directory(cwd)?).map_err(|error| error.to_string())?;
    let cwd = cwd.to_str().ok_or("Command directory is not UTF-8")?;
    let fingerprint = blake3::hash(
        &serde_json::to_vec(&(command, &argv, cwd, timeout_ms))
            .map_err(|error| error.to_string())?,
    )
    .to_hex()
    .to_string();
    let id = format!("command/{token}");
    let directory = tasks(root);
    private_directory(&directory)?;
    // A single admission lock makes the retained-task count and token claim atomic.
    let _admission = gate(&directory.join("admission.lock"))?;
    if let Some(saved) = forgotten(root, &id)? {
        if saved.fingerprint != fingerprint {
            return Err("Command task token already names a different command".into());
        }
        return Ok(forgotten_reply(saved));
    }
    let dir = task_dir(root, &id)?;
    if dir.exists() {
        let _held = gate(&dir.join("record.lock"))?;
        let prior = record(&dir)?;
        if prior.fingerprint != fingerprint {
            return Err("Command task token already names a different command".into());
        }
        drop(_held);
        return reply(root, &id, 0, 0, 0);
    }
    let count = fs::read_dir(&directory)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|kind| kind.is_dir())
                && entry.file_name().to_str().is_some_and(crate::once::valid)
        })
        .count();
    if count >= MAX_TASKS {
        return Err("Retained command task limit reached; remove old task records first".into());
    }
    private_directory(&dir)?;
    let mut current = Record {
        fingerprint,
        timeout_ms,
        argv: argv.clone(),
        worker_pid: None,
        cleanup_confirmed: false,
        status: Status {
            id: id.clone(),
            command: argv
                .as_ref()
                .map(|words| words.join(" "))
                .unwrap_or_else(|| command.into()),
            cwd: cwd.into(),
            phase: Phase::Starting,
            exit: None,
            error: None,
            created_ms: now_ms(),
            started_ms: None,
            ended_ms: None,
            forgotten: false,
        },
    };
    let _held = gate(&dir.join("record.lock"))?;
    save(&dir, &current)?;
    match spawn_worker(root, &id) {
        Ok(pid) => current.worker_pid = Some(pid),
        Err(error) => {
            current.status.phase = Phase::Failed;
            current.status.error = Some(error);
            current.status.ended_ms = Some(now_ms());
            // The record lock fences the worker from admitting its command
            // until this failed status is visible.
            current.cleanup_confirmed = true;
        }
    }
    save(&dir, &current)?;
    drop(_held);
    reply(root, &id, 0, 0, 0)
}

fn spawn_worker(root: &Path, id: &str) -> Result<u32, String> {
    // The published node contract remains compatible with older residents;
    // this private worker entry must come from the image implementing it.
    let image = crate::runtime_image::running_host()?;
    let mut command = Command::new(image.path());
    command
        .args([WORKER, id])
        .env("KHOR_HOME", root)
        .env(crate::PATH_ENV, crate::Node::user_path())
        .env("PATH", crate::Node::user_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    crate::mcp_worker::prepare_durable_child(&mut command);
    let mut child = image
        .spawn(&mut command)
        .map_err(|error| error.to_string())?;
    let pid = child.id();
    std::thread::Builder::new()
        .name("command-worker-reaper".into())
        .spawn(move || {
            let _ = child.wait();
        })
        .map_err(|error| error.to_string())?;
    Ok(pid)
}

fn status(root: &Path, id: &str) -> Result<Status, String> {
    let dir = task_dir(root, id)?;
    let _held = gate(&dir.join("record.lock"))?;
    let mut current = record(&dir)?;
    if current.status.phase == Phase::Running {
        // The lock, rather than a reusable PID, establishes worker ownership.
        if let Ok(_lost) = crate::remote_cancel::gate(&dir.join("worker.lock")) {
            current.status.phase = Phase::Interrupted;
            current.status.error =
                Some("Command worker disappeared before confirming its process outcome".into());
            current.status.ended_ms = Some(now_ms());
            save(&dir, &current)?;
        }
    } else if current.status.phase == Phase::Starting
        && now_ms().saturating_sub(current.status.created_ms) > 60_000
        && let Ok(_lost) = crate::remote_cancel::gate(&dir.join("worker.lock"))
    {
        current.status.phase = Phase::Interrupted;
        current.status.error = Some("Command worker did not confirm startup".into());
        current.status.ended_ms = Some(now_ms());
        save(&dir, &current)?;
    }
    Ok(current.status)
}

fn reply(
    root: &Path,
    id: &str,
    stdout_cursor: u64,
    stderr_cursor: u64,
    limit: usize,
) -> Result<Reply, String> {
    if let Some(saved) = forgotten(root, id)? {
        return Ok(forgotten_reply(saved));
    }
    let task = status(root, id)?;
    let dir = task_dir(root, id)?;
    Ok(Reply {
        task,
        stdout: read_output(&dir, "stdout", stdout_cursor, limit)?,
        stderr: read_output(&dir, "stderr", stderr_cursor, limit)?,
    })
}

fn forget(root: &Path, id: &str) -> Result<Reply, String> {
    let dir = task_dir(root, id)?;
    let _admission = gate(&tasks(root).join("admission.lock"))?;
    if !dir.exists()
        && let Some(saved) = forgotten(root, id)?
    {
        return Ok(forgotten_reply(saved));
    }
    let held = gate(&dir.join("record.lock"))?;
    let current = record(&dir)?;
    if !current.status.phase.terminal() || !current.cleanup_confirmed {
        return Err("Only a command with confirmed process cleanup can be forgotten".into());
    }
    let owned = crate::remote_cancel::gate(&dir.join("worker.lock"))
        .map_err(|_| "Command worker is still finalizing its result")?;
    let mut answer = Reply {
        task: current.status.clone(),
        stdout: read_output(&dir, "stdout", 0, READ_LIMIT)?,
        stderr: read_output(&dir, "stderr", 0, READ_LIMIT)?,
    };
    answer.task.forgotten = true;
    let mut summary = current.status;
    summary.command.clear();
    summary.cwd.clear();
    summary.error = None;
    summary.forgotten = true;
    let claims = tasks(root).join(".forgotten");
    private_directory(&claims)?;
    write_json(
        &claims.join(format!("{}.json", valid_id(id)?)),
        &Forgotten {
            fingerprint: current.fingerprint,
            status: summary,
        },
    )?;
    drop(owned);
    drop(held);
    // Admission remains fenced until removal finishes. The small fingerprint
    // tombstone keeps uncertain Start retries from repeating forgotten work.
    fs::remove_dir_all(dir).map_err(|error| error.to_string())?;
    Ok(answer)
}

fn stream_window(dir: &Path, stream: &str) -> Result<Window, String> {
    let path = dir.join(format!("{stream}.json"));
    if !path.exists() {
        return Ok(Window::default());
    }
    let window: Window = read_json(&path)?;
    if window.start > window.end {
        return Err("Command output window is invalid".into());
    }
    Ok(window)
}

fn append_output(dir: &Path, stream: &str, bytes: &[u8]) -> Result<(), String> {
    let _held = gate(&dir.join(format!("{stream}.lock")))?;
    let mut window = stream_window(dir, stream)?;
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let index = window.end / BLOCK_BYTES;
        let offset = window.end % BLOCK_BYTES;
        let count = remaining.len().min((BLOCK_BYTES - offset) as usize);
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(dir.join(format!("{stream}-{index}.bin")))
            .map_err(|error| error.to_string())?;
        if file.metadata().map_err(|error| error.to_string())?.len() != offset {
            return Err("Command output file does not match its committed cursor".into());
        }
        file.write_all(&remaining[..count])
            .map_err(|error| error.to_string())?;
        window.end = window
            .end
            .checked_add(count as u64)
            .ok_or("Output cursor overflow")?;
        remaining = &remaining[count..];
    }
    let first = (window.end.saturating_sub(1) / BLOCK_BYTES).saturating_sub(KEPT_BLOCKS - 1);
    let previous_start = window.start;
    window.start = first * BLOCK_BYTES;
    // Publish the retained window before deleting its old blocks. A worker
    // crash can leave extra old files, but never a committed reference to one
    // that it has already removed.
    write_json(&dir.join(format!("{stream}.json")), &window)?;
    for index in previous_start / BLOCK_BYTES..first {
        let path = dir.join(format!("{stream}-{index}.bin"));
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn read_output(dir: &Path, stream: &str, cursor: u64, limit: usize) -> Result<Output, String> {
    let _held = gate(&dir.join(format!("{stream}.lock")))?;
    let window = stream_window(dir, stream)?;
    if cursor > window.end {
        return Err("Command output cursor is beyond the stream".into());
    }
    let start = cursor.max(window.start);
    let end = window.end.min(start.saturating_add(limit as u64));
    let mut bytes = Vec::with_capacity((end - start) as usize);
    let mut next = start;
    while next < end {
        let index = next / BLOCK_BYTES;
        let offset = next % BLOCK_BYTES;
        let count = (end - next).min(BLOCK_BYTES - offset);
        let mut file = File::open(dir.join(format!("{stream}-{index}.bin")))
            .map_err(|error| error.to_string())?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|error| error.to_string())?;
        let old = bytes.len();
        bytes.resize(old + count as usize, 0);
        file.read_exact(&mut bytes[old..])
            .map_err(|error| error.to_string())?;
        next += count;
    }
    Ok(Output {
        text: String::from_utf8_lossy(&bytes).into_owned(),
        bytes: B64.encode(&bytes),
        next,
        end: window.end,
        truncated: cursor < window.start,
    })
}

fn drain(
    dir: PathBuf,
    stream: &'static str,
    mut reader: Box<dyn Read + Send>,
    control: Arc<Control>,
) -> Result<(), String> {
    let result = (|| {
        let mut bytes = [0; 8192];
        loop {
            let count = match reader.read(&mut bytes) {
                Ok(count) => count,
                // Unix's cancellable pipe can observe cancellation between
                // WouldBlock and its retry, reporting BrokenPipe rather than
                // the normal zero-byte read. This is only an input boundary:
                // failures retaining already-read output must still surface.
                Err(error)
                    if error.kind() == std::io::ErrorKind::BrokenPipe && control.is_cancelled() =>
                {
                    return Ok(());
                }
                Err(error) => return Err(error.to_string()),
            };
            if count == 0 {
                return Ok(());
            }
            append_output(&dir, stream, &bytes[..count])?;
        }
    })();
    if result.is_err() {
        control.terminate_now();
    }
    result
}

fn command(record: &Record) -> Command {
    let mut command = if let Some(words) = &record.argv {
        let program = Path::new(&words[0]);
        let program = if program.is_relative() && program.components().count() > 1 {
            Path::new(&record.status.cwd).join(program)
        } else {
            program.to_path_buf()
        };
        let mut command = Command::new(program);
        command.args(&words[1..]);
        command
    } else {
        #[cfg(windows)]
        let command = {
            let shell = std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into());
            let mut command = Command::new(shell);
            command.args(["/d", "/s", "/c", &record.status.command]);
            command
        };
        #[cfg(not(windows))]
        let command = {
            let mut command = Command::new("sh");
            command.args(["-c", &record.status.command]);
            command
        };
        command
    };
    let cwd = Path::new(&record.status.cwd);
    #[cfg(windows)]
    let cwd = {
        use std::path::{Component, Prefix};
        let cmd = Path::new(command.get_program())
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.eq_ignore_ascii_case("cmd.exe") || name.eq_ignore_ascii_case("cmd")
            });
        // canonicalize retains a verbatim disk prefix for durable identity,
        // but cmd.exe mistakes that local directory for an unsupported UNC
        // path and silently switches to Windows' directory. Translate only
        // CMD's local disk cwd, keeping native programs, task identity,
        // arguments, real UNC paths, and other device namespaces intact.
        match cwd.components().next() {
            Some(Component::Prefix(prefix))
                if cmd && matches!(prefix.kind(), Prefix::VerbatimDisk(_)) =>
            {
                Path::new(
                    record
                        .status
                        .cwd
                        .strip_prefix(r"\\?\")
                        .unwrap_or(&record.status.cwd),
                )
            }
            _ => cwd,
        }
    };
    // The Windows process adapter intentionally only accepts explicit environment
    // entries. Unix uses the identical target-side environment contract.
    command
        .env_clear()
        .envs(std::env::vars_os())
        .env("PATH", crate::Node::user_path())
        .current_dir(cwd);
    command
}

fn worker(root: &Path, id: &str) -> Result<(), String> {
    let dir = task_dir(root, id)?;
    let _owned = crate::remote_cancel::gate(&dir.join("worker.lock"))?;
    let mut current = {
        let _held = gate(&dir.join("record.lock"))?;
        let mut current = record(&dir)?;
        if current.status.phase != Phase::Starting {
            return Ok(());
        }
        if dir.join("cancel").exists() {
            current.status.phase = Phase::Cancelled;
            current.status.ended_ms = Some(now_ms());
            current.cleanup_confirmed = true;
            save(&dir, &current)?;
            return Ok(());
        }
        current.status.phase = Phase::Running;
        current.status.started_ms = Some(now_ms());
        save(&dir, &current)?;
        current
    };
    let control = Arc::new(Control::default());
    let mut cleanup_confirmed = true;
    let result = (|| {
        let (mut session, pipes) = Session::spawn(command(&current), control.clone())
            .map_err(|error| error.to_string())?;
        cleanup_confirmed = false;
        drop(pipes.stdin);
        let out_control = control.clone();
        let err_control = control.clone();
        let out_dir = dir.clone();
        let err_dir = dir.clone();
        let stdout =
            std::thread::spawn(move || drain(out_dir, "stdout", pipes.stdout, out_control));
        let stderr =
            std::thread::spawn(move || drain(err_dir, "stderr", pipes.stderr, err_control));
        let started = Instant::now();
        let mut stopped = None;
        let exit = loop {
            if let Some(exit) = session.poll_exit() {
                break exit.map_err(|error| error.to_string());
            }
            if stopped.is_none() {
                let reason = if dir.join("cancel").exists() {
                    Some(Phase::Cancelled)
                } else if current
                    .timeout_ms
                    .is_some_and(|limit| started.elapsed() >= Duration::from_millis(limit))
                {
                    Some(Phase::TimedOut)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    stopped = Some(reason);
                    control.terminate_now();
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        // The owned process reaper has stopped all descendants. Drain their
        // buffered tail before closing the cancellable pipe handles.
        let out = stdout
            .join()
            .map_err(|_| "Command stdout reader panicked")?;
        let err = stderr
            .join()
            .map_err(|_| "Command stderr reader panicked")?;
        let finished = session.finish(false).map_err(|error| error.to_string());
        cleanup_confirmed = finished.is_ok();
        let exit = exit?;
        finished?;
        out?;
        err?;
        Ok::<_, String>((stopped, exit))
    })();
    current.status.ended_ms = Some(now_ms());
    current.cleanup_confirmed = cleanup_confirmed;
    match result {
        Ok((stopped, exit)) => {
            current.status.exit = exit.code();
            current.status.phase = stopped.unwrap_or(if exit.success() {
                Phase::Completed
            } else {
                Phase::Failed
            });
        }
        Err(error) => {
            current.status.phase = if cleanup_confirmed {
                Phase::Failed
            } else {
                Phase::Interrupted
            };
            current.status.error = Some(error);
        }
    }
    let _held = gate(&dir.join("record.lock"))?;
    save(&dir, &current)
}

/// Early process hook used by CLI and native hosts before opening Node state.
pub fn main_if_requested(root: &Path) -> Option<i32> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some(WORKER) {
        return None;
    }
    let outcome = match args.as_slice() {
        [_, id] => worker(root, id),
        _ => Err("Command worker expects exactly one task identity".into()),
    };
    Some(match outcome {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    })
}

#[cfg(test)]
#[path = "command_task_tests.rs"]
mod tests;
