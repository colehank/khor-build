//! Unsent editor snapshots, replicated as an observed-remove set.
//!
//! Each edit replaces only the revision(s) its editor actually saw. Concurrent
//! offline edits therefore survive as separate alternatives; clearing an old
//! submission cannot clear a new edit. Text is never guessed from PTY traffic.
use std::path::{Path, PathBuf};

use crate::{glue, store::Doc};
use loro::{LoroDoc, LoroValue, VersionVector};

/// The legacy wire name is `titles`: the session-metadata document already
/// traverses every supported peer. Loro preserves maps an older reader does not
/// interpret, so adding editor namespaces does not make mixed-version sync fail.
/// Do not introduce a second pile for the same document.
pub fn drafts_dir(home: &Path) -> PathBuf {
    crate::titles::titles_dir(home)
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct DraftRevision {
    pub id: String,
    pub key: String,
    pub text: String,
    #[ts(type = "number")]
    pub at_ms: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChildAttention {
    pub session: String,
    pub child: String,
    pub revision: u64,
    pub rank: Option<u8>,
}

pub struct DraftDoc {
    inner: LoroDoc,
}

impl DraftDoc {
    pub fn new(peer: u64) -> Result<Self, String> {
        Ok(Self {
            inner: glue::with_peer(peer)?,
        })
    }

    pub fn all(&self) -> Vec<DraftRevision> {
        let mut rows = Vec::new();
        self.inner.get_map("editors").for_each(|_, value| {
            if let Ok(LoroValue::String(value)) = value.into_value() {
                if let Ok(row) = serde_json::from_str::<DraftRevision>(&value) {
                    rows.push(row);
                }
            }
        });
        rows.sort_by(|a, b| a.id.cmp(&b.id));
        rows
    }

    fn child_reports(&self) -> std::collections::BTreeMap<(String, String), ChildAttention> {
        let mut reports: std::collections::BTreeMap<(String, String), ChildAttention> =
            Default::default();
        self.inner.get_map("children").for_each(|_, value| {
            if let Ok(LoroValue::String(value)) = value.into_value() {
                if let Ok(report) = serde_json::from_str::<ChildAttention>(&value) {
                    let key = (report.session.clone(), report.child.clone());
                    if reports
                        .get(&key)
                        .is_none_or(|held| held.revision < report.revision)
                    {
                        reports.insert(key, report);
                    }
                }
            }
        });
        reports
    }

    pub fn child_priorities(&self) -> std::collections::BTreeMap<String, u8> {
        let mut priorities = std::collections::BTreeMap::<String, u8>::new();
        for report in self.child_reports().into_values() {
            if let Some(rank) = report.rank {
                priorities
                    .entry(report.session)
                    .and_modify(|held| *held = (*held).min(rank))
                    .or_insert(rank);
            }
        }
        priorities
    }

    pub fn record_child(&self, report: &ChildAttention) -> Result<(), String> {
        let key = (report.session.clone(), report.child.clone());
        let previous = self.child_reports();
        if previous
            .get(&key)
            .is_some_and(|held| held.revision >= report.revision)
        {
            return Ok(());
        }
        let map = self.inner.get_map("children");
        // Version-qualified keys make a concurrent older poll harmless too.
        let wire_key =
            serde_json::to_string(&(&key, report.revision)).map_err(|e| e.to_string())?;
        map.insert(
            &wire_key,
            serde_json::to_string(report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if let Some(old) = previous.get(&key) {
            let old_key =
                serde_json::to_string(&(&key, old.revision)).map_err(|e| e.to_string())?;
            map.delete(&old_key).map_err(|e| e.to_string())?;
        }
        self.inner.commit();
        Ok(())
    }

    /// Repeating a delivery is harmless. A revision is immutable; an old reply
    /// never assigns the current editor, and unrelated alternatives remain.
    pub fn edit(&self, revision: &DraftRevision, parents: &[String]) -> Result<(), String> {
        if revision.id.is_empty()
            || revision.id.len() > 128
            || !revision
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-._:".contains(&c))
            || revision.key.len() > 1024
            || !(revision.key.starts_with("gui:")
                || revision.key.starts_with("khor:")
                || revision.key.starts_with("term:"))
            || revision.text.len() > 512 * 1024
            || parents.len() > 512
            || parents.iter().any(|id| id == &revision.id)
        {
            return Err("invalid editor revision".into());
        }
        let map = self.inner.get_map("editors");
        // Keep receipts separately: retrying an edit after its successor removed
        // it must not resurrect that old text.
        let receipts = self.inner.get_map("receipts");
        if receipts.get(&revision.id).is_some() {
            return Ok(());
        }
        let rows = self.all();
        if parents.iter().any(|id| {
            rows.iter()
                .any(|row| &row.id == id && row.key != revision.key)
        }) {
            return Err("editor revision belongs to another session".into());
        }
        for parent in parents {
            map.delete(parent).map_err(|e| e.to_string())?;
        }
        if !revision.text.is_empty() {
            map.insert(
                &revision.id,
                serde_json::to_string(revision).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
        receipts
            .insert(&revision.id, true)
            .map_err(|e| e.to_string())?;
        self.inner.commit();
        Ok(())
    }
}

impl Doc for DraftDoc {
    fn open(peer: u64) -> Result<Self, String> {
        DraftDoc::new(peer)
    }

    fn peer_id(&self) -> u64 {
        self.inner.peer_id()
    }

    fn version(&self) -> VersionVector {
        self.inner.oplog_vv()
    }

    fn changes_since(&self, theirs: &VersionVector) -> Result<Vec<u8>, String> {
        glue::changes_since(&self.inner, theirs)
    }

    fn snapshot(&self) -> Result<Vec<u8>, String> {
        glue::snapshot(&self.inner)
    }

    fn merge_report(&self, bytes: &[u8]) -> Result<crate::store::MergeReport, String> {
        glue::merge_report(&self.inner, bytes)
    }

    fn items(&self) -> usize {
        self.all().len() + self.child_reports().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn revision(id: &str, text: &str) -> DraftRevision {
        DraftRevision {
            id: id.into(),
            key: "gui:session".into(),
            text: text.into(),
            at_ms: 1,
        }
    }
    fn merge(a: &DraftDoc, b: &DraftDoc) {
        a.merge(&b.snapshot().unwrap()).unwrap();
        b.merge(&a.snapshot().unwrap()).unwrap();
    }
    #[test]
    fn offline_edits_survive_and_resolving_them_converges() {
        let a = DraftDoc::new(1).unwrap();
        let b = DraftDoc::new(2).unwrap();
        a.edit(&revision("base", "hello"), &[]).unwrap();
        merge(&a, &b);
        a.edit(&revision("a", "hello from phone"), &["base".into()])
            .unwrap();
        b.edit(&revision("b", "hello from desktop"), &["base".into()])
            .unwrap();
        merge(&a, &b);
        assert_eq!(a.all(), b.all());
        assert_eq!(a.all().len(), 2);
        a.edit(
            &revision("resolved", "hello from both"),
            &["a".into(), "b".into()],
        )
        .unwrap();
        merge(&a, &b);
        assert_eq!(a.all(), vec![revision("resolved", "hello from both")]);
        assert_eq!(a.all(), b.all());
    }
    #[test]
    fn old_ack_does_not_clear_new_input() {
        let a = DraftDoc::new(1).unwrap();
        a.edit(&revision("sent", "old text"), &[]).unwrap();
        a.edit(&revision("new", "new text"), &["sent".into()])
            .unwrap();
        a.edit(&revision("ack", ""), &["sent".into()]).unwrap();
        a.edit(&revision("sent", "old text"), &[]).unwrap();
        assert_eq!(a.all(), vec![revision("new", "new text")]);
    }
    #[test]
    fn child_attention_cannot_be_restored_by_an_older_status_reply() {
        let a = DraftDoc::new(1).unwrap();
        let b = DraftDoc::new(2).unwrap();
        let report = |revision, rank| ChildAttention {
            session: "wide/parent".into(),
            child: "task".into(),
            revision,
            rank,
        };
        a.record_child(&report(1, Some(0))).unwrap();
        b.record_child(&report(2, None)).unwrap();
        merge(&a, &b);
        assert!(a.child_priorities().is_empty());
        a.record_child(&report(1, Some(0))).unwrap();
        assert!(a.child_priorities().is_empty());
    }
    #[test]
    fn legacy_metadata_readers_relay_drafts_without_understanding_them() {
        let newer = DraftDoc::new(1).unwrap();
        let legacy = crate::titles::TitleDoc::new(2).unwrap();
        newer
            .edit(&revision("editor", "retained through old peer"), &[])
            .unwrap();
        legacy.merge(&newer.snapshot().unwrap()).unwrap();
        legacy.set("gui:session", "user title").unwrap();
        let phone = DraftDoc::new(3).unwrap();
        phone.merge(&legacy.snapshot().unwrap()).unwrap();
        assert_eq!(phone.all(), newer.all());
        let names = crate::titles::TitleDoc::new(4).unwrap();
        names.merge(&phone.snapshot().unwrap()).unwrap();
        assert_eq!(names.title_of("gui:session").as_deref(), Some("user title"));
    }

    #[test]
    fn concurrent_submission_clear_and_new_edit_keep_the_new_text() {
        let phone = DraftDoc::new(1).unwrap();
        let desktop = DraftDoc::new(2).unwrap();
        phone.edit(&revision("old", "sent text"), &[]).unwrap();
        merge(&phone, &desktop);
        phone
            .edit(&revision("receipt", ""), &["old".into()])
            .unwrap();
        desktop
            .edit(
                &revision("newer", "typing while the receipt arrives"),
                &["old".into()],
            )
            .unwrap();
        merge(&phone, &desktop);
        assert_eq!(
            phone.all(),
            vec![revision("newer", "typing while the receipt arrives")]
        );
        assert_eq!(phone.all(), desktop.all());
    }
}
