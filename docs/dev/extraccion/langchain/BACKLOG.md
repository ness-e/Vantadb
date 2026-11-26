---
title: "Backlog de extracción — LangChain"
kind: research
status: active
description: "Siete candidatas del informe LangChain: grupos causales de compactación, proyección de tools, decisión vs recibo, append por revisión, contabilidad de intentos, score tipado y capa de touch"
tags: [extraccion, backlog, langchain]
---

# Backlog de extracción — LangChain

> **Origen:** informe `docs/dev/extraccion/langchain/LANGCHAIN.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-LCH-01 | 🟠 | Grupos causales tool-call/result y commit de compactación | `vanta-memory/src/context_engine/engine.rs::{assemble_inner,assemble_with_recall}`, `SpillSink`, `IntegratedContext` | 🟠 | 🟠 P1 | ⬜ Candidata | Invariante de grupo causal (IDs de mensaje/llamada/dependencias) y recibo de sustitución ligado a revisión de entrada · No debe recordarse un resultado sin su llamada ni un resumen vacío/fallido equivaler a compactación exitosa · Acción mínima: extender el coordinador de presupuesto existente con grupos y decisión estructurada kept/replaced/evidence_refs; spill obligatorio fallido bloquea sustitución destructiva · **Trazabilidad:** `informe §2 ficha A · evidencia langchain_v1/langchain/agents/middleware/summarization.py:485-523,758-805,900-986 · issue langchain-ai/langchain#41323` | X-LCH-02 valida sus grupos; X-LCH-03 conserva decisión; converge con C7/C8 de `CONSOLIDATED.md` | Presupuesto compartido/estimador y provenance existentes; prefijo protegido puede impedir caber (informe §4) |
| X-LCH-02 | 🟡 | Proyección de tool results que no confunde retención con olvido | `vanta-memory/src/context_engine/engine.rs` (`SpillSink::spill`, reporte de assembly) | 🟡 | 🟠 P1 | ⬜ Candidata | `ToolProjection {call_id, source_revision, retained_ref, fields_removed, reason}`: el contexto lleva placeholder pero el original autorizado queda recuperable · Quitar un resultado grande del contexto no debe ocultar evidencia ni declararse borrado; PII retirada del contexto sigue existiendo en copias · Acción mínima: placeholder compatible vinculado al reporte, idempotente por ID/revisión; borrado real es flujo aparte · **Trazabilidad:** `informe §2 ficha B · evidencia langchain_v1/langchain/agents/middleware/context_editing.py:79-155` | X-LCH-01 valida grupos; X-LCH-03 conserva decisión; converge con C5 de `CONSOLIDATED.md` | X-LCH-01; allowlist de tools/retención pertenece al host; rehidratación revalida ACL |
| X-LCH-03 | 🟠 | Decisión humana separada del recibo de ejecución de tool | `vanta-memory/src/utils/task_checkpoint.rs`, `vanta-memory/src/core/record/approval.rs` (patrón, dominio distinto) | 🟠 | 🟠 P1 | ⬜ Candidata | `ToolDecision {call_id, proposed/reviewed digest, actor, revisión, decision}` más recibo del ejecutor con clave idempotente · Editar o rechazar una tool call no puede transformarse en "ejecutado" durante compactación ni replay duplicar el envío; respuesta perdida tras aceptar remoto es desconocido a reconciliar · Acción mínima: registrar propuesta → decisión autenticada fuera del LLM → invocación idempotente → recibo persistido · **Trazabilidad:** `informe §2 ficha C · evidencia langchain_v1/langchain/agents/middleware/human_in_the_loop.py:759-850` | X-LCH-01/X-LCH-02 retienen decisión; X-LCH-05 comparte operation_id; converge con C10/C13 de `CONSOLIDATED.md` | Checkpoints existentes; actor autenticado fuera del LLM; executor opt-in |
| X-LCH-04 | 🟠 | Append de historial ligado a la revisión leída | `vanta-memory/src/core/record/l1_writer.rs::{plan_write,persist_planned,apply_dedup_batch}` | 🟡 | 🟡 P2 | ⬜ Candidata | `append_turn(session, expected_revision, run_id, input_ids, output_ids)` con Conflict/AlreadyApplied/Unauthorized · Dos invocaciones intercaladas no deben perder inputs ni deducir identidad restando longitud del historial releído; mismo run devuelve mismo recibo · Acción mínima: capturar revisión al leer, append condicionado del turno y recibo; sin CAS disponible, serializar por sesión en host y declarar el límite · **Trazabilidad:** `informe §2 ficha D · evidencia langchain_core/runnables/history.py:522-580` | X-LCH-05 comparte identidad de intento; converge con C13 de `CONSOLIDATED.md` | Contrato de sesión y ownership; no reusar checkpoint global como log de conversación |
| X-LCH-05 | 🟡 | Contabilidad de intentos y retry en la frontera del host | `vanta-memory/src/utils/task_checkpoint.rs` (progreso), adapters/host (dispatch) | 🟡 | 🟡 P2 | ⬜ Candidata | `InvocationUsage {operation_id, attempt_id, proposed, dispatched, outcome, usage_ref}` con clasificación Retryable/Terminal/Interrupted/Unknown · Propuesto, despachado y ejecutado difieren: retry puede multiplicar coste/efectos y un interrupt de control-flow no es error de tool · Acción mínima: reservar intento según política vigente, registrar despacho y resultado, y detener backoff ante cancelación · **Trazabilidad:** `informe §2 ficha E · evidencia langchain_v1/langchain/agents/middleware/tool_call_limit.py:308-388, tool_retry.py:295-353,387-413` | X-LCH-03 comparte operation_id/recibo; converge con C9/C13 de `CONSOLIDATED.md` | Owner de dispatch/cancelación en host; no alterar presupuesto de contexto ni crear cuotas |
| X-LCH-06 | 🟡 | Score tipado y conformidad versionada del vector store | `integrations/langchain/vantadb_langchain/vectorstore.py:254-305,566-576` (+84-85,414-503) | 🟡 | 🟠 P1 | ⬜ Candidata | `score_kind/metric/raw_score` con conversión de relevancia explícita y `UnsupportedScoreMapping` visible; mutación declara reemplazo/parcial/retry por operación e identidad estable por ID del caller · Distancia, similitud y score fusionado no son probabilidad; UUID por texto+posición rompe reemplazo al editar · Acción mínima: inventariar scores por ruta, mapping por métrica/versión y casos de mutación del adapter (IDs explícitos, kwargs.ids, cardinalidad, fallo parcial) · **Trazabilidad:** `informe §2 ficha F · evidencia langchain_core/vectorstores/base.py:234-291,450-540 · adapter vantadb_langchain/vectorstore.py:84-85,414-503` | X-LCH-04 (identidad estable); converge con C11/C15 de `CONSOLIDATED.md` | Versión Core soportada declarada; cosine/clamp ya existente (no reabrir falso positivo) |
| X-LCH-07 | — | Capa de identidad/touch opcional sobre scoring existente | `vanta-memory/src/core/record/scoring.rs:65-144` (`CompositeScoring`), adapters existentes | 🟡 | 🔵 P3 | ⬜ Candidata | *Severidad `—`: contraste LC-2 experimental opt-in, sin brecha funcional en scoring actual* · Capa experimental de touch sobre `(scope, id, revisión)`: efímera o persistida declarada, revalida permiso al tocar y no muta fuente/updated_at por leer · El buffer posicional upstream queda rechazado (stream no durable, touch sin persistir); solo interesa la semántica de acceso para quien la necesite · Acción mínima: host dueño del estado con error explícito y distinción efímero/durable · **Trazabilidad:** `informe §2 contraste LC-2 · evidencia langchain_classic/retrievers/time_weighted_retriever.py:64-198` | X-LCH-06 (semántica de score); converge con C11 de `CONSOLIDATED.md` | Scoring opt-in existente; decisión de persistencia del host; no repetir retrieval ni crear índice |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/langchain/LANGCHAIN.md` (§2 fichas A–F y contraste temporal LC-2). Todos los contratos son **hipotéticos** — propuestas de comportamiento para VantaDB, no APIs existentes ni puertos de código upstream. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\langchain` en SHA `d9caf083577c43a91c5bf6bd1f7dfc289581184d` (rutas relativas al repo LangChain; prefijos `libs/`, `libs/core/` y `libs/langchain/` abreviados en tabla); las rutas VantaDB citan el snapshot `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y requieren revalidación contra HEAD. Recomendación priorizada del informe §1: conformidad antes de funcionalidad nueva; contexto causal y recuperable; decisiones y efectos; historial y costes en la frontera del host.

### X-LCH-01 — Grupos causales tool-call/result y commit de compactación

- **Problema:** un agente con varias tools paralelas no debe recordar un resultado sin su llamada, ni perder una instrucción por un resumen fallido; un resumen vacío no equivale a compactación exitosa.
- **Contrato hipotético (tipos propuestos):** `ContextGroup { message_ids, call_ids, dependencies }`; una decisión ligada a revisión de entrada produce `kept/replaced/evidence_refs/estimated_tokens` en el reporte actual. Errores diferenciados: fuente cambiada, grupo roto, resumen vacío, presupuesto excedido. El prefijo protegido puede impedir caber (informe §4).
- **Fuente de evidencia (clone):** `libs/langchain_v1/langchain/agents/middleware/summarization.py:485-523` (`SummarizationMiddleware.before_model`), `:758-805` (`_find_token_based_cutoff`: búsqueda binaria + corrección de frontera buscando la llamada AI de ToolMessages), `:900-986` (corte seguro y `_create_summary`; retorno vacío pasa por `.strip()` sin guard) [C1] [C13]; issue [langchain-ai/langchain#41323](https://github.com/langchain-ai/langchain/issues/41323) (mecanismo aún visible estáticamente — validar, no atribuir defecto a Vanta).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs::{assemble_inner, assemble_with_recall}`, `SpillSink`, `IntegratedContext` [V2] — añadir invariantes de grupo y recibo de sustitución, no otro compactador.
- **Acciones:**
  1. Resolver IDs de mensajes/llamadas.
  2. Formar grupos causales.
  3. Planificar spill/resumen.
  4. Generar fuera del motor.
  5. Validar no vacío y revisión.
  6. Persistir evidencia requerida.
  7. Publicar proyección y reporte; recontar la unión con recall. Si el spill obligatorio falla, no publicar sustitución destructiva.
- **Dependencias:** presupuesto compartido/estimador y provenance existentes; prefijo protegido puede impedir caber; se relaciona con X-LCH-02/X-LCH-03 sin fusionar texto proyectado y autorización. Converge con C7/C8 de `CONSOLIDATED.md`. Alternativa conservadora documentada: mantener grupo completo o spill sin resumen.
- **FMEA:** corte huérfano → rechazo por IDs; resumen en blanco → error visible; summary que omite rechazo → conservar decisión estructurada; estimador que subcuenta → reportar exceso, no garantizar tokens exactos.
- **Criterios de aceptación propuestos:** tools múltiples/desordenadas, resumen vacío, fallo del proveedor y cambio concurrente dejan la fuente accesible; el contexto final conserva pares y decisión; el diagnóstico usa IDs/motivos, no payload.
- **Criterio negativo:** un spill obligatorio fallido nunca publica sustitución destructiva; cancelar antes de publicar conserva la fuente; tras el commit no se inventa rollback de la llamada LLM (se recupera la misma proyección).
- **Independencia:** parcial — delta sobre el coordinador de presupuesto existente; X-LCH-02 valida sus grupos y X-LCH-03 conserva decisión; migración aditiva del reporte y desactivar el plan nuevo deja originales legibles.
- **No objetivos:** prompt nuevo, calidad garantizada ni nuevo framework.

### X-LCH-02 — Proyección de tool results que no confunde retención con olvido

- **Problema:** un resultado grande ocupa contexto aunque el agente solo necesita saber que se obtuvo; quitarlo no debe ocultar evidencia ni declararse borrado — la PII retirada del contexto sigue existiendo en copias.
- **Contrato hipotético (tipos propuestos):** `ToolProjection { call_id, source_revision, retained_ref, fields_removed, reason }`, con error `SourceUnavailable` cuando se requiere rehidratación garantizada.
- **Fuente de evidencia (clone):** `libs/langchain_v1/langchain/agents/middleware/context_editing.py:79-155` (`ClearToolUsesEdit.apply`: sustituye contenido por placeholder, borra `artifact` de la copia y marca metadata `cleared`; no borra la fuente externa) [C2].
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` (`SpillSink::spill` y reporte de `assemble_with_recall` [V2]); Vanta ya compacta — el delta es la proyección vinculada.
- **Acciones:**
  1. Clasificar resultado y política de retención.
  2. Guardar el original autorizado.
  3. Construir placeholder compatible con el proveedor.
  4. Publicar la copia vinculada al reporte, idempotente por ID/revisión.
  5. El delete real es flujo aparte sobre todas las copias bajo política; nunca declarar borrado por haber puesto `artifact=None`.
- **Dependencias:** X-LCH-01 valida grupos; X-LCH-03 conserva decisión; allowlist de tools/retención pertenece al host; rehidratación revalida ACL. Converge con C5 de `CONSOLIDATED.md`. Alternativas: truncar vista sin persistir placeholder, o no editar tools críticas.
- **FMEA:** log/artifact conserva secreto → inventario de copias y redacción; placeholder inválido → validar formato del proveedor; referencia borrada → presentar indisponible, no evidencia inexistente como presente.
- **Criterios de aceptación propuestos:** tool excluida intacta, edición repetida estable, contenido retirado del contexto pero accesible solo con permiso; borrado posterior impide rehidratación. Observabilidad por campos/IDs, no secretos.
- **Criterio negativo:** poner `artifact=None` nunca se declara borrado de la fuente; no sobrescribir fuente editada; cancelar antes del commit no marca `cleared`.
- **Independencia:** parcial — depende de X-LCH-01 para validación de grupos; versión aditiva con rollback de vista desde la fuente (nunca restaura PII eliminada por política posterior).
- **No objetivos:** sistema universal de retención ni borrado remoto.

### X-LCH-03 — Decisión humana separada del recibo de ejecución de tool

- **Problema:** editar o rechazar una tool call no puede transformarse en «ejecutado» durante compactación ni un replay duplicar el envío; una respuesta perdida tras aceptar remoto es desconocido a reconciliar.
- **Contrato hipotético (tipos propuestos):** `ToolDecision { call_id, proposed_digest, reviewed_digest, actor, revision, decision }` + recibo del ejecutor con clave idempotente. Rechazar actor no autorizado, decisión vieja y cardinalidad incorrecta.
- **Fuente de evidencia (clone):** `libs/langchain_v1/langchain/agents/middleware/human_in_the_loop.py:759-850` (`HumanInTheLoopMiddleware.after_model`: construye peticiones solo para calls configuradas, `interrupt`, valida número de decisiones y reconstruye orden/IDs) [C3].
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs` [V3] y provenance existentes; `vanta-memory/src/core/record/approval.rs` (`CaptureApprovalQueue` aprueba memorias extraídas, no tools [V4]) — reutilizar el patrón de decisión sin presentar ambos dominios como equivalentes.
- **Acciones:**
  1. Registrar propuesta.
  2. Autenticar al aprobador fuera del LLM.
  3. Persistir la decisión.
  4. Invocar con idempotency key.
  5. Persistir el recibo.
  6. Proyectar el contexto. Remoto aceptó y respuesta perdida = desconocido/reconciliar.
- **Dependencias:** checkpoints existentes; actor autenticado fuera del LLM; executor opt-in. Misma identidad que el replay de LangGraph; X-LCH-01/X-LCH-02 retienen decisión; X-LCH-05 comparte operation_id; converge con C10/C13 de `CONSOLIDATED.md`. Alternativa preferida: usar middleware upstream y almacenar solo evidencia en Vanta, antes que reimplementar HITL completo en Rust.
- **FMEA:** aprobación sobre args nuevos → digest/revisión; replay duplica envío → dedup remoto o reconciliación; rechazo resumido como éxito → estado tipado.
- **Criterios de aceptación propuestos:** editar/rechazar y reanudar con varias calls conserva orden e identidad; recibo faltante no aparece como éxito; actor/decisión auditables sin guardar argumentos sensibles por defecto.
- **Criterio negativo:** una decisión nunca se presenta como efecto ejecutado; el estado `applied` exige recibo; cancelar impide nuevos dispatches pero no deshace el efecto ya aplicado.
- **Independencia:** parcial — decisión terminal condicionada a revisión con un solo claim de ejecución; migración aditiva y executor opt-in; comparte operation_id con X-LCH-05 sin depender de él.
- **No objetivos:** exactly-once remoto, autorizar por texto ni nuevo scheduler.

### X-LCH-04 — Append de historial ligado a la revisión leída

- **Problema:** dos invocaciones de una sesión pueden intercalar historial; quitar prefijo por longitud del historial releído no prueba que sea el prefijo usado; no se observa CAS entre lectura y append upstream.
- **Contrato hipotético (tipos propuestos):** `append_turn(session, expected_revision, run_id, input_ids, output_ids) -> receipt` con `Conflict | AlreadyApplied | Unauthorized`. El caller registra los IDs del prefijo leído; el storage no deduce identidad restando longitud ni compara solo texto.
- **Fuente de evidencia (clone):** `libs/core/langchain_core/runnables/history.py:522-580` (`RunnableWithMessageHistory._enter_history/_exit_history`: copia historial, resta el prefijo por longitud del historial releído y hace `add_messages`) [C5]; no se recomienda adoptar esa API deprecada como nueva base.
- **Seam VantaDB:** sesiones/provenance L1 y `vanta-memory/src/core/record/l1_writer.rs::{plan_write, persist_planned, apply_dedup_batch}` [V5] — no crear otro historial ni reusar el checkpoint global como log de conversación.
- **Acciones:**
  1. Resolver sesión autorizada.
  2. Capturar revisión al leer.
  3. Inferir fuera del lock.
  4. Append condicionado del turno.
  5. Emitir recibo. Si el seam disponible no ofrece CAS, serializar por sesión en host y declarar el límite entre procesos; no afirmar atomicidad por el nombre batch.
- **Dependencias:** contrato de sesión y ownership; X-LCH-05 comparte identidad de intento; converge con C13 de `CONSOLIDATED.md`. Alternativa mínima: un solo escritor por sesión en adapter.
- **FMEA:** retry duplica turno → `run_id`; fallo a mitad de input/output → commit de unidad o recovery declarado; contaminación entre sesiones → clave scoped.
- **Criterios de aceptación propuestos:** dos runs sobre la misma revisión no pierden inputs; retry y cancelación distinguen intento/confirmación; logs por revisión/IDs.
- **Criterio negativo:** el mismo run devuelve el mismo recibo; un conflicto no fusiona automáticamente respuestas producidas sobre contexto distinto; migración de eventos viejos no inventa `run_id` histórico.
- **Independencia:** sí — el informe declara que no depende de X-LCH-01/X-LCH-02 para funcionar; requiere contrato de sesión y ownership.
- **No objetivos:** replay de nodos ni resucitar memorias classic.

### X-LCH-05 — Contabilidad de intentos y retry en la frontera del host

- **Problema:** número de tools propuesto, despachado y realmente ejecutado difiere; un retry puede multiplicar coste o efectos sin aumentar turnos, y un interrupt de control-flow no es un error de tool.
- **Contrato hipotético (tipos propuestos):** recibo `InvocationUsage { operation_id, attempt_id, proposed, dispatched, outcome, usage_ref }` y clasificación `Retryable | Terminal | Interrupted | Unknown`. La política de límites sigue en el host; no se proponen números ni thresholds.
- **Fuente de evidencia (clone):** `libs/langchain_v1/langchain/agents/middleware/tool_call_limit.py:308-388` (`ToolCallLimitMiddleware.after_model:327-388`, contador por thread y por run) [C10]; `libs/langchain_v1/langchain/agents/middleware/tool_retry.py:295-353,387-413` (`ToolRetryMiddleware.wrap_tool_call`, no reintenta `GraphBubbleUp`) [C11].
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs` (`TaskCheckpointManager::begin/advance/fail` [V3] puede guardar progreso del trabajo); el host/adapter controla dispatch/retry. No se altera el presupuesto de contexto de `assemble_with_recall`.
- **Acciones:**
  1. Reservar el intento según política vigente.
  2. Registrar el despacho.
  3. Ejecutar.
  4. Registrar resultado/coste conocido.
  5. Decidir retry. Persistir separadamente el control-flow interrupt — no convertirlo en tool error; timeout después de aceptar remoto exige reconciliar antes de repetir acción no idempotente.
- **Dependencias:** owner de dispatch/cancelación en host; X-LCH-03 comparte operation_id/recibo (no su decisión); converge con C9/C13 de `CONSOLIDATED.md`. Alternativa preferida: middleware upstream + trazas de uso Vanta; implementar retry genérico en storage queda rechazado.
- **FMEA:** retry de interrupt → control flow separado; doble gasto por carrera → claim/revisión (el recibo evita dos dispatchers del mismo intento); presupuesto reiniciado al resume → distinguir run/thread; excepción sensible en mensaje → códigos/motivos redactados; wrapper sync usa `time.sleep` y async `asyncio.sleep` — no asumir la misma responsividad.
- **Criterios de aceptación propuestos:** intento bloqueado no aparece ejecutado; replay no duplica recibo; cancelación en backoff no despacha otra tool; fallo remoto ambiguo no es éxito.
- **Criterio negativo:** un interrupt de control-flow nunca se contabiliza como error de tool; datos históricos sin intentos quedan desconocidos, no se fabrican; el backoff debe poder detener nuevos intentos.
- **Independencia:** sí — contabilidad en la frontera host/adapter sobre `TaskCheckpointManager` existente; opt-in desactivable sin borrar recibos; no migrar contador de tokens a contador de tools.
- **No objetivos:** cuotas globales nuevas ni runtime Rust; no alterar presupuesto de contexto.

### X-LCH-06 — Score tipado y conformidad versionada del vector store

- **Problema:** un consumidor no debe interpretar distancia, similitud y score fusionado como probabilidad; UUID por texto+posición rompe el reemplazo al editar, y una mutación debe declarar reemplazo/parcial/retry por operación.
- **Contrato hipotético (tipos propuestos):** resultado con `score_kind/metric/raw_score` y conversión de relevancia explícita en adapter; `UnsupportedScoreMapping` si no existe transformación semántica defendible. Identidad documental y revisión del índice separadas del score; identidad estable por ID del caller.
- **Fuente de evidencia (clone):** `libs/core/langchain_core/vectorstores/base.py:234-291` (`add_documents` prioriza `kwargs.ids` sobre `Document.id`, delega en `add_texts` — sin garantía de atomicidad del conjunto) y `:450-540` (relevancia: convierte score nativo, advierte fuera de `[0,1]`, threshold posterior) [C6].
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/vectorstore.py:254-305` (`similarity_search_with_score`), `:566-576` (`_select_relevance_score_fn` — ya usa cosine y clamp: **no** reabrir el falso positivo de distancia invertida); LC-1 del informe: `:84-85` (`_build_key` UUID5 de texto+posición para `add_texts` sin IDs), `:414-448` (`add_texts`: valida longitudes antes del embedding, puts individuales), `:450-475` (`add_documents`: `Document.id` o UUID4 — no heredar la precedencia de Core sin comprobar colisión con `kwargs.ids`), `:477-503` (`delete`: itera keys, devuelve True también para lista vacía/None).
- **Acciones:**
  1. Inventariar scores por ruta.
  2. Declarar mapping por versión/métrica.
  3. Preservar IDs/metadata.
  4. Verificar filtros antes de top-k donde corresponda.
  5. Exponer la semántica; incorporar el recorrido de mutación LC-1: distinguir IDs explícitos, UUID5 texto/posición y UUID4; declarar reemplazo, parcial y retry por operación individual.
  6. No reindexar por añadir metadata del score; confirmar el contrato de la versión Core soportada, no asumir conformidad con la actual.
- **Dependencias:** versión Core soportada declarada; BM25/HNSW/RRF existentes; cosine/clamp ya existente (falso positivo cerrado). X-LCH-04 aporta identidad estable; converge con C11/C15 de `CONSOLIDATED.md`. Alternativa: devolver solo score raw y rechazar relevance donde no se pueda — no clamping silencioso de cualquier fusión como solución universal.
- **FMEA:** doble normalización → verificar extremos; RRF interpretado como confianza → tipo explícito; filtrado tardío pierde candidatos → prueba discriminante; ID derivado cambia al editar → versión duplicada → identidad estable del caller; embedding corto/fallo después del primer put → prefijo aplicado → no responder «nada escrito», conservar observación por operación en host.
- **Criterios de aceptación propuestos:** vectores conocidos y ruta híbrida mantienen el orden declarado; filtros/IDs sobreviven sync/async; unsupported es error visible, sin cambiar thresholds del benchmark. Casos futuros documentados: add/update del mismo ID con texto/metadata/vector nuevos; sin ID editado/reordenado; `kwargs.ids` frente a `Document.id`; cardinalidad incorrecta antes/después de embedding; fallo/cancel/retry después del primer put y delete parcial.
- **Criterio negativo:** `UnsupportedScoreMapping` nunca se resuelve con una transformación no defendible; cancelación de embedding no produce resultado «vacío exitoso»; delete concurrente no promete snapshot sin soporte probado.
- **Independencia:** sí — conserva y precisa la ficha previa de scores sin proponer índice nuevo; comparte identidad estable con X-LCH-04 pero no requiere su append.
- **No objetivos:** ranking, índice, WAL ni transacción paralelos.

### X-LCH-07 — Capa de identidad/touch opcional sobre scoring existente

- **Problema:** la semántica «último acceso» del buffer upstream solo existe en memoria (touch sin persistir, stream no durable); quien necesite esa semántica requiere una capa explícita que no confunda efímero con durable.
- **Contrato hipotético (tipos propuestos):** capa experimental de touch sobre `(scope, id, revisión)`: el host es dueño del estado, revalida permiso al tocar y declara si es efímera o persistida, con error explícito y sin cambiar fuente/`updated_at` por leer. Tipo no detallado en el informe — solo la semántica de la capa.
- **Fuente de evidencia (clone):** `libs/langchain/langchain_classic/retrievers/time_weighted_retriever.py:64-198` (`buffer_idx` resuelve contra `memory_stream`, no identidad durable; `_get_rescored_docs:115-132` toca `last_accessed_at` solo en buffer — el TODO no persiste; `add_documents:164-198` extiende `memory_stream` antes del vector store: si éste falla, buffer y store divergen) [LC-2].
- **Seam VantaDB:** `vanta-memory/src/core/record/scoring.rs:65-144` (`CompositeScoring`/`composite_score`/`composite_rank`, scoring opt-in con recencia basada en `updated_at`, no necesariamente `last_accessed_at`); adapters existentes — no repetir retrieval ni crear índice.
- **Acciones:**
  1. Rechazar el port del buffer, pesos e índice posicional upstream (decisión del informe).
  2. Definir el dueño del estado touch en el host.
  3. Declarar efímero vs persistido con error explícito.
  4. Revalidar permiso en cada touch.
  5. Rechazar referencia posicional no resoluble.
- **Dependencias:** scoring opt-in existente; decisión de persistencia del host; X-LCH-06 aporta la semántica de score; converge con C11 de `CONSOLIDATED.md`.
- **FMEA:** touch en buffer aparenta durabilidad → mostrar la frontera; store falla tras extender buffer → no afirmar publicación; índice posicional viejo → rechazar referencia no resoluble.
- **Criterios de aceptación propuestos:** fallo de persistencia, reinicio, ID retirado/ajeno y touch repetido conservan identidad/permiso y distinguen estado efímero/durable.
- **Criterio negativo:** un touch nunca muta fuente ni `updated_at` por el solo hecho de leer; lo efímero no se presenta como durable; referencia no resoluble se rechaza, no se reinterpreta.
- **Independencia:** sí — capa experimental opcional sobre scoring/adapters existentes; el informe declara explícitamente que no se adoptan scoring/configs/umbrales upstream ni se implementa la capa aún.
- **No objetivos:** portar el buffer, pesos o índice posicional de `time_weighted_retriever`; repetir retrieval ni crear índice.

[C10]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/tool_call_limit.py#L308-L388
[C11]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/tool_retry.py#L295-L413
[C13]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/summarization.py#L746-L1005
[C1]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/summarization.py#L485-L525
[C2]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/context_editing.py#L79-L155
[C3]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/human_in_the_loop.py#L759-L850
[C5]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/runnables/history.py#L522-L580
[C6]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/vectorstores/base.py#L450-L540
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L71-L266
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L188
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/approval.rs#L1-L168
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L176-L420

<!-- audit: 2026-11-23 justificada severidad — en X-LCH-07, completada ruta vanta-memory/src/core/record/approval.rs y precisados prefijos libs/ -->
