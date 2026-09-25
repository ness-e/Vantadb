// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! OpenAPI YAML ↔ Real Implementation Parity Test
//!
//! Validates that docs/api/openapi.yaml accurately reflects the actual
//! implementation in src/parser/mod.rs, src/cli_server.rs, and
//! vantadb-mcp/src/handlers/tools.rs.

#[cfg(test)]
mod openapi_yaml_parity {
    use serde_yaml::Value;
    use std::fs;
    use vantadb::parser::parse_statement;
    use vantadb::query::Condition;

    /// Load and parse the OpenAPI YAML file (lowercase: docs/api/openapi.yaml —
    /// a previous UPPERCASE spelling broke Linux/macOS CI with NotFound).
    fn load_openapi() -> Value {
        let yaml_content =
            fs::read_to_string("docs/api/openapi.yaml").expect("Failed to read openapi.yaml");
        serde_yaml::from_str(&yaml_content).expect("Failed to parse openapi.yaml")
    }

    #[test]
    fn test_iql_text_match_syntax_matches_parser() {
        let openapi = load_openapi();

        // The OpenAPI doc describes IQL vector/hybrid search with `vec(<field>) <~> <text_query>`
        // and text search with `~`. The parser uses `Condition::TextMatch` for `~` without min score.
        // Verify the parser accepts the documented syntax.

        // Test text match syntax: `bio ~ "rust expert"` via parse_statement
        let query = r#"FROM Test WHERE bio ~ "rust expert""#;
        let (_, stmt) =
            parse_statement(query).expect("Parser should accept text match syntax with ~");

        // Extract the condition from the parsed query
        match stmt {
            vantadb::query::Statement::Query(q) => {
                let conditions = q.where_clause.expect("Should have WHERE clause");
                assert_eq!(conditions.len(), 1);
                match &conditions[0] {
                    Condition::TextMatch(field, query) => {
                        assert_eq!(field, "bio");
                        assert_eq!(query, "rust expert");
                    }
                    _ => panic!("Expected TextMatch condition, got {:?}", conditions[0]),
                }
            }
            _ => panic!("Expected Query statement"),
        }

        // Test vector similarity syntax: `bio ~ "rust expert", min = 0.88`
        let query = r#"FROM Test WHERE bio ~ "rust expert", min = 0.88"#;
        let (_, stmt) =
            parse_statement(query).expect("Parser should accept vector similarity syntax");

        match stmt {
            vantadb::query::Statement::Query(q) => {
                let conditions = q.where_clause.expect("Should have WHERE clause");
                assert_eq!(conditions.len(), 1);
                match &conditions[0] {
                    Condition::VectorSim(field, query, min_score) => {
                        assert_eq!(field, "bio");
                        assert_eq!(query, "rust expert");
                        assert!((min_score - 0.88).abs() < 1e-5);
                    }
                    _ => panic!("Expected VectorSim condition, got {:?}", conditions[0]),
                }
            }
            _ => panic!("Expected Query statement"),
        }

        // Verify OpenAPI doesn't incorrectly document `textMatch` as a keyword
        let paths = openapi["paths"]["/api/v2/query"]["post"]["description"]
            .as_str()
            .unwrap();
        // The OpenAPI should use `~` for text match, not `textMatch`
        assert!(
            !paths.contains("textMatch"),
            "OpenAPI incorrectly documents 'textMatch' keyword; IQL uses '~' for text match"
        );
        assert!(
            paths.contains("~"),
            "OpenAPI should document '~' for text match"
        );
    }

    #[test]
    fn test_iql_keywords_are_case_sensitive_uppercase() {
        // GOV-TK3 drift 1: the OpenAPI description documents lowercase IQL
        // (`from <entity> [where …]`, `insert <id> as <type>`) but the nom
        // parser uses case-sensitive `tag("FROM")` etc. — lowercase fails.
        // Live-fire evidence: tasks/GOV-B5.md (lowercase `from`/`insert` → parse error).
        assert!(
            parse_statement("from Test").is_err(),
            "lowercase 'from' must fail: IQL keywords are UPPERCASE"
        );
        assert!(
            parse_statement("FROM Test").is_ok(),
            "uppercase 'FROM' must parse"
        );
        // Canonical UPPERCASE write statements from src/parser/mod.rs must parse.
        for q in [
            r#"INSERT NODE#7 TYPE note {title: "hello"} VECTOR [0.5, 0.5]"#,
            r#"UPDATE NODE#7 SET title = "hi""#,
            r#"DELETE NODE#7"#,
            r#"RELATE NODE#1 --"knows"--> NODE#2 WEIGHT 0.5"#,
            r#"INSERT MESSAGE USER "hello" TO THREAD#9"#,
            r#"FROM Test SIGUE 1..3 "amigo" TYPE Persona AS p"#,
        ] {
            assert!(parse_statement(q).is_ok(), "must parse: {q}");
        }
        // Lowercase write keywords must fail.
        for q in [
            r#"insert 7 as note fields title=hello"#,
            r#"update 7 set title=hi"#,
            r#"relate 1 -> 2 as knows"#,
        ] {
            assert!(parse_statement(q).is_err(), "must fail: {q}");
        }

        // The OpenAPI description must document the UPPERCASE grammar.
        let openapi = load_openapi();
        let query_desc = openapi["paths"]["/api/v2/query"]["post"]["description"]
            .as_str()
            .unwrap();
        assert!(
            query_desc.contains("FROM <entity>"),
            "OpenAPI must document UPPERCASE 'FROM <entity>'"
        );
        assert!(
            query_desc.contains("SIGUE <min>..<max>"),
            "OpenAPI must document 'SIGUE <min>..<max>' (.. range, not -)"
        );
        assert!(
            !query_desc.contains("from <entity> [where"),
            "OpenAPI must not document lowercase 'from <entity> [where'"
        );
        assert!(
            !query_desc.contains("insert <id> as <type>"),
            "OpenAPI must not document lowercase 'insert <id> as <type>'"
        );
    }

    #[test]
    fn test_graph_request_bodies_match_http_handlers() {
        // GOV-TK3 drift 2: the yaml used to expose a single `GraphTraversalBody`
        // (`start: string[]`, `mode`, `direction: outgoing|…` — the MCP
        // `graph_traverse` shape) for all HTTP graph endpoints, but the real
        // HTTP handlers take numeric `roots` + required `max_depth`
        // (`GraphTraversalRequest` in src/server/handlers.rs).
        // Live-fire evidence: tasks/GOV-B5.md (`{"roots":["7"]}` → 400 invalid
        // number; `{"roots":[7]}` → 400 missing `max_depth`).
        let openapi = load_openapi();

        // The drifted shared body must be gone.
        assert!(
            openapi["components"]["requestBodies"]["GraphTraversalBody"].is_null(),
            "drifted GraphTraversalBody must be removed from the yaml"
        );

        // bfs/dfs: numeric roots + required max_depth + optional direction.
        for path in ["/api/v2/graph/bfs", "/api/v2/graph/dfs"] {
            let body_ref = openapi["paths"][path]["post"]["requestBody"]["$ref"]
                .as_str()
                .unwrap();
            assert_eq!(
                body_ref, "#/components/requestBodies/GraphBfsDfsBody",
                "{path} must use GraphBfsDfsBody"
            );
        }
        let schema = &openapi["components"]["requestBodies"]["GraphBfsDfsBody"]["content"]
            ["application/json"]["schema"];
        let required: Vec<&str> = schema["required"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"roots"), "bfs/dfs must require 'roots'");
        assert!(
            required.contains(&"max_depth"),
            "bfs/dfs must require 'max_depth'"
        );
        assert_eq!(
            schema["properties"]["roots"]["items"]["type"]
                .as_str()
                .unwrap(),
            "integer",
            "roots must be NUMERIC (u128 JSON numbers — strings are rejected with 400)"
        );
        let direction_enum: Vec<&str> = schema["properties"]["direction"]["enum"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            direction_enum,
            vec!["forward", "reverse", "both"],
            "direction must be forward|reverse|both (GraphDirection), not outgoing|…"
        );

        // degree/centrality: roots only.
        for path in ["/api/v2/graph/degree", "/api/v2/graph/centrality"] {
            let body_ref = openapi["paths"][path]["post"]["requestBody"]["$ref"]
                .as_str()
                .unwrap();
            assert_eq!(
                body_ref, "#/components/requestBodies/GraphRootsBody",
                "{path} must use GraphRootsBody (roots only, no max_depth)"
            );
        }

        // pagerank: roots + optional tuning params.
        let body_ref = openapi["paths"]["/api/v2/graph/pagerank"]["post"]["requestBody"]["$ref"]
            .as_str()
            .unwrap();
        assert_eq!(
            body_ref, "#/components/requestBodies/GraphPageRankBody",
            "pagerank must use GraphPageRankBody"
        );

        // v2 traversal: STRING roots (u128-safe wire) + required max_depth.
        for path in ["/api/v2/graph/v2/bfs", "/api/v2/graph/v2/dfs"] {
            let body_ref = openapi["paths"][path]["post"]["requestBody"]["$ref"]
                .as_str()
                .unwrap();
            assert_eq!(
                body_ref, "#/components/requestBodies/GraphV2TraversalBody",
                "{path} must use GraphV2TraversalBody"
            );
        }
        let v2_schema = &openapi["components"]["requestBodies"]["GraphV2TraversalBody"]["content"]
            ["application/json"]["schema"];
        assert_eq!(
            v2_schema["properties"]["roots"]["items"]["type"]
                .as_str()
                .unwrap(),
            "string",
            "v2 roots must be decimal-u128 STRINGS"
        );

        // v2 degree: namespace, not node ids.
        let body_ref = openapi["paths"]["/api/v2/graph/v2/degree"]["post"]["requestBody"]["$ref"]
            .as_str()
            .unwrap();
        assert_eq!(
            body_ref, "#/components/requestBodies/GraphV2DegreeBody",
            "v2/degree must use GraphV2DegreeBody (namespace)"
        );
    }

    #[test]
    fn test_search_endpoint_documents_index_ensure() {
        let openapi = load_openapi();

        let search_desc = openapi["paths"]["/api/v2/search"]["post"]["description"]
            .as_str()
            .unwrap();

        // The cli_server.rs:2100-2113 calls ensure_indexes_current() at startup
        // to avoid "text_index not found" on fresh DBs. This should be documented.
        // GOV-TK3 drift 3 (precise): ensure runs ONCE at startup
        // (src/server/bootstrap.rs) and does NOT cover records written
        // afterwards via the record API — fresh DB + record PUT + text search
        // fails with `text_index not found: bm25` until a manual rebuild
        // (live-fire evidence: tasks/GOV-B5.md). The doc must state the
        // condition + symptom + remedy, and must not point at the stale
        // `src/cli_server.rs` (a shim since the 09-01 server split).
        assert!(
            search_desc.contains("text_index not found"),
            "Search endpoint must document the 'text_index not found' symptom on fresh DBs"
        );
        assert!(
            !search_desc.contains("cli_server"),
            "Search endpoint must not reference stale src/cli_server.rs (see src/server/bootstrap.rs)"
        );
        assert!(
            search_desc.contains("ensure_indexes_current") || search_desc.contains("index-rebuilds"),
            "Search endpoint should document that ensure_indexes_current() runs at startup or reference /api/v2/maintenance/index-rebuilds"
        );

        // Also verify the rebuild-index endpoint exists
        let rebuild_desc = openapi["paths"]["/api/v2/maintenance/index-rebuilds"]["post"]
            ["description"]
            .as_str()
            .unwrap();
        assert!(
            rebuild_desc.contains("Rebuilds secondary indexes"),
            "rebuild-index endpoint should exist and be documented"
        );
    }

    // ─── REST-06 / API-03: OpenAPI-first contract invariants ─────────────────
    //
    // The YAML is the contract owner (Gate P, API-STD-15). These tests pin the
    // API-03 standard on the YAML side; the implementation is pinned by
    // `scripts/check_openapi_parity.mjs` (routes/methods) + e2e tests (wire).

    /// Every `next_cursor` anywhere in the yaml must be a single type:
    /// `string` + nullable (never a number / different type).
    fn assert_next_cursor_single_type(value: &Value, ctx: &str) {
        match value {
            Value::Mapping(map) => {
                for (k, v) in map {
                    if k.as_str() == Some("next_cursor") {
                        assert_eq!(
                            v["type"].as_str(),
                            Some("string"),
                            "{ctx}: every `next_cursor` must be type string|null"
                        );
                        assert_eq!(
                            v["nullable"].as_bool(),
                            Some(true),
                            "{ctx}: every `next_cursor` must be nullable"
                        );
                    }
                    assert_next_cursor_single_type(v, ctx);
                }
            }
            Value::Sequence(seq) => {
                for item in seq {
                    assert_next_cursor_single_type(item, ctx);
                }
            }
            _ => {}
        }
    }

    /// Assert the yaml documents a page response with a cursor + has_more.
    fn assert_page_schema(openapi: &Value, schema_name: &str, collection: &str) {
        let schema = &openapi["components"]["schemas"][schema_name];
        assert!(schema.is_mapping(), "page schema {schema_name} must exist");
        assert_eq!(
            schema["properties"]["next_cursor"]["type"].as_str(),
            Some("string"),
            "{schema_name}.next_cursor must be string|null"
        );
        assert_eq!(
            schema["properties"]["has_more"]["type"].as_str(),
            Some("boolean"),
            "{schema_name}.has_more must be a boolean"
        );
        assert!(
            schema["properties"][collection].is_mapping(),
            "{schema_name} must carry the `{collection}` collection"
        );
    }

    /// Assert a path method answers `201` (create) and no longer `200`.
    fn assert_created_status(openapi: &Value, path: &str, method: &str) {
        let responses = &openapi["paths"][path][method]["responses"];
        assert!(
            responses["201"].is_mapping(),
            "{method} {path} must document 201 Created"
        );
        assert!(
            responses["200"].is_null(),
            "{method} {path} must not document a stale 200"
        );
    }

    #[test]
    fn test_rest_routes_are_plural_versioned_and_verb_free() {
        let openapi = load_openapi();

        // Migrated routes (breaking, feat!): verbs/legacy paths are gone.
        assert!(
            openapi["paths"]["/conversation/add"].is_null(),
            "/conversation/add must migrate to /api/v2/conversations"
        );
        assert!(
            openapi["paths"]["/skill/listing"].is_null(),
            "/skill/listing must migrate to GET /api/v2/skills"
        );
        assert!(
            openapi["paths"]["/api/v2/maintenance/purge"].is_null(),
            "verb-style /maintenance/purge must be renamed"
        );

        // New maintenance sub-resources (plural nouns, no verbs).
        assert!(openapi["paths"]["/api/v2/maintenance/expired-records"]["delete"].is_mapping());
        assert!(openapi["paths"]["/api/v2/maintenance/compactions"]["post"].is_mapping());
        assert!(openapi["paths"]["/api/v2/maintenance/flushes"]["post"].is_mapping());
        assert!(openapi["paths"]["/api/v2/maintenance/index-rebuilds"]["post"].is_mapping());

        // Conversation turn ingestion under /api/v2.
        assert!(openapi["paths"]["/api/v2/conversations"]["post"].is_mapping());

        // Skills collection serves GET (listing) + POST (create).
        assert!(openapi["paths"]["/api/v2/skills"]["get"].is_mapping());
        assert!(openapi["paths"]["/api/v2/skills"]["post"].is_mapping());

        // Thread messages live in a sub-resource; /threads/{id} keeps GET+DELETE.
        assert!(openapi["paths"]["/api/v2/threads/{id}/messages"]["post"].is_mapping());
        assert!(
            openapi["paths"]["/api/v2/threads/{id}"]["post"].is_null(),
            "POST /threads/{{id}} must migrate to /threads/{{id}}/messages"
        );
        assert!(openapi["paths"]["/api/v2/threads/{id}"]["get"].is_mapping());
        assert!(openapi["paths"]["/api/v2/threads/{id}"]["delete"].is_mapping());

        // `/api/v2` total: every documented path is versioned except the
        // documented exceptions (liveness/metrics probes + UI).
        for (path, _) in openapi["paths"].as_mapping().unwrap() {
            let path = path.as_str().unwrap();
            let allowed_unversioned = ["/health", "/metrics", "/dashboard", "/dashboard/{path}"];
            assert!(
                path.starts_with("/api/v2/") || allowed_unversioned.contains(&path),
                "path {path} must live under /api/v2/ (or be a documented probe/UI exception)"
            );
        }

        assert_next_cursor_single_type(&openapi, "openapi.yaml");
    }

    #[test]
    fn test_create_endpoints_document_201_matching_impl() {
        let openapi = load_openapi();
        // impl returns StatusCode::CREATED (201) in all six create handlers.
        assert_created_status(&openapi, "/api/v2/records", "post");
        assert_created_status(&openapi, "/api/v2/records/batch", "post");
        assert_created_status(&openapi, "/api/v2/threads", "post");
        assert_created_status(&openapi, "/api/v2/conversations", "post");
        assert_created_status(&openapi, "/api/v2/snapshots/{name}", "post");
        assert_created_status(&openapi, "/api/v2/skills", "post");
    }

    #[test]
    fn test_pagination_contract_is_cursor_only() {
        let openapi = load_openapi();

        // Page responses: cursor (string|null) + has_more + collection.
        assert_page_schema(&openapi, "ListPageResponse", "records");
        assert_page_schema(&openapi, "SearchPageResponse", "records");
        assert_page_schema(&openapi, "AuditPageResponse", "events");
        assert_page_schema(&openapi, "ThreadsPageResponse", "threads");
        assert_page_schema(&openapi, "SkillListingPageResponse", "items");

        // Cursor request params are strings (opaque tokens), never integers.
        for path in ["/api/v2/list", "/api/v2/audit", "/api/v2/threads"] {
            let params = openapi["paths"][path]["get"]["parameters"]
                .as_sequence()
                .unwrap();
            let cursor = params
                .iter()
                .find(|p| p["name"].as_str() == Some("cursor"))
                .unwrap_or_else(|| panic!("{path} must document a `cursor` query param"));
            assert_eq!(
                cursor["schema"]["type"].as_str(),
                Some("string"),
                "{path} cursor must be a string"
            );
        }
        assert_eq!(
            openapi["components"]["schemas"]["SearchPageRequest"]["properties"]["cursor"]["type"]
                .as_str(),
            Some("string"),
            "search request cursor must be a string"
        );

        // No `offset` anywhere in the contract (cursor is the single scheme).
        let raw = fs::read_to_string("docs/api/openapi.yaml").expect("read openapi.yaml");
        assert!(
            !raw.contains("offset"),
            "openapi.yaml must not document offset pagination (cursor only)"
        );
    }

    #[test]
    fn test_record_input_required_fields_match_core_sdk() {
        let openapi = load_openapi();

        // Core `MemoryInput` (src/sdk/types/record.rs) accepts a metadata-only
        // body: namespace/key/payload/metadata required; vector/sparse_vector/
        // ttl_ms optional (Option fields).
        let input: vantadb::MemoryInput =
            serde_json::from_str(r#"{"namespace":"ns","key":"k1","payload":"p","metadata":{}}"#)
                .expect("core MemoryInput must deserialize without vector/sparse_vector/ttl_ms");
        assert!(input.vector.is_none());
        assert!(input.sparse_vector.is_none());
        assert!(input.ttl_ms.is_none());

        let required: Vec<&str> = openapi["components"]["schemas"]["RecordInput"]["required"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let mut required_sorted = required.clone();
        required_sorted.sort_unstable();
        assert_eq!(
            required_sorted,
            vec!["key", "metadata", "namespace", "payload"],
            "RecordInput.required must match the core MemoryInput required fields"
        );
    }

    #[test]
    fn test_yaml_drifts_fixed() {
        let openapi = load_openapi();

        // Drift 2: no Lisp example — the documented example must be valid IQL.
        let example = openapi["components"]["schemas"]["QueryRequest"]["properties"]["query"]
            ["example"]
            .as_str()
            .expect("QueryRequest.query must carry an example");
        assert!(
            !example.contains("(memory:get"),
            "stale Lisp example must be replaced with an IQL statement"
        );
        assert!(
            parse_statement(example).is_ok(),
            "documented QueryRequest example must parse as IQL: {example}"
        );

        // Drift 3: distance metric enum matches the core `DistanceMetric`
        // variants (src/node/vector_data.rs): Cosine | Euclidean | SparseDot.
        let enum_values: Vec<&str> = openapi["components"]["schemas"]["SearchPageRequest"]
            ["properties"]["distance_metric"]["enum"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            enum_values,
            vec!["Cosine", "Euclidean", "SparseDot"],
            "distance_metric enum must mirror the core DistanceMetric variants"
        );

        // Drift 5: every tag used by an operation is declared.
        let declared: Vec<&str> = openapi["tags"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(
            declared.contains(&"Skills"),
            "tag `Skills` must be declared in the top-level tags list"
        );
        // ... and no operation may use a tag that is not declared.
        for (path, ops) in openapi["paths"].as_mapping().unwrap() {
            let path = path.as_str().unwrap();
            for (_, op) in ops.as_mapping().unwrap() {
                if let Some(tags) = op["tags"].as_sequence() {
                    for tag in tags {
                        let tag = tag.as_str().unwrap();
                        assert!(
                            declared.contains(&tag),
                            "{path}: tag `{tag}` is not declared"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_traversal_parsing_via_parse_statement() {
        // OpenAPI describes: `from <entity> traverse <min>-<max> via <edge_label>`
        // Parser uses `SIGUE <min>..<max> "<edge_label>"` (Spanish keyword)
        // Test via parse_statement which is public

        let query = r#"FROM Test SIGUE 1..3 "amigo""#;
        let (_, stmt) =
            parse_statement(query).expect("Parser should accept SIGUE traversal syntax");

        match stmt {
            vantadb::query::Statement::Query(q) => {
                let trav = q.traversal.expect("Should have traversal");
                assert_eq!(trav.min_depth, 1);
                assert_eq!(trav.max_depth, 3);
                assert_eq!(trav.edge_label, "amigo");
            }
            _ => panic!("Expected Query statement with traversal"),
        }

        // Verify OpenAPI examples use the correct syntax
        let openapi = load_openapi();
        let query_desc = openapi["paths"]["/api/v2/query"]["post"]["description"]
            .as_str()
            .unwrap();

        // The OpenAPI should document the actual IQL syntax (SIGUE, not traverse)
        // This is a documentation check - the OpenAPI currently says "traverse" but parser uses "SIGUE"
        // We'll note this as a drift if present
        if query_desc.contains("traverse") && !query_desc.contains("SIGUE") {
            panic!("OpenAPI documents 'traverse' but parser uses 'SIGUE' keyword - drift detected");
        }
    }
}
