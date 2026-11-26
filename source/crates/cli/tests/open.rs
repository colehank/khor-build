//! The persistent host through the real binary: the opener exits, the
//! host lives on (reparented to init), the six-word shell face moves
//! with the foreground process group, attachers replay-then-stream over
//! the socket, the ending is recorded, and close kills exactly one
//! session. Controls: wrong cookie refused by name, a live close leaves
//! no process behind. Every wait has a deadline.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use khor_node::host::{self, ClientOp, HostFile};

#[test]
fn literal_startup_and_structured_arguments_cannot_be_combined() {
    let home = std::env::temp_dir().join(format!("khor-open-ambiguous-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    let output = khor(&home)
        .args([
            "open",
            "-d",
            "--command",
            "echo literal",
            "--",
            "echo",
            "argument",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success(), "ambiguous startup was accepted");
    assert!(
        output.stdout.is_empty(),
        "no terminal ID should be admitted"
    );
    assert!(
        !home.join(".khor/sessions").exists(),
        "argument rejection must happen before session registration"
    );
    let _ = std::fs::remove_dir_all(home);
}

fn khor(home: &Path) -> Command {
    // The subprocess resolves its CLI language from the environment. This
    // test process otherwise keeps the catalog's unresolved Chinese default,
    // so an English Idle/Interrupted row looked like a lifecycle timeout.
    khor_catalog::lang::set(khor_catalog::lang::from_env());
    let mut c = Command::new(env!("CARGO_BIN_EXE_khor"));
    c.env("KHOR_HOME", home)
        .env("KHOR_NAME", "box")
        .env("HOME", home)
        .env("KHOR_SHELL", "/bin/sh")
        .env_remove("KHOR_SHELL_LOGIN")
        .env_remove("ENV")
        .env_remove("ZDOTDIR")
        .env_remove("KHOR_SESSION");
    c
}

fn sessions(home: &Path) -> String {
    let out = khor(home).arg("sessions").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn wait_until(what: &str, secs: u64, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        if f() {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("timed out waiting for: {what}");
}

/// Kills a pid when it goes out of scope, however it goes.
///
/// A test that deliberately makes a process outlive its host is a test
/// that leaks one when it fails — and this one failed on purpose during
/// its own red check, leaving a `sh` that ignores SIGHUP running under
/// init with a `sleep` under it. An assertion panics past its cleanup;
/// a `Drop` does not.
struct Reaper(u32);

impl Drop for Reaper {
    fn drop(&mut self) {
        let _ = Command::new("kill")
            .args(["-9", &self.0.to_string()])
            .stderr(std::process::Stdio::null())
            .status();
    }
}

fn pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn open_detached(home: &Path, args: &[&str]) -> (String, PathBuf) {
    let out = khor(home)
        .arg("open")
        .arg("-d")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sid = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let dir = home
        .join(".khor")
        .join("sessions")
        .join(sid.replace('/', "-"));
    assert!(
        dir.join("host.json").exists(),
        "the host should have checked in"
    );
    (sid, dir)
}

/// Reads from the attach stream until `needle` shows up (or panics).
fn read_until(stream: &mut std::net::TcpStream, needle: &str, secs: u64) -> String {
    stream
        .set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(secs);
    let mut seen = Vec::new();
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => seen.extend_from_slice(&buf[..n]),
            Err(_) => {}
        }
        if String::from_utf8_lossy(&seen).contains(needle) {
            return String::from_utf8_lossy(&seen).into_owned();
        }
    }
    panic!(
        "never saw {needle:?} in: {}",
        String::from_utf8_lossy(&seen)
    );
}

/// **The host says what terminal it is; it does not pass one on.**
///
/// A 持久 session outlives its opener and takes attachers with
/// different terminals, so "whatever `TERM` the opener had" is not even
/// a coherent answer — and on a machine where khor installed itself and
/// runs `serve` as a daemon it is `unknown`, which is not a terminfo
/// entry. Measured on turing: every tmux bridge stood up there died at
/// birth (`open terminal failed: missing or unsuitable terminal:
/// unknown`), and the person clicking the row read 读不到帧:
/// Connection reset.
///
/// The opener is given a bogus `TERM` on purpose. Without it this test
/// passes on any developer's machine — the ambient `TERM` there is
/// already `xterm-256color`, so an implementation that simply inherits
/// would look correct exactly where nobody is affected, and stay broken
/// exactly where everybody is.
#[cfg(unix)]
#[test]
fn the_child_is_told_what_terminal_it_is_in_not_what_the_opener_had() {
    let home = std::env::temp_dir().join(format!("khor-term-{}", std::process::id()));
    let out = khor(&home)
        .env("TERM", "khor-not-a-real-terminfo-entry")
        .env("NO_COLOR", "1")
        .env("FORCE_COLOR", "0")
        .env("CLICOLOR", "0")
        .env("CLICOLOR_FORCE", "0")
        .args([
            "open",
            "-d",
            "--",
            "sh",
            "-c",
            "printf 'TERM=[%s] COLORTERM=[%s] NO_COLOR=[%s] FORCE_COLOR=[%s] CLICOLOR=[%s] CLICOLOR_FORCE=[%s] %s%s\\n' \"$TERM\" \"$COLORTERM\" \"${NO_COLOR-unset}\" \"${FORCE_COLOR-unset}\" \"${CLICOLOR-unset}\" \"${CLICOLOR_FORCE-unset}\" KHOR-ENV -DONE; sleep 30",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sid = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let dir = home
        .join(".khor")
        .join("sessions")
        .join(sid.replace('/', "-"));

    // The `sleep` outlives an assertion that panics; a Drop does not.
    let hf: HostFile =
        serde_json::from_str(&std::fs::read_to_string(dir.join("host.json")).unwrap()).unwrap();
    let _reap = Reaper(hf.child_pid);

    let mut s = host::connect(&dir, 100, 30).unwrap();
    let seen = read_until(&mut s, "KHOR-ENV-DONE", 10);
    assert!(
        seen.contains("TERM=[xterm-256color]"),
        "the child must be told khor's own screen model, not the opener's: {seen}"
    );
    for field in [
        "COLORTERM=[truecolor]",
        "NO_COLOR=[unset]",
        "FORCE_COLOR=[unset]",
        "CLICOLOR=[unset]",
        "CLICOLOR_FORCE=[unset]",
    ] {
        assert!(
            seen.contains(field),
            "PTY must own its color capability, missing {field}: {seen}"
        );
    }
    assert!(
        khor(&home)
            .args(["close", &sid])
            .status()
            .unwrap()
            .success()
    );
    let _ = std::fs::remove_dir_all(&home);
}

#[cfg(unix)]
#[test]
fn a_hosted_session_outlives_its_opener_and_walks_the_shell_face() {
    let home = std::env::temp_dir().join(format!("khor-open-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let (sid, dir) = open_detached(&home, &["--title", "work"]);
    assert!(sid.starts_with("shell/"), "{sid}");

    // Truly detached: the opener is gone (its command returned), and the
    // host has been reparented to init.
    let hf: HostFile =
        serde_json::from_str(&std::fs::read_to_string(dir.join("host.json")).unwrap()).unwrap();
    let ppid = Command::new("ps")
        .args(["-o", "ppid=", "-p", &hf.host_pid.to_string()])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&ppid.stdout).trim(),
        "1",
        "the host must not hang off anyone"
    );

    // The shell reaches its prompt: no foreground job = 空闲.
    wait_until("the prompt to read as idle", 15, || {
        sessions(&home)
            .lines()
            .any(|l| l.contains(&sid) && l.contains(khor_catalog::state::word("idle")))
    });

    // Attach over the socket: type into the pty, see the output.
    let mut s = host::connect(&dir, 100, 30).unwrap();
    host::write_frame(&mut s, &ClientOp::Input(b"echo khor-marker-ok\n".to_vec())).unwrap();
    read_until(&mut s, "khor-marker-ok", 10);

    // A foreground job flips the face to 忙碌 and back.
    host::write_frame(&mut s, &ClientOp::Input(b"sleep 3\n".to_vec())).unwrap();
    wait_until("the sleep to read as busy", 10, || {
        sessions(&home)
            .lines()
            .any(|l| l.contains(&sid) && l.contains(khor_catalog::state::word("busy")))
    });
    wait_until("the prompt to read as idle again", 10, || {
        sessions(&home)
            .lines()
            .any(|l| l.contains(&sid) && l.contains(khor_catalog::state::word("idle")))
    });

    // A late attacher gets the past from the replay ring without asking.
    let mut late = host::connect(&dir, 100, 30).unwrap();
    read_until(&mut late, "khor-marker-ok", 5);
    drop(late);

    // Control: the port alone opens nothing.
    let mut bad = std::net::TcpStream::connect(("127.0.0.1", hf.port)).unwrap();
    host::write_frame(
        &mut bad,
        &host::Hello {
            control: false,
            cookie: "not-the-cookie".into(),
            cols: 80,
            rows: 24,
        },
    )
    .unwrap();
    let w: host::Welcome = host::read_frame(&mut bad).unwrap();
    assert!(
        !w.ok && w.why == khor_catalog::msg::wrong_cookie(),
        "{}",
        w.why
    );

    // The ending is recorded even with nobody attached: exit 7 = 失败,
    // the row stays (sinks), the host leaves.
    host::write_frame(&mut s, &ClientOp::Input(b"exit 7\n".to_vec())).unwrap();
    wait_until("the ending to read as failed", 10, || {
        sessions(&home)
            .lines()
            .any(|l| l.contains(&sid) && l.contains(khor_catalog::state::word("failed")))
    });
    wait_until("the host to leave", 10, || !pid_alive(hf.host_pid));
    assert!(dir.exists(), "a settled row stays listed until closed");

    // Close removes the settled row.
    assert!(
        khor(&home)
            .args(["close", &sid])
            .status()
            .unwrap()
            .success()
    );
    assert!(!dir.exists());
    assert!(!sessions(&home).contains(&sid));

    let _ = std::fs::remove_dir_all(&home);
}

/// A failed startup program is a command result, not the terminal's lifetime.
#[cfg(unix)]
#[test]
fn a_nonzero_startup_command_returns_to_the_same_interactive_shell() {
    let home = std::env::temp_dir().join(format!("khor-startup-stays-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();
    let out = khor(&home)
        .env("KHOR_SHELL", "/bin/bash")
        .args([
            "open",
            "-d",
            "--title",
            "persistent-after-failure",
            "--",
            "sh",
            "-c",
            "printf 'STARTUP-%s-END\\n' COMMAND; exit 9",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sid = String::from_utf8(out.stdout).unwrap().trim().to_owned();
    let dir = home.join(".khor/sessions").join(sid.replace('/', "-"));
    let hf = host::read_host_file(&dir).unwrap();
    let _reap_host = Reaper(hf.host_pid);
    let _reap_shell = Reaper(hf.child_pid);
    let mut terminal = host::connect(&dir, 80, 24).unwrap();
    read_until(&mut terminal, "STARTUP-COMMAND-END", 10);
    wait_until(
        "nonzero startup to finish inside a living shell",
        10,
        || {
            let saved = khor_node::commands::read_saved(&dir);
            saved.is_some_and(|saved| saved.recent.iter().any(|c| c.exit == Some(9)))
        },
    );
    let node = khor_node::Node::open(home.clone()).unwrap();
    let row = node
        .sessions()
        .unwrap()
        .into_iter()
        .find(|r| r.session.id.0 == sid)
        .unwrap();
    assert!(
        !row.session.dead && pid_alive(hf.child_pid),
        "the interactive shell must survive its command"
    );
    host::write_frame(
        &mut terminal,
        &ClientOp::Input(b"printf 'FOLLOWUP-%s-END\\n' COMMAND\r".to_vec()),
    )
    .unwrap();
    read_until(&mut terminal, "FOLLOWUP-COMMAND-END", 10);
    assert!(
        khor(&home)
            .args(["close", &sid])
            .status()
            .unwrap()
            .success()
    );
    let _ = std::fs::remove_dir_all(home);
}

/// **A host that dies while its child lives must not read as 失败.**
///
/// The registry records the host's pid, and the missing-ending rule
/// turns a dead recorded pid into 失败 — which for a hosted session
/// means khor's own plumbing dying is reported as the user's work
/// failing. It is 中断: the process is there, and 中断 vs 失败 is
/// exactly the question of whether it is (`live::face` derives it).
///
/// Real, not simulated: a real host, a real child, `kill -9` on the
/// host only, the word read back out of the real binary.
///
/// **The child has to ignore SIGHUP or there is nothing to test.**
/// Measured on this machine: killing the host closes the pty master, and
/// the kernel hangs up the foreground group — a plain `sleep 300` dies
/// with it, and so does an interactive `zsh`. Only a child that ignores
/// the signal outlives its host, which is both why this hole is narrow
/// and why the test has to say `trap "" HUP` out loud.
#[cfg(unix)]
#[test]
fn a_host_that_dies_while_its_child_lives_is_not_a_failure() {
    let home = std::env::temp_dir().join(format!("khor-orphan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let work_pid = home.join("work.pid");
    let startup = format!(
        r#"trap "" HUP; sleep 300 & eval 'work=$!'; printf '%s' "$work" > '{}'; printf 'KHOR-%s-ARMED\n' ORPHAN; wait "$work""#,
        work_pid.display().to_string().replace('\'', "'\\''")
    );
    let (sid, dir) = open_detached(&home, &["--command", &startup]);
    let hf: HostFile =
        serde_json::from_str(&std::fs::read_to_string(dir.join("host.json")).unwrap()).unwrap();
    // Whatever happens below, this child does not outlive the test: it
    // ignores SIGHUP, so nothing else would take it away.
    let _reap = Reaper(hf.child_pid);
    assert!(
        pid_alive(hf.host_pid) && pid_alive(hf.child_pid),
        "control: both are up"
    );

    // The startup is terminal input now; wait until the recorded outer
    // shell has installed its HUP handler before killing its host.
    wait_until("the HUP-ignoring work process to be recorded", 10, || {
        std::fs::read_to_string(&work_pid)
            .ok()
            .and_then(|pid| pid.parse::<u32>().ok())
            .is_some()
    });
    let work = std::fs::read_to_string(&work_pid).unwrap().parse().unwrap();
    let _reap_work = Reaper(work);
    let mut armed = host::connect(&dir, 80, 24).unwrap();
    read_until(&mut armed, "KHOR-ORPHAN-ARMED", 10);
    drop(armed);

    // The host alone.
    Command::new("kill")
        .args(["-9", &hf.host_pid.to_string()])
        .status()
        .unwrap();
    wait_until("the host to be gone", 5, || !pid_alive(hf.host_pid));
    assert!(
        pid_alive(hf.child_pid),
        "the child ignores SIGHUP, so it is still working"
    );

    wait_until("the row to say 中断", 10, || {
        sessions(&home)
            .lines()
            .any(|l| l.contains(&sid) && l.contains(khor_catalog::state::word("errored")))
    });
    assert!(
        !sessions(&home).contains(khor_catalog::state::word("failed")),
        "失败 would tell the user to start over on work that is still running"
    );

    // The control group: now the child goes too, and the ordinary
    // missing-ending rule applies again. Without this half, a `face`
    // that simply never said 失败 would pass the assertion above.
    Command::new("kill")
        .args(["-9", &hf.child_pid.to_string()])
        .status()
        .unwrap();
    wait_until("the child to be gone", 5, || !pid_alive(hf.child_pid));
    wait_until("the row to say 失败", 10, || {
        sessions(&home)
            .lines()
            .any(|l| l.contains(&sid) && l.contains(khor_catalog::state::word("failed")))
    });

    assert!(
        khor(&home)
            .args(["close", &sid])
            .status()
            .unwrap()
            .success()
    );
    let _ = std::fs::remove_dir_all(&home);
}

#[cfg(unix)]
#[test]
fn closing_a_live_hosted_session_leaves_no_process_behind() {
    let home = std::env::temp_dir().join(format!("khor-open-close-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let (sid, dir) = open_detached(&home, &[]);
    let hf: HostFile =
        serde_json::from_str(&std::fs::read_to_string(dir.join("host.json")).unwrap()).unwrap();
    assert!(pid_alive(hf.host_pid) && pid_alive(hf.child_pid));

    assert!(
        khor(&home)
            .args(["close", &sid])
            .status()
            .unwrap()
            .success()
    );
    assert!(!dir.exists(), "closing forgets the session");
    wait_until("the child to be gone", 5, || !pid_alive(hf.child_pid));
    wait_until("the host to be gone", 5, || !pid_alive(hf.host_pid));

    let _ = std::fs::remove_dir_all(&home);
}

/// **The person is told what khor could not find, not how long it
/// waited.**
///
/// 2026-08-31, on the machine this is written on: `khor serve` is a
/// launchd agent, so it searches `/usr/bin:/bin:/usr/sbin:/sbin` —
/// and `tmux`, which every tui row's terminal is a client of, lives in
/// `/opt/homebrew/bin`. The host died at exec in about a millisecond,
/// its three streams were on `/dev/null` because that is what makes it
/// outlive its opener, and so the one thing the person saw was the
/// opener's timeout: 宿主没在 5 秒内就位.
///
/// **A sentence about speed, for something that never waited.** Read
/// straight, it says the machine is slow or the session is wedged, and
/// both send somebody to look at a thing that is not sick — the same
/// shape as the `npx` refusal that blamed a working agent for not
/// speaking ACP.
///
/// The `PATH`s are pinned on purpose, both of them: with an ambient
/// `PATH` this passes on any developer's machine, where the program is
/// found and no message is ever produced, and would stay broken exactly
/// where everybody is affected. `KHOR_PATH` is pinned too because
/// otherwise the login-shell repair finds the program and there is
/// nothing to report.
#[cfg(unix)]
#[test]
fn a_host_that_cannot_find_its_program_says_so_instead_of_timing_out() {
    let home = std::env::temp_dir().join(format!("khor-nopath-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    let bare = "/usr/bin:/bin:/usr/sbin:/sbin";
    let out = khor(&home)
        .env("PATH", bare)
        .env("KHOR_PATH", bare)
        .args([
            "open",
            "-d",
            "--tui",
            "--",
            "khor-no-such-program",
            "--hello",
        ])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "it cannot have opened: {said}");
    assert!(
        said.contains("khor-no-such-program"),
        "the refusal must name the program khor could not find: {said}"
    );
    assert!(
        said.contains("/usr/sbin"),
        "and where it looked, which is the other half of the next step: {said}"
    );
    assert!(
        !said.contains("5 秒"),
        "and it must not be reported as a wait — it never waited: {said}"
    );
    let _ = std::fs::remove_dir_all(&home);
}

/// The control the sentence above cannot supply: **the same command,
/// the same binary, the same everything, and only `KHOR_PATH`
/// differing.** Without this the test above passes for an
/// implementation that merely says a nicer sentence and still cannot
/// start anything — a message is not a repair.
#[cfg(unix)]
#[test]
fn the_same_command_starts_once_khor_is_told_where_to_look() {
    let home = std::env::temp_dir().join(format!("khor-onpath-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    let bin = home.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    // A real program, reachable only through a directory that is not on
    // `PATH` — which is the whole of what launchd does to `khor serve`.
    let prog = bin.join("khor-test-only-here");
    std::fs::write(&prog, "#!/bin/sh\nsleep 30\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&prog, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let bare = "/usr/bin:/bin:/usr/sbin:/sbin";

    let blind = khor(&home)
        .env("PATH", bare)
        .env("KHOR_PATH", bare)
        .args(["open", "-d", "--tui", "--", "khor-test-only-here"])
        .output()
        .unwrap();
    assert!(
        !blind.status.success(),
        "the control arm has to actually fail"
    );

    let told = khor(&home)
        .env("PATH", bare)
        .env("KHOR_PATH", format!("{}:{bare}", bin.display()))
        .args(["open", "-d", "--tui", "--", "khor-test-only-here"])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&told.stdout),
        String::from_utf8_lossy(&told.stderr)
    );
    assert!(
        told.status.success(),
        "only KHOR_PATH differs, and it must now start: {said}"
    );

    let sid = String::from_utf8_lossy(&told.stdout).trim().to_owned();
    let dir = home
        .join(".khor")
        .join("sessions")
        .join(sid.replace('/', "-"));
    let hf: HostFile =
        serde_json::from_str(&std::fs::read_to_string(dir.join("host.json")).unwrap()).unwrap();
    let _reap = Reaper(hf.child_pid);
    let _ = khor(&home).args(["close", &sid]).output();
    let _ = std::fs::remove_dir_all(&home);
}

#[cfg(unix)]
#[test]
fn a_private_host_does_not_inherit_an_unrelated_native_agent_store() {
    let home = std::env::temp_dir().join(format!("khor-host-vendor-home-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    let out = khor(&home)
        .env("CODEX_HOME", "/unrelated-codex")
        .env("CLAUDE_CONFIG_DIR", "/unrelated-claude")
        .args([
            "open",
            "-d",
            "--",
            "sh",
            "-c",
            "printf 'codex=%s;claude=%s;%s%s\\n' \"$CODEX_HOME\" \"$CLAUDE_CONFIG_DIR\" do ne; sleep 30",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sid = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let dir = home.join(".khor/sessions").join(sid.replace('/', "-"));
    let hf = host::read_host_file(&dir).unwrap();
    let _reap = Reaper(hf.host_pid);
    let mut stream = host::connect(&dir, 80, 24).unwrap();
    let shown = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read_until(&mut stream, ";done", 10)
    })) {
        Ok(shown) => shown,
        Err(failure) => {
            // The viewer replay is a rendered snapshot, which omits OSC
            // readiness marks. Capture the host's raw, read-only state before
            // unwinding kills this private fixture's host.
            let diagnostic = khor_node::terminal_control::handle(
                &home,
                khor_node::terminal_control::Request::Read {
                    session: sid.clone(),
                    cursor: None,
                    limit: 8192,
                    wait_ms: 0,
                },
            );
            eprintln!("private startup host diagnostic: {diagnostic:?}");
            eprintln!(
                "private startup command ledger: {:?}",
                khor_node::commands::read_saved(&dir)
            );
            std::panic::resume_unwind(failure)
        }
    };
    let owned_home = std::fs::canonicalize(&home).unwrap();
    assert!(
        shown.contains(&format!(
            "codex={};claude={};done",
            owned_home.join(".codex").display(),
            owned_home.join(".claude").display()
        )),
        "wrong store: {shown}"
    );
    assert!(
        khor(&home)
            .args(["close", &sid])
            .status()
            .unwrap()
            .success()
    );
    let _ = std::fs::remove_dir_all(home);
}

#[cfg(windows)]
mod windows {
    use super::*;
    use khor_node::terminal_control::{self, HostOutput, Reply, Request};
    use std::{
        fs::{self, File},
        io::{self, Read},
        process::{Child, ExitStatus, Output, Stdio},
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc,
        },
        time::{SystemTime, UNIX_EPOCH},
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
            std::thread::sleep(Duration::from_millis(25));
        }
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
                Err(_) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }

    struct OpenerChild(Child);

    impl Drop for OpenerChild {
        fn drop(&mut self) {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            let killed = self.0.kill();
            if let Err(error) = wait_for_exit(&mut self.0, Duration::from_secs(5)) {
                let message = format!("piped opener cleanup failed: kill={killed:?}, reap={error}");
                eprintln!("{message}");
                if !std::thread::panicking() {
                    panic!("{message}");
                }
            }
        }
    }

    #[derive(Default)]
    struct StdoutReader {
        handle: Option<std::thread::JoinHandle<()>>,
        home: PathBuf,
    }

    impl Drop for StdoutReader {
        fn drop(&mut self) {
            let Some(reader) = self.handle.take() else {
                return;
            };
            let deadline = Instant::now() + Duration::from_secs(5);
            while !reader.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(25));
            }
            let failure = if !reader.is_finished() {
                Some("stdout reader did not exit after closing the terminal")
            } else if reader.join().is_err() {
                Some("stdout reader panicked")
            } else {
                None
            };
            if let Some(failure) = failure {
                let message = format!("{failure}; preserving {:?}", self.home);
                eprintln!("{message}");
                if !std::thread::panicking() {
                    panic!("{message}");
                }
            }
        }
    }

    struct Fixture(PathBuf, std::cell::Cell<bool>);

    impl Fixture {
        fn new() -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let home = std::env::temp_dir()
                .join(format!("khor-open-windows-{}-{stamp}", std::process::id()));
            fs::create_dir_all(home.join(".khor")).unwrap();
            fs::write(
                home.join(".khor/relays.json"),
                br#"{"relays":[],"only_mine":true}"#,
            )
            .unwrap();
            Self(home, std::cell::Cell::new(false))
        }

        fn command(&self) -> Command {
            let mut command = Command::new(env!("CARGO_BIN_EXE_khor"));
            command
                .env("KHOR_HOME", &self.0)
                .env("KHOR_NAME", "open-windows")
                .env("KHOR_NO_MDNS", "1")
                .env("KHOR_RUNTIME_DIR", self.0.join("runtime"))
                .env("KHOR_RUNTIME_INSTALL_SOURCE", env!("CARGO_BIN_EXE_khor"))
                .env(
                    "KHOR_SHELL",
                    PathBuf::from(std::env::var_os("SystemRoot").unwrap())
                        .join("System32/WindowsPowerShell/v1.0/powershell.exe"),
                )
                .env_remove("KHOR_SHELL_INTEGRATION")
                .env_remove("KHOR_SESSION")
                .current_dir(&self.0);
            if let Some(path) = std::env::var_os("PATH") {
                command.env("KHOR_PATH", path);
            }
            command
        }

        fn cli_result(&self, args: &[&str]) -> io::Result<Output> {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let serial = NEXT.fetch_add(1, Ordering::Relaxed);
            let stdout = self.0.join(format!("cli-{serial}.stdout.log"));
            let stderr = self.0.join(format!("cli-{serial}.stderr.log"));
            let mut child = self
                .command()
                .args(args)
                .stdin(Stdio::null())
                .stdout(File::create_new(&stdout)?)
                .stderr(File::create_new(&stderr)?)
                .spawn()?;
            let status = match wait_for_exit(&mut child, Duration::from_secs(40)) {
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
            // Descendants can retain their writer handles; regular files
            // let the test read available output without waiting for pipe EOF.
            Ok(Output {
                status,
                stdout: fs::read(stdout)?,
                stderr: fs::read(stderr)?,
            })
        }

        fn cli(&self, args: &[&str]) -> Option<Output> {
            eprintln!("Windows open fixture CLI starting: {args:?}");
            match self.cli_result(args) {
                Ok(output) => {
                    eprintln!("Windows open fixture CLI finished: {}", output.status);
                    Some(output)
                }
                Err(error) => {
                    eprintln!(
                        "Windows open fixture CLI failed: {args:?}: {error}; preserving {:?}",
                        self.0
                    );
                    None
                }
            }
        }

        fn read_until(&self, id: &str, marker: &str) -> HostOutput {
            let deadline = Instant::now() + Duration::from_secs(30);
            let mut cursor = None;
            let mut text = String::new();
            loop {
                let reply = terminal_control::handle(
                    &self.0,
                    Request::Read {
                        session: id.into(),
                        cursor,
                        limit: 16 * 1024,
                        wait_ms: 100,
                    },
                )
                .unwrap();
                let Reply::Read { output, .. } = reply else {
                    panic!("unexpected terminal reply: {reply:?}")
                };
                text.push_str(&String::from_utf8_lossy(&output.bytes));
                if text.contains(marker) {
                    return output;
                }
                assert!(
                    Instant::now() < deadline,
                    "native Windows terminal never produced {marker:?}: {text}"
                );
                cursor = Some(output.cursor);
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let mut failures = Vec::new();
            // Discover admitted hosts even after a failed/late CLI response.
            // The native close path owns process identity; a recorded PID alone
            // cannot safely authorize taskkill after a deadline.
            if !self.1.get() {
                if let Ok(entries) = fs::read_dir(self.0.join(".khor/sessions")) {
                    for entry in entries.flatten() {
                        let directory = entry.path();
                        let id = fs::read(directory.join("meta.json"))
                            .ok()
                            .and_then(|bytes| {
                                serde_json::from_slice::<serde_json::Value>(&bytes).ok()
                            })
                            .and_then(|meta| meta["id"].as_str().map(str::to_owned));
                        let closed = id
                            .as_deref()
                            .and_then(|id| self.cli(&["close", id]))
                            .is_some_and(|output| output.status.success());
                        if !closed {
                            failures.push(format!("terminal cleanup unconfirmed: {directory:?}"));
                        }
                    }
                }
            }
            if failures.is_empty() && !std::thread::panicking() {
                if let Err(error) = remove_fixture_home(&self.0) {
                    failures.push(format!("fixture directory cleanup: {error}"));
                }
            } else if failures.is_empty() {
                eprintln!("Preserving failed Windows open fixture at {:?}", self.0);
            }
            if !failures.is_empty() {
                let message = format!(
                    "Windows terminal cleanup unconfirmed; preserving {:?}: {}",
                    self.0,
                    failures.join("; ")
                );
                eprintln!("{message}");
                if !std::thread::panicking() {
                    panic!("{message}");
                }
            }
        }
    }

    #[test]
    fn native_powershell_open_remains_interactive_after_a_failed_startup() {
        let fixture = Fixture::new();
        let output = fixture
            .cli(&[
                "open",
                "-d",
                "--command",
                "Write-Output ('STARTUP-' + 'WINDOWS'); & $env:ComSpec /d /c 'exit 9'",
            ])
            .expect("native open CLI did not finish");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let id = String::from_utf8(output.stdout).unwrap().trim().to_owned();
        assert!(id.starts_with("shell/"), "{id}");
        let directory = fixture.0.join(".khor/sessions").join(id.replace('/', "-"));
        let host = host::read_host_file(&directory).unwrap();
        assert_ne!(host.host_pid, host.child_pid);
        let observed = fixture.read_until(&id, "STARTUP-WINDOWS");
        assert!(
            observed.shell_ready && observed.startup_released,
            "headless open must initialize its native shell"
        );
        wait_until("native nonzero command to return to the prompt", 15, || {
            khor_node::commands::read_saved(&directory)
                .is_some_and(|saved| saved.recent.iter().any(|command| command.exit == Some(9)))
        });
        let node = khor_node::Node::open(fixture.0.clone()).unwrap();
        assert!(
            node.sessions()
                .unwrap()
                .iter()
                .any(|row| row.session.id.0 == id && !row.session.dead)
        );
        let acknowledged = terminal_control::handle(
            &fixture.0,
            Request::Input {
                session: id.clone(),
                token: "windows-open-followup".into(),
                generation: observed.generation,
                revision: observed.input_revision,
                text: "Write-Output ('FOLLOWUP-' + 'WINDOWS')".into(),
                enter: true,
            },
        )
        .unwrap();
        assert!(matches!(acknowledged, Reply::Input { .. }));
        let after_followup = fixture.read_until(&id, "FOLLOWUP-WINDOWS");
        let exiting = terminal_control::handle(
            &fixture.0,
            Request::Input {
                session: id.clone(),
                token: "windows-open-exit".into(),
                generation: after_followup.generation,
                revision: after_followup.input_revision,
                text: "exit 0".into(),
                enter: true,
            },
        )
        .unwrap();
        assert!(matches!(exiting, Reply::Input { .. }));
        let session = khor_node::SessionId(id.clone());
        wait_until("native terminal ending", 10, || {
            node.ended(&session).is_some()
        });
        assert_eq!(
            node.ended(&session).unwrap().ending,
            khor_node::live::Ending::Exit(0)
        );
        assert!(directory.exists(), "exiting retains the terminal history");
        let closed = fixture
            .cli(&["close", &id])
            .expect("native close CLI did not finish");
        assert!(
            closed.status.success(),
            "{}",
            String::from_utf8_lossy(&closed.stderr)
        );
        assert!(!directory.exists(), "close removes the retained record");
        assert!(node.ended(&session).is_none());
        fixture.1.set(true);
    }

    #[test]
    fn piped_opener_stdout_reaches_eof_while_its_terminal_stays_alive() {
        // Reverse declaration order guarantees: stop the opener, close all
        // admitted fixture terminals, then bound the pipe reader's join.
        let mut reader = StdoutReader::default();
        let fixture = Fixture::new();
        reader.home = fixture.0.clone();
        let stderr = fixture.0.join("piped-opener.stderr.log");
        let mut opener = OpenerChild(
            fixture
                .command()
                .args([
                    "open",
                    "-d",
                    "--command",
                    "Write-Output ('PIPE-' + 'ALIVE')",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(File::create_new(&stderr).unwrap())
                .spawn()
                .unwrap(),
        );
        let mut stdout = opener.0.stdout.take().unwrap();
        let (send, receive) = mpsc::channel();
        reader.handle = Some(std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stdout.read_to_end(&mut bytes).map(|_| bytes);
            let _ = send.send(result);
        }));
        let status = wait_for_exit(&mut opener.0, Duration::from_secs(40))
            .expect("piped opener did not exit within 40 seconds");
        assert!(
            status.success(),
            "{}",
            fs::read_to_string(stderr).unwrap_or_default()
        );
        eprintln!("Piped opener exited; waiting for actual stdout EOF");
        let bytes = receive
            .recv_timeout(Duration::from_secs(5))
            .expect("resident terminal retained the exited opener's stdout pipe")
            .expect("reading the piped opener's stdout failed");
        let id = String::from_utf8(bytes).unwrap().trim().to_owned();
        assert!(id.starts_with("shell/"), "{id:?}");
        let observed = fixture.read_until(&id, "PIPE-ALIVE");
        assert!(
            observed.shell_ready && observed.startup_released,
            "the terminal must remain alive after its opener's stdout reaches EOF"
        );
        let acknowledged = terminal_control::handle(
            &fixture.0,
            Request::Input {
                session: id.clone(),
                token: "piped-opener-followup".into(),
                generation: observed.generation,
                revision: observed.input_revision,
                text: "Write-Output ('AFTER-' + 'EOF')".into(),
                enter: true,
            },
        )
        .unwrap();
        assert!(matches!(acknowledged, Reply::Input { .. }));
        fixture.read_until(&id, "AFTER-EOF");
        let closed = fixture
            .cli(&["close", &id])
            .expect("native close CLI did not finish");
        assert!(
            closed.status.success(),
            "{}",
            String::from_utf8_lossy(&closed.stderr)
        );
        fixture.1.set(true);
    }
}
