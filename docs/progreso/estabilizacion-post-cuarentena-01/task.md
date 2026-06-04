# Fase POST-CUARENTENA: Estabilización de Tests

## Objetivo
Llevar el pipeline `./dev-tools/verify.ps1` a 0 fallos tras la refactorización CUARENTENA-01.

---

## Bugs corregidos

### Bug 1 — structured_api_v2_certification (RESUELTO ✅)
- **Causa raíz 1:** `.unwrap()` dentro del closure de `.find()` → pánico en cualquier nodo sin campo "label"
- **Causa raíz 2:** `NodeTier::Cold` por defecto → nodos INSERT nunca entraban al `volatile_cache`
- **Fix 1:** `get_field("label").and_then(|v| v.as_str())` en `tests/api/structured_api_v2.rs`
- **Fix 2:** `node.tier = NodeTier::Hot` en `src/executor.rs` (Statement::Insert)
- `[x]` PASA: `PASS [0.678s] vantadb::structured_api_v2 structured_api_v2_certification`

### Bug 2 — version_coherence (RESUELTO ✅)
- **Causa raíz:** Path hardcodeado a `vantadb-server/src/mcp.rs` eliminado en CUARENTENA-01; código MCP ahora en `vantadb-mcp/src/lib.rs`
- **Fix:** Actualizar path en `tests/version_coherence.rs` línea 65
- `[x]` PASA: `PASS [0.017s] vantadb::version_coherence public_surfaces_report_same_version`

### Bug 3 — mcp_integration mcp_protocol_certification (RESUELTO ✅)
- **Causa raíz:** Test enviaba query LISP `(INSERT :node {...})`. Tras CUARENTENA-01, `execute_hybrid` rechaza queries con `starts_with('(')`, retornando error en lugar de `affected_nodes`
- **Fix:** Cambiar query de sintaxis LISP a IQL estable en `vantadb-server/tests/mcp_integration.rs`
- `[x]` PASA: `PASS [0.662s] vantadb-server::mcp_integration mcp_protocol_certification`

---

## Archivos modificados
| Archivo | Cambio |
|---------|--------|
| `tests/api/structured_api_v2.rs` | `.unwrap()` → `.and_then()` en closures de búsqueda |
| `src/executor.rs` | `node.tier = NodeTier::Hot` al crear nodo en Statement::Insert |
| `tests/version_coherence.rs` | Path MCP: `vantadb-server/src/mcp.rs` → `vantadb-mcp/src/lib.rs` |
| `vantadb-server/tests/mcp_integration.rs` | Query LISP → IQL en test del tool dispatcher |

---

## Resultado final
```
Summary [64.524s] 131 tests run: 131 passed, 10 skipped
[PASSED] Rust Tests (Nextest)
SUCCESS: All local checks passed cleanly!
```
**git push origin main → OK**
