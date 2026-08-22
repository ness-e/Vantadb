//! MEM-33 — D19 tests for the `wiki_*` MCP tools.
//!
//! Contract: 4 query-only tools (`wiki_search/read/list/graph`) over a
//! seeded `WikiStore` (MEM-28): (a) search ranks BM25-style with title
//! weight; (b) read surfaces `locked:true` as visible metadata; (c) list;
//! (d) graph BFS multi-hop capped at 200 nodes; (e) a wiki not in `ready`
//! state yields a clear error carrying the current state; (f) strict
//! read-only (no mutation of pages or lifecycle records).

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb::wiki::{WikiState, WikiStore};
use vantadb_mcp::{handle_tools_call, handle_tools_list, McpConfig};

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine");
    (dir, Arc::new(storage))
}

fn call(
    name: &str,
    args: Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let executor = Executor::new(storage);
    handle_tools_call(
        &Some(json!({ "name": name, "arguments": args })),
        &executor,
        storage,
        config,
    )
}

/// Message of a tool result (error_content text or JSON-RPC error message).
fn msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

/// Seed a ready wiki with the given `(page_type, title, content)` pages.
/// Returns the wiki store and the seeded canonical paths.
fn seed_wiki(storage: &StorageEngine, pages: &[(&str, &str, &str)]) -> Vec<String> {
    let store = WikiStore::new(storage);
    let wiki = store.create("testns", "main").expect("create");
    assert_eq!(wiki.state, WikiState::Pending);
    let wiki = store.begin_processing("testns", "main").expect("begin");
    for (ptype, title, content) in pages {
        store
            .put_page("testns", "main", ptype, title, content)
            .expect("put_page");
    }
    store
        .complete("testns", "main", wiki.run_id.as_deref().unwrap())
        .expect("complete");
    pages
        .iter()
        .map(|(ptype, title, _)| vantadb::wiki::canonical_path(ptype, title))
        .collect()
}

#[test]
fn test_all_four_wiki_tools_listed() {
    let list = handle_tools_list().expect("tools/list");
    let names: Vec<&str> = list["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    for tool in ["wiki_search", "wiki_read", "wiki_list", "wiki_graph"] {
        assert!(names.contains(&tool), "{tool} should be listed");
    }
}

/// (a) search ranks: title matches beat body-only mentions.
#[test]
fn test_wiki_search_ranks_title_above_body() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    seed_wiki(
        &storage,
        &[
            ("concept", "Vector Database Overview", "a short note"),
            (
                "person",
                "Gardening Guide",
                "mentions vector database once in passing",
            ),
            ("recipe", "Cooking Basics", "boil water add pasta"),
        ],
    );
    let paths = [
        canonical("concept", "Vector Database Overview"),
        canonical("person", "Gardening Guide"),
        canonical("recipe", "Cooking Basics"),
    ];

    let text = msg(call(
        "wiki_search",
        json!({"namespace": "testns", "slug": "main", "query": "vector database"}),
        &storage,
        &cfg,
    ));
    assert!(!text.contains("Error"), "no error on ready wiki: {text}");
    let v: Value = serde_json::from_str(&text).expect("json");
    let results = v["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2, "only matching docs ranked: {text}");
    assert_eq!(
        results[0]["path"], paths[0],
        "title match ranked first: {text}"
    );
    assert_eq!(results[1]["path"], paths[1], "body mention second: {text}");
    // Ranking is meaningful: title hit scores strictly higher.
    let top: f64 = results[0]["score"].as_f64().unwrap();
    let second: f64 = results[1]["score"].as_f64().unwrap();
    assert!(top > second, "title weight must outrank body hit");

    // No-match query → empty result set, not an error.
    let none = msg(call(
        "wiki_search",
        json!({"namespace": "testns", "slug": "main", "query": "quantum"}),
        &storage,
        &cfg,
    ));
    let nv: Value = serde_json::from_str(&none).unwrap();
    assert_eq!(nv["results"].as_array().unwrap().len(), 0, "{none}");
}

fn canonical(page_type: &str, title: &str) -> String {
    vantadb::wiki::canonical_path(page_type, title)
}

/// (b) read respects `locked:true` as visible metadata + missing page error.
#[test]
fn test_wiki_read_surfaces_locked_metadata() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    seed_wiki(
        &storage,
        &[("person", "Alice Smith", "knows about vector databases")],
    );
    let path = canonical("person", "Alice Smith");

    let text = msg(call(
        "wiki_read",
        json!({"namespace": "testns", "slug": "main", "path": path}),
        &storage,
        &cfg,
    ));
    let v: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(v["locked"], true, "locked surfaced as metadata: {text}");
    assert_eq!(v["path"], path);
    assert_eq!(v["title"], "Alice Smith");
    assert!(
        v["content"].as_str().unwrap().contains("vector databases"),
        "content present: {text}"
    );

    let missing = msg(call(
        "wiki_read",
        json!({"namespace": "testns", "slug": "main", "path": "wiki/nope.md"}),
        &storage,
        &cfg,
    ));
    assert!(missing.contains("Page not found"), "{missing}");
}

/// (c) list returns every page ordered by canonical path.
#[test]
fn test_wiki_list_pages() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    seed_wiki(
        &storage,
        &[
            ("concept", "Vector Database Overview", "note"),
            ("person", "Bob Jones", "gardening fan"),
            ("recipe", "Cooking Basics", "pasta"),
        ],
    );

    let text = msg(call(
        "wiki_list",
        json!({"namespace": "testns", "slug": "main"}),
        &storage,
        &cfg,
    ));
    let v: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(v["count"], 3, "{text}");
    let pages: Vec<&str> = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["path"].as_str().unwrap())
        .collect();
    let mut sorted = pages.clone();
    sorted.sort_unstable();
    assert_eq!(pages, sorted, "ordered by canonical path");
    assert!(v["pages"][0]["locked"] == true, "list exposes locked flag");
}

/// (d) graph BFS multi-hop with hard cap 200 visited nodes.
#[test]
fn test_wiki_graph_bfs_multi_hop_cap_200() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();

    // Hub topology: Root → [[Hub]] → [[Leaf i]] ×300. Default max_hops=2:
    // root(hop0) → hub(hop1) → leaves(hop2); node cap truncates at 200.
    let mut leaf_links = String::new();
    for i in 0..300 {
        leaf_links.push_str(&format!("[[Leaf {i}]] "));
    }
    let mut pages = vec![
        ("concept", "Root", "[[Hub]]"),
        ("concept", "Hub", leaf_links.as_str()),
    ];
    for i in 0..300 {
        pages.push((
            "concept",
            Box::leak(format!("Leaf {i}").into_boxed_str()),
            "",
        ));
    }
    seed_wiki(&storage, &pages);

    let text = msg(call(
        "wiki_graph",
        json!({"namespace": "testns", "slug": "main", "root_path": canonical("concept", "Root")}),
        &storage,
        &cfg,
    ));
    let v: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(v["visited"], 200, "hard cap 200 nodes: {text}");
    assert_eq!(v["truncated"], true, "cap reached → truncated flag");
    assert_eq!(v["node_cap"], 200);

    // Multi-hop respected: hub at hop 1, leaves at hop 2, nothing beyond.
    let nodes = v["nodes"].as_array().unwrap();
    assert!(
        nodes.iter().any(|n| n["title"] == "Hub" && n["hop"] == 1),
        "hub at hop 1: {text}"
    );
    assert!(
        nodes.iter().any(|n| n["hop"] == 2),
        "leaves reached at hop 2: {text}"
    );
    assert!(
        !nodes.iter().any(|n| n["hop"].as_u64().unwrap_or(0) > 2),
        "max_hops respected"
    );
    assert!(
        !v["edges"].as_array().unwrap().is_empty(),
        "traversed edges reported"
    );

    // Unknown root → clear self-correctable error.
    let missing = msg(call(
        "wiki_graph",
        json!({"namespace": "testns", "slug": "main", "root_path": "wiki/none.md"}),
        &storage,
        &cfg,
    ));
    assert!(missing.contains("Page not found"), "{missing}");
}

/// (e) pending (not ready) → clear error carrying the current state.
#[test]
fn test_wiki_pending_clear_error() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    WikiStore::new(&storage)
        .create("testns", "main")
        .expect("create");

    for (tool, args) in [
        (
            "wiki_search",
            json!({"namespace": "testns", "slug": "main", "query": "x"}),
        ),
        (
            "wiki_read",
            json!({"namespace": "testns", "slug": "main", "path": "wiki/a/b.md"}),
        ),
        ("wiki_list", json!({"namespace": "testns", "slug": "main"})),
        (
            "wiki_graph",
            json!({"namespace": "testns", "slug": "main", "root_path": "wiki/a/b.md"}),
        ),
    ] {
        let text = msg(call(tool, args, &storage, &cfg));
        assert!(
            text.contains("wiki not ready"),
            "{tool} refuses non-ready wiki: {text}"
        );
        assert!(
            text.contains("pending"),
            "{tool} reports current state: {text}"
        );
    }

    // Nonexistent wiki also fails clearly.
    let missing = msg(call(
        "wiki_list",
        json!({"namespace": "nope", "slug": "main"}),
        &storage,
        &cfg,
    ));
    assert!(missing.contains("not found"), "{missing}");
}

/// (f) read-only estricto: all four tools leave page set + lifecycle record untouched.
#[test]
fn test_wiki_tools_are_read_only() {
    let (_dir, storage) = setup_storage();
    let cfg = McpConfig::default();
    seed_wiki(
        &storage,
        &[
            ("concept", "Vector Database Overview", "[[Bob Jones]]"),
            ("person", "Bob Jones", "gardening fan"),
        ],
    );
    let root = canonical("concept", "Vector Database Overview");
    let scope = json!({"namespace": "testns", "slug": "main"});

    let fingerprint = || -> String { msg(call("wiki_list", scope.clone(), &storage, &cfg)) };
    let before = fingerprint();
    let version_before = WikiStore::new(&storage)
        .get("testns", "main")
        .unwrap()
        .unwrap()
        .version;

    call(
        "wiki_search",
        json!({"namespace": "testns", "slug": "main", "query": "vector"}),
        &storage,
        &cfg,
    )
    .unwrap();
    call(
        "wiki_read",
        json!({"namespace": "testns", "slug": "main", "path": root}),
        &storage,
        &cfg,
    )
    .unwrap();
    call(
        "wiki_graph",
        json!({"namespace": "testns", "slug": "main", "root_path": root}),
        &storage,
        &cfg,
    )
    .unwrap();

    let after = fingerprint();
    assert_eq!(before, after, "page set unchanged after all reads");
    let wiki_after = WikiStore::new(&storage)
        .get("testns", "main")
        .unwrap()
        .unwrap();
    assert_eq!(
        wiki_after.version, version_before,
        "lifecycle record untouched"
    );
    assert_eq!(wiki_after.state, WikiState::Ready, "state still ready");
}
