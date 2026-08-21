//! Wiki store: lifecycle record + managed pages over `InternalMetadata`.

use serde::{Deserialize, Serialize};
use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{ChainedError, Result, VantaError};
use crate::storage::StorageEngine;

use super::state::WikiState;
use super::SYNC_ERROR_MAX_CHARS;

/// A wiki space lifecycle record.
///
/// `run_id` identifies the active/last build (one per transition into
/// `processing`, TDAM wiki-service.ts:1026); `version` is bumped on every
/// write so consumers can detect concurrent modification (MEM-06 optimistic
/// lock convention).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wiki {
    pub namespace: String,
    pub slug: String,
    pub state: WikiState,
    pub run_id: Option<String>,
    /// Why the last build failed (`failed` only), truncated to 500 chars.
    pub sync_error: Option<String>,
    pub version: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

/// A managed wiki page, addressed by canonical path.
///
/// Managed pages are always `locked: true` — external writers must not edit
/// them (TDAM frontmatter injection, wiki-service.ts:1164-1183).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiPage {
    pub namespace: String,
    pub wiki_slug: String,
    /// Canonical path, e.g. `wiki/person/alice-smith.md`.
    pub path: String,
    pub page_type: String,
    pub title: String,
    pub locked: bool,
    pub content: String,
    pub updated_at_ms: u64,
}

// ── WikiStore ──

/// Store for wiki spaces and their managed pages, backed by a
/// [`StorageEngine`] (InternalMetadata partition, same pattern as
/// [`crate::entity::EntityStore`] / MEM-12 scene store).
pub struct WikiStore<'a> {
    engine: &'a StorageEngine,
}

impl<'a> WikiStore<'a> {
    /// Wrap a storage engine reference.
    pub fn new(engine: &'a StorageEngine) -> Self {
        Self { engine }
    }

    /// Create a wiki space in the `pending` state (initial build queued).
    ///
    /// Errors with [`VantaError::ExecutionConflict`] if the slug already
    /// exists in the namespace.
    pub fn create(&self, namespace: &str, slug: &str) -> Result<Wiki> {
        validate_scope(namespace, slug)?;
        if self.get(namespace, slug)?.is_some() {
            return Err(VantaError::ExecutionConflict {
                resource: format!("wiki:{namespace}:{slug}"),
                detail: "wiki already exists".into(),
            });
        }
        let now = now_ms();
        let wiki = Wiki {
            namespace: namespace.to_string(),
            slug: slug.to_string(),
            state: WikiState::Pending,
            run_id: None,
            sync_error: None,
            version: 1,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.put_wiki(&wiki)?;
        Ok(wiki)
    }

    /// Retrieve a wiki space by scope, or `None` when absent.
    pub fn get(&self, namespace: &str, slug: &str) -> Result<Option<Wiki>> {
        validate_scope(namespace, slug)?;
        match self.engine.get_from_partition(
            BackendPartition::InternalMetadata,
            &wiki_key(namespace, slug),
        )? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("wiki", e))),
            None => Ok(None),
        }
    }

    /// Delete a wiki and cascade-delete every managed page. Returns `true`
    /// when the wiki existed (TDAM cascade delete, wiki-service.ts:827-859).
    // ponytail: read-then-batch-delete without cross-key txn; single-process
    // callers (D27 worker único). Upgrade path: engine-level batch atomicity.
    pub fn delete(&self, namespace: &str, slug: &str) -> Result<bool> {
        validate_scope(namespace, slug)?;
        let existed = self.get(namespace, slug)?.is_some();
        if !existed {
            return Ok(false);
        }
        let mut ops = vec![BackendWriteOp::Delete {
            partition: BackendPartition::InternalMetadata,
            key: wiki_key(namespace, slug),
        }];
        for (key, _) in self.engine.scan_partition_prefix(
            BackendPartition::InternalMetadata,
            page_prefix(namespace, slug).as_bytes(),
        )? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::InternalMetadata,
                key,
            });
        }
        self.engine.write_backend_batch(ops)?;
        Ok(true)
    }
}

// ── state transitions ──
//
// Every transition is a read-validate-write CAS against the expected source
// state, bumping `version` (MEM-06 optimistic-lock convention).
// ponytail: no cross-key txn in the engine; single-process callers (D27
// worker único). Upgrade path: engine-level txn if multi-writer appears.

impl WikiStore<'_> {
    /// Request a (re-)ingest: `ready|failed → pending`. Rejected with
    /// [`VantaError::ExecutionConflict`] while a build is queued/running
    /// (TDAM 409-busy, wiki-service.ts:272-288).
    pub fn request_ingest(&self, namespace: &str, slug: &str) -> Result<Wiki> {
        validate_scope(namespace, slug)?;
        let mut wiki = require(self.engine, namespace, slug)?;
        if wiki.state.is_busy() {
            return Err(wiki.state.busy_error(namespace, slug));
        }
        wiki.state = WikiState::Pending;
        wiki.run_id = None;
        wiki.sync_error = None;
        wiki.version += 1;
        wiki.updated_at_ms = now_ms();
        self.put_wiki(&wiki)?;
        Ok(wiki)
    }

    /// Begin the queued build: `pending → processing`, assigning a fresh
    /// `run_id` (one per build, TDAM wiki-service.ts:1026). Any other source
    /// state is rejected.
    pub fn begin_processing(&self, namespace: &str, slug: &str) -> Result<Wiki> {
        validate_scope(namespace, slug)?;
        let mut wiki = require(self.engine, namespace, slug)?;
        if wiki.state != WikiState::Pending {
            return Err(VantaError::ExecutionConflict {
                resource: format!("wiki:{namespace}:{slug}"),
                detail: format!(
                    "cannot start build from state `{}`; expected `pending`",
                    wiki.state
                ),
            });
        }
        wiki.state = WikiState::Processing;
        wiki.run_id = Some(new_run_id());
        wiki.version += 1;
        wiki.updated_at_ms = now_ms();
        self.put_wiki(&wiki)?;
        Ok(wiki)
    }

    /// Complete the build identified by `run_id`: `processing → ready`.
    /// A stale `run_id` (packet from an older build) is rejected — MEM-31
    /// late-packet guard.
    pub fn complete(&self, namespace: &str, slug: &str, run_id: &str) -> Result<Wiki> {
        validate_scope(namespace, slug)?;
        let mut wiki = require(self.engine, namespace, slug)?;
        expect_processing(&wiki, namespace, slug, run_id)?;
        wiki.state = WikiState::Ready;
        wiki.sync_error = None;
        wiki.version += 1;
        wiki.updated_at_ms = now_ms();
        self.put_wiki(&wiki)?;
        Ok(wiki)
    }

    /// Fail the build identified by `run_id`: `processing → failed`,
    /// storing `sync_error` truncated to 500 chars.
    pub fn fail(
        &self,
        namespace: &str,
        slug: &str,
        run_id: &str,
        sync_error: &str,
    ) -> Result<Wiki> {
        validate_scope(namespace, slug)?;
        let mut wiki = require(self.engine, namespace, slug)?;
        expect_processing(&wiki, namespace, slug, run_id)?;
        wiki.state = WikiState::Failed;
        wiki.sync_error = Some(truncate_sync_error(sync_error));
        wiki.version += 1;
        wiki.updated_at_ms = now_ms();
        self.put_wiki(&wiki)?;
        Ok(wiki)
    }
}

/// Shared guard: wiki must be `processing` under exactly this `run_id`.
fn expect_processing(wiki: &Wiki, namespace: &str, slug: &str, run_id: &str) -> Result<()> {
    if wiki.state != WikiState::Processing {
        return Err(VantaError::ExecutionConflict {
            resource: format!("wiki:{namespace}:{slug}"),
            detail: format!(
                "build completion for `{}` requires state `processing`",
                wiki.state
            ),
        });
    }
    match &wiki.run_id {
        Some(current) if current == run_id => Ok(()),
        other => Err(VantaError::ExecutionConflict {
            resource: format!("wiki:{namespace}:{slug}"),
            detail: format!(
                "stale run_id `{run_id}` (active: {})",
                other.as_deref().unwrap_or("none")
            ),
        }),
    }
}

/// Fresh build id (`wikirun-{ts}{rand}`, base36) — no new dependency; TDAM
/// uses randomUUID but only uniqueness-per-build matters here.
fn new_run_id() -> String {
    crate::entity::generate_id("wikirun")
}

// ── managed pages ──

/// Canonical page path from type + title (dedup key): lowercased title
/// slugified into `wiki/{type}/{slug}.md` (TDAM wiki-service.ts:392-410).
pub fn canonical_path(page_type: &str, title: &str) -> String {
    let slug: String = title
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() {
                c
            } else {
                '-'
            }
        })
        .collect();
    // collapse runs of '-' and trim edges
    let mut out = String::with_capacity(slug.len());
    let mut prev_dash = true; // trims leading dashes too
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                out.push('-');
            }
            prev_dash = true;
        } else {
            out.push(c);
            prev_dash = false;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    format!("wiki/{}/{}.md", page_type.trim().to_ascii_lowercase(), out)
}

impl WikiStore<'_> {
    /// Insert or replace a managed page. The storage path is the canonical
    /// `type + title` path (dedup: same type+title overwrites), and the page
    /// is stored `locked: true`.
    pub fn put_page(
        &self,
        namespace: &str,
        slug: &str,
        page_type: &str,
        title: &str,
        content: &str,
    ) -> Result<WikiPage> {
        validate_scope(namespace, slug)?;
        validate_component("page_type", page_type)?;
        validate_component("title", title)?;
        require(self.engine, namespace, slug)?;
        let path = canonical_path(page_type, title);
        let page = WikiPage {
            namespace: namespace.to_string(),
            wiki_slug: slug.to_string(),
            path,
            page_type: page_type.to_string(),
            title: title.to_string(),
            locked: true,
            content: content.to_string(),
            updated_at_ms: now_ms(),
        };
        let bytes = serde_json::to_vec(&page)
            .map_err(|e| VantaError::serialization(ChainedError::with_source("wiki", e)))?;
        self.engine.put_to_partition(
            BackendPartition::InternalMetadata,
            &page_key(namespace, slug, &page.path),
            &bytes,
        )?;
        Ok(page)
    }

    /// Retrieve a managed page by canonical path.
    pub fn get_page(&self, namespace: &str, slug: &str, path: &str) -> Result<Option<WikiPage>> {
        validate_scope(namespace, slug)?;
        validate_component("path", path)?;
        match self.engine.get_from_partition(
            BackendPartition::InternalMetadata,
            &page_key(namespace, slug, path),
        )? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| VantaError::serialization(ChainedError::with_source("wiki", e))),
            None => Ok(None),
        }
    }

    /// List every managed page of a wiki, ordered by canonical path.
    pub fn list_pages(&self, namespace: &str, slug: &str) -> Result<Vec<WikiPage>> {
        validate_scope(namespace, slug)?;
        let rows = self.engine.scan_partition_prefix(
            BackendPartition::InternalMetadata,
            page_prefix(namespace, slug).as_bytes(),
        )?;
        let mut pages: Vec<WikiPage> = Vec::with_capacity(rows.len());
        for (_, bytes) in rows {
            pages
                .push(serde_json::from_slice(&bytes).map_err(|e| {
                    VantaError::serialization(ChainedError::with_source("wiki", e))
                })?);
        }
        pages.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(pages)
    }

    /// Delete one managed page by canonical path. Returns `true` when it existed.
    pub fn delete_page(&self, namespace: &str, slug: &str, path: &str) -> Result<bool> {
        validate_scope(namespace, slug)?;
        validate_component("path", path)?;
        let existed = self.get_page(namespace, slug, path)?.is_some();
        if !existed {
            return Ok(false);
        }
        self.engine
            .write_backend_batch(vec![BackendWriteOp::Delete {
                partition: BackendPartition::InternalMetadata,
                key: page_key(namespace, slug, path),
            }])?;
        Ok(true)
    }
}

// ── shared internals ──

/// Serialize and persist a wiki record (bumps nothing — callers set fields).
pub(super) fn persist(engine: &StorageEngine, wiki: &Wiki) -> Result<()> {
    let bytes = serde_json::to_vec(wiki)
        .map_err(|e| VantaError::serialization(ChainedError::with_source("wiki", e)))?;
    engine.put_to_partition(
        BackendPartition::InternalMetadata,
        &wiki_key(&wiki.namespace, &wiki.slug),
        &bytes,
    )
}

/// Load a wiki or error when absent (shared by transitions/pages).
pub(super) fn require(engine: &StorageEngine, namespace: &str, slug: &str) -> Result<Wiki> {
    match WikiStore::new(engine).get(namespace, slug)? {
        Some(wiki) => Ok(wiki),
        None => Err(VantaError::NotFound {
            kind: "wiki".into(),
            id: format!("{namespace}:{slug}"),
        }),
    }
}

/// Truncate a sync error to the 500-char budget (setter-side guard).
pub(super) fn truncate_sync_error(err: &str) -> String {
    err.chars().take(SYNC_ERROR_MAX_CHARS).collect()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl WikiStore<'_> {
    pub(crate) fn put_wiki(&self, wiki: &Wiki) -> Result<()> {
        persist(self.engine, wiki)
    }
}

// ── keys & validation ──

/// Key for a wiki lifecycle record.
fn wiki_key(namespace: &str, slug: &str) -> Vec<u8> {
    format!("wiki:{{{ns}}}::{{{slug}}}", ns = namespace, slug = slug).into_bytes()
}

/// Key prefix covering every managed page of a wiki.
fn page_prefix(namespace: &str, slug: &str) -> String {
    format!(
        "wiki:{{{ns}}}::{{{slug}}}:page:",
        ns = namespace,
        slug = slug
    )
}

/// Key for a single managed page record.
pub(super) fn page_key(namespace: &str, slug: &str, path: &str) -> Vec<u8> {
    format!(
        "wiki:{{{ns}}}::{{{slug}}}:page:{path}",
        ns = namespace,
        slug = slug
    )
    .into_bytes()
}

fn validate_component(field: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(VantaError::InvalidInput(format!(
            "{field} must be non-empty"
        )));
    }
    if value.len() > 512 {
        return Err(VantaError::InvalidInput(format!(
            "{field} must be at most 512 bytes"
        )));
    }
    if value.as_bytes().contains(&0) {
        return Err(VantaError::InvalidInput(format!(
            "{field} must not contain NUL bytes"
        )));
    }
    if value.contains(['{', '}', ':']) {
        return Err(VantaError::InvalidInput(format!(
            "{field} must not contain '{{', '}}' or ':'"
        )));
    }
    Ok(())
}

/// Validate a namespace/slug pair (key delimiters forbidden).
pub(super) fn validate_scope(namespace: &str, slug: &str) -> Result<()> {
    validate_component("namespace", namespace)?;
    validate_component("slug", slug)
}
