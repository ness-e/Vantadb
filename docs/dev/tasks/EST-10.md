# EST-10: Barrido API stale fuera de CI (~35 archivos → ~55 reales)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-estabilizacion-pendiente.md` (§EST-10) + `docs/dev/plans/2026-09-24-sesion-continuidad.md` (§6)
- **Backlog:** fila `EST-10` en P57 (`docs/dev/Backlog.md`)
- **Creado:** 2026-09-24
- **last-synced:** 2026-09-24
- **Estado:** ⏳ IN PROGRESS
- **Tipo:** docs/código no-CI (bulk idempotente) — ejecución inline (subagentes caídos por provider gating)

## Objetivo
Migrar las referencias a la API Python pre-0.5.0 en superficies vivas (docs de usuario, READMEs, benches no-CI, scripts, skills, docs de estrategia) a la API canónica actual.

## Mapeo canónico (verificado contra `vantadb_py.pyi` + `BINDINGS_NAMESPACES.md` + `PYTHON_SDK.md`)
| Viejo (stale) | Nuevo (canónico) |
|---|---|
| `VantaDB(path)` / `vanta.VantaDB(` / `vantadb.VantaDB(` | `Client(path)` / `vanta.Client(` / `vantadb.Client(` |
| `AsyncVantaDB` | `AsyncClient` (STU-002) |
| `db.search_memory(...)` | `db.search(namespace, query_vector, ...)` (AST-008) |
| `db.get_memory(ns, key)` | `db.memory.get(ns, key)` (AST-012) |
| `db.list_memory(ns, ...)` | `db.memory.list(ns, ...)` (AST-012) |
| `db.delete_memory(ns, key)` | `db.memory.delete(ns, key)` (AST-012) |
| `db.search(vector, top_k)` (ANN puro) | `db.search_vector(vector, top_k)` (AST-008) |
| `Client(":memory:")` | `Client(":memory:", backend="memory")` |

## Blast Radius
- **Alcance (superficies vivas):** `docs/user/**` (glosario/operations/blog/learning/benchmarks), `README.md`, `README_ES.md`, `vantadb-ts/README.md`, `docs/api/PYTHON_SDK.md`, `docs/dev/{vision,graphrag,architecture,operations,strategy}`, `skills/vantadb/SKILL.md`, `skills/vantadb-mcp/references/api-reference.md`, benches no-CI + `evals/` + `dev-tools/validate_doc_snippets.py` + `vantadb-python/{sanity_check,migrate/*}`.
- **Excluido a propósito (registros históricos / otro scope):**
  - Histórico congelado: `docs/dev/{plans,reviews,research,avance,archive}/**`, ADRs (`architecture/adr/**`), `docs/dev/strategy/VantaDB-Informe*.md`, `docs/CHANGELOG.md`, `docs/dev/Backlog.md` (cita patrones), `test_subclients.py` (guard AST-012).
  - **MCP scope** (nombre de tool `search_memory` sigue vigente hasta API-04): `skills/vantadb-mcp/SKILL.md`, `references/recall-policy.md`, `scripts/test-mcp.py` (comentario), `docs/api/MCP.md`, `docs/api/scores.md`, `docs/api/BINDINGS_NAMESPACES.md` (notas de migración), `desktop/README.md`, `embeddings/README.md`, `UPGRADE.md`, `EDITOR_INTEGRATIONS.md`, `glosario/mcp.md`.
  - Falsos positivos de patrón: `get_memory_stats()` (`MEMORY_TELEMETRY.md` — API Rust interna, intacta).
- **Espejo skills (FIND-83):** `skills/vantadb/SKILL.md` y `skills/vantadb-mcp/references/api-reference.md` → copiar a `.opencode/skills/...` tras editar (hash-SAME).

## Contrato
> (a) `py_compile` OK en todos los `.py` modificados; (b) re-grep global: 0 hits en superficies vivas — hits restantes solo en la lista de exclusión (histórico/MCP/falsos positivos); (c) `validate-docs-coverage.ps1` 0 gaps + skills mirror hash-SAME; (d) `dev-tools/validate_doc_snippets.py` ejecuta sin ImportError (snippet harness).

## Herramientas
- Script de barrido idempotente con conteo antes/después por archivo (`$TEMP/opencode/est10-sweep.py`), `python -m py_compile`, `git grep`, edits manuales para tablas/secciones con semántica.

## Steps
### Step 1: task file + script de barrido con whitelist
- **Estado:** ⏳

### Step 2: barrido automático (reglas por whitelist)
- **Estado:** ⬜

### Step 3: revisiones manuales (secciones semánticas)
- README.md nota de naming (claim falso "get_memory/search_memory stay canonical"); `vantadb-ts/README.md` tabla Cross-SDK; `skills/vantadb-mcp/references/api-reference.md` sección "VantaDB Class" (reescritura); `PYTHON_SDK.md` (Async + posibles secciones node-level); `PERFORMANCE_GUIDE.md` caja ASCII línea 26.
- **Estado:** ⬜

### Step 4: verificación (contrato a-d) + mirror skills
- **Estado:** ⬜

### Step 5: commit + push + progreso
- **Estado:** ⬜

## Dependencias
- Ninguna. Nota: WIRE-10 (P56) cubre `install.sh`/Colab/hooks — alcance distinto; Colab queda para WIRE-10.

## Notas
- La estimación del plan (~35 archivos) subestimó los hits en `docs/user/**` (glosario) y docs/dev no-archivo; el total real de superficies vivas es ~55 archivos.
- `dev-tools/validate_doc_snippets.py` está roto en 2 frentes: import `VantaDB` (API stale) y paths `docs/tutorials`/`docs/QUICKSTART.md` (movidos por C-02 a `docs/user/...`) — se corrigen ambos en este barrido (2 líneas).

## Context Save Point
- **Fecha:** 2026-09-24
- **Branch:** `develop`
- **CI pendiente:** no (no toca CI; verificación local)
- **Decisiones:** alcance = superficies vivas; MCP tool-name hits quedan para API-04; históricos intactos.
- **Problemas conocidos:** `integrations/README.md:65` describe adapters con API vieja (FIND-94) — verificado que `integrations/**/*.py` ya no tiene llamadas stale; nota potencialmente obsoleta → candidata a FIND separado (no en este barrido).
- **Próxima tarea:** EST-05 (verificar bench verde).
