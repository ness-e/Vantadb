---
title: "ADR-COMP-019: Binary protocol (rkyv/FlatBuffers over gRPC) — WONTFIX"
type: adr
status: accepted
tags: [vantadb, architecture, network, serialization, wontfix]
created: 2026-08-02
last_reviewed: 2026-08-02
---

# ADR-COMP-019: Binary protocol (rkyv/FlatBuffers over gRPC) — WONTFIX

## Context

COMP-019 proponía un protocolo binario (rkyv/FlatBuffers over gRPC) como
reemplazo del HTTP JSON para la API. El backlog lo marcaba ⚠️ Parcial porque
rkyv (serialización zero-copy) ya se usa internamente en storage/WAL.

Dos auditorías independientes recomendaron WONTFIX:

- `docs/audit-reports/meta-001-root-cause-analysis.md` (§4): "VantaDB es
  embedded-first. gRPC es contraproducente para el caso de uso local."
- `docs/audit-reports/backlog-validation-2026-07-28.md` (§13): "WONTFIX
  recomendado — proyecto es embedded-first."

## Decision

Cerrar COMP-019 como **WONTFIX**. No se implementa gRPC ni un wire protocol
binario externo.

Racional:

1. **gRPC contradice el posicionamiento embedded-first.** gRPC arrastra
   HTTP/2, protobuf, mTLS y un framework servidor-cliente pesado — infra
   enterprise (Pinecone/Weaviate hosteados), no una librería embebida
   (Rust SDK, WASM, Python) que promete "it just works" con zero config.
2. **El valor técnico ya está capturado.** rkyv (serialización binaria
   zero-copy) ya se usa internamente en storage/WAL. Ese era el 80% del
   valor de COMP-019; lo restante (transport binario hacia afuera) es la
   parte cara y sin demanda.
3. **Sin demanda ni dependencias.** Ninguna tarea del backlog depende de
   COMP-019, no hay issue de usuario pidiendo protocolo binario. Es YAGNI:
   construir un wire protocol especulativo que nadie consumiría.

## Consequences

- **Pros:**
  - Evita ~2 semanas de esfuerzo en infraestructura de red no solicitada.
  - Mantiene el graph de dependencias liviano (sin ton/tokio-http2/protobuf).
  - Alinea el backlog con la visión de producto (Embedded AI Memory).
  - Documenta la decisión para que el debate no se repita (META-001 §B).
- **Cons:**
  - JSON sobre HTTP sigue siendo más lento que un protocolo binario para
    cargas masivas (ej: exportar 1M vectores).
  - Si algún día se requiere servir VantaDB como servidor remoto a otros
    procesos, el wire protocol binario se tendrá que evaluar de nuevo.

## Criterio de re-apertura

Re-evaluar si aparece demanda real: un caso de uso que sirva VantaDB como
servidor remoto con transferencia masiva de vectores, o un issue de usuario
que lo requiera. La base rkyv ya deja la serialización lista — solo faltaría
el transport.
