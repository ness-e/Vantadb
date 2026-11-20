---
title: "Backlog de extracción — Letta"
kind: research
status: active
description: "Candidatas A–G del informe Letta: publicación por revisión, validación de proyección, reparación por firma, compactación honesta, estados separados, recall con cobertura y provenance de eco"
tags: [extraccion, backlog, letta]
---

# Backlog de extracción — Letta

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/letta.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
>
> **Nota de procedencia:** el clon seleccionado (`C:\Users\Eros\letta`,
> `letta-ai/letta@8816d3a`) es una landing sin código — su aporte es
> declarativo (redirección a letta-code y prohibiciones de uso de V1). Las
> evidencias `archivo:línea` de cada fila pertenecen al **complemento remoto
> fijado** `letta-ai/letta-code@9176d8c0b16f408e9240422a98379a423e2176c3`,
> citado en el informe como [L2]–[L23]; no son rutas del clon ni del servidor
> V1 retirado.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-LET-01 | 🟠 | Publicación de contexto desde revisión aceptada | `vanta-memory/src/context_engine/engine.rs`; `vanta-memory/src/core/hooks/auto_recall.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | **Qué:** ensamblar el contexto contra una revisión de fuente fijada, con identidad de snapshot (`source_refs`, `revisions`, `policy_revision`, `content_digest`) en `IntegratedContext`. **Por qué:** hoy selección, render y entrega pueden mezclar borrador/persistido/derivado; Letta separa committed→compilado→entregado y guarda `memfsRevision`/`rawSystemHash`. **Acción mínima:** contrato `assemble_published(request, revision)` aditivo sobre `assemble_with_recall`; errores `StaleRevision/Denied/SourceUnavailable/Cancelled` distintos de vacío. **Trazabilidad:** informe §Ficha A · evidencia `src/backend/local/system-prompt-compilation.ts:23-39,65-130`; `src/agent/memory-git.ts:1180-1268,1833-1977` | C4, C5, C6, C8; X-LET-02, X-LET-05 | Cómo fijar revisión conjunta y distinguir selección de entrega real en los callers de [V1][V2] |
| X-LET-02 | 🟠 | Validador de proyección con política confiable antes de publicar | `vanta-memory/src/ingest/merge.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | **Qué:** validador puro `validate_projection(before, proposed, policy)` con rutas canónicas, índices requeridos, read-only y violaciones tipadas, común a ingesta y UI. **Por qué:** un documento hostil no puede autoconcederse autoridad ni cuota; Letta verifica índices ascendentes, archivos regulares y protección read-only contra un árbol aceptado confiable. **Acción mínima:** extender `is_structural`/`commit` existentes; política del host, nunca del candidato; `ProtectedPath/MissingIndex/InvalidPath/BudgetExceeded/StaleBase`. **Trazabilidad:** informe §Ficha B · evidencia `src/memory-constraints.ts:21-41,46-155`; `src/agent/memory-format.ts:6-40`; `src/backend/local/initial-memory.ts:9-38,78-115` | C3, C7; X-LET-01 | Alcance de trusted policy en cada ingestor, autorización de publicación y tratamiento de legado inválido |
| X-LET-03 | 🟡 | Reparación acotada por firma de conflicto, identidad y generación | `vanta-memory/src/services/pipeline_worker.rs`; `vanta-memory/src/utils/task_checkpoint.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | **Qué:** `claim_repair(scope, source_revision, problem_digest, actor)` + `publish_repair(claim_generation, expected_revision, candidate)`; un intento vigente por problema, fencing al publicar. **Por qué:** evita reintentos infinitos del mismo conflicto y que un worker tardío sobrescribe una corrección nueva; el claim Letta usa firma de conflicto, PID+inicio y token de generación. **Acción mínima:** extender payload/checkpoint del scheduler existente con firma+generación; cálculo fuera de lock; validar con X-LET-02 al commit. **Trazabilidad:** informe §Ficha C · evidencia `src/agent/memory-conflict-repair.ts:119-234`; `src/agent/memory-operation.ts:1-58`; `src/utils/file-lock.ts:1-140` | C9, C13; X-LET-01, X-LET-02 | Fencing efectivo en publicación del resultado, no solo reclamar trabajo; el LLM propone contenido, nunca autorización |
| X-LET-04 | 🟡 | Compactación con progreso explícito y unidades conversacionales | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** `plan_compaction(history_revision, budget, protected_units)` con outcomes `NoProgress/Irreducible/StaleHistory/Cancelled` y preservación de relaciones call/result. **Por qué:** un resumen que no reduce contexto produce bucles de retry, y cortar a mitad de tool call deja resultados huérfanos; Letta distingue fallo de planificación de irreducible y no fabrica resumen ante overflow. **Acción mínima:** delta sobre `CompactionReport`/assembler existentes — detectar no-progreso, reportar residual e informar cuándo el prefijo protegido excede presupuesto. **Trazabilidad:** informe §Ficha D · evidencia `src/backend/local/compaction.ts:43-205,738-820` | C7, C8; X-LET-01 | Unidad call/result y detección de no-progreso en los callers concretos del assembler |
| X-LET-05 | 🟡 | Estado separado de guardado, sincronización y entrega | `vanta-memory/src/services/pipeline_worker.rs` (LayerTelemetry); Studio `feature-freez/desktop/src/components/` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | **Qué:** `source_status(auth, source_ref)` con `{source_revision, accepted_revision, projected_revision, delivered_revision?, sync_outcome?}` y `Unknown/Denied/Unavailable` explícitos por dimensión. **Por qué:** «guardado» no debe fundir commit local, sync remoto, proyección y entrega; el issue #4831 muestra ediciones dirty presentadas como reflexión aplicada. **Acción mínima:** traducir outcomes por adapter hacia ActivityPanel/MemoryLens existentes; reintentos reautorizados, sin ejecutar Git desde UI ni exponer tokens. **Trazabilidad:** informe §Ficha E · evidencia `src/agent/memory-git.ts:1833-1977`; `src/backend/local/system-prompt-compilation.ts:23-39`; issue letta-code#4831 (informe §3) | C8, C14, C16; X-LET-01, X-LET-03 | Semántica de estados que transporta MCP/Studio; condicionado a X-LET-01 para revisiones |
| X-LET-06 | 🟠 | Recall que declara backend, modo efectivo y cobertura | `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** `recall_with_coverage(auth, request, cancel)` devolviendo `{refs, source_revisions, backend_kind, effective_mode, coverage, skipped_sources}`; `UnsupportedMode`, `UnreadableSource` y `Cancelled` nunca se convierten en vacío. **Por qué:** «sin recuerdos» es engañoso cuando solo se buscaron summaries recientes o el modo pedido no está soportado; el backend local Letta rechaza vector/hybrid y su `rrf_score` local es fracción de términos, no RRF. **Acción mínima:** propagar ruta efectiva y fuentes fallidas desde los backends existentes a `RecallResult`; cobertura visible en X-LET-05. **Trazabilidad:** informe §Ficha F · evidencia `src/backend/message-search.ts:19-59`; `src/backend/local/transcript-search.ts:558-630`; `src/backend/conversation-search.ts:44-115`; `src/backend/api/search.ts:1-19` | C7, C15; X-LET-05, X-LET-07 | Cobertura y modo efectivos por caller en `RecallConfig`/`core_search`; ACL del transporte previa |
| X-LET-07 | 🟡 | Clasificación de eco de recall en provenance | `vanta-memory/src/core/conversation/l0_recorder.rs`; `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | **Qué:** `classify_recall_origin(refs, authorized_turn_identity)` con `Original/RecallEcho/CurrentQuery/Unknown`, ligado a vínculo call/result acreditado. **Por qué:** recuperar una recitación de búsqueda anterior como evidencia independiente refuerza errores; Letta atenúa ecos por `searchToolCallId`/`echoToolCallId`, sin borrado global de tool results. **Acción mínima:** propagar linaje call/result existente en L0 hacia la selección previa al assembler; decisión de ranking la conserva el caller — advertir primero, política solo tras spec. **Trazabilidad:** informe §Ficha G · evidencia `src/backend/local/transcript-search.ts:285-325,384-415,538-542,606-619` | C7, C14; X-LET-06 | Disponibilidad real del linaje call/result en L0; `Unknown` para clientes que solo entregan texto |

## Fichas completas

> Contratos y nombres de tipos son **propuestos no implementados**; las rutas
> `src/…` de evidencia pertenecen al complemento remoto fijado
> `letta-ai/letta-code@9176d8c0b16f408e9240422a98379a423e2176c3` (referencias
> [L4]–[L23] del informe), no al clon local ni al servidor V1 retirado. Las
> rutas `vanta-memory/…` fijan el seam real del repo VantaDB.

### X-LET-01 — Publicación de contexto desde revisión aceptada

- **Problema:** una reflexión cambia preferencias mientras otra conversación prepara el siguiente turno; hoy selección, render y entrega pueden mezclar borrador, persistido y derivado. El modelo no debe recibir una mezcla de versiones.
- **Contrato hipotético:** `assemble_published(request: AuthorizedContextRequest, revision: AcceptedRevision) -> Result<PublishedContext, PublishError>`; resultado `{source_refs, revisions, policy_revision, content_digest, compaction_report}`. Errores `StaleRevision/Denied/SourceUnavailable/Cancelled` distintos de vacío.
- **Fuente de evidencia:** `src/backend/local/system-prompt-compilation.ts:23-39,65-130` (recopilación committed, `memfsRevision`/`rawSystemHash`); `src/agent/memory-git.ts:1230-1268,1833-1977` (separación commit/sync) — informe §Ficha A [L5][L4].
- **Seam VantaDB:** `ContextEngine::assemble_with_recall`, `IntegratedContext`, `InjectionPolicy` — `vanta-memory/src/context_engine/engine.rs`; `vanta-memory/src/core/hooks/auto_recall.rs`.
- **Acciones:** 1) inventariar revisión fuente y contrato de entrega; 2) propagar refs hasta el assembler; 3) seleccionar contra revisión aceptada; 4) invalidar compilaciones al modificar/borrar fuente; 5) exponer diagnóstico aditivo.
- **Dependencias:** X-LET-02 define la validación previa; X-LET-05 muestra el estado. Mismo contrato de revisión que el ledger OpenViking y la caché Supermemory — no tres almacenes. Las revisiones provienen de registros existentes, no del nombre de sesión.
- **FMEA:** edición concurrente → mezcla de versiones → detectar mismatch y replanificar; caché vieja tras delete → fuga → invalidar/reautorizar; persistencia OK pero delivery fallido → falso positivo → estados distintos.
- **Criterios de aceptación:** dos conversaciones, edición sin publicar, revisión nueva, revocación entre selección y entrega, fuente borrada; ninguna revisión inventada ni contenido en logs; medir counts, revisión opaca y outcome.
- **Criterio negativo:** persistencia confirmada no acredita entrega; `content_digest` no se expone como prueba criptográfica de entrega.
- **Independencia:** mecanismo independiente — adapter Git optativo solo si un origen lo exige; no crea memoria paralela ni sustituye el assembler por Git.
- **No objetivos:** no Git obligatorio, sincronización Cloud ni exactly-once del proveedor. Abierto: quién fija la revisión aceptada.

### X-LET-02 — Validador de proyección con política confiable antes de publicar

- **Problema:** un recuerdo masivo o un índice roto expulsa contexto útil; un agente edita una instrucción protegida para elevar su autoridad.
- **Contrato hipotético:** `validate_projection(before: AuthorizedTree, proposed: CandidateTree, policy: TrustedProjectionPolicy) -> Result<ProjectionPlan, ProjectionError>` con rutas canónicas, revisiones y violaciones tipadas `ProtectedPath/MissingIndex/InvalidPath/BudgetExceeded/StaleBase`.
- **Fuente de evidencia:** `src/memory-constraints.ts:21-41,46-155` (`validateMemoryTreeConstraints`: reader + árbol aceptado confiable + política; índices ascendentes, archivos regulares, read-only); `src/agent/memory-format.ts:6-40` (selección jerárquica); `src/backend/local/initial-memory.ts:9-38,78-115` (colisiones `a/b` vs `a_b`, case-insensitive en inicialización) — informe §Ficha B [L6][L7][L18].
- **Seam VantaDB:** `normalize_wiki_path`, `is_structural`, `commit` — `vanta-memory/src/ingest/merge.rs`; entrega posterior por `vanta-memory/src/context_engine/engine.rs`.
- **Acciones:** 1) mapear políticas existentes; 2) validar árbol antes de merge; 3) registrar plan contra revisión base; 4) revalidar al persistir; 5) presentar errores en Studio.
- **Dependencias:** X-LET-01 consume solo lo aprobado; no depende de X-LET-03. Política versionada aparte del candidato; la validación no escribe. Las rutas son autoridad de almacenamiento, no autorización por sí mismas.
- **FMEA:** path traversal/symlink → escape → canonicalización/control del host; read-only definido en el candidato → bypass → política trusted del host, nunca del documento; presupuesto parcial → falsa cabida → reporte de excedentes protegidos.
- **Criterios de aceptación:** índice padre faltante, borrado protegido, renombre, Unicode, archivo no regular, dos revisiones y documento que intenta cambiar la política; detectar sin filtrar texto sensible en errores.
- **Criterio negativo:** rutas canónicas validadas no constituyen autorización; límites en unidad declarada, no equivalencia chars=tokens; errores previos no deben quedar ocultos como nuevos éxitos.
- **Independencia:** validador puro común a ingesta y UI; extensión de `is_structural`/`commit` existentes, no componente nuevo ni hooks Git copiados.
- **No objetivos:** no copiar límites numéricos de Letta ni rediseñar wiki; no aplanar labels a claves Vanta ni interpretar un label como path autorizado. Abierto: tratamiento explícito de legado inválido.

### X-LET-03 — Reparación acotada por firma de conflicto, identidad y generación

- **Problema:** la reparación automática repite el mismo conflicto o un worker tardío sobrescribe un recuerdo ya corregido.
- **Contrato hipotético:** `claim_repair(scope, source_revision, problem_digest, actor) -> Result<RepairClaim, RepairError>`; `publish_repair(claim_generation, expected_revision, candidate) -> RepairOutcome`; errores `AlreadyAttempted/InProgress/StaleClaim/Conflict/Cancelled`.
- **Fuente de evidencia:** `src/agent/memory-conflict-repair.ts:119-234` (firma de conflicto/estado inválido, `claimed/in_progress/attempted`, identidad PID+inicio y token de generación); `src/agent/memory-operation.ts:1-58` y `src/utils/file-lock.ts:1-140` (exclusión de checkout) — informe §Ficha C [L10][L8][L9].
- **Seam VantaDB:** `PipelineWorker::with_owner/run_once/reclaim_stale` y dead letters — `vanta-memory/src/services/pipeline_worker.rs`; checkpoints en `vanta-memory/src/utils/task_checkpoint.rs`.
- **Acciones:** 1) inventariar `run_once`/`reclaim_stale`; 2) extender payload/checkpoint con firma y generación; 3) ejecutar el cálculo fuera de lock; 4) validar con X-LET-02 y revisión al commit; 5) registrar resultado y habilitar reintento humano explícito.
- **Dependencias:** X-LET-01/X-LET-02 para la publicación del resultado; recibos/ownership de Agentmemory describen la misma frontera transversal; no requiere adoptar Git.
- **FMEA:** firma que omite revisión → se suprime una reparación válida → incluir revisión en la firma; holder pausado tratado como muerto → dos commits → fencing al publicar, no solo al reclamar; token viejo libera trabajo nuevo → comparar generación.
- **Criterios de aceptación:** mismo conflicto dos veces, cambio de conflicto, reinicio del worker, cancelación, revisión editada durante el LLM y resultado rechazado; outcomes/attempts sin prompt.
- **Criterio negativo:** reclamar trabajo no equivale a publicación autorizada; el LLM propone contenido, nunca autorización ni commit sin validación.
- **Independencia:** extiende scheduler, owner, reclaim y `TaskCheckpointManager` existentes — no otro scheduler ni lease distribuido sin necesidad; migración aditiva sin reindexar recuerdos intactos.
- **No objetivos:** no prometer resolver semántica ni fusionar tenants; nunca copiar PID como identidad distribuida. Abierto: reset humano de `AlreadyAttempted`.

### X-LET-04 — Compactación con progreso explícito y unidades conversacionales

- **Problema:** un resumen que no reduce contexto produce bucles de retry; cortar a mitad de tool call deja resultados huérfanos o pierde la última petición.
- **Contrato hipotético:** `plan_compaction(history_revision, budget, protected_units) -> Result<CompactionPlan, CompactionError>`; `apply(expected_revision, plan, summary) -> CompactionOutcome` con `NoProgress/Irreducible/StaleHistory/Cancelled`.
- **Fuente de evidencia:** `src/backend/local/compaction.ts:43-205,738-820` (`assertLocalCompactionProgress`, `planLocalSlidingWindowCompaction`, fallback completo y errores diferenciados) — informe §Ficha D [L12].
- **Seam VantaDB:** `CompactionReport` y assembler existentes — `vanta-memory/src/context_engine/engine.rs` (`assemble_inner`).
- **Acciones:** 1) mapear unidades actuales en `assemble_inner`; 2) identificar protección y progreso existentes; 3) añadir outcomes solo donde falten; 4) revalidar historia al aplicar; 5) exponer reporte al host.
- **Dependencias:** X-LET-01 registra la revisión entregada; distinto de X-LET-03 (reparar fuente) y del checkpoint de tarea. Plan puro, resumen fuera de lock, commit condicionado a revisión.
- **FMEA:** token estimator subestima → overflow → reporte de residual y salida irreducible; resumen sin progreso → bucle → detección de no-progreso; edición concurrente → resumen obsoleto → mismatch y descarte.
- **Criterios de aceptación:** solo protegido, último tool pendiente, summary vacío, fallo de proveedor, no progreso y revisión concurrente; counters/estimaciones sin mensajes.
- **Criterio negativo:** un resumen fabricado ante fallo u overflow no cuenta como compactación; un prefijo protegido que excede presupuesto debe informarse, no ocultarse.
- **Independencia:** delta sobre `CompactionReport`/assembler existentes — no otra implementación de summarizer ni prompts nuevos; proveedor LLM optativo.
- **No objetivos:** no copiar el porcentaje upstream ni umbral nuevo; no garantizar exactitud semántica del resumen; no purgar L0 para ahorrar contexto.

### X-LET-05 — Estado separado de guardado, sincronización y entrega

- **Problema:** la UI dice «guardado» aunque el archivo siga dirty, el push falle o la conversación continúe con contexto viejo; el issue #4831 muestra ediciones dirty presentadas como reflexión aplicada.
- **Contrato hipotético:** `source_status(auth, source_ref) -> Result<PublicationStatus, StatusError>` con `{source_revision, accepted_revision, projected_revision, delivered_revision?, sync_outcome?}`; `Unknown/Denied/Unavailable` explícitos por dimensión.
- **Fuente de evidencia:** `src/agent/memory-git.ts:1833-1977` (outcomes post-turno); `src/backend/local/system-prompt-compilation.ts:23-39` (revisión compilada); issue letta-code#4831 (informe §3) — informe §Ficha E [L4][L5].
- **Seam VantaDB:** `LayerTelemetry` y `load_assembled_context` — `vanta-memory/src/services/pipeline_worker.rs`; Studio `feature-freez/desktop/src/components/` (ActivityPanel, MemoryLens existentes).
- **Acciones:** 1) mapear `LayerTelemetry` y `load_assembled_context`; 2) sumar traducción por adapter; 3) publicar metadatos mínimos; 4) mostrar diferencias en ActivityPanel/MemoryLens; 5) asociar acciones autorizadas de reintento — nunca botones que ejecuten Git arbitrario.
- **Dependencias:** X-LET-01/X-LET-03; estados de procesamiento de Supermemory; un único modelo de outcomes con traducciones por adapter, no un DTO idéntico para mecanismos distintos.
- **FMEA:** merge de estados reduce un fallo a clean → confianza falsa → mostrar cada dimensión por separado; error contiene token Git → fuga → redacción y código tipado; acción de reintento ajena → daño → reautorizar.
- **Criterios de aceptación:** dirty, commit local, conflicto, push fallido, proyectado viejo y delivery desconocido visibles por separado; sin contenido ni secretos en telemetría.
- **Criterio negativo:** fallo de autenticación del refresh padre y checkout ausente no son el mismo estado (discriminación L-1 del informe); un fix de runner Git reportado no acredita sync universal.
- **Independencia:** condicionada a X-LET-01 para las revisiones; traduce outcomes por adapter hacia los paneles existentes, sin cliente nuevo.
- **No objetivos:** no cliente desktop nuevo ni prueba de sync Cloud; no ejecutar Git desde la UI ni exponer tokens. Abierto: retención de metadata.

### X-LET-06 — Recall que declara backend, modo efectivo y cobertura

- **Problema:** pedir una decisión antigua y recibir «sin recuerdos» cuando solo se buscaron summaries recientes o el modo vectorial pedido no estaba soportado.
- **Contrato hipotético:** `recall_with_coverage(auth, request, cancel) -> Result<CoveredRecall, RecallFailure>`; resultado `{refs, source_revisions, backend_kind, effective_mode, coverage, skipped_sources}`; `UnsupportedMode`, `UnreadableSource`, `Denied`, `Cancelled`, `UnknownRevision` nunca se convierten en vacío.
- **Fuente de evidencia:** `src/backend/message-search.ts:19-59` (dispatch); `src/backend/local/transcript-search.ts:558-630` (validación/filtrado; el backend local rechaza vector/hybrid y su `rrf_score` local es fracción de términos, no RRF); `src/backend/conversation-search.ts:44-115` (búsqueda de summaries); `src/backend/api/search.ts:1-19` — informe §Ficha F [L19][L20][L21][L23]. `src/agent/prompts/recall_subagent_local.md` [L22] son instrucciones, no implementación.
- **Seam VantaDB:** `RecallConfig::core_search`, `RecallResult` y scopes — `vanta-memory/src/core/hooks/auto_recall.rs`; cobertura propagada a `assemble_with_recall` en `vanta-memory/src/context_engine/engine.rs`.
- **Acciones:** 1) mapear `RecallResult`/`RecallConfig`; 2) propagar ruta efectiva desde el backend existente; 3) conservar errores de fuente por clase; 4) pasar cobertura a `assemble_with_recall`; 5) enseñar el estado en X-LET-05.
- **Dependencias:** X-LET-01 fija revisión de entrega, X-LET-07 marca ecos, X-LET-05 comunica degradación. ACL del transporte y cancelación del host son previas; no depende de Git ni de LLM.
- **FMEA:** modo solicitado no soportado → vacío aparente → detectar capability/`UnsupportedMode`; JSONL saltado → falsa exhaustividad → contadores de fuente fallida; summary-only → decisión ausente → coverage explícita y expansión autorizada.
- **Criterios de aceptación:** local FTS, modo no soportado, summary sin contenido, archivo corrupto, archivo hidden, fecha inválida, revocación y cancelación; IDs/revisiones o `Unknown` honestos, logs sin query ni reasoning.
- **Criterio negativo:** un campo llamado `rrf_score` en un backend no es un score RRF comparable entre backends; «sin recuerdos» no acredita búsqueda exhaustiva.
- **Independencia:** DTO aditivo sobre backends existentes; históricos quedan `Unknown`, sin reingesta ni reindexado; adapter Letta solo si después se especifica el SDK actual.
- **No objetivos:** no universalizar un score ni crear archivo semántico; no activar `core_search` universalmente por inferencia. Abierto: si el consumidor admite éxito parcial y cómo expresa un rango temporal desconocido.

### X-LET-07 — Clasificación de eco de recall en provenance

- **Problema:** el resultado de una búsqueda anterior contiene la misma frase que el usuario pregunta ahora; recuperar esa recitación como evidencia independiente refuerza errores.
- **Contrato hipotético:** `classify_recall_origin(refs, authorized_turn_identity) -> OriginReport` con `Original/RecallEcho/CurrentQuery/Unknown`, refs/revisión y enlace call/result; errores `BrokenCallLink/StaleHistory/Denied/Cancelled` diferenciados.
- **Fuente de evidencia:** `src/backend/local/transcript-search.ts:285-325,384-415,538-542,606-619` (`searchToolCallId`/`echoToolCallId`, asociación call/result, pregunta activa, `rankMatches`; upstream atenúa ecos sin borrado global de tool results) — informe §Ficha G [L20].
- **Seam VantaDB:** enlaces de tool calls en `vanta-memory/src/core/conversation/l0_recorder.rs`; propagación al recall en `vanta-memory/src/core/hooks/auto_recall.rs` y clasificación previa al assembler en `vanta-memory/src/context_engine/engine.rs`.
- **Acciones:** 1) inventariar enlaces de tool calls en `L0Recorder`; 2) propagar referencias al record/recall existente; 3) clasificar antes del assembler; 4) emitir motivos; 5) habilitar política solo tras spec.
- **Dependencias:** X-LET-06 declara cobertura y X-LET-01 revisión; mismo problema transversal que el dedup de presentación de Supermemory, pero aquí importa linaje, no igualdad textual.
- **FMEA:** tool result legítimo clasificado como eco → pérdida de evidencia → vínculo explícito y fallback `Unknown`; recap sin vínculo → circularidad → detectar provenance incompleta; pregunta activa vence a fuente histórica → falsa confirmación → distinguir `CurrentQuery` sin borrarla del historial.
- **Criterios de aceptación:** resultado recitado, herramienta no buscadora, misma frase de fuente independiente, resultado sin call, consulta explícita de la conversación actual y append concurrente; observar razones/counts, no contenido.
- **Criterio negativo:** contenido idéntico no basta para clasificar eco — solo un enlace de origen acreditado; no se importan pesos 0.1, boosts ni nueva fórmula de ranking.
- **Independencia:** metadata opcional, legacy `Unknown`; la decisión de ranking la conserva el caller — advertir primero, política solo tras spec aprobada.
- **No objetivos:** no dedup semántico adicional ni blacklist de tool results; no parser de comandos shell como autoridad; sin garantía de eliminar toda circularidad. Abierto: propagación de linaje cuando el host solo entrega texto.

## Notas de alcance

- **Sin fila para §7 (benchmark):** la adaptación de latencia propuesta reutiliza
  el protocolo existente de [METHOD](../agent-memory-2026-11-07/METHOD.md) y el
  runner upstream leído (`scripts/latency-benchmark.ts:1-340`) no evalúa
  memoria; converge en C2 (evaluadores confiables) en vez de abrir unidad nueva.
- **Rechazos del informe (§5)** no generan filas: adaptador V1, Git como base
  obligatoria, auto-commit ciego post-turno, read-only definido por contenido,
  prompts/porcentajes de compactación y UI/Cloud completa.
- Prefijo `X-LET` reservado a Letta. Estados `⬜ Candidata` por definición:
  nada de esta campaña entra al triage sin fila propia conforme a la taxonomía
  oficial de `docs/dev/Backlog.md`.
