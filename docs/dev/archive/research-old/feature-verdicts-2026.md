---
title: Investigaciones v0.1 — Veredictos por Feature (histórico)
kind: research
status: archived
description: "Rescatado de OLD (2026-09-30) — marco de decisión por feature del análisis competitivo v0.1."
tags: [vantadb, archive, historico, competitivo]
---

> **[Histórico — rescatado 2026-09-30 de OLD]** Veredictos por feature (investigación competitiva 2026-06): marco reutilizable de decisión. Contexto actual: `docs/user/COMPARISON.md` + `benchmarks/COMPETITIVE_ANALYSIS.md`.

---
## Veredictos por Feature

---

### 🔴 SQL — NO antes del lanzamiento. Evaluar en Fase 5+

**Veredicto:** Déjalo fuera del roadmap hasta post-seed.

**Por qué no ahora:**
- Tu ICP (developers de agentes de IA) escribe código, no SQL. No usan SQL para consultar memoria de un agente.
- Implementar un parser SQL correcto es 3-6 meses de trabajo para 2-3 personas. Es una distracción total antes del lanzamiento.
- SQL no encaja bien con vectores y grafos — la razón por la que DuckDB existe como capa separada en LanceDB es precisamente porque el SQL no es el query nativo del vector space.
- pgvector ya "owns" el nicho SQL+vectores. No puedes ganarle ahí.

**Lo que SÍ deberías hacer en cambio:** Un sistema de filtros estructurado sobre metadata (tipo MongoDB query syntax simple) — mucho menor complejidad, mucho mayor utilidad para el ICP.

**Tarea a agregar al backlog:**
```
TSK-60 | Filtros estructurados de metadata | Phase 5 | Bajo
Sintaxis tipo: filter={"department": "legal", "version": {"$gte": 2}}
Sin parser SQL completo. Solo predicados sobre FieldValue.
```

---

### 🟢 TypeScript SDK — SÍ. Alta prioridad. Fase 4.

**Veredicto:** Es la brecha más subestimada del proyecto. Agrégalo como tarea de Fase 4.

**Por qué sí:**
- LangChain.js, LlamaIndex.TS, Vercel AI SDK, la mitad del ecosistema de agentes de IA corre en Node.js/Bun/Deno.
- Sin TS SDK, el TAM real de VantaDB es la mitad del mercado addressable.
- Cursor, Claude Code y Windsurf (tu ICP terciario mencionado en el MPTS) son herramientas que corren en entornos Node.js.
- LanceDB tiene TS SDK. ChromaDB tiene TS SDK. Si alguien busca "embedded vector db typescript" y VantaDB no aparece, pierdes el usuario.

**Cómo hacerlo sin morir en el intento:**
La ruta más pragmática es WASM. El core Rust ya existe, compilarlo a `wasm32-wasi` es menos trabajo que escribir bindings nativos NAPI desde cero. ROAD-01 ya está en tu backlog como "WASM Build" — conéctalo directamente con el SDK de TypeScript.

**Tareas a agregar:**
```
TSK-61 | TypeScript SDK vía WASM | Fase 4 | Crítico
  - Compilar vantadb-core a wasm32-wasi
  - Wrapper TypeScript sobre WASM
  - API: new VantaDB(path), put(), search(), delete()
  - Publicar en npm como vantadb

TSK-62 | TypeScript types + documentación | Fase 4 | Alto
  - Tipos TypeScript estrictos
  - Quickstart en Node.js, Bun, Deno
  - Ejemplo de integración con LangChain.js
```

---

### 🟡 Cuantización — SÍ, pero solo SQ8 escalar. Fase 3. Las demás NO por ahora.

**Veredicto:** Solo SQ8 (int8). El resto es distracción.

**Por qué SQ8 sí:**
- Ya lo tienes en el backlog como TSK-47. Solo necesitas priorizarlo.
- Reduce memoria 4x con pérdida de recall mínima (<1% en la mayoría de datasets).
- Para tu ICP que tiene 100K-500K vectores en RAM: la diferencia entre 1.17 GB y 293 MB es muy real en una máquina de desarrollo.
- Es relativamente sencillo: convertir `Vec<f32>` a `Vec<i8>` con factor de escala, distancia coseno con SIMD int8.

**Por qué NO las demás cuantizaciones ahora:**
- Cuantización 1.5-bit, 2-bit, asimétrica (como tiene Qdrant): extremadamente compleja de implementar correctamente, retorno marginal para datasets <1M vectores, y la pérdida de recall empieza a ser un problema real.
- Product quantization (PQ/IVF-PQ): requiere entrenamiento de centroides, cambio estructural del índice. Es semanas de trabajo solo para hacerlo bien.
- **Regla:** SQ8 para Fase 3. Todo lo demás para post-seed cuando tengas usuarios con datasets reales que lo pidan.

**Tarea actualizada (ya existe como TSK-47):**
```
TSK-47 | Cuantización escalar SQ8 | Fase 3 | Medio → Re-priorizar a Alto
  - Conversión f32 → int8 con factor de escala por dimensión
  - Distancia coseno con SIMD int8 (wide::i8x16)
  - Flag en put(): quantize=True (opt-in)
  - Benchmark: recall@10 antes/después
  - Target: 4x reducción memoria, <1% pérdida recall
```

---

### 🔴 Dataset > RAM con IVF-PQ disk-based — NO. No es tu mercado.

**Veredicto:** Fuera del scope. Tu ICP no tiene este problema.

**Por qué no:**
- LanceDB es el "multimodal lakehouse" — su ICP son Netflix, Uber, Harvey. Ese no es tu mercado.
- Un desarrollador de agentes de IA con 1M vectores en memoria cognitiva de un agente estaría manejando ~1.17 GB RAM. Eso cabe en cualquier laptop moderna.
- Perseguir IVF-PQ disk-based significa competir en el terreno donde LanceDB lleva años de ventaja y $41M de financiación.
- Sería semanas de trabajo para un caso de uso que tu ICP actual no tiene.

**Lo que SÍ puedes hacer en cambio:** mmap-backed HNSW (TSK-46 ya en tu backlog). Permite índices más grandes que la RAM disponible sin cambiar la arquitectura del índice. Es la solución correcta para tu escala objetivo.

```
TSK-46 | MMap-backed HNSW | Fase 3-4 | Alto (ya en backlog)
  Re-priorizar. Esto resuelve "quiero 500K-1M vectores en una máquina de 8GB"
  sin necesidad de IVF-PQ.
```

---

### 🔴 Versionado de datos (git-style) — NO. No es el dolor de tu ICP.

**Veredicto:** Fuera del scope hasta post-seed.

**Por qué no:**
- LanceDB tiene versionado porque su ICP son data engineers y equipos de ML que necesitan reproducibilidad de experimentos. Eso no es tu ICP.
- Un agente de IA actualiza memorias continuamente — no necesita git-style branching, necesita que sus writes sean durables y no se pierdan en un crash. Eso ya lo resuelves con el WAL.
- Implementar versionado correcto (copy-on-write, snapshot isolation, compactación de versiones antiguas) es complejidad enorme para beneficio marginal en tu caso de uso.

**Lo que SÍ tienes que tener:** export/import de snapshots (ya está en tu SDK como `export_all` / `import_file`). Es suficiente para "quiero hacer un backup manual de la memoria de mi agente".

---

### 🟡 Backup/Restore nativo — SÍ, pero simple. Fase 4.

**Veredicto:** Sí, pero no S3. Solo snapshot local primero.

**Por qué sí:**
- Es la pregunta número uno de cualquier developer que pone algo en producción: "¿cómo hago backup de esto?"
- Tu WAL ya tiene toda la información necesaria para un snapshot consistente.
- Sin esto, los enterprise pilots (Fase 5) son imposibles de vender.

**Por qué no S3 todavía:**
- Añadir dependencia de AWS SDK rompe la filosofía zero-config.
- S3 es Fase 5 (VantaDB Cloud). Para la comunidad, un backup local es suficiente.

**Tareas a agregar:**
```
TSK-63 | CLI: comando `backup` | Fase 4 | Alto
  vanta-cli backup --db ./data --output ./data.vantadb.bak
  - Flush WAL antes de copiar
  - Copia atómica del directorio (WAL + Fjall + mmap)
  - Verificación de integridad con CRC32C

TSK-64 | CLI: comando `restore` | Fase 4 | Alto
  vanta-cli restore --from ./data.vantadb.bak --to ./data
  - Verificar integridad del backup
  - Reconstruir índices si es necesario
```

---

### 🔴 Modelos de embedding y módulos de vectorización integrados — NO. Nunca en el core.

**Veredicto:** No. Contradice completamente tu filosofía.

**Por qué no:**
- Weaviate tiene esto porque es un servidor completo que se encarga de todo el pipeline. Tú eres una librería embebida.
- Añadir modelos de embedding significa añadir dependencias de PyTorch, CUDA, ONNX Runtime, y/o APIs externas al core. Eso destruye el "zero-config" de un `pip install vantadb-py`.
- Tu ICP ya tiene sus embeddings — usa OpenAI, Cohere, Ollama, sentence-transformers. No necesita que VantaDB se los proporcione.
- Una dependencia de modelo haría que el wheel de PyPI pase de ~5MB a ~500MB o más.

**Lo que SÍ puedes hacer:** Adaptadores opcionales en paquetes separados.

```
TSK-65 | vantadb-openai (paquete opcional) | Fase 4 | Bajo
  pip install vantadb-openai
  from vantadb_openai import OpenAIEmbedder
  db.put(key, text=doc, embedder=OpenAIEmbedder())
  # Genera embedding automáticamente antes del put()

TSK-66 | vantadb-ollama (paquete opcional) | Fase 4 | Bajo
  pip install vantadb-ollama
  Integración con Ollama local para embeddings offline completos
```

Esto da la conveniencia de Weaviate sin romper la arquitectura.

---

### 🔴 GraphQL / knowledge graph API — NO. Ya tienes el grafo. GraphQL es overhead.

**Veredicto:** Ya tienes un knowledge graph (UnifiedNode con edges + traversal). GraphQL es solo una interfaz de query — y no la correcta para tu ICP.

**Por qué no:**
- Tu ICP son developers de Python y Rust que llaman funciones, no clients de GraphQL.
- GraphQL añade: schema definition, parser, execution engine, introspection. Meses de trabajo.
- Weaviate usa GraphQL porque es un servidor web completo. Tú eres una librería.
- El MCP server que ya tienes es una interfaz mucho más relevante para el ecosistema de agentes.

**Lo que SÍ tienes que documentar mejor:** Que VantaDB ya tiene graph traversal nativo. El MPTS lo menciona pero los ejemplos de código no muestran cómo hacer multi-hop queries. Eso es un gap de DX, no de features.

```
TSK-67 | Documentar y ejemplificar graph traversal | Fase 3 | Alto
  - Ejemplo: BFS/DFS desde un nodo con profundidad N
  - Ejemplo: GraphRAG completo (vector search → expand neighbors → inject context)
  - Benchmark de reducción de tokens (el claim 40-60% necesita un ejemplo reproducible)
```

---

### 🟡 Escalabilidad — Sí, pero dentro del modelo single-node. Nada distribuido.

**Veredicto:** Mejora la escalabilidad del nodo único. No toques distribución.

**Las tres palancas correctas para escalar sin cambiar arquitectura:**

**1. Python SDK latency (el más urgente — ya discutido):**
```
TSK-68 | Zero-copy FFI: devolver memoryview/NumPy arrays | Fase 3 | CRÍTICO
  - Resultado de search: devolver buffer NumPy en lugar de lista Python
  - Reducir de ~40ms a ~5ms en conversión de datos
  - Objetivo: 62ms → <20ms total en Python SDK
```

**2. search_batch con paralelismo Rayon (ya tienes 4x speedup, mejorar):**
```
TSK-69 | Expandir search_batch a todas las operaciones | Fase 3 | Alto
  - put_batch() con Rayon paralelo
  - delete_batch() atómico
  - Import masivo con validación paralela de CRC32C
```

**3. mmap-backed HNSW para datasets grandes (TSK-46, ya en backlog):**
- Escalar de 100K a 1M vectores sin OOM. Esta es la palanca de escalabilidad correcta.

**Lo que NO debes tocar para escalar:**
- Distributed mode, sharding, Raft — fuera hasta post-seed. Te comerán meses.
- Multi-tenancy en el core — una instancia por tenant es el workaround documentado y es suficiente.

---

### 🔴 Uptime SLA — NO aplica al producto actual. Aplica a VantaDB Cloud en Fase 5.

**Veredicto:** No es una feature de producto, es una garantía de servicio. Completamente irrelevante para una librería embebida.

**Qué significa realmente:** Un SLA (Service Level Agreement) del 99.9% de uptime es un compromiso contractual de un managed service. Tú eres una librería que corre in-process — el "uptime" es responsabilidad del proceso del usuario.

**Lo que SÍ debes documentar:** Las garantías de durabilidad (WAL + CRC32C), el comportamiento en crash (recovery automático), y los chaos tests que lo validan. Eso es tu equivalente al SLA.

```
TSK-70 | Documento de garantías de durabilidad | Fase 4 | Alto
  - Tabla: "VantaDB garantiza X en escenario Y"
  - Ej: "Zero pérdida de datos en kill -9 validado por chaos tests"
  - Ej: "Recovery automático en <100ms tras crash"
  - Ej: "CRC32C detecta y rechaza registros WAL corruptos"
  Esto es lo que un developer quiere saber antes de poner VantaDB en producción.
```

---

### 🟡 Edge Devices — Ya eres un edge device. Solo falta el WASM build.

**Veredicto:** VantaDB ya es una base de datos para edge devices por diseño. La única pieza que falta es WASM para browser/serverless.

**Lo que ya tienes:** Embedded, local-first, zero-network, offline — eso ya es edge computing. Cuando Qdrant lanzó "Qdrant Edge" básicamente describió lo que VantaDB es por defecto.

**Lo que te falta para completar la historia de edge:**
```
TSK-71 | WASM build (wasm32-wasi) | ✅ Completado | Medio
  - Compilar vantadb-core a WASM
  - Habilitar ejecución en browser (Cloudflare Workers, Deno Deploy, browsers)
  - Nota: WASM no tiene acceso a filesystem nativo, usar OPFS (Origin Private File System)
  - Objetivo: VantaDB corre en el browser → caso de uso privacidad extrema
  Este task ya está como ROAD-01 en el backlog. Re-priorizar a Fase 4.
```

---

### 🔴 Distributed / Replicación / Raft / Zero-Downtime — NO. Post-seed. Con equipo más grande.

**Veredicto:** Explícitamente fuera del roadmap hasta Q2 2027+.

**Por qué no:**
- Raft consensus correcto es 6-12 meses de ingeniería para un equipo de 2-3 personas. Es el proyecto paralelo que mata proyectos.
- Contradice la filosofía embedded-first. VantaDB distribuido ya no es "el SQLite para agentes" — es "el Qdrant pero peor porque tiene menos equipo".
- Tu ICP actual no necesita distribución. Un agente de IA tiene memoria local. Distribución es un problema de infraestructura enterprise, no de developer tools.

**La hoja de ruta correcta:** Si un enterprise pilot pide distribución, el workaround es múltiples instancias + WAL shipping manual. DIST-01 y DIST-02 permanecen en el backlog como postpuestos indefinidamente hasta post-seed.

**Una pequeña excepción sí vale:** WAL shipping asíncrono (BIZ-02 ya en backlog) — es mucho más simple que Raft y satisface el caso de uso básico de replicación para disaster recovery.

```
BIZ-02 | WAL Shipping asíncrono (módulo comercial) | Fase 5 | Medio
  Re-evaluar después del lanzamiento comunitario.
  No es Raft. Es: "copia el WAL a otra máquina cada N segundos".
  Suficiente para el primer caso de uso enterprise de alta disponibilidad.
```

---

### 🔴 RBAC / SSO — NO en el core. Solo en VantaDB Cloud (Fase 5+).

**Veredicto:** No aplica a una librería embebida single-process.

**Por qué no:**
- RBAC (Role-Based Access Control) requiere que haya múltiples usuarios accediendo a la misma instancia. En el modelo embebido, el proceso que abre la DB es el único "usuario" — el control de acceso es responsabilidad del OS.
- SSO/OAuth2/OIDC es infraestructura de servicio web, no de librería.
- Cuando VantaDB Cloud exista (Fase 5), RBAC y SSO son obligatorios. Hasta entonces: irrelevante.

**Lo que SÍ puedes hacer para enterprise readiness básica:**
```
TSK-72 | Encriptación at-rest (opcional) | Fase 5 | Medio
  Encriptar archivos de datos con clave proporcionada por el usuario.
  Satisface requerimientos HIPAA/GDPR básicos sin RBAC.
  Más relevante que SSO para tu ICP de compliance.
```

---

### 🔴 GPU Acceleration — NO. Contradice todo lo que eres.

**Veredicto:** Nunca en el core embebido. Quizás en VantaDB Cloud como feature premium.

**Por qué no:**
- GPU requiere CUDA, que requiere drivers de NVIDIA, que destruye el zero-config inmediatamente.
- El cuello de botella de VantaDB hoy no es la velocidad de indexación HNSW (ya está en 12.4ms p50). Es la **latencia del Python SDK (62ms)**. GPU no resuelve eso.
- Tu ICP (developer de agentes en un MacBook) no tiene GPU NVIDIA.
- La aceleración GPU de Qdrant es para enterprise que indexa 100M vectores en batch. Tu ICP indexa 1K-100K vectors de forma incremental.

---

## Síntesis: Backlog Priorizado por Fases

### Fase 3 — Completar ANTES del lanzamiento (urgente)

```
CRÍTICO — Bloquea el lanzamiento:
  TSK-56 | Fix Windows CI (runner inexistente)
  TSK-68 | Python SDK: zero-copy FFI → <20ms latencia
  DISC-05 | Fix telemetría de memoria (~225GB falsos)

ALTO — Necesario para credibilidad en benchmarks:
  TSK-47 | Cuantización SQ8 (4x reducción memoria)
  TSK-46 | mmap-backed HNSW (1M vectores sin OOM)
  TSK-67 | Documentar y ejemplificar graph traversal con benchmark
  TSK-55 | Datasets de prueba reales (GloVe, NQ) en CI
```

### Fase 4 — Para el lanzamiento comunitario (septiembre 2026)

```
CRÍTICO para crecimiento:
  TSK-61 | TypeScript SDK vía WASM
  TSK-62 | TypeScript types + documentación

ALTO — Developer experience:
  TSK-63 | CLI: comando backup
  TSK-64 | CLI: comando restore
  TSK-70 | Documento de garantías de durabilidad
  TSK-25 | CLI: comando search semántico
  TSK-26 | CLI: comando delete

MEDIO — Ecosistema:
  TSK-65 | vantadb-openai (paquete opcional)
  TSK-66 | vantadb-ollama (paquete opcional)
  TSK-71 | WASM build para edge/browser (✅ Completado — 2026-06-21)
```

### Fase 5 — Post-lanzamiento comunitario, pre-seed (Q4 2026)

```
ALTO — Para enterprise pilots:
  TSK-60 | Filtros estructurados de metadata (no SQL)
  TSK-72 | Encriptación at-rest opcional
  BIZ-02 | WAL Shipping asíncrono (replicación básica)
  TSK-45 | Publicar en crates.io

MEDIO — Para escala:
  TSK-53 | Validación estricta de metadata en FFI
  TSK-50 | Filtro de admisión (backpressure al 80% RAM)
```

### Explícitamente fuera del roadmap (hasta post-seed con equipo ampliado)

```
❌ SQL completo
❌ IVF-PQ disk-based (>RAM)
❌ Raft distributed / sharding
❌ RBAC / SSO en core
❌ GPU acceleration en core
❌ Versionado de datos git-style
❌ GraphQL API
❌ Embedding models bundled en el core
❌ S3 backup nativo (cloud only, Fase 5+)
❌ Cuantización 1.5-bit / 2-bit / asimétrica
```

---

## La Regla de Oro

Hay una sola métrica que define si VantaDB tiene éxito en el Show HN de septiembre: **¿puede un developer hacer `pip install vantadb-py` y tener un agente con memoria persistente, búsqueda híbrida y GraphRAG en menos de 10 líneas de código, con latencia <20ms, en menos de 2 minutos?**

Todo lo que no contribuya directamente a esa demo no entra al roadmap antes de septiembre. Lo que sí contribuye: fix del Python SDK, TypeScript SDK para el segundo tutorial, backup básico para credibilidad en producción, y documentación de GraphRAG con un benchmark reproducible del 40-60% de reducción de tokens.
