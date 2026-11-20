---
title: "Agentmemory: recibos de captura y coordinación por leases"
kind: research
status: proposed
description: "Lectura estática de captura recuperable, leases y runners de agentmemory para unidades independientes en Rust"
tags: [research, memory, engineering]
---

# Agentmemory: recibos de captura y coordinación por leases

## Identidad y dictamen

- Repositorio **`rohitg00/agentmemory`**, no otros proyectos homónimos.
- Clon `C:\Users\Eros\agentmemory`; SHA
  `4f758eff36c8403570e1bf2ac6d986239fcdb662`, rama `main`.
- Snapshot congelado: **30 183 estrellas**, Apache-2.0, según
  [candidatos](github-candidates.json) y [manifest](clone-manifest.json).
- [METHOD](METHOD.md); contraste fijo Vanta
  `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y [referencia](VANTA-REFERENCE.md).
  Ampliación por lectura local/pública; sin ejecución del producto.

**Índice:** [arquitectura](#1-arquitectura-tecnologías-flujo-y-recomendaciones),
[fichas A/B](#2-unidades-mínimas-candidatas-para-rust),
[riesgos](#3-defectos-riesgos-e-issues-código-frente-a-hipótesis),
[prerrequisitos](#4-prerrequisitos-vanta), [rechazos](#5-qué-no-trasladar),
[método](#6-método-licencia-y-límites), [runners](#7-runners-upstream-leídos-métricas-y-efectos),
[matriz](#8-matriz-frente-a-vantadb), [negativos](#9-deuda-sugerida-sin-inventar-bugs-vanta),
[UI](#10-uiescritorio-y-studio), [cierre ampliado](#complemento-de-autoría-y-cierre).

**Dictamen:** estudiar el recibo de captura con estados y coordinación
acotada de trabajos. No portar iii-engine, el inventario de herramientas ni
su producto de memoria completo. Vanta ya tiene provenance/supersession/dedup,
BM25/HNSW/RRF, assembler/compaction, scheduler, checkpoints e ingestors.
El contraste sugiere garantías a verificar, no ausencia de esas capacidades.

## 1. Arquitectura, tecnologías, flujo y recomendaciones

TypeScript ESM, distribución tsdown, CLI, hooks, MCP, REST y viewer.
AGENTS exige iii-engine/iii-sdk como composición Worker/Function/Trigger;
el manifiesto identifica versión del paquete `0.9.32` [A0].
La documentación describe estado KV respaldado por archivo y opción Redis.
No se inspeccionó ni ejecutó el motor externo para certificar durabilidad.

Captura real leída [A1]:

1. Si `eventId` cliente es válido lo conserva; de lo contrario genera ID servidor.
2. `captureKey` deriva hash de proyecto, sesión y evento con separadores NUL.
3. Dentro de lock por key consulta evento completado/tombstone e inbox existente;
   reenvíos conocidos retornan estado duplicate, no una nueva observación.
4. Rechaza inbox lleno, elimina campos de control suministrados en payload,
   aplica `scrubRecord` a datos y escribe registro pending antes de procesar.
5. `attempt` llama `mem::observe` con observation ID estable. Si obtiene ID,
   registra completed y borra inbox; duplicación puede terminar sin nuevo ID.
6. Excepción transitoria produce retrying/backoff hasta límite; una respuesta
   sin observación ni dedup se trata como fallo permanente/dead.

`CaptureResult` diferencia accepted/completed, accepted/retrying,
duplicate y rejected. «Aceptado» **no equivale** automáticamente a «derivado
persistido y durable en disco». Se leyó orden de escrituras KV, no commit
atómico entre todas ellas ni el cuerpo completo de `observe`/spool.

`search.ts:1-130` acredita índices keyword/vector, hook de ranking híbrido,
estados de rebuild y fallback cuando falla embedding [A4].
No se revisó íntegra la autorización/expansión del grafo ni la consolidación;
no inferir garantías de todas las rutas por esos imports y estados.

Coordinación [A2]: `lease-acquire` valida acción/agent, limita TTL,
rechaza acciones terminadas/bloqueadas y busca lease activo. El mismo agente
puede recibir el existente sin renovarlo. Publica lease, auditoría y actualización
de acción como operaciones separadas. Release comprueba owner/expiración;
renew y cleanup tienen rutas propias. El mutex usa `Map`/promesas y
`AsyncLocalStorage`, por tanto es una coordinación **del proceso** [A3].

**Recomendación:** adoptar vocabulario preciso de aceptación/visibilidad y
garantías de ownership de jobs donde hagan falta. No equiparar un mutex local
a lease distribuido ni introducir coordinación distribuida si Vanta mantiene
un solo escritor y su scheduler ya satisface el requisito.

## 2. Unidades mínimas candidatas para Rust

### Unidad A — recibo idempotente con recuperación y tombstone

- **Problema:** reenviar un hook tras timeout no debe duplicar evidencia ni
  resucitar una observación borrada; debe poder saberse qué fue aceptado.
- **Unidad fuente:** `captureKey`, DTOs y protocolo inbox/completed [A1].
- **Seam Vanta:** `vanta-memory/src/core/conversation/l0_recorder.rs`,
  `vanta-memory/src/ingest/merge.rs`, checkpoints/scheduler existentes.
- **DTO HIPOTÉTICO Rust:** `CaptureEnvelope { scope, source_id, session_id,
  event_id, source_revision, payload }`; `ReceiptState` distingue queued,
  processing, committed, retryable, dead y deleted. No son tipos actuales.
- **Interfaz HIPOTÉTICA:** `accept(envelope) -> Result<CaptureReceipt,
  CaptureError>`; consultar recibo es una operación separada del renderizado.
- **Acciones concretas:** inventariar idempotencia presente; acordar ID estable
  con adaptador de origen; registrar recibo y evidencia mediante mecanismos
  transaccionales ya disponibles; publicar checkpoints después; conservar
  tombstone según retención; reintentar por ID con errores tipados.
- **Dependencias/relaciones:** fuente/provenance y dedup existentes;
  políticas de borrado y presión de cola; scheduler procesa y expone estado,
  no decide identidad de eventos. La Unidad B solo coordina al consumidor.
- **FMEA:** ID servidor nuevo en cada reenvío rompe idempotencia;
  write completed sin evidencia produce falso positivo; recaptura después
  de borrado resucita datos; inbox lleno pierde evento si cliente no reintenta;
  scrub incompleto deja secretos en payload/errores.
- **Aceptación funcional propuesta:** mismo scope/evento converge al mismo
  recibo; scopes distintos no colisionan; interrupción permite recuperación;
  tombstone impide resurrección; rejected/retrying nunca aparece como commit
  durable; falla de derivados no destruye la evidencia autorizada.
- **Fuera de unidad:** no prometer exactamente-una-vez para efectos externos
  ni portar spool, codecs, engine y hooks como bloque.

### Unidad B — ownership de trabajo con fencing explícito

- **Problema:** dos agentes o workers pueden intentar procesar o completar
  una misma acción, especialmente después de expiración y reasignación.
- **Fuente mínima:** acquire/release/renew de `leases.ts` [A2] y alcance
  in-process de `withKeyedLock` [A3].
- **Seam Vanta:** scheduler/checkpoints de `vanta-memory`; si una tarea exige
  varios procesos, ubicar coordinación en aplicación/adaptador de jobs,
  no en BM25/HNSW ni como dependencia obligatoria del motor embebido.
- **DTO HIPOTÉTICO:** `JobLease { job_id, owner, generation, expires_at }`,
  `CompleteJob { lease_generation, expected_revision, outcome }`.
  `generation` es fencing candidato; no se afirma que upstream ya lo tenga.
- **Interfaz HIPOTÉTICA:** `claim(request) -> Result<JobLease, ClaimError>`
  y `complete(request) -> Result<Checkpoint, ClaimError>`.
- **Acciones:** definir si realmente hay multiworker; mapear lock actual;
  adquirir y actualizar estado de job con revisión; rechazar generación vieja
  al publicar; renovar con owner; recuperar expirados y auditar transiciones.
- **Dependencias:** reloj/expiración documentados, operación condicional del
  backend o serialización de escritor, identidad de actor y recibos de A.
  El token no sustituye autorización ni cancela una llamada LLM ya enviada.
- **FMEA:** worker antiguo termina tras expirar; reloj salta;
  mutex local se usa entre procesos; audit falla después de lease write;
  barrido marca expirado un lease recién renovado.
- **Aceptación propuesta:** solo la generación vigente publica resultado;
  owner ajeno no renueva/libera; retry no duplica commit;
  expiración permite reasignar sin aceptar resultado viejo; fallos se
  distinguen de trabajo realizado. Validación futura, no benchmark diseñado.
- **No objetivo:** importar iii-engine, añadir Redis sin requisito o copiar
  `kv.list` completo de leases como estrategia de escalabilidad.

## 3. Defectos, riesgos e issues: código frente a hipótesis

- **Código:** el lock de leases es memoria local [A3]. **Riesgo condicionado:**
  dos procesos con mismo KV no quedan serializados por ese helper. No se
  comprobó si el despliegue permitido habilita tal configuración.
- **Código:** lease, audit y action se escriben secuencialmente ([A2], líneas 70-85).
  Hay frontera multiescritura; **hipótesis de fallo**: interrupción entre ellas
  puede dejar estado/auditoría desalineados. No se auditó transacción interna KV.
- **Código:** si el cliente no envía ID válido, captura genera nuevo evento;
  idempotencia de reenvío no queda acreditada en esa ruta ([A1], líneas 298-302).
  No equivale a bug si el contrato cliente requiere ID estable.
- **Código:** `storedObservation` puede devolver `undefined` ante error de
  lectura y la comprobación `stored !== null` trata ese caso como duplicate
  ([A1], líneas 243-249,311-316). Riesgo de confirmar duplicación sin verificar
  existencia, no pérdida durable demostrada.
- **Código del runner histórico:** captura errores de embedding con catch
  vacío, y su rama no-BM25 llama `HybridSearch` aun cuando etiqueta modo vector
  ([A5], líneas 171-200). No usar sus etiquetas para inferir comparación vector-only.
- **Issues:** cuerpos públicos y PRs sí leídos en la reanudación; registro
  fechado abajo. No se presenta un reporte histórico como defecto vigente.

## 4. Prerrequisitos Vanta

Revalidar garantías de writer/checkpoint e idempotencia actuales antes de
crear más tablas/colas. Definir durabilidad del ack, retención de recibos,
eventos borrados, backpressure y permisos de actor.
Separar clases de error del proveedor de errores permanentes de validación.
Diseñar multiworker solo si existe necesidad; ningún lock debe cruzar
`await` de forma bloqueante ni convertir un LLM en parte de una transacción larga.

## 5. Qué no trasladar

No portar Worker/Function/Trigger, catálogos MCP/REST ni hooks por cantidad.
No añadir una base KV paralela al motor Vanta.
No tratar fingerprint dedup como sustituto de IDs de origen.
No asumir que «accepted» significa fsync o que TTL solo evita commits tardíos.
No importar prompts/evaluadores ni pesos de híbrido del benchmark.

## 6. Método, licencia y límites

Apache-2.0 en snapshot, manifiesto y comienzo de LICENSE local [A0].
Antes de reutilizar código revisar archivo, notices y dependencias del motor
externo/modelos; traducir TS a Rust no garantiza independencia.
Clon superficial, una rama, sin historial/tags completos ni ejecución.
Muestra no exhaustiva: no auditoría de auth de todas las rutas, privacidad,
spool completo, consolidación, borrado transitivo o recuperación iii-engine.

## 7. Runners upstream leídos, métricas y efectos

**Dos superficies distintas, no una sola cifra de producto:**

1. `benchmark/longmemeval-bench.ts:1-317` [A5]: carga
   `longmemeval_s_cleaned.json`, excluye cuatro tipos de abstención,
   concatena turnos con rol por sesión y crea observaciones sin extracción.
   Métricas: indicador de algún gold en top-5/10/20, nDCG@10 y MRR;
   no respuesta generada ni juez. Usa `MockKV`, índice nuevo por pregunta.
   BM25 es default; el otro camino usa provider local con
   `Xenova/all-MiniLM-L6-v2`, mean pooling, normalización y q8 [A6].
   Puede descargar modelo y requiere dependencia opcional Transformers.
   Embebe solo primeros 512 caracteres de sesión y silencia errores.
   Escribe JSON de resultados en `benchmark/data`, no valida el ciclo durable
   de captura ni la calidad del producto completo.
2. `eval/runner/longmemeval.ts:1-17`, `run.ts:1-114` y `score.ts:1-89` [A7][A8][A9]:
   runner más explícito de **retrieval por sesión**, no QA. Recibe dataset,
   llama loader/adaptadores, permite límite/estratificación ya definidos
   upstream y registra fingerprint, digest y IDs seleccionados.
   `runEvaluation` valida respuestas de adaptadores, conserva errores de
   init/query/teardown y separa ingestión de latencia de consulta.
   El score deduplica session IDs, calcula precision/recall/recallAll y hit;
   preguntas unanswerable no reciben score de respuesta.

El segundo escribe manifest, evidence.ndjson, scores y summary; rechaza
directorio de salida no vacío, llama teardown si init terminó. Implementaciones
de adaptadores/CLI/loader no se leyeron completas: dependencias reales,
limpieza si init falla y llamadas externas dependen del adaptador seleccionado.
No extrapolar que todo es offline ni que cada adapter es equivalente.
`strictSuccessOverAttempts` mantiene fallos en denominador de answerable;
medias de calidad condicionadas a completados deben leerse junto con fallos.

No se alteraron prompts, judges, umbrales, muestreo ni configuración.
No comparar el runner histórico y el nuevo como si sus denominadores fueran
idénticos. Futuras comparaciones: [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Capacidad | Estado usado | Diferencia investigada |
|---|---|---|
| BM25/HNSW/RRF, dedup/provenance | Existe | no portar los índices JS |
| Captura/ingestors | Existe | semántica del recibo por evento |
| Scheduler/checkpoints | Existe | ownership/reintento y publicación |
| Assembly/compaction | Existe | no sustituir por hooks de contexto |
| Recuperación de jobs | Parcial para este contraste | piezas existentes; contrato de fencing no certificado |
| Recibos/tombstones por evento en toda ruta | No acreditado | verificación actual necesaria |
| iii-engine como runtime | No aplicable | dependencia upstream, no requisito Rust |
| Calidad de respuesta / liderazgo de benchmarks | No acreditado | runners leídos son retrieval |

## 9. Deuda sugerida sin inventar bugs Vanta

Preguntar qué confirma cada ack; si un reenvío tras borrado resucita evidencia;
si un worker expirado puede publicar; si errores de captura/cleanup se
contabilizan; y si el backlog ya cubre estas garantías.
No se declara ningún fallo actual desde otro HEAD; defectos históricos
del estudio base no son evidencia del estado presente.

## 10. UI/escritorio y Studio

**Código:** `src/viewer/server.ts:1-135` implementa infraestructura HTTP de
viewer, favicon y controles de host/origin; default de bind loopback y
comentarios sobre proxy privilegiado [A10]. Es superficie real, no solo README.
Se localizó plantilla HTML, pero no se auditó toda UI ni todas las rutas proxy.
El texto de mitigaciones no acredita cobertura de cada endpoint.
No se acredita una app desktop nativa propia en la muestra.

Studio Tauri/React ya existe en `develop-desktop`: encajan estados de captura,
dead-letter/pendientes y ownership dentro de actividad/diagnóstico existentes.
No se propone un segundo servidor viewer ni trasladar su HTML.

### Ledger de lectura efectiva

| Ref. | Archivo/rango | Cobertura |
|---|---|---|
| A0 | `package.json:1-90`, `LICENSE:1-17` | stack/licencia |
| A1 | `src/functions/capture.ts:1-175,226-356` | contrato/clave/inbox/attempt |
| A2 | `src/functions/leases.ts:1-210` | acquire/release/renew y comienzo de cleanup |
| A3 | `src/state/keyed-mutex.ts:1-67` | alcance del lock |
| A4 | `src/functions/search.ts:1-130` | estados y ranking |
| A5 | `benchmark/longmemeval-bench.ts:1-317` | runner histórico |
| A6 | `src/providers/embedding/local.ts:1-47` | modelo/dependencia |
| A7 | `eval/runner/longmemeval.ts:1-17` | entrada actual |
| A8 | `eval/runner/run.ts:1-114` | trazabilidad y errores |
| A9 | `eval/runner/score.ts:1-89` | métricas/denominadores |
| A10 | `src/viewer/server.ts:1-135` | frontera viewer |

AGENTS leído; búsqueda de firmas no se cuenta como cuerpo auditado.
Pendientes: durabilidad real, multiescritor, privacy enforcement, UI completa
y ejecución de runners/adaptadores. No scripts de proyectos, tests/builds,
apps/installs, servicios, benchmarks, validadores, commits ni push.

[A0]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/package.json#L1-L90
[A1]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/functions/capture.ts#L1-L356
[A2]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/functions/leases.ts#L1-L210
[A3]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/state/keyed-mutex.ts#L1-L67
[A4]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/functions/search.ts#L1-L130
[A5]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/benchmark/longmemeval-bench.ts#L1-L317
[A6]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/providers/embedding/local.ts#L1-L47
[A7]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/longmemeval.ts#L1-L17
[A8]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/run.ts#L1-L114
[A9]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/score.ts#L1-L89
[A10]: https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/viewer/server.ts#L1-L135

## Complemento de autoría y cierre

### Modelo, API y tres flujos completos

Fuente [types](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/types.ts#L1-L129):
`Session` pertenece a proyecto/agent, `RawObservation` conserva evento/captureKey,
`CompressedObservation` distingue source y estado/revisión de compresión;
`Origin` representa frontera user/agent/tool/import/shared, no autenticación.
No confundir una observación capturada con `Memory` versionada mediante
`parentId/supersedes/isLatest`.

```text
hook stdin → filtro/scrub → HTTP → capture(inbox por key) → mem::observe
  → evidencia/compression → índices BM25/vector → completed → retirar inbox
MCP/REST search → validación layer/scope → ranking → enriquecer → recall format
evolve/forget → KV primario → índices/refs/facets/audit → resultado parcial
viewer HTML → api() con bearer → proxy local → mismo REST, no motor adicional
```

**Ingesta:** estados y orden en [A1]; no hay una sola transacción que abarque
inbox/observation/completed/índices. Retry mantiene observation ID; conservar
tombstone impide reenvío conocido, pero su retención finita limita la garantía.
Cancelar HTTP no prueba cancelar trabajo iii ya disparado. Ningún ack se
describe como fsync del backend externo.

**Recall:** `rankMemoryIds:116-150` [A4] sobrerrecupera, combina ranks y
degrada keyword si embedding falla; `rebuildPromise:46-52` comparte reconstrucción
cold en proceso. Flags distinguen ready/incomplete/pending/backfill; no
certifican frescura de todo índice. REST `api::search:855-934` valida query,
limit y targetLayer; scope y bearer no son conceptos intercambiables.

**Update/delete:** [evolve](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/functions/relations.ts#L146-L270)
adquiere `mem:remember`, rechaza no-latest/unchanged; marca anterior no-latest,
lo desindexa, escribe sucesor, indexa y crea relación; cascade-update es
fire-and-forget con catch. Fallar entre escrituras **puede** dejar transición
parcial: no se afirma atomicidad ni reproducción. [forget](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/functions/remember.ts#L266-L440)
marca evento deleted **antes** de borrar observación; retira índices y refs,
separa `failures`/`cleanupFailures`/notFound, usa lock por sesión para observaciones.
[markCaptureEventDeleted](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/capture/event-record.ts#L1-L55)
es idempotente sobre marker ya deleted, no un GC físico de backups.
La ruta [REST forget](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/triggers/api.ts#L1689-L1713)
comprueba auth e identidad requerida; status 200 no significa todas las
limpiezas completas, hay que leer el body.

### Completar A/B sin reemplazar contratos válidos

**A, campos complementarios.** Delta: `WriteReceipt` **ya existe** en
[attestation](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L657-L735):
binding/integridad SHA-256, **no firma**, y no es recibo de entrega del hook.
`MutationReceipt` C13 sigue **propuesta**, no nuevo nombre de capacidad actual.
Alternativas: extender adaptador de ingesta y correlacionar WriteReceipt es
preferible a otro ledger/WAL; solo fingerprint pierde identidad de evento.
Errores propuestos `CaptureError::{InvalidId,Unauthorized,Full,Storage,Cancelled}`;
`CaptureReceipt` devuelve ID/revisión y fase durable/derivada separadas.
Invariantes: misma identidad+revisión converge; identidad reutilizada con otro
payload se rechaza, no «duplicate» silencioso; no aceptar tombstone como vacío.
Seam real [L0Recorder::record_turn:274-376](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L258-L376),
`ingest/merge.rs::commit:239` y checkpoints existentes. Pasos: primero acordar
ack/replay y comparar receipts, después metadata aditiva y reconciliación,
sin reindexar contenido idéntico; rollback desactiva productor nuevo, conserva
markers hasta política aprobada. FMEA: caída tras primario y antes de completed
→reenvío→duplicado; detectar correlación ID/revisión, consultar primario antes de
retry. Fallo KV read no puede clasificarse como «ausente». Cancelación previa
al commit no publica; posterior retorna estado consultable.
Aceptación normal/adversa de A se amplía con conflicto de revisión, inbox lleno,
replay tras borrado y reinicio entre etapas. Métricas por fase/error/cola,
nunca payload/credenciales. Mínimo adapter; no exactamente-una-vez externo;
abierto duración de tombstone y frontera durable del SDK.

**B, campos complementarios.** Delta a `TaskCheckpointManager` existente, no
otro scheduler: [begin/advance/complete/fail:129-188](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L129-L188).
Alternativa elegida: serialización local existente para un escritor; fencing
solo si requisito multiworker acreditado. `ClaimError` propuesto distingue
`Busy,Expired,NotOwner,StaleGeneration,Storage,Cancelled`; entrada claim lleva
job/actor/revisión y complete generación+expected_revision, salida checkpoint.
TTL nunca autoriza commit de un owner viejo. Migración opcional de generation
versionada; jobs legados se drenan/pausan antes de activar, rollback no devuelve
generaciones antiguas al pool ni reindexa memoria. FMEA reloj salta→expiración
incorrecta→dos trabajadores; detectar renovaciones/conflictos, revalidar owner
y generación en la misma publicación. Crash entre lease/audit exige reconciliar,
no repetir efectos LLM a ciegas. Aceptación: expirado se reasigna, antiguo se
rechaza; owner ajeno no renueva; cancelar libera localmente y resultado tardío
no completa checkpoint. Contadores claim/renew/stale/expired sin actor/payload
por defecto. Abierto backend condicional; no importar Redis ni prometer lease
distribuido desde `withKeyedLock` [A3].

### C — proyección vigente y resultado de invalidación

1. Problema: un agente no debe usar una política eliminada porque quedó en
   vector/BM25 o recibió successor incompleto.
2. Unidad fuente: `mem::evolve`/`mem::forget` arriba y estados rebuild [A4];
   son fronteras primario/proyección, no otro algoritmo de retrieval.
3. Delta: Vanta ya tiene supersession/dedup e índices; interesa diagnóstico
   por superficie y exclusión de versión vieja, no reconstruir índices TS.
4. Alternativas: restart como reparación es opaco; elegir ampliar lifecycle y
   resultado existente, no una transacción externa de índices duplicada.
5. Contrato propuesto `invalidate_projection(scope: &RecallScope, id: &str,
   expected_version: u64) -> Result<InvalidationOutcome, InvalidationError>`;
   salida primario/tombstone/proyecciones pendientes, errores stale/denied/
   storage/cancelled; ID eliminado no reaparece como evidencia vigente.
6. Seam [lifecycle](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/lifecycle.rs#L1-L100)
   y `l1_writer::persist_planned:289`, `apply_dedup_batch:389`, no SDK ficticio.
7. Pasos: inventariar cascadas, aplicar primario condicionado, registrar
   superficies pendientes en checkpoint, reparar idempotentemente, exponer
   resultado. Metadata aditiva; rollback conserva tombstone y bloquea stale,
   no restaura datos borrados; reindexación solo si proyección lo requiere.
8. Dependencias: A correlación/tombstone, B ownership si aplica; provenance
   determina derivados, autorización antecede toda limpieza.
9. FMEA: índice antiguo→hit obsoleto ocupa top-k→evidencia vigente desplazada;
   detectar enriquecimiento contra versión primaria y métricas stale, mitigar
   filtro/backfill por ruta real, sin afirmar nuevo peso de ranking.
10. Aceptación futura: normal update muestra sucesor; adverso caída entre
    primario/index y rebuild concurrente no resucita predecessor; cancelación
    conserva estado pendiente, observabilidad solo estados/cantidades.
11. Mínimo invalidación rastreable; no promesa de borrado de backups ni nuevo
    repair daemon; abierto cobertura efectiva por cada transporte Vanta.

### D — redacción antes de truncamiento y persistencia

1. Problema: captura tool output puede guardar fragmentos de credencial que
   dejan de coincidir con regex tras truncar.
2. Fuente: [stripPrivateData/scrubRecord:53-88](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/functions/privacy.ts#L3-L88)
   y [truncateCaptureOutput:63-87](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/hooks/_capture-filter.ts#L43-L95).
   PR #1701 posterior muestra por qué no atribuir su fix al clon.
3. Delta: provenance/taint/injection de Vanta no equivalen a remover secretos
   antes de disco o de enviar a proveedor.
4. Alternativas: regex final es tardía; elegir normalización/redacción al
   borde, no reemplazar ACL ni prometer DLP universal.
5. Contrato propuesto `prepare_capture(input: CaptureEnvelope, policy:
   CapturePrivacyPolicy) -> Result<SanitizedCapture, PrivacyError>`; output
   redactado+counts/truncated, errores invalid/unsupported/limit; ninguna
   persistencia/telemetría del original no autorizado.
6. Seam `L0Recorder::record_turn` y adapter de ingesta de A, antes de commit
   y del proveedor; `InjectionPolicy` sigue gobernando salida.
7. Pasos: delimitar tipos JSON/texto, aplicar política antes de truncar, medir
   bytes y conservar razón; versión de política en metadata. No reindexar
   secretos viejos automáticamente; remediación requiere política separada.
   Rollback fail-closed para captura riesgosa, no restaurar originales.
8. Dependencias: A recibe solo sanitizado; C limpia derivados si hay remediación.
9. FMEA: truncar primero→token parcial ya irreconocible→fuga; detectar casos
   compuestos/estructurados, mitigar redacción de valores por clave antes del cut.
10. Aceptación futura: normal conserva texto útil; adverso private anidado,
    Unicode y claves compuestas, errores no incluyen original; cancelación
    antes de write deja cero payload nuevo; observabilidad solo conteos.
11. Mínimo política opt-in de adapter; no clasificador semántico obligatorio;
    abierto exhaustividad de patrones y compatibilidad de payload opaco.

### Incidencias públicas: reporte, propuesta y merge no son lo mismo

Cuerpos leídos por API REST pública keyless el **2026-11-07**. Router cargado;
websearch funcionó en esta sesión; REST permitió leer sin el fallo MCP histórico.

| URL / fecha / estado observado | Revisión y mecanismo reportado | Corrección y aplicabilidad |
|---|---|---|
| [Issue #1273](https://github.com/rohitg00/agentmemory/issues/1273), 2026-08-28, open | v0.9.29: governance find devuelve una observación, delete devuelve deleted:0; memoria errónea permanece en recall | Causa/fix no acreditados por body. `mem::forget` actual sí tiene camino de observation/tombstone; no prueba que governance use ese camino. Motiva C, no bug Vanta |
| [Issue #1603](https://github.com/rohitg00/agentmemory/issues/1603), 2026-10-22, open | v0.9.27–0.9.30 npm: compresión por observación y retry amplifican llamadas en gateway; múltiples workers con entorno viejo complican diagnóstico | Autor mitiga desactivando auto-compress; no fix upstream acreditado ni cifras verificadas. Reutilizar scheduler/admisión Vanta, no agregar cola por analogía |
| [PR #1704](https://github.com/rohitg00/agentmemory/pull/1704), 2026-11-07, open, merged_at null | Propone unindex de import-replace/consolidation/auto-forget/heal y guards isLatest para ghost hits | Propuesta leída, **no merge**; merge_commit_sha de PR abierta no acredita fix. C cubre frontera, no se atribuye totalidad al snapshot |
| [PR #1699](https://github.com/rohitg00/agentmemory/pull/1699), 2026-11-06, closed, merged_at null | Propone diagnóstico/heal de facets huérfanos, dry-run y recheck con lock local | Cerrada sin merge acreditado; no declarar «resuelto». Facilita criterio de no borrar target no verificable |
| [PR #1701](https://github.com/rohitg00/agentmemory/pull/1701), creada 2026-11-07, merged 15:53:22Z | Claves compuestas de secreto no reconocidas antes de truncar | API acredita merge `50a55014c8d807b1f88891eafcd23ebaa3aeb9ea`; no presencia en clon `4f758eff…` ni versión publicada. D aplica antes de cualquier proveedor |

### Runners: completar defaults, adapter y efectos

El [CLI real](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/cli.ts#L1-L33)
elige **grep,bm25**, k=5; no agentmemory/vector por defecto. No hay paralelismo
por preguntas en `runEvaluation` [A8]: loops adapter→question, init/query/teardown.
[Adapter agentmemory](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/adapters/agentmemory.ts#L12-L56)
guarda una fact por sesión vía `/remember`, scope aleatorio por caso; `/search`
pide hasta `min(100,k*3)`, transforma memory ID a source session y rechaza
fuentes extrañas. No mide compresión/captura real ni QA.
Limpia sandbox también si init falla, detalle antes no acreditado.

[Sandbox.start:59-110](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/sandbox.ts#L59-L110)
requiere build e iii instalado/pin, crea home/state temporales, lanza daemon,
provider noop, auto-compress/consolidation/graph off; usa puertos aleatorios
y registra procesos para limpieza. **No ejecutar aquí**; no trasladar su
kill/reset a runtime compartido. [Vector adapter:9-90](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/adapters/vector.ts#L9-L90)
llama OpenAI text-embedding-3-small (constante DIM=1536), batches de 50 y
trunca sesión a 8000 chars en ese tramo; pagos y transferencia de corpus,
no benchmark local gratuito. Histórico [A5] usa otro modelo/local y otro corte.

[scoreQuestion/aggregate](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/score.ts#L1-L89):
precision, recall y recallAll se agregan sobre completadas scoreables;
`strictSuccessOverAttempts` mantiene intentadas answerable en denominador;
hit es **cantidad**, no porcentaje. Unanswerable y errores se registran aparte.
p50/p95 query y p50 ingesta solo completadas; eso exige informar fallos, no
ocultar colas/casos fallidos tras percentiles. No hay warmup acreditado en A8.
Evidencia guarda hashes de contenido, IDs/gold y errores redactados limitados;
hash no anonimiza por completo fuentes de baja entropía.

Otras superficies leídas acotadamente, no confundidas con LongMemEval:
[lifecycle.ts:1-85](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/lifecycle.ts#L1-L85)
declara 11 checks locales deterministas, reader de comando sin ejecutarlo,
noop/embeddings off; [recall-quality.ts:1-85](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/eval/runner/recall-quality.ts#L1-L85)
importa fixtures y comprueba layer/scope/budget; son pruebas de producto,
no exactitud de respuesta. [capture-costs.ts:1-85](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/benchmark/capture-costs.ts#L1-L85)
distingue perfiles fake/keyless/embed-llm; [load-100k.ts:29-85](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/benchmark/load-100k.ts#L29-L85)
declara Ns 1k/10k/100k, concurrencias 1/10/100, 200 ops/celda, seed 12648430,
autostart opt-in. Sus cuerpos restantes/denominadores completos y coding-life
no auditados: **no se deriva claim de performance de esos inventarios**.

Adaptación Vanta: integrar en ingesta/recall existentes la transformación
source-session y conservar IDs/fechas/scope, sin inventar adapter implementado;
reusar superficies `evals/eval_longmemeval.py`/`evals/eval_locomo.py` según
[METHOD](METHOD.md). No diseñar harness, fixture, prompt, rúbrica, score,
umbral ni configuración nuevos. Registrar modo real legacy/core, degradación
embedding, errores y reset en namespace desechable; rendimiento del motor
y retrieval por sesión siguen pistas distintas.

### Matriz complementaria y UI real

| Capacidad upstream | Vanta en SHA 97c9623c8ef5a70fdb51aff215bfbb3b05b15612 / fuente | Delta / decisión |
|---|---|---|
| Inbox/recibo por evento | Parcial: `L0Recorder::record_turn` citado, WriteReceipt existente | A correlación durable; no otro WAL |
| Tombstone y cleanup por superficie | Existe lifecycle/supersession; no acreditada paridad en todos los adapters | C, inspección dirigida antes de cambio |
| Lease local | Existe task checkpoint citado; fencing multiworker no acreditado | B solo ante requisito, no distributed lease inferido |
| Rebuild único y ready/incomplete | Existe búsqueda híbrida y auditoría textual [referencia Vanta](VANTA-REFERENCE.md) | Diagnóstico por proyección, no índice JS |
| Budget/formato recall | Existe `RecallConfig`, `apply_recall_budget` en [auto_recall:302-335](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L302-L335) | No equiparar chars con tokens |
| Privacy pre-capture | Parcial governance/taint; no acreditada política equivalente a D | Inventariar antes de implementar, no bug global |
| iii-engine y viewer HTML | No aplicable al runtime Rust; Studio ya existe | No portar servidor/engine |
| Estado A/C y conflicto B en Studio | Componentes existentes al pin separado; nueva proyección no acreditada | AM-1: ampliar inspectores, no atribuir jobs ni bytes dirty a HEAD |

UI web implementada, no solo plantilla localizada:
[index.html::api:2021-2075](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/viewer/index.html#L2021-L2075)
añade bearer, ante 401 muestra unlock y devuelve null para ciertos fallos;
[server:130-210](https://github.com/rohitg00/agentmemory/blob/4f758eff36c8403570e1bf2ac6d986239fcdb662/src/viewer/server.ts#L130-L210)
compara bearer, limita body y prepara proxy. Shell HTML abierto no concede
datos protegidos; tampoco acredita seguridad de todos los endpoints.
Studio leído al SHA separado `0ad38c542b39a589035b0e17fdedb4c09dca24b9`:
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:52-89`
pagina auditEvents y distingue errores/no configurado;
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409,488-550`
consulta generation log/anchors y selecciona sesión/tabs;
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228`
busca con explain y abre get/vecinos/audit. Son cuerpos releídos con
`git show` del SHA completo, no bytes dirty movidos a `desktop/...`.
Ahí encajan **como propuesta** estados A/C y conflictos B; no segundo viewer.
FMEA/aceptación: audit no configurado no es lista vacía; genlog no es recibo
de entrega ni lease vigente; una proyección A/C no se anuncia implementada
por existir el panel. Cambio de sesión/conexión debe limpiar datos y
reautorizar anchors. No hay JobChip/jobs acreditados en ese pin.
Recibo [AM-1](reviews/memory-additions.md#agentmemory): corregida identidad
de evidencia y propagada a matriz/aceptación; pendiente de closure independiente.
Fuentes Studio inmutables de esos cuerpos (relectura 2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

**Licencia/componentes:** Apache-2.0 raíz y fuente propia TS; iii-engine/
iii-sdk/helpers, Transformers/pesos Xenova, proveedor OpenAI y archivos HTML
requieren notices/términos propios según distribución. Licencia del paquete
no licencia pesos ni servicio; reescribir Rust no elimina derivación.
**Cobertura:** ledger A0–A10 más types, event-record completo, remember
266–440, relations 146–270, privacy 1–135, _capture-filter 1–95, CLI/adapter/
sandbox y rangos arriba. No KV externo completo, auth global, spool íntegro,
todos los runners ni privacidad universal. Fuente errónea `eval/adapters`
se resolvió en `eval/runner/adapters`, `_capture-filter` con guion bajo.
**Cierre de autoría:** cuatro mecanismos desarrollados, sin ejecución,
autorrevisión independiente ni aceptación. Tras superar 300 líneas, futura
separación autorizada: spec adapter A/D y lifecycle/ownership B/C; no se
crearon specs ni se tocaron contratos/índices/backlog.
