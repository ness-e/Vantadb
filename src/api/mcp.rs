use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use crate::storage::StorageEngine;
use crate::executor::{Executor, ExecutionResult};

#[derive(Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn error_code(code: i32, message: &str) -> Result<Value, Value> {
    Err(json!({"code": code, "message": message}))
}

pub async fn run_stdio_server(storage: Arc<StorageEngine>) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let executor = Executor::new(&storage);

    // Bucle Stdio principal (JSON-RPC)
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        
        if line.trim().is_empty() { continue; }

        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[MCP] Error (stderr): Unparseable JSON-RPC: {}", e);
                let err_res = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                if let Ok(out) = serde_json::to_string(&err_res) {
                    writeln!(stdout, "{}", out).unwrap();
                    stdout.flush().unwrap();
                }
                continue;
            }
        };

        if req.jsonrpc != "2.0" { continue; }

        let res = match req.method.as_str() {
            "initialize" => handle_initialize(),
            "tools/list" => handle_tools_list(),
            "tools/call" => handle_tools_call(&req.params, &executor, &storage).await,
            _ => error_code(-32601, "Method not found"),
        };

        let (result, error) = match res {
            Ok(val) => (Some(val), None),
            Err(err) => (None, Some(err)),
        };

        let response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result,
            error,
        };

        if let Ok(out) = serde_json::to_string(&response) {
            writeln!(stdout, "{}", out).unwrap();
            stdout.flush().unwrap();
        }
    }
}

pub fn handle_initialize() -> Result<Value, Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "connectomedb",
            "version": "0.4.0"
        },
        "capabilities": {
            "tools": {}
        }
    }))
}

pub fn handle_tools_list() -> Result<Value, Value> {
    Ok(json!({
        "tools": [
            {
                "name": "query_lisp",
                "description": "Ejecuta código NeuLISP. Permite leer estructuras e insertar/mutar STNeurons aportando entropía semántica.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Programa o sentencia en NeuLISP" }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "search_semantic",
                "description": "Búsqueda vectorial semántica cruda directamente en el índice HNSW.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "vector": { "type": "array", "items": {"type": "number"}, "description": "Vector F32 de consulta" },
                        "k": { "type": "number", "description": "Top K vecinos" }
                    },
                    "required": ["vector", "k"]
                }
            },
            {
                "name": "get_node_neighbors",
                "description": "Inspecciona vecinos o linaje arqueológico de un nodo (Onírico o Shadow).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "number", "description": "ID del Nodo a explorar" }
                    },
                    "required": ["node_id"]
                }
            },
            {
                "name": "inject_context",
                "description": "Inyecta estado o contexto externo conectándolo a un hilo específico para consolidación posterior (Neural Summarization).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Contenido del contexto" },
                        "thread_id": { "type": "number", "description": "ID del hilo al que pertenece" }
                    },
                    "required": ["content", "thread_id"]
                }
            },
            {
                "name": "read_axioms",
                "description": "Retorna los Axiomas (Iron Axioms) del Devil's Advocate activos en la base de datos.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        ]
    }))
}

pub async fn handle_tools_call(params: &Option<Value>, executor: &Executor<'_>, storage: &StorageEngine) -> Result<Value, Value> {
    let p = params.as_ref().ok_or_else(|| json!({"code": -32602, "message": "Missing params"}))?;
    let name = p["name"].as_str().unwrap_or("");
    let args = &p["arguments"];

    match name {
        "query_lisp" => {
            let query = args["query"].as_str().unwrap_or("");
            match executor.execute_hybrid(query).await {
                Ok(ExecutionResult::Read(nodes)) => {
                    let content = serde_json::to_string(&nodes).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(ExecutionResult::Write { affected_nodes, message, node_id }) => {
                    let content = serde_json::to_string(&json!({
                        "affected_nodes": affected_nodes,
                        "message": message,
                        "node_id": node_id
                    })).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(ExecutionResult::StaleContext(summary_id)) => {
                    let content = serde_json::to_string(&json!({
                        "stale_context": true,
                        "rehydration_available": true,
                        "summary_id": summary_id,
                        "message": "Arqueología Semántica sugerida (TrustScore Crítico)."
                    })).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => {
                    Ok(json!({"isError": true, "content": [{"type": "text", "text": format!("LISP Runtime Error: {}", e)}]}))
                }
            }
        }
        "search_semantic" => {
            let vec_arr = args["vector"].as_array().ok_or_else(|| json!({"code": -32602, "message": "Missing 'vector' array"}))?;
            let mut vector = Vec::new();
            for v in vec_arr { vector.push(v.as_f64().unwrap_or(0.0) as f32); }
            let k = args["k"].as_i64().unwrap_or(5) as usize;
            
            let mut results = Vec::new();
            if let Ok(index) = storage.hnsw.read() {
                let neighbors = index.search_nearest(&vector, 0, k);
                for (id, distance) in neighbors {
                    if let Ok(Some(node)) = storage.get(id) {
                        results.push(json!({"id": id, "distance": distance, "node": node}));
                    }
                }
            }
            let content = serde_json::to_string(&results).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": content}]}))
        }
        "get_node_neighbors" => {
            let node_id = args["node_id"].as_u64().ok_or_else(|| json!({"code": -32602, "message": "Missing 'node_id"}))?;
            
            if let Ok(Some(node)) = storage.get(node_id) {
                let mut neighbors = Vec::new();
                for edge in &node.edges {
                    if let Ok(Some(target_node)) = storage.get(edge.target) {
                        neighbors.push(json!({
                            "rel": edge.label,
                            "target_id": edge.target,
                            "target_trust": target_node.trust_score,
                            "target_valence": target_node.semantic_valence
                        }));
                    }
                }
                let content = serde_json::to_string(&json!({"node": node, "neighbors": neighbors})).unwrap_or_default();
                Ok(json!({"content": [{"type": "text", "text": content}]}))
            } else {
                Ok(json!({"isError": true, "content": [{"type": "text", "text": "Node not found"}]}))
            }
        }
        "inject_context" => {
            let content = args["content"].as_str().ok_or_else(|| json!({"code": -32602, "message": "Missing 'content'"}))?;
            let thread_id = args["thread_id"].as_u64().ok_or_else(|| json!({"code": -32602, "message": "Missing 'thread_id'"}))?;
            
            let query = format!("INSERT MESSAGE SYSTEM \"{}\" TO THREAD#{}", content, thread_id);
            match executor.execute_hybrid(&query).await {
                Ok(ExecutionResult::Write { affected_nodes, message, .. }) => {
                    let out = serde_json::to_string(&json!({
                        "affected_nodes": affected_nodes,
                        "message": message,
                        "status": "Context Anchored"
                    })).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": out}]}))
                }
                Ok(_) => Ok(json!({"isError": true, "content": [{"type": "text", "text": "Unexpected read result for insert"}]})),
                Err(e) => Ok(json!({"isError": true, "content": [{"type": "text", "text": format!("Execution Error: {}", e)}]}))
            }
        }
        "read_axioms" => {
            let axioms = json!([
                {"id": 1, "name": "Axioma Topológico", "description": "No se permiten referencias (edges) a nodos huérfanos o en el Shadow Archive."},
                {"id": 2, "name": "Axioma de Confianza", "description": "DevilsAdvocate: Se rechazan mutaciones vectoriales divergentes con alto TrustScore histórico."},
                {"id": 3, "name": "Axioma Inmortal", "description": "SleepWorker: Nodos marcados como PINNED evaden degradación por Olvido Bayesiano."},
                {"id": 4, "name": "Presupuesto de Amígdala", "description": "SleepWorker: Reservado el 5% de memoria para nodos con valencia semántica >= 0.8."}
            ]);
            let content = serde_json::to_string(&axioms).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": content}]}))
        }
        _ => error_code(-32601, "Tool not found"),
    }
}
