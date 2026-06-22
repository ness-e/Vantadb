---
type: glosario-entry
status: stable
tags: [concepto, arquitectura, embedded, database]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Embedded Database, Base de Datos Embebida, In-Process]
description: "Sistema de gestión de datos que opera in-process dentro de la misma aplicación que la consume, sin requerir un servidor separado, demonio de red ni proceso independiente"
---

# Embebido (Embedded Database)

## Definición

Una **base de datos embebida** es un sistema de gestión de datos que opera **in-process** dentro de la misma aplicación que la consume, sin requerir un servidor separado, demonio de red ni proceso independiente. La base de datos se compila como una librería que se enlaza directamente al binario de la aplicación.

## Características Fundamentales

| Característica | Descripción |
|---------------|-------------|
| **In-Process** | Corre en el mismo espacio de memoria que la aplicación |
| **Zero-Network** | No usa TCP/IP, HTTP, ni protocolos de red |
| **Zero-Config** | No requiere instalación, configuración ni administración |
| **Single Binary** | Se distribuye como parte del binario de la aplicación |
| **Low Latency** | Llamadas directas a función, sin serialización de red |

## Ejemplos Históricos

| Sistema | Lenguaje | Caso de Uso |
|---------|----------|-------------|
| **SQLite** | C | Almacenamiento relacional embebido |
| **LevelDB** | C++ | Key-value store de Google |
| **RocksDB** | C++ | Storage engine de Facebook |
| **DuckDB** | C++ | OLAP embebido |
| **VantaDB** | Rust | Memoria persistente para agentes de IA |

## Por Qué Importa en VantaDB

VantaDB se define como **"El SQLite para Agentes de IA"** porque adopta la filosofía embebida:

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

### Ventajas de VantaDB como Embebido

1. **Latencia sub-milisegundo**: Sin overhead de red (roundtrip TCP + serialización JSON)
2. **Zero-ops**: Sin servidores que administrar, sin clusters que mantener
3. **Portabilidad**: Corre en cualquier plataforma donde Rust compile (Linux, macOS, Windows, ARM)
4. **Determinismo**: Comportamiento predecible sin variables de red
5. **Seguridad**: Sin superficie de ataque de red expuesta

### Trade-offs Aceptados

| Ventaja | Costo |
|---------|-------|
| Simplicidad | Sin replicación nativa (aún) |
| Low latency | Single-writer por defecto |
| Zero-ops | Escalabilidad vertical, no horizontal |

## Comparación: Embebido vs Cliente-Servidor

| Dimensión | Embebido (VantaDB, SQLite) | Cliente-Servidor (PostgreSQL, Milvus) |
|-----------|---------------------------|--------------------------------------|
| **Latencia** | μs (llamada a función) | ms (roundtrip de red) |
| **Instalación** | `pip install vantadb-py` | Docker, cluster, configuración |
| **Concurrencia** | Single-writer, multi-reader | Multi-writer con MVCC distribuido |
| **Escalabilidad** | Vertical (más RAM/CPU) | Horizontal (más nodos) |
| **Administración** | Zero | DBA requerido |
| **Caso de uso ideal** | Agentes locales, edge, desktop | Aplicaciones web multi-tenant |

## Anti-Patrón: "Embedded con Modo Servidor"

VantaDB mantiene un **wrapper de servidor opcional** (`vantadb-server` con Axum), pero esto es explícitamente **secundario**:

- ✅ **Core embebido**: Prioridad absoluta, identidad del producto
- ⚠️ **Servidor HTTP**: Wrapper opcional, no el diseño central
- ❌ **No competir** con Milvus/Qdrant en distributed vector DBs

## Véase También

- [Local-First](Local-First.md) — Filosofía complementaria
- [Zero-Config](Zero-Config.md) — Consecuencia natural del diseño embebido
- [Fjall](Fjall.md) — Backend embebido 100% Rust
- [Transaccional](Transaccional.md) — Garantías ACID in-process

---

*La identidad de VantaDB como base de datos embebida es su diferenciador estratégico fundamental.*

