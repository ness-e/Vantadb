---
title: "Auditoría — Seguridad"
type: audit-log
status: active
tags: [vantadb, avance, security, audit, fuzz, miri, ffi]
last_reviewed: 2026-08-07
aliases: []
---

# Auditoría — Seguridad

> Registro consolidado de hallazgos de seguridad, auditorías de código (AUD), fuzzing, Miri, FFI/unsafe. IDs originales conservados.

## Auditoría masiva 2026-06-19 (AUD-01..44) — resumen por severidad

- 🔴 **8 críticos** — TODOS resueltos (ver tabla codefix)
- 🟡 **14 medios** — todos abordados, 1 parcial
- 🟢 **22 bajos/sugerencia** — la mayoría aplicados o documentados
- 🔵 2 no aplicables (WASM-only paths sin efectos en server)

> Detalle completo por hallazgo: `docs/historial/autopsias-2026-06-19.md`

### Hallazgos críticos resueltos (AUD)
| ID | Hallazgo | Fix |
|---|---|---|
| AUD-01 | Panic path no atrapado (unwrap en archivar reader) | `handle_panic` + tests |
| AUD-02 | Chain de retry WAL sin supresión de duplicados | dedup vía `seen` |
| AUD-03 | Sink context channel full → block | bounded channel + async |
| AUD-04 | Cast unsafe sin check de alineación (`rkyv_archives.rs:54-71`) | fix alineación |
| AUD-05 | `.ok()` silencia errores UTF-8 | `map_err` + LogError |
| AUD-06 | N+1 query en `scan_nodes()` | batch lookup |
| AUD-07 | `ensure_indexes_current` 3 scans → 1 | unify |
| AUD-08 | `memory_record_to_node_owned` clones | reduce clones |

### Resueltos medios (AUD-10..)
| ID | Hallazgo | Fix |
|---|---|---|
| AUD-10 | `mapped_file_resident_bytes()` removida | delete function |
| AUD-11 | `wal_path` asignado pero nunca leído | remove field/param |
| AUD-12 | `HybridSearchResult` `#[allow(dead_code)]` | remove promote |
| AUD-13 | `cfg(feature = "encryption")` no activo en bench | feature |
| AUD-14 | `memory_usage()` mut → const | const fn |
| AUD-15 | Deadlock `Mutex` 2 locks — pattern `lock(L1)->lock(L2)` | reorder |
| AUD-18 | manual elided lifetime | fix |
| AUD-19 | test feature-gated | cfg gating |
| AUD-20 | .gitignore legal the Fixtures | cleanup |
| AUD-21 | dead `table` module | remove |
| AUD-22 | `spin` dependency + `Mutex` manual | replace with usefrom |
| AUD-23 | `web` demo commit + `ai-string-size` | allow |
| AUD-26/27/28/29/31 | minor cleanup | done |
| AUD-30 | unused `open` in FFI | remove |
| AUD-32 | obsolete tmdb | update |
| AUD-33 | framework repo: meta | done |

### AUD-09
- `delete_by_filter`, `count`, `similar_to_key` — despect given CLI-only (removed de public SDK). Ver `2026-07-28-sdk-gap-audit.md`.

### AUD-24/25
- AUD-24: coordenadas × 3 SI (loop unroll). ✅
- AUD-25: dead cli flags → cleanup.

### AUD-010? nota — `docs/historial/autopsias-2026-06-19.md`

### AUD-020: Tests HTTP auth/RBAC/rate-limit — ✅ 2026-08-11
- `cargo test -p vantadb-server --test server` → 19/19; root cause: helpers mandaban `{"query":"test"}`/`SELECT 1` (IQL inválido → 400 correcto post-ERR-027); fix: `SELECT * FROM Node`. RBAC vía `token_role_map` ya conectado. Backlog row removido; registrado en progreso.

---

## Auditoría de bindings (IA)

- **VFY-002:** CDV->VitaBuf — objeto CODeXCE (Jobs) deprecado; fix definitivo del vm NO completado. Ver `historial/autopsias-2026-06-19.md`.

## SEC-01 / SEC-02: Security audit FFI/bindings (P8-06)
- **Fecha:** 2026-07-28
- **Resultado:** ✅ SEC-01 (auditoría de FFI del core con PyO3/WASM) y SEC-02 (supply chain: dependencias con `unsafe` no auditadas) pasan gate; fixes aplicados en batch CODE-036/056/057/058/061/062/063.

### Batch SEC (detalle)
| ID | Tarea | Commit |
|---|---|---|
| CODE-036 | TLS 1.3 only (relajado a 1.2) | `df1479a` |
| CODE-056 | Duplicate reqwest 0.12+0.13 | `df1479a` |
| CODE-057 | debug=0 en test profile | `df1479a` |
| CODE-058 | Ignored advisories sin rationale | `df1479a` |
| CODE-061 | SIGBUS handler no signal-safe | `df1479a` |
| CODE-062 | Cursor reset sin zero-fill | `df1479a` |
| CODE-063 | grow_to puede shrink | `df1479a` |

---

## UnSafe & FFI review

### INV-024: Auditoría de unsafe (2026-06-19)
- **Fecha:** 2026-06-19
- **Resultado:** ✅ Audit MISRA-like de `unsafe` blocks. Hallazgos en `docs/historial/autopsias-2026-06-19.md`.

### Fuzz (DRV-133)
- **Resultado:** Fuzz de WAL/Disk persistence (chaos) — estado en Backlog fase chaos.

### AUDIT-01: Fix UAF PyO3 `__array_interface__` (release-blocker)
- **Fecha:** 2026-08-05
- **Resultado:** ✅ Fix de use-after-free en `__array_interface__` Python (release-blocker). Detalle en snapshot-2026-08-07.

### AUDIT-02: Sparse hot-path micro-opt (gate de medición) — WONTFIX
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Declarado WONTFIX — gate de medición. Ver `decisiones/wontfix.md`.

### P13 Audit Report: AUDREP-01, AUDREP-04, AUDIT-03
- **Fecha:** 2026-08-05
- **Resultado:** ✅ Reporte de auditoría (Miri/correctness). Detalle en snapshot-2026-08-07.

### AUD-020: Tests HTTP auth/RBAC/rate-limit en vantadb-server
- **Fecha:** 2026-08-11
- **Resultado:** ✅ 9 tests rotos por ERR-027 arreglados (query inválido → 400; fix: `SELECT * FROM Node`) + 4 tests RBAC HTTP nuevos (reader 403, writer/admin 200, reader GET /metrics 200). `cargo test -p vantadb-server --test server` = 19/19. Commits `90f85d9f`, `24a15cdf` (fmt drift).

---

## Prevención de breaking changes

- `cargo semver-checks` como gate pre-publish obligatorio (vanta-lead).

## Fuentes
- `docs/Backlog.md` fases security/audit.
- `docs/historial/autopsias-2026-06-19.md` (AUD-01..44).