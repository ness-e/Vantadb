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
- [METHOD](METHOD.md); Vanta base `c561158`, HEAD comunicado `167435a9e`.
  Sin reauditoría Vanta, red, issues ni ejecución.

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
- **Issues:** no investigados. No se presenta como vigente una incidencia
  mencionada en comentarios o en benchmarks antiguos.

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
No se declara ningún fallo actual del HEAD `167435a9e`; defectos históricos
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
apps/installs, servicios, red, benchmarks, validadores, commits ni push.

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
