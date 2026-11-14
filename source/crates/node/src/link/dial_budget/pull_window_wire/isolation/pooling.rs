//! Actual QUIC connections, shared by active file owners only.
use super::*;

mod concurrent_files;
mod pressure;
mod resources;

async fn pooled_extra(
    f: &Fixture,
    ep: &iroh::Endpoint,
    target: &DeviceInfo,
    server: &iroh::Endpoint,
) -> (Arc<pull_pool::Lease>, iroh::endpoint::Connection) {
    let owner = f.node.pull_peer(ep, target).unwrap();
    let (lease, remote) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(f.node.pooled_transfer(ep, target, &owner), async {
            server.accept().await.unwrap().accept().unwrap().await
        })
    })
    .await
    .unwrap();
    (lease.unwrap(), remote.unwrap())
}

async fn endpoint(key: Option<iroh::SecretKey>) -> Arc<iroh::Endpoint> {
    let mut builder = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .relay_mode(iroh::RelayMode::Disabled)
        .alpns(vec![ALPN.to_vec()])
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap();
    if let Some(key) = key {
        builder = builder.secret_key(key);
    }
    Arc::new(builder.bind().await.unwrap())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_preparations_share_one_real_extra_until_the_last_owner() {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    let owner = f.node.pull_peer(&f.ep, &target).unwrap();
    let (a, b, remote) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(
            f.node.pooled_transfer(&f.ep, &target, &owner),
            f.node.pooled_transfer(&f.ep, &target, &owner),
            async { f.server.accept().await.unwrap().accept().unwrap().await },
        )
    })
    .await
    .unwrap();
    let (a, b, remote) = (a.unwrap(), b.unwrap(), remote.unwrap());
    assert!(Arc::ptr_eq(&a, &b));
    assert_ne!(a.line.0.conn.stable_id(), f.local.stable_id());
    assert_eq!(f.node.wires.pulls.available_extra(), 3);
    assert!(
        tokio::time::timeout(Duration::from_millis(500), f.server.accept())
            .await
            .is_err()
    );
    drop(a);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), remote.closed())
            .await
            .is_err()
    );
    cached_answer(&f).await;
    drop(b);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    assert_eq!(f.node.wires.pulls.available_extra(), 4);
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-pool] {{\"event\":\"shared_real_extra\",\"owners\":2,\"handshakes\":1,\"cached_main_control\":true,\"fixture_closed\":true}}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn retiring_an_extra_releases_capacity_while_an_old_owner_is_retained() {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    let (old, remote) = pooled_extra(&f, &f.ep, &target, &f.server).await;
    assert_eq!(f.node.wires.pulls.available_extra(), 3);
    f.node.wires.let_go_served(wires::Retired::Suspended);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    assert_eq!(f.node.wires.pulls.available_extra(), 4);
    cached_answer(&f).await;
    let (new, remote) = pooled_extra(&f, &f.ep, &target, &f.server).await;
    drop(old);
    assert!(new.line.0.conn.close_reason().is_none());
    assert!(
        tokio::time::timeout(Duration::from_millis(100), remote.closed())
            .await
            .is_err()
    );
    drop(new);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-pool] {{\"event\":\"retained_retired_owner\",\"capacity_released\":true,\"replacement_survived\":true,\"cached_main_control\":true,\"fixture_closed\":true}}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn same_key_endpoint_replacement_never_reuses_the_old_extra() {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    let old_owner = f.node.pull_peer(&f.ep, &target).unwrap();
    let (old, remote) = pooled_extra(&f, &f.ep, &target, &f.server).await;
    let next = endpoint(Some(f.ep.secret_key().clone())).await;
    assert_eq!(f.ep.id(), next.id());
    let (wake, _wakes) = tokio::sync::mpsc::unbounded_channel();
    f.node.set_ears(&next, &wake);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    assert!(f.node.pull_peer(&f.ep, &target).is_none());
    assert!(matches!(
        f.node.pooled_transfer(&f.ep, &target, &old_owner).await,
        Err(PreparationFailure::Stop(_))
    ));
    let (new, remote) = pooled_extra(&f, &next, &target, &f.server).await;
    assert_ne!(old.line.0.conn.stable_id(), new.line.0.conn.stable_id());
    drop(old);
    assert!(new.line.0.conn.close_reason().is_none());
    assert!(
        tokio::time::timeout(Duration::from_millis(100), remote.closed())
            .await
            .is_err()
    );
    drop(new);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    next.close().await;
    f.close().await;
    eprintln!(
        "[pull-pool] {{\"event\":\"same_key_new_endpoint\",\"old_scope_stopped\":true,\"replacement_survived\":true,\"cached_main_control\":true,\"fixture_closed\":true}}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fifth_peer_uses_cached_fallback_when_four_real_extras_are_owned() {
    let f = Fixture::with_identity(Held::All, true).await;
    let mut servers = Vec::new();
    for _ in 0..4 {
        servers.push(endpoint(None).await);
    }
    let mut targets = vec![peer(&f)];
    for (index, server) in servers.iter().enumerate() {
        let loaded = f.node.devices_loaded().unwrap();
        let addrs = server
            .addr()
            .ip_addrs()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        loaded
            .doc
            .upsert(&server.id().to_string(), &format!("pool-{index}"), &addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
        targets.push(
            f.node
                .devices()
                .unwrap()
                .into_iter()
                .find(|p| p.id == server.id().to_string())
                .unwrap(),
        );
    }
    let mut leases = Vec::new();
    let mut remotes = Vec::new();
    for (index, target) in targets[..4].iter().enumerate() {
        let server = if index == 0 {
            &f.server
        } else {
            servers[index - 1].as_ref()
        };
        let (lease, remote) = pooled_extra(&f, &f.ep, target, server).await;
        leases.push(lease);
        remotes.push(remote);
    }
    assert_eq!(f.node.wires.pulls.available_extra(), 0);
    let fifth = f.node.pull_peer(&f.ep, &targets[4]).unwrap();
    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        f.node.pooled_transfer(&f.ep, &targets[4], &fifth),
    )
    .await
    .unwrap();
    assert!(matches!(outcome, Err(PreparationFailure::RetryCached(_))));
    assert!(
        tokio::time::timeout(Duration::from_millis(500), servers[3].accept())
            .await
            .is_err()
    );
    cached_answer(&f).await;
    drop(leases.remove(0));
    tokio::time::timeout(Duration::from_secs(5), remotes[0].closed())
        .await
        .unwrap();
    let (last, remote) = pooled_extra(&f, &f.ep, &targets[4], &servers[3]).await;
    assert_eq!(f.node.wires.pulls.available_extra(), 0);
    drop(last);
    drop(leases);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    assert_eq!(f.node.wires.pulls.available_extra(), 4);
    cached_answer(&f).await;
    for server in servers {
        server.close().await;
    }
    f.close().await;
    eprintln!(
        "[pull-pool] {{\"event\":\"four_real_extras_fifth_fallback\",\"peers\":5,\"peak_extra\":4,\"fifth_no_handshake\":true,\"later_positive_handshake\":true,\"cached_main_control\":true,\"fixture_closed\":true}}"
    );
}
