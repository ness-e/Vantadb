---
title: "Redis: entrega, persistencia, retención y contratos para memoria de agentes"
kind: research
status: proposed
description: "Estudio de Streams, PEL, AOF, expiración y límites de recursos; propuestas Rust sin dependencia Redis ni trasplante de código"
tags: [research, memory, engineering]
---

# Redis: entrega, persistencia, retención y contratos para memoria de agentes

## Índice

- [0. Identidad, arquitectura y flujos](#0-identidad-arquitectura-y-flujos)
- [1. Recomendaciones](#1-recomendaciones)
- [2. Unidades candidatas Rust](#2-unidades-candidatas-rust)
- [3. Issues y seguridad](#3-issues-y-seguridad)
- [4. Contratos previos en VantaDB](#4-contratos-previos-en-vantadb)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Advertencias metodológicas](#6-advertencias-metodológicas)
- [7. Benchmarks](#7-benchmarks)
- [8. Matriz de capacidades](#8-matriz-de-capacidades)
- [9. Negativos sustentados y falsos positivos](#9-negativos-sustentados-y-falsos-positivos)
- [10. UI y Studio](#10-ui-y-studio)
- [11. Evidencias, cobertura y handoff](#11-evidencias-cobertura-y-handoff)

## 0. Identidad, arquitectura y flujos

### 0.1 Snapshot y alcance de la revisión

**Dictamen:** aprovechar la separación entre entrega, efecto, retención y barrera
durable; no convertir la memoria persistente de VantaDB en una caché evictable.
Esta ampliación sustituye la ficha corta: mantiene sus tres mecanismos válidos,
añade trazas, unidades independientes, issues efectivamente leídos y una lectura
del runner que delimita exactamente qué representa su latencia.

| Campo | Evidencia congelada / límite |
|---|---|
| Proyecto | `redis/redis`, clon `C:\Users\Eros\redis` |
| SHA releído, sin actualizar clon | `5b69ba2ce91fea8e9fc70ebac8ce341b37e75533` |
| Fecha del inventario y consulta pública de esta ampliación | 2026-11-07; estados de issues observados en esa consulta, no fijados por el SHA |
| Inventario original | 77 268 estrellas, licencia API `NOASSERTION`; se conserva [captura](github-candidates.json) |
| Preparación | [clone-manifest.json](clone-manifest.json): shallow, sin tags/submódulos/LFS completos; no se modifica |
| Licencia material leída | [LICENSE.txt:1-8][RL]: Redis 8+ ofrece RSALv2, SSPLv1 o AGPLv3; no asumir BSD por versiones antiguas |
| Baseline Vanta revalidado | `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` en `develop`; evidencia estática, no certificación ejecutada |
| Runtime | Ninguno: sin scripts, tests, builds, apps, servicios, instalaciones, benchmarks ni operaciones git de escritura |

Las opciones de licencia no se extienden automáticamente a cada dependencia:
`deps/` incluye hiredis, jemalloc, Lua, HDR, xxhash y otras piezas con sus propios
avisos; no se auditó su árbol legal completo. Las fichas son contratos conceptuales
para aprobación, **no autorización de copiar, traducir o enlazar código**.
Protocolo vinculante: [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md),
[METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md).

### 0.2 Mapa, stack y responsabilidades

| Superficie | Ruta inspeccionada y rol | Frontera relevante |
|---|---|---|
| Servidor y red | `src/server.c`, `networking.c`, configuración raíz; mapa, no auditoría completa | Proceso servidor C, event loop y RESP; no motor Rust embebido |
| Streams | `src/t_stream.c`, `src/stream.h`; [append][RA], [ACK][RACK], [claim][RCLAIM] | Datos del stream separados de estado de entrega del grupo |
| Expiración | [src/expire.c:303-337][REXP], [src/db.c:3137-3185][RDB] | Tiempo lógico, borrado oportunista y trabajo activo con presupuesto |
| Recursos | [src/evict.c:543-563][REVICT] | `noeviction` puede rechazar; TTL no es política de presión de memoria |
| Persistencia | [src/aof.c:1500-1650][RAOF], [recovery:2094-2129][RREC] | Escribir al kernel y confirmar fsync son hechos distintos |
| Réplicas | [src/replication.c:4916-4989][RWAIT] | Offset procesado frente a offset fsynced; no mismo ACK |
| Ejecución diferida | [src/multi.c:182-221][RACL] | ACL revalidada al ejecutar comandos encolados |
| Medición | [src/redis-benchmark.c][RBENCH] | Cliente C con hiredis/HDR; carga sintética contra servidor |
| Módulos / tooling | `modules/`, `deps/`, tests Tcl; inventario acotado | No se auditó Search, JSON, ANN ni todos los módulos |

El mapa no demuestra que todo Redis se ejecute siempre en un único hilo: no se
deduce la política de IO/background desde el dispatcher. Para esta investigación
importa la atomicidad de cada comando sobre su estado, no una falsa transacción
global con efectos externos. El cliente puede ser Python, Rust o cualquier RESP;
el consumidor implementa el dominio de agente por encima de las primitivas.

### 0.3 Modelos de datos: identidad no equivale a semántica

Un stream contiene entradas ordenadas con ID y pares campo/valor. Un consumer
group mantiene su cursor de entrega y una PEL; un consumidor mantiene referencias
a pendientes. `streamNACK` relaciona consumidor, tiempo y número de entregas.
La entrada, su presencia en PEL y su efecto de negocio **no son el mismo objeto**
[RACK][RCLAIM]. Los IDs del stream sirven para orden/identificación de entradas,
no certifican identidad de sesión, revisión de fuente, autorización o idempotencia
de una llamada LLM. El snapshot además contiene opciones IDMP en XADD [RA]:
sería incorrecto afirmar que Redis carece de todo mecanismo de deduplicación;
no se auditó ese subsistema ni se presume exactly-once del efecto externo.

El TTL de una clave tampoco es TTL semántico de cada recuerdo. Borrar una clave
stream puede destruir el contenedor y sus grupos; recortar entradas no equivale a
cerrar el trabajo del consumidor. La memoria conversacional precisa payload,
provenance y revisión; la PEL describe estado de ejecución. Embeddings, extraction,
answerer y juez LLM no forman parte de los tramos leídos.

### 0.4 Flujo trazado A — captura, entrega y consolidación

1. Cliente envía XADD con campos. `streamAppendItem` puede rechazar ID no creciente
   o entrada demasiado grande antes de presentar éxito; XADD genera la respuesta
   ID, modifica estado y reescribe el ID generado para propagación [RA].
2. `xreadCommand` (`src/t_stream.c:3056` en el SHA) distingue lectura normal y de
   grupo. La entrega con pending y la opción que evita ACK no son intercambiables.
   La revisión profundiza en el cierre PEL/claim, no en toda variante de bloqueo.
3. Worker recibe entrada y ejecuta un efecto de consolidación. **Este paso es
   aplicación, no código Redis**: no queda transaccionado con una DB distinta.
4. XACK valida todos los IDs antes de mutar. Para cada pendiente existente,
   desenlaza PEL de grupo y consumidor y destruye el NACK; no borra la entrada del
   stream por ese hecho [RACK]. Un ID ausente aporta cero, no prueba éxito de efecto.
5. Falla entre efecto y XACK: posible reentrega. Falla tras XACK pero antes del
   efecto: la cola no lo recupera como pendiente. Un ACK no certifica fsync.

**Ownership y cancelación:** la identidad del consumidor organiza pending, no
impide que un worker lento termine después de un reclaim. Cancelar el cliente o
perder la conexión no deshace un efecto ya confirmado. A Vanta le interesa vincular
`task.id`, intento y revisión de efecto, no copiar la estructura radix de PEL.
El scheduler Vanta ya reclama tareas y llama al handler antes de `complete_task`
[VWORK]; esa secuencia no se propone de nuevo como funcionalidad ausente.

### 0.5 Flujo trazado B — claim, recorte y reintento

1. XAUTOCLAIM recorre PEL desde un cursor con límite de búsqueda; hace merge con
   un iterador del stream para comprobar que existe la entrada [RCLAIM].
2. Si la entrada ya no existe, limpia el NACK y devuelve IDs borrados en la
   respuesta correspondiente: pendiente no implica payload recuperable.
3. Para la entrada elegible por idle, cambia propiedad, resetea tiempo de
   entrega y devuelve datos o IDs según variante. Un proceso previo puede seguir
   ejecutando el efecto: atomicidad del claim no es exclusión de código externo.
4. Un consumidor sin terminar antes del nuevo idle puede sufrir otra reclamación.
   Cursor de fin de recorrido no certifica ausencia futura de pending.
5. Reinicio exige recuperar estado de cola **y** del efecto. El rango leído de
   AOF no certifica toda serialización/replay de PEL ni todas las opciones de XADD.

**Errores discriminantes:** una entrada recortada, un pending transferido, un
payload expirado y un efecto fallido requieren razones distintas; reintentar
indiscriminadamente un efecto no idempotente duplica consolidaciones o gasto LLM.
En Vanta, `reclaim_stale` ya existe [VWORK]; el delta candidato es explicar la
resolución del efecto y la revisión, no instalar una segunda cola.

### 0.6 Flujo trazado C — escritura, durabilidad y expiración tras reinicio

1. Mutación del diccionario y respuesta del comando no son por sí solas fsync.
   El flush AOF distingue `always` y `everysec`; este último agenda trabajo en
   background, con estados de error y retraso [RAOF].
2. WAIT compara ACK de procesamiento por offset; WAITAOF comprueba offset local
   fsynced y ACK AOF de réplicas, y rechaza exigir local si AOF está desactivado
   [RWAIT]. Timeout devuelve observación incompleta, no rollback de la escritura.
3. Recovery AOF distingue EOF inesperado y formato inválido. Con opciones
   explícitas puede truncar hasta `valid_up_to`; registra truncamiento y retorna
   categoría específica. Sin permiso/éxito de esa reparación falla la carga
   [RREC]. No es una garantía de recuperar toda operación reconocida.
4. Al leer una clave, el contrato distingue válida/expirada/trimmed. Réplicas e
   importación no borran con las mismas reglas que el primario [RDB].
5. El ciclo activo de expiración usa esfuerzo y presupuesto [REXP]. Dejar de
   devolver la clave y recuperar sus páginas físicas son hitos diferentes.

**Límites:** RDB, manifest multipart AOF, rewrite y todo el replay no se auditaron
integralmente. Un rewrite AOF no es rebuild de ANN, y una entrada reproducible de
log no prueba que el índice derivado de Vanta esté listo. El contrato unificado
[MutationReceipt](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén)
separa `primary_commit`, `primary_durability`, `projections` y
`visibility_generation`: **primary durable puede preceder a readiness**.

## 1. Recomendaciones

| Prioridad cualitativa | Decisión | Valor incremental y coste |
|---|---|---|
| Primero | Hacer explícito dominio de confirmación en C13 | Evita interpretar ACK de trabajo como commit/fsync; exige trazar backend real |
| Primero, independiente de C13 completo | Explicar pending/efecto/retry del scheduler actual | Permite decidir una recuperación sin fingir exactly-once |
| Después del inventario de borrado | Informe de disposición TTL/purga | Vanta ya tiene TTL y purga; falta acreditar un relato uniforme entre superficies |
| Si hay presión operacional demostrada | Progreso reanudable/budgetado de mantenimiento | No copiar parámetros de Redis ni introducir otros thresholds |
| En paralelo, read-only | Diagnóstico de capacidades/configuración y UI honesta | Evita claims «durable» sobre in-memory o métricas de replay como estado live |

No hay resultado medido que justifique cambiar estructuras de datos o política
de eviction. El camino mínimo es extender receipts, reportes y trazas existentes.
Una ficha queda descartada si el inventario actual acredita el contrato completo.

## 2. Unidades candidatas Rust

Todos los nombres siguientes son **interfaces hipotéticas**, no APIs presentes.
No son tareas aprobadas ni nuevos IDs de backlog. Cada unidad usa el almacén,
scheduler, auditoría y control de acceso existentes. No requiere Redis en runtime.

### 2.1 Receipt de entrega y resolución idempotente del efecto

- **Problema/escenario:** worker L1 muere tras guardar resultado y antes de cerrar
  claim; otro worker no debe volver a aplicar una suma ni pagar extracción evitable.
- **Unidad/fuente:** relación entrada→pending→ACK de [RACK]/[RCLAIM], sin radix tree.
  [VWORK] ya implementa claim, reintento y complete tras handler: delta sólo donde
  falta vínculo explícito entre intento, efecto y revisión durable.
- **Contrato propuesto:** `resolve_delivery(task_id, attempt_epoch, effect_ref,
  source_revision) -> Result<EffectResolution, DeliveryError>`. Resolución distingue
  confirmado, no aplicado e incierto; errores incluyen stale owner, revisión
  superada, no autorizado y almacenamiento. Receipt no contiene texto fuente.
- **Invariantes:** efecto se identifica por tenant/sesión/tarea y revisión; un
  owner viejo no cierra el claim nuevo; replay del mismo efecto no lo duplica;
  timeout no se convierte en ausencia ni en rollback confirmado.
- **Seam real:** `PipelineWorker::run_task/reclaim_stale` [VWORK] y
  `LocalStateBackend::claim_task/complete_task` [VQUEUE]; storage SDK para resultado.
  El backend local conserva estado de proceso: no atribuirle durabilidad por el
  mero nombre `snapshot`. Persistencia de control es una decisión separada.
- **Alternativas:** sólo ampliar logs si ya hay resolución equivalente; adapter
  del worker si basta información existente; reimplementar receipt sólo con
  necesidad acreditada; rechazar cola distribuida nueva.
- **Pasos:** (1) trazar handler→put→complete; (2) acordar identidad/revisión;
  (3) ampliar resultado del handler sin alterar retries; (4) persistir resolución
  sólo si debe sobrevivir al proceso; (5) presentar estados a auditoría/Studio.
- **Migración/rollback:** campos opcionales o tabla auxiliar versionada del store
  existente; tareas antiguas quedan sin resolución acreditada, no «confirmadas».
  Desactivar emisión conserva lectura de receipts y no borra trabajo pendiente.
- **Dependencias:** C13 para afirmar primary durable; autorización y dedup propios.
  No depende de modificar TTL ni de añadir fencing multinodo.
- **FMEA:** crash después del efecto→replay duplica→detectar mismo effect_ref/
  revisión→resolver antes de ejecutar; cierre con owner viejo→pierde pending nuevo
  →comparar epoch/owner en settlement→rechazar stale. Se valida por interleavings,
  no por similitud con un issue Redis.
- **Aceptación/observabilidad:** dos intentos del mismo efecto dejan una resolución
  coherente; revisión nueva no se suprime; unknown sigue visible tras timeout;
  métricas por resultado, intento y duración, sin payload ni session IDs en labels.
- **Mínimo/no objetivos:** un tipo de tarea y su retry; sin exactly-once de red,
  nuevo broker ni rediseño de almacenamiento. Abierto: retención del receipt.

### 2.2 Barrera durable explícita, sin confundirla con visibilidad

- **Caso agente:** capturar un turno y comunicar al host si sobrevivirá al cierre
  antes de que termine su extracción o indexación. Una respuesta «guardado» que
  mezcla ambos hechos induce retries duplicados o falsa seguridad.
- **Evidencia:** `flushAppendOnlyFile` [RAOF] separa write/fsync y errores;
  `waitaofCommand` [RWAIT] comprueba offsets fsynced, no calidad del índice.
- **Delta Vanta:** ya hay store/WAL e índices reconstruibles. Extender el C13
  canónico, no añadir un receipt Redis ni renombrar el `ResourceReceipt` de TiDB.
  Propuesta: `await_primary_barrier(operation_id, backend_domain) ->
  Result<MutationReceipt, BarrierError>`, consumiendo el tipo de C13.
- **Contrato:** conserva `operation_id`, `primary_commit`,
  `primary_durability: Option<DurablePosition>`, `projections`,
  `visibility_generation`, `outcome`. `accepted`/`applied`/`durable` no forman un
  orden total ni estados excluyentes. No exigir fsync individual de derivados.
  Errores: backend sin barrera, operación desconocida, acceso denegado, IO;
  timeout devuelve conocimiento parcial, nunca rollback inferido.
- **Pasos:** localizar confirmación por backend; mapear posición y dominio;
  separar espera de emisión de receipt; propagar por bindings existentes;
  documentar operación antigua sin evidencia como desconocida.
- **Concurrencia/ownership:** el commit pertenece al store, no al future de
  espera; cancelar ese future no cancela el commit. No mantener locks de índice
  esperando IO ni convertir offset de otra instancia en prueba local.
- **Persistencia/deps:** posición válida tras reopen y política WAL del backend;
  no depende de 2.1 salvo que se afirme efecto durable de una tarea.
  Campo opcional versionado; desactivar espera no borra receipts ni modifica WAL.
- **Alternativas:** documentar garantías actuales si son suficientes; adapter
  que traduce receipt existente; rechazar simulación de fsync para InMemory.
- **FMEA:** ACK confundido con fsync → pérdida tras crash → detectar dominio/
  posición ausentes → mantener desconocido; primario durable e índice retrasado
  → retry innecesario → mostrar proyección pendiente, no reinsertar.
- **Aceptación manual futura:** interrumpir espera después del commit y consultar
  la misma operación; observar primario durable con proyección pendiente sin
  degradarlo a fallo; backend no durable no anuncia garantía física.
  Métricas por dominio/resultado sin texto, rutas sensibles ni IDs en labels.
- **No objetivos/abierto:** replicación, WAIT remoto o AOF Redis; falta decidir
  qué backends pueden acreditar barrera y cuánto receipt conservar.

### 2.3 Retención semántica separada de GC y presión de memoria

- **Caso agente:** un recuerdo caducado deja de participar en recall aunque su
  vector o páginas se recojan después; una memoria persistente no desaparece
  porque el proceso necesita liberar caché.
- **Evidencia mínima:** `expireIfNeeded` [RDB] distingue validez/expiración;
  `activeExpireCycle` [REXP] hace limpieza incremental; `getMaxmemoryState` y
  política `noeviction` [REVICT] separan presión de TTL.
- **Delta:** Vanta ya tiene TTL en registros, lifecycle y gobernador. No crear
  otro recolector: ampliar su diagnóstico para distinguir exclusión lógica,
  cleanup derivado y espacio recuperado cuando el inventario no lo expresa.
- **Contrato propuesto:** `retention_status(record_ref, source_revision) ->
  Result<RetentionObservation, RetentionError>` con razones
  `Live | Expired | Superseded | Deleted`, estado de cleanup por proyección y
  alcance observado. Ausencia, error de lectura y no autorizado son distintos.
  La observación no concede permiso para borrar.
- **Pasos:** trazar filtros TTL actuales en get/search/recall; definir qué reloj
  gobierna validez persistida; conectar lifecycle y auditoría; sólo después
  ampliar GC existente con revalidación de revisión antes de cada borrado.
- **Ownership/concurrencia:** tombstone y revisión pertenecen al store;
  una tarea GC vieja no elimina una revisión nueva. Cancelar limpieza conserva
  tombstone y cursor reanudable; no resucita contenido fuera de plazo.
- **Persistencia/deps:** política de retención, TTL y autorización existentes;
  C13 sólo si se reporta durabilidad del tombstone. Migración aditiva; rollback
  del diagnóstico no reactiva objetos caducados ni elimina sus evidencias.
- **Alternativas:** sólo mejorar reporte si filtros/GC ya son suficientes;
  evaluación bajo demanda antes de sweep; rechazar eviction de memoria canónica
  como sustituto de una política de retención autorizada.
- **FMEA:** reloj retrocede → reaparece recuerdo → validar política al leer y
  registrar frontera temporal; GC lee revisión vieja → borra update → comparar
  revisión al aplicar; liberar caché interpretado como borrado → pérdida semántica
  → separar eventos y contadores por recurso.
- **Aceptación futura:** expirado no llega al contexto aunque GC esté pausado;
  update concurrente no se borra por tarea anterior; fallo de storage es error,
  no `Deleted`; reporte sin payload por defecto.
- **No objetivos/abierto:** purga forense de backups, relojes distribuidos,
  TTL nuevo por cada tipo de memoria o política automática decidida por LLM.

### 2.4 Diagnóstico de pendientes sin payload y settlement con owner vigente

- **Caso agente:** explicar por qué una consolidación sigue pendiente tras borrar
  su fuente; impedir loops que pagan de nuevo al proveedor sin datos válidos.
- **Evidencia:** `xautoclaimCommand:5225-5305` [RCLAIM] comprueba entrada antes
  de entregar pendiente; `xackCommand:4248-4310` [RACK] retira NACK, no payload.
  [RISSUE1] muestra por qué claim atómico no equivale a exclusión del efecto.
- **Delta:** worker y cola Vanta ya reclaman/reintentan. A diferencia de 2.1,
  esta unidad diagnostica y clasifica huérfanos antes del handler; no resuelve
  commit ni introduce dedup de negocio.
- **Contrato propuesto:** `inspect_pending(task_id, attempt_epoch) ->
  Result<PendingObservation, QueueError>` con fuente/revisión presente,
  owner vigente, causa conocida o desconocida; `settle_orphan` exige expectativa
  de revisión/owner y autorización del scheduler, no del modelo.
- **Pasos:** resolver referencia de fuente en cola existente; separar
  `NotFound`/`Expired`/`Superseded` de IO; producir diagnóstico; habilitar
  settlement sólo para causas inequívocas, conservando dead-letter cuando proceda.
- **Ownership/cancelación:** revalidar owner bajo frontera de mutación de cola;
  observación previa no es CAS. Cancelar inspección no completa ni reencola.
  Si efecto sigue activo o es incierto, enlazar 2.1, no eliminar evidencia.
- **Persistencia/deps:** referencias y epochs de tarea; no nuevo broker.
  Metadata opcional compatible; dejar de emitir diagnóstico conserva pendientes.
- **Alternativas:** dead-letter existente si ya expresa la causa; reporte read-only
  si no hay settlement seguro; no copiar PEL/radix ni su ID temporal.
- **FMEA:** lectura fallida parece fuente borrada → trabajo perdido → conservar
  error; owner cambia tras inspección → cierre ajeno → comparación atómica;
  eliminar fuente con efecto en curso → resultado tardío → validar revisión.
- **Aceptación futura:** distinguir huérfano real, error transitorio y trabajo
  reclamado; owner antiguo no completa nuevo intento; un diagnóstico no cambia
  la cola. Observabilidad por causa sin contenido ni nombres sensibles.
- **No objetivos/abierto:** reentrega exactamente una vez, borrar consumidores
  de terceros; decidir qué razones admiten settlement automático.

### 2.5 Preflight de recuperación con pérdida explícita

- **Caso agente:** un operador debe conocer qué captura podría perderse al abrir
  un WAL dañado, antes de que el host interprete «arrancó» como «recuperó todo».
- **Evidencia:** `aof.c:2094-2129` [RREC] distingue truncamiento y corrupción;
  [RISSUE2] debate el coste de reparar automáticamente, no demuestra un defecto
  actual Vanta ni acredita una causa filesystem universal.
- **Delta:** auditoría/WAL/migraciones existentes; ampliar reporte de recuperación
  si falta la frontera válida y la decisión explícita. No otro formato de log.
- **Contrato propuesto:** `inspect_recovery(scope) ->
  Result<RecoveryAssessment, RecoveryError>` con último tramo válido, categoría
  de daño, bytes no acreditados y acciones permitidas. Es read-only; reparación
  es otra operación autorizada con precondición de identidad del archivo.
- **Pasos/deps:** localizar decoder/recovery real; conservar original y versión;
  delimitar daño sin adivinar operaciones perdidas; enlazar auditoría y C13;
  proponer acción sólo si política backend la permite. No migración en inspección.
- **Ownership/concurrencia:** una inspección no repara un WAL vivo; archivo
  cambiado invalida assessment. Cancelar antes de reparar no escribe; después
  de comenzar una reparación no inferir rollback por timeout.
- **Alternativas:** reporte backend actual si completo; abortar apertura ante
  daño no interpretable; rechazo de recuperación silenciosa.
- **FMEA:** truncate sobre daño interior → pérdida no acotada → rechazar
  automaticidad; bytes inválidos tomados como vacíos → falsa integridad →
  categoría `Unknown`; archivo reemplazado tras inspección → daño ajeno →
  verificar identidad/revisión.
- **Aceptación futura:** inspección no altera bytes; pérdida potencial permanece
  visible aunque apertura posterior tenga éxito; no inventa IDs de operaciones
  a partir de bytes ilegibles. Informe sin payload.
- **No objetivos/abierto:** reparar ahora, garantías de power-loss por simulación,
  reemplazar la recuperación del backend; precisar alcance de forensic backup.

## 3. Issues y seguridad

Consulta pública **2026-11-07**, separada del SHA del clon. Se leyeron cuerpos
completos y comentarios por REST GitHub; no se ejecutaron reproducciones.

| Incidencia leída | Estado observado y mecanismo | Aplicación discriminante |
|---|---|---|
| [#13956 — XAUTOCLAIM concurrent access duplicates messages][RISSUE1], creada 2025-04-18 | Abierta; el reporte atribuye duplicados a varios workers. [Respuesta 2025-04-19](https://github.com/redis/redis/issues/13956#issuecomment-2816596033): claim atómico resetea idle; revisar `min-idle-time`. No acredita carrera interna ni solución «hacer blocking» | Contrastar ejecución que supera idle frente a claim simultáneo. En Vanta probar owner viejo finalizando después de reclaim, no presumir duplicado actual |
| [#5177 — AOF file corrupted after a power loss…][RISSUE2], creada 2018-07-26 | Abierta; logs de Redis 4.0.9, cola con ceros, `aof-load-truncated` no resuelve formato inválido. Antirez propone explicación por orden filesystem; autor reporta `data=ordered`, por lo que causa no queda universalmente probada. Comentarios hasta 2024-09-12 debaten reparación automática y pérdida | El SHA actual ya tiene opción diferenciada para corrupt tail [RREC]. No atribuir al snapshot la carencia histórica; Vanta debe distinguir tail corto, corrupción interior y error IO antes de ofrecer acción |

No se identificó en esos hilos una PR que permita certificar corrección de ambas
incidencias en este SHA. Abierto no significa «Redis entero roto».
La segunda se conserva pese a antigüedad por su evidencia directa de recovery;
no se usa como sustituto de una auditoría de seguridad reciente.

**Fronteras:** cliente autenticado→comando/ACL; cola→worker con permisos;
payload recuperado→texto tainted, nunca instrucción ejecutable; store→filesystem;
diagnóstico→operador autorizado. `multi.c:182-221` [RACL] revalida ACL en ejecución:
encolar no otorga autorización permanente. No se propone portar MULTI/SQL.
No registrar credenciales de conexión ni payload en métricas. El permiso de
consultar un namespace no concede borrar, recortar o reparar sus datos.

## 4. Contratos previos en VantaDB

| Candidata | Condición específica, no bloqueo universal |
|---|---|
| 2.1 | Identidad/revisión del efecto y propiedad del intento; `run_task` ejecuta antes de completar [VWORK] |
| 2.2 | Frontera durable demostrable del backend y C13; no depende del checkpoint global |
| 2.3 | TTL/lifecycle y autorización; GC condicionado a revisión vigente |
| 2.4 | Fuente referenciable y settlement owner-aware de cola; diagnóstico puede existir sin mutar |
| 2.5 | Decoder y auditoría recovery; no reparación automática ni nuevo WAL |

`CheckpointManager::mutate` [VCP] hace read→closure→write separado. Es evidencia
estática de ese helper, no prueba de carrera en cada caller ni requisito de
reescribir todos los checkpoints. Tampoco la cola en memoria se vuelve durable
por llamarse `LocalStateBackend`. C13 es **propuesto**, no API ya implementada.

## 5. Qué no trasladar

- Redis servidor/RESP, cluster, replicación, módulos Search/JSON completos,
  allocators C y radix PEL: deuda operacional no justificada para motor Rust.
- Eviction de recuerdos canónicos como política de presión: liberar caché no
  autoriza olvidar fuente persistida.
- Offsets globales/IDs temporales como revisión semántica o autorización.
- ACK antes de efecto como garantía exactly-once ni WAIT como rollback.
- Reparación destructiva al arrancar «para no bloquear»: requiere decisión
  explícita y evidencia de pérdida, no una opción heredada por comodidad.
- Source-to-source C→Rust: no es escape a RSAL/SSPL/AGPL; revisar material concreto.

## 6. Advertencias metodológicas

El commit congelado contiene funciones posteriores a Redis 7; no trasladar
supuestos de tutoriales antiguos (BSD, ausencia de IDMP o corrupt-tail recovery).
Las opciones de AOF y la política efectiva del despliegue no se deducen sólo
por existir una rama en C. No se abrió ninguna base ni se leyó configuración
de un servidor real. Los tres flujos son lectura estática, no reproducciones.
Streams proporciona estado de entrega, no modelo de memoria conversacional.

Ruta web realmente usada: `websearch.search` sobre issues XAUTOCLAIM devolvió
cero; `metasearch.search_web` con Bing/Yahoo también cero. Fallback REST
`api.github.com/search/issues` y `/repos/redis/redis/issues/{13956,5177}`
con `/comments` permitió leer las incidencias. El intento de árbol de
RedisInsight por REST sufrió rate-limit; se leyó `package.json` por raw al SHA.
La URL inicial `/develop/tools/redisinsight/` devolvió 404; la ruta oficial
`/develop/tools/insight/` sí respondió. No se tomó un snippet como cuerpo completo.

## 7. Benchmarks

### Runner y población realmente leídos

`src/redis-benchmark.c::main` [RBDEFAULT] → `benchmark` [RBSTART] →
`createClient/createMissingClients`, event loop → `writeHandler/readHandler`
[RBREAD] → `showLatencyReport` [RBREPORT]. El recorrido de red inspeccionado
se concentra en handlers y fronteras de medición; no audita todo TLS/cluster.

| Dimensión | Fuente y significado exacto |
|---|---|
| Defaults leídos | `main:1764-1797`: 50 clientes, 100 000 requests, payload de 3 bytes, pipeline 1, keepalive, randomkeys desactivado, host 127.0.0.1:6379, DB 0, sin auth, 0 threads configurados |
| Dataset/unidad | Comandos y claves sintéticas; unidad respuesta de comando, no documento relevante ni sesión. No split ni ground truth semántico |
| Semilla | `main:1758-1759` usa reloj y PID; no es semilla reproducible fijada |
| Tiempo | `readHandler:452-455` mide hasta primer read event; excluye parsing del cliente y reutiliza la latencia para replies pendientes del pipeline. No es latencia aislada de cada elemento de un pipeline |
| Throughput | `showLatencyReport:860`: `requests_finished / (totlatency/1000)`; `benchmark:999-1002` delimita reloj de la prueba. No sustituir denominador por requests deseadas si hubo aborto |
| Percentiles | HDR guarda latencias en μs con saturación al máximo; reporte las convierte a ms. Distribución sobre muestras registradas, no recall ni exactitud |
| Errores | Fallo de lectura/parseo y errores ordinarios del servidor abortan; cluster tiene tratamiento especial de topología. Prefijos AUTH/SELECT no incrementan la población medida [RBREAD] |
| Warmup | No fase separada de warmup en el `benchmark` leído; no inferir caché fría ni estado vacío del servidor |
| Modelos/costes | Sin embedding, tokenizer, answerer ni juez. No llamadas pagadas en este runner; su coste de red/servidor no se midió aquí |

El runner escribe datos; distintos comandos mutan colecciones y sobrescriben
claves. No se inspeccionó todo el menú como garantía de limpieza/reset.
No ejecutarlo contra DB compartida. Requiere binario cliente, hiredis/HDR,
servidor Redis accesible y configuración efectiva conocida; nada se instaló
ni arrancó. AOF/eviction/auth/TLS del servidor alteran lo medido y no quedan
congelados por los defaults del cliente.

### Adaptación sin autorar evaluador nuevo

Aplicar [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md): sólo pista **motor** para este runner.
Un adapter sobre el SDK Vanta mapearía clave→namespace/key y bytes→payload,
con idéntica semántica de sobrescritura/lectura y acknowledgements explicitados
por C13; RESP y cluster no son portables al motor embebido.
Su latencia hasta primer evento no es comparable directamente con get embebido
deserializado. Para memoria/retrieval seguir runners existentes separados;
conservar errores y poblaciones del método, sin añadir prompts, scoring,
thresholds, sampling, configuraciones ni harnesses. No hay números propios ni
tabla de superioridad; ningún claim comercial queda reproducido.

## 8. Matriz de capacidades

Referencia Vanta `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`,
[inventario discriminante](VANTA-REFERENCE.md). «No acreditado» limita la muestra.

| Capacidad | Estado Vanta | Evidencia / decisión |
|---|---|---|
| Claim/reclaim/handler/complete | Existe | [VWORK]/[VQUEUE]; extender identidad del efecto, no instalar broker |
| Estado durable uniforme de esa cola | No acreditado | Backend leído usa mutex/estado local; no inferir persistence end-to-end |
| Memoria, TTL y lifecycle | Existe | `lifecycle.rs`, `PutRequest.ttl_ms`, referencia; 2.3 sólo explica fronteras |
| Gobernador/contadores | Existe | `src/memory_governor.rs:56-68`; no usar eviction sobre canon |
| Primario/WAL/ANN/BM25/RRF | Existe | Referencia; 2.2 no propone otro índice |
| Receipt multialmacén uniforme | No acreditado | C13 es diseño; separar durable/readiness |
| Efecto con resolución por intento/revisión | Parcial | Seams worker presentes; resultado durable uniforme no probado |
| Provenance/sesiones/dedup/supersession | Existe | Referencia y `l1_writer`; no reemplazar con stream ID |
| Presupuesto y gobernanza de recall | Existe | `auto_recall.rs::RecallConfig/InjectionPolicy`; scope no sustituye ACL |
| Auditoría recovery con assessment de pérdida | Parcial | Auditoría/migración existentes; 2.5 condicionado a alcance real |
| Studio | Existe | Worktree inspeccionado §10; nuevos paneles no implementados |
| Cluster Redis, PEL como memoria semántica | N/A | Fuera de alcance y modelo incorrecto para contenido |

## 9. Negativos sustentados y falsos positivos

1. **Hecho local acotado:** `CheckpointManager::mutate:112-117` [VCP] separa
   read/write sin CAS en ese helper. Dos callers superpuestos podrían perder
   actualización; **hipótesis**, depende de locks externos. Validación futura:
   intercalar dos incrementos del mismo checkpoint bajo callers reales.
2. **Hecho de frontera:** [VQUEUE] reclama/completa estado bajo mutex de proceso.
   No acredita supervivencia de cola a crash ni vínculo con commit del efecto.
   No afirmar que todos los backends Vanta pierdan tareas.
3. **Riesgo funcional:** [VWORK] handler y complete son pasos separados.
   Un handler no idempotente puede repetirse al perder ACK; comprobar tipo de
   handler e identidad antes de atribuir pérdida/duplicado actual.
4. **Falsos positivos descartados:** no faltan TTL, presupuestos, scheduler,
   dedup, índices híbridos ni Studio. `snapshot` no es sinónimo de fsync;
   contar pendientes no demuestra disponibilidad del contenido.

No hay bug reproducido ni hallazgo de corrupción WAL actual. Revalidación de
concurrencia, crash y bindings pendiente, por candidato, no bloqueo universal.

## 10. UI y Studio

**Producto externo real:** [documentación Redis Insight](https://redis.io/docs/latest/develop/tools/insight/),
leída 2026-11-07, describe desktop GUI, conexiones, browser CRUD, CLI,
Workbench, profiler y análisis. La página advierte extrapolación sobre más de
10 000 claves: no presentar un gráfico como inventario exhaustivo.
Se leyó hasta el límite devuelto por la herramienta (9 000 caracteres), no
sus subsecciones finales de telemetría/licencia completas.

**Código externo leído:** [package.json][RUI] al SHA
`43bd288ed30dc6d2355723158dad5d7ecf4a280e`: scripts separan UI/API/desktop,
Electron-builder empaqueta, dependencias incluyen React/TypeScript. Declara
SSPL, no BSD ni licencia del motor. No se inspeccionaron componentes de browser,
gestión de secretos o LICENSE completo: esos flujos son evidencia documental,
no verificación de implementación ni permiso de reutilizar UI.

**Destino real:** Vanta Studio, worktree `develop-desktop` SHA
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`; leídos
[`feature-freez/desktop/package.json:1-56`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/package.json#L1-L56)
(Vite/React/Tauri) y
[`feature-freez/desktop/src-tauri/tauri.conf.json:1-38`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src-tauri/tauri.conf.json#L1-L38)
(ventana y CSP), cuerpos contrastados por `git show`.
Las rutas actuales `desktop/...` tienen movimientos/modificaciones locales:
no se atribuyen sus bytes a este pin ni se usa JobChip/jobs como evidencia del SHA.
Oportunidad: inspector read-only de tarea→intento→efecto→barrera→proyección;
acciones destructivas aparte, autorizadas y con preview. No copiar Copilot ni
permitir al LLM reparar/borrar por contenido recuperado. Fuente UI Vanta acredita
producto existente, no integración de las propuestas ni seguridad runtime.

## 11. Evidencias, cobertura y handoff

**Conservado:** flujos de entrega, AOF y expiración de la ampliación parcial.
**Completado:** fichas 2.2–2.5, incidencias leídas, runner, matriz y límites.
Todos los ejes tienen desarrollo; no se certifica exhaustividad ni aprobación.
No leídos integralmente: módulos Search/JSON, implementación IDMP, RDB/rewrite,
replay PEL, ACL completa, todos los clientes, UI y árbol legal de `deps/`.
Licencia: RSAL/SSPL son source-available no OSI; opción AGPL tiene obligaciones
distintas. Reescribir no exime de revisión legal; dependencias tienen avisos propios.

Dependencias: 2.1→2.2 sólo para afirmar efecto durable; 2.4→2.1 para resolver
efecto incierto; 2.3 y 2.5 independientes. Validación manual futura por ficha,
sin ejecución en esta sesión. Revisión adversarial independiente pendiente del
responsable. No se tocaron contrato/método, JSON, backlog, código ni índices.

**Split plan (>300 líneas):** al aprobar specs, separar entrega/settlement
(2.1/2.4), persistencia/recovery (2.2/2.5), retención (2.3) y anexo de evidencia;
mantener este dossier como referencia y enlaces bidireccionales. No crear esos
archivos ahora ni dividir para ocultar lagunas.

[RL]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/LICENSE.txt#L1-L8
[RA]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/t_stream.c#L2806-L2965
[RACK]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/t_stream.c#L4248-L4310
[RCLAIM]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/t_stream.c#L5153-L5345
[REXP]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/expire.c#L303-L337
[RDB]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/db.c#L3137-L3185
[REVICT]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/evict.c#L543-L563
[RAOF]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/aof.c#L1500-L1650
[RREC]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/aof.c#L2094-L2129
[RWAIT]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/replication.c#L4916-L4989
[RACL]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/multi.c#L182-L221
[RBENCH]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c
[RBDEFAULT]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c#L1751-L1799
[RBSTART]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c#L974-L1010
[RBREAD]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c#L445-L554
[RBREPORT]: https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c#L845-L935
[RISSUE1]: https://github.com/redis/redis/issues/13956
[RISSUE2]: https://github.com/redis/redis/issues/5177
[RUI]: https://github.com/RedisInsight/RedisInsight/blob/43bd288ed30dc6d2355723158dad5d7ecf4a280e/package.json#L1-L55
[VWORK]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L265-L390
[VQUEUE]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/local_backend.rs#L220-L290
[VCP]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/checkpoint.rs#L112-L117
