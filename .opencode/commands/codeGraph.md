> **ENTRY POINT — CodeGraph Audit Command**
> El agente DEBE leer este archivo cuando el usuario envía `/codeGraph` (o `/codeGraph <scope>`).
> Orquesta auditorías estructurales/semánticas de VantaDB usando **CodeGraph** + **codebase-memory-mcp**
> (CBM), **una por una**, y produce un informe final con tareas accionables clasificadas por tipo de acción.

**Anti-proliferación (Regla del sistema):** `/codeGraph` NO reemplaza a `/audit`
(gate mecánico fmt/clippy/test/deny vía `unified-review`), `/webperf` (performance web),
ni a `vanta-arch` (decisiones de arquitectura). Cubre el espectro *estructural/semántico*
que esas herramientas no abordan: acoplamiento, dead-code, complejidad, impacto de cambios,
API pública, fronteras FFI, deriva de documentación. Complementa, no solapa.

**Skills a cargar:** `ponytail` (full), `codebase-memory` (skill del proyecto), `progreso`.

**Prerequisitos:**
- CodeGraph indexado: verificar con `codegraph_status` / `codegraph_explore "status"`.
- CBM indexado: `codebase-memory-mcp_list_projects` → debe listar `C-Users-Eros-VantaDB-Proyect-VantaDB`.
  Si no aparece: `codebase-memory-mcp_index_repository` en modo `moderate` (`full` falla en este repo).
- Project key CBM: `C-Users-Eros-VantaDB-Proyect-VantaDB`.

**Contrato de scope:** `<scope>` opcional — `--crate <name>`, `--path <dir>`, o nada = workspace completo.
Pasar el scope a cada consulta de abajo.

## Fase 0 — Sanity (antes de auditar)

0. **Mantenimiento:** al cargar la skill `codebase-memory` ya corre su ritual automático
   (health check CBM+CodeGraph + re-index `moderate` si >7d o falla, leyendo
   `maintenance-state.json`). Si no lo hiciste, ejecutalo ahora — sin índices sanos,
   las fases siguientes dan falsos negativos.
1. `codebase-memory-mcp_check_index_coverage(scopes:["."])` → confirmar cobertura del workspace.
   Si hay archivos `parse_partial`/`skipped`, grep allí antes de afirmar "no existe".
2. `git diff --name-only HEAD` → capturar archivos sin commit para la auditoría de impacto (Fase 7).
   Si vacío, usar `HEAD~1`.

## Fases de auditoría (una por una, en orden)

Cada fase: correr las llamadas indicadas, registrar hallazgos con `file:line` + evidencia (query usada).
No saltar fases.

### Fase 1 — Arquitectura y capas (CBM)
- `codebase-memory-mcp_get_architecture(aspects:["clusters","layers","boundaries","cycles"])` →
  mapear módulos *de facto* (Leiden) vs carpetas; detectar ciclos.
- Para cada cluster sospechoso: `codegraph_explore "<cluster> responsabilidades"` para leer source y validar fronteras.
- Hallazgo tipo: acoplamiento indebido, capa violada, ciclo entre crates → 🔧 ARREGLAR / ♻️ REFACTORIZAR.

### Fase 2 — Acoplamiento inter-crate (CBM)
- `codebase-memory-mcp_query_graph` para edges `CALLS`/`WORKSPACE_DEPENDENCY` entre crates;
  identificar "god-crate" (muchas dependencias entrantes) y ciclos.
- `codegraph_explore` para confirmar dependencias reales vs transitivas.
- → ♻️ REFACTORIZAR (acoplamiento) / 🔍 INVESTIGAR.

### Fase 3 — Código muerto / símbolos huérfanos (CBM + CodeGraph)
- `codebase-memory-mcp_query_graph`: funciones/clases sin inbound ni outbound edges (grado 0 o solo self).
- `codegraph_explore` del símbolo para confirmar que no es API pública ni usado por tests.
- → ♻️ REFACTORIZAR (borrar) / 🔍 INVESTIGAR si es API pública.

### Fase 4 — Duplicación semántica (CBM)
- `codebase-memory-mcp_search_graph(semantic_query:[...])` por dominios clave
  (parseo, serialización, distance, storage) → funciones conceptualmente idénticas.
- `codegraph_explore` de ambas para comparar firmas.
- → ♻️ REFACTORIZAR (consolidar) / 🔍 INVESTIGAR.

### Fase 5 — Hotspots de complejidad (CBM)
- `codebase-memory-mcp_query_graph` filtrando:
  `transitive_loop_depth >= 3`, `linear_scan_in_loop >= 1`, `alloc_in_loop >= 1`,
  `unguarded_recursion = true`, `complexity` alto.
- `codegraph_explore` del hot path para leer el bucle.
- → ⚡ OPTIMIZAR (con benchmark per Regla 9) / ♻️ REFACTORIZAR.

### Fase 6 — API pública / breaking changes (CodeGraph + CBM)
- `codegraph_explore "pub fn / pub struct en lib.rs de <crate>"` → superficie pública de cada crate.
- `codebase-memory-mcp_trace_path(function_name, direction:"inbound")` → callers externos
  (bindings PyO3/WASM/TS).
- Contrastar contra `cargo semver-checks` (gate de release) → listar superficie en riesgo.
- → ✏️ MODIFICAR (API) / 🔧 ARREGLAR (contrato roto) / 🔍 INVESTIGAR.

### Fase 7 — Radio de impacto / pre-merge (CBM)
- `codebase-memory-mcp_detect_changes(project:"C-Users-Eros-VantaDB-Proyect-VantaDB",
  scope:"impact", direction:"inbound", base_branch:"main", since:"HEAD")` → qué toca el diff actual.
- `codegraph_explore` de los símbolos impactados para leer source.
- → 🔍 INVESTIGAR (alcance del cambio) / 🔧 ARREGLAR (efecto colateral).

### Fase 8 — Fronteras FFI / unsafe (CBM + CodeGraph)
- `codebase-memory-mcp_search_code(pattern:"unsafe|PyO3|#[pyfunction]|wasm", project:"C-Users-Eros-VantaDB-Proyect-VantaDB")`
  → bloques unsafe y fronteras FFI.
- `codegraph_explore` de cada bloque unsafe para blast-radius.
- Validar `// SAFETY:` y test de Miri (Regla 4). Ausente → 🔧 ARREGLAR.
- → 🔧 ARREGLAR / 🔍 INVESTIGAR (seguridad).

### Fase 9 — Fragilidad (CodeGraph + CBM)
- `codegraph_explore "unwrap! panic! todo! expect("` + `codebase-memory-mcp_search_graph(name_pattern:".*(unwrap|panic|todo|expect).*")`.
- `codegraph_explore` para contexto (¿es recuperable? ¿hay logging?).
- → 🔧 ARREGLAR / ✏️ MODIFICAR.

### Fase 10 — Gaps de cobertura de tests (CBM)
- `codebase-memory-mcp_query_graph`: símbolos de `src/` no referenciados por archivos `test`/`tests/`.
- → ✏️ MODIFICAR (agregar test) / 🔍 INVESTIGAR.

### Fase 11 — Deriva de documentación (CBM + CodeGraph)
- `codebase-memory-mcp_search_graph` de símbolos API vs `docs/api/`; `codegraph_explore` para firma actual.
- Doc no coincide → ✏️ MODIFICAR (doc-driven dev, Regla 3).

### Fase 12 — ADRs / decisiones (CBM)
- `codebase-memory-mcp_manage_adr(project:"C-Users-Eros-VantaDB-Proyect-VantaDB", mode:"get")`
  → listar ADRs; verificar que decisiones de arquitectura estén registradas (Regla 5).
  Faltante → 🔍 INVESTIGAR (proponer ADR).

## Fase Final — Consolidar y reportar

1. **Agrupar hallazgos** en 5 buckets de acción:
   - 🔧 **ARREGLAR** (fix): bugs, contratos rotos, unsafe sin `SAFETY`.
   - ✏️ **MODIFICAR** (modify): API, docs, config, tests faltantes.
   - ⚡ **OPTIMIZAR** (optimize): hotspots (al ejecutar, requiere benchmark per Regla 9).
   - ♻️ **REFACTORIZAR** (refactor): duplicación, dead code, acoplamiento.
   - 🔍 **INVESTIGAR** (investigate): riesgos, semántica a confirmar, ADR pendiente.
2. **Severidad:** Critical / High / Medium / Low (por impacto + alcance).
3. **Escribir informe** `docs/reviews/codegraph-<YYYYMMDD>-<HHMMSS>.md`:
   - Resumen ejecutivo (conteo por bucket + severidad).
   - Por cada fase: hallazgos con `file:line`, evidencia (tool + query), severidad, acción sugerida.
   - Tabla consolidada: `| ID | Título | Acción | Severidad | Ubicación | Recomendación |`.
4. **Backlog (FIND-*)**: cada hallazgo ≥ Medium → fila en `docs/Backlog.md`
   sección `## Hallazgos pendientes de reportes`:
   `| FIND-<n> | <desc> | ref: codegraph-<ts> | 🟡 | <acción> |`
   (numerador continuar de `rg -o "FIND-\d+" docs/Backlog.md | sort -V | tail -1`;
   Origen obligatorio = este reporte; esquema único `prompts/findings.md`).
5. **INDEX**: fila en `docs/reports/INDEX.md` apuntando al reporte.
6. **Mensaje final**: resumen de conteos por bucket + ruta del reporte +
   "Ejecutá `/pipeline plan docs/Backlog.md` para triage de FIND-*".

## Notas
- **Solo lectura:** este comando NO edita código (es auditoría). Los fixes se delegan vía `FIND-*`
  a `/pipeline task` / `vanta-worker`.
- Respeta Regla 4 (unsafe), Regla 9 (optimizar requiere benchmark), Regla 3 (doc-driven).
- Si el usuario pasa `--apply`, NO aplica: la aplicación es tarea separada vía backlog.
- Complementa `/audit certify` (mecánico) — corrélo antes de merge para el gate duro.
