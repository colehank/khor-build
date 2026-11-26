//! All ACP providers use this catalog and admission path. Reads do not launch
//! agents; explicit refresh and apply run only at the authenticated source home.
use crate::{AgentSpec, Node, SessionId};
pub use khor_core::session_import::*;
use khor_core::{ObservationNamespace, recovery::RecoveryExecution};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_CACHE: usize = 4 * 1024 * 1024;
const FRESH_MS: i64 = 60_000;

/// Aliases are proven by identical launch specifications. The final two names
/// are only the explicit migration bridge from Khor's old built-in adapters to
/// the exact shipped upstream ACP launches; custom overrides are never guessed.
fn agent_aliases(agent: &str, spec: &AgentSpec, agents: &[(String, AgentSpec)]) -> Vec<String> {
    let mut aliases: Vec<String> = agents
        .iter()
        .filter(|(_, other)| other == spec)
        .map(|(name, _)| name.clone())
        .collect();
    if !aliases.iter().any(|name| name == agent) {
        aliases.push(agent.into());
    }
    for (registered, native) in [("claude-acp", "claude"), ("codex-acp", "codex")] {
        let official = crate::agent_registry::KNOWN
            .iter()
            .find(|known| known.id == registered)
            .and_then(|known| known.argv)
            .and_then(|argv| {
                AgentSpec::from_argv(&argv.iter().map(|part| part.to_string()).collect::<Vec<_>>())
            });
        if official.as_ref() == Some(spec) {
            aliases.push(native.into());
        }
    }
    aliases
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    spec: AgentSpec,
    at_ms: i64,
    catalog: khor_acp::SessionCatalog,
}
fn snapshot_path(root: &Path, agent: &str) -> PathBuf {
    root.join(".khor/import-catalog")
        .join(format!("{}.json", blake3::hash(agent.as_bytes()).to_hex()))
}
fn read_snapshot(root: &Path, agent: &str, spec: &AgentSpec) -> Result<Option<Snapshot>, String> {
    let path = snapshot_path(root, agent);
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if bytes.len() > MAX_CACHE {
        return Err("ACP import snapshot exceeds its byte budget".into());
    }
    let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok((snapshot.spec == *spec).then_some(snapshot))
}

/// Missing lifecycle evidence is never interpreted as a free execution slot.
fn intent(
    caps: &khor_acp::CatalogCapabilities,
    row: &khor_acp::CatalogSession,
    execution: RecoveryExecution,
    fresh: bool,
) -> (Option<ImportIntent>, Option<String>) {
    let reason = if !fresh {
        Some("stale")
    } else if row.archived != Some(false) {
        Some("archive_unknown")
    } else if !row.cwd.is_absolute() {
        Some("directory_unknown")
    } else if execution == RecoveryExecution::Ended && (caps.resume || caps.load) {
        return (Some(ImportIntent::Import), None);
    } else if caps.fork {
        return (Some(ImportIntent::Fork), None);
    } else if execution == RecoveryExecution::Live {
        Some("running_elsewhere")
    } else if execution == RecoveryExecution::Ended {
        Some("restore_unsupported")
    } else {
        Some("owner_unknown")
    };
    (None, reason.map(str::to_owned))
}

impl Node {
    fn import_identity(
        &self,
        agent: &str,
        native: &str,
    ) -> crate::session_content::journal::Identity {
        crate::session_content::journal::Identity {
            home: self.device(),
            agent: agent.to_owned(),
            native_id: native.to_owned(),
        }
    }

    fn import_managed_sources(
        &self,
    ) -> Result<std::collections::HashMap<(String, String), SessionId>, String> {
        let mut out = std::collections::HashMap::new();
        let roster = self.roster_loaded()?;
        for view in self.every_session()? {
            let row = &view.session;
            if row.home != self.device() || !super::listed(&roster.doc, row) {
                continue;
            }
            if let Some(reference) = row
                .observation
                .as_ref()
                .filter(|reference| reference.home == self.device())
            {
                out.insert(
                    (reference.agent.clone(), reference.native_id.clone()),
                    row.id.clone(),
                );
            }
            if let Some(directory) = self.live.dir_of(&row.id) {
                if let Some(identity) = crate::session_content::journal::bound_identity(&directory)?
                    .filter(|identity| identity.home == self.device())
                {
                    out.insert((identity.agent, identity.native_id), row.id.clone());
                }
            }
            if let Some((_, identity)) = self.live.legacy_gui_source(&row.id)? {
                out.insert((identity.agent, identity.native_id), row.id.clone());
            }
        }
        Ok(out)
    }

    pub(crate) fn import_execution(
        &self,
        agent: &str,
        native: &str,
        running: &std::collections::HashSet<String>,
    ) -> Result<RecoveryExecution, String> {
        if running.contains(native) {
            return Ok(RecoveryExecution::Live);
        }
        let identity = self.import_identity(agent, native);
        if self.live.observed_execution_ended(&identity)? {
            return Ok(RecoveryExecution::Ended);
        }
        Ok(RecoveryExecution::Unknown)
    }

    fn import_catalog_here(&self) -> Result<ImportCatalog, String> {
        let now = crate::live::now_ms();
        let mut out = ImportCatalog {
            home: self.device_str().into(),
            at_ms: now,
            agents: Vec::new(),
            rows: Vec::new(),
        };
        let managed = self.import_managed_sources()?;
        let running = self.live.import_running_ids();
        let agents = self.agents()?;
        let mut sources: std::collections::HashMap<(String, String), (usize, i64)> =
            std::collections::HashMap::new();
        for (agent, spec) in &agents {
            let snapshot = read_snapshot(self.root(), agent, spec)?;
            let caps = snapshot
                .as_ref()
                .map(|snapshot| snapshot.catalog.capabilities.clone())
                .unwrap_or_default();
            out.agents.push(ImportAgent {
                agent: agent.clone(),
                at_ms: snapshot.as_ref().map(|s| s.at_ms),
                status: if snapshot.is_none() {
                    "unasked"
                } else if !caps.list {
                    "cannot"
                } else {
                    "sessions"
                }
                .into(),
                reason: None,
                load: caps.load,
                resume: caps.resume,
                fork: caps.fork,
                archive: caps.archive,
            });
            let Some(snapshot) = snapshot else {
                continue;
            };
            let aliases = agent_aliases(agent, spec, &agents);
            let signature = serde_json::to_string(spec).map_err(|error| error.to_string())?;
            let fresh = now >= snapshot.at_ms && now - snapshot.at_ms <= FRESH_MS;
            for row in snapshot.catalog.sessions {
                if row.archived == Some(true)
                    || aliases
                        .iter()
                        .any(|alias| managed.contains_key(&(alias.clone(), row.id.clone())))
                {
                    continue;
                }
                let source = (signature.clone(), row.id.clone());
                if sources
                    .get(&source)
                    .is_some_and(|(_, at)| *at >= snapshot.at_ms)
                {
                    continue;
                }
                let execution = self.import_execution(agent, &row.id, &running)?;
                let (intent, reason) = intent(&caps, &row, execution, fresh);
                let candidate = ImportRow {
                    agent: agent.clone(),
                    native_id: row.id,
                    title: row.title.unwrap_or_default(),
                    cwd: row.cwd.display().to_string(),
                    updated_at: row.updated_at,
                    intent,
                    reason,
                };
                if let Some((index, at)) = sources.get_mut(&source) {
                    out.rows[*index] = candidate;
                    *at = snapshot.at_ms;
                } else {
                    sources.insert(source, (out.rows.len(), snapshot.at_ms));
                    out.rows.push(candidate);
                }
            }
        }
        out.rows.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.agent.cmp(&b.agent))
                .then_with(|| a.native_id.cmp(&b.native_id))
        });
        Ok(out)
    }

    pub(crate) async fn refresh_import_here(&self, agent: &str) -> Result<(), String> {
        self.refuse_unless_hosting()?;
        // Serialize this provider's small atomic snapshot publication across
        // processes. This gate is separate from the execution reservation.
        let path = snapshot_path(self.root(), agent);
        std::fs::create_dir_all(path.parent().expect("catalog directory"))
            .map_err(|error| error.to_string())?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))
            .map_err(|error| error.to_string())?;
        lock.try_lock()
            .map_err(|_| "This source is already refreshing its catalog".to_owned())?;
        let spec = self.agent_or_refuse(agent)?;
        let command = Self::agent_launch(&spec)?;
        let catalog = tokio::time::timeout(
            std::time::Duration::from_secs(28),
            khor_acp::session_catalog(&command, None),
        )
        .await
        .map_err(|_| "The agent did not finish listing within 28 seconds".to_owned())?
        .map_err(|error| error.said().to_owned())?;
        // Preserve the existing recovery source lookup as well as the richer
        // import snapshot. A fresh catalog never changes a provider identity.
        let answer = if catalog.capabilities.list {
            crate::AgentAnswer::Sessions {
                home: Some(self.device().0),
                rows: catalog
                    .sessions
                    .iter()
                    .map(|row| crate::AgentSeen {
                        id: row.id.clone(),
                        cwd: row.cwd.display().to_string(),
                        title: row.title.clone(),
                        updated_at: row.updated_at.clone(),
                    })
                    .collect(),
            }
        } else {
            crate::AgentAnswer::Cannot {
                home: Some(self.device().0),
            }
        };
        let mut loaded = self.agents_loaded()?;
        loaded.doc.set_seen(agent, &answer)?;
        loaded.store.flush(&loaded.doc)?;
        let snapshot = Snapshot {
            spec,
            at_ms: crate::live::now_ms(),
            catalog,
        };
        let bytes = serde_json::to_vec(&snapshot).map_err(|error| error.to_string())?;
        if bytes.len() > MAX_CACHE {
            return Err("ACP import snapshot exceeds its byte budget".into());
        }
        crate::live::write_whole(&path, &bytes)
    }

    pub(crate) async fn session_import_here(
        &self,
        action: SessionImportAction,
    ) -> Result<SessionImportReply, String> {
        if serde_json::to_vec(&action)
            .map_err(|error| error.to_string())?
            .len()
            > 16 * 1024
        {
            return Err("Import request exceeds its metadata budget".into());
        }
        match action {
            SessionImportAction::Read => Ok(SessionImportReply::Catalog {
                value: self.import_catalog_here()?,
            }),
            SessionImportAction::Refresh { agent } => {
                self.refresh_import_here(&agent).await?;
                Ok(SessionImportReply::Catalog {
                    value: self.import_catalog_here()?,
                })
            }
            SessionImportAction::Apply {
                agent,
                native_id,
                intent: requested,
                token,
            } => {
                self.refuse_unless_hosting()?;
                if !crate::once::valid(&token)
                    || agent.is_empty()
                    || agent.len() > 256
                    || native_id.is_empty()
                    || native_id.len() > 4096
                {
                    return Err("Invalid session import action".into());
                }
                let directory = self.root().join(".khor/import-actions").join(&token);
                std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
                let fingerprint = serde_json::to_vec(&(&agent, &native_id, requested))
                    .map_err(|error| error.to_string())?;
                // The cross-process action gate is held through publication.
                // A crash after a fork started is uncertain, never a new fork.
                let lock = std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(false)
                    .read(true)
                    .write(true)
                    .open(directory.join("action.lock"))
                    .map_err(|error| error.to_string())?;
                lock.try_lock()
                    .map_err(|_| "This import action is already in progress".to_owned())?;
                let request_path = directory.join("request.json");
                if let Ok(previous) = std::fs::read(&request_path) {
                    if previous != fingerprint {
                        return Err("Import token was used for another action".into());
                    }
                    return std::fs::read(directory.join("result.json"))
                        .map_err(|_| "Previous import result is uncertain; check the session list before retrying".to_owned())
                        .and_then(|bytes|serde_json::from_slice(&bytes).map_err(|error|error.to_string()));
                }
                if requested == ImportIntent::Import {
                    let spec = self.agent_or_refuse(&agent)?;
                    let aliases = agent_aliases(&agent, &spec, &self.agents()?);
                    let managed = self.import_managed_sources()?;
                    if let Some(id) = aliases
                        .iter()
                        .find_map(|alias| managed.get(&(alias.clone(), native_id.clone())))
                    {
                        return Ok(SessionImportReply::Imported {
                            home: self.device_str().into(),
                            id: id.0.clone(),
                            intent: requested,
                        });
                    }
                }
                self.refresh_import_here(&agent).await?;
                let catalog = self.import_catalog_here()?;
                let row = catalog
                    .rows
                    .into_iter()
                    .find(|row| row.agent == agent && row.native_id == native_id)
                    .ok_or("This session is archived, already managed, or no longer listed")?;
                if row.intent != Some(requested) {
                    return Err(format!(
                        "Session import is unavailable: {}",
                        row.reason.as_deref().unwrap_or("action_changed")
                    ));
                }
                if !Path::new(&row.cwd).is_dir() {
                    return Err("The recorded session directory is unavailable".into());
                }
                crate::live::write_whole(&request_path, &fingerprint)?;
                let id = if requested == ImportIntent::Import {
                    let mut record = crate::session_content::Observation::new(
                        self.device(),
                        self.device(),
                        ObservationNamespace::Acp,
                        agent.clone(),
                        native_id.clone(),
                        row.title,
                        Some(row.cwd),
                        None,
                    );
                    record.adopt_on_open = true;
                    record.import_spec = Some(self.agent_or_refuse(&agent)?);
                    self.register_observation(record)?
                } else {
                    let spec = self.agent_or_refuse(&agent)?;
                    let command = Self::agent_launch(&spec)?;
                    let root = self.root().to_path_buf();
                    let parent = native_id.clone();
                    let title = if row.title.is_empty() {
                        agent.clone()
                    } else {
                        format!("{} · 分支", row.title)
                    };
                    let vendor = agent.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::gui_host::spawn_gui_host_context(
                            &root,
                            Path::new(&row.cwd),
                            &title,
                            &[command],
                            &vendor,
                            &khor_acp::Environment {
                                fork_from: Some(parent),
                                ..Default::default()
                            },
                        )
                    })
                    .await
                    .map_err(|error| error.to_string())??
                };
                let result = SessionImportReply::Imported {
                    home: self.device_str().into(),
                    id: id.0,
                    intent: requested,
                };
                crate::live::write_whole(
                    &directory.join("result.json"),
                    &serde_json::to_vec(&result).map_err(|error| error.to_string())?,
                )?;
                Ok(result)
            }
        }
    }

    pub(crate) fn import_recovery_support(
        &self,
        agent: &str,
        native: &str,
    ) -> Option<khor_core::recovery::RecoverySupport> {
        let spec = self.agent(agent).ok().flatten()?;
        let snapshot = read_snapshot(self.root(), agent, &spec).ok().flatten()?;
        if !snapshot
            .catalog
            .sessions
            .iter()
            .any(|row| row.id == native && row.archived == Some(false))
        {
            return None;
        }
        if snapshot.catalog.capabilities.resume {
            Some(khor_core::recovery::RecoverySupport::Resume)
        } else if snapshot.catalog.capabilities.load {
            Some(khor_core::recovery::RecoverySupport::Load)
        } else {
            None
        }
    }
    pub(crate) fn import_recovery_metadata(
        &self,
        agent: &str,
        native: &str,
    ) -> Option<crate::AgentSeen> {
        let spec = self.agent(agent).ok().flatten()?;
        let snapshot = read_snapshot(self.root(), agent, &spec).ok().flatten()?;
        let row = snapshot
            .catalog
            .sessions
            .into_iter()
            .find(|row| row.id == native && row.archived == Some(false))?;
        Some(crate::AgentSeen {
            id: row.id,
            cwd: row.cwd.display().to_string(),
            title: row.title,
            updated_at: row.updated_at,
        })
    }
    pub(crate) async fn recheck_import_recovery(
        &self,
        agent: &str,
        native: &str,
    ) -> Result<(), String> {
        self.refresh_import_here(agent).await?;
        if self.import_recovery_support(agent, native).is_none() {
            return Err("The source no longer offers this unarchived recoverable session".into());
        }
        if self.import_execution(agent, native, &self.live.import_running_ids())?
            != RecoveryExecution::Ended
        {
            return Err("The original execution is no longer confirmed ended".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_import_aliases_require_an_identical_launch_and_never_titles() {
        let spec = AgentSpec::from_argv(&["fixture-acp".into(), "--acp".into()]).unwrap();
        let other = AgentSpec::from_argv(&["different-acp".into()]).unwrap();
        let agents = vec![
            ("one".into(), spec.clone()),
            ("alias".into(), spec.clone()),
            ("other".into(), other),
        ];
        let aliases = agent_aliases("one", &spec, &agents);
        assert!(aliases.iter().any(|name| name == "alias"));
        assert!(
            !aliases
                .iter()
                .any(|name| name == "other" || name == "claude" || name == "codex")
        );
        let official = crate::agent_registry::builtin("codex-acp")
            .unwrap()
            .argv
            .unwrap();
        let official =
            AgentSpec::from_argv(&official.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                .unwrap();
        assert!(
            agent_aliases("codex-acp", &official, &[])
                .iter()
                .any(|name| name == "codex")
        );
    }

    #[test]
    fn session_import_catalog_hides_managed_full_id_but_keeps_other_same_title() {
        let root = tempfile::tempdir().unwrap();
        let node = Node::open(root.path().to_path_buf()).unwrap();
        node.register_agent("generic", &["must-not-run".into()])
            .unwrap();
        node.register_agent("alias", &["must-not-run".into()])
            .unwrap();
        let spec = node.agent("generic").unwrap().unwrap();
        for (agent, at_ms) in [
            ("generic", crate::live::now_ms()),
            ("alias", crate::live::now_ms() - 120_000),
        ] {
            let mut rows = vec![record(), record(), record()];
            rows[1].id = "a-different-full-id".into();
            rows[2].id = "archived".into();
            rows[2].archived = Some(true);
            let snapshot = Snapshot {
                spec: spec.clone(),
                at_ms,
                catalog: khor_acp::SessionCatalog {
                    capabilities: khor_acp::CatalogCapabilities {
                        list: true,
                        fork: true,
                        ..Default::default()
                    },
                    sessions: rows,
                },
            };
            let path = snapshot_path(root.path(), agent);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
        }
        node.register_observation(crate::session_content::Observation::new(
            node.device(),
            node.device(),
            ObservationNamespace::Acp,
            "alias".into(),
            record().id,
            "same name".into(),
            None,
            None,
        ))
        .unwrap();
        let catalog = node.import_catalog_here().unwrap();
        assert_eq!(catalog.rows.len(), 1);
        assert_eq!(catalog.rows[0].native_id, "a-different-full-id");
        assert_eq!(
            catalog.rows[0].agent, "generic",
            "the freshest proven alias supplies the action"
        );
        assert_eq!(catalog.rows[0].intent, Some(ImportIntent::Fork));
    }

    fn record() -> khor_acp::CatalogSession {
        khor_acp::CatalogSession {
            id: "provider/full/id".into(),
            cwd: std::env::temp_dir(),
            title: Some("same name".into()),
            updated_at: None,
            archived: Some(false),
        }
    }
    #[test]
    fn session_import_policy_never_treats_absence_as_a_free_owner() {
        let caps = khor_acp::CatalogCapabilities {
            list: true,
            resume: true,
            ..Default::default()
        };
        assert_eq!(
            intent(&caps, &record(), RecoveryExecution::Unknown, true)
                .1
                .as_deref(),
            Some("owner_unknown")
        );
        assert_eq!(
            intent(&caps, &record(), RecoveryExecution::Ended, true).0,
            Some(ImportIntent::Import)
        );
        assert_eq!(
            intent(&caps, &record(), RecoveryExecution::Live, true)
                .1
                .as_deref(),
            Some("running_elsewhere")
        );
        assert_eq!(
            intent(&caps, &record(), RecoveryExecution::Ended, false)
                .1
                .as_deref(),
            Some("stale")
        );
    }
    #[test]
    fn session_import_archive_and_fork_are_capabilities_not_agent_names() {
        let caps = khor_acp::CatalogCapabilities {
            list: true,
            fork: true,
            ..Default::default()
        };
        assert_eq!(
            intent(&caps, &record(), RecoveryExecution::Live, true).0,
            Some(ImportIntent::Fork)
        );
        let mut unknown = record();
        unknown.archived = None;
        assert_eq!(
            intent(&caps, &unknown, RecoveryExecution::Live, true)
                .1
                .as_deref(),
            Some("archive_unknown")
        );
        unknown.archived = Some(true);
        assert_eq!(
            intent(&caps, &unknown, RecoveryExecution::Ended, true).0,
            None
        );
    }
    #[tokio::test]
    async fn session_import_catalog_includes_defaults_without_spawning_or_importing() {
        let root = tempfile::tempdir().unwrap();
        let node = Node::open(root.path().to_path_buf()).unwrap();
        node.register_agent("generic-acp", &["this-executable-must-never-start".into()])
            .unwrap();
        let SessionImportReply::Catalog { value } = node
            .session_import_here(SessionImportAction::Read)
            .await
            .unwrap()
        else {
            panic!("catalog")
        };
        assert!(
            value
                .agents
                .iter()
                .any(|agent| agent.agent == "dsh" && agent.status == "unasked")
        );
        for name in ["claude-acp", "codex-acp"] {
            assert!(
                value
                    .agents
                    .iter()
                    .any(|agent| agent.agent == name && agent.status == "unasked")
            );
        }
        assert!(
            value
                .agents
                .iter()
                .any(|agent| agent.agent == "generic-acp" && agent.status == "unasked")
        );
        assert!(value.rows.is_empty());
        assert!(!root.path().join(".khor/import-actions").exists());
    }
    #[test]
    fn session_import_snapshot_does_not_survive_a_different_launch() {
        let root = tempfile::tempdir().unwrap();
        let path = snapshot_path(root.path(), "generic");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let spec = AgentSpec {
            command: "first".into(),
            args: Vec::new(),
        };
        let snapshot = Snapshot {
            spec: spec.clone(),
            at_ms: 1,
            catalog: khor_acp::SessionCatalog {
                capabilities: Default::default(),
                sessions: Vec::new(),
            },
        };
        std::fs::write(path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
        assert!(
            read_snapshot(root.path(), "generic", &spec)
                .unwrap()
                .is_some()
        );
        assert!(
            read_snapshot(
                root.path(),
                "generic",
                &AgentSpec {
                    command: "second".into(),
                    args: Vec::new()
                }
            )
            .unwrap()
            .is_none()
        );
    }
    #[test]
    fn session_import_recovery_uses_local_catalog_even_when_synced_seen_has_another_home() {
        let root = tempfile::tempdir().unwrap();
        let node = Node::open(root.path().to_path_buf()).unwrap();
        node.register_agent("generic", &["must-not-run".into()])
            .unwrap();
        let spec = node.agent("generic").unwrap().unwrap();
        let mut record = crate::session_content::Observation::new(
            node.device(),
            node.device(),
            ObservationNamespace::Acp,
            "generic".into(),
            record().id,
            "same name".into(),
            Some(root.path().display().to_string()),
            None,
        );
        record.adopt_on_open = true;
        record.import_spec = Some(spec.clone());
        let id = node.register_observation(record).unwrap();
        let snapshot = Snapshot {
            spec,
            at_ms: crate::live::now_ms(),
            catalog: khor_acp::SessionCatalog {
                capabilities: khor_acp::CatalogCapabilities {
                    list: true,
                    resume: true,
                    ..Default::default()
                },
                sessions: vec![self::record()],
            },
        };
        let path = snapshot_path(root.path(), "generic");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
        {
            let loaded = node.agents_loaded().unwrap();
            loaded
                .doc
                .set_seen(
                    "generic",
                    &crate::AgentAnswer::Sessions {
                        home: Some([3; 32]),
                        rows: vec![],
                    },
                )
                .unwrap();
            let mut store = loaded.store;
            store.flush(&loaded.doc).unwrap();
        }
        let offer = node
            .session_recovery_here(node.device(), &id, None)
            .unwrap();
        assert_eq!(offer.execution, RecoveryExecution::Unknown);
        assert_eq!(
            offer.native.unwrap().support,
            khor_core::recovery::RecoverySupport::Resume
        );
        node.register_agent("generic", &["changed-launch".into()])
            .unwrap();
        assert!(
            node.session_recovery_here(node.device(), &id, None)
                .is_err(),
            "an override cannot silently adopt another source"
        );
    }
}
