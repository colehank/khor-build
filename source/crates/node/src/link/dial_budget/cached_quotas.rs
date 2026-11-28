use super::*;

#[tokio::test]
async fn a_residents_cached_quotas_do_not_need_a_second_root_mutation_lease() {
    let (_root, node, _, ep) = fixture().await;
    let node = std::sync::Arc::new(node);
    node.me.set(std::sync::Arc::downgrade(&node)).unwrap();
    let before = node.device_str().to_owned();
    let held = node.root_mutation_guard().unwrap();
    let answer = tokio::time::timeout(
        Duration::from_secs(5),
        node.dispatch(node.device_str(), Request::Quotas, &ep, None),
    )
    .await;
    // Release the actual fence even on the original rejected response, so
    // background work and endpoint cleanup cannot retain this test's home.
    drop(held);
    ep.close().await;
    assert!(
        matches!(&answer, Ok(Response::Quotas { readings }) if readings.is_empty()),
        "a cached read on an already admitted resident must answer while the real mutation fence is held: {answer:?}"
    );
    assert_eq!(node.device_str(), before);
}
