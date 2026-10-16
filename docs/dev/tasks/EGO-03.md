---
title: "EGO-03 (épico): resto tren 0.9.0 — búsqueda, storage, DX, grafo (EGO-02/03/04 hechos, resto por slices)"
kind: task
description: "Resto del tren 0.9.0: EGO-06/10/11/12/13/14/15/16/17 (+EGO-02 si incompleto). EGO-02/03/04 NO reimplementar (commits c82f4b2d, 36ac14ff, b04d33bf verificados en árbol/historial). Slice 1: EGO-15 max_degree_per_node en graphrag."
---

# EGO-03 (épico): resto tren 0.9.0

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (Task 9, Wave 2 tras EGO-01 ✅ `db74daed`)
- **Fuente:** Backlog §EGO-02/03/04/06/10/11/12/13/14/15/16/17 ← Ego `VDB-REQ-01/02/03/04/05/07`, `VDB-BUG-03/04/05/06`, `VDB-INC-02/03/04` (2026-10-07/10)
- **Esfuerzo:** 🔴 max 3d · **Prioridad:** 🟠
- **Tipo:** Rust core (graphrag/index) + bindings solo donde el slice lo exija
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE + commit orquestador; EGO-10/13 como diseños diferidos con Gate D)
- **Next:** DIST-19
- **SDP:** codebase-memory, source-driven-development, progreso (lectura), ponytail
- **Reglas área (leídas completas):** `.opencode/rules/indexes.md` + `query-dsl.md` + `durability.md` + `js-ecosystem.md` + `python-bindings.md`
- **PROHIBIDOS (verificados):** `vantadb-python/**` unstaged (WIP ajeno — intacto), stash, `.opencode/**`, CHANGELOG, `docs/index.md`, `llms.txt`, `src/sdk/api/memory.rs` + `src/sdk/types.rs` (líneas EGO-01 cerradas: leer sí, editar solo con justificación), `docs/dev/plans/` (orquestador)

## DISCOVERY — qué ya está hecho (NO reimplementar)

| ID | Commit/evidencia | Estado verificado 2026-10-16 |
|----|------------------|-------------------------------|
| EGO-02 query_vector opcional | `c82f4b2d` | ✅ en árbol (`vantadb-ts/src/guards.ts:226-228` comentario EGO-02, `types.ts:203` opcional). NO tocar |
| EGO-03 comodines nativos | `36ac14ff` | ✅ en árbol (`src/sdk/search/multi.rs:15-33` `wildcard_prefix`, tests `*_ego03`). `validate_namespace` intacto en put/get (solo-search). NO tocar |
| EGO-04 tokenizador Unicode | `b04d33bf` | ✅ en historial. NO tocar |
| EGO-06 finalizer Fjall | — | 🟡 EN CURSO (Slice 6): `#[napi(custom_finalize)]` + registro path canónico implementados en `vantadb-node/src/lib.rs`; verificado en fuente napi (derive-backend + `raw_finalize_unchecked`); corrección de premisa: flock-OS muere con el proceso → el `Locked` es handle vivo in-process, no crash. Falta: napi build + vitest lifecycle |
| EGO-09 open_readonly | `db74daed` (EGO-01 Slice 2) | ✅ hecho en Wave 1. NO tocar |
| EGO-10 VantaValue Map/Array | — | ⬜ pendiente (`src/sdk/types.rs:123-146` sin `Map`/`Array`; EGO-05 solo escalares). Requiere editar `types.rs` con justificación |
| EGO-11 text_query "" | `guards.ts:234-235` + `fusion.rs:56` | ⬜ pendiente por DECISIÓN (guard exige `trim().length > 0`; core `trimmed_text_query` → None). Backlog: match-all o `scan()` + documentar. Gate D: cambia semántica search — va a `question` al orquestador, NO implementar unilateral |
| EGO-12 reopen | — | ⬜ pendiente (sin `reopen` en `vantadb-node/src/lib.rs`) |
| EGO-13 zero-copy | — | ⬜ pendiente (vectores vía serde genérico; exige bench Regla 9 + divergencia NAPI/PyO3) |
| EGO-14 vacuum HNSW | árbol | 🟡 PARCIAL: `StorageEngine::vacuum()` (`maintenance.rs:655`) + `Embedded::vacuum()` (`api/graph.rs:72`) + `merge_segments` + `pipeline()` EXISTEN. Falta: verificar exposición en bindings + `vacuum_vector_index(namespace)` scoped (Backlog pide por-namespace) |
| EGO-15 poda grado | — | ⬜ pendiente (`expand_nodes` solo `max_nodes`; sin `max_degree_per_node`/`min_edge_weight`; `Edge.weight` existe `node/edge.rs:15`). → **Slice 1** |
| EGO-16 borrado físico | árbol + auditoría Slice 8 | 🟡 HALLAZGO (FIND-343 🔴 Engine): payload legible en WAL rotado + journal Fjall tras mantenimiento completo; certificado solo garantiza nivel API. Test pin en `#[ignore]`; re-exports listos. NO fix worker (WAL = commit point) |
| EGO-17 native.js dep real | `vantadb-ts/package.json:73` | ⬜ pendiente (`vantadb-node: file:../vantadb-node` en **devDependencies**, `dependencies: ['vantadb-wasm']`; `dist/native.js` existe como artefacto). Cambio es 1 línea pero toca publish (`file:` se empaqueta) → Slice 2 candidato con cautela release-ci |
| EGO-05 escalares | `db74daed` | ✅ Wave 1. NO tocar |

- **Corrección de paths (stale en Backlog, verificado en árbol):** `src/sdk/api/graphrag.rs` NO existe → real `src/graphrag/{pipeline,expand,seed,retrieve,context}.rs`; `src/sdk/types.rs` es archivo (no dir) + `src/sdk/types/`; `vantadb/dist/native.js` NO existe → real `vantadb-ts/dist/native.js` (artefacto) + fuente `vantadb-ts/src/native.ts`; `src/sdk/api/memory.rs` existe pero PROHIBIDO editar en este épico.
- **Contrato (plan Task 9):** `cargo nextest run -p vantadb --test multilingual_tokenizer_integration --test mmap_hnsw --test fjall_cold_copy_restore --test graphrag_test --test message_thread_test` verdes + wildcards solo-search + bench Regla 9 en lo perf.

## Contrato (este épico, por slice)

- Slice 1 (EGO-15): `tests/graphrag_test.rs` nuevo test hub-cap verde + suite `graphrag_test` completa verde + `validate_namespace` intacto (sin tocar) + `public-api.txt` actualizado (aditivo) o nota de regen para CI nightly.
- Resto slices: sus contratos al abrir cada slice (Gate C).

## Impacto mapeado (Regla 0) — Slice 1 EGO-15

- **Archivos leídos (completos):** `src/graphrag/expand.rs` (77L), `src/graphrag/pipeline.rs` (124L), `src/node/edge.rs` (`Edge.weight: f32`), `src/sdk/builder.rs:223-231` (`graphrag_search` usa `GraphRagPipeline::new()`), `src/sdk/api/graph.rs:72` (`vacuum`), `tests/graphrag_test.rs:1-200` (estilo), `tests/api/public_api.rs` (snapshot `#[ignore]`, nightly-only), `tests/api/public-api.txt:2899-2918` (líneas graphrag)
- **Referencias hacia dentro:** `expand_nodes` ← `pipeline.rs:93` (único caller prod); `GraphRagPipeline` struct-literal ← `benchmarks/graphrag_bench.rs` (4× con `..new()`, source-compat), `tests/graphrag_test.rs:157` (con `..new()`, source-compat); `graphrag_search` ← bindings (usan `new()`, default intacto)
- **Referencias entrantes:** `expand_nodes` ← `public-api.txt:2914` (snapshot; test ignorado en Fast Gate, regen en job nightly)
- **Veredicto:** BAJO — cambio aditivo (1 campo pub con default uncapped + 1 fn nueva + wrapper intacto); comportamiento default byte-idéntico; sin locks/Tokio/multi-índice nuevos (BFS read-only vía `engine.get`; Regla 8 no dispara evidencia chaos — se declara); sin bench Regla 9 (no es optimización hot path, es cap de fan-out; la prueba cuenta nodos, no P99)
- **Gate D:** se agrega símbolo público (`max_degree_per_node`, `expand_nodes_capped`) pero con spec explícita del Backlog (VDB-REQ-07 + acción literal) → NO dispara `question`; EGO-11 sí queda en `question` (semántica ambigua match-all vs scan) y NO se implementa aquí
- **Gate P:** no aplica (no hay plan nuevo; plan Task 9 existe y manda este slice)

## Slices

- [x] **Slice 0 — DISCOVERY + task file (este archivo):** git log (c82f4b2d/36ac14ff/b04d33bf presentes), árbol verificado por ID, blast radius por slice, paths corregidos
- [x] **Slice 1 — EGO-15 `max_degree_per_node`:** campo pub en `GraphRagPipeline` (default `usize::MAX` = uncapped) + `expand_nodes_capped` + wrapper `expand_nodes` intacto + test hub RED→GREEN + `public-api.txt` aditivo. `graphrag_test` 7/7 + 5 suites contrato verdes
- [x] **Slice 2 — EGO-17:** `vantadb-node` a `dependencies` en `vantadb-ts/package.json` + test packaging RED→GREEN (2/2) + `check-npm-versions.mjs` OK
- [x] **Slice 3 — min_edge_weight (follow-up Gate C):** `Option<f32>` en pipeline + filtro pre-cap en expand + test RED→GREEN (`graphrag_test` 8/8; clippy `is_none_or`)
- [x] **Slice 4 — EGO-11 `scan()` (decisión owner: método explícito, sin match-all):** `ScanInput` + `Client.scan()`/`NativeVantaDB.scan()` + `MemoryClient.scan` + docs "searchMulti no barre" + guard apunta a `scan()` + core intacto (R-8). Tests 6/6 + 41/41 guards/tipos + tsc/eslint
- [x] **Slice 5 — EGO-14 `NativeVantaDB.vacuum()`:** wrapper + `VacuumReport` + test RED→GREEN (contrato honesto: removed suele ser 0 porque `delete()` ya desengancha HNSW; vacuum cosecha huérfanos). Core `vacuum()`/`merge_segments`/`pipeline()` ya existían
- [x] **Slice 6 — EGO-06 finalizer + registro (VERDE 5/5):** `#[napi(custom_finalize)]` + `ObjectFinalize` (drain+close best-effort, `catch_unwind`, SAFETY) + registro exclusión total fail-fast `VANTADB_BUSY` + `close()` libera slot + `tests/lifecycle.test.ts` RED→GREEN. Hallazgo: Fjall ignora `read_only` → matriz total (deuda Engine). Suite node 52/52 intacta. `index.d.ts` +5 (doc-comment, sin cambio API)
- [x] **Slice 7 — EGO-12 reopen (VERDE):** `Mutex<VantaInner>` + `snapshot()` (34 replaces mecánicos) + `reopen(path?)` (drain→close→re-reserva→open→swap; en fallo queda limpio-cerrado) + `NativeVantaDB.reopen()` (resetea `_closed` solo en éxito) + tests RED→GREEN. lifecycle 7/7 + node 59/59 + reopen TS 2/2 + tsc/eslint. `index.d.ts` +15 (doc + firma reopen, sin breaking)
- [ ] **Slices 8+: EGO-10 Value Map/Array (🔴 breaking, requiere spec wire + paridad bindings), EGO-13 zero-copy+bench Regla 9, EGO-16 auditoría borrado físico, WASM/Python vacuum gaps** — con briefs abajo, uno por vez
- [x] **Slice 8 — EGO-16 auditoría borrado físico (HALLAZGO, no fix):** test `physical_erasure_*` en `tests/certified_delete.rs` → FALLA por diseño actual (payload legible en `vanta.shard0.wal` rotado + `0.jnl` Fjall tras delete+vacuum+merge+compact+flush+close) → pinneado en `#[ignore]` + fila **FIND-343** (🔴, Engine) + re-exports `PipelineMode/MergeReport/PipelineReport/VacuumReport` (hacen llamable `Embedded::pipeline()`). Suite: 6 passed + 1 ignored. El fix exige rediseño Engine (WAL = commit point + VER-01) — boundary respetado
- [x] **Slice 9 — EGO-10 DISEÑO (Gate D: STOP sin implementar):** discovery + tabla Spec §Spec-EGO-10 + veredicto breaking documentado. Implementar exige decisión owner (wire format + semántica por subsistema exceden boundary worker)
- [x] **Slice 10 — EGO-13 VEREDICTO (Gate C: DIFERIR sin implementar):** discovery + veredicto documentado. El path actual (`number[]` → serde_json `Value::Array` → `as_f64` uno por uno en `get_f32_vec`) exige rediseñar firmas de parseo para `Float32Array` (todos los `parse_*` reciben `Value` genérico) + bench de binding inexistente (`canonical_p99` mide core/HNSW, insensible al overhead FFI; el bench correcto sería `bench-abi.mjs`-like before/after). Con PyO3 prohibido solo saldría mitad del Backlog. Regla 9 prohibiría el merge sin bench. Veredicto abajo.
- **Nota diseño Slice 7 (EGO-12 reopen):** `Config: Clone` ✅ (`config.rs:656`), `Embedded.config` es `pub(crate)` (inaccesible al binding) → guardar `Config` clonado en `VantaDB`; reopen exige `Mutex<Inner{engine,gate,slot}>` (op_gate es one-way) + re-reserva en registro EGO-06. Slice mediano, tras EGO-06 verde.

## Spec (Gate mecánico — símbolos públicos nuevos del Slice 1)

| Decisión | Opción elegida | Evidencia/justificación |
|----------|----------------|-------------------------|
| Dónde vive el cap | Campo `pub max_degree_per_node: usize` en `GraphRagPipeline` (no `GraphRagOptions` nuevo) | Backlog pide "`max_degree_per_node` + `max_total_nodes` + `min_edge_weight` en `GraphRagOptions`" pero no existe tal struct; el pipeline ya es el options-bag (campos pub + `new()`). Crear struct nuevo = abstracción prematura (ponytail). `max_total_nodes` ya cubierto por `max_expansion_nodes` existente |
| Default | `usize::MAX` (uncapped = comportamiento actual) | Regla 5 rollback-friendly + safe defaults; `graphrag_search` y bindings sin cambio observable |
| Semántica del cap | Por nodo: seguir como máximo N aristas en orden de almacenamiento (determinista por grafo) | Simple, testeable, sin heurística de centralidad (Ego ya acota en su `GraphRagEngine`; esto es el freno en el motor) |
| `expand_nodes` existente | Firma INTACTA (wrapper uncapped) + nueva `expand_nodes_capped` | Blast radius mínimo; `public-api.txt` solo gana líneas (aditivo); benches/tests con `..new()` compilan igual |
| `min_edge_weight` | FUERA de este slice | Gate C scope: un slice = una cosa; `Edge.weight` existe así que el follow-up es trivial |
| Cap 0 | Válido: solo seeds, sin expansión (con hops>0) | Caso borde testeado implícitamente por diseño; no se testea aparte (ponytail) |

## Spec-EGO-10 (Gate mecánico — Slice 9 DISEÑO; Gate D dispara → STOP sin implementar)

> Veredicto: implementar `Map`/`Array` recursivos exige breaking on-disk + semántica por subsistema + paridad 3 bindings con PyO3 hoy bloqueado. Excede boundary worker (diseño Engine/Arch + decisión wire del owner). Este slice entrega el diseño; la implementación espera Gate D del orquestador.

| Decisión | Análisis | Veredicto |
|----------|----------|-----------|
| Forma del tipo | `Map(HashMap<String,VantaValue>)` + `Array(Vec<VantaValue>)` (Backlog) vs `Json(serde_json::Value)` | `Json` inviable: postcard no serializa tipos dinámicos sin schema. `Map`/`Array` nativos son la única forma real — ambas exigen lo mismo abajo |
| Posición en el enum | Variantes NUEVAS al FINAL de `Value` y `FieldValue` | Preserva índices postcard existentes (backward-read OK); binarios viejos no leen datos nuevos (forward-incompat — breaking 0.x, precedente EGO-04 en este tren) |
| `Eq+Hash` de `FieldValue` (`node/field.rs`) | `HashMap` no implementa `Hash`; `Eq` con floats ya tensionado (NaN hoy: `attestation.rs:1335`) | Exige `Hash` manual con claves ordenadas + documentar `Eq` débil, o cambiar bounds (breaking de API pública). Decisión Arch |
| Matches exhaustivos | `conversions.rs:82-94,100-112` + filtros (`physical_plan/filter.rs`, `join.rs`) + `scalar_index` + text index + shred + export + linking + `field.rs:57-121` — el compilador los encuentra, pero cada sitio exige SEMÁNTICA (¿`Eq` contra un Map? ¿cómo indexa el scalar un Array anidado? ¿tokeniza el text index un Map?) | Spec por subsistema (Arch); no es mecánico |
| Recursión adversarial | `put` acepta JSON de usuario → `visit_seq`/`visit_map` recursivos sin límite = stack overflow / DoS | Límite de profundidad explícito en el Visitor (diseño, no existe hoy) |
| Deserialize tolerante | Extender `ValueVisitor` (EGO-05): maps planos → `Map`, arrays heterogéneos/anidados → `Array`, homogéneos → `List*` actual | La única parte worker-viable, pero sin el resto deja tipos que el motor no entiende |
| Bindings | WASM (genérico, probable OK) · Node: sin `VantaValue` propio hoy (`vantadb-node/src/` solo `lib.rs` — path del Backlog stale) · Python: mapping `dict`/`list` en `convert.rs` — **PROHIBIDO** (`vantadb-python/**` WIP ajeno) | Paridad imposible hoy → bloquearía el slice aunque el core estuviera listo |
| Backward-compat lectura | Datos viejos (sin Map/Array) se leen igual (variantes al final) | ✅ único punto no-breaking |

**STOP Gate D:** documentado, sin código. Desbloqueo: (1) decisión owner del wire format + bump versión formato, (2) spec semántica por subsistema (Arch), (3) Python desbloqueado, (4) review adversarial (breaking). Estimación: épico propio 3-5d, no slice.

## Veredicto-EGO-13 (Slice 10: DIFERIR sin implementar — Gate C + Regla 9)

> Discovery 2026-10-16 (`vantadb-node/src/lib.rs:1206-1253`, `benches/canonical_p99.rs`, `vantadb-node/bench/bench-abi.mjs` existente). No hay código cambiado en este slice.

| Punto | Evidencia |
|-------|-----------|
| Path actual | `number[]` JS → napi serde_json `Value::Array` (1 alloc por elemento) → `get_f32_vec` recorre con `as_f64()` + valida finitud + `collect::<Vec<f32>>`. Doble costo: deserialización genérica + recorrido validante |
| Por qué no es thin | Todos los `parse_*` (`parse_memory_input`, `parse_search_request`, …) reciben `Value` genérico. Aceptar `Float32Array`/`Buffer` exige cambiar firmas de parámetros a `Either` + re-parseo por campo + `.d.ts` + tests. Toca put/put_batch/search/graphrag de una vez (o deja firmas inconsistentes) |
| Bench | `canonical_p99` existe pero mide core/HNSW directo — INSENSIBLE al overhead FFI. El bench honesto es de binding (pasaje JS→Rust en `put_batch` masivo, estilo `bench-abi.mjs`) before/after con entorno+comando+fecha. Ese bench no existe aún; crearlo es su propio slice |
| PyO3 | Bloqueado (`vantadb-python/**` WIP ajeno) — solo saldría la mitad napi del Backlog |
| Regla 9 | Prohibiría el merge sin bench before/after. Implementar el cambio sin el bench sería violarla |

**Veredicto:** diferir épico EGO-13 a slice propio con: (1) bench de binding before (baseline), (2) rediseño de firmas napi (`Either<Value, Float32Array>`), (3) bench after + decisión con datos, (4) parte PyO3 cuando se desbloquee el WIP. Estimación: 2-3d con bench incluido.

## Invariantes (handoff — MUST)

- `validate_namespace` intacto (wildcards solo-search de EGO-03) — este épico NO lo toca
- `Value` serialización intacta (EGO-05 Wave 1) — `types.rs` solo se edita en Slice EGO-10 con justificación propia
- `vantadb-python/**` unstaged intacto; sin stash; sin push; commit solo orquestador
- Comandos con Regla dura `-p`; nextest timeout ≥600s; cargo vía `$env:USERPROFILE\.cargo\bin`
- Si nextest falla por `vanta-cli.exe` bloqueado (causa ajena conocida): binarios de test directos + declararlo

## Steps

- [x] DISCOVERY + task file (este archivo)
- [x] Slices 1-6 RED→GREEN + verifies (graphrag 8/8, 5 suites contrato, TS 41/41+6/6+1/1, node 52/52+lifecycle 5/5, tsc/eslint, fmt/clippy/diff--check, public-api.txt aditivo, check-npm-versions OK, 2× napi build + clippy/fmt)
- [x] Slice 7 EGO-12 RED→GREEN (lifecycle 7/7 + node 59/59 + reopen TS 2/2 + tsc/eslint + index.d.ts +15)
- [x] Slice 8 EGO-16 auditoría (hallazgo FIND-343 + test en #[ignore] + re-exports PipelineMode & co; suite 6 passed + 1 ignored; clippy/fmt/diff-check)
- [x] Slice 9 EGO-10 diseño + STOP Gate D (tabla Spec-EGO-10; breaking documentado, sin código)
- [x] Slice 10 EGO-13 veredicto DIFERIR (discovery + sección Veredicto-EGO-13; Regla 9 + PyO3 bloqueado)
- [ ] Save Point + RESULTADO 🟡 (PROXIMO_STEP="review + commit", orquestador)

## Blast radius por slice (tabla)

| Slice | Archivos tocados | Compartido search/write | HNSW recall/disco | FFI | Riesgo |
|-------|------------------|-------------------------|-------------------|-----|--------|
| 1 EGO-15 | `src/graphrag/{pipeline,expand}.rs`, `tests/graphrag_test.rs`, `tests/api/public-api.txt` | No (solo grafo) | No (BFS sobre `engine.get`, sin índice) | No | 🟢 bajo |
| 2 EGO-17 | `vantadb-ts/package.json` (+`dist/` artefacto NO commitear) | No | No | No (empaquetado) | 🟡 publish |
| 3 min_edge_weight | mismos que Slice 1 | No | No | No | 🟢 bajo |
| 4 EGO-11 scan | `vantadb-ts/src/{types,vantadb,native,guards}.ts` + `scan.test.ts`; core intacto | No (TS-only, delega a `list`) | No | No (TS, sin Rust) | 🟢 bajo |
| 5 EGO-14 vacuum | `vantadb-ts/src/native.ts` + `vacuum.test.ts`; core intacto | No | Lectura (vacuum existente) | No (wrapper napi existente) | 🟢 bajo |
| 6 EGO-06 | `vantadb-node/src/lib.rs` + `tests/lifecycle.test.ts` (+`index.d.ts` regen) | No | No | Sí NAPI finalizer | 🟡 GC-thread + matriz |
| 7 EGO-12 | `vantadb-node/src/lib.rs` (Mutex<Inner>) + `native.ts` + tests | No | No | Sí NAPI | 🟡 refactor mecánico |
| 8 EGO-16 | `tests/certified_delete.rs` (+1 test #[ignore]) + `src/{lib,storage/mod}.rs` (re-exports) + fila FIND-343 | No (auditoría lectura) | Lectura disco post-mantenimiento | No | 🟢 (sin fix; boundary Engine) |
| EGO-10 | `src/sdk/types.rs` (+bindings) | Wire serialización | No | Sí (todos) | 🔴 breaking potencial |
| EGO-13 | `vantadb-node/src/`, `vantadb-py/` | No | Throughput ingesta | Sí ambos | 🟡 bench Regla 9 |

## Save Point (2026-10-16 — Slices 1-10, sin commit)

- **Archivos para el commit (SOLO estos):**
  - Slice 1+3: `src/graphrag/expand.rs`, `src/graphrag/pipeline.rs`, `tests/graphrag_test.rs`, `tests/api/public-api.txt`
  - Slice 2: `vantadb-ts/package.json`, `vantadb-ts/src/__tests__/native-packaging.test.ts`
  - Slice 4: `vantadb-ts/src/{types,vantadb,native,guards}.ts`, `vantadb-ts/src/__tests__/scan.test.ts`
  - Slice 5: `vantadb-ts/src/native.ts` (vacuum), `vantadb-ts/src/__tests__/vacuum.test.ts`
  - Slice 6+7: `vantadb-node/src/lib.rs`, `vantadb-node/index.d.ts` (regen), `vantadb-node/tests/lifecycle.test.ts`, `vantadb-ts/src/native.ts` (reopen), `vantadb-ts/src/__tests__/reopen.test.ts`
  - Slice 8: `tests/certified_delete.rs` (+1 test #[ignore]), `src/lib.rs` + `src/storage/mod.rs` (re-exports), `docs/dev/Backlog.md` (fila FIND-343)
  - Este archivo: `docs/dev/tasks/EGO-03.md`
  - `Cargo.toml` raíz intacto (workaround `[[bin]]` revertido + bak eliminado — verificado).
- **NO incluir:** `vantadb-python/**` (WIP ajeno), `docs/dev/plans/*` (orquestador), `desktop/**` + `feature-freez/**` (freeze orquestador en paralelo), resto del `git status`. `index.js/cjs` node sin cambios (solo `index.d.ts`); `.node` no trackeado.
- **Evidencia:** `graphrag_test` 8/8 · 5 suites contrato verdes · TS scan 6/6 + guards/tipos 41/41 + vacuum 1/1 + reopen 2/2 + tsc/eslint · node suite 59/59 (lifecycle 7/7) · `certified_delete` 6 passed + 1 ignored (auditoría EGO-16) · `cargo build -p vantadb --lib` ✅, clippy ✅, fmt ✅, `diff --check` ✅, `check-npm-versions.mjs` ✅, 3× napi build release ✅.
- **Desviaciones conocidas:** (1) `cargo nextest run` literal NO ejecutado — `target/debug/vanta-cli.exe` bloqueado por servidores MCP ajenos; workaround `rustc --test` directo documentado. Re-correr nextest al commitear. (2) `public-api.txt` actualizado a mano (aditivo Slices 1/3); los re-exports del Slice 8 (PipelineMode & co) NO se añadieron al snapshot — regen obligatorio con `VANTADB_PUBLIC_API_UPDATE=1` en CI nightly. (3) Dos builds napi en background murieron por infra (PATH sin cargo / pausa); los builds en foreground con PATH correcto produjeron los `.node` verificados.
- **Briefs restantes (para el orquestador — con diseño/veredicto en este archivo, §§Spec-EGO-10 y Veredicto-EGO-13):**
  - EGO-10 Value Map/Array (🔴, STOP Gate D): veredicto breaking documentado; desbloqueo = decisión owner wire + spec Arch por subsistema + Python desbloqueado + review adversarial. Épico 3-5d.
  - EGO-13 zero-copy (🟡, DIFERIDO Gate C + Regla 9): veredicto documentado; desbloqueo = bench binding before + rediseño firmas napi + bench after + parte PyO3. 2-3d.
  - FIND-343 borrado físico (🔴 + Regla 8): requiere `vanta-arch` (rediseño purga WAL/journal) + `vanta-chaos`/`vanta-review`; test pin en `#[ignore]` corre en certificación.
  - WASM/Python vacuum gaps: exponer `vacuum()` en `vantadb-wasm` (requiere wasm-pack build) y Python (tras WIP ajeno).
- **Si la sesión se interrumpe:** nada a mitad — artefactos commiteables tal cual; falta review P2-01 + commit del orquestador.

## Review

- **Revisor:** vanta-review, sesión distinta (veredicto 2026-10-16, re-ejecución completa del diff acumulado).
- **Veredicto:** ✅ APPROVE EGO-03 — slices aditivos verificados, auditoría honesta con FIND-343, EGO-10/13 diferidos con veredicto escrito, Regla 8 limpia. Fixes del review aplicados por orquestador: `FIND-34x`→`FIND-343` + regen `public-api.txt` (test verde, +188/-9 con churn `serde_core` de toolchain).
- **Estado del épico:** ✅ COMPLETED (implementable cerrado; decisiones owner/Engine en Backlog).

## Recitation

| Campo | Valor |
|-------|-------|
| `activeGoal` | EGO-03 épico resto 0.9.0; Slices 1-10 (7 verdes + 1 auditoría + 2 diseños/veredictos) — épico técnicamente completo |
| `lastAction` | Slices 9-10: Spec-EGO-10 + STOP Gate D (breaking) y Veredicto-EGO-13 DIFERIR (Regla 9 + PyO3 bloqueado), ambos documentados sin código |
| `result` | PARTIAL (todo lo implementable hecho con evidencia; restan decisiones owner/Engine + review P2-01 + commit orquestador) |
| `nextAction` | Orquestador: review P2-01 + re-correr nextest + regen public-api nightly (obligatorio por re-exports Slice 8) + commit selectivo + decisiones EGO-10/FIND-343 + planificar EGO-13 con bench |
| `contract.verificacion` | graphrag 8/8 · 5 suites contrato · TS 6/6+41/41+1/1+2/2 + tsc/eslint · node 59/59 · certified_delete 6+1 · build lib/clippy/fmt/diff-check/check-npm/napi×3 limpios |
| `contract.deuda` | review P2-01 + commit (orquestador); decisiones EGO-10 (owner) + FIND-343 (Engine) + EGO-13 (bench); nextest literal + public-api regen en CI |
