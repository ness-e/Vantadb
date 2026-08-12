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
