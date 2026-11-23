---
title: "Backlog de extracción — CrewAI"
kind: research
status: active
description: "Candidatas del informe CrewAI: conformidad del adapter existente, consolidación con invariantes de propiedad, recibo durable de saves, recall autorizado y capability de herramientas"
tags: [extraccion, backlog, crewai]
---

# Backlog de extracción — CrewAI

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/crewai.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-CRW-01 | 🟠 | Conformidad del adapter de memoria con protocolo por scopes | `integrations/crewai/vantadb_crewai/memory.py` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: revisar y extender el `VantaDBMemoryBackend` ya existente para que save/search/update/delete respeten segmentos de scope, identidad de embedding y capability declarada, opt-in por instancia explícita. Por qué: prefijos vecinos `/team/a` y `/team/ab` no deben mezclarse; filtrar después de top-k no prueba ACL end-to-end y atrapar `Exception` como `None`/`[]` confunde fallo con ausencia. Acción mínima: inventariar compatibilidad del adapter actual, separar fallo de vacío, filtrar antes de selección cuando el API lo permita y declarar cardinalidad parcial. Trazabilidad: informe §Ficha A · evidencia `lib/crewai/src/crewai/memory/storage/backend.py:45-105`, `lib/crewai/src/crewai/memory/storage/factory.py:1-55` | X-CRW-04, C11, C3 | Binding Python/SDK embebido, versión de CrewAI y dimensión/métrica fijadas, contrato de scores |
| X-CRW-02 | 🔴 | Consolidación condicionada que conserva propiedad y provenance | `vanta-memory/src/core/record/l1_writer.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: un plan de transformación multirregistro exige objetivos dentro del snapshot autorizado, revisión esperada y conservación de campos de control (private/source/provenance) por defecto. Por qué: upstream reconstruye el registro sin `private/source` en el update de consolidación y los defaults los pierden (issue #8659); el modelo sugiere, nunca autoriza ni sobrescribe el sello confiable. Acción mínima: extender `plan_write`/`persist_planned`/`apply_dedup_batch` con precondiciones de plan, persistencia de decisión y aplicación en la frontera real disponible; rechazar IDs fuera del conjunto autorizado. Trazabilidad: informe §Ficha B · evidencia `lib/crewai/src/crewai/memory/encoding_flow.py:388-517`, `lib/crewai/src/crewai/memory/analyze.py:94-134`, `lib/crewai/src/crewai/memory/types.py:20-105` | X-CRW-03, X-CRW-01, C1, C6 | Snapshot de candidatos autorizados, dedup/supersession existentes, inventario de frontera de escritura antes de CAS fuerte |
| X-CRW-03 | 🟠 | Recibo durable de operación de escritura separado del save | `vanta-memory/src/services/pipeline_worker.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: cada operación de escritura produce un receipt con ID, revisión y estado Accepted/Persisted/Failed/CancelledBeforeApply/OutcomeUnknown consultable tras reinicio. Por qué: `drain_writes` es barrera de lectura de futures, no log durable ni watermark; una Future completada, un evento o un timeout del lector no acreditan que el hecho se persistió. Acción mínima: comprobar si los IDs/estados de `TaskCheckpointManager::{begin,advance,complete,fail}` cubren el receipt y exponerlos antes de crear tabla adicional; conservar identidad aplicada para reconciliar al reintentar. Trazabilidad: informe §Ficha C · evidencia `lib/crewai/src/crewai/memory/unified_memory.py:302-375,716-724` | X-CRW-02 (para planes), C1, C9, C13 | Scheduler/checkpoints existentes, ID de operación estable, ownership del worker; no requiere X-CRW-01 |
| X-CRW-04 | 🟡 | Recall por ramas con resultado parcial explicable | `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: el reporte gobernado de recall distingue por rama completada/vacía/denegada/fallida/cancelada y conserva la lista de fallos aunque no se recupere ningún hecho. Por qué: un resultado parcial o vacío no debe confundir «subconsulta falló», «no hay evidencia» y «scope denegado»; `Denied`/`BudgetExhausted` no son scores bajos. Acción mínima: extender `RecallGovernance`/reporte existente con procedencia por rama sin reemplazar BM25/HNSW/RRF ni adoptar planificación LLM multirama sin necesidad medida. Trazabilidad: informe §Ficha D · evidencia `lib/crewai/src/crewai/memory/recall_flow.py:89-222,354-388` | X-CRW-01, C7, C3 | Scopes/presupuesto existentes del host, retrieval vigente; multiquery opcional y dependiente de evidencia de necesidad |
| X-CRW-05 | 🟠 | Capability de herramientas de memoria con confirmación honesta | `spec` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: la proyección del resultado de herramienta de memoria distingue Applied/Pending/Denied/ReadOnly derivado del receipt, con capability read/write proveniente del host y mutadores ocultos en modo lectura. Por qué: un texto «Saved» cuando solo se encoló un lote permite usar un hecho inexistente; `Pending` es resultado operativo, no rechazo ni confirmación durable. Acción mínima: inventariar tools MCP ya expuestas, volver a autorizar al ejecutar, usar captura con ID de operación y devolver receipt de X-CRW-03 si es asíncrono. Trazabilidad: informe §Ficha E · evidencia `lib/crewai/src/crewai/tools/memory_tools.py:13-130` | X-CRW-01, X-CRW-03, C3, C13 | Binding/MCP existentes y capability derivada del usuario por el host; X-CRW-03 para saves diferidos |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/crewai.md` (Fichas A–E de §2 y
> precisiones de cierre). Todos los contratos son **hipotéticos** — propuestas de
> comportamiento para VantaDB, no APIs existentes ni puertos de código upstream.
> Las referencias `archivo:línea` son del clon `C:\Users\Eros\crewAI` en SHA
> `73f48320dbe6e2dc7a9bc54fefc6a13637b9485c`; las referencias `CV*` fijan el
> snapshot VantaDB `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.

### X-CRW-01 — Conformidad del adapter de memoria con protocolo por scopes

- **Problema:** prefijos vecinos `/team/a` y `/team/ab` no deben mezclarse; filtrar scopes/categorías/metadata después de top-k no prueba ACL end-to-end (más candidatos ajenos pueden desplazar a los elegibles); atrapar `Exception` como `None`/`[]` confunde fallo de backend con ausencia.
- **Contrato hipotético (tipos propuestos):** `ScopedRecordQuery { tenant, namespace, categories, metadata, vector, limit, score_kind }`; adapter Python que implementa el protocolo `StorageBackend` y declara capacidades — adapter opt-in, no trait Rust público nuevo. Entrada confiable: tenant, raíz permitida, identidad del embedding y capability de escritura; entrada no confiable: texto, filtros y contenido sugerido por el modelo. Salida: registros con ID/revisión y score etiquetado. Errores propuestos: `Denied`, `DimensionMismatch`, `UnsupportedFilter`, `Conflict`, `StoreFailure`.
- **Fuente de evidencia:** `lib/crewai/src/crewai/memory/storage/backend.py:45-105` (R6, protocolo save/search/delete/update/get/list/count/reset), `lib/crewai/src/crewai/memory/storage/factory.py:1-55` (R5, factory global; la instancia explícita gana). Estado actual del adapter VantaDB [CV6]: `memory.py::_to_meta:113-125` (claves reservadas `__mem_*`), `save:165-173`, `search:190-226` (`limit*2+10` candidatos y filtros post-selección — default observado, no recomendación), `get_record:175-179`/`search:201-206` (capturan `Exception` → `None`/`[]`, negativo local concreto).
- **Seam VantaDB:** `integrations/crewai/vantadb_crewai/memory.py::VantaDBMemoryBackend` — ya existe; revisar y extender, no crear otro backend ni una integración nueva.
- **Acciones:**
  1. Fijar versión de CrewAI y dimensión/métrica del embedding.
  2. Mapear IDs/scopes por segmentos (prefijo canónico, no por string).
  3. Inventariar compatibilidad del adapter actual; separar fallo de vacío sin romper clientes.
  4. Filtrar antes de top-k cuando el API lo permita; si no, declarar cardinalidad parcial.
  5. Implementar factory opt-in y lifecycle explícito (resolver instancia → validar capability → normalizar scope por segmentos → llamar SDK → convertir resultados).
  6. Probar conformidad manual después; conservar lectura del formato `__mem_*`.
- **Dependencias:** binding Python/SDK embebido, versión de CrewAI y dimensión/métrica fijadas, contrato de scores. Solo binding/protocolo y contrato de scores para el modo read-only: X-CRW-02/X-CRW-03 no son prerrequisitos.
- **FMEA:** factory global afecta otras aplicaciones del proceso; string desconocido interpretado como path; scopes parciales se cruzan; score distancia versus similitud (no reinterpretar RRF como coseno para pasar `min_score`). Detección propuesta: comparar tenant/scope/score-kind del receipt con la petición, registrando IDs y causas — nunca contenido privado.
- **Criterios de aceptación propuestos:** `/team/a` y `/team/ab` no se mezclan; IDs/dimensión/filtros consistentes; borrar scope no afecta otro tenant; override explícito no cambia instancias ya construidas; búsqueda fallida se distingue de vacía; resultados autorizados escasos se declaran parciales.
- **Criterio negativo:** un k incompleto no significa store vacío ni autorización; el commit es el de la operación SDK, no el retorno de la factory; `close` libera el handle propio, nunca una DB prestada.
- **Independencia:** útil sin X-CRW-02/X-CRW-03 en modo lectura; desactivar el factory devuelve la selección al host sin borrar datos.
- **No objetivos:** copiar LanceDB ni sustituir BM25/HNSW/RRF; migrar datos o reindexar salvo cambio explícito de modelo/dimensión (entonces proyección separada y cambio de referencia solo cuando esté disponible); publicar API nueva aún.

### X-CRW-02 — Consolidación condicionada que conserva propiedad y provenance

- **Problema:** upstream reconstruye el `MemoryRecord` sin `private`/`source` en el update de consolidación y los defaults del modelo los pierden (issue pública #8659, open al 2026-11-07); el modelo sugiere, nunca autoriza ni sobrescribe el sello confiable.
- **Contrato hipotético (tipos propuestos):** `ConsolidationOp { target_id, expected_revision, authorized_scope, change, source_refs }`. Patch tipado: omitir metadata conserva, reemplazar metadata es explícito, cambiar ownership requiere capability; los campos de privacidad no cambian por omisión. Entrada: snapshot de candidatos autorizados y operaciones con revisión esperada. Salida: receipt por plan y por objetivo; errores `StaleRevision`, `UnknownTarget`, `Denied`, `EmbeddingFailed`, `CommitFailed`.
- **Fuente de evidencia:** `lib/crewai/src/crewai/memory/encoding_flow.py:388-517` (R2, `execute_plans`; el update reconstruye `MemoryRecord` sin `private`/`source` en `:473-490` y deduplica acciones por ID, primera gana), `lib/crewai/src/crewai/memory/analyze.py:94-134` (R4, `ConsolidationAction`/`Plan` propuestos por LLM), `lib/crewai/src/crewai/memory/types.py:20-105` (R3, defaults `source: Optional[str]`/`private: bool`); issue [#8659](https://github.com/crewAIInc/crewAI/issues/8659).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` — `plan_write`, `persist_planned` y `apply_dedup_batch` [CV1] ya separan decisión y escritura; la propuesta añade precondiciones de transformación multirregistro y conservación de ownership. Scheduler/checkpoints para consolidación diferida.
- **Acciones:**
  1. Congelar el snapshot de candidatos autorizados.
  2. Validar los IDs devueltos por el LLM contra ese conjunto; rechazar IDs fuera de él.
  3. Conservar metadata de propiedad (private/source/provenance) por defecto.
  4. Persistir plan/decisión antes de aplicar.
  5. Aplicar en la frontera real disponible (CAS o lote atómico si existe; si no, limitar inicialmente a una operación y declarar el resto no soportado).
  6. Registrar supersession. Orden propuesto: calcular texto/vector fuera del commit → revalidar objetivo/tenant/generación → persistir transformación y supersession → emitir eventos después.
- **Dependencias:** snapshot de candidatos autorizados, dedup/supersession existentes, inventario de la frontera de escritura antes de prometer CAS fuerte. El worker es dueño del intento, no de registros de otro tenant; leases no reemplazan revisión al publicar.
- **FMEA:** target fuera de candidatos; conflicto first-wins entre planes (el orden del batch influye); borrado parcial; cambio involuntario de private/source; plan viejo tras borrado → resurrección → detectar generación obsoleta y rechazar; fallo de proyección tras primario → marcar pendiente de reindexación, no repetir la mutación como si nunca hubiera ocurrido.
- **Criterios de aceptación propuestos:** update de contenido conserva private/source y referencias; IDs fuera del snapshot se rechazan; crash conserva plan recuperable; conflicto de revisión no pisa escritura nueva; fuentes siguen auditables.
- **Criterio negativo:** un éxito parcial nunca se anuncia como total; una acción arbitraria del LLM fuera del conjunto autorizado se rechaza, no se traduce.
- **Independencia:** no necesita el adapter de X-CRW-01; X-CRW-03 solo es necesario para prometer recuperación diferida; X-CRW-04 no lo es. Desactivar nuevas consolidaciones no deshace hechos posteriores.
- **No objetivos:** introducir otro juicio LLM obligatorio; borrar fuentes originales por recomendación libre del modelo; implementar otra deduplicación (el adapter solo traduce; se extiende el plan existente).

### X-CRW-03 — Recibo durable de operación de escritura separado del save

- **Problema:** `drain_writes` es barrera de lectura de futures pendientes, no log durable ni watermark: espera y consulta excepciones pero no las relanza; una Future completada, un evento o un timeout del lector no acreditan que el hecho se persistió.
- **Contrato hipotético (tipos propuestos):** `MemoryWriteReceipt { operation_id, revision, status, error_class }` con `status: Accepted|Persisted|Failed`, refinado con `CancelledBeforeApply` y `OutcomeUnknown` cuando la frontera no pueda acreditar aplicación; consulta por ID. El receipt identifica sesión, operación, intento y generación.
- **Fuente de evidencia:** `lib/crewai/src/crewai/memory/unified_memory.py:302-375` (R1, `_submit_save`/`_on_save_done`/`drain_writes`/`close`) y `:716-724` (barrera de recall que no relanza fallos).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` — `PipelineWorker::run_once/run_task` y `TaskCheckpointManager::{begin,advance,complete,fail}` [CV2] ya poseen claim/retry/estados: comprobar si esos IDs/reportes cubren el receipt y preferir exponerlos antes de crear una tabla adicional.
- **Acciones:**
  1. Persistir la intención con ID de operación estable antes del trabajo costoso (admisión durable).
  2. Claim bajo ownership del worker.
  3. Cálculo opcional LLM y revalidación de generación.
  4. Escritura y checkpoint terminal; si escritura y checkpoint no comparten commit, conservar la identidad aplicada para reconciliar al reintentar.
  5. Notificación best-effort; capturar el fallo de forma durable, no solo por evento.
  6. Distinguir timeout del lector de fallo del write; coordinar shutdown y borrado; opción sync solo si el caller la pide.
- **Dependencias:** scheduler/checkpoints existentes, ID de operación estable, ownership del worker. Depende del contrato de escritura de X-CRW-02 solo para planes de consolidación; el host posee shutdown y finalización, no una Future abandonada.
- **FMEA:** proceso muere con pool pendiente; evento perdido durante shutdown; recall espera pero silencia fallo; retry duplica extracción o consolida dos veces; un lock local no garantiza crash safety.
- **Criterios de aceptación propuestos:** tras reinicio cada operación tiene estado o es reanudable; el fallo es visible sin depender del bus de eventos; retry con mismo ID no duplica; borrar scope invalida trabajo viejo que intenta escribir después. Aceptación manual adicional del informe: cancelación antes/después de commit, caída entre commit y notificación, lector que exige watermark y borrado con trabajo viejo; receipt final coherente con registros supervivientes, sin logs de payload.
- **Criterio negativo:** «cancelado» nunca significa automáticamente «no escrito»; el timeout del cliente no revoca una escritura confirmada; la cancelación previa impide iniciar efectos nuevos solo si el worker la observa.
- **Independencia:** no requiere X-CRW-01 ni X-CRW-04; rollout compatible como proyección de estados existentes — desactivar el adapter no borra operaciones admitidas.
- **No objetivos:** exactly-once de herramientas externas ni reemplazar el scheduler existente.

### X-CRW-04 — Recall por ramas con resultado parcial explicable

- **Problema:** una pregunta larga puede requerir varias búsquedas, pero un resultado parcial o vacío no debe confundir «subconsulta falló», «no hay evidencia» y «scope denegado». Upstream omite búsquedas fallidas y adjunta `evidence_gaps` solo al primer match: un resultado vacío no puede transportar ese gap.
- **Contrato hipotético (tipos propuestos):** `RecallBranch { query_ref, allowed_scope, parent_branch }` → `RecallBranchOutcome { evidence_refs, status, cost_ref }`, donde `status` distingue completada/vacía/denegada/fallida/cancelada. La lista de fallos existe aunque no se recupere ningún hecho. `Denied` y `BudgetExhausted` no son scores bajos.
- **Fuente de evidencia:** `lib/crewai/src/crewai/memory/recall_flow.py:89-222,354-388` (CH/R9, `RecallFlow._do_search`/`analyze_query_step`/`synthesize_results`: producto subqueries × scopes, paralelismo acotado a cuatro workers, omisión de búsquedas fallidas, dedup por ID, gaps en el primer match).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` — ya existen `RecallConfig`, `InjectionPolicy`, `perform_auto_recall_governed` y `RecallGovernance` [CV3]; extender su reporte con procedencia por rama. El assembler `vanta-memory/src/context_engine/engine.rs` [CV4] decide el contexto final.
- **Acciones:**
  1. Host fija scopes y presupuesto existente.
  2. Planner opcional sugiere ramas (sin nuevo prompt ni parámetros aquí).
  3. Intersección con capability y validación antes de recuperar — el planner no amplía ACL, no modifica la raíz por encontrar un nombre de scope y no autoriza `include_private`.
  4. Retrieval existente (sin reemplazar BM25/HNSW/RRF ni recalibrar el scoring compuesto opcional).
  5. Dedup por ID/revisión.
  6. El assembler decide contexto final y reporte; si hay touch, separarlo del éxito de lectura; no entregar resultados tardíos a una generación de consulta nueva.
- **Dependencias:** scopes/presupuesto existentes del host, retrieval vigente; multiquery opcional y dependiente de evidencia de necesidad. Solo depende de X-CRW-01 para la integración CrewAI; puede ampliar VantaDB sin X-CRW-01/02/03.
- **FMEA:** scope sugerido hostil → fuga → comparar con capability antes del search; rama caída → falsa exhaustividad → estado parcial explícito; fan-out → consumo excesivo → presupuesto/cancelación del host; orden de futures → dedup inestable → identidad/revisión y orden de salida definido por el consumidor.
- **Criterios de aceptación propuestos:** ramas vacía/fallida/denegada distinguibles sin matches; ACL consistente en cada rama; presupuesto global respetado con la excepción del prefijo protegido documentada por VantaDB.
- **Criterio negativo:** un resultado parcial nunca se presenta como búsqueda exhaustiva; una consulta pura no produce commit de memoria.
- **Independencia:** el primer paso sostenible es instrumentar el recall simple; un adapter que conserve `RecallGovernance` es preferible a portar `RecallFlow`. Reporte aditivo versionado y desactivable, sin migración de índices.
- **No objetivos:** prometer mejor QA, copiar confidence/oversampling/pesos upstream ni crear otro orquestador.

### X-CRW-05 — Capability de herramientas de memoria con confirmación honesta

- **Problema:** el LLM puede pedir guardar y afirmar éxito cuando solo se encoló un lote; un modo consulta no debe ofrecer mutaciones. Upstream guarda un hecho síncronamente, anuncia background para varios y omite `Remember` si `read_only` — superficie de herramientas del agente, no gate HITL.
- **Contrato hipotético (tipos propuestos):** `MemoryToolResult { operation_ref, applied_refs, state, governance_ref }` con `state` derivado del receipt: `Applied`/`Pending`/`Denied`/`ReadOnly`. El host deriva capability read/write del usuario, no de argumentos del modelo. Input: contenido no confiable y operación/scope sellados por host. Errores: `Denied`, `ReadOnly`, `InvalidInput`, `Pending`, `StoreFailure`.
- **Fuente de evidencia:** `lib/crewai/src/crewai/tools/memory_tools.py:13-130` (CJ, `RecallMemoryTool._run`, `RememberTool._run`, `create_memory_tools`: dedup por ID entre queries, save síncrono unitario, background para lotes, omisión de mutadores en read_only).
- **Seam VantaDB:** `spec` — reutilizar las herramientas MCP ya expuestas y el binding hacia `vanta-memory`; no añadir otro tool runtime. Proyección de estado/ref en Studio existente sin payload por defecto.
- **Acciones:**
  1. Inventariar tools MCP ya expuestas.
  2. Ocultar mutadores para capability de lectura y volver a autorizar al ejecutar (revocación entre listado e invocación deniega en la segunda frontera).
  3. Usar la captura existente con ID de operación.
  4. Devolver el receipt de X-CRW-03 si la operación es asíncrona.
  5. Mostrar estado/ref en Studio sin payload por defecto; usar solo refs autorizadas en metadata de herramienta.
- **Dependencias:** binding/MCP existentes y capability derivada del usuario por el host; X-CRW-01 solo para la fachada CrewAI; X-CRW-03 para saves diferidos. Writes usan el commit existente; no hay commit adicional por renderizar. Cancelar la respuesta no revierte un write confirmado.
- **FMEA:** UI dice guardado antes de commit → uso de hecho inexistente → estado derivado de receipt; capability revocada → write tardío → comprobar al aplicar; metadata de herramienta revela secreto → refs autorizadas; batch repetido → duplicado → ID existente de captura.
- **Criterios de aceptación propuestos:** el caso normal guarda una vez; un lote que falla parcialmente explica cada operación; read-only no ofrece mutadores y los rechaza si se invocan por nombre.
- **Criterio negativo:** un texto «Saved» exige estado aplicado acreditado; `Pending` es resultado operativo — nunca rechazo ni confirmación durable.
- **Independencia:** rollback = retirar la exposición de la herramienta conservando las operaciones ya admitidas; alternativa mínima del informe: documentar `Pending` en las tools actuales antes que añadir API pública.
- **No objetivos:** aprobación genérica de pagos, nuevo scheduler, extracción LLM obligatoria ni cambio de prompts del agente.

[CV1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L176-L459
[CV2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L213-L353
[CV3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L123-L435
[CV4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L197-L349
[CV6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/crewai/vantadb_crewai/memory.py#L91-L245

<!-- audit: 2026-11-23 añadidas definiciones de referencia [CV1]-[CV4],[CV6] omitidas del informe -->
