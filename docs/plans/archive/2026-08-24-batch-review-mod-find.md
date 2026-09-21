# Plan de Ejecución: Batch REVIEW/MOD/FIND (pipeline paralelo)

> **Campaign ID:** 4b9e337a-2fd0-4625-9cba-e26ea37f780b
> **Inicio:** 2026-08-24
> **Estado:** ✅ COMPLETADO
> **Fuente:** docs/Backlog.md (selección del lead, sesión 2026-08-24)
> **Modo:** FAIL_MODE=parallel, MAX_CONCURRENT=3

## Resumen
| DO | SKIP | BLOQUEADO |
|----|------|-----------|
| 10 | 2    | 0         |

| ID | Descripción | Archivos | Ruta | Contrato | Estado |
|----|-------------|----------|------|----------|--------|
| `REVIEW-06` | OOM rustc en `cargo test --workspace` — fix `[profile.test]` | root `Cargo.toml`, `src/lib.rs` | vanta-tuner | `cargo nextest run -p vantadb --profile audit` compila sin OOM | ✅ (fix commiteado `167a8d4c`; verify 2055 tests compilan/ejecutan sin OOM + `cargo check --workspace` 42.84s OK; 3 failures = tests nuevos MOD-02, colateral) |
| `REVIEW-07` | nextest default-filter stale | — | — | — | ⛔ SKIP — resuelto por BND-06/`db337b00` (filtro scope-safe verificado en `.config/nextest.toml`) |
| `REVIEW-11` | Dependabot sin pip | `.github/dependabot.yml` | vanta-lead | yaml válido con ecosistema pip | ✅ COMMITTED `ci(deps)` |
| `MOD-02` | Transacciones no crash-atómicas | `src/storage/engine/txn.rs`, `wal_sharded.rs` | vanta-worker | tests txn + chaos pasan; replay respeta Commit marker | ✅ COMMITTED `fix(storage)` — 45/45 init+txn, chaos 1/1, crash 8/8 |
| `MOD-08`+`MOD-09` | Loop stdio serial + shutdown descarta respuesta in-flight | `vantadb-mcp/src/server.rs` | vanta-worker | mcp_tests pasan; respuesta in-flight se escribe antes de salir | ✅ COMMITTED `5aa42007` `fix(mcp)` — 60/60 mcp_tests |
| `MOD-19` | ~30% API core sin exponer en Python | `vantadb-python/` | vanta-worker | pytest pasa; similar_to_key/count/delete_by_filter expuestos | ✅ COMMITTED `dc65c242` `feat(python)` — pytest 118 pass, docs coverage 0 gaps |
| `FIND-27` | Provider Ollama endpoint legacy roto | `src/llm.rs` | vanta-worker | test contra mock; POST /api/embed {model,input} | ✅ COMMITTED `447a07d7` `fix(llm)` — 2/2 tests mock PASS |
| `FIND-28` | Casts u8*→f32* sin align check ×3 | `src/index/ivf.rs:69`, `src/storage/engine/mapper.rs:191`, `src/sdk/serialization/bytes.rs:136` | vanta-worker | cargo check + clippy limpios; align_to aplicado | ✅ COMMITTED `2d9fa75f` `fix(index)` — nextest 2055/2055, review vanta-audit APPROVE |
| `UX-01`+`UX-05` | LensShell compartido + token `.label-tech` | `desktop/src/components/*`, `desktop/src/index.css` | vanta-worker | `npm run build` (desktop) exit 0; 6 lenses usan LensShell | ✅ COMMITTED `6260938e` `refactor(desktop)` — build exit 0, 6 lenses usan LensShell |
| `FIND-04` | Tabla cross-SDK search() Python↔TS | READMEs SDK, `docs/api/BINDINGS_NAMESPACES.md` | vanta-docs | tabla presente en ambos READMEs, link al doc de namespaces | ✅ COMMITTED `9de39702` `docs` — tabla en ambos READMEs, coverage 0 gaps |

## Waves
- **Wave 0:** REVIEW-06 · MOD-02 · FIND-27
- **Wave 1:** FIND-28 · MOD-19 · MOD-08+09
- **Wave 2:** UX-01+05 · FIND-04
- Inline (lead): REVIEW-11 ✅ · REVIEW-07 ⛔

## Notas
- Árbol tenía trabajo desktop P34 a medio hacer → checkpoint commiteado (`5a7f31e0`, `89ab5e2c`) antes de arrancar.
- Segunda sesión activa editando WorkspaceShell.tsx → re-verificar churn antes de Wave 2.
- Sub-agentes NO commitean (evita race del index); el lead verifica mecánico y commitea por tarea.

=== RECITATION REVIEW-06 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: REVIEW-06: OOM rustc en cargo test --workspace — fix [profile.test]
Estado: completed
Última acción: DISCOVERY + verificación completa. Fix ya commiteado (167a8d4c): [profile.test] debug=1/opt-level=0 + .cargo/config.toml jobs=2. Verify contrato: cargo nextest run -p vantadb --profile audit compila sin OOM (2055 tests, 2052 pass) + cargo check --workspace 42.84s sin OOM. Zero ediciones de código (fix del lead).
Resultado: OK
Próxima acción: Cierre: plan file marcado ✅. Próxima tarea MOD-02 (wave 0).
Contrato: verificacion: cargo nextest run -p vantadb --profile audit (compile sin OOM ✅, Summary 2052 passed/3 failed/1 skipped) + cargo check --workspace (42.84s ✅ sin OOM) + cargo nextest run --no-run -p vantadb (0.57s ✅); evidencia: claim=OOM resuelto, evidencia=cargo nextest run full 2055 tests ejecutados sin OOM, confianza=alta; claim=3 failures son de MOD-02 no de REVIEW-06, evidencia=git diff src/storage/engine/tests/init.rs muestra +fn test_crash_full_txn_with_commit_recovered / test_records_after_partial_txn_survive_recovery / test_crash_partial_txn_without_commit_not_recovered (líneas 516/543/579), confianza=alta; artefactos: .opencode/skills/campaign-executor/tasks/REVIEW-06.md; invariantes: [profile.release] intacto, features default intactas, jobs=2 se mantiene; deuda: ninguna; queda_pendiente: lead verifica 3 failures de MOD-02 cuando ese worker cierre su tarea
Próxima tarea si completa: MOD-02 (wave 0)
=== END RECITATION ===

=== RECITATION MOD-02 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: MOD-02 (H-2): transacciones crash-atómicas — fix recover_state para descartar txns incompletas sin perder registros ajenos; tests txn+chaos pasan
Estado: completed
Última acción: Root-cause: (1) tests usaban BackendKind::InMemory que jamás llama recover_state; (2) crash-sim escribía en path raíz pero el engine usa data_dir/vanta.wal (subdir data/); (3) el fix txn_end aplicado era no-op y test 3 malformado. Fix: tracking per-id open_txn + bound por próximo Begin; tests reescritos vía engine.wal con backend real. Verify: 45/45 init+txn, 22/22 recovery, chaos failpoints 1/1, crash_injection 1/1 + durability_recovery 7/7, fmt+clippy limpios
Resultado: OK
Próxima acción: Lead: verificar mecánico del diff (git diff src/storage/engine/init.rs src/storage/engine/tests/init.rs) y commitear feat/fix MOD-02
Contrato: verificacion: cargo nextest run -p vantadb -E 'test(init::) or test(txn)' (45/45) + --profile chaos --features failpoints (1/1) + --test durability_recovery --test crash_injection (8/8) + cargo check -p vantadb + cargo fmt --check + cargo clippy -p vantadb --all-targets — todo ✅; evidencia: claim=replay descarta txn incompleta y preserva batch completo ajeno, evidencia=src/storage/engine/tests/init.rs:483-593 (3 tests PASS), confianza=alta; artefactos: src/storage/engine/init.rs, src/storage/engine/tests/init.rs (worktree, sin commit); invariantes: no tocar wal.rs/vector//storage/ (solo init.rs + tests), commit lo hace el lead; deuda: registro plano sin markers inmediatamente después de txn parcial es indistinguible y se descarta fail-safe (ceiling documentado en task file); queda_pendiente: ninguno
Próxima tarea si completa: ninguna (tarea única)
=== END RECITATION ===

=== RECITATION MOD-19 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: MOD-19: exponer count/delete_by_filter/similar_to_key en binding Python PyO3
Estado: completed
Última acción: Implementacion completa: 3 metodos flat + sub-client memory + AsyncVantaDB + stubs .pyi + tests (7 nuevos) + docs. Verify: 118 pytest passed, cargo check/fmt/clippy limpios, docs coverage 0 gaps.
Resultado: OK
Próxima acción: Lead: verificar diff y commitear MOD-19
Contrato: verificacion: python -m pytest vantadb-python/tests/ -v (118 passed, 0 failed) + cargo check -p vantadb_py + cargo fmt --check + cargo clippy -p vantadb_py --all-targets (exit 0) + pwsh scripts/validate-docs-coverage.ps1 (0 gaps); evidencia: claim=3 metodos expuestos flat+memory+async, evidencia=dir(vantadb_py.VantaDB) → ['count','delete_by_filter','similar_to_key'], confianza=alta; artefactos: vantadb-python/src/{lib.rs,convert.rs}, vantadb_py/{__init__.py,__init__.pyi,vantadb_py.pyi}, tests/{test_sdk.py,test_subclients.py}, docs/api/{PYTHON_SDK.md,BINDINGS_NAMESPACES.md}, task file MOD-19.md; invariantes: no tocar src/ core (wal/vector/storage), commit lo hace el lead; deuda: ninguna; queda_pendiente: lead verifica mecánico + commit por tarea
Próxima tarea si completa: ninguna (tarea unica del plan batch)
=== END RECITATION ===

=== RECITATION FIND-28 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: FIND-28: reemplazar 3 casts u8*→f32* sin align check por as_f32_slice() (helper align_to REVIEW-15)
Estado: completed
Última acción: Steps 1-4 OK: 3 casts reemplazados por as_f32_slice() + const muerta eliminada. Verify completo: check/clippy/fmt/nextest 2055/2055. Review P2-01 vanta-audit APPROVE. FIND-29 creado en Backlog (layer.rs, recomendacion no bloqueante).
Resultado: OK
Próxima acción: Lead: verificar diff y commitear (fix: FIND-28); resolver WIP lock (DESKTOP-24/28, REVIEW-06 stale) para marcar FIND-28 completed en el server
Contrato: verificacion: cargo check -p vantadb OK + cargo clippy -p vantadb --all-targets -- -D warnings OK + rustfmt --check 4 archivos OK + cargo nextest run -p vantadb 2055/2055 OK; evidencia: claim=3 casts crudos reemplazados por as_f32_slice (align_to), evidencia=git diff src/index/ivf.rs src/index/distance/mapper.rs src/index/serialize/bytes.rs, confianza=alta; claim=const muerta MAX_VEC_F32_LEN eliminada sin refs colgantes, evidencia=rg MAX_VEC_F32_LEN src/ solo vector_data.rs:7 (copia local), confianza=alta; claim=review agente distinto APPROVE, evidencia=vanta-audit ses_fc912e222ffejOotrZgihVsa2B, confianza=alta; artefactos: .opencode/skills/campaign-executor/tasks/FIND-28.md, docs/Backlog.md (FIND-29 creado); invariantes: mismo slice/semantica (ivf->None, mapper->0.0, bytes->Err InvalidData), sin cambio de logica de negocio; deuda: ninguna (elimina 3 unsafe); queda_pendiente: lead verifica diff y commitea (worker NO commitea — regla batch); fmt workspace falla en vantadb-python/src/lib.rs (drift pre-existente MOD-19, no tocado)
Próxima tarea si completa: MOD-19 (wave 1)
=== END RECITATION ===

=== RECITATION MOD-08 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: MOD-08+MOD-09: loop stdio serial + shutdown descarta respuesta in-flight
Estado: completed
Última acción: Implementado: serve_lines despacha cada request en background task (MOD-08: reader drena stdin, sin backpressure), JoinSet drena respuestas in-flight en shutdown (MOD-09: response se escribe antes de salir). Removido EnteredSpan de dispatch_request (barrera !Send). Tests RED->GREEN añadidos. Verify completo verde.
Resultado: OK
Próxima acción: Lead: commit del diff (vantadb-mcp/src/server.rs) + auditoría concurrencia Regla 8
Contrato: artefactos: [.opencode/skills/campaign-executor/tasks/MOD-08.md, vantadb-mcp/src/server.rs (sin commit, lo commitea el lead)]; deuda: ninguna; evidencia: [claim=loop ya no bloquea el reader mientras despacha (spawn + JoinSet), evidencia=server.rs serve_lines inflight.spawn, confianza=alta, claim=respuesta in-flight se escribe antes de salir, evidencia=test in_flight_response_written_on_shutdown PASS, confianza=alta]; invariantes: protocolo JSON-RPC y semántica tools intactos (R-1), motor en spawn_blocking + semáforo (R-2/R-3), stdout serializado por tokio Mutex, no guard síncrono a través de .await; queda_pendiente: lead verifica mecánico y commitea (git add vantadb-mcp/src/server.rs) + auditoría de concurrencia Regla 8 delegada a vanta-chaos/vanta-review; verificacion: cargo check -p vantadb-mcp OK + cargo nextest run -p vantadb-mcp 60/60 + cargo test -p vantadb-mcp --test mcp_tests 60/60 + cargo fmt --check OK + cargo clippy -p vantadb-mcp --all-targets -- -D warnings OK
Próxima tarea si completa: ninguna (tarea única, Wave 1)
=== END RECITATION ===

=== RECITATION FIND-04 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: FIND-04: tabla cross-SDK search() Python↔TS en ambos READMEs + link a BINDINGS_NAMESPACES.md
Estado: completed
Última acción: Implementacion completa: seccion Cross-SDK Search Parity agregada a vantadb-python/README.md y vantadb-ts/README.md con tabla comparativa de 11 capacidades + link a docs/api/BINDINGS_NAMESPACES.md (TS ya lo tenia; Python lo agrega). Verify: docs coverage 0 gaps, tablas presentes en ambos, link resuelve.
Resultado: OK
Próxima acción: Lead: verificar diff de los 2 READMEs y commitear (docs: FIND-04); resolver WIP lock stale (DESKTOP-24/28, REVIEW-06) para desbloquear el server
Contrato: verificacion: tabla presente en ambos READMEs (grep 'Cross-SDK Search Parity' TS:163 Py:104) + link a docs/api/BINDINGS_NAMESPACES.md existe (TS:168 Py:109) + scripts/validate-docs-coverage.ps1 (0 gaps, exit 0); evidencia: claim=Python search() es pure ANN (vector,top_k) devuelve (node_id,distance), evidencia=vantadb-python/src/lib.rs:1596-1620, confianza=alta; claim=TS search() es hybrid SearchRequest{namespace,query_vector,filters,text_query,top_k,distance_metric,explain}, evidencia=vantadb-ts/src/vantadb.ts:595 + types.ts:61, confianza=alta; claim=divergencia de nombre search() documentada, evidencia=secciones Cross-SDK Search Parity + BINDINGS_NAMESPACES.md:31, confianza=alta; artefactos: vantadb-python/README.md, vantadb-ts/README.md, .opencode/skills/campaign-executor/tasks/FIND-04.md; invariantes: docs tecnicos en ingles, sin cambios de codigo; deuda: ninguna; queda_pendiente: lead verifica mecanico (grep + docs coverage) y commitea
Próxima tarea si completa: ninguna (tarea unica, Wave 2)
=== END RECITATION ===

=== RECITATION UX-01 ===
Campaign ID: 4b9e337a-2fd0-4625-9cba-e26ea37f780b
Objetivo activo: UX-01+UX-05: LensShell compartido + token .label-tech en lentes desktop
Estado: completed
Última acción: LensShell.tsx creado + 6 lentes refactorizadas (Consolidate/Indices/Retrieval/Graph/Space/Memory). Verify: npm run build exit 0, vitest 68/68, grep 6 lentes. label-tech ya existía (index.css:252) y ahora se usa vía LensShell.
Resultado: OK
Próxima acción: Lead: verificar diff (desktop/src/components/layout/LensShell.tsx + 6 lentes) y commitear refactor(desktop) UX-01+UX-05
Contrato: verificacion: cd desktop && npm run build exit 0 (campaign_verify_cmd PASSED, tsc+vite 10.89s) + npx vitest run 68/68 pass; evidencia: claim=6 lentes usan LensShell, evidencia=grep LensShell desktop/src/components → Consolidate/Indices/Retrieval/Graph/Space/Memory (6 archivos), confianza=alta; claim=label-tech usado consistentemente, evidencia=desktop/src/components/layout/LensShell.tsx:25 className='label-tech text-muted-foreground' + definicion pre-existente index.css:252, confianza=alta; artefactos: desktop/src/components/layout/LensShell.tsx, ConsolidateLens.tsx, IndicesLens.tsx, RetrievalLens.tsx, GraphLens.tsx, SpaceLens.tsx, MemoryLens.tsx, task file UX-01.md; invariantes: 6 lentes con mismas props en WorkspaceShell, layout WebGL full-height Graph/Space preservado (flex-col + canvas flex-1), label-tech no duplicado; deuda: ninguna; queda_pendiente: lead verifica + commitea (worker no commitea — regla batch); comando exacto para lead: cd desktop && npm run build + git add desktop/src/components/layout/LensShell.tsx desktop/src/components/consolidate/ConsolidateLens.tsx desktop/src/components/indices/IndicesLens.tsx desktop/src/components/lens/retrieval/RetrievalLens.tsx desktop/src/components/graph/GraphLens.tsx desktop/src/components/space/SpaceLens.tsx desktop/src/components/memory/MemoryLens.tsx
Próxima tarea si completa: FIND-04 (docs, wave 2 restante)
=== END RECITATION ===
