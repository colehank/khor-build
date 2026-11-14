//! A refused Incoming is khor's existing paused/background signal.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_actual_incoming_refusal_remains_terminal() {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    let mut attempt = Box::pin(f.node.transfer_line_for(&f.ep, &target));
    let incoming = tokio::select! {
        result = &mut attempt => panic!("preparation finished before refusal; success={}", result.is_ok()),
        incoming = tokio::time::timeout(Duration::from_secs(5), f.server.accept()) => incoming.unwrap().unwrap(),
    };
    incoming.refuse();
    let terminal = matches!(
        tokio::time::timeout(Duration::from_secs(5), &mut attempt).await,
        Ok(Err(PreparationFailure::Stop(_)))
    );
    drop(attempt);
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-negotiation-fallback] {}",
        serde_json::json!({
            "event":"resting_refusal", "actual_incoming_refused":true,
            "terminal":terminal, "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(
        terminal,
        "paused/background refusal must not become optional fallback"
    );
}
