# MCP-39: Output budgeting (truncado explícito + next_cursor)

## Metadata
- **Plan file:** docs/plans/2026-08-28-backlog-triage.md
- **Creado:** 2026-09-01T10:00
- **last-synced:** 2026-09-01T10:30
- **Estado:** ✅ COMPLETED

## Blast Radius

| Category | Files/Functions |
|----------|-----------------|
| **Callers** | `handle_tools_call` → `memory_list`, `search_multi` |
| **Callees** | `budget_value` (validation.rs), `text_content_hits_with_budget` (validation.rs) |
| **Config** | `McpConfig::byte_budget`, `min_byte_budget`, `max_byte_budget` (config.rs) |
| **Tests** | `validation.rs` (budget_value tests, text_content_hits_with_budget tests) |
| **Docs** | `docs/api/MCP.md` (Output budgeting section) |

## Contrato

```
Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "next_cursor|byte_budget|truncated" | Measure-Object | Select-Object Count >= 2
cargo check -p vantadb-mcp ✅
cargo test -p vantadb-mcp ✅
cargo fmt --check ✅
cargo clippy -p vantadb-mcp -- -D warnings ✅
```

## Herramientas

- cargo-mcp, rust-analyzer-mcp, codegraph, campaign-executor

## Steps

### Step 1: Create generic `apply_output_budget` helper in validation.rs
- **Archivos:** `vantadb-mcp/src/validation.rs`
- **Acción:** Implement `apply_output_budget<T: Serialize>(items: &T, byte_budget: usize, cursor: Option<usize>) -> (Value, bool, usize)` that returns envelope `{items, truncated, next_cursor}` with proper truncation
- **Verify:** `cargo test -p vantadb-mcp -- budget_value -- --test-threads=1` passes

### Step 2: Update memory_list handler to use apply_output_budget
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (lines ~1402-1465)
- **Acción:** Replace inline envelope creation with call to `apply_output_budget`, preserving existing `next_cursor` from page
- **Verify:** `cargo check -p vantadb-mcp` ✅

### Step 3: Update search_multi handler to use apply_output_budget
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (lines ~1695-1735)
- **Acción:** Replace `text_content_hits_with_budget` with `apply_output_budget` for consistent shape
- **Verify:** `cargo check -p vantadb-mcp` ✅

### Step 4: Add tests for truncation + next_cursor + truncated: true
- **Archivos:** `vantadb-mcp/src/validation.rs` (test module)
- **Acción:** Add tests verifying: (a) truncation with next_cursor preserved, (b) truncated=true when budget exceeded, (c) empty items when budget too small
- **Verify:** `cargo test -p vantadb-mcp -- --test-threads=1` ✅

### Step 5: Verify docs/api/MCP.md has output budgeting section
- **Archivos:** `docs/api/MCP.md`
- **Acción:** Confirm section exists (already present), ensure examples match new shape
- **Verify:** `scripts/validate-docs-coverage.ps1` ✅

### Step 6: Full verification and commit
- **Archivos:** All modified files
- **Acción:** Run full verify suite, commit with conventional message
- **Verify:** `cargo check -p vantadb-mcp && cargo test -p vantadb-mcp && cargo fmt --check && cargo clippy -p vantadb-mcp -- -D warnings` ✅

## Dependencias
- MCP-37 (completed)

## Notas
- Default byte_budget: 40960 (40KB = 80% of OpenCode 50KB cap)
- Helper must be generic for both memory_list (object with records array) and search_multi (array of hits)
- next_cursor must be preserved during truncation for pagination continuity
- Contract grep expects >=2 matches of next_cursor|byte_budget|truncated in tools.rs

## Context Save Point
- **Fecha:** 2026-09-01T10:00
- **Branch:** develop
- **CI pendiente:** sí
- **Decisiones:** Reuse existing budget_value, create apply_output_budget as thin wrapper
- **Problemas conocidos:** None
- **Próxima tarea:** MCP-39 completion → commit + skill progreso