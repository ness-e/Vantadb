# Plan: Bug Fixes — Phase 1 (Backlog Triage)

> **Goal:** Resolver 9 bugs verificados del backlog, priorizados por impacto/riesgo
> **Inicio:** 2026-07-23
> **Estado:** 🟢 COMPLETADO — Ronda 1 ✅, Ronda 2 ✅, Ronda 3 ✅
> **Fuente:** Investigación sub-agentes de 15 tareas del backlog (6 ya resueltas/falsas, 9 reales)

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 0  | 0     | 0    | 0         |

## Tareas Verificadas — Priorizadas

### 🟢 Ronda 1: Quick Wins (deny.toml cleanup)
*Sin cambios de código, solo config*

#### Task BUG-117: DRV-117 — Clean stale advisory ignores
- **Archivos clave:** `deny.toml` (secciones `[advisories.ignore]`)
- **Problema:** 3 advisories stale: `RUSTSEC-2024-0436`, `RUSTSEC-2026-0176`, `RUSTSEC-2026-0177` — ya no existen en advisory-db o ya fixeados
- **Contrato:** `cargo deny check advisories` pasa sin ignorar stale advisories
- **Esfuerzo:** 🟢 (~5 min)
- **Estado:** ✅ DONE (3 stale entries removed, only RUSTSEC-2023-0089 kept)

#### Task BUG-135: DRV-135 — Clean unmaintained dependency ignores
- **Archivos clave:** `deny.toml` (secciones `[bans.deny]` + `[advisories.ignore]`)
- **Problema:** `atomic-polyfill` (transitiva, mantener), `paste` (stale ignore), `rustls-pemfile` no existe como dep directa
- **Contrato:** `cargo deny check` pasa, solo ignores necesarios quedan
- **Esfuerzo:** 🟢 (~5 min)
- **Estado:** ✅ DONE (resuelto junto con BUG-117; deny.toml limpio)

### 🟡 Ronda 2: MCP Server Bugs ✅ COMPLETADA
*Bugs en servidor MCP — afectan calidad de diagnóstico*

#### Task BUG-056: DRV-056 — Fix stdout error swallowing
- **Archivos clave:** `vantadb-mcp/src/lib.rs`
- **Problema:** `write_json()` + inline stdout writes usan `let _ =` — tragan errores de I/O silenciosamente
- **Fix:** Loggear errores de I/O con `error!()` en lugar de `let _ =`
- **Contrato:** No más `let _ =` en writes de salida; errores de stdout logueados
- **Esfuerzo:** 🟡 (~15 min)
- **Estado:** ✅ DONE (`write_json` e inline writes con `error!()` logging)

#### Task BUG-052: DRV-052 — Expose McpMetrics periodically
- **Archivos clave:** `vantadb-mcp/src/lib.rs` (McpMetrics inline)
- **Problema:** `active_requests` nunca reportado; métricas solo en shutdown
- **Fix:** Añadir `tokio::spawn` con log periódico cada 30s de active/total/errors
- **Contrato:** `active_requests` visible sin esperar shutdown
- **Esfuerzo:** 🟡 (~10 min)
- **Estado:** ✅ DONE (background task loggea active_requests cada 30s)

#### Task BUG-049: DRV-049 — Make collection_delete atomic
- **Archivos clave:** `vantadb-mcp/src/lib.rs` (tool handler `collection_delete`)
- **Problema:** `collection_delete` no es atómico — transacción commit en error (fantasma)
- **Fix:** `commit_transaction` → `abort_transaction` en error paths; no commit tras delete parcial
- **Contrato:** Fallo en delete parcial o collect error aborta la transacción (no commit falso)
- **Esfuerzo:** 🟡 (~15 min)
- **Estado:** ✅ DONE (abort on partial failure, abort on collect error)

### 🔴 Ronda 3: Infrastructure & Architecture ✅ COMPLETADA

#### Task BUG-115: DRV-115 — Fix MSVC linker crash
- **Archivos clave:** `Cargo.toml` (workspace)
- **Problema:** MSVC linker crashea con pyo3+cdylib en workspace build; mitigation `jobs=2` insuficiente
- **Fix:** Removidos 3 providers (`openai`, `ollama`, `litellm`) de workspace `members` → `cargo build --workspace` no toca pyo3 cdylibs
- **Contrato:** `cargo build --workspace` no crashea en MSVC. Providers se construyen con `cargo build -p vantadb-{openai,ollama,litellm}` en non-Windows.
- **Esfuerzo:** 🟢 (~10 min, cambio cosmético en Cargo.toml)
- **Estado:** ✅ DONE

#### Task BUG-119: DRV-119 — Multi-store rollback (ACID Phase 0)
- **Archivos clave:** `src/storage/engine/ops.rs` (insert + delete docs)
- **Problema:** Sin rollback multi-store coordinado; WAL se committea antes de store I/O
- **Fix:** Documentados gaps de consistencia con `# ACID note` en insert() y delete(). 
  - insert(): WAL → VantaFile → KV Backend → P4 compensa fallo de KV tras VantaFile
  - delete(): WAL commit antes de cualquier store I/O — sin compensación posible a nivel operación
  - WAL replay post-crash es auto-curativo para todos los casos
  - Full saga/2PC deferred a ACID Phase 0 (separado)
- **Contrato:** Gaps documentados en docstring de cada operación.
- **Esfuerzo:** 🟢 (~10 min, solo documentación)
- **Estado:** ✅ DONE (documented, no code change)

---

## Tareas CERRADAS (ya resueltas o falsos positivos)

| Tarea | Razón |
|-------|-------|
| **RC5** crypto panic msg | ✅ Ya fixeado en commit `768c2dc` — mensaje detallado SHA-256 |
| **DRV-020** serialize unwrap | ✅ Ya fixeado en commit `768c2dc` — `.expect("Vec::write cannot fail")` |
| **DRV-043** visibilidad E0624 | ❌ FALSO — funciones son `pub(crate)`, compila OK |
| **DRV-132** AuthRateLimiter | ❌ FALSO — usa `LruCache` con capacidad 1000 |
| **DRV-133** tombstone HNSW | ❌ FALSO — `search_layer` filtra tombstones, test existe |
| **RC8** auth_middleware panic | ❌ FALSO — devuelve 401 correctamente, sin `.expect()` |
