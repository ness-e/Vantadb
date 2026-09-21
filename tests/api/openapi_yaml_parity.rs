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
            search_desc.contains("ensure_indexes_current") || search_desc.contains("rebuild-index"),
            "Search endpoint should document that ensure_indexes_current() runs at startup or reference /api/v2/maintenance/rebuild-index"
        );

        // Also verify the rebuild-index endpoint exists
        let rebuild_desc = openapi["paths"]["/api/v2/maintenance/rebuild-index"]["post"]
            ["description"]
            .as_str()
            .unwrap();
        assert!(
            rebuild_desc.contains("Rebuilds secondary indexes"),
            "rebuild-index endpoint should exist and be documented"
        );
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
