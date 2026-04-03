use connectomedb::storage::StorageEngine;
use connectomedb::executor::{Executor, ExecutionResult};
use connectomedb::error::ConnectomeError;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_lisp_rule_insertion() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let executor = Executor::new(&storage);

    // Inyectamos una S-Expression LISP (Homoiconicidad)
    let lisp_query = r#"(INSERT :neuron {:label "CognitiveRule" :trust 0.99 :desc "Regla generada"})"#;

    let result = executor.execute_hybrid(lisp_query).await;
    assert!(result.is_ok(), "Fallo al ejecutar instrucción LISP");

    // Verificar si el motor lo guardó y aplicó sys_rule
    if let Ok(ExecutionResult::Write { affected_nodes, message: _ }) = result {
        assert_eq!(affected_nodes, 1);
        // Extraer id insertado parsing el mensaje (MVP) o buscando el registro en Storage
        // Como no devolvemos el ID exacto en MVP fácil (rand id), iteraremos `cortex_ram` que está en memoria 
        // porque INSERT Lisp entra directo 
    }

    let mut found = false;
    {
        let cortex = storage.cortex_ram.read().unwrap();
        for (_, node) in cortex.iter() {
            if let Some(crate::node::FieldValue::Bool(is_rule)) = node.get_field("sys_rule") {
                if *is_rule {
                    found = true;
                    assert_eq!(
                        node.get_field("label"), 
                        Some(&crate::node::FieldValue::String("CognitiveRule".to_string()))
                    );
                }
            }
        }
    }

    assert!(found, "No se encontró el nodo insertado vía LISP");

    // Test DoR protection (Fuel)
    // El interprete base consume Fuel. Para un loop o recursión forzada (A futuro)
    // aquí validaremos que se lance el ConnectedError de Sandbox
}
