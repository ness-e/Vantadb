# FIND-54: Fix flake cors_layer_none_when_empty — origins CORS vacíos

## Metadata
- **Plan file:** docs/Backlog.md (fila FIND-54, origen: verificación ERR-OBS-01 2026-09-02)
- **Creado:** 2026-09-02
- **Estado:** ✅ COMPLETED (commits `2030c743` fix + `f1389ce3` docs/progreso)
- **Owner:** vanta-worker
- **Tipo:** bug-fix (systematic-debugging cargada)
- **Branch:** develop
- **Esfuerzo:** 🟢 15m

## Root Cause (Phase 1-3 systematic-debugging)

- **Síntoma:** `server::router::tests::cors_layer_none_when_empty` falla determinístico con `--features server`; panic en `src/server/router.rs:378` (`assert!(cors_layer(&["".to_string()]).is_none())`).
- **Causa raíz:** en http 1.x `HeaderValue::from_str("")` es **válido** (valores de header vacíos permitidos), por lo que el `filter_map` de `cors_layer` (`src/server/router.rs:87-96`) NO descarta origins vacíos → `origins` queda `[""]` (vec no-vacío) → retorna `Some(CorsLayer)` → assert `.is_none()` roto.
- **Por qué no se veía:** el perfil audit de CI no habilita la feature `server`; el test nunca corrió en CI.
- **Fix:** filtrar origins vacíos ANTES de construir headers: `.filter(|o| !o.is_empty())` antes del `filter_map` + test sibling.

## Contrato (validación mecánica)

1. `cargo test -p vantadb --lib --features server server::router::tests::cors_layer_none_when_empty -- --exact` → pasa
2. `cargo test -p vantadb --lib --features server` → 0 failed (suite server completa verde)
3. `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → 0 warnings
4. `cargo fmt --all -- --check` → 0 diffs

## Steps

### Step 1: RED — reproducir el fallo
- **Acción:** `cargo test -p vantadb --lib --features server server::router::tests::cors_layer_none_when_empty -- --exact`
- **Estado:** ✅ COMPLETED — panic exacto `assertion failed: cors_layer(&["".to_string()]).is_none()` en `src\server\router.rs:378` (evidencia capturada antes de tocar nada)

### Step 2: GREEN — fix en cors_layer
- **Archivos:** `src/server/router.rs`
- **Acción:** `.filter(|o| !o.is_empty())` antes del `filter_map` en `cors_layer`
- **Estado:** ✅ COMPLETED — +3 líneas con comentario de contexto

### Step 3: Test sibling
- **Archivos:** `src/server/router.rs` (mod tests)
- **Acción:** `cors_layer(&["", "http://valid"])` → Some (blank descartado, válido sobrevive)
- **Estado:** ✅ COMPLETED — `cors_layer_blank_origin_mixed_with_valid_keeps_layer`

### Step 4: Verify mecánico completo
- **Acción:** contrato 1-4
- **Estado:** ✅ COMPLETED — 1) --exact ok; 2) suite lib --features server 2039/2039 (1 ignored); 3) clippy --all-targets --all-features -D warnings 0; 4) fmt --check 0. Re-check post-commit: 7/7 `server::router::tests` ok

### Step 5: Commit + Cierre
- **Commit:** `fix(server): descarta origins CORS vacíos — flake cors_layer_none_when_empty (FIND-54)`
- **NO stagear:** `completions/*`, `.opencode` (submodule); NO tocar `stash@{0}`
- **Cierre:** fila FIND-54 removida de `docs/Backlog.md` (progreso Trigger 1) + registro en `docs/avance/` + memory lesson
- **Estado:** ✅ COMPLETED — `2030c743` (router.rs only) + `f1389ce3` (docs Backlog+core-engine) + lesson escrita. `completions/*` y `.opencode` permanecen unstaged.

## Notas
- Fix mínimo según contrato — sin trim de whitespace, sin warn-log para blanks (YAGNI; el doc-comment ya dice "blank origins are skipped").
- `cors_layer_skips_invalid_origin` sigue passando: "not-a-url" es HeaderValue válido y el mix retorna Some por el origin válido.

## Context Save Point
- **Branch:** develop (dirty pre-existente: `.opencode`, `completions/*` — NO tocar)
- **Próxima tarea:** —
