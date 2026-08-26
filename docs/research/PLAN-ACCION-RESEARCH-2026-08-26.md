---
title: "Plan de Acción Research — Validación de 53 investigaciones (2026-08-26)"
type: plan
status: active
date: 2026-08-26
scope: Consolidación de docs/research/*.md (53 archivos) — pendientes VIGENTES + cerrados verificados + referencias únicas
method: lectura completa (52 directos + 2 gigantes vía sub-agentes) + verificación mecánica contra código/backlog/workflows/registries hoy
convencion_carpeta: >
  Esta carpeta es el hogar único de investigación (GOV-D4). Campañas estructuradas =
  subcarpeta con PLAN.md + NN-*.md + SYNTHESIS.md. Investigaciones puntuales = archivo
  single-file aquí. Historial archivado en docs/research/archive/.
---

# Plan de Acción — Research Unificado

> **Fecha de validación:** 2026-08-26 · Branch develop @ `c141c1ce`
> **Fuente:** los 53 archivos de `docs/research/` (ahora en `archive/`, índice abajo).
> Cada hallazgo fue **verificado contra el código actual** antes de clasificarlo.

---

## 1. Pendientes VIGENTES priorizadas

### 🟠 Alta — producto/engine

| ID | Tarea | Origen | Evidencia de vigencia hoy |
|---|---|---|---|
| **ACID-4a** | **WAL v2 `Prepare`** — keystone de rollback multi-capa (errores truthful + MVCC stamps sobreviven restart). Diseño completo en archive/res01+ACID_ROLLBACK_DESIGN; conditional GO pendiente ADR owner (S1-S2 tras flag → S5 bench → decisión) | res01 / ACID_ROLLBACK_DESIGN / Backlog **RES-01** ⬜ | Fila Backlog Pendiente; `WAL_FORMAT_VERSION=1` sin Prepare confirmado 2026-08-25. Ya ticketeado — no duplicar |
| **ACID-GAP** | **Truthful-error gap**: apply failure tras Commit durable → ops "resucitan" en restart (`txn.rs` + replay). Fixable independiente del WAL v2 (Abort-after-Commit o defer buffer drop hasta apply OK) | res01 §8 | Bug lógico abierto; solapa RES-01 pero tiene fix mínimo autónomo |
| **PHRASE-01** | **Sintaxis IQL phrase** (`Condition::TextMatch(field, query)` + parser reusando `string_literal`) — el matching ya funciona (`phrase.rs`, enforcement en `lexical_search`), falta exponerlo por IQL | INV-009 gap#1 | Verificado 2026-08-04 y no tocado desde: parser sin condición de texto |
| **PHRASE-02** | Tokenización literal de frases (sin stopwords/stemming dentro de comillas — puede romper adyacencia) + highlight de frase completa en snippets (`<strong>machine learning</strong>`) | INV-009 gaps §6.3-4 | Storage listo; solo snippet.rs + query_plan_with_config |
| **BATCH-Q** | **`search_batch_requests` Python** — batch híbrido/filtrado con SearchRequest completo (namespace/text_query/filters/explain), patrón Rayon+GIL eager ya probado. 1-2 días, 0 deps nuevas | INV-008 (decisión Opción 2 firmada) | Hoy solo `search_batch` vector-only; el core soporta todo. Diseño completo listo para ejecutar |
| **CRASH-KILL** | **Test de kill REAL a mitad de escritura** (`tests/storage/crash_kill_recovery.rs` según plan §5) + separar failpoint tests en binario propio (race entre tests hermanos, flaky local) + failpoint fsync falso | FND-15-01/03/05 | Confirmado hoy: ni `crash_kill_recovery.rs` ni `chaos_failpoints.rs` existen; los suites pasan solo serial |

### 🟡 Media

| ID | Tarea | Origen | Evidencia de vigencia hoy |
|---|---|---|---|
| **DOCS-RUSTDOC** | **Job CI rustdoc** (`cargo doc --no-deps --workspace` + artifact) — 0 deps nuevas, gatea docstrings rotos. Plan YAML completo en archive/FND-17 §5 (solo faltaba aprobación lead para tocar workflows) | FND-17 | Confirmado hoy: ningún workflow genera docs (solo `gate-docs-21.yml` valida). typedoc/pdoc siguen diferidos (0 docstrings en SDKs) |
| **PY-CM** | Context manager nativo en `VantaDB` sync (`__enter__`/`__exit__` → close) ~10 líneas PyO3 + README quickstart. FND-05 dejó prototipo en `docs/examples/fnd05_python_context_manager.py` | FND-05 PY-4 | Confirmado hoy: 0 matches `__enter__` en `vantadb-python/src/lib.rs`. Los stub-drift PY-1/2/3 SÍ quedaron resueltos (MOD-18 test_stub_drift) |
| **TS-DISPOSE** | `[Symbol.asyncDispose]` en `NativeVantaDB` (+ tsconfig lib esnext.disposable) + normalizar tagged-union `VantaMetadata` | FND-05 TS-1/TS-2 | No implementado (FND-05 recommendation; verificado parcialmente) |
| **MEM-GUARD** | Validar `DEFAULT_RSS_THRESHOLD=0.80` con RSS real (FND-01-F2) + correr bench full scale [10k..100k] en CI heavy (FND-01-F3) | FND-01 §6 | F1 aplicado; F2/F3 ⬜ declarados pendientes en el propio doc |
| **SESS-DOCS** | 2 pages docs-only (~medio día): convención session↔thread_id↔namespace (threads/scenes/genlog) + guía "connect vantadb-mcp a Claude Code". Único residuo viable del roadmap Cognee (DEC-01 = no-go-as-scoped) | COGNEE_EVALUATION / res03 | Confirmado hoy: ninguna de las 2 páginas existe. DEC-01 resuelto NO-GO salvo esto — no construir cache/sync/improve |
| **VECTARA-1** | Tutorial "Migrate from Vectara" + comparativa Chroma→VantaDB (oportunidad de mercado real: Vectara cerró self-service 2026) | vectara-competitive-research | Solo existe stub en book SUMMARY; falta tutorial activo. Prioridad post-Show-HN |
| **DESK-ADAPT** | Adaptadores desktop MCP-sidecar-rmcp formal / drivers Node-Python stdio — nunca materializados. YAGNI salvo demanda demostrada (la UI ya cubre embebida/server/proxy/MCP por otro camino: `child_process.rs`) | DESKTOP-01b | Sub-agente confirma: adapter mcp/node/python/wasm/jsonrpc .rs no existen; decisión de reabrir solo con señal de usuarios |

### 🟢 Baja / verificar oportunista

| ID | Tarea | Origen | Nota |
|---|---|---|---|
| VERIFY-BIN | Dockerfile CRIT-07/08 (dirs eliminados, Rust < MSRV) + providers fuera de workspace | VantaDB-28-07 | Re-verificar en 5 min; plausiblemente ya corregido (Docker/PITR/etc cambiaron mucho desde v0.4) |
| SIGN-WIN | Authenticode signing Windows (decisión negocio/certificado) | VantaDB-28-07 / research-desktop H-08 | Duplicado con deuda desktop conocida |
| BENCH-RAG | Benchmarks RAG reales (no solo SIFT sintético) + comparativa honesta vs LanceDB/sqlite-vec | VantaDB-28-07 / INV-007 | INV-7 ya construyó harness (`competitive_bench.py`); falta emitir JSON contrato + Slice 2 tabla web |
| IQL-RBAC | RBAC integrado al parser IQL (hoy roles por método HTTP solamente) | legacy (señalado) | Solapa SRV scoping-by-namespace; decidir juntos |
| NUM-NODE | napi TypedArray output (vector Float32Array, patrón PERF-08, ~2h) — reabrir si profiling Node muestra serde_json >30% del tiempo de query | FND-04 plan futuro | Señal de reapertura medible definida en FND-04 §4 |

---

## 2. Resuelto / superado (NO re-ticketear — verificado hoy)

| Hallazgo del research | Estado real hoy | Evidencia |
|---|---|---|
| ERR-022 clamp top_k (alloc gigante, crash) | ✅ RESUELTO — clamps core/MCP/HTTP presentes | Audits 20260822 verifican `top_k.min`; MCP `min(config.max_list_limit)` en tools.rs:790 |
| ERR-021 MCP OOM (materializa tablas completas) | ✅ RESUELTO — paginación/límites | `limit.min(max_list_limit)` tools.rs:790; deep-module MCP 8/10 con tests round-trip |
| UAF `__array_interface__` (SEC-01) | ✅ RESUELTO — copia owned deliberada | types.rs anti-UAF; FND-04 lo confirma como decisión de seguridad |
| ERR-016 parser WHERE/RANK alias | ✅ RESUELTO 2026-08-09 | `non_keyword_ident` + tests |
| ERR-035/036 read-lock global hot path | ✅ RESUELTO — `src/physical_plan.rs` eliminado; FND-02 fixes H1-H3 + get_many try_write | hot path limpio (audit 200850) |
| ERR-010 raza checkpoint↔snapshot | ✅ RESUELTO v0.4.0 | maintenance.rs lock único |
| Cognee phases 1/3/4 (session cache, sync/improve, lessons) | ✅ NO-GO decidido (DEC-01) — primitivas ya existen (threads/scenes/genlog/context_assemble/axioms) | res03 verifica existencia real en código |
| Cognee Phase 2 plugin Claude Code | ✅ innecesario como plugin — stdio MCP ya sirve cualquier cliente; Streamable HTTP resuelto en spec; quedó SESS-DOCS como residuo docs | res03 §Phase 2 |
| INV-005 error boundary web + dep muerta `@mdxeditor` | ✅ RESUELTO | `web/src/app/error.tsx` existe; @mdxeditor 0 hits en package.json |
| INV-013 JSON-LD ausente | ✅ RESUELTO (WDA-07) | `application/ld+json` en layout.tsx hoy |
| INV-014 plomería dark inerte | ✅ RESUELTO (WDA-05/WDA-01) | theme-provider.tsx eliminado; next-themes fuera |
| INV-015 touch targets <44px | ✅ RESUELTO (WDA-04) | 0 errores lint-a11y dirigidos post-fix |
| INV-016 motion tokens ausentes | ✅ RESUELTO | `--duration-*` + `--ease-default` en globals.css:52-55 hoy |
| FND-16 WASM build sin gate PR | ✅ IMPLEMENTADO | `pull_request:` trigger presente en release-npm-61.yml hoy |
| INV-017 sccache CI | ✅ IMPLEMENTADO | rust-setup action usa sccache-action pinneado SHA (verificado en audits posteriores) |
| FND-13 Regla 11 claims sin fuente | ✅ aplicada + WDA-03 eliminó 9 claims falsos del frontend (incluye los números fantasma BENCH01 de vanta-data.ts) | web-design-audit §Información |
| bincode maintenance risk (CRIT SEC) | ✅ migración a postcard hecha | 0 matches bincode en Cargo.toml/Cargo.lock |
| CRIT-06 durabilidad WAL efectiva nula | ✅ MITIGADO | `DEFAULT_PERIODIC_THRESHOLD=1` → sync cada record (wal.rs:340); FAQ corregida |
| TIR-02 recovery time métrica | ✅ IMPLEMENTADO | `recoveryPairs()` en evals/dora.mjs:207 |
| TIR-08 saturación/broadening en prompts | ✅ IMPLEMENTADO | research-agent.md:31-32 |
| TIR-07/TSYS-06 chaos runner task-system | ✅ RESUELTO — behavior changes + parsers.mjs + 29 tests; runner Fase 4 DEFERIDO ≤2026-09 | TSYS-06 §10 COMPLETO |
| TIR-04 dead-letter queue | ✅ Decisión: contenedor citado (tasks/closed/) — infra nueva WONTFIT | TIR-04 |
| TIR-06 post-release monitoring | ✅ DECIDIDO: DEFER, per-release ownership (DoD + /ship) | TIR-06 |
| TIR-05 LLM-as-judge | ✅ DECIDIDO: DEFER reactivo (<$1/trim, volumen bajo, review humano alcanza) | TIR-05 |
| TIR-01 compaction runtime | ✅ DECIDIDO: WONTFIT técnico (OpenCode no permite editar historial); micro-cambio prompt opcional | TIR-01 |
| META-001 falsos negativos backlog | ✅ mitigado (skill progreso + derivación atómica Q8 INV-DECIDE) | meta-001 |
| FND-01 guard RSS ciego | ✅ F1 aplicado (guard lee RSS real del proceso) | bench: pressure_ratio 0.002→0.011 lineal |
| FND-02 reentrancia insert_lock/deadlock evicción | ✅ Fijas aplicadas + M3 race delete↔consolidate cerrado + tests + Regla 8 | FND-02 §4/§9 |
| FND-03 wheels feature set | ✅ ESTADO OK sin cambios (verificación exitosa) | FND-03 |
| FND-12/AVANZADO HNSW CRUD rebuild gap vs Milvus | ✅ Superado — snapshots físicos + quiesce + restore (MCP-34a/b) + tombstones | commits FIND-25/29d21cba/bde2fc9e |
| DESKTOP-01 decisión plataforma Tauri | ✅ CONFIRMADA — app construida y operando | desktop/ completo con WorkspaceShell+lenses+CSP+E2E |
| MVP frontend DESKTOP-01b | ✅ Superado — creció a workspace multi-lens no anticipado por el plan | sub-agente evaluación |
| Workspace desacoplado src-tauri | ✅ vigente y correcto | estructura actual |
| Ollama/OpenAI provider trait Rust `src/llm.rs` | sigue vivo pero **solapado con PROV-14** (ADR unificación pendiente, ya en Backlog P-providers) | doble surface documentada en research-providers |

### Decisiones coyunturales ya tomadas (referencia)

- **INV-007:** ann-benchmarks muerto → usar solo datasets HDF5 + metodología Recall/QPS; harness propio standalone. (JSON contrato + tabla web aún pendientes → ver BENCH-RAG arriba.)
- **INV-009:** NO tantivy (YAGNI total — positions/matching propios con 12 tests).
- **INV-012 anti-locality:** WONTFIX re-confirmado (~7% mejora < umbral 15%).
- **INV-011 core/server separation:** limpia, no cambiar nada.
- **DESktop eficiencias/ROIs:** Tauri sobre Electron validado con fuentes; benchmark propio sigue siendo gap G1 (Regla 11).

---

## 3. Referencias únicas preservadas en archive/

Estos archivos son extracciones/analysis que NO debe perderse aunque estén históricos:

- **Competidores profundos:** INV-018 (Weaviate, 34 refs), INV-020 (Milvus, bitsets/segments), INV-026 (Pinecone, Slabs/Ananas/LSN), vectara (pivot 2026), legacy-docs-investigacion (matriz 10 competidores) — alimento directo para product-positioning.
- **Engine design gold:** ACID_ROLLBACK_DESIGN + ACID_TRANSACTIONS (recuperados de git history) + MVCC_SNAPSHOT_ISOLATION + res01/02/03 — son el spec de referencia cuando se ejecute RES-01/fases 4b-d.
- **Process/process-engineering:** TIR-* y TSYS-06 quedan como precedentes de decisiones ponytail bien razonadas.

---

## 4. Método y límites

- Lectura completa de los 53 archivos (los 2 gigantes vía sub-agentes con reportes incluidos).
- Verificación dirigida de ~40 claims con grep/Test-Path/read sobre código, workflows, backlog y registries el día de hoy (2026-08-26).
- Claims externos (competidores) fechados abril-julio 2026 mantienen sus disclaimers de re-verificación original.
