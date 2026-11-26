use super::*;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let base = std::env::var_os("KHOR_TRANSFER_SCRATCH")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = base.join(format!(
            "khor-portable-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Worker(Child);
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn manifest(binding: &[u8], body: &[u8]) -> Manifest {
    Manifest {
        source: [51; 32],
        recipient: [52; 32],
        binding: binding.to_vec(),
        total_bytes: body.len() as u64,
        digest: *blake3::hash(body).as_bytes(),
    }
}

#[test]
fn portable_body_resumes_and_shares_quotas_with_legacy_directories() {
    let scratch = Scratch::new();
    let root = scratch.0.join("stages");
    let store = TransferStore::open(&root).unwrap();
    let body = vec![7; CHUNK_BYTES + 17];
    let m = manifest(b"portable", &body);
    publish(&root, &m, m.id().unwrap()).unwrap();
    let legacy = manifest(b"legacy directory", b"old");
    create_stage(
        &root.join(legacy.id().unwrap().to_string()),
        &legacy,
        legacy.id().unwrap(),
    )
    .unwrap();
    assert_eq!(store.reserved_bytes().unwrap(), body.len() as u64 + 3);
    let mut first = store.receive(m.clone()).unwrap();
    first.append(0, &body[..CHUNK_BYTES]).unwrap();
    assert!(matches!(store.receive(m.clone()), Err(Error::Busy)));
    drop(first);
    let mut second = store.receive(m.clone()).unwrap();
    assert_eq!(second.next_offset(), CHUNK_BYTES as u64);
    second
        .append(CHUNK_BYTES as u64, &body[CHUNK_BYTES..])
        .unwrap();
    assert!(matches!(
        second.progress().unwrap(),
        Progress::VerifiedComplete { .. }
    ));
    let mut tail = [0; 17];
    assert_eq!(
        second.read_verified(CHUNK_BYTES as u64, &mut tail).unwrap(),
        17
    );
    assert_eq!(tail, [7; 17]);
    drop(second);
    assert_eq!(store.cancel(&m).unwrap(), Cancelled::Removed);
    assert!(!root.join(DIRECTORY).exists());
    assert_eq!(store.reserved_bytes().unwrap(), 3);
    assert_eq!(store.cancel(&legacy).unwrap(), Cancelled::Removed);
    assert_eq!(store.reserved_bytes().unwrap(), 0);
}

#[test]
fn portable_publication_keeps_existing_names_and_external_link_targets() {
    for kind in ["empty", "sentinel", "file", "link", "dangling_owned_shape"] {
        let scratch = Scratch::new();
        let root = scratch.0.join("stages");
        let store = TransferStore::open(&root).unwrap();
        let m = manifest(b"collision", b"body");
        let dest = root.join(m.id().unwrap().to_string());
        let external = scratch.0.join("external");
        fs::write(&external, b"outside unchanged").unwrap();
        match kind {
            "file" => fs::write(&dest, b"existing file").unwrap(),
            "link" => std::os::unix::fs::symlink(&external, &dest).unwrap(),
            "dangling_owned_shape" => {
                std::os::unix::fs::symlink(target(&m.id().unwrap().to_string()), &dest).unwrap()
            }
            _ => {
                create_dir(&dest).unwrap();
                if kind == "sentinel" {
                    fs::write(dest.join("keep"), b"existing sentinel").unwrap();
                }
            }
        }
        assert!(publish(&root, &m, m.id().unwrap()).is_err());
        assert!(store.reserved_bytes().is_err());
        assert_eq!(fs::read(&external).unwrap(), b"outside unchanged");
        match kind {
            "file" => assert_eq!(fs::read(&dest).unwrap(), b"existing file"),
            "link" => assert_eq!(fs::read_link(&dest).unwrap(), external),
            "dangling_owned_shape" => assert_eq!(
                fs::read_link(&dest).unwrap(),
                target(&m.id().unwrap().to_string())
            ),
            "sentinel" => assert_eq!(fs::read(dest.join("keep")).unwrap(), b"existing sentinel"),
            _ => assert_eq!(fs::read_dir(&dest).unwrap().count(), 0),
        }
    }
}

#[test]
fn unknown_unpublished_or_cancelled_controls_are_preserved() {
    for cancelling in [false, true] {
        let scratch = Scratch::new();
        let root = scratch.0.join("stages");
        let store = TransferStore::open(&root).unwrap();
        let m = manifest(b"sentinel preservation", b"body");
        publish(&root, &m, m.id().unwrap()).unwrap();
        let object = root.join(target(&m.id().unwrap().to_string()));
        fs::write(object.join("keep"), b"unknown must survive").unwrap();
        if cancelling {
            assert!(store.cancel(&m).is_err());
        } else {
            fs::remove_file(root.join(m.id().unwrap().to_string())).unwrap();
        }
        if cancelling {
            // Quota reads do not delete controls or interpret body bytes.
            // Explicit cancel must still refuse the unknown entry.
            assert_eq!(store.reserved_bytes().unwrap(), 4);
            assert!(store.cancel(&m).is_err());
        } else {
            assert!(store.reserved_bytes().is_err());
        }
        assert_eq!(
            fs::read(object.join("keep")).unwrap(),
            b"unknown must survive"
        );
        assert_eq!(read_manifest(&object).unwrap(), m);
        assert!(object.join("body").exists());
    }
}

#[test]
fn an_unknown_hardlink_is_not_cancellation_authority() {
    let scratch = Scratch::new();
    let root = scratch.0.join("stages");
    let store = TransferStore::open(&root).unwrap();
    let m = manifest(b"foreign hardlink", b"body");
    publish(&root, &m, m.id().unwrap()).unwrap();
    let external = scratch.0.join("copied manifest");
    fs::write(&external, m.encode()).unwrap();
    fs::hard_link(&external, marker(&root, &m.id().unwrap().to_string())).unwrap();
    assert!(store.reserved_bytes().is_err());
    assert_eq!(fs::read(&external).unwrap(), m.encode());
    assert!(root.join(m.id().unwrap().to_string()).exists());
}

#[test]
fn container_object_and_control_links_cannot_escape_the_private_store() {
    for boundary in ["container", "object", "body"] {
        let scratch = Scratch::new();
        let root = scratch.0.join("stages");
        let store = TransferStore::open(&root).unwrap();
        let m = manifest(b"link boundary", b"body");
        publish(&root, &m, m.id().unwrap()).unwrap();
        let object = root.join(target(&m.id().unwrap().to_string()));
        let outside = scratch.0.join("outside");
        let moved = match boundary {
            "container" => root.join(DIRECTORY),
            "object" => object.clone(),
            _ => object.join("body"),
        };
        fs::rename(&moved, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, &moved).unwrap();
        let sentinel = match boundary {
            "container" => outside.join(m.id().unwrap().to_string()).join("body"),
            "object" => outside.join("body"),
            _ => outside.clone(),
        };
        fs::write(&sentinel, b"outside bytes must remain").unwrap();
        assert!(store.receive(m.clone()).is_err(), "{boundary}");
        assert!(store.cancel(&m).is_err(), "{boundary}");
        assert_eq!(fs::read(&sentinel).unwrap(), b"outside bytes must remain");
        assert_eq!(fs::read_link(&moved).unwrap(), outside);
    }
}

#[test]
#[ignore = "owned subprocess, invoked by crash recovery parent only"]
fn portable_crash_worker() {
    let root =
        PathBuf::from(std::env::var_os("KHOR_PORTABLE_TEST_ROOT").expect("owned fixture root"));
    let store = TransferStore::open(&root).unwrap();
    let m = manifest(b"portable crash recovery", b"crash bytes");
    publish(&root, &m, m.id().unwrap()).unwrap();
    if std::env::var_os("KHOR_PORTABLE_CANCEL").is_some() {
        let mut sink = store.receive(m.clone()).unwrap();
        sink.append(0, b"crash bytes").unwrap();
        drop(sink);
        store.cancel(&m).unwrap();
    }
}

#[test]
fn crashes_before_and_after_publication_and_each_cancel_boundary_recover() {
    let phases = [
        "after_mkdir",
        "manifest_partial",
        "manifest_synced",
        "body_created",
        "offsets_partial",
        "offsets_synced",
        "portable_before_publish",
        "portable_after_publish",
        "portable_cancel_marked",
        "portable_cancel_unlinked",
        "portable_cancel_control_removed",
        "portable_cancel_object_removed",
        "portable_cancel_marker_removed",
    ];
    for phase in phases {
        let scratch = Scratch::new();
        let root = scratch.0.join("stages");
        let store = TransferStore::open(&root).unwrap();
        let ready = scratch.0.join("ready");
        let log = File::create(scratch.0.join("worker.log")).unwrap();
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args([
            "--exact",
            "transfer::portable::tests::portable_crash_worker",
            "--ignored",
            "--nocapture",
        ])
        .env("KHOR_PORTABLE_TEST_ROOT", &root)
        .env("KHOR_TRANSFER_INIT_PAUSE_ROOT", root.join(DIRECTORY))
        .env("KHOR_TRANSFER_INIT_PHASE", phase)
        .env("KHOR_TRANSFER_INIT_READY", &ready)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log));
        if phase.starts_with("portable_cancel_") {
            cmd.env("KHOR_PORTABLE_CANCEL", "1");
        }
        let mut child = Worker(cmd.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(15);
        while !ready.exists() {
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "{phase}: {}",
                fs::read_to_string(scratch.0.join("worker.log")).unwrap()
            );
            assert!(Instant::now() < deadline, "{phase}: no ready marker");
            std::thread::sleep(Duration::from_millis(5));
        }
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        let m = manifest(b"portable crash recovery", b"crash bytes");
        if phase.starts_with("portable_cancel_") {
            assert_eq!(store.reserved_bytes().unwrap(), 0, "{phase}");
            assert_eq!(store.cancel(&m).unwrap(), Cancelled::Absent);
            assert!(!root.join(DIRECTORY).exists());
        } else {
            let expected = if phase == "portable_after_publish" {
                11
            } else {
                0
            };
            assert_eq!(store.reserved_bytes().unwrap(), expected, "{phase}");
            let mut sink = store.receive(m.clone()).unwrap();
            sink.append(0, b"crash bytes").unwrap();
            drop(sink);
            assert_eq!(store.cancel(&m).unwrap(), Cancelled::Removed);
        }
    }
}
