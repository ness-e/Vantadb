# VantaDB — Análisis Completo: Qué es, Qué será, Proyección y Backlog

> **Propósito:** Evaluación holística del proyecto VantaDB — estado actual,
> trayectoria, proyección de mercado, y revisión crítica del backlog con
> recomendaciones de qué agregar, mantener y eliminar.
>
> **Metodología:** Análisis del codebase completo (79 archivos Rust + 13 crates
> de integración), documentos estratégicos (Backlog, Roadmap, Action Plan, GTM),
> y benchmark contra 20+ competidores verificados en internet.
>
> **Fecha:** Julio 2026

---

## Parte 1: ¿Qué es VantaDB Hoy?

### Definición Técnica

VantaDB es una **base de datos vectorial embebida** escrita en Rust, enfocada en
memoria a largo plazo para agentes de IA. Su core es un índice HNSW con:

- **3 esquemas de cuantización** (RaBitQ 1-bit, TurboQuant 3-bit, SQ8 8-bit)
- **ShardedWAL** con detección de gaps por shard (único en el mercado)
- **SIMD distance kernels** (AVX-512 + portable f32x8)
- **QuantizationGovernor** automático (f32↔SQ8 por access frequency)
- **FilterBitset** multi-tenant
- **MemoryGovernor** con eviction por watermark
- **3 backends** (Fjall default, RocksDB opt, InMemory)

### Madurez Real

| Aspecto | Nivel | Evidencia |
|---------|-------|-----------|
| Core engine | **Producción-ready** | 46 test files, chaos testing, certification framework |
| SDKs | **Best-in-class** | 6 plataformas (Python, TS, Rust, WASM, MCP, REST) — nadie más tiene esta cobertura |
| Documentación | **Excelente para 1 dev** | ~109 archivos, glosario de 63 términos, ADRs |
| Integraciones | **Scaffold completo** | 13 crates (9 stale en v0.1.5, pero existen) |
| CLI | **Superior** | 33 comandos, completions para 4 shells |
| WASM | **Funcional pero básico** | OPFS persistence, sin IndexedDB fallback, sin Worker, sin multi-tab |
| Enterprise | **Stub** | Solo RBAC implementado |

### Su Posición en el Mercado 2026

```
Embedded Vector DBs Landscape 2026:

  Feature-Rich
      │
      │        Qdrant (server)
      │        Weaviate (server)
      │        Milvus (server)
      │
      │        LanceDB (embeddable)  ●─── VantaDB en target
      │        ChromaDB (embeddable)  ●
      │        Quiver (embeddable)    ●
      │
      │        VantaDB (hoy)  ●
      │
      └─────────────────────────────── Comunidad
         1 dev              20K+ GH
```

VantaDB compite en **embedded vector DBs** (ChromaDB, LanceDB, Quiver).
No compite directamente con Qdrant/Weaviate/Milvus (server-mode con redes).

Hoy está en **desventaja técnica**: 1 tipo de índice vs 8 de Quiver, sin PQ,
sin sparse vectors, sin filtered search avanzado. Pero tiene **ventaja
estratégica**: 6 SDKs, MCP server único, CLI superior, WAL sharding único.

---

## Parte 2: ¿Qué Será VantaDB?

### La Visión Actual

Según el roadmap y GTM:

```
v0.2.0 (Jul-Ago 2026)  →  Lanzamiento público (Show HN)
v1.0 (Q4 2026)         →  Enterprise readiness + Cloud beta
v2.0 (2027)            →  Multi-index, PQ, segmentación, distribuido
```

Es una **visión sensata pero conservadora**. El proyecto no tiene
pretensiones de competir con Qdrant/Milvus en el corto plazo. Se posiciona
como la opción embedded **multi-SDK** para agentes de IA.

### Lo Que REALMENTE Debería Ser

Basado en el análisis de mercado y sus fortalezas reales:

1. **La base de datos vectorial para agentes de IA** — no "una DB más".
   Su ventaja real es el ecosistema SDK + MCP, no el engine.

2. **El MongoDB de las vector DBs embedded** — fácil de empezar, SDK en
   cualquier lenguaje, buen DX. Eso es alcanzable y defendible.

3. **Un puente WASM+OPFS** para AI en el browser — nadie lo hace bien hoy.
   TalaDB se acerca pero VantaDB tiene más recursos.

4. **No debería intentar ser Qdrant** — no inviertas en gRPC, clustering,
   o distributed mode. El embedded-first es el diferenciador.

### Vector Correcto vs Incorrecto

| ✅ Correcto (hacer) | ❌ Incorrecto (no hacer) |
|--------------------|--------------------------|
| Multi-SDK parity | gRPC streaming |
| MCP server | Distributed Raft |
| WASM+OPFS multi-tab | Server-mode clustering |
| PQ compresión | Cloud hosting propio |
| Flat index threshold | Stripe billing |
| CI regresión/sanitizers | Enterprise sales |
| Tutoriales + learning path | SOC2/HIPAA compliance |
| Community building | Multi-tenant isolation |

---

## Parte 3: Proyección

### Análisis de Mercado

**Tamaño de mercado:** El mercado de vector databases creció de ~$1.5B (2024)
a ~$8B estimado (2027). El nicho embedded está menos competido que server-mode.

**TAM que VantaDB puede capturar:**

| Segmento | Tamaño | VantaDB fit |
|----------|--------|-------------|
| AI agents (memory layer) | $500M+ | ✅ Perfecto — WAL + TTL + cuantización |
| RAG pipelines embedded | $200M+ | ⚠️ Sin sparse vectors, sin PQ |
| Browser AI (WASM) | $100M+ | ✅ Ventaja early mover |
| Edge / IoT | $150M+ | ⚠️ Sin ARM64 wheels |
| Prototyping / DevEx | $200M+ | ✅ Multi-SDK + CLI superior |

**Target más realista:** AI agent memory. Es donde VantaDB tiene ventajas
genuinas (WAL sharding, multi-SDK, MCP, quantización). Los agentes necesitan
persistencia local, TTL, memoria a largo plazo — VantaDB lo tiene.

### Proyección a 12 Meses (Jul 2026 — Jul 2027)

**Escenario optimista** (Show HN exitoso + community traction):

| Métrica | Hoy | Jul 2027 | Condición |
|---------|-----|----------|-----------|
| GitHub Stars | ~0 (no publicado) | 2,000-5,000 | Show HN → viral en r/rust |
| PyPI descargas/mes | ~100 | 10,000-50,000 | LangChain + LlamaIndex integrados |
| Contribuidores | 1 | 20-50 | CONTRIBUTING en raíz + Discord |
| SDKs publicados | 2/6 | 6/6 | WASM npm + Python/Typescript stables |
| Index types | 1 (HNSW) | 3 (+Flat, +PQ) | PQ + Flat threshold |
| Revenue | $0 | $0-5K MRR | Cloud pilot o consulting |

**Escenario realista:**

| Métrica | Jul 2027 |
|---------|----------|
| GitHub Stars | 500-1,500 |
| PyPI descargas/mes | 2,000-10,000 |
| Contribuidores | 5-15 |
| SDKs publicados | 4/6 |
| Revenue | $0 |

**Escenario pesimista** (no hay community traction):

El proyecto sigue siendo de 1 dev. Las integraciones se vuelven stale.
Eventualmente pierde relevancia frente a LanceDB y ChromaDB que avanzan
rápido. Sin comunidad, el mantenimiento es insostenible.

### Riesgos Reales (no cubiertos en backlog)

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| **Burnout del único dev** | 🟡 Media | 🔴 Catastrófico | Community building NOW |
| **ChromaDB/LanceDB agregan WAL sharding** | 🟡 Media | 🟡 Medio | No es copiable fácil |
| **Quiver se vuelve dominante** | 🟢 Baja | 🟡 Medio | Quiver no tiene MCP ni CLI |
| **AI agents cambian de stack** | 🟡 Media | 🟡 Alto | Diversificar casos de uso |
| **No hay tracción en Show HN** | 🔴 Alta | 🟡 Alto | Plan B: product hunt, blog técnico |
| **RUSTSECs acumuladas** | 🟡 Media | 🔴 Release bloqueado | CI de seguridad continuo |

### Señales que Definirán el Futuro (a monitorear)

1. **Show HN reactions** — Si no hay interés, repensar target audience
2. **Primeros 5 contribuidores externos** — Indican si el proyecto es atractivo
3. **LangChain/LlamaIndex PR mergeados** — Validación del ecosistema
4. **WASM demo usage** — Cuántos pruebas la demo en browser
5. **Issues/PR quality** — Bugs reales vs feature requests vs spam

---

## Parte 4: Revisión del Backlog Actual

### Metodología de Evaluación

Cada item del backlog se evalúa contra 4 criterios:

1. **Impacto** — ¿Cuánto mejora el producto o la adopción?
2. **Esfuerzo** — ¿Cuánto tiempo requiere?
3. **Timing** — ¿Es ahora o puede esperar?
4. **Alineación** — ¿Contribuye a la visión "DB para agentes AI"?

Los items se clasifican en: ✅ Mantener | 👀 Mover de prioridad | ❌ Eliminar

### TIER 0 — Bloqueantes de Release

| ID | Tarea | Veredicto | Razón |
|----|-------|-----------|-------|
| INT-01/02 | LangChain + LlamaIndex → PyPI | ✅ **Mantener 🔴** | Bloquea adopción real |
| INT-03→11 | Adaptadores → PyPI | 👀 **Mover a 🟡** | No todos son críticos. Priorizar LangChain/LlamaIndex |
| INT-11 | Semantic Kernel adapter | ❌ **Eliminar** | Microsoft SK tiene poco uso. No vale el esfuerzo ahora |
| DEVOPS-05 | Pipeline CI unificado adapters | ✅ **Mantener 🔴** | Sin pipeline, publicar es manual |
| DEVOPS-12 | PyPI signing pipeline | 👀 **Mover a 🟡** | No bloquea release. Sigstore puede esperar |
| REL-02 | Publicar vantadb-ts en npm | ✅ **Mantener 🔴** | WASM build es showcase importante |

**Adición sugerida a TIER 0:**

| ID Propuesto | Tarea | Esfuerzo | Razón |
|-------------|-------|----------|-------|
| `REL-03` | **Version sync: 9 crates a workspace version** | 🟢 1h | Sin esto, cada release manual es error-prone |

### TIER 1 — Pre-Lanzamiento

| ID | Tarea | Veredicto | Razón |
|----|-------|-----------|-------|
| "—" | Eliminar `OldSerializationError` deprecated | ✅ **Mantener 🟡** | Code hygiene |
| MKT-13 | Demo WASM en hero | ✅ **Subir a 🔴** | Es el mejor showcase del producto |
| MKT-14 | Case studies en landing | ✅ **Mantener 🟡** | 2 studies alcanza |
| DOC-20 | Migration guide LanceDB | ✅ **Mantener 🟡** | Bueno para capturar usuarios |
| MCP-IDE | Docs setup MCP por IDE | ✅ **Subir a 🔴** | MCP es diferenciador único |
| DEVOPS-02 | ARM64 wheels | 👀 **Mover a 🟡** | No bloquea launch, sí bloquea edge/RPi |
| DEVOPS-10 | Windows signing | 👀 **Mover a 🟢** | SmartScreen no es blocker para OSS |
| PERF-24 | GIL scope optimization | ❌ **Eliminar** | Impacto marginal post-DX-02 |
| PERF-25 | Object pool PyDict | ❌ **Eliminar** | PERF-16 (#[pyclass]) ya resuelve el problema |
| PERF-26 | Lazy serialization | ✅ **Mantener 🟡** | Bueno para hot paths |
| PERF-29 | Cosine→Euclidean mapping | ❌ **Eliminar** | Optimización prematura |

### TIER 2 — Launch Campaign

| ID | Tarea | Veredicto | Razón |
|----|-------|-----------|-------|
| LEG-01 | Trademark | ✅ **Mantener 🔴** | Iniciar YA (proceso lento) |
| MKT-03 | Show HN post | ✅ **Mantener 🔴** | Core del launch |
| MKT-04 | Reddit posts | ✅ **Mantener 🟠** | Segundo canal más importante |
| MKT-15 | Página benchmarks | ✅ **Mantener 🔴** | Sin benchmarks no hay credibilidad |
| MKT-16 | Metodología GraphRAG | 👀 **Mover a 🟡** | Bueno tener, no bloquea |
| MKT-17 | Página comparación interactiva | 👀 **Mover a 🟢** | Costoso, impacto medio |
| COM-01 | Discord server | ✅ **Subir a 🔴** | Crítico para comunidad |
| TSK-106 | GitHub Discussions | ✅ **Mantener 🟡** | Complementa Discord |
| TSK-107 | Community showcase | ✅ **Mantener 🟡** | Bueno para retention |
| TS SDK hardening | Type safety + tests | ✅ **Subir a 🔴** | SDK de segunda clase da mala imagen |
| CODE-074 | Visual regression tests | ❌ **Eliminar** | No hay recursos para mantener Percy/Chromatic |

### TIER 3 — Post-Lanzamiento

| ID | Tarea | Veredicto | Razón |
|----|-------|-----------|-------|
| Publicar 8 workspace crates.io | ✅ **Mantener 🟡** | Consistencia del ecosistema |
| PERF-31→38 | 8 perf optimizations | 👀 **Mover a 🟢 o eliminar** | Muchas son premature optimization |

**Análisis de PERF-31→38:**

| ID | Tarea | Veredicto |
|----|-------|-----------|
| PERF-31 | Output batch via NumPy | ✅ Mantener 🟢 — mejora DX Python |
| PERF-32 | Async ingestion pipeline | ❌ Eliminar — ya existe via channel |
| PERF-33 | Prefetching HNSW | ❌ Eliminar — micro-optimization |
| PERF-34 | Norm caching | ❌ Eliminar — premature |
| PERF-35 | Async transcript I/O | ❌ Eliminar — transcript no es hot path |
| PERF-36 | Config hot-reload | ✅ Mantener 🟢 — buen DX |
| PERF-37 | FilterBitset overhead | ❌ Eliminar — premature |
| PERF-38 | Runtime multiversion dispatch | ✅ Mantener 🟢 — consolida PERF-21 |

### PHASE 5 — Enterprise

| ID | Tarea | Veredicto | Razón |
|----|-------|-----------|-------|
| TSK-72 | AES-256-GCM encryption | ✅ **Mantener 🟡** | Necesario para enterprise credibility |
| TSK-107b | Audit logging | ✅ **Mantener 🟡** | Útil incluso sin enterprise |
| BIZ-02 | WAL shipping replication | ❌ **Eliminar** | No hay mercado para esto hoy. Postergar a 2027 |
| TSK-122 | Sharded-slab HNSW | ✅ **Mantener 🟡** | Podría eliminar el Mutex bottleneck |
| TSK-131 | PITR via WAL | ❌ **Eliminar** | Feature enterprise que nadie pide |
| TSK-142 | WASM+OPFS+Web Workers | ✅ **Subir a 🟡** | Browser AI es diferenciador |
| ENT-01/02 | SOC2/HIPAA | ❌ **Eliminar** | Equivale a un año de trabajo. No hay negocio |
| ENT-03 | Multi-tenant isolation | ❌ **Eliminar** | No hasta que haya Cloud |
| GOV-01 | Governance redesign | ✅ **Mantener 🟡** | 12 bugs conocidos merecen atención |
| CLD-01→07 | Cloud business | ❌ **Eliminar todo** | No hay producto-market fit aún. Cloud es distracción |

---

## Parte 5: Items NUEVOS que Deberían Entrar al Backlog

### Críticos (entrar como TIER 0 o TIER 1)

| ID Propuesto | Tarea | Justificación | Esfuerzo | Prioridad |
|-------------|-------|---------------|----------|-----------|
| `NUEVO-01` | **Show HN prep: README hero** con readme-aura + benchmark gráfico + GIF demo WASM | El README es la landing page de facto para devs. Sin hero visual, Show HN falla | 🟡 2-3d | 🔴 |
| `NUEVO-02` | **WASM demo publicada en Vercel** con Transformers.js + OPFS | Showcase interactivo que cualquiera puede probar sin instalar nada | 🟡 2-3d | 🔴 |
| `NUEVO-03` | **llms.txt en raíz del repo** (mover desde web/public/) | AI-readability estándar 2025+. Sin esto, LLMs no entienden el proyecto | 🟢 1h | 🔴 |
| `NUEVO-04` | **CONTRIBUTING.md + CODE_OF_CONDUCT.md en raíz** (mover desde .github/) | Sin esto, contribuidores no llegan. GitHub detecta en raíz mejor | 🟢 1h | 🔴 |
| `NUEVO-05` | **Sanitizer CI: ASan + TSan** en rust_ci.yml | Sin sanitizers, bugs de memoria no detectados. Database-quality exige esto | 🟡 2-3d | 🔴 |
| `NUEVO-06` | **Flat index threshold** <10K brute-force | 10-100x más rápido en datasets pequeños. Competidores lo tienen | 🟡 2-3d | 🔴 |

### Alta Prioridad (TIER 1-2)

| ID Propuesto | Tarea | Justificación | Esfuerzo | Prioridad |
|-------------|-------|---------------|----------|-----------|
| `NUEVO-07` | **Migration tools: Chroma→Vanta, LanceDB→Vanta** | Capturar usuarios de Chroma/LanceDB es la estrategia de growth más obvia | 🟡 3-5d | 🟠 |
| `NUEVO-08` | **Learning path estructurado en tutorials/** con 5-7 ejemplos progresivos | Onboarding -> adopción. Sin learning path, loses devs | 🟡 2-3d | 🟠 |
| `NUEVO-09` | **TypeScript SDK: 50+ tests + type stubs + JSDoc** | SDK TS es la cara del producto. Hoy es débil | 🟡 2-3d | 🟠 |
| `NUEVO-10` | **Benchmark suite pública reproducible** (script + resultados) | Sin benchmarks públicos, las claims no son creíbles | 🟡 3-5d | 🟠 |

### Media Prioridad (TIER 2-3)

| ID Propuesto | Tarea | Justificación | Esfuerzo | Prioridad |
|-------------|-------|---------------|----------|-----------|
| `NUEVO-11` | **WASM IndexedDB fallback** cuando OPFS no está disponible | Compatibilidad cross-browser (Firefox private, Safari <15.2) | 🟡 2-3d | 🟡 |
| `NUEVO-12` | **WASM multi-tab coordination** (Web Locks + BroadcastChannel) | UX browser madura. TalaDB ya lo tiene | 🟡 2-3d | 🟡 |
| `NUEVO-13` | **HNSW auto-tuning PID loop** (ef_search dinámico) | FerresDB lo tiene. 5-15% mejora recall sin tuning manual | 🟡 3-5d | 🟡 |
| `NUEVO-14` | **WASM bundle size <500KB gzip** | Bundle grande mata adopción web | 🟡 1-2d | 🟡 |
| `NUEVO-15` | **Code coverage report en CI** + upload | Visibilidad. Stoolap lo tiene | 🟢 1d | 🟡 |

### Baja Prioridad (TIER 3+)

| ID Propuesto | Tarea | Razón |
|-------------|-------|-------|
| `NUEVO-16` | **Product Quantization (PQ) 96x** | Alto esfuerzo, no bloquea launch |
| `NUEVO-17` | **Segment LSM-style** | Muy alto esfuerzo, pico necesidad post-PQ |
| `NUEVO-18` | **Sparse vectors nativos** | Alto esfuerzo, depende de PQ+LSM |
| `NUEVO-19` | **Mover SourceDesign/ fuera de src/** | Bajo esfuerzo, ya identificado |
| `NUEVO-20` | **Server Docker image** | Bueno tener, no urgente |
| `NUEVO-21` | **Vectara** agregar a competitive research | Missing competitor |

---

## Parte 6: Items a ELIMINAR del Backlog

Tabla completa de items que recomiendo eliminar definitivamente:

| ID | Tarea | Razón para Eliminar |
|----|-------|---------------------|
| INT-11 | Semantic Kernel adapter | Mercado irrelevante para VantaDB |
| PERF-25 | Object pool PyDict | PERF-16 ya resuelve el problema raíz |
| PERF-29 | Cosine→Euclidean mapping | Optimización prematura sin data |
| PERF-33 | Prefetching HNSW | Micro-optimización, sin evidencia de bottleneck |
| PERF-34 | Norm caching | Premature optimization |
| PERF-35 | Async transcript I/O | Transcript no es hot path |
| PERF-37 | FilterBitset overhead | Bitset no es bottleneck conocido |
| CODE-074 | Visual regression tests | Mantener Percy cuesta tiempo que no hay |
| BIZ-02 | WAL shipping replication | Nadie va a pagar por esto en 2026 |
| TSK-131 | PITR via archival WAL | Feature enterprise sin demand |
| ENT-01 | SOC2 prep | 3-5 días para SOC2 es irreal. SOC2 toma meses |
| ENT-02 | HIPAA assessment | Sin negocio healthcare, es distracción |
| ENT-03 | Multi-tenant isolation | No hasta que exista Cloud |
| CLD-01→07 | Todo VantaDB Cloud | Product-market fit no está validado. Cloud es fuga de energía |
| CLD-06 | Stripe billing | Sin Cloud, no hay billing |
| PERF-32 | Async ingestion pipeline | Ya existe via channel — marcar ✅ |

Total eliminados: **18 items**. El backlog pasa de ~~78~~ a **60 items activos**.

---

## Parte 7: Backlog Reprocesado — Propuesta Final

### TIER 0 — Bloqueantes de Release (14→11 items)

| Prioridad | ID | Tarea | Esfuerzo |
|-----------|----|-------|----------|
| 🔴 | INT-01 | LangChain → PyPI + PR upstream | 🟡 1-2d |
| 🔴 | INT-02 | LlamaIndex → PyPI + PR upstream | 🟡 1-2d |
| 🔴 | DEVOPS-05 | Pipeline CI adapters | 🟡 1-2d |
| 🔴 | REL-02 | vantadb-ts → npm | 🟡 1-2d |
| 🔴 | NUEVO-03 | llms.txt en raíz | 🟢 1h |
| 🔴 | NUEVO-04 | CONTRIBUTING + CODE_OF_CONDUCT en raíz | 🟢 1h |
| 🔴 | NUEVO-05 | Sanitizer CI (ASan + TSan) | 🟡 2-3d |
| 🔴 | NUEVO-06 | Flat index threshold | 🟡 2-3d |
| 🔴 | REL-03 | Version sync 9 crates | 🟢 1h |
| 🔴 | MKT-13 | Demo WASM en hero | 🟡 1-2d |
| 🔴 | MCP-IDE | Docs setup MCP por IDE | 🟡 1-2d |

### TIER 1 — Pre-Lanzamiento (16→14 items)

| Prioridad | ID | Tarea | Esfuerzo |
|-----------|----|-------|----------|
| 🟠 | INT-03→11 | Resto de adaptadores PyPI | 🟡 1d c/u |
| 🟠 | DEVOPS-12 | PyPI signing pipeline | 🟡 1-2d |
| 🟠 | MKT-14 | Case studies (2) en landing | 🟡 1-2d |
| 🟠 | DOC-20 | Migration guide LanceDB | 🟡 1d |
| 🟠 | NUEVO-07 | Migration tools (Chroma, LanceDB) | 🟡 3-5d |
| 🟠 | NUEVO-08 | Learning path tutorials | 🟡 2-3d |
| 🟠 | NUEVO-09 | TypeScript SDK 50+ tests | 🟡 2-3d |
| 🟠 | NUEVO-10 | Benchmark suite pública | 🟡 3-5d |
| 🟠 | DEVOPS-06 | Homebrew formula | 🟢 4-6h |
| 🟠 | NUEVO-01 | README hero (readme-aura) | 🟡 2-3d |
| 🟡 | DEVOPS-10 | Windows SmartScreen signing | 🟡 2-3d |
| 🟡 | PERF-26 | Lazy serialization | 🟡 1-2d |
| 🟡 | "—" | Eliminar OldSerializationError | 🟢 1h |
| 🟡 | CODE-074 | Eliminar del backlog | 🟢 0h |

### TIER 2 — Launch Campaign (25→18 items)

| Prioridad | ID | Tarea |
|-----------|----|-------|
| 🔴 | LEG-01 | Trademark (iniciar ya) |
| 🔴 | MKT-03 | Show HN post |
| 🔴 | MKT-15 | Página benchmarks |
| 🔴 | COM-01 | Discord server |
| 🔴 | TS SDK hardening | Type safety + tests |
| 🟠 | MKT-04 | Reddit posts |
| 🟠 | MKT-05 | Technical blog posts |
| 🟠 | MKT-10 | "AI Agent Memory" campaign |
| 🟠 | TSK-106 | GitHub Discussions |
| 🟠 | NUEVO-12 | WASM multi-tab |
| 🟡 | MKT-16 | Metodología GraphRAG |
| 🟡 | MKT-17 | Página comparación (mover a 🟢) |
| 🟡 | TSK-107 | Community showcase |
| 🟡 | NUEVO-11 | WASM IndexedDB fallback |
| 🟡 | NUEVO-13 | HNSW auto-tuning PID |
| 🟡 | NUEVO-14 | WASM bundle size |
| 🟡 | TSK-104 | Demo LangChain + Ollama |
| 🟢 | NUEVO-15 | Code coverage CI |

### TIER 3 — Post-Lanzamiento (11→6 items)

| Prioridad | ID | Tarea |
|-----------|----|-------|
| 🟡 | Publicar 8 workspace crates.io |
| 🟡 | PERF-38 | Runtime multiversion dispatch |
| 🟢 | PERF-31 | Output batch via NumPy |
| 🟢 | PERF-36 | Config hot-reload |
| 🟢 | NUEVO-19 | Mover SourceDesign/ |
| 🟢 | NUEVO-20 | Server Docker image |

### PHASE 5 — Postergado Estratégicamente (21→6 items)

| Prioridad | ID | Tarea |
|-----------|----|-------|
| 🟡 | TSK-72 | AES-256-GCM encryption |
| 🟡 | TSK-107b | Audit logging |
| 🟡 | TSK-122 | Sharded-slab HNSW |
| 🟡 | TSK-142 | WASM+OPFS+Workers |
| 🟡 | GOV-01 | Governance redesign |
| 🟡 | NUEVO-21 | Vectara competitive research |

---

## Parte 8: Resumen de Carga de Trabajo Reprocesada

| Categoría | Items ❌ | Semanas | Depende de |
|-----------|---------|---------|------------|
| 🔴 Críticos pre-release | 11 | 1-2 | Nada |
| 🟠 Pre-lanzamiento | 14 | 2-3 | Críticos |
| 🟡 Launch campaign | 18* | 2-3 | Pre-lanzamiento |
| 🔵 Post-lanzamiento | 6 | 2-4 | Launch |
| ⬜ Phase 5 | 6 | Q4 2026 | Traction |
| **Total** | **55** | **~10-12 semanas** | |

*Incluye items de marketing que se ejecutan en paralelo al desarrollo.

### Comparación antes/después

```
Antes:  78 items ❌  +  1 ⏳  =  79 open
Después: 55 items ❌  +  0 ⏳  =  55 open
Reducción: 24 items eliminados o completados
```

---

## Parte 9: Conclusión Estratégica

### La Tesis de VantaDB

VantaDB tiene **un diferenciador real pero estrecho**: es la única base de
datos vectorial con cobertura multi-SDK completa + MCP server. Eso es
suficiente para ganar en el nicho de **memoria para agentes de IA**,
pero insuficiente para competir en el mercado general de vector DBs.

### Lo Más Importante que Puedes Hacer (en orden)

1. **Publicar.** Show HN + Reddit + Product Hunt. Sin tracción, no hay proyecto.
2. **LangChain/LlamaIndex en PyPI.** Sin integraciones, no hay adopción.
3. **WASM demo funcional.** El mejor sales pitch del producto.
4. **Community building.** Discord + CONTRIBUTING + good first issues.
5. **No construyas Cloud.** No hay demanda validada. Enfócate en embedded.
6. **Mantén el feature set acotado.** PQ > LSM > Sparse > Distributed. En ese orden.

### Lo Que NO Debes Hacer

| Tentación | Por qué evitarla |
|-----------|------------------|
| Construir Cloud/Server hosting | Product-market fit no validado. Distrae del core |
| SOC2/HIPAA compliance | Meses de trabajo para 0 clientes enterprise |
| gRPC streaming | Qdrant lidera ahí. No compitas en server-mode |
| Distributed Raft | Embedded-first es tu ventaja. No la abandones |
| Multi-tenancy nativa | Nadie la pide. Llega cuando haya Cloud |
| Pricing page antes de launch | No hay producto que vender aún |
| Perfect el engine antes de publicar | El engine es bueno. Publícalo YA |

### Frase Final

> VantaDB no necesita ser mejor que Qdrant.
> Necesita ser la base de datos vectorial más fácil de integrar
> en cualquier lenguaje. Ese es su territorio. Ahí gana o pierde.
