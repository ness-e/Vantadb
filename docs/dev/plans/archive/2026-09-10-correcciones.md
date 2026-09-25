# Plan de Ejecución: Correcciones 2026-09-10 — 8 High + 18 Medium

> **Campaign ID:** 3d4e5f6a-7b8c-9d0e-1f2a-3b4c5d6e7f01
> **Inicio:** 2026-09-10
> **Estado:** ⬜ PENDIENTE (plan listo, sin iniciar)
> **Fuente:** docs/dev/Backlog-correcciones.md (derivado de review 19 módulos 2026-09-10)
> **Autonomous:** false
> **FAIL_MODE:** `parallel` (MAX 3; secuencial interno si colisionan archivos)
> **SPEC:** no existe SPEC.md — 0 greenfield (todo fix/docs/tests sobre trabajo verificado hoy).
> **SDP:** triage read-only + base; `campaign_discover_skills_v2` fino por tarea en pipeline-full Paso 0b. Skills base: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail (full), spec-driven-development, idea-refine.
> **Selección owner:** 26 DO aprobadas vía `question` (Recomendada) 2026-09-10.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 26 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 1 (FIND-88 alcance output-side) · ⬇️ downhill = 25

## Gate P — confirmación usuario (2026-09-10)

Triage + Paso 0 + `question` → owner aprobó "Aprobar plan 26 (Recomendado)". FIND-74 re-scopeada (README ya existe).

## Verificación real global (Paso 0, 2026-09-10)

- FIND-63: `desktop/vitest.config.ts` existe; 13 fails `localStorage` verificados en review.
- FIND-64: `ci-rust-10.yml:6-19` paths sin `vanta-memory/` (solo `vantadb-*/**`) — gap confirmado por lectura directa.
- FIND-65: `cache.rs:760-763` resta `Instant - 10_000s` — código leído, overflow real en hosts jóvenes.
- FIND-66: `Formula/README.md:23,40,52` — las 3 afirmaciones falsas leídas.
- FIND-67: `QUICKSTART.md:12,90` — v0.4.x + wheel 0.1.1 leídos vs tag v0.5.0.
- Resto (68-88): evidencia file:línea + comando en `docs/dev/reviews/archive/review-full-20260910-modulos.md` + apéndice (no re-verificado hoy por economía; cada ejecutor re-verifica en DISCOVERY y reporta divergencia como HALLAZGO).

## Tasks

### Wave0 — tests JS (disjuntos: desktop, ts, node-docs)

**Task 1: FIND-63 — Vitest desktop localStorage**
- Appetite 4h · 🟢 · 🔴 Alta · `desktop/vitest.config.ts`, `desktop/src/store/undo.test.ts`
- Contrato: `npx vitest run` desktop 86/86 + tsc 0. Pre-mortem: jsdom rompe otros tests (aplicar solo a archivos afectados); fake timers colisionan.
- Stop: fix contamina otros tests → storage mock por archivo. Risk: 🟡×🟢 scope mock.
- Task file `docs/dev/tasks/FIND-63.md` · ⬜ PENDING · Ruta vanta-worker. DoD task/commit/release §plan. ShapeUp sí/sí/sí.

**Task 2: FIND-79 — Tests export/import/reindex TS + importRecords estricto**
- Appetite 1d · 🟡 · 🟡 · `vantadb-ts/src/`
- Contrato: `npx vitest run` con tests nuevos export/import/reindex + `importRecords` sin try/catch blando + tsc/eslint 0.
- Pre-mortem: filesystem en WASM tests (paths temporales); round-trip real puede exponer bug (bienvenido). Stop: FS inviable → test con stubs + DEFER e2e.
- Task file `docs/dev/tasks/FIND-79.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 3: FIND-78 — Node README link + nota engines**
- Appetite 2h · 🟢 · 🟢 · `vantadb-node/README.md`
- Contrato: 0 links rotos (muestreo total del README) + nota compat node>=18 vs ts>=22.19.
- Task file `docs/dev/tasks/FIND-78.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave1 — Rust tests + MCP + docs gap (disjuntos)

**Task 4: FIND-65 — TTL test overflow**
- Appetite 2h · 🟢 · 🟠 · `vanta-proxy/src/cache.rs:760-763`
- Contrato: `cargo test -p vanta-proxy --test <suite> -j 2` verde en host joven + prod intacto + clippy 0. Fix: `checked_sub` o duraciones pequeñas.
- Task file `docs/dev/tasks/FIND-65.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 5: FIND-77 — Comentarios conteo MCP 76→79**
- Appetite 1h · 🟢 · 🟢 · `vantadb-mcp/src/handlers/tools.rs:1090`, `config.rs:14`
- Contrato: grep 76 ausente en comentarios de conteo + `cargo check -p vantadb-mcp` 0.
- Task file `docs/dev/tasks/FIND-77.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 6: FIND-76 — jwt_secret docs + link roto HTTP_API**
- Appetite 2h · 🟢 · 🟡 · `docs/user/operations/CONFIGURATION.md`, `docs/api/HTTP_API.md:600`
- Contrato: `pwsh scripts/validate-docs-coverage.ps1` EXIT 0 + link verificado con Test-Path.
- Task file `docs/dev/tasks/FIND-76.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave2 — docs grandes (disjuntos)

**Task 7: FIND-67 — QUICKSTART a 0.5.0**
- Appetite 4h · 🟡 · 🟠 · `docs/user/QUICKSTART.md`
- Contrato: boundary 0.5.0 + wheel path real + revalidación corriendo el quickstart + `last_reviewed` actualizada.
- Task file `docs/dev/tasks/FIND-67.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 8: FIND-68 — docs/api/PROXY.md + config ejemplo**
- Appetite 1d · 🟡 · 🟡 · `docs/api/PROXY.md` (nuevo), `vanta-proxy/config.toml`
- Contrato: endpoints + 8 features + defaults + env documentados; `master-index.md` enlaza; Regla 11 (0 claims sin fuente).
- Pre-mortem: doc diverge del código al mes (generar checklist de sync en el task file). Stop: scope a endpoints+features (no tutorial).
- Task file `docs/dev/tasks/FIND-68.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 9: FIND-74 re-scopeada — requirements + enlaces + TS**
- Appetite 4h · 🟢 · 🟢 · `examples/demo/requirements.txt`, `docs/user/QUICKSTART.md`, `README.md`
- Contrato: requirements `vantadb-py>=0.5.0` + 1 línea QUICKSTART→examples + 1 línea README→demo/colab + decisión TS documentada (mover o referenciar).
- Task file `docs/dev/tasks/FIND-74.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave3 — packaging/docs Rust (disjuntos)

**Task 10: FIND-66 — Formula README sync**
- Appetite 2h · 🟢 · 🟠 · `Formula/README.md`
- Contrato: quitar mcp no instalado (o añadirlo al install), ARM64 ✅, quitar `--head` (o añadir stanza); sin Ruby en runner → validación por lectura + `git diff` revisado.
- Task file `docs/dev/tasks/FIND-66.md` · ⬜ PENDING · Ruta vanta-docs.

**Task 11: FIND-75 — WASM README bundle + zero-copy input**
- Appetite 4h · 🟢 · 🟢 · `vantadb-wasm/README.md`, `vantadb-wasm/src/lib.rs:2378`
- Contrato: tamaño/fecha re-medidos + input zero-copy implementado o DEFER-ratificado con evidencia + clippy 0.
- Task file `docs/dev/tasks/FIND-75.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 12: FIND-81 — Server higiene**
- Appetite 2h · 🟢 · 🟢 · `vantadb-server/`
- Contrato: `vanta_certification.json` movido/borrado con justificación + `vantadb_data/` en gitignore + mini README 5 líneas.
- Task file `docs/dev/tasks/FIND-81.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave4 — integrations + providers (disjuntos)

**Task 13: FIND-69 — dspy fallback**
- Appetite 2h · 🟢 · 🟡 · `integrations/dspy/vantadb_dspy/vectorstore.py:62`
- Contrato: `pytest integrations/dspy/tests/` verde sin `dspy` instalado + con instalado (si hay) + try/except TypeError.
- Task file `docs/dev/tasks/FIND-69.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 14: FIND-84 — pins + fixtures + dist/PyPI integrations**
- Appetite 1d · 🟡 · 🟡 · `integrations/*/pyproject.toml`, tests, README central
- Contrato: upper-bounds en 7/9 + fixtures sin subdir inexistente + decisión `dist/` + PyPI/Alpha documentada + pytest verde (mocks).
- Task file `docs/dev/tasks/FIND-84.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 15: FIND-73 — pyi + READMEs providers**
- Appetite 4h · 🟢 · 🟡 · `providers/*/[*.pyi, README]`, `verify_pyi.py`
- Contrato: `key` en 3 `.pyi` + `embed_batch` documentado + `verify_pyi.py` chequea firmas (`inspect.signature`) + README ollama corregido.
- Task file `docs/dev/tasks/FIND-73.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave5 — benches + fuzz + bench py (disjuntos)

**Task 16: FIND-70 — ingestion_concurrent flag**
- Appetite 2h · 🟢 · 🟡 · `heavy-bench-nightly-51.yml`, `Cargo.toml:240`
- Contrato: nightly pasa `--features async-ingestion` o documenta skip + BENCHMARKS.md coherente.
- Task file `docs/dev/tasks/FIND-70.md` · ⬜ PENDING · Ruta vanta-lead (CI).

**Task 17: FIND-80 — fuzz corpus + docs**
- Appetite 1d · 🟡 · 🟢 · `fuzz/corpus/`, `fuzz-40.yml`, `docs/dev/workflow/fuzz-40.md`
- Contrato: seed mínimo commiteado + upload-artifact corpus/crashes + docs actualizadas (ci-gate + fuzz-pr) + `cargo check --manifest-path fuzz/Cargo.toml --bins` 0.
- Task file `docs/dev/tasks/FIND-80.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 18: FIND-72 — bench py CLI + pins + Chroma**
- Appetite 1d · 🟡 · 🟢 · `benchmarks/batch_vs_sequential_bench.py`, `requirements.txt`, `competitive_bench.py`
- Contrato: `--help` no ejecuta bench + pins + retry/fix WinError32 documentado + `py_compile` 0.
- Task file `docs/dev/tasks/FIND-72.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave6 — embeddings + python + memory (disjuntos)

**Task 19: FIND-71 — embeddings peso + verify + sizes**
- Appetite 1d · 🟡 · 🟡 · `embeddings/download.py`, `verify.py`, `README.md`
- Contrato: `ALLOW_PATTERNS` recortado + `verify`→smoke aclarado + sizes reales + `download --check` verde.
- Task file `docs/dev/tasks/FIND-71.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 20: FIND-85 — python CI + limpieza + firma**
- Appetite 1d · 🟡 · 🟡 · `release-wheels-60.yml`, `vantadb-python/`
- Contrato: matriz 3.12/3.14 o classifiers recortados + `probe_lock_db/` en gitignore + firma unificada o documentada + pytest 139 verde.
- Task file `docs/dev/tasks/FIND-85.md` · ⬜ PENDING · Ruta vanta-lead (CI) + worker.

**Task 21: FIND-86 — memory diferidos**
- Appetite 2d · 🟡 · 🟡 · `vanta-memory/`
- Contrato: MEM-69 wiring + tool 77 diseñada o implementada + MEM-70 números o DEFER-ratificado + suite 336 verde + clippy 0.
- Pre-mortem: scope triple → orden wiring → tool → números; cualquiera tranca → shippear resto. Stop: 1 sub-item trancado no bloquea otros.
- Task file `docs/dev/tasks/FIND-86.md` · ⬜ PENDING · Ruta vanta-worker.

### Wave7 — ts + proxy + skills docs (disjuntos)

**Task 22: FIND-87 — native exports + wiki note**
- Appetite 4h · 🟢 · 🟡 · `vantadb-ts/package.json`, `docs/api/TS_SDK.md`
- Contrato: `./native` en exports (o decisión documentada) + nota wiki explícita + build/test verdes.
- Task file `docs/dev/tasks/FIND-87.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 23: FIND-88 — proxy cost output + simétrico**
- Appetite 2d · 🟠 · 🟢 · `vanta-proxy/src/server.rs`
- Contrato: `record_response_usage` al SSE drain (o DEFER-ratificado) + simétrico diseñado o implementado + suite verde + clippy 0. Uphill: alcance output-side.
- Task file `docs/dev/tasks/FIND-88.md` · ⬜ PENDING · Ruta vanta-worker.

**Task 24: FIND-83 — skills unify**
- Appetite 1d · 🟡 · 🟠 · `skills/`, `.opencode/skills/`, `SKILLS-MANIFEST.md`
- Contrato: copias sincronizadas con hash gate + MCP-27/29 unificadas + api-ref completa + manifest al día.
- Task file `docs/dev/tasks/FIND-83.md` · ⬜ PENDING · Ruta vanta-docs.

### Wave8 — CI + skills count (disjuntos)

**Task 25: FIND-64 — CI paths memory**
- Appetite 1h · 🟢 · 🔴 · `.github/workflows/ci-rust-10.yml:6-19`
- Contrato: `'vanta-memory/**'` en paths push+PR + actionlint/yaml parse OK.
- Task file `docs/dev/tasks/FIND-64.md` · ⬜ PENDING · Ruta vanta-lead (CI).

**Task 26: FIND-82 — skills count assert**
- Appetite 4h · 🟡 · 🔴 · `skills/vantadb-mcp/scripts/test-mcp.py`
- Contrato: script aserta conteo por perfil + buildeado desde fuente documentado + handshake 4/4 verde.
- Task file `docs/dev/tasks/FIND-82.md` · ⬜ PENDING · Ruta vanta-worker.

## SKIP / DEFER / BLOQUEADO
Ninguno (todo verificado real, esfuerzo acotado, sin dependencias bloqueantes).

## Grafo de dependencias / Waves (FAIL_MODE=parallel, MAX 3)
```
Wave0: FIND-63 + FIND-79 + FIND-78
Wave1: FIND-65 + FIND-77 + FIND-76
Wave2: FIND-67 + FIND-68 + FIND-74
Wave3: FIND-66 + FIND-75 + FIND-81
Wave4: FIND-69 + FIND-84 + FIND-73
Wave5: FIND-70 + FIND-80 + FIND-72
Wave6: FIND-71 + FIND-85 + FIND-86
Wave7: FIND-87 + FIND-88 + FIND-83
Wave8: FIND-64 + FIND-82
```
Nota runner: si waves ×3 abortan, fallback secuencial por wave (precedente 2026-09-09/10).

## Riesgos globales
| Riesgo | Respuesta |
|--------|-----------|
| Mismo archivo en una wave | waves separan por área; si colisiona → secuencial interno |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO (5 reportes) |
| rustc 1.95 crash paralelo / OOM | `-j 2` siempre en cargo |
| validate-docs-coverage roto | FIND-76 lo cubre (jwt_secret); resto manual |

## Notas
- plan-adjust template: `plan-adjust [YYYY-MM-DD]: <ID> — qué cambió · ⬆️ antes/después · ⬇️ antes/después`.
- SKILLS_CARGADAS: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail (full), spec-driven-development, idea-refine.
- Herramientas: `cargo test/check/clippy/fmt -p <crate> --tests -j 2`, `npx vitest run`/`tsc`/`eslint`, `npm pack --dry-run`, `cargo fuzz list`, `Test-Path`/`Select-String`, `question` (gates D/V/C).

=== RECITATION 7 ===
Campaign ID: 3d4e5f6a-7b8c-9d0e-1f2a-3b4c5d6e7f01
Objetivo activo: FIND-67 QUICKSTART a 0.5.0
Estado: pending
Última acción: Reversión: marcado completed erróneo por colisión de taskId con campaña anti-stutter; sin trabajo realizado aquí
Resultado: ❌
Próxima acción: FIND-67 queda PENDING para su campaña
Contrato: pendiente de ejecución
Próxima tarea si completa: FIND-67
=== END RECITATION ===
