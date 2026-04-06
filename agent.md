# 🧠 ConnectomeDB — AGENT MAESTRO (v0.5.0 · Actualizado: 2026-04-05)

> **ConnectomeDB** es un Motor de Inferencia Cognitiva escrito en Rust.
> Combina vectores (HNSW), grafos dirigidos y campos relacionales en un único `UnifiedNode` persistido sobre RocksDB.
> El motor aprende, olvida y razona mediante gobernanza biológica.

---

## ⚙️ REGLAS ABSOLUTAS (NUNCA VIOLAR)

1. **LEE `docDev/` ANTES de escribir código.** Cada fase tiene especificación técnica aprobada.
2. **LA NUMERACIÓN DE FASES SIGUE LOS ARCHIVOS DE `docDev/`** (ej. Fase 31 = `31_Uncertainty_Zones.md`).
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
| **Rehydration** | `StorageEngine::rehydrate()` | Arqueología Semántica desde Shadow Archive |

---

## 📦 HISTORIAL DE VERSIONES

### ✅ v0.1.0 — Fundación
Parser IQL, `UnifiedNode`, serialización bincode.

### ✅ v0.2.0 — Motor de Almacenamiento
RocksDB, Zero-copy pinning, Bloom Filters nativos.

### ✅ v0.3.0 — Aceleración SIMD y Cognición
SIMD vectorial (`wide`), CP-Index bitsets `u128`, HNSW.

### ✅ v0.4.0 — Cognitive OS (Fases 20–30)
Arquitectura Cognitiva completa: NeuronType, CognitiveUnit, SleepWorker, DevilsAdvocate, NeuLISP VM, MCP STDIO, Modo Camaleón, Neural Summarization, Memory Rehydration.

### 🚧 v0.5.0 — Quantum Cognition (EN PROGRESO)
Siguiente evolución: Superposición Lógica, Depresión Sináptica, Caché Anticipatorio.

---

## 🏗️ ARQUITECTURA ACTIVA (v0.5.0)

### Archivos Principales
| Archivo | Responsabilidad |
|---|---|
| `src/node.rs` | `UnifiedNode`, `VectorData`, `Edge`, `NodeFlags` (8 flags: ACTIVE..REHYDRATED), `CognitiveUnit` trait |
| `src/storage.rs` | `StorageEngine` — RocksDB multi-CF, `cortex_ram`, `rehydrate()`, Bloom L0 Pinning |
| `src/executor.rs` | `Executor` — Orquestador IQL/LISP híbrido, `StaleContext` no-bloqueante |
| `src/index.rs` | `CPIndex` — HNSW vectorial con bitset pre-filtering |
| `src/eval/vm.rs` | `NeuLispVM` — Bytecode: `OpPushFloat`, `OpPushVector`, `OpTrustCheck`, `OpVecSim`, `OpRehydrate` |
| `src/eval/mod.rs` | `LispSandbox` — Parser + Fuel-limited execution |
| `src/parser/` | IQL parser (`nom`) + LISP S-Expression parser |
| `src/governance/` | `DevilsAdvocate`, `TrustArbiter`, `SleepWorker` (REM + Neural Summarization + Rehydration Purge) |
| `src/hardware/mod.rs` | `HardwareCapabilities` — Modo Camaleón (Survival/Performance/Enterprise) |
| `src/server.rs` | HTTP API Axum (`/health`, `/api/v1/query`) |
| `src/api/mcp.rs` | MCP STDIO (JSON-RPC 2.0) — `query_lisp`, `search_semantic`, `inject_context`, `read_axioms` |
| `src/llm.rs` | `LlmClient` — Ollama bridge (`generate_embedding`, `summarize_context`) |

### Column Families (RocksDB)
| CF | Propósito | Bloom Pinning |
|---|---|---|
| `default` | Datos primarios activos | ✅ L0 pinned |
| `deep_memory` | Neuronas de Resumen (LTN inmutables) | ✅ L0 pinned |
| `shadow_kernel` | Archive arqueológico (nodos originales pre-tombstone) | ❌ |
| `tombstones` | Registro auditable de eliminaciones | ❌ |

### NodeFlags Bitfield
| Bit | Constante | Propósito |
|---|---|---|
| 0 | `ACTIVE` | Nodo vivo |
| 1 | `INDEXED` | Indexado en HNSW |
| 2 | `DIRTY` | Pendiente de flush |
| 3 | `TOMBSTONE` | Eliminado lógicamente |
| 4 | `HAS_VECTOR` | Tiene embedding vectorial |
| 5 | `HAS_EDGES` | Tiene aristas |
| 6 | `PINNED` | Inmutable a recolección |
| 7 | `REHYDRATED` | Provenance arqueológica (dato resucitado del Shadow) |

---

## 🚦 FASES COMPLETADAS (v0.4.0 · Resumen Ejecutivo)

> Detalle completo de cada fase en `docDev/XX_*.md`.

| Fase | Nombre | Test | Estado |
|---|---|---|---|
| 20 | SleepWorker (Circadian GC) | — | ✅ |
| 21 | SIMD Optimization | — | ✅ |
| 22 | Lisp Cognition / NeuLISP | `lisp_logic.rs` | ✅ |
| 23 | Sovereignty Governance | — | ✅ |
| 24 | Memory Hierarchy | `memory_promotion.rs` | ✅ |
| 25 | Lobe Segmentation | — | ✅ |
| 26 | Bayesian Forgetfulness + Neural Summarization | `neural_summarization.rs` | ✅ |
| 27 | Hardware Adapters (Modo Camaleón) | `hardware_profiles.rs` | ✅ |
| 28 | Inference Optimization (MCP + Bloom) | `mcp_integration.rs` | ✅ |
| 29 | NeuLISP Spec (VM Bytecode) | — | ✅ |
| 30 | Memory Rehydration (Arqueología Semántica) | `memory_rehydration.rs` | ✅ |

---

## 🔲 ROADMAP v0.5.0 (FASES PENDIENTES)

> Cada fase requiere **primero crear su `docDev/XX_*.md`** antes de implementar.

---

### 🔲 FASE 31 — **Hybrid Quantization & Reactive Invalidation**
**Spec:** `docDev/31_Hybrid_Quantization_Architecture.md`

Concepto: Cuantización de 3 niveles para vector indexing y validación axiomática con backpressure.
- `VectorRepresentations`: `Binary(L1)`, `Turbo(L2)`, `Full(L3)`.
- Re-ranking L2 y validación L3 con `InvalidationDispatcher` para Pánico Axiomático.

---

### 🔲 FASE 32 — **Uncertainty Zones (Superposición Lógica)**
**Spec:** `docDev/32_Uncertainty_Zones.md`

Concepto: Nodos en "superposición" generados por la disonancia de cuantización e I/O.
- `QuantumNeuron { candidates: Vec<UnifiedNode>, collapse_deadline_ms: u64 }`.
- Si el nivel L3 contradice el L2, el Devil's Advocate empuja el nodo a incertumbre en lugar de descartarlo de inmediato.

---

### 🔲 FASE 33 — **LTD Synaptic Depression (Edges)**
**Spec:** `docDev/33_Synaptic_Depression.md`

Concepto: Decaimiento del peso de los `Edge` generados como ruido espaciotemporal (Hash Collisions L1).
- `SleepWorker` aplica decaimiento a edges sin traversal.
- Limpia el grafo fantasma producido por el índice binario RaBitQ de la Fase 31.

---

### 🔲 FASE 34 — **Contextual Priming (Caché Anticipatorio)**
**Spec:** `docDev/34_Contextual_Priming.md`

Concepto: Pre-cargar bloques TurboQuant MMap y vecinos de nodos calientes a L1 RAM.
- Carga predictiva ante hits altos para mitigar I/O bottleneck en el Executor.

---

### 🔲 FASE 35 — **mmap Neural Index (Survival Mode)**
**Spec:** `docDev/35_MMap_NeuralIndex.md`

Concepto: Configuración de hardware para almacenar L2 (Turbo 3-bit) fuera de la RAM.
- Activar de forma selectiva MMap fallback si Hardware == Survival Profile (< 16GB).

---

## 📊 ESTADO DE TESTS

| Test File | Estado | Fase |
|---|---|---|
| `tests/parser.rs` | ✅ PASSING | Core |
| `tests/lisp_logic.rs` | ✅ PASSING | 22 |
| `tests/structured_api_v2.rs` | ✅ PASSING | 22 |
| `tests/memory_promotion.rs` | ✅ PASSING | 24 |
| `tests/neural_summarization.rs` | ✅ PASSING | 26 |
| `tests/hardware_profiles.rs` | ✅ PASSING | 27 |
| `tests/mcp_integration.rs` | ✅ PASSING | 28 |
| `tests/vector_scale_check.rs` | ✅ PASSING | 28 |
| `tests/memory_rehydration.rs` | ✅ PASSING | 30 |

---

## 🔑 DECISIONES TÉCNICAS APROBADAS

- HNSW: NO persistido (rebuild en cold start, 3-5s para 100k vec)
- Bitset: `u128` (128 dims filtrables, cache-friendly)
- LISP INSERT: crea `STNeuron` directamente en `cortex_ram`
- Amygdala Budget: 5% máximo de `cortex_ram` protegido por `semantic_valence >= 0.8`
- NeuLISP VM: retorno probabilístico `(Value, TrustScore)`
- 4 Column Families: `default` | `shadow_kernel` | `deep_memory` | `tombstones`
- ResourceGovernor: 2GB OOM limit + 50ms timeout por query
- LlmClient: Ollama vía `CONNECTOME_LLM_URL` + `CONNECTOME_LLM_MODEL`
- Bloom Filter: nativo RocksDB (10 bits/key), L0 pinned en `default` y `deep_memory`
- MCP: STDIO puro (JSON-RPC 2.0), logs a stderr con `--mcp`
- Rehydration: Non-blocking `StaleContext` + Transparencia Selectiva

---

## 🚫 LIMITACIONES TÉCNICAS

- ❌ NO cloud-first (target: 16GB laptop edge)
- ❌ NO ML-heavy (heurístico → estadístico → LLM solo para compresión cognitiva)
- ❌ NO sharding en v0.5.x (diferido a v1.0 Enterprise)
- ❌ NO mutaciones directas en `deep_memory` sin cirugía lógica explícita

---

## CI/CD Y GITHUB ACTIONS

1. **Path Filtering (`rust_ci.yml`)**: Solo dispara con cambios en `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`.
2. **Ejecución Unificada (Monolito)**: Un solo Job secuencial con `--test-threads=2` y swapfile 6GB.
3. **Workflow Dispatch**: Gatillo manual en `release.yml` y `rust_ci.yml`.

---

## 📈 MÉTRICAS GTM

```
Mes 1:  50 stars · Docker demo publicado
Mes 3: 200 stars · 20 forks · MCP endpoint live
Mes 6: 500 stars · 50 contribs · v0.5 Quantum Cognition
```