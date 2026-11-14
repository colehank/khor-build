//! Preparation is optional: a healthy cached Main can deliver first content.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cached_main_delivers_first_content_without_accepting_an_extra_handshake() {
    let f = Fixture::with_identity(Held::All, true).await;
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&peer(&f).id, "peer", &peer(&f).addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    // No acceptor is started for another connection. The already authenticated
    // cached connection remains served by the fixture, including its first slice.
    let first_content = {
        let landing = crate::files::landing(PATH, f.root.path()).unwrap();
        let pull = f
            .node
            .pull_into_with_layout(&f.ep, "peer", PATH, landing, None, None, false, true);
        tokio::pin!(pull);
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Ok(bytes) = std::fs::read(f.part())
                    && bytes.len() >= proto::SLICE as usize
                {
                    return bytes[..proto::SLICE as usize] == f.bytes[..proto::SLICE as usize];
                }
                tokio::select! {
                    outcome = &mut pull => panic!("held pull returned before first content: {outcome:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(5)) => {}
                }
            }
        })
        .await
        .unwrap_or(false)
        // The actual pull, its pending requests and optional dial are dropped
        // here, before checking the original connection or closing the fixture.
    };
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-preparation] {}",
        serde_json::json!({"first_content":first_content,"cached_main_control":true,"fixture_closed":true})
    );
    assert!(
        first_content,
        "healthy cached Main must deliver exact first content before an optional extra handshake"
    );
}
