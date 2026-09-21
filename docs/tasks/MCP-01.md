# MCP-01: S1 — text_query/hybrid/filters-text rotos vía MCP (text_index not found: bm25)

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 1)
- **Fuente:** Backlog P22 `MCP-01` (batería pruebas 2026-08-17, test-busqueda.py T09/T11/T13)
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 30-60
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T14:30
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 abierta (dónde construir el text index en el path MCP)
- **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-mcp/src/handlers/tools.rs` (search_memory), `vanta-server` CLI server |
| Callees | `src/sdk/serialization/impl_index.rs:15` (ensure_indexes_current), `src/sdk/builder.rs:105` (open_with_config), `src/sdk/api.rs:660` (rebuild_index), `src/text_index.rs:18` (ensure_text_index_query_ready), `src/storage/engine/mod.rs:306` (StorageEngine) |
| Implicaciones | Contrato MCP roto (skill declara text search disponible); no afecta API embedded; requiere decisión de arquitectura (dónde construir el índice) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `vantadb-server/src/main.rs`, `vantadb-mcp/src/handlers/tools.rs` (search_memory)
- **Archivos referenciados hacia dentro:** `StorageEngine::open`, `ensure_indexes_current`
- **Archivos que referencian a los editados:** `vantadb-mcp/tests/mcp_tests.rs` (no cubre text search)
- **Veredicto impacto:** ALTO — fix toca el arranque del server MCP; cualquier opción debe mantener DBs existentes (text index puede construirse lazy)

## Contrato
"`python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` pasa T09 (text_query solo), T11 (hybrid) y T13 (filters text) contra `vanta-cli server --mcp --db <temp-fresh>`; además `cargo check -p vantadb-mcp -p vantadb-server` y `cargo nextest run --profile audit -p vantadb-mcp` pasan"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** DBs existentes sin text index deben seguir abriendo (no panic); el text index se construye sin romper datos vectoriales; el path embedded (`VantaEmbedded::open_with_config`) no cambia de comportamiento
- **Comandos de verificación:** `cargo check -p vantadb-mcp -p vantadb-server` ✅; `cargo nextest run --profile audit -p vantadb-mcp` ✅; `python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` (T09/T11/T13 ✅)
- **Deuda pendiente:** ninguna

## Fase 1 — Evidencia de Debugging (GATE — Bug)
- **Repro:** `vanta-cli server --mcp --db <fresh>` → `search_memory` con `text_query:"hola"` (o hybrid con text_query, o filters en path textual) → error `Search Error: text_index not found: bm25`
- **Hipótesis:** el server MCP abre `StorageEngine` directo sin llamar `ensure_indexes_current()` — esa construcción solo ocurre en `open_with_config` (builder.rs:105) o `rebuild_index()` (api.rs:660), ninguna expuesta vía MCP. Los puts escriben postings (metrics `text_postings_written: 26`) pero el estado del índice nunca se crea → `ensure_text_index_query_ready` (text_index.rs:18) falla siempre
- **1 variable controlada:** UNA opción de construcción del text index por intento
- **Test RED:** test-busqueda.py T09 falla (RED confirmado en batería 2026-08-17)

## Steps

### Step 1: Decidir punto de construcción del text index en el path MCP (vanta-arch)
- **Archivos:** `vantadb-server/src/main.rs`, `vantadb-mcp/src/handlers/tools.rs`, `src/sdk/api.rs:660`
- **Acción:** evaluar 3 opciones: (a) llamar `ensure_indexes_current` en arranque del server; (b) exponer `rebuild_index` como tool MCP; (c) lazy-build en primer put/search. Elegir UNA con justificación (impacto en startup time, DBs existentes, idempotencia)
- **Verify:** ADR o nota de decisión en task file; `cargo check -p vantadb-mcp` ✅
- **Estado:** ⬜ PENDING

### Step 2: Implementar el fix
- **Archivos:** el elegido en Step 1
- **Acción:** implementar la construcción del text index en el path MCP (arranque o lazy), con logging
- **Verify:** `cargo check -p vantadb-mcp -p vantadb-server` ✅
- **Estado:** ⬜ PENDING

### Step 3: Rebuild binario y re-ejecutar batería
- **Archivos:** — (binario `vanta-cli` 0.5.0 → rebuild local)
- **Acción:** `cargo build -p vantadb-cli` (o como se llame el binario) y copiar a PATH; re-ejecutar `test-busqueda.py` completo
- **Verify:** `python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` → T09/T11/T13 ✅ (mínimo 20/20 o solo FAILs documentados distintos a S1)
- **Estado:** ⬜ PENDING

### Step 4: Tests de regresión
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs` (agregar test de text search si no existe)
- **Acción:** agregar test MCP de text_query/hybrid/filters-text en DB fresca
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp` ✅
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (primer bug del Bloque 1; desbloquea MCP-05)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit (código) + vanta-arch (approach)
- **Enfoque:** ¿la opción elegida (arranque vs lazy vs tool) es la correcta para DBs existentes y startup?
- **Cómo se probó:** test-busqueda.py T09/T11/T13 verdes con salida real (no auto-reporte)
- **Veredicto:** ⏳ pendiente

## Notas
- Root cause trazado por sub-agente de pruebas 2026-08-17 (sesión ses_fef7fccf4ffeoVZr14uqNRmVD4)
- Bloque 2 (MCP-05) documentará la realidad corregida

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** pendiente Step 1 (vanta-arch)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-02
