---
title: "CONSOLIDATED — Investigación comparativa agent-memory (2026-11-07)"
kind: research
status: proposed
description: "Síntesis documental de 42 informes de memoria/contexto para agentes, con límites explícitos y OpenViking como referencia"
tags: [research, memory, consolidated]
---

# CONSOLIDATED — Investigación comparativa agent-memory (2026-11-07)

> **Estado — autoridad congelada del lead:** síntesis de **42 informes con autoría y revisión independiente**: **41 aceptados documentalmente con límites** y **Meilisearch entregado con restricción expresa, E3 incidencias NO acreditado** por prohibición de forge del usuario. No se suple ese eje con títulos, mirrors ni nuevas consultas. [Cobertura por proyecto](REVIEW-COVERAGE.md) y [estado de ampliación](EXPANSION-STATUS.md) conservan los límites; aceptación documental no aprueba features, implementación ni runtime.
> **Referencia inmutable:** motor/adapters **V** = `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; Studio **S** = `0ad38c542b39a589035b0e17fdedb4c09dca24b9`, rutas `feature-freez/desktop/...` del worktree separado. Son snapshots estáticos, no certificación del estado actual ni equivalencia con bytes dirty. `c561158`/dirty y el cierre inicial quedan **solo como históricos**.
> **Fuentes:** [contrato de ampliación](EXPANSION-CONTRACT.md), [referencia Vanta](VANTA-REFERENCE.md), [METHOD.md](METHOD.md), [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md), [inventario](README.md), informe central [OpenViking](../../Backlog-OpenViking-VantaDB.md) (referencia separada, no otro miembro de los 42). Reconfirmaciones T7: [memoria/código](reviews/corrections-memory-closure.md), [motores/RAG](reviews/corrections-engines-closure.md), [plataformas/frameworks](reviews/corrections-platforms-closure.md).
> **Alcance:** consolidación de esos cierres y fichas por mecanismo, sin reinvestigar upstream. Ningún test, benchmark, app o servicio ejecutado. Las lagunas por eje permanecen explícitas: no se afirma verificación total de todas las capacidades ni de los diez ejes de cada producto.

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

**Restricciones conservadas:** Rust embebido, `vanta-memory` y adaptadores+Studio separados; **no importar productos completos ni reemplazar HNSW/BM25/RRF**. Se seleccionan mecanismos para eventual adaptación, no código autorizado para portar; licencias source-available/`NOASSERTION`, componentes, datos y dependencias se discriminan por ficha.

**Orden de decisión (prioridad cualitativa, no números inventados):**

1. **Garantías y contratos + evaluación confiable** — C1, C2, C13, C14: antes de amplificar escritura ni certificar calidad.
2. **Identidad, derivados, scopes + assembler y ledger** — C3–C8.
3. **Ciclo de vida, experiencias, adaptadores, evidencia** — C9–C12, C15.
4. **UI diagnóstica read-only** — C16: puede avanzar en paralelo.

Reglas transversales: extender `ContextEngine::assemble_with_recall`, `cmd_audit_index`, `MemoryScheduler::run_pass`, writer/ingestors y Studio existentes; LLM/proveedores externos opcionales. El orden anterior no impone C1/C13 a lecturas puras, diagnóstico o validadores sin escritura; cada dependencia corresponde a la frontera reutilizada.

**Deltas de las cinco excepciones del propietario — integrados en C1–C16, sin crear C17:**

| Fuente y mecanismo discriminante | Relación y prerrequisito concreto | Rechazo / límite conservado |
|---|---|---|
| [Weaviate](weaviate.md), §§2/4/6/7: objeto, inverted, vector/cola y flush producen resultados por posición; filtro efectivo y compatibilidad índice/compresión | C13 observa ítem + error terminal; C7/C15 exponen soporte real; C4/C14 ligan publicación a generación; C9 solo para recursos separables; C16 agrega auditoría/backup ya existentes | No otro WAL, shard obligatorio por tenant, RAFT ni ANN importado; checksum no prueba snapshot coherente ni search-ready |
| [pgvector](pgvector.md), §§2/4/6/7: executor/MVCC, continuación ANN y mantenimiento son fronteras distintas; pocos hits no vacíos pueden agotar candidatos | C7 informa cobertura/parada sobre filtros actuales; C15 separa dominio/precisión/codec; C9/C16 observan rebuild y retención; C13 solo si hay evidencia por revisión de proyección Vanta | No PostgreSQL/PGXS ni VACUUM como ready/secure erase; no exigir C13 a capabilities o lecturas; no prometer continuación donde `VecIndex::search` solo devuelve hits |
| [Letta](letta.md), §§0/2/4/5: clon landing; complemento Letta Code fijado separa committed, sync, compilado, recall local/remoto y uso | C3–C5/C14: validación/revisión de fuente; C7/C8: presupuesto, cobertura y entrega; C9: repair cercado por intento; C16 muestra fases. Reusar protección estructural, assembler y scheduler | No V1/archive, Git como primario ni otro runtime; fetch autenticado actual y fixes reportados no prueban resolución global; compilado no acredita entrega |
| [LanceDB](lancedb.md), §§0–2/4/5: handle/version, commit de valores, stamp posterior, job específico y cobertura de índice no son equivalentes | C4/C7: lectura+detalle coherentes y proyección sin fingir streaming acotado; C5/C14: definición+origen y reparación por intención; C13: outcome desconocido; C9/C16: objetivo de readiness fijo | No Lance/Arrow/DataFusion paralelos, branches por sesión ni otro RRF; `verified_version=None` no es fence; job done no cubre escrituras posteriores; dependencias por ficha, no globales |
| [Chroma](chroma.md), §§1/3/5/6: commit de log, materialización/overlay, checkpoint, registro de generación y retención separados | C13 discrimina dominios; C5/C6/C14 preservan tombstones/revisión; C7 explica máscaras/plan; C15 declara backend/ownership; C9/C16 usan rebuild/auditoría existentes | No wal3/control plane/ONNX obligatorio ni segundo scheduler; flush no publica todo el catálogo, cleanup no es borrado seguro; TUI real no equivale a desktop/web |

## 2. Candidatos unificados

IDs **propuestos**, no filas oficiales de backlog ni specs aprobadas. Las fichas fuente conservan alternativas, ownership, persistencia/migración, rollback y casos adversos; aquí se relacionan sus deltas sin pegarlas. Los DTO nuevos son **vistas propuestas**, no una API estable implementada ni una firma única impuesta a todas las fuentes. Se distinguen de tipos reales como `WriteReceipt`, `RecallScope` e `IntegratedContext`.

### C1 — Garantías propias por frontera: L1, helper RMW, cursor L0 y proyecciones/WAL

- **Prioridad:** alta en rutas de mutación. **Fuentes:** central histórico §4/§9; [Mem0](mem0.md) §9, [LanceDB](lancedb.md) §4/§9, [TencentDB](tencentdb-agent-memory.md) ficha C, [pgvector](pgvector.md) §4.5, [Chroma](chroma.md) §3.
- **Seams en V:** [l1_writer.rs][v-l1] `plan_write/persist_planned/apply_dedup_batch`; [checkpoint.rs][v-checkpoint] `CheckpointManager::mutate`; [l0_recorder.rs][v-l0] `record_turn/read_cursor/write_cursor`; [memory.rs][v-write] `put_batch_inner/put_certified`, [wal.rs][v-wal] `WalWriter::sync`. No son el mismo recurso ni error.
- **Hechos estáticos:** plan L1 no es persistencia y el batch conserva delete-before-put con fallo parcial documentado; `mutate` separa read/modify/write, pero concurrencia efectiva exige caller; L0 escribe mensajes antes del cursor y ya tiene tie-break. `TaskCheckpointManager` maneja step/partial en otro namespace.
- **Acciones propuestas:** acreditar reemplazo indivisible antes de cascadas que lo exijan; discriminar intercalación/caller del helper RMW y del cursor por sesión por separado; mapear barrera durable y readiness por proyección. FIND-221/TXN-01/CRASH-01 quedan referencias históricas distintas.
- **FMEA:** fallo intermedio de reemplazo → efecto parcial; RMW concurrente alcanzable → posible lost-update; error posterior al primario → outcome incierto. No se reprodujo carrera, pérdida, exploit ni crash; timeout no prueba rollback.
- **Aceptación futura/dependencias:** validación adversa específica de cada ruta, sin inferir atomicidad de un nombre batch ni mezclar checkpoint/WAL. C8 puede usar otro registro atómico; lectura pura y diagnóstico no quedan bloqueados universalmente por C1.

### C2 — Evaluadores confiables: dataset real explícito, backend real, holdout congelado

- **Prioridad:** alta (primero). **Fuentes:** central §7, [mem0.md](mem0.md) §7 (submódulo a gitlink `4b61c5d`), [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md), [graphiti.md](graphiti.md) (contexto del juez — §3).
- **Seam Vanta:** `evals/*` existentes (`eval_locomo`, `eval_longmemeval`, `memory_harness`).
- **Acciones propuestas, no ejecutadas:** extender los evaluadores existentes para distinguir dataset/backend reales de sintético/fake, conservar identidad de datos y registrar intentadas/completadas/juzgadas/errores/abstenciones. Preservar la definición de cada métrica upstream y declarar su población efectiva; registrar fallos operativos por separado, sin alterar silenciosamente su fórmula para hacer comparables scores diferentes.
- **Dependencias:** [METHOD](METHOD.md), [notas de benchmarks](BENCHMARK-NOTES.md) y fichas contrastadas. MEM0-05/EVAL-02 son referencias históricas, no tareas nuevas ni confirmación automática de vigencia. Instancias desechables; ningún reset destructivo sobre datos compartidos.
- **FMEA:** error de búsqueda convertido en vacío y posible PASS de abstención; corpus preingerido o reutilizado; tareas sin directorio ausentes del censo; porcentaje de evaluadores simples presentado como QA. Detectar por el recorrido cliente/runner/agregado y conservar la distinción de pistas, procedencia y errores.
- **Aceptación futura:** trazabilidad de población, backend, datos y fallos sin convertir retrieval en calidad de respuesta. No se crea ni ejecuta un harness, juez, prompt, umbral, muestreo o configuración en esta investigación; no se afirma mejora medida.

### C3 — Scope jerárquico sobre `RecallScope`, `RecallConfig`, `InjectionPolicy` y filtros existentes

- **Prioridad:** alta. **Fuentes:** central F1, [Hindsight](hindsight.md), [Understand Anything](understand-anything.md), [Khoj](khoj.md), [Letta](letta.md) B/F.
- **Seams en V:** [auto_recall.rs][v-recall] `perform_auto_recall_governed`, `RecallScope/RecallConfig/InjectionPolicy`; [vector.rs][v-vector] `vector_memory_search/exact_scan`; [l1_writer.rs][v-l1] `default_tenancy/plan_write`. `MemoryScope` era nombre conceptual, **no tipo implementado acreditado**.
- **Acciones propuestas:** ampliar compilación del scope/filtro con subárbol+depth/navegación si el caso lo necesita; canonicalizar paths y conservar intersección autorizada. Ya hay límites por segmento y marcado tainted: no inventar colisión sess-1/sess-12.
- **Dependencias:** C4 solo para identidad/revisión de navegación compuesta; host autenticado construye política y revalida al hidratar/entregar. La policy recibida y el assembler puro no autentican al actor; cadena completa Quivr sigue no acreditada.
- **FMEA/aceptación futura:** path no canónico, tenant cruzado, revocación y operador no soportado no amplían el acceso; fallo de lectura no se representa como ausencia. MEMG-25 es spec histórica, no enum ni autorización actual.

### C4 — Identidad de contexto opaca + revisiones de fuente

- **Prioridad:** alta. **Fuentes:** central F1, [Graphify](graphify.md), [CodeGraph](codegraph.md), [LanceDB](lancedb.md) A/B, [Letta](letta.md) A.
- **Seams en V:** [record.rs][v-input] `MemoryInput` con `derived_from/confidence_class`; [l1_writer.rs][v-l1] `plan_write/persist_planned`; [code_index.rs][v-code-index] `index_repo`. Ya hay claves/provenance/versiones: no añadir otra identidad física por analogía.
- **Acciones:** `ContextId` opaco estable; `ContextObject { id, source_ref, parent: Option<ContextId>, revision, kind, scope }` (revisión/relaciones fuera de la identidad); `resolve(id)` sin duplicar payload.
- **Dependencias:** definir mapeo al registro actual; MEMG-25/FIND-320 históricos. Lectura+detalle puede detectar conflicto de revisión sin exigir MVCC/branches nuevos ni C13.
- **FMEA:** rename/move actualiza relaciones, no identidad; revisión nueva no cambia identidad; padre mutable jamás dentro del id.
- **Aceptación:** rename preserva provenance; nueva revisión no es nueva identidad.

### C5 — Preservación de raw + derivados versionados

- **Prioridad:** alta. **Fuentes:** central F2, [Cognee](cognee.md), [MemPalace](mempalace.md), [GraphRAG](graphrag.md), [LightRAG](light-rag.md), [LanceDB](lancedb.md) B/G, [Chroma](chroma.md) B/D, [PageIndex](pageindex.md).
- **Seams en V:** [ingest/merge.rs][v-merge] `CandidatePage/commit`, [worker.rs][v-worker] `ingest_body`, [WikiStore::put_page][v-wiki-page] y el struct [MemoryInput][v-input], con `confidence_class` y `derived_from` para linaje de padres del mismo namespace. Wiki, registro y relación no tienen un commit común acreditado.
- **Acciones:** abstract/overview materializados por unidad con `source_revision`+`generator_version`+`state`; generación async; fuente canónica nunca borrada por fallo del derivado. Comunidades/proyecciones GraphRAG son opcionales: error de workflow no compensa etapas previas y Parquet+vector no prueban commit conjunto.
- **Dependencias:** C4 para fuente/revisión; C1 para reemplazo L1/cascada destructiva que lo use, no para leer raw o validar una proyección. La propuesta no prueba que campos de provenance ya hagan CAS.
- **FMEA:** derivado viejo sobreescribe uno nuevo; update durante generación → descarte de stale; fallo borra fuente (prohibido).
- **Aceptación:** publicación condicionada a revisión vigente; fuente sobrevive a fallo de derivado.

### C6 — Consolidación/publicación condicionada de derivados

- **Prioridad:** alta. **Fuentes:** central F2/F5, [Hindsight](hindsight.md), [Agno](agno.md), [CrewAI](crewai.md), [Chroma](chroma.md) B/D, [LanceDB](lancedb.md) G.
- **Seams en V:** [l1_writer.rs][v-l1] `plan_write/persist_planned/apply_dedup_batch`, dedup/supersession existentes; worker.rs: [`execute`][v-worker-run]/[`ingest_body`][v-worker] y [merge.rs][v-merge] `commit` para derivados wiki.
- **Acciones:** merge declarativo con enum cerrado (patch/replace/sum/immutable/link); precedencia explícita (autoridad, revisión, política, desempate estable); guarda de replay para sum; rechazo de mutaciones desconocidas/incompatibles/protegidas.
- **Dependencias:** C1 para mutación indivisible que la operación prometa; C4/C5 para publicación por revisión; C9 solo si diferida. Planificador/validador puro no exige C13 ni otro scheduler.
- **FMEA:** replay de sum duplica; worker obsoleto limpia trabajo ajeno; fuentes vivas no sustituyen metadata actual; reconstruir registro puede perder privacidad/provenance. Hindsight ya tiene guard de fuentes y lock de tags: delta no parte de ausencia upstream.
- **Aceptación futura:** operación protegida/target ajeno rechazados, replay en su alcance, conflicto explícito; aplicado/skipped/unknown no se funden en éxito. Repair por intención solo si se justifica; un UUID o `run_id` no sustituyen comparación+publicación.

### C7 — Retrieval y ensamblado existentes: cobertura, descartes, render y presupuesto efectivo

- **Prioridad:** alta. **Fuentes:** central F3, [LightRAG](light-rag.md), [Mem0](mem0.md), [Letta](letta.md) D/F/G, [LanceDB](lancedb.md) D/E, [pgvector](pgvector.md) §4.1, [Weaviate](weaviate.md) §4.3, [Chroma](chroma.md) C, [Faiss](faiss.md).
- **Seams en V:** [engine.rs][v-context] `assemble/assemble_inner/assemble_with_recall`, `IntegratedContext/CompactionReport`; [auto_recall.rs][v-recall] `apply_recall_budget`; [vector.rs][v-vector] `vector_memory_search/exact_scan`, [explain.rs][v-explain] `explain_memory_search`. Presupuesto, compactación, filtros y explain **ya existen**.
- **Delta propuesto:** propagar motivos de descarte, procedencia/unidad de score, ruta efectiva y cobertura/parada de candidatos a render y C8; declarar reservas/cuotas que el caller realmente aplica. Prefijo protegido puede **exceder el presupuesto**; recall por caracteres y estimación de tokens no garantizan cabida exacta.
- **Dependencias:** C4 para citas/revisión e hidratación compuesta; C15 para soporte de filtros/continuación; política de derivados y supersession actuales. Lectura/proyección en una respuesta no exige C13, nuevo ANN ni nuevo assembler; `core_search` es opt-in en la muestra.
- **FMEA:** pocos hits no vacíos no prueban exhaustividad; máscara Bloom requiere filtro exacto final; stream puede materializar ambas ramas; score ≠ probabilidad; eco de recall no es fuente independiente. No se importan pesos ni políticas de ranking.
- **Aceptación futura:** descartes/error/ausencia/parcialidad se distinguen; Unicode, tool call/result, fuente cambiada y exceso protegido se reportan sin prometer límite absoluto. Renderizado no equivale a entrega confirmada por el host.

### C8 — Ledger `Selected|Rendered|Delivered` por sesión

- **Prioridad:** alta. **Fuentes:** central F4, [Mem0](mem0.md), [Agentmemory](agentmemory.md), [Supermemory](supermemory.md), [Letta](letta.md) A/E, [RAGFlow](ragflow.md) B.
- **Seams en V:** selección/render en [ContextEngine::assemble_with_recall][v-context] y [perform_auto_recall_governed][v-recall]; store/registro existente para estado dedicado por sesión. `core/memory_generation_log/` acredita generación, **no entrega al modelo**. `L0Recorder::write_cursor` sigue captura L0, no checkpoint de tarea ni ledger de delivery.
- **Acciones propuestas:** entradas `{context_id, revision, state, turn}` con actualización atómica en el recurso elegido; `confirm_delivery` idempotente **solo desde evidencia del host que entrega**. Sin confirmación queda Rendered/Unknown, no Delivered por estar en genlog o compilado.
- **Dependencias:** C4 y puntos C7 de selección/render; autorización/revisión revalidadas al entregar. No reutilizar RMW del JSON global como CAS; la carrera global sigue hipótesis pendiente de caller/reproducción, no prerrequisito universal del ledger nuevo.
- **FMEA/aceptación futura:** cancel-before-delivery no suprime; nueva revisión no hereda supresión anterior; dos recalls concurrentes conservan confirmaciones y repetir una no duplica estado. Alcance idempotente local, **no exactly-once de red**, replay ni retirada de lo ya enviado.

### C9 — Lifecycle de ingesta observable + cancelación

- **Prioridad:** alta. **Fuentes:** central F7, [Serviette](serviette.md), [AnythingLLM](anything-llm.md), [Dify](dify.md) C, [Langflow](langflow.md) E, [LanceDB](lancedb.md) F/G, [pgvector](pgvector.md) §4.3.
- **Seams en V:** ingest/worker.rs: [`begin/execute`][v-worker-run]/[`ingest_body`][v-worker], [merge.rs][v-merge] `commit`, [WikiStore::put_page][v-wiki-page]; [WikiStore][v-wiki-terminal] `request_ingest/begin_processing/complete/fail`; [scheduler.rs][v-scheduler] `run_pass` y [TaskCheckpointManager][v-task] `begin/load/advance/complete/fail`.
- **Delta propuesto:** proyectar captura/extracción/indexación/readiness con revisión/generación, resultados por ítem y terminal, errores y reintentos acotados. `run_id` protege transición terminal de WikiStore; `put_page` **no recibe run_id**, por tanto no acredita fencing/CAS por página ni publicación global.
- **Dependencias:** C1 solo para reemplazos que lo requieran; C4/C14 para resultado ligado a fuente; C13 cuando se afirme estado de mutación. Observación read-only de fases no requiere todos esos contratos ni otra cola. Cola/timers/leases locales no son durables por existir checkpoints.
- **FMEA:** deferral de lock ≠ fallo; página anterior puede quedar escrita ante error posterior; cancelar espera no cancela owner. Tras dispatch sin evidencia terminal, **Unknown**: no prometer «sin efectos», rollback ni frenar factura/proveedor ya invocado.
- **Aceptación futura:** fases completadas/parciales/failed/interrupted se conservan en el reporte; ready se refiere al objetivo observado, no solo índice homónimo existente. Reanudación/persistencia exigen contrato y comprobación específicos; no están acreditadas por el panel ni por este cierre.

### C10 — Experiencias/propuestas aprobables

- **Prioridad:** media. **Fuentes:** central F8, [Hindsight](hindsight.md), [Supermemory](supermemory.md), [CrewAI](crewai.md), [LangChain](langchain.md) C.
- **Seams en V:** reflexión/personas/escenas/skills existentes ([Hindsight, V3](hindsight.md)); [l1_writer.rs][v-l1] `plan_write/persist_planned` y [CaptureApprovalQueue][v-approval]. La cola aprueba memorias extraídas, **no ejecución de tools** ni toda promoción experiencia→skill.
- **Acciones:** `Experience { situation, objective, preconditions, actions, outcome, evidence_refs, evidence_status: Claimed|Observed|Verified, expiry }`; promoción declarativa experiencia→skill con evidencia + aprobación humana configurada; update preserva metadatos (`source`, `private`, refs).
- **Dependencias:** preservar tenancy, scopes, privacy/provenance actuales y actor autorizado; C1 si hay reemplazo, C8 solo para evidencia de entrega, C9 si publicación diferida. FIND-290/295/288 son antecedentes, no nuevas tareas.
- **FMEA/aceptación futura:** claim ≠ verificado; contenido recuperado no autoriza ejecución. Update conserva metadatos heredados salvo cambio explícito autorizado; `source/private` identifica el mecanismo CrewAI, no campos nuevos supuestamente ausentes en Vanta. Revocación/revisión y destino efectivo se revalidan antes de promoción o envío externo; sin evidencia no marcar `Verified`.

### C11 — Adaptadores de integración versionados (frameworks)

- **Prioridad:** media. **Fuentes:** [LangGraph](langgraph.md), [LangChain](langchain.md), [LlamaIndex](llama-index.md), [CrewAI](crewai.md), [Agno](agno.md), [AutoGen](autogen.md), [OpenAI Agents Python](openai-agents-python.md).
- **Seams en V:** `integrations/langchain/vantadb_langchain/vectorstore.py::add_texts/add_documents/delete`, `checkpointer.py::VantaDBCheckpointer`, `store.py::VantaDBStore.batch/_put`; `integrations/llamaindex/vantadb_llamaindex/vectorstore.py::query`; `integrations/openai/vantadb_openai/vectorstore.py::VantaDBOpenAI`. [Contraste de adapters](reviews/corrections-platforms-closure.md), [LlamaIndex](reviews/corrections-engines-closure.md#llamaindex).
- **Acciones propuestas:** declarar versión de framework/adapter/binding y capacidades efectivas de filtro, scope, mutación, checkpoint y cancelación. Reutilizar adaptadores actuales antes de añadir otros; Python es capa de integración, **no todos esos paquetes pertenecen al motor** ni tienen paridad automática con Rust/Node/TS/MCP.
- **Dependencias:** C7 donde se traduzcan score/cobertura, C8 si se confirma entrega, C13 si se exponen mutaciones; lectura simple no exige todos. Ningún runtime de framework o llamada LLM obligatoria en core.
- **FMEA/aceptación futura:** IDs inestables, puts/deletes individuales y fallo tardío conservan parcialidad; filtro ignorado/desalineación del adapter LlamaIndex no niegan filtros del motor. Resume reautoriza checkpoint/copia retenida antes de decode/entrega; no exploit ni borrado universal acreditados. Unsupported falla explícito y combinación de versiones se valida antes de prometer conformidad.

### C12 — `CodeEvidenceImport`: contrato canónico único de evidencia code-memory

- **Prioridad:** media. **Fuentes:** [codebase-memory-mcp.md](codebase-memory-mcp.md), [graphify.md](graphify.md), [understand-anything.md](understand-anything.md), [codegraph.md](codegraph.md).
- **Seams en V:** [code_index.rs][v-code-index] `index_repo` y [code.rs][v-code] `code_search/code_explore/code_impact`; [MemoryInput][v-input] con linaje Derived. Ya hay indexación/grafo propios; `add_edge` no acredita metadata arbitraria ni transacción de todos los extremos.
- **Contrato (hipotético):** `identity: {repo, revision, producer, schema_version, generation}`; `refs: {source, target}` con `relation`+`direction`; `evidence: {origin: Ast|Heuristic|Model|Unknown, span?, hash?, reported_weight?}` — `reported_weight` opcional, **peso ≠ confianza**. Cada productor mapea al mismo contrato — no tres productos de memoria.
- **Acciones propuestas:** adapter por productor; invalidación por fuente/gramática/productor; preservar schema, snapshot y cobertura parcial. Codebase Memory E (`commit_baselines` tras `index_fn==0`) aporta ACK por trabajo aplicado, no prueba persistencia/recovery end-to-end.
- **Dependencias:** E es **integración nueva no acreditada**: caller/job code-index, ACK y persistencia no están conectados por la mera existencia de `MemoryScheduler::run_pass`, `TaskKind` L1/L2/L3/Dream/Flush o TaskCheckpoint. Definir ese enlace antes de prometer auto-refresh; no crear otro daemon, parsers ni scheduler.
- **FMEA/aceptación futura:** mención textual no es llamada AST; origen desconocido queda `Unknown`; hash/fingerprint no acredita cobertura; cambio durante trabajo conserva pendiente posterior. Consulta pura no requiere C13; publicación requiere revisión/ownership apropiados.
- **Non-goals/límite:** no fusionar con memoria conversacional ni portar productos. La coherencia UI residual de Codebase Memory se limita a comentario/imports frente a flujo React no acreditado; no se presenta como navegación implementada ([closure](reviews/corrections-memory-closure.md#codebase-memory-mcp)).

### C13 — `MutationReceipt`: recibo de mutación multialmacén

- **Prioridad:** alta en mutaciones observadas. **Fuentes:** [Qdrant](qdrant.md), [Milvus](milvus.md), [Meilisearch](meilisearch.md) (E3 restringido), [Redis](redis.md), [Mem0](mem0.md), [Weaviate](weaviate.md) §4.1, [pgvector](pgvector.md) §4.5, [Chroma](chroma.md) A, [LanceDB](lancedb.md) C/F, [Quivr](quivr.md) A.
- **Seams en V:** memory.rs: [`put_one`][v-put-one]/[`put_certified/put_batch_inner`][v-write]; attestation.rs: [`WriteReceipt`][v-attest]/[`build_write_receipt`][v-attest-builder]; [ops.rs][v-hnsw] `try_push_pending_hnsw/drain_hnsw_batch_locked`; [wal.rs][v-wal] `WalWriter::sync`; `replace_derived_indexes` y `Embedded::rebuild_index` existentes.
- **Contrato transversal propuesto:** cuatro ejes **independientes**: admisión/aceptación, aplicación, durabilidad con dominio/barrera, readiness de cada proyección. No escalera `accepted→applied→durable→ready`; aceptación conocida puede coexistir con aplicación Unknown, o primario synced con ANN pendiente. Cada evidencia identifica `(namespace,key,version)` y revisión/generación de proyección; posición WAL, versión L1 y checkpoint de tarea no son intercambiables.
- **Vistas, no API única:** `BatchMutationReport/MutationReceipt` de Weaviate, `ProjectionEvidence` de pgvector, observaciones Chroma y receipts de etapa/fuente de LlamaIndex/Quivr son propuestas de alcance distinto. Sus DTO no se homologan imponiendo aquí una firma estable ni se adjudican al código actual.
- **Delta/errores:** conservar progreso por ítem/ordinal, causa inicial, fallos posteriores y error terminal del lote. Un `Err` tardío no puede ocultar éxitos ya aplicados sin reporte recuperable; `Ok(report)` tampoco equivale a éxito de todos. La forma concreta de transporte queda en cada ficha. `Unknown/NotAssessed` si no hay prueba por revisión; cola vacía o ID presente no prueban Applied de esa versión.
- **Existente frente a propuesto:** `WriteReceipt` **ya existe**: binding SHA-256 de contenido/cadena y verificación/relectura en su alcance; no firma digital, identidad de firmante, commit conjunto ni crash guarantee automática. Preservarlo, no reemplazarlo ni llamar inexistentes a todos los recibos.
- **Dependencias/FMEA/aceptación futura:** trazar commit/sync/caller de la ruta C1 pertinente; C4/C14 para generación cuando proceda; no requisito universal de C7/C15/C16 read-only. Timeout/cancel postdispatch conserva Unknown o commit conocido; restart revalida evidencia volátil. No segundo WAL, fsync por índice, 2PC ni exactly-once global; retry idempotente solo en el alcance acreditado.

### C14 — Frescura y evidencia: derivado↔fuente por revisión

- **Prioridad:** alta. **Fuentes:** central F6, [Graphify](graphify.md), [MemPalace](mempalace.md), [GraphRAG](graphrag.md), [LanceDB](lancedb.md) B/F/G, [Weaviate](weaviate.md) §4.5/§11, [Chroma](chroma.md) D/E, [SurrealDB](surrealdb.md) A.
- **Seams en V:** [cli_handlers/index.rs][v-audit] `cmd_audit_index`, [impl_index.rs][v-derived] `ensure_derived_indexes_current_with/replace_derived_indexes`, [admin.rs][v-admin] `Embedded::rebuild_index`; worker.rs: [`ingest_body`][v-worker]/[`execute`][v-worker-run] para derivados wiki. Auditoría/rebuild y estados **ya existen**.
- **Acciones:** detectar missing/orphan/stale/inválido por revisión; diagnose separado de repair; jobs idempotentes guardados por revisión vigente; `error≠ausencia`.
- **Dependencias:** C4/C5 para linaje; C1/C13 solo si se promete publicación/reparación mutante. Diagnóstico puede empezar sin CAS ni nuevo manifiesto; mantenimiento offline existente es alternativa a swap concurrente.
- **FMEA/aceptación futura:** conteos/schema iguales no prueban identidad del conjunto; eso no demuestra corrupción frente a audit deep. Stamp fallido después de valores committed queda pendiente, no rollback; repair no limpia revisiones ajenas ni borra fuente. Missing/orphan/stale/error se distinguen, generaciones y retención tienen alcance declarado.

### C15 — Capacidades de backend/embedding declaradas por entorno

- **Prioridad:** media. **Fuentes:** [Qdrant](qdrant.md), [Milvus](milvus.md), [Redis](redis.md), [TiDB](tidb.md), [Faiss](faiss.md), [Weaviate](weaviate.md) §4.4, [pgvector](pgvector.md) §4.2, [Chroma](chroma.md) F, [Dify](dify.md) E, [AnythingLLM](anything-llm.md) E, central OV.
- **Seams en V:** `src/sdk/serialization/mod.rs::validate_vector_dimension`, [admin.rs][v-admin] `capabilities`, [vector.rs][v-vector] `vector_memory_search`; `core_embedding_hook/local_embedding_hook` del writer ([Dify](dify.md) E), índices/cuantización existentes. Consulta de proceso no acredita manifest por colección; namespace no implica índice físico independiente.
- **Acciones propuestas:** declarar soporte real por backend/binding/entorno y separar identidad/revisión del modelo, dominio dense/sparse/binary, precisión, dimensión/nnz, métrica/normalización y codec/compresión. No se incorporan formatos ni combinaciones porque existan upstream; validación dimensional ya existe.
- **Dependencias:** C7 consume soporte/semántica de score; C9 solo para cambio incompatible que exija mantenimiento; no C13 para consultar capabilities. Destino/consentimiento se verifica en host/adapters/conexión existente justo antes de egress, ligado a configuración/scope efectivos; no dentro de BM25/HNSW ni desde texto del modelo.
- **FMEA/aceptación futura:** igual dimensión no prueba mismo embedding-space; batching no cambia asociación input/vector; cambio de endpoint/finalidad invalida decisión previa según política aprobada. Capacidad/consentimiento por destino no acreditados en la muestra no significan ausencia global de privacy/auth; preservar provenance/scopes existentes y rechazar Unsupported sin fallback a otro destino.

### C16 — Inspector read-only + onboarding en Studio existente

- **Prioridad:** media (paralelo). **Fuentes:** central F10, [AnythingLLM](anything-llm.md), [Dify](dify.md), [Khoj](khoj.md), las cinco excepciones y sus distinciones de UI (§10).
- **Seams en S:** [MemoryLens.tsx][s-memory] (genlog/anchors/sesión), [ActivityPanel.tsx][s-activity] (auditEvents, paginación/error/no configurado), [RetrievalLens.tsx][s-retrieval] (search con explain, get/vecinos/audit), bajo `feature-freez/desktop/src/components/`. Extender bridge/conexiones actuales, no crear otra app.
- **Acciones:** timeline captura→extracción→recall→inyección con evidencia real; detección de integraciones con consentimiento + preview/repair sin sobreescrituras; sync selectivo; `installed≠functioning`.
- **Dependencias:** C7/C8/C9/C13 solo para evidencia nueva que la pantalla afirme; auditoría/consulta existente puede avanzar sola. Detalle/reparación/restore requieren autorización/revisión propias, nunca permisos concedidos por UI.
- **FMEA/aceptación futura:** error/stale/no configurado no se convierten en vacío o Healthy; genlog no marca Delivered y cancel visual no revierte commit. Onboarding consiente destino/scope efectivo sin exponer tokens ni sobrescribir; progreso/ítems parciales conservan unidad y generación reales.
- **Limitación:** `JobChip/jobs` localizados como `??` en worktree no están acreditados por S; jobs/cancelación/restart durables y nuevos DTO/bridge siguen **propuestos**, no UI shipped. Los componentes reales son seams de extensión, no integración end-to-end lista ni runtime certificado.

## 3. Issues y defectos transferibles

El central §3 conserva el dossier **histórico** OpenViking: ledger concurrente #7170, delete-before-replace #7099, configuración #6923, read-error #7132 y stale #5873 tienen mecanismos distintos. Sus IDs no diagnostican automáticamente Vanta ni acreditan presencia actual/fix upstream. Los informes ampliados y sus closures son la autoridad para el alcance estático nuevo.

**Muestra por mecanismo — fuente congelada, reporte e hipótesis no se confunden:**

| Fuente | Mecanismo y evidencia delimitada | Transferencia / límite |
|---|---|---|
| [Agno](agno.md), [closure A-1](reviews/corrections-platforms-closure.md#agno) | Tool `update_memory` construye UserMemory y llama `upsert_user_memory`; SQLite resuelve por memory_id sin lookup/condición de propietario en ese camino directo; clear+upserts es otra frontera | C3/C6/C11: DB compartida + ID ajeno conocido + ausencia de wrapper autorizado son condiciones, no exploit REST ni fallo de todos los drivers |
| [CrewAI](crewai.md) | Update de consolidación reconstruye registro sin `source/private`, cuyos defaults cambian esos valores | C6/C10 preservan ownership/privacy/provenance; no afirmar que toda memoria Vanta pierda esos metadatos |
| [Hindsight](hindsight.md), [closure H-1](reviews/corrections-memory-closure.md#hindsight) | Código fijado revalida fuentes y bloquea/lee tags vigentes; fixes/cancelación tienen seguimiento reportado, no garantía universal | C6 distingue vigencia de fuentes de metadata editada; C9 cancelación por etapa/proveedor, no supuesto de ausencia upstream |
| [Weaviate](weaviate.md) §2/§4 | Objeto se publica antes de ramas derivadas; batch/flush asignan errores por posición y pueden sobrescribir una causa previa en la implementación leída | C13 conserva progreso, primer fallo y terminal; no deducir mismo fallo durable en Vanta ni resolverlo creando más logs |
| [pgvector](pgvector.md) §4.1, [Faiss](faiss.md) | Exploración/candidatos/stop y filtro efectivo determinan cobertura; discusión #259/#678 no equivale a fuga ni recall completo | C7 informa parcialidad y soporte; no trasladar planner SQL ni llamar ausentes a filtros Vanta |
| [LanceDB](lancedb.md) §3/§1 | #4844 reporta merge sobre handle pinned; el camino remoto fijado espera job ID si lo recibe y se solicita espera, corrigiendo la atribución indiscriminada de #4402 | C4/C13/C14 distinguen revisión, intención, job y cobertura; fallback por nombre es menos fuerte, no fix/release universal |
| [Chroma](chroma.md) §3/§4 | Materializador/overlay excluyen versiones compactadas modificadas; #7946 tiene límites de caché/formato/ownership descritos | C5/C14 extienden auditoría y orden propios; no inferir misma caché, disparador RLIMIT ni corrupción Vanta |
| [Dify](dify.md), [closure DIFY-1](reviews/corrections-platforms-closure.md#dify) | `worker::{begin,execute,ingest_body}` llama `WikiStore::put_page`; terminal run_id no entra en cada write | C9/C14: publicación por revisión es delta, no CAS existente ni bug concurrente demostrado |
| [LlamaIndex](llama-index.md), [closure](reviews/corrections-engines-closure.md#llamaindex) | Adapter Vanta tiene caminos de query/postfiltro/delete cuya fidelidad y alineación se delimitan en §0.8/§9 | C11 revisa adapter existente; no equivale a que el motor carezca de filtros o transacciones |
| [Meilisearch](meilisearch.md), [closure](reviews/corrections-platforms-closure.md#meilisearch) | E3 incidencias **no acreditado por prohibición expresa**; otras lecturas estáticas siguen acotadas a su ficha | C13/C16 usan solo evidencia permitida; no forge, mirrors, títulos como cuerpo ni recomendación de modificar upstream |

**Advertencia:** no hay nueva investigación de issues aquí. Estado/comentario/PR son observaciones vivas separadas del pin; cerrado no prueba fix/merge/release y abierto no prueba defecto de todas las versiones. Tests leídos acreditan casos escritos, no ejecución.

## 4. Prerrequisitos en VantaDB

Prerrequisitos **por operación en V**, no un orden universal heredado del cierre inicial:

| Caso | Reutilización y comprobación previa | Qué no se bloquea / no se acredita |
|---|---|---|
| Reemplazo/consolidación L1 C1/C5/C6 | `plan_write→persist_planned/apply_dedup_batch`; delimitar delete/put y efectos posteriores con fallo parcial documentado | Plan/validación/lectura raw no requieren volver atómico todo el motor; el nombre batch no prueba indivisibilidad |
| Ledger C8, captura y progreso | Actualización atómica del estado de entrega elegido; cursor de `L0Recorder` y `TaskCheckpointManager` se analizan en sus callers/namespaces respectivos | Helper global RMW no es CAS ni fallo demostrado en todos los callers; no bloquea todo read/write |
| Publicación wiki/derivados C5/C9/C14 | `ingest_body→merge::commit→WikiStore::put_page`, luego complete/fail con run_id; revalidar fuente/actor en frontera mutante | run_id terminal no cerca páginas; fuente/relación/wiki no forman commit conjunto por existir `CommitReport` |
| Proyección de mutación C13 | `put_batch_inner`, `WalWriter::sync`, HNSW pendiente y rebuild: identificar revisión y barrera que realmente se observa | `WriteReceipt` no reemplaza cuatro ejes; no exigir C13 a consulta de capabilities, explain o auditoría read-only |
| Recall/hidratación C3/C4/C7/C11 | Policy autenticada del host, filtros exactos y revisión al acceder/resume/entregar; conservar prefijo protegido y budgets actuales | Assembler/policy no autentican por sí solos; Quivr principal→policy→resolver→hydrate y permisos de todos los transports siguen no acreditados |
| Código C12 E | Definir caller code-index y enlace publicación→ACK/pending/reconciliación dentro de coordinador existente | `TaskKind` de memoria no es job code-index ni prueba cola persistente; consulta manual puede permanecer independiente |
| Destino/experiencias C10/C15/C16 | Conservar tenancy, privacidad, provenance y aprobación de captura existentes; comprobar decisión del host por destino/scope/config al transmitir | Captura aprobada no autoriza tool, promoción ni nuevo endpoint; falta de recibo en una pantalla no prueba ausencia global de consentimiento |
| Lectura de resultados C2 | [Revalidación estática propia](BENCHMARK-NOTES.md#6-revalidacion-estatica-de-locomo-durante-la-ampliacion) y ficha C2, sin alterarlas aquí | MEM0-05/EVAL-02, FIND-313 y CRASH-01/TXN-01 son referencias históricas, no tareas actuales ni fallos de todas las rutas |

## 5. Que no trasladar

Productos completos/SaaS, otro runtime/daemon/parser/scheduler/WAL/IAM o índice ANN; PostgreSQL/PGXS, Lance/Arrow/DataFusion, Git primario, control plane Chroma/Weaviate; LLM/ONNX/proveedor obligatorio en primitivas; pesos/rankings importados. Rechazar defaults permisivos, reset destructivo sobre datos compartidos, contenido recuperado que conceda permisos, paths/prefijos como autenticación y port source-to-source como bypass de licencia.

No trasladar equivalencias falsas: stream=memoria acotada, cola vacía=aplicado, task succeeded=durable/ready, compiled=Delivered, timeout=rollback, VACUUM/cleanup=secure erase, hash=firma o snapshot coherente. Mecanismos y alternativas están enlazados por excepción en §1; no se abren nuevas features ni migraciones.

## 6. Advertencias metodológicas

Fuentes de código fijadas, documentación, reporte upstream, inferencia y validación futura permanecen separados. «No acreditado» se limita a rutas/callers inspeccionados, no ausencia global; licencia raíz no cubre transitivas/modelos/datos ni acredita seguridad. Estrellas son selección congelada, no calidad/OSI; no hay cifras propias, superioridad medida ni nueva tabla competitiva.

Los cierres T7 rectifican atribuciones documentales, no ejecutan mitigaciones. Residuales conservados: Codebase Memory E, autorización end-to-end de Quivr, frontend externo parcial y jobs Studio sin pin. Lecturas Studio locales complementarias de pgvector/LanceDB/Supermemory no se convierten en cuerpos S por reemplazar el prefijo de ruta. La [referencia S](VANTA-REFERENCE.md#identidad-de-la-evidencia-studio) y los cuerpos de las closures sustentan los tres paneles aquí citados.

No se cambia ignore, inventario/JSON, backlog, contratos, fuentes, reviews ni artefactos benchmark. La incorporación selectiva del cierre anterior es histórica, no prueba de staging/commit actual; esta consolidación no hace Git writes.

## 7. Benchmarks

Los runners y sus denominadores se documentan en las fichas y en BENCHMARK-NOTES.md. Sus protocolos de adaptación remiten a METHOD.md. No hay resultados propios ni una tabla competitiva comparable; error operativo, vacío legítimo, evidencia recuperada y respuesta evaluada no se homologan. Las correcciones de la ampliación conservan las fórmulas upstream, explicitan su población efectiva y no crean un nuevo harness, scoring, umbral o configuración.

## 8. Matriz de brechas

Matriz respecto de **V/S**, no del HEAD concurrente. «Existe» acredita mecanismo estático, no paridad/runtime; «no acreditado» expresa laguna de la muestra, no inexistencia.

| Superficie propia verificada en fuentes | Delta candidato, no capacidad implementada por este cierre |
|---|---|
| BM25/HNSW/RRF, sparse, exact scan y Pre/In/PostFilter; `explain_memory_search` | C3/C7: jerarquía si necesaria, cobertura/parada, soporte por transporte y procedencia de ejecución; no otro motor/filtro |
| `RecallScope/RecallConfig/InjectionPolicy`, tenancy agent/team y provenance | C3/C4/C10: identidad/revisión y reautorización por caso; scope no sustituye autenticación |
| Assembler/compaction, `IntegratedContext/CompactionReport`, budget de recall | C7: descartes/render y exceso de prefijo protegido explícitos, no «añadir budgets» ni cabida absoluta |
| L0/sesiones, tie-break, generation log, checkpoints de tarea/global | C8: ledger de entrega confirmado por host; cursor/captura/generación/progreso no prueban Delivered |
| Writer plan/persist/dedup/supersession, reflexión, personas/escenas/skills, aprobación de captura | C1/C5/C6/C10: reemplazo y publicación/transformación condicionada; conservar privacidad/ownership, no dedup desde cero |
| Ingestors/wiki, worker, scheduler y transición terminal run_id | C9: progreso/resultado por ítem y fencing por publicación solo si se acredita; no job system nuevo |
| `WriteReceipt`, `put_certified`, WAL sync, tombstones, rebuild y auditoría textual/deep | C13/C14: admisión/aplicación/durabilidad/readiness por revisión, integridad fuente↔derivado y retención con alcance; no «sin recibos» |
| Validación dimensional, índices/cuantización y capabilities de proceso; cuotas/gobernador | C15: compatibilidad/modelo/dominio por entorno o colección lógica; no heterogeneidad física ni enforcement total inferidos de configuración |
| `code_index::index_repo`, grafo y herramientas MCP; adapters de integración existentes | C11/C12: fidelidad/capabilities y evidencia canónica; E caller/job/ACK/persistencia **no acreditados** |
| Studio S MemoryLens/ActivityPanel/RetrievalLens; auth fuera de loopback ya reconocida | C16: vistas incrementales de evidencia y destino; jobs/JobChip, nuevos bridges y autorización universal no acreditados |

## 9. Negativos y deuda propia

- **Hecho estático en V, no solo antecedente:** [L1][v-l1] separa plan/persist/batch y documenta delete-before-put/fallo parcial; [put_batch][v-write] confirma por chunks y retorna resultado global. Esto motiva C1/C13, no pérdida/crash reproducidos ni deuda universal de toda escritura.
- **Hecho + hipótesis separados:** [CheckpointManager::mutate][v-checkpoint] usa read/modify/write separados; carrera depende de caller/intercalación aún no demostrados. [L0Recorder][v-l0] publica mensajes antes del cursor y ya tiene tie-break; su posible concurrencia por sesión no se resuelve modificando step/partial de TaskCheckpoint. WAL/proyecciones son otra frontera.
- **Límites de observación propios:** vigencia derivada por schema/conteos no prueba igualdad de conjuntos; drain HNSW y rebuild por fases requieren reportar parcial/unknown ante error tardío, no inferir corrupción o pérdida durable sin caller/recovery. Manifest conjunto, ACK code-index y cadena Quivr de autoridad siguen no acreditados.
- **Evaluación propia:** revalidación estática LoCoMo fijada en [BENCHMARK-NOTES](BENCHMARK-NOTES.md#6-revalidacion-estatica-de-locomo-durante-la-ampliacion); C2 permanece autoría del lead, sin extender su diseño aquí.
- **Falsos negativos descartados:** no faltan globalmente híbrido, filtros, budgets, checkpoints, cuotas, dedup/provenance, erasure/auditoría, receipts, adapters o Studio. Prefijo protegido puede exceder budget; `WriteReceipt` no es firma. `c561158`/dirty, FIND-* previos y supuestos post-L1 no revalidados quedan históricos, no hechos actuales. Antes de una tarea, contrastar su ruta contra el estado entonces vigente.

## 10. Escritorio / UI

| Familia / fuente | Código observado frente a producto/documentación externa | Lección para Studio existente |
|---|---|---|
| Apps/plataformas | [AnythingLLM](anything-llm.md): SPA; Electron referido en compatibilidad/docs, main/preload no leídos. [Khoj](khoj.md): Electron/web/plugins con fronteras distintas. [Dify](dify.md)/[Langflow](langflow.md)/[Flowise](flowise.md): web/canvas/runs; [Quivr](quivr.md): demo feed/AddText/facade | C16: destino/consentimiento, resultados por ítem y error; canvas/IPC/cancel modal no prueban commit, censo global ni undo |
| Frameworks | [AutoGen](autogen.md) Studio web real; [LangChain](langchain.md)/[LangGraph](langgraph.md), [CrewAI](crewai.md), [Agno](agno.md), [OpenAI Agents](openai-agents-python.md): tooling, tracing o SaaS con cobertura individual, no todo frontend inspeccionado | C11 declara capacidades; producto alojado no equivale a UI OSS o binario nativo |
| DB/vector | [Qdrant](qdrant.md) sirve web, frontend externo no auditado; [Milvus](milvus.md) delimita cliente externo; [Redis](redis.md) Insight package/docs, [TiDB](tidb.md) Dashboard externo; [SurrealDB](surrealdb.md) Surrealist con flujo a SHA separado; [Meilisearch](meilisearch.md) launcher CDN a tag, no zip/SHA equivalente | C13/C16 separan evidencia del motor, frontend y control mutante; no prometer todos los paneles auditados |
| Excepciones retrieval | [Weaviate](weaviate.md): demo web implementada, Explorer Cloud documentado. [pgvector](pgvector.md): extensión + clientes SQL externos, UI propia no acreditada en muestra. [LanceDB](lancedb.md): SDK/CLI/docs, Cloud externo no inspeccionado. [Chroma](chroma.md): TUI Ratatui/Crossterm real, no desktop/web por inferencia | C7/C14/C16: plan, readiness, mantenimiento y alcance; ninguna ausencia global de clientes del ecosistema |
| RAG/documentos | [GraphRAG](graphrag.md): Streamlit con búsqueda/citas/error; [RAGFlow](ragflow.md): CompileProgressBoard web; [LightRAG](light-rag.md): WebUI/cancel-request; [Serviette](serviette.md): HTML same-origin; [LlamaIndex](llama-index.md)/[PageIndex](pageindex.md): Cloud externo no inspeccionado como frontend completo | C5/C9/C14/C16: derivado, progreso, error y cancel solicitado no prueban publicación íntegra, grounding ni rollback |
| Memoria | [Mem0](mem0.md), [Cognee](cognee.md), [Graphiti](graphiti.md), [Hindsight](hindsight.md), [TencentDB](tencentdb-agent-memory.md), [Agentmemory](agentmemory.md): flujos web delimitados; [MemPalace](mempalace.md) landing no inspector; [Supermemory](supermemory.md) redirección/hook grafo no consola completa; [Letta](letta.md) TUI Ink real, desktop/web externos no auditados | C8/C16: persistido/generado/renderizado/entregado distintos; color, contador o turno exitoso no acreditan entrega/borrado |
| Code-memory/KG | [CodeGraph](codegraph.md), [Graphify](graphify.md), [Understand Anything](understand-anything.md): viewers con fuentes/cobertura propias; [Codebase Memory](codebase-memory-mcp.md) conserva límite de imports/comentario, no caller React acreditado | C12/C14: origen/cobertura/revisión visibles; no presentar AST, heurística y modelo como una confianza común |
| OpenViking, referencia histórica | Central §10: WebStudio con código; Helper anunciado, fuente no localizada; SaaS declarado | C16 distingue código/anuncio/documentación/SaaS, no inventa implementación ni ausencia global |

**Destino:** ampliar Studio S existente y sus tres paneles C16. Ninguna nueva App; JobChip/jobs sin pin, nuevos contratos y runtime continúan propuestos/no acreditados. Las lecturas locales complementarias citadas por algunas fichas no sustituyen los cuerpos S. Mostrar diagnóstico no concede permiso de reparación, restore, acceso a citas ni transmisión externa.

## Apéndice — fuentes

Las fichas enlazadas en §2/§10 cubren los **42 informes**; OpenViking es referencia separada. Pins, licencias y fuentes por proyecto permanecen en [README](README.md), [clone-manifest](clone-manifest.json) y [adiciones](clone-additions-manifest.json), sin recalcular selección. Autoridad de aceptación/límites: [REVIEW-COVERAGE](REVIEW-COVERAGE.md); cierres T7 enlazados en la introducción. Esta síntesis no reemplaza el detalle por ficha ni modifica esos registros.

**Navegación técnica congelada:** enlaces V/S siguientes fijan revisión; los enlaces relativos a fichas/closures son navegación local mutable. Si después se aprueban specs, separar por contrato/publicación, contexto/entrega, integración y presentación conservando este índice y las fichas, no duplicar aquí sus implementaciones.

[v-l1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L52-L620
[v-checkpoint]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/checkpoint.rs#L79-L117
[v-l0]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L274-L495
[v-write]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1029-L1529
[v-put-one]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L862-L1010
[v-wal]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L850-L1028
[v-recall]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L133-L1189
[v-vector]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L67-L312
[v-input]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types/record.rs#L184-L225
[v-code-index]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vantadb-mcp/src/code_index.rs#L152-L327
[v-code]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vantadb-mcp/src/code.rs#L33-L301
[v-merge]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L16-L288
[v-worker]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L163-L309
[v-worker-run]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L64-L153
[v-wiki-page]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L287-L320
[v-wiki-terminal]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L107-L217
[v-context]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L93-L348
[v-explain]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/explain.rs#L12-L90
[v-scheduler]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L25-L119
[v-task]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L188
[v-approval]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/approval.rs#L1-L168
[v-attest]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L678-L745
[v-attest-builder]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L914-L955
[v-hnsw]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/storage/engine/ops.rs#L163-L240
[v-audit]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/index.rs#L118-L205
[v-derived]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_index.rs#L16-L242
[v-admin]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/admin.rs#L16-L145
[s-memory]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550
[s-activity]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L38-L91
[s-retrieval]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228
