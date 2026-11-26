use super::*;

fn seed(root: &Path, token: &str, command: &str, timeout_ms: Option<u64>) -> String {
    let id = format!("command/{token}");
    let canonical = fs::canonicalize(root).unwrap();
    let cwd = canonical.to_str().unwrap();
    let dir = task_dir(root, &id).unwrap();
    private_directory(&dir).unwrap();
    let _held = gate(&dir.join("record.lock")).unwrap();
    save(
        &dir,
        &Record {
            fingerprint: blake3::hash(
                &serde_json::to_vec(&(command, &Option::<Vec<String>>::None, cwd, timeout_ms))
                    .unwrap(),
            )
            .to_hex()
            .to_string(),
            timeout_ms,
            argv: None,
            worker_pid: None,
            cleanup_confirmed: false,
            status: Status {
                id: id.clone(),
                command: command.into(),
                cwd: cwd.into(),
                phase: Phase::Starting,
                exit: None,
                error: None,
                created_ms: now_ms(),
                started_ms: None,
                ended_ms: None,
                forgotten: false,
            },
        },
    )
    .unwrap();
    id
}

fn read(root: &Path, id: &str) -> Reply {
    handle(
        root,
        Request::Read {
            id: id.into(),
            stdout_cursor: 0,
            stderr_cursor: 0,
            limit: READ_LIMIT,
            wait_ms: 0,
        },
    )
    .unwrap()
}

#[test]
fn command_arguments_and_output_budgets_are_checked_before_dispatch() {
    assert!(
        Request::Start {
            token: "safe-task-token-1234".into(),
            command: "echo yes".into(),
            argv: None,
            cwd: String::new(),
            timeout_ms: None,
        }
        .validate()
        .is_ok()
    );
    for id in ["shell/abc", "command/../escape", "command/short"] {
        assert!(Request::Status { id: id.into() }.validate().is_err());
    }
    assert!(
        Request::Read {
            id: "command/safe-task-token-1234".into(),
            stdout_cursor: 0,
            stderr_cursor: 0,
            limit: READ_LIMIT + 1,
            wait_ms: 0,
        }
        .validate()
        .is_err()
    );
}

#[test]
fn exact_output_bytes_and_cursors_survive_utf8_chunk_boundaries() {
    let root = crate::testdir::TempTree::fresh("command-task-utf8");
    let id = seed(&root, "task-output-utf8-1234", "unused", None);
    let dir = task_dir(&root, &id).unwrap();
    let bytes = "a界b".as_bytes();
    append_output(&dir, "stdout", bytes).unwrap();
    let mut cursor = 0;
    let mut restored = Vec::new();
    while cursor < bytes.len() as u64 {
        let output = read_output(&dir, "stdout", cursor, 1).unwrap();
        restored.extend(output.raw_bytes().unwrap());
        assert_eq!(output.next, cursor + 1);
        assert!(!output.truncated);
        cursor = output.next;
    }
    assert_eq!(restored, bytes);
    assert!(read_output(&dir, "stdout", cursor + 1, 1).is_err());
}

#[test]
fn output_retains_a_bounded_tail_and_reports_lost_prefix() {
    let root = crate::testdir::TempTree::fresh("command-task-output-window");
    let id = seed(&root, "task-output-window-1234", "unused", None);
    let dir = task_dir(&root, &id).unwrap();
    for i in 0..KEPT_BLOCKS + 2 {
        append_output(&dir, "stdout", &vec![i as u8; BLOCK_BYTES as usize]).unwrap();
    }
    let output = read_output(&dir, "stdout", 0, 8).unwrap();
    assert!(output.truncated);
    assert_eq!(output.next, 2 * BLOCK_BYTES + 8);
    assert_eq!(B64.decode(output.bytes).unwrap(), vec![2; 8]);
    let blocks = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("stdout-"))
        .count();
    assert_eq!(blocks as u64, KEPT_BLOCKS);
}

#[test]
fn a_retried_token_returns_the_original_task_and_rejects_different_input() {
    let root = crate::testdir::TempTree::fresh("command-task-retry");
    let token = "task-duplicate-token-1234";
    let id = seed(&root, token, "echo once", None);
    let answer = handle(
        &root,
        Request::Start {
            token: token.into(),
            command: "echo once".into(),
            argv: None,
            cwd: root.to_string_lossy().into_owned(),
            timeout_ms: None,
        },
    )
    .unwrap();
    assert_eq!(answer.task.id, id);
    assert_eq!(answer.task.phase, Phase::Starting);
    assert!(
        handle(
            &root,
            Request::Start {
                token: token.into(),
                command: "echo twice".into(),
                argv: None,
                cwd: root.to_string_lossy().into_owned(),
                timeout_ms: None,
            },
        )
        .is_err()
    );
    assert!(!root.join(".khor/sessions").exists());
}

#[test]
fn a_lost_worker_is_not_reported_as_confirmed_cancellation() {
    let root = crate::testdir::TempTree::fresh("command-task-worker-lost");
    let id = seed(&root, "task-lost-worker-1234", "unused", None);
    let dir = task_dir(&root, &id).unwrap();
    let mut current = record(&dir).unwrap();
    current.status.phase = Phase::Running;
    current.worker_pid = Some(std::process::id());
    save(&dir, &current).unwrap();
    let reply = handle(&root, Request::Cancel { id: id.clone() }).unwrap();
    assert_eq!(reply.task.phase, Phase::Interrupted);
    assert!(reply.task.error.unwrap().contains("outcome"));
    assert!(handle(&root, Request::Forget { id }).is_err());
}

#[test]
fn forgetting_refuses_active_work_and_releases_only_a_confirmed_result() {
    let root = crate::testdir::TempTree::fresh("command-task-forget");
    let id = seed(&root, "task-forget-result-1234", "unused", None);
    let dir = task_dir(&root, &id).unwrap();
    assert!(handle(&root, Request::Forget { id: id.clone() }).is_err());
    assert!(dir.exists());
    let mut current = record(&dir).unwrap();
    current.status.phase = Phase::Completed;
    current.status.exit = Some(0);
    current.status.ended_ms = Some(now_ms());
    current.cleanup_confirmed = true;
    save(&dir, &current).unwrap();
    append_output(&dir, "stdout", b"retained result").unwrap();
    let final_result = handle(&root, Request::Forget { id: id.clone() }).unwrap();
    assert_eq!(final_result.task.phase, Phase::Completed);
    assert_eq!(final_result.stdout.text, "retained result");
    assert!(!dir.exists());
    assert!(
        handle(&root, Request::Status { id: id.clone() })
            .unwrap()
            .task
            .forgotten
    );
    assert!(
        handle(&root, Request::Forget { id: id.clone() })
            .unwrap()
            .task
            .forgotten
    );
    let retry = handle(
        &root,
        Request::Start {
            token: valid_id(&id).unwrap().into(),
            command: "unused".into(),
            argv: None,
            cwd: root.to_string_lossy().into_owned(),
            timeout_ms: None,
        },
    )
    .unwrap();
    assert!(retry.task.forgotten);
    assert!(
        !dir.exists(),
        "a forgotten token must not create another execution"
    );
}

#[cfg(unix)]
#[test]
fn successful_and_failed_commands_retain_both_streams_without_sessions() {
    for (name, text, code) in [
        ("success", "printf stdout; printf stderr >&2", 0),
        ("failure", "printf stdout; printf stderr >&2; exit 7", 7),
    ] {
        let root = crate::testdir::TempTree::fresh(&format!("command-task-{name}"));
        let id = seed(&root, &format!("task-command-{name}-1234"), text, None);
        worker(&root, &id).unwrap();
        let answer = read(&root, &id);
        assert_eq!(answer.task.exit, Some(code));
        assert_eq!(
            answer.task.phase,
            if code == 0 {
                Phase::Completed
            } else {
                Phase::Failed
            }
        );
        assert_eq!(answer.stdout.text, "stdout");
        assert_eq!(answer.stderr.text, "stderr");
        assert!(!root.join(".khor/sessions").exists());
    }
}

#[cfg(unix)]
#[test]
fn cancellation_is_idempotent_and_confirms_the_owned_process_stopped() {
    let root = crate::testdir::TempTree::fresh("command-task-cancel");
    let id = seed(
        &root,
        "task-command-cancel-1234",
        "printf ready; sleep 30",
        None,
    );
    std::thread::scope(|scope| {
        let run = scope.spawn(|| worker(&root, &id));
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if read(&root, &id).stdout.text == "ready" {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(20));
        }
        let answer = handle(&root, Request::Cancel { id: id.clone() }).unwrap();
        assert_eq!(answer.task.phase, Phase::Cancelled, "{answer:?}");
        run.join().unwrap().unwrap();
        let repeated = handle(&root, Request::Cancel { id: id.clone() }).unwrap();
        assert_eq!(repeated.task.phase, Phase::Cancelled, "{repeated:?}");
    });
}

#[cfg(unix)]
#[test]
fn timeout_uses_the_same_owned_cleanup_and_preserves_output() {
    let root = crate::testdir::TempTree::fresh("command-task-timeout");
    let id = seed(
        &root,
        "task-command-timeout-1234",
        "printf before-timeout; sleep 30",
        Some(100),
    );
    worker(&root, &id).unwrap();
    let answer = read(&root, &id);
    assert_eq!(answer.task.phase, Phase::TimedOut);
    assert_eq!(answer.stdout.text, "before-timeout");
    assert!(answer.task.ended_ms.is_some());
}

#[cfg(unix)]
#[test]
fn large_stdout_and_stderr_are_drained_without_blocking_and_keep_their_tails() {
    let root = crate::testdir::TempTree::fresh("command-task-large-pipes");
    let id = seed(
        &root,
        "task-large-pipes-1234",
        "head -c 1200000 /dev/zero; printf OUT-END; head -c 1200000 /dev/zero >&2; printf ERR-END >&2",
        Some(10_000),
    );
    worker(&root, &id).unwrap();
    let answer = read(&root, &id);
    assert_eq!(answer.task.phase, Phase::Completed);
    assert!(answer.stdout.truncated && answer.stderr.truncated);
    let tail = handle(
        &root,
        Request::Read {
            id,
            stdout_cursor: answer.stdout.end - 7,
            stderr_cursor: answer.stderr.end - 7,
            limit: 7,
            wait_ms: 0,
        },
    )
    .unwrap();
    assert_eq!(tail.stdout.text, "OUT-END");
    assert_eq!(tail.stderr.text, "ERR-END");
    assert!(!tail.stdout.truncated && !tail.stderr.truncated);
}

#[cfg(windows)]
#[test]
fn a_canonical_local_directory_remains_the_cmd_workers_directory() {
    let root = crate::testdir::TempTree::fresh("command-task-native-cwd");
    let cwd = root.join("中文 工作目录");
    fs::create_dir(&cwd).unwrap();
    fs::write(cwd.join("relative.txt"), b"correct-working-directory").unwrap();
    let canonical = fs::canonicalize(&cwd).unwrap();
    let canonical = canonical.to_str().unwrap();
    let id = seed(&root, "task-cmd-native-cwd-1234", "type relative.txt", None);
    let dir = task_dir(&root, &id).unwrap();
    let mut current = record(&dir).unwrap();
    current.status.cwd = canonical.into();
    save(&dir, &current).unwrap();
    worker(&root, &id).unwrap();
    let answer = read(&root, &id);
    assert_eq!(answer.task.phase, Phase::Completed);
    assert_eq!(answer.task.exit, Some(0));
    assert_eq!(answer.stdout.text, "correct-working-directory");
    assert!(answer.stderr.text.is_empty(), "{}", answer.stderr.text);
    assert_eq!(
        answer.task.cwd, canonical,
        "retain the durable canonical identity"
    );
}

#[test]
fn cancelled_pipe_retry_preserves_its_drained_tail_without_an_output_failure() {
    struct CancelledRetry {
        control: Arc<Control>,
        tail: std::io::Cursor<Vec<u8>>,
    }
    impl Read for CancelledRetry {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            let count = self.tail.read(bytes)?;
            if count != 0 {
                return Ok(count);
            }
            // Reproduce the owned Unix Pipe's WouldBlock -> cancellation ->
            // retry error after its last successfully read output chunk.
            self.control.terminate_now();
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "agent pipe closed",
            ))
        }
    }
    let root = crate::testdir::TempTree::fresh("command-cancelled-retry");
    let id = seed(&root, "task-cancelled-retry-1234", "unused", None);
    let dir = task_dir(&root, &id).unwrap();
    let control = Arc::new(Control::default());
    let reader = CancelledRetry {
        control: control.clone(),
        tail: std::io::Cursor::new(b"last-output".to_vec()),
    };
    drain(dir.clone(), "stdout", Box::new(reader), control).unwrap();
    assert_eq!(
        read_output(&dir, "stdout", 0, READ_LIMIT).unwrap().text,
        "last-output"
    );
}

#[test]
fn read_and_retention_failures_are_not_disguised_as_successful_cancellation() {
    struct FailedRead(std::io::ErrorKind);
    impl Read for FailedRead {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(self.0, "fixture read failure"))
        }
    }
    for (name, cancelled, kind) in [
        (
            "unexpected-broken-pipe",
            false,
            std::io::ErrorKind::BrokenPipe,
        ),
        (
            "cancelled-permission-error",
            true,
            std::io::ErrorKind::PermissionDenied,
        ),
    ] {
        let root = crate::testdir::TempTree::fresh(name);
        let id = seed(&root, "task-real-read-error-1234", "unused", None);
        let dir = task_dir(&root, &id).unwrap();
        let control = Arc::new(Control::default());
        if cancelled {
            control.terminate_now();
        }
        assert!(
            drain(dir, "stdout", Box::new(FailedRead(kind)), control)
                .unwrap_err()
                .contains("fixture read failure")
        );
    }
    let root = crate::testdir::TempTree::fresh("command-cancelled-retention-error");
    let id = seed(&root, "task-real-write-error-1234", "unused", None);
    let dir = task_dir(&root, &id).unwrap();
    // The output bytes were read before cancellation, but committing them
    // still fails. That disk failure must not become an acknowledged cancel.
    std::fs::create_dir(dir.join("stdout-0.bin")).unwrap();
    let control = Arc::new(Control::default());
    control.terminate_now();
    assert!(
        drain(
            dir,
            "stdout",
            Box::new(std::io::Cursor::new(b"tail".to_vec())),
            control
        )
        .is_err()
    );
}
