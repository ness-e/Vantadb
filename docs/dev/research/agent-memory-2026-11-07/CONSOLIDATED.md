---
title: "CONSOLIDATED — Investigación comparativa agent-memory (2026-11-07)"
kind: research
status: proposed
description: "Síntesis de los diez puntos del encargo sobre 37 repositorios de memoria/contexto para agentes más OpenViking"
tags: [research, memory, consolidated]
---

# CONSOLIDATED — Investigación comparativa agent-memory (2026-11-07)

> **Estado:** síntesis sobre las **37 fichas completas** (22 iniciales + 15 de la segunda tanda) + informe central OpenViking. Evaluación **estática**: sin ejecución de código, tests, benchmarks ni servicios.
> **Vigencia:** toda lectura de VantaDB es histórica (HEAD inicial `c561158` + worktree dirty de la fase; HEAD avanzó por commits ajenos). Nada aquí afirma defectos vigentes ni superioridad medida — **revalidar antes de tareas**.
> **Fuentes:** [METHOD.md](METHOD.md) (contrato de evidencia), [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md), [README.md](README.md) (inventario congelado), informe central [../../Backlog-OpenViking-VantaDB.md](../../Backlog-OpenViking-VantaDB.md), fichas individuales de este directorio.

## Índice

1. [Recomendaciones](#1-recomendaciones)
2. [Candidatos unificados](#2-candidatos-unificados)
3. [Issues y defectos transferibles](#3-issues-y-defectos-transferibles)
4. [Prerrequisitos en VantaDB](#4-prerrequisitos-en-vantadb)
5. [Que no trasladar](#5-que-no-trasladar)
6. [Advertencias metodológicas](#6-advertencias-metodológicas)
7. [Benchmarks](#7-benchmarks)
8. [Matriz de brechas](#8-matriz-de-brechas)
9. [Negativos y deuda propia](#9-negativos-y-deuda-propia)
10. [Escritorio / UI](#10-escritorio--ui)

## 1. Recomendaciones

**Decisiones cerradas:** conservar Rust embebido, `vanta-memory` y adaptadores+Studio separados; **no importar productos completos ni reemplazar HNSW/BM25/RRF**. Toda adopción es reimplementación de diseño con provenance; licencias source-available/`NOASSERTION` se documentan por proyecto, no se asumen OSI.

**Orden de decisión (prioridad cualitativa, no números inventados):**

1. **Garantías y contratos + evaluación confiable** — C1, C2, C13, C14: antes de amplificar escritura ni certificar calidad.
2. **Identidad, derivados, scopes + assembler y ledger** — C3–C8.
3. **Ciclo de vida, experiencias, adaptadores, evidencia** — C9–C12, C15.
4. **UI diagnóstica read-only** — C16: puede avanzar en paralelo.

Reglas transversales: extender seams existentes (context_assemble, auditoría de índice, scheduler/jobs, dedup post-decisión, ingestors, Studio) sin duplicar sistemas; ninguna recomendación es claim de rendimiento (no hay comparación medida); LLM opcional fuera de primitivas del motor.

## 2. Candidatos unificados

IDs propios, no filas oficiales de backlog. Cada candidato recoge acciones/dependencias/FMEA/aceptación de propuestas ya escritas en las fichas enlazadas; los DTO son **hipotéticos**, no APIs existentes.

### C1 — Revalidar garantías propias: atomicidad delete+replace L1, ACK≠ready, recovery

- **Prioridad:** alta (primero). **Fuentes:** central §4/§9, [agno.md](agno.md) (clear+upserts sin transacción global visible — lección análoga), [flowise.md](flowise.md), [mem0.md](mem0.md).
- **Seam Vanta:** `l1_writer`, `checkpoint`, capas txn/WAL del engine.
- **Acciones:** reproducir fallo inyectado entre delete y put en ruta batch/secuencial; contrato de escritura indivisible; prueba determinista de la carrera RMW del checkpoint global; matriz ACK/ready/restart por frontera.
- **Dependencias:** revalidación de hallazgos [Histórico] contra el estado vigente (FIND-221/TXN-01/CRASH-01 son distintos, no fusionar).
- **FMEA:** crash entre delete y put → pérdida sin reemplazo; escritor concurrente; timeout no equivale rollback.
- **Aceptación:** fallo inyectado no deja deletción aplicada sin reemplazo; RMW concurrente sin lost-update demostrado en prueba.

### C2 — Evaluadores confiables: dataset real explícito, backend real, holdout congelado

- **Prioridad:** alta (primero). **Fuentes:** central §7, [mem0.md](mem0.md) §7 (submódulo a gitlink `4b61c5d`), [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md), [graphiti.md](graphiti.md) (contexto del juez — §3).
- **Seam Vanta:** `evals/*` existentes (`eval_locomo`, `eval_longmemeval`, `memory_harness`).
- **Acciones:** blindar LoCoMo (MEM0-05): dataset y backend reales obligatorios para certificación; sintético/fake solo en modos explícitos marcados como no certificables; adapter al esquema original; CSV por pregunta (attempted/completed/graded/error/abstention); calibración del juez en conjunto de desarrollo separado, holdout congelado (EVAL-02).
- **Dependencias:** MEM0-05/EVAL-02 del backlog histórico; ningún runner destructivo sobre datos compartidos.
- **FMEA:** unidad sesión/turno ambigua; denominador que descarta fallos; tool-disabling en QA; resultado "declarado" ≠ "superado".
- **Aceptación:** corrida reporta intentadas/completadas/graduadas/errores sin omitir errores del denominador; dataset real citado por nombre.

### C3 — Scope de consulta jerárquico como complemento de `MemoryScope`

- **Prioridad:** alta. **Fuentes:** central F1, [hindsight.md](hindsight.md), [understand-anything.md](understand-anything.md).
- **Seam Vanta:** filtros del motor + MEMG-25 (spec histórica).
- **Acciones:** pushdown de `PathScope`-equivalente al filtro del motor; canonicalización en escritura y consulta; paginación estable de hijos/ancestros.
- **Dependencias:** C4; enforcement en la frontera gobernada de read/write (no solo middleware HTTP).
- **FMEA:** escape de subárbol por path no canónico; colisión entre tenants; scope ≠ authz.
- **Aceptación:** colisión de tenant, escape por `..`/encoding, límites de profundidad cubiertos por contrato.

### C4 — Identidad de contexto opaca + revisiones de fuente

- **Prioridad:** alta. **Fuentes:** central F1, [graphify.md](graphify.md) (`worktree_digest`, `extractor_version`, `schema_version`, `generation`), [codegraph.md](codegraph.md).
- **Seam Vanta:** registro de memoria/contexto existente + canonicalización.
- **Acciones:** `ContextId` opaco estable; `ContextObject { id, source_ref, parent: Option<ContextId>, revision, kind, scope }` (revisión/relaciones fuera de la identidad); `resolve(id)` sin duplicar payload.
- **Dependencias:** esquema de registro; MEMG-25/FIND-320 como evidencia.
- **FMEA:** rename/move actualiza relaciones, no identidad; revisión nueva no cambia identidad; padre mutable jamás dentro del id.
- **Aceptación:** rename preserva provenance; nueva revisión no es nueva identidad.

### C5 — Preservación de raw + derivados versionados

- **Prioridad:** alta. **Fuentes:** central F2, [cognee.md](cognee.md), [mempalace.md](mempalace.md), [light-rag.md](light-rag.md).
- **Seam Vanta:** wiki/scenes/jobs + `FilterBitset` por nivel de derivado.
- **Acciones:** abstract/overview materializados por unidad con `source_revision`+`generator_version`+`state`; generación async; fuente canónica nunca borrada por fallo del derivado.
- **Dependencias:** C1 (escritura segura L1) antes de cualquier cascada de borrado.
- **FMEA:** derivado viejo sobreescribe uno nuevo; update durante generación → descarte de stale; fallo borra fuente (prohibido).
- **Aceptación:** publicación condicionada a revisión vigente; fuente sobrevive a fallo de derivado.

### C6 — Consolidación/publicación condicionada de derivados

- **Prioridad:** alta. **Fuentes:** central F2/F5 (merge declarativo), [hindsight.md](hindsight.md) (vigencia de fuentes), [agno.md](agno.md) (plan clear+upsert — rehacer transaccional).
- **Seam Vanta:** dedup post-decisión (`l1_dedup`) + jobs.
- **Acciones:** merge declarativo con enum cerrado (patch/replace/sum/immutable/link); precedencia explícita (autoridad, revisión, política, desempate estable); guarda de replay para sum; rechazo de mutaciones desconocidas/incompatibles/protegidas.
- **Dependencias:** C1; nunca reemplaza dedup existente.
- **FMEA:** replay de sum duplica; conflicto de precedencia sin regla; tipo incompatible silencioso.
- **Aceptación:** replay idempotente, conflicto con precedencia declarada, mutación protegida rechazada.

### C7 — Contrato de retrieval: score/candidato/filtro/budget

- **Prioridad:** alta. **Fuentes:** central F3, [light-rag.md](light-rag.md), [mem0.md](mem0.md) §2.
- **Seam Vanta:** `context.rs`/`context_engine` (assembler existente — extender, no duplicar).
- **Acciones:** presupuesto total/reservado para respuesta/cuotas por categoría/redistribución de sobrante; descartes con razón y conteo; estimador de tokens declarado (no tokenizer exacto); desempates estables.
- **Dependencias:** C4 (revisión para citar); política source→overview→abstract; supersession.
- **FMEA:** presupuesto diminuto/unicode; scores de dominios distintos mezclados; cita a revisión obsoleta.
- **Aceptación:** descartes enumeran razón; presupuesto respetado con mínimo viable; `error≠ausencia` propagado.

### C8 — Ledger `Selected|Rendered|Delivered` por sesión

- **Prioridad:** alta. **Fuentes:** central F4, [mem0.md](mem0.md) (recibo multialmacén).
- **Seam Vanta:** estado dedicado por sesión sobre el store existente — **no** el JSON global del checkpoint (ver §4).
- **Acciones:** entradas `{context_id, revision, state, turn}`; update atómico (CAS o sección crítica sobre read+write); `confirm_delivery` idempotente; ventana de turnos.
- **Dependencias:** C4; C7 puntos de selección/render. La carrera del checkpoint se resuelve aparte.
- **FMEA:** cancel-before-delivery no suprime; concurrencia sin lost-update; nueva revisión no suprimida; **sin promesa exactly-once de red**.
- **Aceptación:** dos recalls concurrentes preservan ambas entregas; confirmación repetida idempotente.

### C9 — Lifecycle de ingesta observable + cancelación

- **Prioridad:** alta. **Fuentes:** central F7, [serviette.md](serviette.md), [anything-llm.md](anything-llm.md).
- **Seam Vanta:** scheduler + ingestors de claves deterministas.
- **Acciones:** estados `Captured→ExtractionQueued→Extracted→Indexing→Ready` + `Failed/Retryable/Cancelled`; revisión, timestamps, idempotencia, error/resultado parcial; reintentos acotados; ACK capture ≠ ready.
- **Dependencias:** C1 para límites de recuperación.
- **FMEA:** deferral de lock ≠ fallo de ejecución; restart en cada frontera; reintento infinito (acotar).
- **Aceptación:** job falla/reintenta/cancela con razón visible; restart no pierde estado.

### C10 — Experiencias/propuestas aprobables

- **Prioridad:** media. **Fuentes:** central F8, [hindsight.md](hindsight.md), [supermemory.md](supermemory.md), [crewai.md](crewai.md) (update que pierde `source/private` — lección).
- **Seam Vanta:** reflection/skills/scenes existentes.
- **Acciones:** `Experience { situation, objective, preconditions, actions, outcome, evidence_refs, evidence_status: Claimed|Observed|Verified, expiry }`; promoción declarativa experiencia→skill con evidencia + aprobación humana configurada; update preserva metadatos (`source`, `private`, refs).
- **Dependencias:** C1/C8/C9 para evidencia confiable; FIND-290/295/288.
- **FMEA:** claim ≠ verificado; instrucción recuperada nunca se ejecuta; secretos; contaminación train/test.
- **Aceptación:** sin evidencia no marca `Verified`; experiencia fallida recuperable; update conserva `private/source`.

### C11 — Adaptadores de integración versionados (frameworks)

- **Prioridad:** media. **Fuentes:** [langgraph.md](langgraph.md), [langchain.md](langchain.md), [crewai.md](crewai.md), [autogen.md](autogen.md), [openai-agents-python.md](openai-agents-python.md).
- **Seam Vanta:** bindings/integrations existentes (Python/Node/TS/MCP) — adaptadores versionados por framework sobre esos bindings, **no** runtime de framework ni schema on-disk genérico.
- **Acciones:** contrato de adaptador con `adapter_version` + capacidades declaradas (scopes, filtros, escritura); superficie mínima recall/record/config; migración declarativa de versiones.
- **Dependencias:** contratos C7/C8 estables; nada de llamadas obligatorias al LLM en camino síncrono.
- **FMEA:** drift de API del framework; adapter finge paridad que no tiene; versión de framework no fijada.
- **Aceptación:** adaptador declara versiones/framework probados; capacidad no soportada falla explícito, no silencio.

### C12 — `CodeEvidenceImport`: contrato canónico único de evidencia code-memory

- **Prioridad:** media. **Fuentes:** [codebase-memory-mcp.md](codebase-memory-mcp.md), [graphify.md](graphify.md), [understand-anything.md](understand-anything.md), [codegraph.md](codegraph.md).
- **Seam Vanta:** `code_index` / wiki / jobs.
- **Contrato (hipotético):** `identity: {repo, revision, producer, schema_version, generation}`; `refs: {source, target}` con `relation`+`direction`; `evidence: {origin: Ast|Heuristic|Model|Unknown, span?, hash?, reported_weight?}` — `reported_weight` opcional, **peso ≠ confianza**. Cada productor mapea al mismo contrato — no tres productos de memoria.
- **Acciones:** adapter por productor al contrato; invalidación por cambio de fuente/gramática/productor; conservar schema y snapshot original.
- **FMEA/aceptación:** mención textual no aparece como call AST; origen desconocido queda `Unknown`; cambio de fuente invalida la evidencia.
- **Non-goals:** propuesta, no implementación ahora; no fusiona con memoria conversacional.

### C13 — `MutationReceipt`: recibo de mutación multialmacén

- **Prioridad:** alta. **Fuentes:** [qdrant.md](qdrant.md), [milvus.md](milvus.md), [meilisearch.md](meilisearch.md), [redis.md](redis.md), [mem0.md](mem0.md).
- **Seam Vanta:** store/WAL existentes.
- **Contrato (hipotético, diseño corregido):** `MutationReceipt { operation_id, primary_commit, primary_durability: Option<DurablePosition>, projections: Vec<ProjectionStatus>, visibility_generation, outcome }`. **Clave:** `accepted`/`applied`/`durable` **no** son un orden total ni estados mutuamente excluyentes — la durabilidad de la fuente/WAL puede preceder a la visibilidad de índices reconstruibles. Barrera y dominio de durabilidad explícitos por backend; readiness es campo separado; no se exige fsync individual de cada índice derivado.
- **Acciones:** recibo por operación con proyecciones pendientes observables; retry idempotente en su alcance.
- **Dependencias:** C1 (dominio de durabilidad del primary).
- **FMEA/aceptación:** timeout ≠ rollback; proyección retrasada visible como `pending`; primary durable no promete índice listo; éxito primario no certifica proyecciones; no exactly-once global.

### C14 — Frescura y evidencia: derivado↔fuente por revisión

- **Prioridad:** alta. **Fuentes:** central F6, [graphify.md](graphify.md) (`worktree_digest`), [mempalace.md](mempalace.md).
- **Seam Vanta:** `cli_handlers/index.rs` (audit existente — extender eje contenido↔vector↔derivado).
- **Acciones:** detectar missing/orphan/stale/inválido por revisión; diagnose separado de repair; jobs idempotentes guardados por revisión vigente; `error≠ausencia`.
- **Dependencias:** C4/C5; revalidar que el índice auditado cubre derivados.
- **FMEA:** repair borra algo generado concurrentemente; lectura con error reportada como ausencia.
- **Aceptación:** derivado stale detectado por revisión; reparación idempotente y acotada.

### C15 — Capacidades de backend/embedding declaradas por entorno

- **Prioridad:** media. **Fuentes:** [qdrant.md](qdrant.md), [milvus.md](milvus.md), [redis.md](redis.md), [tidb.md](tidb.md), central OV.
- **Seam Vanta:** config existente de backends/embeddings.
- **Acciones:** manifiesto por entorno (sparse/dense/híbrido soportado por el backend configurado); capacidad no soportada rechazada explícito, no degradada silenciosa.
- **Dependencias:** C7 (dominio de score declarado).
- **FMEA:** feature anunciada no soportada por el backend elegido; fallback silencioso.
- **Aceptación:** matriz backend↔capacidad declara límites; consulta incompatible es error explícito.

### C16 — Inspector read-only + onboarding en Studio existente

- **Prioridad:** media (paralelo). **Fuentes:** central F10, [anything-llm.md](anything-llm.md), [dify.md](dify.md), [khoj.md](khoj.md).
- **Seam Vanta:** VantaDB Studio (worktree `develop-desktop`).
- **Acciones:** timeline captura→extracción→recall→inyección con evidencia real; detección de integraciones con consentimiento + preview/repair sin sobreescrituras; sync selectivo; `installed≠functioning`.
- **Dependencias:** C7/C8/C9 para la evidencia mostrada.
- **FMEA:** tokens expuestos; diagnóstico presenta "instalado" como "funcionando".
- **Aceptación:** inspector muestra evidencia real del ciclo; onboarding detecta sin sobreescribir.
- **Limitación declarada:** APIs/componentes concretos de integración quedan fuera del alcance de estas fichas — no se presenta seam listo.

## 3. Issues y defectos transferibles

Matriz completa y muestra (79 issues upstream, 15 leídos) en el central §3. Prioridades conservadas: lost-update de ledger concurrente (#7170), delete-before-replace (#7099 — mecanismo distinto), knob hard-coded→FIND-313 (#6923), read-error≠ausencia (#7132), stale tras update (#5873).

**Hallazgos estáticos selectivos de las fichas (defecto en código; impacto no medido, no issue inventada):**

| Fuente | Hallazgo estático | Transferencia |
|---|---|---|
| [graphiti.md](graphiti.md) | En el bucle del juez `episodes` es string: `message=episodes[0]`/`previous_messages=episodes[1:]` indexan caracteres, no mensajes — contexto del juez degradado | Validar que un evaluador propio no degrade el contexto del juez (C2) |
| [codegraph.md](codegraph.md) | `scoreSearchNodes` fija `firstRank` con el primer símbolo **esperado** encontrado, no el menor rank | Scorer: primer rank real vs primer esperado (C2/C14) |
| [flowise.md](flowise.md) | Dedup por score → pérdida de documentos; resultados sobreescritos; disable de UI sobreescrito por env | Dedup nunca por score solo; identidad por contenido+source; env vs UI documentado |
| [agno.md](agno.md) | Optimización: `clear` + upserts sin transacción global visible | Análogo al reemplazo L1 (C1): consolidación siempre atómica |
| [crewai.md](crewai.md) | Update reconstruye registro sin `source/private`; con defaults los pierde | Update preserva metadatos (C10) — lección directa |

**Advertencia:** similitud ≠ diagnóstico; cerrado ≠ resuelto, abierto ≠ roto; nada se afirma vigente en upstream sin revalidación.

## 4. Prerrequisitos en VantaDB

Orden del central §4 (todo [Histórico], revalidar): (1) reemplazo L1 indivisible; (2) carrera RMW del checkpoint global; (3) blindaje LoCoMo (MEM0-05); (4) recovery/atomicidad (CRASH-01/TXN-01); (5) FIND-313.

**Alcance condicionado:** los prerequisitos condicionan al mecanismo que cada feature reutiliza — **no** bloquean toda escritura por el checkpoint global. Ej.: el ledger (C8) puede ser un nuevo registro atómico sin depender del manager global; solo C5 (cascadas de borrado) exige (1) antes que nada; certificación exige (3).

## 5. Que no trasladar

Productos completos/SaaS; LLM obligatorio en core o hot path (opcional; coste no medido); duplicados de sistemas existentes; índice/ANN de terceros; runners destructivos (`benchmark/RAG/src/pipeline.py:187-202` → `db.clear`); defaults permisivos (auth vacío, CORS `*`); contenido recuperado tratado como instrucción confiable; port source-to-source como bypass de licencia; exactly-once global como promesa.

## 6. Advertencias metodológicas

Evidencia estática fechada; ausencia = "no acreditado en la muestra", nunca censo; estrellas = filtro de selección, no calidad ni OSI; cerrado≠resuelto / abierto≠roto; ACK≠listo; read-error≠ausencia; licencia ≠ vulnerabilidad pero exige provenance+gate; claims publicados etiquetados; ningún número propio sin corrida; `.gitignore` cubre `docs/dev/research/` y se preserva para el resto; los 43 archivos de esta investigación fueron incorporados selectivamente (force-add revisado por el propietario).

## 7. Benchmarks

Sin metodología nueva: [METHOD.md](METHOD.md) (contrato de evidencia, protocolo común) y [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md) (definiciones métricas VantaDB/MemPalace/mem0, claims vs ejecución, gitlink `4b61c5d`). Reglas heredadas: memoria≠RAG; retrieval≠generado≠juzgado≠éxito-de-tarea; denominadores declarados; CSV por pregunta; datos desechables; claims externos no reproducidos; **sin rúbricas/prompts/umbrales nuevos aquí**.

## 8. Matriz de brechas

"Existente" no certifica paridad; "no acreditado" = no visto en la muestra, no ausencia probada. **Son deltas candidatos respecto a una muestra histórica — el estado actual de VantaDB no está certificado:**

- **Deltas a evaluar:** scope/navegación por subárbol+depth (C3); derivados materializados versionados (C5/C6); budget/cuotas explícitas (C7); ledger de entrega (C8); lifecycle observable (C9); watches incrementales (FIND-299 histórico); receipts de mutación (C13); evidencia code-memory canónica (C12); inspector de evidencia (C16).
- **No brechas (mecanismos propios confirmados):** híbrido BM25/HNSW/RRF, provenance, sessions, supersession, dedup con LLM opcional, auditoría de índice textual, auth non-loopback-gate, Studio existente.

## 9. Negativos y deuda propia

Todo [Histórico] — central §9: reemplazo L1 no indivisible (`l1_writer.rs`), checkpoint global RMW (`checkpoint.rs`), harness LoCoMo no apto para certificación (`eval_locomo.py`), falsos positivos descartados registrados (loopback-gate ya existía). **Revalidar contra el estado vigente antes de crear tareas** — el worktree dirty de la fase fue absorbido por commits ajenos.

## 10. Escritorio / UI

| Familia | UI/inspector observado (fuentes) | Lección para Studio existente |
|---|---|---|
| Apps/plataformas | [anything-llm.md](anything-llm.md), [dify.md](dify.md), [langflow.md](langflow.md), [flowise.md](flowise.md), [quivr.md](quivr.md), [khoj.md](khoj.md) — dashboards/flujos visuales | C16: onboarding con consentimiento, sin sobreescrituras |
| Frameworks agentes | [autogen.md](autogen.md), [crewai.md](crewai.md), [agno.md](agno.md), [openai-agents-python.md](openai-agents-python.md), [langgraph.md](langgraph.md) — tooling/tracing opcional | C11: adaptadores exponen capacidad diagnóstica declarada |
| DB/vector | [qdrant.md](qdrant.md), [milvus.md](milvus.md), [redis.md](redis.md), [tidb.md](tidb.md), [surrealdb.md](surrealdb.md), [meilisearch.md](meilisearch.md) — consolas admin | Receipts/estados C13 visibles, no en logs |
| Memoria | [mem0.md](mem0.md), [cognee.md](cognee.md), [graphiti.md](graphiti.md), [hindsight.md](hindsight.md), [supermemory.md](supermemory.md), [mempalace.md](mempalace.md), [tencentdb-agent-memory.md](tencentdb-agent-memory.md), [agentmemory.md](agentmemory.md) — mayoría API-first, dashboards parciales | Inspector read-only del ciclo de memoria (C16) |
| Code-memory/KG | [codegraph.md](codegraph.md), [graphify.md](graphify.md), [understand-anything.md](understand-anything.md), [codebase-memory-mcp.md](codebase-memory-mcp.md) — viewers de grafo | Evidencia code-memory con provenance (C12/C14) |
| OpenViking | central §10: WebStudio (código), Helper (anuncio, fuente no localizada), SaaS declarado | C16; estados: código/anuncio/SaaS/no localizado |

**Limitación declarada:** ideas de UI van a Studio existente; componentes/APIs concretos de integración quedan fuera del alcance de estas fichas.

## Apéndice — fuentes

37 fichas `.md` de este directorio + central [../../Backlog-OpenViking-VantaDB.md](../../Backlog-OpenViking-VantaDB.md) + [github-candidates.json](github-candidates.json) + [clone-manifest.json](clone-manifest.json) (snapshots UTC 2026-11-07; 37 elegidos + OV, 12 excluidos; no censo exhaustivo).
