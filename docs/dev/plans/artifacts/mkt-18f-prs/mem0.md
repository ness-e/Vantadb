# Upstream PR draft — Mem0

**Prereq (bloquea merge upstream):** publicar `vantadb-mem0` en PyPI (tag `adapters-v0.5.0`).

**Target repo:** `mem0ai/mem0`
**Branch sugerida:** `patch-vantadb-vector-store-provider`

## Cambio propuesto

Mem0 soporta providers de vector store vía config (`vector_store.provider: <name>`). Dos variantes:

- **A (docs only, barata):** documentar el provider externo `vantadb-mem0` en `docs/vector-stores/` de mem0 — nuestro paquete implementa su `VectorStoreBase`, se registra vía config custom.
- **B (código, cara):** integrar como provider nativo en `mem0/vector_stores/` — requiere review de su maintainer; no lo pedimos de entrada.

Elegir A; B solo si aceptan.

## PR title

`docs(vector-stores): add VantaDB (embedded Rust) via vantadb-mem0 integration package`

## PR body

```markdown
Adds documentation for using [VantaDB](https://github.com/ness-e/Vantadb) as a mem0 vector store through the out-of-tree provider package `vantadb-mem0`.

- Implements mem0's `VectorStoreBase` — registered via mem0 config, no core changes needed
- Embedded, local-first Rust storage (Fjall), hybrid BM25+HNSW
- Source: https://github.com/ness-e/Vantadb/tree/main/integrations/mem0
- License: Apache-2.0
```
