# TSK-107b: Audit logging enterprise (JSONL, timestamp + op)

## Metadata
- **Plan file:** (backlog directo — no hay plan file activo)
- **Fuente:** docs/Backlog.md:172
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 30-60
- **Creado:** 2026-08-02
- **last-synced:** 2026-08-02
- **Estado:** ✅ COMPLETADO

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `VantaEmbedded` (sdk/builder.rs) es el punto de convergencia de CLI, WASM, Python (PyO3), MCP, TS — un hook por operación en el SDK cubre todos los frontends |
| Callees | `serde_json` (presente), `web_time` (presente, usado en impl_export.rs), `std::fs` |
| Implicaciones | No rompe contrato público: el audit log es opt-in vía config. `StorageEngine::delete(id, _reason)` (ops.rs:1443) y `delete_in_txn` (ops.rs:278) ya tienen el parámetro `reason` reservado — solo consumirlo. NO tocar WAL, export/import de memorias, ni el formato `LogFormat::Json` (son logs de runtime, no audit de operaciones). |

- **RIESGO:** medio. Toque mínimo en hot paths (solo cuando `audit_log_path` está configurado; no-op otherwise).
- **CONTRATO:** `cargo nextest run --profile audit -p vantadb --build-jobs 2` pasa, clippy `-D warnings`, fmt limpio, y el test de audit log verifica: archivo JSONL creado en la ruta configurada, cada línea parsea a JSON con campo `timestamp` (ISO 8601) + `op` + `namespace` + `key` + `outcome`.

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, nextest)
- rust-analyzer-mcp (diagnostics, goto def)
- codegraph_explore (blast radius — ya hecho, ver Notas)

## Investigation Notes
- El audit report `docs/audit-reports/backlog-validation-2026-07-28.md:134` confirmó: "Sin módulo audit. JSONL solo para export/import, no para operaciones. Placeholder en ops.rs:233."
- Patrón estándar de audit logging (append-only JSONL, WORM): timestamp ISO 8601 + operación + sujeto + target + resultado. Sin ambigüedad técnica → no se hizo web research (patrón conocido, infra existente en el repo).
- No es un log de tracing (`LogFormat::Json`) — es un registro de OPERACIONES de negocio (put/get/delete/search/export/import) para compliance/debugging. Distinto objetivo y distinto sink.
- Un solo hook por operación en `VantaEmbedded` cubre CLI + WASM + Python + MCP + TS (todos delegan al mismo método). NO instrumentar cada frontend.

## Steps

### Step 1: Módulo `src/audit.rs` — `AuditLogger` + `AuditEvent`
- **Archivos:** `src/audit.rs` (nuevo), `src/lib.rs` (export `pub mod audit;`)
- **Acción:** struct `AuditEvent` con `#[derive(Serialize, Clone)]`: `timestamp: String` (ISO 8601 vía `web_time::SystemTime`), `op: String`, `namespace: String`, `key: String`, `outcome: String` ("ok"|"err"), `reason: Option<String>` (para deletes). Struct `AuditLogger { path: PathBuf, writer: Mutex<BufWriter<File>> }` con `fn new(path) -> Result<Self>` (crea dirs parent), `fn record(&self, event) -> Result<()>` (serializa `serde_json::to_string` + newline, flush best-effort), `fn is_enabled(&self)`. Append-only: abrir con `OpenOptions::create(true).append(true)`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 2: Config `audit_log_path` en VantaConfig
- **Archivos:** `src/config.rs` (field ~126, builder ~772, `from_env`/hot-reload ~558, tests ~1261)
- **Acción:** campo `pub audit_log_path: Option<PathBuf>` en `VantaConfig` (default `None`), env var `VANTADB_AUDIT_LOG_PATH`, builder `with_audit_log_path(path)`, test de parse. NO incluir en `HotReloadConfig` (no es hot-reloadable).
- **Verify:** `cargo check -p vantadb` + tests de config (`cargo nextest run -p vantadb config`)
- **Estado:** ✅ COMPLETADO

### Step 3: Instanciar `AuditLogger` en VantaEmbedded
- **Archivos:** `src/sdk/builder.rs` (struct VantaEmbedded ~15, constructores)
- **Acción:** agregar campo `audit: Option<AuditLogger>` a `VantaEmbedded`, inicializarlo en `open_with_config`/`from_engine`/`connect` si `config.audit_log_path` está seteado (si falla al abrir → `tracing::warn!` y `None`, no fallar el open). Método helper `fn audit(&self, event: AuditEvent)` que hace no-op si `None`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 4: Hooks en operaciones de escritura del SDK
- **Archivos:** `src/sdk/api.rs` (`put` ~194, `delete` ~366, `delete_by_filter` ~1068, `put_batch`), `src/sdk/serialization/impl_export.rs` (`export_namespace` ~121, `export_all` ~135, `import_file`)
- **Acción:** en cada método, tras validar inputs, registrar `self.audit(AuditEvent { op: "put"|"delete"|"delete_by_filter"|"put_batch"|"export_namespace"|"export_all"|"import_file", timestamp: now_iso(), namespace, key (o "N/A"), outcome: "ok"/"err", reason: Some(reason) para delete })`. En `delete`, pasar el `reason` que ya existe en la llamada a `engine.delete(node_id, "memory delete")`. En paths con `?` temprano, registrar el evento ANTES del resultado no hace falta — registrar el outcome real al final; para errores usar `let res = ...; audit(... outcome: if res.is_ok()...); res`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 5: Test unitario del audit log
- **Archivos:** `src/audit.rs` (mod tests) o `tests/audit_log.rs`
- **Acción:** test: config con `audit_log_path` en tempdir, hacer `put` + `delete` + `search`, cerrar, leer el archivo JSONL, assert: existe, cada línea parsea a `serde_json::Value`, cada una tiene `timestamp` (no vacío) y `op` (put/delete/search), el delete tiene `reason`. Test adicional: sin `audit_log_path` → no se crea archivo, ops funcionan igual (no-op).
- **Verify:** `cargo nextest run -p vantadb audit` + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` + `cargo fmt --check`
- **Estado:** ✅ COMPLETADO

### Step 6: Docs + cierre
- **Archivos:** `docs/operations/CONFIGURATION.md` (sección `audit_log_path` + env var), `docs/api/EMBEDDED_SDK.md` (nota: operaciones auditan si está configurado), `docs/Backlog.md:172` (✅)
- **Acción:** documentar el nuevo campo en CONFIGURATION.md y validar con `pwsh scripts/validate-docs-coverage.ps1` (los gaps pre-existentes fuera de scope se mantienen). Migrar TSK-107b a `docs/progreso/README.md`.
- **Verify:** `scripts/validate-docs-coverage.ps1` sin gaps NUEVOS
- **Estado:** ✅ COMPLETADO

## Dependencias
- Ninguna (task standalone del backlog P8 — Post-Launch & Enterprise)

## Notas
- BIZ-01 (enterprise crate separado con encryption+RBAC+audit+replication) es UNA tarea distinta y más grande. TSK-107b entrega audit dentro del crate principal (mismo patrón que crypto.rs y rbac.rs, que ya viven en `src/`). Si se crea el crate enterprise luego, el módulo `audit` se puede extraer.
- El placeholder de `ops.rs:287` (`delete_in_txn`) y `ops.rs:1443` (`delete`) ya reservaron el parámetro `reason` — esta tarea lo consume a nivel SDK, sin cambiar firmas internas.
- `delete_in_txn` bufferiza; el reason se pierde en `BufferedWrite::Delete(id)`. Para TSK-107b basta con auditar a nivel SDK (donde el reason llega completo). No tocar `BufferedWrite` — fuera de scope.

## Context Save Point
- **Fecha:** 2026-08-02
- **Branch:** develop
- **CI pendiente:** workflow de smoke tests de examples ya está en develop (GH-142, commit bf5033ac)
- **Decisiones:** hook único en VantaEmbedded (no por frontend); audit opt-in via `audit_log_path`; append-only JSONL; módulo en crate principal (extraíble si BIZ-01 crea crate enterprise)
- **Problemas conocidos:** CodeGraph auto-sync DISABLED (índice stale) → usar Read directo tras ediciones
- **Próxima tarea:** BIZ-01 o ENT-04 (siguientes enterprise en Backlog)
