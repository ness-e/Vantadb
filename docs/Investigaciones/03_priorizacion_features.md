# Investigación 3 — Priorización de Features

> Marco de decisión para features pre-lanzamiento.

## Veredictos por Feature

| Feature | Decisión | Fase |
|---|---|---|
| TypeScript SDK vía WASM | ✅ SÍ — Alta prioridad | Fase 4 |
| Cuantización SQ8 | ✅ SÍ — Solo SQ8 escalar | Fase 3 |
| Backup/Restore nativo | ✅ SÍ — Simple, local | Fase 4 |
| SQL completo | 🔴 NO — Fuera del roadmap | Post-seed |
| IVF-PQ disk-based | 🔴 NO — No es tu mercado | Post-seed |
| Raft distributed | 🔴 NO — Post-seed | Post-seed |
| Embedding models bundled | 🔴 NO — Nunca en el core | — |
| GraphQL API | 🔴 NO — Ya tienes MCP | — |

## Regla de Oro

> ¿Puede un developer hacer `pip install vantadb-py` y tener un agente con memoria persistente, búsqueda híbrida y GraphRAG en <10 líneas, <20ms, en <2 minutos?

Todo lo que no contribuya a esa demo no entra al roadmap antes de septiembre 2026.
