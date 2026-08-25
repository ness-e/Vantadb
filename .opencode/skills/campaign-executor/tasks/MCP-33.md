# MCP-33: write_axiom/delete_axiom — axiomas gestionables por el agente

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 12)
- **Fuente:** docs/Backlog.md fila MCP-33 (G18)
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Tipo:** Rust (MCP)
- **Turns estimados:** 10
- **Creado:** 2026-08-25
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 → resuelta en DISCOVERY (no hay API de escritura de axiomas en core)
- **Pendientes (downhill):** 5 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/lib.rs:9` (`mod axioms`), `vantadb-mcp/src/handlers/tools.rs:3` (`use resolve_axioms`), `handle_tools_call`/`handle_tools_list` re-exportados en lib.rs y despachados en server.rs (firmas intactas) |
| Callees | `vantadb::VantaEmbedded::{from_engine, put, delete, list}` (`src/sdk/api.rs:217,503,601`), `vantadb::sdk::VantaMemoryInput::new`, `vantadb::sdk::VantaMemoryListOptions`, `StorageEngine` |
| Implicaciones | No cambia firmas públicas de `handle_tools_call/list`. Añade 2 tools al catálogo. No toca core (`src/`). `resolve_axioms` cambia semántica (merge Iron + `_axioms`) — el path legacy `_system/axioms` (dead, nunca escrito) se elimina. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `vantadb-mcp/src/axioms.rs` (31 — `resolve_axioms`, `HARDCODED_AXIOMS`, `SYSTEM_NAMESPACE`, `AXIOMS_STORAGE_KEY`)
  - `vantadb-mcp/src/handlers/tools.rs` (2057 — patrón `handle_tools_list` defs + `handle_tools_call` dispatch, helpers `text_content`/`serialize_content`/`error_content`/`validate_identifier`/`validate_payload`)
  - `vantadb-mcp/src/validation.rs` (537 — helpers; `validate_identifier` :11, `validate_payload` :37, `text_content` :346, `error_content` :363)
  - `vantadb-mcp/tests/mcp_tests.rs` (4050 — patrón round-trip, `setup_storage`, `default_config`, test tools list :280, read_axioms assertion :327)
  - `src/sdk/api.rs` (`put` :217, `delete` :503, `list` :601), `src/sdk/types.rs` (`VantaMemoryInput::new` :152, `VantaMemoryListOptions`)
  - `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME actual 3C776DD5…)
  - `docs/api/MCP.md` (tabla tools — Context & Axioms :151-156)
- **Referenciados hacia dentro:** `crate::validation::*`, `crate::config::McpConfig`, `crate::error::McpError`, `vantadb::VantaEmbedded`, `vantadb::sdk::*`, `serde_json`.
- **Referencias entrantes (a los editados):** `resolve_axioms` solo lo usa `tools.rs:1097` (read_axioms). `handle_tools_call/list` re-exportados y despachados — firmas intactas.
- **Veredicto impacto:** bajo — refactor interno de `resolve_axioms` + 2 arms + 2 tool defs + tests + docs. Ninguna firma pública del crate cambia.

## Decisión de diseño (Regla crítica / UP-HILL)

**Investigación:** NO existe API de escritura de axiomas en core.
- `src/agentic/` solo contiene `mod.rs` y `thread.rs` — no hay `axioms*` ni gestión de axiomas.
- `grep -i axiom src/` solo matchea `src/executor.rs:313` (comentario "Axiom: Topological Consistency").
- Los únicos axiomas viven en `vantadb-mcp/src/axioms.rs`: `HARDCODED_AXIOMS` (4 Iron Axioms, ids 1-4) + `resolve_axioms` (lee `_system/axioms` — record que NUNCA se escribe → siempre cae al hardcoded; path legacy dead).

**Decisión (opción B del plan):** definir los axiomas del agente como records en un namespace reservado `_axioms` con convención documentada. NO se cambia core.

**Convención:**
- Namespace reservado: `_axioms`.
- 1 axioma = 1 record: `key` = nombre del axioma (único, validado como identifier), `payload` = JSON string `{"id": <n>, "name": "...", "description": "..."}`.
- `id` auto-asignado en write = max(id existentes: Iron 1-4 + stored) + 1 → los axiomas del agente no colisionan con Iron (1-4).
- `write_axiom` = upsert (put semantics: mismo nombre reemplaza).
- `delete_axiom(name)` = delete del record.

**Preservación de Iron Axioms (invariante):** los 4 Iron Axioms son hardcoded y SIEMPRE se incluyen como base en `resolve_axioms`. `read_axioms` devuelve Iron + axiomas del agente (merge), ordenados por id. **Iron Axioms intactos** — nunca se escriben ni borran.
- El path legacy `_system/axioms` (dead, rompería la invariante de "Iron intactos" si existiera un record que reemplazara el hardcoded) se elimina. `SYSTEM_NAMESPACE`/`AXIOMS_STORAGE_KEY` quedan sin uso → se remueven.

## Contrato
"Tools `write_axiom`/`delete_axiom`; round-trip (write → read lo incluye → delete → read lo quita); Iron Axioms (read_axioms) intactos tras writes; docs skill ×2 hash SAME."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** los 4 Iron Axioms (ids 1-4) SIEMPRE presentes en `read_axioms` (hardcoded, nunca escritos/borrados). `_axioms` records con payload JSON parseable y id > 4. Las 2 copias de SKILL.md byte-idénticas (hash SAME). No se toca core (`src/`).
- **Comandos de verificación:** `cargo check -p vantadb-mcp` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅ · `cargo test -p vantadb-mcp --test mcp_tests` ✅ · `Get-FileHash skills/vantadb-mcp/SKILL.md` == `Get-FileHash .opencode/skills/vantadb-mcp/SKILL.md`
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6)

**Sin deuda nueva.** Elimina un path legacy dead (`_system/axioms`) — saldo neto ≤ 0.

## Definition of Done

| Nivel | Gate |
|-------|------|
| Task | Contrato del task file (mcp_tests pasa + round-trip + Iron intactos + hash SAME) |
| Commit | convencional `feat(mcp):` — lo prepara el worker, lo ejecuta el lead (sub-agentes NO commitean) |
| Release | N/A (verify mecánico del lead) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — toca trust boundary (input de usuario MCP): `name` validado con `validate_identifier` (no vacío, ≤ max_key_length, sin NUL), `description` con `validate_payload` (≤ max_payload_length). Namespace fijo `_axioms` (el agente no elige namespace). Output serializado como JSON. Reutiliza helpers existentes.
- [ ] **PERFORMANCE** — N/A (wrappers finos; lista `_axioms` acotada — es metadata de reglas, no hot path). Justificado en Notas.

## Steps

### Step 1: Task file + decisión + baseline
- **Acción:** crear task file con Impacto mapeado (Regla 0) + decisión de diseño documentada. Confirmar baseline limpio.
- **Verify:** `cargo check -p vantadb-mcp` ✅ (baseline limpio)
- **Estado:** ✅ COMPLETED

### Step 2: Refactor axioms.rs
- **Acción:** reemplazar `resolve_axioms` → Iron hardcoded (base, siempre) + merge records de `_axioms` (parse payload, sort por id). Añadir const `AXIOMS_NAMESPACE = "_axioms"`. Remover `SYSTEM_NAMESPACE`/`AXIOMS_STORAGE_KEY` (dead).
- **Verify:** `cargo check -p vantadb-mcp` ✅
- **Estado:** ✅ COMPLETED

### Step 3: tools.rs defs + dispatch
- **Acción:** añadir tool defs `write_axiom` (name, description) y `delete_axiom` (name) a `handle_tools_list`; añadir arms en `handle_tools_call` (validación + `embedded.put`/`delete` + `json!` de retorno).
- **Verify:** `cargo check -p vantadb-mcp` ✅ · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅
- **Estado:** ✅ COMPLETED

### Step 4: tests round-trip
- **Acción:** test tools list incluye write_axiom/delete_axiom; test round-trip (write → read incluye → delete → read quita); test Iron Axioms intactos tras writes; test validación (name/description vacíos → Err).
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests` ✅ 70/70 (+2 nuevos)
- **Estado:** ✅ COMPLETED

### Step 5: docs + cierre
- **Acción:** documentar write_axiom/delete_axiom en ambos SKILL.md (byte-idénticos) y en docs/api/MCP.md (Context & Axioms 2→4). Actualizar task file + recitation.
- **Verify:** hash SAME ×2 = `5ED86246…` ✅ · `cargo fmt --check` ✅ · `cargo test -p vantadb-mcp --test mcp_tests` 70/70 ✅ · docs-coverage MCP 44/44 ✅
- **Estado:** ✅ COMPLETED

## Context Save Point

**Estado al cierre (2026-08-25):** implementación completa y verificada mecánicamente.
- Código: `vantadb-mcp/src/axioms.rs` (resolve_axioms merge), `vantadb-mcp/src/handlers/tools.rs` (2 defs + 2 arms), `vantadb-mcp/tests/mcp_tests.rs` (2 tests).
- Docs: `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME `5ED86246…`), `docs/api/MCP.md`.
- Verify: mcp_tests 70/70 · cargo check -p vantadb-mcp ✅ · cargo check -p vantadb ✅ · cargo fmt --check ✅ · cargo clippy -p vantadb-mcp --all-targets -- -D warnings ✅.
- **No commiteado** (regla: el lead verifica mecánico y commitea por tarea).
- **Colateral pre-existente ruteado al lead:** docs-coverage reporta 1 gap en `vantadb-python` (`query_structured` no documentado en PYTHON_SDK.md) — NO es de MCP-33 (no toqué python); relacionado con MOD-20/FIND. El lead decide ruteo a Backlog.
- **Bloqueo bookkeeping:** `campaign_update_task_state MCP-33 → in-progress/completed` NO pudo ejecutarse por lock global one-task-at-a-time (FIND-06 in-progress de otra sesión). El lead desbloquea/actualiza el plan y la recitation.
- Próximo step: ninguno (tarea completa). Commit `feat(mcp):` lo ejecuta el lead.

## Notas

- Contrato MCP-33 del plan: "tools write_axiom/delete_axiom; round-trip; Iron Axioms (read_axioms) intactos; docs skill ×2 hash SAME" — se cumple íntegramente.
- Rendimiento: no aplica Regla 9 — no es optimización de hot path; es wrapper sobre put/delete/list (operación O(1)/O(k) metadata).
- Concurrencia (Regla 8): no toca dashmap/parking_lot/Tokio/multi-índice — solo wrapper sobre SDK put/delete existentes.
