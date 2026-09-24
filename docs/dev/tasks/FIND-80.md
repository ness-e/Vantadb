# FIND-80 — seed corpus + crash upload + fuzz-pr

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 24, Wave7)
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ✅ COMPLETED (commit `5f0d54b6`, review vanta-review approve, hooks verdes)
- **Tipo:** devops (CI/CD) — `campaign_detect_task_type` → skills `ci-cd-and-automation`, checks `yamllint .github/`
- **SDP:** `campaign_discover_skills_v2` phase=BUILD keywords=[fuzz-corpus, crash-upload, cargo-fuzz, ci-artifacts] → campaign-executor, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, source-driven-development (+ progreso base). NO cargadas (scope discipline + ponytail): frontend-ui-engineering, api-and-interface-design — sin UI ni APIs en este task.
- **Gate D (question-gates):** NO disparado — blast radius 3 archivos, sin hot path, sin símbolos `pub` nuevos, sin API pública, contrato no ambiguo (seed mínimo + upload + doc + bins). Tipo devops → gate mecánico spec-first no aplica (sin lógica nueva).

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `.github/workflows/fuzz-40.yml` (149L), `docs/workflow/fuzz-40.md` (61L), `fuzz/Cargo.toml` (48L), `fuzz/fuzz_targets/fuzz_parser.rs` (23L), `fuzz/fuzz_targets/fuzz_node_deserialize.rs` (21L), `fuzz/fuzz_targets/fuzz_wal.rs` (25L), `fuzz/fuzz_targets/fuzz_archive.rs` (20L)
- **Referencias hacia dentro:** yml `fuzz` job usa `fuzz/corpus/<target>` como cache path (`:91`); `fuzz-pr` job idem (`:138`); doc describe jobs `build`+`fuzz` y ci-gate; `fuzz/Cargo.toml` declara 4 bins con harness `fuzz_target!` (los 4 verificados con harness real)
- **Referencias entrantes:** ningún workflow depende de `fuzz/corpus/` en disco (cache-only hoy); `docs/workflow/fuzz-40.md` referenciado por índice de workflows; `fuzz-40.yml` referenciado por plan FIND-80 únicamente
- **Veredicto de impacto:** BAJO — aditivo (seeds nuevos + steps yml aditivos con `if-no-files-found: warn` + sección doc aditiva). Sin cambios de lógica Rust, sin features, sin permisos nuevos (upload-artifact no requiere scopes extra). Rollback = revert commit.

## Contrato
Seed mínimo commiteado en `fuzz/corpus/` + upload corpus/crashes en `fuzz-40.yml` + doc fuzz-pr en `docs/workflow/fuzz-40.md` + `cargo check --manifest-path fuzz/Cargo.toml --bins` exit 0.

## Herramientas
- `cargo check --manifest-path fuzz/Cargo.toml --bins -j 2`
- `actionlint` (yml, si disponible; fallback parse YAML + diff revisado)
- `git diff --check`, `git status --short` (solo propios)
- `campaign_verify_cmd` (bug exit -1 conocido → fallback bash directa y anotarlo)

## Steps
### Step 1: Seeds mínimos en `fuzz/corpus/`
- **Archivos:** `fuzz/corpus/fuzz_parser/seed`, `fuzz/corpus/fuzz_node_deserialize/seed`, `fuzz/corpus/fuzz_wal/seed`, `fuzz/corpus/fuzz_archive/seed` (nuevos, bytes)
- **Acción:** 1 seed mínimo por target. Parser = query válida real (`FROM Person p WHERE edad = "25"`, de `src/parser/mod.rs:555`). Binarios = bytes mínimos que ejercitan el path de validación (wal: 20 bytes ceros = tamaño header; resto: 4 bytes). Resto del corpus por cache (pre-mortem: nada de MB al repo).
- **Verify:** `git check-ignore` vacío (commiteable) + `cargo check --manifest-path fuzz/Cargo.toml --bins -j 2` exit 0
- **Estado:** ✅ DONE (seeds 31B+4B+20B+4B=59B; check-ignore exit 1 = commiteable; bins exit 0 en 0.39s)

### Step 2: Upload corpus/crashes en `fuzz-40.yml`
- **Archivos:** `.github/workflows/fuzz-40.yml` (aditivo, jobs `fuzz` + `fuzz-pr`)
- **Acción:** step `Upload fuzz corpus + crashes` tras cada `Run` con patrón del repo (`actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`, `if: always()`, `if-no-files-found: warn`, path `fuzz/artifacts/<target>/` + `fuzz/corpus/<target>/`). Precedente: `heavy-bench-nightly-51.yml:82-88`, `ci-rust-10.yml:389-394`.
- **Verify:** `actionlint` (o parse YAML) + `git diff --check`
- **Estado:** ✅ DONE (actionlint exit 0; YAML parse OK jobs build/ci-gate/fuzz/fuzz-pr; diff-check limpio)

### Step 3: Doc `fuzz-pr` en `docs/workflow/fuzz-40.md`
- **Archivos:** `docs/workflow/fuzz-40.md` (solo añadir sección, NO reescribir; ci-gate `:54-61` intacto)
- **Acción:** añadir sección `fuzz-pr` (gate PR: 75s/target, Ubuntu-only, paths `src/**`+`fuzz/**`) + sección artefactos (qué se sube, dónde verlo). Sin tocar lo existente.
- **Verify:** `git diff --check` + links/paths citados existen
- **Estado:** ✅ DONE (aditiva, ci-gate intacto; diff-check limpio)

## Dependencias
- Wave7 paralela disjunta: FIND-84 (DONE `ebf76936`, `integrations/`) y FIND-85 (DONE `15ea513f`, wheels/pyproject) — NO tocar sus archivos. Previa Wave6 DONE 21/30. Next Wave8.
- Prohibidos (NO TOCAR): `.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, archivos FIND-84/85, `docs/pipeline-state.json`, `Cargo.toml` workspace.

## Notas
- Baseline verificado 2026-09-15: `fuzz/corpus/` NO existe (`Test-Path` False); yml cache (`:88-94`) + cleanup (`:96-97`) sin upload; doc SÍ cubre ci-gate (`:54-61`) — parte stale del reporte.
- Los 4 targets tienen harness `fuzz_target!` real (no hay targets sin harness; `cargo check --bins` exit 0 en 22.39s).
- Upload resuelto con ejemplo del repo (punto 8 del contrato) → SIN investigación internet (punto 9 no aplica, sin deuda TSYS-13).
- Regla 11: 0 claims sin fuente — cada claim cita file:línea o output de comando.

## Context Save Point
- **Fecha:** 2026-09-15
- **Branch:** develop
- **CI pendiente:** no (verificación local: bins + actionlint + diff-check)
- **Decisiones:** seeds bytes-mínimos (cache hace el resto); upload en ambos jobs fuzz+fuzz-pr con `always()`+`warn`; doc aditiva sin rewrite
- **Problemas conocidos:** `campaign_verify_cmd` bug exit -1 → fallback bash directa
- **Próxima tarea:** Wave8 (orquestador decide; FIND-74 post-FIND-67 por QUICKSTART compartido)
