//! Gateway knowledge handlers: typed request/response layer for the MCP
//! scene tools `scene_read` / `scene_list` / `scene_query` (MEM-21, F4).
//!
//! Port of TDAM `MC/gateway/knowledge-handlers.ts` adapted to the record
//! store: there is no HTTP router here — these are pure handlers over
//! [`crate::core::scene::scene_index`] that a future MCP server wraps. The
//! TDAM pattern (validate input → store method → typed envelope) maps to
//! (boundary validation → scene_index call → `Result<Response, KnowledgeError>`).
//!
//! Soft-delete contract (MEM-14): deleted scenes are invisible through this
//! gateway — `scene_read` answers [`KnowledgeError::NotFound`], `scene_list`
//! and `scene_query` exclude them (parity with `list_scenes`).
//!
//! LLM-free by default (Principio 4): `scene_query` ranks by keyword-term
//! overlap; since MEM-47 an optional embedding hook upgrades it to D38
//! dual-pool ranking (keyword overlap ⊕ cosine similarity fused via RRF).
//! `None` (the default) runs the pre-MEM-47 keyword path byte-identically.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::record::l1_reader::{
    cosine_similarity, overlap_score, rrf_merge, MIN_COSINE_SIMILARITY,
};
use crate::core::record::l1_writer::EmbedFn;
use crate::core::scene::scene_format::SceneBlock;
use crate::core::scene::scene_index::{get_scene, list_scenes, read_blocks, SceneError};
use crate::core::scene::scene_tools::{validate_scene_name, SceneToolError};

/// Default result budget of [`scene_query`] when the request omits `top_k`
/// (TDAM recall `maxResults` default).
pub const DEFAULT_QUERY_TOP_K: usize = 5;

/// Errors surfaced by the gateway knowledge handlers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KnowledgeError {
    /// Underlying scene index / storage error.
    #[error("knowledge handler: {0}")]
    Scene(#[from] SceneError),
    /// Input rejected at the gateway boundary (empty session/keyword,
    /// invalid scene name).
    #[error("invalid knowledge request: {0}")]
    Invalid(String),
    /// The requested scene does not exist (or is soft-deleted — indistinguishable
    /// by design, parity with the TDAM 404 envelope).
    #[error("scene not found: {0}")]
    NotFound(String),
}

impl From<SceneToolError> for KnowledgeError {
    fn from(err: SceneToolError) -> Self {
        match err {
            SceneToolError::Scene(inner) => KnowledgeError::Scene(inner),
            SceneToolError::Invalid(msg) => KnowledgeError::Invalid(msg),
            SceneToolError::NotFound(name) => KnowledgeError::NotFound(name),
        }
    }
}

// ── scene_read ──

/// Request of [`scene_read`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneReadRequest {
    /// Session whose scene store is queried.
    pub session_key: String,
    /// Scene name (block key before sanitization).
    pub scene_name: String,
}

/// Response of [`scene_read`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneReadResponse {
    /// The live scene block.
    pub scene: SceneBlock,
}

/// Read one live scene by name. Missing **or soft-deleted** scenes answer
/// [`KnowledgeError::NotFound`] (soft-delete respected, MEM-14).
pub fn scene_read(
    db: &vantadb::sdk::VantaEmbedded,
    request: &SceneReadRequest,
) -> Result<SceneReadResponse, KnowledgeError> {
    validate_session(&request.session_key)?;
    validate_scene_name(&request.scene_name)?;
    match get_scene(db, &request.session_key, &request.scene_name)? {
        Some(block) if !block.is_deleted() => Ok(SceneReadResponse { scene: block }),
        _ => Err(KnowledgeError::NotFound(request.scene_name.clone())),
    }
}

// ── scene_list ──

/// Request of [`scene_list`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneListRequest {
    /// Session whose scene index is listed.
    pub session_key: String,
}

/// Response of [`scene_list`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneListResponse {
    /// Index entries, heat descending (parity with `list_scenes`; soft-deleted
    /// scenes excluded).
    pub scenes: Vec<crate::core::abstractions::SceneIndexEntry>,
}

/// List the scene index of a session (heat desc, soft-deleted excluded) —
/// direct parity with [`list_scenes`].
pub fn scene_list(
    db: &vantadb::sdk::VantaEmbedded,
    request: &SceneListRequest,
) -> Result<SceneListResponse, KnowledgeError> {
    validate_session(&request.session_key)?;
    Ok(SceneListResponse {
        scenes: list_scenes(db, &request.session_key)?,
    })
}

// ── scene_query ──

/// Request of [`scene_query`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneQueryRequest {
    /// Session whose scene store is searched.
    pub session_key: String,
    /// Free-text keyword query (tokenized via `significant_terms`).
    pub keyword: String,
    /// Maximum hits returned (default [`DEFAULT_QUERY_TOP_K`]).
    #[serde(default)]
    pub top_k: Option<usize>,
}

/// One hit of [`scene_query`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneQueryHit {
    /// Scene name (load it via `scene_read`).
    pub scene_name: String,
    /// Summary of the scene.
    pub summary: String,
    /// Heat score of the scene.
    pub heat: u32,
    /// Last update timestamp (ISO 8601).
    pub updated: String,
    /// Shared significant terms between the keyword and content+summary.
    pub score: usize,
}

/// Response of [`scene_query`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneQueryResponse {
    /// Hits ranked by overlap score descending (ties: heat descending);
    /// soft-deleted scenes excluded. Empty when nothing matches.
    pub hits: Vec<SceneQueryHit>,
}

/// Keyword search over the live scene blocks of a session.
///
/// Scores each block by term overlap between `keyword` and
/// `content + summary` (`overlap_score`), returns the top `top_k`.
///
/// `embed` (MEM-47/D38): optional embedding hook — when present, blocks are
/// ALSO ranked by cosine similarity between the embedded keyword and the
/// embedded block content (query-time embedding; blocks carry no persisted
/// vectors), fused with the keyword ranks via RRF. `None` keeps the
/// pre-MEM-47 behavior byte-identical. Hit `score` stays the keyword overlap
/// count in both cases.
///
/// # ponytail: O(N) embeds per query
/// Every live block is embedded on each query — fine at session-sized scene
/// counts. Upgrade path: persist block vectors at write time and query the
/// HNSW index instead.
pub fn scene_query(
    db: &vantadb::sdk::VantaEmbedded,
    request: &SceneQueryRequest,
    embed: Option<&EmbedFn>,
) -> Result<SceneQueryResponse, KnowledgeError> {
    validate_session(&request.session_key)?;
    let keyword = request.keyword.trim();
    if keyword.is_empty() {
        return Err(KnowledgeError::Invalid(
            "keyword must not be empty or whitespace-only".into(),
        ));
    }

    let query_vector = embed
        .and_then(|hook| hook(keyword))
        .filter(|v| !v.is_empty() && v.iter().any(|&x| x != 0.0));

    // (hit, keyword score, cosine similarity when semantic ranking applies)
    let mut hits: Vec<(SceneQueryHit, usize, Option<f32>)> = read_blocks(db, &request.session_key)?
        .into_iter()
        .filter(|block| !block.is_deleted())
        .filter_map(|block| {
            let haystack = format!("{} {}", block.meta.summary, block.content);
            let kw = overlap_score(&haystack, keyword);
            let sim = match (&query_vector, embed) {
                (Some(query), Some(hook)) => hook(&haystack)
                    .and_then(|haystack_vector| cosine_similarity(&haystack_vector, query))
                    .filter(|sim| *sim >= MIN_COSINE_SIMILARITY),
                _ => None,
            };
            (kw > 0 || sim.is_some()).then_some((
                SceneQueryHit {
                    scene_name: block.scene_name,
                    summary: block.meta.summary,
                    heat: block.meta.heat,
                    updated: block.meta.updated,
                    score: kw,
                },
                kw,
                sim,
            ))
        })
        .collect();

    if query_vector.is_none() {
        // Legacy ordering: overlap desc, then heat desc (pre-MEM-47 exact).
        hits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.heat.cmp(&a.0.heat)));
    } else {
        // D38 dual-pool: rank each pool independently, fuse via RRF so term
        // counts and similarities never compete directly.
        let mut by_keyword: Vec<&(SceneQueryHit, usize, Option<f32>)> = hits.iter().collect();
        by_keyword.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.heat.cmp(&a.0.heat)));
        let mut by_similarity: Vec<&(SceneQueryHit, usize, Option<f32>)> =
            hits.iter().filter(|h| h.2.is_some()).collect();
        by_similarity.sort_by(|a, b| {
            let (sa, sb) = (a.2.unwrap_or_default(), b.2.unwrap_or_default());
            sb.total_cmp(&sa).then_with(|| b.0.heat.cmp(&a.0.heat))
        });
        let keyword_ids: Vec<String> = by_keyword.iter().map(|h| h.0.scene_name.clone()).collect();
        let similarity_ids: Vec<String> = by_similarity
            .iter()
            .map(|h| h.0.scene_name.clone())
            .collect();
        let order = rrf_merge(&keyword_ids, &similarity_ids, usize::MAX);
        let mut ordered = Vec::with_capacity(hits.len());
        for name in &order {
            if let Some(pos) = hits.iter().position(|h| &h.0.scene_name == name) {
                ordered.push(hits.swap_remove(pos));
            }
        }
        hits = ordered;
    }

    hits.truncate(request.top_k.unwrap_or(DEFAULT_QUERY_TOP_K));
    Ok(SceneQueryResponse {
        hits: hits.into_iter().map(|(hit, _, _)| hit).collect(),
    })
}

// ── boundary validation ──

fn validate_session(session_key: &str) -> Result<(), KnowledgeError> {
    if session_key.trim().is_empty() {
        return Err(KnowledgeError::Invalid(
            "session_key must not be empty or whitespace-only".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::VantaConfig;
    use vantadb::storage::BackendKind;

    use crate::core::scene::scene_index::{soft_delete_scene, upsert_scene};

    fn open_db() -> vantadb::sdk::VantaEmbedded {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        vantadb::sdk::VantaEmbedded::open_with_config(config).expect("open in-memory db")
    }

    fn seed(db: &vantadb::sdk::VantaEmbedded) {
        upsert_scene(
            db,
            "sess-1",
            "deploy",
            "deployment notes",
            "we deploy with cargo and docker",
        )
        .expect("seed deploy");
        // Second write bumps heat to 2 (deterministic ordering in tests).
        upsert_scene(
            db,
            "sess-1",
            "deploy",
            "deployment notes",
            "we deploy with cargo and docker",
        )
        .expect("reseed deploy");
        upsert_scene(
            db,
            "sess-1",
            "research",
            "pricing research",
            "user researched vanta pricing tiers",
        )
        .expect("seed research");
        upsert_scene(db, "sess-1", "hot", "hot scene", "deploy pipeline hotfix").expect("seed hot");
    }

    // ── scene_read ──

    #[test]
    fn read_returns_live_block() {
        let db = open_db();
        seed(&db);
        let resp = scene_read(
            &db,
            &SceneReadRequest {
                session_key: "sess-1".into(),
                scene_name: "deploy".into(),
            },
        )
        .expect("read");
        assert_eq!(resp.scene.scene_name, "deploy");
        assert!(!resp.scene.is_deleted());
    }

    #[test]
    fn read_missing_scene_is_not_found() {
        let db = open_db();
        let err = scene_read(
            &db,
            &SceneReadRequest {
                session_key: "sess-1".into(),
                scene_name: "ghost".into(),
            },
        )
        .expect_err("not found");
        assert!(matches!(err, KnowledgeError::NotFound(name) if name == "ghost"));
    }

    #[test]
    fn read_soft_deleted_scene_is_not_found() {
        let db = open_db();
        seed(&db);
        soft_delete_scene(&db, "sess-1", "deploy").expect("delete");
        let err = scene_read(
            &db,
            &SceneReadRequest {
                session_key: "sess-1".into(),
                scene_name: "deploy".into(),
            },
        )
        .expect_err("deleted hidden");
        assert!(matches!(err, KnowledgeError::NotFound(_)));
    }

    #[test]
    fn read_rejects_invalid_input() {
        let db = open_db();
        for (session, name) in [("", "x"), ("  ", "x"), ("s", ""), ("s", "a\0b")] {
            let err = scene_read(
                &db,
                &SceneReadRequest {
                    session_key: session.into(),
                    scene_name: name.into(),
                },
            )
            .expect_err("invalid rejected");
            assert!(matches!(err, KnowledgeError::Invalid(_)), "{err}");
        }
    }

    // ── scene_list ──

    #[test]
    fn list_matches_index_parity_heat_desc_and_hides_deleted() {
        let db = open_db();
        seed(&db);
        soft_delete_scene(&db, "sess-1", "hot").expect("delete");

        let resp = scene_list(
            &db,
            &SceneListRequest {
                session_key: "sess-1".into(),
            },
        )
        .expect("list");
        let names: Vec<&str> = resp.scenes.iter().map(|e| e.filename.as_str()).collect();
        assert_eq!(names, vec!["deploy", "research"], "heat desc, deleted gone");

        // Parity with the underlying index function.
        let direct = list_scenes(&db, "sess-1").expect("direct");
        assert_eq!(resp.scenes, direct);
    }

    #[test]
    fn list_rejects_empty_session() {
        let db = open_db();
        let err = scene_list(
            &db,
            &SceneListRequest {
                session_key: "".into(),
            },
        )
        .expect_err("invalid");
        assert!(matches!(err, KnowledgeError::Invalid(_)));
    }

    // ── scene_query ──

    #[test]
    fn query_ranks_by_overlap_and_respects_top_k() {
        let db = open_db();
        seed(&db);

        let resp = scene_query(
            &db,
            &SceneQueryRequest {
                session_key: "sess-1".into(),
                keyword: "deploy".into(),
                top_k: Some(1),
            },
            None,
        )
        .expect("query");
        assert_eq!(resp.hits.len(), 1, "top_k truncates");
        // Both "deploy" and "hot" mention deploy; higher heat wins the tie.
        assert_eq!(resp.hits[0].scene_name, "deploy");

        let all = scene_query(
            &db,
            &SceneQueryRequest {
                session_key: "sess-1".into(),
                keyword: "deploy".into(),
                top_k: None,
            },
            None,
        )
        .expect("query default top_k");
        assert_eq!(all.hits.len(), 2, "default top_k=5 keeps both");
        assert_eq!(all.hits[0].scene_name, "deploy", "tie broken by heat desc");
        assert_eq!(all.hits[1].scene_name, "hot");
        assert_eq!(all.hits[0].score, 1);
    }

    #[test]
    fn query_scores_summary_and_content_and_excludes_deleted() {
        let db = open_db();
        seed(&db);
        soft_delete_scene(&db, "sess-1", "research").expect("delete");

        let resp = scene_query(
            &db,
            &SceneQueryRequest {
                session_key: "sess-1".into(),
                keyword: "pricing research".into(),
                top_k: None,
            },
            None,
        )
        .expect("query");
        assert!(
            resp.hits.iter().all(|h| h.scene_name != "research"),
            "deleted excluded: {:?}",
            resp.hits
        );
        assert!(resp.hits.is_empty(), "only matching scene was deleted");
    }

    #[test]
    fn query_no_match_yields_empty_hits() {
        let db = open_db();
        seed(&db);
        let resp = scene_query(
            &db,
            &SceneQueryRequest {
                session_key: "sess-1".into(),
                keyword: "kubernetes helm charts".into(),
                top_k: None,
            },
            None,
        )
        .expect("query");
        assert!(resp.hits.is_empty());
    }

    #[test]
    fn query_rejects_blank_keyword_and_empty_session() {
        let db = open_db();
        for (session, keyword) in [("sess-1", ""), ("sess-1", "   "), ("", "deploy")] {
            let err = scene_query(
                &db,
                &SceneQueryRequest {
                    session_key: session.into(),
                    keyword: keyword.into(),
                    top_k: None,
                },
                None,
            )
            .expect_err("invalid rejected");
            assert!(matches!(err, KnowledgeError::Invalid(_)), "{err}");
        }
    }

    // ── wire format ──

    #[test]
    fn requests_roundtrip_snake_case_json() {
        let req = SceneQueryRequest {
            session_key: "s".into(),
            keyword: "deploy".into(),
            top_k: Some(3),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("\"session_key\""), "{json}");
        assert!(json.contains("\"top_k\":3"), "{json}");
        let back: SceneQueryRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, req);

        // top_k omitted → default parse.
        let bare: SceneQueryRequest =
            serde_json::from_str(r#"{"session_key":"s","keyword":"k"}"#).expect("bare");
        assert_eq!(bare.top_k, None);
    }
}
