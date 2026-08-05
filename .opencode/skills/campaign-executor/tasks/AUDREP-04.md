# AUDREP-04: Storage-Durabilidad — compact traga error de flush + sin sync_all antes de rename

## Metadata
- **Plan file:** (P13 backlog, verificado 2026-08-05)
- **Creado:** 2026-08-05
- **last-synced:** 2026-08-05
- **Estado:** ✅ COMPLETED
- **Severidad:** 🔴 CRÍTICO (corrupción silenciosa)

## Objetivo
En `compact_layout` de `src/storage/archive.rs`:
1. `let _ = tmp_mmap.flush()` (línea 99) descarta fallos de E/S (disco lleno) →
   propagar error, no tragarlo.
2. No hay `tmp_file.sync_all()` (o equivalente de mmap) antes del `rename`
   final (línea 126) → un crash post-rename deja un archivo parcial renombrado
   como "válido". Durabilidad garantizada = datos no se pierden en crash.

## Blast Radius
- **Archivo:** `src/storage/archive.rs` (compact_layout: líneas ~99 y ~126)
- **Callers:** `compact_layout` callers (ver AUDREP-01 Step 1).
- **Riesgo:** hoy la corrupción es silenciosa (no error visible). Fix cambia
  comportamiento de E/S (más lento: sync_all) — evaluar si aplica a TODO renombra
  o solo al compact final.
- Sin cambios de API pública (firma ya devuelve `Result`).

## Contrato
`cargo check -p vantadb` + clippy limpio + (si existe test de compact) que siga
pasando. Comportamiento de error: fallo de flush → `Err` propagado, nunca dump.

## Herramientas
- codegraph, cargo-mcp, rust-analyzer-mcp

## Steps
### Step 1: discovery + fix flush
- **Archivos:** `src/storage/archive.rs`
- **Acción:** `let _ = tmp_mmap.flush()` → `tmp_mmap.flush().map_err(VantaError::IoError)?`
  dentro del `if end > new_file_size`; el crecimiento del archivo ahora aborta
  el compact con error si el flush falla (disco lleno).
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅ COMPLETED

### Step 2: sync_all antes de rename
- **SO:** `src/storage/archive.rs` (línea 126)
- **Acción:** antes del `rename(&tmp_path, &vstore_path)` ahora hay:
  `tmp_mmap.flush()?` → `drop(tmp_mmap)` → `tmp_file.sync_all()?`. El archivo
  temporal queda fsync'ed a disco antes de renombrarse. `replace_backing_file`
  NO requiere sync adicional — re-opens `create(false)` del archivo ya
  renombrado y hace `remap_mut` (patrón existente en `vfile.rs`, sin fsync
  previo). No se duplican guards.
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅ COMPLETED

### Step 3: colaterales + dump
- clippy -D warnings ✅, fmt --check (archive.rs limpio). Commit atómico
  `fix(AUDREP-04): propagate compact flush errors + sync_all before rename`.
- **Estado:** ✅ COMPLETED — ver commit hash en Context Save Point.

## Dependencias
- Mismo agente que AUDREP-01 (mismo archivo, cambios adyacentes — build único).
- Auditar invocador previo para no duplicar guards.

## Notas
- Regla de duración Aplica `.opencode/rules/durability.md` (WAL/storage):
  leer antes de editar. Regla MUST: crash recoverability.
- Disco lleno es el caso de fallo que hoy se traga — el error debe propagarse.

## Context Save Point
- **Fecha:** 2026-08-05
- **Branch:** develop
- **CI pendiente:** sí
- **Decisiones:** grupo con AUDREP-01 en un solo vanta-worker.
- **Commit:** `fix(AUDREP-04): propagate compact flush errors + sync_all before rename` — hash en git log
- **Problemas conocidos:** archivo objetivo limpio en git status.
- **Próxima tarea:** AUDIT-03 (vanta-audit, paralelo independiente).