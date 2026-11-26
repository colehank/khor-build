//! Two isolated nodes, resident IPC and real QUIC. The caller never launches
//! the provider and cannot interpret an unknown owner as an idle execution.
mod util;
use khor_node::{
    Node,
    session_import::{ImportIntent, SessionImportAction, SessionImportReply},
};
use std::{path::Path, time::Duration};
use tokio::time::timeout;
struct Resident(tokio::task::JoinHandle<Result<(), String>>);
impl Drop for Resident {
    fn drop(&mut self) {
        self.0.abort();
    }
}
async fn resident(root: &Path, name: &str) -> Resident {
    let node = Node::open_as(root.to_path_buf(), name).unwrap();
    let guard = Resident(tokio::spawn(async move { node.serve().await }));
    timeout(Duration::from_secs(90), async {
        while !root.join(".khor/endpoint.json").exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    guard
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn source_owned_catalog_crosses_resident_ipc_and_quic_without_resuming_originals() {
    let fixture =
        std::env::var("KHOR_TEST_IMPORT_STUB").expect("private fixture executable is required");
    let source_root = tempfile::tempdir().unwrap();
    let caller_root = tempfile::tempdir().unwrap();
    util::hermetic(source_root.path());
    util::hermetic(caller_root.path());
    let _source = resident(source_root.path(), "source").await;
    let source = Node::open_as(source_root.path().to_path_buf(), "source").unwrap();
    let caller = Node::open_as(caller_root.path().to_path_buf(), "caller").unwrap();
    timeout(
        Duration::from_secs(60),
        caller.pair(&source.invite().unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    let _caller = resident(caller_root.path(), "caller").await;
    source
        .register_agent(
            "private-provider",
            &[
                fixture,
                "full".into(),
                source_root.path().display().to_string(),
            ],
        )
        .unwrap();
    assert!(
        caller.agent("private-provider").unwrap().is_none(),
        "registration exists only at the source before its first query"
    );
    let home = source.device_str();
    let SessionImportReply::Catalog { value } = caller
        .session_import_on(home, SessionImportAction::Read)
        .await
        .unwrap()
    else {
        panic!("catalog")
    };
    assert_eq!(value.home, home);
    assert!(value.rows.is_empty());
    assert!(
        value
            .agents
            .iter()
            .any(|row| row.agent == "private-provider" && row.status == "unasked")
    );
    assert!(
        !source_root.path().join("methods.jsonl").exists(),
        "reading never starts an agent"
    );
    let SessionImportReply::Catalog { value } = caller
        .session_import_on(
            home,
            SessionImportAction::Refresh {
                agent: "private-provider".into(),
            },
        )
        .await
        .unwrap()
    else {
        panic!("catalog")
    };
    let rows: Vec<_> = value
        .rows
        .iter()
        .filter(|row| row.agent == "private-provider")
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].title, rows[1].title);
    assert_ne!(rows[0].native_id, rows[1].native_id);
    assert!(
        rows.iter()
            .all(|row| row.intent == Some(ImportIntent::Fork)),
        "unknown ownership allows an independent fork, never import"
    );
    assert!(value.rows.iter().all(|row| row.native_id != "archived"));
    let rejected = caller
        .session_import_on(
            home,
            SessionImportAction::Apply {
                agent: "private-provider".into(),
                native_id: "original-a".into(),
                intent: ImportIntent::Import,
                token: "1234567890abcdef1234567890abcdef".into(),
            },
        )
        .await;
    assert!(
        rejected.is_err(),
        "forged original-import intent is rechecked at the source"
    );
    let log = std::fs::read_to_string(source_root.path().join("methods.jsonl")).unwrap();
    for line in log.lines() {
        let value: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(matches!(
            value["method"].as_str(),
            Some("initialize" | "session/list")
        ));
    }
    assert!(
        !caller_root.path().join("methods.jsonl").exists(),
        "no provider process runs on the viewer"
    );
    assert!(
        !source_root.path().join(".khor/observations").exists(),
        "a refused import publishes no observation, regardless of ambient terminal discovery"
    );
}
