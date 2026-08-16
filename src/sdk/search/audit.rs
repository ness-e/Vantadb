use super::super::builder::VantaEmbedded;
use super::super::types::*;
use super::snippet;
use super::text_index;
use crate::error::{Result, VantaError};
use tracing;

impl VantaEmbedded {
    /// Run a read-only structural audit of the derived persistent text index.
    #[tracing::instrument(skip(self), err)]
    pub fn audit_text_index(&self, namespace: Option<&str>) -> Result<VantaTextIndexAuditReport> {
        let engine = self.engine_handle()?;
        text_index::run_audit(&engine, namespace)
    }

    /// Run a deep structural audit of the derived persistent text index.
    #[tracing::instrument(skip(self), err)]
    pub fn audit_text_index_deep(
        &self,
        namespace: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let engine = self.engine_handle()?;
        text_index::run_audit_deep(&engine, namespace)
    }

    /// Public repair primitive for the text index.
    #[tracing::instrument(skip(self), err)]
    pub fn repair_text_index(&self) -> Result<VantaTextIndexRepairReport> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "repair_text_index is not available when VantaDB is opened read-only"
                    .into(),
            });
        }
        crate::metrics::record_text_index_repair();
        let report = self.rebuild_text_index_with_report()?;
        Ok(text_index::run_repair(report))
    }

    /// Generate a text snippet with optional highlighting of matched terms.
    #[tracing::instrument(skip(self, payload))]
    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        snippet::generate_snippet_with_highlighting(payload, text_query, with_highlighting)
    }
    // highlight_terms, generate_snippet_with_highlighting moved to snippet.rs
}
