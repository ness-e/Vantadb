//! MCP tool handlers for the `wiki_*` tools (MEM-33).
//!
//! Four query-only tools over the core [`vantadb::wiki::WikiStore`] (MEM-28,
//! commit 0c3a9dcf) — thin read-only wrappers, all semantics live in the
//! core. Every tool refuses to answer while the wiki lifecycle state is not
//! `ready` (D27: `pending → processing → ready | failed`), surfacing the
//! current state so the caller can poll or retry later.
//!
//! Tool↔primitive mapping (documented per plan pre-mortem 1):
//!
//! | Tool | Primitive | Notes |
//! |---|---|---|
//! | `wiki_search` | `list_pages()` + local BM25-style scan+rank | pages are
//!   serde records in the `InternalMetadata` partition, invisible to the
//!   core's memory-record text_index; MEM-30 built no separate index →
//!   scan+rank (k1=1.5, b=0.75), title terms weighted ×5 (TDAM
//!   manager.ts:381-391 parity). NO SQLite FTS5. |
//! | `wiki_read` | `get_page()` | `locked:true` surfaced as visible metadata |
//! | `wiki_list` | `list_pages()` | ordered by canonical path |
//! | `wiki_graph` | BFS multi-hop over `[[wikilink]]` edges extracted from
//!   page content, hard cap 200 visited nodes (TDAM graph-search.ts:38) | |

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use vantadb::storage::StorageEngine;
use vantadb::wiki::{WikiState, WikiStore};

/// Hard cap on visited nodes for `wiki_graph` (TDAM graph-search.ts:38).
const GRAPH_MAX_NODES: usize = 200;
/// Default hop depth for `wiki_graph`.
const GRAPH_DEFAULT_HOPS: usize = 2;
/// Hard cap on hop depth for `wiki_graph`.
const GRAPH_MAX_HOPS: usize = 10;
/// Default result count for `wiki_search`.
const SEARCH_DEFAULT_K: usize = 10;

/// Tool definitions for `tools/list` (MEM-33).
pub(crate) fn wiki_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "wiki_search",
            "description": "BM25-style full-text search over the pages of a ready wiki (title terms weighted x5). Read-only.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Wiki namespace" },
                    "slug": { "type": "string", "description": "Wiki slug" },
                    "query": { "type": "string", "description": "Text query" },
                    "top_k": { "type": "number", "description": "Max results, default 10" }
                },
                "required": ["namespace", "slug", "query"]
            }
        }),
        json!({
            "name": "wiki_read",
            "description": "Reads a single wiki page by canonical path; managed pages carry locked:true as visible metadata. Read-only.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Wiki namespace" },
                    "slug": { "type": "string", "description": "Wiki slug" },
                    "path": { "type": "string", "description": "Canonical page path, e.g. wiki/person/alice-smith.md" }
                },
                "required": ["namespace", "slug", "path"]
            }
        }),
        json!({
            "name": "wiki_list",
            "description": "Lists every page of a wiki ordered by canonical path. Read-only.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Wiki namespace" },
                    "slug": { "type": "string", "description": "Wiki slug" }
                },
                "required": ["namespace", "slug"]
            }
        }),
        json!({
            "name": "wiki_graph",
            "description": "Breadth-first multi-hop traversal over the [[wikilink]] edges of a ready wiki, capped at 200 visited nodes. Read-only.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Wiki namespace" },
                    "slug": { "type": "string", "description": "Wiki slug" },
                    "root_path": { "type": "string", "description": "Canonical path of the root page" },
                    "max_hops": { "type": "number", "description": "Max hops, default 2, capped at 10" }
                },
                "required": ["namespace", "slug", "root_path"]
            }
        }),
    ]
}

/// Dispatch a `tools/call` for one of the `wiki_*` tools.
///
/// Param errors surface as JSON-RPC invalid-params; domain errors (missing
/// wiki/page, not-ready state) surface as `error_content` results the LLM can
/// self-correct — matching the existing MCP tool pattern (MEM-32 learning:
/// never propagate via `?`, the client would lose the message).
pub(crate) fn handle_wiki_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    _config: &McpConfig,
) -> Result<Value, Value> {
    match name {
        "wiki_search" => {
            let (namespace, slug) = required_scope(args)?;
            let query = required_str(args, "query")?;
            let top_k = args["top_k"].as_u64().map(|k| k as usize);
            with_store(storage, namespace, slug, |store| {
                let pages = store.list_pages(namespace, slug)?;
                let ranked = bm25_rank(&pages, query, top_k.unwrap_or(SEARCH_DEFAULT_K));
                Ok(text_content(serialize_content(&json!({
                    "query": query,
                    "total_pages": pages.len(),
                    "results": ranked,
                }))))
            })
        }

        "wiki_read" => {
            let (namespace, slug) = required_scope(args)?;
            let path = required_str(args, "path")?;
            with_store(storage, namespace, slug, |store| {
                match store.get_page(namespace, slug, path)? {
                    Some(page) => Ok(text_content(serialize_content(&json!({
                        "path": page.path,
                        "page_type": page.page_type,
                        "title": page.title,
                        // Managed pages are always locked:true (MEM-28) —
                        // surfaced so writers know they must not edit.
                        "locked": page.locked,
                        "updated_at_ms": page.updated_at_ms,
                        "content": page.content,
                    })))),
                    None => Ok(error_content(format!("Page not found: {path}"))),
                }
            })
        }

        "wiki_list" => {
            let (namespace, slug) = required_scope(args)?;
            with_store(storage, namespace, slug, |store| {
                let pages = store.list_pages(namespace, slug)?;
                let items: Vec<Value> = pages
                    .iter()
                    .map(|p| {
                        json!({
                            "path": p.path,
                            "page_type": p.page_type,
                            "title": p.title,
                            "locked": p.locked,
                            "updated_at_ms": p.updated_at_ms,
                        })
                    })
                    .collect();
                Ok(text_content(serialize_content(&json!({
                    "count": items.len(),
                    "pages": items,
                }))))
            })
        }

        "wiki_graph" => {
            let (namespace, slug) = required_scope(args)?;
            let root_path = required_str(args, "root_path")?;
            let max_hops = args["max_hops"]
                .as_u64()
                .map(|h| h as usize)
                .unwrap_or(GRAPH_DEFAULT_HOPS)
                .clamp(1, GRAPH_MAX_HOPS);
            with_store(storage, namespace, slug, |store| {
                let pages = store.list_pages(namespace, slug)?;
                if !pages.iter().any(|p| p.path == root_path) {
                    return Ok(error_content(format!(
                        "Page not found: {root_path} (use wiki_list for valid paths)"
                    )));
                }
                let graph = bfs_graph(&pages, root_path, max_hops);
                Ok(text_content(serialize_content(&graph)))
            })
        }

        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

// ── Shared guards & helpers ──────────────────────────────────────────────

/// Extract a required string argument as a JSON-RPC param error.
fn required_str<'a>(args: &'a Value, field: &'static str) -> Result<&'a str, Value> {
    args[field]
        .as_str()
        .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid '{field}'")).to_json())
}

fn required_scope(args: &Value) -> Result<(&str, &str), Value> {
    Ok((
        required_str(args, "namespace")?,
        required_str(args, "slug")?,
    ))
}

/// Open a `WikiStore` and run `f` only when the wiki exists AND is `ready`.
/// Any other state yields the clear "wiki not ready" domain error carrying
/// the current state (test e). Store errors become error_content too.
fn with_store(
    storage: &Arc<StorageEngine>,
    namespace: &str,
    slug: &str,
    f: impl FnOnce(&WikiStore<'_>) -> Result<Value, vantadb::VantaError>,
) -> Result<Value, Value> {
    let store = WikiStore::new(storage);
    let wiki = match store.get(namespace, slug) {
        Ok(Some(wiki)) => wiki,
        Ok(None) => {
            return Ok(domain_err(vantadb::VantaError::NotFound {
                kind: "wiki".into(),
                id: format!("{namespace}:{slug}"),
            }));
        }
        Err(e) => return Ok(domain_err(e)),
    };
    if wiki.state != WikiState::Ready {
        return Ok(domain_err(vantadb::VantaError::ExecutionConflict {
            resource: format!("wiki:{namespace}:{slug}"),
            detail: format!(
                "wiki not ready (state is `{}`); query tools require `ready`",
                wiki.state
            ),
        }));
    }
    match f(&store) {
        Ok(value) => Ok(value),
        Err(e) => Ok(domain_err(e)),
    }
}

/// Domain errors are self-correctable tool results (`error_content`), never
/// propagated protocol errors (MEM-32 learning).
fn domain_err(e: vantadb::VantaError) -> Value {
    // Ok(error_content), NOT Err: an Err payload loses the {content:[...]}
    // shape and the client LLM never sees the self-correctable message
    // (MEM-32 learning).
    error_content(e.to_string())
}

// ── BM25-style scan+rank ─────────────────────────────────────────────────

const K1: f32 = 1.5;
const B: f32 = 0.75;
/// Title-term weight ×5 (TDAM manager.ts:381-391 parity).
const TITLE_WEIGHT: f32 = 5.0;

fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Rank pages against `query` with BM25 over title(×5)+content terms.
fn bm25_rank(pages: &[vantadb::wiki::WikiPage], query: &str, top_k: usize) -> Vec<Value> {
    if pages.is_empty() {
        return Vec::new();
    }
    // Weighted term frequencies per doc + document lengths.
    let mut tfs: Vec<HashMap<String, u32>> = Vec::with_capacity(pages.len());
    let mut doc_lens = Vec::with_capacity(pages.len());
    for page in pages {
        let mut tf: HashMap<String, u32> = HashMap::new();
        for tok in tokens(&page.title) {
            *tf.entry(tok).or_insert(0) += TITLE_WEIGHT as u32;
        }
        let title_len = tf.values().sum::<u32>();
        for tok in tokens(&page.content) {
            *tf.entry(tok).or_insert(0) += 1;
        }
        doc_lens.push(title_len + tokens(&page.content).len() as u32);
        tfs.push(tf);
    }
    let avgdl = doc_lens.iter().sum::<u32>() as f32 / pages.len() as f32;
    let n = pages.len() as f32;

    let query_terms: Vec<String> = tokens(query);
    if query_terms.is_empty() {
        return Vec::new();
    }
    let df = |term: &str| tfs.iter().filter(|tf| tf.contains_key(term)).count();

    let mut scored: Vec<(usize, f32)> = Vec::new();
    for (i, tf) in tfs.iter().enumerate() {
        let dl = doc_lens[i] as f32;
        let mut score = 0.0f32;
        let mut matched = false;
        for term in &query_terms {
            let freq = match tf.get(term) {
                Some(f) if *f > 0 => *f as f32,
                _ => continue,
            };
            matched = true;
            let occurrences = df(term) as f32;
            let idf = ((n - occurrences + 0.5) / (occurrences + 0.5))
                .max(0.0)
                .ln_1p();
            score += idf * (freq * (K1 + 1.0)) / (freq + K1 * (1.0 - B + B * dl / avgdl));
        }
        if matched {
            scored.push((i, score));
        }
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k.max(1));

    scored
        .into_iter()
        .map(|(i, score)| {
            let page = &pages[i];
            json!({
                "path": page.path,
                "title": page.title,
                "score": score,
                "locked": page.locked,
            })
        })
        .collect()
}

// ── Wikilink graph BFS ───────────────────────────────────────────────────

/// Extract `[[wikilink]]` targets from markdown content (the only linking
/// form MEM-30's ingest produces — prompts.rs:25,56).
fn wikilinks(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        match after.find("]]") {
            Some(end) => {
                out.push(after[..end].trim().to_string());
                rest = &after[end + 2..];
            }
            None => break,
        }
    }
    out
}

/// Resolve a link target to a page path: case-insensitive match against the
/// page title first, then the canonical-path file stem.
fn resolve_link<'a>(by_title: &HashMap<String, &'a str>, link: &str) -> Option<&'a str> {
    let key = link.to_ascii_lowercase();
    if let Some(path) = by_title.get(&key) {
        return Some(path);
    }
    let stem = key.rsplit('/').next().unwrap_or("").trim_end_matches(".md");
    let candidate = match stem.rfind('.') {
        Some(dot) => &stem[..dot],
        None => stem,
    };
    by_title.get(candidate).copied()
}

/// Multi-hop BFS from `root_path` over wikilink edges, visiting at most
/// `GRAPH_MAX_NODES` nodes (TDAM graph-search.ts:38 DEFAULT_MAX_NODES=200).
fn bfs_graph(pages: &[vantadb::wiki::WikiPage], root_path: &str, max_hops: usize) -> Value {
    let by_title: HashMap<String, &str> = pages
        .iter()
        .map(|p| (p.title.to_ascii_lowercase(), p.path.as_str()))
        .collect();
    let content_of: HashMap<&str, &str> = pages
        .iter()
        .map(|p| (p.path.as_str(), p.content.as_str()))
        .collect();

    let mut hops: HashMap<&str, usize> = HashMap::new();
    let mut edges: Vec<Value> = Vec::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    hops.insert(root_path, 0);
    queue.push_back(root_path);

    while let Some(current) = queue.pop_front() {
        let depth = hops[current];
        if depth >= max_hops {
            continue;
        }
        let links = content_of
            .get(current)
            .map(|c| wikilinks(c))
            .unwrap_or_default();
        for link in &links {
            let Some(target) = resolve_link(&by_title, link) else {
                continue;
            };
            edges.push(json!({ "from": current, "link": link, "to": target }));
            if !hops.contains_key(target) && hops.len() < GRAPH_MAX_NODES {
                hops.insert(target, depth + 1);
                queue.push_back(target);
            }
        }
    }
    // ponytail: adjacency rebuilt from full page scan per visit; fine at
    // wiki scale (≤ few thousand pages). Upgrade path: prebuilt edge index.
    let nodes: Vec<Value> = pages
        .iter()
        .filter_map(|p| hops.get(p.path.as_str()).map(|d| (p, d)))
        .map(|(p, d)| json!({ "path": p.path, "title": p.title, "hop": d }))
        .collect();
    json!({
        "root": root_path,
        "max_hops": max_hops,
        "node_cap": GRAPH_MAX_NODES,
        "visited": hops.len(),
        "truncated": hops.len() >= GRAPH_MAX_NODES,
        "nodes": nodes,
        "edges": edges,
    })
}
