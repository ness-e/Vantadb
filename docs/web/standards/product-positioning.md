# Posicionamiento de Producto — VantaDB Web

> **Propósito:** Define cómo se posiciona VantaDB en el mercado, contra quién compite, para quién es, y qué NO es.
> **Fuente:** Contenido extraído de `vanta-data.ts`, `dictionaries.ts`, y la identidad de marca en `brand-identity.md`. Validado contra docs/ de ingeniería.
> **Audiencia:** Equipo de producto, marketing, y cualquier agente que cree copy o landing pages.
>
> **⚠️ NOTA:** Pricing y monetización NO están definidos aún. La sección §7 refleja lo que está en `vanta-data.ts` pero no ha sido aprobado por estrategia GTM. Ver `WEB-18` en backlog.

---

## 1. Declaración de Posicionamiento

**VantaDB** es un motor de memoria embebido, local-first, escrito en Rust y expuesto vía PyO3. Resuelve un problema específico:

> Recuperación híbrida (BM25 + HNSW vía RRF) dentro del proceso, sin servidores, sin red, sin costo por query.

No es una base de datos cloud. Es **SQLite para memoria de IA** — el mismo golpe conceptual: una biblioteca que tu proceso linkea, no un servicio al que te conectás.

---

## 2. El Problema que Resuelve

El ecosistema actual de vectores tiene tres problemas estructurales que VantaDB ataca directamente:

| Problema | Síntoma | Costo |
|----------|---------|-------|
| **Latencia de red** | Cada search() viaja 50-200ms a un servidor remoto | Agentes lentos, pipelines que no escalan a edge |
| **Costo recurrente** | $300-8,000/mes por clúster cloud para vectores | Proyectos personales inviables, side projects mueren |
| **Fragilidad de sesión** | Agentes pierden memoria al reiniciar el proceso | Re-embedding constante, contexto perdido |

VantaDB reduce los tres a cero: 0ms de red, $0 de infraestructura, memoria durable con WAL.

---

## 3. Target Audience

Segmentos identificados en el contenido del sitio:

| Segmento | Señal en el código | Dolor principal | Lo que VantaDB les da |
|----------|-------------------|-----------------|----------------------|
| **AI Agent Developers** | Tutorial 01, Blog "SQLite for AI Agents", caso de estudio agent_local_memory_ollama | Agentes olvidan contexto entre sesiones | Memoria durable con put()/search() por namespace |
| **RAG Pipeline Builders** | Tutorial 02 (local RAG pipeline), caso de estudio rag_edge_device | Datos salen de la máquina, costos de API | RAG 100% local, zero data egress |
| **Edge / IoT Engineers** | Caso de estudio rag_edge_device (air-gapped) | Sin conectividad de red confiable | Ejecución in-process, binary size reducido (~3MB Rust core) |
| **Indie Developers** | Pricing page: Community $0 forever | $300/mes por Pinecone para un side project | Apache 2.0, pip install, $0 |
| **Privacy-Conscious Teams** | Security pillars: "No telemetry", "Zero network surface" | Datos sensibles no pueden salir de la máquina | Sin phone-home, sin tracking, air-gap nativo |

---

## 4. Competencia Directa e Indirecta

De `vanta-data.ts` → `WHY_VANTADB.comparison`:

| Feature | **VantaDB** | Pinecone | Weaviate | Chroma |
|---------|-------------|----------|----------|--------|
| Latencia (Rust core, 10K vectors) | **~1.2ms p50** | ~50-150ms | ~20-80ms | ~0.9ms |
| Latencia (Python SDK, 10K vectors) | **~40-62ms** | — | — | ~0.9ms |
| Network hops | **0** | 1+ | 1+ | 0-1 |
| Deploy | **pip install** | Cloud account | Docker cluster | pip install |
| Crash recovery | **WAL + CRC32C** | Managed | WAL | Limited |
| Hybrid search | **BM25 + HNSW · RRF (v1)** | Vector only* | BM25 + HNSW | Vector only |
| Data egress | **None** | Cloud | Self-host/cloud | None |
| Costo 1M vectores | **$0** | $1,800/mo | $600/mo | $0 |

> **✅ Hallazgo B1 cerrado (2026-08-04):** Las latencias de Weaviate (~20-80ms) pasan de "sin validar" a **respaldadas** por investigación documental — ver `docs/research/INV-018-weaviate-competitive-analysis.md` (34 refs citadas, abril 2026; datos ligados a releases, re-verificación pendiente contra docs actuales). Nota: Weaviate es cloud/self-hosted en Go y **queda fuera del harness de benchmarks local** (INV-007) — no hay medición propia; el rango proviene de `vanta-data.ts` y el contexto cualitativo de arquitectura de INV-018.

### Mapa de posicionamiento

```
                    Más features
                         ↑
               Weaviate ─┤
               Pinecone ─┤
                         │
    Cloud ───────────────┼────────────── Local-First
                         │
              Chroma ────┤
              VantaDB ───┤
                         ↓
                    Menos features
                     (más enfocado)
```

VantaDB compite en el cuadrante **local-first + enfocado** — no intenta ser una plataforma multimodelo. Donde gana:
1. **Latencia:** ~1.2ms p50 en Rust core (10K vectors). Gap con Python SDK ~40-62ms por FFI — documentado para evitar decepción.
2. **Simplicidad:** pip install, no Docker, no Kubernetes
3. **Costo:** $0 perpetuo vs suscripción mensual
4. **Durabilidad:** WAL con CRC32C, probado bajo caos

Donde NO compite:
- Multi-region / HA / replicación (deferred)
- SQL / OLTP / time-series (deferred)
- Escala masiva (10M+ vectores): requires HNSW persistence work-in-progress (ver ROADMAP.md R7)

> **Deep-dive competidor:** ver `docs/research/INV-019-pinecone-architecture-competitor.md` para el análisis arquitectónico completo de Pinecone (Slabs/LSM, FJLT/Ananas, IVF→PQFS, límites de 40KB metadatos y "impuesto RAM").
>
> **Competidor indirecto — pgvector (Postgres/cloud):** filtrado SQL nativo, sin lógica híbrida nativa (BM25+HNSW fuera del box), y requiere servidor PostgreSQL (no embebido). Relevante para RAG builders que ya tienen Postgres, pero cae fuera del cuadrante local-first/embedded de VantaDB. No listado como competidor directo en la tabla comparativa porque no cabe en el harness local del benchmark.

---

## 5. Product Boundary (qué es y qué NO es)

De `PRODUCT_BOUNDARY` en `vanta-data.ts`:

### ✅ Production-facing
- SDK embebido (Rust + Python)
- CLI completo: put, list, export, rebuild-index, audit-index, repair-text-index
- Memoria CRUD con namespaces
- WAL con CRC32C + crash recovery
- Búsqueda híbrida: BM25 + HNSW vía RRF
- Índices derivados rebuildables desde registros canónicos
- Export/import JSONL
- Server wrapper opcional (vanta-server)

### 🔄 Experimental / No MVP
- IQL/LISP/DQL
- MCP (Model Context Protocol)
- Integración directa LLM/Ollama
- Gobernanza y semánticas de mantenimiento
- Grafo traversal más allá de edges locales

### ❌ Deferred (explícitamente no construido)
- Cloud / enterprise platform
- HA / replicación / clustering distribuido
- SQL / OLTP / data warehouse / time-series
- Ranking avanzado
- RBAC / multi-tenancy
- Interfaz web / dashboard

**Esto es una ventaja, no una carencia.** El producto es honesto sobre lo que no es. Los deferred items generan confianza: "no cloud sync, no replication, no RBAC yet" en el hero.

---

## 6. Tone of Voice (Extraído del copy del sitio)

### Principios

1. **Preciso técnicamente**
   - Cada claim incluye evidencia numérica. No afirmaciones vagas.
   - "~1.2ms p50 por query (Rust core, 10K vectors), ≥0.98 Recall@10" en vez de "muy rápido y preciso"

2. **Anti-establishment**
   - "The fastest network hop is no network hop."
   - "No cloud tax on your own data."
   - Vocabulario: zero daemons, no Kubernetes, no cloud tax, pip install

3. **Honesto sobre límites**
   - Sección `Product Boundary` explicita qué NO está construido
   - "No cloud sync, no replication, no RBAC yet" — promesa cumplible

4. **Comunitario**
   - Apache 2.0 como feature, no nota legal
   - "Fork it, audit it, ship it."
   - "No strings attached"

5. **Manga / analógico**
   - Léxico de imprenta: "Forged in rust", "printed on cream stock", "ink", "paper"

### Taglines del sitio

| Contexto | Tagline |
|----------|---------|
| Hero title | "Embedded Rust engine for durable local memory and hybrid vector retrieval." |
| Hero subhead | "Persistent memory, crash-safe WAL recovery, and native hybrid search — without external services, containers, or network dependencies." |
| Footer | "forged in rust · printed on cream stock" |
| Meta / SEO | "VantaDB is a local-first, embedded Rust database engine for AI agents and local RAG." |

### Mensaje nuclear

> The fastest network hop is no network hop.

Este es el mensaje que distingue a VantaDB de todos los competidores. No es "mejor rendimiento" — es una arquitectura diferente.

---

## 7. Pricing y Monetización (NO DEFINIDO — REVISAR)

> **⚠️ IMPORTANTE:** El pricing y la estrategia de monetización **no están definidos aún**. Lo que sigue es lo que actualmente está en `vanta-data.ts` (código fuente del sitio), pero NO ha sido aprobado por estrategia GTM (`docs/strategy/GO_TO_MARKET.md`). Ver tarea `WEB-18` en backlog.

### Estado actual en `vanta-data.ts` (provisional, sujeto a cambio)

| Plan | Precio | Para quién |
|------|--------|------------|
| **Community** | $0 forever | Indie devs, side projects, open source |
| **Team** | $49/mes por seat ⚠️ NO VERIFICADO | Equipos shipping producción local-first |
| **Enterprise** | Custom | On-prem, air-gapped, SOC2-path |

### Lo que GO_TO_MARKET.md realmente documenta

| Plan | Precio | Notas |
|------|--------|-------|
| Community / Open Source | $0 | Apache 2.0, engine completo |
| Cloud Free | $0 | (plan cloud, no implementado) |
| Cloud Pro | $99/mes | (plan cloud, no implementado) |
| Cloud Business | $499/mes | (plan cloud, no implementado) |
| Enterprise On-Prem Starter | $10K/año | |
| Enterprise On-Prem Professional | $50K/año | |
| Enterprise | Custom | |

**Discrepancia:** `vanta-data.ts` tiene un plan "Team $49/mes por seat" que **no existe** en GO_TO_MARKET.md. Decidir si agregarlo a la estrategia o eliminarlo.

### Principio acordado (estable)
El motor completo es gratis y open source (Apache 2.0). Se cobra por soporte, SLA, tuning, y compliance — no por features del engine. No hay "pricing tiers" que capen features.

---

## 8. Mensajes Clave por Audiencia

| Audiencia | Ángulo | Frase |
|-----------|--------|-------|
| AI Agents | "Memoria durable sin servidor" | "Your agent remembers everything, costs nothing." |
| RAG local | "Zero data egress" | "Your documents never leave your machine." |
| Edge / IoT | "In-process, air-gapped" | "Runs where the cloud can't reach." |
| Indie devs | "$0 forever" | "Ship without a credit card on file." |
| Privacy | "No telemetry, no phone-home" | "What you store stays on your machine." |

---

## 9. Anti-Posicionamiento (lo que NO somos)

Tan importante como lo que VantaDB es, es lo que NO es:

- ❌ **No** es una base de datos cloud (Pinecone, Weaviate, Qdrant)
- ❌ **No** es una plataforma multimodelo
- ❌ **No** es un servicio gestionado (no hay cluster, no hay replicación)
- ❌ **No** tiene dashboard web, UI de administración, ni playground online
- ❌ **No** requiere Docker, Kubernetes, ni ninguna infraestructura
- ❌ **No** tiene telemetría, analytics, ni phone-home

El copy del sitio debe reforzar estas diferencias. Cada vez que un visitante viene de Pinecone o Weaviate, hay que mostrarle por qué esta alternativa es diferente — no "mejor", sino *fundamentalmente distinta* en arquitectura.

---

## 10. Reglas para Nuevo Copy

1. **Siempre liderar con "local-first" o "embedded"** — es el diferenciador principal.
2. **Mencionar latencia numérica** en el primer párrafo (~1.2ms p50 Rust core, 10K vectors). Aclarar gap Rust/Python (~40-62ms Python SDK). Nunca usar "per dimension".
3. **No prometer lo deferred.** El `PRODUCT_BOUNDARY` es ley. Si no está en production-facing, no se menciona como disponible.
4. **Incluir comparación de costo** cuando se hable de alternativas cloud ($0 vs $1,800/mo para 1M vectores).
5. **El footer siempre termina** con "forged in rust · printed on cream stock".
6. **Apache 2.0 se menciona temprano** — es un feature de producto, no una nota legal.
7. **No usar "base de datos" genérico** — preferir "motor de memoria", "embedded engine", "memory store".
