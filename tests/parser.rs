use iadbms::query::*;
use iadbms::parser::*;
use iadbms::node::FieldValue;

#[test]
fn test_parse_full_query() {
    let q = r#"
        FROM Usuario#usr45
        SIGUE 1..3 "amigo" Persona
        WHERE Persona.pais="VZLA" AND Persona.bio ~ "rust", min=0.88
        FETCH Persona.nombre, Persona.email
        RANK BY Persona.relevancia DESC
        WITH TEMPERATURE 0.5
    "#;

    let (_, parsed) = parse_query(q).unwrap();

    assert_eq!(parsed.from_entity, "Usuario#usr45");
    assert_eq!(parsed.traversal.as_ref().unwrap().edge_label, "amigo");
    assert_eq!(parsed.traversal.as_ref().unwrap().max_depth, 3);
    assert_eq!(parsed.where_clause.as_ref().unwrap().len(), 2);
    
    match &parsed.where_clause.as_ref().unwrap()[0] {
        Condition::Relational(f, op, v) => {
            assert_eq!(f, "Persona.pais");
            assert_eq!(op, &RelOp::Eq);
            assert_eq!(v, &FieldValue::String("VZLA".to_string()));
        },
        _ => panic!("Expected relational")
    }

    match &parsed.where_clause.as_ref().unwrap()[1] {
        Condition::VectorSim(f, t, m) => {
            assert_eq!(f, "Persona.bio");
            assert_eq!(t, "rust");
            assert_eq!(*m, 0.88);
        },
        _ => panic!("Expected vectorsim")
    }

    assert_eq!(parsed.fetch.as_ref().unwrap().len(), 2);
    assert_eq!(parsed.rank_by.as_ref().unwrap().field, "Persona.relevancia");
    assert!(parsed.rank_by.as_ref().unwrap().desc);
    assert_eq!(parsed.temperature.unwrap(), 0.5);
}

#[test]
fn test_parse_insert() {
    let q = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", edad: 28 } VECTOR [0.1, -0.4]"#;
    let (_, stmt) = parse_statement(q).unwrap();
    match stmt {
        Statement::Insert(ins) => {
            assert_eq!(ins.node_id, 101);
            assert_eq!(ins.node_type, "Usuario");
            assert_eq!(ins.fields.get("nombre").unwrap(), &FieldValue::String("Eros".to_string()));
            assert_eq!(ins.fields.get("edad").unwrap(), &FieldValue::Int(28));
            assert_eq!(ins.vector.unwrap()[0], 0.1);
        },
        _ => panic!("Expected insert"),
    }
}

#[test]
fn test_parse_update() {
    let q = r#"UPDATE NODE#101 SET nombre = "Eros Dev", activo = true"#;
    let (_, stmt) = parse_statement(q).unwrap();
    match stmt {
        Statement::Update(upd) => {
            assert_eq!(upd.node_id, 101);
            assert_eq!(upd.fields.get("activo").unwrap(), &FieldValue::Bool(true));
        },
        _ => panic!("Expected update"),
    }
}

#[test]
fn test_parse_relate() {
    let q = r#"RELATE NODE#1 --"amigo"--> NODE#2 WEIGHT 0.95"#;
    let (_, stmt) = parse_statement(q).unwrap();
    match stmt {
        Statement::Relate(rel) => {
            assert_eq!(rel.source_id, 1);
            assert_eq!(rel.target_id, 2);
            assert_eq!(rel.label, "amigo");
            assert_eq!(rel.weight.unwrap(), 0.95);
        },
        _ => panic!("Expected relate"),
    }
}

#[test]
fn test_parse_delete() {
    let q = r#"DELETE NODE#5"#;
    let (_, stmt) = parse_statement(q).unwrap();
    match stmt {
        Statement::Delete(del) => {
            assert_eq!(del.node_id, 5);
        },
        _ => panic!("Expected delete"),
    }
}
