---
title: "Backlog de extracción — Redis"
kind: research
status: active
description: "Cinco unidades candidatas del informe Redis (entrega/efecto, barrera durable, retención, pendientes y recovery) como filas del esquema canónico de 10 columnas"
tags: [extraccion, backlog, redis]
---

# Backlog de extracción — Redis

> **Origen:** informe `docs/dev/extraccion/redis/REDIS.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `X-RED-01` | 🟠 | Resolución idempotente del efecto por intento y revisión | `vanta-memory/src/services/pipeline_worker.rs` · `vanta-memory/src/utils/local_backend.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: vínculo explícito entre tarea, intento, efecto aplicado y revisión durable de fuente; la resolución distingue confirmado, no aplicado e incierto, y un owner viejo no cierra el claim nuevo. · Por qué: un worker que muere tras aplicar el efecto y antes de cerrar el claim no debe provocar doble aplicación ni doble gasto LLM; el ACK de la cola no certifica el efecto ni fsync. · Acción mínima: trazar handler→put→complete en `run_task`/`reclaim_stale`, acordar identidad/revisión del efecto y ampliar el resultado del handler sin alterar retries. · Trazabilidad: informe §2.1 · evidencia `src/t_stream.c:4248-4310`, `src/t_stream.c:5153-5345` (clon `C:\Users\Eros\redis`). | X-RED-04 · C13 | Identidad/revisión del efecto y propiedad del intento; `run_task` ejecuta antes de completar; C13 sólo para afirmar efecto durable |
| `X-RED-02` | 🟠 | Barrera durable explícita separada de commit y visibilidad | `spec` (C13 `MutationReceipt`) | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: dominio de confirmación que distingue `primary_commit`, `primary_durability`, `projections` y `visibility_generation`; timeout devuelve conocimiento parcial, nunca rollback inferido. · Por qué: una respuesta «guardado» que mezcla efecto y fsync induce retries duplicados o falsa seguridad ante crash; primario durable puede preceder a readiness. · Acción mínima: localizar la confirmación por backend, mapear posición/dominio y extender el contrato C13 canónico sin renombrar receipts existentes ni simular fsync en backends no durables. · Trazabilidad: informe §2.2 · evidencia `src/aof.c:1500-1650`, `src/replication.c:4916-4989`. | X-RED-01 · X-RED-05 · C13 | Frontera durable demostrable por backend; C13 es diseño propuesto, no checkpoint global |
| `X-RED-03` | 🟡 | Disposición de retención: exclusión lógica separada de GC y presión de memoria | `vanta-memory/src/core/record/lifecycle.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: observación de retención por registro/revisión con razones `Live`/`Expired`/`Superseded`/`Deleted` y estado de cleanup por proyección; la observación no concede permiso para borrar. · Por qué: un recuerdo caducado debe salir del recall aunque sus páginas/vector se recojan después; liberar caché por presión no autoriza olvidar la fuente persistida. · Acción mínima: trazar filtros TTL en get/search/recall y ampliar el diagnóstico del lifecycle existente; el GC revalida revisión vigente antes de cada borrado. · Trazabilidad: informe §2.3 · evidencia `src/db.c:3137-3185`, `src/expire.c:303-337`, `src/evict.c:543-563`. | X-TAM-06 (retención/cierre de referencias, backlog tencentdb-agent-memory) | TTL/lifecycle, autorización y gobernador existentes; C13 sólo si se reporta durabilidad del tombstone |
| `X-RED-04` | 🟡 | Diagnóstico de pendientes sin payload y settlement con owner vigente | `vanta-memory/src/services/pipeline_worker.rs` · `vanta-memory/src/utils/local_backend.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: inspección de tarea pendiente que separa fuente ausente/expirada/reemplazada de error de lectura y owner vigente; settlement sólo con expectativa de revisión/owner y autorización del scheduler. · Por qué: reintentar indiscriminadamente un efecto no idempotente duplica consolidaciones o gasto; un pending no implica payload recuperable y un claim atómico no excluye el efecto externo. · Acción mínima: resolver la referencia de fuente en la cola existente, clasificar causas y habilitar settlement sólo para causas inequívocas conservando dead-letter. · Trazabilidad: informe §2.4 · evidencia `src/t_stream.c:5225-5305`, `src/t_stream.c:4248-4310`, issue redis/redis#13956. | X-RED-01 · C13 | Fuente referenciable y settlement owner-aware de la cola; enlaza X-RED-01 si el efecto sigue incierto; diagnóstico puede existir sin mutar |
| `X-RED-05` | 🟡 | Preflight de recuperación con pérdida explícita | `src/wal.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: assessment read-only de recuperación que reporta último tramo válido, categoría de daño, bytes no acreditados y acciones permitidas, antes de que el host interprete «arrancó» como «recuperó todo». · Por qué: truncar sobre daño interior produce pérdida no acotada; la reparación es otra operación autorizada con precondición de identidad del archivo, no una opción heredada por comodidad. · Acción mínima: localizar decoder/recovery real y ampliar el reporte existente con frontera válida y decisión explícita; la inspección no altera bytes. · Trazabilidad: informe §2.5 · evidencia `src/aof.c:2094-2129`, issue redis/redis#5177. | X-RED-02 · X-WEA-06 (diagnóstico recovery, backlog weaviate) | Decoder y auditoría de recuperación existentes; no reparación automática ni nuevo WAL |

## Fichas completas

> Expansión de la tabla anterior con el contenido del informe fuente. Todos los tipos, firmas Rust y errores son **interfaces hipotéticas** — no describen APIs existentes de VantaDB ni autorizan implementación; ninguna requiere Redis en runtime. Las rutas de evidencia corresponden al pin upstream `5b69ba2ce91fea8e9fc70ebac8ce341b37e75533` (clon `C:\Users\Eros\redis`); las de VantaDB, a la base `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`. `—` marca campos no documentados por el informe.

### X-RED-01 — Resolución idempotente del efecto por intento y revisión

- **Problema:** un worker que muere tras aplicar el efecto y antes de cerrar el claim no debe provocar doble aplicación ni doble gasto LLM; el ACK de la cola no certifica el efecto ni fsync.
- **Contrato hipotético (tipos propuestos):** `resolve_delivery(task_id, attempt_epoch, effect_ref, source_revision) -> Result<EffectResolution, DeliveryError>`; la resolución distingue confirmado, no aplicado e incierto; errores incluyen stale owner, revisión superada, no autorizado y almacenamiento. El receipt no contiene texto fuente.
- **Fuente de evidencia:** `src/t_stream.c:4248-4310` (`xackCommand`: valida todos los IDs antes de mutar; desenlaza PEL de grupo y consumidor sin borrar la entrada — ID ausente aporta cero, no prueba éxito de efecto) · `src/t_stream.c:5153-5345` (`xautoclaimCommand`: comprueba que la entrada existe antes de entregar el pendiente; claim atómico no es exclusión del efecto externo).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs::run_task/reclaim_stale` (claim, reintento y complete tras handler ya existen — el delta es el vínculo intento/efecto/revisión) y `vanta-memory/src/utils/local_backend.rs::claim_task/complete_task` (estado de proceso bajo mutex — `snapshot` no acredita durabilidad por el nombre). Storage SDK para el resultado.
- **Acciones:**
  1. Trazar handler→put→complete en `run_task`/`reclaim_stale`.
  2. Acordar identidad/revisión del efecto (tenant/sesión/tarea).
  3. Ampliar el resultado del handler sin alterar retries.
  4. Persistir resolución solo si debe sobrevivir al proceso; presentar estados a auditoría/Studio.
  5. Migración: campos opcionales o tabla auxiliar versionada del store existente; tareas antiguas quedan sin resolución acreditada — no «confirmadas».
- **Dependencias:** identidad/revisión del efecto y propiedad del intento; `run_task` ejecuta antes de completar; C13 solo para afirmar efecto durable; autorización y dedup propios. No depende de modificar TTL ni de fencing multinodo.
- **FMEA:** crash después del efecto → replay duplica → detectar mismo `effect_ref`/revisión → resolver antes de ejecutar; cierre con owner viejo → pierde pending nuevo → comparar epoch/owner en settlement → rechazar stale. Se valida por interleavings, no por similitud con un issue Redis.
- **Criterios de aceptación propuestos:** dos intentos del mismo efecto dejan una resolución coherente; revisión nueva no se suprime; `unknown` sigue visible tras timeout; métricas por resultado, intento y duración — sin payload ni session IDs en labels.
- **Criterio negativo:** un timeout no se convierte en ausencia ni en rollback confirmado; un owner viejo no cierra el claim nuevo.
- **Independencia:** un tipo de tarea y su retry como unidad mínima; desactivar la emisión conserva lectura de receipts y no borra trabajo pendiente.
- **No objetivos:** exactly-once de red, nuevo broker ni rediseño de almacenamiento; no copiar la estructura radix de la PEL. Abierto: retención del receipt.

### X-RED-02 — Barrera durable explícita separada de commit y visibilidad

- **Problema:** una respuesta «guardado» que mezcla efecto y fsync induce retries duplicados o falsa seguridad ante crash; primario durable puede preceder a readiness. Caso agente: capturar un turno y comunicar al host si sobrevivirá al cierre antes de que termine su extracción o indexación.
- **Contrato hipotético (tipos propuestos):** `await_primary_barrier(operation_id, backend_domain) -> Result<MutationReceipt, BarrierError>` consumiendo el tipo C13: `MutationReceipt { operation_id, acceptance, primary_commit, primary_durability: Option<DurablePosition>, projections, visibility_generation, outcome }`. `accepted`/`applied`/`durable` no forman un orden total ni estados excluyentes; no exigir fsync individual de derivados reconstruibles. Errores: backend sin barrera, operación desconocida, acceso denegado, IO — timeout devuelve conocimiento parcial, nunca rollback inferido.
- **Fuente de evidencia:** `src/aof.c:1500-1650` (`flushAppendOnlyFile`: separa write/fsync con estados de error y retraso; `everysec` agenda trabajo en background) · `src/replication.c:4916-4989` (`waitaofCommand`: comprueba offset fsynced local y ACK AOF de réplicas; rechaza exigir local si AOF está desactivado; timeout devuelve observación incompleta).
- **Seam VantaDB:** spec C13 `MutationReceipt` (contrato propuesto transversal, no implementado); store/WAL e índices reconstruibles existentes. Extender el C13 canónico — no añadir un receipt Redis ni renombrar el `ResourceReceipt` de TiDB.
- **Acciones:**
  1. Localizar la confirmación por backend; mapear posición y dominio durable.
  2. Separar la espera de la emisión del receipt; propagar por bindings existentes.
  3. Documentar operación antigua sin evidencia como desconocida.
  4. Posición válida tras reopen y política WAL del backend; campo opcional versionado; desactivar la espera no borra receipts ni modifica WAL.
- **Dependencias:** frontera durable demostrable por backend; C13 es diseño propuesto, no checkpoint global. El commit pertenece al store, no al future de espera — cancelar ese future no cancela el commit.
- **FMEA:** ACK confundido con fsync → pérdida tras crash → detectar dominio/posición ausentes → mantener desconocido; primario durable e índice retrasado → retry innecesario → mostrar proyección pendiente, no reinsertar; no mantener locks de índice esperando IO ni convertir offset de otra instancia en prueba local.
- **Criterios de aceptación propuestos:** interrumpir la espera después del commit y consultar la misma operación; observar primario durable con proyección pendiente sin degradarlo a fallo; backend no durable no anuncia garantía física. Métricas por dominio/resultado sin texto, rutas sensibles ni IDs en labels.
- **Criterio negativo:** un timeout devuelve conocimiento parcial — nunca rollback inferido ni garantía fabricada sobre backend no durable.
- **Independencia:** no depende de X-RED-01 salvo que se afirme efecto durable de una tarea.
- **No objetivos:** replicación, WAIT remoto ni AOF Redis; rechazar simulación de fsync para backend InMemory. Abierto: qué backends acreditan barrera y cuánto receipt conservar.

### X-RED-03 — Disposición de retención: exclusión lógica separada de GC y presión de memoria

- **Problema:** un recuerdo caducado debe salir del recall aunque sus páginas/vector se recojan después; liberar caché por presión no autoriza olvidar la fuente persistida.
- **Contrato hipotético (tipos propuestos):** `retention_status(record_ref, source_revision) -> Result<RetentionObservation, RetentionError>` con razones `Live | Expired | Superseded | Deleted`, estado de cleanup por proyección y alcance observado. Ausencia, error de lectura y no autorizado son distintos; la observación no concede permiso para borrar.
- **Fuente de evidencia:** `src/db.c:3137-3185` (`expireIfNeeded`: distingue validez/expiración al leer; réplicas e importación no borran con las mismas reglas que el primario) · `src/expire.c:303-337` (`activeExpireCycle`: limpieza incremental con esfuerzo y presupuesto) · `src/evict.c:543-563` (`getMaxmemoryState`/`noeviction`: presión de memoria separada de TTL).
- **Seam VantaDB:** `vanta-memory/src/core/record/lifecycle.rs` (TTL, lifecycle y gobernador ya existen — ampliar el diagnóstico para distinguir exclusión lógica, cleanup derivado y espacio recuperado; `memory_governor.rs` existe — no usar eviction sobre el canon).
- **Acciones:**
  1. Trazar filtros TTL actuales en get/search/recall.
  2. Definir qué reloj gobierna la validez persistida; conectar lifecycle y auditoría.
  3. Solo después, ampliar el GC existente con revalidación de revisión vigente antes de cada borrado.
  4. Migración aditiva; rollback del diagnóstico no reactiva objetos caducados ni elimina sus evidencias.
- **Dependencias:** política de retención, TTL y autorización existentes; C13 solo si se reporta durabilidad del tombstone. Tombstone y revisión pertenecen al store — una tarea GC vieja no elimina una revisión nueva; cancelar limpieza conserva tombstone y cursor reanudable.
- **FMEA:** reloj retrocede → reaparece recuerdo → validar política al leer y registrar frontera temporal; GC lee revisión vieja → borra update → comparar revisión al aplicar; liberar caché interpretado como borrado → pérdida semántica → separar eventos y contadores por recurso.
- **Criterios de aceptación propuestos:** expirado no llega al contexto aunque GC esté pausado; update concurrente no se borra por tarea anterior; fallo de storage es error, no `Deleted`; reporte sin payload por defecto.
- **Criterio negativo:** la observación nunca concede permiso para borrar; un fallo de lectura no se reporta como `Deleted`.
- **Independencia:** unidad mínima = diagnóstico de disposición por registro/revisión; relacionada con X-TAM-06 (retención/cierre de referencias, backlog tencentdb-agent-memory).
- **No objetivos:** purga forense de backups, relojes distribuidos, TTL nuevo por cada tipo de memoria ni política automática decidida por LLM; no crear otro recolector.

### X-RED-04 — Diagnóstico de pendientes sin payload y settlement con owner vigente

- **Problema:** reintentar indiscriminadamente un efecto no idempotente duplica consolidaciones o gasto; un pending no implica payload recuperable y un claim atómico no excluye el efecto externo. Caso: explicar por qué una consolidación sigue pendiente tras borrar su fuente; impedir loops que pagan de nuevo al proveedor sin datos válidos.
- **Contrato hipotético (tipos propuestos):** `inspect_pending(task_id, attempt_epoch) -> Result<PendingObservation, QueueError>` con fuente/revisión presente, owner vigente, causa conocida o desconocida; `settle_orphan` exige expectativa de revisión/owner y autorización del scheduler — no del modelo.
- **Fuente de evidencia:** `src/t_stream.c:5225-5305` (XAUTOCLAIM: si la entrada ya no existe, limpia el NACK y devuelve IDs borrados — pendiente no implica payload recuperable) · `src/t_stream.c:4248-4310` (XACK retira NACK, no payload) · issue redis/redis#13956 (claim atómico ≠ exclusión del efecto: respuesta del mantenedor — revisar `min-idle-time`, sin carrera interna acreditada).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` y `vanta-memory/src/utils/local_backend.rs` — worker y cola ya reclaman/reintentan; a diferencia de X-RED-01, esta unidad diagnostica y clasifica huérfanos antes del handler, sin resolver commit ni introducir dedup de negocio.
- **Acciones:**
  1. Resolver la referencia de fuente en la cola existente.
  2. Separar `NotFound`/`Expired`/`Superseded` de error de IO; producir diagnóstico.
  3. Habilitar settlement solo para causas inequívocas, conservando dead-letter cuando proceda.
  4. Revalidar owner bajo frontera de mutación de cola — observación previa no es CAS; cancelar inspección no completa ni reencola.
- **Dependencias:** fuente referenciable y settlement owner-aware de la cola; enlaza X-RED-01 si el efecto sigue incierto; diagnóstico puede existir sin mutar. Metadata opcional compatible; dejar de emitir diagnóstico conserva pendientes.
- **FMEA:** lectura fallida parece fuente borrada → trabajo perdido → conservar el error; owner cambia tras inspección → cierre ajeno → comparación atómica; eliminar fuente con efecto en curso → resultado tardío → validar revisión.
- **Criterios de aceptación propuestos:** distinguir huérfano real, error transitorio y trabajo reclamado; owner antiguo no completa nuevo intento; un diagnóstico no cambia la cola. Observabilidad por causa sin contenido ni nombres sensibles.
- **Criterio negativo:** si el efecto sigue activo o es incierto, se enlaza X-RED-01 — no se elimina evidencia ni se completa el pending como si estuviera resuelto.
- **Independencia:** el diagnóstico read-only puede existir sin settlement ni mutación; settlement es la parte condicionada.
- **No objetivos:** reentrega exactamente una vez ni borrar consumidores de terceros; no copiar PEL/radix ni su ID temporal. Abierto: qué razones admiten settlement automático.

### X-RED-05 — Preflight de recuperación con pérdida explícita

- **Problema:** truncar sobre daño interior produce pérdida no acotada; el operador debe conocer qué captura podría perderse al abrir un WAL dañado, antes de que el host interprete «arrancó» como «recuperó todo». La reparación es otra operación autorizada con precondición de identidad del archivo — no una opción heredada por comodidad.
- **Contrato hipotético (tipos propuestos):** `inspect_recovery(scope) -> Result<RecoveryAssessment, RecoveryError>` con último tramo válido, categoría de daño, bytes no acreditados y acciones permitidas. Read-only; la reparación es otra operación autorizada con precondición de identidad del archivo.
- **Fuente de evidencia:** `src/aof.c:2094-2129` (recovery AOF: distingue EOF inesperado y formato inválido; con opciones explícitas trunca hasta `valid_up_to`, registra truncamiento y retorna categoría específica; sin permiso/éxito de esa reparación falla la carga) · issue redis/redis#5177 (cola con ceros, `aof-load-truncated` no resuelve formato inválido; debate sobre reparación automática — no acredita defecto actual ni causa filesystem universal).
- **Seam VantaDB:** `src/wal.rs` (auditoría/WAL/migraciones existentes; ampliar el reporte de recuperación si falta la frontera válida y la decisión explícita — no otro formato de log).
- **Acciones:**
  1. Localizar decoder/recovery real; conservar original y versión.
  2. Delimitar el daño sin adivinar operaciones perdidas; enlazar auditoría y C13.
  3. Proponer acción solo si la política del backend la permite; la inspección no altera bytes ni migra.
  4. Una inspección no repara un WAL vivo; archivo cambiado invalida el assessment. Cancelar antes de reparar no escribe; tras comenzar una reparación no se infiere rollback por timeout.
- **Dependencias:** decoder y auditoría de recuperación existentes; no reparación automática ni nuevo WAL. Relacionada con X-WEA-06 (diagnóstico recovery, backlog weaviate).
- **FMEA:** truncate sobre daño interior → pérdida no acotada → rechazar automaticidad; bytes inválidos tomados como vacíos → falsa integridad → categoría `Unknown`; archivo reemplazado tras inspección → daño ajeno → verificar identidad/revisión.
- **Criterios de aceptación propuestos:** la inspección no altera bytes; la pérdida potencial permanece visible aunque la apertura posterior tenga éxito; no se inventan IDs de operaciones a partir de bytes ilegibles. Informe sin payload.
- **Criterio negativo:** una inspección nunca repara; un archivo distinto al inspeccionado invalida el assessment — no se aplica la decisión a bytes ajenos.
- **Independencia:** assessment read-only independiente; la reparación autorizada es una operación separada fuera de esta candidata.
- **No objetivos:** reparar ahora, garantías de power-loss por simulación ni reemplazar la recuperación del backend; abierto: alcance de forensic backup.
