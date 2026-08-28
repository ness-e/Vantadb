# Task MCP-37 — Perfiles de tool surface (cap Cursor 40 tools)

## Metadata
- **Plan file**: docs/plans/2026-08-28-backlog-triage.md
- **Task ID**: MCP-37
- **Estado**: 🟡 IN PROGRESS
- **Archivos clave**: `vantadb-mcp/src/handlers/tools.rs` (handle_tools_list), `vantadb-mcp/src/config.rs`, `docs/api/MCP.md`
- **Contrato**: `Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "VANTADB_MCP_PROFILE|mcp_profile" | Measure-Object | Select-Object Count` >=1 AND `cargo test -p vantadb-mcp -- --test-threads=1 2>&1 | Select-String "profile" | Measure-Object | Select-Object Count` >=1 (tests por perfil)

## Appetite & Esfuerzo
- **Appetite**: max 1d
- **Esfuerzo**: 🟡 1d
- **Prioridad**: 🔴 Alta (bloquea Cursor)

## SDP Skills
- `source-driven-development` — verifica docs oficiales primero
- `security-and-hardening` — trust boundary config
- `incremental-implementation` — slices verticales delgados
- `test-driven-development` — Red-Green-Refactor
- `context-engineering` — empaqueta contexto relevante

## Pre-mortem
- Fallo 1: Filtro rompe `tools/call` para tool no listada pero invocada → error claro `tool not in profile X`
- Fallo 2: Default `full` sigue excediendo 40 en Cursor — documentar default `dev` para Cursor
- Fallo 3: `test-mcp.py` no cubre perfiles → agregar matrix

## Risk Register
| Prob×Impacto | Riesgo | Respuesta | Trigger |
|--------------|--------|-----------|---------|
| 🟡×🔴 | Breaking change para clientes que esperan 72 tools | Default `full` preserva compat, perfiles opt-in | verify |
| 🟢×🟠 | Cursor sigue truncando en `dev` (42 tools) | Medir counts por perfil y ajustar | test |

## Cynefin
🟨 Complicado — requiere diseñar taxonomía de perfiles

## Top 3 riesgos
1. Taxonomía incorrecta
2. Tool count por perfil >40 en Cursor
3. Tests frágiles

## Uphill/Downhill
⬆️ 2 (qué tools en cada perfil) · ⬇️ 3

## DoD
- Task: 3 profiles filter + tests
- Commit: `feat:` + `cargo test -p vantadb-mcp`
- Release: docs/api/MCP.md § profiles

## Validación Appetite vs Effort
max 1d ≥ 🟡 1d ✅

## Steps

### Step 1: Restore full `handle_tools_list` implementation from git history
- **Action**: Restore the full `handle_tools_list` function from commit 7817188b (before EMB-05 stub)
- **Files**: `vantadb-mcp/src/handlers/tools.rs`
- **Verify**: `cargo check -p vantadb-mcp`

### Step 2: Add `McpProfile` enum and profile field to `McpConfig`
- **Action**: Add `McpProfile { Memory, Dev, Full }` enum and `profile` field to `McpConfig` in `config.rs`
- **Files**: `vantadb-mcp/src/config.rs`
- **Verify**: `cargo check -p vantadb-mcp`

### Step 3: Read `VANTADB_MCP_PROFILE` env var in `McpConfig::from_storage`
- **Action**: Parse env var `VANTADB_MCP_PROFILE` (values: `memory`, `dev`, `full`) and set `profile` field
- **Files**: `vantadb-mcp/src/config.rs`
- **Verify**: `cargo check -p vantadb-mcp`

### Step 4: Implement profile filtering in `handle_tools_list`
- **Action**: Filter tool definitions based on `config.profile`:
  - `memory`: Only core memory CRUD + search + list (≤20 tools)
  - `dev`: Memory + graph + collections + maintenance + introspection (≤35 tools)
  - `full`: All 76 tools (default, preserves compat)
- **Files**: `vantadb-mcp/src/handlers/tools.rs`
- **Verify**: `cargo check -p vantadb-mcp`

### Step 5: Update docs/api/MCP.md with profiles section
- **Action**: Document the three profiles, tool counts, and recommended defaults per client
- **Files**: `docs/api/MCP.md`
- **Verify**: `scripts/validate-docs-coverage.ps1`

### Step 6: Add tests for profile filtering
- **Action**: Add test `test_mcp_tool_profiles` that verifies tool counts per profile
- **Files**: `vantadb-mcp/tests/mcp_tests.rs`
- **Verify**: `cargo test -p vantadb-mcp -- --test-threads=1`

### Step 7: Full verify and commit
- **Action**: Run full verify (fmt, clippy, nextest, docs coverage) and commit
- **Verify**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run -p vantadb-mcp --profile audit`, `scripts/validate-docs-coverage.ps1`

## Context Save Point
- **Last completed step**: 0 (Discovery)
- **Next step**: Step 1
- **Git status**: Clean (working tree has stub files from EMB-05)