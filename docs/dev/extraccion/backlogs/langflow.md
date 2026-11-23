---
title: "Backlog de extracción — Langflow"
kind: research
status: active
description: "Cinco candidatas del informe Langflow: capacidad de filtro por adaptador, descriptores sin conexión, fencing de generaciones para derivados, recibo de lote conversacional e historial de ingesta por item"
tags: [extraccion, backlog, langflow]
---

# Backlog de extracción — Langflow

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/langflow.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-LFL-01 | 🟠 | Plan de ejecución de filtro con capacidad declarada | `src/sdk/search/explain.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** contrato que declara si el predicado se aplica como prefiltro exacto, postfiltro sobre pool ampliado o no soportado (`ExactPrefilter/Postfilter/Unsupported`), con diagnóstico visible del modo aplicado. **Por qué:** un filtro tras top-k no equivale al mismo filtro en la selección de candidatos; el JSON inválido degradado a filtro vacío y el postfiltro que devuelve menos de k son decisiones que deben ser explícitas, no silenciosas. **Acción mínima:** inventariar operadores por interfaz pública, reutilizar `explain_memory_search` existente, validar predicado tipado en el borde y emitir el modo aplicado sin cambiar ranking. **Trazabilidad:** informe §Candidata A · evidencia `src/lfx/src/lfx/components/files_and_knowledge/knowledge.py:1748-1859,1887-1941`, `src/lfx/src/lfx/base/knowledge_bases/backends/sqlite.py:639-734`. | C7, C15, C16 | Tipos de metadata, scope autorizado, límites de candidatos, reportes de retrieval existentes, contrato de score por backend |
| X-LFL-02 | 🟡 | Descriptor de adaptador sin abrir conexión | `spec` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | **Qué:** descriptor puro de adaptador (id estable, revisión de config, localidad `Embedded/Remote/Unknown`, capacidades, origen de credencial) que puede describirse y validarse sin instanciar el cliente. **Por qué:** inspeccionar capacidades no debe importar SDKs, resolver secretos ni contactar proveedores; la localidad inferida por nombre clasifica mal un destino y un rename visual no debe cambiar la identidad. **Acción mínima:** derivar el descriptor del inventario actual, separar validación estructural de apertura, fallar explícito ante backend desconocido o migración requerida; `describe_without_connecting` no hace red. **Trazabilidad:** informe §Candidata B · evidencia `src/lfx/src/lfx/base/knowledge_bases/backends/registry.py:1-135`, `src/lfx/src/lfx/base/knowledge_bases/backends/base.py:1-85`, `src/lfx/src/lfx/components/files_and_knowledge/retrieval.py:1-69`. | X-LFL-01 (consume sus capabilities), C15, C11 | Configuración tipada, política de destinos, permisos filesystem, cancelación y ciclo de vida de conexiones existentes |
| X-LFL-03 | 🟠 | Ticket de revisión para publicación de derivados externos | `vanta-memory/src/core/record/l1_writer.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | **Qué:** token (`DerivedWriteTicket` propuesto) que conserva fuente, revisión de fuente/config y operation_id desde la planificación hasta la publicación de un derivado externo; la escritura revalida tras el I/O y rechaza fuente retirada. **Por qué:** un delete/migrate durante una generación lenta de embeddings puede resucitar la fuente vieja con el resultado tardío si la escritura no refencea. **Acción mínima:** capturar ticket autorizado antes del I/O, preparar vectores sin locks del motor, revalidar el ticket en el seam de escritura existente y registrar recibo o descartar trabajo obsoleto. **Trazabilidad:** informe §Candidata C · evidencia `src/lfx/src/lfx/components/files_and_knowledge/knowledge.py:1022-1057`, `src/backend/base/langflow/services/knowledge_base_storage/runtime.py:282-316,448-535`, `src/lfx/src/lfx/base/knowledge_bases/backends/sqlite.py:494-528`. | X-LFL-04 (usa la misma revisión), X-LFL-02 (identifica destino), C14, C13, C4 | Revisiones de fuente/configuración compatibles con provenance/supersession; seams de planificación y persistencia L1 existentes; no locks distribuidos nuevos |
| X-LFL-04 | 🟠 | Recibo de lote conversacional con reanudación sin nueva decisión | `vanta-memory/src/utils/task_checkpoint.rs` | 🟠 1-2sem | 🟡 P2 | ⬜ Candidata | **Qué:** recibo por lote (`ConversationBatchReceipt` propuesto) que enlaza mensajes fuente, revisión, output de preprocesado y fase de publicación; `resume_batch` no repite la decisión de modelo si el output autorizado ya existe. **Por qué:** compactar una conversación una sola vez por lote y reanudar tras fallo de indexación sin pagar otra decisión inconsistente; el cursor vivo bajo lock evita usar el snapshot obsoleto de dispatch y no se avanza posición sin recibo inequívoco. **Acción mínima:** enlazar identidad/revisión del lote fuente al log/checkpoint existente, preparar output fuera de escritura, revalidar mensajes relevantes y avanzar cursor solo tras publicación confirmada. **Trazabilidad:** informe §Candidata D · evidencia `src/backend/base/langflow/services/memory_base/task.py:284-429,490-646,838-879`. | X-LFL-03 (depende cuando publica en fuente migrable), C13, C9, C8 | Context assembly/compaction, pipeline worker, log de generación y checkpoints existentes; definir unidad de cursor y efecto de editar/purgar un mensaje; condicionada a aprobar resumir sesiones con proveedor |
| X-LFL-05 | 🟡 | Historial de ingesta por item separado de spans | `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | **Qué:** vista de run de ingesta (`IngestionRunView` propuesta) con outcomes por item (`Published/Skipped/Failed/Cancelled`) y contadores derivados del propio run, separada de la traza de ejecución. **Por qué:** explicar por qué un archivo no entró sin confundir un span correcto, un job terminado y todas las filas publicadas; cerrar un modal no confirma éxito. **Acción mínima:** mapear outcomes por ID estable sin duplicar jobs, capturar resultados en el dueño de ingesta, publicar proyección redactada en Studio e invalidar queries al completar/cancelar. **Trazabilidad:** informe §Candidata E · evidencia `src/frontend/src/pages/MainPage/pages/knowledgePage/components/IngestionRunDetailModal.tsx:14-222`, `src/frontend/src/pages/MainPage/pages/knowledgePage/components/IngestionRunsSection.tsx:15-188`. | X-LFL-03, X-LFL-04 (presenta outcomes de sus owners), C9, C16 | `TaskCheckpointManager`/auditoría y actividad Studio (RetrievalLens) existentes; revocación de permisos invalida el preview; exportar contenido requiere autorización separada |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/agent-memory-2026-11-07/langflow.md` (§2 mecanismos A–E y «A/B: precisión de contrato, ownership y pasos»). Todos los DTO/contratos son **hipotéticos** — propuestas de comportamiento para VantaDB, no APIs existentes ni puertos de código upstream. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\langflow` en SHA `60999cc9a7350e44dde3ef641baae864e10861f0`; las rutas VantaDB citan el snapshot del informe (`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, §0.6) y requieren revalidación contra HEAD. Dictamen del informe: estudiar el contrato de capacidades/filtros y la carga diferida de adaptadores; no incorporar el editor visual ni su runtime Python al core.

### X-LFL-01 — Plan de ejecución de filtro con capacidad declarada

- **Problema:** un filtro aplicado después de top-k no equivale al mismo filtro dentro de la selección de candidatos; el JSON inválido degradado a filtro vacío y el postfiltro que devuelve menos de k son decisiones que deben ser explícitas, no silenciosas. Una ACL no se degrada a postfiltro opcional.
- **Contrato hipotético (tipos propuestos):** `FilterExecutionPlan { scope, predicate, capability: ExactPrefilter | Postfilter | Unsupported, candidate_budget, diagnostics }`; `predicate` no contiene autorización y `scope` ya fue validado. Operaciones propuestas: `validate_filter`, `resolve_capability`, `explain_plan`, `execute_with_outcome`; errores `InvalidPredicate`, `UnsupportedOperator`, `StaleConfig`, `Cancelled`. Entrada: predicado tipado y configuración revisionada; salida: plan puro y outcome de ejecución asociado al mismo request. No exigir un LLM para interpretar filtros.
- **Fuente de evidencia (clone):** `src/lfx/src/lfx/components/files_and_knowledge/knowledge.py:1748-1859` (filtro exacto para SQLite en `:1815-1829`; otros backends reciben búsqueda ampliada y filtrado posterior), `:1887-1941` (`:1887-1916` convierte JSON inválido en filtro vacío deliberadamente); `src/lfx/src/lfx/base/knowledge_bases/backends/sqlite.py:639-734` (filtro SQL vinculado a valores y orden distancia/ID antes del límite).
- **Seam VantaDB:** `src/sdk/search/explain.rs` — `Embedded::explain_memory_search` [V1] ya valida namespace/metadata y obtiene ruta/rangos de candidatos (delta revalidado: no está ausente un explain; la propuesta describe la capacidad del filtro de un adapter y la distingue del filtrado de presentación). Filtros/retrieval híbrido existentes; no implementar un segundo buscador SQLite ni copiar la regla de sobrebúsqueda.
- **Acciones:**
  1. Inventariar operadores por interfaz pública y capacidades ya publicadas por Vanta/SDK.
  2. Documentar la semántica de operadores.
  3. Reutilizar `explain_memory_search` existente y validar el predicado tipado en el borde.
  4. Emitir el modo aplicado sin cambiar ranking.
  5. Mostrar cuándo un postfiltro limita la exhaustividad de resultados.
- **Dependencias:** tipos de metadata, scope autorizado, límites de candidatos, reportes de retrieval existentes y contrato de score por backend; puede consumir capabilities de X-LFL-02. Converge con C7/C15/C16 de `CONSOLIDATED.md`. Alternativa preferible si ya está cubierto: documentar el contrato existente, no crear `FilterExecutionPlan` paralelo.
- **FMEA:** parser tolerante → expansión inesperada; postfiltro → menos de k aunque haya elegibles fuera del pool; stringify → colisión de tipos; capacidad mal declarada → UI promete exactitud falsa. Mitigar con validación tipada, error explícito y modo aplicado visible.
- **Criterios de aceptación propuestos:** filtros inválidos no amplían scope; el plan identifica exacto/postfiltro; cero elegibles no se confunde con error; los desempates conservan identidad de fuentes.
- **Criterio negativo:** un predicado inválido nunca amplía el scope ni degrada una ACL a postfiltro opcional; resolver el plan no escribe y una consulta no confirma ingesta.
- **Independencia:** sí para diagnóstico — sin migración del índice; desactivar la vista conserva la semántica de filtrado y errores; puede consumir capabilities de X-LFL-02 sin requerirla.
- **No objetivos:** garantizar k resultados por sobrebúsqueda fija ni retocar RRF.

### X-LFL-02 — Descriptor de adaptador sin abrir conexión

- **Problema:** inspeccionar capacidades no debería importar todos los SDKs, resolver secretos ni contactar proveedores; la localidad inferida por nombre clasifica mal un destino y un rename visual no debe cambiar la identidad.
- **Contrato hipotético (tipos propuestos):** `AdapterDescriptor { adapter_id, config_revision, locality: Embedded | Remote | Unknown, capabilities, credential_origin }`; `credential_origin` indica procedencia, nunca el valor del secreto. Operaciones propuestas: `describe_without_connecting`, `validate_destination`, `open_scoped`, `close`; errores `UnknownAdapter`, `InvalidDestination`, `MissingCredential`, `OpenFailed`, `Cancelled`; IDs estables distintos de etiquetas editables; `describe_without_connecting` no falla por no tener red o secreto.
- **Fuente de evidencia (clone):** `src/lfx/src/lfx/base/knowledge_bases/backends/registry.py:1-135` (`get_backend_class` carga el módulo solo al seleccionar; `register_backend` rechaza reemplazar clase distinta bajo el mismo identificador; `is_local_backend` distingue localidad del nombre genérico — Chroma legacy no puede suponerse local solo por el enum); `src/lfx/src/lfx/base/knowledge_bases/backends/base.py:1-85` (instancia por uso y teardown); `src/lfx/src/lfx/components/files_and_knowledge/retrieval.py:1-69` (resuelve destino antes de embeddings).
- **Seam VantaDB:** `spec` — frontera SDK/conexiones: límites de SDK/ingestor/conexiones ya existentes y configuración de Studio; el adapter LangChain `VantaDBVectorStore` [V5] ya encapsula `vantadb_py.Client` (su constructor abre cliente). `AdapterDescriptor` es propuesta de frontera, no nuevo tipo del core ni sistema de plugins ejecutables.
- **Acciones:**
  1. Derivar el descriptor del inventario actual.
  2. Centralizar descriptores en el dueño existente.
  3. Separar validación estructural (pura) de apertura de conexión.
  4. Abrir bajo dueño de conexión y cerrar en error/cancelación.
  5. Fallar explícitamente ante backend desconocido o migración requerida; persistir solo campos no secretos compatibles con perfiles existentes; rollback ignora el descriptor añadido sin modificar datos.
- **Dependencias:** configuración tipada, política de destinos, permisos filesystem, cancelación y ciclo de vida de conexiones existentes; config/revisión pertenecen a la conexión y el secreto permanece en su almacén dueño. Converge con C15/C11 de `CONSOLIDATED.md`.
- **FMEA:** localidad inferida por nombre → exportación de ruta inexistente; registro sobrescrito → backend distinto del esperado; import ansioso → fallo por dependencia no usada; teardown omitido → locks/sesiones retenidos; cambio concurrente de destino exige nueva apertura, no mutar el handle activo. Mitigar con registro inmutable/revisión, capacidades puras y cierre por scope.
- **Criterios de aceptación propuestos:** describir un adaptador no hace red; un desconocido no se clasifica como local seguro; un rename visual no cambia la identidad; un error de apertura no deja recursos sin dueño.
- **Criterio negativo:** `describe_without_connecting` nunca hace red ni resuelve secretos; ningún backend se clasifica como local por inferencia de nombre.
- **Independencia:** sí — registro/descriptor puro de frontera SDK, sin marketplace ni proceso plugin nuevo; X-LFL-01 puede consumir sus capabilities.
- **No objetivos:** recrear marketplace ni instalar bundles desde el core Rust.

### X-LFL-03 — Ticket de revisión para publicación de derivados externos

- **Problema:** un delete/migrate durante una generación lenta de embeddings puede resucitar la fuente vieja con el resultado tardío si la escritura no refencea; mantener el lock durante la red bloquea y cancela tarde.
- **Contrato hipotético (tipos propuestos):** `DerivedWriteTicket { source_ref, source_revision, adapter_revision, operation_id }`; `publish_derived(ticket, prepared) -> Published(receipt) | StaleSource | RetiredSource | InvalidEmbedding | Cancelled | StorageError`. La evidencia de commit debe distinguir fuente y proyección.
- **Fuente de evidencia (clone):** `src/lfx/src/lfx/components/files_and_knowledge/knowledge.py:1022-1057` (N2); `src/backend/base/langflow/services/knowledge_base_storage/runtime.py:282-316,448-535` (N4: lease de escritura revalida routing/owner/generación después del proveedor; header del backend aporta segunda comprobación); `src/lfx/src/lfx/base/knowledge_bases/backends/sqlite.py:494-528` (N3). No se porta el proxy dinámico.
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` — Vanta ya planifica y persiste L1 [V3] y tiene provenance, supersession y scheduler [V4]; el delta es conservar el ticket de revisión de origen/configuración desde la planificación hasta la publicación de un derivado externo, sin locks distribuidos nuevos.
- **Acciones:**
  1. Localizar la representación actual de revisión y retirada.
  2. Capturar el ticket autorizado antes del I/O.
  3. Preparar vectores sin locks del motor.
  4. Revalidar el ticket en el seam de escritura existente.
  5. Publicar/registrar el recibo o descartar el trabajo obsoleto.
  6. Reconciliar proyecciones solo si su owner lo permite; fuente/retirada mandan, no el worker.
- **Dependencias:** revisiones de fuente/configuración compatibles con provenance/supersession; seams de planificación y persistencia L1 existentes; X-LFL-04 usa la misma revisión; X-LFL-02 identifica el destino; converge con C14/C13/C4 de `CONSOLIDATED.md`. Alternativas: ampliar `PlannedWrite` internamente si ese flujo lo requiere, o resolver en adapter; rechazado un sistema paralelo de leases. No abrir migración global sin demostrar que los campos actuales no bastan.
- **FMEA:** borrado durante embedding → escritura tardía → resurrección lógica (detectar revisión retirada y rechazar); configuración cambia → vector en generación equivocada (ticket revisionado); mantener lock durante la red → bloqueo/cancelación tardía (separar preparación); publicación parcial en proveedor remoto → residuo (recibo y compensación idempotente por operation ID). No se infiere atomicidad de borrados/contradicciones en todo batch L1.
- **Criterios de aceptación propuestos:** ingesta normal, cancelar antes/después de embedding, delete/migrate concurrente, retry del mismo ticket y provider timeout; el resultado obsoleto no aparece como nuevo vigente; diagnóstico solo IDs, estado y fases — sin contenido/vector.
- **Criterio negativo:** una fuente retirada nunca se resucita por un resultado tardío; el rollback cancela admisión nueva y descarta prepared data, nunca resucita la fuente.
- **Independencia:** parcial — delta sobre el seam de escritura existente; X-LFL-02 lo identifica como destino y X-LFL-04 lo usa cuando publica en fuente migrable, pero el ticket es unidad propia.
- **No objetivos:** prometer exactly-once externo; portar migraciones de Langflow; reemplazar WAL/checkpoints de Vanta; portar APSW, SQL advisory locks ni tablas de migración por similitud nominal.

### X-LFL-04 — Recibo de lote conversacional con reanudación sin nueva decisión

- **Problema:** compactar una conversación una sola vez por lote y reanudar tras fallo de indexación sin pagar otra decisión de modelo inconsistente; el cursor vivo bajo lock evita usar el snapshot obsoleto de dispatch y no se avanza posición sin recibo inequívoco.
- **Contrato hipotético (tipos propuestos):** `ConversationBatchReceipt { session_ref, source_message_refs, source_revision, generation_ref, operation_id, phase }` con fases `Prepared | Published | Rejected | Cancelled`; `resume_batch` no vuelve a pedir modelo si el output válido y autorizado ya existe. Errores `SourceChanged`, `SessionRetired`, `PartialProjection`, `GenerationUnavailable`, `StorageError`; proveedor opcional.
- **Fuente de evidencia (clone):** `src/backend/base/langflow/services/memory_base/task.py:284-429,490-646,838-879` (N6: persiste output de preprocesado y fuente en fase A, escribe backend y marca output/mensajes/cursor en fase B; fingerprint solo de identidad textual relevante en `:390-405`; cursor vivo bajo lock de sesión).
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs` — context assembly/compactación, pipeline worker, log de generación y checkpoints ya existen ([V2], [V3], [V4]); enlazar identidad/revisión del lote fuente, derivado y commit de publicación en esos owners, no crear otra memoria ni scheduler.
- **Acciones:**
  1. Resolver scope y lote estable.
  2. Consultar log/checkpoint actual.
  3. Preparar el output fuera de escritura.
  4. Registrar la referencia — no copiar texto por defecto a auditoría.
  5. Revalidar mensajes relevantes.
  6. Persistir por la ruta L1 existente.
  7. Avanzar posición solo con recibo inequívoco; definir unidad de cursor y efecto de editar/purgar un mensaje antes de aprobar.
- **Dependencias:** context assembly/compaction, pipeline worker, log de generación y checkpoints existentes; condicionada a aprobar resumir sesiones con proveedor; depende de X-LFL-03 únicamente cuando publica derivados en fuente migrable, y de revisiones/scope existentes en todos los casos; converge con C13/C9/C8 de `CONSOLIDATED.md`. Alternativa: ampliar el recibo actual de generación; para compaction efímera no persistir cursor nuevo.
- **FMEA:** cursor adelantado antes del vector → memoria perdida (recibo de publicación); output nuevo en cada retry → coste/inconsistencia (reutilizar output válido); comparar toda la fila → cancelar por actualización de tokens inocua (fingerprint solo de identidad textual relevante); purge concurrente → resucitar datos (revalidación de fuente y fencing); DB y proveedor no comparten commit → parcialidad (reconciliación y limpieza scoped — no llamar atómica a una compensación best-effort). Un lock de proceso no equivale a coordinación entre procesos; cancelar un job no autoriza borrar otro lote.
- **Criterios de aceptación propuestos:** retry sin nueva llamada al modelo; mensaje editado o purgado invalida prepared; finalización de usage no lo invalida; fallo de índice no avanza el cursor; mismo lote en dos jobs conserva dueño. Observabilidad: operación/revisión/fase y causa, sin conversación.
- **Criterio negativo:** el cursor nunca avanza sin recibo de publicación inequívoco; readers legacy no deben interpretar `Prepared` como memoria vigente.
- **Independencia:** parcial — núcleo propio sobre checkpoints/log existentes; depende de X-LFL-03 solo cuando publica en fuente migrable; migración aditiva solo si se demuestra campo faltante; desactivar reanuda la ruta anterior sin repetir operaciones irreversibles.
- **No objetivos:** copiar prompt, kill phrase, threshold del issue ni score de recencia/confianza; nueva política de lifecycle de hechos.

### X-LFL-05 — Historial de ingesta por item separado de spans

- **Problema:** explicar por qué un archivo no entró sin confundir un span correcto, un job terminado y todas las filas publicadas; cerrar un modal no confirma éxito.
- **Contrato hipotético (tipos propuestos):** `IngestionRunView { operation_id, source_ref, revision, attempted_items, outcomes, publication_state }` con `ItemOutcome::Published | Skipped(reason) | Failed(code) | Cancelled`. Los contadores se derivan de outcomes de ese run; token/latencia de span conservan unidad y owner independientes.
- **Fuente de evidencia (clone):** `src/frontend/src/pages/MainPage/pages/knowledgePage/components/IngestionRunDetailModal.tsx:14-222` y `src/frontend/src/pages/MainPage/pages/knowledgePage/components/IngestionRunsSection.tsx:15-188` (historial con `succeeded/failed/skipped`, status `partial`, modal por run y polling solo mientras hay runs no terminales — entidad distinta de la traza/span de ejecución).
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs` (`TaskCheckpointManager`/auditoría [V4]) y actividad Studio; `RetrievalLens` [VS1] ya inspecciona resultados — no debe ser reemplazada.
- **Acciones:**
  1. Cotejar los reportes actuales.
  2. Mapear outcomes por ID estable sin duplicar jobs.
  3. Capturar resultados en el dueño de ingesta.
  4. Publicar la proyección redactada en Studio.
  5. Invalidar las queries de UI al completar/cancelar; revocar permisos invalida el preview aunque el run persista.
- **Dependencias:** `TaskCheckpointManager`/auditoría y actividad Studio (`RetrievalLens`) existentes; X-LFL-03 define publicación y X-LFL-04 añade lote conversacional si aplica — X-LFL-05 no los sustituye; converge con C9/C16 de `CONSOLIDATED.md`. Alternativa: documentación/UI sobre recibos existentes si cubren el contrato.
- **FMEA:** error leído como cero items → falso éxito (outcome explícito); polling conserva terminal viejo → UI desactualizada (asociar revisión/ID); inputs de span en historial → exposición (metadata-only y fetch autorizado); cancelación durante commit → estado contradictorio (mostrar resultado confirmado y fase, no inventar rollback).
- **Criterios de aceptación propuestos:** lote parcial, skips por dedup, error de proveedor, cancelación tardía, revocación de acceso, reabrir historial después de restart; los contadores conservan el denominador sin secretos.
- **Criterio negativo:** un error nunca se lee como «cero items» exitoso; cerrar el modal no confirma éxito; exportar contenido requiere autorización separada.
- **Independencia:** sí como vista/proyección — usa el almacenamiento/versionado de reportes actual y puede desactivarse sin alterar el estado del job; presenta outcomes de los owners de X-LFL-03/X-LFL-04 cuando existan.
- **No objetivos:** nueva consola, copiar el frontend ni instrumentar por obligación cada integración con proveedor externo de tracing.

[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/explain.rs#L1-L100
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L302-L335
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L131-L330
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L190
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L15-L48
[VS1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L130

<!-- audit: 2026-11-23 normalizada ref A -> X-LFL-01, completada ruta IngestionRunsSection.tsx y añadidas definiciones [V1]-[V5],[VS1] -->
