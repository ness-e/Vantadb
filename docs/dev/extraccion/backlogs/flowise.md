---
title: "Backlog de extracción — Flowise"
kind: research
status: active
description: "Candidatas del informe Flowise: outcome por fuente, configuración efectiva de observabilidad, binding ejecución/state/memoria, recibo de mutación y binding autorizado por transporte"
tags: [extraccion, backlog, flowise]
---

# Backlog de extracción — Flowise

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/flowise.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
> Nota de contexto: el repositorio upstream figura **archivado** en el snapshot; las candidatas son referencia de composición/diagnóstico, no dependencia estratégica.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-FLW-01 | 🔴 | Resultado compuesto con identidad y outcome por fuente | `src/sdk/search/fusion.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: cada fuente de una consulta compuesta produce un batch con source_id, revisión, status y hits con identidad estable hasta el assembler, distinguiendo error, vacío y cancelación. Por qué: concatenar texto o asignar resultados por iteración pierde fuentes anteriores; desempatar por score o reconciliar por contenido colapsa documentos distintos y genera identidad ambigua. Acción mínima: comprobar la composición actual, preservar outcome por fuente hasta `assemble_with_recall`/`fuse_rrf_many` existentes y usar desempate por identidad sin nuevo agregador ni LLM. Trazabilidad: informe §Ficha A · evidencia `packages/components/nodes/agentflow/Retriever/Retriever.ts:138-218`, `packages/server/src/services/documentstore/index.ts:1489-1576`, `packages/components/nodes/retrievers/RRFRetriever/ReciprocalRankFusion.ts:1-96` | X-FLW-04 (si promete revisión visible), C7 | IDs/revisiones, autorización por fuente, cancelación y presupuesto compartido existentes |
| X-FLW-02 | 🟡 | Configuración efectiva de observabilidad con dueño único | `spec` | 🟢 1d | 🟠 P1 | ⬜ Candidata | Qué: la configuración de diagnóstico resuelve precedencia (User/Environment/Default) con revisión, dueño único de emisión y estado efectivo inspeccionable sin revelar credenciales. Por qué: entorno y UI pueden activar trazadores duplicados o contradecir la preferencia aparente; una caché eterna de configuración deja obsoleto el valor efectivo. Acción mínima: cotejar la precedencia actual, hacer visible origen y cuándo entra en vigor un cambio, elegir un único emisor y separar el fallo del exportador del éxito del retrieval. Trazabilidad: informe §Ficha B · evidencia `packages/components/src/tracingEnv.ts:1-111` | X-FLW-04, C16, C8 | Configuración versionada y lifecycle existentes, redacción y canal de actividad; conectividad solo si el usuario aprueba exportación |
| X-FLW-03 | 🟠 | Binding explícito entre ejecución, state y memoria | `vanta-memory/src/utils/task_checkpoint.rs:112-246` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: un run/task enlaza session, scope autorizado, revisión de definición y revisión de checkpoint, con disposición Resumable/RequiresReconciliation/Terminal y errores tipados. Por qué: reanudar trabajo interrumpido no debe interpretar checkpoint como recuerdo, repetir herramienta ya aplicada ni reutilizar la sesión de otro workspace; restaurar «última ejecución» no demuestra CAS ni serialización. Acción mínima: inventariar IDs y callers, asociar checkpoint/sesión en el adapter con definición revisionada y rechazar reanudación ambigua; preferir extender `TaskCheckpointManager` antes que crear tabla nueva. Trazabilidad: informe §Ficha C · evidencia `packages/server/src/utils/buildAgentflow.ts:1575-1868`, `packages/components/nodes/memory/BufferMemory/BufferMemory.ts:86-146` | X-FLW-04 (efectos inciertos), X-FLW-01, C1, C9 | Aislamiento/IDs actuales y escritor único por task ya documentado; CAS solo con soporte acreditado |
| X-FLW-04 | 🔴 | Recibo de mutación con conciliación de proyección externa | `vanta-memory/src/core/record/l1_writer.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: cada mutación produce un recibo que separa Accepted/Applied/Durable/Visibility por dimensión con Unknown explícito, operation_id estable y conciliación de derivados por ID/revisión. Por qué: un ACK de encolado, un status UPSERTED o un timeout tras commit no acreditan aplicación ni visibilidad; borrar el catálogo local no purga la proyección remota. Acción mínima: inventariar garantías por camino, mapear evidencia real a dimensiones, correlacionar operation_id antes de red y consultar al proveedor por ID/revisión antes de reintentar; complementa el `WriteReceipt` existente (SHA-256, no firma) sin crear otro WAL. Trazabilidad: informe §Ficha D · evidencia `packages/server/src/services/documentstore/index.ts:319-470,1270-1444` | X-FLW-03, X-FLW-01, C13 (candidato transversal que formaliza), C9 | Provenance y checkpoints existentes, contrato de commit real por backend; qué proveedores soportan inspección queda abierto |
| X-FLW-05 | 🟠 | Binding de memoria autorizado por recurso y transporte | `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: el adapter negocia capacidades y mapea actor autenticado, recurso, transporte y acción a un scope/session autorizado con versión de política; sessionId del JSON o del LLM nunca crea permiso. Por qué: misma API visual y API externa pueden elegir sesiones distintas o confundir la clave de un flow con la identidad del usuario que puede recordar; un clear no soportado no debe ser no-op silencioso. Acción mínima: inventariar actor/recurso/transporte, resolver la política existente, validar la acción antes de recall/mutación y conservar la decisión en el reporte. Trazabilidad: informe §Ficha E · evidencia `packages/server/src/utils/buildChatflow.ts:990-1110`, `packages/components/nodes/memory/BufferMemory/BufferMemory.ts:86-146`, `packages/server/src/routes/documentstore/index.ts:8-79` | X-FLW-03 (consume el binding), X-FLW-01, X-FLW-04, C3 | Política/auth del host e `InjectionPolicy`/tenancy existentes; actor efectivo del API consumer queda abierto por decisión |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/flowise.md` (Fichas A–D de §2,
> complementos §11.2 y Ficha E de §11.3). Todos los contratos son **hipotéticos**
> — propuestas de comportamiento para VantaDB, no APIs existentes ni puertos de
> código upstream. Las referencias `archivo:línea` son del clon
> `C:\Users\Eros\Flowise` en SHA `9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb`
> (repositorio **archivado**: referencia de composición, no dependencia
> estratégica); las rutas de VantaDB fijan el snapshot
> `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.

### X-FLW-01 — Resultado compuesto con identidad y outcome por fuente

- **Problema:** componer varias fuentes exige conservar qué devolvió cada una, su identidad estable y sus errores; concatenar texto no lo garantiza. Upstream asigna `docs = ...` en cada vuelta de store (la última fuente gana), intenta reconciliar hits por contenido con fallback a UUID y usa score como dedup — score no es identidad.
- **Contrato hipotético (tipos propuestos):** `SourceRetrievalBatch { scope, source_id, source_revision, config_revision, status, hits }`; `EvidenceHit { evidence_id, revision, rank, score_domain, source_ref }`. Operaciones propuestas: `retrieve_scoped(req, sources, cancel) -> Result<Vec<SourceRetrievalBatch>, RetrievalAdapterError>`, `record_source_outcome`, `merge_by_identity`, `select_existing_rrf`. Error tipado: `Unauthorized`, `UnsupportedRevision`, `SourceFailed`, `Canceled`; éxito vacío requiere fuente ejecutada. `scope` deriva de autorización, no de texto; sin LLM obligatorio.
- **Fuente de evidencia:** `packages/components/nodes/agentflow/Retriever/Retriever.ts:138-218` (S1; `:153-176` sobrescribe `docs` por store — dos stores exitosos conservan solo la última asignación), `packages/server/src/services/documentstore/index.ts:1489-1576` (S3; reconcilia contenido de resultados con chunks, UUID de fallback no acredita continuidad de identidad ni revisión), `packages/components/nodes/retrievers/RRFRetriever/ReciprocalRankFusion.ts:1-96` (S4; `:83-90` usa `relevancy_score` en `seenScores` — dos documentos distintos con igual score no se conservan ambos; `RRFRetriever.ts:112` elige default de `c` usando `topK`).
- **Seam VantaDB:** `src/sdk/search/fusion.rs::fuse_rrf_many` (fusión existente), `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` (FV3, `:252-326`), `vanta-memory/src/core/hooks/auto_recall.rs` (`RecallConfig`/`InjectionPolicy`) — provenance → assembler/reporte; ingestors o conexiones externas como adaptadores, no nuevo agregador en core.
- **Acciones:**
  1. Comprobar la composición actual antes de agregar tipos.
  2. Mapear IDs externos a namespace/key y revisión de provenance existentes.
  3. Resolver por fuente preservando outcome (error/vacío/cancelación distinguibles) hasta el ensamblado.
  4. Usar la fusión vigente y el ensamblador; mantener desempates por identidad sin descartar documentos de igual score.
  5. Exponer solo metadatos autorizados en el reporte.
- **Dependencias:** IDs/revisiones, autorización por fuente, cancelación y presupuesto compartido existentes; revisión del adaptador si usa red/modelos. Depende de X-FLW-04 solo cuando se promete revisión visible; X-FLW-02 no bloquea consulta.
- **FMEA:** asignación por iteración → fuentes anteriores desaparecen; empate deduplicado por score → pérdida de documentos; match por contenido → identidad ambigua; fallo parcial → falso éxito total. Mitigar con batches por fuente, clave revisionada, outcome tipado y política explícita de parcialidad. Adicional §2: detectar cardinalidad/identidades por fuente antes/después del merge; cancelación posterior al retorno no invalida ownership de hits.
- **Criterios de aceptación propuestos (manuales):** dos fuentes conservan sus hits; scores iguales no colapsan IDs distintos; mismo contenido con distinta procedencia se representa; cancelar no se informa como éxito vacío; no mezclar scopes. Complemento §11.2: fuente vacía y otra fallida conservan outcomes distintos; cancelación preconsulta no fabrica negativo.
- **Criterio negativo:** un fallo parcial nunca se reporta como éxito total; un resultado vacío requiere que la fuente se haya ejecutado — la cancelación no fabrica outcome.
- **Independencia:** alternativa mínima: ampliar el reporte existente sin DTO público; adaptador solo si hay frontera de proveedor. Registros legacy sin revisión quedan `Unknown`, no se inventa UUID estable; no exige reindexar contenido y desactivar el envelope conserva el retrieval actual.
- **No objetivos:** implementar RRF de nuevo ni copiar la generación multi-query upstream; `MemoryScope` no se presenta como tipo existente.

### X-FLW-02 — Configuración efectiva de observabilidad con dueño único

- **Problema:** entorno y UI pueden activar trazadores duplicados o hacer que una preferencia aparente no coincida con el proveedor realmente usado; una caché eterna de configuración deja obsoleto el valor efectivo.
- **Contrato hipotético (tipos propuestos):** `EffectiveObservationConfig { revision, enabled, origin: User|Environment|Default, emitter_owner, destination_ref, redaction }` — nunca incluir token, URL con secreto ni texto recuperado por defecto. Operaciones propuestas: `resolve(user, environment_snapshot, defaults)`, `inspect_redacted(revision)` (origen y necesidad de reload, nunca credencial), `emit_once`, `report_reload_requirement`.
- **Fuente de evidencia:** `packages/components/src/tracingEnv.ts:1-111` (S5: cachea configuración de entorno durante la vida del proceso — un cambio de env no actualiza el valor cacheado y el comentario exige reinicio; UI gana cuando su proveedor tiene `status:true`, en otros casos el entorno habilitado aporta entrada; al adoptar LangSmith elimina flags del auto-tracer para evitar doble emisión — decisión upstream, no recomendación de mutar env global de VantaDB).
- **Seam VantaDB:** `spec` — configuración/diagnóstico y auditoría existentes; `ActivityPanel` de Studio como seam visible; exportador externo opt-in fuera del motor.
- **Acciones:**
  1. Inventariar la precedencia actual.
  2. Resolver de forma pura origen y revisión efectiva.
  3. Redactar; conectar un emisor único opt-in (no un segundo auto-tracer).
  4. Mostrar origen, revisión efectiva y cuándo entra en vigor un cambio.
  5. Mantener valores secretos en el dueño existente de credenciales.
- **Dependencias:** configuración versionada y lifecycle existentes, redacción y canal de actividad; conectividad solo si el usuario aprueba exportación externa. Puede existir sin X-FLW-01/03/04 ni red.
- **FMEA:** dos emisores → costes/trazas duplicadas; caché eterna → config obsoleta; disable de UI sobrescrito por env → expectativa de privacidad rota; error del exportador → bloqueo del retrieval. Mitigar con política explícita, estado efectivo inspeccionable y fallo de exportación separado. Detección: revisión configurada ≠ efectiva exige diagnóstico de reload; contar dueño/evento, no guardar payload.
- **Criterios de aceptación propuestos:** un evento tiene un dueño; origen efectivo es visible; desactivación tiene semántica documentada; inspección no revela credenciales; retrieval local funciona sin exportadores ni red. Complemento §11.2: env/UI en conflicto declara origen y owner efectivo; secreto no aparece en inspección; error de exporter no aborta búsqueda local.
- **Criterio negativo:** ningún fallo de exportación revierte una escritura (X-FLW-04) ni vuelve terminal un checkpoint (X-FLW-03); `inspect_redacted` nunca devuelve credencial.
- **Independencia:** migración aditiva sin reindex; legacy muestra origen `Unknown` hasta resolución; desactivar el exportador no cambia persistencia ni resultados.
- **No objetivos:** adoptar LangSmith/analytics, reiniciar servicios en esta tarea ni mutar variables de entorno globales desde una consulta.

### X-FLW-03 — Binding explícito entre ejecución, state y memoria

- **Problema:** reanudar trabajo interrumpido no debe interpretar checkpoint como recuerdo, repetir herramienta ya aplicada ni reutilizar la sesión de otro workspace; restaurar «última ejecución» no demuestra CAS ni serialización.
- **Contrato hipotético (tipos propuestos):** `RunMemoryBinding { run_id, task_id, session_key, session_id, scope_ref, definition_revision, checkpoint_revision: Option<u64> }`; `inspect_binding(&AuthorizedScope, &RunId) -> Result<RunMemoryBinding, BindingError>`; `resume_existing_task(binding, expected_revision, cancel) -> Result<ResumeDisposition, BindingError>`. Disposition: `Resumable`/`RequiresReconciliation`/`Terminal`; errores: `ScopeDenied`, `RevisionConflict`, `MissingCheckpoint`, `UnsupportedLegacy`. `scope_ref` es referencia a autorización vigente, no bypass guardado en JSON; `checkpoint_revision` es diseño, no promesa de CAS implementado.
- **Fuente de evidencia:** `packages/server/src/utils/buildAgentflow.ts:1575-1868` (F3; `:1664-1715` busca últimas ejecuciones por sessionId+agentflowId+workspaceId y restaura state solo si `startPersistState===true`; `:1776-1868` valida STOPPED/resume; `:2041-2042` clasifica error por substring `"Aborted"` — no trasladar esa clasificación textual a un contrato Rust), `packages/components/nodes/memory/BufferMemory/BufferMemory.ts:86-146` (F4; `getChatMessages` por sessionId+chatflowid ASC; `addChatMessages`/`clearChatMessages` son no-op deliberados — el servidor es dueño de esas escrituras).
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs:112-246` (`TaskCheckpointManager::{begin,load,advance,complete,fail}` — documenta un escritor por task_id y RMW; su RMW no prueba una carrera efectiva sin caller/reproducción) y `vanta-memory/src/core/record/l1_writer.rs::plan_write/persist_planned/log_and_mark` (FV4 `:289-330`, separa recuerdo de efectos secundarios). Preferir extender el checkpoint/reporte actual antes que crear tabla nueva.
- **Acciones:**
  1. Inventariar IDs y callers existentes.
  2. Asociar checkpoint/sesión en el adapter, con definición revisionada.
  3. Rechazar reanudación ambigua y reconciliar vía X-FLW-04.
  4. Mostrar referencias sin contenido en Activity.
  5. Habilitar solo transport autorizado; si no hay flujo reanudable, solo inspección.
- **Dependencias:** aislamiento/IDs actuales y escritor único por task ya documentado; X-FLW-04 cuando hay mutación externa; X-FLW-01 para evidencias de consulta; X-FLW-02 solo diagnóstico. CAS solo con soporte acreditado. Registro legacy sin revisión no es resumible por inferencia; migración lazy del vínculo, sin reindex de recuerdos.
- **FMEA (tabla del informe):** dos resumes → RMW pierde avance → doble herramienta → conflicto de revisión/owner → escritor único; editar definición → mismo run/otro grafo → paso erróneo → revision mismatch → reconciliar, no replay ciego; reset de sesión → checkpoint confundido con recuerdo → IDs/tipos separados → no borrar uno al limpiar otro; timeout después de efecto → retry → duplicado → D incierto → consultar/reconciliar antes de repetir.
- **Criterios de aceptación propuestos (manuales):** mismo task no se inicia dos veces; vínculo de otra scope se deniega; legacy sin definición no se reanuda silenciosamente; STOPPED/resumible se distingue de Failed; borrar conversación no afirma cancelación ni purga de efectos. Observabilidad por IDs/estados, sin prompts.
- **Criterio negativo:** una pausa humana no marca `Failed`; cancelar el await no elimina checkpoint ni revierte efectos; nunca reanudar automáticamente desde una cadena de error que contenga «Aborted».
- **Independencia:** desactivación vuelve al camino actual — no borra checkpoints ni habilita repetir side effects; no reanudar automáticamente.
- **No objetivos:** ejecución visual, exactly-once distribuido, perfil de usuario nuevo, ni copiar snapshots completos de Flowise a memoria semántica.

### X-FLW-04 — Recibo de mutación con conciliación de proyección externa

- **Problema:** un ACK de encolado, un status `UPSERTED` o un timeout tras commit no acreditan aplicación ni visibilidad; borrar el catálogo local no purga la proyección remota — delete del catálogo y delete vectorial son rutas distintas.
- **Contrato hipotético (tipos propuestos):** `MutationReceipt { operation_id, scope_ref, source_revision, accepted: Observation<Acceptance>, applied: Observation<AppliedRevision>, durable: Observation<DurabilityEvidence>, visibility: Vec<ProjectionObservation> }` donde `Observation<T> = Unknown|Pending|Confirmed(T)|Failed(ErrorRef)` y `ProjectionObservation = projection_id + target_revision + Observation<ReadEvidence>`. Accepted: responsable reconoce operación; Applied: cambio primario acreditado; Durable: evidencia y política concreta de persistencia (no bool «fsync» inferido); Visibility: observación por proyección/revisión/scope. Operaciones propuestas: `inspect_mutation(scope, operation_id)`, `reconcile_projection(receipt, projection_id, cancel)`; errores `Unauthorized`/`UnsupportedEvidence`/`RevisionConflict`/`Transport` separados del outcome observado.
- **Fuente de evidencia:** `packages/server/src/services/documentstore/index.ts:1270-1444` (F6; upsert remoto **anterior** a historial/telemetría/status — error posterior no compensa el upsert ya realizado) y `:319-470` (delete catálogo `:319-354` vs `deleteVectorStoreFromStore:399-470`: filtra por id+workspaceId, exige configs embedding/vector/record manager y llama delete solo si el proveedor lo expone — ausencia de método no equivale a purga confirmada).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` (`persist_planned`, `apply_dedup_batch`, `log_and_mark`), SDK Embedded/backend/WAL y checkpoints existentes. Complementa el `WriteReceipt` ya existente en `src/attestation.rs:678-745` (FV5: integridad por SHA-256 y relectura — **no** firma digital, no garantía remota): `MutationReceipt` es el candidato transversal C13 propuesto que extiende la respuesta del adapter solo donde falta evidencia por dimensión, sin crear otro WAL/ledger/worker.
- **Acciones:**
  1. Inventariar garantías por camino.
  2. Mapear evidencia real a dimensiones (Accepted/Applied/Durable/Visibility).
  3. Correlacionar `operation_id` estable antes de la red.
  4. Persistir en checkpoint/actividad existentes.
  5. Consultar al proveedor por ID/revisión antes de reintentar — nunca reinsertar ciegamente.
  6. Exponer inspección; alternativa inicial sin persistencia nueva: envelope de respuesta con `Unknown`.
- **Dependencias:** provenance y checkpoints existentes, contrato de commit real por backend; X-FLW-03 para reanudar; X-FLW-01 puede consumir visibility sin depender de X-FLW-02. Decisión abierta del informe: qué proveedores soportan inspección de revisión/read-after-write.
- **FMEA (tabla del informe):** error de telemetría tras upsert → error global → retry duplica → comparar receipt/proveedor → no degradar Applied a no aplicado; delete catálogo → vector remoto permanece → recuerdo reaparece → visibilidad por derivado → conciliación explícita, no «borrado total»; worker viejo usa config nueva → revisión mezclada → config_revision/target_revision → rechazar o declarar incertidumbre; ack sin read-after-write → falso listo → Visibility Pending/Unknown → consulta autorizada, no deducir desde Accepted.
- **Criterios de aceptación propuestos (manuales):** ack sin aplicación no es ready; indexación demorada conserva primario aplicado/durable según evidencia; falla secundaria no borra commit; timeout tras commit conserva operation_id; delete sin método remoto se declara unsupported, no purga exitosa; ninguna inspección revela contenido/credenciales.
- **Criterio negativo:** una dimensión `Unknown` nunca se rellena desde otra; timeout devuelve resultado incierto/recibo conocido — **timeout no es rollback** ni `Failed` de una mutación aplicada; una respuesta tardía de revisión vieja no pisa una nueva.
- **Independencia:** legacy se conserva como `Unknown` sin upgrade ficticio; no reindex salvo cambio de embedding/chunking decidido aparte; deshabilitar conciliación no borra documentos ni recibos. Guardar recibo durable no vuelve durable al dato remoto.
- **No objetivos:** atomicidad global, scheduler nuevo, barrier de lectura inventado ni port del runtime de Flowise.

### X-FLW-05 — Binding de memoria autorizado por recurso y transporte

- **Problema:** la misma API visual y la API externa pueden elegir sesiones distintas, o confundir la clave de un flow con la identidad del usuario que puede recordar; un clear no soportado no debe ser no-op silencioso. El sessionId/control de texto son datos, no actor autenticado.
- **Contrato hipotético (tipos propuestos):** `authorize_memory_binding(actor: &AuthenticatedActor, resource: MemoryResourceRef, transport: TransportCapability, action: MemoryAction) -> Result<AuthorizedMemoryBinding, BindingAuthorizationError>`. Salida: scope/session/recurso autorizado y versión de política; errores `MissingIdentity`/`ScopeDenied`/`UnsupportedAction`/`LegacyUnmapped`. El sessionId del JSON o del LLM nunca crea permiso.
- **Fuente de evidencia:** `packages/server/src/utils/buildChatflow.ts:990-1110` (F1; `:990-1095` deriva chatId de `incomingInput.chatId ?? overrideConfig.sessionId ?? UUID`, valida API key para externo y deriva workspace→organization→cuota — no equivale a autorizar user final a todo recuerdo), `packages/components/nodes/memory/BufferMemory/BufferMemory.ts:86-146` (F4), `packages/server/src/routes/documentstore/index.ts:8-79` (F8; checkPermission/checkAnyPermission son referencias a RBAC enterprise comercial — no permiso para copiarlo ni evidencia de autorización idéntica en todas las rutas).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs::InjectionPolicy/RecallConfig` (`:133-335`), tenancy `vanta-memory/src/core/record/l1_writer.rs::default_tenancy/plan_write`, bindings y adapters LangChain existentes (FV1 `integrations/vantadb_shared/__init__.py:31-163`, FV2 `integrations/langchain/vantadb_langchain/checkpointer.py:232-296`) — negociación de capacidades y mapping en adapter, no IAM ni sesiones nuevas; no inventar `MemoryScope` implementado.
- **Acciones:**
  1. Inventariar actor/recurso/transporte.
  2. Resolver la política existente.
  3. Validar la acción antes de recall/mutación.
  4. Mapear namespace/sesión autorizados.
  5. Conservar la decisión en el reporte; legado sin owner se rechaza o resuelve explícitamente — nunca migrar a público.
- **Dependencias:** política/auth del host e `InjectionPolicy`/tenancy existentes; X-FLW-03 consume el binding; X-FLW-01/X-FLW-04 comparten la decisión. Sin modelo ni dependencia de queue. Decisión abierta: actor efectivo del API consumer y compartición explícita entre agentes — no una política inferida del canvas.
- **FMEA:** UI y API usan distinta session → pérdida aparente → mostrar mapping redactado; mismo sessionId en dos tenants → cruce si falta actor → autorización previa; clear no soportado → silencio/no-op → `UnsupportedAction`. Detectar conteos/denegaciones/IDs seguros, no contenidos de cada tenant.
- **Criterios de aceptación propuestos:** UI/API mapeadas deliberadamente a la misma sesión ven el mismo historial; sesiones distintas no se unen por texto; actor revocado no borra ni consulta; cancelación post-commit no reautoriza ni revierte.
- **Criterio negativo:** un `sessionId` del JSON o del LLM nunca crea permiso; un legado sin owner nunca migra a público por inferencia.
- **Independencia:** desactivar un transporte revoca el acceso sin borrar el primario; alternativas evaluadas y descartadas: DB por tenant (multiplica gestión), filtro por session sin actor (insuficiente) — se prefiere la política externa existente.
- **No objetivos:** IAM, runtime ni scheduler nuevos; copiar RBAC comercial ni routing Express.
