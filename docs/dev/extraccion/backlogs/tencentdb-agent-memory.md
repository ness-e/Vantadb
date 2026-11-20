---
title: "Backlog de extracción — TencentDB Agent Memory"
kind: research
status: active
description: "Ocho unidades candidatas del informe TencentDB Agent Memory (captura/cursor, préstamo, derivados, hooks, retención, recall y embeddings) como filas del esquema canónico de 10 columnas"
tags: [extraccion, backlog, tencentdb-agent-memory]
---

# Backlog de extracción — TencentDB Agent Memory

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/tencentdb-agent-memory.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `X-TAM-01` | 🟠 | Recibo de captura por etapa con cursor confirmado | `vanta-memory/src/core/conversation/l0_recorder.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: recibo que separa persistencia de mensajes, publicación de cursor y estado de derivados (léxico/embedding/extracción); el cursor sólo avanza tras confirmar el primario y `durable_revision` sale tras esa confirmación. · Por qué: confirmar captura no puede confundirse con que indexación y extracción terminaron; avanzar el cursor sobre un append fallido pierde evidencias y un retry sobre buffer efímero no es recuperable. · Acción mínima: mapear la confirmación actual de `record_turn`, conservar el `tie_break` ya existente para timestamps iguales y devolver resultado por etapa; reintentos sobre IDs, no buffers. · Trazabilidad: informe §2/Unidad A · evidencia `MemoryCore/src/utils/checkpoint.ts:735-768`, `MemoryCore/src/core/hooks/auto-capture.ts:1-180,285-347`, `MemoryCore/src/core/conversation/l0-recorder.ts:274-313` (clon `C:\Users\Eros\TencentDB-Agent-Memory`). | X-TAM-03 · C13 | Fuente/provenance/dedup existentes; cierre ordenado de tareas diferidas; no segundo scheduler ni ledger paralelo |
| `X-TAM-02` | 🟠 | Préstamo de activos autorizado con revisión y revocación | `vanta-memory/src/core/hooks/auto_recall.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: grant por activo/scope/revisión decidido por el backend y aplicado antes de expandir fuentes en recall; ver, usar y editar son permisos distintos y la revocación invalida cachés y derivados. · Por qué: compartir contenido entre agentes no puede ser inferencia de un toggle UI ni de un default permisivo — pertenencia al equipo, visibilidad e importación son decisiones separadas; un JSON corrupto no puede ampliar acceso. · Acción mínima: inventariar autorización actual, validar pertenencia con identidad confiable y aplicar el grant antes de búsqueda/expansión, con auditoría explicable. · Trazabilidad: informe §2/Unidad B · evidencia `MemoryPanel/src/panel/domain/chat-memory-governance.ts:1-107`. | X-TAM-07 · C16 | Requisito colaborativo explícito; catálogo de activos, identidad de actor confiable, provenance y revisión; Studio sólo presenta la decisión |
| `X-TAM-03` | 🟠 | Publicación de cursor de captura con exclusión RMW por sesión | `vanta-memory/src/core/conversation/l0_recorder.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: avance condicionado del cursor `l0_cursor/<sesión>` con expectativa de revisión; el siguiente valor no retrocede ni refiere evidencia no confirmada, y sesiones distintas no se bloquean entre sí. · Por qué: dos capturas de la misma sesión pueden leer el mismo cursor y publicar un avance obsoleto o regresivo; `TaskCheckpointManager::advance` es otro recurso y no guarda este cursor. · Acción mínima: delimitar exclusión por sesión en caller/recorder existente, revalidar cursor leído y mensajes confirmados antes de actualizar y reconciliar tras crash. · Trazabilidad: informe §11/Candidata C · evidencia `MemoryCore/src/utils/checkpoint.ts:150-375` (`mutate:364-375`). | X-TAM-01 | Cursor `l0_cursor/<sesión>` y sus callers (`auto_capture.rs`, `conversation_hook.rs`); serializar writer existente sin Redis obligatorio ni nueva cola |
| `X-TAM-04` | 🟡 | Publicación de derivados por generación con fence de revisión | `vanta-memory/src/core/reflection/mod.rs` · `vanta-memory/src/core/record/l1_writer.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: publicación de derivado ligada a revisión de fuente y generación, con outcome `Applied`/`Stale`/`Deleted`/`Cancelled`; una versión vieja nunca publica sobre la sucesora ni resucita lo borrado. · Por qué: un build tardío puede pisar un reemplazo más reciente o revivir un activo eliminado; un marker en RAM o un `ingestRunId` de progreso no es fencing durable. · Acción mínima: capturar generación al planificar, revalidar revisión/tombstone en el commit de publicación y registrar el estado; metadata aditiva y reindexado sólo del derivado cambiado. · Trazabilidad: informe §11/Candidata D · evidencia `MemoryKnowledge/src/store/wiki-service.ts:1350-1425`, `MemoryKnowledge/src/store/wiki-service.ts:423-530`, `MemoryKnowledge/src/store/build-queue.ts`. | X-TAM-01 · X-TAM-03 · X-TAM-06 | Scheduler/checkpoints, scenes/reflection y jobs existentes; no nuevo motor wiki ni cola por asset adicional |
| `X-TAM-05` | 🟡 | Reporte de capacidad efectiva del hook de contexto | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: outcome tipado de aplicación del contexto por etapa del hook (`effective`/`missing_field`/`empty_messages`/`n-a`) que distingue «configurado» de «entregado», sin contenido de mensajes en el reporte. · Por qué: si el host ignora la modificación de mensajes, el contexto queda ausente y la respuesta carece de evidencia sin que nadie lo observe; un warning textual es insuficiente. · Acción mínima: detectar la forma observada por etapa, confirmar la aplicación en el consumidor del host y reportar la degradación con un outcome explícito. · Trazabilidad: informe §11/Candidata E · evidencia `MemoryCore/src/offload/state-reporter.ts:61-90`. | X-TAM-07 | `IntegratedContext`/`CompactionReport` y assembly existentes; consumidor real del host instrumentado |
| `X-TAM-06` | 🟡 | Plan de retención con cierre de referencias por superficie | `vanta-memory/src/utils/erasure.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: planificación de retención por scope/política con resultado por superficie (`Deleted`/`Preserved`/`Pending`/`Failed`); jamás éxito físico por conteo incompleto. · Por qué: caducar el raw deja referencias, refs y contexto derivado accesibles; un fallo en una superficie tras borrar el raw crea huérfanos si se reporta éxito global. · Acción mínima: inventariar originales/derivados/cachés, preview, revalidar, borrar, invalidar y reconciliar fallos sobre el sweep/erasure existente. · Trazabilidad: informe §11/Candidata F · evidencia `MemoryCore/src/offload/reclaimer.ts:62-130`, `MemoryCore/src/utils/memory-cleaner.ts:69-155`. | X-TAM-04 · X-TAM-01 · X-RED-03 (retención, backlog redis) | Lifecycle/erasure existentes; política y permisos aprobados; TTL distinto de cuotas y budgets de recall |
| `X-TAM-07` | 🟡 | Outcomes de recall por fase con cancelación cooperativa | `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: resultado de recall por fase (`Ready`/`Empty`/`TimedOut`/`Denied`/`Failed`/`Cancelled`) preservando scope, revisión y presupuesto; el timeout no prueba que la operación perdida se abortó. · Por qué: un timeout o una lectura de persona fallida no pueden confundirse con contexto vacío; una carrera que no cancela amplifica el coste del proveedor al reintentar. · Acción mínima: mapear errores por fase, propagar la cancelación cooperativa existente, ensamblar una sola vez y reautorizar la entrega final. · Trazabilidad: informe §11/Candidata G · evidencia `MemoryCore/src/core/hooks/auto-recall.ts:92-242`. | X-TAM-05 · X-TAM-02 | Budgets, governed recall e híbrido existentes; cancelación/admisión vigente del consumidor; no nuevo ranker |
| `X-TAM-08` | 🟠 | Validación de lote de embeddings por contrato de proveedor | `vanta-memory/src/core/record/l1_writer.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: validación de la respuesta de embeddings (modelo/revisión/dimensión/cardinalidad/IDs de entrada) antes de persistir; errores tipados, nunca vector cero ni correspondencia inferida por orden sin comprobar unicidad. · Por qué: ordenar por `index` sin validar duplicados puede asociar el vector de otro texto y producir recall falso; truncar o hacer padding para autodetectar dimensión mezcla espacios vectoriales distintos. · Acción mínima: identificar modelo/config, validar orden/cardinalidad/dimensión en el adapter del borde y preservar el fallback textual permitido con la degradación registrada. · Trazabilidad: informe §11/Candidata H · evidencia `MemoryCore/src/core/store/embedding.ts:520-621`. | X-TAM-07 · X-TAM-01 | Hook de embeddings opcional y validación vectorial existentes; configuración confiable del proveedor, nunca texto del agente |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/tencentdb-agent-memory.md` (Unidades A–H). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `2002196` rama `feat/server_team` (`C:\Users\Eros\TencentDB-Agent-Memory`); las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD. Licencia local MIT vs metadatos `NOASSERTION`: discrepancia documentada sin sobreinterpretar.

### X-TAM-01 — Recibo de captura por etapa con cursor confirmado

- **Problema:** confirmar la captura puede confundirse con que toda la indexación y extracción haya terminado; avanzar el cursor sobre un append fallido pierde evidencias; un retry sobre un buffer efímero no es recuperable.
- **Contrato hipotético:** `CaptureReceipt { source, session, cursor, durable_revision, lexical_state, embedding_state, derivation_state }`; `capture_batch(ScopedBatch) -> Result<CaptureReceipt, CaptureError>` con `InvalidEvent | Denied | AppendFailed | CheckpointFailed | Cancelled`. Invariante: `durable_revision` sólo sale tras confirmar el primario.
- **Fuente de evidencia (clone):** `MemoryCore/src/utils/checkpoint.ts:735-768` (`captureAtomically` con callback), `MemoryCore/src/core/hooks/auto-capture.ts:1-180,285-347` (`performAutoCapture`, resultados por etapa), `MemoryCore/src/core/conversation/l0-recorder.ts:274-313`.
- **Seam VantaDB:** `vanta-memory/src/core/conversation/l0_recorder.rs` (`L0Recorder::record_turn` y `read_cursor`/`write_cursor`); el checkpoint de tarea y el merge wiki no publican ese cursor.
- **Acciones:**
  1. Mapear la confirmación actual de la ruta L0.
  2. Conservar `tie_break` para timestamps iguales.
  3. Distinguir publicación de mensajes/cursor y progreso L1.
  4. Separar disponibilidad de evidencia e índice.
  5. Registrar fallos recuperables; el scheduler existente reintenta sobre IDs, no buffers.
- **Dependencias:** fuente/provenance/dedup existentes; cierre ordenado de tareas diferidas; X-TAM-03, C13; no segundo scheduler ni ledger paralelo.
- **FMEA:** cursor avanza antes de write durable; mismo timestamp omite mensajes; retry duplica; cold-start floor pierde backlog; cierre destruye índice mientras embedding sigue activo. Complemento del informe: append falla → callback devuelve mensajes → cursor avanza (detectar receipt/cursor divergentes; reconciliar por ID).
- **Criterios de aceptación propuestos:** la confirmación distingue persistencia de derivados; retry converge a las mismas evidencias; fallo de embeddings conserva recuperación textual; interrupción no presenta lote incompleto como íntegro; shutdown identifica trabajo pendiente. Adversa: replay, timestamp empatado, append fallido, caída entre primario/cursor, embedding fallido.
- **Criterio negativo:** el cursor nunca avanza sobre un append fallido; «capturado» nunca equivale a indexado/extraído.
- **Independencia:** mínimo adapter sobre L0 existente; abierto el commit del storage concreto.
- **No objetivos:** copiar IDs aleatorios ni el suelo de arranque como política obligatoria; no segundo scheduler ni otro archivo de checkpoints.

### X-TAM-02 — Préstamo de activos autorizado con revisión y revocación

- **Problema:** compartir contenido entre agentes no puede inferirse de un toggle UI ni de un default permisivo; pertenencia al equipo, visibilidad e importación son decisiones separadas; un JSON corrupto no puede ampliar acceso.
- **Contrato hipotético:** `AssetGrant { asset_id, owner_scope, reader_scope, asset_kind, revision, decision_source }`; `BorrowRequest {actor, objetivo, expected_revision}`; `authorize_borrow(request) -> Result<Grant, PolicyError>` con `Denied | UnknownAsset | StalePolicy | Unavailable`. Invariantes: nunca distingue existencia de activo ajeno; el límite de dos upstream no se propone como universal.
- **Fuente de evidencia (clone):** `MemoryPanel/src/panel/domain/chat-memory-governance.ts:1-107` (DTO y `validateImportedAgents`; `readChatMemoryRel` devuelve default con sharing `true` ante ausencia/JSON inválido — riesgo fail-open condicionado).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` (`perform_auto_recall_governed`, antes de expandir fuentes); frontera de scopes de memoria/escenas/skills y adaptadores server/MCP; Studio sólo presenta la decisión.
- **Acciones:**
  1. Inventariar la autorización actual entre scopes.
  2. Distinguir ver/usar/editar.
  3. Validar pertenencia con identidad confiable.
  4. Persistir cambios con revisión.
  5. Aplicar el grant antes de búsqueda/expansión.
  6. Revocar cachés y derivados al retirar el grant.
  7. Auditoría explicable sin filtrar existencia de activos ajenos.
- **Dependencias:** requisito colaborativo explícito; catálogo de activos; identidad de actor confiable; provenance y revisión; X-TAM-07, C16; `InjectionPolicy`/`RecallScope` existentes (delta: préstamo entre activos, no ACL desde cero).
- **FMEA:** default compartido abre datos; metadata corrupta habilita permisos; equipo cambia durante préstamo → grant cacheado → lectura indebida (detectar `policy_revision`, reautorizar en entrega); UI oculta activo pero retrieval lo entrega; snapshot/cache conserva acceso revocado.
- **Criterios de aceptación propuestos:** actor de otro equipo no obtiene contenido; configuración inválida no amplía acceso; autoimportación y duplicados se rechazan; revocación afecta recall y preview; revisiones en conflicto no pisan concesiones; el error explica política sin divulgar activos ajenos; editar y usar son permisos distintos.
- **Criterio negativo:** un JSON corrupto o ausente nunca amplía acceso; una revocación durante recall nunca entrega contenido.
- **Independencia:** sólo si la colaboración es requisito (P0 de frontera); migración con grants aditivos — el legado no aumenta acceso; rollback revoca grants nuevos e invalida cachés, no recupera contenido ya entregado.
- **No objetivos:** trasladar el modelo provisional en JSON ni una plataforma de identidad/administración empresarial.

### X-TAM-03 — Publicación de cursor de captura con exclusión RMW por sesión

- **Problema:** dos capturas de la misma sesión pueden leer el mismo cursor y publicar un avance obsoleto o regresivo; no se declara carrera Vanta reproducida ni cursor global — es una frontera read-modify-write por recurso.
- **Contrato hipotético:** `advance_capture(expected: CursorRevision, next: ConfirmedCursor) -> Result<CursorRevision, CursorError>`; errores `Stale | Storage | Cancelled`. Invariante: `next` no retrocede ni refiere evidencia no confirmada.
- **Fuente de evidencia (clone):** `MemoryCore/src/utils/checkpoint.ts:150-375` (`mutate`: lock local por archivo, lock externo opcional, read→callback→write; tmp+rename no es transacción multirrecurso).
- **Seam VantaDB:** `vanta-memory/src/core/conversation/l0_recorder.rs` (`record_turn` → `read_cursor`, `write_cursor`); callers `core/hooks/auto_capture.rs` y `services/conversation_hook.rs`. `TaskCheckpointManager::advance` es otro recurso; el helper global `utils/checkpoint.rs` tampoco es este cursor.
- **Acciones:**
  1. Delimitar la exclusión por sesión en el caller/recorder existente.
  2. Confirmar X-TAM-01 antes de publicar.
  3. Revalidar el cursor leído y los mensajes confirmados antes de actualizar.
  4. Reconciliar tras crash. Metadata revisionada; sin reindexado; rollback pausa callers nuevos, no restaura cursor antiguo sobre evidencia nueva.
- **Dependencias:** X-TAM-01; ownership de captura en caller/`L0Recorder`; `TaskCheckpoint` sólo progreso/reconciliación si se acredita el vínculo; no Redis obligatorio ni nueva cola.
- **FMEA:** dos capturas leen igual cursor → avance obsoleto/regresión (detectar conflicto por sesión, mitigar en la publicación del cursor L0); mensaje persistido y cursor fallido → replay (conservar `tie_break` y recibo parcial de X-TAM-01; no «arreglar» `step/partial` esperando proteger otro recurso).
- **Criterios de aceptación propuestos:** normal/paralela, interrupción y lock perdido; cancelación libera guard y no anuncia avance; sesiones distintas no se bloquean; avanzar checkpoint de tarea no cuenta como avance de captura; fallo de cursor no se presenta como nada escrito ni como extracción L1 terminada. Métricas wait/conflict sin payload.
- **Criterio negativo:** un avance nunca retrocede el cursor ni refiere evidencia no confirmada; el checkpoint de tarea nunca cuenta como cursor de captura.
- **Independencia:** una frontera RMW sobre el recurso existente; no WAL/scheduler nuevo; abierta la primitiva CAS real.
- **No objetivos:** no WAL/scheduler nuevo ni Redis obligatorio.

### X-TAM-04 — Publicación de derivados por generación con fence de revisión

- **Problema:** un build tardío puede pisar un reemplazo más reciente o revivir un activo eliminado; un marker en RAM o `ingestRunId` de progreso no es fencing durable.
- **Contrato hipotético:** `publish_derived(source: SourceRevision, generation: u64, output: DerivedCandidate) -> Result<PublishOutcome, PublishError>`; outcome `Applied | Stale | Deleted | Cancelled` y storage. Invariante: una versión vieja nunca publica sobre la sucesora.
- **Fuente de evidencia (clone):** `MemoryKnowledge/src/store/wiki-service.ts:1350-1425` (`runBuild`, delete/finishCancelled), `MemoryKnowledge/src/store/wiki-service.ts:423-530`, `MemoryKnowledge/src/store/build-queue.ts` (cola por asset en RAM).
- **Seam VantaDB:** `vanta-memory/src/core/reflection/mod.rs` (`reflect_session`/`ReflectionRun`), `vanta-memory/src/core/record/l1_writer.rs`; scenes/reflection/generation log y jobs existentes.
- **Acciones:**
  1. Capturar la generación al planificar.
  2. Inferir sin mantener locks.
  3. Revalidar revisión/tombstone en el commit de publicación.
  4. Publicar.
  5. Registrar el estado. Metadata aditiva; reindexar sólo el derivado cambiado; rollback pausa publicación nueva y preserva la última versión válida, no revierte purga.
- **Dependencias:** scheduler/checkpoints, scenes/reflection y jobs existentes; X-TAM-01/X-TAM-03 confirman origen/cursor; X-TAM-06 invalida referencias tras retención; no nuevo motor wiki ni cola por asset.
- **FMEA:** cancelación sólo en RAM → restart pierde marker → output viejo (detectar revisión/tombstone en commit, no confiar en ausencia en cola local).
- **Criterios de aceptación propuestos:** normal ready; replace/delete durante worker; crash tras archivos antes de metadata; cancelar no transforma fallo en ready. Métricas por fase.
- **Criterio negativo:** una versión vieja nunca publica sobre la sucesora ni resucita lo borrado; un marker en RAM nunca se presenta como fencing durable.
- **Independencia:** contrato de publicación mínimo sobre jobs existentes; abiertos la atomicidad metadata/archivos y la compensación de outputs parciales.
- **No objetivos:** no nuevo motor wiki ni cola por asset adicional.

### X-TAM-05 — Reporte de capacidad efectiva del hook de contexto

- **Problema:** si el host ignora la modificación de mensajes, el contexto queda ausente y la respuesta carece de evidencia sin que nadie lo observe; «configurado» no equivale a «entregado»; un warning textual es insuficiente.
- **Contrato hipotético:** `inspect_hook(stage: HookStage, observed: HookShape) -> HookCapabilityReport`; entradas sin contenido; salida estado+cantidad (`effective | missing_field | empty_messages | n-a`); malformed explícito; no afirmar entrega desde la presencia de un campo.
- **Fuente de evidencia (clone):** `MemoryCore/src/offload/state-reporter.ts:61-90` (`classifyPatchEffectiveness` por etapa; no acredita patch global del runtime).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` (`assemble_with_recall`) y consumidor del host; `IntegratedContext`/`CompactionReport` existentes.
- **Acciones:**
  1. Detectar la forma observada por etapa.
  2. Ensamblar con la ruta existente.
  3. Confirmar la aplicación por el host.
  4. Reportar la degradación. Efímero/versionado, sin migración ni reindexado; rollback omite el reporte nuevo, no modifica mensajes.
- **Dependencias:** `IntegratedContext`/`CompactionReport` y assembly existentes; consumidor real del host instrumentado; X-TAM-07 da outcomes de recall; X-TAM-01 no sustituye recibo de entrega.
- **FMEA:** host ignora la modificación → contexto ausente → respuesta sin evidencia (detectar ack/capacidad observada; fallback explícito autorizado).
- **Criterios de aceptación propuestos:** forma válida/ausente/vacía y cancelación; no medir «ahorro» cuando no aplicado. Contadores forma/etapa sin mensajes.
- **Criterio negativo:** «configurado» nunca se reporta como «entregado»; la presencia de un campo nunca acredita la aplicación.
- **Independencia:** diagnóstico mínimo; no parche de runtime; abierto el ack concreto por host.
- **No objetivos:** outcome tipado, no warning textual; no assembler alternativo ni parche de runtime.

### X-TAM-06 — Plan de retención con cierre de referencias por superficie

- **Problema:** caducar el raw deja referencias, refs y contexto derivado accesibles; un fallo en una superficie tras borrar el raw crea huérfanos si se reporta éxito global.
- **Contrato hipotético:** `plan_retention(scope: &RecallScope, now_ms: u64, policy: RetentionPolicy) -> Result<RetentionPlan, RetentionError>`; apply devuelve por superficie `Deleted | Preserved | Pending | Failed`; errores `denied | stale | storage`; jamás éxito físico por conteo incompleto.
- **Fuente de evidencia (clone):** `MemoryCore/src/offload/reclaimer.ts:62-130` (`reclaimOffloadData`: JSONL→refs→MMD→logs→registro, errores independientes), `MemoryCore/src/utils/memory-cleaner.ts:69-155` (`MemoryCleaner.runOnce`: limpia L0/L1 por día local, luego DB con mínimos — políticas diferentes).
- **Seam VantaDB:** `vanta-memory/src/utils/erasure.rs` (`erase_scope`, `emit_cascade_erasure_receipt`), scheduler; lifecycle/erasure existentes.
- **Acciones:**
  1. Inventario original/derivado/cache.
  2. Preview.
  3. Revalidar.
  4. Borrar por superficie.
  5. Invalidar referencias.
  6. Reconciliar fallos. Metadata aditiva y reindexado de afectados; desactivar detiene sweeps futuros, no restaura lo purgado.
- **Dependencias:** lifecycle/erasure existentes; política y permisos aprobados; X-TAM-04 conoce generación; X-TAM-01 conserva tombstone/replay; X-RED-03 (retención, backlog redis); TTL distinto de cuotas y budgets de recall.
- **FMEA:** mtime o zona local mal interpretados → borrado prematuro (detectar cutoff/origen temporal; semántica explícita/revisión); fallo en refs tras raw borrado → huérfano (reportar pendiente, no éxito global).
- **Criterios de aceptación propuestos:** activo preservado, vencido retirado, ref aún usada preservada, error parcial/restart/cancelación sin resurrección; métricas por superficie.
- **Criterio negativo:** ninguna superficie fallida se reporta como éxito global; ningún conteo incompleto se presenta como purga física.
- **Independencia:** retención de una clase aprobada sobre sweep/erasure existente; abierto política UTC/local y legal.
- **No objetivos:** no garantías sobre backups ni importar mínimos/tres días upstream; no TTL mágico global.

### X-TAM-07 — Outcomes de recall por fase con cancelación cooperativa

- **Problema:** un timeout o una lectura de persona fallida no pueden confundirse con contexto vacío; una carrera que no cancela amplifica el coste del proveedor al reintentar.
- **Contrato hipotético:** `recall_scoped(request: ScopedRecall, cancel: CancelSignal) -> Result<RecallPhaseReport, RecallError>`; `Ready | Empty | TimedOut | Denied | Failed` por fase (+ Cancelled); scope/revisión/budget preservados. Invariante: timeout no prueba que la operación perdida se abortó.
- **Fuente de evidencia (clone):** `MemoryCore/src/core/hooks/auto-recall.ts:92-242` (`performAutoRecall`, core y wrapper con race sin cancelación cooperativa).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` (`perform_auto_recall_governed`/`RecallConfig`), assembly en `engine.rs`; budgets, governed recall e híbrido existentes — ampliar discriminación por fase, no nuevo ranker.
- **Acciones:**
  1. Mapear errores por fase.
  2. Propagar cancelación cooperativa.
  3. Ensamblar una sola vez.
  4. Reautorizar delivery. DTO efímero, sin reindexado; rollback render compatible.
- **Dependencias:** budgets, governed recall e híbrido existentes; cancelación/admisión vigente del consumidor; X-TAM-05 observa aplicación; X-TAM-02 autoriza fuentes; no nuevo ranker.
- **FMEA:** `Promise.race` pierde → operación continúa → retry amplifica coste (detectar in-flight abandonado; cancelación/admisión existente; no prometer devolución de coste de proveedor ya enviado).
- **Criterios de aceptación propuestos:** normal, vacío, persona caída, query caída, timeout/revocación; `Cancelled` distinto de `Empty`; conteos/latencia/outcomes sin query.
- **Criterio negativo:** timeout/fallo de una fase nunca se presenta como `Empty`; una señal de cancelación nunca afirma que la operación se abortó.
- **Independencia:** outcomes mínimos sobre recall existente; abierta cancelación por driver.
- **No objetivos:** no nuevos pesos ni benchmark; no reemplazar try/catch por fail-open.

### X-TAM-08 — Validación de lote de embeddings por contrato de proveedor

- **Problema:** el modelo remoto puede rechazar `dimensions` o devolver un batch incompatible; ordenar por `index` sin validar duplicados puede asociar el vector de otro texto y producir recall falso; truncar/pad para autodetectar dimensión mezcla espacios vectoriales.
- **Contrato hipotético:** `validate_embedding_batch(expected: EmbeddingBatchContract, response: Vec<EmbeddingItem>) -> Result<ValidatedBatch, EmbeddingError>`; modelo/revisión/dimensión/IDs de entrada; output misma correspondencia; errores `dimension | missing | duplicate | nonfinite | denied | cancelled`.
- **Fuente de evidencia (clone):** `MemoryCore/src/core/store/embedding.ts:520-621` (`_callApi`: dimensions condicional, proxy QClaw, timer/retry, ordena por index y normaliza la respuesta).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` (`core_embedding_hook`/`EmbedFn` y `write_memory`); provider en el borde; embeddings opcionales y validación vectorial existentes.
- **Acciones:**
  1. Identificar modelo/config.
  2. Validar orden/cardinalidad/dimensión en el adapter del borde.
  3. Preservar fallback textual permitido.
  4. Registrar la degradación. Metadata aditiva; cambiar modelo requiere reembedding explícito separado; rollback mantiene índice compatible.
- **Dependencias:** hook de embeddings opcional y validación vectorial existentes; configuración confiable del proveedor (nunca texto del agente); X-TAM-07 expone degradación; X-TAM-01 confirma el raw aunque el vector falle.
- **FMEA:** sort por index sin validar duplicados → vector de otro texto → recall falso (detectar unicidad/cardinalidad; rechazo antes de persistir).
- **Criterios de aceptación propuestos:** normal, dimensiones incompatibles, duplicado/NaN/timeout/429; cancelación no publica vector parcial; métricas por error sin input/token.
- **Criterio negativo:** nunca vector cero ni correspondencia inferida por orden sin comprobar unicidad; vectores de espacios distintos nunca se mezclan.
- **Independencia:** validación mínima del adapter; no modelo nuevo ni reindexación automática; abierto contrato de versiones y permisos del proveedor.
- **No objetivos:** no autodetectar dimensión por truncar/pad ni convertir fallo a vector cero; no otro motor de embeddings.
