# Blueprint Técnico: Fase 31B - Uncertainty Zones & Quantum Search

## Visión General
El motor HNSW de ConnectomeDB es inherentemente consitente una vez que se indexa un vector. Sin embargo, en el razonamiento de agentes autónomos, la inferencia frecuentemente es dubitativa o conjetural. Si indexamos vectores conjeturales directamente en el HNSW, corremos el riesgo de contaminar la arquitectura del índice con rutas sub-óptimas ("Aislamiento en Búfer de Penumbra").

Esta fase introduce el concepto de **Uncertainty Buffer** y el nodo en superposición **QuantumNeuron**, permitiendo que vectores dudosos se alojen en una "Penumbra" de RAM. Son accesibles para lecturas especulativas, pero no forman parte del grafo navegable HNSW hasta que un árbitro los "colapse" basados en el incremento de su `TrustScore`.

---

## 1. Patrón Arquitectónico: Shadow Buffer (Penumbra)

Los nodos inciertos se abstraen temporalmente del `StorageEngine` físico y del `HnswIndex` global.

### Estructura de Datos en Rust

```rust
use std::time::Instant;
use tokio::sync::RwLock;

/// Representa el estado de un nodo pre-colapsado. 
pub enum QuantumState {
    Superposition,
    Collapsed,
    Decayed,      // Descartado antes del colapso (Trust muy bajo)
}

/// Nodo que habita la Penumbra
pub struct QuantumNeuron {
    pub node_id: u64,
    pub payload: crate::node::UnifiedNode,
    pub state: QuantumState,
    pub injected_at: Instant,
    pub collapse_deadline_ms: u128,
}

/// El Búfer de Penumbra (Aislamiento)
pub struct UncertaintyBuffer {
    pub quantum_nodes: RwLock<std::collections::HashMap<u64, QuantumNeuron>>,
}

impl UncertaintyBuffer {
    pub fn new() -> Self {
        Self {
            quantum_nodes: RwLock::new(std::collections::HashMap::new()),
        }
    }
}
```

---

## 2. Dual-Path Execution (Modos de Búsqueda)

El `Executor` de consultas debe bifurcar su comportamiento en base a la voluntad de la Query.

En `src/executor.rs`, la búsqueda de vectores operará bajo un `SearchPathMode`:

1. **Path Estándar (Consistente):** Iteración normal al `HnswIndex`. Ignora el Búfer de Penumbra. Retorna la realidad materializada.
2. **Path Uncertain (Conjetural/Especulativo):**
   - Ejecuta la búsqueda estándar en el `HnswIndex`.
   - Ejecuta un escaneo lineal / mini-índice exhaustivo sobre los vectores del `UncertaintyBuffer`.
   - Mezcla los resultados (`MergeSort`) según el `cosine_similarity`, aplicando una penalidad matemática ligera a la similitud de los nodos cuánticos debido a su incertidumbre inherente (`TrustScore < 0.5`).

---

## 3. Mecanismo de Colapso (Integración HNSW)

El único vector de entrada permisible desde la Penumbra hacia la materia oscura lógica (HNSW / LTS) es a través de una función de colapso atómica.

```rust
impl UncertaintyBuffer {
    /// Desata el Colapso Quántico: Integra materialmente la idea al motor.
    pub async fn collapse(
        &self, 
        node_id: u64, 
        storage: &crate::storage::StorageEngine,
        invalidation_tx: &tokio::sync::mpsc::Sender<crate::governance::invalidations::InvalidationEvent>
    ) -> Result<(), String> {
        let mut buffer = self.quantum_nodes.write().await;
        if let Some(mut quantum) = buffer.remove(&node_id) {
            quantum.state = QuantumState::Collapsed;
            
            // 1. Inserción atómica manual al LTS y HNSW
            storage.insert(&quantum.payload).map_err(|e| e.to_string())?;
            
            // 2. Emisión MCP Webhook - Evento reactivo
            crate::governance::invalidations::InvalidationDispatcher::emit_zone_collapsed(
                invalidation_tx,
                node_id,
                "Excedió Trust Threshold. Integración material completa.".to_string()
            ).await;
            
            Ok(())
        } else {
            Err("QuantumNeuron not found or already collapsed".to_string())
        }
    }
}
```

---

## 4. Ciclo de Vida y Gouvernance (SleepWorker)

El `SleeperWorker` asume el rol del Colapsador Asíncrono / Destructor de Universos inútiles.

En el ciclo `execute_rem_phase`:
1. El `SleepWorker` bloquea gentilmente en lectura el `UncertaintyBuffer`.
2. Escanea todos los `QuantumNeuron` cuya edad actual (determinada desde `injected_at`) haya sobrepasado su `collapse_deadline_ms`.
3. Evaluador de TrustScore:
   - Si el `TrustScore > 0.6` (ha recibido retroalimentación en encuestas especulativas o coincidencias por LISP proxy): Forzar colapso afirmativo llamando a `UncertaintyBuffer::collapse()`.
   - Si el `TrustScore < 0.6`: Forzar Decay (Purgar del buffer sin integrarlo al HNSW, liberando RAM).
4. Limpiar los buffers zombis evitando memory leaks.
