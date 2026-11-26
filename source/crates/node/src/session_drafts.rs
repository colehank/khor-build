//! Replicated unsent editor state; separate from execution and delivery state.
use super::*;
use khor_sync::drafts::{DraftDoc, DraftRevision, drafts_dir};

impl Node {
    pub(crate) fn session_attention(
        &self,
    ) -> Result<(Vec<DraftRevision>, std::collections::BTreeMap<String, u8>), String> {
        let loaded = load_pile::<DraftDoc>(&drafts_dir(self.root()), self.peer)?;
        Ok((loaded.doc.all(), loaded.doc.child_priorities()))
    }
    pub(crate) fn record_child_attention(
        &self,
        report: &khor_sync::drafts::ChildAttention,
    ) -> Result<(), String> {
        let mut loaded = load_pile::<DraftDoc>(&drafts_dir(self.root()), self.peer)?;
        loaded.doc.record_child(report)?;
        loaded.store.flush(&loaded.doc)?;
        Ok(())
    }

    pub fn session_drafts(&self) -> Result<Vec<DraftRevision>, String> {
        Ok(load_pile::<DraftDoc>(&drafts_dir(self.root()), self.peer)?
            .doc
            .all())
    }

    pub fn edit_session_draft(
        &self,
        revision: &DraftRevision,
        parents: &[String],
    ) -> Result<Vec<DraftRevision>, String> {
        let mut loaded = load_pile::<DraftDoc>(&drafts_dir(self.root()), self.peer)?;
        loaded.doc.edit(revision, parents)?;
        loaded.store.flush(&loaded.doc)?;
        Ok(loaded.doc.all())
    }
}
