use connectomedb::api::mcp::{handle_initialize, handle_tools_list, handle_tools_call};
use connectomedb::storage::StorageEngine;
use connectomedb::executor::Executor;
use connectomedb::node::UnifiedNode;
use serde_json::json;

#[tokio::test]
async fn test_mcp_protocol_standard_responses() {
    // 1. Probamos init
    let init_res = handle_initialize().expect("Debe devolver ok");
    assert_eq!(init_res["protocolVersion"], "2024-11-05");
    assert_eq!(init_res["serverInfo"]["name"], "connectomedb");

    // 2. Probamos tool list
    let list_res = handle_tools_list().expect("Debe devolver ok");
    let tools = list_res["tools"].as_array().expect("Debe ser un array");
    assert!(tools.iter().any(|t| t["name"] == "query_lisp"));
    assert!(tools.iter().any(|t| t["name"] == "search_semantic"));
    assert!(tools.iter().any(|t| t["name"] == "get_node_neighbors"));
}

#[tokio::test]
async fn test_mcp_tool_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap();
    let executor = Executor::new(&storage);

    // Insertar nodo dummy para get_node_neighbors
    let mut node = UnifiedNode::new(Some(100));
    node.trust_score = 0.99;
    node.semantic_valence = 0.5;
    storage.insert(&node).unwrap();

    // 1. Probar llamada get_node_neighbors (Exitosa)
    let params = Some(json!({
        "name": "get_node_neighbors",
        "arguments": {
            "node_id": 100
        }
    }));

    let tool_res = handle_tools_call(&params, &executor, &storage).await.expect("Debería ejecutar tool");
    let content = tool_res["content"].as_array().unwrap();
    assert_eq!(content[0]["type"], "text");
    assert!(content[0]["text"].as_str().unwrap().contains("\"trust_score\":0.99"));

    // 2. Probar tool query_lisp (Insert STNeuron)
    let lisp_params = Some(json!({
        "name": "query_lisp",
        "arguments": {
            "query": "(insert-node (list (tuple \"type\" \"MCP_TEST\")))"
        }
    }));
    let lisp_res = handle_tools_call(&lisp_params, &executor, &storage).await.expect("Debería parsear");
    let content_lisp = lisp_res["content"].as_array().unwrap();
    assert!(content_lisp[0]["text"].as_str().unwrap().contains("affected_nodes"));
}
