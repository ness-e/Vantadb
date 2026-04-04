# 🧠 ConnectomeDB — AGENT MAESTRO (v0.4.0 · Actualizado: 2026-04-03)

> **ConnectomeDB** es un Motor de Inferencia Cognitiva escrito en Rust.
> Combina vectores (HNSW), grafos dirigidos y campos relacionales en un único `UnifiedNode` persistido sobre RocksDB.
> El motor aprende, olvida y razona mediante gobernanza biológica.

---

## ⚙️ REGLAS ABSOLUTAS (NUNCA VIOLAR)

1. **LEE `docDev/` ANTES de escribir código.** Cada fase tiene especificación técnica aprobada.
2. **LA NUMERACIÓN DE FASES SIGUE LOS ARCHIVOS DE `docDev/`** (ej. Fase 20 = `20_SleepWorker_Spec.md`).
3. **UNA FASE POR COMMIT.** No mezclar fases distintas en un solo commit.
4. **NUNCA código sin su `.md` de especificación correspondiente en `docDev/`.**
5. **Mover `docDev/XX_*.md` → `complete/XX_*/` SOLO cuando:**
   - ✅ Tests unitarios pasan en CI
   - ✅ Benchmarks dentro de tolerancia
   - ✅ README y CHANGELOG actualizados
6. **GIT PIPELINE RIGUROSO (EN CADA PASO):**
   - `git add .` → `git commit -m "feat(fase-XX): <título>"` → `git push`
   - El cuerpo explica el **QUÉ** y el **POR QUÉ** arquitectónico.
7. **CI PATH FILTERING activo:** `rust_ci.yml` solo dispara ante cambios en `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`.

---

## 🗺️ GLOSARIO RÁPIDO (Ver `docDev/00_Glossary.md`)

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
- RocksDB WAL atómico y serialización `bincode`.

### ✅ v0.2.0 — Motor de Almacenamiento
- RocksDB como motor primario de persistencia.
- Zero-copy buffer pinning (`get_pinned`).
- Bloom Filters (nivel RocksDB) y Block Cache 2GB.

### ✅ v0.3.0 — Aceleración SIMD y Cognición
- SIMD vectorial mediante crate `wide` (`f32x8`) en `cosine_similarity`.
- CP-Index con bitsets `u128` para pre-filtrado semántico.
- `HNSW` para navegación vectorial sub-milisegundo.

### ✅ v0.4.0 — Cognitive OS (ESTADO ACTUAL)
- `UnifiedNode` + campos: `hits`, `last_accessed`, `trust_score`, `semantic_valence`.
- `NeuronType` enum (`STNeuron` / `LTNeuron`) + `CognitiveUnit` trait.
- `LispSandbox` con Cognitive Fuel (1000 steps).
- `NeuLISP VM` (`src/eval/vm.rs`): `OP_VEC_SIM`, `OP_TRUST_CHECK`. Retorna `(Value, TrustScore)`.
- `SleepWorker` (Fase REM): Olvido Bayesiano + migración STN→LTN + Presupuesto de Amígdala (5%).
- `DevilsAdvocate` + `TrustArbiter`: auditoría de escrituras.
- 4 Column Families: `default`, `shadow_kernel`, `deep_memory`, `tombstones`.
- `ResourceGovernor`: OOM guard 2GB + timeout 50ms.
- `LlmClient` (`src/llm.rs`): cliente Ollama para embeddings (`generate_embedding`).
- `GcWorker` (`src/gc.rs`): purga por TTL asíncrona.
- `Server` (`src/server.rs`): endpoints `/health` y `/api/v1/query` via Axum.

---

## 🚦 ROADMAP DE IMPLEMENTACIÓN POR FASES

> Numeración basada en los archivos de `docDev/`. ✅ = implementado. ⚠️ = parcial. 🔲 = pendiente.

---

### ✅ FASE 20 — `20_SleepWorker_Spec.md`
**Archivo:** `src/governance/sleep_worker.rs`

- ✅ `SleepWorker` daemon con cadencia 10s e inception condition (5s inactividad).
- ✅ Fase REM: Olvido Bayesiano `hits *= 0.5` por ciclo.
- ✅ Migración STN→LTN si `hits < 10 && !PINNED && last_accessed > 60s`.
- ✅ Poda al Shadow Archive si `trust_score < 0.2`.
- ✅ **Presupuesto de Amígdala**: máx 5% de `cortex_ram` blindado por `semantic_valence >= 0.8`.
- ✅ Interrupción de Fase REM si se detecta I/O activo del usuario.

---

### ✅ FASE 21 — `21_SIMD_Optimization.md`
**Archivo:** `src/node.rs` → `cosine_similarity()`

- ✅ `wide::f32x8` para procesar 8 floats simultáneos.
- ✅ Fallback escalar automático para hardware sin AVX.
- ✅ HNSW validaciones con read-locks refinados en `src/index.rs`.

---

### ✅ FASE 22 — `22_Lisp_Cognition.md` (Evolutivo: NeuLISP)
**Archivos:** `src/parser/lisp.rs`, `src/eval/mod.rs`, `src/eval/vm.rs`

- ✅ Parser `nom` para S-Expressions balanceadas.
- ✅ `LispSandbox` con Cognitive Fuel (1000 steps).
- ✅ `INSERT` LISP crea nodos directamente como `STNeuron` en `cortex_ram`.
- ✅ **Operador de Similitud (`~`)** nativo: despacha `OP_VEC_SIM`.
- ✅ **Valencia Gated-Macros**: macros condicionadas por `semantic_valence`.
- ✅ NeuLISP VM: `OP_VEC_SIM` (cosine SIMD), `OP_TRUST_CHECK`. Retorno `(Value, TrustScore)`.

---

### ✅ FASE 23 — `23_Sovereignty_Governance.md`
**Archivos:** `src/governance/mod.rs`, `src/executor.rs`

- ✅ `DevilsAdvocate`: detección de contradicciones vectoriales (similitud > 0.95 + Trust divergente).
- ✅ `TrustArbiter`: `Accept / Reject / Shadow`.
- ✅ Bloom Filters a nivel RocksDB para axioma topológico.
- ✅ `trigger_panic_state()` para violaciones de Axiomas de Hierro.
- ✅ Borrados atómicos (`WriteBatch`): Clone → Shadow → Tombstone → Delete.

---

### ✅ FASE 24 — `24_Memory_Hierarchy.md`
**Archivos:** `src/storage.rs`, `src/node.rs`

- ✅ Dualidad `STNeuron` (RAM / `cortex_ram`) vs `LTNeuron` (RocksDB SST).
- ✅ Promoción dinámica LTN→STN al alcanzar `hits >= 50`.
- ✅ `last_query_timestamp: AtomicU64` para perfilado de inactividad.
- ⚠️ **PENDIENTE:** Estrategia mmap para el Neural Index en hardware limitado (< 16GB RAM).

---

### ✅ FASE 25 — `25_Lobe_Segmentation.md`
**Archivo:** `src/storage.rs`

- ✅ 4 Column Families: `default` (Primario), `shadow_kernel` (Sombra), `deep_memory` (Histórico), `tombstones`.
- ✅ Aislamiento de I/O entre lóbulos.
- ⚠️ **PENDIENTE:** Compresión Zstd diferenciada para `shadow_kernel` y `deep_memory` (actualmente usa LZ4 global).
- ⚠️ **PENDIENTE:** Flag Read-Only explícito para `deep_memory` (actualmente es convención, no enforcement).

---

### ✅ FASE 26 — `26_Bayesian_Forgetfulness.md` (COMPLETA)
**Archivos:** `src/governance/sleep_worker.rs`, `src/llm.rs`, `src/storage.rs`

- ✅ Poda de Entropía: `hits *= 0.5` por ciclo REM.
- ✅ Axioma de Inmortalidad (`PINNED` flag).
- ✅ Tabla de estados (Lúcido / Dudoso / Onírico / Difunto).
- ✅ `LlmClient` con `generate_embedding()` vía Ollama.
- ✅ Detección de grupos "Oníricos" por campo `belongs_to_thread` (clustering por thread).
- ✅ `LlmClient::summarize_context()` con prompt estructurado (incluye `semantic_valence`, `keywords`, `trust_score`).
- ✅ Creación de **Neurona de Resumen** (`NeuralSummary`) en `deep_memory` CF con linaje semántico (`ancestors`).
- ✅ Movimiento atómico de originales a `shadow_kernel` como `AuditableTombstone` (solo si resumen exitoso).
- ✅ Presupuesto de tiempo: 8s máx para Stage 3 (`MAX_SUMMARIZATION_DURATION_MS`).
- ✅ Validación de peso mínimo: grupos con `sum(hits) < 3` se purgan sin gastar LLM.
- ✅ `StorageEngine::consolidate_node()` — fix del gap HNSW (sincroniza disco + index).
- ✅ `StorageEngine::insert_to_cf()` — escritura directa a CFs nombrados.
- ✅ Test `tests/neural_summarization.rs` (4 tests unitarios + 1 test integración `#[ignore]`).


---

### 🔲 FASE 27 — `27_Hardware_Adapters.md` → **Modo Camaleón**
**Archivos a crear/modificar:** `src/config.rs` (nuevo), `src/main.rs`, `src/storage.rs`, `Cargo.toml`

- 🔲 Enum `HardwareProfile { Survival, Standard, Enterprise }` en `src/config.rs`.
- 🔲 Función `detect_hardware_profile() -> HardwareProfile`:
  - CPU: `std::is_x86_feature_detected!("avx512f")` → activa SIMD full.
  - RAM: crate `sysinfo` → si total < 16GB → `SurvivalProfile` forzado.
  - I/O: escritura dummy para medir latencia del directorio de datos.
- 🔲 Inyectar perfil en `StorageEngine::open()`:
  - `Survival`: BlockCache 512MB, SleepWorker cada 5s, I8 quantization.
  - `Enterprise`: BlockCache ∝ RAM, poda diferida, FP32 completo.
- 🔲 Throttling Cognitivo: delay configurable entre inferencias si CPU sobreca lentada.
- 🔲 Test `tests/hardware_profiles.rs`.

---

### ⚠️ FASE 28 — `28_Inference_Optimization.md` (PARCIAL)
**Archivos:** `src/storage.rs`, `src/server.rs`, `Cargo.toml`

- ✅ NeuLISP VM (`src/eval/vm.rs`) con bytecode `OP_VEC_SIM` + `OP_TRUST_CHECK`.
- ✅ SIMD para `OP_VEC_SIM` usando `cosine_similarity` existente.
- ✅ Servidor Axum con `/health` y `/api/v1/query`.
- 🔲 **FALTA:** Bloom Filter explícito en RAM por CF (pre-filtro de existencia antes de RocksDB lookup).
  - *(El actual Bloom Filter es solo la opción `set_bloom_filter(10.0)` de RocksDB SST, no un filtro en memoria controlable manualmente desde Rust).*
- 🔲 **FALTA:** Endpoint `POST /mcp/context` → ingesta contexto de agente como `STNeuron`.
- 🔲 **FALTA:** Endpoint `GET /mcp/axioms` → devuelve Axiomas de Hierro activos como JSON.
- 🔲 **FALTA:** Tests `tests/bloom_filter.rs` y `tests/mcp_integration.rs`.

---

### ✅ FASE 29 — `29_NeuLISP_Spec.md`
**Archivos:** `src/eval/vm.rs`, `docDev/29_NeuLISP_Spec.md`

- ✅ Especificación de gramática NeuLISP con operador `~`.
- ✅ NeuLISP VM bytecode con pila de floats y vectores.
- ✅ Inferencia probabilística: retorno `(Value: f32, TrustScore: f32)`.
- ✅ Penalización de `TrustScore` si similitud es incalculable.

---

### 🔲 FASE 30 — `30_Memory_Rehydration_Protocol.md` → **Arqueología Semántica**
**Archivos a crear/modificar:** `src/eval/vm.rs`, `src/storage.rs`, `src/governance/sleep_worker.rs`

- 🔲 Opcode `OP_REHYDRATE` en NeuLISP VM.
- 🔲 Función `StorageEngine::rehydrate(summary_id: u64) -> Result<Vec<UnifiedNode>>`:
  - Busca nodos con relación `belonged_to` en `shadow_kernel` CF.
  - Los restaura como `STNeuron` de solo lectura (`PINNED`).
- 🔲 Auto-proponer rehydration si `TrustScore < 0.4` en Neurona de Resumen.
- 🔲 Post-aprobación del usuario: consolidar nodos y elevar `TrustScore`.
- 🔲 Test `tests/memory_rehydration.rs`.

---

## 🧬 NUEVAS FASES PROPUESTAS (Post-30, sin docDev aún)

> Estas fases requieren **primero crear su `docDev/XX_*.md`** antes de implementar.

---

### 🔲 FASE 31 — **Uncertainty Zones (Superposición Lógica)**
**Spec a crear:** `docDev/31_Uncertainty_Zones.md`

Concepto: Nodos en "superposición" (contradicciones pendientes de colapso temporal).
- `QuantumNeuron { candidates: Vec<UnifiedNode>, collapse_deadline_ms: u64 }`.
- `TrustArbiter` colapsa el candidato de mayor `TrustScore` al vencer el plazo.
- Contradictions del `DevilsAdvocate` crean `QuantumNeuron` en lugar de rechazar.

---

### 🔲 FASE 32 — **LTD Synaptic Depression (Edges)**
**Spec a crear:** `docDev/32_Synaptic_Depression.md`

Concepto: Decaimiento del peso de los `Edge` no traversados (Long-Term Depression biológica).
- Campos `last_traversed_ms: u64`, `traversal_count: u32` en `Edge`.
- `SleepWorker` aplica `edge.weight *= 0.95` a edges sin traversal en 24h.
- Si `edge.weight < 0.05` → remover edge y registrar `AuditableTombstone`.

---

### 🔲 FASE 33 — **Contextual Priming (Caché Anticipatorio)**
**Spec a crear:** `docDev/33_Contextual_Priming.md`

Concepto: Pre-cargar vecinos de nodos populares antes de ser consultados.
- En `StorageEngine::get()`: si `hits > 20` → `tokio::spawn` carga edges nivel 1 a `cortex_ram`.
- Límite: máx 50 nodos por operación de priming.
- Configurable: `CONNECTOME_PRIMING_ENABLED=true/false`.

---

### 🔲 FASE 34 — **mmap Neural Index (Survival Mode)**
**Spec a crear:** `docDev/34_MMap_NeuralIndex.md`

Concepto: Completar el pendiente de Fase 24 — acceso a vectores via Memory-Mapped Files.
- Mapear descriptores vectoriales del HNSW desde disco al espacio virtual.
- Activar automáticamente en `SurvivalProfile` (RAM < 16GB).

---

## 📊 ESTADO REAL DE TESTS

| Test File | Estado | Fase Doc |
|---|---|---|
| `tests/lisp_logic.rs` | ✅ PASSING | Fase 22 |
| `tests/memory_promotion.rs` | ✅ PASSING | Fase 24 |
| `tests/neural_summarization.rs` | ✅ IMPLEMENTED | Fase 26 |
| `tests/hardware_profiles.rs` | 🔲 PENDIENTE | Fase 27 |
| `tests/bloom_filter.rs` | 🔲 PENDIENTE | Fase 28 |
| `tests/mcp_integration.rs` | 🔲 PENDIENTE | Fase 28 |
| `tests/memory_rehydration.rs` | 🔲 PENDIENTE | Fase 30 |
| `tests/uncertainty_zones.rs` | 🔲 PENDIENTE | Fase 31 |
| `tests/synaptic_depression.rs` | 🔲 PENDIENTE | Fase 32 |
| `tests/contextual_priming.rs` | 🔲 PENDIENTE | Fase 33 |

---

## 🎯 OBJETIVOS CRÍTICOS

```
✅ MVP: 1M nodos + 100k vectores en 16GB RAM
✅ Latencia: <20ms hybrid queries
✅ Overhead: <128MB cold start
✅ Docker: <5min setup mundial
✅ Integración: Ollama native (CONNECTOME_LLM_URL)
✅ NeuLISP: retorna (Value, TrustScore) — inferencia probabilística
🔲 Neural Summarization: grupos degenerados → Neurona de Resumen via Ollama (Fase 26)
🔲 Modo Camaleón: auto-detección Survival/Standard/Enterprise (Fase 27)
🔲 Bloom explícito en RAM + MCP Endpoint /mcp/context (Fase 28)
🔲 Memory Rehydration: OP_REHYDRATE + arqueología semántica (Fase 30)
```

---

## 🚫 LIMITACIONES TÉCNICAS

```
❌ NO cloud-first (target: 16GB laptop edge)
❌ NO ML-heavy (heurístico → estadístico → LLM solo para compresión cognitiva)
❌ NO sharding en v0.4.x (diferido a v0.5+ Enterprise)
❌ NO mutaciones directas en deep_memory sin cirugía lógica explícita
```

---

## 🔑 DECISIONES TÉCNICAS APROBADAS

```
✅ HNSW: NO persistir índice (rebuild en cold start, 3-5s para 100k vec)
✅ Bitset: u128 (128 dims filtrables, cache-friendly)
✅ WAL: Bincode Fase 1 (Arrow IPC diferido a v0.5)
✅ LISP INSERT: crea STNeuron directamente en cortex_ram
✅ Amygdala Budget: 5% máximo de cortex_ram protegido por valencia >= 0.8
✅ NeuLISP VM: retorno probabilístico (Value, TrustScore)
✅ 4 Column Families: default | shadow_kernel | deep_memory | tombstones
✅ ResourceGovernor: 2GB OOM limit + 50ms timeout por query
✅ LlmClient: Ollama vía CONNECTOME_LLM_URL + CONNECTOME_LLM_MODEL
```

---

## 🤖 COMANDOS ANTI-GRAVITY

```
"Implementa Fase 26: Neural Summarization completa (Ollama)"
"Implementa Fase 27: Modo Camaleón / Hardware Profiles"
"Implementa Fase 28: Bloom explícito en RAM + endpoints MCP"
"Implementa Fase 30: Memory Rehydration / OP_REHYDRATE"
"Implementa Fase 31: Uncertainty Zones / QuantumNeuron"
"Implementa Fase 32: LTD Synaptic Depression en Edges"
"Implementa Fase 33: Contextual Priming / Caché Anticipatorio"
"Implementa Fase 34: mmap Neural Index para Survival Mode"
"Lee docDev/XX antes de código"
"Genera tests primero, luego implementación"
```

---

## CI/CD Y GITHUB ACTIONS

1. **Path Filtering (`rust_ci.yml`)**: Solo dispara con cambios en `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`.
2. **Ejecución Unificada (Monolito)**: Un solo Job secuencial con `--test-threads=2` y swapfile 6GB.
3. **Workflow Dispatch**: Gatillo manual en `release.yml` y `rust_ci.yml`.
4. **Recordatorio**: Al agregar nuevas carpetas raíz, actualizar el path filtering en `rust_ci.yml`.

---

## 📈 MÉTRICAS GTM

```
Mes 1:  50 stars · Docker demo publicado
Mes 3: 200 stars · 20 forks · MCP endpoint live
Mes 6: 500 stars · 50 contribs · v0.5 Federación de Lóbulos
```