use connectomedb::storage::StorageEngine;
use connectomedb::governance::sleep_worker::SleepWorker;
use connectomedb::node::{UnifiedNode, NeuronType};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_circadian_rem_cycle() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());

    // Iniciar el worker en background (con timeout de 2 segundos en lugar de 5 para acelerar el test)
    // En el worker base hardcodeamos 5000ms. Para testing validaremos invirtiendo manualmente control,
    // o simplemente ejecutaremos una "Fase REM Forzada" invocando la logica privada si estuviera expuesta,
    // pero el worker la encapsula.
    // Como el `SleepWorker` corre un bucle sin fin, podríamos spawnear el thread real, 
    // pero esperaría `inactivity_threshold_ms` realista (5s) + sleep interval (10s), sumando ~15s al bench.
    
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
    // Inyectar 10000 Nodos Transitorios (STNeuron)
    for i in 1..=10000 {
        let mut node = UnifiedNode::new(i);
        node.neuron_type = NeuronType::STNeuron;
        node.hits = 5; // Bajo número de hits para inducir consolidación
        node.last_accessed = now - 65_000; // Simular envejecimiento > 60s
        storage.insert(&node).unwrap();
    }

    {
        let cortex = storage.cortex_ram.read().unwrap();
        assert_eq!(cortex.len(), 10000, "Cortex RAM no retuvo los STNeurons");
    }

    // Avanzamos el reloj de Storage para simular Inactividad Máxima
    // (SleepWorker real espera elapsed > 5000ms). Manipulamos la inactividad retrasando o seteando un valor.
    // Aquí spawn_worker no termina y traba tests asíncronos si usamos test.
    // Vamos a realizar una instanciación manual del bloque lógico ya que `SleepWorker::start` tiene un loop "while true".
    // 
    // Nota Técnica: Para testing en CI sin bloquear hilos infinitos, lo correcto es invocar 
    // el núcleo heurístico en forma sincrónica. Sin acceso a `execute_rem_phase`, usaremos 
    // tokio::spawn y un tokio::time::sleep lo suficientemente largo, 
    // O reexpondremos el bloque REM en un refactor de testing futuro. 

    let worker_storage = storage.clone();
    tokio::spawn(async move {
        SleepWorker::start(worker_storage).await;
    });

    println!("💤 Dejando al sistema dormir por 16 segundos para permitir a Tokio lanzar el SleepWorker REM...");
    sleep(Duration::from_secs(16)).await;

    // Verificar Consolidación
    {
        let cortex = storage.cortex_ram.read().unwrap();
        assert!(cortex.len() < 10000, "El SleepWorker no consolidó la memoria a disco");
        assert_eq!(cortex.len(), 0, "El Cortex RAM debería estar vacío tras la Fase REM prolongada");
    }

    // Verificar Migración LTN (Los Nodos deben seguir listos para IQL pero leídos desde disco)
    let reactivated_node = storage.get(500).unwrap().unwrap();
    assert_eq!(reactivated_node.id, 500);
}
