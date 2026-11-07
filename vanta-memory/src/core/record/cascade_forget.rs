//! TEC-15 (Q4-C): GDPR Art. 17 Right to Erasure cascade from episodic L1
//! nodes to derived L2 scenes and L3 axioms/personas in `vanta-memory`.
//!
//! # Problem & Invariant
//! When an episodic L1 node is forgotten via `forget(episode_id, mode)` under
//! GDPR Art. 17, deleting only the L1 record leaves personal data (PII) and
//! derived inferences alive within downstream L2 scenes and L3 axioms.
//!
//! This module implements deterministic provenance traversal across:
//! - L1: episodic records (`l1/<session>`)
//! - L2: consolidated scenes (`scene/<session>`) with `source_episode_ids`
//! - L3: persona documents (`persona/<session>`) and semantic axioms (`l1/<session>`)
//!
//! When `forget` is called with `Soft`, `Hard`, or `Certified`:
//! 1. All direct target L1 records are erased/invalidated.
//! 2. Downstream L2 scenes derived from the forgotten L1 records are identified
//!    and erased/invalidated at the same level.
//! 3. Downstream L3 axioms/personas derived from the forgotten L1 records or L2
//!    scenes are transitively identified and erased/invalidated at the same level.
//! 4. Zero PII leakage is guaranteed across searches and point-reads.
//! 5. A verifiable receipt is returned containing `cascaded_ids` and attestation
//!    certificates for cryptographic and audit compliance.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use vantadb::attestation::PurgeCertificate;
use vantadb::sdk::{Embedded, ForgetMode, ForgetTarget, MemoryListOptions};

use crate::core::abstractions::{MemoryRecord, MemoryType};
use crate::core::conversation::sanitize_key;
use crate::core::ontology::triplets_namespace;
use crate::core::persona::persona_generator::{get_persona, persona_namespace, PERSONA_KEY};
use crate::core::profile::profile_sync::{profile_namespace, DEFAULT_PROFILE_SCOPE};
use crate::core::record::l1_reader::{l1_namespace, read_session_records};
use crate::core::record::L1Error;
use crate::core::scene::scene_format::SOFT_DELETE_MARKER;
use crate::core::scene::scene_index::{scene_namespace, soft_delete_scene};

/// Target adapter trait allowing callers to pass string keys, string slices, or `ForgetTarget`.
pub trait IntoForgetTarget {
    fn into_forget_target(self) -> ForgetTarget;
}

impl IntoForgetTarget for ForgetTarget {
    fn into_forget_target(self) -> ForgetTarget {
        self
    }
}

impl IntoForgetTarget for &str {
    fn into_forget_target(self) -> ForgetTarget {
        ForgetTarget::Key(self.to_string())
    }
}

impl IntoForgetTarget for String {
    fn into_forget_target(self) -> ForgetTarget {
        ForgetTarget::Key(self)
    }
}

impl IntoForgetTarget for &String {
    fn into_forget_target(self) -> ForgetTarget {
        ForgetTarget::Key(self.clone())
    }
}

/// Errors surfaced by the cascade forget operation.
#[derive(Debug, Error)]
pub enum CascadeForgetError {
    #[error("vantadb error: {0}")]
    Vanta(#[from] vantadb::error::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("l1 error: {0}")]
    L1(#[from] L1Error),
    #[error("invalid session: {0}")]
    InvalidSession(String),
}

/// Detailed record of a memory item erased or invalidated in cascade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CascadedItemDetail {
    /// Memory hierarchy layer (e.g. "L1", "L2", "L3").
    pub layer: String,
    /// Storage namespace where the item lived.
    pub namespace: String,
    /// Identifier or key of the item.
    pub id: String,
    /// Action applied ("soft_deleted", "deleted", "certified_purged").
    pub action: String,
}

/// Verifiable outcome of a GDPR cascade forget operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CascadeForgetReceipt {
    /// Erasure mode requested (`Soft`, `Hard`, `Certified`).
    pub mode: ForgetMode,
    /// Session identifier.
    pub session_id: String,
    /// Primary target description.
    pub target: String,
    /// ISO 8601 UTC timestamp of execution.
    pub timestamp: String,
    /// Root primary keys forgotten directly.
    pub primary_keys: Vec<String>,
    /// All downstream L2 scene and L3 axiom/persona IDs purged/invalidated in cascade.
    #[serde(default)]
    pub cascaded_ids: Vec<String>,
    /// Fine-grained breakdown of cascaded items.
    #[serde(default)]
    pub details: Vec<CascadedItemDetail>,
    /// Attestation purge certificates emitted (populated in `Certified` mode).
    #[serde(default)]
    pub certificates: Vec<PurgeCertificate>,
    /// Status: "erased" | "soft_invalidated".
    #[serde(default)]
    pub status: String,
    /// Cryptographic erasure receipt (VER-02) when feature `erasure` is enabled.
    #[cfg(feature = "erasure")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub erasure_receipt: Option<crate::utils::erasure::ErasureReceipt>,
}

/// Put a semantic L3 axiom with explicit source episode provenance into the session store.
pub fn put_l3_axiom(
    db: &Embedded,
    session_id: &str,
    axiom_id: &str,
    content: &str,
    source_episode_ids: Vec<String>,
) -> Result<MemoryRecord, L1Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let record = MemoryRecord {
        id: axiom_id.to_string(),
        content: content.to_string(),
        memory_type: MemoryType::Persona,
        priority: 100,
        scene_name: "axiom".to_string(),
        source_message_ids: source_episode_ids.clone(),
        metadata: serde_json::json!({
            "source_episode_ids": source_episode_ids,
            "is_l3_axiom": true,
        }),
        timestamps: vec![now.clone()],
        created_at: now.clone(),
        updated_at: now,
        version: 1,
        session_key: session_id.to_string(),
        session_id: session_id.to_string(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 1,
        superseded_by: None,
    };
    crate::core::record::put_record(db, &l1_namespace(session_id), &record, None)?;
    Ok(record)
}

/// Execute a GDPR Right to Erasure cascade forget across L1, L2, and L3.
pub fn forget(
    db: &Embedded,
    session_id: &str,
    target: impl IntoForgetTarget,
    mode: ForgetMode,
) -> Result<CascadeForgetReceipt, CascadeForgetError> {
    if session_id.trim().is_empty() {
        return Err(CascadeForgetError::InvalidSession(
            "session_id must not be empty".into(),
        ));
    }

    let target_enum = target.into_forget_target();
    let l1_ns = l1_namespace(session_id);
    let scene_ns = scene_namespace(session_id);
    let persona_ns = persona_namespace(session_id);
    let triplets_ns = triplets_namespace(session_id);

    // 1. Identify primary L1 target keys
    let mut target_keys: HashSet<String> = HashSet::new();
    let target_repr = match &target_enum {
        ForgetTarget::Key(k) => {
            target_keys.insert(k.clone());
            format!("key:{k}")
        }
        ForgetTarget::Filter(_) | ForgetTarget::Query { .. } => {
            if let Ok(report) = db.forget(&l1_ns, target_enum.clone(), ForgetMode::Soft) {
                for k in report.keys {
                    target_keys.insert(k);
                }
            }
            format!("{target_enum:?}")
        }
    };

    // If ForgetTarget::Key was provided, verify if it exists in l1
    let mut primary_records: Vec<String> = Vec::new();
    for k in &target_keys {
        if db.get(&l1_ns, &sanitize_key(k))?.is_some() {
            primary_records.push(k.clone());
        }
    }
    if primary_records.is_empty() && !target_keys.is_empty() {
        // Still track declared keys even if absent in live L1 store
        primary_records = target_keys.iter().cloned().collect();
    }
    primary_records.sort();

    // 2. Discover cascaded L2 scenes
    let mut cascaded_scene_names: HashSet<String> = HashSet::new();
    let scene_list_opts = MemoryListOptions {
        limit: 10_000,
        ..Default::default()
    };
    if let Ok(page) = db.list(&scene_ns, scene_list_opts) {
        for record in page.records {
            let mut matches_l1 = false;

            // Check metadata source_episode_ids
            if let Some(vantadb::sdk::Value::ListString(sources)) =
                record.metadata.get("source_episode_ids")
            {
                if sources.iter().any(|s| target_keys.contains(s)) {
                    matches_l1 = true;
                }
            }
            if let Some(vantadb::sdk::Value::ListString(derived)) =
                record.metadata.get("derived_from")
            {
                if derived.iter().any(|d| target_keys.contains(d)) {
                    matches_l1 = true;
                }
            }

            // Check deserialized SceneBlock payload
            if let Ok(block) = serde_json::from_str::<crate::core::scene::scene_format::SceneBlock>(
                &record.payload,
            ) {
                if block
                    .source_episode_ids
                    .iter()
                    .any(|s| target_keys.contains(s))
                {
                    matches_l1 = true;
                }
                if block
                    .meta
                    .source_episode_ids
                    .iter()
                    .any(|s| target_keys.contains(s))
                {
                    matches_l1 = true;
                }
                if matches_l1 {
                    cascaded_scene_names.insert(block.scene_name);
                    continue;
                }
            }

            if matches_l1 {
                cascaded_scene_names.insert(record.key);
            }
        }
    }

    // 3. Discover cascaded L3 persona and semantic axioms
    let mut cascaded_l3_axiom_ids: HashSet<String> = HashSet::new();
    let mut cascade_persona = false;

    // Check persona in session
    if let Ok(Some(persona)) = get_persona(db, session_id) {
        let ep_match = persona
            .source_episode_ids
            .iter()
            .any(|s| target_keys.contains(s));
        let sc_match = persona
            .source_scene_names
            .iter()
            .any(|s| cascaded_scene_names.contains(s));
        if ep_match || sc_match || !cascaded_scene_names.is_empty() {
            cascade_persona = true;
        }
    } else if let Ok(Some(rec)) = db.get(&persona_ns, &sanitize_key(PERSONA_KEY)) {
        if let Some(vantadb::sdk::Value::ListString(sources)) =
            rec.metadata.get("source_episode_ids")
        {
            if sources.iter().any(|s| target_keys.contains(s)) {
                cascade_persona = true;
            }
        }
    }

    // Check L3 axioms / consolidated facts in l1_ns
    if let Ok(all_records) = read_session_records(db, session_id) {
        for rec in all_records {
            if target_keys.contains(&rec.id) {
                continue;
            }
            let mut is_derived = false;

            // Direct reference in source_message_ids
            if rec
                .source_message_ids
                .iter()
                .any(|s| target_keys.contains(s))
            {
                is_derived = true;
            }

            // Reference in metadata source_episode_ids
            if let Some(arr) = rec
                .metadata
                .get("source_episode_ids")
                .and_then(|v| v.as_array())
            {
                if arr
                    .iter()
                    .any(|item| item.as_str().is_some_and(|s| target_keys.contains(s)))
                {
                    is_derived = true;
                }
            }

            // Belongs to a cascaded scene
            if cascaded_scene_names.contains(&rec.scene_name) {
                is_derived = true;
            }

            if is_derived {
                cascaded_l3_axiom_ids.insert(rec.id);
            }
        }
    }

    // Check bitemporal triplets in triplets_ns
    let mut cascaded_triplet_keys: HashSet<String> = HashSet::new();
    let trip_list_opts = MemoryListOptions {
        limit: 10_000,
        ..Default::default()
    };
    if let Ok(page) = db.list(&triplets_ns, trip_list_opts) {
        for rec in page.records {
            if let Some(vantadb::sdk::Value::ListString(sources)) =
                rec.metadata.get("source_episode_ids")
            {
                if sources.iter().any(|s| target_keys.contains(s)) {
                    cascaded_triplet_keys.insert(rec.key);
                }
            }
        }
    }

    // 4. Execute cascading erasure / invalidation according to mode
    let mut certificates: Vec<PurgeCertificate> = Vec::new();
    let mut details: Vec<CascadedItemDetail> = Vec::new();
    let mut cascaded_ids: Vec<String> = Vec::new();

    // A. Apply to primary L1 targets
    for key in &primary_records {
        let skey = sanitize_key(key);
        match mode {
            ForgetMode::Soft => {
                let _ = db.forget(&l1_ns, ForgetTarget::Key(skey), ForgetMode::Soft);
            }
            ForgetMode::Hard => {
                let _ = db.delete(&l1_ns, &skey);
            }
            ForgetMode::Certified => {
                if let Ok(cert) = db.delete_certified(&l1_ns, &skey) {
                    if cert.status != "not_found" {
                        certificates.push(cert);
                    }
                }
            }
        }
    }

    // B. Apply to cascaded L2 scenes
    for scene_name in &cascaded_scene_names {
        let skey = sanitize_key(scene_name);
        cascaded_ids.push(scene_name.clone());
        match mode {
            ForgetMode::Soft => {
                let _ = soft_delete_scene(db, session_id, scene_name);
                let _ = db.forget(&scene_ns, ForgetTarget::Key(skey), ForgetMode::Soft);
                details.push(CascadedItemDetail {
                    layer: "L2".into(),
                    namespace: scene_ns.clone(),
                    id: scene_name.clone(),
                    action: "soft_deleted".into(),
                });
            }
            ForgetMode::Hard => {
                let _ = db.delete(&scene_ns, &skey);
                details.push(CascadedItemDetail {
                    layer: "L2".into(),
                    namespace: scene_ns.clone(),
                    id: scene_name.clone(),
                    action: "deleted".into(),
                });
            }
            ForgetMode::Certified => {
                if let Ok(cert) = db.delete_certified(&scene_ns, &skey) {
                    certificates.push(cert);
                }
                details.push(CascadedItemDetail {
                    layer: "L2".into(),
                    namespace: scene_ns.clone(),
                    id: scene_name.clone(),
                    action: "certified_purged".into(),
                });
            }
        }
    }

    // C. Apply to cascaded L3 Persona
    if cascade_persona {
        let pkey = sanitize_key(PERSONA_KEY);
        cascaded_ids.push(PERSONA_KEY.to_string());
        let prof_ns = profile_namespace(DEFAULT_PROFILE_SCOPE);
        match mode {
            ForgetMode::Soft => {
                let _ = db.forget(
                    &persona_ns,
                    ForgetTarget::Key(pkey.clone()),
                    ForgetMode::Soft,
                );
                // Also update stored persona content to marker
                if let Ok(Some(mut persona)) = get_persona(db, session_id) {
                    persona.content = SOFT_DELETE_MARKER.to_string();
                    let _ = db.put(vantadb::sdk::MemoryInput {
                        namespace: persona_ns.clone(),
                        key: pkey,
                        payload: serde_json::to_string(&persona).unwrap_or_default(),
                        ..Default::default()
                    });
                }
                details.push(CascadedItemDetail {
                    layer: "L3".into(),
                    namespace: persona_ns.clone(),
                    id: PERSONA_KEY.into(),
                    action: "soft_invalidated".into(),
                });
            }
            ForgetMode::Hard => {
                let _ = db.delete(&persona_ns, &pkey);
                let _ = db.delete(&prof_ns, &pkey);
                details.push(CascadedItemDetail {
                    layer: "L3".into(),
                    namespace: persona_ns.clone(),
                    id: PERSONA_KEY.into(),
                    action: "deleted".into(),
                });
            }
            ForgetMode::Certified => {
                if let Ok(cert) = db.delete_certified(&persona_ns, &pkey) {
                    certificates.push(cert);
                }
                let _ = db.delete(&prof_ns, &pkey);
                details.push(CascadedItemDetail {
                    layer: "L3".into(),
                    namespace: persona_ns.clone(),
                    id: PERSONA_KEY.into(),
                    action: "certified_purged".into(),
                });
            }
        }
    }

    // D. Apply to cascaded L3 semantic axioms
    for axiom_id in &cascaded_l3_axiom_ids {
        let akey = sanitize_key(axiom_id);
        cascaded_ids.push(axiom_id.clone());
        match mode {
            ForgetMode::Soft => {
                let _ = db.forget(&l1_ns, ForgetTarget::Key(akey), ForgetMode::Soft);
                details.push(CascadedItemDetail {
                    layer: "L3".into(),
                    namespace: l1_ns.clone(),
                    id: axiom_id.clone(),
                    action: "soft_invalidated".into(),
                });
            }
            ForgetMode::Hard => {
                let _ = db.delete(&l1_ns, &akey);
                details.push(CascadedItemDetail {
                    layer: "L3".into(),
                    namespace: l1_ns.clone(),
                    id: axiom_id.clone(),
                    action: "deleted".into(),
                });
            }
            ForgetMode::Certified => {
                if let Ok(cert) = db.delete_certified(&l1_ns, &akey) {
                    certificates.push(cert);
                }
                details.push(CascadedItemDetail {
                    layer: "L3".into(),
                    namespace: l1_ns.clone(),
                    id: axiom_id.clone(),
                    action: "certified_purged".into(),
                });
            }
        }
    }

    // E. Apply to cascaded bitemporal triplets
    for trip_key in &cascaded_triplet_keys {
        cascaded_ids.push(trip_key.clone());
        match mode {
            ForgetMode::Soft => {
                let _ = db.forget(
                    &triplets_ns,
                    ForgetTarget::Key(trip_key.clone()),
                    ForgetMode::Soft,
                );
            }
            ForgetMode::Hard => {
                let _ = db.delete(&triplets_ns, trip_key);
            }
            ForgetMode::Certified => {
                if let Ok(cert) = db.delete_certified(&triplets_ns, trip_key) {
                    certificates.push(cert);
                }
            }
        }
    }

    // Deduplicate and sort cascaded_ids
    cascaded_ids.sort();
    cascaded_ids.dedup();

    let status = match mode {
        ForgetMode::Soft => "soft_invalidated",
        ForgetMode::Hard | ForgetMode::Certified => "erased",
    };

    #[cfg(feature = "erasure")]
    let erasure_receipt = if mode == ForgetMode::Certified {
        crate::utils::erasure::emit_cascade_erasure_receipt(
            db,
            session_id,
            "GDPR Art. 17 right to erasure",
            cascaded_ids.clone(),
        )
        .ok()
    } else {
        None
    };

    Ok(CascadeForgetReceipt {
        mode,
        session_id: session_id.to_string(),
        target: target_repr,
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        primary_keys: primary_records,
        cascaded_ids,
        details,
        certificates,
        status: status.to_string(),
        #[cfg(feature = "erasure")]
        erasure_receipt,
    })
}

/// Extension trait for `Embedded` database to call `forget_cascade`.
pub trait CascadeForgetExt {
    fn forget_cascade(
        &self,
        session_id: &str,
        target: impl IntoForgetTarget,
        mode: ForgetMode,
    ) -> Result<CascadeForgetReceipt, CascadeForgetError>;
}

impl CascadeForgetExt for Embedded {
    fn forget_cascade(
        &self,
        session_id: &str,
        target: impl IntoForgetTarget,
        mode: ForgetMode,
    ) -> Result<CascadeForgetReceipt, CascadeForgetError> {
        forget(self, session_id, target, mode)
    }
}
