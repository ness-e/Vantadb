//! MCP resource handlers.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::*;
use serde_json::{json, Value};
use std::sync::Arc;
use vantadb::storage::StorageEngine;

// ── Resources handlers ────────────────────────────────────────────────────

/// Handle `resources/list`, returning the available operational-metrics and cognitive resources.
pub fn handle_resources_list() -> Result<Value, Value> {
    Ok(json!({
        "resources": [
            {
                "uri": "metrics://",
                "name": "Operational Metrics",
                "description": "Current operational metrics including memory usage, HNSW statistics, and storage information",
                "mimeType": "application/json"
            },
            {
                "uri": "schema://",
                "name": "Database Schema",
                "description": "Database schema information including HNSW configuration and text index version",
                "mimeType": "application/json"
            },
            {
                "uri": "vanta://brain/operating-manual",
                "name": "Brain Operating Manual",
                "description": "Cognitive lifecycle operating manual (L0 Working Context -> L1 Episodic Memory -> L2 Narrative Scenes -> L3 Dreams/Axioms)",
                "mimeType": "text/markdown"
            },
            {
                "uri": "vanta://tools/catalog",
                "name": "Tool Catalog",
                "description": "Compact catalog of all 88 VantaDB tools grouped by family with required parameters",
                "mimeType": "application/json"
            }
        ]
    }))
}

/// Operating manual markdown content explaining the cognitive lifecycle L0->L3.
const BRAIN_OPERATING_MANUAL: &str = r#"# VantaDB Brain Operating Manual

## Cognitive Lifecycle (L0 -> L3)

VantaDB organizes agent cognition into four hierarchical layers with deterministic progression:

### L0: Working Context
- **Tools**: `context_assemble`, `thread_send`, `thread_get`, `thread_list`
- **Purpose**: Manage active conversation state, token budget allocation, and turn curation.
- **Workflow**:
  1. At turn start, assemble relevant prompt context via `context_assemble` (combining recency, recall, and budget).
  2. At turn end, persist the completed turn to conversational history via `thread_send`.

### L1: Episodic Memory
- **Tools**: `memory_recall`, `memory_search`, `memory_put`, `memory_get`, `memory_forget`, `memory_supersede`
- **Purpose**: Fast, persistent storage and retrieval of factual and conversational episodes.
- **Workflow**:
  1. Recall-first rule: Always query `memory_recall` before answering questions about past sessions or user preferences.
  2. Store atomic facts with `memory_put`.
  3. Update decaying facts with `memory_supersede` to maintain temporal validity.
  4. Forget outdated, superseded, or erroneous records with `memory_forget` (soft, hard, or certified).

### L2: Narrative Scenes
- **Tools**: `vanta_scene` (or `scene_read`, `scene_list`, `scene_query`, `scene_write`, `scene_edit`)
- **Purpose**: Structured hierarchical narrative blocks capturing complex session state and milestone progression.
- **Workflow**: Group sequential turns into topical scenes ranked by heat and semantic relevance.

### L3: Axioms & Dreams
- **Tools**: `vanta_dream` (or `dream_list`, `dream_load`, `dream_consolidate`, `dream_promote`, `dream_discard`), `read_axioms`, `write_axiom`
- **Purpose**: Offline consolidation of episodic patterns into generalized heuristics and permanent persona principles.
- **Workflow**:
  1. Run `vanta_dream(action: "consolidate")` over accumulated episodic memory.
  2. Inspect candidate insights via `vanta_dream(action: "list")` and `load`.
  3. Promote validated insights into permanent axioms via `vanta_dream(action: "promote")`.

### Tool Access and Surface Profiles
- **Brain Profile (`VANTADB_MCP_PROFILE=brain`)**: Advertises exactly 12 compact tools (~2.8K schema tokens) while allowing execution of all 88 tools directly or via `vanta_dispatch`.
- **Universal Dispatch**: Any specialized tool (e.g. `code_explore`, `wiki_read`, `snapshot_create`) can be executed via `vanta_dispatch(tool: "<name>", arguments: {...})`.
"#;

/// Compact tool catalog JSON string.
fn get_tools_catalog_json() -> &'static str {
    r#"{"families":{"memory":[{"name":"memory_put","required":["namespace","key","payload"]},{"name":"memory_put_batch","required":["inputs"]},{"name":"memory_get","required":["namespace","key"]},{"name":"memory_delete","required":["namespace","key"]},{"name":"memory_delete_by_filter","required":["namespace","filters"]},{"name":"memory_list","required":["namespace"]},{"name":"memory_list_namespaces","required":[]},{"name":"memory_versions","required":["namespace","key"]},{"name":"memory_supersede","required":["namespace","key","new_key","new_payload"]},{"name":"memory_reinforce","required":["namespace","key","outcome"]},{"name":"memory_search","required":["namespace"]},{"name":"memory_recall","required":["query"]},{"name":"memory_forget","required":["namespace","target"]},{"name":"memory_verify_certificate","required":["certificate"]},{"name":"search_semantic","required":["namespace","query"]},{"name":"search_with_method","required":["namespace"]},{"name":"search_multi","required":["namespaces"]},{"name":"query_iql","required":["query"]}],"graph":[{"name":"vanta_graph","required":["action"]},{"name":"get_node_neighbors","required":["node_id"]},{"name":"graph_traverse","required":["node_id"]},{"name":"graph_page_rank","required":[]},{"name":"graph_degree_centrality","required":[]},{"name":"graph_topological_sort","required":[]},{"name":"graph_is_dag","required":[]},{"name":"remove_edge","required":["from","to","edge_type"]}],"threads":[{"name":"thread_create","required":[]},{"name":"thread_send","required":["thread_id","role","content"]},{"name":"thread_get","required":["thread_id"]},{"name":"thread_list","required":[]},{"name":"thread_delete","required":["thread_id"]},{"name":"thread_purge_expired","required":[]}],"scenes":[{"name":"vanta_scene","required":["action"]},{"name":"scene_read","required":["session_key","scene_name"]},{"name":"scene_list","required":["session_key"]},{"name":"scene_query","required":["session_key","query"]},{"name":"scene_write","required":["session_key","scene_name","content"]},{"name":"scene_edit","required":["session_key","scene_name","content"]}],"context":[{"name":"context_assemble","required":["messages"]},{"name":"inject_context","required":["namespace","key","content"]}],"dreams":[{"name":"vanta_dream","required":["action"]},{"name":"dream_list","required":[]},{"name":"dream_load","required":["dream_id"]},{"name":"dream_consolidate","required":["namespace"]},{"name":"dream_promote","required":["dream_id"]},{"name":"dream_discard","required":["dream_id"]}],"axioms":[{"name":"read_axioms","required":[]},{"name":"write_axiom","required":["key","value"]},{"name":"delete_axiom","required":["key"]}],"code":[{"name":"code_search","required":["query"]},{"name":"code_explore","required":["node_id"]},{"name":"code_index","required":["root_path"]},{"name":"code_callers","required":["node_id"]},{"name":"code_callees","required":["node_id"]},{"name":"code_impact","required":["node_id"]},{"name":"code_node","required":["node_id"]},{"name":"code_status","required":[]},{"name":"code_files","required":[]}],"wiki":[{"name":"wiki_search","required":["query"]},{"name":"wiki_read","required":["title"]},{"name":"wiki_list","required":[]},{"name":"wiki_graph","required":[]},{"name":"wiki_ingest","required":["root_path"]},{"name":"wiki_ingest_status","required":["job_id"]}],"skills":[{"name":"skill_list","required":[]},{"name":"skill_view","required":["name"]},{"name":"skill_create","required":["name","description","instructions"]},{"name":"skill_update","required":["name"]},{"name":"skill_patch","required":["name"]},{"name":"skill_files_write","required":["name","files"]},{"name":"skill_extract","required":["session_id"]}],"meta":[{"name":"vanta_dispatch","required":["tool"]}],"admin":[{"name":"capabilities","required":[]},{"name":"collection_stats","required":[]},{"name":"collection_delete","required":["namespace"]},{"name":"generate_snippet","required":["text"]},{"name":"embed_texts","required":["texts"]},{"name":"rehydrate","required":["namespace"]},{"name":"purge_expired","required":[]},{"name":"compact_wal","required":[]},{"name":"flush","required":[]},{"name":"compact_layout","required":[]},{"name":"vacuum","required":[]},{"name":"rebuild_index","required":[]},{"name":"audit_text_index","required":[]},{"name":"repair_text_index","required":[]},{"name":"list_snapshots","required":[]},{"name":"snapshot_create","required":[]},{"name":"snapshot_restore","required":["snapshot_id"]},{"name":"export","required":["namespace"]},{"name":"import","required":["namespace","data"]},{"name":"bulk_import_file","required":["namespace","file_path"]},{"name":"bulk_import_stream","required":["namespace","stream"]}]},"total_tools":88}"#
}

/// Handle `resources/read`, serving metrics, memory records or namespace listings.
pub fn handle_resources_read(
    params: &Option<Value>,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let p = params
        .as_ref()
        .ok_or_else(|| McpError::invalid_params("Missing params").to_json())?;

    let uri = p["uri"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'uri'").to_json())?;

    if uri == "metrics://" {
        let embedded = vantadb::Embedded::from_engine(storage.clone());
        let metrics_val = embedded.operational_metrics();
        let text = serialize_content(&metrics_val);
        Ok(json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}))
    } else if uri == "schema://" {
        let schema = build_schema_resource(storage);
        let text = serialize_content(&schema);
        Ok(json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}))
    } else if uri == "vanta://brain/operating-manual" {
        Ok(
            json!({"contents": [{"uri": uri, "mimeType": "text/markdown", "text": BRAIN_OPERATING_MANUAL}]}),
        )
    } else if uri == "vanta://tools/catalog" {
        Ok(
            json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": get_tools_catalog_json()}]}),
        )
    } else if uri.starts_with("memory://") {
        let path = uri.strip_prefix("memory://").unwrap_or("");
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() != 2 {
            return McpError::invalid_params(
                "Invalid memory URI format. Expected: memory://namespace/key",
            )
            .into_err();
        }
        let namespace = parts[0];
        let key = parts[1];

        if let Err(e) = validate_identifier(namespace, "namespace", config.max_namespace_length) {
            return e.into_err();
        }
        if let Err(e) = validate_identifier(key, "key", config.max_key_length) {
            return e.into_err();
        }

        let embedded = vantadb::Embedded::from_engine(storage.clone());
        match embedded.get(namespace, key) {
            Ok(Some(record)) => {
                let text = serialize_content(&record);
                Ok(
                    json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}),
                )
            }
            Ok(None) => McpError::invalid_params("Memory record not found").into_err(),
            Err(e) => McpError::from(e).into_err(),
        }
    } else if uri.starts_with("namespace://") {
        let namespace = uri.strip_prefix("namespace://").unwrap_or("");
        if namespace.is_empty() {
            return McpError::invalid_params(
                "Invalid namespace URI format. Expected: namespace://namespace",
            )
            .into_err();
        }
        if let Err(e) = validate_identifier(namespace, "namespace", config.max_namespace_length) {
            return e.into_err();
        }

        let embedded = vantadb::Embedded::from_engine(storage.clone());
        // MOD-11 (H7): page size aligned to the memory_list default instead
        // of a hardcoded 100. The resource returns the FIRST page with its
        // `next_cursor`; full pagination (up to config.max_list_limit) lives
        // in the memory_list tool, which accepts the cursor — documented in
        // SKILL.md § Available MCP Resources.
        let options = vantadb::sdk::MemoryListOptions {
            limit: config.default_list_limit,
            cursor: None,
            #[allow(deprecated)]
            filters: vantadb::sdk::MemoryMetadata::new(),
            filter_ops: None,
            exclude_superseded: false,
            // SCH-03 temporal params: not exposed here (SCH-07).
            as_of_ms: None,
            valid_window: None,
            include_quarantined: false,
            min_confidence: None,
        };
        match embedded.list(namespace, options) {
            Ok(page) => {
                let result = json!({
                    "namespace": namespace,
                    "records": page.records,
                    "next_cursor": page.next_cursor
                });
                let text = serialize_content(&result);
                Ok(
                    json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}),
                )
            }
            Err(e) => McpError::from(e).into_err(),
        }
    } else {
        McpError::method_not_found("Resource not found").into_err()
    }
}

/// Build the `schema://` resource payload: active HNSW configuration and
/// text index schema/tokenizer version.
pub(crate) fn build_schema_resource(storage: &Arc<StorageEngine>) -> Value {
    let index = storage.vec_index();
    // F3X: `vec_index()` is `dyn IndexPort` (no `.config` field). Rebuild the
    // serializable config from defaults + live routed getters. The engine only
    // ever creates default configs (`port_impl` factories use `CPIndex::new()`),
    // so this matches the old `.config.clone()` in every reachable state while
    // keeping the exact serde shape of `HnswConfig`.
    let live_config = vantadb::index::HnswConfig {
        distance_metric: index.distance_metric(),
        flat_threshold: index.flat_threshold(),
        index_type: index.index_kind(),
        ..Default::default()
    };
    let hnsw_config = serde_json::to_value(&live_config).unwrap_or_else(|_| json!({}));
    let text_spec = vantadb::TextIndexSpec::default();
    json!({
        "vector_index": {
            "type": "HNSW",
            "format_version": vantadb::VECTOR_INDEX_VERSION,
            "config": hnsw_config
        },
        "text_index": {
            "schema_version": text_spec.schema_version,
            "tokenizer": {
                "name": text_spec.tokenizer.name,
                "version": text_spec.tokenizer.version
            },
            "key_format": text_spec.key_format
        }
    })
}
