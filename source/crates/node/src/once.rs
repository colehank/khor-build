//! **A session made at most once per token** (round NET-7, from ux's
//! first-run walk): the phone asked the Mac for a session, the Mac quit
//! before the answer came back, the phone said 「studio 没有响应」 — and
//! the session had been started. Its row arrived when the Mac was back,
//! by which time the person may have pressed 创建 again and had two.
//!
//! So a create carries a token the asker chose before asking
//! (`proto::Request::OpenOnce`, `OpenAgentOnce`), and the machine that
//! makes the session keeps which session each token made, **on disk**,
//! beside its registry: a retry after a lost answer — the same press,
//! or the asker's own second ask — is answered with the session the
//! first one made, across a restart of either end. Stripe's
//! `Idempotency-Key` (<https://docs.stripe.com/api/idempotent_requests>)
//! is the same arrangement for the same failure ("the
//! server … saves the resulting status code and body of the first
//! request … subsequent requests with the same key return the same
//! result").
//!
//! One token at a time per process (`lock`): two retries in flight wait
//! for each other, and the second finds what the first made.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

/// How long a token's record is kept. A retry is seconds to minutes
/// after the first ask; a day covers a machine that was off overnight.
const KEPT_MS: u64 = 24 * 60 * 60 * 1000;

/// A token is what a face makes (`crypto.randomUUID()`): letters, digits
/// and dashes, long enough not to meet another by chance, short enough
/// to be a file name everywhere.
pub(crate) fn valid(token: &str) -> bool {
    (16..=64).contains(&token.len())
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn dir(root: &Path) -> PathBuf {
    root.join(".khor").join("opened")
}

/// The session this token made, if it made one.
pub(crate) fn made(root: &Path, token: &str) -> Option<String> {
    let id = std::fs::read_to_string(dir(root).join(token)).ok()?;
    let id = id.trim();
    (!id.is_empty()).then(|| id.to_owned())
}

/// Records what this token made, and forgets records older than
/// [`KEPT_MS`] while it is there.
pub(crate) fn note(root: &Path, token: &str, id: &str) -> Result<(), String> {
    let d = dir(root);
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now();
    if let Ok(rd) = std::fs::read_dir(&d) {
        for e in rd.flatten() {
            let old = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| now.duration_since(t).ok())
                .is_some_and(|age| age.as_millis() as u64 > KEPT_MS);
            if old {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    crate::live::write_whole(&d.join(token), id.as_bytes())
}

type Locks = Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>;
static LOCKS: std::sync::OnceLock<Locks> = std::sync::OnceLock::new();

/// Holds `token` for this process until the guard drops.
pub(crate) async fn lock(token: &str) -> tokio::sync::OwnedMutexGuard<()> {
    let one = {
        let mut locks = LOCKS
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Both held guards and queued lock_owned futures keep their mutex alive.
        locks.retain(|_, lock| lock.strong_count() != 0);
        match locks.get(token).and_then(Weak::upgrade) {
            Some(one) => one,
            None => {
                let one = Arc::new(tokio::sync::Mutex::new(()));
                locks.insert(token.to_owned(), Arc::downgrade(&one));
                one
            }
        }
    };
    one.lock_owned().await
}

#[cfg(test)]
#[path = "once_lock_tests.rs"]
mod lock_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_a_safe_file_name_and_only_that() {
        assert!(valid("3f2b9c1e-8a7d-4e5f-9b0c-1d2e3f4a5b6c"));
        for bad in [
            "short",
            "../../etc/passwd-xxxxxxxx",
            "a b c d e f g h i j",
            &"x".repeat(65),
        ] {
            assert!(!valid(bad), "{bad:?}");
        }
    }

    #[test]
    fn what_a_token_made_is_found_again() {
        let root = std::env::temp_dir().join(format!("khor-once-{}", std::process::id()));
        let token = "3f2b9c1e-8a7d-4e5f-9b0c-1d2e3f4a5b6c";
        assert_eq!(made(&root, token), None);
        note(&root, token, "shell/abc").unwrap();
        assert_eq!(made(&root, token).as_deref(), Some("shell/abc"));
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// Bind a terminal create token to its arguments and identity across processes.
/// The file lock covers registration and spawn; retries never mint a second ID.
pub(crate) fn claim_shell(
    root: &Path,
    token: &str,
    fingerprint: &str,
) -> Result<(std::fs::File, String), String> {
    claim_shell_with_id(root, token, fingerprint, None)
}

/// Adopt a verified older identity under the same durable token gate. Existing
/// bindings are immutable, including a reservation that refuses replay.
pub(crate) fn claim_shell_with_id(
    root: &Path,
    token: &str,
    fingerprint: &str,
    legacy_id: Option<&str>,
) -> Result<(std::fs::File, String), String> {
    if !valid(token) {
        return Err("Invalid shell creation token".into());
    }
    let directory = root.join(".khor").join("shell-opened");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let held = crate::remote_cancel::gate(&directory.join(format!("{token}.lock")))?;
    let file = directory.join(format!("{token}.json"));
    let id = if file.exists() {
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if value["fingerprint"].as_str() != Some(fingerprint) {
            return Err("Shell creation token already names different arguments".into());
        }
        let id = value["session"]
            .as_str()
            .ok_or("Invalid shell creation receipt")?
            .to_owned();
        if legacy_id.is_some_and(|legacy| legacy != id.as_str()) {
            return Err("Shell creation token already names another session".into());
        }
        id
    } else {
        let id = match legacy_id {
            Some(id)
                if id.starts_with("shell/") && id.len() > "shell/".len() && id.len() <= 512 =>
            {
                id.to_owned()
            }
            Some(_) => return Err("Invalid legacy shell identity".into()),
            None => format!("shell/{}", crate::link::fresh_leaf()?),
        };
        let bytes =
            serde_json::to_vec(&serde_json::json!({"fingerprint":fingerprint,"session":id}))
                .map_err(|e| e.to_string())?;
        crate::live::write_whole(&file, &bytes)?;
        id
    };
    Ok((held, id))
}

#[cfg(test)]
mod shell_claim_tests {
    #[test]
    fn an_adopted_legacy_identity_cannot_be_rebound_or_overwritten() {
        let root = crate::testdir::TempTree::fresh("shell-claim-adopted");
        let token = "shell-legacy-adopt-token";
        let original = "shell/already-started";
        let (held, id) =
            super::claim_shell_with_id(&root, token, "original", Some(original)).unwrap();
        assert_eq!(id, original);
        drop(held);
        let (held, retry) = super::claim_shell(&root, token, "original").unwrap();
        assert_eq!(retry, original);
        drop(held);
        assert!(
            super::claim_shell_with_id(&root, token, "original", Some("shell/another")).is_err()
        );
        assert!(super::claim_shell_with_id(&root, token, "changed", Some(original)).is_err());
        let (held, retry) = super::claim_shell(&root, token, "original").unwrap();
        assert_eq!(retry, original);
        drop(held);
    }

    #[test]
    fn shell_identity_is_argument_bound_and_survives_another_claimant() {
        let root = crate::testdir::TempTree::fresh("shell-claim");
        let (held, id) = super::claim_shell(&root, "shell-create-token-12345", "original").unwrap();
        assert!(super::claim_shell(&root, "shell-create-token-12345", "original").is_err());
        drop(held);
        let (held, retry) =
            super::claim_shell(&root, "shell-create-token-12345", "original").unwrap();
        assert_eq!(id, retry);
        drop(held);
        assert!(super::claim_shell(&root, "shell-create-token-12345", "different").is_err());
        drop(root);
    }
}
