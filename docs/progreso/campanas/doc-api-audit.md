# Correcciones DOC-API (2026-07-21)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-21 — Correcciones de auditoría DOC-API (6/6 tareas completadas)

**Objetivo:** Corregir 9 incidencias (5 críticas, 4 medias) encontradas en auditoría de `docs/api/` — tipos desactualizados, referencias rotas, métodos faltantes, creación de documentación faltante.

**Wave 0 (4 en paralelo):**

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| DOC-API-01 | Corregir EMBEDDED_SDK.md: u64→u128 en node_id, edge.target, firmas | `docs/api/EMBEDDED_SDK.md` | ✅ `184869b` |
| DOC-API-02 | Corregir openapi.yaml: NodeDTO alineado con VantaNodeRecord real | `docs/api/openapi.yaml` | ✅ `eb27b68` |
| DOC-API-03 | Corregir MCP.md: vantadb-cli→vanta-cli, query_lisp→query | `docs/api/MCP.md` | ✅ `7d69416` |
| DOC-API-04 | Corregir PYTHON_SDK.md: +6 métodos faltantes, VectorInput types | `docs/api/PYTHON_SDK.md` | ✅ `fd5a0de` |

**Wave 1 (2 en paralelo):**

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| DOC-API-05 | Corregir TS_SDK.md: +connect_idb(), searchVector naming | `docs/api/TS_SDK.md` | ✅ `92c49bc` |
| DOC-API-06 | Crear IQL.md + verificar HTTP_API.md endpoints | `docs/api/IQL.md` (+213), `docs/api/HTTP_API.md` | ✅ `13c5a0f` |

**Total:** 6/6 tareas completadas, ~0 líneas de código Rust/TS/Python, ~250 líneas de documentación nuevas/corregidas.

**Ids:** `DOC-API-01`, `DOC-API-02`, `DOC-API-03`, `DOC-API-04`, `DOC-API-05`, `DOC-API-06`

**Verificación:** `cargo check` en `vantadb-python/` — 0 errores.

<!-- movido a ARCHIVO_HISTORICO.md -->
