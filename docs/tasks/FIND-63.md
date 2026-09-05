# FIND-63 — rama explícita SyncMode::Never en maybe_sync

> Plan: `docs/plans/2026-09-04-durability-release-readiness.md` Task 2, Wave 0 · Ruta: vanta-worker · Branch: develop
> Tipo auto-detect: **bug-fix** (`campaign_detect_task_type`: rust core) · Gate D: no dispara (blast radius ≤3 archivos, sin símbolos públicos nuevos, contrato no ambiguo)
> SDP: systematic-debugging, test-driven-development, incremental-implementation, context-engineering, source-driven-development, doubt-driven-development, campaign-executor, progreso, ponytail

## Objetivo

`SyncMode::Never` existe como variante (`src/config.rs:100-102`, doc: "Disables explicit flushing to disk")
pero `maybe_sync` (`src/wal.rs:376-389`) solo ramifica `Always`: `Never` cae al `else` y fsyncea igual que
`Periodic` (threshold default 1 = cada write). `rg "Never" src/wal.rs` → 0 hits. Alinear código con doc pública.

## Contrato (ley)

`rg "Never" src/wal.rs` muestra brazo explícito con semántica testeada + suite `wal` verde + clippy/fmt limpios.
Commit solo si el contrato pasa: `fix(wal): rama explícita SyncMode::Never (FIND-63)`.
NUNCA stagear ajenos (worktree con cambios de otras sesiones: assets/, web/, .opencode, docs/plans/…).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/wal.rs` (maybe_sync 371-389, `sync` 391-397, `open_with_buffer` 244-302, tests head 815-929), `src/config.rs` (enum 85-103, parse hot-reload 1304-1312), `.opencode/rules/durability.md`, grep `SyncMode::|sync_mode` en `src/` (95 matches).
- **Referencias hacia dentro (qué toca maybe_sync):** campo `sync_mode`, `flush_threshold`, `DEFAULT_PERIODIC_THRESHOLD`, `sync()` (pública — escape hatch manual intacto). Llamado solo por `append` (325) y `batch_append` (366), mismo archivo.
- **Referencias entrantes:** `wal_sharded.rs` (pass-through de `sync_mode` → fix propaga solo), `storage/wal.rs:21` (pasa `config.sync_mode` → `Never` vía config-file hot-reload `"never"` SÍ llega a WalWriter), `engine.rs:148` (Periodic hardcodeado, no afectado), `crash_helper.rs` (Always), tests wal/wal_sharded/wal_shipping (todos Periodic). **Ningún caller actual usa `Never`** salvo el path config-file; el fix solo cambia comportamiento de quien pidió `Never` explícitamente → alinea con doc pública.
- **Veredicto:** editar SOLO `src/wal.rs` (match exhaustivo en `maybe_sync` + ajuste doc `flush_threshold` + 1 test inline). NO tocar `config.rs` (doc ya correcta), `wal_sharded.rs`, `storage/`, ni API pública (cero símbolos nuevos).

## Steps atómicos

- [x] Step 0 — DISCOVERY: tipo bug-fix, blast radius, task file creado. Verify: este archivo existe con Impacto mapeado lleno.
- [x] Step 1 — RED: test `test_sync_mode_never_skips_auto_sync` en `mod tests` de `src/wal.rs` (Never+threshold Some(1) tras 2 appends → `records_since_sync == 2`; control Periodic → `== 0`; `sync()` manual resetea). Verify: FALLÓ como esperado (`left: 0, right: 2` en `src/wal.rs:896` — Never auto-synceaba).
- [x] Step 2 — GREEN: `match` exhaustivo `Always|Never|Periodic` en `maybe_sync` + doc `flush_threshold` ("Never never auto-syncs; use sync()"). Verify: test RED pasa + suite `wal` 63/63 verde.
- [x] Step 3 — CIERRE: `cargo fmt --check -p vantadb` ✅ + `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ (+ pre-commit hook fmt/clippy/actionlint ✅) → commit `a7285969` SOLO `src/wal.rs` (+62/-9) → RESULTADO estructurado. Plan file NO tocado (untracked, de otra sesión; el orquestador lo actualiza desde RESULTADO).

## Decisiones

- Opción A (elegida): brazo `Never => {}` (nunca auto-sync; `sync()` manual sigue disponible). El contrato permite A o doc+test; A respeta el doc público existente ("Disables explicit flushing") sin nueva superficie pública (ponytail: sin getters nuevos, el test usa campo privado visible desde `mod tests` hijo).
- Opción B (descartada): documentar "Never no desactiva fsync" — contradice doc pública y el parse `"never"` del hot-reload; sería fijar el bug como feature.
