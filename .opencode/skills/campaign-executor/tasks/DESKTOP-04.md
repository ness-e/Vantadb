# DESKTOP-04 - Contracto multi-connection: trait VantaConnection + DTOs serde + VantaError unificado

- **Estado:** ✅ COMPLETED (2026-08-06)
- **Esfuerzo:** 🟡
- **Archivos clave:** `desktop/src-tauri/src/connections/trait.rs`, `desktop/src-tauri/src/connections/types.rs`, `desktop/src-tauri/src/error.rs`
- **Agente:** `vanta-worker`

## Context

La app desktop (`desktop/`) es un workspace propio que **no depende del core `vantadb`**
(eso es DESK-03). Para conectar múltiples backends (native / server / HTTP / MCP / WASM)
se define un contrato común.

**Decisión reusar vs duplicar (documentada, sin dep core):**
- El core `vantadb` (`src/sdk/builder.rs`) expone `VantaEmbedded`, y define `UnifiedNode`,
  `VantaMemoryInput` y su propio `VantaError` — pero **NO** define `IngestItem`,
  `MemoryRecord`, `SearchQuery`, `SearchResult`, `HealthReport` ni `Capability`.
- Sin colisiones de nombres → **duplicar DTOs serde livianos en el crate desktop es lo
  correcto**; no hay que acoplar al ws raíz ni depender del core (verificación `cargo check`
  sin esa dep pasa). El puente al core es DESK-03.

## Contrato cumplido

- `trait VantaConnection: Send + Sync` (object-safe, `#[async_trait]`) en
  `src/connections/trait.rs`: `info/capabilities` (síncronas), `connect/disconnect`,
  `ingest(IngestItem)->Result<String>`, `ingest_batch(Vec<IngestItem>)->Result<Vec<String>>`,
  `search(SearchQuery)->Result<Vec<SearchResult>>`, `get(&str,Option<&str>)->Result<MemoryRecord>`,
  `delete(&str,Option<&str>)->Result<()>`, `list(Option<&str>,usize)->Result<Vec<MemoryRecord>>`,
  `health()->Result<HealthReport>`. Todos devuelven `Result<_, VantaError>`.
- `src/connections/types.rs`: DTOs serde `Capability`(Native/Http/Mcp/Node/Python/Wasm),
  `ConnectionStatus`, `HealthStatus{Healthy/Degraded/Unhealthy}`, `IngestItem`,
  `SearchQuery`, `SearchResult`, `MemoryRecord`, `ConnectionInfo`, `HealthReport`
  (sin `Copy`: E0204 por `message: Option<String>`).
- `src/error.rs`: `VantaError` unificado `#[non_exhaustive]` con serde + variants
  `Lock/Io/Serialization/Other/...` + `From<std::io::Error>` + `From<serde_json::Error>`
  y clasificación HTTP (`HttpErrorKind::from_status`).
- `wire_types.rs`: DTOs de la API real del server preservados verbatim (pertenecen a
  DESK-08/DESK-02), reubicados al módulo `connections::wire_types` para no colisionar
  con el contracto.

## Verificación

- `cargo check` (desktop/src-tauri) → `exit 0`.
- `cargo test --lib` → ✅ `17 passed` (roundtrips JSON de cada DTO + defaults ausentes +
  `From<io>`/`From<serde_json>` + clasificación HTTP + 4 tests de `wire_types`).
  Nota: el primer intento crasheó el compilador (rustc `STATUS_STACK_BUFFER_OVERRUN`
  codegen de `windows`/`h2` en perfil test, con `target-cpu=native`); un reintento
  con cache caliente compiló y pasó — es crash transitorio del toolchain, no del código.

## Decisiones / notas

- `build.rs` vaciado y `[lib] name="vantadb_desktop_lib"` (los espera el bin `main.rs`
  de DESK-02) — ya incluidos en el commit scaffold de DESK-02 (`9feefea7`).
- Reconciliación con WIP paralelo de DESK-02/DESK-08 en este modulo (ver nota en
  `connections/mod.rs`).
- Fuera de alcance: WAL/vector/storage, plan file, `tests/server_client_mock.rs` (WIP de
  DESK-08, no commiteado).