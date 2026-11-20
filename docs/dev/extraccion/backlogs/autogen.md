---
title: "Backlog de extracción — AutoGen"
kind: research
status: active
description: "Candidatas del informe autogen: adapter Memory gobernado, manifiesto de sesión, budget con reinyección idempotente, refs de handoff y export tipado prevalidado"
tags: [extraccion, backlog, autogen]
---

# Backlog de extracción — AutoGen

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/autogen.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
> Repo upstream archivado (README del pin): adapter opcional, no dependencia estratégica.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-ATG-01 | 🟠 | Adapter de memoria gobernado sin apropiarse del context engine | `integrations/vantadb_shared/__init__.py` (`EmbeddingVectorStore`); `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: adapter Python pequeño del contrato Memory que devuelve `RecallEnvelope { query, tenant, scope, evidence, score_kind, budget_consumed }` y metadata con IDs/versiones, validando scope antes de retrieval. Por qué: contenido hostil promovido a SystemMessage, filtro solo después de top-k y UUID nuevo por add son los fallos a evitar; el modelo no autoriza scopes. Acción mínima: fijar versión archivada soportada, mapear MIME, separar query de render, conservar raw IDs/provenance y exigir operation ID del caller para que retries conserven identidad. Trazabilidad: informe §Ficha A · evidencia `python/packages/autogen-core/src/autogen_core/memory/_base_memory.py:26-130`, `python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py:310-413` | X-ATG-03 consume sus IDs; X-ATG-04 usa sus refs; converge con C13 MutationReceipt (propuesto) | Scope/provenance/budgets existentes; decisión previa sobre si un adapter archivado justifica coste de soporte |
| X-ATG-02 | 🟡 | Manifiesto de sesión con referencias a memoria | `integrations/langchain/vantadb_langchain/checkpointer.py` (`VantaDBCheckpointer`); `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `SessionManifest { schema_version, conversation_revision, memory_scope_refs, checkpoint_ref }` que distingue incluido/referenciado/omitido; snapshot del contexto y referencias al store se validan por separado. Por qué: guardar/restaurar un agente no debe aparentar backup de la memoria; upstream exporta `llm_context` sin la DB de memoria y sin atomicidad entre componentes. Acción mínima: inventariar ownership de cada recurso, registrar revisiones, restaurar con validación de tenant/schema y diagnosticar referencias faltantes como error visible. Trazabilidad: informe §Ficha B · evidencia `python/packages/autogen-agentchat/src/autogen_agentchat/agents/_assistant_agent.py:1028-1053,1626-1639`, `python/packages/autogen-core/src/autogen_core/model_context/_chat_completion_context.py:49-74` | X-ATG-05 valida su serialización; X-ATG-04 usa su generación | Sesiones, provenance y checkpoints existentes; restauración no cruza scope ni pisa nuevas escrituras |
| X-ATG-03 | 🟠 | Budget global y reinyección idempotente de evidencia | `vanta-memory/src/context_engine/engine.rs::assemble_with_recall`; `IntegratedContext`/`CompactionReport` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `InjectionKey { evidence_id, revision, placement }` y decisión de reinyección dentro del reporte de assembly existente; el presupuesto considera tools, instrucciones y todas las memorias juntas. Por qué: el número de stores no debe multiplicar silenciosamente tokens ni duplicar memoria en contexto; append cronológico y recorte del medio que deja tool result huérfano son patrones rechazados. Acción mínima: separar contexto crudo de vista, registrar IDs inyectados, agrupar llamada/resultado de tools y explicar omisiones bajo presupuesto. Trazabilidad: informe §Ficha C · evidencia `python/packages/autogen-core/src/autogen_core/memory/_list_memory.py:104-172`, `python/packages/autogen-agentchat/src/autogen_agentchat/agents/_assistant_agent.py:1028-1053`, `python/packages/autogen-core/src/autogen_core/model_context/_token_limited_chat_completion_context.py:19-94` | X-ATG-01 aporta IDs; X-ATG-02 aporta generación | Presupuesto compartido, compactación, spill/dedup existentes; prefijo protegido puede exceder budget (excepción documentada) |
| X-ATG-04 | 🟡 | Referencias de memoria en handoff sin transferencia de privilegios | `vanta-memory/src/core/hooks/auto_recall.rs` (`InjectionPolicy`, `perform_auto_recall_governed`); checkpoint existente | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: `prepare_handoff(actor, target, refs, rev) -> HandoffMemoryRefs` con refs autorizadas y missing explícito; el destino no obtiene ACL por recibir el ID. Por qué: un especialista que recibe control debe conocer qué evidencia autorizada sustentaba la tarea sin duplicar transcript ni ampliar audiencia. Acción mínima: identificar refs, validar actor/destino, persistir versión en checkpoint y reautorizar al consumir con la ACL vigente. Trazabilidad: informe §Ficha D · evidencia `python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_swarm_group_chat.py:47-112` | X-ATG-01/X-ATG-02 aportan IDs/generación; X-ATG-03 evita duplicar contexto en retry | Solo si hay consumidor de handoff; governance de recall existente; refs opt-in, no transferencia automática de IAM |
| X-ATG-05 | 🟡 | Export tipado prevalidado | `integrations/langchain/vantadb_langchain/checkpointer.py` (`VantaDBCheckpointer`); `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `validate_export(manifest, resources) -> ValidatedManifest` y `encode_export` con errores `UnsupportedSchema`, `Unserializable`, `MissingResource`, `ScopeDenied`; preflight antes de sustituir archivo. Por qué: un export exitoso nominal puede quedar truncado por datetime/enum y un restore parcial modifica agentes antes de descubrir el recurso faltante. Acción mínima: inventario de recursos, validar todos los refs/schema, serializar en buffer controlado y publicar con el mecanismo de archivo existente; restore solo tras preflight. Trazabilidad: informe §Ficha E · evidencia `python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_base_group_chat.py:748-834`, `python/packages/autogen-core/src/autogen_core/memory/_base_memory.py:40-45`, `python/packages/autogen-agentchat/src/autogen_agentchat/messages.py:45-57` | X-ATG-02 aporta el manifiesto; motivada por issues #6793 y #5688 del informe (mecanismo ya cubierto en pin, protege evolución) | X-ATG-02 (manifiesto); formatos tipados del host, no deserialización ejecutable ni pickle en entrada no confiable |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/autogen.md` (§Fichas A–C y §11.2–11.4). Todos los contratos son **hipotéticos**: firmas propuestas, no APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `d07c667a` (repo archivado); las rutas VantaDB requieren revalidación contra HEAD. Errores comunes propuestos en §11.2: `ScopeDenied`, `UnsupportedMime`, `StaleRevision`, `MissingReference`, `CancelledBeforeWrite`, `OutcomeUnknown`; cancelación posterior a commit no significa `NotApplied`.

### X-ATG-01 — Adapter de memoria gobernado sin apropiarse del context engine

- **Problema:** conectar VantaDB como store del contrato `Memory` sin crear AutoGen en Rust: contenido hostil promovido a `SystemMessage`, filtro solo después de top-k y UUID nuevo por `add` son los fallos a evitar; el modelo no autoriza scopes ni cambia la jerarquía de confianza.
- **Contrato hipotético:** `RecallEnvelope { query, tenant, scope, evidence, score_kind, budget_consumed }` (conserva `score_kind` y provenance — no declara distancia homogénea entre backends); `query_memory(req, cancel) -> Result<RecallEnvelope, AdapterError>` y `capture_memory(req, op_id, cancel) -> Result<MutationObservation, AdapterError>`; el adapter devuelve `MemoryContent.metadata` con IDs/versiones y una proyección contextual.
- **Fuente de evidencia (clone):** `python/packages/autogen-core/src/autogen_core/memory/_base_memory.py:26-130` (`Memory`/`MemoryContent`/`MemoryQueryResult`, `serialize_mime_type` en `:40-45`); `python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py:310-413` (`update_context`/`query`: consulta desde el último mensaje, distancias→scores, IDs/score en metadata; `add` genera UUID — retry no es dedup por identidad de origen); `python/packages/autogen-core/src/autogen_core/memory/_list_memory.py:104-172` (`ListMemory.query` ignora consulta/cancelación y devuelve todo; `update_context` inserta todo como `SystemMessage` cronológico).
- **Seam VantaDB:** `integrations/vantadb_shared/__init__.py` (`EmbeddingVectorStore`), binding Python y retrieval existente BM25/HNSW/RRF; `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` para que el adapter no duplique budgets.
- **Acciones:**
  1. Fijar la versión archivada soportada y mapear MIME.
  2. Separar query de render; validar scope **antes** del retrieval/top-k.
  3. Capturar con `op_id` del caller para que retries conserven identidad.
  4. Conservar raw IDs y provenance; declarar operaciones de lifecycle soportadas.
  5. Metadata opcional versionada; legado `Unknown`; quitar el adapter no borra hechos.
- **Dependencias:** scope/provenance/budgets existentes; decisión previa del owner sobre si un adapter archivado justifica coste de soporte (repo upstream archivado — adapter opcional, no dependencia estratégica); independiente de X-ATG-02.
- **FMEA:** contenido hostil promovido a `SystemMessage` → rol de dato gobernado; cancelación ignorada → `CancelToken` real; duplicación de memoria inyectada → `InjectionKey` (X-ATG-03); filtro solo después de k → scope previo; retry UUID → duplicado → `op_id`; stores sucesivos que consultan contexto ya mutado (hipótesis, requiere caso discriminante).
- **Criterios de aceptación propuestos:** consulta aislada por tenant; resultados trazables; contexto respeta budget global y rol de dato; MIME no soportado falla explícitamente; retries de `add` conservan identidad si el caller aporta operation ID; MIME normal trazable; tenant ajeno negado; cancel/timeout conserva estado conocido. Observabilidad: IDs/revisiones/código/latencia, sin texto.
- **Criterio negativo:** el modelo nunca autoriza scopes; un retry de `add` sin `op_id` nunca se presenta como dedup; contenido recuperado nunca asciende a instrucción de sistema.
- **Independencia:** implementable solo con el contrato `Memory`, sin runtime nuevo; no exige extracción/consolidación automática ni export de todo el store.
- **No objetivos:** no portar runtime ni índice nuevo; no convertir todos los hechos en instrucciones; no adoptar `ListMemory` como memoria de producción ni instrucciones del modelo como ACL. Abierto: backend soportado.

### X-ATG-02 — Manifiesto de sesión con referencias a memoria

- **Problema:** guardar/restaurar un agente no debe aparentar backup de la memoria: upstream exporta `llm_context` sin la DB de memoria y sin atomicidad entre componentes.
- **Contrato hipotético:** `SessionManifest { schema_version, conversation_revision, memory_scope_refs, checkpoint_ref }` que distingue incluido/referenciado/omitido; `export_session(req) -> Result<SessionManifest, AdapterError>` y `restore_session(manifest, expected_revision) -> Result<RestoreReport, AdapterError>`; snapshot del contexto y referencias al store se validan por separado — no se infiere atomicidad entre componentes.
- **Fuente de evidencia (clone):** `python/packages/autogen-agentchat/src/autogen_agentchat/agents/_assistant_agent.py:1028-1053,1626-1639` (`save_state`/`load_state` guardan `llm_context`, no la DB de memoria; `on_reset` limpia contexto, no necesariamente el store); `python/packages/autogen-core/src/autogen_core/model_context/_chat_completion_context.py:49-74` (`ChatCompletionContext.save_state`/`load_state`).
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/checkpointer.py` (`VantaDBCheckpointer`), `vanta-memory/src/utils/task_checkpoint.rs` (`TaskCheckpointManager`); sesiones, provenance y checkpoints ya existentes.
- **Acciones:**
  1. Inventariar ownership de cada recurso.
  2. Decidir qué se exporta y qué solo se referencia; registrar revisiones.
  3. Persistir refs y serializar la versión del schema.
  4. Restaurar con validación de tenant/schema y revisión esperada.
  5. Diagnosticar referencias faltantes como error visible; legado sin refs se marca incompleto; rollback no pisa nuevos hechos.
- **Dependencias:** sesiones, provenance y checkpoints existentes; X-ATG-01 si incluye memory refs; X-ATG-05 para serialización; sin scheduler propio; la restauración no cruza scope ni pisa nuevas escrituras.
- **FMEA:** contexto antiguo con memoria nueva (mezcla de generaciones → quiescencia/preflight); referencias borradas → error visible; restauración que pisa nuevas escrituras → validación de revisión esperada; backup que aparenta incluir una DB externa → manifest declara incluido/referenciado/omitido; snapshot en vivo → consistencia no acreditada se declara.
- **Criterios de aceptación propuestos:** restaurar no cruza scope; ausencia/versión incompatible es error visible y falla antes de alterar; clear de conversación no elimina hechos salvo orden explícita; export distingue incluidos/referenciados/omitidos; no promete atomicidad remota. Observabilidad: incluidos/refs/faltantes/schema, sin volcar conversación.
- **Criterio negativo:** un restore nunca modifica agentes antes de validar schema/referencias; un export nunca aparenta contener la DB de memoria externa.
- **Independencia:** manifiesto con refs antes que backup distribuido; reutilizable por X-ATG-05.
- **No objetivos:** no checkpoint distribuido exactly-once ni nuevo backup engine; abierto: snapshot inter-recurso.

### X-ATG-03 — Budget global y reinyección idempotente de evidencia

- **Problema:** el número de stores no debe multiplicar silenciosamente tokens ni duplicar memoria en contexto; append cronológico y recorte del medio que deja un tool result huérfano son patrones rechazados.
- **Contrato hipotético:** `InjectionKey { evidence_id, revision, placement }` (ID/revisión/placement, no igualdad textual) y decisión de reinyección dentro del reporte de assembly existente; `plan_injection(view, context_snapshot) -> Result<InjectionPlan, AdapterError>`; el presupuesto considera tools, instrucciones y todas las memorias juntas.
- **Fuente de evidencia (clone):** `python/packages/autogen-core/src/autogen_core/memory/_list_memory.py:104-172` (`ListMemory.update_context` inserta todo cronológicamente); `python/packages/autogen-agentchat/src/autogen_agentchat/agents/_assistant_agent.py:1028-1053` (`_update_model_context_with_memory`); `python/packages/autogen-core/src/autogen_core/model_context/_token_limited_chat_completion_context.py:19-94` (`get_messages`: la vista elimina mensajes desde el medio hasta caber — no certifica coherencia de grupos de tools).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs::assemble_with_recall`, `IntegratedContext`/`CompactionReport` — presupuesto compartido, compactación, spill/dedup y reporte de context assembly ya existentes.
- **Acciones:**
  1. Separar contexto crudo de la vista enviada al proveedor.
  2. Registrar los IDs efectivamente inyectados por `InjectionKey`.
  3. Agrupar llamada/resultado de tools como unidad indivisible.
  4. Elegir prioridad bajo presupuesto global; actualizar solo evidencia revisada, no append de todo el store.
  5. Emitir reporte con budget efectivo y omisiones explicadas; reiniciar el ledger de vista por generación, sin reescribir raw ni reindexar.
- **Dependencias:** presupuesto compartido, compactación y spill/dedup existentes; X-ATG-01 aporta IDs; X-ATG-02 aporta generación; prefijo protegido puede exceder budget (excepción documentada).
- **FMEA:** append repetido → inflación → `InjectionKey` evita duplicar; recorte del medio que deja tool result huérfano → rechazo de grupo completo; exposición de instrucciones hostiles → rol de dato; falsa seguridad por limitar solo la vista → budget sobre el envío efectivo; no copiar literalmente el algoritmo upstream.
- **Criterios de aceptación propuestos:** dos assemble sin nueva evidencia no duplican IDs; vista válida para el proveedor; originals intactos; budget efectivo y omisiones explicados; tool-call/result nunca se separan; sin afirmar calidad QA a partir de tokens ahorrados. Observabilidad: conteos/budget/omisión/revisión, sin payload.
- **Criterio negativo:** ningún ensamblado repite un `evidence_id`+revisión ya entregado en la ventana; ningún recorte separa una tool call de su resultado.
- **Independencia:** extensión del assembler/reporte existente, no un segundo tokenizador ni nueva compaction.
- **No objetivos:** no nueva política de benchmark ni resumidor obligatorio; abierto: accounting por provider.

### X-ATG-04 — Referencias de memoria en handoff sin transferencia de privilegios

- **Problema:** un especialista que recibe control debe conocer qué evidencia autorizada sustentaba la tarea, sin duplicar transcript ni ampliar audiencia; el destino no obtiene ACL por recibir el ID.
- **Contrato hipotético:** `prepare_handoff(actor, target, refs, rev) -> Result<HandoffMemoryRefs, AdapterError>` con refs autorizadas y missing explícito; errores `ScopeDenied`, `UnknownTarget`, `StaleReference`.
- **Fuente de evidencia (clone):** `python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_swarm_group_chat.py:47-112` (`SwarmGroupChatManager.validate_group_state`/`select_speaker`/`save_state` — referencia de mecanismo, no se porta su selección de speakers).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` (`InjectionPolicy`, `perform_auto_recall_governed`); el checkpoint existente conserva solo la referencia al paquete, no tareas nuevas.
- **Acciones:**
  1. Identificar refs de la evidencia autorizada.
  2. Validar actor/destino antes de construir el paquete.
  3. Persistir versión en el checkpoint existente.
  4. Reautorizar al consumir con la ACL vigente.
  5. Resolver refs vía el assemble existente; legado sin refs significa «no suministrado»; desactivar no borra memoria.
- **Dependencias:** X-ATG-01/X-ATG-02 aportan IDs/generación; X-ATG-03 evita duplicar contexto en retry; solo procede si hay consumidor de handoff; governance de recall existente; independiente de consolidación.
- **FMEA:** target malicioso pide scope mayor → denegación; evidencia sustituida entre entrega y lectura → `StaleReference`; retry duplica contexto → `InjectionKey` de X-ATG-03; detección solo de IDs/decisiones, mitigación con ACL vigente y provenance.
- **Criterios de aceptación propuestos:** handoff normal usa las mismas revisiones; revocación impide leer; delete/supersession no resucita la fuente; cancelación prepublicación no crea paquete visible — después no revierte la herramienta.
- **Criterio negativo:** recibir un ID nunca concede acceso; un transcript íntegro nunca se usa como atajo de handoff; referencias ausentes nunca se resuelven en silencio.
- **Independencia:** refs opt-in sobre el handoff del host; paquete de refs/revisiones, no runtime nuevo; P2 — solo si hay consumidor.
- **No objetivos:** no scheduler ni transferencia automática de IAM; abierto quién decide la compatibilidad del destino.

### X-ATG-05 — Export tipado prevalidado

- **Problema:** un export nominalmente exitoso puede quedar truncado por datetime/enum, y un restore parcial modifica agentes antes de descubrir el recurso faltante; la validación debe preceder a sustituir el archivo.
- **Contrato hipotético:** `validate_export(manifest, resources) -> Result<ValidatedManifest, ExportError>` y `encode_export(validated) -> Result<Vec<u8>, ExportError>` con errores `UnsupportedSchema`, `Unserializable`, `MissingResource`, `ScopeDenied`; la validación no asegura atomicidad de restore ni verdad del contenido.
- **Fuente de evidencia (clone):** `python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_base_group_chat.py:748-834` (save/load de estado de equipo); `python/packages/autogen-core/src/autogen_core/memory/_base_memory.py:40-45` (`MemoryContent.serialize_mime_type`); `python/packages/autogen-agentchat/src/autogen_agentchat/messages.py:45-57` (`BaseMessage.dump`). Motivada por issues #6793 y #5688 del informe (mecanismo ya cubierto en el pin — la ficha protege evolución, no acredita bug vigente).
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/checkpointer.py` (`VantaDBCheckpointer`), `vanta-memory/src/utils/task_checkpoint.rs` y bindings — preflight y manifest de schema/recursos sobre el export existente.
- **Acciones:**
  1. Inventario de recursos del export.
  2. Validar todos los refs/schema antes de serializar.
  3. Serializar en buffer controlado con límite de tamaño.
  4. Publicar con el mecanismo de archivo existente; versionar el envelope conservando el importador previo.
  5. Restore solo tras preflight; no migrar raw a pickle.
- **Dependencias:** X-ATG-02 aporta el manifiesto; formatos tipados del host — no deserialización ejecutable ni pickle en entrada no confiable; sin nuevo motor de backup.
- **FMEA:** dato nuevo no serializable → export falla antes de sustituir el archivo; load parcial → reportar recursos aplicados, no éxito global; schema hostil → parse tipado y size limit existentes; detección por código/schema, no por contenido.
- **Criterios de aceptación propuestos:** fechas/MIME hacen roundtrip; schema desconocido no altera estado; un snapshot tomado corriendo declara su consistencia como no acreditada.
- **Criterio negativo:** un export nunca sustituye el archivo anterior sin preflight completo; un restore parcial nunca se reporta como éxito global.
- **Independencia:** requiere X-ATG-02; independiente de X-ATG-04 y de modelos; alternativa rechazada: pickle/jsonpickle (acoplan clases, inapropiados para entrada no confiable).
- **No objetivos:** no snapshot distribuido ni nuevo backup engine; abierto: contrato de quiescencia del host.
