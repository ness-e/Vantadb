# ERR-DESK-01 - Desktop: preservar HttpErrorKind/code en memory commands

> **Status:** ✅ COMPLETED (2026-09-02, vanta-worker)
> **Task ID:** ERR-DESK-01
> **Plan:** `docs/plans/2026-09-02-error-observability-excellence.md` (Task 6, Wave 3)
> **Owner:** vanta-worker
> **SDP:** incremental-implementation, test-driven-development, context-engineering (core del agente, embebidas en spec §3), ponytail (full)

## Goal

Eliminar la degradación: `commands/memory.rs:91` `mem_err = VantaError::Native(e.to_string())`
colapsaba errores del core (`vantadb::VantaError` con `code()` canónico post-ERR-CORE-01)
en un string plano. El front Tauri no podía distinguir retry (`VANTADB_BUSY`→`Lock`) de
not-found ni ningún `VANTADB_*`.

## DISCOVERY (verificación de tipo en el call-site)

- **El `Http{kind,status}` de `server.rs` NO pasa por `mem_err`**: los comandos memory usan
  solo la ruta embedded (`active_embedded`); `server.rs` devuelve desktop-`VantaError::Http`
  directo desde sus métodos. → la opción "propagar Http sin re-wrapear" no aplica al sitio 91.
- El tipo que LLEGA a `mem_err` (28 call-sites) es el **core** `vantadb::VantaError` (vía
  `db.list*` directo) o envoltorios foráneos que lo retienen como `source()`:
  `L0Error::Vanta`, `RecallError::Vanta`, `SceneError::Vanta`, `PersonaError::Vanta`,
  `GenLogError::Vanta`, `KnowledgeError::Scene(SceneError)` — todos `#[from]` thiserror → cadena navegable.
- Core ya expone `pub fn code() -> &'static str` (`src/error.rs:306`, 10 códigos `VANTADB_*`) ✓ Task 1 listo.
- Sweep `Native(` en `desktop/src-tauri/src/`: 3 sitios degradan errores propios
  (memory.rs:91, connection.rs:42, native.rs:143); el resto son join/parse/sintéticos — no envolturas de core.
- Blast radius de agregar variante: sin matches exhaustivos sobre `VantaError` en el crate (solo `matches!` con brazos concretos). Enum ya `#[non_exhaustive]` + serde derive → `Domain` es aditivo en el wire.

## Implementación (slices verticales, TDD)

1. **error.rs:** variante `Domain { code: String, message: String }` + `VantaError::from_core(&vantadb::VantaError)`
   canónico: `DatabaseBusy→Lock`, `IoError→Io`, resto→`Domain{code: e.code(), message: e.to_string()}`.
   Status HTTP deliberadamente fuera: los errores embebidos no tienen status; los HTTP ya lo llevan en `Http`.
   Roundtrip test extendido con `Domain`.
2. **memory.rs (RED→GREEN):** test `mem_err_propagates_core_error_structured` falló con
   `Native("Node not found: 7")` (RED, degradación confirmada). Fix: `mem_err(impl Error + 'static)`
   recorre `source()` con downcast → `from_core`; fallback foráneo = `Native(format!("{e}"))`
   (Display idéntico al anterior → `assemble_rejects_zero_budget` sigue verde).
3. **Sweep:** `map_core_error` en `connections/native.rs` y `commands/connection.rs` delegan a
   `from_core` (un solo mapeo, todos los consumidores).
4. **Tests nuevos:** `from_core_preserves_canonical_code` (Domain JSON roundtrip con `code` intacto),
   `from_core_keeps_lock_and_io_semantics`, `mem_err_falls_back_to_native_for_foreign_errors`.
5. Fix bloqueante preexistente en HEAD: `tests/server_connection_real.rs:135` le faltaba
   `sparse_vector: None` al initializer de `IngestItem` (campo añadido por H-04 sin actualizar
   el test del crate aislado). 1 línea.
6. `cargo fmt` crate completo: drift preexistente (el crate no es miembro del workspace raíz,
   el fmt global nunca lo cubrió). Mecánico, sin cambios de comportamiento.

## Verificación (contrato mecánico)

| Check | Resultado |
|-------|-----------|
| `grep -c "Native(e.to_string" desktop/src-tauri/src/commands/memory.rs` == 0 | ✅ 0 matches (rg exit 1) |
| `cargo check --manifest-path desktop/src-tauri/Cargo.toml --all-targets` | ✅ exit 0 |
| `cargo test --manifest-path desktop/src-tauri/Cargo.toml` | ✅ 87 lib + 2+11+2 integration = 102 passed / 0 failed (`-j 1`: link tauri OOM con build paralelo en esta máquina) |
| `cargo clippy --manifest-path desktop/src-tauri/Cargo.toml --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --manifest-path desktop/src-tauri/Cargo.toml -- --check` | ✅ exit 0 |
| RED→GREEN observado | ✅ `core error collapsed: Native("Node not found: 7")` → 2/2 ok |

Commit: `fix(desktop): preserva HttpErrorKind en memory commands (ERR-DESK-01)` — SOLO desktop/src-tauri (web/* y completions/* quedan para sus tareas).

## Notas

- `Domain` serializa `{"Domain":{"code":"VANTADB_*","message":"..."}}` — el front puede migrar de
  parsear `Native` a matchear `err.Domain.code` / `err.Http.kind`. Front sin tocar (fuera de scope).
- ponytail techo: `from_core` usa `code()`→String en cada conversión (allocation trivial en cold path de error).
- Tests que requieren runtime: ninguno — `server_connection_real` usa mock/child levantado en el propio test y pasó.
