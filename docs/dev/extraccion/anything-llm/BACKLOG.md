---
title: "Backlog de extracción — Anything-LLM"
kind: research
status: active
description: "Candidatas del informe anything-llm: recibo de ingesta por proyección, freshness por revisión, scope por transporte, manifest de entrega, consentimiento de egress, resultado de extracción y selección completa"
tags: [extraccion, backlog, anything-llm]
---

# Backlog de extracción — Anything-LLM

> **Origen:** informe `docs/dev/extraccion/anything-llm/ANYTHING-LLM.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-ALL-01 | 🟠 | Recibo de ingesta por documento y proyección | `vanta-memory/src/ingest/worker.rs`; `vanta-memory/src/ingest/auto_sync.rs`; `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: `IngestReceipt` por identidad de fuente+revisión+destino con fase durable y error tipado (`Parse`, `Embed`, `Persist`, `Cancelled`, `Conflict`); `Accepted` nunca se presenta como `Searchable`. Por qué: upstream devuelve HTTP 200 al despachar el worker nativo y otro 200 puede ocultar documentos fallidos; un spinner/contador no basta para saber si el archivo es usable. Acción mínima: mapear fases reales del worker existente, persistir transición tras cada efecto confirmado y publicar snapshot consultable en Studio. Trazabilidad: informe §Ficha A · evidencia `server/endpoints/workspaces.js:234-294`, `server/utils/EmbeddingWorkerManager.js:91-155` | X-ALL-02 aporta freshness; X-ALL-07 aporta selección completa; converge con C13 MutationReceipt (propuesto) | Contrastar `IngestReport`/`run_with_progress`/`ProgressTracker` y `ingest/merge.rs::commit` existentes antes de añadir campos |
| X-ALL-02 | 🟠 | Freshness por revisión y reemplazo recuperable | `vanta-memory/src/ingest/worker.rs`; `vanta-memory/src/ingest/auto_sync.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: derivación por `(source_id, expected_revision, target, derived_revision)` con watermark por destino y estados `Pending`/`Applied`/`Failed`; preparar nueva proyección, confirmar y activar antes de retirar la vieja. Por qué: upstream borra antes de re-embeber y guarda el texto nuevo aunque un workspace falle; el siguiente sync lo ve sin cambios y no reintenta, dejando texto nuevo con vector viejo. Acción mínima: registrar identidad y revisión sin mutar fuentes, serializar activación por fuente/destino y rechazar publicar una derivación anterior a una edición nueva (`RevisionConflict`). Trazabilidad: informe §Ficha B · evidencia `server/jobs/sync-watched-documents.js:97-260`, `server/__tests__/jobs/sync-watched-documents.test.js:125-158` | X-ALL-01 observa intentos; X-ALL-03 autoriza destino; motivada por issues #3958 y #6806 | Índices y provenance existentes; capacidades del adapter deciden transacción única vs staged revision; no adoptar delete-before-success |
| X-ALL-03 | 🔴 | Scope autorizado y capacidades declaradas por transporte | `vanta-memory/src/core/hooks/auto_recall.rs` (`RecallScope`, `InjectionPolicy`); `desktop/src/transport.ts` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: `AuthorizedRecallRequest { principal_ref, requested_scope, policy_revision }` validado por el host, con resultados autorizados y denegaciones redacted; errores `Unauthenticated`, `Forbidden`, `UnsupportedCapability`. Por qué: los contratos Desktop single-player y servidor multiusuario no son intercambiables; un slug/namespace no sustituye la comprobación de principal y la configuración del cliente nunca amplía privilegios. Acción mínima: inventariar llamadas reales, propagar principal por cada transporte/binding, validar antes de buscar/expandir citas y declarar capacidad real de la conexión. Trazabilidad: informe §Ficha C · evidencia `server/utils/middleware/validatedRequest.js:8-110`, `server/utils/middleware/validWorkspace.js:23-47` | X-ALL-04 reautoriza citas; X-ALL-05 verifica egress; converge con C3/C8/C9 (referencias conceptuales de campaña) | Inventario de principals y salidas por transporte; revisar enforcement de `InjectionPolicy` sin declarar todos los transports cubiertos; no copiar bypass single-user |
| X-ALL-04 | 🟡 | Manifest de entrega de contexto con revisión | `vanta-memory/src/context_engine/engine.rs::assemble_inner`; `vanta-memory/src/core/hooks/auto_recall.rs::perform_auto_recall_governed` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `DeliveryManifest` por turno con source ID, revisión, origen current/history/pin y estado incluido/compactado/excluido. Por qué: el contexto entregado y las citas mostradas pueden diferir por diseño; una lista de hits no equivale a ledger de entrega ni prueba de que el modelo consumió la afirmación. Acción mínima: vincular referencias al ensamblado final tras presupuesto, persistir manifest mínimo ligado al turno y abrir en Studio la revisión, no el archivo actual por nombre. Trazabilidad: informe §Ficha D · evidencia `server/utils/chats/stream.js:224-240`, `frontend/src/components/WorkspaceChat/ChatContainer/ChatHistory/Citation/index.jsx:60-90` | X-ALL-02 suministra revisión; X-ALL-03 autoriza; `memory_generation_log` registra generación, no el mismo evento | Provenance e `IntegratedContext`/`CompactionReport` existentes; mapear source/revision al assembler y generation log |
| X-ALL-05 | 🟠 | Consentimiento de egress ligado al destino efectivo | Configuración de host/adapters de embedding; `desktop/src/transport.ts`, `desktop/src/components/layout/ConnectionSidebar.tsx` (Studio) | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `EgressDecision` por principal+operación+destino normalizado+revisión de configuración, con recibo versionado que se invalida al cambiar endpoint/finalidad; `ConsentRequired`, `DestinationChanged`, `UnknownPolicy` nunca se convierten en permiso. Por qué: la pantalla de privacidad upstream informa pero no persiste aceptación auditable; cambiar embedder local→remoto debe exigir nueva decisión. Acción mínima: inventariar salidas (incluidos favicon/telemetría), mostrar destino y clase de datos antes de transmitir y auditar decisiones sin secretos. Trazabilidad: informe §Ficha E · evidencia `frontend/src/pages/OnboardingFlow/Steps/DataHandling/index.jsx:7-35`, `frontend/src/components/ProviderPrivacy/index.jsx:21-70` | X-ALL-03 identifica actor; converge con C11/C15 (referencias conceptuales) | X-ALL-03 para principal; proveedores opcionales existentes; jurisdicción y retención de recibos requieren decisión del owner |
| X-ALL-06 | 🟠 | Resultado de extracción no ambiguo por ventana | `vanta-memory/src/core/record/l1_writer.rs` (`plan_write`, `persist_planned`, `apply_dedup_batch`); generation log existente | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `ExtractionOutcome` por ventana y revisión con `NoFacts`, `Applied(ids)`, `RetryableFailure`, `Rejected` o `Cancelled`; cursor confirmado solo por efecto aplicado. Por qué: upstream marca chats procesados en `finally` incluso en error; extracción vacía, fallida y aplicada deben distinguirse, y un timeout no puede contarse como «sin hechos». Acción mínima: delimitar ventana, registrar intento, validar salida y propiedad antes de `plan_write`, persistir y confirmar cursor tras el commit. Trazabilidad: informe §Ficha F · evidencia `server/jobs/extract-memories.js:119-181`, `server/models/memory.js:345-439` | X-ALL-01 observa; X-ALL-03 autoriza; X-ALL-04 enlaza origen; converge con C16 (referencia conceptual) | Extracción/dedup/contradicciones existentes; ningún ID sugerido por LLM autoriza editar otro propietario; cupos revalidados donde se publica |
| X-ALL-07 | 🟡 | Plan de selección de corpus completo antes de mutar | Studio `desktop/src/components/lens/retrieval/RetrievalLens.tsx`; `desktop/src/vanta.ts` (`transport.call`, `listPage`) | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: `SelectionPlan(scope, filter, cursor_revision)` que resuelve IDs/revisiones y declara alcance visible/todo; el botón de stage no ejecuta ingest ni delete. Por qué: preparar UI no es mutación; una carpeta mayor que la página visible debe resolverse completa o declararse parcial, nunca amplificar silenciosamente el lote. Acción mínima: distinguir visible/todo, resolver páginas con reconciliación count/cursor, confirmar plan y enviar operación idempotente de X-ALL-01. Trazabilidad: informe §Ficha G · evidencia `frontend/src/components/Modals/ManageWorkspace/Documents/hooks/useDocumentPicker.js:643-689`, `frontend/src/components/Modals/ManageWorkspace/Documents/index.jsx:44-76` | X-ALL-01 ejecuta la operación; X-ALL-03 autoriza cada ID al commit | Verificar semántica de `listPage` y alcance visible/todo de Studio; principal autoriza cada ID al commit; cancelar expansión no muta |

## Fichas completas

> Expansión de cada candidata a partir del informe `anything-llm/ANYTHING-LLM.md` (§Fichas A–G). Todos los contratos son **hipotéticos**: propuestas, no APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `308b2766`; las rutas VantaDB requieren revalidación contra HEAD.

### X-ALL-01 — Recibo de ingesta por documento y proyección

- **Problema:** un host debe saber si puede usar el archivo recién enviado; upstream devuelve HTTP 200 al despachar el worker y otro 200 puede ocultar documentos fallidos. Un spinner/contador no basta para saber si el archivo es usable: `Accepted` nunca equivale a `Searchable`.
- **Contrato hipotético:** `IngestReceipt` recibe identidad de fuente+revisión+destino+operación idempotente y devuelve fase, revisión visible y error tipado (`Parse`, `Embed`, `Persist`, `Cancelled`, `Conflict`). `Accepted` no implica `Searchable` ni obliga a ejecutar una consulta pagada. Un coordinador por identidad/revisión correlaciona eventos con el intento y rechaza progreso de un intento obsoleto.
- **Fuente de evidencia (clone):** `server/endpoints/workspaces.js:234-294` (workspace API acepta trabajo antes de terminar); `server/utils/EmbeddingWorkerManager.js:91-155` (worker manager); `server/utils/workspaceParsedFiles.js:110-174` (borra fila en `finally` antes de confirmar embed — lección issue #6806, cerrada not-planned como UX intencional).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs`, `vanta-memory/src/ingest/auto_sync.rs` (`AutoSyncScheduler` ya evita busy y descarta paquetes de runs antiguos), `vanta-memory/src/utils/task_checkpoint.rs` (`TaskCheckpointManager`, `IngestReport`, `run_with_progress`, `ProgressTracker` ya informan fases). El delta es persistencia/lectura por documento y proyección — no otra cola ni «añadir observabilidad» desde cero.
- **Acciones:**
  1. Mapear las fases reales del worker existente y contrastar `ingest/merge.rs::commit` antes de añadir campos (prerrequisito §5-1).
  2. Persistir transición después de cada efecto confirmado, por documento/revisión/destino.
  3. Publicar snapshot consultable por API y mostrarlo en Studio.
  4. Migrar registros previos como estado desconocido, no éxito inventado; desactivar la emisión nueva mantiene lectura y reintentos existentes.
- **Dependencias:** X-ALL-02 aporta freshness; X-ALL-07 aporta selección completa; converge con C13 MutationReceipt (propuesto). Desconectar SSE no cancela trabajo; cancelación explícita devuelve el estado observado, sin prometer deshacer embeddings.
- **FMEA:** crash tras vectorizar y antes del recibo → reintento duplicado (detectar por identidad de proyección y reconciliar antes de repetir); SSE viejo → falso Ready (detectar mismatch de revisión y consultar estado durable); logs que incluyen texto del documento (prohibido: logs sin texto).
- **Criterios de aceptación propuestos:** envío normal acaba con revisión consultable; caída del parser, del proveedor y de la persistencia quedan distinguibles; reinicio y dos envíos iguales no pierden ni duplican trabajo; cancelar no produce Ready.
- **Criterio negativo:** un `Accepted` nunca se presenta como documento usable; un intento obsoleto no puede reportar progreso sobre el intento vigente.
- **Independencia:** delta sobre el worker/checkpoint existentes; alternativas descartadas: solo SSE (pierde reinicios) y polling del primario (no prueba proyección).
- **No objetivos:** no crear servicio collector nuevo ni progreso porcentual ficticio; queda abierto qué frontera acredita la disponibilidad de cada índice.

### X-ALL-02 — Freshness por revisión y reemplazo recuperable

- **Problema:** una fuente compartida por dos workspaces cambia y uno falla al embeber: upstream borra antes de re-embeber y guarda el texto nuevo aunque un workspace falle; el siguiente sync lo ve sin cambios y no reintenta, dejando texto nuevo con vector viejo.
- **Contrato hipotético:** derivación con entrada `(source_id, expected_revision, target, derived_revision)` y salida revisión activa por destino con estados `Pending`/`Applied`/`Failed`; el error de un destino no avanza su watermark; `RevisionConflict` impide publicar una derivación anterior a una edición nueva.
- **Fuente de evidencia (clone):** `server/jobs/sync-watched-documents.js:97-112,124-260` (syncQueue borra antes de añadir y guarda fuente aun con fallos); `server/__tests__/jobs/sync-watched-documents.test.js:125-158` (el test conserva ese comportamiento). Lecciones: issue #3958 (persistencia tras delete + `skipCache` invertido; cierre `d1d68af` no leído completo) e issue #6806.
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs` y `vanta-memory/src/ingest/auto_sync.rs` — integrar revisión de derivación en la escritura existente, no reemplazar WAL ni diagnosticar un defecto local desde un patrón upstream.
- **Acciones:**
  1. Registrar identidad y revisión sin mutar fuentes.
  2. Derivar la nueva proyección, confirmarla y activarla; solo después retirar la vieja.
  3. Persistir pendientes y tombstones antes del cleanup.
  4. Serializar la activación por fuente/destino (no todas las ingestas).
  5. Legado: revisión desconocida hasta reconciliar; rollback desactiva la activación nueva conservando la revisión anterior, sin resucitar documentos borrados.
- **Dependencias:** índices y provenance existentes; X-ALL-01 observa intentos; X-ALL-03 autoriza destino. La elección entre transacción única (si ambos efectos están en el mismo dominio) y staged revision+activación (proveedor externo) requiere las capacidades del adapter.
- **FMEA:** pérdida de mapping → huérfanos (detectar inventario contra IDs, reconciliar antes de purgar); texto nuevo/vector viejo → inconsistencia (mostrar ambas revisiones); proveedor sin versionado → no prometer atomicidad, exigir degradación explícita; cancelación durante embedding → conserva pendiente y última revisión activa; delete gana frente a una derivación que termina tarde.
- **Criterios de aceptación propuestos:** cambio normal actualiza todos los destinos; un fallo deja uno atrasado y reintentable; edición concurrente no publica versión vieja; delete concurrente y reinicio no reactivan contenido; todo sin inspeccionar payloads.
- **Criterio negativo:** nunca queda texto nuevo servido con vector viejo marcado como fresco; un borrado nunca se revierte por una derivación tardía.
- **Independencia:** integración sobre índices/provenance existentes; delete-before-add rechazado explícitamente; no adoptar la semántica upstream de delete-before-success.
- **No objetivos:** no transacción distribuida universal, no nuevo índice, no garantía de purga física de backups/citas históricas (la retención exige decisión separada).

### X-ALL-03 — Scope autorizado y capacidades declaradas por transporte

- **Problema:** los contratos Desktop single-player y servidor multiusuario no son intercambiables; un slug/namespace no sustituye la comprobación de principal y la configuración del cliente nunca amplía privilegios. Un scope omitido es una fuga, no un default razonable.
- **Contrato hipotético:** `AuthorizedRecallRequest { principal_ref, requested_scope, policy_revision }` validado por el host; la salida devuelve resultados autorizados y denegaciones redacted; errores `Unauthenticated`, `Forbidden`, `UnsupportedCapability`. La petición se liga a revisión de política; `MemoryScope` no se acredita como tipo existente.
- **Fuente de evidencia (clone):** `server/utils/middleware/validatedRequest.js:8-110` (auth multiusuario), `server/utils/middleware/validWorkspace.js:23-47` (scope por workspace); documentación Desktop oficial como contrato distinto. El widget/gitlinks no se usan como evidencia.
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` (`RecallScope`, `InjectionPolicy`, `perform_auto_recall_governed` ya gobiernan inyección); bindings/MCP y `desktop/src/transport.ts` deben informar la capacidad real de su conexión.
- **Acciones:**
  1. Inventariar llamadas reales y principals por cada transporte/binding (prerrequisito §5-2).
  2. Propagar principal por transporte; validar antes de buscar/expandir citas.
  3. Exponer capacidad real de la conexión sin convertirla en autorización.
  4. Versionar política y configuración de conexión sin almacenar bearer tokens en registros de memoria.
  5. Migración de legado explícita; rollback fail-closed para uso compartido.
- **Dependencias:** inventario de principals y salidas por transporte; revisar enforcement de `InjectionPolicy` sin declarar todos los transports cubiertos por su existencia; X-ALL-04 reautoriza citas; X-ALL-05 verifica egress.
- **FMEA:** scope omitido → fuga (detectar petición sin principal y denegar); entorno Desktop expuesto a LAN → alcance ampliado (advertir capacidad real, no asumir auth); revocación durante streaming → detener nuevas entregas, sin prometer borrar lo enviado; cancelar request libera la consulta, no cambia pertenencia ni borra memoria.
- **Criterios de aceptación propuestos:** principal válido ve solo su alcance; usuario suspendido, thread ajeno, ruta alternativa y política revocada no entregan contenido; soporte desconocido produce error, no fallback silencioso.
- **Criterio negativo:** la configuración del cliente nunca amplía privilegios; un namespace nunca actúa como ACL; capacidad desconocida nunca degrada a permiso.
- **Independencia:** gobierna X-ALL-04/X-ALL-05/X-ALL-07; alternativas rechazadas: namespace como ACL y confiar solo en la UI.
- **No objetivos:** no crear servicio IAM, otro protocolo, ni copiar el bypass single-user.

### X-ALL-04 — Manifest de entrega de contexto con revisión

- **Problema:** el contexto entregado y las citas mostradas pueden diferir por diseño (backfill amplía contexto sin añadir fuentes a las citas); una lista de hits no equivale a ledger de entrega ni prueba de que el modelo consumió la afirmación.
- **Contrato hipotético:** `DeliveryManifest` por turno recibe el ensamblado y las referencias autorizadas y devuelve por cada una: source ID, revisión, origen `current`/`history`/`pin` y estado `incluido`/`compactado`/`excluido`. Errores `MissingRevision`, `PolicyChanged`, `SourceUnavailable`. El manifest no implica que el modelo leyó o verificó la afirmación.
- **Fuente de evidencia (clone):** `server/utils/chats/stream.js:224-240` (`streamChatWithWorkspace` amplía contexto histórico sin añadir esas fuentes a las citas actuales); `frontend/src/components/WorkspaceChat/ChatContainer/ChatHistory/Citation/index.jsx:60-90` (Citation presenta metadata).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs::assemble_inner` y `vanta-memory/src/core/hooks/auto_recall.rs::perform_auto_recall_governed`, sobre provenance e `IntegratedContext`/`CompactionReport` existentes; `memory_generation_log` registra generación — no el mismo evento.
- **Acciones:**
  1. Vincular referencias al ensamblado final **después** del presupuesto.
  2. Persistir manifest mínimo ligado al turno (host dueño del manifest).
  3. Abrir en Studio la revisión citada, no el archivo actual por nombre.
  4. Legado = desconocido; rollback desactiva nuevos manifests sin eliminar provenance ni historial.
  5. Borrado/revocación obliga a reautorizar navegación y evita rescatar contenido.
- **Dependencias:** X-ALL-02 suministra revisión; X-ALL-03 autoriza; mapear source/revision/cursor a `assemble_inner`, generation log y escritura L1 (prerrequisito §5-3); preservar el prefijo protegido, que puede superar presupuesto (`engine.rs:197-218`).
- **FMEA:** cita vieja tras refresh → detectar mismatch y mostrar no disponible; fuente recuperada pero descartada por presupuesto → no mostrarla como prueba usada; cerrar el manifest antes de conocer lo enviado → entrega parcial etiquetada; contadores sin texto.
- **Criterios de aceptación propuestos:** current+history+pins quedan diferenciados; la compactación no inventa fuentes; interrupción muestra parcial etiquetada; borrado no revela payload desde el inspector.
- **Criterio negativo:** una cita nunca certifica que el modelo verificó la afirmación; un descarte por presupuesto nunca aparece como evidencia usada.
- **Independencia:** extensión del assembler existente; alternativas descartadas: lista de hits (insuficiente) y guardar el prompt completo (riesgo de privacidad).
- **No objetivos:** no judge LLM, no score de confianza, no almacenar prompts íntegros; la verificación factual requiere otro mecanismo.

### X-ALL-05 — Consentimiento de egress ligado al destino efectivo

- **Problema:** documentos privados pueden cambiar de embedder local a remoto; la pantalla de privacidad upstream informa pero no persiste aceptación auditable, y provider privacy no es enforcement de egress. Cambiar embedder local→remoto debe exigir nueva decisión.
- **Contrato hipotético:** `EgressDecision` recibe principal, operación, destino normalizado y revisión de configuración; devuelve permiso/denegación con referencia a recibo versionado que se invalida al cambiar endpoint/finalidad. `ConsentRequired`, `DestinationChanged`, `UnknownPolicy` nunca se convierten en permiso por disponibilidad del proveedor.
- **Fuente de evidencia (clone):** `frontend/src/pages/OnboardingFlow/Steps/DataHandling/index.jsx:14-35` (informa pero no persiste aceptación); `frontend/src/components/ProviderPrivacy/index.jsx:29-46` (distingue tres proveedores — información, no enforcement).
- **Seam VantaDB:** configuración del host/adapters de embedding y conexiones Studio (`desktop/src/transport.ts`, `desktop/src/components/layout/ConnectionSidebar.tsx`); proveedores opcionales ya existen — no insertar consentimiento dentro de BM25/HNSW.
- **Acciones:**
  1. Inventariar salidas reales, incluidos favicon y telemetría.
  2. Mostrar destino y clase de datos antes de transmitir.
  3. Persistir recibo mínimo versionado e invalidarlo al cambiar endpoint/finalidad.
  4. Validar justo antes de enviar en el host; auditar decisiones sin secretos ni documentos.
  5. Migración sin recibo pide nueva autorización; rollback no habilita egress previamente denegado.
- **Dependencias:** X-ALL-03 identifica al actor; proveedores opcionales existentes; jurisdicción y retención de recibos requieren decisión del owner; converge con C11/C15 (referencias conceptuales).
- **FMEA:** endpoint custom clasificado como local → fuga (comparar destino resuelto y avisar desconocido); recibo viejo → detectar revision mismatch; edición concurrente invalida la decisión; revocar impide envíos futuros pero no borra datos ya transmitidos ni garantiza cancelación del proveedor remoto.
- **Criterios de aceptación propuestos:** destino estable aceptado funciona; cambio de hostname, embedder o política exige nueva decisión; rechazar no transmite; abrir citas no produce egress oculto sin política.
- **Criterio negativo:** `ConsentRequired`/`DestinationChanged`/`UnknownPolicy` nunca se tratan como permiso; un endpoint desconocido nunca se clasifica como local por defecto.
- **Independencia:** requiere X-ALL-03 para el principal; alternativas descartadas: aviso genérico al instalar (caduca) y bloquear toda red (limita uso legítimo).
- **No objetivos:** no asesoría legal ni afirmar que la pantalla upstream demuestra cumplimiento; jurisdicción y retención de recibos quedan como decisión del owner.

### X-ALL-06 — Resultado de extracción no ambiguo por ventana

- **Problema:** upstream marca chats procesados en `finally` incluso en error; extracción vacía, fallida y aplicada deben distinguirse — un timeout no puede contarse como «sin hechos» ni un booleano `processed` como resultado.
- **Contrato hipotético:** `ExtractionOutcome` recibe IDs/revisiones de chats, configuración y scope; salida `NoFacts`, `Applied(ids)`, `RetryableFailure`, `Rejected` o `Cancelled`; `NoFacts` exige llamada válida — no timeout ni parse fallido. El cursor se confirma solo por efecto aplicado; ningún ID sugerido por LLM autoriza editar otro propietario.
- **Fuente de evidencia (clone):** `server/jobs/extract-memories.js:119-181` (`processGroup` distingue candidatos nulos/vacíos en logs pero marca procesados en `finally`); `server/models/memory.js:345-439` (`applyExtractedMemories` valida ownership de updates y aplica transacción Prisma).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` (`plan_write`, `persist_planned`, `apply_dedup_batch`) y checkpoint/generation log existentes — Vanta ya extrae, deduplica y marca contradicciones; no crear un Reflector nuevo.
- **Acciones:**
  1. Delimitar la ventana y registrar el intento.
  2. Validar salida y propiedad antes de `plan_write`.
  3. Persistir y confirmar el cursor tras el commit.
  4. Reconciliar efecto/cursor tras crash.
  5. Revalidar cupos donde se publica (los revisados antes del plan pueden quedar obsoletos); legado conserva significado desconocido; rollback suspende extracción nueva sin revertir hechos aplicados.
- **Dependencias:** extracción/dedup/contradicciones existentes; X-ALL-01 observa, X-ALL-03 autoriza, X-ALL-04 enlaza origen; converge con C16 (referencia conceptual); exclusión por ventana/scope y revisión esperada.
- **FMEA:** timeout marcado como vacío → pérdida de memoria (detectar error tipado, no avanzar); cursor atrasado tras commit → duplicados (reconciliar IDs con la dedup existente); guardar raw LLM (prohibido); cancelación antes del commit no marca éxito — posterior devuelve aplicado.
- **Criterios de aceptación propuestos:** ventana sin hechos avanza como `NoFacts`; error temporal queda pendiente; respuesta hostil/ID ajeno se rechaza; dos workers y reinicio no duplican hechos ni exceden política sin señal explícita.
- **Criterio negativo:** un timeout o parse fallido nunca se contabiliza como «sin hechos»; un chat procesado-en-finally sin efecto no avanza el cursor.
- **Independencia:** delta sobre el writer/dedup existentes; alternativas descartadas: booleano `processed` (ambiguo) y reintentar todo (duplica coste).
- **No objetivos:** no prompts nuevos, no consenso factual, no copiar cupos upstream.

### X-ALL-07 — Plan de selección de corpus completo antes de mutar

- **Problema:** el usuario marca una carpeta mayor que la página visible; preparar UI no es mutación y una selección parcial silenciosa no debe amplificar el lote — debe resolverse completa o declararse parcial.
- **Contrato hipotético:** `SelectionPlan(scope, filter, cursor_revision)` devuelve IDs/revisiones y alcance `visible`/`todo`; errores `SelectionChanged`, `Forbidden`, `Cancelled`. El botón de stage no ejecuta ingest ni delete.
- **Fuente de evidencia (clone):** `frontend/src/components/Modals/ManageWorkspace/Documents/index.jsx:44-76` (resuelve selección antes de stage y POST); `frontend/src/components/Modals/ManageWorkspace/Documents/hooks/useDocumentPicker.js:643-689` (frontera de expansión).
- **Seam VantaDB:** Studio (`desktop/src/components/lens/retrieval/RetrievalLens.tsx`:82-104 carga una página para elegir vector, :121-129 filtra visualmente — no acredita selección masiva completa) y `desktop/src/vanta.ts` (`transport.call`, `listPage`); verificar la semántica de `listPage` y el alcance visible/todo antes de implementar (prerrequisito §5-4).
- **Acciones:**
  1. Distinguir selección visible de «todo» en la UI.
  2. Resolver páginas con reconciliación count/cursor.
  3. Confirmar el plan y enviar la operación idempotente de X-ALL-01.
  4. Persistir solo el manifest de operación si cruza reinicios.
  5. Rollback de UI conserva la selección visible explícita, nunca amplía silenciosamente el lote.
- **Dependencias:** X-ALL-01 ejecuta la operación; X-ALL-03 autoriza cada ID al commit; cancelar la expansión no muta nada.
- **FMEA:** página faltante → omisión (reconciliar count/cursor); documento nuevo durante el paginado → scope drift (exigir revisión o avisar selección no congelada); cambio concurrente → conflicto o resultado por ID; progreso sin nombres sensibles por defecto.
- **Criterios de aceptación propuestos:** carpeta multipágina se resuelve completa o se declara parcial; buscar y seleccionar visibles no incluye ocultos; cancelación y revocación no disparan borrado ni embedding.
- **Criterio negativo:** una selección mayor que la página visible nunca se ejecuta como si fuera completa sin declararlo; el filtro visual nunca actúa como filtro de motor.
- **Independencia:** trabajo de Studio separado del motor; reutiliza `listPage` y mutaciones oficiales, sin migración del motor.
- **No objetivos:** no crear un explorador nuevo ni convertir el filtro visual en filtro del motor.

<!-- audit: 2026-11-23 completadas rutas desktop/src en X-ALL-05 y X-ALL-07 -->
