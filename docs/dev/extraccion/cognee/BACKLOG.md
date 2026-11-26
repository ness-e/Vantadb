---
title: "Backlog de extracción — Cognee"
kind: research
status: active
description: "Candidatas del informe cognee: readiness por hitos, ownership de derivados, resultado por etapa, orden de locks, update incremental, contexto autorizado, promoción de sesiones y canales tipados"
tags: [extraccion, backlog, cognee]
---

# Backlog de extracción — Cognee

> **Origen:** informe `docs/dev/extraccion/cognee/COGNEE.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-COG-01 | 🟠 | Disponibilidad progresiva por hitos de revisión | `vanta-memory/src/ingest/worker.rs`; `vanta-memory/src/services/pipeline_worker.rs`; `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `MemoryReadiness { source_id, source_revision, index_generation, chunk_state, enrichment_state, failures }` por documento; «stored» no implica «searchable» ni «enriched». Por qué: un agente puede preguntar antes de que termine el enriquecimiento; servir todo sin indicar generación mezcla revisiones y anunciar grafo listo al guardar el original miente. Acción mínima: asociar hitos al checkpoint existente, publicar índice temprano solo tras confirmación del primario, enriquecer fuera de la mutación crítica y mostrar estado en actividad existente. Trazabilidad: informe §Ficha A · evidencia `cognee/api/v1/cognify/cognify.py:543-767` | X-COG-02 protege ownership; X-COG-03 registra etapas; X-COG-04 controla publicación concurrente; X-COG-08 consume readiness | Inventario de estados de `ingest/worker.rs` y `MemoryTaskHandler`; migración aditiva con legado «desconocido», no ready por defecto |
| X-COG-02 | 🟠 | Procedencia con múltiples propietarios y recibo de borrado | `vanta-memory/src/core/record/l1_writer.rs` (`plan_write`, `persist_planned`, `apply_dedup_batch`); `vanta-memory/src/core/abstractions/types.rs` (`MemoryRecord`) | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: `SourceRef` versionado, `DerivedOwners` y `SourceRemovalReceipt { detached, removed, preserved_shared, pending_projections, verification }`; borrar una fuente desenlaza ownership pero no destruye el derivado mientras queden respaldos válidos. Por qué: dos documentos pueden respaldar la misma relación; desenlazar una fuente no equivale a destruir el hecho y un recibo distingue aproximado de verificado. Acción mínima: inventario de cardinalidad, refs versionadas con preview, quitar ref, borrar derivado solo sin respaldos, invalidar índices y confirmar recibo. Trazabilidad: informe §Ficha B · evidencia `cognee/tasks/storage/add_data_points.py:74-143,307-363`, `cognee/api/v1/datasets/datasets.py:457-575` | X-COG-04 da exclusión; X-COG-06 da alcance; motivada por issues #3235 y #1366; frontera `apply_dedup_batch` delete→put_batch citada en §9 | X-COG-06 (alcance) y X-COG-04 (exclusión real); primario+refs necesitan frontera coherente; `put_batch` no hace atómico el delete previo del writer |
| X-COG-03 | 🟠 | Resultado de enriquecimiento por etapa | `vanta-memory/src/services/pipeline_worker.rs` (`PipelineWorker::run_task`, `TaskHandler::handle`); generation log | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `EnrichmentOutcome { operation_id, source_revision, stages, resumable_from, usage }` donde cada etapa distingue `Applied`/`Unchanged`/`Skipped`/`Failed`/`UnknownCommit` con motivo y fatalidad propias. Por qué: «el job terminó» no dice si feedback, Q&A o conflictos se publicaron; un fallo no fatal no debe contarse como éxito ni un agotamiento de presupuesto como dato vacío. Acción mínima: mapear resultados existentes sin perder errores, persistir tras cada efecto confirmado y avanzar cursor solo con la confirmación correspondiente. Trazabilidad: informe §Ficha C · evidencia `cognee/api/v1/improve/improve.py:152-206,219-284` | X-COG-01 expone disponibilidad; X-COG-07 usa sus cursores; converge con C13 MutationReceipt (propuesto) | Retries/leases/dead letters existentes en `PipelineWorker`; idempotencia de cada efecto; `Result<(), String>` de `TaskHandler::handle` no acredita resultado durable por subetapa |
| X-COG-04 | 🟡 | Orden de locks declarado como contrato | `vanta-memory/src/services/pipeline_worker.rs`; `vanta-memory/src/services/scheduler.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: orden contractual de adquisición (p. ej. recurso→permiso) con guardias no transferibles implícitamente a hijos y errores `LockOrderViolation`, `Cancelled`, `LeaseLost`. Por qué: un job que retiene un permiso limitado esperando un recurso cuyo escritor espera ese permiso produce deadlock; un mutex local no sustituye fencing entre procesos. Acción mínima: dibujar orden real de sesión/storage/índice/recursos externos, adquirir en orden total, publicar en sección corta y liberar en RAII sin conservar transacción mientras se espera LLM. Trazabilidad: informe §Ficha D · evidencia `cognee/infrastructure/locks/dataset_lock.py:1-133` | X-COG-02/X-COG-05 usan su exclusión; X-COG-03 valida fence al commit | Inventario de primitivas reales (locks de sesión, leases, scheduler); identidad de ejecución explícita; despliegue coherente de callers si cambia el orden |
| X-COG-05 | 🟡 | Actualización incremental con plan validado completo | `vanta-memory/src/ingest/worker.rs`; `vanta-memory/src/core/record/l1_writer.rs` | 🟠 1-2sem | 🟡 P2 | ⬜ Candidata | Qué: `ChunkUpdatePlan { base, policy_version, kept, moved, replaced, fresh, invalidated_dependencies }` y `DocumentUpdateOutcome::{Applied,Unchanged,RebuildRequired,Conflict,Partial}` conservando identidad del documento. Por qué: editar una página no debe reinterpretar silenciosamente todos los chunks; un plan debe cubrir el documento nuevo bajo el contrato del chunker o caer a rebuild declarado, nunca a adivinar. Acción mínima: registrar versión del chunker, plan puro validado por reconstrucción, preparar embeddings fuera del commit, revalidar baseline y publicar con manifest de generación. Trazabilidad: informe §Ficha E · evidencia `cognee/api/v1/update/incremental.py:662-750,864-925,1063-1210`, `cognee/api/v1/update/update.py:219-349` | X-COG-01 muestra readiness; X-COG-02 mantiene ownership; X-COG-04 da exclusión | Condicional a documentos mutables; dedup/supersession existentes; no usar hash de texto como única revisión de trabajo concurrente |
| X-COG-06 | 🔴 | Contexto autorizado y capacidad declarada del backend | `vanta-memory/src/core/hooks/auto_recall.rs` (`InjectionPolicy`, `RecallConfig`); tenancy del writer (`l1_writer.rs`) | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `AuthorizedOperationContext { principal, scope, authorization_revision, operation_id }` creado solo por frontera gobernada y `BackendMemoryCapabilities` que declara garantías verificables; `UnsupportedIsolation`, `ScopeDenied`, `StaleAuthorization`, `CapabilityUnavailable`. Por qué: la configuración de una conexión no concede permisos y un backend sin soporte no hereda aislamiento por su interfaz; filtrar después de recuperar no repara el envío previo al LLM. Acción mínima: inventariar rutas SDK/MCP/HTTP/ingest, comprobar acceso antes del fan-out y al commit de jobs largos, propagar contexto inmutable y exponer ausencia de capacidad. Trazabilidad: informe §Ficha F · evidencia `cognee/modules/search/methods/search.py:220-288`, `cognee/infrastructure/databases/dataset_database_handler/supported_dataset_database_handlers.py:35-85`, `cognee/context_global_variables.py:95-125` | Gobierna X-COG-01…X-COG-08; motivada en parte por issue #2291 | Identidad existente; legado requiere política explícita; revocar permiso invalida publicación pendiente; desconocido nunca se trata como permitido |
| X-COG-07 | 🟡 | Promoción de sesión con watermark de consumo explícito | `vanta-memory/src/services/pipeline_worker.rs` (`MemoryTaskHandler`); `vanta-memory/src/core/reflection/mod.rs` | 🟠 1-2sem | 🔵 P3 | ⬜ Candidata | Qué: `PromotionCursor { source_session, source_revision, consumer_scope, policy_version, last_confirmed_unit }` y `PromotionOutcome { applied, already_consumed, skipped, failed }` por consumidor y destino explícito. Por qué: un cursor global avanzado por un consumidor hace perder datos al segundo; «ya consumido en otro destino» debe mostrarse, no aparecer como éxito vacío. Acción mínima: comparar cursores existentes, definir unidad de consumo y destino, validar revisión/permisos y confirmar outputs y avance de cursor coherentemente. Trazabilidad: informe §Ficha G · evidencia `cognee/api/v1/improve/improve.py:152-206` | X-COG-02/X-COG-03/X-COG-06 son prerequisitos | X-COG-02, X-COG-03, X-COG-06; cursor por consumidor aumenta retención y obligaciones de revocación; opt-in posterior |
| X-COG-08 | 🟡 | Canales tipados y referencias de evidencia sin answerer | `vanta-memory/src/context_engine/engine.rs` (`assemble_with_recall`, `IntegratedContext`) | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: `EvidenceBundle { source_revision, channel, references, truncations, channel_status }` que distingue `EmptySource`, `NoMatches`, `ChannelFailed` y `BudgetTruncated`, devolviendo referencias vigentes y texto gobernado sin inferencia obligatoria. Por qué: un fallo de rama no debe leerse como cero hits y el fan-out no rebasa el presupuesto global; contexto obtenido y respuesta generada son operaciones distintas. Acción mínima: inventariar explain actual, preservar identidad de fuente al unir canales, aplicar allowlist antes de serializar y reportar truncamiento en el assembler. Trazabilidad: informe §Ficha H · evidencia `cognee/modules/retrieval/hybrid_retriever.py:120-208` | X-COG-06 autoriza; X-COG-01 aporta readiness; lectura independiente de escritura | X-COG-06 y X-COG-01; read-only sin migración ni reindexación; top-k por canal no sustituye budget global |

## Fichas completas

> Expansión de cada candidata a partir del informe `cognee/COGNEE.md` (§Fichas A–H). Todos los contratos son **hipotéticos**: propuestas, no APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `8853378e`; las rutas VantaDB citan el snapshot `97c9623` y requieren revalidación contra HEAD.

### X-COG-01 — Disponibilidad progresiva por hitos de revisión

- **Problema:** un agente sube un manual y pregunta antes de que termine su enriquecimiento; «stored» no implica «searchable» ni «enriched», y servir todo sin indicar generación mezcla revisiones. Hay estados utilizables intermedios, no una operación indivisible.
- **Contrato hipotético:** `MemoryReadiness { source_id, source_revision, index_generation, chunk_state, enrichment_state, failures }`; entrada: documento autorizado y revisión esperada; salida: hitos confirmados y trabajo pendiente — `StaleRevision`, `IndexUnavailable`, `Cancelled` y fallo de proveedor no son «sin contenido». Cada hito pertenece a una misma revisión.
- **Fuente de evidencia (clone):** `cognee/api/v1/cognify/cognify.py:543-767` (`get_indexing_tasks`/`get_default_tasks` separan índice temprano y extracción de grafo).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs`, `vanta-memory/src/services/pipeline_worker.rs` (`PipelineWorker::run_task`, `MemoryTaskHandler`), `vanta-memory/src/utils/task_checkpoint.rs` — extender el resultado existente por revisión, no otro pipeline. No se afirma que falten todos los estados propios: estos seams son los que hay que comparar.
- **Acciones:**
  1. Inventariar estados de `ingest/worker.rs` y `MemoryTaskHandler`.
  2. Asociar los hitos al checkpoint existente.
  3. Publicar índice temprano solo tras confirmación del primario (el primario es autoridad; proyecciones fallidas quedan pendientes).
  4. Enriquecer fuera de la mutación crítica; confirmar generación y resultado.
  5. Mostrar el estado en la actividad existente; migración aditiva con legado «desconocido», nunca ready por defecto.
- **Dependencias:** X-COG-02 protege ownership; X-COG-03 registra etapas; X-COG-04 controla publicación concurrente; X-COG-08 consume readiness. Sin reembebido obligatorio al completar grafo; cambiar chunker/extractor exige generación nueva; rollback desactiva el productor nuevo sin eliminar originales confirmados.
- **FMEA:** hito anticipado → consulta vacía presentada como ready (detectar generación no visible y confirmar antes de anunciar); revisión mezclada → respuesta obsoleta (comparar revisión de fuente/índice); reintento de grafo → embeddings duplicados (reutilizar identidad y recibo de etapa); dos jobs no promocionan una revisión antigua sobre la nueva; cancelar entre hitos preserva lo confirmado y deja reanudación identificada.
- **Criterios de aceptación propuestos:** detener el enriquecimiento y recuperar chunks de la revisión confirmada con estado parcial; reanudar sin duplicar; editar/borrar la fuente mientras trabaja y rechazar el resultado viejo; telemetría de hito/revisión/conteos/error, sin texto.
- **Criterio negativo:** una proyección pendiente nunca se anuncia como disponible; una revisión antigua nunca se promociona sobre la vigente.
- **Independencia:** delta sobre ingesta/workers/checkpoints existentes; alternativa documentada: esperar a todo (simplifica consistencia pero oculta chunks disponibles).
- **No objetivos:** no importar el runner Python, no inventar SLA ni prometer menor latencia.

### X-COG-02 — Procedencia con múltiples propietarios y recibo de borrado

- **Problema:** dos documentos pueden respaldar la misma relación; borrar el primero no debe eliminar el conocimiento del segundo. Desenlazar una fuente no equivale a destruir el hecho, y un recibo debe distinguir aproximado de verificado.
- **Contrato hipotético:** `SourceRef { scope, source_id, source_revision, chunk_id }`, `DerivedOwners` y `SourceRemovalReceipt { detached, removed, preserved_shared, pending_projections, verification }`; entrada: intención autorizada, revisión e ID de operación; salida por recurso, no éxito vacío. Refs únicas, nunca compartidas entre ámbitos incompatibles; `NotFound`, `Conflict`, `ProjectionFailed`, `VerificationIncomplete` son resultados distintos.
- **Fuente de evidencia (clone):** `cognee/tasks/storage/add_data_points.py:74-143,307-363` (adjunta fuentes a aristas ya vistas; ordena grafo antes que vector; el custom graph no exige siempre fila Data); `cognee/api/v1/datasets/datasets.py:457-575` (`delete_data` autoriza, bloquea, resuelve ID, desenlaza ownership y devuelve recibo; su ledger audit-grade opt-in es distinto del ownership para eliminar). Motivada por issues #3235 (delete sin fila Data, closed/not-planned — no afirmar bypass ni fix) y #1366 (preview con counts).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` (`plan_write`, `persist_planned`, `apply_dedup_batch`), `vanta-memory/src/core/abstractions/types.rs` (`MemoryRecord` ya conserva fuentes y versión), lifecycle y auditoría — semántica de conjunto de respaldos y «último propietario», sin sidecar opaco paralelo.
- **Acciones:**
  1. Inventario de cardinalidad existente.
  2. Refs versionadas con preview de la operación.
  3. Quitar la ref; borrar el derivado solo cuando no queden respaldos válidos.
  4. Invalidar índices/caches y confirmar el recibo.
  5. Reconstruir proyección por generación; legado sin ownership se clasifica explícitamente (no inferir propietarios desde texto); rollback conserva lectura de refs.
- **Dependencias:** X-COG-06 (alcance) y X-COG-04 (exclusión real); primario+refs necesitan frontera coherente — `put_batch` no hace atómico el delete previo del writer propio (frontera citada en §9 del informe); auditoría puede conservar IDs/razones sin conservar el texto purgado.
- **FMEA:** refcount cero obsoleto → se destruye hecho compartido (detectar contra refs del primario y usar mutación condicional); índice huérfano → revive texto (validar fuente viva al leer y reconciliar); fallo tras detach → retry destructivo (recibo por operación); fuente añadida mientras se elimina otra → no puede perderse por RMW (validar revisión al aplicar el plan).
- **Criterios de aceptación propuestos:** dos fuentes y dos ámbitos; borrar una, ambas, un ID custom/ausente; añadir otra concurrentemente; interrumpir cada frontera; el recibo distingue aproximado/verificado y no deja a la UI vender purga completa.
- **Criterio negativo:** un derivado con respaldos válidos nunca se destruye al desenlazar una fuente; un cambio entre preview y ejecución nunca se ejecuta con la estimación vieja sin invalidarla.
- **Independencia:** sobre `MemoryRecord`/writer existentes; recontar fuentes al borrar evita contador derivado inconsistente (un refcount solo no permite reparación); alternativa: no compartir derivados (más almacenamiento, menos coordinación).
- **No objetivos:** no deducir ausencia de provenance Vanta, no fijar retención legal ni certificar borrado de backups/soporte físico.

### X-COG-03 — Resultado de enriquecimiento por etapa

- **Problema:** «el job terminó» no dice si feedback, Q&A o conflictos se publicaron; un fallo no fatal no debe contarse como éxito ni un agotamiento de presupuesto como dato vacío.
- **Contrato hipotético:** `EnrichmentOutcome { operation_id, source_revision, stages: Vec<StageOutcome>, resumable_from, usage }`; cada etapa distingue `Applied`/`Unchanged`/`Skipped`/`Failed`/`UnknownCommit` con motivo y fatalidad propias — la fatalidad y la posibilidad de retry pertenecen a la etapa, no a una heurística sobre texto de excepción; `usage` desconocido no se reporta como cero; presupuesto monetario no es presupuesto de contexto.
- **Fuente de evidencia (clone):** `cognee/api/v1/improve/improve.py:219-284` (`execute_stages` registra cada etapa; aborta la fatal `persist_session_qa`; agota budget sin repetir llamadas condenadas; marca la operación failed aunque el error no fatal no escape) y `:152-206` (etapas sin LLM se omiten con causa).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` (`PipelineWorker::run_task:286-339`, `TaskHandler::handle`) y generation log — ya tiene retries, leases y dead letters; el delta es detalle por etapa confirmada y clasificación de errores. `Result<(), String>` de `TaskHandler::handle` no acredita resultado durable por subetapa.
- **Acciones:**
  1. Inventariar `TaskHandler::handle` y el generation log.
  2. Mapear resultados existentes sin perder errores.
  3. Persistir el resultado tras cada efecto confirmado.
  4. Avanzar el cursor solo con la confirmación correspondiente; reanudar por operación.
  5. Migración compatible y versión del plan; desactivar una etapa deja sus derivados identificables, no los borra por rollback.
- **Dependencias:** retries/leases/dead letters existentes en `PipelineWorker`; idempotencia de cada efecto; X-COG-01 expone disponibilidad; X-COG-06 autoriza; X-COG-07 estudia los cursores de consumo; no ampliar locks sobre HTTP/LLM para simular transacción.
- **FMEA:** fallo no fatal contado como exitoso → watermark engañoso (estado agregado conservador y razón por etapa); cursor antes del write → pérdida de Q&A (confirmar juntos o recuperación explícita); 402 reintentado por cada etapa → gasto/ruido (clasificar agotamiento sin confundirlo con dato vacío); lease expirado → el worker viejo no sigue publicando (validar propietario/revisión en commit); timeout tras escribir → reconciliación, no repetir todo.
- **Criterios de aceptación propuestos:** éxito, noop por falta de LLM, error fatal/no fatal, timeout antes/después del commit, cancelación y lease vencido; ninguna etapa confirmada se ejecuta dos veces sin semántica idempotente; logs de fase/estado, no conversación.
- **Criterio negativo:** un `UnknownCommit` nunca se agrega como éxito; un agotamiento de presupuesto nunca se reporta como «sin datos».
- **Independencia:** delta sobre `PipelineWorker` existente; una transacción de motor cubre datos locales, no el proveedor LLM; no introduce otra cola ni una segunda política de reintentos.
- **No objetivos:** no copiar fail-open para datos críticos ni rediseñar scheduler, budgets de evaluación o prompts.

### X-COG-04 — Orden de locks declarado como contrato

- **Problema:** un job que retiene un permiso limitado mientras espera un recurso cuyo escritor espera ese permiso produce deadlock; un mutex local no sustituye fencing entre procesos.
- **Contrato hipotético:** `DatasetMutationGuard`/`ProjectionPermit` (nombres ilustrativos — Vanta no tiene por ello un «Dataset» de Cognee) como guardias del sistema existente; inputs: scope autorizado, propietario de tarea y cancelación; output: guard no transferible implícitamente a hijos; errores `LockOrderViolation`, `Cancelled`, `LeaseLost` — nunca un timeout convertido en noop.
- **Fuente de evidencia (clone):** `cognee/infrastructure/locks/dataset_lock.py:1-133` (declara orden dataset→queue slot, verifica inversión y explica el límite in-process; `held_datasets` se hereda a tareas hijas — no prueba exclusión entre hijos que escriban en paralelo).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` y `vanta-memory/src/services/scheduler.rs` — Vanta ya tiene locks de sesión, leases y scheduler; la unidad transferible es el orden contractual y el ownership de guard, no «poner un mutex global».
- **Acciones:**
  1. Dibujar el orden real de sesión, storage, índice y recursos externos.
  2. Identificar reentrancia deliberada e identidad de ejecución explícita.
  3. Adquirir en orden total.
  4. Publicar en sección corta y liberar en RAII; no conservar una transacción mientras se espera LLM.
  5. Cambio de orden requiere despliegue coherente de los callers; rollback no puede mezclar órdenes viejos/nuevos en un mismo proceso.
- **Dependencias:** inventario de primitivas reales (locks de sesión, leases, scheduler); identidad de ejecución explícita; X-COG-02/X-COG-05 usan su exclusión; X-COG-03 valida fence al commit; persistencia del plan pertenece a X-COG-03, no al lock.
- **FMEA:** orden invertido → deadlock (detectar esperas/owner y fallo temprano); contexto heredado → falsa reentrancia (identidad de ejecución explícita); lease perdido → escritor zombie (validar fence al commit); cancelación antes de adquirir no tiene efecto — después libera permisos y registra lo ya confirmado.
- **Criterios de aceptación propuestos:** dos mutaciones y borrado del mismo scope más otro scope, cancelación durante espera, tarea hija y lease expirado; progreso independiente y ausencia de efectos después de perder autoridad; observabilidad de wait/hold por clase de recurso, sin claves sensibles.
- **Criterio negativo:** ningún timeout de lock se convierte en noop silencioso; una guardia nunca se transfiere implícitamente a tareas hijas; un lease perdido nunca permite publicar.
- **Independencia:** contrato sobre primitivas existentes, sin sustituir SDK/storage; alternativas documentadas: serializar todo (bloquea scopes independientes) o revisión optimista (exige detectar conflictos) — se decide por recurso.
- **No objetivos:** no locks distribuidos por analogía, no garantías multiproceso desde un mutex ni prometer fairness no acreditada; sin migración de datos.

### X-COG-05 — Actualización incremental con plan validado completo

- **Problema:** editar una página no debe cambiar la identidad del documento ni reinterpretar silenciosamente todos sus chunks; el plan debe cubrir el documento nuevo bajo el contrato del chunker o caer a rebuild declarado — nunca adivinar.
- **Contrato hipotético:** `DocumentRevision`, `ChunkUpdatePlan { base, policy_version, kept, moved, replaced, fresh, invalidated_dependencies }` y `DocumentUpdateOutcome::{Applied, Unchanged, RebuildRequired, Conflict, Partial}`; input autorizado con revisión esperada; output contadores y causa sin texto. Cada chunk mapea a su fuente; IDs de contenido y posición son diferentes.
- **Fuente de evidencia (clone):** `cognee/api/v1/update/incremental.py:662-750,864-925,1063-1210` (autoriza write+delete, relee baseline dentro del lock, prepara contenido sin cambiar Data, valida que el plan reconstruye el texto y escribe antes del publish SQL; matiz adversarial: `:1144-1150` elimina primero los chunks redatados que conservan ID — un crash ahí deja hueco reparable por rebuild posterior) y `cognee/api/v1/update/update.py:219-349` (preserva ID incluso al reconstruir y devuelve causa tipada de fallback; no convierte permiso denegado en fallback).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs` y `vanta-memory/src/core/record/l1_writer.rs` — Vanta tiene ingestors, dedup y supersession; el delta es plan por documento/revisión con reuso, validación y explicación, no otro motor documental.
- **Acciones:**
  1. Registrar versión del chunker y dependencias de extracción.
  2. Plan puro validado por reconstrucción del texto.
  3. Preparar embeddings fuera del commit.
  4. Revalidar baseline; publicar cambios y manifest de generación; reparar proyecciones.
  5. Si las primitivas no permiten flip coherente, declarar `Partial` en lugar de prometer rollback; legado sin baseline cae a rebuild identificado; reindexar solo la generación afectada.
- **Dependencias:** condicional a documentos mutables; dedup/supersession existentes; X-COG-01 muestra readiness; X-COG-02 mantiene ownership; X-COG-03 recibos; X-COG-04 da exclusión; no usar hash de texto como única revisión del trabajo concurrente.
- **FMEA:** texto igual con contexto temporal distinto → hecho viejo (registrar dependencias y redatar derivados); chunker incompatible → omisión (rechazo tipado); borrar antes de nueva extracción → hueco (generación/recuperación explícitas); edición simultánea invalida el plan; cancelar antes de publish deja staging no activo para GC explícito — no ready; tras commit devuelve identidad de operación para reconciliar.
- **Criterios de aceptación propuestos:** inserción al principio, eliminación, texto repetido, cambio de fecha, chunker legado, permiso insuficiente y crash entre etapas; identidad estable y causa de rebuild visible.
- **Criterio negativo:** un plan que no reconstruye el documento completo nunca se aplica parcialmente en silencio; un permiso denegado nunca degrada a fallback.
- **Independencia:** extensión de ingestors/writer existentes; rebuild completo sigue siendo alternativa explícita cuando no se conserva baseline/chunker/ownership — rechazar la actualización es mejor que adivinar.
- **No objetivos:** no copiar ChunkPolicy, pesos/prompts, atomicidad distribuida ni ahorro medido sin ejecución; el publish SQL atómico upstream no implica snapshot consistente de grafo/vector (lección, no bug reproducido).

### X-COG-06 — Contexto autorizado y capacidad declarada del backend

- **Problema:** la configuración de una conexión no concede permisos y un backend sin soporte no hereda aislamiento por su interfaz; filtrar después de recuperar no repara el envío previo al LLM.
- **Contrato hipotético:** `AuthorizedOperationContext { principal, scope, authorization_revision, operation_id }` creado solo por la frontera gobernada; `BackendMemoryCapabilities` declara garantías verificables de consulta, mutación y publicación. Entrada de jobs y adapters usa ese contexto, no metadata libre; errores `UnsupportedIsolation`, `ScopeDenied`, `StaleAuthorization`, `CapabilityUnavailable` — desconocido nunca se trata como permitido.
- **Fuente de evidencia (clone):** `cognee/modules/search/methods/search.py:220-288` (`authorized_search` resuelve read antes de routing/fan-out); `cognee/infrastructure/databases/dataset_database_handler/supported_dataset_database_handlers.py:35-85` (compatibilidad de grafo y vector); `cognee/context_global_variables.py:95-125` (contexto por variables globales — riesgo de `ContextVar` heredado); `datasets.py:457-575` (delete autoriza antes de resolver el recurso). Motivada en parte por issue #2291.
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` (`InjectionPolicy`, `RecallConfig`) y tenancy del writer (`l1_writer.rs`) — extender propagación/diagnóstico de capacidad por transporte, distinto de inventar IAM o una base por tenant.
- **Acciones:**
  1. Inventariar rutas SDK/MCP/HTTP/ingest.
  2. Separar identidad de metadata.
  3. Comprobar acceso antes del fan-out y al commit de jobs largos.
  4. Propagar contexto inmutable; exponer ausencia de capacidad.
  5. Versionar el contrato del adapter sin alterar el primario para lectura; legado requiere política explícita, no asignación al usuario actual.
- **Dependencias:** identidad existente; gobierna X-COG-01…X-COG-08; revocar permiso invalida una publicación pendiente; cancelación corta ramas pendientes sin borrar resultados confirmados; rollback de UI no desactiva controles backend.
- **FMEA:** `ContextVar` heredado de otro trabajo → fuga (detectar principal/scope inconsistente; comprobar dos tareas y contextos anidados); backend que declara soporte que no implementa → filtrado incompleto (contrato condicionado, no sello de seguridad); sesión cacheada con ACL antigua → devolver contenido revocado (invalidación por revisión).
- **Criterios de aceptación propuestos:** acceso normal aceptado; revocación durante trabajo, ID ajeno, backend unsupported, error de routing y no-enumeración cubiertos; logs de clase de rechazo, sin nombres de datasets ajenos.
- **Criterio negativo:** capacidad desconocida nunca se trata como permitida; ninguna metadata libre sustituye al contexto autorizado; un revocado nunca recibe contenido por sesión cacheada.
- **Independencia:** prerequisito transversal de las demás candidatas; un adapter single-tenant honesto es alternativa válida a prometer multitenancy no soportada.
- **No objetivos:** no copiar globals Python, no autenticar con tags, no imponer Postgres/Neo4j ni certificar todos los transports.

### X-COG-07 — Promoción de sesión con watermark de consumo explícito

- **Problema:** una sesión genera Q&A, feedback y trazas que el operador quiere promover a conocimiento permanente en un destino concreto; un cursor global avanzado por un consumidor hace perder datos al segundo — «ya consumido en otro destino» debe mostrarse, no aparecer como éxito vacío.
- **Contrato hipotético:** `PromotionCursor { source_session, source_revision, consumer_scope, policy_version, last_confirmed_unit }` y `PromotionOutcome { applied, already_consumed, skipped, failed }` por consumidor y destino explícito; input: sesión autorizada, destino autorizado e intención explícita; output ancla a fuentes y al recibo de X-COG-03. Feedback no autoriza cambiar identidad ni elevar una preferencia inferida a regla de sistema.
- **Fuente de evidencia (clone):** `cognee/api/v1/improve/improve.py:152-206` (enumera etapas y declara Q&A/trazas por usuario+sesión, no por dataset; contenido ya promovido a otro destino no se repite, mientras distillation sí tiene scope por dataset — semántica que debe mostrarse al consumidor).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` (`MemoryTaskHandler`) y `vanta-memory/src/core/reflection/mod.rs` (reflection y escenas existentes); Vanta captura L0→L1 — distinguir captura, promoción, interpretación y consumo por destino, no añadir «memoria de sesiones» paralela.
- **Acciones:**
  1. Comparar cursores de pipeline existentes.
  2. Definir unidad de consumo y destino.
  3. Snapshot de fuentes; proponer derivados.
  4. Validar revisión y permisos.
  5. Confirmar outputs y avance de cursor coherentemente; si no caben en una mutación, reconciliar por operación.
- **Dependencias:** X-COG-02, X-COG-03, X-COG-06 son prerequisitos; cursor por consumidor aumenta retención y obligaciones de revocación; opt-in posterior (P3); migración conserva cursor previo con semántica conocida — no reprocesar todas las sesiones sin consentimiento.
- **FMEA:** dos consumidores avanzan cursor global → el segundo pierde datos (detectar motivo «ya consumido en otro destino», no éxito vacío); cursor antes del output → pérdida (confirmar después/coherentemente); retry o lease perdido → lección duplicada (identidad idempotente y revisión); fuente borrada durante LLM → revivir texto (invalidar al commit); cancelar conserva cursor confirmado y deja pendiente lo restante.
- **Criterios de aceptación propuestos:** mismo destino repetido, dos destinos, edición/borrado de sesión, permiso revocado y reintento tras timeout; contadores por consumidor sin texto personal; rollback desactiva promoción nueva, no restaura datos purgados.
- **Criterio negativo:** un cursor de un consumidor nunca avanza por el consumo de otro; «ya consumido en otro destino» nunca se reporta como éxito vacío.
- **Independencia:** posterior y opt-in; un único destino simplifica permisos (alternativa documentada: no se prescribe cuál debe ser default).
- **No objetivos:** no entrenamiento online, no aprendizaje de instrucciones hostiles ni copiar flags/prompts de feedback/evaluación; cambiar política crea generación nueva solo donde se solicite.

### X-COG-08 — Canales tipados y referencias de evidencia sin answerer

- **Problema:** un agente necesita evidencia inspeccionable, no una respuesta que oculte el origen; un fallo de rama no debe leerse como cero hits y el fan-out no rebasa el presupuesto global; contexto obtenido y respuesta generada son operaciones distintas.
- **Contrato hipotético:** `EvidenceBundle { source_revision, channel, references, truncations, channel_status }` con `EmptySource`, `NoMatches`, `ChannelFailed` y `BudgetTruncated` distintos; entrada: consulta y contexto autorizado de X-COG-06; salida: referencias vigentes y texto gobernado, sin inferencia obligatoria.
- **Fuente de evidencia (clone):** `cognee/modules/retrieval/hybrid_retriever.py:120-208` (diferencia grafo vacío `NoDataError` de miss; recupera canales en paralelo; carga pesos opcionales; proyecta metadata permitida — opt-in, no toda metadata entra al prompt; `only_context` separa contexto de completion; top-k de chunks/entidades/hechos no es budget global).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` (`assemble_with_recall`, `IntegratedContext`) — extensión del ensamblado existente, no otro RRF.
- **Acciones:**
  1. Inventariar el explain actual.
  2. Preservar identidad de fuente al unir canales.
  3. Reservar presupuesto en el assembler.
  4. Aplicar allowlist antes de serializar.
  5. Devolver bundle sin answerer o pasarlo al generador explícito; read-only, sin migración/reindexación; traza opt-in versionada.
- **Dependencias:** X-COG-06 autoriza y X-COG-01 aporta readiness (no depende de X-COG-07); lectura independiente de escritura; top-k por canal no sustituye el budget global; revisión de lectura evita anexar texto borrado durante la expansión.
- **FMEA:** fallo de rama leído como cero hits → falso diagnóstico (estado de canal explícito); metadata libre → prompt injection/secretos (allowlist y taint existentes); fan-out que rebasa contexto → el assembler mantiene autoridad y reporta truncamiento; cancelar detiene ramas y no publica un bundle completo ficticio.
- **Criterios de aceptación propuestos:** origen vacío, consulta sin hits, fallo parcial, duplicado en dos canales, referencia revocada y modo sin generación cubiertos; la traza no altera resultados; rollback desactiva campos opt-in.
- **Criterio negativo:** un `ChannelFailed` nunca se presenta como `NoMatches`; el modo sin generación nunca invoca un answerer implícito.
- **Independencia:** extensión read-only del assembler; alternativa documentada: solo chunks (menor complejidad pero sin relaciones); rechazados los pesos de personalización no auditados.
- **No objetivos:** no nueva fórmula/rúbrica, no pesos «truth» como verdad acreditada ni UI/generador paralelo.
