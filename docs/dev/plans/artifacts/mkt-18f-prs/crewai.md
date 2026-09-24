# Upstream PR draft — CrewAI

**Prereq (bloquea merge upstream):** publicar `vantadb-crewai` en PyPI (tag `adapters-v0.5.0`).

**Target repo:** `crewAIInc/crewAI`
**Branch sugerida:** `patch-add-vantadb-memory-backend-docs`

## Cambio propuesto

- Docs de "knowledge / memory sources": listar VantaDB como backend de vector store soportado por el paquete externo `vantadb-crewai` (nuestro `VantaDBTool` se registra como storage backend en `CrewAI(knowledge=...)`).

## PR title

`docs: list VantaDB as a supported knowledge/memory storage backend`

## PR body

```markdown
Adds [VantaDB](https://github.com/ness-e/Vantadb) to the supported storage backends for CrewAI knowledge/memory.

- Out-of-tree integration package: `vantadb-crewai`
- Embedded persistent Rust vector memory; hybrid BM25+HNSW; no server
- Source: https://github.com/ness-e/Vantadb/tree/main/integrations/crewai
- License: Apache-2.0
```
