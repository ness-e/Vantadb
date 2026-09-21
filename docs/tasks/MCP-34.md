# MCP-34: snapshot create/restore vía MCP

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 13)
- **Fuente:** Backlog — `list_snapshots` lista snapshots Fjall pero no hay create/restore vía MCP
- **Esfuerzo:** 🟡 1d (potencial) — **STOP CONDITION: DEFER**
- **Prioridad:** 🟡
- **Tipo:** Rust (MCP wrapper)
- **Turns estimados:** N/A (no implementado — DEFER)
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** 🟡 DEFER (STOP CONDITION — requiere core nuevo para `snapshot_restore`)
- **Incógnitas (uphill):** 1 resuelta en DISCOVERY (¿existe restore público? → NO)
- **Pendientes (downhill):** 0 — no se implementó; re-triaje como feature core

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (de create_snapshot) | `src/sdk/builder.rs` (VantaEmbedded::create_snapshot), CLI `src/cli_handlers/snapshot.rs`, HTTP `src/cli_server.rs:2992` (POST /api/v2/snapshots/{name}) |
| Callers (de list_snapshots) | `src/sdk/builder.rs`, `vantadb-mcp/src/handlers/tools.rs:1467` (MCP-26 tool) |
| Callees | `StorageEngine::create_snapshot` → fs hard_link/copy sobre `data_dir`; `FsSnapshot` struct (mod.rs:154) |
| Implicaciones | Ninguna para este intento — NO se editó código. Si se implementara restore, tocaría `storage/engine` (fuera de scope worker + batch wrappers) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/engine/mod.rs` (struct StorageEngine, FsSnapshot, create_snapshot :507/:540, list_snapshots :569), `src/backend.rs` (trait StorageBackend — solo `checkpoint`, sin restore), `src/sdk/builder.rs` (create_snapshot :253), `vantadb-mcp/src/handlers/tools.rs` (list_snapshots :461/:1467), reglas `.opencode/rules/server-mcp.md` + `durability.md`
- **Archivos referenciados hacia dentro:** FsSnapshot (mod.rs:154); StorageEngine usa `data_dir`, `wal` (restore implicaría reload del engine)
- **Archivos que referencian a los editados:** create_snapshot → builder.rs, cli_handlers/snapshot.rs, cli_server.rs; list_snapshots → builder.rs, tools.rs
- **Veredicto impacto:** NO SE EDITÓ NADA. Hallazgo de DISCOVERY documentado abajo.

## Hallazgo DISCOVERY (⬆️ UP-HILL) — veredicto

**Pregunta:** ¿existen métodos públicos `snapshot_create` / `snapshot_restore` en `StorageEngine`?

| Tool | ¿Método público existe? | Evidencia |
|------|------------------------|-----------|
| `snapshot_create` | ✅ SÍ | `StorageEngine::create_snapshot(&self, name) -> Result<FsSnapshot>` (`src/storage/engine/mod.rs:507` unix, `:540` windows/wasm) — `pub fn`; SDK `VantaEmbedded::create_snapshot` (`src/sdk/builder.rs:253`); CLI `snapshot create`; HTTP `POST /api/v2/snapshots/{name}` (`cli_server.rs:2992`) |
| `snapshot_restore` | ❌ **NO EXISTE** | `rg -n "fn (create_snapshot|restore_snapshot|snapshot_restore|restore)" src` → NO hay `restore_snapshot`/`snapshot_restore` en StorageEngine, SDK, CLI, ni server. Los únicos `restore` son ajenos: `restore_graph_nodes` (grafos, api.rs:1308), `restore_to_timestamp` (WAL PITR, wal_archiver.rs:238), `VantaError::restore_error_sourced` (constructor de error). Trait `StorageBackend` solo tiene `checkpoint` (backend.rs:206), sin restore/rollback. MCP solo tiene `list_snapshots` (tools.rs:1467). |

**Conclusión:** `snapshot_create` es un wrapper fino viable. **`snapshot_restore` NO existe como método público en NINGÚN lado** — implementarlo requiere feature core nueva en `storage/engine` (swap del contenido del `data_dir` con el snapshot + flush WAL + reload del engine). Eso es propiedad de Arch/Engine, fuera del scope de "wrappers MCP" del batch, y está explícitamente en el plan como STOP CONDITION (pre-mortem Fallo 1, Risk Register Prob🟡×🔴, plan Task 13 stop conditions).

## Decisión — STOP CONDITION → DEFER

- **Regla aplicada (plan Task 13 stop conditions):** "si no existen métodos públicos y requiere core nuevo → re-triaje como DEFER (uphill)."
- **Regla usuario 4:** "si create/restore NO existen como métodos públicos y requiere implementación core nueva → marcá la tarea como DEFER (no fuerces core nuevo en un batch de wrappers MCP), devolvé 🟡 INCOMPLETO con el próximo step = re-triaje. NO cambies scope."
- **Resultado:** NO se implementó código. NO se cambió scope. La tarea se re-trieja como **feature core** (restore físico) para `vanta-arch`/`vanta-engine`; `snapshot_create` puede ser wrapper MCP independiente si el lead lo desglosa (candidato MCP-34a) en un batch de wrappers.

## Contrato
Original (no cumplido — DEFER): "tools `snapshot_create`/`snapshot_restore` si existen métodos públicos; path sanitizado (solo data dir); tests + docs ×2 hash SAME"
**Estado:** NO aplicable — `snapshot_restore` sin método público → contrato no satisfacible sin core nuevo.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `create_snapshot` y `list_snapshots` intactos (no tocados). `snapshot_restore` no existe — cualquier implementación futura debe ser feature core de storage (Arch/Engine), NO forzada en wrapper MCP.
- **Comandos de verificación:** n/a (sin cambios). Baseline: `cargo test -p vantadb-mcp --test mcp_tests` (70/70 tras MCP-33).
- **Deuda pendiente:** `snapshot_restore` como feature core pendiente → re-triaje. `snapshot_create` wrapper MCP candidato a desglosar (MCP-34a) si el lead lo prioriza.

## Deuda técnica (Regla 6 — MUST)
Sin deuda registrada (no se introdujo código).

## Definition of Done
No aplica — tarea DEFER sin implementación. Nivel Task/Commit/Release no se evalúan (no hay diff).

## Herramientas necesarias
- codegraph_explore (uso: DISCOVERY completo) ✅

## Investigation Notes
- Verificado con `codegraph_explore` (src/storage/engine/mod.rs snapshots, tools.rs list_snapshots) + `rg` exhaustivo en `src/` y `vantadb-mcp/`.
- `list_snapshots` lee `<data_dir>/snapshots` (dirs). `create_snapshot` escribe `<snap_dir>/data/...` vía hard_link (unix) / copy (windows) para reabrirse como DB.
- Un `snapshot_restore` no es trivial: requeriría cerrar/flushear el engine vivo y reemplazar `data_dir` — decisión de diseño de storage (Arch).

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 (resuelta: restore NO existe) |
| Pendientes de ejecución (downhill) | 0 (DEFER — sin pasos de implementación) |
| % completado | 0% (solo DISCOVERY; no se implementó) |

## Steps
Ninguno — STOP CONDITION. La tarea se re-trieja (DEFER como feature core).

## Review (GATE — agente distinto, P2-01)
No aplica — sin implementación que revisar. La decisión DEFER está documentada y alineada al plan (Task 13 stop conditions) + instrucción del usuario (Regla 4).

## Notas
- Decisiones de diseño: DECISIÓN DEFER — `snapshot_restore` requiere core nuevo (storage/engine), fuera del scope del batch de wrappers MCP. No forzar.
- Contexto aprendido: `create_snapshot` ya está expuesto por CLI y HTTP server; `snapshot_restore` es el gap real (no existe primitiva de restore físico en todo el codebase).
