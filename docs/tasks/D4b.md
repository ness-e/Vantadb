# D4b — Enum `Error`: `IoError→Io`, `BackendError→Backend`, … (DISEÑO, cero código)

> **Plan file:** `docs/plans/2026-09-11-cleanCA-remediation.md` (§6-D4b, §10 Adaptador)
> **Agentes:** `vanta-arch` diseña · `vanta-worker` implementa (D4b-impl) · `vanta-review` visa
> **Alcance:** SOLO DISEÑO. PROHIBIDO editar `src/error.rs` o cualquier código en esta tarea.
> **Norma:** cleanCA Apéndice V.3 (stuttering) + §4.1 OCP/LSP + Regla 5 (ADR lo escribe el HUMANO) + Regla 7 (release major si es breaking)

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)

- **Tipo:** refactor-breaking de API pública (enum `Error` no-exhaustivo, 30 variantes). Routing plan §5: `vanta-arch` diseña.
- **Fuente leída (solo lectura, sin edición):** `src/error.rs` (1336 líneas, enum `Error` líneas 118–325, `code()` 337–370, `is_retriable()` 373–382, `recovery_hint()` 385–413, constructores 418–478, tests 483–1336).
- **Blast radius (codegraph_explore + grep, 2026-09-12):**
  - Core match exhaustivo: `code()` (3 brazos agrupan las 13 variantes), `is_retriable()` (solo `WalError` + `BackendError` de las 13 son `true`), `recovery_hint()` (solo `SchemaError`/`RestoreError`/`BackupError` + overflows tienen hint; las otras 10 caen a `_ => None`).
  - Constructores: `wal_error/wal_error_sourced`, `serialization()`, `backend_error`, `restore_error/restore_error_sourced`, `backup_error/backup_error_sourced`, `generic_error*` — nombres de fns NO cambian (solo variantes).
  - Python: `providers/shared_py.rs:59-61,71-86` (`err_to_py` + `attach_err_meta` con `code/is_retriable/hint`) y `vantadb-python/src/convert.rs:777-794,831-833` (misma tabla espejo). Matchean por variante → rename rompe ambos.
  - TS/WASM: `vantadb-ts/src/errors.ts:79-117` (`classifyWasmError` por regex de Display + `wrapWasmError`/`wrapNativeError` en `vantadb.ts:147,173,207,228,372`, `native.ts:43,145,169,194`); `vantadb-wasm/src/lib.rs:1976` (`Reflect::set code = e.code()`); `vantadb-node/src/lib.rs:649` (`let code = e.code()`). Clasificación por Display/MESSAGE — Display NO cambia en este diseño (ver §2), `code()` tampoco → TS/WASM/Node inafectados a nivel wire.
  - MCP: `vantadb-mcp/src/error.rs:74-87` (matchea por `e.code()`, nunca por variante → inafectado).
  - Server: `src/server/errors.rs` (`vanta_error_response`/`vanta_error_status` por `code()` + `crate::metrics::record_vanta_error(e.code())` → inafectado).
  - Desktop: `desktop/src-tauri/src/error.rs:151-161` (`from_core`: `DatabaseBusy→Lock`, `IoError→Io`, resto `Domain{code,message}`) + test `:269` construye `IoError` explícito → SÍ afectado (1 match + 1 test).
  - Rust call-sites: `Error::IoError` en `src/backends/rocksdb_backend.rs` (15+ `map_err`), `src/binary_header.rs:62`, `src/cli_handlers/diagnostics.rs:141`, docs `AUDREP-01/04`; `Error::CliError` en `src/cli_handlers/{data,crud,backup,index}.rs`; `Error::SchemaError` en `src/cli_handlers/diagnostics.rs:72`; resto disperso (grep `Error::(Wal|Serialization|Iql|Cli|Search|Runtime|Restore|Backup|Backend|Schema)` ≈ 100 matches).
  - Tests que asertan NOMBRE de variante: `debug_format` (`contains("NodeNotFound")`), `display_backend_error`, `display_search_error`, `display_runtime_error`, `display_restore_error`, `display_backup_error`, `display_cli_error`, `display_iql_error`, `all_variants()` + `code_snapshot_all_variants` + `is_retriable_stays_consistent_with_code_table` → todos rompen con el rename y deben actualizarse en D4b-impl (lista exacta en §3 slice 0).
  - Docs contrato: `docs/api/ERROR_HANDLING.md` (§1.1 tabla codes, §2 `is_retriable`, §3 `recovery_hint`), `docs/api/EMBEDDED_SDK.md:654-678` (lista de variantes), `docs/api/PYTHON_SDK.md:1120`, `docs/api/TS_SDK.md` — actualizar en el mismo PR (Regla 3).
- **Baseline `/cleanCA src/error.rs`:** N2 🟡 (stuttering `Error::XxxError`, excepciones V.3 no aplican — ninguna variante es `std::io`-wrapper legítimo salvo `IoError`, que igual tartamudea `Error::IoError`). Veredicto de cierre para D4b-impl: N2 ✅.

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)

### Qué cambia (solo en D4b-impl, NO en esta tarea)
- 13 renames de variante en `src/error.rs:140-288` (tabla §A). Cambio mecánico: nombre de variante + todos sus matches/constructores/tests/docs.
- `#[non_exhaustive]` se conserva (ya existe, línea 121) — no se añade ni se quita.

### Qué NO cambia (invariantes, Hyrum surface preservada)
- `Display` de cada variante (strings `"WAL error:…"`, `"IO error:…"`, etc.) — NO se tocan: `classifyWasmError` (TS) hace regex sobre ellos; cambiarlos rompería el wire TS/WASM/Node sin necesidad.
- `code()` (10 códigos `VANTADB_*`), `is_retriable()`, `recovery_hint()` — misma semántica por variante renombrada; solo cambia el patrón del `match`.
- Nombres de constructores (`wal_error`, `serialization`, `backend_error`, …) — se mantienen (son fns, no variantes; renombrarlos duplicaría el breaking sin beneficio V.3).
- `SerdeMsgError`, `ChainedError`, `Result<T>`, variantes NO listadas (`NodeNotFound`, `WALVersionMismatch`, `Generic`, overflows, …) — intactas.

### Archivos exactos (D4b-impl, informativo — NO tocar aquí)
- `src/error.rs` (enum + 3 matches + tests `all_variants` y debugs que nombran variantes).
- Call-sites Rust que construyen/matchean las 13 (no exhaustivo, el worker corre `rg "Error::(WalError|SerializationError|IoError|IqlParseError|ValidationError|IqlError|CliError|SearchError|RuntimeError|RestoreError|BackupError|BackendError|SchemaError)"` y actualiza todos).
- `providers/shared_py.rs`, `vantadb-python/src/convert.rs` (tablas espejo Python).
- `desktop/src-tauri/src/error.rs` (`from_core` + test:269).
- Docs: `docs/api/EMBEDDED_SDK.md`, `docs/api/ERROR_HANDLING.md` (si nombra variantes), `docs/api/PYTHON_SDK.md:1120`.
- NO tocar: `vantadb-mcp/src/error.rs`, `src/server/errors.rs`, `vantadb-wasm/src/lib.rs`, `vantadb-node/src/lib.rs`, `vantadb-ts/src/*` (wire por `code()`/Display, invariantes).

### Comandos de verify (para D4b-impl, no correr aquí — tarea de diseño)
- `cargo check --workspace` (renames completos, cero `Error::XxxError` restante salvo alias si aplica)
- `cargo nextest run --profile audit -p vantadb error` + suite completa `cargo nextest run --profile audit --workspace --build-jobs 2`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo fmt --check`
- `cargo semver-checks check-release` (plan §C) + `cargo public-api` diff si disponible
- `/cleanCA src/error.rs` (N2 debe pasar a ✅)

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)

> Secuencia para `vanta-worker` en D4b-impl. Cada slice es 1 commit chico del lead. Orden minimiza ventana rota.

- ☐ **Slice 0 — inventario mecánico:** `rg` de las 13 variantes viejas en todo el workspace (incluye tests + desktop + providers + docs); lista archivo:línea por variante; NO edita. Verify: tabla §A reproducida por el worker idéntica a la de este diseño.
- ☐ **Slice 1 — core enum + matches internos:** rename en `src/error.rs` (enum + `code()` + `is_retriable()` + `recovery_hint()` + constructores que llaman `Error::WalError/...`). Verify: `cargo check -p vantadb`.
- ☐ **Slice 2 — call-sites Rust (por familia, 1 commit por familia o por archivo si >3 archivos):** (a) `IoError` (rocksdb_backend, binary_header, cli_handlers/diagnostics); (b) `CliError` + `ValidationError` (cli_handlers/*); (c) resto (`Wal/Serialization/Iql*/Search/Runtime/Restore/Backup/Backend/Schema`). Verify por slice: `cargo check -p vantadb` + nextest del módulo.
- ☐ **Slice 3 — bindings Python:** `providers/shared_py.rs` + `vantadb-python/src/convert.rs` (tablas espejo). Verify: `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py` (o gate Python vigente) + `cargo check -p vantadb-python`.
- ☐ **Slice 4 — desktop:** `desktop/src-tauri/src/error.rs` (`from_core` + test:269). Verify: `cargo check -p desktop` / gate Tauri vigente.
- ☐ **Slice 5 — tests core:** `all_variants()`, `code_snapshot_all_variants`, debugs `contains("…Error")`, displays por variante. Verify: `cargo nextest run --profile audit -p vantadb error` verde.
- ☐ **Slice 6 — docs + semver:** `EMBEDDED_SDK.md` + `ERROR_HANDLING.md` + `PYTHON_SDK.md:1120` en el mismo PR (Regla 3); `cargo semver-checks check-release` + `cargo public-api` diff adjuntados al PR; ADR humano linkeado. Verify: §C verde + `/cleanCA src/error.rs` N2 ✅.
- **Prohibido en D4b-impl:** cambiar `Display`, `code()`, `is_retriable()`, `recovery_hint()`, nombres de constructores, o el wire TS/WASM/Node/MCP/server; slices >1 familia; `expect()` nuevo.

### §A — Tabla variante → nuevo nombre + quién la matchea (evidencia 2026-09-12)

| # | Variante actual (`src/error.rs`) | Nuevo nombre | `code()` | `is_retriable()` | `recovery_hint()` | Quién matchea (evidencia) |
|---|---|---|---|---|---|---|
| 1 | `WalError(ChainedError)` :142 | `Wal` | `VANTADB_IO_ERROR` | **true** | None (`_`) | core ctor `wal_error*`; Python `shared_py.rs:86`, `convert.rs:794` (→CorruptError); TS regex `/wal error/` → IO_ERROR (`errors.ts:94`); tests `display_wal_error`, `is_retriable_true_for_wal_error`, `wal_error_sourced`, `all_variants` |
| 2 | `SerializationError(Box…)` :157 | `Serialization` | `VANTADB_CORRUPT` | false | None | Python `shared_py.rs:78`, `convert.rs:786` (→ValidationError); TS regex `/serialization error/` → CORRUPT; tests `display_serialization_*`, `vanta_error_source_for_serialization`, `all_variants` |
| 3 | `IoError(#[from] io::Error)` :161 | `Io` | `VANTADB_IO_ERROR` | false | None | 15+ `map_err(Error::IoError)` en `rocksdb_backend.rs:157-341`, `binary_header.rs:62`, `diagnostics.rs:141`, docs AUDREP-01/04; desktop `from_core` → `Io` (`error.rs:155`) + test `:269`; Python `shared_py.rs:71`, `convert.rs:777`; TS regex `/io error/`; tests `io_error_*`, `all_variants` |
| 4 | `IqlParseError{msg,line,col}` :196 | `IqlParse` | `VANTADB_VALIDATION_ERROR` | false | None | Python `shared_py.rs:82`, `convert.rs:790` (→ValidationError); TS regex `/iql parse error/` → VALIDATION; test `display_iql_parse_error`, `all_variants` |
| 5 | `ValidationError{field,reason}` :216 | `Validation` | `VANTADB_VALIDATION_ERROR` | false | None | `cli_handlers/crud.rs:114`; SECURITY.md:21; Python `shared_py.rs:75`; server FIND-55 (4xx envelope); tests `display_validation_error`, `is_retriable_false_for_validation`, `all_variants` |
| 6 | `IqlError(ChainedError)` :252 | `Iql` | `VANTADB_INVALID_ARGUMENT` | false | None | Python `shared_py.rs:83`, `convert.rs:791` (→ValidationError); test `display_iql_error`, `all_variants` |
| 7 | `CliError(ChainedError)` :256 | `Cli` | `VANTADB_IO_ERROR` | false | None | `cli_handlers/{data:153,crud:41/97/104,index:120,backup:154,diagnostics:510}`; Python `shared_py.rs:71` (→StorageError); test `display_cli_error`, `all_variants` |
| 8 | `SearchError(ChainedError)` :260 | `Search` | `VANTADB_IO_ERROR` | false | None | AUD-025 (`search/mod.rs` `Err(VantaError::SearchError…)`); Python `shared_py.rs:71` (→StorageError); test `display_search_error`, `all_variants` |
| 9 | `RuntimeError(ChainedError)` :264 | `Runtime` | `VANTADB_IO_ERROR` | false | None | B2a plan (`flat/scann/diskann/sync_ext` → `Error::RuntimeError` futuro); Python `shared_py.rs:71`; test `display_runtime_error`, `all_variants` |
| 10 | `RestoreError(ChainedError)` :268 | `Restore` | `VANTADB_CORRUPT` | false | Some("backup file exists…") | ctor `restore_error*`; test `display_restore_error`, `recovery_hint_for_restore_error`, `all_variants` |
| 11 | `BackupError(ChainedError)` :272 | `Backup` | `VANTADB_CORRUPT` | false | Some("writable…") | ctor `backup_error*`; test `display_backup_error`, `recovery_hint_for_backup_error`, `all_variants` |
| 12 | `BackendError(ChainedError)` :280 | `Backend` | `VANTADB_IO_ERROR` | **true** | None | ctor `backend_error`; snapshot-2026-08-07 (backend key 8 bytes); Python `shared_py.rs:71`, `convert.rs:777` (→StorageError); TS regex `/backend error/`; tests `display_backend_error`, `is_retriable_true_for_backend_error`, `all_variants` |
| 13 | `SchemaError(String)` :288 | `Schema` | `VANTADB_CORRUPT` | false | Some("Reinitialize…") | `diagnostics.rs:72` (`Error::SchemaError(msg) => …`); Python `shared_py.rs:80`, `convert.rs:788`; TS regex `/schema error/` → CORRUPT; tests `display_schema_error`, `recovery_hint_for_schema_error`, `all_variants` |

> Nota: `WALVersionMismatch` (línea 146, mayúsculas `WAL`) NO entra en este rename (es otro patrón — siglas; tratar aparte si se quiere, fuera de scope D4b).

### §B — Estrategia de compat (datos + recomendación técnica; la DECISIÓN la escribe el HUMANO en el ADR)

- **Opción 1 — Major directo (breaking, sin alias).** Costos: 1 PR mecánico + major bump (Regla 7) + `cargo semver-checks` en rojo esperado (enum variant rename = major) + actualización de los ~100 call-sites + tests Debug-contains + Python/desktop espejos + docs mismo PR. Beneficios: cero deuda residual; `rg "Error::XxxError"` queda en 0; sin superficie duplicada que mantener; coherente con `#[non_exhaustive]` (el enum ya avisa que crece, pero renombrar igual rompe matches exhaustivos — major honesto). Riesgo: consumidores externos con `match` exhaustivo (sin wildcard) rompen al actualizar — mitigado por major + CHANGELOG + `cargo public-api` diff.
- **Opción 2 — Type aliases `#[deprecated]` + major diferido (2 fases).** Forma: `#[deprecated(note="Use Error::Io since v0.X, will be removed in v0.Y")]` shims — en enums Rust no existen "alias de variante" directos; requiere variantes viejas que deleguen o `pub use` wrappers + `#[allow(deprecated)]` solo en tests de migración (nunca en código nuevo). Costos: doble superficie durante 1 minor (13 variantes viejas + 13 nuevas), matches internos deben migrar igual, `code()/is_retriable()/recovery_hint()` con 26 brazos o delegación, riesgo de que un caller nuevo use la variante vieja, `clippy::deprecated` noise. Beneficios: ventana de migración para consumidores externos; `semver-checks` verde en la fase 1 (aditivo+deprecated).
- **Recomendación técnica del arquitecto (no vinculante): Opción 1, major directo.** Justificación: (a) el universo de matches externos es chico y conocido (Python/desktop internos + consumidores por `code()`, no por variante — MCP/server/TS/WASM/Node van por `code()`/Display, invariantes); (b) la Opción 2 en enums Rust no es un alias barato — son 13 variantes fantasma con delegación que duplican los 3 matches y confunden `Debug`-contains; (c) el repo ya hace BREAKING 0.x documentados (ERR-TS-01: prefijo `VANTADB_`, guards→VantaError) con CHANGELOG como vehículo; (d) Regla 7 + `cargo semver-checks` están diseñados exactamente para este caso. Tradeoff aceptado: un major con migración mecánica guiada por compilador (cada `match` roto es un error de compilación localizable, no un bug runtime).
- **Decisión final: PENDIENTE — la escribe el HUMANO en el ADR (Regla 5).** Este diseño aporta datos, no veredicto.

### §C — Plan de `cargo semver-checks` (+ `cargo public-api`) para D4b-impl

1. Pre-requisito: `cargo install cargo-semver-checks` (versión fijada en CI si la matriz la usa) + baseline `main` fetchado.
2. Antes del rename: `cargo semver-checks check-release --baseline-rev main` → verde (baseline limpio; si hay rojo preexistente, registrarlo como fondo).
3. Después del rename (Opción 1): mismo comando → rojo esperado con 13 `enum_variant_missing`/`enum_variant_renamed`-clase + auxiliares; adjuntar output al PR como evidencia de major justificado.
4. `cargo public-api diff` (si disponible en el repo): adjuntar diff del API pública (`Error::Wal→…`); debe mostrar SOLO los 13 renames + nada más (si aparece `Display`/`code()` cambiado = slice contaminado → revertir).
5. CI: la matriz debe correr con y sin features que toquen `error.rs` (`--no-default-features` / `--all-features` según `Cargo.toml`) — coordina `vanta-lead` (Feature Gate Pipeline §1b) para que el rename no esconda un `#[cfg]` roto.
6. Cierre: CHANGELOG (release-plz) con entrada `feat!:` + `BREAKING CHANGE:` + guía `Error::IoError → Error::Io` (sed-like table §A).

### §D — ADR-borrador (SOLO DATOS — contexto, opciones, costos; sin decisión)

```markdown
# ADR-XXX: Rename variantes `Error::XxxError → Error::Xxx` (D4b)

## Contexto
- `src/error.rs:118-325` enum `Error` (30 variantes, `#[non_exhaustive]`), cleanCA N2 🟡 stuttering.
- 13 variantes `XxxError` (§A). `Display`/`code()`/`is_retriable()`/`recovery_hint()` son contrato cross-binding (ERROR_HANDLING.md §1-3); MCP/server/TS/WASM/Node consumen `code()`/Display (invariantes); Python + desktop + call-sites Rust + tests matchean por variante (afectados).
- Precedentes breaking 0.x documentados: ERR-TS-01 (prefijo VANTADB_*), guards→VantaError.

## Opciones
1. Major directo (rename puro, sin alias).
2. Fase deprecated (variantes viejas `#[deprecated]` 1 minor, luego remover en major).

## Costos medidos (diseño 2026-09-12)
- Matches a migrar: ~100 grep-hits + 2 tablas Python + 1 `from_core` desktop + ~15 tests que nombran variantes + 3 docs API.
- Wire inafectado: MCP/server/TS/WASM/Node (evidencia §1).
- semver-checks: Opción 1 = rojo mayor esperado (justifica major); Opción 2 fase 1 = verde (aditivo+deprecated) + fase 2 = rojo.
- Deuda residual: Opción 1 = 0; Opción 2 = 13 variantes fantasma + matches dobles durante 1 minor.

## Decisión
- PENDIENTE (la escribe el HUMANO — Regla 5). Recomendación técnica no vinculante del arquitecto: Opción 1 (ver §B).
```

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)

- **RESULTADO (diseño):** ver bloque RESULTADO al pie de la respuesta del agente (formato exigido por el despacho D4b-diseño; este archivo es el entregable, no el canal del veredicto).
- **`/cleanCA src/error.rs`:** baseline N2 🟡 registrado (§1); PASS (N2 ✅) corresponde a D4b-impl, no a esta tarea de diseño.
- **Recitation:** D4b-diseño entrega `docs/tasks/D4b.md` completo (tabla §A con evidencia, compat §B, slices+semver §C, ADR-datos §D); PROXIMO_STEP = D4b-impl (worker, slices §3); sin commit (diseño no commitea por despacho); Gate D = GO (blast radius diseño <10 archivos tocados — 1 archivo creado, 0 código modificado).
