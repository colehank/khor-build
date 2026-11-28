use super::*;

fn startup(root: &std::path::Path, token: &str) -> crate::proto::LegacyShellStart {
    crate::proto::LegacyShellStart {
        token: token.into(),
        title: "original shell".into(),
        cwd: root.display().to_string(),
        command: "legacy startup must not run again".into(),
        cols: 80,
        rows: 24,
    }
}

#[tokio::test]
async fn a_legacy_creation_receipt_returns_its_original_shell_without_admitting_startup() {
    let root = crate::testdir::TempTree::fresh("legacy-shell-receipt");
    let node = Node::open_as(root.path(), "legacy-shell-test").unwrap();
    let id = crate::SessionId("shell/original-legacy-terminal".into());
    node.live
        .register(&id, crate::kind::SHELL, "original shell", None, None)
        .unwrap();
    let token = "legacy-shell-create-token";
    crate::once::note(&root.path(), token, &id.0).unwrap();
    let directory = node.live.dir_of(&id).unwrap();
    let before = fs::read(directory.join("meta.json")).unwrap();
    // The original request's directory may since have disappeared. Its
    // already-admitted identity still answers a retry without a second spawn.
    let mut request = startup(&root.path(), token);
    request.cwd = root
        .path()
        .join("removed-since-first-open")
        .display()
        .to_string();
    let current: crate::shell_start::ShellStart = request.clone().into();
    assert_eq!(node.open_shell_here(current.clone()).await.unwrap(), id);
    let claim_path = root
        .path()
        .join(".khor/shell-opened")
        .join(format!("{token}.json"));
    let claim: serde_json::Value = serde_json::from_slice(&fs::read(&claim_path).unwrap()).unwrap();
    assert_eq!(claim["session"], id.0);
    assert_eq!(claim["fingerprint"].as_str().unwrap().len(), 64);
    fs::remove_file(root.path().join(".khor/opened").join(token)).unwrap();
    assert_eq!(
        node.open_shell_here(current).await.unwrap(),
        id,
        "the adopted identity outlives legacy ledger cleanup"
    );
    assert_eq!(node.open_legacy_shell_here(request).await.unwrap(), id);
    assert_eq!(
        crate::once::made(&root.path(), token).as_deref(),
        Some(id.0.as_str()),
        "legacy callers retain their rollback ledger before the reply"
    );
    assert_eq!(fs::read(directory.join("meta.json")).unwrap(), before);
    assert!(!directory.join("host.json").exists());
    let after: serde_json::Value = serde_json::from_slice(&fs::read(claim_path).unwrap()).unwrap();
    assert_eq!(after, claim);
}

#[tokio::test]
async fn advanced_startup_on_an_old_token_is_refused_even_after_the_legacy_ledger_is_pruned() {
    let root = crate::testdir::TempTree::fresh("legacy-shell-advanced-receipt");
    let node = Node::open_as(root.path(), "legacy-shell-test").unwrap();
    let id = crate::SessionId("shell/legacy-advanced-terminal".into());
    node.live
        .register(&id, crate::kind::SHELL, "original shell", None, None)
        .unwrap();
    let token = "legacy-shell-advanced-token";
    crate::once::note(&root.path(), token, &id.0).unwrap();
    let mut request: crate::shell_start::ShellStart = startup(&root.path(), token).into();
    request.headless = true;
    let why = node.open_shell_here(request.clone()).await.unwrap_err();
    assert!(
        why.contains("new startup options require a new token"),
        "{why}"
    );
    let claim_path = root
        .path()
        .join(".khor/shell-opened")
        .join(format!("{token}.json"));
    let before = fs::read(&claim_path).unwrap();
    let claim: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(claim["fingerprint"], "legacy-startup-reserved");
    assert_eq!(claim["session"], id.0);
    fs::remove_file(root.path().join(".khor/opened").join(token)).unwrap();
    assert!(node.open_shell_here(request.clone()).await.is_err());
    request.headless = false;
    assert!(
        node.open_shell_here(request).await.is_err(),
        "changing arguments must not retire the reservation"
    );
    assert_eq!(fs::read(&claim_path).unwrap(), before);
    assert!(!node.live.dir_of(&id).unwrap().join("host.json").exists());
}

#[tokio::test]
async fn a_missing_or_malformed_legacy_receipt_row_refuses_instead_of_repeating_startup() {
    let root = crate::testdir::TempTree::fresh("legacy-shell-invalid-receipt");
    let node = Node::open_as(root.path(), "legacy-shell-test").unwrap();
    for (token, id) in [
        ("legacy-shell-missing-token", "shell/missing"),
        ("legacy-shell-invalid-token", "not-a-shell"),
        ("legacy-shell-empty-token", ""),
    ] {
        crate::once::note(&root.path(), token, id).unwrap();
        let why = node
            .open_legacy_shell_here(startup(&root.path(), token))
            .await
            .unwrap_err();
        assert!(why.contains("refusing to repeat startup"), "{why}");
    }
    assert!(!root.path().join(".khor/shell-opened").exists());
}

#[tokio::test]
async fn a_legacy_receipt_requires_its_exact_registered_shell_identity() {
    let root = crate::testdir::TempTree::fresh("legacy-shell-mismatched-receipt");
    let node = Node::open_as(root.path(), "legacy-shell-test").unwrap();
    let id = crate::SessionId("shell/legacy-mismatch".into());
    node.live
        .register(&id, crate::kind::SHELL, "original shell", None, None)
        .unwrap();
    let directory = node.live.dir_of(&id).unwrap();
    let path = directory.join("meta.json");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let token = "legacy-shell-mismatch-token";
    crate::once::note(&root.path(), token, &id.0).unwrap();
    for (field, value) in [("kind", "tui"), ("id", "shell/some-other-terminal")] {
        let mut meta = original.clone();
        meta[field] = serde_json::Value::String(value.into());
        fs::write(&path, serde_json::to_vec(&meta).unwrap()).unwrap();
        let why = node
            .open_legacy_shell_here(startup(&root.path(), token))
            .await
            .unwrap_err();
        assert!(why.contains("refusing to repeat startup"), "{why}");
    }
    assert!(!root.path().join(".khor/shell-opened").exists());
}
