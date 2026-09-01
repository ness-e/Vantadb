# Plan de Ejecución: Fast Gate Green (lint cascade + mcp tests)

> **Inicio:** 2026-09-01
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/Backlog.md` (110 items — triage completo 2026-09-01, reality-check ejecutado hoy)
> **Autonomous:** false
> **Campaign ID:** (se asigna al claim)
> **SDP:** campaign-executor, ponytail (full), progreso, test-driven-development, systematic-debugging (bugs), source-driven-development (símbolos) — phase=PLAN

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO     | 2     |
| 🟡 DEFER  | ~85   |
| ❌ SKIP   | ~12   |
| 🔴 BLOQUEADO | 8  |

**Objetivo del plan:** dejar `just verify` / pre-push / CI Fast Gate en verde absoluto. Dos fallas verificadas-vivas hoy rompen el gate tras el split REVIEW-10 y los commits recientes de MCP:

1. **FIND-035** — clippy rojo en `-p vantadb --all-targets --all-features` (5 unused imports en `src/server/routing.rs` + 1 `assertions_on_constants` en `src/config.rs:1767` + cascada lib-test). Verificado vivo 2026-09-01: `cargo clippy ... -D warnings` → 8 errores.
2. **FIND-036** — `cargo check -p vantadb-mcp --tests` rojo en `test_embed_texts.rs` (3 errores: call con 0 args + `max_embed_tokens` + `max_embed_batch_size` inexistentes en `McpConfig`). Verificado vivo 2026-09-01. **La falla se amplió** respecto de la fila original del Backlog: ahora son 3 errores, no 1.

### SKIP verificados hoy (premisas falsas contra código real — regla 1 del gate)

| ID | Evidencia |
|----|-----------|
| FIND-44 "Sin ADRs registrados" | ❌ STALE — `(Get-ChildItem docs/architecture/adr/ADR-*.md).Count` == **22** |
| TS-01 tipos grafo ficticios | ❌ COMPLETADA 2026-08-28 (contract satisfecho, `types.ts:237 GraphBfsResult = bigint[]` confirmado hoy) — fila Backlog stale |
| TS-02 `_native` async | ❌ STALE — `vantadb-ts/src/native.ts:148` ya es `private async _native<T>(...)` |
| FIND-24 list 10k debug | ❌ ya resuelto con cursor+skip+limit (decisión triage 2026-08-31, re-verificada) |
| SRV-01 / WSM-02 / RES-13 / FIND-23 | ❌ ya implementado (triage previo 2026-08-31, sin cambios desde entonces) |
| AUD-043 / FIND-MCP-001 / TBH-06 / RES-11 / MCP-37 / MCP-39 / PY-01 / FIND-24b / FIND-40 / SRV-04 | ✅ completadas 2026-08-31→2026-09-01 (filas Backlog se limpian al archivar planes vía progreso Trigger 1) |

### 🔴 BLOQUEADO (persisten, sin cambio upstream)

AUD-042 (tantivy ≥0.27.0 no publicada), CORE-02 (PITR — requiere ADR + decisión owner), FIND-33 (snapshot layout — requiere ADR vanta-arch), STABLE-01..09 (decisión owner ADR-031 + medición), MCP-34/34b (depende FIND-33), BND-08..10 (estrategia npm post-launch), SRV-06 (OIDC DISCOVERY vanta-arch), TS-10/11/WSM-06 (core expose wiki/skills).

### 🟡 DEFER (sin cambio vs triage 2026-08-31)

P5/P6/P8 (docs community, launch, enterprise), P24 (I+D), P25-P28 (cognitiva/TDAM/deuda core follow-ups), P32-P34 (reviews módulos, DX, UX Studio), P38-P47 (investigaciones huérfanas, proxy, INV-*, STABLE, desktop), AUD-045 (requiere baseline Regla 9), REVIEW-12 (god-file api.rs 🟡), FIND-38/41/43/47 (batch refactor), TS-03..13 (viven en plan 2026-08-25-research-vantadb-ts-quickwins propio), RES-02..15, MEM-63..70, PROV-*, PRX-*, WEB-*, MKT-*.

---

## Shape Up aplicado

- ¿Problema correcto? Sí — el fast gate verde es prerrequisito de TODO trabajo posterior (Regla 1: sin verify no hay push legítimo).
- ¿Appetite correcto? Sí — ambas 🟢 (<1h), verificadas contra código real hoy.
- ¿Es AHORA? Sí — FIND-035 rompe `just verify` en CADA push actual (el pre-push de hoy pasó solo por `--no-verify` implícito del barrier al no haber .rs staged — el gate está roto para el próximo commit Rust). FIND-036 bloquea `cargo check --workspace --tests` y el camino STABLE-04.

## Tasks

### Task 1: FIND-035 — Fix lint cascade clippy (routing.rs unused imports + config.rs assertions_on_constants)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 ~30 min
- **Prioridad:** 🔴 Alta (rompe `just verify` / pre-push / CI Fast Gate para cualquier commit Rust)
- **Archivos clave:** `src/server/routing.rs:11,12,13,17,42,57`, `src/config.rs:1767`
- **Verificación real:** ✅ CÓDIGO-REAL 2026-09-01 — `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → 8 errores: `unused imports: REQUEST_ID_HEADERS, REQUEST_ID_MAX_LEN, simple_url_decode` (línea 11-13), `AuditLogger` (17), `parking_lot::Mutex` (42), `RbacConfig` (57), `ConversationTrigger` (12), + `clippy::assertions_on_constants` en `config.rs:1767` (`assert!(MAX_K >= 1_000)` con const) + 3 cascada lib-test. Blast radius: imports puros — borrar línea no cambia comportamiento.
- **Gate Justificación:** único blocker real del fast gate en crate core; 6 líneas de imports + 1 assert; introduced por REVIEW-10 split (imports traídos que quedaron sin uso en routing.rs).
- **Contrato:** `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` exit 0 **AND** `cargo check -p vantadb` exit 0
- **Pre-mortem:**
  - Fallo 1: un "unused" import es usado tras `#[cfg(feature=...)]` condicional → verificar con `--all-features` (ya incluido en contrato) y grep antes de borrar
  - Fallo 2: `const { assert!(..) }` no compila en MSRV 1.94.1 (inline const asserts estables en 1.79 ✅ — OK, pero validar MSRV)
  - Fallo 3: lib-test cascada no desaparece sola → re-corre tras fix de lib
- **Stop conditions:** un import resulta ser usado bajo algún feature-set → re-verificar con cfg-guard documentado en vez de borrar; clippy introduce warning nuevo → rollback + FIND-037
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟡 | Import usado bajo cfg específico | grep `#[cfg]` + `--all-features` verify | pre-delete |
  | 🟢×🟢 | const-assert cambia semántica de test | `cargo test -p vantadb config` post-fix | post-fix |
- **Cynefin:** 🟦 Obvio — fix mecánico de lint
- **Top 3 riesgos:** (1) falso unused por cfg; (2) MSRV del const-block; (3) warnings nuevos en cascada
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2 (borrar imports, fix assert)
- **DoD multi-nivel:** Task: contrato clippy exit 0 · Commit: `fix(lint): drop unused imports post-REVIEW-10 split + const assert in config (FIND-035)` + verify full · Release: N/A (cero API change)
- **Validación Appetite vs Effort:** max 1h ≥ 🟢 30min ✅
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-035.md`
- **Branch:** develop
- **Ruta:** vanta-worker

---

### Task 2: FIND-036 — Fix compile de `vantadb-mcp/tests/test_embed_texts.rs` (3 errores vs McpConfig)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 ~30-60 min
- **Prioridad:** 🔴 Alta (bloquea `cargo check --workspace --tests` — pre-push barrier completo y STABLE-04)
- **Archivos clave:** `vantadb-mcp/tests/test_embed_texts.rs:78` (+ línea del call E0061), `vantadb-mcp/src/config.rs` (`McpConfig`)
- **Verificación real:** ✅ CÓDIGO-REAL 2026-09-01 — `cargo check -p vantadb-mcp --tests` → 3 errores E0061 (function takes 1 argument, supplied 0), E0609 (`no field max_embed_tokens`), E0609 (`no field max_embed_batch_size`). Ampliado vs fila original del backlog (era 1 error; MCP-39 agregó `byte_budget` a McpConfig pero los campos embed_* nunca existieron). Falta decidir: ¿agregar campos a `McpConfig` o corregir el test al config real — DISCOVERY debe leer `test_embed_texts.rs` completo + git log del test primero.
- **Gate Justificación:** bug de test pre-existente que se amplió; único blocker restante de `--workspace --tests`; prerequisite de promoción STABLE-04 (vantadb-mcp) y del pre-push completo.
- **Contrato:** `cargo check -p vantadb-mcp --tests` exit 0 AND `cargo test -p vantadb-mcp --test test_embed_texts` exit 0 (o `--ignored` documentado con理由 si el test requiere provider externo)
- **Pre-mortem:**
  - Fallo 1: el test fue escrito contra un config que NUNCA existió (test zombie de una rama abandonada) → el fix correcto es agregar los campos o #[ignore] + FIND nuevo; decidir con evidencia de git log
  - Fallo 2: el E0061 (0 args) es de un helper con firma cambiada por MCP-39 → alinear call site, no re-crear API vieja
  - Fallo 3: los campos embed_* requieren plumbing real hacia el embedder → si es feature no terminada, NO implementar: DEFER con fila nueva
- **Stop conditions:** descubrir que `max_embed_*` implica plumbing de provider nuevo (>1h) → cerrar como 🟡 DEFER con hallazgo documentado; rollback si tocar config.rs rompe otros tests
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | Campos embed_* son feature no terminada | DEFER + FIND nueva, no media-implementación | DISCOVERY |
  | 🟢×🟡 | Fix test enmascara API roto | leer git log del test + de McpConfig antes | pre-fix |
- **Cynefin:** 🟨 Complicado-lite — la decisión (test mal vs config incompleto) requiere leer historia; el fix es acotado
- **Top 3 riesgos:** (1) implementar feature disfrazada de fix; (2) test zombie ignorado silenciosamente; (3) regresión en otros tests mcp
- **Uphill/Downhill:** ⬆️ 1 (decidir test-vs-config por git log) · ⬇️ 2
- **DoD multi-nivel:** Task: contrato exit 0 · Commit: `fix(vantadb-mcp): align test_embed_texts with McpConfig (FIND-036)` + verify full · Release: N/A
- **Validación Appetite vs Effort:** max 1h ≥ 🟢 ✅ (con stop condition a DEFER si excede)
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-036.md`
- **Branch:** develop
- **Ruta:** vanta-worker

---

## Dependencias entre Tasks

Ninguna entre Task 1 y 2 (archivos disjuntos: `src/server/`+`src/config.rs` core vs `vantadb-mcp/`). Ejecutables en paralelo (Wave 0, 2 sub-agentes, MAX_CONCURRENT=2) o secuenciales — FAIL_MODE default `stop`.

**Ojo — mismo sub-agente no debería tocar ambas si corren en paralelo** (evitar conflictos en Cargo.lock/registry). Si FAIL_MODE=parallel: Task1 (core crate) y Task2 (mcp crate) usan `-p` distintos → sin solapamiento de archivos. Safe.

## Checkpoint post-plan

Tras ambas tareas: `cargo clippy --workspace --all-targets --all-features -- -D warnings` exit 0 + `cargo check --workspace --tests` exit 0 → **fast gate completo en verde** → habilita `/audit quick` limpio y desbloquea STABLE-04 hacia la decisión owner de default-members.

## Notas

- **Reality-check 2026-09-01 ejecutado sobre código actual** (no texto del backlog): clippy/mcp-check reales, 22 ADRs contados, `types.ts` y `native.ts` leídos. 3 filas "🔴 Alta Pendiente" resultaron premisa-falsa (FIND-44, TS-01, TS-02).
- **Filas Backlog de tareas completadas** (AUD-043→SRV-04, ~10 filas): la limpieza ocurre vía `progreso` Trigger 1.C al archivar cada plan — no se tocan acá (decisión usuario 2026-08-31).
- **`.opencode` es ahora submodule** (configOpencode, e48985fb) — los task files nuevos se commitean en el submodule, no en el repo padre. Regla 0 aplica igual.
- FIND-035/036 nacieron como derivadas de las campañas 2026-08-31 (fast-gate-residues). Este plan las convierte de "fila de backlog" a DO con contrato verificado vivo.

## Context Save Point

- **Fecha:** 2026-09-01
- **Branch:** develop
- **Estado:** ⏳ EN PROGRESO (plan creado, 0/2 tareas)
- **Próxima tarea:** Task 1 (FIND-035) — lint core, ejecutar primero (desbloquea push para todo lo demás)
- **Decisiones registradas:**
  - Gate P aprobado por usuario: 2 DO + resto DEFER/SKIP/BLOQUEADO igual que triage previo
  - TS-03..13 NO se suman (viven en su propio plan quickwins)