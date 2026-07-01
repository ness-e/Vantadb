---
type: glossary-entry
status: stable
tags: [concept, architecture, embedded, database]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Embedded Database, Embedded Database, In-Process]
description: "Data management system that operates in-process within the same application that consumes it, without requiring a separate server, network daemon or independent process"
---
# Embedded (Embedded Database)

##Definition

An embedded database is a data management system that operates **in-process** within the same application that consumes it, without requiring a separate server, network daemon, or independent process. The database is compiled as a library that links directly to the application binary.

## Key Characteristics

| Característica | Descripción |
|---------------|-------------|
| **In-Process** | Corre en el mismo espacio de memoria que la aplicación |
| **Zero-Network** | No usa TCP/IP, HTTP, ni protocolos de red |
| **Zero-Config** | No requiere instalación, configuración ni administración |
| **Single Binary** | Se distribuye como parte del binario de la aplicación |
| **Low Latency** | Llamadas directas a función, sin serialización de red |

## Historical Examples

| Sistema | Lenguaje | Caso de Uso |
|---------|----------|-------------|
| **SQLite** | C | Almacenamiento relacional embebido |
| **LevelDB** | C++ | Key-value store de Google |
| **RocksDB** | C++ | Storage engine de Facebook |
| **DuckDB** | C++ | OLAP embebido |
| **VantaDB** | Rust | Memoria persistente para agentes de IA |

##Why it Matters in VantaDB

VantaDB is defined as **"The SQLite for AI Agents"** because it adopts the embedded philosophy:

```
┌─────────────────────────────────────────┐
│         Aplicación (Python/Rust)         │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │     VantaDB (linked library)       │ │
│  │  ┌──────┐  ┌──────┐  ┌──────────┐ │ │
│  │  │ WAL  │  │ HNSW │  │ Storage  │ │ │
│  │  └──────┘  └──────┘  └──────────┘ │ │
│  └────────────────────────────────────┘ │
│                                          │
│              [Disco Local]               │
└─────────────────────────────────────────┘
```

### Advantages of VantaDB as Embedded

1. **Latencia sub-milisegundo**: Sin overhead de red (roundtrip TCP + serialización JSON)
2. **Zero-ops**: Sin servidores que administrar, sin clusters que mantener
3. **Portabilidad**: Corre en cualquier plataforma donde Rust compile (Linux, macOS, Windows, ARM)
4. **Determinismo**: Comportamiento predecible sin variables de red
5. **Seguridad**: Sin superficie de ataque de red expuesta

### Accepted Trade-offs

| Ventaja | Costo |
|---------|-------|
| Simplicidad | Sin replicación nativa (aún) |
| Low latency | Single-writer por defecto |
| Zero-ops | Escalabilidad vertical, no horizontal |

## Comparison: Embedded vs Client-Server

| Dimensión | Embebido (VantaDB, SQLite) | Cliente-Servidor (PostgreSQL, Milvus) |
|-----------|---------------------------|--------------------------------------|
| **Latencia** | μs (llamada a función) | ms (roundtrip de red) |
| **Instalación** | `pip install vantadb-py` | Docker, cluster, configuración |
| **Concurrencia** | Single-writer, multi-reader | Multi-writer con MVCC distribuido |
| **Escalabilidad** | Vertical (más RAM/CPU) | Horizontal (más nodos) |
| **Administración** | Zero | DBA requerido |
| **Caso de uso ideal** | Agentes locales, edge, desktop | Aplicaciones web multi-tenant |

## Anti-Pattern: "Embedded with Server Mode"

VantaDB maintains an **optional server wrapper** (`vantadb-server` with Axum), but this is explicitly **secondary**:

- ✅ **Embedded Core**: Absolute priority, product identity
- ⚠️ **HTTP Server**: Optional Wrapper, not the core layout
- ❌ **Do not compete** with Milvus/Qdrant in distributed vector DBs

## See Also

- [[local-first]] — Complementary Philosophy
- [[zero-config]] — Natural consequence of embedded design
- [[fjall]] — 100% Rust embedded backend
- [[transactional]] — ACID in-process guarantees

---

*VantaDB's identity as an embedded database is its fundamental strategic differentiator.*

