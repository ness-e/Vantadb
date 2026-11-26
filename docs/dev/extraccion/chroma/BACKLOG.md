---
title: "Backlog de extracción — Chroma"
kind: research
status: active
description: "Candidatas del informe chroma: recibo de mutación por dominios, materialización ordenada, dominio efectivo de filtros, publicación/retención de generaciones, auditoría offline y contrato de backend"
tags: [extraccion, backlog, chroma]
---

# Backlog de extracción — Chroma

> **Origen:** informe `docs/dev/extraccion/chroma/CHROMA.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-CHR-01 | 🟠 | Recibo de mutación por dominios de durabilidad y aplicación | `src/wal.rs` (append/sync/SyncMode); `src/backend.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: `mutation_status(namespace, mutation_id, revision) -> MutationStatus` con dominios `Accepted`, `PrimaryDurable`, `Applied`, `Checkpointed`, `Unknown` y watermark observado. Por qué: tras un error poscommit el agente no sabe si la mutación escribió o no; aplicado no implica fsync y una posición WAL no es revisión MVCC. Acción mínima: inventariar retornos existentes, marcar punto de commit por backend, enlazar aplicación de derivados y exponer fase/ID al SDK y al diagnóstico. Trazabilidad: informe §Ficha A · evidencia `rust/log/src/sqlite_log.rs:330-414`, `rust/segment/src/local_hnsw.rs:715-788`, `rust/wal3/README.md:1-105` | X-CHR-02 aporta evidencia de aplicación; X-CHR-04 publica; converge con C13 MutationReceipt (propuesto) | Identidad de mutación, SyncMode efectiva y punto de commit por backend; registros previos migran como desconocido, no durable inventado |
| X-CHR-02 | 🟠 | Materialización ordenada donde delete invalida la versión compactada | `src/backend.rs` (tombstones/versions/particiones); escritura L1 (`l1_writer`, dedup/supersession) | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: `apply_derived_change(VersionedChange, expected: DerivedRevision)` con identidad `(namespace,key,source_revision,operation_id)`; solo la revisión vigente produce derivado consultable. Por qué: un dato borrado/corregido no debe seguir en el índice antiguo ni reaparecer tras compactar/reiniciar; la consulta debe ocultar el viejo compactado antes de su purga física. Acción mínima: trazar insert/update/delete de cada rama, formalizar casos add→delete y delete→upsert y aplicar delta con revisión y máscara de tombstones. Trazabilidad: informe §Ficha B · evidencia `rust/segment/src/types.rs:764-1020`, `rust/worker/src/execution/operators/filter.rs:715-875`, `rust/worker/src/execution/orchestration/knn.rs:166-235` | X-CHR-01 identifica mutación; X-CHR-04 publica generación; X-CHR-05 verifica cobertura | Revisión/tombstone propios y orden explícito por ID; derivado antiguo sin revisión se audita/reconstruye, no se asume fresco |
| X-CHR-03 | 🟡 | Reporte del dominio efectivo de filtros y cobertura | `src/sdk/search/vector.rs`; `src/sdk/search/hybrid.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `explain_filter(namespace, query, filters, revision) -> FilterPlanReport` opt-in que declara por rama `PreIndex`, `ExactScan`, `PostCandidate` o `Unsupported`, con revisión y límite aplicado. Por qué: un scope restrictivo aplicado después de truncar ANN produce «sin memoria» falso; un filtro de seguridad no puede degradarse a opcional ni explicarse como estimación. Acción mínima: reutilizar la decisión efectiva del planner existente y adjuntar reporte versionado en API, sin exponer payload ni conteos entre tenants. Trazabilidad: informe §Ficha C · evidencia `rust/worker/src/execution/operators/filter.rs:715-875`, `rust/worker/src/execution/operators/knn_hnsw.rs:38-70`, `rust/frontend/src/impls/service_based_frontend.rs:3031-3115` | X-CHR-02/X-CHR-04 aportan revisión; X-CHR-06 declara capacidades | Planner prefilter/scan/máscara y autorización existentes; reporte efímero sin migración de datos ni nueva gramática |
| X-CHR-04 | 🟠 | Compactación preparada, publicación y retención separadas | `src/sdk/api/admin.rs::Embedded::rebuild_index`; `src/sdk/serialization/impl_rebuild.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: `prepare_derived_revision(source_revision, cancel) -> PreparedDerived` y `publish(prepared, expected_current) -> PublishedRevision`, preparando fuera del lector y publicando con comparación de revisión. Por qué: una tarea completada no debe dejar vector nuevo con metadata vieja ni perder archivo activo; aplicar/flush no es publicar catálogo. Acción mínima: inventariar locks y manifiesto vigentes, mapear cambios sobre las fases de `rebuild_index` (vector, derivados, texto, sparse, scalar) y retirar generación vieja solo tras liberar lectores con cobertura documentada. Trazabilidad: informe §Ficha D · evidencia `rust/worker/src/execution/orchestration/compact.rs:710-728,919-1096`, `rust/worker/src/execution/orchestration/register_orchestrator.rs:59-77` | X-CHR-01/X-CHR-02 alimentan; X-CHR-05 inspecciona el preparado; X-CHR-06 evita doble writer | Inventario de ownership/publicación y scheduler existentes; manifiesto versionado/CAS/lease como alternativas condicionadas, no mínimo implícito |
| X-CHR-05 | 🟠 | Auditoría relacional y reparación solo sobre copia | `src/cli_handlers/index.rs` (rebuild); `src/sdk/search/audit.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: `audit_generation(scope, revision) -> GenerationAudit`, `plan_repair(audit, expected) -> RepairPlan` y `apply_offline_copy(plan, target)` sobre destino nuevo autorizado; `CannotRead` nunca es `Clean`. Por qué: un checkpoint inconsistente puede dejar recuerdos inaccesibles y una reparación automática puede destruir evidencia de recovery; check, findings y error deben distinguirse. Acción mínima: ampliar auditoría existente con relación fuente↔vector↔ID↔watermark, emitir plan y reparar copia verificada dejando al operador la decisión de promoción. Trazabilidad: informe §Ficha E · evidencia `rust/cli/src/hnsw_integrity_check.rs:25-179`, `rust/cli/src/hnsw_config_repair.rs:146-329`, `rust/segment/src/local_hnsw.rs:1353-1417` | X-CHR-01/X-CHR-04 aportan watermarks y publicación; motivada en parte por issue #7946 del informe | Auditorías/rebuild existentes, copia consistente y rutas seguras; exclusión de escritores; dominio de reparación declarado por caso, no universal |
| X-CHR-06 | 🟡 | Contrato de backend y ownership de índice vivo | `src/backend.rs` (`StorageBackend`, capabilities); bindings oficiales | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: `describe_backend(handle) -> BackendContract` con persistencia, filtros, sync y formato; opcional `acquire_index(index_id, expected_revision) -> IndexLease` RAII donde deleted/failed impiden mutación y un miss concurrente no fabrica otro writer. Por qué: la misma firma puede funcionar en local y fallar en servidor; una expulsión de caché no debe crear dos escritores del mismo índice. Acción mínima: revisar capacidades por binding/backend, propagar `Unsupported` sin fallback silencioso y verificar identidad de handles bajo misses. Trazabilidad: informe §Ficha F · evidencia `chromadb/__init__.py:201-315,375-433`, `rust/segment/src/local_segment_manager.rs:40-105,173-258`, `rust/segment/src/local_hnsw.rs:777-788` | X-CHR-01 para semántica de errores; X-CHR-04/X-CHR-05 para ciclo de vida | Capabilities por backend/binding ya declaradas; formatos previos pasan por X-CHR-05 antes de marcarse usables |

## Fichas completas

> Expansión de cada candidata a partir del informe `chroma/CHROMA.md` (§Fichas A–F). Todos los contratos son **hipotéticos**: firmas conceptuales no implementadas; los nombres de revisión/token deben reutilizar tipos existentes si ya expresan el invariante. La evidencia `archivo:líneas` cita el clone upstream fijado en `1a3ac2c7` (rangos de lectura acotada, no auditoría del archivo completo); las rutas VantaDB citan el snapshot `97c9623` y requieren revalidación contra HEAD.

### X-CHR-01 — Recibo de mutación por dominios de durabilidad y aplicación

- **Problema:** tras guardar una corrección el agente recibe error y la vuelve a insertar sin saber si falló antes o después del commit; aplicado no implica fsync y una posición WAL no es revisión MVCC.
- **Contrato hipotético:** `mutation_status(namespace, mutation_id, revision) -> Result<MutationStatus, StatusError>` con dominios `Accepted`, `PrimaryDurable`, `Applied`, `Checkpointed`, `Unknown`; el input incluye backend/instancia y el output watermark observado y capacidades. Invariantes: `Unsupported`, I/O y generación obsoleta no son «pendiente» ni éxito.
- **Fuente de evidencia (clone):** `rust/log/src/sqlite_log.rs:330-414` (`SqliteLog::push_logs`: transacción de embeddings_queue, commit y después backfill/purge); `rust/segment/src/local_hnsw.rs:715-788` (distingue `applied_seq_id` de checkpoint); `rust/wal3/README.md:1-105` (offsets/fragments/cursors — su nombre no prueba soporte de migración «v2→v3»).
- **Seam VantaDB:** `src/wal.rs` (append/batch/sync, formatos y política de sync) y `src/backend.rs` (ya persiste derivados) — extender el recibo del camino SDK de escritura y health existentes; no crear otro WAL ni usar el checkpoint de tareas.
- **Acciones:**
  1. Inventariar retornos existentes del camino de escritura.
  2. Marcar el punto de commit por backend.
  3. Enlazar la aplicación de derivados al recibo.
  4. Propagar fase/ID al SDK y al diagnóstico — ninguna espera mantiene locks que necesite el productor del estado.
  5. Empezar como proyección diagnóstica del estado real; registros anteriores = `Unknown`, no durable inventado; desactivar la exposición no altera formato WAL.
- **Dependencias:** identidad de mutación, SyncMode efectiva y punto de commit por backend (prerrequisito §5); X-CHR-02 aporta evidencia de aplicación; X-CHR-04 publica; recuperación y publicación existentes.
- **FMEA:** cancelación tras commit → retry duplicado (detectar fase/ID y devolver indeterminado recuperable); fallo de índice tras append → falso rechazo (distinguir durable/applied y conservar log); reopen con ID de otra instancia → falsa observación (scope por instancia+namespace).
- **Criterios de aceptación propuestos:** favorable: escritura y reopen conservan el mismo estado demostrado; adversos: error poscommit, cancelación y backend sin persistencia no mienten; observar posiciones/fases, no documentos.
- **Criterio negativo:** un error poscommit nunca se responde como «no escrito» ni como «escrito» sin dominio; `Unknown` nunca se presenta como durable.
- **Independencia:** proyección diagnóstica sobre el recibo existente; alternativas documentadas: solo documentar (menor coste, no consultable) o extender resultado SDK (preferida); ledger paralelo rechazado por doble verdad. Un adaptador Chroma no es requisito.
- **No objetivos:** no garantía distribuida, no formato wal3, no migración de una versión de WAL no leída, no exactly-once universal ni nuevo scheduler.

### X-CHR-02 — Materialización ordenada donde delete invalida la versión compactada

- **Problema:** un dato borrado/corregido sigue en el HNSW antiguo o reaparece tras compactar/reiniciar; la consulta debe ocultar el viejo compactado antes de su purga física.
- **Contrato hipotético:** `apply_derived_change(change: VersionedChange, expected: DerivedRevision) -> Result<AppliedDelta, ApplyError>` con identidad `(namespace, key, source_revision, operation_id)`; la salida expresa ramas aplicadas y tombstone; errores `Stale`, `MissingSource`, `Corrupt`, `Unsupported`; ausencia voluntaria no se confunde con fallo de lectura; solo la revisión vigente produce derivado consultable.
- **Fuente de evidencia (clone):** `rust/segment/src/types.rs:764-1020` (`materialize_logs`: estados y merges por ID — contrato de replay, no dedup LLM); `rust/worker/src/execution/operators/filter.rs:715-875` (`Filter::run` excluye del compactado IDs modificados por log); `rust/worker/src/execution/orchestration/knn.rs:166-235` (`Knn::run` omite `DeleteExisting`).
- **Seam VantaDB:** `src/backend.rs` (`Tombstones`, `Versions`, índices payload/text/sparse existentes) y escritura L1 (`l1_writer`, dedup/supersession) — revisar la actualización coordinada de derivados, no introducir un lifecycle competidor.
- **Acciones:**
  1. Trazar insert/update/delete de cada backend/rama.
  2. Formalizar los casos add→delete, delete→upsert y update de ausente según el contrato propio.
  3. Comprobar la máscara de tombstones.
  4. Aplicar el delta con revisión y orden explícito por ID.
  5. Mantener la generación anterior mientras un lector la referencie; rollback de índice no puede retroceder tombstones de la fuente.
- **Dependencias:** revisión/tombstone propios y orden de deltas (prerrequisito §5); X-CHR-01 identifica la mutación; X-CHR-04 publica generación; X-CHR-05 verifica cobertura; recursos compartidos con ownership claro.
- **FMEA:** mezcla revisión nueva/embedding viejo → recuerdo incorrecto (detectar revisión de cada rama y bloquear/publicar coherentemente); cancelación entre texto y vector → parcialidad (detectar deltas incompletos, reanudar idempotentemente); purge temprano → recovery imposible (exigir checkpoint de consumidores, no solo «tarea acabó»).
- **Criterios de aceptación propuestos:** normal: update cambia todas las ramas prometidas; adversos: delete durante query/rebuild, replay duplicado y reapertura no resucitan la revisión eliminada; contadores de deltas, sin IDs sensibles.
- **Criterio negativo:** un delete nunca reaparece como hit tras compactar o reiniciar; un derivado antiguo sin revisión nunca se asume fresco (se audita/reconstruye).
- **Independencia:** sobre Tombstones/Versions existentes; alternativas: actualización síncrona de índices (menor lag, mayor coste de write), overlay (solo si el backend lo necesita), reconstrucción completa (operación existente, no hot path); no imponer la semántica upstream «update ausente = ignorar».
- **No objetivos:** no borrado criptográfico, no GC de backups remotos, no cambio de política de memoria ni copia de estados/algoritmo upstream.

### X-CHR-03 — Reporte del dominio efectivo de filtros y cobertura

- **Problema:** un scope restrictivo aplicado después de truncar ANN produce «sin memoria» falso; un filtro de seguridad no puede degradarse a opcional ni explicarse como estimación.
- **Contrato hipotético:** `explain_filter(namespace, query, filters, revision) -> Result<FilterPlanReport, PlanError>` opt-in; salida por rama `PreIndex`, `ExactScan`, `PostCandidate` o `Unsupported`, con revisión y límite aplicado. Invariante: explicar lo ejecutado, no una estimación presentada como hecho.
- **Fuente de evidencia (clone):** `rust/worker/src/execution/operators/filter.rs:715-875` (devuelve máscaras separadas); `rust/worker/src/execution/operators/knn_hnsw.rs:38-70` (máscaras como allow/disallow a HNSW); `rust/frontend/src/impls/service_based_frontend.rs:3031-3115` (`query` puede rechazar un predicado sin índice habilitado — no demuestra autorización por el mero tenant/filter).
- **Seam VantaDB:** `src/sdk/search/vector.rs` (`vector_memory_search` ya selecciona prefilter/scan/máscara) y `src/sdk/search/hybrid.rs` (fusión lexical+dense+sparse con presupuesto) — exponer razón/cobertura efectiva, no rehacer filtros, HNSW ni RRF.
- **Acciones:**
  1. Reutilizar la decisión efectiva del planner existente.
  2. Adjuntar reporte versionado opt-in en la API.
  3. Distinguir cardinalidad candidata/devuelta sin exponer payload.
  4. Limitar el reporte al scope autorizado.
  5. Si hay rebuild requerido, informar por el mecanismo de X-CHR-04/X-CHR-05 — no lanzarlo al consultar.
- **Dependencias:** planner prefilter/scan/máscara y autorización existentes (prerrequisito §5); X-CHR-02/X-CHR-04 aportan revisión; X-CHR-06 declara capacidades; cancelar la consulta cancela el reporte — no lanzar segunda query con datos potencialmente distintos.
- **FMEA:** reporte stale → falsa explicación (detectar generación del plan); filtro incompatible omitido → filtración (error tipado); conteos globales entre tenants → canal lateral (limitar al scope autorizado); adapter que traduce `$contains` a una operación no equivalente (prohibido: conserva significado).
- **Criterios de aceptación propuestos:** filtros vacíos/selectivos y deletes pendientes concuerdan con el plan; caso adverso `Unsupported` produce error explícito; paridad de resultados con reporte desactivado, sin nuevo scoring.
- **Criterio negativo:** un filtro aplicado post-truncamiento nunca se reporta como pre-index; un predicado no soportado nunca degrada a resultado vacío silencioso.
- **Independencia:** reporte efímero sin migración de datos ni nueva gramática; alternativas: logging interno (insuficiente para SDK), reporte opcional (mínimo, elegido), gramática nueva (rechazada).
- **No objetivos:** no ACL nueva, no parámetros de ranking, no gramática Chroma completa ni promesa de recall basada en el nombre «prefilter».

### X-CHR-04 — Compactación preparada, publicación y retención separadas

- **Problema:** una tarea de compactación figura completada pero los lectores observan vector nuevo con metadata vieja o pierden un archivo activo; aplicar/flush no es publicar catálogo.
- **Contrato hipotético:** `prepare_derived_revision(source_revision, cancel) -> Result<PreparedDerived, PrepareError>` y `publish(prepared, expected_current) -> Result<PublishedRevision, PublishError>`; `PreparedDerived` posee los temporales y no es clonable como permiso de publicación; errores conflict/cancel/I/O no avanzan el catálogo.
- **Fuente de evidencia (clone):** `rust/worker/src/execution/orchestration/compact.rs:710-728,919-1096` (`run_regular_compaction_workflow`, `run_register`, `run_compaction`); `rust/worker/src/execution/orchestration/register_orchestrator.rs:59-77` (`CollectionFlushInfo::try_from` enlaza offset, versión y archivos — los tokens de tarea se propagan, no se acreditó atomicidad de un workflow multicolección).
- **Seam VantaDB:** `src/sdk/api/admin.rs::Embedded::rebuild_index:19-32` (coordina vector, derivados, texto, sparse y scalar secuencialmente propagando `?`) y `src/sdk/serialization/impl_rebuild.rs:16-58,155-180` (batch de derivados y después estado/health) — el publicador real existente a extender por fases, no otro publicador. `src/backend.rs` particiones/`InternalMetadata` solo enumera almacenamiento (sin CAS, pinning ni swap): manifiesto versionado, CAS y lease de lectores son **alternativas condicionadas**, no mínimo implícito.
- **Acciones:**
  1. Inventariar locks y manifiesto vigentes (la lectura actual no prueba por sí sola carrera ni atomicidad insuficiente).
  2. Mapear cambios sobre las fases de `rebuild_index` y los publicadores de cada rama con sus estados y locks.
  3. Preparar fuera del lector; validar todas las ramas y la fuente.
  4. Publicar con comparación de revisión.
  5. Retirar la generación vieja solo tras liberar lectores, con cobertura desconocida documentada (no escondida).
- **Dependencias:** inventario de ownership/publicación y scheduler existentes (prerrequisito §5); X-CHR-01/X-CHR-02 alimentan; X-CHR-05 inspecciona el preparado; X-CHR-06 evita doble writer; scheduler actual para límites, sin nuevas colas.
- **FMEA:** crash tras flush antes del publish → huérfano (detectar manifiesto no referenciado y recolectar después); cancelación durante publish → estado indeterminado (consultar generación durable); recolectar con lector vivo → fallo (referencias/pinning y GC diferido); no guardar texto en trazas.
- **Criterios de aceptación propuestos:** favorable: lectores viejos/nuevos tienen generaciones válidas; adversos: CAS fallido, disco lleno y cancelación no publican preparado incompleto; reopen identifica huérfanos sin borrarlos ciegamente.
- **Criterio negativo:** una compactación «completada» nunca deja vector nuevo con metadata vieja ni pierde un archivo activo; revertir puntero sin validar deletes actuales está prohibido.
- **Independencia:** extiende `rebuild_index` existente; alternativas: rebuild offline (mínimo operacional), swap de generación preparado (si la concurrencia lo exige), puerto distribuido de workers/sysdb (rechazado). Compactar contexto de agente y compactar índice son procesos distintos.
- **No objetivos:** no servicio sysdb, no formato blockfile, no refactor del scheduler ni garantía de transacción entre todos los objetos remotos; migrar reconstruyendo desde fuente verificable, nunca inventando offsets.

### X-CHR-05 — Auditoría relacional y reparación solo sobre copia

- **Problema:** la colección contiene la fuente pero un checkpoint inconsistente deja recuerdos inaccesibles, y una reparación automática puede destruir evidencia de recovery; check, findings y error deben distinguirse.
- **Contrato hipotético:** `audit_generation(scope, revision) -> Result<GenerationAudit, AuditError>`, `plan_repair(audit, expected) -> Result<RepairPlan, RepairError>` y `apply_offline_copy(plan, target)` sobre destino nuevo autorizado — la salida es artefacto verificado, no promoción automática; `CannotRead` nunca es `Clean`.
- **Fuente de evidencia (clone):** `rust/cli/src/hnsw_integrity_check.rs:25-179` (`HnswIntegrityCheckArgs`, `Report`, códigos de salida — distingue findings de check fallido; `inspect_segment:493-660`/`inspect_hnsw_metadata:662+`: stub vacío sin watermark ≠ checkpoint corrupto; fast-forward no certifica integridad del log retenido); `rust/cli/src/hnsw_config_repair.rs:146-329` (`repair_copy`: repara **únicamente** `ef_construction` fuera de rango en staging/copia, rechaza destino existente, symlinks y especiales; rechazos explícitos: SQLite adelantado al checkpoint, formato ≠v1, recovery pendiente, pérdida parcial del índice; M fuera de límite exige rebuild); `rust/segment/src/local_hnsw.rs:1353-1417` (`persist` ordena sync, mapa y directorio — no afirmar misma garantía de directorio en Windows).
- **Seam VantaDB:** `src/cli_handlers/index.rs` (`cmd_rebuild_index`) y `src/sdk/search/audit.rs` (`audit_text_index`) — ampliar la relación fuente↔vector↔ID↔watermark y la separación check/repair, sin crear una CLI paralela; no se ha probado que falten auditorías ANN.
- **Acciones:**
  1. Inventariar audit/rebuild existentes.
  2. Capturar metadata sin payload; validar formatos y revisiones.
  3. Emitir plan de reparación separado del diagnóstico.
  4. Reparar sobre copia verificada con rutas confinadas y exclusión de escritores.
  5. Verificar y dejar al operador la decisión de promoción; rollback = conservar origen y retirar copia.
- **Dependencias:** X-CHR-01/X-CHR-04 aportan watermarks y publicación; auditorías/rebuild existentes, copia consistente y rutas seguras (prerrequisito §5); dominio de reparación declarado por caso, no universal; motivada en parte por issue #7946 del informe.
- **FMEA:** snapshot online incoherente → falso corrupto (detectar revisión cambiante y abortar); symlink hacia otro tenant → copia/borrado fuera de scope (resolución confinada y rechazo); mapa leído que no coincide con watermark → fast-forward destructivo (exigir replay verificable); en cada fase la cancelación deja el original intacto.
- **Criterios de aceptación propuestos:** check limpio/findings/error separados; copy repair repetido no sobrescribe; disco lleno/cancelación no toca el origen; `ef_construction` inválido se repara en copia; SQLite adelantado, formato ≠v1, recovery pendiente o pérdida parcial son rechazo explícito; M fuera de límite devuelve rebuild requerido; reportes sin documentos por defecto.
- **Criterio negativo:** `CannotRead` nunca se reporta como `Clean`; una reparación nunca muta la instancia viva ni borra sus archivos; stub vacío sin watermark no se reporta como corrupción.
- **Independencia:** amplía el audit existente (preferido sobre verificar solo conteos); reparación in-place online rechazada salvo contrato de exclusión propio futuro.
- **No objetivos:** no reparación universal, no copiar pickle upstream ni afirmar seguridad de crash sin interrupción real; formato nuevo explícito, no mutación oportunista al consultar.

### X-CHR-06 — Contrato de backend y ownership de índice vivo

- **Problema:** la misma firma puede funcionar en local y fallar en servidor, y una expulsión de caché no debe crear dos escritores del mismo índice; evicción no equivale a eliminación lógica ni cerrar FDs a eliminar el índice.
- **Contrato hipotético:** `describe_backend(handle) -> BackendContract` con persistencia, filtros soportados, sync y formato; opcional `acquire_index(index_id, expected_revision) -> Result<IndexLease, LeaseError>` — lease RAII que posee referencia a la identidad nativa; deleted/failed impiden mutación y un miss concurrente no fabrica otro writer.
- **Fuente de evidencia (clone):** `chromadb/__init__.py:201-315,375-433` (clientes/default: misma firma local/servidor con comportamiento distinto); `rust/segment/src/local_segment_manager.rs:40-105,173-258` (`LocalSegmentManager` serializa misses y conserva identidad débil); `rust/segment/src/local_hnsw.rs:777-788` (`ensure_usable` invalida handles fallidos/deleted); `rust/log/src/lib.rs:24-45` (selección de backend; rechaza sparse local).
- **Seam VantaDB:** `src/backend.rs` (`StorageBackend`, capabilities opcionales ya declaradas) y bindings oficiales — alinear transporte, errores y ownership con esas capacidades; no asumir que un runtime Rust homogéneo elimina diferencias.
- **Acciones:**
  1. Revisar capacidades por binding/backend.
  2. Propagar `Unsupported` sin fallback silencioso.
  3. Verificar identidad de handles bajo misses/reloads.
  4. Adaptar solo donde el ownership actual no exprese la garantía.
  5. Formatos previos pasan por la auditoría de X-CHR-05 antes de marcarse usables; rollback de adapter preserva archivos y no libera recursos aún referenciados.
- **Dependencias:** capabilities por backend/binding ya declaradas (prerrequisito §5); X-CHR-01 para semántica de errores; X-CHR-04/X-CHR-05 para ciclo de vida; cancelación de acquire libera la espera; el drop de un cliente no puede cerrar un recurso que otro lease utiliza.
- **FMEA:** caché expulsa pero query aún usa el handle → doble writer (detectar identidad/número de propietarios, compartir lease); backend que no soporta un predicado → resultado vacío engañoso (error `Unsupported`); transporte corta la respuesta tras commit → no retry ciego (consultar X-CHR-01).
- **Criterios de aceptación propuestos:** dos misses/reload concurrentes reutilizan identidad; adversos: eviction/delete/error nativo invalidan o conservan handles según revisión sin data race; capabilities iguales entre SDK y servidor efectivo.
- **Criterio negativo:** un miss de caché nunca fabrica un segundo writer del mismo índice; una capacidad ausente nunca se simula con fallback silencioso.
- **Independencia:** adapter sobre capabilities existentes (preferido); documentación por SDK necesaria pero insuficiente si diverge; nueva caché global rechazada mientras el inventario no demuestre necesidad.
- **No objetivos:** no portar la caché Chroma, no añadir un SDK nuevo, no prometer compatibilidad binaria Python/Rust ni certificar Cloud con evidencia del core local; lease solo en memoria, capabilities versionadas sin cambiar registros.
