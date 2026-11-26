use khor_acp::{session_catalog, CatalogCapabilities};
use serde_json::json;

#[test]
fn optional_archive_and_fork_facts_survive_the_pinned_sdk() {
    let facts=CatalogCapabilities::from_initialize(&json!({"protocolVersion":1,"agentCapabilities":{"loadSession":true,"sessionCapabilities":{"list":{},"resume":{},"fork":{},"archive":{}}}})).unwrap();
    assert!(facts.list && facts.load && facts.resume && facts.fork && facts.archive);
    let absent =
        CatalogCapabilities::from_initialize(&json!({"protocolVersion":1,"agentCapabilities":{}}))
            .unwrap();
    assert!(!absent.list && !absent.fork && !absent.archive);
    assert!(CatalogCapabilities::from_initialize(&json!({"protocolVersion":2})).is_err());
}

#[tokio::test]
async fn generic_discovery_never_creates_or_loads_a_session() {
    let log = std::env::temp_dir().join(format!("khor-catalog-{}", std::process::id()));
    let _ = std::fs::remove_file(&log);
    let command=json!({"command":env!("CARGO_BIN_EXE_acp-stub"),"env":{"KHOR_STUB_LISTS":"1","KHOR_STUB_SETUP_LOG":log}}).to_string();
    let catalog = session_catalog(&command, None).await.unwrap();
    assert!(catalog.capabilities.list);
    assert_eq!(catalog.sessions.len(), 2);
    assert!(catalog.sessions.iter().all(|row| row.archived.is_none()));
    let methods = std::fs::read_to_string(&log).unwrap();
    assert!(methods.contains("initialize"));
    assert!(
        !methods.contains("session/new")
            && !methods.contains("session/load")
            && !methods.contains("session/resume")
    );
    let _ = std::fs::remove_file(log);
}

fn fixture(mode: &str) -> (std::path::PathBuf, String) {
    let root = std::env::temp_dir().join(format!("khor-import-{}-{}", std::process::id(), mode));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let command =
        json!({"command":env!("CARGO_BIN_EXE_acp-import-fixture"),"args":[mode,root]}).to_string();
    (root, command)
}

#[tokio::test]
async fn archive_filter_pagination_and_exact_id_dedup_cross_the_actual_wire() {
    let (root, command) = fixture("full");
    let catalog = session_catalog(&command, None).await.unwrap();
    assert_eq!(
        catalog.sessions.len(),
        2,
        "Same title is two IDs, repeated ID is one row"
    );
    assert!(catalog
        .sessions
        .iter()
        .all(|row| row.archived == Some(false)));
    let requests = std::fs::read_to_string(root.join("methods.jsonl")).unwrap();
    let requests: Vec<serde_json::Value> = requests
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let lists: Vec<_> = requests
        .iter()
        .filter(|request| request["method"] == "session/list")
        .collect();
    assert_eq!(lists.len(), 2);
    assert!(lists
        .iter()
        .all(|request| request["params"]["archived"] == "exclude"));
    assert!(!requests
        .iter()
        .any(|request| request["method"] == "session/new"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn fork_creates_an_independent_native_identity_and_never_resumes_the_parent() {
    let (root, command) = fixture("fork-wire");
    let (handle, _events) = khor_acp::start_configured(
        &command,
        root.clone(),
        None,
        khor_acp::Environment {
            fork_from: Some("parent".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(handle.session().to_string(), "independent-branch");
    handle.prompt("A new branch turn").await.unwrap();
    handle.shutdown().await.unwrap();
    let requests = std::fs::read_to_string(root.join("methods.jsonl")).unwrap();
    let requests: Vec<serde_json::Value> = requests
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        requests
            .iter()
            .filter(|request| request["method"] == "session/fork")
            .count(),
        1
    );
    assert!(!requests.iter().any(|request| matches!(
        request["method"].as_str(),
        Some("session/new" | "session/load" | "session/resume")
    )));
    assert!(requests
        .iter()
        .filter(|request| request["method"] == "session/prompt")
        .all(|request| request["params"]["sessionId"] == "independent-branch"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn unsupported_or_identity_reusing_forks_are_refused() {
    for mode in ["no-fork", "bad-fork"] {
        let (root, command) = fixture(mode);
        assert!(khor_acp::start_configured(
            &command,
            root.clone(),
            None,
            khor_acp::Environment {
                fork_from: Some("parent".into()),
                ..Default::default()
            }
        )
        .await
        .is_err());
        let requests = std::fs::read_to_string(root.join("methods.jsonl")).unwrap();
        if mode == "no-fork" {
            assert!(!requests.contains("session/fork"));
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
