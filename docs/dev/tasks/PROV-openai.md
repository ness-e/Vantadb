# PROV-openai: Fix compilación provider openai (verify-first)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-10-fixes.md (Task 6, Wave2)
- **Creado:** 2026-09-10 (DISCOVERY completo — task file no existía)
- **Estado:** ⏳ IN PROGRESS → cierre verify-first
- **Tipo (campaign_detect_task_type):** unknown / bug-fix compile (crate standalone)
- **Workflow:** bug-fix (localizing → planning → implementing → testing → review → accept → close)
- **Esfuerzo:** 🟡 1d (appetite max 1d ✅)
- **Prioridad:** 🟠 Media-Alta (regresión 5.0→4.0 documentada en review)
- **Branch:** develop (verificado `git branch --show-current` → develop ✅)
- **SDP (campaign_discover_skills_v2, phase BUILD):** systematic-debugging (bug) + incremental-implementation + test-driven-development + context-engineering + source-driven-development + doubt-driven-development (lifecycle BUILD); frontend-ui-engineering y api-and-interface-design descartadas — tarea compile Rust, no web/API pública nueva

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `providers/openai/Cargo.toml` | 23 | crate standalone `vantadb-openai` 0.5.0, `[workspace]` vacío (aislado — NO es workspace member, coherente con `Cargo.toml:707` + CI comment "MSVC linker crash on Windows"); dep path `vantadb` + `pyo3 0.29` opcional |
| `providers/openai/src/lib.rs` | 10 | re-export `python::*` tras feature `python`; `#![warn(missing_docs)]` |
| `providers/openai/src/python.rs` | 335 | `VantaDBOpenAI` pyclass completo (new/embed/search/store/delete/get/list/list_namespaces) + 2 unit tests `include_str!` sanity (PROV-07, PROV-10); usa `#[path = "../../shared_py.rs"] mod common` |
| `.github/workflows/ci-rust-10.yml:415-439` | — | job `experimental-check` corre EXACTO el contrato: `cargo check --manifest-path providers/openai/Cargo.toml` (ubuntu, sin --locked) |

### Referencias hacia adentro (outbound references)
- `python.rs` → `vantadb::config::VantaConfig`, `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions}` (path dep `../..`, features `fjall,memmap2`)
- `python.rs` → `providers/shared_py.rs` (`mod common`: `err_to_py`, `parse_distance_metric`, `build_search_request`, `record_to_pydict`, `extract_metadata`, `register_errors`) — existe ✅ (`Test-Path` True)
- `python.rs` → módulo Python `openai` (runtime, no build-dep)

### Referencias hacia afuera (inbound)
- Ningún miembro del workspace depende de `vantadb-openai` (standalone, `publish = false`); consumidor = Python (`vantadb_openai.pyi` + `tests/test_openai.py`, runtime, fuera del contrato)
- CI `experimental-check` = único gate mecánico

### Veredicto de impacto
**Cero cambios necesarios.** Historial muestra fixes ya aplicados: `2754c783` (PROV-01 E0063 `exclude_superseded` + stubs .pyi + timeout litellm), `e078cd28` (PROV-07 validación), `294486e3` (PROV-05 shared helpers), `86784baa` (PROV-10 custom key), `8cb40d44`, `2da5f782` (ERR-PY-01). El `cargo check` del contrato pasa exit 0 sin tocar código. Blast radius de escritura: 0 archivos.

## Contrato (verificable mecánicamente)

```
`cargo check --manifest-path providers/openai/Cargo.toml` exit 0
(offline-safe; si requiere red, documentar + intentarlo en CI)
```

## Steps

### Step 0: DISCOVERY verify-first (contrato + historial + CI)
- **Archivos:** `providers/openai/`, `.github/workflows/ci-rust-10.yml`, `git log -- providers/openai/`
- **Acción:** correr contrato por bash directa (campaign_verify_cmd con bug exit -1); leer fuentes; confirmar CI corre el mismo comando.
- **Verify:** exit 0 + evidencia abajo.
- **Estado:** ✅ COMPLETED.

### Step 1: Cierre sin fix fantasma (cero cambios de código)
- **Archivos:** ninguno de código (solo task file + sync plan file)
- **Acción:** NO editar providers/; revertir churn de `providers/openai/Cargo.lock` (re-resolución automática lru 0.16.4→0.18.4 + hashbrown 0.17.1 — drift transitivo del core, regenerable en cada run/CI; precedente FIND-20/DESKTOP-40: lock churn NO stageado).
- **Verify:** `git diff --quiet providers/openai/Cargo.lock` exit 0; `git status --short` sin archivos propios de código.
- **Estado:** ✅ COMPLETED.

### Step 2: Sync plan file + commit solo archivos propios + memoria + progreso
- **Archivos:** `docs/dev/tasks/PROV-openai.md` (nuevo), `docs/dev/plans/2026-09-10-fixes.md` (Task 6 → COMPLETED)
- **Acción:** sync Task 6 (Estado/Branch/Commit/Iteraciones/Notas) + commit `fix: PROV-openai — ...` solo esos 2 paths + `campaign_memory_write` lessons + `skill progreso`.
- **Verify:** `git show --stat HEAD` solo contiene los 2 paths; WIP ajeno intacto.
- **Estado:** ✅ COMPLETED.

## Dependencias
- Ninguna. Wave2 disjunta con DESKTOP-40 ✅ (desktop vs providers, archivos disjuntos).

## Notas
- **Stop condition evaluada:** error = migración mayor SDK → DEFER. NO dispara: compila sin cambios, pyo3 0.29 resuelve, sin breaking.
- **Red:** `--offline` también exit 0 (0.51s, caché warm); cold-run requiere índice crates.io (normal, igual que CI con `rust-setup`). Sin fallback CI necesario — CI corre el comando exacto en ubuntu.
- **`--locked`:** falla (exit 101) porque el lock commiteado está stale vs grafo actual — NO es parte del contrato (CI no usa --locked); se deja revertido, no se commitea churn.
- **Tests unitarios del crate** (`#[cfg(test)]` en python.rs, sanity `include_str!`): fuera del contrato; `cargo check` no los compila; CI tampoco los corre. No se agregan (scope discipline).
- **Deuda:** ninguna nueva. Lock stale recurrente en providers/* standalone (cada `cargo check` re-resuelve) — candidato a sync periódico tipo `d5cc4ef5`, NOTICED BUT NOT TOUCHING.
- **WIP ajeno respetado:** `M .opencode`, `M opencode.jsonc`, `M desktop/src-tauri/Cargo.lock`, `?? Investigacion-plan.md` — NO tocar ni stagear.

## Context Save Point
- **Fecha:** 2026-09-10
- **Branch:** develop
- **CI pendiente:** no (contrato == comando CI `experimental-check`; corre en merge a main)
- **Decisiones:**
  - PROV-openai = verify-first cerrada sin código (bug pre-fixado en historial 2754c783 y serie PROV-01/04/05/07/10).
  - Lock churn revertido, no commiteado.
- **Problemas conocidos:** ninguno bloqueante.
- **Próxima tarea:** ninguna — Wave2 última tarea (6/6). Plan completo tras este cierre.

## Verificación 2026-09-10 (bash directa — campaign_verify_cmd con bug exit -1)
1. `cargo check --manifest-path providers/openai/Cargo.toml` → exit 0 (Finished dev 25.16s) — contrato ✅
2. `cargo check --manifest-path providers/openai/Cargo.toml --offline` → exit 0 (0.51s) — offline-safe ✅
3. `cargo fmt --check --manifest-path providers/openai/Cargo.toml` → exit 0 ✅
4. `git log --oneline -8 -- providers/openai/` → serie PROV-01/04/05/07/10 + ERR-PY-01 ✅ (fix pre-aplicado)
5. `Test-Path providers/shared_py.rs` → True ✅; `rustc 1.95.0` ≥ `rust-version 1.94.1` ✅
6. `git diff --quiet providers/openai/Cargo.lock` → exit 0 tras revert ✅ (cero cambios código)
- **Gate D:** no disparado — cero ediciones, sin símbolos públicos nuevos, contrato mecánico exacto.
- **Gate V:** no disparado — 0 fallas de verify.
- **Gate C:** WIP ajeno verificado intacto antes del commit (solo task file + plan sync stageados).
- **Próxima tarea:** ninguna (6/6 plan completo).
