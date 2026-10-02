---
title: "API-09: W8 cierre — VERSIONING 11 superficies + docs/api sincronizadas + gates de cierre"
kind: task
description: "VERSIONING.md lista 11 superficies Y scripts/validate-docs-coverage.ps1 verde Y dev-tools/verify.ps1 verde Y MCP re-smoke verde Y dev-tools/ocr-review.ps1 sin Critical/High Y plan 18/18 + este 9/9 para /ship\""
---

# API-09: W8 cierre — VERSIONING 11 superficies + docs/api sincronizadas + gates de cierre

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 9 · investigación `docs/dev/plans/2026-09-24-api-estandarizacion.md` (API-STD-01/15/17)
- **Fuente:** Backlog Phase 51 (fila `API-09`) — **última tarea de la campaña**
- **Esfuerzo:** 🟢 2d (tool estimate: 10-15 turns)
- **Prioridad:** 🟡
- **Tipo:** Docs (+ gates de cierre; sin código de producto)
- **Turns estimados:** 10-15
- **Creado:** 2026-09-26
- **last-synced:** 2026-09-26
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 6/6; commit local pendiente del LEAD (política owner 2026-09-25).
- **Incógnitas (uphill):** 0 abiertas (Gate P cerró las 4 🔴 en API-STD-15)
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Enlazan a `VERSIONING.md`: `TS_SDK.md:13`, `PYTHON_SDK.md:13`, `NODE_SDK.md:12` (§Stability), `docs/user/operations/UPGRADE.md:13`, `CONTRIBUTING.md`, `docs/dev/master-index.md:85`. `scores.md` ← `TS_SDK.md:312`, `WASM_API.md`, `MCP.md` |
| Callees | `docs/api/` (19 files), `scripts/validate-docs-coverage.ps1`, plan `api-ejecucion.md`, `docs/dev/Backlog.md`, `target/session-api01/` (smoke) |
| Implicaciones | Docs-only: sin cambios de código, sin migración, sin impacto en tests. Gates de cierre (`verify.ps1`, MCP smoke, OCR) se corren con `CARGO_TARGET_DIR=target\session-api01` (lock de `target/debug` por MCP local) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/api/VERSIONING.md` (64L), `scripts/validate-docs-coverage.ps1` (227L), `dev-tools/verify.ps1` (102L), `dev-tools/ocr-review.ps1` (78L), `vanta-mcp-local.ps1` (113L), `target/session-api01/smoke-api04.py` (201L), `CONSTRAINTS.md`, `docs/dev/tasks/API-STD-01.md` (11 superficies), `API-STD-15.md` (Gate P), `API-STD-17.md` (derivas+updates), plan ejecución (verificado: 8/9) y estandarización (verificado: 18/18). Parciales con secciones clave: `docs/api/scores.md` (score/distance), `docs/api/ERROR_HANDLING.md` (envelope), `docs/api/MCP.md` (canónicos), `docs/api/HTTP_API.md` (error responses), `src/sdk/serialization/vector_types.rs:69-76` (`MemorySearchHit.score`), `vantadb-mcp/src/handlers/tools.rs:3140-3143,1858-1913` (dispatch memory_search/search_semantic).
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** VERSIONING.md ← enlaces desde 3 SDK docs + UPGRADE + CONTRIBUTING + master-index; scores.md ← TS_SDK.md/WASM_API.md/MCP.md; ERROR_HANDLING.md ← enlaces de bindings/server docs.
- **Archivos que referencian a los editados (referencias entrantes):** `rg -l "VERSIONING"` → 10 archivos (arriba); `rg "scores.md"` en `docs/api` → TS_SDK/WASM_API; `rg "ERROR_HANDLING"` en docs/api → varios (no se renombra, solo se corrige 1 línea).
- **Veredicto impacto:** **bajo** — docs sin consumidores mecánicos directos, salvo `scripts/validate-docs-coverage.ps1` (no cubre VERSIONING/scores/ERROR_HANDLING: solo EMBEDDED_SDK/CONFIGURATION/PYTHON_SDK/MCP/mirror) y el review OCR del diff.

## Contrato

"`VERSIONING.md` lista 11 superficies Y `scripts/validate-docs-coverage.ps1` verde Y `dev-tools/verify.ps1` verde Y MCP re-smoke verde Y `dev-tools/ocr-review.ps1` sin Critical/High Y plan 18/18 + este 9/9 para `/ship`"

## Spec (docs — resoluciones por evidencia)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Fuente de la lista 11 superficies | A: tabla API-STD-01 (código-real, entrypoints) / B: re-derivar de cero | ✅ A — API-STD-01 §2 (11 filas con entry-point verificado) + Gate P API-STD-15 "Alcance 11" |
| 2 | Estabilidad de `vanta-memory` en VERSIONING | A: core-only Rust API (no binding) / B: prometer binding futuro | ✅ A — Gate P (API-STD-15 eje vanta-memory: "core-only + API Rust estable; exponer post-release con demanda") + API-08 cierre |
| 3 | Doc de contrato de la superficie CLI | A: crear `docs/api/CLI.md` / B: enlazar `CONFIGURATION.md` §4 + `--help` (los comandos se chequean ahí) | ✅ B — ponytail: la sección existe (API-07 la actualizó) y `validate-docs-coverage.ps1` ya la usa como doc de comandos; crear archivo nuevo duplica |
| 4 | RFC 9457 en HTTP (Gate P lo listó; W2 no lo implementó) | A: implementarlo (código — PROHIBIDO en esta tarea) / B: ajustar doc a la realidad + FIND-161 | ✅ B — evidencia: sin `problem+json` en `src/server/`; `HTTP_API.md:780` documenta `{success,error,hint}`; hallazgo → fila FIND (routing obligatorio) |
| 5 | `scores.md` fila MCP `search_memory`/`search_semantic` = `distance` | A: dejar / B: separar por tool (hybrid=score, ANN=distance) | ✅ B — evidencia: `dispatch_search_memory` → `embedded.search` → `MemorySearchHit.score` (`vector_types.rs:72-73`); `search_semantic` → convierte a `distance` (`tools.rs:1889-1908`, comentario remite a MCP.md); `search_multi` ya dice "sorted by score" (`MCP.md:291`) |
| 6 | Extender `validate-docs-coverage.ps1` a vanta-memory | A: hacerlo aquí / B: no (FIND-160 ya tiene dueño memory) | ✅ B — API-STD-17 §3: "no cubre vanta-memory" es conocido + FIND-160 (cobertura memory) ya ruteada; el contrato solo pide el script verde |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** NO tocar código de producto (`src/**`, `vanta-memory/**`, `vantadb-mcp/**`, `vantadb-server/**`, `vanta-proxy/**`, `vantadb-python/**`, `vantadb-wasm/**`, bindings) — si un gate falla por código → `BLOQUEO:`; NO editar `docs/CHANGELOG.md` ni versiones (release-plz dueño — pre-mortem F1); NO archivar planes (lo hace el LEAD); docs técnicas en EN (F2); gates siempre con `CARGO_TARGET_DIR=target\session-api01`; los binarios del MCP smoke salen de `target/session-api01/debug`.
- **Comandos de verificación:** `pwsh -NoProfile -File scripts/validate-docs-coverage.ps1` (exit 0) · `cargo build --target-dir target/session-api01 --bin vanta-cli` + `-p vantadb-server` · `python target/session-api01/smoke-api04.py` (SMOKE OK 11 checks) · `$env:CARGO_TARGET_DIR='target\session-api01'; dev-tools/verify.ps1` (ALL PASS) · `pwsh dev-tools/ocr-review.ps1 -Format json` (sin Critical/High).
- **Deuda pendiente:** FIND-161 nueva (RFC 9457 HTTP no implementado por W2 — doc ajustada); deuda previa intacta (FIND-155..160, FIND-79 DEFER, FIND-156/157, P2 restantes). No se introduce deuda técnica (docs-only).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|---------------------------|
| `activeGoal` | Encabezado `# API-09: W8 cierre …` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | ninguna (última de la campaña) — LEAD: `skill progreso` + archivado + `/audit quick` → `/ship` |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (docs-only). FIND-161 registrada como hallazgo de discovery (doc quedó truthful; implementación RFC 9457 pendiente con dueño sugerido server).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file: VERSIONING 11 filas + coverage verde + verify.ps1 verde + MCP re-smoke + OCR sin Critical/High |
| **Commit** | N/A — la política owner 2026-09-25 asigna el commit local al LEAD (subagente no commitea); changeset = docs only |
| **Release** | N/A — release-plz decide versiones/tags/changelog; esta tarea NO toca CHANGELOG ni versiones |

## Herramientas necesarias
- `codegraph_explore` (blast radius docs/scripts)
- `campaign_verify_cmd` (verify por step) + `campaign_update_task_state` / `campaign_memory_write` / `campaign_diagnose_pipeline`
- PowerShell + cargo/nextest + `ocr` CLI (instalado: `C:\Users\Eros\AppData\Roaming\npm\ocr.ps1`)

**Skills cargadas (SDP):** `documentation-and-adrs` (política versionado/ADR+docs), `ci-cd-and-automation` (gates de cierre + OIDC/secrets verificados), `source-driven-development` (cada claim doc↔código con `file:line`), `doubt-driven-development` (gate adversarial 🔴/cierre), `writing-guidelines` (estilo docs), `ponytail` (plugin, full→lite) — SDP sin candidatas adicionales.

## Investigation Notes

- **Estado base (verificado 2026-09-26, HEAD `05c4c490`):** `validate-docs-coverage.ps1` → 0 gaps ✅; plan `api-estandarizacion.md` → ✅ 18/18; plan `api-ejecucion.md` → 8/9 (API-09 pendiente); `docs/api/` = 19 files (18 md + 1 yaml); 27 workflows con publish OIDC tokenless (API-STD-17, re-verificado por contexto de campaña).
- **Derivas detectadas en discovery (2 + 1 colateral):** (1) `scores.md:66` fila MCP mezcla hybrid con ANN (ver Spec #5); (2) `ERROR_HANDLING.md:68` "RFC 9457 … in the HTTP wave" quedó stale: W2 (API-03) cerró sin `problem+json` (`rg` en `src/server/` = 0) → FIND-161 + línea truthful; (3) colateral (Step 3): `HTTP_API.md` §Error responses omitía el campo `code` que ERR-CORE-01 sí emite (`src/server/errors.rs:132-163`) → corregido + `last_reviewed` bump.
- **Derivas cubiertas por waves (no tocar):** TS_SDK/BINDINGS/PYTHON/NODE/WASM (W1 API-02), HTTP_API/openapi.yaml (W2 API-03), MCP.md (W3 API-04), PROXY.md (W4 API-05), IQL.md (W5 API-06), CONFIGURATION.md CLI (W6 API-07), VANTA_MEMORY.md (W7 API-08).
- **MCP re-smoke:** `smoke-api04.py` pinnea `target/session-api01/debug/{vanta-cli,vantadb-server}.exe` + `PATH` con BIN_DIR → re-smoke reproducible sin el lock de `target/debug`. El refresh del MCP local de OpenCode (rebuild `target/debug` + restart) queda como owner-action explícita (no bloquea si el resto del contrato está verde).
- **Bug de tooling detectado en Step 6 (pre-existente, no del cambio):** `dev-tools/verify.ps1:79` invocaba `cargo llvm-cov nextest run …`; cargo-llvm-cov trata `run` como **filtro de test** (35/2268 tests, TOTAL 9.5% → `--fail-under-lines 60` rojo). La invocación canónica (CI `ci-rust.yml:363` y `cargo llvm-cov nextest --help`) NO lleva `run`. Fix aplicado: quitado el token `run` (restaura la semántica del gate — no lo debilita). Re-verificación: coverage standalone + verify full re-run.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — Gate P cerró decisiones; derivas localizadas con evidencia |
| Pendientes de ejecución (downhill) | 8 — steps 1-8 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — NO aplica: docs-only, sin trust boundaries, sin auth, sin dependencias nuevas. El ajuste de docs de errores no cambia el contrato de seguridad. Justificado.
- [x] **PERFORMANCE** — NO aplica: sin hot paths, sin código. Justificado.

## Steps

### Step 1: `VERSIONING.md` — tabla de las 11 superficies API
- **Archivos:** `docs/api/VERSIONING.md`
- **Acción:** Reescribir la sección "What counts as stable public API" → tabla de 11 superficies (fuente: API-STD-01 §2 + Gate P API-STD-15) con entry-point, doc normativa por superficie, marcadores de breaking; conservar pre-1.0/deprecation/release mechanics; agregar "Enforcement" (gates mecánicos existentes) y separar "On-disk format" como contrato de storage (no superficie API); `last_reviewed: 2026-09-26`. EN.
- **Verify:** `Select-String VERSIONING.md -Pattern '^\| \d+ \|'` → 11 filas; cada doc enlazado existe (`Test-Path` ×10 + CONFIGURATION + STORAGE_VERSIONING); `rg "11 public API surfaces"` presente.
- **Estado:** ✅ DONE (2026-09-26) — SURFACE ROWS=11; LINKS_BAD=0 (20 links chequeados); `last_reviewed` bump.

### Step 2: `scores.md` — sincronizar fila MCP (hybrid=score / ANN=distance)
- **Archivos:** `docs/api/scores.md`
- **Acción:** Separar la fila `search_memory/search_semantic` en dos: `memory_search` (hybrid, `score` higher-is-better, ex-`search_memory`) y `search_semantic` (raw ANN, `distance` lower-is-better); `last_reviewed: 2026-09-26`. Evidencia: `tools.rs:3140-3143` + `vector_types.rs:69-76` + `tools.rs:1858-1913`.
- **Verify:** `Select-String scores.md -Pattern "memory_search|search_semantic"` → 2 filas (hybrid=score / ANN=distance); `rg "search_memory`/`search_semantic" scores.md` sin fila combinada.
- **Estado:** ✅ DONE (2026-09-26) — filas 66-67 separadas; trailing space corregido; probe live (Step 5) confirma `score`/`distance` en el wire.

### Step 3: `ERROR_HANDLING.md` truth + FIND-161 (RFC 9457 HTTP pendiente)
- **Archivos:** `docs/api/ERROR_HANDLING.md`, `docs/dev/Backlog.md`
- **Acción:** Línea 68: reemplazar "RFC 9457 … in the HTTP wave" por estado real (envelope `{success,error,hint}` documentado en `HTTP_API.md` §Error responses; RFC 9457 no implementado por W2) + fila `FIND-161` en Backlog (routing de hallazgo; dueño sugerido server). `last_reviewed: 2026-09-26`. NO tocar código.
- **Verify:** `rg "FIND-161" docs/dev/Backlog.md` → 1 fila; `rg "problem\+json|RFC 9457" docs/api/ERROR_HANDLING.md` → statement truthful sin "HTTP wave"; `git diff --stat` solo docs.
- **Estado:** ✅ DONE (2026-09-26) — FIND-161 registrada; **deriva 2b**: `HTTP_API.md` §Error responses no documentaba `code` (ERR-CORE-01) → corregido + `last_reviewed` bump; solo docs tocados.

### Step 4: Docs-coverage gate
- **Archivos:** — (solo ejecución)
- **Acción:** Correr el gate de cobertura de docs.
- **Verify:** `pwsh -NoProfile -File scripts/validate-docs-coverage.ps1` → exit 0, "0 gaps".
- **Estado:** ✅ DONE (2026-09-26) — exit 0, 0 gaps (8 bloques ok).

### Step 5: Rebuild session + MCP re-smoke (11 checks)
- **Archivos:** `target/session-api01/` (binarios + smoke)
- **Acción:** `$env:CARGO_TARGET_DIR='target\session-api01'; cargo build --bin vanta-cli` + `cargo build -p vantadb-server`; correr `python target/session-api01/smoke-api04.py`; probe extra `memory_search`→campo `score` / `search_semantic`→`distance` (`target/session-api01/probe-api09.py`) como evidencia del Step 2.
- **Verify:** build exit 0; `SMOKE API-04 OK — 11 checks passed`; probe OK.
- **Estado:** ✅ DONE (2026-09-26) — builds ok (32s + 1m20s); SMOKE 11/11; PROBE API-09 2/2 (`memory_search`→`score=1.0` sin distance; `search_semantic`→`distance=0.0`).

### Step 6: `dev-tools/verify.ps1` (full, scoped target session)
- **Archivos:** — (solo ejecución)
- **Acción:** `$env:CARGO_TARGET_DIR='target\session-api01'; pwsh -NoProfile -File dev-tools/verify.ps1`.
- **Verify:** `ALL N PASS` + exit 0 (fmt/check/clippy/audit/deny/nextest/coverage/docs-coverage/cli-probes/consumo/backup).
- **Estado:** ✅ DONE (2026-09-26) — r2 `ALL 11 PASS` + exit 0 (`api09-verify-r2.log`): fmt/check/clippy/audit/deny/nextest/coverage (81.63% ≥60)/docs-coverage/cli-probes/consumo/backup. 1ª corrida falló por bug del wrapper → fix `dev-tools/verify.ps1:79` (quitado `run`), sin debilitar el gate.

### Step 7: OCR delegation review (sin Critical/High)
- **Archivos:** — (solo ejecución + spec JSON)
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` → `ocr delegate rule <paths>` → revisar cada archivo con su Rule Group (`.opencode/references/ocr-review.md`).
- **Verify:** spec generado; Critical+High = 0 (Medium → FIND-* si aplica, Low se descarta).
- **Estado:** ✅ DONE (2026-09-26) — diff docs-only: 6 archivos `unsupported_ext` (0 reviewable) → **0 Critical/High** (spec en `target/session-api01/api09-ocr.json`; OCR no revisa markdown por diseño — cobertura del diff por doubt-driven degradado en §Review).

### Step 8: Bookkeeping plan + task file + recitation
- **Archivos:** `docs/dev/plans/2026-09-24-api-ejecucion.md`, `docs/dev/tasks/API-09.md`
- **Acción:** Task 9 → ✅ COMPLETED (fecha, contrato 6/6, pendiente del LEAD); header del plan → 9/9; recitation API-09 al pie; sync del task file (steps ✅ + Context Save Point). NO archivar (LEAD). NO commit (LEAD).
- **Verify:** `rg "9/9" docs/dev/plans/2026-09-24-api-ejecucion.md`; `rg "API-09" plan` → ✅; task file steps ✅.
- **Estado:** ✅ DONE (2026-09-26) — plan header `✅ COMPLETADO 9/9` + Task 9 `✅ COMPLETED` (contrato 6/6); recitation vía MCP; archivar/commit → LEAD.

## Dependencias
- API-01..08 ✅ completadas (commits locales `f6c395ef`/`ade86a1c`/`05c4c490` + anteriores). Última tarea de la campaña → sin sucesora.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

**Ronda 1 (degradada, historia):**
- **Revisor:** `doubt-driven-development` degradado (subagente sin spawn de reviewer fresco — degradación declarada) + OCR delegation (0 reviewable, docs-only).
- **Enfoque:** 4 claims adversariales: (1) ¿scores.md hybrid=score matchea el wire? → probe live: `score=1.0` sin distance ✔; (2) ¿VERSIONING 11 correctas? → API-STD-01 tabla + Gate P; on-disk conservado en sección aparte ✔; (3) ¿FIND-161 truthful? → `rg problem+json src/server` = 0 + `errors.rs` envelope real = `{success,code,error|data}` ✔; (4) ¿gates reales? → outputs pegados (coverage exit 0, smoke 11/11, probe 2/2, verify log) ✔.
- **Cómo se probó:** verificación mecánica (comandos + logs en `target/session-api01/`), no auto-reporte.
- **Veredicto:** ✅ aprobado (degradado, sin hallazgos accionables; Critical/High = 0).

**Ronda 2 (fresca, P2-01 — satisface el gate):**
- **Revisor:** `vanta-review` — sesión `ses_f2329896bffeh83Kc70QBccoVe` (agente distinto, contexto fresco).
- **Enfoque:** re-verificación independiente del contrato 6/6.
- **Cómo se probó (re-ejecuciones del revisor, no auto-reporte):** `validate-docs-coverage.ps1` → 0 gaps · `smoke-api04.py` → 11/11 · `probe-api09.py` → 2/2 · links VERSIONING 20/20 · coverage 81.63% ≥60.
- **Veredicto:** ✅ **APPROVE** — 0 hallazgos Medium+; 3 nits Low aplicados: nit 1 (`Enforcement` sin claim inerte) + nit 2 (YAML test vs docs-sync) en `docs/api/VERSIONING.md`; nit 3 (wording envelope) en `FIND-161`; `FIND-162` registrada.

## Notas
- Reglas duras del encargo: NO commit/push; NO código de producto; NO CHANGELOG/versiones (release-plz); NO archivar planes; `CARGO_TARGET_DIR=target\session-api01` ante cualquier cargo (lock MCP local en `target/debug`); disco C: ~69 GB libres.
- Pre-mortem cubierto: F1 (CHANGELOG/versión manual) → no se toca; F2 (ES en docs técnicas) → edits en EN; F3 (planes sin archivar) → plan actualizado, archivado del LEAD.
- **Tooling fix (`dev-tools/verify.ps1:79`):** bug pre-existente `cargo llvm-cov nextest run` (`run` = filtro de test → 35/2268, 9.5% → FAIL) → corregido a `cargo llvm-cov nextest` (forma canónica en CI `ci-rust.yml:363`); coverage real **81.63%** líneas. No se debilitó el umbral (60): el fix restaura el gate. Logs: `api09-coverage.log`, `api09-verify-r2.log`.

## Context Save Point

API-09 ✅ COMPLETED (2026-09-26) — contrato 6/6 verificado mecánicamente; Review P2-01 ronda 2 (fresca, `ses_f2329896bffeh83Kc70QBccoVe`) ✅ APPROVE — 3 nits Low aplicados. Changeset: `docs/api/VERSIONING.md` (11 superficies), `docs/api/scores.md`, `docs/api/ERROR_HANDLING.md`, `docs/api/HTTP_API.md`, `docs/dev/Backlog.md` (FIND-161 + FIND-162), `docs/dev/plans/2026-09-24-api-ejecucion.md` (9/9), `dev-tools/verify.ps1` (fix coverage), `docs/dev/tasks/API-09.md`. Próximo (LEAD): commit local + `skill progreso` + archivar planes + `/audit quick` → `/ship`. Owner action pendiente: rebuild `target/debug` + restart OpenCode (refresh MCP local; el re-smoke corrió contra `target/session-api01`).
