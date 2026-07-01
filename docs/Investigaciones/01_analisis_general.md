# Investigación 1 — Análisis General de VantaDB

> Estado técnico, posicionamiento, roadmap, calidad de documentación y estrategia de negocio.

## ¿Qué es VantaDB?

Un motor de persistencia cognitiva escrito en Rust (~32K LOC), posicionado como **"el SQLite para Agentes de IA"**: embebido, local-first, zero-config, con búsqueda híbrida (HNSW + BM25 + RRF) y garantías transaccionales (WAL + fsync + CRC32C).

## 1. Estado Técnico Real

### Fortalezas confirmadas (alta confianza)

**Core Engine (95%)** — La base técnica es sólida y bien diseñada:
- WAL con CRC32C en cada registro + fsync síncrono antes de ACK
- HNSW con Recall@10 = 0.998 en 100K vectores / 12.4ms p50 en Rust
- File locking exclusivo (fs2), GIL liberado consistentemente en PyO3
- Backend Fjall 100% Rust — eliminando dependencias de C++
- Modelo de datos `UnifiedNode` con 13 campos, incluyendo tier Hot/Cold, curva de olvido, importance/confidence

**Python SDK (90%)** — Completo con 20+ métodos, todos usando `py.allow_threads()`.

### Brechas críticas actuales

| Problema | Impacto | Estado |
|---|---|---|
| **Latencia Python SDK: 62ms vs objetivo <20ms** | 3x sobre el target | ⚠️ En progreso |
| **Telemetría de memoria reporta ~225 GB en máquina de 34 GB** | Métricas falsas | ⬜ Pendiente |
| **Windows CI roto** (runner inexistente) | Releases bloqueados | ⬜ Pendiente |
| **Testing: Fuzzing 10%, Integration 60%** | Confianza limitada | 🔄 En progreso |

## 2. Posicionamiento y Competitividad

**Ventajas únicas genuinas:**
- Búsqueda híbrida HNSW + BM25 + RRF nativa en el core
- GraphRAG integrado con traversal transaccional
- Durabilidad certificable con WAL + fsync + CRC32C
- Zero-config real: 100% Rust sin dependencias C++

## 3. Roadmap y Ruta Crítica

**Ruta crítica antes del lanzamiento público (Show HN septiembre 2026):**
1. Fix Python SDK latency (62ms → <20ms)
2. Fix Windows CI
3. Corregir telemetría de memoria
4. LangChain + LlamaIndex adapters
5. Publicar en crates.io

## 4. Síntesis

VantaDB tiene una base técnica sólida con una propuesta diferenciada. Los problemas son de ejecución, no de visión.
