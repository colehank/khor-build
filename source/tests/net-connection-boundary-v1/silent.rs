//! Transport-only witness. No application retry, reconnect, or watchdog.
use crate::upstream_util::{self as util, Pair, PathConnectionExt, ping_accept, ping_open};
use iroh::{EndpointAddr, TransportAddr, endpoint::Connection};
use n0_error::{Result, StackResultExt, StdResultExt};
use n0_future::StreamExt;
use patchbay::{Device, Nat};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::Barrier, time::Instant};
const SETUP: Duration = Duration::from_secs(15);
const OBSERVE: Duration = Duration::from_secs(30);
const CALLER: Duration = Duration::from_secs(2);
#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    DirectClient,
    DirectServer,
    RelayClient,
    RelayServer,
}
impl Fault {
    fn here(self, server: bool) -> bool {
        matches!(
            (self, server),
            (Self::DirectServer | Self::RelayServer, true)
                | (Self::DirectClient | Self::RelayClient, false)
        )
    }
    fn relay(self) -> bool {
        matches!(self, Self::RelayClient | Self::RelayServer)
    }
}
fn emit(origin: Instant, event: &str, side: &str, fields: serde_json::Value) {
    eprintln!(
        "[direct-fallback] {}",
        serde_json::json!({"schema":1,"event":event,"side":side,"at_us":origin.elapsed().as_micros(),"fields":fields})
    );
}
fn selected(conn: &Connection) -> String {
    format!(
        "{:?}",
        conn.paths()
            .iter()
            .find(|p| p.is_selected())
            .map(|p| p.remote_addr().clone())
    )
}
fn watch(conn: &Connection, origin: Instant, side: &'static str) -> tokio::task::JoinHandle<()> {
    let mut events = conn.path_events();
    tokio::spawn(async move {
        while let Some(e) = events.next().await {
            emit(
                origin,
                "path",
                side,
                serde_json::json!({"event":format!("{e:?}")}),
            );
        }
    })
}
async fn rules(dev: &Device, fault: Fault, server: bool) -> serde_json::Value {
    let table = if fault.here(server) {
        let (incoming, outgoing) = if fault.relay() {
            (
                "tcp sport { 80, 443 } counter drop",
                "tcp dport { 80, 443 } counter drop",
            )
        } else {
            (
                "meta l4proto udp udp sport != 53 udp dport != 53 counter drop",
                "meta l4proto udp udp sport != 53 udp dport != 53 counter drop",
            )
        };
        format!(
            "table inet khor_boundary {{\n chain input {{\n type filter hook input priority -50; policy accept;\n {incoming}\n }}\n chain output {{\n type filter hook output priority -50; policy accept;\n {outgoing}\n }}\n}}\n"
        )
    } else {
        "table inet khor_boundary {\n chain output {\n type filter hook output priority -50; policy accept;\n counter\n }\n}\n".to_owned()
    };
    use std::io::Write;
    let mut cmd = std::process::Command::new("nft");
    cmd.args(["-f", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = dev.spawn_command_sync(cmd).unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(table.as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "firewall: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::json!({"rules":table,"returncode":result.status.code()})
}
fn counters(dev: &Device) -> serde_json::Value {
    let mut cmd = std::process::Command::new("nft");
    cmd.args(["-j", "list", "table", "inet", "khor_boundary"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let result = dev
        .spawn_command_sync(cmd)
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert!(result.status.success());
    serde_json::from_slice(&result.stdout).unwrap()
}
async fn measured(conn: &Connection, origin: Instant, event: &str) -> (bool, bool, u128) {
    let began = Instant::now();
    let work = ping_open(conn, OBSERVE);
    tokio::pin!(work);
    let (out, within) = tokio::select! {
        out=&mut work => (out, true),
        _=tokio::time::sleep(CALLER) => {
            emit(origin, "caller_deadline", "client", serde_json::json!({"operation":event,"elapsed_us":began.elapsed().as_micros()}));
            ((&mut work).await, false)
        }
    };
    let ok = out.is_ok();
    let elapsed = began.elapsed().as_micros();
    emit(
        origin,
        event,
        "client",
        serde_json::json!({"ok":ok,"within_caller":within,"elapsed_us":elapsed,"error":out.err().map(|e|format!("{e:#}")),"selected":selected(conn),"stable_id":conn.stable_id()}),
    );
    (ok, within, elapsed)
}
async fn run(fault: Fault) -> Result {
    let origin = Instant::now();
    emit(
        origin,
        "setup_start",
        "controller",
        serde_json::json!({"fault":format!("{fault:?}")}),
    );
    let out =
        std::path::PathBuf::from(std::env::var_os("BOUNDARY_OUT").expect("owned report directory"));
    let (lab, map, _relay, guard) = tokio::time::timeout(SETUP, util::lab_with_relay(out))
        .await
        .with_std_context(|_| "lab setup timeout")??;
    emit(
        origin,
        "setup_lab_ready",
        "controller",
        serde_json::json!({}),
    );
    let nat1 = lab.add_router("n1").nat(Nat::Moderate).build().await?;
    let nat2 = lab.add_router("n2").nat(Nat::Moderate).build().await?;
    let server = lab.add_device("server").uplink(nat1.id()).build().await?;
    let client = lab.add_device("client").uplink(nat2.id()).build().await?;
    let ready = Arc::new(Barrier::new(2));
    let armed = Arc::new(Barrier::new(2));
    let sr = ready.clone();
    let sa = armed.clone();
    let probe_map = map.clone();
    let result = Arc::new(Mutex::new(None));
    let client_result = result.clone();
    Pair::new(map)
        .server(server, async move |dev, ep, conn| {
            let observer=watch(&conn, origin, "server");
            let incoming=tokio::time::timeout(SETUP,ep.accept()).await.with_std_context(|_| "probe accept timeout")?.context("probe accept")?;
            let probe=incoming.accept().unwrap().await.context("probe handshake")?;
            assert!(util::is_relayed(&probe));
            ping_accept(&probe,SETUP).await.context("Relay preflight")?;
            conn.wait_ip(SETUP).await?;
            ping_accept(&conn,SETUP).await.context("Direct preflight")?;
            assert!(conn.paths().iter().any(|p|p.is_relay()),"main retains Relay path");
            emit(origin,"preflight","server",serde_json::json!({"selected":selected(&conn),"stable_id":conn.stable_id(),"relay_probe_selected":selected(&probe)}));
            sr.wait().await;
            let rule=rules(&dev,fault,true).await;emit(origin,"firewall","server",rule);
            sa.wait().await;
            let (business,relay)=tokio::join!(ping_accept(&conn,OBSERVE),async {if fault.relay(){Ok(())}else{ping_accept(&probe,OBSERVE).await}});
            emit(origin,"received","server",serde_json::json!({"main_ok":business.is_ok(),"relay_ok":relay.is_ok(),"stable_id":conn.stable_id(),"selected":selected(&conn),"counters":counters(&dev)}));
            let _=tokio::time::timeout(Duration::from_secs(5),conn.closed()).await;
            observer.abort();let _=observer.await;
            Ok(())
        })
        .client(client,async move |dev, _ep, conn| {
            let observer=watch(&conn,origin,"client");
            let probe_ep=util::endpoint_builder(&dev,probe_map).clear_ip_transports().bind().await.context("probe bind")?;
            // Explicit Relay dial proves readiness without waiting for home selection.
            emit(origin,"setup_probe_dial","client",serde_json::json!({}));
            let addr=EndpointAddr::from_parts(conn.remote_id(),[TransportAddr::Relay("https://relay.test".parse().unwrap())]);
            let probe=tokio::time::timeout(SETUP,probe_ep.connect(addr,b"test")).await.with_std_context(|_| "probe dial timeout")?.context("probe dial")?;
            assert!(util::is_relayed(&probe));
            ping_open(&probe,SETUP).await.context("Relay preflight")?;
            conn.wait_ip(SETUP).await?;
            ping_open(&conn,SETUP).await.context("Direct preflight")?;
            assert!(conn.paths().iter().any(|p|p.is_relay()));
            let id=conn.stable_id();
            emit(origin,"preflight","client",serde_json::json!({"selected":selected(&conn),"stable_id":id,"relay_probe_selected":selected(&probe)}));
            ready.wait().await;
            let rule=rules(&dev,fault,false).await;emit(origin,"firewall","client",rule);
            armed.wait().await;
            let (business,relay)=tokio::join!(measured(&conn,origin,"main_result"),async{if fault.relay(){None}else{Some(measured(&probe,origin,"relay_witness").await)}});
            let rule=counters(&dev);let same=conn.stable_id()==id;
            emit(origin,"done","client",serde_json::json!({"fault":format!("{fault:?}"),"main":business,"relay_witness":relay,"same_connection":same,"selected":selected(&conn),"counters":rule}));
            *client_result.lock().unwrap()=Some((business,relay,same));
            conn.close(0u32.into(),b"fixture done");probe.close(0u32.into(),b"fixture done");
            observer.abort();let _=observer.await;
            Ok(())
        }).run().await?;
    guard.ok();
    let (business, relay, same) = result
        .lock()
        .unwrap()
        .take()
        .expect("complete client result");
    assert!(
        same && relay.is_none_or(|(ok, within, _)| ok && within),
        "same connection and independent allowed Relay witness"
    );
    assert!(
        business.0 && business.1,
        "real stream must complete within existing 2s interactive reference; longer exact-stream diagnosis retained"
    );
    Ok(())
}
#[tokio::test]
#[n0_tracing_test::traced_test]
async fn healthy() -> Result {
    run(Fault::None).await
}
#[tokio::test]
#[n0_tracing_test::traced_test]
async fn direct_client() -> Result {
    run(Fault::DirectClient).await
}
#[tokio::test]
#[n0_tracing_test::traced_test]
async fn direct_server() -> Result {
    run(Fault::DirectServer).await
}
#[tokio::test]
#[n0_tracing_test::traced_test]
async fn relay_client() -> Result {
    run(Fault::RelayClient).await
}
#[tokio::test]
#[n0_tracing_test::traced_test]
async fn relay_server() -> Result {
    run(Fault::RelayServer).await
}
