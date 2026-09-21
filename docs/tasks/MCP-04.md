# MCP-04: S4 — Sin validación de dimensionalidad en `search_semantic`

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 1)
- **Fuente:** Backlog P22 `MCP-04` (test-busqueda.py T19)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 5-10
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T18:30
- **Estado:** 🟡 IMPLEMENTADO — verificación de batería bloqueada por stack overflow separado (MCP-15); T19 verificado aislado ✅
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3 steps (2 ✅, 1 🟡)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/handlers/tools.rs` (search_semantic, search_memory vectorial) |
| Callees | `VantaError::DimensionMismatch` (src/error.rs), índice (dims del espacio vectorial) |
| Implicaciones | Query mal dimensionada hoy devuelve distancias 0.0 silenciosamente; el fix agrega error explícito — cambia el contrato de error (mejora) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `vantadb-mcp/src/handlers/tools.rs` (710 líneas, completo), `src/error.rs` (DimensionMismatch líneas 100-107), `src/index/graph.rs` (CPIndex 330, HnswNode 145, vector_slice 162, get_entry_point 465), `src/storage/engine/mod.rs` (vec_index 443), `src/cli_handlers/server.rs` (cmd_server_mcp 245-329), `vantadb-mcp/Cargo.toml`, `vantadb-server/Cargo.toml`, `src/index/mod.rs` (pub use graph::* 23)
- **Archivos referenciados hacia dentro:** `VantaError::DimensionMismatch { expected, got }`
- **Archivos que referencian a los editados:** skill `vantadb-mcp` api-reference (MCP-08 doc del error)
- **Veredicto impacto:** BAJO — cambio localizado en handler; agrega validación en trust boundary de input

## Contrato
"`python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` T19: query 3-dim contra índice 4-dim → error `DimensionMismatch` con expected=4, got=3 (isError content), no distancias 0.0; `cargo check -p vantadb-mcp` ✅"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** queries bien dimensionadas siguen funcionando sin cambio de respuesta; el error llega como isError content (canal MCP documentado en F6/MCP-11)
- **Comandos de verificación:** `cargo check -p vantadb-mcp` ✅; test-busqueda.py T19 ✅ (verificado aislado; corrida completa bloqueada por MCP-15)
- **Deuda pendiente:** ver MCP-15 (stack overflow del child en search_semantic)

## Fase 1 — Evidencia de Debugging (GATE — Bug)
- **Repro:** índice 4-dim, `search_semantic` con query `[0.1,0.2,0.3]` (3-dim) → éxito con distances 0.0
- **Hipótesis:** el handler no valida la dim del query contra la dim del índice; `VantaError::DimensionMismatch` existe pero nunca se invoca en este path
- **1 variable controlada:** agregar validación en search_semantic (y search_memory vectorial) — UNA validación por intento
- **Test RED:** T19 (FAIL confirmado 2026-08-17)
- **HALLAZGO (sesión paralela MCP-03 + repro propio):** T19 en la corrida completa de `test-busqueda.py` falla por un bug SEPARADO — el child `vantadb-server` crashea con **stack overflow** (`thread 'tokio-rt-worker' (18480) has overflowed its stack`) en T17 (`search_semantic` con dim VÁLIDA 4-dim) tras la secuencia T09-T16. El crash mata el server → T18/T19 heredan `[Errno 22] Invalid argument` (pipe roto). NO es falta de validación: la validación de MCP-04 corta ANTES del search, pero T17 no la dispara (dims correctas). Evidencia: `repro-full.py` reproduce el crash 100% (stderr en `C:\Users\Eros\AppData\Local\Temp\opencode\crash-stderr.txt`, línea 20). **Bug separado sugerido: MCP-15** (stack overflow en search_semantic del child — recursión probablemente en serialización del node payload, como reportó MCP-03).

## Steps

### Step 1: Leer handler y error.rs (Regla 0) ✅
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `src/error.rs`
- **Acción:** leer completo el handler de search_semantic/search_memory y la variante `DimensionMismatch`; identificar dónde obtener la dim del índice
- **Verify:** — (lectura)
- **Estado:** ✅ COMPLETED 2026-08-17T18:30
- **Hallazgo clave:** `HnswConfig` NO tiene campo `dim`; la dim del índice se deriva de los nodos (`CPIndex.nodes` → `HnswNode::vector_slice()`). `storage.vec_index()` (pub, engine/mod.rs:443) expone el índice.

### Step 2: Implementar validación ✅
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** antes de buscar, validar `query_vec.len() == dims_índice`; si no, devolver error con `DimensionMismatch { expected, got }` en formato isError content
- **Verify:** `cargo check -p vantadb-mcp` ✅ (2026-08-17T18:25, sin errores nuevos; warnings pre-existentes de flat.rs)
- **Estado:** ✅ COMPLETED 2026-08-17T18:25
- **Implementación:**
  - Helper `index_vector_dim(storage)` al final de tools.rs: `storage.vec_index().nodes.iter().find_map(|e| e.value().vector_slice().map(|v| v.len()))` — dim del primer nodo con vector; índice vacío → `None` → skip validación.
  - `search_semantic`: tras `validate_vector`, si `vector.len() != expected` → `Ok(error_content(VantaError::DimensionMismatch { expected, got }.to_string()))`.
  - `search_memory`: misma validación sobre `query_vector` solo si no está vacío (text-only search no aplica).
  - Error como isError content (canal F6/MCP-11). Trust boundary: validación en el handler, no en profundidad (SECURITY del task file).

### Step 3: Verificar con batería 🟡
- **Archivos:** — (binario rebuild)
- **Acción:** rebuild binario, re-ejecutar test-busqueda.py T19
- **Verify:** T19 ✅ en aislamiento — `diag-mcp04.py t19` (server limpio, solo seed + T19): `isError=True`, texto `"Vector dimension mismatch: expected 4, got 3"` (expected=4, got=3 exactos, sin distancias 0.0). La corrida completa (`test-busqueda.py`) queda bloqueada en T17 por MCP-15 (stack overflow) — el server no se recupera y T18/T19 fallan por pipe roto, NO por la validación.
- **Estado:** 🟡 VERIFICADO AISLADO; batería completa pendiente de MCP-15 (desbloquea cuando se arregle el stack overflow)
- **Nota:** `cargo build -p vantadb-server` + copia a `C:\Users\Eros\.cargo\bin\vantadb-server.exe` (MCP-03 ya estableció este flujo de rebuild).

## Dependencias
- Ninguna (independiente; desbloquea MCP-08)
- **BLOQUEANTE (no de MCP-04):** MCP-15 (stack overflow del child en search_semantic) impide la corrida completa de test-busqueda.py

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit
- **Enfoque:** ¿la validación está en el trust boundary correcto (handler)? ¿no rompe la firma del search?
- **Cómo se probó:** T19 con salida real (`diag-mcp04.py t19` → isError=True, expected=4 got=3)
- **Veredicto:** ⏳ pendiente

## Notas
- SECURITY: toca input de usuario (query vector) → validación en handler es el lugar correcto (ver task.md fases SECURITY)
- T19 AISLADO PASA aunque la batería completa esté bloqueada: la validación se ejecuta en la frontera del handler, sin recursión, y corta ANTES del search (no toca el path del stack overflow).

## Context Save Point
- **Fecha:** 2026-08-17T18:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** validación en handler con `VantaError::DimensionMismatch` (reutiliza el error core existente); dim del índice derivada de nodos vía `storage.vec_index()`
- **Problemas conocidos:** MCP-15 (stack overflow del child en search_semantic) — bug separado, bloquea la corrida completa de test-busqueda.py; T15 (explain=true) falla por contrato de respuesta (campo explanation ausente) — verificar si es otra tarea MCP
- **Próxima tarea:** MCP-09
