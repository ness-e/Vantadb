# OLD-19: Rehidratación desde Shadow Archive

**Fuente:** Backlog Phase 9, Estado: ✅ COMPLETED (2026-07-26, verificado batch 6: `recover_archived_nodes` en sdk/builder.rs:158, python lib.rs:1088, MCP lib.rs:1396)  
**Effort:** 🟡 1d (ponytail — ya existe infraestructura)  

## Gate
✅ DO — Dependencia OLD-07 (AutoHot/Cold tiering) ✅ completada. `StorageEngine::recover_archived_nodes()` ya implementado con 6 tests.

## Objetivo
Conectar `StorageEngine::recover_archived_nodes(summary_id)` → SDK público → MCP tool → Python binding.

Ya existe: `src/storage/engine/maintenance.rs:590` + 6 tests en `src/storage/engine/tests/maintenance.rs`:

```rust
pub fn recover_archived_nodes(&self, summary_id: u128) -> Result<Vec<UnifiedNode>>
```
- Scanea `TombstoneStorage` partition
- Busca nodos con edge `belonged_to` → `summary_id`
- Reactiva (ACTIVE + RECOVERED flags, tier=Hot)
- Inserta vía `self.insert(&node)`

## Archivos

| Archivo | Qué hacer |
|---------|-----------|
| `src/sdk/api.rs` | Agregar `VantaEmbedded::recover_archived_nodes(summary_id: u128) -> Result<Vec<VantaNodeRecord>>` |
| `src/sdk/mod.rs` | Re-exportar `recover_archived_nodes` si aplica |
| `src/sdk/builder.rs` | Alternativa: agregar método aquí (más simple, es el entry point de todas las features) |
| `vantadb-mcp/src/lib.rs` | Agregar tool `rehydrate(args.summary_id)` + handler |
| `vantadb-python/src/lib.rs` | Agregar `recover_archived_nodes(summary_id)` pyclass method |
| Test files | Tests para SDK + MCP tool |

## Pasos

### 1. Exponer en VantaEmbedded

En `src/sdk/builder.rs` (o `api.rs`), agregar:

```rust
/// Recover archived (shadow-archived) nodes that belonged to a summary node.
///
/// Scans the TombstoneStorage partition for nodes with a `belonged_to`
/// edge targeting `summary_id`, re-activates them, and inserts them
/// back into the active store.
pub fn recover_archived_nodes(&self, summary_id: u128) -> Result<Vec<VantaNodeRecord>> {
    self.check_read_only()?;
    let engine = self.engine_handle()?;
    let nodes = engine.recover_archived_nodes(summary_id)?;
    Ok(nodes.into_iter().map(Into::into).collect())
}
```

Asegurarse que `VantaNodeRecord` implementa `From<UnifiedNode>` (ya debería).

### 2. Agregar tool MCP

En `vantadb-mcp/src/lib.rs`, agregar handler para `"rehydrate"`:

```rust
"rehydrate" => {
    let summary_id = args["summary_id"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'summary_id'").to_json())?;
    let sid: u128 = summary_id.parse()
        .map_err(|_| McpError::invalid_params("Invalid summary_id (must be u128)").to_json())?;
    let recovered = embedded.recover_archived_nodes(sid)
        .map_err(|e| McpError::internal_error(e.to_string()).to_json())?;
    Ok(text_content(serialize_content(&json!({
        "recovered_count": recovered.len(),
        "summary_id": summary_id,
        "nodes": recovered,
    }))))
}
```

Agregar a `fn handle_call()` (donde están las otras tools como `insert`, `search`, etc.).

### 3. Agregar binding Python

En `vantadb-python/src/lib.rs`, agregar método:

```rust
/// Recover nodes from the shadow archive by summary_id.
#[pyo3(signature = (summary_id))]
fn recover_archived_nodes(&self, summary_id: String) -> PyResult<Vec<PyObject>> {
    let sid: u128 = summary_id.parse()
        .map_err(|_| VantaError::InvalidInput(format!("Invalid summary_id: {summary_id}")))?;
    let nodes = self.inner.recover_archived_nodes(sid)?;
    Ok(nodes.into_iter().map(|n| {
        let dict = PyDict::new(self.py);
        // ... convert to py dict or use existing converters
    }).collect())
}
```

O usar el conversor existente si `VantaNodeRecord` → PyDict ya existe.

### 4. Tests

- Test `VantaEmbedded::recover_archived_nodes` (usar tempdir + engine)
- Test MCP tool `rehydrate` (llamar con summary_id válida)

## Verification

```bash
cargo check -p vantadb && cargo check -p vantadb-mcp && cargo check -p vantadb_py
cargo fmt --check
cargo clippy -p vantadb -- -D warnings
cargo nextest run --profile audit -p vantadb -- maintenance::test_recover_archived
cargo nextest run --profile audit -p vantadb-mcp
```

## Notas
- La infraestructura ya existe (StorageEngine::recover_archived_nodes + 6 tests)
- Este task conecta los puntos: Engine → SDK → MCP → Python
- Dependencia OLD-07 ya ✅ completada — no bloquea
