use core::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};
use vantadb::config::VantaConfig;
use vantadb::sdk::*;
use vantadb::BackendKind;
use vantadb::VantaError;
use wasm_bindgen::prelude::*;

static TRACING_INIT: AtomicBool = AtomicBool::new(false);

fn init_tracing() {
    if !TRACING_INIT.swap(true, Ordering::Relaxed) {
        tracing_wasm::set_as_global_default();
    }
}

fn to_js_err(e: VantaError) -> JsValue {
    js_sys::Error::new(&e.to_string()).into()
}

fn from_js<T: serde::de::DeserializeOwned>(val: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(val).map_err(|e| js_sys::Error::new(&e.to_string()).into())
}

fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(val).map_err(|e| js_sys::Error::new(&e.to_string()).into())
}

/// Minimal WASM-friendly config that maps to VantaConfig
#[derive(Deserialize)]
#[serde(default)]
struct WasmConfig {
    storage_path: String,
    read_only: bool,
    rss_threshold: f64,
    memory_limit: Option<u64>,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            storage_path: "vantadb_data".to_string(),
            read_only: false,
            rss_threshold: 0.80,
            memory_limit: None,
        }
    }
}

fn build_config(wasm: WasmConfig) -> VantaConfig {
    VantaConfig {
        storage_path: wasm.storage_path,
        read_only: wasm.read_only,
        rss_threshold: wasm.rss_threshold,
        memory_limit: wasm.memory_limit,
        backend_kind: BackendKind::InMemory,
        ..VantaConfig::default()
    }
}

/// Serializable wrapper for VantaMemoryInput
#[derive(Serialize, Deserialize)]
struct MemoryInput {
    namespace: String,
    key: String,
    payload: String,
    #[serde(default)]
    metadata: VantaMemoryMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_ms: Option<u64>,
}

/// Search request
#[derive(Serialize, Deserialize)]
struct SearchRequest {
    namespace: String,
    query_vector: Vec<f32>,
    #[serde(default)]
    filters: VantaMemoryMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_query: Option<String>,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default = "default_distance")]
    distance_metric: String,
    #[serde(default)]
    explain: bool,
}

fn default_top_k() -> usize { 10 }
fn default_distance() -> String { "Cosine".to_string() }

#[derive(Serialize, Deserialize)]
struct ListOptions {
    #[serde(default)]
    filters: VantaMemoryMetadata,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<usize>,
}

fn default_limit() -> usize { 100 }

#[wasm_bindgen]
pub struct VantaDB {
    inner: VantaEmbedded,
}

#[wasm_bindgen]
impl VantaDB {
    #[wasm_bindgen(constructor)]
    pub fn new(config_val: Option<JsValue>) -> Result<VantaDB, JsValue> {
        init_tracing();
        let wasm_cfg = match config_val {
            Some(val) => from_js::<WasmConfig>(val)?,
            None => WasmConfig::default(),
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        Ok(VantaDB { inner })
    }

    pub fn open(path: &str) -> Result<VantaDB, JsValue> {
        init_tracing();
        let wasm_cfg = WasmConfig {
            storage_path: path.to_string(),
            ..WasmConfig::default()
        };
        let config = build_config(wasm_cfg);
        let inner = VantaEmbedded::open_with_config(config).map_err(to_js_err)?;
        Ok(VantaDB { inner })
    }

    pub fn close(&self) -> Result<(), JsValue> {
        self.inner.close().map_err(to_js_err)
    }

    pub fn capabilities(&self) -> Result<JsValue, JsValue> {
        let caps = self.inner.capabilities();
        to_js(&caps)
    }

    pub fn put(&self, input: JsValue) -> Result<JsValue, JsValue> {
        let input: MemoryInput = from_js(input)?;
        let vanta_input = VantaMemoryInput {
            namespace: input.namespace,
            key: input.key,
            payload: input.payload,
            metadata: input.metadata,
            vector: input.vector,
            ttl_ms: input.ttl_ms,
        };
        let record = self.inner.put(vanta_input).map_err(to_js_err)?;
        to_js(&record)
    }

    pub fn put_batch(&self, inputs: JsValue) -> Result<JsValue, JsValue> {
        let inputs: Vec<MemoryInput> = from_js(inputs)?;
        let vanta_inputs: Vec<VantaMemoryInput> = inputs
            .into_iter()
            .map(|i| VantaMemoryInput {
                namespace: i.namespace,
                key: i.key,
                payload: i.payload,
                metadata: i.metadata,
                vector: i.vector,
                ttl_ms: i.ttl_ms,
            })
            .collect();
        let records = self.inner.put_batch(vanta_inputs).map_err(to_js_err)?;
        to_js(&records)
    }

    pub fn get(&self, namespace: &str, key: &str) -> Result<JsValue, JsValue> {
        let record = self.inner.get(namespace, key).map_err(to_js_err)?;
        to_js(&record)
    }

    pub fn delete(&self, namespace: &str, key: &str) -> Result<bool, JsValue> {
        self.inner.delete(namespace, key).map_err(to_js_err)
    }

    pub fn list_namespaces(&self) -> Result<JsValue, JsValue> {
        let nss = self.inner.list_namespaces().map_err(to_js_err)?;
        to_js(&nss)
    }

    pub fn list(&self, namespace: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let opts: ListOptions = from_js(options)?;
        let vanta_opts = VantaMemoryListOptions {
            filters: opts.filters,
            limit: opts.limit,
            cursor: opts.cursor,
        };
        let page = self.inner.list(namespace, vanta_opts).map_err(to_js_err)?;
        to_js(&page)
    }

    pub fn search(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let req: SearchRequest = from_js(request)?;
        let distance = match req.distance_metric.as_str() {
            "Euclidean" => vantadb::DistanceMetric::Euclidean,
            _ => vantadb::DistanceMetric::Cosine,
        };
        let vanta_req = VantaMemorySearchRequest {
            namespace: req.namespace,
            query_vector: req.query_vector,
            filters: req.filters,
            text_query: req.text_query,
            top_k: req.top_k,
            distance_metric: distance,
            explain: req.explain,
        };
        let hits = self.inner.search(vanta_req).map_err(to_js_err)?;
        to_js(&hits)
    }

    pub fn search_vector(&self, vector: Vec<f32>, top_k: usize) -> Result<JsValue, JsValue> {
        let hits = self.inner.search_vector(&vector, top_k).map_err(to_js_err)?;
        to_js(&hits)
    }

    pub fn explain_memory_search(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let req: SearchRequest = from_js(request)?;
        let distance = match req.distance_metric.as_str() {
            "Euclidean" => vantadb::DistanceMetric::Euclidean,
            _ => vantadb::DistanceMetric::Cosine,
        };
        let vanta_req = VantaMemorySearchRequest {
            namespace: req.namespace,
            query_vector: req.query_vector,
            filters: req.filters,
            text_query: req.text_query,
            top_k: req.top_k,
            distance_metric: distance,
            explain: true,
        };
        let explanation = self
            .inner
            .explain_memory_search(vanta_req)
            .map_err(to_js_err)?;
        to_js(&explanation)
    }

    pub fn export_namespace(&self, path: &str, namespace: &str) -> Result<JsValue, JsValue> {
        let report = self
            .inner
            .export_namespace(path, namespace)
            .map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn export_all(&self, path: &str) -> Result<JsValue, JsValue> {
        let report = self.inner.export_all(path).map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn import_records(&self, records: JsValue) -> Result<JsValue, JsValue> {
        let records: Vec<VantaMemoryRecord> = from_js(records)?;
        let report = self.inner.import_records(records).map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn import_file(&self, path: &str) -> Result<JsValue, JsValue> {
        let report = self.inner.import_file(path).map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn rebuild_index(&self) -> Result<JsValue, JsValue> {
        let report = self.inner.rebuild_index().map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn compact_layout(&self) -> Result<u64, JsValue> {
        self.inner.compact_layout().map_err(to_js_err)
    }

    pub fn audit_text_index(&self, namespace: Option<String>) -> Result<JsValue, JsValue> {
        let report = self
            .inner
            .audit_text_index(namespace.as_deref())
            .map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn audit_text_index_deep(&self, namespace: Option<String>) -> Result<JsValue, JsValue> {
        let report = self
            .inner
            .audit_text_index_deep(namespace.as_deref())
            .map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn repair_text_index(&self) -> Result<JsValue, JsValue> {
        let report = self.inner.repair_text_index().map_err(to_js_err)?;
        to_js(&report)
    }

    pub fn flush(&self) -> Result<(), JsValue> {
        self.inner.flush().map_err(to_js_err)
    }

    pub fn compact_wal(&self) -> Result<(), JsValue> {
        self.inner.compact_wal().map_err(to_js_err)
    }

    pub fn purge_expired(&self) -> Result<u64, JsValue> {
        self.inner.purge_expired().map_err(to_js_err)
    }

    pub fn operational_metrics(&self) -> Result<JsValue, JsValue> {
        let metrics = self.inner.operational_metrics();
        to_js(&metrics)
    }

    pub fn query(&self, query: &str) -> Result<JsValue, JsValue> {
        let result = self.inner.query(query).map_err(to_js_err)?;
        to_js(&result)
    }

    pub fn insert_node(
        &self,
        id: u64,
        content: Option<String>,
        vector: Option<Vec<f32>>,
        fields: JsValue,
    ) -> Result<(), JsValue> {
        let fields: VantaFields = from_js(fields)?;
        let input = VantaNodeInput {
            id,
            content,
            vector,
            fields,
        };
        self.inner.insert_node(input).map_err(to_js_err)
    }

    pub fn get_node(&self, id: u64) -> Result<JsValue, JsValue> {
        let node = self.inner.get_node(id).map_err(to_js_err)?;
        to_js(&node)
    }

    pub fn delete_node(&self, id: u64, reason: &str) -> Result<(), JsValue> {
        self.inner.delete_node(id, reason).map_err(to_js_err)
    }

    pub fn add_edge(
        &self,
        source_id: u64,
        target_id: u64,
        label: &str,
        weight: Option<f32>,
    ) -> Result<(), JsValue> {
        self.inner
            .add_edge(source_id, target_id, label, weight)
            .map_err(to_js_err)
    }

    pub fn graph_bfs(&self, roots: Vec<u64>, max_depth: usize) -> Result<JsValue, JsValue> {
        let result = self.inner.graph_bfs(&roots, max_depth).map_err(to_js_err)?;
        to_js(&result)
    }

    pub fn graph_dfs(&self, roots: Vec<u64>, max_depth: usize) -> Result<JsValue, JsValue> {
        let result = self.inner.graph_dfs(&roots, max_depth).map_err(to_js_err)?;
        to_js(&result)
    }

    pub fn graph_topological_sort(&self, roots: Vec<u64>) -> Result<JsValue, JsValue> {
        let result = self
            .inner
            .graph_topological_sort(&roots)
            .map_err(to_js_err)?;
        to_js(&result)
    }

    pub fn graph_is_dag(&self, roots: Vec<u64>) -> Result<bool, JsValue> {
        self.inner.graph_is_dag(&roots).map_err(to_js_err)
    }

    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        self.inner
            .generate_snippet(payload, text_query, with_highlighting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    fn create_db() -> VantaDB {
        VantaDB::new(None).expect("failed to create VantaDB")
    }

    #[wasm_bindgen_test]
    fn test_put_and_get() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "hello",
            "payload": "world"
        }))
        .unwrap();
        db.put(input).unwrap();
        let got = db.get("test", "hello").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_get_nonexistent() {
        let db = create_db();
        let got = db.get("nosuch", "nonexistent").unwrap();
        assert!(got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_delete_record() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "todelete",
            "payload": "bye"
        }))
        .unwrap();
        db.put(input).unwrap();
        let deleted = db.delete("test", "todelete").unwrap();
        assert!(deleted);
        let got = db.get("test", "todelete").unwrap();
        assert!(got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_delete_nonexistent() {
        let db = create_db();
        let deleted = db.delete("test", "ghost").unwrap();
        assert!(!deleted);
    }

    #[wasm_bindgen_test]
    fn test_empty_vector_put() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "empty_vec",
            "payload": "no vector",
            "vector": []
        }))
        .unwrap();
        let record = db.put(input).unwrap();
        assert!(!record.is_null());
        let got = db.get("test", "empty_vec").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_put_and_get_with_vector() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "vec_key",
            "payload": "vector data",
            "vector": [0.1, 0.2, 0.3, 0.4]
        }))
        .unwrap();
        db.put(input).unwrap();
        let got = db.get("test", "vec_key").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_large_metadata() {
        let db = create_db();
        let mut meta = serde_json::Map::new();
        for i in 0..100 {
            meta.insert(
                format!("key_{}", i),
                serde_json::Value::String(format!("value_{}", i)),
            );
        }
        let input_val = serde_json::json!({
            "namespace": "test",
            "key": "large_meta",
            "payload": "big metadata payload",
            "metadata": meta
        });
        let input = serde_wasm_bindgen::to_value(&input_val).unwrap();
        db.put(input).unwrap();
        let got = db.get("test", "large_meta").unwrap();
        assert!(!got.is_null());
    }

    #[wasm_bindgen_test]
    fn test_put_batch_empty() {
        let db = create_db();
        let items: Vec<serde_json::Value> = vec![];
        let batch = serde_wasm_bindgen::to_value(&items).unwrap();
        let records = db.put_batch(batch).unwrap();
        assert!(records.is_array());
    }

    #[wasm_bindgen_test]
    fn test_put_batch_multiple() {
        let db = create_db();
        let items: Vec<serde_json::Value> = (0..10)
            .map(|i| {
                serde_json::json!({
                    "namespace": "batch",
                    "key": format!("item_{}", i),
                    "payload": format!("batch item {}", i),
                    "vector": [i as f32 * 0.1, 0.2, 0.3, 0.4]
                })
            })
            .collect();
        let batch = serde_wasm_bindgen::to_value(&items).unwrap();
        db.put_batch(batch).unwrap();
        for i in 0..10 {
            let got = db.get("batch", &format!("item_{}", i)).unwrap();
            assert!(!got.is_null());
        }
    }

    #[wasm_bindgen_test]
    fn test_concurrent_put_get() {
        let db = create_db();
        for i in 0..20 {
            let input = serde_wasm_bindgen::to_value(&serde_json::json!({
                "namespace": "concurrent",
                "key": format!("key_{}", i),
                "payload": format!("data {}", i),
                "vector": [i as f32 * 0.05, 0.1, 0.2, 0.3]
            }))
            .unwrap();
            db.put(input).unwrap();
            let got = db.get("concurrent", &format!("key_{}", i)).unwrap();
            assert!(!got.is_null());
        }
    }

    #[wasm_bindgen_test]
    fn test_capabilities() {
        let db = create_db();
        let caps = db.capabilities().unwrap();
        assert!(!caps.is_null());
    }

    #[wasm_bindgen_test]
    fn test_list_namespaces() {
        let db = create_db();
        let nss = db.list_namespaces().unwrap();
        assert!(nss.is_array());
    }

    #[wasm_bindgen_test]
    fn test_search_without_results() {
        let db = create_db();
        let input = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "key": "only_text",
            "payload": "some text content for text-only search"
        }))
        .unwrap();
        db.put(input).unwrap();
        let req = serde_wasm_bindgen::to_value(&serde_json::json!({
            "namespace": "test",
            "query_vector": [0.1, 0.2, 0.3, 0.4],
            "top_k": 5
        }))
        .unwrap();
        let hits = db.search(req).unwrap();
        assert!(hits.is_array() || hits.is_null());
    }

    #[wasm_bindgen_test]
    fn test_flush_and_compact() {
        let db = create_db();
        db.flush().unwrap();
        db.compact_wal().unwrap();
        let freed = db.compact_layout().unwrap();
        assert_eq!(freed, 0);
    }
}
