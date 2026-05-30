# Walkthrough: Estabilización Post-CUARENTENA-01

## Contexto
Tras completar la refactorización CUARENTENA-01 (aislamiento de código experimental LISP y gobernanza en subcrates separados), el pipeline de verificación `./dev-tools/verify.ps1` presentaba 3 tests fallando como consecuencia directa de cambios arquitecturales no reflejados en el suite de tests.

---

## Bug 1: `structured_api_v2_certification`

### Síntoma
```
panicked at tests\api\structured_api_v2.rs:36:18:
called `Option::unwrap()` on a `None` value
```

### Causa raíz (dual)
**1a.** El closure en `.find()` llamaba `.unwrap()` sobre `get_field("label")`. El `volatile_cache` contiene nodos de distintos tipos; cualquier nodo sin campo "label" hace explotar el unwrap antes de encontrar los nodos buscados.

**1b.** `UnifiedNode::new()` inicializa con `tier: NodeTier::Cold`. El método `storage.insert()` solo agrega al `volatile_cache` si `tier == Hot`. Por tanto, todos los INSERT vía IQL dejaban los nodos **únicamente** en el backend persistente, nunca en el cache que el test inspeccionaba.

### Fixes aplicados
```rust
// tests/api/structured_api_v2.rs — uso seguro de Option
- .find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S1"))
+ .find(|(_, n)| n.get_field("label").and_then(|v| v.as_str()) == Some("S1"))

// src/executor.rs — nodo recién insertado es Hot por definición
  let mut node = UnifiedNode::new(insert.node_id);
+ node.tier = crate::node::NodeTier::Hot;
```

---

## Bug 2: `version_coherence public_surfaces_report_same_version`

### Síntoma
```
panicked at tests\version_coherence.rs:13:37:
read file: Os { code: 2, kind: NotFound, "El sistema no puede encontrar el archivo especificado." }
```

### Causa raíz
El test buscaba el archivo `vantadb-server/src/mcp.rs` para verificar que el handler MCP usa `metadata::reported_version()`. Durante CUARENTENA-01, ese archivo fue eliminado y el código MCP movido a `vantadb-mcp/src/lib.rs`.

### Fix aplicado
```rust
// tests/version_coherence.rs
- let mcp = read(root.join("vantadb-server").join("src").join("mcp.rs"));
+ // MCP was extracted to its own crate during CUARENTENA-01.
+ let mcp = read(root.join("vantadb-mcp").join("src").join("lib.rs"));
```

---

## Bug 3: `mcp_integration mcp_protocol_certification`

### Síntoma
```
panicked at vantadb-server\tests\mcp_integration.rs:59:9:
assertion failed: lisp_res["content"][0]["text"].as_str().unwrap().contains("affected_nodes")
```

### Causa raíz
El test enviaba una query LISP `(INSERT :node {:label "MCP_TEST"})` al tool `query_lisp`. Tras CUARENTENA-01, `execute_hybrid` detecta el `(` inicial y retorna `Err("LISP queries require the experimental-lisp extension/crate.")`. El handler devuelve `{"isError": true, ...}` en lugar de `{"affected_nodes": ...}`.

### Fix aplicado
```rust
// vantadb-server/tests/mcp_integration.rs
- "arguments": { "query": "(INSERT :node {:label \"MCP_TEST\"})" }
+ // Note: 'query_lisp' routes through execute_hybrid (IQL) since LISP was extracted.
+ "arguments": { "query": "INSERT NODE#999 TYPE node { label: \"MCP_TEST\" }" }
```

---

## Resultado final

```
Summary [64.524s] 131 tests run: 131 passed, 10 skipped
[PASSED] Rust Tests (Nextest)
SUCCESS: All local checks passed cleanly!
git push origin main → Everything up-to-date
```

### Archivos modificados
| Archivo | Tipo de cambio |
|---------|---------------|
| `tests/api/structured_api_v2.rs` | Bug fix (seguridad Option) |
| `src/executor.rs` | Bug fix (semántica NodeTier) |
| `tests/version_coherence.rs` | Actualización de path post-refactorización |
| `vantadb-server/tests/mcp_integration.rs` | Actualización de sintaxis post-refactorización |
