---
title: "Redis: streams, sesiones, retención y límites de confirmación"
kind: research
status: proposed
description: "Lectura estática de pending deliveries, expiración y persistencia sin equiparar ACK con durabilidad"
tags: [research, memory, engineering]
---

# Redis: streams, sesiones, retención y límites de confirmación

## 1. Recomendación y arquitectura

**Dictamen:** separar entrega de trabajo, retención lógica y barrera durable.
No convertir toda la memoria persistente de VantaDB en caché evictable.
Local `C:\Users\Eros\redis`; HEAD con fuentes comprobado:
`5b69ba2ce91fea8e9fc70ebac8ce341b37e75533`.
[Snapshot](github-candidates.json): **77 268 estrellas, NOASSERTION**.
[Manifiesto de preparación](clone-manifest.json).

```text
cliente RESP → servidor C / despacho de comandos / ACL
  diccionario de claves → string/hash/list/set/zset/stream/otros
  XREADGROUP → consumer group + pending entries (PEL)
  XACK → retira pending; no borra necesariamente entrada del stream
  lectura clave → expiración perezosa; cron → expiración activa
  presión maxmemory → política de eviction o rechazo
  persistencia → RDB y/o AOF; fsync según política
  replicación → offsets/ACK; WAIT ≠ WAITAOF
redis-benchmark → cliente de carga; Redis Insight → producto externo
```

Redis sirve como estado de sesión/cola mediante sus primitivas, no porque el
PEL sea una sesión semántica de agente. La aplicación define idempotencia,
identidad de sesión y efecto durable. No se auditó Redis Search/vector modules:
el README actual anuncia más que caché; no se declara inexistencia de ANN.

## 2. Fuentes verificadas y fichas

| Ref. | Código con rango/enlace inmutable | Hecho observado |
|---|---|---|
| R1 | [`src/t_stream.c:4248-4304`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/t_stream.c#L4248-L4304) | XACK valida IDs antes de mutar y retira entradas de PEL grupo/consumidor |
| R2 | [`src/expire.c:303-326`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/expire.c#L303-L326) | Expiración activa incremental con presupuesto y esfuerzo configurado |
| R3 | [`src/db.c:3137-3182`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/db.c#L3137-L3182) | Distingue expired/valid/trimmed; réplicas/import tienen reglas distintas de borrado |
| R4 | [`src/evict.c:543-563`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/evict.c#L543-L563) | Memoria insuficiente con noeviction retorna fallo, no borra para fingir éxito |
| R5 | [`src/aof.c:1625-1650`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/aof.c#L1625-L1650) | Always ejecuta fsync; everysec agenda fsync en background; fallo always es fatal |
| R6 | [`src/replication.c:4916-4989`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/replication.c#L4916-L4989) | WAIT espera procesamiento; WAITAOF comprueba offsets fsynced y soporte AOF |
| R7 | [`src/multi.c:182-221`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/multi.c#L182-L221) | ACL se vuelve a comprobar al ejecutar comandos encolados |
| R8 | [`src/redis-benchmark.c:845-875`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c#L845-L875) | HDR histogram, throughput y p50/p95/p99; reconoce cero muestras |

### Ficha candidata A — receipt de trabajo pendiente y efecto idempotente

- **Objetivo:** distinguir entrega a un worker de consolidación de su efecto.
- **Mínimo / fuente exacta:** identidad de entrada + pending + ack tras efecto,
  inspirado en R1; no portar radix tree ni protocolo Streams.
- **Seam Vanta:** sesiones/scheduler/checkpoints/ingestors existentes según
  [METHOD](METHOD.md#vantadb-como-referencia), operaciones persistentes SDK.
- **Tipos propuestos:** `DeliveryReceipt { job_id, attempt, owner_epoch }` y
  `EffectKey`; sólo extensión de contratos existentes si falta esa distinción.
- **Acciones:** inventariar jobs actuales, identificar punto de efecto y ACK,
  revisar idempotencia ya implementada y regla de reentrega después de reinicio.
- **Dependencias:** almacenamiento del receipt, propiedad de worker, cancelación,
  autorización y vínculo estable sesión→trabajo→resultado.
- **FMEA:** efecto durable antes de ack produce reentrega; ack antes del efecto
  pierde trabajo; TTL del receipt prematuro permite repetir efecto.
- **Aceptación funcional propuesta:** reentrega no duplica efecto identificado;
  estado incierto no se presenta como éxito/rollback; otro tenant no reclama trabajo.
- **No objetivos:** exactamente-una-vez universal, nueva cola de red ni Redis como runtime obligatorio.

### Ficha candidata B — contrato de retención separado de caché y purga

- **Objetivo:** evitar confundir dejar de devolver un registro con borrarlo de
  índices, WAL, backups y almacenamiento físico.
- **Mínimo / fuente:** R2/R3 separan expiración lógica/borrado incremental;
  R4 separa presión de memoria de expiración.
- **Seam Vanta existente:** `src/sdk/serialization/mod.rs:528-538`,
  `purge_expired` y `src/server/ttl_tests.rs:1-10`: **TTL perezoso, default por
  namespace y sweeper de purga ya existen**. No añadirlos otra vez.
- **Tipos propuestos:** `RetentionDisposition::{HiddenExpired, PurgedPrimary,
  ReclaimPending}` como diagnóstico compatible, sujeto a inventario actual.
- **Acciones:** mapear estado visible/índices/borrado certificado/compactación;
  explicitar qué retiene WAL/backups y qué clase de dato sí es cacheable.
- **Dependencias:** reloj, tombstones, eliminación multiíndice, certificados
  actuales y política de persistencia; no alterar valores de TTL.
- **FMEA:** eviction de memoria canónica pierde recuerdos; purga sólo en metadata
  deja recuperación por otro índice; reloj regresivo cambia visibilidad.
- **Aceptación funcional propuesta:** expirado no reaparece por otro brazo;
  diagnóstico no promete secure erase; rechazo de memoria no se oculta como eviction.
- **No objetivos:** implementar TTL, ajustar sweeper, umbrales de memoria ni nueva política LRU.

### Ficha candidata C — confirmación de aplicación frente a sync

- **Objetivo:** que clientes sepan qué frontera alcanzó una escritura.
- **Mínimo / fuente:** distinción WAIT/WAITAOF de R6 y fsync de R5,
  no replicación completa.
- **Seam:** [`src/wal.rs:999-1028`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/wal.rs#L999-L1028)
  ya separa Always/Periodic/Never y sync explícito; backend batch/flush existen.
- **Tipos propuestos:** `DurabilityReceipt { applied_position,
  synced_position, backend_capability }`, ampliar respuesta actual si procede.
- **Acciones/dependencias:** trazar flush real por backend, índice derivado y
  error IO hasta SDK; preservar garantía actual y documentar in-memory.
- **FMEA:** contar réplicas o ACKs como fsync sobrepromete; timeout no deshace
  escritura; fallback de backend puede cambiar garantía sin avisar.
- **Aceptación funcional propuesta:** durable sólo si barrera correspondiente
  tuvo éxito; unsupported y unknown no se convierten en confirmado.
- **No objetivos:** comando WAIT compatible Redis, consenso o garantía de energía perdida sin evidencia.

## 3. Riesgos, issues y seguridad

**Estático:** XACK altera PEL (R1), no representa fsync ni commit del efecto
externo; WAIT tampoco certifica sync a disco (R6). Son contratos distintos,
no bugs de Redis. R7 evidencia que permisos al encolar no bastan para ejecución diferida.
**Hipótesis:** pérdida de efecto por ack temprano, duplicación por retry,
retención insuficiente del receipt o eviction de estado de sesión canónico.
No se reprodujo ninguna vulnerabilidad ni incidencia.
**Issues upstream:** no consultados.

**Configuración default estática:** [`redis.conf:88-112`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/redis.conf#L88-L112)
liga loopback y protected-mode yes.
[`redis.conf:1545-1604`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/redis.conf#L1545-L1604)
incluye appendonly no y appendfsync everysec: el segundo no habilita el primero.
No afirmar «sin persistencia» por ello: RDB es otra superficie, no auditada aquí.
Frontera: red/ACL y configuración de despliegue; cambios de bind/protected-mode,
auth/TLS y ACL efectivas requieren revisión propia. No se certificó hardening.

## 4. Prerrequisitos VantaDB

Comprobar implementado antes de proponer: ya existen TTL/sweeper, sesiones,
jobs, checkpoints, borrado certificado, HNSW/BM25/RRF y backend persistente.
[`src/server/ttl_tests.rs:1-10`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/server/ttl_tests.rs#L1-L10)
es evidencia de contrato escrito, **no resultado ejecutado** en esta tarea.
Revalidar auditorías ANN/híbrido/WAL/backend y estado dirty antes de intervención.
El [contraste forense previo](../../reviews/2026-11-03-forensic-verification-external-claims.md)
ya distingue existencia de controles de una garantía universal de crash safety.

## 5. Qué no trasladar

No usar maxmemory eviction de Redis como política tácita de memoria a largo
plazo; no tratar consumer group como autoridad de sesión/tenant.
No portar semántica MULTI/EXEC suponiendo rollback SQL de cualquier error:
el rango R7 sólo prueba chequeo ACL, no una auditoría transaccional completa.
No añadir Redis obligatorio ni reemplazar WAL por ACK de cola.

## 6. Método, licencias y omisiones

Lectura estática de R1-R8, config, README y licencia raíz; no se ejecutó fuente,
runner, tests/builds, scripts o servicios. Historial shallow/tags omitidos;
LFS/submódulos no materializados según manifiesto.
Omitidos: módulos de búsqueda/vector/JSON, RDB, cluster/failover completos,
persistencia integral del PEL, reclamación de pending, Lua y todos los comandos TTL.

Snapshot **NOASSERTION** permanece: no sustituirlo por una licencia supuesta.
[`LICENSE.txt:1-8`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/LICENSE.txt#L1-L8)
declara elección RSALv2/SSPLv1/AGPLv3 para Redis 8+, BSD para releases anteriores.
Cabecera R1: mismas tres opciones. No se clasificó todo archivo/dependencia:
per-file unknown cuando no verificado; no asumir BSD por historia.
Cambiar C a Rust no exime de derivación/licencia.
Antecedentes Vanta históricos no se elevan a bugs actuales.

## 7. Runner upstream existente

R8 es el cliente C **redis-benchmark** sobre servidor real, no evaluación ANN
ni calidad de memoria. Parámetros/defaults visibles en
[`src/redis-benchmark.c:1760-1777`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/redis-benchmark.c#L1760-L1777):
clientes/peticiones, payload y claves sintéticas; admite rango de claves,
pipeline, comandos y threads. No hay dataset semántico ni groundtruth de recuerdos.
Reporta throughput, media, min/max y p50/p95/p99 con HDR; precisión temporal
depende del ciclo de lectura/pipeline, no equivale automáticamente a latencia
aislada de cada efecto durable.

Dependencias/costes: binario nativo y servidor, hiredis/HDR, CPU del cliente,
red, conexiones, RAM/datos escritos, política AOF/RDB y disco del servidor.
TLS/cluster/pipeline alteran el experimento; no se cuantificó coste cloud.
La búsqueda acotada no encontró FLUSHDB en ese archivo; no asumir limpieza de
claves ni servidor después de la prueba. Usar instancia desechable en una
validación manual futura; no compartir sesiones reales.
No se ejecutó ni se fijaron ajustes/umbrales; protocolo único:
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Superficie | Estado VantaDB | Delta defendible |
|---|---|---|
| Sesiones/jobs/checkpoints | Existe | Relación delivery→efecto→receipt pendiente de acreditar |
| TTL perezoso/default namespace/sweeper | Existe | Explicar hidden/purged/reclaim, no reimplementar |
| Borrado certificado/compactación | Existe | No confundir con TTL ni secure erase universal |
| ANN/BM25/dense+sparse RRF | Existe | Redis Search fuera del alcance, no comparación negativa |
| WAL/SyncMode/backend batch | Existe | Receipt por barrera, no reemplazo de persistencia |
| Auditorías ANN/híbrido/WAL/backend | Existen históricas; vigencia parcial | Revalidar dirty; sin certificación nueva |
| Eviction de memoria canónica | No aplicable como requisito | Rechazar adopción implícita |

## 9. Debilidades sugeridas

La cuestión útil es si «confirmado», «expirado» y «borrado» tienen significados
uniformes en memoria, jobs y SDK. Redis muestra por qué deben separarse.
No prueba que Vanta pierda trabajo o conserve expirados indebidamente.
Prioridad condicionada: C por honestidad de garantías; A si hay reentrega real;
B como claridad de diagnóstico existente, no feature de TTL nueva.

## 10. UI y escritorio

[`README.md:154-159`](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/README.md#L154-L159)
remite a **Redis Insight externo**. No se leyó su repositorio ni se demuestra
en este checkout su frontend o empaquetado desktop; no atribuir esas funciones
al servidor C. redis-cli es consola, no desktop nativo.
Vanta Studio ya tiene memoria/actividad/diagnóstico en `develop-desktop`;
sería destino de receipts/retención, no otra UI a implementar.
