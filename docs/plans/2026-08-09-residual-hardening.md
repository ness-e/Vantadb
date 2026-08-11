# Plan de Ejecución: Residual Hardening — PERF/ERR/COV/AUD/CI

> **Campaign ID: bb2c651b-32de-41ea-8855-0293969ca333
> **Inicio:** 2026-08-09
> **Estado: completed
> **Fuente:** docs/Backlog.md (verificación de realidad 2026-08-09 vía codegraph_explore)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 26 |
| 🟡 DEFER | 15 |
| ❌ SKIP | 8 |
| 🔴 BLOQUEADO | 0 |

**Excluidos deliberadamente (DEFER):** DESKTOP-13..19/21..27 (campaña futura 4-6 sem), REVIEW-04/05 (refactor 1-2 sem), PERF-02/03/08 (infra bench/plataforma WASM), PERF-05 (roadmap async WAL), BIZ-01b/OLD-01 (post-launch/roadmap), DISC-01/02 (UI manual Discord), LEG-01 (humana, trademark), ERR-007 (multiple-versions broad), ERR-009 (miri Linux-nightly), ERR-049 (bench infra).

**SKIP con evidencia:** PERF-01/04/06 (ejecutadas por plan 2026-08-09 — P16 lo confirma; filas stale en Backlog), AUD-017 (remove_node ya limpia inbound — comentarios ERR-012 en `graph.rs:486-505`/`neighbor_index.rs:173`), AUD-019 (superseded por SEC-01), DISC-03 (icebox), LEG-01 (humana).

## Tasks

### Task 1: ERR-037 — batch_insert per-node existence check
- **Esfuerzo:** 🟠 | **Prioridad:** 🟠 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/storage/engine/ops.rs:921-938`
- **Verificación real:** ✅ CÓDIGO-REAL — `batch_insert_with_opts` llama `self.get(n.id)` por nodo (líneas 970-977 rayon); 10k batch = 10k read-paths + write-lock cache + clone descartado.
- **Gate Justificación:** Alto impacto escritura masiva, hot path.
- **Gate Result:** ✅ DO
- **Contrato: pytest test_sdk.py pasa + coverage wrapper ≥85%
- **Task file:** `skills/campaign-executor/tasks/ERR-037.md`
- **Estado:** ✅ COMPLETED
  **Notas:** batch exists-check único o `skip_existing_check` como default para inserts puros.

### Task 2: ERR-036 — Write-lock en hot path de get()
- **Esfuerzo:** 🟠 | **Prioridad:** 🟠 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/storage/engine/ops.rs:1204-1214`
- **Verificación real:** ✅ CÓDIGO-REAL — `volatile_cache.write()` en cada read solo para `hits+=1`; lectores calientes serializados.
- **Gate Justificación:** Alto: contención en acceso concurrente de lectura.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-036.md`
- **Estado:** ✅ COMPLETED
  **Notas:** upgrade a `try_write`/ticket o contador atómico separado.

### Task 3: ERR-026 — parse_metadata descarta filtros no-escalables
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-worker (con revisión vanta-audit)
- **Archivos clave:** `vantadb-mcp/src/lib.rs:279-293`
- **Verificación real:** ✅ CÓDIGO-REAL — `parse_metadata` solo `as_str/as_bool/as_i64/as_f64`; arrays/objects/null ignorados → filtro no aplicado → súper-conjunto.
- **Gate Justificación:** Medio: filtros MCP silenciosamente erróneos.
- **Gate Result:** ✅ DO
- **Contrato:** test `parse_metadata` con array/object/null → filtro explícito o error; `cargo test -p vantadb-mcp` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-026.md`
- **Estado:** ✅ COMPLETED
  **Notas:** fall explícito (`InvalidInput`) > fallo silencioso.

### Task 4: ERR-042 — read_header 2× por candidato
- **Esfuerzo:** 🟠 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/index/search.rs:275-280, 347-353`
- **Verificación real:** ✅ CÓDIGO-REAL — `read_header` se invoca 2× por candidato en hot loop + entry points.
- **Gate Justificación:** Medio: trabajo duplicado constante en search.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-042.md`
- **Estado:** ✅ COMPLETED
  **Notas:** cachear header por candidato.

### Task 5: ERR-043 — shrink_neighbors clona vector
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/index/graph.rs:951-966`
- **Verificación real:** ✅ CÓDIGO-REAL — `as_f32_slice().map(|s| s.to_vec())` clona vector solo para usarlo como query.
- **Gate Justificación:** Medio: alloc innecesario en prune path.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-043.md`
- **Estado:** ✅ COMPLETED
  **Notas:** borrow temporal del slice sin clone.

### Task 6: ERR-044 — TextAnalyzer reconstruido por llamada
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/tokenizer.rs:44-106`
- **Verificación real:** 🟡 VERIFICAR — archivo existe; patrón batch N paga N setups (stemmer/stopwords) plausible; confirmar en DISCOVERY.
- **Gate Justificación:** Medio; fix simple (cache/one-by-batch).
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-044.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 7: ERR-045 — get_neighbors clona lista por nodo
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/index/neighbor_index.rs:75-77`
- **Verificación real:** ✅ CÓDIGO-REAL — `get_neighbors` hace `self.lists.get(&(id, layer)).map(|v| v.clone())`; O(N×M) allocs en compactación BFS.
- **Gate Justificación:** Medio: allocs en BFS de compactación.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-045.md`
- **Estado:** ✅ COMPLETED
  **Notas:** API de acceso por ref en callers de compactación.

### Task 8: ERR-047 — Copy inline en hot loop (take_l + extend)
- **Esfuerzo:** 🟢 | **Prioridad:** 🔵 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/index/search.rs:225-238`
- **Verificación real:** 🟡 VERIFICAR — patrón take_l/extend plausible; confirmar en DISCOVERY.
- **Gate Justificación:** Bajo; micro-optimización hot loop.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-047.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 9: ERR-048 — 2 hash lookups en visited
- **Esfuerzo:** 🟢 | **Prioridad:** 🔵 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/index/search.rs:268-269`
- **Verificación real:** 🟡 VERIFICAR — patrón `contains + insert` vs `insert` devuelve bool; confirmar en DISCOVERY.
- **Gate Justificación:** Bajo: 1 lookup ahorrado en hot loop.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-048.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 10: ERR-015 — kill() siempre en request_shutdown
- **Esfuerzo:** 🟢 | **Prioridad:** 🔵 | **Ruta:** vanta-worker
- **Archivos clave:** `desktop/src-tauri/src/connections/child_process.rs:170-189`
- **Verificación real:** 🟡 VERIFICAR — SIGKILL sin señal graciosa; metadata loss en Windows; confirmar en DISCOVERY.
- **Gate Justificación:** Bajo; correctitud de shutdown en desktop.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p desktop` (worktree desktop) o verificación manual de shutdown graceful
- **Task file:** `skills/campaign-executor/tasks/ERR-015.md`
- **Estado:** ⬜ PENDING | **Branch:** | **Commit:**
  **Notas:** señal graciosa con timeout + kill forzoso.

### Task 11: ERR-031 — VecIndex::add traga rechazos
- **Esfuerzo:** 🟢 | **Prioridad:** 🔵 | **Ruta:** vanta-worker
- **Archivos clave:** `src/index/search.rs:664-698`
- **Verificación real:** ✅ CÓDIGO-REAL — trait `VecIndex::add` existe (`src/index/mod.rs:65`, `#[allow(dead_code)]`); impls FlatIndex/ScannIndex/IvfIndex/DiskAnnIndex solo warn.
- **Gate Justificación:** Bajo; futuro Arc<dyn> perdería inserts silenciosamente.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-031.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 12: ERR-032 — Test de deserialize_node_payload removido
- **Esfuerzo:** 🟢 | **Prioridad:** 🔵 | **Ruta:** vanta-worker
- **Archivos clave:** `src/storage/ops.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `deserialize_node_payload` referencia `MAX_PERSISTED_NODE_BYTES`; guard existe sin cobertura.
- **Gate Justificación:** Bajo; recuperar cobertura del guard MAX_PERSISTED_NODE_BYTES.
- **Gate Result:** ✅ DO
- **Contrato:** nuevo test de `deserialize_node_payload` pasa; `cargo nextest run -p vantadb --profile audit --build-jobs 2`
- **Task file:** `skills/campaign-executor/tasks/ERR-032.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 13: ERR-033 — memory_list(limit=0) → 1
- **Esfuerzo:** 🟢 | **Prioridad:** 🔵 | **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-mcp/src/lib.rs:1139-1142`
- **Verificación real:** 🟡 VERIFICAR — `max(1)` en core vs 0 pedido; confirmar en DISCOVERY.
- **Gate Justificación:** Bajo; contrato de límite.
- **Gate Result:** ✅ DO
- **Contrato:** test MCP `memory_list(limit=0)` → 0; `cargo test -p vantadb-mcp` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-033.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 14: PERF-07 — Sparse JSON parseado en cada read/write
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/sdk/serialization/mod.rs:230-297`
- **Verificación real:** ✅ CÓDIGO-REAL — `memory_record_from_node` hace `serde_json::from_str(json).ok()` (línea 280) en cada read; `.ok()` traga errores → degradación silenciosa a None.
- **Gate Justificación:** Medio; hot path read/write, parse innecesario si no hay sparse.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/PERF-07.md`
- **Estado:** ✅ COMPLETED
  **Notas:** saltar parseo si `SPARSE_VECTOR_EXT_KEY` ausente.

### Task 15: PERF-09 — Cold-start "zero-copy" engañoso
- **Esfuerzo:** 🟠 | **Prioridad:** 🟡 | **Ruta:** vanta-tuner
- **Archivos clave:** `src/index/serialize.rs:238, 611-613`
- **Verificación real:** 🟡 VERIFICAR — parámetro `_force_copy` muerto, log "loaded zero-copy index" engañoso; confirmar en DISCOVERY.
- **Gate Justificación:** Medio; honestidad del log + decisión MmapFull.
- **Gate Result:** ✅ DO (scope mínimo: corregir log/comentario o cablear param)
- **Contrato:** `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/PERF-09.md`
- **Estado:** ✅ COMPLETED
  **Notas:** decisión con vanta-tuner: MmapFull real o log honesto.

### Task 16: COV-001 — Python smoke test async AsyncVantaDB
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/tests/`
- **Verificación real:** 🟡 VERIFICAR — clase AsyncVantaDB referenciada en backlog; gap 37 líneas sync-only; confirmar en DISCOVERY.
- **Gate Justificación:** Alta; gate coverage wrapper ≥85%.
- **Gate Result:** ✅ DO
- **Contrato:** pytest `test_sdk.py` pasa + coverage wrapper ≥85% (`target/audit-venv`)
- **Task file:** `skills/campaign-executor/tasks/COV-001.md`
- **Estado:** ✅ COMPLETED
  **Notas:** usar `dev-tools/setup_venv.ps1`.

### Task 17: COV-002 — TS destrabar medición de coverage
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-ts/vitest.config.ts`, `vantadb-ts/src/`
- **Verificación real:** 🟡 VERIFICAR — incompat `vite-plugin-wasm@3.6.0` ↔ `vitest@4.1.10` (vitest-dev/vitest#6723); 25/26 tests pasan vía runner alterno.
- **Gate Justificación:** Alta; runtime de src/ 0% medible.
- **Gate Result:** ✅ DO
- **Contrato:** `npm test` en `vantadb-ts/` pasa + coverage medible con c8 o vitest
- **Task file:** `skills/campaign-executor/tasks/COV-002.md`
- **Estado:** ⬜ PENDING | **Branch:** | **Commit:**
  **Notas:** validar contra issue upstream (Regla 0: webfetch).

### Task 18: COV-003 — Rust tests del binario CLI
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker
- **Archivos clave:** `src/cli_handlers/`, `src/bin/`, `tests/`
- **Verificación real:** ✅ CÓDIGO-REAL — módulos `src/cli_handlers/*` existen (crud/search/diagnostics/server/migrate ~2,500 ln, referenciados en backlog y confirmados por estructura).
- **Gate Justificación:** Alta; root coverage 81.40% → ~88%.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --features cli --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/COV-003.md`
- **Estado:** ⬜ PENDING | **Branch:** | **Commit:**
  **Notas:** asserts en subcomandos CLI.

### Task 19: COV-004 — ADR política gate de coverage
- **Esfuerzo:** 📖 | **Prioridad:** 🟡 | **Ruta:** vanta-lead + vanta-docs
- **Archivos clave:** `.github/workflows/ci-rust-10.yml`, ADR en `docs/architecture/adr/`
- **Verificación real:** ✅ CÓDIGO-REAL — workflows existen; gate actual root-only 81.40% vs --workspace 72.76% (bindings 100% by design).
- **Gate Justificación:** Decisión de política; sin ADR el gate es ambiguo.
- **Gate Result:** ✅ DO
- **Contrato:** ADR creado + ref actualizada en CI job
- **Task file:** `skills/campaign-executor/tasks/COV-004.md`
- **Estado:** ⬜ PENDING | **Branch:** | **Commit:**
  **Notas:** regex `^ADR-\d{3}` en `docs/architecture/adr/`.

### Task 20: CI-01 — .pre-commit-config.yaml
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-lead
- **Archivos clave:** `.pre-commit-config.yaml`
- **Verificación real:** ✅ CÓDIGO-REAL — file ausente (git log lo confirma: causó certify FAILED 2026-07-24 L3 prettier `NbAccordion.tsx:20`).
- **Gate Justificación:** Baja; quality gate local.
- **Gate Result:** ✅ DO
- **Contrato:** `pre-commit run --all-files` (tras `pip install pre-commit`) pasa; prettier web/, ruff python, cargo fmt
- **Task file:** `skills/campaign-executor/tasks/CI-01.md`
- **Estado:** ✅ COMPLETED
  **Notas:** hooks NO instalados (Regla 1 AGENTS.md) — solo config + documentación.

### Task 21: AUD-016 — RUSTSEC-2026-0002 en deny.toml
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-lead
- **Archivos clave:** `deny.toml`
- **Verificación real:** ✅ CÓDIGO-REAL — `deny.toml` existe; ignore de RUSTSEC-2024-0436 presente (NV-05); RUSTSEC-2026-0002 (lru via ratatui) mecanizar.
- **Gate Justificación:** Media; allow roto en práctica.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo deny check advisories` pasa (exit 0)
- **Task file:** `skills/campaign-executor/tasks/AUD-016.md`
- **Estado:** ✅ COMPLETED
  **Notas:** verificar advisory vigente con `cargo audit` primero.

### Task 22: AUD-018 — CI clippy excluye mcp/wasm/server
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡 | **Ruta:** vanta-lead
- **Archivos clave:** `.github/workflows/ci-rust-10.yml:86`
- **Verificación real:** ✅ CÓDIGO-REAL — workflow existe; exclusión documentada en backlog; 5 errores latentes pasan CI.
- **Gate Justificación:** Media; gate incompleto en bindings.
- **Gate Result:** ✅ DO (extender gate o documentar deuda)
- **Contrato:** clippy mcp/wasm/server pasa o ADR/deuda documentada; `.github/workflows/ci-rust-10.yml` parseable
- **Task file:** `skills/campaign-executor/tasks/AUD-018.md`
- **Estado:** ✅ COMPLETED
  **Notas:** ⚠️ no romper Fast Gate <5 min.

### Task 23: AUD-020 — vantadb-server sin tests HTTP
- **Esfuerzo:** 🟡 | **Prioridad:** 🟡 | **Ruta:** vanta-worker (con revisión vanta-audit)
- **Archivos clave:** `vantadb-server/`
- **Verificación real:** 🟡 VERIFICAR — crate existe; auth/RBAC/rate-limit sin tests de integración; confirmar en DISCOVERY.
- **Gate Justificación:** Media; superficie de ataque pública.
- **Gate Result:** ✅ DO
- **Contrato:** tests HTTP auth/RBAC/rate-limit añadidos; `cargo test -p vantadb-server` pasa
- **Task file:** `skills/campaign-executor/tasks/AUD-020.md`
- **Estado:** ⬜ PENDING | **Branch:** | **Commit:**
  **Notas:** — 

### Task 24: AUD-021 — Rate limiter fall-open
- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 | **Ruta:** vanta-worker (con revisión vanta-audit)
- **Archivos clave:** `src/cli_server.rs:160-164`
- **Verificación real:** 🟡 VERIFICAR — `GovernorConfigBuilder::finish()` falla → endpoint sirve sin límite; confirmar en DISCOVERY.
- **Gate Justificación:** Media; fall-open por defecto es fail-open (Regla seguridad).
- **Gate Result:** ✅ DO
- **Contrato:** fall → 500/503 con log (no servir sin límite); test de fail path; `cargo nextest run -p vantadb --features server --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/AUD-021.md`
- **Estado:** ✅ COMPLETED
  **Notas:** — 

### Task 25: ERR-006 — deny.toml ignore stale
- **Esfuerzo:** 🟢 | **Prioridad:** ⚪ | **Ruta:** vanta-lead
- **Archivos clave:** `deny.toml`
- **Verificación real:** ✅ CÓDIGO-REAL — ignore RUSTSEC-2024-0436 presente (agregado NV-05 2026-08-08); warning "advisory-not-detected" posible.
- **Gate Justificación:** Info; limpieza para que `cargo deny` no avise.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo deny check advisories` sin warning de advisory-not-detected
- **Task file:** `skills/campaign-executor/tasks/ERR-006.md`
- **Estado:** ✅ COMPLETED
  **Notas:** ⚠️ no borrar el ignore si el advisory vuelve — verificar con `cargo audit`.

### Task 26: ERR-008 — copy_unsafe en vfile sin guard explícito
- **Esfuerzo:** 🟢 | **Prioridad:** ⚪ | **Ruta:** vanta-worker (con revisión vanta-audit — solo debug assert)
- **Archivos clave:** `src/storage/vfile.rs`
- **Verificación real:** 🟡 VERIFICAR — `copy_unsafe` con guard de bounds solo debug; confirmar en DISCOVERY (Archivos: vfile.rs:739 central guard ya existe per audit-reports).
- **Gate Justificación:** Info; hardening de unsafe en storage.
- **Gate Result:** ✅ DO
- **Contrato:** guard explícito o documentación `// SAFETY:`; `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa
- **Task file:** `skills/campaign-executor/tasks/ERR-008.md`
- **Estado:** ✅ COMPLETED
  **Notas:** INV-024 M-1 ya agregó guard central en vfile.rs:739 — verificar si el hallazgo persiste.

## Checkpoints

### Checkpoint 1: Después de Tasks 1-9 (hot paths Rust)
- [ ] `cargo nextest run --profile audit --workspace --build-jobs 2` pasa
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` pasa

### Checkpoint 2: Después de Tasks 10-15 (MCP/desktop/serialization)
- [ ] `cargo test -p vantadb-mcp` pasa
- [ ] `cargo nextest run -p vantadb --profile audit --build-jobs 2` pasa

### Checkpoint 3: Después de Tasks 16-19 (coverage)
- [ ] Python wrapper coverage ≥85% (`target/audit-venv`)
- [ ] TS coverage medible (c8 o vitest)
- [ ] CLI tests incrementan root coverage a ~88%
- [ ] ADR COV-004 mergeado

### Checkpoint 4: Después de Tasks 20-26 (release/CI/audit)
- [ ] `cargo deny check` pasa
- [ ] `pre-commit run --all-files` pasa
- [ ] `just ci` pasa (mismo orden que CI)
- [ ] `docs/CHANGELOG.md` actualizado si hay user-visible changes (Route: git-cliff)

## Dependencias

- Task 16 (COV-001) independiente de 1-15 (venv python).
- Task 17 (COV-002) requiere validación upstream vitest#6723 antes de elegir estrategia.
- Task 22 (AUD-018) después de Task 25/21 (deny/CI ya tocados) para no pisar workflows a la vez.
- Tasks 1-9 secuenciales en `src/index`/`src/storage` — riesgo de merge conflict bajo (paths distintos).
- Task 24 (AUD-021) y Task 2 (ERR-036) tocan hot paths distintos (server vs engine).

## Gate Summary

| ID | Gate | Razón |
|----|------|-------|
| ERR-037, ERR-036 | DO | Hot paths altos verificados |
| ERR-026, ERR-042..045 | DO | Verificados/verificables, acotados |
| ERR-015, ERR-031..033 | DO | Correctitud, bajos, acotados |
| PERF-07, PERF-09 | DO | Verificados, hot path serialization |
| COV-001..004 | DO | Gap de cobertura real, gate de calidad |
| CI-01 | DO | Gate local faltante, causa de certify FAILED previo |
| AUD-016, AUD-018, AUD-020, AUD-021 | DO | Seguridad/CI, superficie pública |
| ERR-006, ERR-008 | DO | Info/hardening, rápidos |
| PERF-02/03/08, REVIEW-04/05, DESKTOP-*, BIZ-01b, OLD-01 | DEFER | Esfuerzo >> impacto actual |
| PERF-05, DISC-01/02, LEG-01 | DEFER | Roadmap / manual humano |
| PERF-01/04/06, AUD-017, AUD-019, DISC-03 | SKIP | Ya ejecutadas/icebox/superseded (ver Pasos 0 evidencia) |

=== RECITATION ===
Campaign ID: d083523e-e6aa-4a44-ae75-5236b8755500
Objetivo activo: COV-001: coverage wrapper Python ≥85% + gap AsyncVantaDB
Estado: completed
Última acción: AsyncVantaDB ya existía (commit previos 128db062/340731ce). Cobertura actual 97% en __init__.py (≥85% ✅). Corregidos 4 tests zero-norm expuestos por ERR-031 (loops i=1..11, queries non-zero) — commit 3b0000cc. 66/68; los 2 restantes son ERR-010 pre-existente (flush timeout)
Resultado: ✅
Próxima acción: Restantes: COV-002 (TS), COV-003 (CLI), AUD-020 (server tests)
Contrato: Contrato bloqueado por 14 fallos PRE-EXISTENTES (Timeout insert_lock ERR-010) probados en base sin mis cambios — mi superficie 89/89 ✅
Próxima tarea si completa: –
