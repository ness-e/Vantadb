use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn setup() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
    (dir, Arc::new(storage))
}

#[test]
fn embed_texts_basic() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();

    // test handle_tools_list contains embed_texts
    let list = handle_tools_list().unwrap();
    let tools = list["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(
        names.contains(&"embed_texts"),
        "tools/list missing embed_texts: {:?}",
        names
    );
    // check inputSchema
    let embed_tool = tools.iter().find(|t| t["name"] == "embed_texts").unwrap();
    assert_eq!(embed_tool["inputSchema"]["required"][0], "texts");

    // call embed_texts
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": ["hola", "hello world"]
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    // should be text_content with JSON
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(val["count"], 2);
    assert_eq!(val["dim"], 384);
    let embeddings = val["embeddings"].as_array().unwrap();
    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].as_array().unwrap().len(), 384);
    assert!(val["next_cursor"].is_null());
    assert_eq!(val["truncated"], false);
}

#[test]
fn embed_texts_with_model_param() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": ["hola mundo"],
            "model": "multilingual-e5-small"
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(val["count"], 1);
    assert_eq!(val["model"], "multilingual-e5-small");
}

#[test]
fn embed_texts_budgeting_truncation() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    // set small budget to force truncation
    let mut cfg = McpConfig::default();
    cfg.max_embed_tokens = 20; // each text ~7 tokens (len 28/4), => 2 fit (14), third would exceed
    cfg.max_embed_batch_size = 2;
    // Create 5 texts, each ~28 chars => ~7 tokens
    let texts: Vec<String> = (0..5)
        .map(|i| format!("text number {} with some words", i))
        .collect();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": texts.clone(),
            "cursor": 0
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    // should be truncated (batch size 2)
    assert_eq!(val["truncated"], true);
    assert_eq!(val["count"], 2);
    assert_eq!(val["next_cursor"], 2);
    // second page
    let params2 = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": texts.clone(),
            "cursor": 2
        }
    }));
    let res2 = handle_tools_call(&params2, &executor, &storage, &cfg).unwrap();
    let text2 = res2["content"][0]["text"].as_str().unwrap();
    let val2: serde_json::Value = serde_json::from_str(text2).unwrap();
    assert_eq!(val2["count"], 2);
    assert_eq!(val2["next_cursor"], 4);
}

#[test]
fn embed_texts_rejects_empty() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": []
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg);
    assert!(res.is_err());
}

#[test]
fn embed_texts_rejects_missing_texts() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "model": "foo"
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg);
    assert!(res.is_err());
}
