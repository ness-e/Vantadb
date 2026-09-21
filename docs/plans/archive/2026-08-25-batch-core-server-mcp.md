# Plan de Ejecución: Batch Core/Server/MCP/Python/TS correctness (2026-08-25)

> **Campaign ID:** 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
> **Inicio:** 2026-08-25
> **Estado:** ✅ COMPLETADO
> **Fuente:** docs/Backlog.md (selección del lead + confirmación del usuario 2026-08-25)
> **Modo:** FAIL_MODE=parallel, MAX_CONCURRENT=3

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 14 |
| 🟡 DEFER | 6 |
| ❌ SKIP | 1 |
| 🔴 BLOQUEADO | 1 |

Status: ⬆️ uphill = 0 resueltos (MCP-33/34 investigados en DISCOVERY: MCP-33 wrappers ✅, MCP-34 DEFER por snapshot_restore core-nuevo) · ⬇️ downhill = 14

> **Restricción:** NO tocar `desktop/` (sesión P34/P37 concurrente activa, H1/H2 del backlog). Todas las tareas son core/server/mcp/python/ts/docs.

## Tasks

### Task 1: REVIEW-13 — supersede() TOCTOU concurrente

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Archivos clave:** `src/sdk/api.rs:840-894`
- **Verificación real:** ✅ CÓDIGO-REAL — `codegraph_explore` confirma: `supersede` lee `old.superseded_by` en :857, valida, y escribe vía `engine.insert` en :886 sin atomicidad → dos threads pueden double-mark/overwrite divergente (guard ya marcado con `ponytail:`). Gap real en `src/sdk/api.rs`.
- **Gate Justificación:** race de datos real en API pública, fix acotado (serializar vía write lock del engine o re-check en sección crítica). Esfuerzo 🟢, impacto 🟡.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb supersede` pasa; test concurrente (2 threads supersede mismo key → 1 gana, estado consistente)
- **Task file:** `skills/campaign-executor/tasks/REVIEW-13.md`
- **Estado:** ✅ COMMITTED `fix` — supersede_lock (Arc<Mutex>) RMW serializado; test concurrente 10/10, clippy ok

  **Pre-mortem:**
  - Fallo 1: el write lock del engine no cubre el read de `get()` → race persiste
  - Fallo 2: el test concurrente es flaky si no hay barrera de sincronización
  - Fallo 3: romper la idempotency guard (permitir double-supersede)

  **Stop conditions:** 2 fallas verify mismo-error → Gate V → pregunta al usuario.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟡 | cambiar semántica de supersede | tests de idempotencia existentes como guard | 2 iteraciones sin green |

  **Cynefin:** 🟨 complicado — race en storage requiere análisis del lock del engine.
  **Top 3 riesgos:** (1) lock incorrecto → race persiste; (2) test flaky; (3) romper idempotencia.

### Task 2: MOD-04 — purge_expired O(N) full-scan → índice TTL

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `src/sdk/api.rs:898-963`, `src/scalar_index.rs`, `src/node/*`
- **Verificación real:** ✅ CÓDIGO-REAL — `codegraph_explore`: `purge_expired` hace `engine.scan_nodes()` + `get_field(FIELD_EXPIRES_AT_MS)` por nodo (O(N) full scan). `ScalarIndex` (`src/scalar_index.rs`) existe infrautilizado (1 caller en engine.rs). Gap real.
- **Gate Justificación:** optimización de mantenimiento TTL real; hay primitiva de índice disponible. Requiere Regla 9 (bench before/after) — es perf.
- **Gate Result:** ✅ DO
- **Contrato:** bench before/after `purge_expired` con N records expirados documentado; tests existentes TTL pasan; `cargo nextest run -p vantadb` verde
- **Task file:** `skills/campaign-executor/tasks/MOD-04.md`
- **Estado:** ✅ COMMITTED `perf(storage)` — purge selectivo vía scalar index range, bench −24%/−9%, 2060/2060

  **Pre-mortem:**
  - Fallo 1: mantener consistencia del índice TTL en write/delete/update paths
  - Fallo 2: benchmark sin dataset representativo → regresión no medida
  - Fallo 3: el índice TTL no se reconstruye en reopen/rebuild

  **Stop conditions:** rabbit hole en consistencia de índice → abort → DEFER.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟡 | índice TTL desincronizado tras crash | rebuild en recovery como los demás índices | test de persistencia falla |

  **Cynefin:** 🟨 complicado — mantenimiento de índice derivado.
  **Top 3 riesgos:** (1) consistencia del índice; (2) bench no representativo; (3) rebuild.

### Task 3: MOD-10 — MCP tools: versions/supersede/vacuum/remove_edge

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `src/sdk/api.rs:469,840,81,1218`
- **Verificación real:** ✅ CÓDIGO-REAL — `codegraph_explore` confirma los 4 métodos existen en SDK (`versions` :469, `supersede` :840, `vacuum` :81, `remove_edge` :1218) pero sin tool MCP. Gap real.
- **Gate Justificación:** wrappers finos sobre API ya pública; completa cobertura MCP (MOD-10). Sin semver implications.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp --test mcp_tests` pasa; 4 tools nuevas round-trip; docs skill ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MOD-10.md`
- **Estado:** ✅ COMPLETED

  **Pre-mortem:**
  - Fallo 1: remove_edge requiere ids u128 como strings (wire format)
  - Fallo 2: docs skill desincronizadas (hash SAME es obligatorio)
  - Fallo 3: vacuum expone report serde shape distinto a lo esperado

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟢 | wire format u128 en remove_edge | reutilizar patrón ids-as-strings de MCP-21 | test round-trip falla |

  **Cynefin:** 🟦 obvio — wrappers sobre API existente.
  **Top 3 riesgos:** (1) u128 wire format; (2) hash SAME docs; (3) shape vacuum.

### Task 4: MOD-13 — server sin TimeoutLayer

- **Appetite:** max 1h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `src/cli_server.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — backlog: 0 matches TimeoutLayer en cli_server.rs; handler atascado retiene conexión indefinidamente. Gap real.
- **Gate Justificación:** robustez del HTTP server, fix acotado (agregar timeout layer axum). Effort 🟡, impacto 🟡.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb --features server`; test e2e request lento → timeout 408/500; `cargo nextest run -p vantadb-server` (o el test del server) verde
- **Task file:** `skills/campaign-executor/tasks/MOD-13.md`
- **Estado:** ✅ COMMITTED `feat(server)` — TimeoutLayer 30s/600s, cli_server 42/42

  **Pre-mortem:**
  - Fallo 1: timeout demasiado agresivo rompe requests legítimos largos (import/export)
  - Fallo 2: TimeoutLayer no compila con la versión axum actual
  - Fallo 3: no distinguir timeout global vs por-ruta

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟡 | timeout rompe bulk ops | excluir rutas de import/export o configurar timeout generoso | test bulk falla |

  **Cynefin:** 🟦 obvio.
  **Top 3 riesgos:** (1) timeout en bulk ops; (2) compat axum; (3) granularidad.

### Task 5: MOD-14 — test rate-limit e2e laxo

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-server/tests/e2e.rs:296-300`
- **Verificación real:** 🟡 VERIFICAR — backlog cita e2e.rs:296-300 acepta `200||429`. Confirmar en DISCOVERY con read.
- **Gate Justificación:** endurecer test para exigir ≥1 429 con burst conocido — calidad de test, sin riesgo.
- **Gate Result:** ✅ DO
- **Contrato:** test e2e rate-limit exige ≥1 429 en burst; `cargo nextest run -p vantadb-server` verde
- **Task file:** `skills/campaign-executor/tasks/MOD-14.md`
- **Estado:** ✅ COMMITTED `test(server)` — burst 10 requests exige ≥1 429; e2e 12/12

  **Pre-mortem:**
  - Fallo 1: el burst en CI no alcanza el límite por timing → test flaky
  - Fallo 2: governor no activado en test env

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟢 | test flaky en CI | burst con margen sobre límite conocido | test falla intermitente |

  **Cynefin:** 🟦 obvio.
  **Top 3 riesgos:** (1) flaky; (2) governor apagado; (3) timing.

### Task 6: MOD-18 — stubs .pyi duplicados

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-python/vantadb_py/__init__.pyi`, `vantadb-python/vantadb_py/vantadb_py.pyi`
- **Verificación real:** 🟡 VERIFICAR — backlog: `.pyi` duplicados y desactualizados (put_batch type, métodos faltantes). Confirmar en DISCOVERY.
- **Gate Justificación:** consolidar stubs + test anti-drift — DX Python, sin riesgo runtime.
- **Gate Result:** ✅ DO
- **Contrato:** 1 stub consolidado; test anti-drift firma↔stub; `python -m pytest vantadb-python/tests/` verde; mypy si aplica
- **Task file:** `skills/campaign-executor/tasks/MOD-18.md`
- **Estado:** ✅ COMMITTED `fix(python)` — stubs consolidados + anti-drift 7/7, pytest 125

  **Pre-mortem:**
  - Fallo 1: consolidar stubs rompe imports de usuarios (nombres de archivo)
  - Fallo 2: test anti-drift no detecta drift real

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟢×🟡 | romper path de import | mantener re-export en stub legacy | test import falla |

  **Cynefin:** 🟦 obvio.
  **Top 3 riesgos:** (1) path import; (2) anti-drift débil; (3) métodos faltantes.

### Task 7: MOD-20 — excepciones Python genéricas

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-python/src/convert.rs:659-684`, `vantadb-python/src/lib.rs:1578`
- **Verificación real:** 🟡 VERIFICAR — backlog: catch-all RuntimeError + query() retorna string formateado. Confirmar en DISCOVERY.
- **Gate Justificación:** jerarquía VantaError propia + retornar estructura en query — mejora DX Python real. API contract: aditivo si no rompe.
- **Gate Result:** ✅ DO
- **Contrato:** pytest pasa; query() retorna estructura; errores tipados (VantaError) no solo RuntimeError; docs PYTHON_SDK actualizada
- **Task file:** `skills/campaign-executor/tasks/MOD-20.md`
- **Estado:** ✅ COMMITTED `feat(python)` — jerarquía VantaError + query_structured, pytest 132

  **Pre-mortem:**
  - Fallo 1: cambiar query() a estructura rompe callers existentes (breaking)
  - Fallo 2: mapeo VantaError→PyErr inconsistente con MOD-19

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🔴 | query() breaking change | aditivo: añadir método nuevo, mantener query() | tests legacy fallan |

  **Cynefin:** 🟨 complicado — diseño de API.
  **Top 3 riesgos:** (1) breaking query(); (2) mapeo errores; (3) docs.

### Task 8: FIND-10 — TS packaging ESM-only + errores genéricos

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-ts/package.json`, `vantadb-ts/dist/errors.js`, `vantadb-ts/src/vantadb.ts` (wrapWasmError)
- **Verificación real:** 🟡 VERIFICAR — backlog: package ESM-only sin export CJS; errores colapsan a WASM_ERROR. Confirmar en DISCOVERY.
- **Gate Justificación:** DX TS real: `require()` falla; errores indistinguibles. Esfuerzo 🟡.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` (ts) exit 0; `require("vantadb")` funciona (export CJS) o decisión documentada; errores distinguen corrupto/not-found/validación
- **Task file:** `skills/campaign-executor/tasks/FIND-10.md`
- **Estado:** ✅ COMMITTED `fix(ts)` — require(esm) + error codes, npm 261/261

  **Pre-mortem:**
  - Fallo 1: dual ESM/CJS export rompe el build wasm
  - Fallo 2: mapeo de errores requiere acceso al discriminante VantaError en wasm

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟡 | dual export rompe wasm | test require + import ESM | build falla |

  **Cynefin:** 🟨 complicado — dual packaging.
  **Top 3 riesgos:** (1) dual export; (2) mapeo errores wasm; (3) build.

### Task 9: FIND-29 — último cast manual u8*→f32* en layer.rs

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `src/index/search/layer.rs:66-71,234-237`
- **Verificación real:** ✅ CÓDIGO-REAL — backlog + FIND-28 audit: último `from_raw_parts(u8*→f32*)` manual sobre mmap; sound hoy (INV-024 align) pero candidato a `as_f32_slice()`. Gap real (consistencia, no UB).
- **Gate Justificación:** hermano de FIND-28 (ya hecho), elimina último UB manual del index; fix mecánico, effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb layer` (o tests index) pasa; `cargo clippy -p vantadb --all-targets -- -D warnings`; `as_f32_slice` aplicado; 0 `from_raw_parts` u8*→f32* residual en layer.rs
- **Task file:** `skills/campaign-executor/tasks/FIND-29.md`
- **Estado:** ✅ COMMITTED `fix(index)` — align_to en layer.rs, 0 casts residuales, layer 10/10, index 354/354

  **Pre-mortem:**
  - Fallo 1: tocar hot path de search → regresión sin medir
  - Fallo 2: la semántica de casos límite (None/Err) difiere del cast manual

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟢×🟡 | hot path search | tests suite completa | regresión tests |

  **Cynefin:** 🟦 obvio (mismo fix que FIND-28).
  **Top 3 riesgos:** (1) hot path; (2) casos límite; (3) semántica.

### Task 10: REVIEW-17 — unsafe wasm32 innecesarios

- **Appetite:** max 2h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `src/storage/vfile*.rs`, `src/storage/archive.rs`
- **Verificación real:** 🟡 VERIFICAR — backlog: 7 bloques `unsafe` bajo wasm32 (vfile.rs:202,206,277; archive.rs:74,105; vfile_mmap.rs:73,112), mmap safe en ese backend. Confirmar en DISCOVERY.
- **Gate Justificación:** eliminar/documentar unsafe innecesarios — higiene de seguridad (Regla 4). Effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check --target wasm32-unknown-unknown` (o el target disponible); unsafe removidos o `// SAFETY:` por-plataforma; `cargo clippy --workspace -- -D warnings`
- **Task file:** `skills/campaign-executor/tasks/REVIEW-17.md`
- **Estado:** ✅ COMMITTED `fix(storage)` — 7 unsafe wasm32 removidos, wasm 0 warnings, storage 368/368

  **Pre-mortem:**
  - Fallo 1: remover unsafe en un path que sí lo necesita (vfile mmap)
  - Fallo 2: no se puede compilar wasm32 en Windows fácilmente → verify parcial

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟡 | remover unsafe necesario | documentar invariante por-plataforma en vez de forzar | cargo check wasm falla |

  **Cynefin:** 🟨 complicado — seguridad FFI.
  **Top 3 riesgos:** (1) unsafe necesario; (2) verify wasm en Windows; (3) invariantes.

### Task 11: MCP-24 — search_with_method / search_multi

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `src/sdk/search/mod.rs:78`, `src/sdk/search/multi.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — backlog + previo: `search_with_method` y `search_multi` existen en SDK sin tool MCP. Gap real.
- **Gate Justificación:** control fino de backend + multi-consulta desde el agente; wrappers finos.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp --test mcp_tests` pasa; tools round-trip; docs skill ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MCP-24.md`
- **Estado:** ✅ COMMITTED `feat(mcp)` — search_with_method + search_multi, mcp_tests 68/68, hash SAME

  **Pre-mortem:**
  - Fallo 1: shape del array de queries en wire format
  - Fallo 2: hash SAME docs

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟢×🟢 | wire shape | reutilizar patrones MCP existentes | test falla |

  **Cynefin:** 🟦 obvio.
  **Top 3 riesgos:** (1) wire shape; (2) hash SAME; (3) defaults.

### Task 12: MCP-33 — axiom write/delete

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/axioms.rs`, `vantadb-mcp/src/handlers/tools.rs`, `src/agentic/axioms*`
- **Verificación real:** 🟡 VERIFICAR — backlog: solo existe `read_axioms`; investigar si hay API de escritura en core (`src/agentic/axioms*`); si no, definir records en namespace `_axioms`. Confirmar en DISCOVERY (uphill).
- **Gate Justificación:** axiomas gestionables por el agente; es la feature de reglas invariantes. Necesita investigación previa.
- **Gate Result:** ✅ DO
- **Contrato:** tools `write_axiom`/`delete_axiom`; round-trip; Iron Axioms (read_axioms) intactos; docs skill ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MCP-33.md`
- **Estado:** ✅ COMMITTED `feat(mcp)` — write/delete_axiom, mcp_tests 70/70, hash SAME

  **Pre-mortem:**
  - Fallo 1: no hay API de escritura core → solución ad-hoc en namespace `_axioms` mal diseñada
  - Fallo 2: colisión con Iron Axioms internos

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟡 | sin API core de escritura | documentar convención namespace `_axioms` | DISCOVERY no encuentra API |

  **Cynefin:** 🟨 complicado — requiere investigación de diseño.
  **Top 3 riesgos:** (1) sin API core; (2) colisión Iron Axioms; (3) convención.

### Task 13: MCP-34 — snapshot create/restore

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `src/storage/engine/mod.rs`, `vantadb-mcp/src/handlers/tools.rs`
- **Verificación real:** 🟡 VERIFICAR — backlog: `list_snapshots` existe; verificar si hay create/restore en StorageEngine. Confirmar en DISCOVERY (uphill).
- **Gate Justificación:** backup físico puntual desde el agente; restore exige validación de path (trust boundary). Necesita verificación de API.
- **Gate Result:** ✅ DO
- **Contrato:** tools `snapshot_create`/`snapshot_restore` si existen métodos públicos; path sanitizado (solo data dir); tests + docs ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MCP-34.md`
- **Estado:** 🟡 DEFER (STOP CONDITION — `snapshot_restore` NO existe como método público; requiere core nuevo)

  **Pre-mortem:**
  - Fallo 1: create/restore no existen como métodos públicos → tarea se convierte en implementación core (scope crece)
  - Fallo 2: restore path traversal (trust boundary)

  **Stop conditions:** si no existen métodos públicos y requiere core nuevo → re-triaje como DEFER (uphill).

  **DISCOVERY (worker 2026-08-25):** ⬆️ UP-HILL resuelto. `snapshot_create` SÍ existe como método público (`StorageEngine::create_snapshot` mod.rs:507/:540 + SDK builder.rs:253 + CLI + HTTP). `snapshot_restore` NO existe en NINGÚN lado (core/SDK/CLI/server/MCP) — los únicos `restore` son `restore_graph_nodes` (grafos) y `restore_to_timestamp` (WAL PITR); el trait `StorageBackend` solo tiene `checkpoint` sin restore. Implementar restore = feature core nueva en `storage/engine` (swap data_dir + flush WAL + reload engine) → STOP CONDITION según plan Task 13. **DEFER**; candidato a desglose `MCP-34a` (wrapper snapshot_create) en un batch de wrappers si el lead lo prioriza.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🔴 | requiere core nuevo | verificar API primero en DISCOVERY; si no, DEFER | DISCOVERY no encuentra create/restore |

  **Cynefin:** 🟨 complicado.
  **Top 3 riesgos:** (1) sin API core; (2) path traversal; (3) scope.

### Task 14: FIND-06 — READMEs SDK en español → inglés

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-python/README.md`, `vantadb-ts/README.md` (y otros SDK READMEs en español)
- **Verificación real:** ✅ CÓDIGO-REAL — FIND-04 (ya hecho) notó README Python íntegramente en español, violación de la regla de idioma (AGENTS.md Doc Language Split). Gap real.
- **Gate Justificación:** consistencia de idioma (técnico=en inglés); no-código, docs. Effort 🟡 (migración de contenido).
- **Gate Result:** ✅ DO
- **Contrato:** READMEs SDK técnicos en inglés; `scripts/validate-docs-coverage.ps1` 0 gaps; FIND-06 en backlog marcado DONE
- **Task file:** `skills/campaign-executor/tasks/FIND-06.md`
- **Estado:** ✅ COMMITTED `docs` — Python README en inglés, docs coverage 0 gaps

  **Pre-mortem:**
  - Fallo 1: traducción masiva introduce imprecisiones técnicas
  - Fallo 2: no detectar todos los READMEs en español

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟡×🟢 | traducción imprecisa | revisar términos técnicos vs código | coverage falla |

  **Cynefin:** 🟦 obvio.
  **Top 3 riesgos:** (1) precisión traducción; (2) alcance; (3) coverage.

### Task 15: FIND-18 — NOTICE file ausente

- **Appetite:** max 30m
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** raíz repo, `deny.toml`
- **Verificación real:** 🟡 VERIFICAR — backlog: NOTICE file ausente (recomendado Apache-2.0 para atribución de terceros). Confirmar en DISCOVERY.
- **Gate Justificación:** compliance de licencias (Apache-2.0 atribución); effort 🟢. Tarea de release/lead.
- **Gate Result:** ✅ DO
- **Contrato:** `NOTICE` creado con atribuciones de deps de deny.toml; `cargo deny check` sigue pasando
- **Task file:** `skills/campaign-executor/tasks/FIND-18.md`
- **Estado:** ✅ COMMITTED `chore` — NOTICE creado, cargo deny licenses ok

  **Pre-mortem:**
  - Fallo 1: listado de terceros incompleto o desactualizado
  - Fallo 2: NOTICE duplica contenido de LICENSE

  **Stop conditions:** —.

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger |
  |--------------|--------|-----------|---------|
  | 🟢×🟢 | atribución incompleta | generar desde deny.toml | audit deny falla |

  **Cynefin:** 🟦 obvio.
  **Top 3 riesgos:** (1) listado; (2) duplicación; (3) deny.

## Waves

- **Wave 0** (independientes, core correctness): REVIEW-13 · FIND-29 · MOD-14
- **Wave 1**: MOD-04 · REVIEW-17 · FIND-18
- **Wave 2**: MOD-10 · MCP-24 · MOD-13
- **Wave 3**: MOD-18 · MOD-20 · FIND-10
- **Wave 4**: MCP-33 · MCP-34 · FIND-06

> MAX_CONCURRENT = 3. Sub-agentes NO commitean; el lead verifica mecánico y commitea por tarea.

## DEFER (justificado)

| ID | Motivo |
|----|--------|
| MOD-05 | Deprecar InMemoryEngine (~850 líneas) — refactor grande, no es AHORA vs otras tareas (Shape Up: scope > appetite) |
| MOD-06 | Nits agrupados WAL — bajo impacto, agrupar con próxima sesión WAL |
| MOD-11 | Nits MCP agrupados (k clamp, timeout, etc.) — bajo impacto |
| MOD-21 | Nits Python agrupados — bajo impacto, esperar a que MOD-18/20 cierren para no colisionar en vantadb-python |
| MCP-34 | `snapshot_restore` no existe como método público en core/SDK/CLI/server — requiere feature core nueva en `storage/engine` (fuera del batch de wrappers MCP). `snapshot_create` wrapper MCP candidato a desglose MCP-34a. |
| FIND-11 | desktop/ README + rutas — BLOQUEADO por restricción desktop |
| FIND-20 | persistencia ventana Tauri — desktop |
| FIND-21 | menú contextual/atajos Tauri — desktop |
| AGT-01..06 | P36 agentes — dependen de commits de otra sesión (AGT-01) / requerirían coordinar con sesión P34 |
| DAUD-01..09 | P37 desktop — no tocar desktop (sesión concurrente) |
| UX-02..19 | P34 desktop — no tocar desktop |
| GOV-TK1..9 | tickets GOV — varios requieren infra/decisiones, no AHORA |

## SKIP

| ID | Motivo |
|----|--------|
| UX-18 | Wontfix documental (HMR noise de desarrollo) |

## BLOQUEADO

| ID | Motivo |
|----|--------|
| CORE-01 | Persistencia Binary on-disk — requiere ADR de formato ANTES (gate Regla 5), no arranca sin decisión de formato |

## Notas

- plan-adjust [2026-08-25]: creado desde docs/Backlog.md; set DO ampliado por decisión del usuario (15 tareas); restricción NO-desktop por sesión P34/P37 concurrente.
- ⬆️ uphill = 3 (MCP-33, MCP-34, MOD-04 diseño) · ⬇️ downhill = 12. [2026-08-25] MCP-33 ✅ wrappers, MCP-34 🟡 DEFER (snapshot_restore core-nuevo), MOD-04 ✅ → uphill = 0.

=== RECITATION FIND-29 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: FIND-29: eliminar los 2 from_raw_parts u8*→f32* de layer.rs vía mecanismo align_to de as_f32_slice
Estado: completed
Última acción: Steps 1-2: reemplacé los 2 casts por `vec_data.align_to::<f32>()` + guard `len != expected → 0.0`, debug_assert preservados, SAFETY actualizado. Step 3: test test_layer_align_to_decodes_original_values (equivalencia de valores). Step 4: verify — index 354/354, layer/search 10/10, check OK, rustfmt de mis 2 archivos OK, 0 casts residuales. Task file actualizado.
Resultado: OK
Próxima acción: ninguno — tarea completa; el lead verifica mecánico full-workspace y commitea
Contrato: verificacion: cargo nextest run -p vantadb -E 'test(index)' 354/354 ✅; -E 'test(search_layer)|test(search_vfile)|test(layer_)' 10/10 ✅; cargo check -p vantadb ✅; rustfmt --check layer.rs tests.rs ✅; 0 from_raw_parts/as*const f32 residual en layer.rs ✅; evidencia: {claim: 'approach align_to = mecanismo de as_f32_slice (REVIEW-15)', evidencia: 'src/node/vector_data.rs:176 + git show 2d9fa75f', confianza: alta}, {claim: 'cambio clippy-clean', evidencia: 'clippy --lib --all-targets: único diagnóstico es builder.rs:25 (REVIEW-13), ninguno en layer.rs/tests.rs', confianza: alta}; invariantes: mismos valores &[f32], fallback 0.0 idéntico, debug_assert preservados, 0 cast residual; deuda: colateral clippy/fmt full-workspace = dead-code builder.rs:25 (WIP REVIEW-13 concurrente, NO mi cambio) — el lead lo resuelve en su tarea; queda_pendiente: el lead resuelve WIP del server (MOD-14/REVIEW-13 in-progress) y commitea layer.rs+tests.rs+FIND-29.md
Próxima tarea si completa: ninguna (única tarea de esta invocación)
=== END RECITATION ===

=== RECITATION REVIEW-13 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: REVIEW-13 — eliminar TOCTOU concurrente en supersede() serializando el read-modify-write
Estado: completed
Última acción: Implementación completa: supersede_lock (Arc<Mutex<()>>) en VantaEmbedded + guard en supersede() + test concurrente con Barrier. RED confirmado 7/7 sin guard, GREEN 10/10 con fix. fmt/clippy/check verdes.
Resultado: OK
Próxima acción: Lead: git add src/sdk/builder.rs src/sdk/api.rs + commit fix: REVIEW-13; auditoría Regla 8 (vanta-chaos + vanta-review) antes de merge
Contrato: verificacion: cargo nextest run -p vantadb supersede ✅ 10/10 + cargo check -p vantadb ✅ + cargo fmt --check ✅ + cargo clippy -p vantadb --all-targets -- -D warnings ✅; evidencia: claim=supersede() tenía TOCTOU confirmado (RED 7/7 con r1=Ok r2=Ok), evidencia=src/sdk/api.rs:851-886 + guard, confianza=alta; claim=fix serializa RMW sin cambiar semántica (idempotency guard intacto, 4 tests existentes pasan), evidencia=src/sdk/builder.rs supersede_lock, confianza=alta; artefactos=.opencode/skills/campaign-executor/tasks/REVIEW-13.md; invariantes=idempotency guard y semántica de supersede intactas, VantaEmbedded sigue Clone, deadlock-free (lock order supersede_lock->engine.read->insert_lock); deuda=ninguna; queda_pendiente=auditoría Regla 8 delegada al lead (vanta-chaos + vanta-review)
Próxima tarea si completa: MOD-04
=== END RECITATION ===

=== RECITATION REVIEW-17 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: REVIEW-17: remover/documentar 7 bloques unsafe innecesarios bajo wasm32 (vfile.rs, vfile_mmap.rs, archive.rs)
Estado: completed
Última acción: EJECUCION completa: agregue helpers safe map_readonly/map_readwrite en vfile_mmap.rs (cfg-gated, SAFETY+invariante); migre callers vfile.rs:202,206,277 y archive.rs:74,105; removi bloques internos redundantes vfile_mmap.rs:73,112. 7 unsafe eliminados; unsafe real de memmap2 concentrado en 2 helpers. wasm32: 7 warnings->0. Hallazgo pre-existente FIND-30 (cli_server.rs) ruteado a Backlog.
Resultado: OK
Próxima acción: Ninguno (tarea completa). Lead verifica y commitea por tarea (NO commiteado).
Contrato: verificacion: cargo check --target wasm32-unknown-unknown -p vantadb-wasm 0 warnings (baseline 7) PASSA; cargo check -p vantadb PASSA; cargo clippy -p vantadb --lib -- -D warnings (memmap2) PASSA; cargo clippy -p vantadb --no-default-features --features wasm --lib -- -D warnings (shim) PASSA; cargo nextest run -p vantadb storage:: 363/363 PASSA; cargo fmt --check PASSA. evidencia: claim='7 unsafe removidos sin cambio de comportamiento nativo' evidencia=git diff src/storage/*.rs + helpers cfg-gated confianza=alta; claim='wasm32 0 warnings' evidencia=cargo check --target wasm32-unknown-unknown -p vantadb-wasm confianza=alta. artefactos: src/storage/vfile_mmap.rs, src/storage/vfile.rs, src/storage/archive.rs, docs/Backlog.md (FIND-30), task REVIEW-17.md. invariantes: NO cambiar firmas publicas Mmap::map/MmapMut::map_mut/MmapOptions (callers directos graph.rs, serialize/file.rs, maintenance.rs, vector_data.rs); shim mantiene unsafe fn parity; helpers no rompen alineacion (AlignedBytes 4-aligned, memmap2 page-aligned). deuda: cargo clippy --workspace --all-targets -- -D warnings falla SOLO por FIND-30 pre-existente (cli_server.rs:1302 unused ns, feature server, ajeno a REVIEW-17, dueño MOD-13) — fuera de blast radius, ruteado a Backlog. queda_pendiente: lead commitea los 4 archivos (storage x3 + Backlog FIND-30); MOD-13 puede absorber FIND-30.
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION MOD-04 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: MOD-04: purge_expired O(N) full-scan → índice TTL selectivo
Estado: completed
Última acción: DISCOVERY completo (codegraph + reglas): StorageEngine ya mantiene scalar_index en writes incluyendo expires_at_ms, pero sin range query ni rebuild-on-open. Implementé lookup_int_le + scalar_lookup_int_le + rebuild_scalar_index (open + rebuild_index) + purge_expired selectivo metadata-only. Bench before/after con mejora real. Test unitario rebuild-on-open. FIND-31 registrado. Verify full 2763/2763.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea los archivos de MOD-04 (NO commit hecho por diseño)
Contrato: verificacion: cargo bench --bench purge_expired -- --warm-up-time 1 --measurement-time 3 --sample-size 12 ANTES(137.20ms/1.2341s)→DESPUÉS(117.22ms/1.0726s) −23.9%/−9.1% p<0.05 ✅; cargo nextest run -p vantadb 2060/2060 ✅; cargo nextest run --profile audit --workspace --build-jobs 2 2763/2763 ✅; cargo fmt --check ✅; cargo clippy --workspace --all-targets --all-features -- -D warnings ✅; scripts/validate-docs-coverage.ps1 0 gaps ✅; evidencia: {claim: 'ScalarIndex ya indexa expires_at_ms en writes del StorageEngine persistente', evidencia: 'src/storage/engine/insert.rs:214-218', confianza: alta}, {claim: 'scalar_index se creaba vacío en open y no se reconstruía (init.rs:141 + recover_state usa replay_write_node)', evidencia: 'src/storage/engine/init.rs:141,479-598', confianza: alta}, {claim: 'metadata-only purge gana −23.9%/−9.1% vs full scan; engine.get() por candidato REGRESABA (+11%/+24%)', evidencia: 'benches/purge_expired.rs + criterion output 3 runs', confianza: alta}, {claim: 'bug text index df negativo tras reopen es PRE-EXISTENTE (reproducido con código original via git stash)', evidencia: 'cargo test --test fuzz_proptest con stash + FIND-31 en Backlog', confianza: alta}; artefactos: benches/purge_expired.rs, task file MOD-04.md; invariantes: purge_expired mantiene semántica idéntica (filtro now>expires, misma construcción de VantaMemoryRecord, mismo loop de delete/text index); deuda: FIND-31 (bug text index tras reopen, pre-existente) queda en Backlog para el lead; queda_pendiente: el lead verifica y commitea (regla: sub-agentes NO commitean) — archivos: src/scalar_index.rs, src/sdk/api.rs, src/storage/engine/{init,mod}.rs, src/storage/engine/tests/init.rs, benches/purge_expired.rs, Cargo.toml, docs/Backlog.md, task file
Próxima tarea si completa: MOD-04
=== END RECITATION ===

=== RECITATION MOD-13 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: MOD-13: TimeoutLayer de request en cli_server.rs (handler atascado no retiene conexión)
Estado: completed
Última acción: Implementacion completa: feature timeout tower-http, REQUEST_TIMEOUT 30s + LONG_REQUEST_TIMEOUT 600s, sub-router long-running mergeado, 3 tests, verify full verde
Resultado: OK
Próxima acción: Lead: verifica mecanico y commitea
Contrato: verificacion: cargo check -p vantadb --features server ✅; cargo test -p vantadb --features server --lib cli_server::tests:: 42/42 ✅; cargo nextest run -p vantadb-server 5/5 ✅; cargo test -p vantadb-server --test e2e 12/12 ✅; rustfmt --check src/cli_server.rs ✅; cargo clippy -p vantadb --features server --all-targets ✅; evidencia: [{claim: gap real 0 TimeoutLayer, evidencia: rg TimeoutLayer src/cli_server.rs vacio, confianza: alta}, {claim: request lento 408, evidencia: slow_request_times_out_with_408 42/42, confianza: alta}, {claim: 2 fallos rate-limit server.rs pre-existentes, evidencia: git stash reproduce en base + FIND-32 Backlog, confianza: alta}]; artefactos: .opencode/skills/campaign-executor/tasks/MOD-13.md, docs/Backlog.md; invariantes: rutas 1:1, auth+governor intactos, firma app* intacta, public sin timeout; deuda: ninguna; queda_pendiente: lead commitea Cargo.toml + src/cli_server.rs + Backlog FIND-32
Próxima tarea si completa: MOD-10 / MCP-24 segun lead
=== END RECITATION ===

=== RECITATION MCP-24 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: MCP-24: exponer search_with_method y search_multi como tools MCP
Estado: completed
Última acción: Implementación completa: parser compartido parse_search_request (enum ParsedSearchRequest Ready/Rejected), parse_search_method, arms search_with_method/search_multi + refactor search_memory, 2 tools defs, 3 tests round-trip, docs SKILL.md x2 hash SAME + docs/api/MCP.md. Verify full verde: mcp_tests 68/68, check, fmt, clippy.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea; coordinar merge con MOD-10 (mismos archivos tools.rs/mcp_tests.rs)
Contrato: verificacion: cargo test -p vantadb-mcp --test mcp_tests ✅ 68/68; cargo check -p vantadb-mcp ✅; cargo fmt --check ✅; cargo clippy -p vantadb-mcp --all-targets -- -D warnings ✅; Get-FileHash SKILL.md x2 = D0C2DE2E… SAME ✅. evidencia: {claim: 'search_with_method/search_multi existían en SDK sin tool MCP', evidencia: 'src/sdk/search/mod.rs:84 + multi.rs:20 (leídos)', confianza: alta}, {claim: '3 tests MCP-24 round-trip pasan', evidencia: 'cargo test -p vantadb-mcp --test mcp_tests 68/68', confianza: alta}, {claim: 'canales de error de search_memory preservados', evidencia: 'test_search_profile_validation_errors + test_mcp_search_filters_accept_eq_and_reject_range verdes tras refactor', confianza: alta}; artefactos: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md, .opencode/skills/vantadb-mcp/SKILL.md, docs/api/MCP.md, task file MCP-24.md; invariantes: canales de error intactos (Rejected=Ok(error_content), param=Err), SKILL.md x2 hash SAME, no tocar sdk/search/*; deuda: ninguna propia; colateral MOD-10 (supersede test) fallaba temporalmente y fue resuelto por su agente durante la sesión; queda_pendiente: el lead verifica y commitea (sub-agentes NO commitean); coordinar con commit MOD-10 por solapamiento de archivos
Próxima tarea si completa: ninguna
=== END RECITATION ===

=== RECITATION 3 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: MOD-10: exponer versions/supersede/vacuum/remove_edge como tools MCP
Estado: completed
Última acción: Implementación completa: 4 tool definitions + 4 dispatch arms en tools.rs, 5 tests round-trip en mcp_tests.rs, docs SKILL.md + api-reference.md en ambos trees (hash SAME, 0 mismatches). Verify: mcp_tests 68/68, check, fmt, clippy todos verdes. Hallazgo core documentado (snapshots dropean supersession). Colaboración concurrente con MCP-24 en mismos archivos — lead mergea diff combinado.
Resultado: OK
Próxima acción: Lead: mergear/commitear los 6 archivos (tools.rs, mcp_tests.rs, SKILL.md x2, api-reference.md x2) + resolver WIP del server
Contrato: verificacion: cargo test -p vantadb-mcp --test mcp_tests 68/68 OK; cargo check -p vantadb-mcp OK; cargo fmt --check OK; cargo clippy -p vantadb-mcp --all-targets -- -D warnings OK; evidencia: {claim: '4 tools nuevas round-trip', evidencia: 'vantadb-mcp/tests/mcp_tests.rs test_mcp_memory_versions_round_trip, test_mcp_memory_supersede_round_trip, test_mcp_vacuum_round_trip, test_mcp_remove_edge_round_trip', confianza: alta}, {claim: 'docs skill x2 hash SAME', evidencia: 'Get-FileHash skills/vantadb-mcp/SKILL.md == .opencode/skills/vantadb-mcp/SKILL.md + tree 0 mismatches', confianza: alta}; artefactos: .opencode/skills/campaign-executor/tasks/MOD-10.md; invariantes: shape MEM-32 (error_content para dominio), u128 como strings, VacuumReport JSON manual, docs idénticos entre trees; deuda: ninguna; queda_pendiente: lead resuelve WIP del server (MOD-13 in-progress bloquea update_task_state) y commitea diff combinado con MCP-24
Próxima tarea si completa: MCP-24
=== END RECITATION ===

=== RECITATION FIND-10 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: FIND-10 — fix DX TS: require('vantadb') + errores distinguibles (corrupto/not-found/validación)
Estado: completed
Última acción: Implementación completa: (1) wasm to_js_err adjunta code estructural vía Reflect::set (vanta_error_code: NOT_FOUND/VALIDATION_ERROR/CORRUPT/RESOURCE_LIMIT/TIMEOUT/BUSY/IO_ERROR/WASM_ERROR, mensaje intacto); (2) errors.ts: ERROR_CODES ampliados + classifyWasmError (fallback por prefijo) + wrapWasmError prioriza e.code; (3) package.json: condición require→ESM + engines >=22.12; (4) README: Module Formats + Errors + Runtimes; (5) 8 tests nuevos (unit + integración real wasm). pkg regenerado con wasm-pack --release. Verify completo verde.
Resultado: OK
Próxima acción: Lead: git add de los 6 archivos + commit fix: FIND-10; rebuild pkg --release en CI
Contrato: {"verificacion": "npm run build (vantadb-ts) exit 0 ✅; npm test 261/261 ✅ (8 files); npx tsc --noEmit exit 0 ✅; node -e require('vantadb') self-reference OK (put/get roundtrip) ✅; cargo check -p vantadb-wasm (host + wasm32) ✅; cargo clippy -p vantadb-wasm --all-targets -- -D warnings ✅; rustfmt --check lib.rs ✅", "evidencia": [{"claim": "require('vantadb') fallaba ERR_PACKAGE_PATH_NOT_EXPORTED y ahora funciona", "evidencia": "node -e require('vantadb') desde vantadb-ts (Node 24.16) + docs nodejs.org/api/packages.html#conditional-exports (condición require admite ES modules)", "confianza": "alta"}, {"claim": "errores wasm distinguen clases: get('',k)/zero-norm→VALIDATION_ERROR, add_edge nodos faltantes→NOT_FOUND", "evidencia": "probe node + tests integración FIND-10 (hardening.test.ts) con pkg release", "confianza": "alta"}, {"claim": "mensajes wasm byte-idénticos — cero breakage tests existentes", "evidencia": "npm test 261/261 incluye /zero-norm|undefined|InvalidInput/i y toThrow(VantaError) intactos", "confianza": "alta"}], "artefactos": ".opencode/skills/campaign-executor/tasks/FIND-10.md; dist/ y pkg/ regenerados (git-ignored)", "invariantes": "mensajes wasm byte-idénticos; API TS sync intacta; import consumers ESM intactos; pkg regenerado con --release (--dev trapea unreachable en put/close)", "deuda": "ninguna — dual build CJS real (tsup) descartado documentado (grafo wasm ESM-only + API sync); clasificación por prefijo marcada ponytail: (depende de strings Display estables)", "queda_pendiente": "lead commitea los 6 archivos; colateral README stale (await db.put() en API sync) ruteado a FIND-06"}
Próxima tarea si completa: MCP-33 (uphill) o la que asigne el lead
=== END RECITATION ===

=== RECITATION MOD-18 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: MOD-18: consolidar stubs .pyi (fuente única de verdad nativa) + test anti-drift firma↔stub
Estado: completed
Última acción: Implementación completa TDD: (1) tests/test_stub_drift.py 7 tests anti-drift (ast+inspect), RED 6/7 contra stubs viejos; (2) vantadb_py.pyi rewrite = fuente única de verdad nativa (48 métodos VantaDB + subclients + params corregidos); (3) __init__.pyi rewrite = wrapper re-export (SearchRequest + AsyncVantaDB completo). Verify full: 125 passed 4 deselected (160s), ast.parse stubs OK, cargo check OK. Sin commits (lead).
Resultado: OK
Próxima acción: Lead: verifica y commitea los 3 archivos (2 stubs + test) + task file con fix(python): consolidate vantadb_py stubs + anti-drift test (MOD-18)
Contrato: verificacion: python -m pytest tests/ -q 125 passed 4 deselected ✅; python -m pytest tests/test_stub_drift.py -q 7/7 ✅; ast.parse de ambos stubs ✅; cargo check -p vantadb Finished dev ✅; mypy/pyright NO instalados → NO CORRIDO (documentado); evidencia: [claim=drift real mapeado y corregido (put_batch list[dict]→list[VantaMemoryRecord], entries requerido, 6 métodos VantaDB faltantes, 4 params faltantes, AsyncVantaDB 5 métodos faltantes), evidencia=RED 6/7 + inspect.signature módulo compilado 0.5.0, confianza=alta; claim=test anti-drift valida firmas reales contra stub (métodos+params+requiredness+subclients+return types), evidencia=tests/test_stub_drift.py 7 tests GREEN, confianza=alta; claim=estructura 3 capas documentada en vez de forzar 1 stub, evidencia=vantadb_py.pyi docstring + pyproject include, confianza=alta]; artefactos=vantadb-python/vantadb_py/vantadb_py.pyi, vantadb-python/vantadb_py/__init__.pyi, vantadb-python/tests/test_stub_drift.py, task MOD-18.md; invariantes=estructura de import intacta (vantadb_py.pyd → __init__.py → vantadb alias), pyproject sin cambios, métodos ya correctos sin tocar; deuda=mypy/pyright no instalados — verify mypy pendiente (opcional); queda_pendiente=lead verifica mecánico (ya corrido) y commitea
Próxima tarea si completa: MOD-20
=== END RECITATION ===

=== RECITATION FIND-06 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: FIND-06: migrar READMEs SDK de español a inglés + validar-docs-coverage 0 gaps + FIND-06 DONE en backlog
Estado: completed
Última acción: Discovery: único SDK README en español es vantadb-python/README.md (TS ya en inglés, integrations/providers/wasm-demo en inglés, wasm-pkg generado, node sin README). Traduje la prosa descriptiva del README Python a inglés preservando code blocks verbatim y la sección Cross-SDK Search Parity. validate-docs-coverage.ps1 mostró 4 gaps pre-existentes en docs/api/MCP.md (memory_versions, memory_supersede, remove_edge, vacuum — tools commiteadas por MOD-10 sin doc) → los documenté inline + actualicé conteos (Core Tools 36→42). Verifiqué grep: 0 prosa en español fuera de code blocks, links intactos. Marcó FIND-06 DONE en Backlog (fila P33 + sección completadas).
Resultado: OK
Próxima acción: Lead: verifica mecánico (validate-docs-coverage.ps1 ya 0 gaps) y commitea: vantadb-python/README.md, docs/api/MCP.md, docs/Backlog.md, task file FIND-06.md
Contrato: verificacion: pwsh scripts/validate-docs-coverage.ps1 exit 0 — 'Validación de cobertura completada - 0 gaps' (7/7 grupos ok, vantadb-mcp 42 items ok) ✅; grep prosa fuera de code blocks: 0 coincidencias español ✅; links internos intactos (../docs/QUICKSTART.md#4-real-embeddings-optional, ../docs/api/BINDINGS_NAMESPACES.md) ✅; evidencia: [{claim: 'README Python ahora en inglés preservando code blocks y sección Cross-SDK', evidencia: 'vantadb-python/README.md (leído post-escritura) + grep stopwords/acentos en prosa = 0', confianza: alta}, {claim: '4 gaps MCP.md pre-existentes de MOD-10 arreglados inline', evidencia: 'git status: MCP.md no modificado por mi antes; tools memory_versions/supersede/remove_edge/vacuum definidas en tools.rs:117,129,306,393 sin entrada previa en docs/api/MCP.md', confianza: alta}, {claim: 'coverage 0 gaps', evidencia: 'pwsh scripts/validate-docs-coverage.ps1 exit 0', confianza: alta}]; artefactos: vantadb-python/README.md, docs/api/MCP.md, docs/Backlog.md, .opencode/skills/campaign-executor/tasks/FIND-06.md; invariantes: code blocks verbatim (no traducir strings/código), sección Cross-SDK Search Parity intacta, links internos sin cambiar rutas, pkg wasm generado no tocado; deuda: ninguna propia; queda_pendiente: el lead commitea los 4 archivos (sub-agentes NO commitean)
Próxima tarea si completa: ninguna (única tarea de esta invocación)
=== END RECITATION ===

=== RECITATION MCP-33 ===
Campaign ID: 145a38aa-b5aa-4ac5-8c81-8a0c14ae4995
Objetivo activo: MCP-33: agregar write_axiom/delete_axiom manteniendo Iron Axioms intactos
Estado: completed
Última acción: Implementación completa: (1) axioms.rs resolve_axioms = Iron hardcoded (ids 1-4, siempre) + merge records namespace _axioms, AXIOMS_NAMESPACE, removidos SYSTEM_NAMESPACE/AXIOMS_STORAGE_KEY dead; (2) tools.rs 2 defs write_axiom/delete_axiom + 2 arms dispatch con validación validate_identifier/validate_payload y id auto >4; (3) mcp_tests.rs 2 tests round-trip (write→read→delete→read + Iron intactos) + validation. Docs SKILL.md x2 + MCP.md. Verify full verde.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea (sub-agentes NO commitean) — archivos: vantadb-mcp/src/axioms.rs, handlers/tools.rs, tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md, .opencode/skills/vantadb-mcp/SKILL.md, docs/api/MCP.md, task MCP-33.md
Contrato: verificacion: cargo test -p vantadb-mcp --test mcp_tests 70/70 ✅; cargo check -p vantadb-mcp ✅; cargo check -p vantadb ✅; cargo fmt --check ✅; cargo clippy -p vantadb-mcp --all-targets -- -D warnings ✅; Get-FileHash SKILL.md x2 = 5ED86246… SAME ✅; validate-docs-coverage MCP 44/44 ✅. evidencia: [{claim: 'no hay API de escritura de axiomas en core', evidencia: 'src/agentic solo mod.rs/thread.rs, grep -i axiom src/ solo comentario executor.rs:313', confianza: alta}, {claim: 'write_axiom/delete_axiom round-trip + Iron Axioms intactos', evidencia: 'test_mcp_axiom_write_delete_round_trip_and_iron_intact + test_mcp_axiom_write_validation_errors (70/70)', confianza: alta}, {claim: 'docs skill x2 hash SAME', evidencia: 'Get-FileHash skills/vantadb-mcp/SKILL.md == .opencode/skills/vantadb-mcp/SKILL.md = 5ED86246…', confianza: alta}]; artefactos: task MCP-33.md; invariantes: Iron Axioms ids 1-4 siempre presentes (hardcoded, nunca escritos/borrados), namespace _axioms con payload JSON {id>4,name,description}, no tocar core src/; deuda: colateral pre-existente docs-coverage gap query_structured en vantadb-python (NO de MCP-33, relacionado MOD-20) — lead rutea; queda_pendiente: lead commitea + desbloquea plan (update_task_state bloqueado por lock FIND-06) + decide ruteo query_structured; progreso skill difiere al lead (plan file uncommitted, sesión concurrente)
Próxima tarea si completa: MCP-34 (uphill) o la que asigne el lead
=== END RECITATION ===
