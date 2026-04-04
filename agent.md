# 🧠 ConnectomeDB — AGENT MAESTRO (v0.4.0 · Actualizado: 2026-04-03)

> **ConnectomeDB** es un Motor de Inferencia Cognitiva escrito en Rust.
> Combina vectores (HNSW), grafos dirigidos y campos relacionales en un único `UnifiedNode` persistido sobre RocksDB.
> El motor aprende, olvida y razona mediante gobernanza biológica.

---

## ⚙️ REGLAS ABSOLUTAS (NUNCA VIOLAR)

1. **LEE `docDev/` ANTES de escribir código.** Cada fase tiene una especificación técnica aprobada.
2. **UNA FASE POR COMMIT.** No mezclar fases distintas en un solo commit.
3. **NUNCA código sin su `.md` de especificación correspondiente.**
4. **Mover `docDev/XX_*.md` → `complete/XX_*/` SOLO cuando:**
   - ✅ Tests unitarios pasan en CI
   - ✅ Benchmarks dentro de tolerancia
   - ✅ README y CHANGELOG actualizados
5. **GIT PIPELINE RIGUROSO (EN CADA PASO):**
   - `git add .` → `git commit -m "feat(fase-XX): <título descriptivo>"` → `git push`
   - El cuerpo del commit debe explicar el **QUÉ** y el **POR QUÉ** arquitectónico.
6. **CI PATH FILTERING activo:** El workflow `rust_ci.yml` solo dispara ante cambios en `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`. Documentación pura no gasta minutos.

---

## 🗺️ GLOSARIO RÁPIDO (Ver `docDev/00_Glossary.md` para detalles completos)

| Término Biológico | Equivalente en Código | Descripción |
|---|---|---|
| **Neuron** | `UnifiedNode` | Unidad mínima: vector + grafo + relacional |
| **Synapse** | `Edge` | Conexión pesada y dirigida |
| **Cortex** | `Query Planner` | Motor de ejecución híbrida |
| **Lobe** | `Column Family (CF)` | Partición física en RocksDB |
| **Shadow Kernel** | `Audit Layer` | Subconsciente: tombstones y cuarentena |
| **Cognitive Fuel** | `Resource Quota` | Límite de cómputo por evaluación LISP |
| **Axon** | `WAL` | Write-Ahead Log de durabilidad |
| **Sleep Worker** | `GC / Maintenance Daemon` | Consolidador circadiano en segundo plano |
| **Neural Index** | `HNSW Index` | Navegación vectorial optimizada |
| **Amygdala Budget** | `semantic_valence guard` | Protege el 5% más importante de la RAM |

---

## 📦 ESTADO DEL SOFTWARE POR VERSIÓN

### ✅ v0.1.0 — Fundación
- `UnifiedNode`: vectores F32/I8, edges, relational `BTreeMap`.
- Parser IQL con `nom` (sintaxis `FROM`, `SIGUE`, `~`, `RANK BY`).
- RucksDB WAL atómico y serialización `bincode`.

### ✅ v0.2.0 — Motor de Almacenamiento
- RocksDB como motor primario de persistencia.
- Zero-copy buffer pinning (`get_pinned`).
- Bloom Filters y Block Cache 2GB.

### ✅ v0.3.0 — Aceleración SIMD y Cognición
- SIMD vectorial mediante crate `wide` (`f32x8`) en `cosine_similarity`.
- CP-Index con bitsets `u128` para pre-filtrado semántico.
- `HNSW` para navegación vectorial sub-milisegundo.

### ✅ v0.4.0 — Cognitive OS (ESTADO ACTUAL)
- `UnifiedNode` con campos: `hits`, `last_accessed`, `trust_score`, **`semantic_valence`** (nuevo).
- `NeuronType` enum (`STNeuron` / `LTNeuron`) + `CognitiveUnit` trait.
- `LispSandbox` con Cognitive Fuel (1000 steps) y eval de S-Expressions.
- `NeuLISP VM` (`src/eval/vm.rs`): Opcodes `OP_VEC_SIM`, `OP_TRUST_CHECK`. Retorna `(Value, TrustScore)`.
- `SleepWorker` (Fase REM): Olvido Bayesiano + migración STN→LTN + **Presupuesto de Amígdala (5%)**.
- `DevilsAdvocate` + `TrustArbiter`: auditoría de escrituras con resolución `Accept/Reject/Shadow`.
- `StorageEngine` con 4 Column Families: `default`, `shadow_kernel`, `deep_memory`, `tombstones`.
- `ResourceGovernor`: OOM guard 2GB + timeout 50ms por query.
- Documentos: `docDev/29_NeuLISP_Spec.md`, `docDev/30_Memory_Rehydration_Protocol.md`.

---

## 🚦 ROADMAP DE IMPLEMENTACIÓN POR FASES

> Las fases **completadas** están marcadas con ✅. Las **pendientes** con los detalles exactos de implementación.

---

### ✅ FASE 16 — `18_CognitiveArchitecture`
- `CognitiveUnit` trait + `NeuronType` enum en `src/node.rs`.
- Campos `hits`, `last_accessed`, `trust_score` inyectados.
- Flag `PINNED` en `NodeFlags`.
- Estrategia Lazy Write-Back sin romper `bincode`.

---

### ✅ FASE 17 — `19_ShadowKernel & Trust Governance`
- Column Families: `default`, `shadow_kernel`, `tombstones`.
- `AuditableTombstone` con `original_hash` en `src/governance/mod.rs`.
- `WriteBatch` atómico en `.delete()`: Clone → Shadow → Tombstone → Delete.

---

### ✅ FASE 18 — `20_SecurityAxioms`
- Iron Axioms de consistencia topológica en `src/engine.rs`.
- RocksDB Checkpointing ("Life Insurance") en `src/storage.rs`.
- `trigger_panic_state()` para violaciones críticas.

---

### ✅ FASE 19 — `23_Sovereignty_Governance`
- `DevilsAdvocate`: detección de contradicciones vectoriales (similitud > 0.95 + Trust divergente).
- `TrustArbiter`: resolución `Accept / Reject / Shadow`.
- Bloom Filters por lóbulo para axioma topológico sin hit de disco.

---

### ✅ FASE 20 — `20_SleepWorker_Spec`
- `SleepWorker` daemon en `src/governance/sleep_worker.rs`.
- Fase REM con cadencia 10s + inception condition (5s de inactividad).
- Olvido Bayesiano: `hits *= 0.5` por ciclo.
- Migración STN→LTN si `hits < 10 && !PINNED && last_accessed > 60s`.
- Poda al Shadow Archive si `trust_score < 0.2`.
- **Presupuesto de Amígdala**: máx 5% de cortex_ram con `semantic_valence >= 0.8` blindado.

---

### ✅ FASE 21 — `21_SIMD_Optimization`
- `cosine_similarity` reestructurado con `wide::f32x8` (bloques de 8 floats).
- Fallback automático a iteradores escalares en hardware sin AVX.
- Read-locks refinados en HNSW para reducir contención.

---

### ✅ FASE 22 — `22_Lisp_Cognition` (Evolutivo: NeuLISP)
- Parser secundario `nom` para S-Expressions en `src/parser/lisp.rs`.
- `LispSandbox` con Cognitive Fuel (1000 steps).
- **Operador de Similitud (`~`)**: Nativo en NeuLISP → despacha `OP_VEC_SIM`.
- **Valencia Gated-Macros**: macros condicionadas a `semantic_valence` del nodo.
- LISP `INSERT` crea nodos como `STNeuron` directamente en `cortex_ram`.

---

### ✅ FASE 23 — `24_Memory_Hierarchy`
- Dualidad STNeuron (RAM / `cortex_ram`) vs LTNeuron (RocksDB SST).
- Promoción dinámica: LTN → STN al alcanzar `hits >= 50`.
- Campo `last_query_timestamp: AtomicU64` para perfilado de inactividad.
- Estrategia mmap para Neural Index en hardware limitado.

---

### ✅ FASE 24 — `25_Lobe_Segmentation`
- Column Families: `default` (Primario), `shadow_kernel` (Sombra), `deep_memory` (Histórico), `tombstones`.
- Políticas diferenciadas: cache agresivo en `default`, Zstd en `shadow_kernel`, Read-Only en `deep_memory`.
- Aislamiento de I/O entre lóbulos para no bloquear queries activas.

---

### ✅ FASE 25 — `29_NeuLISP_Spec`
- Especificación de la gramática NeuLISP (operador `~`, macros de certeza).
- VM de bytecode (`src/eval/vm.rs`): `OP_VEC_SIM`, `OP_TRUST_CHECK`.
- Retorno probabilístico: `(Value: f32, TrustScore: f32)`.
- Penalización automática de `TrustScore` si similitud es incalculable.

---

### ✅ FASE 26 — `28_Inference_Optimization` (Parcial)
- NeuLISP VM dispatch basado en `match` de Opcodes.
- SIMD para `OP_VEC_SIM` usando `cosine_similarity` existente.
- (Pendiente) Filtros de Bloom integrados al flujo de `Cortex` (ver Fase 28 siguiente).

---

### 🔲 FASE 27 — `26_Bayesian_Forgetfulness` → **Neural Summarization**
**Objetivo:** Cerrar el ciclo del Olvido Bayesano con síntesis asistida por LLM en lugar de eliminación bruta.

**Tareas (en orden):**
- [ ] En `src/governance/sleep_worker.rs`: detectar grupos de neuronas relacionadas (`edges` comunes) con estado "Onírico" (`hits < 5`).
- [ ] Invocar Ollama (`CONNECTOME_LLM_URL`) con prompt `"Summarize Context"` para el grupo de nodos.
- [ ] Crear una nueva **Neurona de Resumen** (`NeuronType::LTNeuron` en `deep_memory`) con el resultado.
- [ ] Mover las neuronas originales al `shadow_kernel` como `AuditableTombstone`.
- [ ] Escribir tests en `tests/neural_summarization.rs`.
- [ ] Doc: ampliar `docDev/26_Bayesian_Forgetfulness.md` con diagrama de flujo.

**Archivos a modificar:** `src/governance/sleep_worker.rs`, `src/storage.rs`, `tests/neural_summarization.rs`.

---

### 🔲 FASE 28 — `27_Hardware_Adapters` → **Modo Camaleón**
**Objetivo:** Auto-detección de hardware y configuración adaptativa del motor.

**Tareas (en orden):**
- [ ] En `src/main.rs` o `src/lib.rs`: función `detect_hardware_profile() -> HardwareProfile`.
  - **CPU Check**: `std::is_x86_feature_detected!("avx512f")` → activa SIMD full.
  - **RAM Check**: `sysinfo` crate para leer RAM total → si < 16GB → `SurvivalProfile`.
  - **I/O Check**: Medir latencia de una escritura dummy en el directorio de datos.
- [ ] Enum `HardwareProfile { Survival, Standard, Enterprise }` en `src/config.rs`.
- [ ] Inyectar el perfil en el constructor de `StorageEngine::open()`.
  - `Survival`: BlockCache 512MB, cadencia SleepWorker 5s, I8 quantization forzada.
  - `Enterprise`: BlockCache proporcional a RAM, poda diferida, FP32 nativo.
- [ ] **Throttling Cognitivo**: si CPU temp > umbral (vía `sysinfo`), insertar delay configurable entre inferencias.
- [ ] Escribir tests en `tests/hardware_profiles.rs`.

**Archivos a modificar:** `src/main.rs`, `src/config.rs` (nuevo), `src/storage.rs`, `Cargo.toml` (añadir `sysinfo`).

---

### 🔲 FASE 29 — `28_Inference_Optimization` → **Bloom Co-Location + MCP**
**Objetivo:** Completar la optimización de inferencias con Bloom Filters y exponer endpoint MCP.

**Tareas (en orden):**
- [ ] Integrar `bloom` crate (o `probabilistic-collections`) en `StorageEngine`.
- [ ] Por cada CF (Lóbulo), mantener un `BloomFilter` en RAM que se reconstruye en cold start.
- [ ] En `StorageEngine::get()`: consultar Bloom Filter antes del `Point-Lookup` en RocksDB.
- [ ] En `src/server/` (handler HTTP): exponer endpoint `POST /mcp/context` que recibe un payload de contexto de agente externo y lo ingresa como `STNeuron` en `cortex_ram`.
- [ ] Endpoint `GET /mcp/axioms` que devuelve los Axiomas de Hierro activos como JSON.
- [ ] Escribir tests en `tests/bloom_filter.rs` y `tests/mcp_integration.rs`.

**Archivos a modificar:** `src/storage.rs`, `src/server/` (handlers), `Cargo.toml`.

---

### 🔲 FASE 30 — `30_Memory_Rehydration_Protocol` → **Arqueología Semántica**
**Objetivo:** Permitir la recuperación controlada de memorias históricas del Shadow Kernel.

**Tareas (en orden):**
- [ ] Añadir opcode `OP_REHYDRATE` a la NeuLISP VM (`src/eval/vm.rs`).
- [ ] En `StorageEngine`: función `rehydrate(summary_id: u64) -> Result<Vec<UnifiedNode>>` que:
  - Busca nodos con relación `belonged_to` hacia `summary_id` en `shadow_kernel` CF.
  - Restaura los nodos como `STNeuron` de solo lectura (`PINNED + TOMBSTONE` flags preservados pero sin nuevo write).
- [ ] Umbral de activación: si `TrustScore < 0.4` en la Neurona de Resumen → auto-proponer rehydration.
- [ ] LLM re-evalúa y, si el usuario aprueba, los nodos rehidratados se consolidan elevando el `TrustScore` de la Neurona de Resumen.
- [ ] Escribir tests en `tests/memory_rehydration.rs`.

**Archivos a modificar:** `src/eval/vm.rs`, `src/storage.rs`, `src/governance/sleep_worker.rs`.

---

### 🔲 FASE 31 — **Uncertainty Zones (Superposición Lógica)**
**Objetivo:** Soporte para nodos en "superposición" (estados lógicamente contradictorios pendientes de resolución).

**Tareas (en orden):**
- [ ] Añadir variante al enum `NeuronType`: `QuantumNeuron { candidates: Vec<UnifiedNode>, collapse_deadline_ms: u64 }`.
- [ ] Lógica de colapso en el `TrustArbiter`: después de `collapse_deadline_ms`, el candidato con mayor `TrustScore` "gana" y los demás van al `shadow_kernel`.
- [ ] Integrar con `DevilsAdvocate`: contradictions no bloquean inmediatamente sino que crean un `QuantumNeuron`.
- [ ] Escribir tests en `tests/uncertainty_zones.rs`.

**Archivos a modificar:** `src/node.rs`, `src/governance/mod.rs`, `src/storage.rs`.

---

### 🔲 FASE 32 — **LTD (Long-Term Depression) para Edges**
**Objetivo:** El peso de las relaciones (Synapses) decae con el tiempo si no son traversadas, emulando la depresión sináptica biológica.

**Tareas (en orden):**
- [ ] Añadir campo `last_traversed_ms: u64` y `traversal_count: u32` al struct `Edge` en `src/node.rs`.
- [ ] En el `SleepWorker` Fase REM: por cada nodo en cortex_ram, aplicar `edge.weight *= 0.95` si `now - edge.last_traversed_ms > 86_400_000` (24h sin traversal).
- [ ] Si `edge.weight < 0.05`: remover el edge del nodo y registrar la remoción como `AuditableTombstone` para el edge.
- [ ] Actualizar `src/graph.rs` para registrar `last_traversed_ms` al recorrer edges en BFS/DFS.
- [ ] Escribir tests en `tests/synaptic_depression.rs`.

**Archivos a modificar:** `src/node.rs`, `src/graph.rs`, `src/governance/sleep_worker.rs`.

---

### 🔲 FASE 33 — **Contextual Priming (Caché Anticipatorio)**
**Objetivo:** Pre-cargar en `cortex_ram` los vecinos de los nodos más consultados antes de que sean pedidos.

**Tareas (en orden):**
- [ ] En `StorageEngine::get()`: si se consulta un nodo con `hits > 20`, disparar un `tokio::spawn` que cargue sus vecinos directos (edges nivel 1) desde RocksDB a `cortex_ram`.
- [ ] Límite de priming: máximo 50 nodos por operación de priming, respetando el presupuesto de Amígdala.
- [ ] Configuración: `CONNECTOME_PRIMING_ENABLED=true/false` como variable de entorno.
- [ ] Escribir tests en `tests/contextual_priming.rs` validando warm cache tras primer acceso.

**Archivos a modificar:** `src/storage.rs`, `src/config.rs`.

---

## 🎯 OBJETIVOS CRÍTICOS (Inmutables)

```
✅ MVP: 1M nodos + 100k vectores en 16GB RAM
✅ Latencia: <20ms hybrid queries
✅ Overhead: <128MB cold start
✅ Docker: <5min setup mundial
✅ Integración: Ollama native (CONNECTOME_LLM_URL)
✅ Inferencia Cognitiva: NeuLISP retorna (Value, TrustScore)
🔲 Neural Summarization: Comprimir grupos degenerados via LLM
🔲 Hardware Profiles: Auto-detección Survival/Standard/Enterprise
🔲 MCP Endpoint: /mcp/context + /mcp/axioms
```

---

## 🚫 LIMITACIONES TÉCNICAS (Inmutables)

```
❌ NO cloud-first (target: 16GB laptop edge)
❌ NO ML-heavy (heurístico → estadístico → LLM solo para compresión)
❌ NO Storage-over-Compute sin justificación mecánica
❌ NO sharding en v0.4.x (diferido a v0.5+ Enterprise)
```

---

## 🛠 CONOCIMIENTOS REQUERIDOS

```
CORE:     Rust async/zero-copy, RocksDB WAL/CF, Tokio runtime
ADVANCED: HNSW indexing, nom parsers, SIMD (wide crate), Bincode
DOMAIN:   PACELC theorem, Mechanical Sympathy, LSM-trees, Bloom Filters
COGNITIVE: Bayesian decay, probabilistic inference, LLM prompt engineering
```

---

## 🔑 DECISIONES TÉCNICAS APROBADAS

```
✅ HNSW: NO persistir índice (rebuild en cold start, 3-5s para 100k vec)
✅ Bitset: u128 (128 dims filtrables, cache-friendly)
✅ WAL: Bincode Fase 1 (Arrow IPC diferido a v0.5)
✅ LISP: eval como STNeuron directo en cortex_ram
✅ Amygdala Budget: 5% máximo de cortex_ram protegido por valencia
✅ NeuLISP VM: retorno probabilístico (Value, TrustScore)
✅ Column Families: default | shadow_kernel | deep_memory | tombstones
✅ ResourceGovernor: 2GB OOM limit + 50ms timeout por query
```

---

## 📊 ESTADO DE TESTS Y CI

| Test File | Estado | Fase |
|---|---|---|
| `tests/lisp_logic.rs` | ✅ PASSING | Fase 22 |
| `tests/memory_promotion.rs` | ✅ PASSING | Fase 23 |
| `tests/neural_summarization.rs` | 🔲 PENDIENTE | Fase 27 |
| `tests/hardware_profiles.rs` | 🔲 PENDIENTE | Fase 28 |
| `tests/bloom_filter.rs` | 🔲 PENDIENTE | Fase 29 |
| `tests/memory_rehydration.rs` | 🔲 PENDIENTE | Fase 30 |
| `tests/uncertainty_zones.rs` | 🔲 PENDIENTE | Fase 31 |
| `tests/synaptic_depression.rs` | 🔲 PENDIENTE | Fase 32 |
| `tests/contextual_priming.rs` | 🔲 PENDIENTE | Fase 33 |

---

## 🤖 COMANDOS ANTI-GRAVITY (Atajos de Contexto)

```
"Implementa Fase 27: Neural Summarization"
"Implementa Fase 28: Modo Camaleón"
"Implementa Fase 29: Bloom + MCP"
"Implementa Fase 30: Memory Rehydration"
"Implementa Fase 31: Uncertainty Zones"
"Implementa Fase 32: LTD Synaptic Depression"
"Implementa Fase 33: Contextual Priming"
"Lee docDev/XX antes de código"
"Genera tests primero, luego implementación"
```

---

## CI/CD Y GITHUB ACTIONS

1. **Path Filtering (`rust_ci.yml`)**: Solo dispara con cambios en `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`. Documentación no gasta minutos.
2. **Ejecución Unificada (Monolito)**: Un solo Job secuencial con `--test-threads=2` y swapfile 6GB para no saturar RAM del runner ante las dependencias C++ de RocksDB.
3. **Workflow Dispatch**: Gatillo manual en `release.yml` y `rust_ci.yml` para forzar ejecución desde GitHub UI.

---

## 📈 MÉTRICAS GTM

```
Mes 1:  50 stars · Docker demo publicado
Mes 3: 200 stars · 20 forks · MCP endpoint live
Mes 6: 500 stars · 50 contribs · v0.5 Federación de Lóbulos
```

---

## RECORDATORIOS CRÍTICOS

- El agente SIEMPRE debe verificar el estado de las fases antes de comenzar.
- Los commits deben ser atómicos por fase: **feat(fase-27): Neural Summarization vía Ollama**.
- Al agregar nuevas carpetas raíz al proyecto, actualizar el Path Filtering del `rust_ci.yml`.
- El campo `semantic_valence` en `UnifiedNode` es el nuevo gate de protección de memoria. Nunca eliminarlo.
- La arquitectura de Lóbulos (4 CFs) es inmutable en v0.4.x. Cambios requieren migraciones explícitas.