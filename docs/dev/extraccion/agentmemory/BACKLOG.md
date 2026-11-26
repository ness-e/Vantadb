---
title: "Backlog de extracción — Agentmemory"
kind: research
status: active
description: "Candidatas del informe agentmemory: recibo idempotente de captura, ownership de trabajos con fencing, invalidación de proyecciones y redacción previa a truncamiento"
tags: [extraccion, backlog, agentmemory]
---

# Backlog de extracción — Agentmemory

> **Origen:** informe `docs/dev/extraccion/agentmemory/AGENTMEMORY.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-AGM-01 | 🟠 | Recibo de captura idempotente con tombstone | `vanta-memory/src/core/conversation/l0_recorder.rs`; `vanta-memory/src/ingest/merge.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: recibo durable por evento con identidad acordada con el origen y estados queued/processing/committed/retryable/dead/deleted. Por qué: un reenvío tras timeout no debe duplicar evidencia ni resucitar una observación borrada; «aceptado» no equivale a derivado persistido. Acción mínima: inventariar idempotencia actual, correlacionar con `WriteReceipt` existente en `src/attestation.rs`, registrar recibo por mecanismos transaccionales disponibles y conservar tombstone según retención. Trazabilidad: informe §Unidad A · evidencia `src/functions/capture.ts:1-175,226-356` | X-AGM-02 coordina al consumidor; X-AGM-03 consume tombstone; converge con C13 MutationReceipt (propuesto) | Revalidar writer/checkpoint e idempotencia actuales; definir durabilidad del ack, retención de recibos, backpressure y permisos de actor |
| X-AGM-02 | 🟡 | Ownership de trabajo con fencing de generación | `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: delta sobre `TaskCheckpointManager` para claim/complete con generación y revisión esperada que rechace publicación de owners caducos. Por qué: dos workers pueden procesar la misma acción tras expiración y reasignación; un mutex local no es un lease distribuido. Acción mínima: acreditar primero si existe requisito multiworker; en tal caso añadir `generation` versionada a begin/advance/complete/fail y rechazar `StaleGeneration` al publicar. Trazabilidad: informe §Unidad B · evidencia `src/functions/leases.ts:1-210`, `src/state/keyed-mutex.ts:1-67` | X-AGM-01 aporta recibos; X-AGM-03 usa ownership si aplica | Reloj/expiración documentados; operación condicional del backend o serialización de escritor; identidad de actor; solo si hay requisito multiworker acreditado |
| X-AGM-03 | 🟠 | Invalidación de proyección con resultado observable | `vanta-memory/src/core/record/lifecycle.rs`; `vanta-memory/src/core/record/l1_writer.rs` (`persist_planned`, `apply_dedup_batch`) | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: invalidación rastreable primario→tombstone→proyecciones pendientes, con `InvalidationOutcome` que expone superficies aún no limpiadas. Por qué: un índice antiguo puede devolver una versión eliminada como hit y desplazar evidencia vigente del top-k; el successor parcial no debe ocultar estado. Acción mínima: inventariar cascadas, aplicar primario condicionado a revisión, registrar superficies pendientes en checkpoint y reparar idempotentemente. Trazabilidad: informe §C · evidencia `src/functions/relations.ts:146-270`, `src/functions/remember.ts:266-440`, `src/functions/search.ts:1-130` | X-AGM-01 (correlación/tombstone), X-AGM-02 (ownership); motivado por issues #1273 y PR #1704 del informe | X-AGM-01 para correlación y tombstone; provenance determina derivados; autorización antecede toda limpieza |
| X-AGM-04 | 🟠 | Redacción de secretos antes de truncar y persistir | `vanta-memory/src/core/conversation/l0_recorder.rs` (`L0Recorder::record_turn`); adapter de ingesta de X-AGM-01 | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: política de captura `CapturePrivacyPolicy` que normaliza y redacta en el borde, antes de truncamiento, disco y proveedor. Por qué: truncar primero puede partir una credencial compuesta y dejarla irreconocible para la regex posterior; provenance/taint de Vanta no equivalen a remover secretos antes de persistir. Acción mínima: delimitar tipos JSON/texto, aplicar política por clave antes del cut, registrar conteos y versión de política en metadata. Trazabilidad: informe §D · evidencia `src/functions/privacy.ts:53-88`, `src/hooks/_capture-filter.ts:43-95` | X-AGM-01 recibe solo sanitizado; X-AGM-03 limpia derivados si hay remediación; motivada por PR #1701 del informe | X-AGM-01 como frontera de ingesta; `InjectionPolicy` sigue gobernando salida; inventariar governance/taint existente antes de implementar |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/agentmemory.md` (§Unidades A/B y §C/D). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `4f758eff`; las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD.

### X-AGM-01 — Recibo de captura idempotente con tombstone

- **Problema:** reenviar un hook tras timeout no debe duplicar evidencia ni resucitar una observación borrada; debe poder saberse qué fue aceptado. «Aceptado» no equivale a derivado persistido y ningún ack upstream se describe como fsync del backend externo.
- **Contrato hipotético:** `CaptureEnvelope { scope, source_id, session_id, event_id, source_revision, payload }`; `ReceiptState = Queued | Processing | Committed | Retryable | Dead | Deleted`; `accept(envelope) -> Result<CaptureReceipt, CaptureError>` con consulta de recibo separada del renderizado. `CaptureError::{InvalidId, Unauthorized, Full, Storage, Cancelled}`; `CaptureReceipt` devuelve ID/revisión con fase durable y derivada separadas. Invariantes: misma identidad+revisión converge al mismo recibo; identidad reutilizada con payload distinto se rechaza (no «duplicate» silencioso); tombstone no se acepta como vacío.
- **Fuente de evidencia (clone):** `src/functions/capture.ts:1-175,226-356` (`captureKey`, DTOs, protocolo inbox/completed, generación de evento nuevo si el cliente no envía ID válido en `:298-302`); `src/capture/event-record.ts:1-55` (`markCaptureEventDeleted` idempotente, no GC físico); modelo `src/types.ts:1-129` (`RawObservation` conserva evento/captureKey; `Origin` es frontera, no autenticación).
- **Seam VantaDB:** `vanta-memory/src/core/conversation/l0_recorder.rs` (`L0Recorder::record_turn`), `vanta-memory/src/ingest/merge.rs::commit`, checkpoints/scheduler existentes; correlación con `WriteReceipt` existente en `src/attestation.rs` (binding/integridad SHA-256 — no firma ni recibo de entrega del hook).
- **Acciones:**
  1. Inventariar la idempotencia presente en writer y checkpoint actuales.
  2. Acordar primero ack/replay e ID estable de evento con el adaptador de origen; después metadata aditiva y reconciliación, sin reindexar contenido idéntico.
  3. Registrar recibo y evidencia mediante los mecanismos transaccionales ya disponibles, correlacionando con `WriteReceipt` (preferible a otro ledger/WAL).
  4. Publicar checkpoints tras el efecto confirmado; conservar tombstone según retención.
  5. Reintentar por ID con errores tipados, consultando el primario antes del retry para detectar correlación ID/revisión.
- **Dependencias:** fuente/provenance y dedup existentes; políticas de borrado y presión de cola; prerrequisito §4 del informe (durabilidad del ack, retención de recibos, eventos borrados, backpressure, permisos de actor). X-AGM-02 coordina al consumidor; X-AGM-03 consume el tombstone; converge con C13 MutationReceipt (propuesta, no capacidad actual).
- **FMEA:** ID nuevo del servidor en cada reenvío rompe idempotencia; write `completed` sin evidencia produce falso positivo; recaptura tras borrado resucita datos; inbox lleno pierde el evento si el cliente no reintenta; scrub incompleto deja secretos en payload/errores; caída tras primario y antes de `completed` → reenvío → duplicado (mitigar consultando primario antes de retry); fallo de lectura clasificado como «ausente» confirma duplicación sin verificar existencia; cancelación previa al commit no publica, posterior devuelve estado consultable.
- **Criterios de aceptación propuestos:** mismo scope/evento converge al mismo recibo; scopes distintos no colisionan; interrupción permite recuperación; tombstone impide resurrección; `rejected`/`retrying` nunca aparece como commit durable; fallo de derivados no destruye la evidencia autorizada; casos adversos cubiertos: conflicto de revisión, inbox lleno, replay tras borrado, reinicio entre etapas; métricas por fase/error/cola, nunca payload ni credenciales.
- **Criterio negativo:** un reenvío tras timeout no duplica evidencia; una recaptura sobre tombstone no resucita la observación borrada; identidad reutilizada con payload distinto se rechaza en lugar de contarse como duplicado.
- **Independencia:** implementable como delta mínimo del adapter de ingesta existente (extender y correlacionar, sin ledger nuevo); es base de X-AGM-02/X-AGM-03/X-AGM-04, que consumen su frontera.
- **No objetivos:** no prometer exactamente-una-vez para efectos externos; no portar spool, codecs, engine ni hooks como bloque; no añadir una base KV paralela al motor; no usar fingerprint dedup como sustituto de IDs de origen; rollback desactiva el productor nuevo conservando markers hasta política aprobada. Abiertos: duración del tombstone y frontera durable del SDK.

### X-AGM-02 — Ownership de trabajo con fencing de generación

- **Problema:** dos agentes o workers pueden intentar procesar o completar la misma acción, especialmente tras expiración y reasignación; un mutex local no es un lease distribuido.
- **Contrato hipotético:** `JobLease { job_id, owner, generation, expires_at }` y `CompleteJob { lease_generation, expected_revision, outcome }` (`generation` es fencing candidato — no se afirma que upstream lo tenga); `claim(request) -> Result<JobLease, ClaimError>` y `complete(request) -> Result<Checkpoint, ClaimError>` con `ClaimError::{Busy, Expired, NotOwner, StaleGeneration, Storage, Cancelled}`; el claim lleva job/actor/revisión y el complete generación+`expected_revision`. TTL nunca autoriza commit de un owner viejo.
- **Fuente de evidencia (clone):** `src/functions/leases.ts:1-210` (acquire/release/renew y comienzo de cleanup; lease, audit y action se escriben secuencialmente en `:70-85` — frontera multiescritura); `src/state/keyed-mutex.ts:1-67` (`withKeyedLock` es memoria local, alcance in-process).
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs` (`TaskCheckpointManager::begin/advance/complete/fail`) como delta, no otro scheduler; si una tarea exige varios procesos, la coordinación vive en aplicación/adaptador de jobs — no en BM25/HNSW ni como dependencia obligatoria del motor embebido.
- **Acciones:**
  1. Acreditar primero si existe requisito multiworker real; serialización local existente basta para un escritor.
  2. Mapear el lock/checkpoint actual y definir identidad de actor.
  3. Adquirir y actualizar estado del job con revisión en la misma operación.
  4. Rechazar `StaleGeneration` al publicar; renovar solo con owner vigente.
  5. Recuperar expirados, auditar transiciones y reconciliar tras crash entre lease/audit (sin repetir efectos LLM a ciegas).
- **Dependencias:** reloj/expiración documentados; operación condicional del backend o serialización de escritor; identidad de actor; recibos de X-AGM-01. El token no sustituye autorización ni cancela una llamada LLM ya enviada.
- **FMEA:** worker antiguo termina tras expirar → su publicación se rechaza por generación; reloj salta → expiración incorrecta → dos trabajadores (revalidar owner y generación en la misma publicación); mutex local usado entre procesos (prohibido como fencing); audit falla después de lease write → reconciliar, no repetir efectos; barrido marca expirado un lease recién renovado.
- **Criterios de aceptación propuestos:** solo la generación vigente publica resultado; owner ajeno no renueva ni libera; retry no duplica commit; expiración permite reasignar sin aceptar resultado viejo; cancelar libera localmente y un resultado tardío no completa el checkpoint; fallos se distinguen de trabajo realizado. Contadores claim/renew/stale/expired sin actor/payload por defecto.
- **Criterio negativo:** un owner caducado no publica resultado; un lock local nunca se presenta como lease distribuido; un resultado tardío tras cancelación no completa el trabajo.
- **Independencia:** candidata **condicional** — solo procede si hay requisito multiworker acreditado; migración opcional de `generation` versionada con jobs legados drenados/pausados antes de activar y rollback que no devuelve generaciones antiguas al pool.
- **No objetivos:** no importar iii-engine; no añadir Redis sin requisito; no copiar `kv.list` completo de leases como estrategia de escalabilidad ni prometer lease distribuido a partir de `withKeyedLock`. Abierto: backend condicional.

### X-AGM-03 — Invalidación de proyección con resultado observable

- **Problema:** un agente no debe usar una política eliminada porque quedó en vector/BM25 ni ocultar que un successor quedó incompleto; un índice antiguo puede devolver una versión eliminada como hit y desplazar evidencia vigente del top-k.
- **Contrato hipotético:** `invalidate_projection(scope: &RecallScope, id: &str, expected_version: u64) -> Result<InvalidationOutcome, InvalidationError>`; la salida expone primario/tombstone/proyecciones pendientes; errores `Stale`, `Denied`, `Storage`, `Cancelled`. El ID eliminado no reaparece como evidencia vigente.
- **Fuente de evidencia (clone):** `src/functions/relations.ts:146-270` (`mem::evolve`: marca anterior no-latest, desindexa, escribe sucesor, indexa — cascade-update fire-and-forget con catch; un fallo entre escrituras puede dejar transición parcial); `src/functions/remember.ts:266-440` (`mem::forget`: marca deleted antes de borrar, retira índices/refs, separa `failures`/`cleanupFailures`/notFound); `src/functions/search.ts:1-130` (estados rebuild ready/incomplete/pending/backfill); `src/triggers/api.ts:1689-1713` (REST forget: status 200 no significa limpiezas completas). Motivada por issue #1273 y PR #1704 (propuesta abierta, no merge — no se atribuye fix al clone).
- **Seam VantaDB:** `vanta-memory/src/core/record/lifecycle.rs`; `vanta-memory/src/core/record/l1_writer.rs` (`persist_planned`, `apply_dedup_batch`) — amplía lifecycle y resultado existentes; no es otro algoritmo de retrieval ni transacción externa de índices duplicada.
- **Acciones:**
  1. Inventariar las cascadas actuales primario→índices→derivados.
  2. Aplicar el primario condicionado a revisión esperada.
  3. Registrar superficies pendientes en el checkpoint existente.
  4. Reparar idempotentemente cada superficie declarada.
  5. Exponer `InvalidationOutcome` con lo aún no limpiado (metadata aditiva; reindexación solo si la proyección lo requiere).
- **Dependencias:** X-AGM-01 (correlación/tombstone), X-AGM-02 (ownership si aplica); provenance determina derivados; la autorización antecede toda limpieza.
- **FMEA:** índice antiguo → hit obsoleto ocupa top-k → evidencia vigente desplazada (detectar enriquecimiento contra versión primaria y métricas stale; mitigar con filtro/backfill por ruta real, sin afirmar nuevo peso de ranking); caída entre primario e índice → transición parcial (superficie pendiente registrada); rebuild concurrente no debe resucitar predecessor.
- **Criterios de aceptación propuestos:** update normal muestra el sucesor; adverso: caída entre primario/índice y rebuild concurrente no resucita el predecessor; cancelación conserva el estado pendiente; observabilidad solo de estados y cantidades.
- **Criterio negativo:** un ID eliminado nunca reaparece como hit de evidencia vigente; un successor parcial no se presenta como limpieza completa.
- **Independencia:** delta sobre supersession/dedup e índices existentes; alternativa descartada: restart como reparación (opaco) y transacción externa de índices duplicada. Requiere X-AGM-01 para correlación/tombstone.
- **No objetivos:** no prometer borrado físico de backups ni crear un repair daemon nuevo; rollback conserva tombstone y bloquea stale, no restaura datos borrados; no borrar target no verificable (lección PR #1699, cerrada sin merge). Abierta: cobertura efectiva por cada transporte Vanta.

### X-AGM-04 — Redacción de secretos antes de truncar y persistir

- **Problema:** la captura de tool output puede guardar fragmentos de credencial que dejan de coincidir con la regex después de truncar; truncar primero parte una credencial compuesta y la deja irreconocible.
- **Contrato hipotético:** `prepare_capture(input: CaptureEnvelope, policy: CapturePrivacyPolicy) -> Result<SanitizedCapture, PrivacyError>`; la salida es redactada + `counts`/`truncated`; errores `Invalid | Unsupported | Limit`; ninguna persistencia ni telemetría del original no autorizado.
- **Fuente de evidencia (clone):** `src/functions/privacy.ts:53-88` (`stripPrivateData`/`scrubRecord`); `src/hooks/_capture-filter.ts:43-95` (`truncateCaptureOutput`). PR #1701 (merge `50a55014`, no presente en el clone `4f758eff`): claves compuestas de secreto no reconocidas antes de truncar — motiva el orden redactar→truncar.
- **Seam VantaDB:** `vanta-memory/src/core/conversation/l0_recorder.rs` (`L0Recorder::record_turn`) y el adapter de ingesta de X-AGM-01, antes del commit y del proveedor; `InjectionPolicy` sigue gobernando la salida (son capas distintas).
- **Acciones:**
  1. Delimitar tipos JSON/texto cubiertos por la política.
  2. Aplicar normalización y redacción por clave **antes** del cut de truncamiento.
  3. Medir bytes y conservar la razón del truncamiento.
  4. Registrar conteos y versión de política en metadata.
  5. No reindexar secretos viejos automáticamente — la remediación de lo persistido requiere política separada; rollback fail-closed para captura riesgosa, sin restaurar originales.
- **Dependencias:** X-AGM-01 como frontera de ingesta (recibe solo sanitizado); X-AGM-03 limpia derivados si hay remediación; inventariar governance/taint existente antes de implementar (provenance/taint/injection de Vanta no equivalen a remover secretos antes de disco).
- **FMEA:** truncar primero → token parcial ya irreconocible → fuga persistida (detectar casos compuestos/estructurados; mitigar con redacción de valores por clave antes del cut); regex final tardía como única defensa (descartada); errores que incluyen el original → fuga secundaria (los errores nunca incluyen el original).
- **Criterios de aceptación propuestos:** caso normal conserva texto útil; adverso: `private` anidado, Unicode y claves compuestas se redactan antes del cut; los errores no incluyen el original; cancelación antes de write deja cero payload nuevo; observabilidad solo de conteos.
- **Criterio negativo:** ninguna credencial partida por truncamiento queda persistida irreconocible; ningún error ni telemetría expone el payload original.
- **Independencia:** política opt-in del adapter; no reemplaza ACL ni `InjectionPolicy` (que sigue gobernando la salida).
- **No objetivos:** no exigir clasificador semántico obligatorio ni prometer DLP universal; abiertos: exhaustividad de patrones y compatibilidad de payload opaco.
