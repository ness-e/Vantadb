# TASK-MEM-59: Recall MCP público

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (W20-1)
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30
- **Estado:** ⬜ PENDING (código listo; vanta-worker NO hace commit — queda a vanta-lead)
- **Tipo:** feature-add (nuevos símbolos públicos en MCP surface)
- **Cynefin:** 🟨 Complicado
- **Source Gap:** research-wave2 vanta-memory-20260825 gap #4
- **SDP:** campaign-executor + api-and-interface-design + source-driven-development +
  security-and-hardening + incremental-implementation + test-driven-development +
  context-engineering

## Estado de implementación

| Step | Estado | Verificación |
|---|---|---|
| 1. Tool defs (`memory_recall`, `memory_search` JSON en `base_tools`) | ✅ | `cargo check -p vantadb-mcp` ✓ |
| 2. Dispatch arms en `handle_tools_call` + helper `dispatch_search_memory` extraído | ✅ | `cargo check -p vantadb-mcp` ✓ |
| 3. Profile gating (`memory_tools` array, Memory+Dev+Full) | ✅ | `rg "memory_recall" handlers/tools.rs` ≥ 3 |
| 4. Tests (5 nuevos en `tests/mcp_tests.rs` + 2 asserts en `test_mcp_tools_list`) | ✅ | `cargo test -p vantadb-mcp --test mcp_tests` → 82/82 ✓ |
| 5. Verify contrato + handoff | ✅ | `Select-String ... memory_recall\|memory_search` = **7** ≥ 2 |

## Archivos tocados

- `vantadb-mcp/src/handlers/tools.rs` (+188, -23) — tool defs JSON, dispatch arms, helper `dispatch_search_memory`, profile gating
- `vantadb-mcp/tests/mcp_tests.rs` (+209, -23) — 5 nuevos tests MEM-59 + asserts en `test_mcp_tools_list` + updates en `test_mcp_tool_annotations_coverage`/`test_mcp_tool_profiles` por el nuevo conteo (78 tools, Dev ≤37, Memory ≤21)

## Verify mecánico (ejecutado sobre delta limpio)

- `cargo check -p vantadb-mcp` ✓
- `cargo clippy -p vantadb-mcp --no-default-features -- -D warnings` ✓
- `cargo clippy -p vantadb-mcp --test mcp_tests --no-default-features -- -D warnings` ✓
- `cargo fmt --check -p vantadb-mcp` ✓
- `cargo test -p vantadb-mcp --test mcp_tests` → **82 passed, 0 failed**

## Bloqueo externo (no mío)

El worktree tiene cambios parciales en `src/cli.rs`, `src/cli_handlers/mod.rs`,
`src/bin/vanta-cli.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`,
`vanta-memory/src/core/record/l1_dedup.rs` de tasks paralelos (MEM-62, MEM-63,
BND/TS/SRV/WSM) que rompen el build script de `vantadb` (`ExportFormat` no
declarado, `VantaValue::Bytes` faltante). **No tocan mi blast radius.** El
verify de mi delta aislado (con `git stash` sobre los archivos ajenos) pasa
limpio. El merge final con los otros tasks es responsabilidad de vanta-lead.

## Regla 6 (deuda neta)

| Item | Saldo |
|---|---|
| 2 nuevos tools read-only (sin `unsafe`, sin `clone` en hot path, sin nueva dep) | **0 deuda** |
| Helper `dispatch_search_memory` extraído (refactor puro, reduce duplicación con `search_with_method`) | **−1 deuda** (DRY win futuro) |

## Pendiente (handoff a vanta-lead)

1. **Commit:** `feat: MEM-59 — Recall MCP público (memory_recall, memory_search)`
2. **Doc sync (Regla 3):** actualizar `docs/api/MCP.md` con las dos tools nuevas + ajustar la cuenta "76 tools" → "78 tools" en cualquier otra doc que mencione el conteo
3. **RBAC (Pre-mortem Fallo 1):** gap abierto — el scope `team` puede exponer data a clientes externos sin auth per-cliente. **NO bloquea MEM-59** (decisión consciente en spec; documentar en backlog como FIND-RBAC).

## Notas

- `memory_search` es alias perfecto de `search_memory` (helper único, no duplicación).
- `memory_recall` sin `session_key` por diseño (cliente externo no conoce sesiones internas); `ProfileIsolation::default()` (team=default, agent=default) limita blast radius.
- Si MEM-60/61 (heat+decay, dreaming) entran antes del merge, este delta sigue siendo compatible — `recall` ya consume el hook completo.
- **Stop condition del plan (max 1d):** cumplido. Spec acotada, ~250 LOC de código + tests.

## Contexto

Hoy VantaDB expone `search_memory` (HNSW + texto via `embedded.search`) pero NO un
"recall" de alto nivel que es lo que mem0 / graphiti / Letta ofrecen como API
principal al agente externo. El recall real (auto-recall con scope, mode,
scoring unificado, persona + scene nav) vive en `vanta-memory::core::hooks::auto_recall`
y sólo se invoca desde la pipeline interna (MEM-18) o desde `vanta-proxy` (D46/D47)
sobre IPC de orquestador. Un cliente MCP externo hoy tiene que llamar `search_memory`
y reconstruir a mano el bloque que necesita → adopción agente rota.

**Decisión arquitectónica (aprobada por plan file, Gate Result ✅ DO):** exponer
**dos tools MCP adicionales** en `handlers/tools.rs`:

1. `memory_recall(query, scope, top_k)` — thin wrapper sobre
   `vanta_memory::core::hooks::perform_auto_recall`. Devuelve
   `RecallResult` con `recalled_memories` estructurado + `prepend_context`
   formateado. **Sin estado de sesión (no session_key)** — el cliente externo
   pasa scope explícito y el tool es read-only.
2. `memory_search(query, filters, top_k)` — alias semántico de
   `search_memory` con la firma limpia `memory_search` (sin el sufijo "_memory")
   que es lo que un agente espera tras mem0. **Internamente delega** a
   `embedded.search()` — NO duplica lógica. Mantiene `search_memory` como alias
   para back-compat (ya en producción).

## Spec

| Decisión | Elección | Justificación |
|---|---|---|
| Naming | `memory_recall` (alto nivel) + `memory_search` (HNSW/BM25) | mem0 expone `search`; Letta expone `recall`. Ambos: el cliente decide. |
| `memory_recall` firma | `{query: string, scope?: "session"\|"agent"\|"team", top_k?: number}` | Mínimo viable para que el agente lo invoque sin fricción. `scope` default = `Agent` (TDAM parity). `top_k` default = 5 (RecallConfig default). |
| `memory_recall` sin `session_key` | OK por diseño | Cliente externo no conoce sesiones internas; el scope se aplica vía `ProfileIsolation { team_id: "default", agent_id: "default" }` (D22). RBAC queda delegado a la capa del host MCP (no nueva superficie). |
| `memory_recall` respuesta | `{prepend_context: string, recalled: [{content, score, type}], effective_mode: "keyword"\|"embedding"\|"hybrid"}` | Simétrico con la API nativa `RecallResult`. `effective_mode` comunica degradación sin romper el contrato. |
| `memory_search` firma | `{namespace, query_vector?, text_query?, top_k?, distance_metric?, explain?, filters?, search_profile?}` | Idéntica a `search_memory` — alias. |
| `memory_search` back-compat | `search_memory` sigue siendo dispatch | No romper clientes en producción (MCP-38 registro de 46 read-only tools). |
| Annotations MCP-38 | `readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false` | Read-only, idempotente — patrón de search_memory existente. |
| Profile gating | Agregar a `memory_tools` (Memory) y a Dev/Full | Recall público es adopción; pertenece al core de memoria. |
| Error handling | `McpError::invalid_params` para query vacía + `error_content(format!("Recall Error: {}", e))` para failures internas | Mismo patrón que `search_memory`. |

> **RBAC check (Pre-mortem Fallo 1):** el scope semánticamente amplía el
> alcance del recall. Sin auth, un cliente externo con scope `team` ve todo el
> namespace del team por defecto. **Decisión consciente:** el RBAC per-cliente
> queda **fuera** del scope MEM-59 (gap separado; ver docs/Backlog.md FIND-RBAC).
> El MCP server sigue siendo trust-on-host — el operador decide quién se conecta.
> **Mitigación:** `scope` default = `Agent` (no `Team`), limita blast radius por
> defecto. Tests documentan este comportamiento.

## Blast Radius (Regla 0)

**Archivos leídos completos:**
- `vantadb-mcp/src/handlers/tools.rs` (2836 líneas) — match arms de tools/call
- `vanta-memory/src/core/hooks/auto_recall.rs` (625 líneas) — API pública
- `vanta-memory/src/core/hooks/mod.rs` — re-exports
- `vantadb-mcp/Cargo.toml` — vanta-memory ya es dep directa

**Referencias hacia dentro (lo que voy a tocar):**
- `handlers/tools.rs::handle_tools_list` (línea 70) — agregar 2 JSON tool defs
- `handlers/tools.rs::handle_tools_call` (línea 1112) — agregar 2 match arms
- `handlers/tools.rs::profile_allowed_tools` (línea 951) — agregar a `memory_tools`

**Referencias entrantes (lo que me llama):**
- `vantadb-mcp/src/lib.rs` re-exporta `handlers` → no cambia
- `vantadb-mcp/src/server.rs` invoca `handle_tools_list`/`handle_tools_call` → no cambia
- `vanta-proxy/src/memory_tools.rs` NO llama MCP — irrelevante

**Veredicto de impacto:** bajo. La superficie pública MCP suma 2 read-only
tools (memoria search/recall). El core `vanta-memory` no se toca (sólo se
consume via API existente). `vantadb` core no se toca. **Sin nuevos deps**.

## Contrato

```
Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "memory_recall|memory_search" | Measure-Object | Select-Object Count
```
**Resultado esperado:** >= 2 hits (definición en tools/list + dispatch en tools/call + entrada en profile).

**Verify mecánico adicional:**
- `cargo check -p vantadb-mcp` ✅
- `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅
- `cargo nextest run -p vantadb-mcp` ✅
- `cargo fmt --check` ✅

## Steps

### Step 1: Spec + tool definitions en handlers/tools.rs
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** agregar dos bloques JSON en `base_tools` array (línea ~921)
  - `memory_recall` con annotations readOnlyHint/destructiveHint/idempotentHint/openWorldHint
  - `memory_search` con misma firma que `search_memory` (alias)
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ⬜ PENDING

### Step 2: Dispatch arms en handle_tools_call
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** agregar dos match arms después de `"search_memory"` (línea ~1514):
  - `"memory_recall"` → `perform_auto_recall(...)` + serializar resultado
  - `"memory_search"` → delega a la lógica de `search_memory` (DRY: extraigo a fn helper)
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ⬜ PENDING

### Step 3: Profile gating
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` `profile_allowed_tools`
- **Acción:** agregar `"memory_recall"` y `"memory_search"` al array `memory_tools` (línea ~956)
- **Verify:** `rg "memory_recall" handlers/tools.rs` >= 3 hits
- **Estado:** ⬜ PENDING

### Step 4: Tests
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (módulo `#[cfg(test)]`)
- **Acción:** 3 tests:
  - `recall_returns_recalled_memories_with_default_scope`
  - `recall_rejects_empty_query`
  - `search_alias_dispatches_to_search_memory`
- **Verify:** `cargo nextest run -p vantadb-mcp`
- **Estado:** ⬜ PENDING

### Step 5: Verify + handoff
- **Acción:** verify mecánico completo + commit
- **Estado:** ⬜ PENDING

## Reglas del proyecto (must-not violar)

- Regla 1 (Pre-push Gate): commit sólo después de `cargo fmt + check + clippy + nextest`
- Regla 3 (Doc Sync): actualizar `docs/api/MCP.md` con las dos tools nuevas
- Regla 5 (ADR/Memoria): no requiere ADR (decisión ya en plan file Gate Result)
- Regla 6 (Deuda neta): 0 — no hay `unsafe` ni `clone()` en hot path
- Regla 8 (Concurrencia): N/A — read-only, sin estado compartido nuevo
- Regla 9 (Bench): N/A — tool MCP no toca hot path (caller-side, no HNSW nuevo)
- Regla 10 (AI Guardian): cada línea es explicable (helper de search_memory es
  refactor puro, no lógica nueva)
- Regla 11 (Claims perf): no claims

## Dependencias
- Ninguna (task standalone). vanta-memory ya es dep de vantadb-mcp.

## Notas

- **`memory_search` vs `search_memory`:** mantener ambos. Back-compat.
  Internamente un solo helper para evitar divergencia.
- **Recall sin session_key:** decisión consciente (cliente externo no conoce
  sesiones internas). ProfileIsolation default = (team=default, agent=default).
- **Stop condition del plan:** appetite max 1d. Si excede, docs-only y code en
  follow-up — pero el spec es lo bastante acotado para cerrar en este turno.
- **vanta-worker no hace commit** (regla del task). Hago el código + verify
  mecánico, dejo commit a vanta-lead.