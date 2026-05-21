//! Vanta Lisp & DQL Parser Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::node::FieldValue;
use vantadb::parser::*;
use vantadb::query::*;

#[test]
fn dql_parser_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (DQL PARSER)");

    harness.execute("DQL: Complex FROM -> FETCH Pipeline", || {
        let q = r#"
            FROM Usuario#usr45
            SIGUE 1..3 "amigo" Persona
            WHERE Persona.pais="VZLA" AND Persona.bio ~ "rust", min=0.88
            FETCH Persona.nombre, Persona.email
            RANK BY Persona.relevancia DESC
            WITH TEMPERATURE 0.5
        "#;

        TerminalReporter::sub_step(
            "Parsing complex DQL query with graph traversal and semantic filter...",
        );
        let (_, parsed) = parse_query(q).expect("DQL Parser failed");

        assert_eq!(parsed.from_entity, "Usuario#usr45");
        assert_eq!(parsed.traversal.as_ref().unwrap().edge_label, "amigo");
        assert_eq!(parsed.traversal.as_ref().unwrap().max_depth, 3);
        assert_eq!(parsed.where_clause.as_ref().unwrap().len(), 2);

        match &parsed.where_clause.as_ref().unwrap()[0] {
            Condition::Relational(f, op, v) => {
                assert_eq!(f, "Persona.pais");
                assert_eq!(op, &RelOp::Eq);
                assert_eq!(v, &FieldValue::String("VZLA".to_string()));
            }
            _ => panic!("Expected relational condition"),
        }
        TerminalReporter::success("DQL Abstract Syntax Tree (AST) generated correctly.");
    });

    harness.execute("DML: Multi-Statement Core Parse", || {
        TerminalReporter::sub_step("Testing INSERT with positional vector...");
        let q_ins =
            r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", edad: 28 } VECTOR [0.1, -0.4]"#;
        let (_, stmt_ins) = parse_statement(q_ins).expect("Insert parse failed");
        if let Statement::Insert(ins) = stmt_ins {
            assert_eq!(ins.node_id, 101);
            assert_eq!(ins.fields.get("edad").unwrap(), &FieldValue::Int(28));
        }

        TerminalReporter::sub_step("Testing UPDATE with multiple fields...");
        let q_upd = r#"UPDATE NODE#101 SET nombre = "Eros Dev", activo = true"#;
        let (_, stmt_upd) = parse_statement(q_upd).expect("Update parse failed");
        if let Statement::Update(upd) = stmt_upd {
            assert_eq!(upd.node_id, 101);
        }

        TerminalReporter::sub_step("Testing RELATE with edge weighting...");
        let q_rel = r#"RELATE NODE#1 --"amigo"--> NODE#2 WEIGHT 0.95"#;
        let (_, stmt_rel) = parse_statement(q_rel).expect("Relate parse failed");
        if let Statement::Relate(rel) = stmt_rel {
            assert_eq!(rel.source_id, 1);
            assert_eq!(rel.weight.unwrap(), 0.95);
        }

        TerminalReporter::sub_step("Testing DELETE physical excision...");
        let q_del = r#"DELETE NODE#5"#;
        let (_, stmt_del) = parse_statement(q_del).expect("Delete parse failed");
        if let Statement::Delete(del) = stmt_del {
            assert_eq!(del.node_id, 5);
        }

        TerminalReporter::success("DML statement family parsing complete.");
    });
}
