//! Windows process observations use owned handles, never Unix signal emulation.
use std::{
    os::windows::{
        ffi::OsStringExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::PathBuf,
    sync::OnceLock,
};
use windows_sys::Win32::{
    Foundation::{
        ERROR_ACCESS_DENIED, FILETIME, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        SetHandleInformation, WAIT_TIMEOUT,
    },
    System::{
        Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
        Threading::{
            GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
            QueryFullProcessImageNameW, SYNCHRONIZATION_SYNCHRONIZE, TerminateProcess,
            WaitForSingleObject,
        },
    },
};

/// Process-owned transport handles must not travel to detached hosts as extra
/// inheritable handles. Explicit Stdio::inherit still works: std duplicates
/// that selected handle for its child. Run once at entry, before any spawning,
/// rather than temporarily changing process-wide flags around each spawn.
pub(crate) fn privatize_standard_handles() -> Result<(), String> {
    static PREPARED: OnceLock<Result<(), String>> = OnceLock::new();
    PREPARED
        .get_or_init(|| {
            for (name, which) in [
                ("stdin", STD_INPUT_HANDLE),
                ("stdout", STD_OUTPUT_HANDLE),
                ("stderr", STD_ERROR_HANDLE),
            ] {
                let handle = unsafe { GetStdHandle(which) };
                if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                    continue;
                }
                if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0 {
                    return Err(format!(
                        "Cannot isolate process {name}: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            }
            Ok(())
        })
        .clone()
}

fn open(pid: u32, access: u32) -> std::io::Result<OwnedHandle> {
    if pid == 0 {
        return Err(std::io::Error::from_raw_os_error(87));
    }
    let handle = unsafe { OpenProcess(access, 0, pid) };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

pub(crate) fn alive(pid: u32) -> bool {
    match open(pid, SYNCHRONIZATION_SYNCHRONIZE) {
        Ok(handle) => unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) == WAIT_TIMEOUT },
        Err(error) => error.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32),
    }
}

pub(crate) fn parent_id() -> Option<u32> {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
    let pid = Pid::from_u32(std::process::id());
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    system.process(pid)?.parent().map(Pid::as_u32)
}

pub(crate) fn executable(pid: u32) -> Option<PathBuf> {
    let handle = open(pid, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(handle.as_raw_handle(), 0, buffer.as_mut_ptr(), &mut size)
    } == 0
    {
        return None;
    }
    Some(std::ffi::OsString::from_wide(&buffer[..size as usize]).into())
}

pub(crate) fn terminate(pid: u32, descendants: bool) -> Result<(), String> {
    let pids = if descendants {
        crate::adaptor::Procs::snapshot().subtree(&[pid])
    } else {
        vec![pid]
    };
    // Open before terminating: a reused numeric pid cannot change an owned handle.
    let mut handles = Vec::new();
    for pid in pids {
        match open(pid, PROCESS_TERMINATE | SYNCHRONIZATION_SYNCHRONIZE) {
            Ok(handle) => handles.push(handle),
            Err(error) if error.raw_os_error() == Some(87) => {} // already exited
            Err(error) => return Err(error.to_string()),
        }
    }
    terminate_handles(handles)
}

fn terminate_handles(handles: Vec<OwnedHandle>) -> Result<(), String> {
    for handle in &handles {
        if unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) } != WAIT_TIMEOUT {
            continue;
        }
        if unsafe { TerminateProcess(handle.as_raw_handle(), 1) } == 0
            && unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) } == WAIT_TIMEOUT
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    wait_handles(&handles)
}

fn wait_handles<'a>(handles: impl IntoIterator<Item = &'a OwnedHandle>) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    for handle in handles {
        let remaining = deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_millis() as u32;
        if unsafe { WaitForSingleObject(handle.as_raw_handle(), remaining) }
            != windows_sys::Win32::Foundation::WAIT_OBJECT_0
        {
            return Err(std::io::Error::from_raw_os_error(
                windows_sys::Win32::Foundation::ERROR_TIMEOUT as i32,
            )
            .to_string());
        }
    }
    Ok(())
}

fn running(handle: &OwnedHandle) -> Result<bool, String> {
    match unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) } {
        windows_sys::Win32::Foundation::WAIT_OBJECT_0 => Ok(false),
        WAIT_TIMEOUT => Ok(true),
        _ => Err(std::io::Error::last_os_error().to_string()),
    }
}

fn creation_ticks(handle: &OwnedHandle) -> Result<u64, String> {
    let mut created = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe {
        GetProcessTimes(
            handle.as_raw_handle(),
            &mut created,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let ticks = (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
    ticks
        .checked_sub(116_444_736_000_000_000)
        .ok_or_else(|| "Process creation time cannot identify its owner".into())
}

/// Keep exact kernel process objects across authentication and termination.
/// A numeric PID, including a descendant from an older snapshot, is not an
/// authority: creation and current parentage must still identify this tree.
pub(crate) fn terminate_owned_tree(
    root: u32,
    procs: &crate::adaptor::Procs,
    marker_written_100ns: u128,
    authenticate: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    struct Pinned {
        pid: u32,
        born: u64,
        handle: OwnedHandle,
    }
    let mut pinned = Vec::new();
    for pid in procs.subtree(&[root]) {
        let expected = procs
            .get(pid)
            .ok_or("Owned process is absent from its identity snapshot")?;
        let handle = match open(
            pid,
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | SYNCHRONIZATION_SYNCHRONIZE,
        ) {
            Ok(handle) => handle,
            Err(error) if error.raw_os_error() == Some(87) => continue,
            Err(error) => return Err(error.to_string()),
        };
        let born = creation_ticks(&handle)?;
        if expected.started_ms < 0
            || born / 10_000_000 != expected.started_ms as u64 / 1000
            || (pid == root && u128::from(born) > marker_written_100ns)
        {
            return Err("Process identity changed; refusing to close a reused PID".into());
        }
        pinned.push(Pinned { pid, born, handle });
    }
    let Some(owner) = pinned.iter().find(|process| process.pid == root) else {
        // Its old child PID is never a substitute for a departed owner.
        return Ok(());
    };
    if !running(&owner.handle)? {
        return wait_handles(pinned.iter().map(|process| &process.handle));
    }
    if let Err(error) = authenticate() {
        // A genuine host may finish while the cookie handshake is in flight.
        // Only its already pinned kernel object can prove that ending. Never
        // turn a failed authentication into permission to terminate anything.
        return if !running(&owner.handle)? {
            wait_handles(pinned.iter().map(|process| &process.handle))
        } else {
            Err(error)
        };
    }
    let current = crate::adaptor::Procs::snapshot();
    for process in &pinned {
        if !running(&process.handle)? {
            continue;
        }
        let before = procs.get(process.pid).unwrap();
        let now = current
            .get(process.pid)
            .ok_or("Owned process disappeared during verification")?;
        if now.started_ms != before.started_ms || now.ppid != before.ppid {
            return Err("Process identity changed while authenticating its host".into());
        }
        if process.pid != root {
            let parent = pinned
                .iter()
                .find(|parent| Some(parent.pid) == now.ppid)
                .ok_or("Process is no longer inside the owned host tree")?;
            if process.born < parent.born {
                return Err(
                    "Process parent PID was reused; refusing to close an unrelated tree".into(),
                );
            }
        }
    }
    if !running(&owner.handle)? {
        return wait_handles(pinned.iter().map(|process| &process.handle));
    }
    // The authenticated root remains pinned until the last
    // termination; no subsequent operation reopens any of these numeric PIDs.
    pinned.sort_by_key(|process| process.pid == root);
    terminate_handles(pinned.into_iter().map(|process| process.handle).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_identity_and_exit_do_not_keep_dead_pids_alive() {
        let own = std::process::id();
        assert!(alive(own));
        assert!(executable(own).unwrap().is_file());
        assert!(parent_id().is_some_and(|pid| pid != own && alive(pid)));
        assert!(!alive(0));
        let mut child = std::process::Command::new("cmd.exe")
            .args(["/D", "/C", "ping -n 30 127.0.0.1 >NUL"])
            .spawn()
            .unwrap();
        assert!(alive(child.id()));
        terminate(child.id(), true).unwrap();
        child.wait().unwrap();
        assert!(!alive(child.id()));
    }

    struct WaitingChild(std::process::Child);
    impl WaitingChild {
        fn new() -> Self {
            Self(
                std::process::Command::new("cmd.exe")
                    .args(["/D", "/Q", "/C", "set /p owned_wait=waiting"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .unwrap(),
            )
        }
    }
    impl Drop for WaitingChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[test]
    fn owned_termination_rejects_stale_identity_authentication_and_foreign_descendants() {
        let mut owner = WaitingChild::new();
        let mut foreign = WaitingChild::new();
        let pid = owner.0.id();
        let procs = crate::adaptor::Procs::snapshot();
        let marker = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            / 100;
        assert!(
            terminate_owned_tree(pid, &procs, 0, || panic!(
                "stale marker reached authentication"
            ))
            .is_err()
        );
        assert!(owner.0.try_wait().unwrap().is_none());
        let mut stale = procs.get(pid).unwrap().clone();
        stale.started_ms += 1_000;
        assert!(
            terminate_owned_tree(
                pid,
                &crate::adaptor::Procs::of([(pid, stale)]),
                marker,
                || panic!("reused PID reached authentication")
            )
            .is_err()
        );
        let mut authenticated = false;
        assert!(
            terminate_owned_tree(pid, &procs, marker, || {
                authenticated = true;
                Err("cookie refused".into())
            })
            .is_err()
        );
        assert!(authenticated && owner.0.try_wait().unwrap().is_none());
        let mut unrelated = procs.get(foreign.0.id()).unwrap().clone();
        unrelated.ppid = Some(pid);
        let forged = crate::adaptor::Procs::of([
            (pid, procs.get(pid).unwrap().clone()),
            (foreign.0.id(), unrelated),
        ]);
        assert!(terminate_owned_tree(pid, &forged, marker, || Ok(())).is_err());
        assert!(owner.0.try_wait().unwrap().is_none());
        assert!(foreign.0.try_wait().unwrap().is_none());
        terminate_owned_tree(pid, &procs, marker, || Ok(())).unwrap();
        owner.0.wait().unwrap();
        assert!(
            foreign.0.try_wait().unwrap().is_none(),
            "an unrelated session must survive"
        );
    }

    #[test]
    fn owned_termination_only_waits_when_the_pinned_owner_ends_during_authentication() {
        let mut ending = WaitingChild::new();
        let mut unrelated = WaitingChild::new();
        let procs = crate::adaptor::Procs::snapshot();
        let marker = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            / 100;
        terminate_owned_tree(ending.0.id(), &procs, marker, || {
            // The same owned process object finishes before its handshake can
            // complete. This failure may wait, never authorize another kill.
            ending.0.kill().unwrap();
            ending.0.wait().unwrap();
            Err("host completed before Welcome".into())
        })
        .unwrap();
        assert!(unrelated.0.try_wait().unwrap().is_none());
    }
}
