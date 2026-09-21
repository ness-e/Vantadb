# MCP-03: S3 — `search_semantic.distance` = similaridad coseno mal etiquetada

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 1)
- **Fuente:** Backlog P22 `MCP-03` (test-busqueda.py T17/T18)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Rust (API pública — requiere `feat!`/semver si se renombra)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T14:30
- **Estado:** ✅ DONE — fix verificado por test Rust determinístico (33/33, `cargo test -p vantadb-mcp --test mcp_tests` — nextest excluye mcp_tests por default-filter) + repros + 3/3 runs verdes del contrato. Flake residual documentado: stack overflow pre-existente del child `vantadb-server` (ver Notas) — NO relacionado con este fix
- **Incógnitas (uphill):** 1 cerrada (corregir valor vs renombrar campo → opción (a))
- **Pendientes (downhill):** 3 steps DONE

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | consumidores de `search_semantic` vía MCP (clientes OpenCode/Claude/Cursor), `vantadb-mcp/src/handlers/tools.rs` |
| Callees | `src/index/search/nearest.rs:154-159` (adjusted_score Cosine → score = similaridad), `cosine_sim_cached_norms` |
| Implicaciones | **API pública**: renombrar `distance` → `similarity` es breaking (semver major); corregir valor a `1-cosine` cambia ordenación del campo pero no del ranking |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/index/search/nearest.rs` (search_nearest completo)
- **Archivos referenciados hacia dentro:** `cosine_sim_cached_norms`, `DistanceMetric::Cosine`
- **Archivos que referencian a los editados:** handler MCP search_semantic, skill `vantadb-mcp` (MCP-07 doc), tests existentes de búsqueda
- **Veredicto impacto:** ALTO — toca API pública del MCP; decisión de semver

## Contrato
"`python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` T17 (idéntico → 0.0 si es distancia, o campo renombrado a `similarity` con 1.0) y T18 (orden ascendente por distancia) pasan; decisión semver documentada (¿feat!? ¿backwards-compatible?)"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** el RANKING por similaridad no cambia (solo el valor expuesto); cosine sigue siendo la métrica default
- **Comandos de verificación:** `cargo check -p vantadb-mcp` ✅; test-busqueda.py T17/T18 ✅
- **Deuda pendiente:** si se renombra el campo, la skill (MCP-07) y cualquier cliente documentado deben actualizarse en el mismo PR

## Fase 1 — Evidencia de Debugging (GATE — Bug)
- **Repro:** `search_semantic` con query idéntica a un vector → `distance: 1.0` (debería ser 0.0 si es distancia)
- **Hipótesis:** `search_nearest` devuelve similaridad (score) tal cual para Cosine; el handler la serializa como "distance" sin invertir (`1 - score`). La skill documenta "lower is more similar" → valor invertido
- **1 variable controlada:** UNA decisión por intento (invertir valor vs renombrar campo)
- **Test RED:** T17/T18 (FAIL confirmado 2026-08-17)

## Steps

### Step 1: Decidir corrección (vanta-worker + vanta-lead para semver)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `src/index/search/nearest.rs`
- **Acción:** elegir entre: (a) devolver distancia real `1 - cosine_sim` (mantiene nombre `distance`, backwards-compatible en forma pero cambia semántica de valores — minor con doc), o (b) renombrar campo a `similarity` (breaking → `feat!`). Documentar decisión con impacto semver (Regla 7)
- **Verify:** decisión documentada
- **Estado:** ✅ DONE — opción (a): mantener `distance`, exponer `1 - cosine_sim` (Euclidean → `-hit.distance`, SparseDot → passthrough). Semver `fix:` (patch): alinea el valor con el contrato ya documentado "lower is more similar"; NO `feat!` (sin renombrar). Confirmación final del lead pendiente al commitear.

### Step 2: Implementar
- **Archivos:** handler MCP search_semantic + (si aplica) transformación de score
- **Acción:** implementar la opción elegida; si es (a), `distance = 1.0 - score` (o `-score` para euclidean); si es (b), renombrar campo en respuesta
- **Verify:** `cargo check -p vantadb-mcp` ✅
- **Estado:** ✅ DONE — `vantadb-mcp/src/handlers/tools.rs` L489-511: lee `storage.vec_index().config.distance_metric` y convierte por métrica. Solo el handler; core sin tocar (WASM score + similar_to_key). `cargo check -p vantadb-mcp` ✅ (15.95s).

### Step 3: Verificar con batería
- **Archivos:** — (binario rebuild)
- **Acción:** rebuild binario, re-ejecutar test-busqueda.py T17/T18
- **Verify:** T17: idéntico → 0.0 (distancia) o `similarity: 1.0`; T18: orden ascendente ✅
- **Estado:** ✅ DONE — rebuild `vanta-cli` + `vantadb-server` (debug); test-busqueda.py 3/3 runs consecutivos: T17 `dist=0.0 distances=[0.0, 0.0299, 1.0]`, T18 `5 hits, top dist=0.0`, T19 `DimensionMismatch expected 4 got 3`. 16/20 total (T09/T11/T13/T15: "text_index not found: bm25" — pre-existente, fuera de scope).

## Dependencias
- Ninguna (independiente; desbloquea MCP-07)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit
- **Enfoque:** ¿la decisión semver es correcta? ¿el cambio de valor no rompe clientes silenciosamente?
- **Cómo se probó:** T17/T18 con salida real
- **Veredicto:** ✅ APPROVE con changes-required leve (2026-08-17): fix correcto (semver `fix:` defendible, monoticidad verificada vs `adjusted_score`, sin unsafe). 3 fixes de DoD aplicados por el worker: (1) test de contrato `test_mcp_search_semantic_distance_semantics` en mcp_tests.rs; (2) doc sync `docs/api/MCP.md:52` ("lower is more similar; distance = 1 − cosine_similarity"); (3) referencia corregida a `docs/api/MCP.md` (no `docs/api/api-reference.md`, inexistente). Hallazgo MEDIUM: clientes con filtros por umbral `distance > X` se invierten — destacar en changelog del release.

## Notas
- **Flake del contrato (documentado, causa raíz encontrada):** test-busqueda.py T17/T18/T19 fallan intermitentemente con "Server closed stdout on tools/call" / "[Errno 22] Invalid argument". `crash-stderr.txt` muestra la causa: `thread 'tokio-rt-worker' has overflowed its stack` en el child `vantadb-server`. NO relacionado con este fix (una resta f32 no puede causar recursión): (1) los runs 2/3 originales, ANTES del fix, ya crasheaban idéntico; (2) el overflow persiste incluso con `thread_stack_size(8 MiB)` en el runtime tokio (probado y revertido — la recursión es real, no stack chico); (3) el fix está verificado independientemente por el test Rust determinístico `test_mcp_search_semantic_distance_semantics` (33/33) + repros manuales + 3/3 runs verdes del contrato. Bug separado a investigar: recursión en el worker tokio durante search_semantic/search_memory en el child (probablemente serialización del node payload). 
- Orden de ejecución sugerido por el lead: decidir con vanta-lead (semver) ANTES de implementar, para no hacer dos pasos

## Context Save Point
- **Fecha:** 2026-08-17T14:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** pendiente Step 1
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MCP-04
