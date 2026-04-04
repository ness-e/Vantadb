# Fase 26: Olvido Bayesiano y Compresión Cognitiva

El crecimiento infinito de datos es insostenible en hardware local. ConnectomeDB resuelve esto mediante el **Mantenimiento Circadiano**, transformando el borrado de datos en un proceso de destilación de conocimiento.

## 1. La Poda de Entropía (Entropy Pruning) ✅
Inspirado en el decaimiento de las conexiones sinápticas.
- **Mecanismo**: El `SleepWorker` recorre el Cortex RAM y el Lóbulo Primario durante periodos de baja actividad.
- **Acción**: Por cada ciclo, el valor `hits` (frecuencia de acceso) de las neuronas que no han sido consultadas recientemente se divide: `hits = hits * 0.5`.
- **Efecto**: Los datos "irrelevantes" pierden energía gradualmente hasta alcanzar un umbral crítico de evacuación.
- **Implementación**: `src/governance/sleep_worker.rs` → Stage 1 dentro de `execute_rem_phase()`.

## 2. Compresión Cognitiva (Neural Summarization) ✅
En lugar de simplemente borrar, ConnectomeDB intenta "entender" qué se está perdiendo.

### Flujo Completo (Stage 3 del SleepWorker):

```
┌────────────────────────────────┐
│ Stage 1: Olvido Bayesiano      │  hits *= 0.5 por ciclo REM
│ Stage 2: Survival Evaluation   │  Purge (trust<0.2) / Consolidate (hits<10)
│ Stage 3: Neural Summarization  │  Compresión LLM de grupos "Oníricos"
└────────────────────────────────┘
```

1. **Clustering**: Los nodos con `hits < 5` y `!PINNED` se agrupan por el campo de edge `belongs_to_thread`.
2. **Validación de Peso Mínimo**: Solo se resumen grupos con `≥ 2 nodos` y `sum(hits) >= 3`. Los nodos basura se purgan directamente sin gastar CPU en LLM.
3. **Prompt Estructurado**: El motor invoca al LLM local (Ollama vía `CONNECTOME_LLM_SUMMARIZE_MODEL`) con un prompt de sistema que incluye:
   - El contenido de cada nodo (`content`)
   - Su `semantic_valence` (para que el resumen preserve los "puntos calientes")
   - Su `trust_score` y keywords
   - El conteo de accesos (`hits`)
4. Se genera una nueva **Neurona de Resumen** con las siguientes propiedades:
   - `neuron_type: LTNeuron`
   - `flags: PINNED` (inmutable)
   - `semantic_valence: 0.9` (protegida por Amygdala Budget)
   - `trust_score`: promedio de los nodos originales
   - **Linaje Semántico**: campo `ancestors` con los IDs originales (para Arqueología Semántica futura)
   - El resumen se embeddea vectorialmente vía `generate_embedding()` para búsqueda semántica
5. **Transacción de Seguridad**: La Neurona de Resumen se persiste PRIMERO en `deep_memory` CF. Solo si esta operación tiene éxito, los nodos originales se mueven al `shadow_kernel` como `AuditableTombstone`. Si falla el resumen, los originales se mantienen intactos.
6. **Presupuesto de Tiempo**: El Stage 3 tiene un límite de ejecución de 8 segundos (`MAX_SUMMARIZATION_DURATION_MS`). Si se excede, los grupos pendientes se difieren al siguiente ciclo de 10s, impidiendo que la compresión bloquee el sistema.

### Implementación:
- `src/governance/sleep_worker.rs` → `execute_neural_summarization()`
- `src/llm.rs` → `LlmClient::summarize_context()`
- `src/storage.rs` → `StorageEngine::insert_to_cf()`, `StorageEngine::consolidate_node()`

---

## 3. Estados de Salud Neuronal

| Estado | Hits / Trust | Acción del SleepWorker |
|:---:|:---:|:---|
| **Lúcido** | Alto | Mantener en Lóbulo Primario (RAM/Hot). |
| **Dudoso** | Medio/Bajo | Migrar de STN a LTN (Disco). |
| **Onírico** | Muy Bajo (<5 hits) | Candidato a Compresión Cognitiva vía LLM. |
| **Difunto** | < 0.1 trust | Mover al Shadow Archive (Tombstone). |

---

## 4. Corrección HNSW (Pre-Fase 26)
Se detectó un gap crítico: cuando el `SleepWorker` consolidaba nodos de STN→LTN, lo hacía con `db.put()` directo, sin actualizar el índice HNSW en memoria. Esto causaba divergencia entre el índice vectorial y el disco.

**Solución implementada**: Nuevo método `StorageEngine::consolidate_node()` que realiza la escritura a disco Y actualiza el HNSW index atómicamente. El `SleepWorker` ahora usa esta función en lugar de `db.put()`.

---

## Axioma de Inmortalidad (Axiom Lock) ✅
Si un nodo posee el flag `PINNED`, el `SleepWorker` tiene prohibido reducir sus `hits` o mutar su ubicación. Este mecanismo se reserva para "Verdades Fundamentales" definidas por el desarrollador o axiomas del sistema.

## Presupuesto de Amígdala ✅
El 5% más alto de `cortex_ram` (medido por `semantic_valence >= 0.8`) está blindado contra el Olvido Bayesiano. Estos nodos no se degradan ni se consolidan durante la Fase REM, preservando las memorias emocionalmente significativas.

---

## Variables de Entorno

| Variable | Descripción | Default |
|---|---|---|
| `CONNECTOME_LLM_URL` | URL de Ollama para embeddings y resúmenes | `http://localhost:11434` |
| `CONNECTOME_LLM_MODEL` | Modelo para embeddings | `all-minilm` |
| `CONNECTOME_LLM_SUMMARIZE_MODEL` | Modelo para compresión cognitiva | `llama3` |
