---
title: "Backlog de extracción — SurrealDB"
kind: research
status: active
description: "Cuatro candidatas del informe SurrealDB (frontera de escritores en rebuild, compatibilidad por formato, diagnóstico de proyección y efectos postcommit) como filas del esquema canónico de 10 columnas"
tags: [extraccion, backlog, surrealdb]
---

# Backlog de extracción — SurrealDB

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/surrealdb.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
> **Licencia:** SurrealDB es BSL 1.1 — sólo extracción conceptual; cualquier
> reutilización material queda detenida hasta revisión legal específica.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `X-SUR-01` | 🟠 | Frontera de escritores inscrita antes del snapshot de rebuild | `src/sdk/serialization/impl_rebuild.rs` · `src/sdk/api/admin.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: inscripción del writer con epoch/generación antes de la toma del snapshot y espera verificable sobre los escritores previos a la publicación de la generación. · Por qué: un índice reconstruido no debe omitir escrituras iniciadas bajo la generación anterior que aún pueden confirmar tras comenzar el escaneo; cancelar un future no equivale a commit/cancel del almacenamiento. · Acción mínima: inventariar fences/generaciones y la publicación actual de rebuild, identificar la toma real de snapshot y especificar inscripción/cierre ligados a esa frontera antes de alterar algoritmos. · Trazabilidad: informe §2/Candidata A · evidencia `surrealdb/datastore/src/triggers.rs:1476-1538`, `surrealdb/datastore/src/triggers.rs:1572-1625`, `surrealdb/core/src/kvs/write_barrier.rs:1-60` (clon `C:\Users\Eros\surrealdb`). | X-SUR-03 · C13 | Orden snapshot/registro, resultado commit/cancel, autoridad de publicación y coordinación con GC existente; revisión legal BSL previa a cualquier material |
| `X-SUR-02` | 🟡 | Reporte read-only de compatibilidad por formato | `src/migration.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: informe de compatibilidad que separa veredicto de apertura, formatos afectados, objetos ilegibles/desconocidos y restricciones de escritura, con alcance de observación declarado. · Por qué: que una base abra no garantiza que todos sus índices, registros u operaciones puedan leerse o mantenerse; un campo desconocido no es «compatible» ni autoriza escrituras. · Acción mínima: inventariar reports/preflights actuales, declarar qué se infiere de cabeceras y qué exige recorrido, e integrar el diagnóstico read-only con `MigrationEngine` sin reparar ni rebajar versión. · Trazabilidad: informe §2/Candidata B · evidencia `surrealdb/core/src/kvs/rollback.rs:1-165`, `surrealdb/core/src/kvs/rollback.rs:165-261`. | — | Versiones/decoders y modo read-only de `MigrationEngine`; acceso autorizado; independiente de X-SUR-01 |
| `X-SUR-03` | 🟡 | Diagnóstico de generación publicada y pendiente real por proyección | `src/cli_handlers/index.rs` · `src/sdk/serialization/impl_rebuild.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: observación por proyección con generación descrita, readiness, existencia conocida/desconocida de pendientes y motivo de error; `NotInspected` nunca se presenta como cero. · Por qué: una captura confirmada que aún no participa en búsqueda exige distinguir índice no publicado, generación vieja, deltas pendientes y error de inspección antes de repetir ingesta o reconstruir; un contador de wakeups no equivale a registros. · Acción mínima: inventariar reportes/marcadores presentes, anotar qué prueba cada campo y exponer la observación read-only en auditoría/SDK/Studio. · Trazabilidad: informe §2/Candidata C · evidencia `surrealdb/datastore/src/values/index_build.rs:20-205`, `surrealdb/datastore/src/index_state.rs:1-180`, `surrealdb/core/src/kvs/index/builder.rs:4049-4123`. | X-SUR-01 · X-SUR-04 · C13 | Marcadores y reportes de rebuild/auditoría existentes; si X-SUR-01 no existe reportar «cobertura no acreditada»; no nuevo scheduler |
| `X-SUR-04` | 🟠 | Follow-ups postcommit diferenciados de la notificación | `vanta-memory/src/core/record/l1_writer.rs` · `vanta-memory/src/services/pipeline_worker.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: observación del efecto posterior al commit (pendiente/aplicado/desconocido) correlacionada con operación y revisión de fuente; un wakeup es evento operativo, no prueba de ejecución ni `DurablePosition`. · Por qué: perder un wakeup tras confirmar el commit no debe convertirlo en rollback ni permitir que una tarea observe datos provisionales; reintentar un efecto no idempotente duplica trabajo. · Acción mínima: trazar un put→enqueue→handler real, clasificar qué trabajo ya es persistido y correlacionar operación/revisión; registro durable sólo con laguna demostrada. · Trazabilidad: informe §2/Candidata D · evidencia `surrealdb/datastore/src/tx.rs:3284-3478`, `surrealdb/datastore/src/tx.rs:3479-3507`, `surrealdb/datastore/src/tx.rs:12734-12750`, `surrealdb/datastore/src/tx.rs:3017-3052`. | X-RED-01 (settlement/intento, backlog redis) · C13 | Una ruta real `persist_planned`/worker, identidad/revisión y obligación persistida o reconstruible; C13 propuesto, no outbox/WAL nuevo sin necesidad |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/surrealdb.md` (Candidatas A–D + precisiones posteriores §2). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `9577ac5` (`C:\Users\Eros\surrealdb`, raíz `surrealdb/` salvo prefijo explícito); las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD. SurrealDB es BSL 1.1: sólo extracción conceptual pendiente de revisión legal.

### X-SUR-01 — Frontera de escritores inscrita antes del snapshot de rebuild

- **Problema:** un índice reconstruido no debe omitir escrituras iniciadas bajo una generación anterior que aún pueden confirmar tras comenzar el escaneo; el snapshot sólo es válido si su frontera quedó inscrita antes de tomarlo.
- **Contrato hipotético:** `WriteEpochGuard {generation, operation_id}` y `RebuildBoundary {source_generation, prior_writers_settled}`; entrada: scope autorizado, generación objetivo y operación; errores tipados: epoch superado, scope no autorizado, writer no resuelto, cierre del backend y almacenamiento. Invariante: no liberar el guard al cancelar un wrapper si el commit aún puede aplicar.
- **Fuente de evidencia (clone):** `surrealdb/datastore/src/triggers.rs:1476-1538` (open/close por epoch y `begin_commit`), `surrealdb/datastore/src/triggers.rs:1572-1625` (avance coordinado y consulta de finalización), `surrealdb/core/src/kvs/write_barrier.rs:1-60` (contrato documental local/distribuido — implementación parcial S3/S4 separada del contrato).
- **Seam VantaDB:** `src/sdk/serialization/impl_rebuild.rs` (`Embedded::rebuild_index`/`rebuild_derived_indexes_with_report`/`rebuild_text_index_with_report`), `src/sdk/api/admin.rs`; particiones derivadas e `InternalMetadata` del backend; auditoría del propietario.
- **Acciones:**
  1. Identificar el punto de registro de escritores en la ruta de commit actual.
  2. Añadir el guard ligado a operación/scope/generación.
  3. Interrogar/cerrar escritores previos antes del snapshot de rebuild.
  4. Publicar la generación sólo si la frontera está resuelta; rollback: diagnóstico aditivo desactivable, sin cambiar el layout.
- **Dependencias:** orden snapshot/registro, resultado commit/cancel, autoridad de publicación y coordinación con GC; revisión legal BSL previa.
- **FMEA:** inscripción tardía (escritura ya empezada queda fuera del snapshot); cierre temprano (guard liberado antes del commit real); espera sin fin ante writer bloqueado; bloqueo global innecesario si la frontera es demasiado gruesa.
- **Criterios de aceptación propuestos:** la publicación cubre cada escritura confirmada de la frontera elegida o mantiene la generación anterior; espera/cancelación no anuncian progreso durable que el backend no confirmó; un error deja estado de salud interpretable. Adversa: abort, timeout de writer, epoch obsoleto.
- **Criterio negativo:** la inscripción nunca es posterior al snapshot que determina la generación; cancelar un future nunca equivale a commit/cancel del almacenamiento.
- **Independencia:** aplicable sin la inspección de downgrade de X-SUR-02; diagnóstico aditivo primero — la barrera se observa fuera de locks de índice y no cambia por sí sola C13 a durable/ready.
- **No objetivos:** claves heartbeat, leases multinodo, Raft ni copiar S3/S4; no nueva autoridad de transacciones.

### X-SUR-02 — Reporte read-only de compatibilidad por formato

- **Problema:** que una base abra no garantiza que todos sus índices, registros u operaciones puedan leerse/mantenerse; un campo desconocido no es «compatible» ni autoriza escrituras.
- **Contrato hipotético:** `CompatibilityReport {target, open_mode, format_findings, unknown_count, unreadable_count, observation_scope}`; entrada: scope autorizado, versiones objetivo y modo de inspección; errores: objetivo no soportado, acceso denegado, decoder/IO, observación invalidada por cambio de fuente.
- **Fuente de evidencia (clone):** `surrealdb/core/src/kvs/rollback.rs:1-165` (fuentes y tipos de hallazgo del escaneo de compatibilidad), `surrealdb/core/src/kvs/rollback.rs:165-261` (veredicto y recorrido read-only).
- **Seam VantaDB:** `src/migration.rs` — `FormatKind` (File/VectorIndex/WAL/Records/Schema), `MigrationPlan`, `MigrationEngine`, `RecordsBackfillReport` ya distinguen versiones y dry-run; el orden records precede al schema bump.
- **Acciones:**
  1. Exponer un reporte read-only sobre los formatos decodificables existentes.
  2. Clasificar hallazgos por archivo/formato con alcance de observación.
  3. Declarar `unknown`/`unreadable` separados de `compatible`.
  4. Sin mutación, sin reparación, sin downgrade implícito. Rollback: retirar el DTO, no toca datos.
- **Dependencias:** decoders versionados, formatos publicados, acceso autorizado y frontera de observación consistente; independiente de X-SUR-01.
- **FMEA:** falso seguro (sólo apertura validada, índice ilegible) → detallar alcance y operaciones restringidas; desconocido tratado como vacío → omite material incompatible (unknown visible, no autoriza escrituras); informe mezclado (lecturas de generaciones distintas) → declarar snapshot/alcance o no atómico.
- **Criterios de aceptación propuestos:** el informe no modifica bytes, no repara ni promete downgrade; distingue «abre», «sólo lectura» y «no acreditado»; los reportes no filtran nombres/payloads de namespaces no inspeccionables. Adversa: un fallo de lectura nunca se vuelve «compatible» aunque el target abra.
- **Criterio negativo:** unknown/error permanece visible y nunca autoriza escritura; «abre» nunca se presenta como compatibilidad total.
- **Independencia:** útil sin rebuild online ni X-SUR-01; puede empezar como inspección efímera.
- **No objetivos:** nuevo formato de disco, migrador automático ni copiar el ledger; reporte efímero por defecto.

### X-SUR-03 — Diagnóstico de generación publicada y pendiente real por proyección

- **Problema:** una captura confirmada que aún no participa en búsqueda exige distinguir índice no publicado, generación vieja, deltas pendientes y error de inspección antes de repetir ingesta o reconstruir.
- **Contrato hipotético:** `inspect_projection(scope, expected_generation) -> Result<ProjectionObservation, ProjectionInspectError>` con generación observada, alcance de fuente, readiness, existencia conocida/desconocida de pendientes y motivo de error; se adjunta a `projections`/`visibility_generation` de C13, no reemplaza `MutationReceipt`. Invariantes: readiness con evidencia de la generación descrita; contador de wakeups ≠ registros; `NotInspected` no es cero; primario durable no retrocede a «no guardado» por índice pendiente.
- **Fuente de evidencia (clone):** `surrealdb/datastore/src/values/index_build.rs:20-205` (fases/cursor del build), `surrealdb/datastore/src/index_state.rs:1-180` (reporteo/backlog), `surrealdb/core/src/kvs/index/builder.rs:4049-4123` (prueba de generación y rescan). No copiar `!bs`/tickets.
- **Seam VantaDB:** `src/cli_handlers/index.rs` (auditoría textual superficial/profunda), `src/sdk/serialization/impl_rebuild.rs` (reportes de rebuild, marcadores `InternalMetadata`).
- **Acciones:**
  1. Mapear fases/cursor de la proyección objetivo a los marcadores existentes.
  2. Exponer generación publicada y pendiente real (o `NotInspected` si no se puede comprobar).
  3. Sellar alcance/generación del diagnóstico.
  4. Adjuntar el resultado al DTO C13 sin prometer durabilidad nueva.
  5. Reconciliar cambio de generación: no heredar porcentajes del build anterior.
- **Dependencias:** marcadores y reportes de rebuild existentes; X-SUR-01 aporta prueba de frontera pero esta ficha puede reportar «cobertura no acreditada»; X-SUR-02 independiente; no nuevo scheduler.
- **FMEA:** cola de triggers vacía → falso ready (contrastar pendiente real; desconocido si no inspeccionado); fase/cursor de generaciones distintas → falso progreso (sellar alcance/generación); reparación lanzada desde diagnóstico → pérdida (separar operaciones y autorización).
- **Criterios de aceptación propuestos:** con proyección retrasada el receipt conserva primario confirmado; un error de lectura no muestra backlog cero; takeover/cambio de generación no hereda porcentajes del anterior. Sin contenido en labels.
- **Criterio negativo:** `NotInspected` nunca se presenta como cero; un contador de wakeups nunca equivale a registros pendientes.
- **Independencia:** una proyección y auditoría existente; observación read-only sin nuevo layout.
- **No objetivos:** no compactor, nuevo scheduler, promesas de latencia, ranking ni inspección forense global. Abierto: qué datos permiten los backends sin escaneo adicional.

### X-SUR-04 — Follow-ups postcommit diferenciados de la notificación

- **Problema:** fallar o perder un wakeup tras confirmar commit no debe convertirlo en rollback ni permitir que una tarea observe datos provisionales.
- **Contrato hipotético:** `observe_followup(operation_id, source_revision, effect_kind) -> Result<FollowupObservation, FollowupError>` con identidad del trabajo existente, fuente requerida, observación pendiente/aplicado/desconocido y referencia al C13 del primario. Errores: revisión superada, tarea desconocida, storage, acceso denegado. Invariante: un wakeup es evento operativo, no `DurablePosition` ni prueba de ejecución.
- **Fuente de evidencia (clone):** `surrealdb/datastore/src/tx.rs:3284-3478` (`commit_body`: deltas/changefeed antes de commit), `surrealdb/datastore/src/tx.rs:3479-3507` (acciones y notificaciones postcommit), `surrealdb/datastore/src/tx.rs:12734-12750` (`run_commit_actions` fuera del lock), `surrealdb/datastore/src/tx.rs:3017-3052` (`cancel_body` descarta efectos provisionales).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` (`plan_write`/`persist_planned`/`apply_dedup_batch` separan planificación, put y efectos — no acredita transacción única), `vanta-memory/src/services/pipeline_worker.rs`.
- **Acciones:**
  1. Mapear la frontera commit→efecto en la ruta real de persistencia.
  2. Distinguir efecto provisional descartado, efecto aplicado y efecto omitido tras crash.
  3. Preservar el receipt del primario aunque el follow-up falle.
  4. Comprobar revisión/tombstone antes de aplicar el efecto diferido.
  5. Registrar obligaciones sólo si hay laguna demostrada; nunca outbox/WAL nuevo por analogía.
- **Dependencias:** ruta real `persist_planned`/worker; identidad/revisión; obligación persistida o reconstruible; X-RED-01 (settlement/intento, backlog redis); C13 propuesto; no outbox/WAL nuevo sin laguna demostrada.
- **FMEA:** wakeup antes de commit → lectura provisional (notificar después); crash commit→wakeup → trabajo omitido (verificar job persistido/reconciliación); callback falla tras commit → reingesta duplicada (preservar receipt primario); worker antiguo sobre revisión nueva → derivado stale (comprobar revisión).
- **Criterios de aceptación propuestos:** no se activa derivado de escritura cancelada; commit confirmado con fallo de activación sigue confirmado y muestra followup pendiente/desconocido; misma tarea repetida no duplica efecto de revisión. Métricas fase/resultado sin texto ni IDs sensibles.
- **Criterio negativo:** perder un wakeup nunca convierte un commit confirmado en rollback; reintentar nunca duplica un efecto no idempotente.
- **Independencia:** mapear a la ruta existente; registro durable sólo con laguna demostrada; relación con redis: settlement/intento allí, orden commit→activación aquí.
- **No objetivos:** exactly-once LLM, atomicidad global, nuevo changefeed, runtime SurrealDB, copiar handlers BSL ni garantizar fsync con `notify_one`. Abierto: qué obligaciones requieren supervivencia a crash.
