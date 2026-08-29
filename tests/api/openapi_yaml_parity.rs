//! OpenAPI YAML ↔ Real Implementation Parity Test
//!
//! Validates that docs/api/OPENAPI.yaml accurately reflects the actual
//! implementation in src/parser/mod.rs, src/cli_server.rs, and
//! vantadb-mcp/src/handlers/tools.rs.

#[cfg(test)]
mod openapi_yaml_parity {
    use serde_yaml::Value;
    use std::fs;
    use vantadb::parser::parse_statement;
    use vantadb::query::Condition;

    /// Load and parse the OpenAPI YAML file
    fn load_openapi() -> Value {
        let yaml_content =
            fs::read_to_string("docs/api/OPENAPI.yaml").expect("Failed to read OPENAPI.yaml");
        serde_yaml::from_str(&yaml_content).expect("Failed to parse OPENAPI.yaml")
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
    fn test_graph_traversal_body_shape_matches_mcp_handler() {
        let openapi = load_openapi();

        // Get the GraphTraversalBody schema from components/requestBodies
        let traversal_body = &openapi["components"]["requestBodies"]["GraphTraversalBody"];
        let schema = &traversal_body["content"]["application/json"]["schema"];

        // Required fields per MCP handler (vantadb-mcp/src/handlers/tools.rs:2356-2441):
        // - start: array of u128 node IDs
        // - mode: string enum ["bfs", "dfs"]
        // - max_depth: integer (required)
        // - direction: string enum ["outgoing", "incoming", "both"]
        // - filter: optional object { labels: u32[], time_range: [u64, u64] }

        let required = schema["required"].as_sequence().unwrap();
        let required_fields: Vec<&str> = required.iter().map(|v| v.as_str().unwrap()).collect();

        // OpenAPI currently only requires "roots" - this is the drift
        // The real handler expects different fields
        assert!(
            required_fields.contains(&"start"),
            "GraphTraversalBody should require 'start' (u128[]), not 'roots'"
        );
        assert!(
            required_fields.contains(&"mode"),
            "GraphTraversalBody should require 'mode' (bfs|dfs)"
        );
        assert!(
            required_fields.contains(&"max_depth"),
            "GraphTraversalBody should require 'max_depth' (integer)"
        );
        assert!(
            required_fields.contains(&"direction"),
            "GraphTraversalBody should require 'direction' (outgoing|incoming|both)"
        );

        // Verify properties exist with correct types (check they're mappings/objects)
        let props = &schema["properties"];
        assert!(props["start"].is_mapping(), "start property should exist");
        assert!(props["mode"].is_mapping(), "mode property should exist");
        assert!(
            props["max_depth"].is_mapping(),
            "max_depth property should exist"
        );
        assert!(
            props["direction"].is_mapping(),
            "direction property should exist"
        );
        assert!(
            props["filter"].is_mapping(),
            "filter property should exist (optional)"
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
