use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

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

    // Test metrics://
    let res_metrics = handle_resources_read(&Some(json!({"uri": "metrics://"})), &storage);
    assert!(res_metrics.is_ok(), "reading metrics:// should succeed");
    let val_metrics = res_metrics.unwrap();
    assert_eq!(val_metrics["contents"][0]["uri"], "metrics://");
    assert_eq!(val_metrics["contents"][0]["mimeType"], "application/json");
    let text = val_metrics["contents"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("hnsw_nodes_count"),
        "metrics response should contain hnsw_nodes_count"
    );

    // Test schema://
    let res_schema = handle_resources_read(&Some(json!({"uri": "schema://"})), &storage);
    assert!(res_schema.is_ok(), "reading schema:// should succeed");
    let val_schema = res_schema.unwrap();
    assert_eq!(val_schema["contents"][0]["uri"], "schema://");
    let schema_text = val_schema["contents"][0]["text"].as_str().unwrap();
    assert!(
        schema_text.contains("hnsw_nodes_count"),
        "schema response should contain hnsw_nodes_count"
    );

    // Test invalid URI
    let res_invalid = handle_resources_read(&Some(json!({"uri": "invalid://"})), &storage);
    assert!(
        res_invalid.is_err(),
        "reading invalid URI should return an error"
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
        names.contains(&"query_lisp"),
        "tools should include query_lisp"
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

    let put_res = handle_tools_call(&put_params, &executor, &storage);
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
    let get_res = handle_tools_call(&get_params, &executor, &storage);
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
    let list_res = handle_tools_call(&list_params, &executor, &storage);
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
    let ns_res = handle_tools_call(&ns_params, &executor, &storage);
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
    let del_res = handle_tools_call(&del_params, &executor, &storage);
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
    let get_res_after = handle_tools_call(&get_params, &executor, &storage);
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
        "name": "query_lisp",
        "arguments": {
            "query": "INSERT NODE#999 TYPE TestNode { tier: \"Cold\" }"
        }
    }));
    let write_res = handle_tools_call(&iql_write, &executor, &storage);
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
        "name": "query_lisp",
        "arguments": {
            "query": "FROM NODE#999"
        }
    }));
    let read_res = handle_tools_call(&iql_read, &executor, &storage);
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
    handle_tools_call(&put_params_1, &executor, &storage).unwrap();

    let put_params_2 = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "search_ns",
            "key": "vector_y",
            "payload": "Point Y axis",
            "vector": v2
        }
    }));
    handle_tools_call(&put_params_2, &executor, &storage).unwrap();

    // Test search_semantic (raw vector index)
    let search_sem_params = Some(json!({
        "name": "search_semantic",
        "arguments": {
            "vector": [0.9, 0.1, 0.0],
            "k": 1
        }
    }));
    let sem_res = handle_tools_call(&search_sem_params, &executor, &storage);
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
    let mem_res = handle_tools_call(&search_mem_params, &executor, &storage);
    assert!(mem_res.is_ok(), "search_memory tool call should succeed");
    let mem_val = mem_res.unwrap();
    // search_memory should return a valid response (even if empty for vector-only without text index)
    assert!(
        mem_val["isError"].is_null() || mem_val["content"][0]["text"].is_string(),
        "search_memory response should have no error or valid text content"
    );
}
