---
title: Decisiones No Tomadas — Registro de descartes
kind: adr
status: active
description: "Registro consolidado de features/decisiones evaluadas y rechazadas o diferidas, con razón y fuente — evita re-litigar descartes sin evidencia nueva."
tags: [vantadb, adr, descartes, decisiones, gobernanza]
---

# Decisiones No Tomadas — Registro de descartes

> **Propósito:** consolidar los descartes explícitos del proyecto (features evaluadas y rechazadas/diferidas, con razón y fuente) para que ninguna se re-litigue sin **evidencia nueva** (benchmark, demanda de usuarios, cambio de contexto). Fuentes: `docs/dev/archive/EXTRACCION-DOC-OLD-2026-08-05.md` §1 + "No Hacer"/Icebox del backlog v0.1 (`OLD\VANTADB  OLD\docs_backup_2026-06-30\Backlog.md`, rescatado 2026-09-30) + decisiones puntuales. **No es un ADR numerado** — los ADRs deciden lo que SÍ se hace; este registro documenta lo que NO.
>
> **Regla de reapertura:** evidencia nueva → actualizar la fila → recién entonces proponer tarea. Prohibido re-litigar por intuición.

## 1. "No Hacer" por diseño (v0.1, hasta post-seed con equipo)

| Feature | Razón | Fuente |
|---------|-------|--------|
| SQL completo | 3-6 meses; el ICP no lo necesita; pgvector ya lo tiene | Backlog v0.1 §No Hacer |
| Distributed / Raft | 6-12 meses; contradice filosofía embedded | ídem |
| IVF-PQ disk-based | LanceDB mejor; no es mercado VantaDB | ídem |
| GPU acceleration | Rompe zero-config; no resuelve el bottleneck real | ídem |
| RBAC / SSO en core | Solo cloud managed, post-seed (nota 2026-09: RBAC básico por namespace YA existe; SSO sigue fuera) | ídem |
| Embedding models bundled | Destruye zero-config (500MB+ wheel) | ídem |
| GraphQL API | El ICP prefiere API/MCP | ídem |
| Versionado git-style | No es dolor del ICP; LanceDB ya lo tiene | ídem |
| Time-series mode | Producto diferente, fuera de scope | ídem |
| Cuantización 1.5/2-bit | Retorno marginal para datasets <1M | ídem |

## 2. Descartes de deep-analysis (2026-07, competitive-features)

| ID | Feature | Competidor/Origen | Justificación |
|----|---------|-------------------|---------------|
| QDR-001 | Gridstore (KV custom sin LSM/WAL) | Qdrant | Reemplazar el storage engine es proyecto de meses; migrar un LSM a segmentos es costo alto para ganancia marginal; sin WAL, power-loss se maneja más manual |
| ARC-015 | Arrow IPC como formato de WAL (Fase 2) | — | Utilidad baja, prematuro; postcard battle-tested + CRC32C/scan_forward_valid robusto |
| ARC-005 | IRI (Implicit Relational Inference) | — | VantaDB es schemaless; IRI requiere schema → joins incorrectos posibles; GraphRAG es más útil como prerequisito |
| WEV-011 | HFresh (índice en disco por freshness) | Weaviate | Alta complejidad; nicho; paper con poca adopción; latencia variable; depende de segment architecture primero |

### Otras diferidas por nicho / bajo retorno (agrupadas)

| Feature | Competidor | Justificación |
|---------|------------|---------------|
| GRF-011 — optimizer Cypher completo | Neo4j | Queries simples (1-2 saltos + vector); el overhead no se amortiza |
| GRF-009 — WASM plugins | SurrealDB | Overhead FFI > beneficio; depurar Wasm difícil |
| GRF-041 — Pipelined commit | SurrealDB | Escrituras batch poco frecuentes; complejidad en recovery |
| GRF-025 — VelocyPack serialization | ArangoDB | Modelo plano, no anidado; rkyv mejor inversión |
| GRF-027 — SmartGraphs (colocation) | ArangoDB | Solo multi-nodo; no hay producto distribuido (nota: co-location sí se evaluó en la ampliación de FUT-03) |
| WEV-004 — Rotational Quantization (RQ) | Weaviate | Sobre PQ; beneficio marginal, añade O(d²) |
| PGV-006 — Statistical BQ (SBQ) | pgvector | Mejora incremental; menos útil con vectores normalizados |
| GRF-016 — MPP vertex-centric | TigerGraph | Solo analytics, no RAG transaccional |
| GRF-014 — GSQL/JIT | TigerGraph | Beneficio solo hot paths pre-registrados; pausa 1-10ms |
| PGV-002 — Iterative Index Scans | pgvector | Obsoleto si in-filter implementado; fallback only |
| PIN-003 — Single-stage filtering | Pinecone | Redundante con in-filter; bitmaps no escalan alta cardinalidad |

## 3. Retiradas / removidas con supersession (registro)

| Sistema / Feature | Estado | Registro |
|---|---|---|
| PITR (Point-in-Time Recovery) | Removido como dead code (2026-08-25); rediseño en PRO-03 | FIND-26, ADR-0014 superseded |
| NeuLISP / arquitectura biológica (ConnectomeDB→VantaDB) | Purga total de terminología biológica; sobrevive IQL | CHANGELOG Legacy, AGENT_INSTRUCTIONS |
| Multi-connection desktop (6 vías) | Reemplazado por transporte pluggable | ADMIN-03, backlog-history |
| gRPC / protocolo distribuido | WONTFIX — embedded-first | ADR-0048 |
| DR multi-región / microservicios | No-objetivo explícito | PRD 90d §9; PRO-02/03 (trigger Pro) |

## 4. Notas de métricas de descartes (deep-analysis-arch)

- **ARC-002 (Bitset inline u128):** ahorro "24-56 B/nodo" + elimina indirección de heap.
- **ARC-004 (Edge Label Interning):** "80MB → 12MB para 1M nodos con 4 edges" (~20 B/edge); match label O(n)→O(1).
- **ARC-003 (Cuantización I8):** compresión 4x — "1544B vs 6144B para 1536d".

## 5. Riesgos "No Hacer" (pre-launch v0.1) — resueltos o migrados

| Riesgo | Estado 2026-09-30 |
|--------|-------------------|
| License audit pendiente | ✅ `cargo deny check licenses` limpio (Apache-2.0 compatible) |
| Trademark "VantaDB" no registrado | → `LEG-01` (Backlog-negocio) |
| CI/CD para forks externos | Workflow approval for first-time contributors + hardening HARD-02 |

---
> **Mantenimiento:** al descartar algo nuevo (o reabrir un descarte), agregar/actualizar la fila con fuente y fecha. Creado 2026-09-30 (auditoría de backlogs; consolida descartes que vivían dispersos).
