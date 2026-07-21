# DOC-API-03: Fix MCP.md — comandos rotos, tool renombrado, last_reviewed

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Estado:** ⬜ PENDING

## Blast Radius
**Callers:** Ninguno. Es doc-only.
**Callees:** `vantadb-mcp/src/lib.rs` (código MCP real)
**Implicaciones:** Usuarios que sigan las instrucciones de instalación literalmente obtendrán error `cargo install vantadb-cli` (crate no existe). Usuarios que busquen `query_lisp` no encontrarán el tool.

## Contrato
"grep -n 'vantadb-cli\|query_lisp' docs/api/MCP.md no encuentra resultados. grep 'vanta-cli\|query (IQL)' encuentra las referencias correctas."

## Herramientas
- Read, Edit, Grep, codegraph

## Steps

### Step 1: Leer MCP.md completo
- **Archivos:** `docs/api/MCP.md`
- **Acción:** Identificar todas las líneas que mencionan `vantadb-cli`, `query_lisp`.
- **Verify:** Lista de ocurrencias incorrectas
- **Estado:** ⬜ PENDING

### Step 2: Verificar nombres correctos en código real
- **Archivos:** `vantadb-mcp/src/lib.rs`, `Cargo.toml` (workspace), `src/main.rs`
- **Acción:** Confirmar que el binary se llama `vanta-cli` (feature `cli`). Confirmar que el tool MCP se llama `query` (IQL), no `query_lisp`.
- **Verify:** grep en Cargo.toml para `[[bin]]` name. grep en lib.rs para tool name.
- **Estado:** ⬜ PENDING

### Step 3: Reemplazar referencias incorrectas en MCP.md
- **Archivos:** `docs/api/MCP.md`
- **Acción:** `vantadb-cli` → `vanta-cli` (o `cargo run --features cli`). `query_lisp` → `query` (IQL). Verificar que no hay otras referencias obsoletas.
- **Verify:** `grep -n "vantadb-cli\|query_lisp" docs/api/MCP.md` vacío
- **Estado:** ⬜ PENDING

### Step 4: Bump last_reviewed
- **Archivos:** `docs/api/MCP.md`
- **Acción:** Cambiar `last_reviewed: 2026-07-10` → `2026-07-21`
- **Verify:** grep "last_reviewed" muestra 2026-07-21
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop o docs-api-fixes
- **Decisiones:** El CLI binary correcto es `vanta-cli` (crate `vantadb` con feature `cli`). El tool MCP se llama `query` (IQL). Se actualizan ambas referencias.
