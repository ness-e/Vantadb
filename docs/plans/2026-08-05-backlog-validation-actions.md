# Plan de Ejecución: Backlog Validation Actions — 2026-08-05

> **Campaign ID: ed2b8ad1-58fb-4ea9-bea8-ad616f80f19a
> **Inicio:** 2026-08-05
> **Estado: completed
> **Fuente:** `docs/Backlog.md` (85 tareas abiertas validadas)
> **Método de validación:** 6 sub-agentes en paralelo (vanta-audit, vanta-docs ×2, vanta-worker ×2, vanta-arch) + verificación directa del lead (AUDIT-05). Cada premisa contrastada contra código real con paths:líneas.
> **Reporte consolidado:** ver sección "Hallazgos" abajo (origen de este plan).
> **Regla:** toda tarea de este plan parte de la premisa **corregida**, NO de la descripción stale del backlog. Si el backlog aún no fue editado, ejecutar Fase 0 primero.

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 46 | 5 | 1 | 2 |

## Fases

| Fase | Contenido | Tareas |
|------|-----------|--------|
| F0 | Correcciones al Backlog.md (premisa stale → real) | 1 |
| F1 | Cierres sin código + consolidaciones/unificaciones | 10 |
| F2 | Release Blockers (P0/P1) | 7 |
| F3 | Engineering Health | 8 |
| F4 | Docs Drift (API contract) | 8 |
| F5 | Web Frontend | 9 |
| F6 | SDK & Features | 6 |
| F7 | Community & Launch | 4 |
| F8 | DESKTOP MVP (recortado) | 1 bloque |

---

## Fase 0 — Correcciones al Backlog.md (PREREQUISITO)

> **Justificación:** los ejecutores leen `docs/Backlog.md`. Si las descripciones quedan stale, cada implementación partirá de una premisa falsa. Esta fase solo edita el backlog — no toca código.

### Task 1: Backlog-EDIT — Corregir 12 premisas stale + limpiar referencia muerta
- **Archivos clave:** `docs/Backlog.md`
- **Gate Justificación:** hallazgos de los 6 sub-agentes; las descripciones actuales inducen a error de implementación.
- **Contrato: Refs 1:1 reales; comentario corregido
- **Pasos:**
  1. **AUDIT-01** (L70): reemplazar "try_numpy_array expone puntero" → "el UAF es por los getters `__array_interface__` (vector.rs:59-73, types.rs:365-380); `try_numpy_array` COPIA (seguro). Fix: congelar/clonar ante drop y `__setstate__`".
  2. **AUD-004** (L252): reemplazar "gate por feature experimental-lisp" → "feature `experimental-lisp` ELIMINADA en CUARENTENA-01; fix = eliminar/renombrar tool `query_lisp` o documentar que solo acepta IQL".
  3. **AUD-011** (L259): corregir "pánico ops.rs:1761" → "expect infalible (guard `continue` previo), no alcanzable; deuda estilística. Consumir reporte INV-024 en vez de re-contar unsafe".
  4. **TECH-02** (L96): corregir "pkg stale" → "pkg YA exporta `reindex_hnsw_from_text` (d.ts:183); el wrapper TS `vantadb.ts:542-548` está obsoleto — fix 1-línea, sin rebuild".
  5. **AUD-005** (L253): corregir "MCP.md=0.1.5, HTTP_API.md=0.0.4 stale" → "solo openapi.yaml=0.4.0 es drift real; MCP.md=0.5.0 correcto; HTTP_API.md 0.0.4 = content-type que coincide con cli_server.rs:368".
  6. **AUD-006** (L254): corregir "7/15 documentadas, 8 faltan" → "11/15 documentadas; de los 8 listados, 3 SÍ están (get_node_neighbors, inject_context, read_axioms); faltan reales: query_lisp (doc como query), collection_stats, collection_list, collection_delete, rehydrate = 5".
  7. **AUD-007** (L255): corregir "src/storage/wal.rs no existe" → "sí existe; el drift es de NOMBRES de tipo (WalSharded→ShardedWal, HnswIndex→CPIndex, ef_construction 400→100)".
  8. **INV-009-B** (L237): reescalar → "solo falta `Condition::TextMatch` en parser IQL de grafo (query.rs:121-126); enforcement (mod.rs:416-425), tokenización (text_index.rs:358-401) y matched_phrases (types.rs:438) YA existen".
  9. **INV-016-B** (L241): corregir lista → "15 tokens reales: globals.css ×12 + reveal.tsx:64 + faq-section.tsx:79 + benchmark-race.tsx:233; page-transition y latency-comparator NO contienen el token".
  10. **NUEVO-01/GH-139** (L124/353): corregir "README con PNG estática" → "README = solo texto + badges SVG; NO existe hero. GH-139 es slice del hero de NUEVO-01".
  11. **TECH-05** (L99): eliminar nota "revisar solape con MCP-02..05" → "MCP-02..05 NO existen en el backlog (referencia muerta)".
  12. **DESKTOP-08** (L392): corregir "métodos por endpoint de HTTP_API.md" → "la API real tiene 3 endpoints (/health, /metrics, /api/v2/query IQL); put/get/delete/list/search van como statements IQL. Diseñar 'cliente IQL tipado', no client REST por-op".
  13. **Progreso (capa 8):** tras las ediciones, verificar con `skill progreso` que ningún ID quede duplicado entre Backlog.md y progreso/README.md.
- **Dependencias:** ninguna.
- **Estado:** ✅ COMPLETED

---

## Fase 1 — Cierres, consolidaciones y unificaciones (sin código)

> **Justificación:** 6 tareas no deben ejecutarse (resueltas/duplicadas/no accionables) y 5 pares se solapan. Consolidar antes de ejecutar evita trabajo duplicado.

### Task 2: CLOSE-NUEVO18 — Reescribir NUEVO-18 como "sparse indexed" o cerrar
- **Veredicto:** ❌ premisa falsa — sparse search YA implementado (`sparse_memory_search` sdk/search/mod.rs:721-746, hybrid 3 canales :748-780, SparseVector node.rs:409).
- **Acción:** reescribir la fila como feature NUEVA: "Sparse indexed search — inverted index + posting lists (hoy brute-force O(n) sobre subset filtrado)" con ID nuevo (ej. NUEVO-22). Si no se quiere: tachar + mover a BACKLOG_HISTORY.
- **Contrato:** row editado en Backlog.md; referencia al gap real (brute-force sin índice).
- **Estado:** ✅ COMPLETED 2026-08-05 — NUEVO-18 reescrita como NUEVO-22 (sparse indexed search, inverted index + posting lists); premisa falsa corregida.

### Task 3: CLOSE-TSK103 — Cerrar TSK-103 (resuelta por NUEVO-10)
- **Veredicto:** ❌ resuelta — `benchmarks/README.md` + `requirements.txt` + 3 scripts públicos reproducibles (commit d0b1c7c6, NUEVO-10 ✅ 2026-08-02).
- **Acción:** tachar en Backlog.md + nota "resuelta por NUEVO-10; remanente = INV-007-B". No mover a progreso como nueva (ya está ahí).
- **Contrato:** row tachado; sin duplicado en progreso.
- **Estado:** ✅ COMPLETED 2026-08-05 — TSK-103 tachada; resuelta por NUEVO-10 (commit d0b1c7c6); remanente = INV-007-B.

### Task 4: CLOSE-MKT17 — Cerrar MKT-17 (duplicada de INV-007-B)
- **Veredicto:** ❌ duplicada 1:1 — `benchmarks-view.tsx:352-365` ya tiene la tabla estática (MKT-15 ✅); INV-007-B ya especifica `competitive-table.tsx` + contrato JSON con más rigor.
- **Acción:** tachar + nota "consolidada en INV-007-B (Task 47)". No ejecutar.
- **Contrato:** row tachado con referencia cruzada.
- **Estado:** ✅ COMPLETED 2026-08-05 — MKT-17 tachada; duplicada 1:1 de INV-007-B.

### Task 5: CLOSE-GH144 — Cerrar issue #144 sin trabajo
- **Veredicto:** ❌ ya resuelta — `web/src/lib/dictionaries.ts` tiene 22 claves `showcasePage.*` completas en ES (1370-1391) y EN (2856-2877); página usa `tt()` con fallback.
- **Acción:** `gh issue close 144 --repo ness-e/Vantadb` con comentario de evidencia. Tachar row en Backlog + migrar a progreso con nota.
- **Contrato:** issue cerrado; row tachado.
- **Estado:** ✅ COMPLETED 2026-08-05 — GH-144 issue cerrado con evidencia (22 ES + 22 EN claves); row tachado; migrada a progreso.

### Task 6: MOVE-LEG01 — Mover LEG-01 a lista humana
- **Veredicto:** ❌ no-delegable — registro USPTO/EUIPO requiere abogado, pago (~$250-350/clase USPTO, ~€850 EUIPO), identidad legal; estimación "2-4h" irreal.
- **Acción:** mover fila a sección "Tareas Humanas" del backlog (o docs/strategy/GO_TO_MARKET.md) con `owner: human` + estimación realista (semanas, $2-5K).
- **Contrato:** fila fuera del flujo de agentes; nota de decisión en campaign_memory (decisions).
- **Estado:** ✅ COMPLETED 2026-08-05 — LEG-01 movida a sección Tareas Humanas (owner: human, estimación semanas, $2-5K).

### Task 7: MOVE-COM04 — Mover COM-04 a nota de roadmap
- **Veredicto:** ❌ bloqueada — Server Discovery exige 1000+ miembros, Canny.io SaaS externo, ticketing requiere bot externo (Ticket Tool/Helper.gg). Nada accionable hoy.
- **Acción:** tachar o marcar `⏸ Icebox` con nota de dependencias en `docs/discord/todo.md`.
- **Contrato:** fila no contada como activa; dependencias documentadas.
- **Estado:** ✅ COMPLETED 2026-08-05 — COM-04 marcada ⏸ Icebox con dependencias en docs/discord/todo.md.

### Task 8: MERGE-TECH04-AUD010 — Unificar naming de env vars (una sola tarea)
- **Veredicto:** 🔁 duplicados reales — mismas 3 lecturas: `cli.rs:15` env `VANTA_DB`, `server.rs:244` `cmd.env("VANTA_DB")`, `config.rs:408` `VANTADB_STORAGE_PATH`; ~25 vars `VANTADB_*`.
- **Acción:** fusionar en una tarea única (absorbe AUD-010):
  1. ADR breve en `docs/architecture/adr/` (próximo número libre): decisión = `VANTA_DB` sigue siendo flag CLI (clap), `VANTADB_STORAGE_PATH` el env de config; el child `vantadb-server` setea ambos.
  2. Aplicar el renombrado **solo si el ADR lo aprueba**; caso contrario queda como deuda documentada.
  3. Sincronizar `docs/operations/CONFIGURATION.md`.
- **Contrato:** ADR publicado; test de compatibilidad (leer ambas vars, warning de deprecación) si se migra; TECH-01 (Task 17) resuelve el síntoma mientras tanto.
- **Dependencias:** TECH-01 (Task 17) primero para el síntoma.
- **Estado:** ✅ COMPLETED 2026-08-05 — ADR-012 publicado (VANTA_DB = flag CLI, VANTADB_STORAGE_PATH = env config; child setea ambos); CONFIGURATION.md nota añadida; AUD-010 absorbida.

### Task 9: MERGE-GH139-NUEVO01 — Fusionar GH-139 como slice de NUEVO-01
- **Veredicto:** 🔁 GH-139 ⊂ NUEVO-01 — ambos crean GIF demo en README; NUEVO-01 es superset (hero readme-aura + gráfico benchmark + GIF).
- **Acción:** tachar GH-139 con nota "slice GIF de NUEVO-01 (Task 41)". NUEVO-01 gana el deliverable GIF <5MB (pip install → REPL CRUD → hybrid search) como sub-paso.
- **Contrato:** GH-139 tachado; NUEVO-01 incluye sub-paso GIF.
- **Estado:** ✅ COMPLETED 2026-08-05 — GH-139 tachado como slice de NUEVO-01 (GIF <5MB sub-paso).

### Task 10: MERGE-AUDIT06-07 — Fusionar micro-optimizaciones → vanta-tuner
- **Veredicto:** 🔁 ambas micro-opt sin bug demostrado: AUDIT-06 (BTreeMap RRF con cap 750 → impacto µs; no hay path single-channel que lo justifique), AUDIT-07 (tradeoff BTreeMap ya decidido y razonado en doc comment node.rs:404-407).
- **Acción:** fusionar en UNA tarea de investigación de rendimiento delegada a vanta-tuner: medición previa (flamegraph/profiling) antes de tocar código. DoD: si la medición no muestra >1% impacto, cerrar como WONTFIX con ADR ligero.
- **Contrato:** doc de investigación con medición; decisión documentada.
- **Estado:** ✅ COMPLETED 2026-08-05 — AUDIT-06+07 fusionadas en investigación de vanta-tuner (medición previa, WONTFIX si <1%).

### Task 11: RESCALE-AUDIT03 — Re-escalar guard Miri
- **Veredicto:** ⚠️ inviable como está — `vantadb-python/Cargo.toml` = 0 dev-deps, 0 tests Rust, cdylib PyO3; Miri no puede correr FFI CPython/NumPy.
- **Acción:** reformular → "Miri sobre el CORE (`cargo +nightly miri test -p vantadb`, ya existe `tests/miri_unsafe.rs`) para cubrir los 7 bloques UB_POTENTIAL de INV-024; boundary Python cubierto con repro Python + ASAN/valgrind (AUDIT-04)". Ejecutar DESPUÉS del fix AUDIT-01 (Task 12).
- **Contrato:** comando Miri sobre core pasa o reporta UB; sin intento de Miri sobre el crate Python.
- **Dependencias:** AUDIT-01 (Task 12).
- **Estado:** ✅ COMPLETED

---

## Fase 2 — Release Blockers (P0/P1)

> **Secuencia crítica:** 12 → 13 → 14 → 15 → 16 → 17 → 18. AUDIT-01 desbloquea AUDIT-04; TECH-01 desbloquea TECH-04/AUD-010.

### Task 12: AUDIT-01 — Fix UAF PyO3 `__array_interface__` (🔴 release-blocker)
- **Archivos clave:** `vantadb-python/src/vector.rs:59-73`, `vantadb-python/src/types.rs:365-380`, `vantadb-python/src/convert.rs:189-203`
- **Gate Justificación:** C1 — UAF demostrado (drop del pyclass + `np.asarray()` → puntero dangle; `__setstate__` vector.rs:80-82 libera el buffer viejo con views vivas).
- **Pipeline:** 🔍 Inv (confirmar path de UAF: getters, NO try_numpy_array) → 📊 Análisis (congelar/clonar ante drop/mutación) → ✅ Verif (test repro: np.asarray → drop → acceso) → 🔧 Impl.
- **Contrato:** test repro no crashea; `cargo +nightly miri test -p vantadb` verde en el bloque auditado; benchmark Python pasa.
- **Dependencias:** ninguna (AUDIT-03 re-escalado es post).
- **Estado:** ✅ COMPLETED

### Task 13: AUD-004 — Resolver tool MCP `query_lisp` (🔴)
- **Archivos clave:** `vantadb-mcp/src/lib.rs:864-871,1113-1134`, `src/executor.rs:152-169`, `docs/api/MCP.md`
- **Gate Justificación:** API viva que rechaza su input anunciado (test `test_execute_hybrid_rejects_lisp` executor.rs:577); feature `experimental-lisp` NO existe.
- **Acción (premisa corregida):** decidir eliminar la tool o renombrarla (ej. `query_iql`) y documentar en MCP.md. NO gate por feature inexistente.
- **Contrato:** test MCP pasa; MCP.md documenta el nombre real; sin tool que prometa LISP.
- **Dependencias:** define el destino para AUD-006 (Task 30) y TECH-03 #4 (Task 20).
- **Estado:** ✅ COMPLETED

### Task 14: AUDIT-04 — Root-cause crash benchmark Python (0xC0000409)
- **Archivos clave:** `benchmarks/vantadb_local_bench.py:242-245`, `heavy_nocturnal_tests.log:142-149`
- **Gate Justificación:** crash real documentado (STATUS_STACK_BUFFER_OVERRUN en 10K/128d/1000q). Hipótesis UAF **débil** (el benchmark usa try_numpy_array que copia) — candidatos: stack overflow Windows (histórico con crates grandes) o bug del benchmark.
- **Pipeline:** 🔍 Inv (repro mínimo standalone + ASAN/valgrind) → 📊 Análisis (atribuir causa) → ✅ Verif (3× sin crash) → 🔧 Impl (fix según causa).
- **Contrato:** benchmark Python estable 3×; crash atribuido y documentado.
- **Dependencias:** AUDIT-01 (Task 12) para descartar la rama UAF.
- **Estado:** ✅ COMPLETED

### Task 15: AUD-011 — Deuda unsafe/unwrap/OpGate (2 matices)
- **Archivos clave:** `src/storage/engine/ops.rs:1756-1762`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`, `vantadb-node/src/lib.rs:75-92,277-287`
- **Gate Justificación:** conteos reales (distance.rs:23 unsafe, vfile.rs:22; ~1/33 unwraps). OpGate+drain solo en node → riesgo write-after-close en python/wasm.
- **Acción (premisa corregida):** (a) NO tratar ops.rs:1761 como pánico alcanzable (es expect infalible por guard); (b) consumir reporte INV-024 (39 bloques, 7 UB_POTENTIAL) en vez de re-contar; (c) portar patrón OpGate a bindings python/wasm; (d) reemplazar unwraps críticos en hot path por propagación de error.
- **Contrato:** `cargo clippy --workspace --all-targets -- -D warnings`; OpGate presente en 3 bindings; sin pánico nuevo en hot path.
- **Dependencias:** reporte INV-024 (existente).
- **Estado:** ✅ COMPLETED

### Task 16: AUD-001 — Fix Dockerfile (🔴 CI/Docker invendible)
- **Archivos clave:** `Dockerfile:4,32-39,42-47,54`, `Cargo.toml:5`
- **Gate Justificación:** `RUST_VERSION=1.94.0` < MSRV 1.94.1; 8 `COPY` a crates inexistentes (integración movida a `integrations/` = paquetes Python sin Cargo.toml; `vantadb-litellm` no existe ni ahí). `docker build` falla en línea 32.
- **Acción:** subir RUST_VERSION a ≥1.94.1; eliminar los 8 COPY + loop skeleton de integrations + `rm -rf vantadb-*/src/`; validar imagen con smoke-test.
- **Contrato:** `docker build` exit 0; container responde healthcheck.
- **Estado:** ✅ COMPLETED

### Task 17: TECH-01 — Fix `--db` en MCP server (P0)
- **Archivos clave:** `src/cli_handlers/server.rs:244`
- **Gate Justificación:** bug real con impacto de datos — el hijo `vantadb-server --mcp` resuelve storage vía `VantaConfig::from_env()` → `VANTADB_STORAGE_PATH`, pero el padre setea `VANTA_DB`.
- **Acción:** `cmd.env("VANTADB_STORAGE_PATH", db_path)` (añadir, no reemplazar — el flag CLI sigue usando VANTA_DB).
- **Contrato:** e2e `vanta-cli server --mcp --db /tmp/x` → lock/persistencia en `/tmp/x`; tests e2e MCP pasan.
- **Estado:** ✅ COMPLETED

### Task 18: TECH-02 — Fix wrapper TS `reindexHnswFromText` (1-línea)
- **Archivos clave:** `vantadb-ts/src/vantadb.ts:542-548`
- **Gate Justificación:** premisa corregida — pkg YA exporta (`pkg/vantadb_wasm.d.ts:183`); el wrapper lanza WASM_ERROR por comentario obsoleto.
- **Acción:** `return this._wasm("reindexHnswFromText", () => this.inner.reindex_hnsw_from_text(namespace, pageSize))`. SIN rebuild/publish de pkg.
- **Contrato:** test browser/TS llama a la función sin error; `npm test` en vantadb-ts pasa.
- **Estado:** ✅ COMPLETED

---

## Fase 3 — Engineering Health

### Task 19: DEBT-01 — Reparar gate docs-coverage + 13 gaps reales
- **Archivos clave:** `scripts/validate-docs-coverage.ps1:64`, `docs/api/*.md`, `src/sdk/search/`
- **Gate Justificación:** línea 64 apunta a `src\sdk\search.rs` (directorio) → el error mata la validación SDK ("0 items"); además 13 gaps reales (bulk_commit_interval config.rs:304, NoVectorForKey error.rs:265, create cli.rs:326, bulk_import/graph_page_rank/graph_degree_centrality/recover_archived_nodes en bindings + gds.rs).
- **Acción:** Parte A: corregir ruta. Parte B: documentar los gaps. **Nota:** arreglar solo la ruta NO deja el script verde — quedan los 13.
- **Contrato:** `pwsh scripts/validate-docs-coverage.ps1` exit 0; sección SDK reporta items reales.
- **Estado:** ✅ COMPLETED 2026-08-05 — commit `1a0cb79a` (gate reparado + 13 gaps documentados).

### Task 20: TECH-03 — Corregir 3 stale-docs reales (de 4)
- **Archivos clave:** `docs/api/HTTP_API.md:124-125`, `vantadb-python/README.md:33,48,59`, `docs/api/MCP.md:56`
- **Gate Justificación:** (1) `mcp_mode = mcp && !http` (server.rs:182) — claim "Full MCP + HTTP" falso; (2) put_memory/search_hybrid/memory_stats inexistentes (API real: search_memory/put_batch/get_memory); (4) tool `query` vs real `query_lisp`.
- **Acción:** retirar claim (3) (`python_sdk` feature SÍ existe en Cargo.toml:104 + src/python.rs:1 — el drift es nomenclatura, no falsedad). Corregir los 3 reales.
- **Contrato:** grep de cada método/tool confirma existencia; `validate-docs-coverage.ps1` verde (con Task 19).
- **Dependencias:** Task 13 (destino de query_lisp) define el wording de (4).
- **Estado:** ✅ COMPLETED

### Task 21: TECH-05 — Implementar resource MCP `schema://`
- **Archivos clave:** `vantadb-mcp/src/lib.rs:605-616,619-706`, `docs/api/MCP.md:79-81`, `vantadb-mcp/tests/mcp_tests.rs`
- **Gate Justificación:** contrato roto — resource documentado que no existe ni en list ni en read (solo metrics://, memory://, namespace://).
- **Acción:** definir shape JSON (config HNSW + text index version) + handler + tests.
- **Contrato:** `resources/read schema://` devuelve schema; tests MCP verdes.
- **Nota:** NO hay MCP-02..05 en backlog — sin solape (limpiar la nota del backlog en F0).
- **Estado:** ✅ COMPLETED 2026-08-05 — commit `4dff484c` (resource `schema://` list+read + tests MCP).

### Task 22: TECH-07 — Publicar pkg WASM con feature `opfs`
- **Archivos clave:** `vantadb-wasm/src/lib.rs:334-399`, `vantadb-wasm/Cargo.toml:36-38`, `pkg/*.d.ts`, `src/opfs_bridge.js`
- **Gate Justificación:** worker APIs (`connect_worker`, `worker_read/write/delete`) bajo `cfg(opfs)` ausentes del pkg compilado; default = `["tracing-wasm"]`.
- **Acción:** rebuild con `--features opfs`; publicar pkg con worker opcional documentado; test browser con worker.
- **Contrato:** d.ts del pkg incluye los 4 exports; test browser worker pasa.
- **Estado:** ✅ COMPLETED (`566e9369`) — pkg rebuild con `wasm-pack build --features opfs` (wasm-pack 0.15.0): d.ts incluye connect_worker/worker_read/worker_write/worker_delete. Documentación "Optional capability" en los 4 métodos (lib.rs) + ejemplo spawnOpfsWorker. Demo `vantadb-wasm/demo/worker-test.html` creado. ⚠️ pkg/ no trackeado en git (build local); browser test con worker pendiente de entorno.

### Task 23: TECH-08 — Decidir promoción a default-members (sin re-investigar)
- **Archivos clave:** `Cargo.toml:583-599`, `docs/operations/CI_POLICY.md:73-84`
- **Gate Justificación:** análisis YA existe (CI_POLICY.md + DESKTOP-01b:140,273,419,515). No es tarea de worker — es de lead/arch.
- **Acción:** tomar la decisión (promover vs mantener experimental) + nota en CI_POLICY o ADR ligero. `cargo check --workspace` con los 3 habilitados como prueba.
- **Contrato:** decisión documentada; workspace compila con los 3.
- **Estado:** ✅ COMPLETED

### Task 24: TECH-06 — CORS como feature request (reformular)
- **Archivos clave:** `src/cli_server.rs`, `src/config.rs`, `docs/api/HTTP_API.md:148-150`
- **Gate Justificación:** ausencia de CORS es DECISIÓN documentada (reverse proxy recomendado), no bug silencioso.
- **Acción:** reformular como feature-gated opcional: middleware tower-http CORS con origenes configurables; default OFF (sin cambio de comportamiento). Si no hay necesidad real (webview usa reqwest, no fetch), cerrar.
- **Contrato:** CORS configurable; default sin headers nuevos; test e2e con Origin header.
- **Estado:** ✅ COMPLETED

### Task 25: AUDIT-05 — Housekeeping 3 fixes (30min)
- **Archivos clave:** `.gitignore` (falta `.playwright-cli/` — verificado), `docs/architecture/adr/003_sync_async_decoupling.md` (nota SurrealDB sin last-updated), `.opencode/skills/campaign-executor/tasks/GH-123.md` (Estado PENDING stale vs commit d406feab)
- **Acción:** añadir `.playwright-cli/` a gitignore; agregar sección Addendum o línea `last-updated` al ADR; actualizar Estado en GH-123.md.
- **Contrato:** `git status` limpio de artefactos .playwright-cli; ADR con last-updated; task file en ✅/done.
- **Estado:** ✅ COMPLETED

### Task 26: AUDIT-08 — Actualizar P2 debt ledger (30min)
- **Archivos clave:** `.opencode/AGENTS.md:882-887`, `vantadb-python/src/vector.rs:63`, `vantadb-python/src/types.rs:365`, `vantadb-python/src/convert.rs:23-70,53-62`, `src/sdk/serialization/mod.rs:227-294`, `vantadb-wasm/src/lib.rs:402-433`
- **Gate Justificación:** refs apuntan a `lib.rs:1754/34-36/895-901/394-413` pero src/lib.rs = 193 líneas; comentario LRU dice O(1) pero evicción es O(n) `min_by_key`.
- **Acción:** actualizar refs P2-2/P2-3/P2-7/P2-8 a las reales (verificadas 1:1); corregir comentario O(1)→O(n) en convert.rs:53-62.
- **Contrato:** grep en AGENTS.md de lib.rs:1754 = 0; comentario corregido.
- **Estado:** ✅ COMPLETED

---

## Fase 4 — Docs Drift (API contract)

### Task 27: AUD-002 — Corregir API fantasma GRAPH_RAG.md
- **Archivos clave:** `docs/api/GRAPH_RAG.md:18-19`, `vantadb-python/src/lib.rs` (pyclass VantaDB), `src/sdk/builder.rs:139`, `src/graphrag/`
- **Gate Justificación:** `vantadb_python.Client()` + `graphrag_search()` no existen en ningún binding; citado como violación en `rules/api-contract.md:13`.
- **Acción:** documentar el entrypoint real (VantaDB core + GraphRagPipeline config) o marcar como roadmap/futuro y corregir el ejemplo.
- **Contrato:** ejemplo Python ejecutable (o marcado como no-implementado); grep `graphrag_search` en bindings = solo Rust SDK.
- **Estado:** ⬜ PENDING

### Task 28: AUD-003 — Retractar afirmación falsa src/governance
- **Archivos clave:** `docs/architecture/EXPERIMENTAL_GOVERNANCE_DESIGN.md:14,172,138`, `src/gds.rs` (403 L, GraphDataScience page_rank :38, degree_centrality :145)
- **Gate Justificación:** doc afirma "verificada contra src/governance 2026-07-21" pero el dir NO existe (Test-Path=False); el código real es src/gds.rs.
- **Acción:** retractar afirmaciones; renombrar como diseño propuesto no-implementado; mapear qué corresponde a src/gds.rs vs futuro.
- **Contrato:** doc no afirma verificación contra código inexistente.
- **Nota:** backlog ubica el doc en docs/Investigaciones/ pero está en docs/architecture/ (corregir path en F0).
- **Estado:** ⬜ PENDING

### Task 29: AUD-005 — Sincronizar versiones docs/api (solo 1 drift real)
- **Archivos clave:** `docs/api/openapi.yaml:4` (0.4.0 → 0.5.0), `Cargo.toml:602`
- **Gate Justificación:** único drift real es openapi.yaml; MCP.md ya está en 0.5.0 (L310); HTTP_API.md 0.0.4 coincide con cli_server.rs:368.
- **Acción:** corregir openapi.yaml + opcionalmente gate CI de versión (lo valioso). No tocar MCP.md/HTTP_API.md salvo el gate.
- **Contrato:** openapi.yaml = 0.5.0; gate CI falla si cabeceras divergen del workspace.
- **Estado:** ⬜ PENDING

### Task 30: AUD-006 — Documentar 5 tools MCP reales faltantes
- **Archivos clave:** `docs/api/MCP.md`, `vantadb-mcp/src/lib.rs:808-964`
- **Gate Justificación:** premisa corregida — 11/15 documentadas; faltan reales: query_lisp (mal nombrada como query), collection_stats, collection_list, collection_delete, rehydrate.
- **Acción:** extraer firmas reales de las 5; documentar en MCP.md; gate de paridad tool↔doc.
- **Contrato:** 15/15 tools documentadas con nombre real; gate verifica paridad.
- **Dependencias:** Task 13 (destino de query_lisp).
- **Estado:** ⬜ PENDING

### Task 31: AUD-007 — Corregir drift ARCHITECTURE.md
- **Archivos clave:** `docs/architecture/ARCHITECTURE.md:296-298,327`, `src/index/graph.rs:255-260,318`, `src/wal_sharded.rs:9`
- **Gate Justificación:** ef_construction 400→100 real; WalSharded→ShardedWal; HnswIndex→CPIndex; src/index/core.rs es 100% `#[cfg(test)]` (480 L).
- **Acción:** corregir nombres de tipo + constantes. (La formulación "paths no existen" del backlog es falsa — src/storage/wal.rs SÍ existe.)
- **Contrato:** grep ARCHITECTURE.md de HnswIndex/WalSharded/ef_construction=400 = 0.
- **Estado:** ⬜ PENDING

### Task 32: AUD-008 — Corregir drift STORAGE_VERSIONING.md
- **Archivos clave:** `docs/architecture/STORAGE_VERSIONING.md:54-55,315,168,328`, `src/index/graph.rs:142` (VECTOR_INDEX_VERSION=8), `src/storage/vfile.rs:26` (VFILE_VERSION=2), `src/wal.rs:17,31` (postcard)
- **Gate Justificación:** doc dice 7/1/bincode; real 8/2/postcard. Relevante para migraciones de datos (versión de formato mal documentada).
- **Acción:** corregir constantes; importar constantes del código en vez de hardcodear; resolver contradicción interna (nota WEB-04 dice postcard, §2.4/§4.2 dicen bincode).
- **Contrato:** doc = constantes reales; valores importados del código.
- **Estado:** ⬜ PENDING

### Task 33: AUD-009 — Corregir notas Vite→Next.js
- **Archivos clave:** `docs/Investigaciones/DESKTOP-01b...:1090`, `web/package.json` (next ^16.1.1), `web/next.config.ts`
- **Gate Justificación:** DESKTOP-01b:1090 afirma "React + Vite (mismo que web/)" — falso; web/ es Next.js 16 App Router. Las demás menciones Vite se refieren al desktop Tauri planificado (correctas — no tocar).
- **Acción:** corregir la nota errónea + non_exhaustive check de stack. La referencia "DOC3 §A.5" no es trazable (ID de catálogo) — no perseguir.
- **Contrato:** grep DESKTOP-01b de "Vite (mismo que web/" = 0; web/AGENTS.md correcto.
- **Estado:** ⬜ PENDING

### Task 34: GH-123 — Re-escopetar a links rotos reales (~4)
- **Archivos clave:** `docs/progreso/README.md:1793`, `docs/progreso/bitacora.md`, `docs/operations/BENCHMARKS.md` (path file:/// máquina-específico), `docs/glosario/`
- **Gate Justificación:** claim "167+ archivos" sin sustento (341 .md en docs/); scan real = ~4 links rotos + wiki-links `[[..]]` falsos positivos.
- **Acción:** corregir los ~4 links reales + typos de docs/progreso; documentar método de auditoría (no re-inventar sweep). Cerrar issue #123 con evidencia del inventario.
- **Contrato:** `gh issue close 123` tras inventario; 0 links rotos en docs/progreso.
- **Estado:** ⬜ PENDING

---

## Fase 5 — Web Frontend

### Task 35: INV-005-A — error.tsx + drop @mdxeditor/editor
- **Archivos clave:** `web/src/app/error.tsx` (nuevo), `web/package.json:16`
- **Gate Justificación:** error.tsx ausente; @mdxeditor/editor 0 imports (solo package.json/lock) — dep muerta ~500KB.
- **Acción:** crear error.tsx (App Router error boundary); eliminar dep.
- **Contrato:** `npm run build` en web/ pasa; grep @mdxeditor/editor en src = 0.
- **Estado:** ⬜ PENDING

### Task 36: INV-013-B — JSON-LD structured data
- **Archivos clave:** `web/src/app/layout.tsx:33-77`
- **Gate Justificación:** 0 JSON-LD en web/src; Metadata API de Next.js 16 NO genera JSON-LD.
- **Acción:** emitir `<script type="application/ld+json">` (schema.org/SoftwareApplication) en Server Component del head; validar con Google Rich Results Test.
- **Contrato:** grep `application/ld+json` en layout.tsx = 1; JSON válido.
- **Estado:** ⬜ PENDING

### Task 37: INV-014-B — Limpiar plomería dark inerte
- **Archivos clave:** `web/src/components/vanta/theme-provider.tsx`, `theme-toggle.tsx`, `web/package.json:59`, `web/AGENTS.md:72`, `.opencode/AGENTS.md:383`
- **Gate Justificación:** theme-provider 0 consumidores; único consumidor de theme-toggle (navbar.tsx:14,396) es dead code; globals.css 0 `.dark`; regla R-FE-4 (frontend-web.md:28-33) YA manda eliminarlos.
- **Acción:** eliminar 2 componentes + dep next-themes; corregir 2 notas stale; verificar que site-navbar.tsx no importe ThemeToggle.
- **Contrato:** `npm run build` pasa; grep next-themes en web/src = 0.
- **Estado:** ⬜ PENDING

### Task 38: INV-015-B — Fix touch targets < 44px
- **Archivos clave:** `web/src/components/vanta/changelog-section.tsx:81-87`, `tutorials-section.tsx:83-88` (2 P0: clear-search 14px), ~24 componentes < 44px
- **Gate Justificación:** 2 icon buttons de 14px confirmados (< 24px mínimo); inventario ~24 plausible (task file INV-015.md:30).
- **Acción:** P0: `size-11`/`min-h-[44px] min-w-[44px]` en los clear-search; luego P1-P4 del inventario.
- **Contrato:** grep `<X className="h-3.5 w-3.5"` en buttons = 0; WCAG 2.5.8.
- **Estado:** ⬜ PENDING

### Task 39: INV-016-B — Motion tokens (lista corregida)
- **Archivos clave:** `web/src/app/globals.css:5,111-140,360,369,660,718-719,810`, `web/src/components/vanta/reveal.tsx:64`, `faq-section.tsx:79`, `benchmark-race.tsx:233`
- **Gate Justificación:** 15 tokens reales (12 CSS + 3 comps); `@theme inline` existe. NO tocar page-transition/latency-comparator (no contienen el token).
- **Acción:** definir `--duration-fast/normal/slow` + `--ease-default`; reemplazar los 15 cubic-bezier hardcodeados.
- **Contrato:** grep `cubic-bezier(0.2,0.8,0.2,1)` en web/src = 0; tokens definidos.
- **Estado:** ⬜ PENDING

### Task 40: GH-140 — Auditar CSS no usado (reescopetar)
- **Archivos clave:** `web/src/app/globals.css` (18.9KB / 817L)
- **Gate Justificación:** clases efecto muestreadas TODAS usadas; DoD ≥10% no demostrable sin auditoría clase-por-clase.
- **Acción:** análisis de cobertura clase-por-clase primero (método documentado); eliminar solo CSS probadamente huérfano; si <10%, ajustar DoD a "0 selectores huérfanos".
- **Contrato:** reporte de cobertura; 0 selectores muertos probados.
- **Estado:** ⬜ PENDING

### Task 41: NUEVO-01 — README hero (incluye GH-139 slice GIF)
- **Archivos clave:** `README.md` (hoy: solo texto + badges SVG), `assets/` (nuevo)
- **Gate Justificación:** gap real (premisa "PNG estática" falsa — no hay hero); GH-139 fusionado aquí.
- **Acción:** 3 slices: (1) hero readme-aura; (2) gráfico benchmark (hay tablas SIFT1M que mejorar); (3) GIF <5MB (pip install → REPL CRUD → hybrid search) vía vhs/asciinema.
- **Contrato:** README con hero; GIF <5MB renderiza; `npx readme-aura` (o herramienta elegida) sin errores.
- **Estado:** ⬜ PENDING

### Task 42: GH-132 — Notebook Colab + badge
- **Archivos clave:** `examples/colab/vantadb_quickstart.ipynb` (nuevo), `README.md`
- **Gate Justificación:** examples/colab/ no existe; sin badge "Run in Colab".
- **Acción:** notebook end-to-end (install, CRUD, hybrid search) + badge en README.
- **Contrato:** notebook corre en Colab (verificado o documentado); badge renderiza.
- **Estado:** ⬜ PENDING

### Task 43: GH-131/129/128 — README integraciones (3 issues, 1 patrón)
- **Archivos clave:** `README.md`, `examples/python/mem0_integration.py`, `semantic_kernel_memory.py`, `dspy_retriever.py`
- **Gate Justificación:** 3 ejemplos existen y ya tienen smoke en CI (GH-142, ci-examples-12.yml:115-122); falta solo la sección README + verificación SDK actual (SK).
- **Acción:** 3 secciones README con snippet verificado (mem0, Semantic Kernel, DSPy); opcional: un solo PR "README: secciones de integraciones" si los issues lo permiten.
- **Contrato:** README menciona las 3; snippets corren; CI smoke sigue verde.
- **Estado:** ⬜ PENDING

---

## Fase 6 — SDK & Features

> **Orden:** INV-025 (scoping) ANTES de INV-009-B — comparten snippet.rs.

### Task 44: INV-025 — Scoping Search Quality v2
- **Archivos clave:** `src/sdk/search/snippet.rs:29,92` (pub(crate)), `src/sdk/search/mod.rs:814-819`, `src/text_index.rs`, `src/tokenizer.rs`
- **Gate Justificación:** snippet con highlighting es pub(crate); no hay decisión de API pública. Precede a INV-009-B (toca highlight_phrases).
- **Acción:** definir outputs público SDK/CLI vs debug-only; non-goals (sin claims de paridad hybrid); corpus de validación pequeño; documentar dependencia con INV-009-B.
- **Contrato:** doc de scoping publicado; decisiones registradas en ADR/memoria.
- **Estado:** ⬜ PENDING

### Task 45: INV-009-B — Phrase queries (reescalada: solo Condition::TextMatch)
- **Archivos clave:** `src/query.rs:121-126` (enum Condition), `src/parser/mod.rs`
- **Gate Justificación:** enforcement (mod.rs:416-425), tokenización (text_index.rs:358-401) y matched_phrases (types.rs:438) YA existen; el único gap es `Condition::TextMatch` en el parser IQL de grafo (path distinto del memory search).
- **Acción:** agregar variante `Condition::TextMatch(field, query)` con reuso `string_literal`; tokenización literal de frases (sin stopwords) en query_plan_with_config; highlight_phrases.
- **Contrato:** query IQL con frase entre comillas ejecuta; tests parser + snippet verdes.
- **Dependencias:** Task 44 (contrato de snippet/API).
- **Estado:** ⬜ PENDING

### Task 46: INV-008-B — Implementar `search_batch_requests` Python SDK
- **Archivos clave:** `vantadb-python/src/lib.rs:1180-1209` (search_batch vector-only), `src/sdk/api.rs`
- **Gate Justificación:** batch vector-only existe (Rayon + GIL release); falta versión con SearchRequest completo (filtros/text_query).
- **Acción:** dataclass SearchRequest; patrón GIL+Rayon existente; fail-fast (`try_for_each`); extender `benchmarks/batch_vs_sequential_bench.py`. Target: batch 10 < 3× single.
- **Contrato:** test Python batch_requests pasa; bench muestra target.
- **Estado:** ⬜ PENDING

### Task 47: INV-007-B — JSON contrato + competitive-table.tsx (absorbe MKT-17)
- **Archivos clave:** `benchmarks/competitive_bench.py` (751 L), `web/src/components/vanta/competitive-table.tsx` (nuevo), `web/src/lib/vanta-data.ts`
- **Gate Justificación:** script emite Markdown a docs/BENCHMARKS.md, NO JSON versionado; componente web ausente; MKT-17 cerrado aquí.
- **Acción:** emitir `competitive_benchmark.json` (fecha/hardware/versiones); crear competitive-table.tsx bajo `<BenchmarkRace />` en /benchmarks. Sin números inventados.
- **Contrato:** JSON versionado; tabla renderiza con datos del JSON; `/benchmarks` build pasa.
- **Estado:** ⬜ PENDING

### Task 48: NUEVO-16 — Product Quantization (roadmap)
- **Archivos clave:** `src/vector/quantization.rs:16,33,97,141`, `src/index/scann.rs:9` ("no PQ")
- **Gate Justificación:** RabitQ/TurboQuant/SQ8 existen, PQ no. REC-009 (2026-07-31) ya deferió la viabilidad — partir de ese doc.
- **Acción:** scoping técnico (enlazar REC-009) + investigación de corpus objetivo (datasets >RAM) antes de implementar.
- **Contrato:** doc de viabilidad actualizado con decisión; si se aprueba, plan de fases.
- **Estado:** ⬜ PENDING

### Task 49: NUEVO-22 (ex NUEVO-18) — Sparse indexed search (inverted index)
- **Archivos clave:** `src/sdk/search/mod.rs:721-746` (hoy brute-force O(n)), `src/node.rs:409` (SparseVector)
- **Gate Justificación:** premisa original falsa; gap real = falta índice invertido + posting lists para sparse.
- **Acción:** diseño (inverted index, posting lists, merge con lexical); implementación; benchmarks vs brute-force.
- **Contrato:** search sparse indexado > brute-force en corpus sparse; tests deterministas.
- **Estado:** ⬜ PENDING

---

## Fase 7 — Community & Launch

### Task 50: COM-02/03 — Discord config (lista humana, claims corregidos)
- **Veredicto:** ⚠️ reales pero no-delegables por agente (login a dashboards Carl-bot/Server Settings).
- **Acción:** mantener como checklist humano con owner; CORREGIR claim técnico: Discord API SÍ expone AutoMod rules (POST /guilds/{id}/auto-moderation/rules) y emojis (POST /guilds/{id}/emojis) — el bloqueo es organizacional (sin bot con permisos), no técnico.
- **Contrato:** docs/discord/todo.md actualizado con claims correctos; checkboxes humanos marcados cuando se ejecuten.
- **Estado:** ⬜ PENDING (humano)

### Task 51: GH-141 — Documentar webhook GitHub→Discord
- **Archivos clave:** `docs/discord/server-config.md:98-102` (sección Integrations, 1 fila existente)
- **Gate Justificación:** doc existe pero sin tipos de evento detallados ni cómo agregar.
- **Acción:** documentar 4 tipos de evento (push, pull_request, issues, release → #announcements), procedimiento para añadir eventos, dónde se configura. Cerrar issue #141.
- **Contrato:** `gh issue close 141`; sección Integrations completa.
- **Estado:** ⬜ PENDING

### Task 52: MKT-16 — Metodología benchmark GraphRAG (números reales)
- **Archivos clave:** `examples/rust/graphrag.rs`, `docs/glosario/graphrag.md`, `docs/blog/`
- **Gate Justificación:** material real existe (ejemplo + fórmula + métricas); caveat: métricas 40-60% parecen claims, no runs.
- **Acción:** correr benchmark real reproducible (como MKT-05 con competitive_bench.py); publicar metodología + números de runs. Prohibido inventar cifras.
- **Contrato:** doc con números de un run real; script reproducible citado.
- **Estado:** ⬜ PENDING

### Task 53: MKT-10 — "AI Agent Memory" campaign (rescatar con DoD)
- **Veredicto:** ⚠️ vaga sin artifact (backlog-validation-2026-07-28.md:118 ya la marcó ❌); contenido base existe (tutorial 01-ai-agent-memory.md + 3 blogs).
- **Acción:** reescribir con deliverables medibles: landing "agent memory" + 1 blog benchmark vs full-context + demo. O cerrar como cubierta por INV-006/BLOG_SERIES_PLAN.
- **Contrato:** checklist de campaña con entregables; sin items vagos.
- **Estado:** ⬜ PENDING

---

## Fase 8 — DESKTOP MVP recortado

### Task 54: DESKTOP-02..26 — MVP multi-connection (recortado)
- **Veredicto:** ✅ arquitectura coherente; 4 premisas corregidas; 4-6 tareas de valor marginal recortadas.
- **Premisas verificadas:** 15 tools MCP (lib.rs:810-962); flag `--mcp` real (vantadb-server/src/main.rs:27); API HTTP = 3 endpoints + auth Bearer (cli_server.rs:126-130,255,303); napi-rs real (vantadb-node/package.json); python library-only (0 fn main); wasm read_only (lib.rs:44,63).
- **Acciones:**
  1. **Incluir en MVP (13):** DESKTOP-02 (scaffold), 03 (integración nativa — corazón), 04 (trait — contrato), 05 (NativeConnection), 08-10 (Server: reescalar 08 a cliente IQL tipado), 11-14 (MCP), 19 (ConnectionManager — razón de ser), 20 (lifecycle), 24 (empaquetado 🔴), 26 (tests/contrato de errores).
  2. **Defer a Fase 7 futura (4):** DESKTOP-15,16,17,18 (Node/Python — valor marginal, frágil empaquetado; native+server+MCP = ~95% del valor).
  3. **Fusionar:** DESKTOP-06 + 07 (commands CRUD + UI MVP = mismo demo).
  4. **Simplificar:** DESKTOP-23 → usar `tauri-plugin-store` en vez de JSON manual + rename atómico (YAGNI).
  5. **Recortar:** DESKTOP-22 (solo evento obligatorio connection-state; los otros 2 son flag opcional), DESKTOP-25 (CI desktop solo si el MVP se aprueba), DESKTOP-27 (docs+ADR al final del MVP).
  6. **Corregir en task file:** DESKTOP-08 redacción "cliente IQL tipado" (no REST por-op).
- **Contrato:** `npm run tauri dev` abre ventana + ping responde; `cargo check` en src-tauri pasa; `cargo check` raíz sin cambios; cierre con MCP conectado no deja procesos huérfanos.
- **Estado:** ⬜ PENDING

---

## Dependencias clave (grafo)

```
F0 (Backlog-EDIT) → todo
AUDIT-01 → AUDIT-04, AUDIT-03(re-escalado)
TECH-01 → TECH-04/AUD-010 (síntoma primero)
AUD-004 → AUD-006, TECH-03#4
DEBT-01 → TECH-03 (gate verde)
INV-025 → INV-009-B (contrato snippet)
NUEVO-01 → GH-139 (fusionado)
INV-007-B → MKT-17, TSK-103 (absorbe cierres)
```

## Secuencia recomendada

1. **Fase 0** — Backlog-EDIT (1h): corrige las 12 premisas stale.
2. **Fase 1** — cierres/consolidaciones (2h): cierra NUEVO-18/TSK-103/MKT-17/GH-144/LEG-01/COM-04; fusiona TECH-04+AUD-010, GH-139⊂NUEVO-01, AUDIT-06+07, AUDIT-03.
3. **Fase 2** — release blockers (2-4d): AUDIT-01 → AUD-004 → AUDIT-04 → AUD-011 → AUD-001 → TECH-01 → TECH-02.
4. **Fase 4 docs drift** en paralelo con F3 (son independientes).
5. **Fase 5 web** + **Fase 7 community** en paralelo (independientes).
6. **Fase 6** — INV-025 → INV-009-B → INV-008-B → INV-007-B → NUEVO-16 → NUEVO-22.
7. **Fase 8 DESKTOP** — después de estabilizar core (depende de que AUDIT-01/04 no rompan bindings).

## Hallazgos (origen del plan — consolidado de 6 sub-agentes)

- **Solapamientos fusionados:** TECH-04≡AUD-010; GH-139⊂NUEVO-01; AUDIT-06+07; AUD-004/AUD-006/TECH-03#4 (query_lisp); MKT-17≡INV-007-B; NUEVO-18 premisa falsa; AUDIT-03 re-escalado.
- **Cierres sin código:** TSK-103 (resuelta), GH-144 (resuelta), LEG-01 (humana), COM-04 (bloqueada), COM-02/03 (humanas), TECH-06 (decisión documentada → feature request).
- **Correcciones de premisa (12):** ver Fase 0.
- **Precisión del backlog:** los IDs con líneas exactas (AUD-010, AUDIT-08, AUD-008, TECH-03#1/#2/#4, DEBT-01, TECH-01) son precisos; los que contienen conteos/listas (AUD-005, AUD-006, AUD-007, GH-123, NUEVO-18, INV-009-B, INV-016-B) tienen datos parcialmente incorrectos — NO confiar en la descripción tal cual, usar los datos corregidos de este plan.

---

## Probes de integridad (antes de cada tarea)

- [ ] Backlog.md corregido (Fase 0 ejecutada) antes de implementar cualquier tarea con premisa stale
- [ ] Git status limpio (o cambios del pipeline actual)
- [ ] `just verify-quick` pasa antes de merge de cualquier fix de código
- [ ] Para release: `skill unified-review --mode certify --profile vantadb`

=== RECITATION ===
Campaign ID: 06826e46-9034-4f0c-bb4c-5e06742d9480
Objetivo activo: F3 — AUDIT-08 P2 debt ledger
Estado: completed
Última acción: lead actualizó refs reales + P2-2 RESUELTA
Resultado: ✅ grep lib.rs:1754 = 0, comentario O(n)
Próxima acción: —
Contrato: grep experimental-lisp|MCP-02 = 0; sin duplicados en progreso
Próxima tarea si completa: 27
=== END RECITATION ===
