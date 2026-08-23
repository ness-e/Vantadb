use serde_json::{json, Value};
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn default_config() -> vantadb_mcp::McpConfig {
    vantadb_mcp::McpConfig::default()
}

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine");
    (dir, Arc::new(storage))
}

#[test]
fn test_mcp_initialize() {
    let res = handle_initialize();
    assert!(res.is_ok(), "handle_initialize should succeed");
    let val = res.unwrap();
    assert_eq!(val["protocolVersion"], "2024-11-05");
    assert_eq!(
        val["serverInfo"]["name"],
        vantadb::metadata::MCP_SERVER_INFO_NAME
    );
    assert!(
        val["capabilities"]["tools"].is_object(),
        "capabilities.tools should be an object"
    );
    assert!(
        val["capabilities"]["resources"].is_object(),
        "capabilities.resources should be an object"
    );
    assert!(
        val["capabilities"]["prompts"].is_object(),
        "capabilities.prompts should be an object"
    );
}

#[test]
fn test_mcp_resources_list() {
    let res = handle_resources_list();
    assert!(res.is_ok(), "handle_resources_list should succeed");
    let val = res.unwrap();
    let resources = val["resources"]
        .as_array()
        .expect("Expected resources array");

    let uris: Vec<&str> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();

    assert!(
        uris.contains(&"metrics://"),
        "resources should include metrics:// URI"
    );
    assert!(
        uris.contains(&"schema://"),
        "resources should include schema:// URI"
    );
}

#[test]
fn test_mcp_resources_read() {
    let (_dir, storage) = setup_storage();
    let cfg = vantadb_mcp::McpConfig::default();

    // Test metrics://
    let res_metrics = handle_resources_read(&Some(json!({"uri": "metrics://"})), &storage, &cfg);
    assert!(res_metrics.is_ok(), "reading metrics:// should succeed");
    let val_metrics = res_metrics.unwrap();
    assert_eq!(val_metrics["contents"][0]["uri"], "metrics://");
    assert_eq!(val_metrics["contents"][0]["mimeType"], "application/json");
    let text = val_metrics["contents"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("hnsw_nodes_count"),
        "metrics response should contain hnsw_nodes_count"
    );

    // Test invalid URI
    let res_invalid = handle_resources_read(&Some(json!({"uri": "invalid://"})), &storage, &cfg);
    assert!(
        res_invalid.is_err(),
        "reading invalid URI should return an error"
    );
}

#[test]
fn test_mcp_resources_read_schema() {
    let (_dir, storage) = setup_storage();
    let cfg = vantadb_mcp::McpConfig::default();

    let res = handle_resources_read(&Some(json!({"uri": "schema://"})), &storage, &cfg);
    assert!(res.is_ok(), "reading schema:// should succeed");
    let val = res.unwrap();
    assert_eq!(val["contents"][0]["uri"], "schema://");
    assert_eq!(val["contents"][0]["mimeType"], "application/json");
    let text = val["contents"][0]["text"].as_str().unwrap();
    let schema: Value = serde_json::from_str(text).expect("schema payload should be valid JSON");

    // vector_index block: HNSW config + format version
    let vector_index = &schema["vector_index"];
    assert_eq!(vector_index["type"], "HNSW");
    assert_eq!(
        vector_index["format_version"],
        vantadb::VECTOR_INDEX_VERSION,
        "schema should report the compiled vector index format version"
    );
    let config = &vector_index["config"];
    assert!(
        config["m"].is_u64() && config["m"].as_u64().unwrap() > 0,
        "HNSW config should expose m"
    );
    assert!(
        config["ef_construction"].is_u64(),
        "HNSW config should expose ef_construction"
    );
    assert!(
        config["ef_search"].is_u64(),
        "HNSW config should expose ef_search"
    );
    assert!(
        config["distance_metric"].is_string(),
        "HNSW config should expose distance_metric"
    );

    // text_index block: schema version + tokenizer
    let text_index = &schema["text_index"];
    assert!(
        text_index["schema_version"].is_u64(),
        "schema should expose text index schema version"
    );
    assert!(
        text_index["tokenizer"]["name"].is_string(),
        "schema should expose tokenizer name"
    );
    assert!(
        text_index["tokenizer"]["version"].is_u64(),
        "schema should expose tokenizer version"
    );
}

#[test]
fn test_mcp_prompts_list() {
    let res = handle_prompts_list();
    assert!(res.is_ok(), "handle_prompts_list should succeed");
    let val = res.unwrap();
    let prompts = val["prompts"].as_array().expect("Expected prompts array");

    let names: Vec<&str> = prompts
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();

    assert!(
        names.contains(&"search_memory"),
        "prompts should include search_memory"
    );
    assert!(
        names.contains(&"analyze_namespace"),
        "prompts should include analyze_namespace"
    );
    assert!(
        names.contains(&"summarize_context"),
        "prompts should include summarize_context"
    );
    assert!(
        names.contains(&"query_builder"),
        "prompts should include query_builder"
    );
}

#[test]
fn test_mcp_prompts_get() {
    // search_memory prompt
    let res_search = handle_prompts_get(Some(&json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "agent_mem",
            "query": "learning rust"
        }
    })));
    assert!(
        res_search.is_ok(),
        "handle_prompts_get for search_memory should succeed"
    );
    let val_search = res_search.unwrap();
    let msg = val_search["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(
        msg.contains("agent_mem"),
        "search_memory prompt should include namespace 'agent_mem'"
    );
    assert!(
        msg.contains("learning rust"),
        "search_memory prompt should include query 'learning rust'"
    );

    // analyze_namespace prompt
    let res_analyze = handle_prompts_get(Some(&json!({
        "name": "analyze_namespace",
        "arguments": {
            "namespace": "billing"
        }
    })));
    assert!(
        res_analyze.is_ok(),
        "handle_prompts_get for analyze_namespace should succeed"
    );
    let val_analyze = res_analyze.unwrap();
    let msg_analyze = val_analyze["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(
        msg_analyze.contains("billing"),
        "analyze_namespace prompt should include namespace 'billing'"
    );

    // summarize_context prompt
    let res_sum = handle_prompts_get(Some(&json!({
        "name": "summarize_context",
        "arguments": {
            "namespace": "chat",
            "limit": 5
        }
    })));
    assert!(
        res_sum.is_ok(),
        "handle_prompts_get for summarize_context should succeed"
    );
    let val_sum = res_sum.unwrap();
    let msg_sum = val_sum["messages"][0]["content"]["text"].as_str().unwrap();
    assert!(
        msg_sum.contains("chat"),
        "summarize_context prompt should include namespace 'chat'"
    );
    assert!(
        msg_sum.contains("5"),
        "summarize_context prompt should include limit 5"
    );

    // query_builder prompt
    let res_qb = handle_prompts_get(Some(&json!({
        "name": "query_builder",
        "arguments": {
            "operation": "SELECT",
            "target": "nodes",
            "conditions": "tier = 'Cold'"
        }
    })));
    assert!(
        res_qb.is_ok(),
        "handle_prompts_get for query_builder should succeed"
    );
    let val_qb = res_qb.unwrap();
    let msg_qb = val_qb["messages"][0]["content"]["text"].as_str().unwrap();
    assert!(
        msg_qb.contains("SELECT"),
        "query_builder prompt should include operation SELECT"
    );
    assert!(
        msg_qb.contains("nodes"),
        "query_builder prompt should include target nodes"
    );
    assert!(
        msg_qb.contains("tier = 'Cold'"),
        "query_builder prompt should include conditions"
    );
}

#[test]
fn test_mcp_tools_list() {
    let res = handle_tools_list();
    assert!(res.is_ok(), "handle_tools_list should succeed");
    let val = res.unwrap();
    let tools = val["tools"].as_array().expect("Expected tools array");

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(
        names.contains(&"memory_put"),
        "tools should include memory_put"
    );
    assert!(
        names.contains(&"memory_get"),
        "tools should include memory_get"
    );
    assert!(
        names.contains(&"memory_delete"),
        "tools should include memory_delete"
    );
    assert!(
        names.contains(&"memory_list"),
        "tools should include memory_list"
    );
    assert!(
        names.contains(&"memory_list_namespaces"),
        "tools should include memory_list_namespaces"
    );
    assert!(
        names.contains(&"query_iql"),
        "tools should include query_iql"
    );
    assert!(
        names.contains(&"search_semantic"),
        "tools should include search_semantic"
    );
    assert!(
        names.contains(&"search_memory"),
        "tools should include search_memory"
    );
    assert!(
        names.contains(&"get_node_neighbors"),
        "tools should include get_node_neighbors"
    );
    assert!(
        names.contains(&"inject_context"),
        "tools should include inject_context"
    );
    assert!(
        names.contains(&"read_axioms"),
        "tools should include read_axioms"
    );
    for maintenance in ["purge_expired", "compact_wal", "flush", "compact_layout"] {
        assert!(
            names.contains(&maintenance),
            "tools should include {maintenance}"
        );
    }
}

#[test]
fn test_mcp_tool_flow_crud() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // 1. memory_put
    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "test_ns",
            "key": "user_status",
            "payload": "User is currently active and coding in Rust",
            "metadata": {
                "priority": 1,
                "verified": true
            }
        }
    }));

    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "memory_put tool call should succeed");
    let put_val = put_res.unwrap();
    assert!(
        put_val["isError"].is_null(),
        "memory_put should not indicate an error"
    );
    let put_text = put_val["content"][0]["text"].as_str().unwrap();
    assert!(
        put_text.contains("user_status"),
        "memory_put response should contain key 'user_status'"
    );
    assert!(
        put_text.contains("test_ns"),
        "memory_put response should contain namespace 'test_ns'"
    );

    // 2. memory_get
    let get_params = Some(json!({
        "name": "memory_get",
        "arguments": {
            "namespace": "test_ns",
            "key": "user_status"
        }
    }));
    let get_res = handle_tools_call(&get_params, &executor, &storage, &default_config());
    assert!(get_res.is_ok(), "memory_get tool call should succeed");
    let get_val = get_res.unwrap();
    assert!(
        get_val["isError"].is_null(),
        "memory_get should not indicate an error"
    );
    let get_text = get_val["content"][0]["text"].as_str().unwrap();
    assert!(
        get_text.contains("active and coding in Rust"),
        "memory_get response should contain stored payload"
    );

    // 3. memory_list
    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": {
            "namespace": "test_ns"
        }
    }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &default_config());
    assert!(list_res.is_ok(), "memory_list tool call should succeed");
    let list_val = list_res.unwrap();
    assert!(
        list_val["isError"].is_null(),
        "memory_list should not indicate an error"
    );
    let list_text = list_val["content"][0]["text"].as_str().unwrap();
    assert!(
        list_text.contains("user_status"),
        "memory_list response should contain key 'user_status'"
    );

    // 4. memory_list_namespaces
    let ns_params = Some(json!({
        "name": "memory_list_namespaces",
        "arguments": {}
    }));
    let ns_res = handle_tools_call(&ns_params, &executor, &storage, &default_config());
    assert!(
        ns_res.is_ok(),
        "memory_list_namespaces tool call should succeed"
    );
    let ns_val = ns_res.unwrap();
    assert!(
        ns_val["isError"].is_null(),
        "memory_list_namespaces should not indicate an error"
    );
    let ns_text = ns_val["content"][0]["text"].as_str().unwrap();
    assert!(
        ns_text.contains("test_ns"),
        "memory_list_namespaces response should include 'test_ns'"
    );

    // 5. memory_delete
    let del_params = Some(json!({
        "name": "memory_delete",
        "arguments": {
            "namespace": "test_ns",
            "key": "user_status"
        }
    }));
    let del_res = handle_tools_call(&del_params, &executor, &storage, &default_config());
    assert!(del_res.is_ok(), "memory_delete tool call should succeed");
    let del_val = del_res.unwrap();
    assert!(
        del_val["isError"].is_null(),
        "memory_delete should not indicate an error"
    );
    let del_text = del_val["content"][0]["text"].as_str().unwrap();
    assert!(
        del_text.contains("\"deleted\":true"),
        "memory_delete response should indicate deleted:true"
    );

    // 6. Verify get after delete
    let get_res_after = handle_tools_call(&get_params, &executor, &storage, &default_config());
    assert!(
        get_res_after.is_ok(),
        "memory_get after delete should still return a response"
    );
    let get_val_after = get_res_after.unwrap();
    assert_eq!(get_val_after["isError"], true);
}

#[test]
fn test_mcp_tool_query_iql() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Execute an INSERT via IQL syntax
    let iql_write = Some(json!({
        "name": "query_iql",
        "arguments": {
            "query": "INSERT NODE#999 TYPE TestNode { tier: \"Cold\" }"
        }
    }));
    let write_res = handle_tools_call(&iql_write, &executor, &storage, &default_config());
    assert!(write_res.is_ok(), "IQL INSERT should succeed");
    let write_val = write_res.unwrap();
    assert!(
        write_val["isError"].is_null(),
        "INSERT should not return isError"
    );
    let write_text = write_val["content"][0]["text"].as_str().unwrap();
    assert!(
        write_text.contains("999"),
        "Response should contain node_id 999"
    );
    assert!(
        write_text.contains("node_id"),
        "Response should contain 'node_id' key"
    );

    // Execute a READ query via IQL syntax (FROM NODE#id)
    let iql_read = Some(json!({
        "name": "query_iql",
        "arguments": {
            "query": "FROM NODE#999"
        }
    }));
    let read_res = handle_tools_call(&iql_read, &executor, &storage, &default_config());
    assert!(read_res.is_ok(), "IQL FROM query should succeed");
    let read_val = read_res.unwrap();
    assert!(
        read_val["isError"].is_null(),
        "FROM query should not return isError"
    );
    let read_text = read_val["content"][0]["text"].as_str().unwrap();
    assert!(
        read_text.contains("999"),
        "Read result should contain node ID 999"
    );
    assert!(
        read_text.contains("Cold"),
        "Read result should contain tier value 'Cold'"
    );
}

/// MCP-27 + MCP-29: documents the chosen IQL semantics.
///
/// Memory records written via `memory_put` live as internal nodes with
/// reserved `__vanta_*` fields and no `type` field. Since MCP-29 each
/// namespace is exposed as an IQL table named by its sanitized form
/// (`/` and `-` map to `_`, leading digit/dot gets a `_` prefix), so
/// `SELECT * FROM <ns>` reaches the records — legacy records included,
/// no migration. Graph nodes created via `INSERT NODE ... TYPE <Type>`
/// keep working unchanged.
#[test]
fn test_query_iql_memory_namespaces_are_queryable_tables_round_trip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // 1) memory_put succeeds...
    let put = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "ProbeNs",
            "key": "k1",
            "payload": "hello world"
        }
    }));
    let put_res = handle_tools_call(&put, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "memory_put should succeed");
    assert!(
        put_res.unwrap()["isError"].is_null(),
        "memory_put should not error"
    );

    // 2) ...and the namespace IS an IQL table now (MCP-29).
    let scan_ns = Some(json!({
        "name": "query_iql",
        "arguments": { "query": "SELECT * FROM ProbeNs" }
    }));
    let res_ns = handle_tools_call(&scan_ns, &executor, &storage, &default_config()).unwrap();
    assert!(
        res_ns["isError"].is_null(),
        "scanning a namespace should not error, got: {}",
        res_ns["content"][0]["text"]
    );
    let ns_text = res_ns["content"][0]["text"].as_str().unwrap();
    assert!(
        ns_text.contains("k1") && ns_text.contains("hello world"),
        "SELECT * FROM ProbeNs must return the memory record, got: {ns_text}"
    );

    // 2b) Namespace with `/` is reachable via its sanitized table name.
    let put_slash = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "mmd/s1/history",
            "key": "k2",
            "payload": "slashed"
        }
    }));
    let put_slash_res =
        handle_tools_call(&put_slash, &executor, &storage, &default_config()).unwrap();
    assert!(
        put_slash_res["isError"].is_null(),
        "memory_put with slashed namespace should not error"
    );
    let scan_slash = Some(json!({
        "name": "query_iql",
        "arguments": { "query": "SELECT * FROM mmd_s1_history" }
    }));
    let res_slash = handle_tools_call(&scan_slash, &executor, &storage, &default_config()).unwrap();
    assert!(res_slash["isError"].is_null());
    let slash_text = res_slash["content"][0]["text"].as_str().unwrap();
    assert!(
        slash_text.contains("k2"),
        "sanitized table must reach slashed namespace records, got: {slash_text}"
    );

    // 3) Graph-node round-trip keeps working end to end from the agent
    //    channel: INSERT via query_iql → SELECT by TYPE returns the node.
    let insert = Some(json!({
        "name": "query_iql",
        "arguments": {
            "query": "INSERT NODE#777 TYPE ProbeType { label: \"graph node\" }"
        }
    }));
    let ins_res = handle_tools_call(&insert, &executor, &storage, &default_config()).unwrap();
    assert!(
        ins_res["isError"].is_null(),
        "IQL INSERT should succeed, got: {}",
        ins_res["content"][0]["text"]
    );

    let select = Some(json!({
        "name": "query_iql",
        "arguments": { "query": "SELECT * FROM ProbeType" }
    }));
    let sel_res = handle_tools_call(&select, &executor, &storage, &default_config()).unwrap();
    assert!(
        sel_res["isError"].is_null(),
        "typed SELECT should not error"
    );
    let text = sel_res["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("777") && text.contains("ProbeType"),
        "SELECT * FROM ProbeType should return the inserted node, got: {text}"
    );
}

#[test]
fn test_mcp_query_iql_sanitization() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Test empty query rejection
    let empty_query = Some(json!({
        "name": "query_iql",
        "arguments": {
            "query": "   "
        }
    }));
    let res_empty = handle_tools_call(&empty_query, &executor, &storage, &default_config());
    assert!(res_empty.is_ok());
    let val_empty = res_empty.unwrap();
    let text_empty = val_empty["content"][0]["text"].as_str().unwrap();
    assert!(text_empty.contains("cannot be empty"));

    // Test null byte injection rejection
    let null_byte_query = Some(json!({
        "name": "query_iql",
        "arguments": {
            "query": "FROM NODE#1\0; DROP TABLE"
        }
    }));
    let res_null = handle_tools_call(&null_byte_query, &executor, &storage, &default_config());
    assert!(res_null.is_ok());
    let val_null = res_null.unwrap();
    let text_null = val_null["content"][0]["text"].as_str().unwrap();
    assert!(text_null.contains("invalid null bytes"));
}

#[test]
fn test_mcp_tool_search() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Insert some memories with vectors
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0];

    let put_params_1 = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "search_ns",
            "key": "vector_x",
            "payload": "Point X axis",
            "vector": v1
        }
    }));
    handle_tools_call(&put_params_1, &executor, &storage, &default_config()).unwrap();

    let put_params_2 = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "search_ns",
            "key": "vector_y",
            "payload": "Point Y axis",
            "vector": v2
        }
    }));
    handle_tools_call(&put_params_2, &executor, &storage, &default_config()).unwrap();

    // Test search_semantic (raw vector index)
    let search_sem_params = Some(json!({
        "name": "search_semantic",
        "arguments": {
            "vector": [0.9, 0.1, 0.0],
            "k": 1
        }
    }));
    let sem_res = handle_tools_call(&search_sem_params, &executor, &storage, &default_config());
    assert!(sem_res.is_ok(), "search_semantic tool call should succeed");
    let sem_val = sem_res.unwrap();
    let sem_text = sem_val["content"][0]["text"].as_str().unwrap();
    // Raw search returns node hits
    assert!(
        sem_text.contains("score") || sem_text.contains("id"),
        "search_semantic response should contain 'score' or 'id'"
    );

    // Test search_memory (vector-only path, no text index dependency)
    let search_mem_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "search_ns",
            "query_vector": [0.95, 0.05, 0.0],
            "top_k": 2
        }
    }));
    let mem_res = handle_tools_call(&search_mem_params, &executor, &storage, &default_config());
    assert!(mem_res.is_ok(), "search_memory tool call should succeed");
    let mem_val = mem_res.unwrap();
    // search_memory should return a valid response (even if empty for vector-only without text index)
    assert!(
        mem_val["isError"].is_null() || mem_val["content"][0]["text"].is_string(),
        "search_memory response should have no error or valid text content"
    );
}

// ── MEM-02: search_profile passthrough + paridad IQL/API/MCP (D13/D19) ─────
//
// The MCP search_memory tool must behave EXACTLY like the native API for the
// same SearchProfileConfig (same struct, same serde shape → passthrough) and
// force the same retrieval channels the IQL PROFILE clause forces. Exact
// score parity MCP↔IQL is NOT asserted: IQL text matching is a scan +
// `text_contains_query` substring filter (src/physical_plan/filter.rs) while
// the SDK uses BM25 postings (src/sdk/search/lexical.rs) — mechanically
// different engines by design (planner.rs `ponytail:` note: IQL applies mode
// only; rrf_k/candidate_k are SDK-side). Parity is asserted where the
// channels are semantically comparable: exact passthrough MCP↔API, and
// mode-driven channel selection identical in all three.

fn hits_from_mcp_search(res: Value) -> Vec<(String, f32)> {
    let text = res["content"][0]["text"]
        .as_str()
        .expect("search_memory text content");
    let hits: Vec<Value> = serde_json::from_str(text).expect("hits JSON");
    hits.iter()
        .map(|h| {
            let key = h["record"]["key"].as_str().unwrap_or_default().to_string();
            let score = h["score"].as_f64().unwrap_or(0.0) as f32;
            (key, score)
        })
        .collect()
}

fn error_text_from(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

#[test]
fn test_search_profile_mcp_passthrough_parity_with_native() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Seed via the MCP memory_put path (same write pipeline as the SDK).
    for (key, payload, vec) in [
        ("a", "cat chases mouse", vec![1.0, 0.0, 0.0]),
        ("b", "dog sleeps all day", vec![0.0, 1.0, 0.0]),
        ("c", "cat food recipe", vec![0.5, 0.5, 0.0]),
    ] {
        let put_params = Some(json!({
            "name": "memory_put",
            "arguments": { "namespace": "parity_ns", "key": key, "payload": payload, "vector": vec }
        }));
        handle_tools_call(&put_params, &executor, &storage, &default_config()).unwrap();
    }

    // StorageEngine::open alone leaves the text_index state missing; rebuild
    // so text queries work (AUD-044: "reopen writable or run rebuild_index").
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded.rebuild_index().expect("text index build");

    // Explicit profile → MCP and native API return IDENTICAL hits (keys + scores).
    let search_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "parity_ns",
            "text_query": "cat",
            "query_vector": [0.9, 0.1, 0.0],
            "top_k": 10,
            "search_profile": { "mode": "hybrid", "rrf_k": 30, "candidate_k": 64 }
        }
    }));
    let res = handle_tools_call(&search_params, &executor, &storage, &default_config()).unwrap();
    let mcp_hits = hits_from_mcp_search(res);

    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded.rebuild_index().expect("text index build");

    let native_req = vantadb::sdk::VantaMemorySearchRequest {
        namespace: "parity_ns".into(),
        query_vector: vec![0.9, 0.1, 0.0],
        text_query: Some("cat".into()),
        top_k: 10,
        search_profile: Some(vantadb::sdk::SearchProfileConfig {
            mode: vantadb::sdk::SearchProfileMode::Hybrid,
            rrf_k: Some(30),
            candidate_k: Some(64),
        }),
        ..Default::default()
    };
    let native_hits = embedded.search(native_req).expect("native search");
    let native: Vec<(String, f32)> = native_hits
        .iter()
        .map(|h| (h.record.key.clone(), h.score))
        .collect();

    assert_eq!(mcp_hits.len(), native.len(), "same hit count");
    for (m, n) in mcp_hits.iter().zip(native.iter()) {
        assert_eq!(m.0, n.0, "same key order");
        assert!(
            (m.1 - n.1).abs() < 1e-4,
            "same score: MCP {} vs native {}",
            m.1,
            n.1
        );
    }

    // No profile on both sides → identical defaults (MEM-01 constants).
    let search_none = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "parity_ns",
            "text_query": "cat",
            "query_vector": [0.9, 0.1, 0.0],
            "top_k": 10
        }
    }));
    let res_none = handle_tools_call(&search_none, &executor, &storage, &default_config()).unwrap();
    let mcp_none = hits_from_mcp_search(res_none);
    let native_none_req = vantadb::sdk::VantaMemorySearchRequest {
        namespace: "parity_ns".into(),
        query_vector: vec![0.9, 0.1, 0.0],
        text_query: Some("cat".into()),
        top_k: 10,
        ..Default::default()
    };
    let native_none = embedded
        .search(native_none_req)
        .expect("native default search");
    let native_none: Vec<(String, f32)> = native_none
        .iter()
        .map(|h| (h.record.key.clone(), h.score))
        .collect();
    assert_eq!(mcp_none, native_none, "no-profile parity");
}

#[test]
fn test_search_profile_mode_force_channels() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Same dataset as the MEM-01 core test (src/sdk/search/tests.rs:968):
    // the vector favors "b" but the text "cat" only matches "a".
    for (key, payload, vec) in [
        ("a", "cat chases mouse", vec![1.0, 0.0, 0.0]),
        ("b", "dog sleeps all day", vec![0.0, 1.0, 0.0]),
    ] {
        let put_params = Some(json!({
            "name": "memory_put",
            "arguments": { "namespace": "mode_ns", "key": key, "payload": payload, "vector": vec }
        }));
        handle_tools_call(&put_params, &executor, &storage, &default_config()).unwrap();
    }

    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded.rebuild_index().expect("text index build");

    let search_with_mode = |mode: &str| {
        handle_tools_call(
            &Some(json!({
                "name": "search_memory",
                "arguments": {
                    "namespace": "mode_ns",
                    "text_query": "cat",
                    "query_vector": [0.0, 1.0, 0.0],
                    "top_k": 10,
                    "search_profile": { "mode": mode }
                }
            })),
            &executor,
            &storage,
            &default_config(),
        )
        .unwrap()
    };

    // keyword → lexical channel only: "a" (vector-only "b" excluded), matching
    // the IQL PROFILE keyword channel selection (planner mode forcing).
    let kw = hits_from_mcp_search(search_with_mode("keyword"));
    let kw_keys: Vec<&str> = kw.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        kw_keys,
        vec!["a"],
        "keyword mode must ignore the vector channel"
    );

    // vector → pure vector ordering ["b", "a"] — text ignored. Mirrors the
    // MEM-01 core assertion (tests.rs:1041) and equals a vector-only search.
    let vec_hits = hits_from_mcp_search(search_with_mode("vector"));
    let vec_keys: Vec<&str> = vec_hits.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        vec_keys,
        vec!["b", "a"],
        "vector mode: pure vector ordering"
    );

    // Control: same search without text_query must produce the same order.
    let control = handle_tools_call(
        &Some(json!({
            "name": "search_memory",
            "arguments": {
                "namespace": "mode_ns",
                "query_vector": [0.0, 1.0, 0.0],
                "top_k": 10
            }
        })),
        &executor,
        &storage,
        &default_config(),
    )
    .unwrap();
    let control_hits = hits_from_mcp_search(control);
    let control_keys: Vec<&str> = control_hits.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        vec_keys, control_keys,
        "mode Vector == vector-only puro (texto ignorado)"
    );
}

#[test]
fn test_search_profile_validation_errors() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let search_with = |profile: Value| {
        handle_tools_call(
            &Some(json!({
                "name": "search_memory",
                "arguments": { "namespace": "val_ns", "search_profile": profile }
            })),
            &executor,
            &storage,
            &default_config(),
        )
    };

    // Unknown mode → serde enum error naming search_profile (no panic).
    let text = error_text_from(search_with(json!({"mode": "bogus"})));
    assert!(text.contains("search_profile"), "bad mode: {}", text);

    // rrf_k = 0 → degenerate RRF fusion → rejected.
    let text = error_text_from(search_with(json!({"rrf_k": 0})));
    assert!(text.contains("rrf_k"), "rrf_k=0: {}", text);

    // candidate_k over the budget → memory-inflation risk → rejected.
    let text = error_text_from(search_with(json!({"candidate_k": 999_999})));
    assert!(
        text.contains("candidate_k"),
        "candidate_k too big: {}",
        text
    );

    // Wrong top-level type → clear error, not a panic.
    let text = error_text_from(search_with(json!("hybrid")));
    assert!(text.contains("object"), "non-object: {}", text);
}

// ── MCP-03: search_semantic.distance semantics ─────────────────────────────
//
// The `distance` field must be a real distance (lower = more similar), NOT the
// raw cosine similarity the HNSW score carries. Identical vector → 0.0,
// orthogonal vector → 1.0 (1 − cosine_sim). The conversion happens at the
// handler serialization boundary (vantadb-mcp/src/handlers/tools.rs); the core
// keeps its score semantics for other consumers (search_memory, WASM).

#[test]
fn test_mcp_search_semantic_distance_semantics() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Seed two records: one identical to the query, one orthogonal to it.
    for (key, vec) in [
        ("identical", vec![1.0, 0.0, 0.0]),
        ("orthogonal", vec![0.0, 1.0, 0.0]),
    ] {
        let put_params = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "mcp03_ns",
                "key": key,
                "payload": format!("{} payload", key),
                "vector": vec
            }
        }));
        let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
        assert!(put_res.is_ok(), "seed {key} should succeed: {:?}", put_res);
    }

    let search_params = Some(json!({
        "name": "search_semantic",
        "arguments": { "vector": [1.0, 0.0, 0.0], "k": 2 }
    }));
    let res = handle_tools_call(&search_params, &executor, &storage, &default_config());
    assert!(res.is_ok(), "search_semantic should succeed");
    let val = res.unwrap();
    assert!(
        val["isError"].is_null(),
        "search_semantic should not error: {:?}",
        val
    );
    let text = val["content"][0]["text"].as_str().unwrap();
    let hits: Value = serde_json::from_str(text).expect("search_semantic response should be JSON");

    let hits = hits
        .as_array()
        .expect("search_semantic should return an array");
    assert_eq!(hits.len(), 2, "k=2 should return 2 hits");

    // Every hit must expose id, distance, and node.
    for hit in hits {
        assert!(hit["id"].is_string(), "hit must expose id field: {hit}");
        assert!(
            hit["distance"].is_f64(),
            "hit must expose distance field: {hit}"
        );
        assert!(hit["node"].is_object(), "hit must expose node field: {hit}");
    }

    // Identical vector → distance ≈ 0.0 under cosine (was 1.0 similarity).
    let identical = &hits[0];
    assert!(
        (identical["distance"].as_f64().unwrap() - 0.0).abs() < 1e-4,
        "identical vector must report distance≈0.0, got {:?}",
        identical["distance"]
    );

    // Orthogonal vector → distance ≈ 1.0 (1 − 0 similarity).
    let orthogonal = &hits[1];
    assert!(
        (orthogonal["distance"].as_f64().unwrap() - 1.0).abs() < 1e-4,
        "orthogonal vector must report distance≈1.0, got {:?}",
        orthogonal["distance"]
    );

    // Distances must be ascending: lower = more similar.
    let dists: Vec<f64> = hits
        .iter()
        .map(|h| h["distance"].as_f64().unwrap())
        .collect();
    let mut sorted = dists.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("distances are finite"));
    assert_eq!(dists, sorted, "distances must be ascending, got {dists:?}");
}

// ── AUD-046: memory_put validates vector dims against the live index ─────

#[test]
fn test_mcp_put_validates_vector_dimensions() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // 1. First 4-dim put on an empty index defines the dim (no index yet).
    let first_put = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "dims_ns",
            "key": "seed",
            "payload": "seed",
            "vector": [1.0, 0.0, 0.0, 0.0]
        }
    }));
    let first_res = handle_tools_call(&first_put, &executor, &storage, &default_config());
    assert!(
        first_res.is_ok(),
        "first 4-dim put should succeed: {:?}",
        first_res
    );
    let first_val = first_res.unwrap();
    assert!(
        first_val["isError"].is_null(),
        "first put must not error: {:?}",
        first_val
    );

    // 2. A put with matching dims still works.
    let good_put = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "dims_ns",
            "key": "good",
            "payload": "good",
            "vector": [0.0, 1.0, 0.0, 0.0]
        }
    }));
    let good_res = handle_tools_call(&good_put, &executor, &storage, &default_config());
    assert!(
        good_res.is_ok(),
        "matching-dim put should succeed: {:?}",
        good_res
    );
    assert!(
        good_res.unwrap()["isError"].is_null(),
        "matching-dim put must not error"
    );

    // 3. A put with wrong dims (2 vs 4) must fail BEFORE inserting, with an
    // explicit DimensionMismatch error (AUD-046: no silent index corruption).
    let bad_put = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "dims_ns",
            "key": "bad",
            "payload": "bad",
            "vector": [0.5, 0.5]
        }
    }));
    let bad_res = handle_tools_call(&bad_put, &executor, &storage, &default_config());
    assert!(
        bad_res.is_ok(),
        "handler must return Ok with isError content, got: {:?}",
        bad_res
    );
    let bad_val = bad_res.unwrap();
    assert_eq!(
        bad_val["isError"], true,
        "wrong-dim put must be flagged as error: {:?}",
        bad_val
    );
    let bad_text = bad_val["content"][0]["text"].as_str().unwrap();
    assert!(
        bad_text.contains("Vector dimension mismatch: expected 4, got 2"),
        "error must name expected and got dims, got: {}",
        bad_text
    );

    // 4. The rejected node must NOT have been inserted.
    let get_params = Some(json!({
        "name": "memory_get",
        "arguments": { "namespace": "dims_ns", "key": "bad" }
    }));
    let get_res = handle_tools_call(&get_params, &executor, &storage, &default_config());
    let get_val = get_res.unwrap();
    assert_eq!(
        get_val["isError"], true,
        "rejected node must not be retrievable: {:?}",
        get_val
    );

    // 5. A put WITHOUT a vector (text-only) is unaffected by dim validation.
    let novec_put = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "dims_ns",
            "key": "text_only",
            "payload": "hello"
        }
    }));
    let novec_res = handle_tools_call(&novec_put, &executor, &storage, &default_config());
    assert!(
        novec_res.is_ok(),
        "text-only put should succeed: {:?}",
        novec_res
    );
    assert!(
        novec_res.unwrap()["isError"].is_null(),
        "text-only put must not error"
    );
}

// ── MCP-04: Collection Management Tests ─────────────────────────────────

#[test]
fn test_collection_stats() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Put records with vectors (all have vectors so they're discoverable)
    for i in 0..3 {
        let params = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "stats_ns",
                "key": format!("k{}", i),
                "payload": format!("record {}", i),
                "vector": [i as f32, 0.0, 0.0],
                "metadata": { "idx": i }
            }
        }));
        handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
    }
    // Put one additional record with a different vector
    let params_with_vec = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "stats_ns",
            "key": "k_vec",
            "payload": "has vector",
            "vector": [5.0, 0.0, 0.0]
        }
    }));
    handle_tools_call(&params_with_vec, &executor, &storage, &default_config()).unwrap();

    // Call collection_stats
    let params = Some(json!({
        "name": "collection_stats",
        "arguments": { "namespace": "stats_ns" }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(
        val["isError"].is_null(),
        "collection_stats should not error"
    );
    let text = val["content"][0]["text"].as_str().unwrap();
    let stats: Value = serde_json::from_str(text).unwrap();
    assert!(
        stats["total_records"].as_u64().unwrap_or(0) >= 1,
        "should have at least 1 record"
    );
    let vector_count = stats["vector_count"].as_u64().unwrap_or(0);
    assert!(vector_count >= 1, "should have at least 1 vector");
    assert!(
        stats["total_bytes"].as_u64().unwrap_or(0) > 0,
        "total_bytes should be positive"
    );
    assert!(
        stats["created_at"].as_u64().unwrap_or(0) > 0,
        "created_at should be positive"
    );
}

#[test]
fn test_collection_list() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Create records in 2 namespaces
    for ns in &["list_a", "list_b"] {
        let params = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": ns,
                "key": "item",
                "payload": format!("in {}", ns)
            }
        }));
        handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
    }

    // Call collection_list
    let params = Some(json!({
        "name": "collection_list",
        "arguments": {}
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(val["isError"].is_null(), "collection_list should not error");
    let text = val["content"][0]["text"].as_str().unwrap();
    let collections: Vec<Value> = serde_json::from_str(text).unwrap();
    let names: Vec<&str> = collections
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"list_a"),
        "collections should include list_a"
    );
    assert!(
        names.contains(&"list_b"),
        "collections should include list_b"
    );
    for c in &collections {
        assert_eq!(c["record_count"], 1);
    }
}

#[test]
fn test_collection_stats_large_namespace_bounded() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    // Force multi-page scans by capping page size well below the record count.
    let cfg = vantadb_mcp::McpConfig {
        max_list_limit: 5,
        ..default_config()
    };

    // Namespace bigger than one page (12 records) so aggregation must cross pages
    // without materializing the whole namespace (AUDREP-21 OOM regression test).
    for i in 0..12u32 {
        let params = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "big_ns",
                "key": format!("k{}", i),
                "payload": format!("record {}", i),
                // +1.0 keeps every vector non-zero-norm: i==0 would otherwise
                // produce [0.0,0.0,0.0], rejected by the zero-norm guard
                // (AUDREP-27) under cosine distance.
                "vector": [i as f32 + 1.0, 0.0, 0.0]
            }
        }));
        handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
    }

    let stats_params = Some(json!({
        "name": "collection_stats",
        "arguments": { "namespace": "big_ns" }
    }));
    let res = handle_tools_call(&stats_params, &executor, &storage, &cfg);
    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(
        val["isError"].is_null(),
        "collection_stats should not error"
    );
    let text = val["content"][0]["text"].as_str().unwrap();
    let stats: Value = serde_json::from_str(text).unwrap();
    assert_eq!(
        stats["total_records"], 12,
        "stats must aggregate across multiple pages"
    );
    assert_eq!(
        stats["vector_count"], 12,
        "vector count must aggregate across multiple pages"
    );

    // collection_list must also report correct per-namespace counts across pages.
    let list_params = Some(json!({ "name": "collection_list", "arguments": {} }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &cfg);
    assert!(list_res.is_ok());
    let list_val = list_res.unwrap();
    assert!(
        list_val["isError"].is_null(),
        "collection_list should not error"
    );
    let list_text = list_val["content"][0]["text"].as_str().unwrap();
    let collections: Vec<Value> = serde_json::from_str(list_text).unwrap();
    let big = collections
        .iter()
        .find(|c| c["name"] == "big_ns")
        .expect("big_ns should be listed");
    assert_eq!(big["record_count"], 12);
}

#[test]
fn test_collection_delete() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Create namespace with records
    for i in 0..3 {
        let params = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "del_ns",
                "key": format!("k{}", i),
                "payload": "to delete"
            }
        }));
        handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
    }

    // Verify records exist
    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": { "namespace": "del_ns" }
    }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &default_config()).unwrap();
    let list_text = list_res["content"][0]["text"].as_str().unwrap();
    assert!(
        list_text.contains("k0"),
        "records should exist before delete"
    );

    // Delete without confirm should fail
    let del_no_confirm = Some(json!({
        "name": "collection_delete",
        "arguments": { "namespace": "del_ns", "confirm": "no" }
    }));
    let res = handle_tools_call(&del_no_confirm, &executor, &storage, &default_config());
    assert!(res.is_ok());
    assert_eq!(res.unwrap()["isError"], true);

    // Delete with confirm
    let del_params = Some(json!({
        "name": "collection_delete",
        "arguments": { "namespace": "del_ns", "confirm": "yes" }
    }));
    let del_res = handle_tools_call(&del_params, &executor, &storage, &default_config());
    assert!(del_res.is_ok());
    let del_val = del_res.unwrap();
    assert!(del_val["isError"].is_null());
    let del_text = del_val["content"][0]["text"].as_str().unwrap();
    let result: Value = serde_json::from_str(del_text).unwrap();
    assert_eq!(result["deleted"], true);
    assert_eq!(result["records_removed"], 3);

    // Verify namespace is empty
    let list_after =
        handle_tools_call(&list_params, &executor, &storage, &default_config()).unwrap();
    let after_text = list_after["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(after_text).unwrap();
    let records = page["records"].as_array().unwrap();
    assert!(records.is_empty(), "namespace should be empty after delete");
}

/// Deleting N records in one `collection_delete` call must be all-or-nothing:
/// if any record fails, the transaction aborts and no records are removed.
/// (Happy path — all N deleted atomically — is covered by `test_collection_delete`.)
#[test]
fn test_collection_delete_abort_leaves_no_partial_deletes() {
    let (_dir, mut storage) = setup_storage();

    // Seed 3 records (executor borrows, it does not own the Arc, so a
    // scoped block lets go of the borrow before we mutate below).
    {
        let executor = Executor::new(&storage);
        for i in 0..3 {
            let params = Some(json!({
                "name": "memory_put",
                "arguments": {
                    "namespace": "abort_ns",
                    "key": format!("k{}", i),
                    "payload": "to delete"
                }
            }));
            handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
        }
    }

    // Make every per-record delete fail mid-loop so the handler aborts its
    // transaction instead of committing. `collection_delete` clones its
    // embedded handle from `storage.config` at call time, so flipping the
    // flag now is enough to gate every delete.
    Arc::get_mut(&mut storage)
        .expect("no live clones")
        .config
        .read_only = true;

    let executor = Executor::new(&storage);
    let del_params = Some(json!({
        "name": "collection_delete",
        "arguments": { "namespace": "abort_ns", "confirm": "yes" }
    }));
    let res = handle_tools_call(&del_params, &executor, &storage, &default_config());
    assert!(
        res.is_ok(),
        "handler should surface failure as error content"
    );
    let val = res.unwrap();
    assert_eq!(val["isError"], true);

    // No partial deletes: all 3 records must still be present.
    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": { "namespace": "abort_ns" }
    }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &default_config()).unwrap();
    let list_text = list_res["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(list_text).unwrap();
    let records = page["records"].as_array().unwrap();
    assert_eq!(
        records.len(),
        3,
        "aborted collection_delete must not remove any records"
    );
}

// ── Error Handling Tests ───────────────────────────────────────────────

#[test]
fn test_mcp_invalid_json() {
    // Test McpError::parse_error produces correct JSON-RPC structure (-32700)
    let err = McpError::parse_error("Expected value at line 1 column 2");
    assert_eq!(err.code, -32700);
    let err_json = err.to_json();
    assert_eq!(err_json["code"], -32700);
    assert!(err_json["message"]
        .as_str()
        .unwrap()
        .contains("Parse error"));

    // Verify that handle_tools_call with None params returns invalid params error (-32602)
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    let res = handle_tools_call(&None, &executor, &storage, &default_config());
    assert!(res.is_err());
    let err_val = res.unwrap_err();
    assert_eq!(err_val["code"], -32602);
}

#[test]
fn test_mcp_unknown_method() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let params = Some(json!({
        "name": "nonexistent_tool",
        "arguments": {}
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_err(), "unknown tool should return error");
    let err = res.unwrap_err();
    assert_eq!(err["code"], -32601, "should be method not found");
    assert!(
        err["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent_tool"),
        "error message should include tool name"
    );
}

#[test]
fn test_mcp_missing_params() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Call memory_list without required 'namespace'
    let params = Some(json!({
        "name": "memory_list",
        "arguments": {}
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_err(), "missing required params should return error");
    let err = res.unwrap_err();
    assert_eq!(err["code"], -32602, "should be invalid params");
}

#[test]
fn test_mcp_oversized_payload() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let huge = "a".repeat(2 * 1024 * 1024);
    let params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "test",
            "key": "big",
            "payload": huge
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_err(), "oversized payload should return error");
}

// ── Edge Cases Tests ───────────────────────────────────────────────────

#[test]
fn test_mcp_empty_namespace() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // List on non-existent namespace should return empty list
    let params = Some(json!({
        "name": "memory_list",
        "arguments": { "namespace": "nonexistent_ns" }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(val["isError"].is_null());
    let text = val["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(text).unwrap();
    assert!(
        page["records"].as_array().unwrap().is_empty(),
        "records for empty namespace should be empty"
    );
}

#[test]
fn test_mcp_empty_key() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "test",
            "key": "",
            "payload": "test payload"
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_err(), "empty key should return error");
    let err = res.unwrap_err();
    assert_eq!(err["code"], -32602);
}

#[test]
fn test_mcp_concurrent_requests() {
    let (_dir, storage) = setup_storage();
    let config = default_config();

    let mut handles = Vec::new();
    for i in 0..5u64 {
        let storage = storage.clone();
        let cfg = config.clone();
        handles.push(thread::spawn(move || {
            let executor = Executor::new(&storage);
            let params = Some(json!({
                "name": "memory_put",
                "arguments": {
                    "namespace": "concurrent_ns",
                    "key": format!("key_{}", i),
                    "payload": format!("Concurrent payload {}", i)
                }
            }));
            handle_tools_call(&params, &executor, &storage, &cfg)
        }));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        let res = handle.join().expect("thread panicked");
        assert!(res.is_ok(), "concurrent request {} should succeed", i);
        let val = res.unwrap();
        assert!(
            val["isError"].is_null(),
            "concurrent request {} should not error",
            i
        );
    }

    // Verify all 5 records were created
    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": { "namespace": "concurrent_ns", "limit": 100 }
    }));
    let executor = Executor::new(&storage);
    let list_res = handle_tools_call(&list_params, &executor, &storage, &config).unwrap();
    let list_text = list_res["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(list_text).unwrap();
    assert_eq!(
        page["records"].as_array().unwrap().len(),
        5,
        "should have 5 concurrent records"
    );
}

#[test]
fn test_mcp_search_no_results() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Search in a namespace that has no records at all
    let search_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "empty_ns_for_search",
            "query_vector": [0.5, 0.5, 0.5],
            "top_k": 5
        }
    }));
    let res = handle_tools_call(&search_params, &executor, &storage, &default_config());
    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(val["isError"].is_null());
    let text = val["content"][0]["text"].as_str().unwrap();
    // Should be empty array or valid JSON response
    let hits: Vec<Value> = serde_json::from_str(text).unwrap_or_else(|_| {
        // If parsing fails, check if it's a search response object
        if text.contains("error") {
            vec![] // treat as empty
        } else {
            panic!("Could not parse search response: {}", text);
        }
    });
    assert!(
        hits.is_empty(),
        "search in empty namespace should return empty results"
    );
}

// ── Resource & Prompt Tests ────────────────────────────────────────────

#[test]
fn test_mcp_resource_invalid() {
    let (_dir, storage) = setup_storage();
    let cfg = vantadb_mcp::McpConfig::default();

    let res = handle_resources_read(
        &Some(json!({"uri": "nonexistent://resource"})),
        &storage,
        &cfg,
    );
    assert!(
        res.is_err(),
        "non-existent resource URI should return error"
    );
    let err = res.unwrap_err();
    assert_eq!(err["code"], -32601, "should be method not found");
}

#[test]
fn test_mcp_prompt_empty_args() {
    // Get search_memory prompt without providing optional arguments
    let res = handle_prompts_get(Some(&json!({
        "name": "search_memory"
    })));
    assert!(res.is_ok(), "prompt without optional args should succeed");
    let val = res.unwrap();
    let text = val["messages"][0]["content"]["text"].as_str().unwrap();
    assert!(!text.is_empty(), "prompt text should not be empty");
    assert!(
        text.contains("namespace"),
        "prompt text should mention namespace"
    );
    assert!(
        text.contains("default"),
        "prompt text should default namespace to 'default'"
    );
}

#[test]
fn test_inject_context_lisp_injection() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Content with parentheses, newlines, and quotes that could break LISP parsing
    let inject_params = Some(json!({
        "name": "inject_context",
        "arguments": {
            "content": "); DROP TABLE nodes; --\n(context (nested \"stuff\" here))\nline2",
            "thread_id": 1
        }
    }));
    let res = handle_tools_call(&inject_params, &executor, &storage, &default_config());
    assert!(
        res.is_ok(),
        "inject_context with special chars should not fail"
    );
    let val = res.unwrap();
    // Should succeed (isError null), not crash the parser
    assert!(
        val["isError"].is_null() || val["isError"] == false,
        "inject_context should not indicate error: {:?}",
        val
    );
    let text = val["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("Context Anchored") || text.contains("affected_nodes"),
        "inject_context response should indicate success: {}",
        text
    );
}

#[test]
fn test_inject_context_thread_id_type_error() {
    // AUD-050: a string thread_id used to surface as "Missing 'thread_id'"
    // even though the field IS present. The error must name the real problem:
    // wrong type, not absence.
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let string_params = Some(json!({
        "name": "inject_context",
        "arguments": {
            "content": "hello",
            "thread_id": "200"
        }
    }));
    let res = handle_tools_call(&string_params, &executor, &storage, &default_config());
    let err = res.unwrap_err();
    assert_eq!(err["code"], -32602, "should be invalid params");
    let msg = err["message"].as_str().unwrap();
    assert!(
        msg.contains("thread_id"),
        "error should name the field, got: {}",
        msg
    );
    assert!(
        msg.contains("numeric") && msg.contains("string"),
        "error should explain numeric requirement and got type, got: {}",
        msg
    );
    assert!(
        !msg.contains("Missing"),
        "must not claim the field is missing, got: {}",
        msg
    );

    // Numeric thread_id must keep working.
    let numeric_params = Some(json!({
        "name": "inject_context",
        "arguments": {
            "content": "hello",
            "thread_id": 200
        }
    }));
    let res = handle_tools_call(&numeric_params, &executor, &storage, &default_config());
    assert!(res.is_ok(), "numeric thread_id should still succeed");
    let val = res.unwrap();
    assert!(
        val["isError"].is_null() || val["isError"] == false,
        "numeric inject_context should not indicate error: {:?}",
        val
    );
}

#[test]
fn test_mcp_prompt_invalid_name() {
    let res = handle_prompts_get(Some(&json!({
        "name": "nonexistent_prompt_name"
    })));
    assert!(res.is_err(), "non-existent prompt name should return error");
    let err = res.unwrap_err();
    assert_eq!(err["code"], -32602, "should be invalid params");
    assert!(
        err["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent_prompt_name"),
        "error message should include prompt name"
    );
}

#[test]
fn test_mcp_get_node_neighbors_preserves_large_u128_ids() {
    // ERR-025: node ids are u128; JSON numbers lose precision above 2^53 and
    // cannot represent ids above 2^64, so IDs must round-trip as strings.
    let big_id = 9007199254740993u128; // 2^53 + 1 — first id a f64 cannot represent exactly
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());

    embedded
        .insert_node(vantadb::sdk::VantaNodeInput::new(7))
        .expect("insert source node");
    embedded
        .insert_node(vantadb::sdk::VantaNodeInput::new(big_id))
        .expect("insert big-id node");
    embedded
        .add_edge(7, big_id, "knows", None, None)
        .expect("add edge");

    // 1. Big id as the *neighbor*: assert it round-trips exactly as a string.
    let neighbors_params = Some(json!({
        "name": "get_node_neighbors",
        "arguments": { "node_id": "7" }
    }));
    let res = handle_tools_call(&neighbors_params, &executor, &storage, &default_config());
    assert!(res.is_ok(), "get_node_neighbors should succeed");
    let val = res.unwrap();
    assert!(
        val["isError"].is_null(),
        "get_node_neighbors should not indicate error: {:?}",
        val
    );
    let text = val["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains(&format!("\"target_id\":\"{big_id}\"")),
        "neighbor id must round-trip exactly as a string, got: {text}"
    );

    // 2. Big id as the *query input*: a string id > 2^53 must parse to u128
    //    and find the node (was previously rejected by `as_u64`).
    let big_params = Some(json!({
        "name": "get_node_neighbors",
        "arguments": { "node_id": big_id.to_string() }
    }));
    let big_res = handle_tools_call(&big_params, &executor, &storage, &default_config());
    assert!(big_res.is_ok(), "big node_id call should succeed");
    let big_val = big_res.unwrap();
    assert!(
        big_val["isError"].is_null(),
        "big node_id should not be an error: {:?}",
        big_val
    );
    let big_text = big_val["content"][0]["text"].as_str().unwrap();
    assert!(
        !big_text.contains("Node not found") && big_text.contains("\"target_id\":\"7\""),
        "big node_id should resolve its reverse edge, got: {big_text}"
    );
}

#[test]
fn test_mcp_parse_metadata_delegates_lists_and_null() {
    // ERR-026: parse_metadata used to silently drop non-scalar metadata
    // values (arrays/objects/null), producing a super-set of results. The
    // contract is: delegate what the core can filter (lists and null via
    // VantaValue::List* / Null with strict equality) and explicitly reject
    // only what it cannot represent (objects, mixed-type arrays).
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // memory_put must accept delegable metadata: lists, null, and scalars.
    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "meta_ns",
            "key": "k1",
            "payload": "p1",
            "metadata": {
                "tags": ["a", "b"],
                "flag": null,
                "priority": 1
            }
        }
    }));
    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(
        put_res.is_ok(),
        "memory_put must accept list/null metadata, got: {:?}",
        put_res
    );

    // Second record without tags: it must NOT match a tags filter.
    let put2_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "meta_ns",
            "key": "k2",
            "payload": "p2",
            "metadata": { "priority": 2 }
        }
    }));
    assert!(handle_tools_call(&put2_params, &executor, &storage, &default_config()).is_ok());

    // memory_list with a list filter must return exactly the matching subset
    // (not a superset and not an error): the core filters ListString by
    // strict equality against the record's stored ListString.
    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": {
            "namespace": "meta_ns",
            "filters": { "tags": ["a", "b"] }
        }
    }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &default_config());
    assert!(
        list_res.is_ok(),
        "list filter must not error: {:?}",
        list_res
    );
    let list_val = list_res.unwrap();
    assert!(
        list_val["isError"].is_null(),
        "list filter must not indicate an error: {:?}",
        list_val
    );
    let list_text = list_val["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(list_text).expect("list response should be JSON");
    let records = page["records"]
        .as_array()
        .expect("list response should contain a records array");
    assert_eq!(
        records.len(),
        1,
        "list filter must return exactly the matching record, got {} records: {page}",
        records.len()
    );
    assert_eq!(
        records[0]["key"],
        json!("k1"),
        "the matching record must be k1, got: {page}"
    );
    // The stored metadata round-trips: ListString serializes as an
    // externally-tagged serde enum.
    assert_eq!(
        records[0]["metadata"]["tags"],
        json!({"ListString": ["a", "b"]}),
        "stored list metadata must round-trip, got: {page}"
    );
    assert_eq!(
        records[0]["metadata"]["flag"],
        json!("Null"),
        "stored null metadata must round-trip as VantaValue::Null, got: {page}"
    );

    // An object cannot be represented by VantaValue → explicit rejection on
    // both memory_put and memory_list filters.
    let object_put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "meta_ns",
            "key": "bad",
            "payload": "p",
            "metadata": { "nested": {"a": 1} }
        }
    }));
    let object_put_res =
        handle_tools_call(&object_put_params, &executor, &storage, &default_config());
    assert!(
        object_put_res.is_err(),
        "memory_put metadata with object value must be rejected, not silently ignored"
    );

    let object_list_params = Some(json!({
        "name": "memory_list",
        "arguments": {
            "namespace": "meta_ns",
            "filters": { "nested": {"a": 1} }
        }
    }));
    let object_list_res =
        handle_tools_call(&object_list_params, &executor, &storage, &default_config());
    assert!(
        object_list_res.is_err(),
        "memory_list filters with object value must be rejected, not silently ignored"
    );

    // Mixed-type arrays are also not representable → explicit rejection.
    let mixed_put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "meta_ns",
            "key": "bad2",
            "payload": "p",
            "metadata": { "tags": ["a", 1] }
        }
    }));
    let mixed_put_res =
        handle_tools_call(&mixed_put_params, &executor, &storage, &default_config());
    assert!(
        mixed_put_res.is_err(),
        "memory_put metadata with mixed array must be rejected, not silently ignored"
    );

    // Scalar metadata must still be accepted (regression guard).
    let ok_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "meta_ns",
            "key": "ok",
            "payload": "p",
            "metadata": {
                "priority": 1,
                "verified": true,
                "label": "doc",
                "score": 1.5
            }
        }
    }));
    let res = handle_tools_call(&ok_params, &executor, &storage, &default_config());
    assert!(res.is_ok(), "scalar metadata must still be accepted");
}

#[test]
fn test_mcp_memory_list_limit_zero_returns_empty() {
    // ERR-033: memory_list(limit=0) used to return 1 record because the core
    // clamps limit to >= 1. A requested limit of 0 must return 0 records.
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "limit_ns",
            "key": "k",
            "payload": "p"
        }
    }));
    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "seed memory_put should succeed");

    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": {
            "namespace": "limit_ns",
            "limit": 0
        }
    }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &default_config());
    assert!(list_res.is_ok(), "memory_list(limit=0) should succeed");
    let list_val = list_res.unwrap();
    assert!(
        list_val["isError"].is_null(),
        "memory_list(limit=0) should not indicate an error: {:?}",
        list_val
    );
    let list_text = list_val["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(list_text).expect("list response should be JSON");
    let records = page["records"]
        .as_array()
        .expect("list response should contain a records array");
    assert!(
        records.is_empty(),
        "memory_list(limit=0) must return 0 records, got {}",
        records.len()
    );

    // Control: the same namespace without a limit returns the seeded record.
    let default_params = Some(json!({
        "name": "memory_list",
        "arguments": { "namespace": "limit_ns" }
    }));
    let default_res = handle_tools_call(&default_params, &executor, &storage, &default_config());
    assert!(default_res.is_ok(), "memory_list default should succeed");
    let default_val = default_res.unwrap();
    let default_text = default_val["content"][0]["text"].as_str().unwrap();
    assert!(
        default_text.contains("limit_ns") && default_text.contains("\"key\":\"k\""),
        "default memory_list should still return the seeded record, got: {default_text}"
    );
}

// ── MCP-01: text index ready on fresh DB (regression) ─────────────────────
//
// The MCP server opens a raw `StorageEngine` (not `VantaEmbedded::open_with_config`),
// so index state was never reconciled: text_query / hybrid / text-filter
// searches failed on fresh DBs with "Search Error: text_index not found: bm25".
// The server now calls `ensure_indexes_current()` at startup; this test
// reproduces that contract: without the ensure the search must fail with the
// documented error, and with it the same search must return hits.

#[test]
fn test_mcp_text_search_requires_index_ensure() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "mcp01_ns",
            "key": "doc1",
            "payload": "concise technical answer",
            "vector": [1.0, 0.0, 0.0]
        }
    }));
    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "seed memory_put should succeed");

    // Without startup index ensure, text search must fail with the documented error.
    let search_params = Some(json!({
        "name": "search_memory",
        "arguments": { "namespace": "mcp01_ns", "text_query": "concise", "top_k": 5 }
    }));
    let raw_res = handle_tools_call(&search_params, &executor, &storage, &default_config());
    let raw_val = raw_res.expect("search_memory call should return");
    let raw_text = raw_val["content"][0]["text"].as_str().unwrap_or_default();
    assert!(
        raw_val["isError"].is_object() || raw_text.contains("text_index not found: bm25"),
        "without ensure_indexes_current, text_query should fail with \
         'text_index not found: bm25', got: {raw_text}"
    );

    // Simulate server startup (run_stdio_server does this before serving).
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded
        .ensure_indexes_current()
        .expect("startup index ensure should succeed");

    // Same search now succeeds and returns the seeded record.
    let fixed_res = handle_tools_call(&search_params, &executor, &storage, &default_config());
    let fixed_val = fixed_res.expect("search_memory call should return");
    assert!(
        fixed_val["isError"].is_null(),
        "after ensure_indexes_current, text_query should not error: {fixed_val}"
    );
    let fixed_text = fixed_val["content"][0]["text"].as_str().unwrap();
    assert!(
        fixed_text.contains("doc1"),
        "after ensure_indexes_current, text_query should return doc1, got: {fixed_text}"
    );

    // Hybrid (text + vector) also works after the ensure.
    let hybrid_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "mcp01_ns",
            "text_query": "concise",
            "query_vector": [1.0, 0.0, 0.0],
            "top_k": 5
        }
    }));
    let hybrid_res = handle_tools_call(&hybrid_params, &executor, &storage, &default_config());
    let hybrid_val = hybrid_res.expect("hybrid search_memory call should return");
    assert!(
        hybrid_val["isError"].is_null(),
        "hybrid search should not error after ensure: {hybrid_val}"
    );
    assert!(
        hybrid_val["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("doc1"),
        "hybrid search should return doc1 after ensure"
    );

    // Text filters (BM25 + metadata filter) also work after the ensure.
    let filter_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "mcp01_ns",
            "text_query": "concise",
            "filters": { "category": "dev" },
            "top_k": 5
        }
    }));
    let filter_res = handle_tools_call(&filter_params, &executor, &storage, &default_config());
    let filter_val = filter_res.expect("filtered search_memory call should return");
    assert!(
        filter_val["isError"].is_null(),
        "filtered text search should not error after ensure: {filter_val}"
    );
}

// ── AUD-048: unified filter semantics (operators on MCP, flat on CLI) ────
//
// Both channels now accept BOTH filter formats, normalized at parse time:
// - flat values `{"field": value}` → implicit `$eq`
// - operator objects `{"field": {"$eq": v}}` / `{"field": {"$gt": v}}` etc.
//
// `memory_list` routes operators through the core's `filter_ops` slot (full
// operator support). `search_memory`'s request has no filter_ops slot, so
// `$eq` folds into the flat metadata (identical equality semantics) and
// range operators return a clear, documented error.

#[test]
fn test_mcp_list_filters_accept_operators() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let put1 = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud048_list_ns",
            "key": "low",
            "payload": "p low",
            "metadata": { "score": 10 }
        }
    }));
    assert!(handle_tools_call(&put1, &executor, &storage, &default_config()).is_ok());

    let put2 = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud048_list_ns",
            "key": "high",
            "payload": "p high",
            "metadata": { "score": 50 }
        }
    }));
    assert!(handle_tools_call(&put2, &executor, &storage, &default_config()).is_ok());

    // `$gt` operator form must work on memory_list (core filter_ops).
    let list_params = Some(json!({
        "name": "memory_list",
        "arguments": {
            "namespace": "aud048_list_ns",
            "filters": { "score": { "$gt": 20 } }
        }
    }));
    let list_res = handle_tools_call(&list_params, &executor, &storage, &default_config());
    assert!(
        list_res.is_ok(),
        "memory_list with $gt operator should succeed: {:?}",
        list_res
    );
    let list_val = list_res.unwrap();
    assert!(
        list_val["isError"].is_null(),
        "memory_list with $gt must not error: {list_val}"
    );
    let list_text = list_val["content"][0]["text"].as_str().unwrap();
    let page: Value = serde_json::from_str(list_text).expect("list response should be JSON");
    let records = page["records"].as_array().expect("records array");
    assert_eq!(
        records.len(),
        1,
        "$gt filter must match exactly the high record, got: {page}"
    );
    assert_eq!(
        records[0]["key"],
        json!("high"),
        "must match key 'high', got: {page}"
    );

    // Flat form still works unchanged (published behavior).
    let flat_params = Some(json!({
        "name": "memory_list",
        "arguments": {
            "namespace": "aud048_list_ns",
            "filters": { "score": 10 }
        }
    }));
    let flat_res = handle_tools_call(&flat_params, &executor, &storage, &default_config());
    assert!(
        flat_res.is_ok(),
        "flat list filter must still work: {flat_res:?}"
    );
    let flat_val = flat_res.unwrap();
    assert!(
        flat_val["isError"].is_null(),
        "flat list filter must not error"
    );
    let flat_page: Value =
        serde_json::from_str(flat_val["content"][0]["text"].as_str().unwrap()).unwrap();
    let flat_records = flat_page["records"].as_array().unwrap();
    assert_eq!(
        flat_records.len(),
        1,
        "flat filter must match low only: {flat_page}"
    );
    assert_eq!(flat_records[0]["key"], json!("low"));
}

#[test]
fn test_mcp_search_filters_accept_eq_and_reject_range() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded
        .ensure_indexes_current()
        .expect("startup index ensure should succeed");

    let put1 = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud048_search_ns",
            "key": "doc1",
            "payload": "concise technical answer about rust",
            "metadata": { "category": "dev" }
        }
    }));
    assert!(handle_tools_call(&put1, &executor, &storage, &default_config()).is_ok());

    // `$eq` operator form folds into the flat metadata — equality semantics.
    let eq_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "aud048_search_ns",
            "text_query": "concise",
            "filters": { "category": { "$eq": "dev" } },
            "top_k": 5
        }
    }));
    let eq_res = handle_tools_call(&eq_params, &executor, &storage, &default_config());
    let eq_val = eq_res.expect("search_memory with $eq should return");
    assert!(
        eq_val["isError"].is_null(),
        "search_memory with $eq must not error: {eq_val}"
    );
    let eq_text = eq_val["content"][0]["text"].as_str().unwrap();
    assert!(
        eq_text.contains("doc1"),
        "search_memory with $eq must return doc1, got: {eq_text}"
    );

    // Range operators cannot be expressed in a search request (flat-only slot
    // in `VantaMemorySearchRequest`) → clear documented error, not silence.
    let gt_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "aud048_search_ns",
            "text_query": "concise",
            "filters": { "category": { "$gt": "abc" } },
            "top_k": 5
        }
    }));
    let gt_res = handle_tools_call(&gt_params, &executor, &storage, &default_config());
    let gt_val = gt_res.expect("search_memory with $gt should return a call result");
    let gt_text = gt_val["content"][0]["text"].as_str().unwrap_or_default();
    assert!(
        gt_val["isError"].is_object() || gt_text.contains("equality only"),
        "search_memory with $gt must error clearly, got: {gt_text}"
    );
    assert!(
        gt_text.contains("memory_list"),
        "range-operator error must point at memory_list, got: {gt_text}"
    );
}

// ── T15: search_memory(explain=true) shape contract (regression) ──────────
//
// The MCP response for `search_memory(explain: true)` is a FLAT ARRAY of hits.
// Each hit is `{record, score, explanation?}` where `explanation` is a
// per-hit scoring breakdown: identity, score, snippet, matched_tokens,
// matched_phrases, bm25_terms, rrf_text_rank, rrf_vector_rank.
// There is NO top-level `route` / `fusion_report` on search_memory — those
// belong to the dedicated `explain_memory_search` method. This test pins the
// real shape so the docs (SKILL.md / api-reference.md) stay truthful.

#[test]
fn test_mcp_search_memory_explain_shape() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded
        .ensure_indexes_current()
        .expect("startup index ensure should succeed");

    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "t15_ns",
            "key": "doc1",
            "payload": "concise technical answer about rust",
            "vector": [1.0, 0.0, 0.0]
        }
    }));
    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "seed memory_put should succeed");

    let search_params = Some(json!({
        "name": "search_memory",
        "arguments": {
            "namespace": "t15_ns",
            "text_query": "concise",
            "query_vector": [1.0, 0.0, 0.0],
            "top_k": 5,
            "explain": true
        }
    }));
    let res = handle_tools_call(&search_params, &executor, &storage, &default_config());
    let val = res.expect("search_memory(explain=true) should return");
    assert!(
        val["isError"].is_null(),
        "search_memory(explain=true) should not error: {val}"
    );
    let text = val["content"][0]["text"].as_str().unwrap();
    let hits: Value = serde_json::from_str(text).expect("response should be JSON");

    // Contract: flat ARRAY of hits — no top-level route/fusion_report envelope.
    let arr = hits
        .as_array()
        .expect("response should be a flat hit array");
    assert!(
        !arr.is_empty(),
        "seeded search should return at least one hit"
    );
    for hit in arr {
        assert!(
            hit.get("route").is_none() && hit.get("fusion_report").is_none(),
            "hit must not carry top-level route/fusion_report, got: {hit}"
        );
        assert!(
            hit.get("record").is_some() && hit.get("score").is_some(),
            "hit must carry record and score, got: {hit}"
        );
        let explanation = hit
            .get("explanation")
            .expect("explain=true must attach explanation");
        for field in [
            "identity",
            "score",
            "snippet",
            "matched_tokens",
            "matched_phrases",
            "bm25_terms",
            "rrf_text_rank",
            "rrf_vector_rank",
        ] {
            assert!(
                explanation.get(field).is_some(),
                "explanation must contain {field}, got: {explanation}"
            );
        }
    }
}

// ── AUD-045: memory_put accepts expires_at_ms + sparse_vector ─────────────
//
// The MCP tool schema used to omit `expires_at_ms` and `sparse_vector`, so
// clients that sent them had the fields silently dropped and the returned
// record showed `expires_at_ms: null` / `sparse_vector: null`. Both fields
// must now round-trip: expires_at_ms (absolute Unix-ms) is converted to the
// SDK input's relative ttl_ms and comes back as a non-null absolute value;
// sparse_vector (object dim id -> weight) persists and is returned.

#[test]
fn test_mcp_memory_put_accepts_expires_at_ms_and_sparse_vector() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let future_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
        + 60_000;

    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud045_ns",
            "key": "k1",
            "payload": "ttl + sparse payload",
            "expires_at_ms": future_ms,
            "sparse_vector": { "0": 0.5, "7": 1.25 }
        }
    }));
    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "memory_put with TTL+sparse should succeed");
    let put_val = put_res.unwrap();
    assert!(
        put_val["isError"].is_null(),
        "memory_put with TTL+sparse should not error: {put_val}"
    );
    let put_text = put_val["content"][0]["text"].as_str().unwrap();
    let record: Value = serde_json::from_str(put_text).expect("put response should be JSON");
    assert!(
        record["expires_at_ms"].is_number(),
        "record must carry a non-null expires_at_ms, got: {record}"
    );
    assert_eq!(
        record["sparse_vector"]["0"], 0.5,
        "sparse_vector dim 0 must round-trip, got: {record}"
    );
    assert_eq!(
        record["sparse_vector"]["7"], 1.25,
        "sparse_vector dim 7 must round-trip, got: {record}"
    );

    // Persistence: a get must return the same stored TTL and sparse vector.
    let get_params = Some(json!({
        "name": "memory_get",
        "arguments": { "namespace": "aud045_ns", "key": "k1" }
    }));
    let get_res = handle_tools_call(&get_params, &executor, &storage, &default_config());
    assert!(get_res.is_ok(), "memory_get should succeed");
    let get_val = get_res.unwrap();
    let get_text = get_val["content"][0]["text"].as_str().unwrap();
    let fetched: Value = serde_json::from_str(get_text).expect("get response should be JSON");
    assert!(
        fetched["expires_at_ms"].is_number(),
        "persisted record must keep expires_at_ms, got: {fetched}"
    );
    assert_eq!(
        fetched["sparse_vector"]["0"], 0.5,
        "persisted record must keep sparse_vector dim 0, got: {fetched}"
    );
    assert_eq!(
        fetched["sparse_vector"]["7"], 1.25,
        "persisted record must keep sparse_vector dim 7, got: {fetched}"
    );
}

// AUD-045 backward compatibility: omitting both new fields must keep the
// previous behaviour (record without TTL / sparse, no error).
#[test]
fn test_mcp_memory_put_without_ttl_or_sparse_stays_backward_compatible() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let put_params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud045_plain_ns",
            "key": "k",
            "payload": "plain"
        }
    }));
    let put_res = handle_tools_call(&put_params, &executor, &storage, &default_config());
    assert!(put_res.is_ok(), "plain memory_put should succeed");
    let put_val = put_res.unwrap();
    assert!(
        put_val["isError"].is_null(),
        "plain memory_put should not error: {put_val}"
    );
    let put_text = put_val["content"][0]["text"].as_str().unwrap();
    let record: Value = serde_json::from_str(put_text).expect("put response should be JSON");
    assert!(
        record["expires_at_ms"].is_null() || record.get("expires_at_ms").is_none(),
        "record without TTL must have null/absent expires_at_ms, got: {record}"
    );
    assert!(
        record["sparse_vector"].is_null() || record.get("sparse_vector").is_none(),
        "record without sparse must have null/absent sparse_vector, got: {record}"
    );
}

// AUD-045: invalid sparse_vector / expires_at_ms must fail explicitly — the
// fix must never silently drop a field a client sent (the original bug).
#[test]
fn test_mcp_memory_put_rejects_invalid_sparse_and_ttl() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let bad_sparse = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud045_bad_ns",
            "key": "k",
            "payload": "p",
            "sparse_vector": [0.5, 1.25]
        }
    }));
    let sparse_res = handle_tools_call(&bad_sparse, &executor, &storage, &default_config());
    let sparse_err = sparse_res.expect_err("array sparse_vector must be rejected");
    assert!(
        sparse_err["code"].is_number(),
        "array sparse_vector must fail with a JSON-RPC error, got: {sparse_err}"
    );

    let bad_ttl = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "aud045_bad_ns",
            "key": "k",
            "payload": "p",
            "expires_at_ms": "not-a-number"
        }
    }));
    let ttl_res = handle_tools_call(&bad_ttl, &executor, &storage, &default_config());
    let ttl_err = ttl_res.expect_err("non-numeric expires_at_ms must be rejected");
    assert!(
        ttl_err["code"].is_number(),
        "non-numeric expires_at_ms must fail with a JSON-RPC error, got: {ttl_err}"
    );
}

// ── MCP-16/MCP-23: maintenance tools (purge_expired/compact_wal/flush/compact_layout) ──

fn maintenance_call(
    executor: &Executor<'_>,
    storage: &Arc<StorageEngine>,
    name: &str,
) -> Result<Value, Value> {
    handle_tools_call(
        &Some(json!({ "name": name, "arguments": {} })),
        executor,
        storage,
        &default_config(),
    )
}

#[test]
fn test_maintenance_tools_round_trip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Expired record: expires_at_ms=1 is in the past → saturates ttl_ms to 0
    // → expires immediately. A live record must survive the purge.
    let put_expired = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "maint_ns",
            "key": "ephemeral",
            "payload": "short lived record",
            "expires_at_ms": 1u64
        }
    }));
    let put_val = handle_tools_call(&put_expired, &executor, &storage, &default_config()).unwrap();
    assert!(
        put_val["isError"].is_null(),
        "expired put failed: {}",
        put_val["content"][0]["text"]
    );

    let put_durable = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "maint_ns",
            "key": "durable",
            "payload": "long lived record"
        }
    }));
    let put_val2 = handle_tools_call(&put_durable, &executor, &storage, &default_config()).unwrap();
    assert!(
        put_val2["isError"].is_null(),
        "durable put failed: {}",
        put_val2["content"][0]["text"]
    );

    // purge_expired compares now > expires_at_ms; give the clock room to move.
    thread::sleep(std::time::Duration::from_millis(20));

    // 1. purge_expired → purged >= 1
    let purge_val = maintenance_call(&executor, &storage, "purge_expired").unwrap();
    assert!(
        purge_val["isError"].is_null(),
        "purge_expired failed: {}",
        purge_val["content"][0]["text"]
    );
    let purged: Value = serde_json::from_str(purge_val["content"][0]["text"].as_str().unwrap())
        .expect("purge_expired payload must be JSON");
    assert_eq!(
        purged["purged"], 1,
        "expected exactly the expired record purged"
    );

    // 2. The durable record survived.
    let get_val = handle_tools_call(
        &Some(json!({
            "name": "memory_get",
            "arguments": { "namespace": "maint_ns", "key": "durable" }
        })),
        &executor,
        &storage,
        &default_config(),
    )
    .unwrap();
    assert!(
        get_val["isError"].is_null(),
        "durable record must survive purge: {}",
        get_val["content"][0]["text"]
    );

    // 3. flush → flushed:true; compact_wal → compacted_wal:true;
    //    compact_layout → bytes_reclaimed number.
    for (tool, key) in [("flush", "flushed"), ("compact_wal", "compacted_wal")] {
        let res = maintenance_call(&executor, &storage, tool).unwrap();
        assert!(
            res["isError"].is_null(),
            "{tool} failed: {}",
            res["content"][0]["text"]
        );
        let payload: Value = serde_json::from_str(res["content"][0]["text"].as_str().unwrap())
            .expect("payload must be JSON");
        assert_eq!(payload[key], true, "{tool} must report success via {key}");
    }

    let layout_val = maintenance_call(&executor, &storage, "compact_layout").unwrap();
    assert!(
        layout_val["isError"].is_null(),
        "compact_layout failed: {}",
        layout_val["content"][0]["text"]
    );
    let layout: Value = serde_json::from_str(layout_val["content"][0]["text"].as_str().unwrap())
        .expect("compact_layout payload must be JSON");
    assert!(
        layout["bytes_reclaimed"].is_u64(),
        "compact_layout must return a byte count, got: {layout}"
    );
}

#[test]
fn test_mcp_tools_list_includes_backup_restore() {
    let res = handle_tools_list().unwrap();
    let tools = res["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    for tool in ["export", "import", "bulk_import_file", "bulk_import_stream"] {
        assert!(names.contains(&tool), "tools should include {tool}");
    }
}

/// MCP-17: round-trip export → import into a FRESH database → get returns the
/// same payload/metadata. Also covers multi-namespace export_all and malformed
/// line handling.
#[test]
fn test_mcp_tool_flow_backup_restore_roundtrip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Seed two namespaces with metadata so fidelity is checkable.
    for (ns, key, payload, prio) in [
        ("backup_ns", "alpha", "Alpha payload", 1),
        ("backup_ns", "beta", "Beta payload", 2),
        ("other_ns", "gamma", "Gamma payload", 3),
    ] {
        let params = Some(json!({
            "name": "memory_put",
            "arguments": {"namespace": ns, "key": key, "payload": payload, "metadata": {"priority": prio}}
        }));
        handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
    }

    // Single-namespace export → exactly 2 lines.
    let export_ns = Some(json!({
        "name": "export",
        "arguments": {"namespace": "backup_ns"}
    }));
    let res = handle_tools_call(&export_ns, &executor, &storage, &default_config()).unwrap();
    assert!(res["isError"].is_null(), "namespace export failed");
    let ns_jsonl = res["content"][0]["text"].as_str().unwrap();
    assert_eq!(
        ns_jsonl.lines().count(),
        2,
        "expected 2 records: {ns_jsonl}"
    );

    // export (all namespaces) → includes both namespaces, 3 lines.
    let export_all = Some(json!({ "name": "export", "arguments": {} }));
    let res = handle_tools_call(&export_all, &executor, &storage, &default_config()).unwrap();
    assert!(res["isError"].is_null(), "export all failed");
    let all_jsonl = res["content"][0]["text"].as_str().unwrap().to_string();
    assert_eq!(
        all_jsonl.lines().count(),
        3,
        "expected 3 records: {all_jsonl}"
    );
    assert!(all_jsonl.contains("backup_ns") && all_jsonl.contains("other_ns"));

    // Restore into a FRESH database and verify the report.
    let (_dir2, storage2) = setup_storage();
    let executor2 = Executor::new(&storage2);
    let import = Some(json!({
        "name": "import",
        "arguments": {"content": all_jsonl}
    }));
    let res = handle_tools_call(&import, &executor2, &storage2, &default_config()).unwrap();
    assert!(res["isError"].is_null(), "import failed");
    let report: Value = serde_json::from_str(res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(report["inserted"], 3);
    assert_eq!(report["errors"], 0);

    // Round-trip equality: same payload/metadata after restore.
    let get = Some(json!({
        "name": "memory_get",
        "arguments": {"namespace": "backup_ns", "key": "alpha"}
    }));
    let res = handle_tools_call(&get, &executor2, &storage2, &default_config()).unwrap();
    let rec: Value = serde_json::from_str(res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(rec["payload"], "Alpha payload");
    // Metadata round-trips as a tagged VantaValue.
    assert_eq!(rec["metadata"]["priority"], json!({"Int": 1}));

    // Malformed line is counted as an error, not a crash; empty lines skipped.
    let bad_import = Some(json!({
        "name": "import",
        "arguments": {"content": "{not json}\n\n"}
    }));
    let res = handle_tools_call(&bad_import, &executor2, &storage2, &default_config()).unwrap();
    let report: Value = serde_json::from_str(res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(report["errors"], 1);
    assert_eq!(report["skipped"], 1);
}

/// MCP-25: NDJSON bulk import via stream (count correct + record landed), and
/// a nonexistent host path returns clear error_content instead of an Err.
#[test]
fn test_mcp_bulk_import_stream_ndjson_and_missing_file() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // NDJSON of 100 records → count correct.
    let mut ndjson = String::new();
    for i in 0..100 {
        ndjson.push_str(&format!(
            "{{\"namespace\":\"bulk_ns\",\"key\":\"k{i}\",\"payload\":\"payload {i}\",\"metadata\":{{}}}}\n"
        ));
    }
    let params = Some(json!({
        "name": "bulk_import_stream",
        "arguments": {"content": ndjson}
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "bulk_import_stream failed: {}",
        res["content"][0]["text"]
    );
    let report: Value = serde_json::from_str(res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(report["total_records"], 100);
    assert_eq!(report["batches_committed"], 1);

    // NOTE: bulk-imported records are intentionally NOT asserted via
    // memory_get: the core's bulk_import_stream writes raw nodes without the
    // internal __vanta_namespace/__vanta_key fields, so they are not
    // addressable through the record API (pre-existing SDK limitation,
    // tracked in docs/Backlog.md). The tool contract is the report counts.

    // Malformed NDJSON line → error_content naming the line.
    let bad = Some(json!({
        "name": "bulk_import_stream",
        "arguments": {"content": "{\"namespace\":\"x\"\n"}
    }));
    let res = handle_tools_call(&bad, &executor, &storage, &default_config()).unwrap();
    assert_eq!(res["isError"], true);
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("malformed NDJSON at line 1"),
        "unexpected error text: {}",
        res["content"][0]["text"]
    );

    // Nonexistent host file path → clear error_content (MEM-32), never Err.
    let missing = Some(json!({
        "name": "bulk_import_file",
        "arguments": {"path": "./definitively/missing_dump.vdbdump"}
    }));
    let res = handle_tools_call(&missing, &executor, &storage, &default_config()).unwrap();
    assert_eq!(res["isError"], true);
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Bulk Import File Error"),
        "unexpected error text: {}",
        res["content"][0]["text"]
    );
}

/// MCP-18: put 3 records with metadata → delete_by_filter matches a subset →
/// list reflects the deletion and the returned count is correct.
#[test]
fn test_memory_delete_by_filter_round_trip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    for (key, env) in [("task_a", "dev"), ("task_b", "prod"), ("task_c", "prod")] {
        let put = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "filter_ns",
                "key": key,
                "payload": format!("payload of {key}"),
                "metadata": {"env": env}
            }
        }));
        let res = handle_tools_call(&put, &executor, &storage, &default_config()).unwrap();
        assert!(
            res["isError"].is_null(),
            "memory_put {key} should succeed: {}",
            res["content"][0]["text"]
        );
    }

    // Delete the single dev record via filter.
    let del = Some(json!({
        "name": "memory_delete_by_filter",
        "arguments": {
            "namespace": "filter_ns",
            "filters": {"env": "dev"}
        }
    }));
    let res = handle_tools_call(&del, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "delete_by_filter should succeed: {}",
        res["content"][0]["text"]
    );
    assert_eq!(
        res["content"][0]["text"].as_str().unwrap(),
        r#"{"deleted_count":1}"#,
        "count returned must match the filtered subset"
    );

    // List reflects the deletion: only the two prod records remain.
    let list = Some(json!({
        "name": "memory_list",
        "arguments": {"namespace": "filter_ns"}
    }));
    let res = handle_tools_call(&list, &executor, &storage, &default_config()).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(
        !text.contains("task_a"),
        "deleted record must not be listed"
    );
    assert!(text.contains("task_b") && text.contains("task_c"));
}

/// MCP-18: operator filter ($gt) and guard rails (empty filters rejected,
/// MEM-32 error shape, missing params → invalid_params).
#[test]
fn test_memory_delete_by_filter_validation() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    for (i, prio) in [1u64, 5, 9].iter().enumerate() {
        let put = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "filter_ns",
                "key": format!("rec_{i}"),
                "payload": "x",
                "metadata": {"priority": prio}
            }
        }));
        handle_tools_call(&put, &executor, &storage, &default_config()).unwrap();
    }

    // $gt operator deletes the subset priority > 1.
    let del = Some(json!({
        "name": "memory_delete_by_filter",
        "arguments": {
            "namespace": "filter_ns",
            "filters": {"priority": {"$gt": 1}}
        }
    }));
    let res = handle_tools_call(&del, &executor, &storage, &default_config()).unwrap();
    assert_eq!(
        res["content"][0]["text"].as_str().unwrap(),
        r#"{"deleted_count":2}"#
    );

    // Empty filters object → SDK guard rail surfaces as error_content (MEM-32).
    let empty = Some(json!({
        "name": "memory_delete_by_filter",
        "arguments": {"namespace": "filter_ns", "filters": {}}
    }));
    let res = handle_tools_call(&empty, &executor, &storage, &default_config()).unwrap();
    assert_eq!(res["isError"], true);
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("at least one filter"),
        "unexpected error text: {}",
        res["content"][0]["text"]
    );

    // Missing 'filters' param → JSON-RPC invalid_params Err (param-shape error).
    let no_filters = Some(json!({
        "name": "memory_delete_by_filter",
        "arguments": {"namespace": "filter_ns"}
    }));
    assert!(
        handle_tools_call(&no_filters, &executor, &storage, &default_config()).is_err(),
        "missing 'filters' must surface as an invalid_params error"
    );

    // Unknown operator → explicit invalid_params.
    let bad_op = Some(json!({
        "name": "memory_delete_by_filter",
        "arguments": {"namespace": "filter_ns", "filters": {"p": {"$weird": 1}}}
    }));
    let err = handle_tools_call(&bad_op, &executor, &storage, &default_config())
        .expect_err("unknown operator must fail");
    assert!(
        err.to_string().contains("$eq, $neq, $gt"),
        "unexpected error: {}",
        err
    );
}

/// MCP-19: put_batch of 3 records → memory_list returns all 3. Duplicate keys
/// inside a batch are UPSERTs with version bump (documented SDK semantics),
/// and one malformed input fails the whole call before any write (all-or-nothing).
#[test]
fn test_memory_put_batch_round_trip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    let batch = Some(json!({
        "name": "memory_put_batch",
        "arguments": {
            "inputs": [
                {"namespace": "batch_ns", "key": "k1", "payload": "one"},
                {"namespace": "batch_ns", "key": "k2", "payload": "two",
                 "metadata": {"env": "prod"}},
                {"namespace": "batch_ns", "key": "k3", "payload": "three"}
            ]
        }
    }));
    let res = handle_tools_call(&batch, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "put_batch should succeed: {}",
        res["content"][0]["text"]
    );
    let records: Vec<Value> = serde_json::from_str(res["content"][0]["text"].as_str().unwrap())
        .expect("put_batch result should serialize as a JSON array of records");
    assert_eq!(records.len(), 3);

    // memory_list returns the 3 batch-inserted records (derived indexes rebuilt
    // by the SDK after each batch — list/count must not return 0).
    let list = Some(json!({
        "name": "memory_list",
        "arguments": {"namespace": "batch_ns"}
    }));
    let res = handle_tools_call(&list, &executor, &storage, &default_config()).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    for key in ["k1", "k2", "k3"] {
        assert!(text.contains(key), "list should contain {key}");
    }

    // Duplicate key in a second batch → upsert, not error; k2 version bumps.
    let dup = Some(json!({
        "name": "memory_put_batch",
        "arguments": {
            "inputs": [
                {"namespace": "batch_ns", "key": "k2", "payload": "two v2"},
                {"namespace": "batch_ns", "key": "k4", "payload": "four"}
            ]
        }
    }));
    let res = handle_tools_call(&dup, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "duplicate key is an upsert, not an error: {}",
        res["content"][0]["text"]
    );
    let get = Some(json!({
        "name": "memory_get",
        "arguments": {"namespace": "batch_ns", "key": "k2"}
    }));
    let res = handle_tools_call(&get, &executor, &storage, &default_config()).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("two v2"), "upserted payload wins");
    assert!(
        text.contains(r#""version":2"#),
        "version must bump on upsert: {}",
        text
    );

    // Malformed input (missing 'payload') → whole call fails as invalid_params,
    // nothing from the batch is written (SDK validates every input upfront).
    let bad = Some(json!({
        "name": "memory_put_batch",
        "arguments": {
            "inputs": [
                {"namespace": "batch_ns", "key": "good_key", "payload": "ok"},
                {"namespace": "batch_ns", "key": "bad_key"}
            ]
        }
    }));
    assert!(
        handle_tools_call(&bad, &executor, &storage, &default_config()).is_err(),
        "malformed input must surface as invalid_params"
    );
    let get_bad = Some(json!({
        "name": "memory_get",
        "arguments": {"namespace": "batch_ns", "key": "good_key"}
    }));
    let res = handle_tools_call(&get_bad, &executor, &storage, &default_config()).unwrap();
    assert_eq!(
        res["isError"], true,
        "all-or-nothing: good_key from the failed batch must NOT exist"
    );
}

/// MCP-19: vector dim mismatch against the live HNSW index is rejected at this
/// trust boundary (parity with memory_put AUD-046), and empty inputs array is
/// a param error.
#[test]
fn test_memory_put_batch_vector_and_empty_validation() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    let cfg = default_config();

    // Define index dim = 3 with a vectorized record.
    let seed = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "vec_ns", "key": "seed", "payload": "s",
            "vector": [1.0, 0.0, 0.0]
        }
    }));
    handle_tools_call(&seed, &executor, &storage, &cfg).unwrap();

    // Batch carrying a mismatched-dim vector → domain error as error_content.
    let mismatch = Some(json!({
        "name": "memory_put_batch",
        "arguments": {
            "inputs": [
                {"namespace": "vec_ns", "key": "ok", "payload": "fine"},
                {"namespace": "vec_ns", "key": "bad_vec", "payload": "v",
                 "vector": [1.0, 2.0]}
            ]
        }
    }));
    let res = handle_tools_call(&mismatch, &executor, &storage, &cfg).unwrap();
    assert_eq!(res["isError"], true);
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("dimension"),
        "unexpected error text: {}",
        res["content"][0]["text"]
    );

    // Empty inputs array → invalid_params.
    let none = Some(json!({
        "name": "memory_put_batch",
        "arguments": {"inputs": []}
    }));
    assert!(handle_tools_call(&none, &executor, &storage, &cfg).is_err());
}

// ── MCP-21: GDS via MCP (graph_page_rank + graph_degree_centrality) ──────

/// Builds a directed chain A(1) -> B(2) -> C(3) through the agent channel
/// (query_iql INSERT + RELATE), the same round-trip proven by MCP-27.
fn build_chain(storage: &Arc<StorageEngine>, executor: &Executor<'_>, ids: [u128; 3]) {
    for id in ids {
        let params = Some(json!({
            "name": "query_iql",
            "arguments": { "query": format!("INSERT NODE#{id} TYPE GdsNode {{ label: \"n{id}\" }}") }
        }));
        let res = handle_tools_call(&params, executor, storage, &default_config())
            .expect("iql insert ok");
        assert!(
            res["isError"].is_null(),
            "INSERT NODE#{id} failed: {}",
            res["content"][0]["text"]
        );
    }
    let (a, b, c) = (ids[0], ids[1], ids[2]);
    for (src, dst) in [(a, b), (b, c)] {
        let params = Some(json!({
            "name": "query_iql",
            "arguments": { "query": format!("RELATE NODE#{src} --\"next\"--> NODE#{dst}") }
        }));
        let res = handle_tools_call(&params, executor, storage, &default_config())
            .expect("iql relate ok");
        assert!(
            res["isError"].is_null(),
            "RELATE {src}->{dst} failed: {}",
            res["content"][0]["text"]
        );
    }
}

#[test]
fn test_mcp_graph_page_rank_converges_on_chain() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    build_chain(&storage, &executor, [1, 2, 3]);

    let params = Some(json!({
        "name": "graph_page_rank",
        "arguments": { "roots": ["1"] }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_ok(), "graph_page_rank should succeed");
    let val = res.unwrap();
    assert!(
        val["isError"].is_null(),
        "page_rank failed: {}",
        val["content"][0]["text"]
    );

    let text = val["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    let scores = parsed["scores"].as_object().expect("scores object");

    // All three nodes of the chain are discovered and ranked; u128 ids are
    // serialized as decimal strings.
    assert_eq!(scores.len(), 3, "all chain nodes ranked: {:?}", scores);
    assert!(scores.contains_key("1") && scores.contains_key("2") && scores.contains_key("3"));

    // Standard PageRank with dangling redistribution sums to ~1.0.
    let sum: f64 = scores.values().map(|v| v.as_f64().unwrap()).sum();
    assert!(
        (sum - 1.0).abs() < 0.01,
        "ranks should sum to ~1.0, got {sum}: {:?}",
        scores
    );

    // Chain A->B->C: the dangling leaf C accumulates the most mass.
    let r1 = scores["1"].as_f64().unwrap();
    let r3 = scores["3"].as_f64().unwrap();
    assert!(r3 > r1, "leaf C ({r3}) should outrank source A ({r1})");
}

#[test]
fn test_mcp_graph_degree_centrality_chain_counts() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    build_chain(&storage, &executor, [11, 12, 13]);

    let params = Some(json!({
        "name": "graph_degree_centrality",
        "arguments": { "roots": ["11"] }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &default_config());
    assert!(res.is_ok(), "graph_degree_centrality should succeed");
    let val = res.unwrap();
    assert!(
        val["isError"].is_null(),
        "degree_centrality failed: {}",
        val["content"][0]["text"]
    );

    let text = val["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    let degrees = parsed["degrees"].as_object().expect("degrees object");

    assert!(degrees.contains_key("11"), "A missing: {:?}", degrees);
    assert!(degrees.contains_key("12"), "B missing: {:?}", degrees);
    assert!(degrees.contains_key("13"), "C missing: {:?}", degrees);

    let a = &degrees["11"];
    let b = &degrees["12"];
    let c = &degrees["13"];
    assert_eq!(a["in"].as_u64(), Some(0), "A has no incoming edges");
    assert_eq!(a["out"].as_u64(), Some(1), "A -> B");
    assert_eq!(b["in"].as_u64(), Some(1));
    assert_eq!(b["out"].as_u64(), Some(1));
    assert_eq!(c["in"].as_u64(), Some(1));
    assert_eq!(c["out"].as_u64(), Some(0), "C is the leaf");
}

// ── MCP-22: graph traversal via MCP ─────────────────────────────────────

#[test]
fn test_mcp_graph_traverse_bfs_order_and_dag_analysis() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    build_chain(&storage, &executor, [21, 22, 23]);

    // BFS from A visits A,B,C in breadth order.
    let bfs = Some(json!({
        "name": "graph_traverse",
        "arguments": { "start": ["21"], "mode": "bfs", "max_depth": 10 }
    }));
    let res = handle_tools_call(&bfs, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "bfs failed: {}",
        res["content"][0]["text"]
    );
    let visited: Vec<String> =
        serde_json::from_str::<Value>(res["content"][0]["text"].as_str().unwrap()).unwrap()
            ["visited"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
    assert_eq!(
        visited,
        vec!["21", "22", "23"],
        "BFS order on chain A->B->C"
    );

    // DFS from A also covers all three nodes.
    let dfs = Some(json!({
        "name": "graph_traverse",
        "arguments": { "start": ["21"], "mode": "dfs", "max_depth": 10 }
    }));
    let res = handle_tools_call(&dfs, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "dfs failed: {}",
        res["content"][0]["text"]
    );
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("21") && text.contains("22") && text.contains("23"),
        "DFS should cover the whole chain: {text}"
    );

    // Unknown mode -> self-correctable error content (MEM-32), not Err.
    let bad = Some(json!({
        "name": "graph_traverse",
        "arguments": { "start": ["21"], "mode": "spiral", "max_depth": 10 }
    }));
    let res = handle_tools_call(&bad, &executor, &storage, &default_config()).unwrap();
    assert_eq!(res["isError"], json!(true), "unknown mode must set isError");
}

#[test]
fn test_mcp_graph_topological_sort_and_is_dag() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);
    build_chain(&storage, &executor, [31, 32, 33]);

    // Topo sort of the DAG returns a valid order (A before B before C).
    let topo = Some(json!({
        "name": "graph_topological_sort",
        "arguments": { "roots": ["31"] }
    }));
    let res = handle_tools_call(&topo, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "topo sort failed: {}",
        res["content"][0]["text"]
    );
    let parsed: Value = serde_json::from_str(res["content"][0]["text"].as_str().unwrap()).unwrap();
    let order: Vec<String> = parsed["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let pos = |id: &str| order.iter().position(|x| x == id).expect("node in order");
    assert!(
        pos("31") < pos("32") && pos("32") < pos("33"),
        "valid topo order: {:?}",
        order
    );

    // Chain without cycles IS a DAG...
    let dag = Some(json!({
        "name": "graph_is_dag",
        "arguments": { "roots": ["31"] }
    }));
    let res = handle_tools_call(&dag, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "is_dag failed: {}",
        res["content"][0]["text"]
    );
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("true"), "chain is a DAG: {text}");

    // ...and closing the cycle C->A makes it not a DAG.
    let cycle = Some(json!({
        "name": "query_iql",
        "arguments": { "query": "RELATE NODE#33 --\"back\"--> NODE#31" }
    }));
    let res = handle_tools_call(&cycle, &executor, &storage, &default_config()).unwrap();
    assert!(
        res["isError"].is_null(),
        "cycle edge failed: {}",
        res["content"][0]["text"]
    );

    let dag_after = Some(json!({
        "name": "graph_is_dag",
        "arguments": { "roots": ["31"] }
    }));
    let res = handle_tools_call(&dag_after, &executor, &storage, &default_config()).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("false"), "cycled graph is not a DAG: {text}");
}

// ── MCP-20: index recovery tools (rebuild_index/audit_text_index/repair_text_index) ──

fn recovery_call(
    executor: &Executor<'_>,
    storage: &Arc<StorageEngine>,
    name: &str,
    arguments: Value,
) -> Result<Value, Value> {
    handle_tools_call(
        &Some(json!({ "name": name, "arguments": arguments })),
        executor,
        storage,
        &default_config(),
    )
}

#[test]
fn test_mcp_tools_list_includes_recovery_and_introspection() {
    let res = handle_tools_list().unwrap();
    let tools = res["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    for tool in [
        "rebuild_index",
        "audit_text_index",
        "repair_text_index",
        "capabilities",
        "generate_snippet",
        "list_snapshots",
    ] {
        assert!(names.contains(&tool), "tools should include {tool}");
    }
}

#[test]
fn test_recovery_tools_round_trip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // Two vector records so rebuild has something to index.
    for (key, payload) in [
        ("alpha", "the quick brown fox"),
        ("beta", "jumps over the dog"),
    ] {
        let put = Some(json!({
            "name": "memory_put",
            "arguments": {
                "namespace": "recover_ns",
                "key": key,
                "payload": payload,
                "vector": [0.1, 0.2, 0.3, 0.4]
            }
        }));
        let val = handle_tools_call(&put, &executor, &storage, &default_config()).unwrap();
        assert!(
            val["isError"].is_null(),
            "put {key} failed: {}",
            val["content"][0]["text"]
        );
    }

    // Tests open a raw StorageEngine (server calls ensure_indexes_current at
    // startup); build the text index before auditing BM25-derived state.
    let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
    embedded
        .ensure_indexes_current()
        .expect("ensure_indexes_current must succeed");

    // 1. rebuild_index → report with counts > 0 and success.
    let rb_val = recovery_call(&executor, &storage, "rebuild_index", json!({})).unwrap();
    assert!(
        rb_val["isError"].is_null(),
        "rebuild_index failed: {}",
        rb_val["content"][0]["text"]
    );
    let rb: Value = serde_json::from_str(rb_val["content"][0]["text"].as_str().unwrap())
        .expect("rebuild_index payload must be JSON");
    assert_eq!(rb["success"], true, "rebuild must succeed: {rb}");
    assert!(
        rb["scanned_nodes"].as_u64().unwrap_or(0) >= 2,
        "scanned_nodes must count the two records: {rb}"
    );
    assert!(
        rb["indexed_vectors"].as_u64().unwrap_or(0) >= 2,
        "indexed_vectors must index both vectors: {rb}"
    );

    // 2. audit_text_index on a clean namespace → no inconsistencies.
    let audit_val = recovery_call(
        &executor,
        &storage,
        "audit_text_index",
        json!({ "namespace": "recover_ns" }),
    )
    .unwrap();
    assert!(
        audit_val["isError"].is_null(),
        "audit_text_index failed: {}",
        audit_val["content"][0]["text"]
    );
    let audit: Value = serde_json::from_str(audit_val["content"][0]["text"].as_str().unwrap())
        .expect("audit payload must be JSON");
    assert_eq!(
        audit["passed"], true,
        "clean namespace must pass audit: {audit}"
    );
    assert_eq!(audit["status"], "ok", "clean namespace status: {audit}");
    assert_eq!(
        audit["mismatches"], 0,
        "no mismatches expected after rebuild: {audit}"
    );

    // 3. repair_text_index → repair report with success.
    let rep_val = recovery_call(&executor, &storage, "repair_text_index", json!({})).unwrap();
    assert!(
        rep_val["isError"].is_null(),
        "repair_text_index failed: {}",
        rep_val["content"][0]["text"]
    );
    let rep: Value = serde_json::from_str(rep_val["content"][0]["text"].as_str().unwrap())
        .expect("repair payload must be JSON");
    assert_eq!(rep["success"], true, "repair must succeed: {rep}");
    assert!(
        rep["record_count"].as_u64().unwrap_or(0) >= 2,
        "repair must reindex both records: {rep}"
    );
}

// ── MCP-26: capabilities / generate_snippet / list_snapshots ──

#[test]
fn test_capabilities_generate_snippet_list_snapshots_round_trip() {
    let (_dir, storage) = setup_storage();
    let executor = Executor::new(&storage);

    // capabilities → object with the fields of VantaCapabilities.
    let caps_val = recovery_call(&executor, &storage, "capabilities", json!({})).unwrap();
    assert!(
        caps_val["isError"].is_null(),
        "capabilities failed: {}",
        caps_val["content"][0]["text"]
    );
    let caps: Value = serde_json::from_str(caps_val["content"][0]["text"].as_str().unwrap())
        .expect("capabilities payload must be JSON");
    assert!(
        caps["runtime_profile"].is_string(),
        "runtime_profile: {caps}"
    );
    assert_eq!(caps["persistence"], true, "embedded DB persists: {caps}");
    assert_eq!(caps["vector_search"], true, "{caps}");
    assert_eq!(caps["iql_queries"], true, "{caps}");
    assert_eq!(caps["read_only"], false, "{caps}");

    // generate_snippet → snippet present for a matching term query.
    let snip_val = recovery_call(
        &executor,
        &storage,
        "generate_snippet",
        json!({
            "payload": "the quick brown fox jumps over the lazy dog",
            "text_query": "fox"
        }),
    )
    .unwrap();
    assert!(
        snip_val["isError"].is_null(),
        "generate_snippet failed: {}",
        snip_val["content"][0]["text"]
    );
    let snip: Value = serde_json::from_str(snip_val["content"][0]["text"].as_str().unwrap())
        .expect("snippet payload must be JSON");
    assert!(
        snip["snippet"].as_str().is_some(),
        "matching query must yield a snippet: {snip}"
    );

    // generate_snippet with no query terms → null handled without error.
    let none_val = recovery_call(
        &executor,
        &storage,
        "generate_snippet",
        json!({ "payload": "anything", "text_query": "   " }),
    )
    .unwrap();
    assert!(
        none_val["isError"].is_null(),
        "empty-query generate_snippet must not error: {}",
        none_val["content"][0]["text"]
    );
    let none: Value = serde_json::from_str(none_val["content"][0]["text"].as_str().unwrap())
        .expect("payload must be JSON");
    assert!(
        none["snippet"].is_null(),
        "empty query terms → null: {none}"
    );

    // list_snapshots → array (empty on a fresh DB).
    let snaps_val = recovery_call(&executor, &storage, "list_snapshots", json!({})).unwrap();
    assert!(
        snaps_val["isError"].is_null(),
        "list_snapshots failed: {}",
        snaps_val["content"][0]["text"]
    );
    let snaps: Value = serde_json::from_str(snaps_val["content"][0]["text"].as_str().unwrap())
        .expect("snapshots payload must be JSON");
    assert!(
        snaps["snapshots"].is_array(),
        "list_snapshots must return an array: {snaps}"
    );
}
