//! Capability-qualified history discovery. Raw envelopes retain the optional
//! archive fields which predate the pinned SDK's typed schema; the wire methods
//! and fields are ACP's, not vendor-specific heuristics.
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogCapabilities {
    pub list: bool,
    pub load: bool,
    pub resume: bool,
    pub fork: bool,
    pub archive: bool,
}
impl CatalogCapabilities {
    pub fn from_initialize(hello: &Value) -> Result<Self, Refusal> {
        if hello["protocolVersion"].as_u64() != Some(1) {
            return Err(Refusal::Version(hello["protocolVersion"].to_string()));
        }
        let caps = &hello["agentCapabilities"];
        let session = &caps["sessionCapabilities"];
        Ok(Self {
            list: session["list"].is_object(),
            load: caps["loadSession"].as_bool() == Some(true),
            resume: session["resume"].is_object(),
            fork: session["fork"].is_object(),
            archive: session["archive"].is_object(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSession {
    #[serde(rename = "sessionId")]
    pub id: String,
    pub cwd: PathBuf,
    pub title: Option<String>,
    pub updated_at: Option<String>,
    /// Unknown is not unarchived. Only the source can answer this.
    pub archived: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionCatalog {
    pub capabilities: CatalogCapabilities,
    pub sessions: Vec<CatalogSession>,
}

pub(crate) async fn raw_request(
    connection: &ConnectionTo<Agent>,
    method: &str,
    params: Value,
) -> Result<Value, Refusal> {
    connection
        .send_request(agent_client_protocol::UntypedMessage::new(method, params).map_err(refusal)?)
        .block_task()
        .await
        .map_err(refusal)
}

pub async fn session_catalog(
    command: &str,
    cwd: Option<PathBuf>,
) -> Result<SessionCatalog, Refusal> {
    let agent = AcpAgent::from_str(command).map_err(|error| Refusal::Wont(error.to_string()))?;
    let (tx, rx) = oneshot::channel();
    // Await the connection owner as well as the answer. Cancellation drops
    // this owner; a discovery request never leaves a detached listing task.
    Client
        .builder()
        .connect_with(agent, move |connection: ConnectionTo<Agent>| async move {
            let answer = discover(&connection, cwd).await;
            let _ = tx.send(answer);
            Ok(())
        })
        .await
        .map_err(|error| Refusal::Wont(error.to_string()))?;
    rx.await
        .map_err(|_| Refusal::Wont("The agent ended before listing sessions".into()))?
}

async fn discover(
    connection: &ConnectionTo<Agent>,
    cwd: Option<PathBuf>,
) -> Result<SessionCatalog, Refusal> {
    let hello = raw_request(
        connection,
        "initialize",
        serde_json::to_value(InitializeRequest::new(ProtocolVersion::V1))
            .map_err(|error| Refusal::Wont(error.to_string()))?,
    )
    .await?;
    let capabilities = CatalogCapabilities::from_initialize(&hello)?;
    let mut catalog = SessionCatalog {
        capabilities,
        sessions: Vec::new(),
    };
    if !catalog.capabilities.list {
        return Ok(catalog);
    }
    let mut cursor = None;
    let mut cursors = std::collections::HashSet::new();
    let mut ids = std::collections::HashSet::new();
    let mut archived = std::collections::HashSet::new();
    let mut bytes = 0;
    for _ in 0..64 {
        let mut params = json!({});
        if let Some(cwd) = &cwd {
            params["cwd"] = json!(cwd);
        }
        if let Some(cursor) = cursor.take() {
            params["cursor"] = cursor;
        }
        if catalog.capabilities.archive {
            params["archived"] = json!("exclude");
        }
        let answer = raw_request(connection, "session/list", params).await?;
        bytes += answer.to_string().len();
        if bytes > 3 * 1024 * 1024 {
            return Err(Refusal::Wont(
                "Session metadata exceeds its byte budget".into(),
            ));
        }
        let rows = answer["sessions"]
            .as_array()
            .ok_or_else(|| Refusal::Wont("Invalid ACP session list".into()))?;
        for value in rows {
            let mut row: CatalogSession = serde_json::from_value(value.clone())
                .map_err(|error| Refusal::Wont(error.to_string()))?;
            if row.id.is_empty() || row.id.len() > 4096 || row.id.chars().any(char::is_control) {
                return Err(Refusal::Wont("Invalid ACP session identity".into()));
            }
            // An advertised archive filter is authoritative even if a source
            // omits the redundant false field. Explicit true always wins.
            if row.archived.is_none() && catalog.capabilities.archive {
                row.archived = Some(false);
            }
            if row.archived == Some(true) {
                archived.insert(row.id.clone());
                catalog.sessions.retain(|previous| previous.id != row.id);
                continue;
            }
            if archived.contains(&row.id) {
                continue;
            }
            if ids.insert(row.id.clone()) {
                catalog.sessions.push(row);
            }
            if catalog.sessions.len() > 4096 {
                return Err(Refusal::Wont(
                    "Too many sessions in one import catalog".into(),
                ));
            }
        }
        let Some(next) = answer.get("nextCursor").filter(|value| !value.is_null()) else {
            return Ok(catalog);
        };
        let next = next
            .as_str()
            .filter(|value| !value.is_empty() && value.len() <= 4096)
            .ok_or_else(|| Refusal::Wont("Invalid session-list cursor".into()))?;
        if !cursors.insert(next.to_owned()) {
            return Err(Refusal::Wont(
                "The agent repeated a session-list cursor".into(),
            ));
        }
        cursor = Some(json!(next));
    }
    Err(Refusal::Wont(
        "Session list exceeded its page budget".into(),
    ))
}
