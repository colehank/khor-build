//! Bounded login-environment inspection for a native GUI/background launcher.
#[cfg(not(windows))]
use crate::agent_process::{Control, Session};
use std::path::Path;
#[cfg(not(windows))]
use std::{
    io::Read,
    process::Command,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[cfg(not(windows))]
const TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(not(windows))]
const OUTPUT_LIMIT: usize = 64 * 1024;

/// Unsupported shell families retain the launcher's environment instead of
/// receiving Unix flags that mean something else to their native parser.
pub(crate) fn probe(shell: &str, script: &str) -> Option<String> {
    let name = Path::new(shell).file_name()?.to_str()?;
    #[cfg(windows)]
    {
        let _ = (name, script);
        None
    }
    #[cfg(not(windows))]
    {
        if !matches!(name, "zsh" | "bash" | "sh" | "dash" | "ksh" | "fish") {
            return None;
        }
        let mut command = Command::new(shell);
        command.args(["-lic", script]);
        capture(command, TIMEOUT, OUTPUT_LIMIT)
    }
}

#[cfg(not(windows))]
fn drain(
    mut reader: Box<dyn Read + Send>,
    keep: bool,
    limit: usize,
    control: &Control,
) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    let mut total = 0usize;
    let mut bytes = [0; 8192];
    loop {
        let count = reader.read(&mut bytes).ok()?;
        if count == 0 {
            return Some(output);
        }
        total = total.checked_add(count)?;
        if total > limit {
            control.terminate_now();
            return None;
        }
        if keep {
            output.extend_from_slice(&bytes[..count]);
        }
    }
}

#[cfg(not(windows))]
fn capture(mut command: Command, timeout: Duration, limit: usize) -> Option<String> {
    command.env_clear().envs(std::env::vars_os());
    let control = Arc::new(Control::default());
    let owner = control.clone();
    let (done, result) = mpsc::sync_channel(1);
    // Process creation and kernel reaping stay with this owner even when the
    // caller's budget expires. The caller never blocks on a pipe or join.
    std::thread::Builder::new()
        .name("user-environment-probe".into())
        .spawn(move || {
            let answer = capture_owned(command, &owner, timeout, limit);
            let _ = done.send(answer);
        })
        .ok()?;
    match result.recv_timeout(timeout) {
        Ok(answer) => answer,
        Err(_) => {
            control.terminate_now();
            None
        }
    }
}

#[cfg(not(windows))]
fn capture_owned(
    command: Command,
    control: &Arc<Control>,
    timeout: Duration,
    limit: usize,
) -> Option<String> {
    let (mut session, pipes) = Session::spawn(command, control.clone()).ok()?;
    drop(pipes.stdin);
    let (output_tx, output_rx) = mpsc::channel();
    let stdout_control = control.clone();
    let stderr_control = control.clone();
    let stderr_tx = output_tx.clone();
    let stdout = std::thread::spawn(move || {
        let bytes = drain(pipes.stdout, true, limit, &stdout_control);
        let _ = output_tx.send((true, bytes));
    });
    let stderr = std::thread::spawn(move || {
        let bytes = drain(pipes.stderr, false, limit, &stderr_control);
        let _ = stderr_tx.send((false, bytes));
    });
    let cleanup_budget = Duration::from_millis(250).min(timeout / 4);
    let deadline = Instant::now() + timeout.saturating_sub(cleanup_budget);
    let mut exit = None;
    let mut bytes = None;
    let mut readers = 0;
    let mut valid = true;
    while exit.is_none() || readers < 2 {
        if Instant::now() >= deadline {
            valid = false;
            control.terminate_now();
        }
        if let Some(result) = session.poll_exit() {
            exit = Some(result);
        }
        match output_rx.recv_timeout(Duration::from_millis(20)) {
            Ok((stdout, output)) => {
                readers += 1;
                valid &= output.is_some();
                if output.is_none() {
                    control.terminate_now();
                }
                if stdout {
                    bytes = output;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) if readers < 2 => {
                valid = false;
                control.terminate_now();
                break;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
    let finished = session.finish(false).ok();
    let stdout_joined = stdout.join().is_ok();
    let stderr_joined = stderr.join().is_ok();
    let joined = stdout_joined && stderr_joined;
    if !valid
        || !joined
        || !finished.is_some_and(|status| status.success())
        || !exit?.ok()?.success()
    {
        return None;
    }
    String::from_utf8(bytes?).ok()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn shell(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn captures_the_owned_shell_and_preserves_binary_field_separators() {
        assert_eq!(
            capture(
                shell("printf 'one\\000two\\000'"),
                Duration::from_secs(2),
                1024
            ),
            Some("one\0two\0".into())
        );
    }

    #[test]
    fn a_hanging_startup_returns_within_the_callers_budget() {
        let began = Instant::now();
        assert_eq!(
            capture(shell("sleep 30"), Duration::from_millis(100), 1024),
            None
        );
        assert!(began.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn stdout_and_stderr_floods_are_both_bounded() {
        for script in [
            "while :; do printf 0123456789; done",
            "while :; do printf 0123456789 >&2; done",
        ] {
            let began = Instant::now();
            assert_eq!(capture(shell(script), Duration::from_secs(2), 1024), None);
            assert!(began.elapsed() < Duration::from_secs(3));
        }
    }

    #[test]
    fn background_descendants_cannot_hold_the_probe_output_open() {
        let began = Instant::now();
        assert_eq!(
            capture(
                shell("sleep 30 & printf done"),
                Duration::from_secs(2),
                1024
            ),
            Some("done".into())
        );
        assert!(began.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn native_or_unknown_shells_do_not_receive_unix_login_flags() {
        assert_eq!(probe("powershell.exe", "unused"), None);
        assert_eq!(probe("cmd.exe", "unused"), None);
        assert_eq!(probe("custom-shell", "unused"), None);
    }
}
