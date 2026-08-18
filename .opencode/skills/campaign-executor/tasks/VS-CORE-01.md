# VS-CORE-01: Cursor/paginación en el bridge desktop (re-scopeado)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-studio-fase0.md`
- **Creado:** 2026-08-18
- **last-synced:** 2026-08-18
- **Estado:** ✅ DONE
- **Tipo:** bridge desktop (Rust Tauri + TS)
- **Bloqueante de:** VS-05 (grid virtualizado) — sin cursor en `vanta_list` no hay paginación real

## Impacto mapeado (Regla 0)
- **Archivos modificados (9):** `desktop/src-tauri/src/commands/data.rs` (comando `vanta_list` acepta `cursor` y devuelve `ListPage`), `desktop/src-tauri/src/connections/trait.rs` (firma de `VantaConnection::list` con `cursor: Option<usize>` → `Result<ListPage>`), `desktop/src-tauri/src/connections/native.rs` (impl vía core `list` con `cursor` en `VantaMemoryListOptions` + `next_cursor` mapeado), `desktop/src-tauri/src/connections/server.rs` (impl ignora cursor, `next_cursor: None` — el server no pagina), `desktop/src-tauri/src/connections/manager.rs` (delegación `list_records` + test roundtrip cursor), `desktop/src-tauri/src/connections/types.rs` (DTO `ListPage` ADITIVO), `desktop/src-tauri/src/connections/mod.rs` (re-export), `desktop/src-tauri/tests/server_connection_real.rs` (call site actualizado), `desktop/src/vanta.ts` (`listPage` + `list` backward-compat)
- **Referencias hacia dentro (quién los importa):** `ListPage` lo importan data.rs, trait.rs, native.rs, server.rs, manager.rs, mod.rs; `listPage` será consumido por VS-05 (DataExplorer virtualizado).
- **Referencias hacia fuera:** `vantadb::VantaMemoryListOptions`/`VantaMemoryListPage` (`src/sdk/types.rs:204-240`) y `VantaEmbedded::list` (`src/sdk/api.rs:545`) — SOLO LECTURA, ya exponen `cursor`/`next_cursor`; `MemoryRecord` (types.rs) intacto (VS-11).
- **Fuente protegida (SOLO LECTURA):** `src/` (core Rust), `desktop/src/components/` + `desktop/src/App.tsx` (VS-04 en paralelo), docs. `types.rs` solo recibió el DTO `ListPage` aditivo (justificado: el shape del retorno lo requiere).
- **Archivos que NO toco (dominio ajeno):** `desktop/src/components/layout/WorkspaceShell.tsx` (modificado por VS-03 en el worktree), `desktop/src/components/home/` (VS-04 en progreso) — presentes en `git status` ANTES de esta tarea, no modificados.
- **Veredicto:** cambio de firma en un trait interno del crate desktop (aislado, 1 workspace member) — solo 2 adapters (`NativeConnection`, `ServerConnection`) lo implementan; ambos actualizados en el mismo diff.

## Blast Radius
| Callers | Callees | Implicaciones |
|---|---|---|
| VS-05 (futuro, vía `listPage`), UI vía `invoke("vanta_list", { cursor })` | `ConnectionManager::list_records` → `VantaConnection::list` → `NativeConnection::list` → `VantaEmbedded::list` | Solo el adapter nativo pagina por cursor; server devuelve una página única con `next_cursor: None`. `list()` en `vanta.ts` se mantiene devolviendo array (unwraps `.records`) para no romper consumidores legacy (DataExplorer, WorkspaceShell, HomeOverview) — `listPage` es el camino nuevo. |

## Contrato
- `cargo check` en `desktop/src-tauri` verde (workspace de 1 member; warning `cfg(mobile)` PRE-EXISTENTE en lib.rs:45).
- `cargo test -j 1` en `desktop/src-tauri` verde (42 tests incl. nuevo `list_records_paginates_by_cursor_without_overlap`) — `-j` alto falla con `os error 1455` (paging file) del entorno, NO del código.
- `npm run build` en `desktop/` verde (tsc strict + vite).
- Comando Tauri `vanta_list(namespace?, limit?, cursor?)` → `Result<ListPage { records, next_cursor }>`; aditivo (args `Option` → omitir = mismo comportamiento), compat backward.
- `listPage({ namespace?, limit?, cursor? }): Promise<ListPage>` en `vanta.ts`; `list()` conserva shape anterior (array) delegando en `listPage().records`.

## Herramientas
- `cargo check`/`cargo test -j 1` en `desktop/src-tauri` (NUNCA `cargo check -p` desde la raíz del repo, el crate está aislado a propósito)
- `npm run build` en `desktop/`
- codegraph (blast radius mapeado), `src/sdk/api.rs` + `src/sdk/types.rs` (solo lectura)

## Steps
### Step 1: DTO `ListPage` + firma del trait
- **Archivos:** `desktop/src-tauri/src/connections/types.rs`, `mod.rs`, `trait.rs`
- **Acción:** `ListPage { records: Vec<MemoryRecord>, next_cursor: Option<usize> }` (espejo de `VantaMemoryListPage` del core, `#[serde(default)]` en next_cursor) + re-export en mod.rs. Trait: `list(&self, namespace, limit, cursor: Option<usize>) -> Result<ListPage>` — doc: cursor = offset zero-based en el orden estable de ids, `None` arranca desde el inicio.
- **Verify:** `cargo check`
- **Estado:** ✅ DONE

### Step 2: Implementación nativa + server
- **Archivos:** `desktop/src-tauri/src/connections/native.rs`, `server.rs`
- **Acción:** Native: `VantaMemoryListOptions { limit, cursor, ..Default::default() }` → `db.list(&ns, options)` → mapea `page.records` + `page.next_cursor` (passthrough del core). Server: `_cursor: Option<usize>` ignorado, `next_cursor: None` (el backend IQL no pagina).
- **Verify:** `cargo check`
- **Estado:** ✅ DONE

### Step 3: Delegación en ConnectionManager + test roundtrip (RED→GREEN)
- **Archivos:** `desktop/src-tauri/src/connections/manager.rs`
- **Acción:** `list_records(namespace, limit, cursor)` delega con `limit.unwrap_or(100)`. Test RED `list_records_paginates_by_cursor_without_overlap`: 5 records → página 1 (2 records + cursor) → página 2 (2 records, sin solapamiento) → página 3 (1 record, `next_cursor: None`). El e2e existente se actualiza a la nueva firma (`Some(2), None` + `.records`).
- **Verify:** `cargo test --lib -j 1`
- **Estado:** ✅ DONE

### Step 4: Comando Tauri + call sites
- **Archivos:** `desktop/src-tauri/src/commands/data.rs`, `tests/server_connection_real.rs`
- **Acción:** `vanta_list(state, namespace, limit, cursor: Option<usize>) -> Result<ListPage>` → `manager.list_records(...)`. Test de integración real: `conn.list(Some("default"), 100, None)` + `.records.iter()`.
- **Verify:** `cargo check`
- **Estado:** ✅ DONE

### Step 5: Wrapper tipado en vanta.ts
- **Archivos:** `desktop/src/vanta.ts`
- **Acción:** interface `ListPage`; `listPage({ namespace?, limit?, cursor? }): Promise<ListPage>` → `invoke<ListPage>("vanta_list", ...)`; `list()` conserva `Promise<MemoryRecord[]>` delegando en `listPage().records` (backward-compat — DataExplorer/WorkspaceShell/HomeOverview aún esperan array).
- **Verify:** `npm run build` (tsc strict)
- **Estado:** ✅ DONE

### Step 6: Cierre — verify full + task file (sin commit)
- **Acción:** `cargo check` ✅, `cargo test -j 1` ✅ (42 tests), `npm run build` ✅. Este task file. NO commit (el lead commitea).
- **Verify:** `git status --short` — solo los 9 archivos de VS-CORE-01 + cambios ajenos de VS-03/VS-04 (WorkspaceShell.tsx, home/, VS-04.md — no tocados).
- **Estado:** ✅ DONE

## Dependencias
- Core `VantaEmbedded::list` + `VantaMemoryListOptions`/`VantaMemoryListPage` — YA existen en `src/sdk/api.rs:545` / `src/sdk/types.rs:204-240` (sin cambios).
- VS-11 (DTO enriquecido) — ya entregado; `MemoryRecord` del bridge tiene todos los campos, `ListPage` los reutiliza.
- VS-05 (grid virtualizado) — consumirá `listPage({ cursor })` para paginar por cursor.

## Notas
- **Decisión de tipo (desviación del prompt):** el prompt pedía `cursor?: string`; el core expone `cursor`/`next_cursor` como `Option<usize>` (offset zero-based) y todos los bindings (Python `VantaPyListResult.next_cursor: Option<usize>`, TS SDK `cursor?: number`, WASM `ListOptions.cursor: Option<usize>`) usan numérico. Seguir el patrón del core (como pedía el prompt: "respetá el patrón") → `Option<usize>` en Rust, `number` en TS. Un string hubiera exigido parse innecesario + error path sin beneficio. Desviación documentada; revertir es un cambio de 2 líneas si el lead insiste.
- **Backward-compat en TS:** cambiar el retorno de `list()` a `ListPage` rompía 3 consumidores legacy que VS-04/VS-05 están editando EN PARALELO (no podía tocarlos). Solución aditiva: `list()` conserva array (unwrap `.records`), `listPage()` es el camino nuevo con cursor. El comando Rust devuelve `ListPage` — un solo comando, dos wrappers.
- **Server backend:** no pagina (IQL `list` trae todo y se corta client-side) → `next_cursor: None` siempre; la UI con backend server paginará en memoria o mostrará una página (decisión de VS-05).
- **Ponytail:** sin DTO intermedio nuevo por adapter; `ListPage` único compartido. Server reusa el mismo DTO con `next_cursor: None` en vez de inventar un trait method extra con default.
- **Entorno Windows:** `cargo test` completo sin `-j 1` falla con `os error 1455` (paging file demasiado pequeño al mmap del rlib) — pre-existente, NO relacionado con este cambio; usar `-j 1`.
- **Warning pre-existente:** `unexpected_cfgs` para `cfg(mobile)` en lib.rs:45 (scaffold Tauri), no introducido aquí.

## Context Save Point
- **Fecha:** 2026-08-18
- **Branch:** develop
- **CI pendiente:** no (bridge desktop aislado del workspace raíz; verify core no aplica — no se tocó `src/`)
- **Decisiones:** cursor numérico `Option<usize>` (patrón del core, desviación documentada del `string` del prompt); `ListPage` DTO aditivo en types.rs (justificado); `list()` conserva array (backward-compat) + `listPage()` nuevo; server sin paginación (`next_cursor: None`).
- **Problemas conocidos:** `cargo test` sin `-j 1` falla por paging file del entorno; cambios ajenos de VS-03/VS-04 en el worktree (WorkspaceShell.tsx, home/) — no tocados, build verificado sobre estado mixto.
- **Próxima tarea:** VS-05 (grid virtualizado) — ya puede consumir `listPage({ cursor })`.