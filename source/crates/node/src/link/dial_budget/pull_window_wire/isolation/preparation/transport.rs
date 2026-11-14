//! An optional connection's transport failure must not discard a healthy read.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn transport_negotiation_failure_preserves_a_complete_cached_main_pull() {
    let f = Fixture::with_identity(Held::Head, true).await;
    f.server
        .set_alpns(vec![b"controlled-incompatible-alpn".to_vec()]);
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&peer(&f).id, "peer", &peer(&f).addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    let landing = crate::files::landing(PATH, f.root.path()).unwrap();
    let mut pull = Box::pin(
        f.node
            .pull_into_with_layout(&f.ep, "peer", PATH, landing, None, None, false, true),
    );
    let mut incoming_count = 0;
    let mut negotiation_errors = Vec::new();
    let mut unexpected_success = false;
    let mut fallback_tail = false;
    let mut held_at_fallback = false;
    let mut premature = None;
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            {
                let seen = f.seen.lock().unwrap();
                if seen.issued.contains(&2) {
                    fallback_tail = true;
                    held_at_fallback = !seen.delivered.contains(&1);
                    break;
                }
            }
            tokio::select! {
                outcome = &mut pull => {
                    premature = Some(format!("{outcome:?}"));
                    break;
                }
                incoming = f.server.accept() => {
                    let Some(incoming) = incoming else { break };
                    incoming_count += 1;
                    match incoming.accept() {
                        Ok(connecting) => match connecting.await {
                            Err(error) => negotiation_errors.push(format!("{error:?}")),
                            Ok(conn) => {
                                unexpected_success = true;
                                conn.close(0u32.into(), b"unexpected compatible handshake");
                            }
                        },
                        Err(error) => negotiation_errors.push(format!("{error:?}")),
                    }
                    if incoming_count >= 8 { break; }
                }
                _ = f.changed.notified() => {}
            }
        }
    })
    .await
    .is_ok();
    let mut completed = false;
    let mut exact_bytes = false;
    let mut cleaned = false;
    if fallback_tail {
        // Only the missing prefix was held. Later slices must have been issued
        // through the real cached Main after optional preparation failed.
        f.release.notify_one();
        if let Ok(Ok((moved, dest))) =
            tokio::time::timeout(Duration::from_secs(20), &mut pull).await
        {
            completed = moved == f.bytes.len() as u64;
            exact_bytes = std::fs::read(dest).is_ok_and(|bytes| bytes == *f.bytes);
            cleaned = !f.part().exists() && !PullNote::path_of(&f.part()).exists();
        }
    }
    drop(pull);
    let unique_offsets = f.seen.lock().unwrap().issued == (0..13).collect();
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-negotiation-fallback] {}",
        serde_json::json!({
            "event":"negotiation_failure", "incoming_count":incoming_count,
            "negotiation_errors":negotiation_errors, "unexpected_success":unexpected_success,
            "observation_completed":observed, "fallback_tail":fallback_tail,
            "held_at_fallback":held_at_fallback, "premature_result":premature,
            "completed":completed, "exact_bytes":exact_bytes,
            "unique_offsets":unique_offsets, "partial_note_cleanup":cleaned,
            "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(
        incoming_count > 0
            && !negotiation_errors.is_empty()
            && !unexpected_success
            && observed
            && fallback_tail
            && held_at_fallback
            && completed
            && exact_bytes
            && unique_offsets
            && cleaned,
        "optional negotiation failure must preserve the complete cached Main file read"
    );
}

async fn terminal_close(remote_close: bool) {
    let f = Fixture::with_identity(Held::All, true).await;
    let (line, remote) = extra(&f).await;
    if remote_close {
        remote.close(42u32.into(), b"controlled application refusal");
    } else {
        line.0.conn.close(42u32.into(), b"controlled local close");
    }
    let end = tokio::time::timeout(Duration::from_secs(5), line.0.conn.closed())
        .await
        .unwrap();
    let actual_kind = if remote_close {
        matches!(&end, iroh::endpoint::ConnectionError::ApplicationClosed(_))
    } else {
        matches!(&end, iroh::endpoint::ConnectionError::LocallyClosed)
    };
    let terminal = matches!(
        DialFailure::of_end(Some(&end), &end),
        DialFailure::Refused(_)
    );
    drop(line);
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-negotiation-fallback] {}",
        serde_json::json!({
            "event":"terminal_close", "remote_close":remote_close,
            "actual_kind":actual_kind, "terminal":terminal,
            "error":format!("{end:?}"), "cached_main_control":true,
            "fixture_closed":true,
        })
    );
    assert!(
        actual_kind && terminal,
        "application or local close must stay terminal"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn transport_application_close_remains_terminal() {
    terminal_close(true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn transport_local_close_remains_terminal() {
    terminal_close(false).await;
}
