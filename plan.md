# Blueprint Técnico: Fase 31B - Uncertainty Zones & Quantum Search

Provide a brief description of the problem, any background context, and what the change accomplishes.
El motor HNSW de ConnectomeDB es inherentemente consistente una vez que se indexa un vector. Sin embargo, en el razonamiento de agentes autónomos, la inferencia frecuentemente es dubitativa o conjetural. Si indexamos vectores conjeturales directamente en el HNSW, corremos el riesgo de contaminar la arquitectura del índice con rutas sub-óptimas ("Aislamiento en Búfer de Penumbra").

Esta fase introduce el concepto de **Uncertainty Buffer** y el nodo en superposición **QuantumNeuron**, permitiendo que vectores dudosos se alojen en una "Penumbra" de RAM. Son accesibles para lecturas especulativas, pero no forman parte del grafo navegable HNSW hasta que un árbitro los "colapse" basados en el incremento de su `TrustScore`.

## User Review Required

> [!IMPORTANT]  
> Este Blueprint ha sido redactado como se solicitó y también copiado físicamente en `docDev/31B_Uncertainty_Zones.md` en tu repositorio.  
> Por favor revisa la arquitectura de `QuantumNeuron`, el `UncertaintyBuffer` (con soporte asíncrono Tokio `RwLock`), el `Uncertain Search Path`, y el proceso atómico de colapso delegado al `SleepWorker`. Si la especificación técnica es correcta, aprueba el plan para proceder con la ejecución.

## Proposed Changes

### `src/node.rs` (o un módulo anexo `src/quantum.rs`)
#### [MODIFY] `src/node.rs`
- Definir estado en enumeración: `QuantumState { Superposition, Collapsed, Decayed }`.
- Insertar estructura `QuantumNeuron` envolviendo `UnifiedNode` con `collapse_deadline_ms` y `injected_at`.
- Insertar el gestor concurrente `UncertaintyBuffer` con un `RwLock<HashMap<u64, QuantumNeuron>>`.
- Crear el método asíncrono `collapse(&self, node_id, storage, invalidation_tx)`.

### `src/executor.rs` (Dual-Path Execution)
#### [MODIFY] `src/executor.rs`
- Adoptar modo de búsqueda (`SearchPathMode`):
  - **Standard**: Sólo interactúa con `HnswIndex` (comportamiento actual).
  - **Uncertain**: Después de invocar a `HnswIndex`, inspecciona el `UncertaintyBuffer` mediante similitud vectorial (fuerza bruta / `L1`/`L2` contra ram) y combina ("merge") resultados, penalizando puntaje de las neuronas cuánticas por baja incerteza.

### `src/governance/sleep_worker.rs` (Quantum Observer)
#### [MODIFY] `src/governance/sleep_worker.rs`
- Alterar la `execute_rem_phase` o añadir un ciclo paralelo de sondeo enfocado al `UncertaintyBuffer`.
- Determinar expiración (`collapse_deadline_ms`):
  - Iniciar un check sobre el `TrustScore`.
  - Confiar en `TrustScore > 0.6` => Invocar `UncertaintyBuffer::collapse()`.
  - Si es menor => Ejecutar Decaimiento Quántico (borrado físico de la penumbra).

## Open Questions

> [!WARNING]
> ¿Deseamos que el `UncertaintyBuffer` asiente un Tombstone en el archivo persistente LTS al hacer el "Decay", o al ser puramente especulativo debe desaparecer sin dejar ningún rastro Arqueológico para ahorrar I/O?

## Verification Plan
### Automated Tests
- Test 1: Comprobar inserción temporal de QuantumNeuron en el macro-buffer.
- Test 2: Comportamiento "Standard" (no lo halla en búsqueda) vs "Uncertain" (lo halla en búsqueda).
- Test 3: Forzar avance de tiempo y confirmar que el "SleeperWorker" promueve/colapsa el nodo positivamente al LTS y el índice HNSW.
