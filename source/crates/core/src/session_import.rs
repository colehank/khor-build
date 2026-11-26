//! Session import contracts shared by the execution owner and every client.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ImportIntent {
    Import,
    Fork,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionImportAction {
    Read,
    Refresh {
        agent: String,
    },
    Apply {
        agent: String,
        native_id: String,
        intent: ImportIntent,
        token: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
pub struct ImportAgent {
    pub agent: String,
    #[ts(type = "number | null")]
    pub at_ms: Option<i64>,
    /// unasked, cannot, sessions, or failed; never collapse these to an empty list.
    pub status: String,
    pub reason: Option<String>,
    pub load: bool,
    pub resume: bool,
    pub fork: bool,
    pub archive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
pub struct ImportRow {
    pub agent: String,
    pub native_id: String,
    pub title: String,
    pub cwd: String,
    pub updated_at: Option<String>,
    pub intent: Option<ImportIntent>,
    /// A stable reason code, localized by the face.
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
pub struct ImportCatalog {
    pub home: String,
    #[ts(type = "number")]
    pub at_ms: i64,
    pub agents: Vec<ImportAgent>,
    pub rows: Vec<ImportRow>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionImportReply {
    Catalog {
        value: ImportCatalog,
    },
    Imported {
        home: String,
        id: String,
        intent: ImportIntent,
    },
}
