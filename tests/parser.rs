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
