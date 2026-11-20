---
title: "Backlog de extracción — Supermemory"
kind: research
status: active
description: "Candidatas A–G del informe Supermemory más la frontera condicionada de adapter SaaS: outcomes por brazo, caché revisionada, dedup condicionado por modo, readiness por proyección, olvido planeado, outcome de captura y auth tipada"
tags: [extraccion, backlog, supermemory]
---

# Backlog de extracción — Supermemory

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/supermemory.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
>
> **Nota de procedencia:** el clon fijado (`C:\Users\Eros\supermemory`,
> `supermemoryai/supermemory@16ed414e`) expone SDKs públicos, wrappers de
> integración y el servidor MCP — **no el motor SaaS privado**. Toda evidencia
> `archivo:línea` cita el clon; los comportamientos atribuidos son contratos de
> cliente observables, no acreditación de internos del servicio.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-SUP-01 | 🟠 | Outcome por brazo de recall antes del render | `vanta-memory/src/core/hooks/auto_recall.rs`; `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | **Qué:** `RecallBundle` tipado con outcome por fuente (`Ready/Empty/TimedOut/Cancelled/Failed/Denied`) conservado hasta el render, no una excepción única ni un vacío indistinguible. **Por qué:** el fan-out Supermemory usa `Promise.all` con `maxRetries:0` — un brazo caído rechaza el conjunto y descarta el éxito parcial; `skipMemoryOnError` degrada a «sin memoria» sin diagnóstico. **Acción mínima:** extender `perform_auto_recall`/`RecallResult` existentes con DTO aditivo de outcomes; deadline compartido del turno; error ≠ vacío. **Trazabilidad:** informe §Candidata A · evidencia `packages/tools/src/shared/memory-client.ts:30-74,98-199`; `packages/tools/src/vercel/middleware.ts:311-379`; `packages/tools/src/vercel/index.ts:149-180` | C4, C7; X-SUP-02, X-SUP-03 | `RecallConfig`/`assemble_with_recall` existentes; cancelación del host; revisión de fuentes disponible antes de entrega |
| X-SUP-02 | 🟠 | Caché de continuación ligada a turno y revisión | `vanta-memory/src/context_engine/engine.rs`; `spec` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | **Qué:** clave de caché de continuación con identidad de turno + revisión de fuente/generación de política (`TurnRecallKey{scope,turn_id,mode,source_generation,policy_revision}`), lookup `Hit/Miss/Stale`, invalidación explícita. **Por qué:** el `MemoryCache` observado (LRU 100, key `ns:thread:modo:mensaje`) no contiene revisión ni TTL: un borrado/revocación entre turnos puede servir memoria revocada. **Acción mínima:** inventariar cachés reales del host; utilidad de identidad/invalidación reutilizando scope y erasure existentes; error nunca se cachea como vacío. **Trazabilidad:** informe §Candidata B · evidencia `packages/tools/src/shared/cache.ts:8-34,60-79`; `packages/tools/src/vercel/middleware.ts:326-378`; `packages/tools/src/voltagent/middleware.ts:234-245,376-384` | C4, C14; X-SUP-01, X-SUP-05 | Inventario real de cachés del host; revisión invalidable; autorización en hit; X-SUP-01 para outcomes |
| X-SUP-03 | 🟡 | Dedup de presentación solo sobre fuentes seleccionadas | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** `select_visible(selected_sources, candidates) -> SelectionReport` con dedup restringido al conjunto que se inyectará, IDs/revisiones conservados y descartes con causa. **Por qué:** Supermemory dedupica static > dynamic > search, pero en modo `query` excluye las fuentes de perfil del conjunto de dedup — el comentario del clon es explícito: no se debe ocultar un resultado por duplicarlo contra un perfil que el modelo nunca verá. **Acción mínima:** extensión determinista de la selección previa al render; igualdad textual normalizada solo como fallback declarado; sin dedup semántico nuevo ni cambiar L1. **Trazabilidad:** informe §Candidata C · evidencia `packages/tools/src/tools-shared.ts:196-205,244-316`; `packages/tools/src/shared/memory-client.ts:134-182`; `packages/livekit-sdk-python/src/supermemory_livekit/utils.py:117-159` | C7; X-SUP-01, X-SUP-02 | X-SUP-01 conserva IDs por fuente; X-SUP-02 incluye modo en la clave; budget/ledger OpenViking sin duplicar |
| X-SUP-04 | 🟠 | Readiness por proyección y recibo consultable | `vanta-memory/src/services/pipeline_worker.rs`; `vanta-memory/src/core/conversation/l0_recorder.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | **Qué:** `IngestReceipt{source_id,revision,accepted,projections}` y `status(receipt,auth) -> ReceiptSnapshot` con proyección `Pending/Ready/Stale/Failed/Unsupported` por dimensión (chunks/memorias/perfil por separado). **Por qué:** el contrato Supermemory separa `system.status` de `system.dreamingStatus` y el propio esquema MCP advierte que «done» no equivale a memorias buscables; confundir documento aceptado con derivado listo produce falsos vacíos. **Acción mínima:** correlacionar `WriteReceipt` existente con eventos del worker; estado aditivo por fuente/revisión; timeout no declara fallo definitivo; durability desconocida explícita. **Trazabilidad:** informe §Candidata D · evidencia `apps/mcp/src/server/tools/output-schemas.ts:10-21`; `apps/mcp/src/server/client/index.ts:40-56,113-129`; `apps/mcp/src/server/tools/list-documents.ts:33-61`; `packages/tools/src/conversations-client.ts:119-139` | C9, C13, C14; X-SUP-06 | `WriteReceipt`/`ErasureReceipt` existentes; worker/checkpoints [V5]; contrato transversal de recibos ya en marcha (Agentmemory/OpenViking) |
| X-SUP-05 | 🟠 | Olvido exacto con plan previo autorizado y revisionado | `vanta-memory/src/utils/erasure.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | **Qué:** `plan_forget(auth,refs_or_text) -> ForgetPlan` + `apply(plan,confirmation) -> ForgetReport{removed,stale,denied,failed}`; preview no destructivo, igualdad exacta preferida, `StalePlan/PartialFailure/Denied` explícitos. **Por qué:** el clon ya discrimina: IDs directos, contenido→búsqueda de candidatos→igualdad textual exacta→forget, y jamás borra por similitud; `maxRetries:0` en `forgetByIds` reconoce que repetir una mutación comprometida es peor que un fallo transitorio. Pero preview→apply no es transacción visible. **Acción mínima:** extender erasure existente con plan+revisión; filtrar scope antes de preview; invalidar X-SUP-02/X-SUP-04; distinguir fuente/derivado/backup — sin prometer purga física remota. **Trazabilidad:** informe §Candidata E · evidencia `apps/mcp/src/server/tools/forget-memory.ts:41-150`; `apps/mcp/src/server/client/index.ts:283-306`; `packages/tools/src/tools-shared.ts:64-169`; `packages/tools/src/claude-memory.ts:547-575` | C5, C6, C13; X-SUP-02, X-SUP-04 | Erasure existente [V6]; inventario de derivados; confirmación contra revisión; autorización — el plan no sustituye permiso |
| X-SUP-06 | 🟠 | Outcome de captura separado de la respuesta | `vanta-memory/src/core/conversation/l0_recorder.rs`; `vanta-memory/src/services/pipeline_worker.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** `capture_response(auth,HostEventId,session_revision,ResponseOutcome) -> CaptureHandle` con `wait -> Captured/Deferred/Failed/CancelledWait`; respuesta completada, memoria aceptada y derivado listo como dimensiones distintas. **Por qué:** la matriz observada es desigual: Vercel dispara `saveMemoryAfterResponse` sin await; Cartesia solo trackea tasks contra GC; LiveKit/Pipecat sí mantienen cola con requeue y drain al cierre — ninguno devuelve recibo al host ni vincula flush con persistencia. **Acción mínima:** mapear a `L0Recorder::record_turn` existente con ID estable por evento; publicar recibo vía worker; `flush/status` al cerrar sesión; pendiente recuperable sin LLM bajo lock. **Trazabilidad:** informe §Candidata F · evidencia `packages/tools/src/vercel/index.ts:182-204,246-284`; `packages/tools/src/vercel/middleware.ts:151-188`; `packages/livekit-sdk-python/src/supermemory_livekit/memory.py:399-454,512-527`; `packages/pipecat-sdk-python/src/supermemory_pipecat/service.py:278-325,499-502`; `packages/cartesia-sdk-python/src/supermemory_cartesia/agent.py:239-240,486-494` | C8, C9, C13; X-SUP-04, X-SUP-02 | `L0Recorder` con cursor/replay existente [V4]; X-SUP-04 publica estado de derivado; lifecycle fiable del host o captura explícita |
| X-SUP-07 | 🟠 | Error de auth tipado: inválido vs backend caído | `spec`; transports SDK/MCP | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | **Qué:** `authorize_remote(request,credential_ref,deadline) -> Result<AuthorizedScope,AuthOutcome>` con `InvalidCredential/Denied/BackendUnavailable/Cancelled`; la indisponibilidad del backend de auth nunca se traduce a credencial inválida ni a fail-open de contenido. **Por qué:** el MCP tipifica `TransientAuthError` (timeout/red/JWKS/no-401-403), lo propaga sin destruir la credencial y lo mapea a JSON-RPC `-32001` + HTTP 503 + `Retry-After` — distinto del 401 `invalid_token` del inválido real. **Acción mínima:** revisar la traducción de errores de transporte actual; reautorizar en delivery; caché efímera por identidad+policy sin TTL importado; outcome no sensible al host. **Trazabilidad:** informe §Candidata G · evidencia `apps/mcp/src/server/auth/index.ts:68-207`; `apps/mcp/src/server/index.ts:132-154,209-224`; `apps/mcp/src/server/auth/rbac.ts:3-36`; `apps/mcp/src/server/client/index.ts:482-538` | C3, C11; X-SUP-01 | `InjectionPolicy` y transports existentes; X-SUP-01 conserva `Denied`≠`Empty`; X-SUP-02 no omite auth en hit; revisión jurídica del proveedor |
| X-SUP-08 | 🟡 | Frontera de adapter SaaS fuera del núcleo (condicionada) | `spec` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | **Qué:** boundary de adapter opcional que traduce contratos remotos (add/search/profile/forget/status) a primitivas Vanta sin introducir dependencia de red en el motor embebido. **Por qué:** el clon demuestra que la superficie portable es pequeña (un cliente HTTP por namespace + tipos de resultado); meter egress en el core crearía dependencia de versión, precio, retención y caída externa — el informe lo lista como decisión condicionada, no aprobada. **Acción mínima:** spec del contrato de adapter (qué traduce, qué no acredita: durability, índices, pipeline); sin implementación hasta caso de producto + consentimiento + contrato comercial. **Trazabilidad:** informe §1 (fila «Condicionado \| Adapter SaaS fuera del núcleo») y §5 · evidencia `apps/mcp/src/server/client/index.ts:241-280,482-538`; `packages/tools/src/tools-shared.ts:26-49` | C11; X-SUP-04, X-SUP-06, X-SUP-07 | Caso de producto aprobado; consentimiento y revisión jurídica; X-SUP-04/06/07 definen los recibos que el adapter traduciría |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/supermemory.md` (fichas A–G y §1/§5 para la frontera SaaS). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `16ed414` (`C:\Users\Eros\supermemory`); las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD.

### X-SUP-01 — Outcome por brazo de recall antes del render

- **Problema:** perfil válido y búsqueda fallida no deberían convertirse en una única excepción indiagnosticable; upstream descarta el éxito parcial con `Promise.all` + `maxRetries:0` y degrada a «sin memoria» con `skipMemoryOnError`, ocultando la causa.
- **Contrato hipotético:** `recall(ScopedTurn, Deadline, Cancel) -> Result<RecallBundle, RecallError>`; ítems `{source_key, namespace, revision, trust, category}`; outcomes por brazo `Ready | Empty | TimedOut | Cancelled | Failed | Denied`. Invariantes: scope autorizado al leer y entregar; error ≠ vacío; tainted no gana autoridad de instrucción; cancelación no publica resultado parcial como completo.
- **Fuente de evidencia (clone):** `packages/tools/src/shared/memory-client.ts:30-74,98-199` (fan-out `Promise.all` perfil+search, threshold 0.6, `.catch` único); `packages/tools/src/vercel/middleware.ts:311-379`; `packages/tools/src/vercel/index.ts:149-180` (`skipMemoryOnError` default true, timeout fijo 5s).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs` (`perform_auto_recall`/`RecallResult`/`RecallGovernance`/`InjectionPolicy`) y `vanta-memory/src/context_engine/engine.rs` (`assemble_with_recall`) — extender los existentes, conservar budget/estimador.
- **Acciones:**
  1. Extender `perform_auto_recall`/`RecallResult` con DTO aditivo de outcomes por brazo.
  2. Compartir el deadline del turno entre brazos.
  3. Renderizar tras la selección, no antes.
  4. Dual-read durante migración; no persistir respuestas remotas por defecto.
- **Dependencias:** `RecallConfig`/`assemble_with_recall` existentes; cancelación del host; revisión de fuentes disponible antes de entrega; integrar con ledger/budget OpenViking sin duplicarlos; C4/C7; X-SUP-02/X-SUP-03.
- **FMEA:** brazo lento → bloqueo → latencia (duración/outcome por brazo, deadline compartido); parcial tratado como total → falsa confianza (degradación explícita); revocación durante espera → fuga de scope (revalidar policy antes de entregar).
- **Criterios de aceptación propuestos:** ambos brazos OK, vacío real, timeout de un brazo, cancelación, revocación y budget mínimo; IDs conservados; contadores/outcomes/latencia sin query ni texto.
- **Criterio negativo:** un brazo caído nunca descarta el éxito parcial ni se presenta como «sin memoria»; `Denied` nunca se fusiona con `Empty`.
- **Independencia:** bundle tipado compatible sobre las rutas existentes; no requiere la caché ni la dedup de presentación.
- **No objetivos:** no ranking nuevo ni reimplementar el motor SaaS/BM25/HNSW/RRF. Abierto: frescura antes de reutilizar entre turnos.

### X-SUP-02 — Caché de continuación ligada a turno y revisión

- **Problema:** el tool loop repite recall; el mismo texto no identifica el mismo turno, revisión ni permiso; la LRU observada no contiene revisión ni TTL y puede servir memoria revocada.
- **Contrato hipotético:** `TurnRecallKey {principal_scope, session, turn_id, policy_revision, source_generation, mode}`; `get -> Hit<AuthorizedRefs> | Miss | Stale`; `invalidate(scope, generation)`. Invariantes: no compartir entre scopes; un hit no revoca `Denied`; una nueva revisión no queda suprimida; `RevisionUnknown` obliga a miss; un error no se cachea como vacío.
- **Fuente de evidencia (clone):** `packages/tools/src/shared/cache.ts:8-34,60-79` (LRU 100 sin TTL, clave `ns:thread:modo:mensaje`); `packages/tools/src/vercel/middleware.ts:326-378` (continuación sólo si string truthy; errores no se cachean); `packages/tools/src/voltagent/middleware.ts:234-245,376-384` (error cacheado como `""` — anti-patrón).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` antes de render/recall; cachear referencias para reautorizar, no el prompt completo.
- **Acciones:**
  1. Inventariar cachés reales del host.
  2. Utilidad de identidad/invalidación reutilizando scope y erasure existentes.
  3. Empezar efímera por sesión.
  4. Invalidar entradas viejas sin revisión.
- **Dependencias:** X-SUP-01 (outcomes por brazo); erasure/generaciones existentes; inventario real de cachés del host; revisión invalidable; autorización en cada hit. Cachear la selección no confirma entrega (distinto del ledger OpenViking).
- **FMEA:** borrado → hit stale → resurrección contextual (detectar generación, invalidar antes de delivery); mensaje repetido → colisión lógica (turn_id en la clave); stampede → trabajo duplicado (single-flight cancelable).
- **Criterios de aceptación propuestos:** dos sesiones con mismo texto, nuevo turno, revocación, borrado y cambio de modo; métricas hit/miss/stale por causa, clave opaca sin query.
- **Criterio negativo:** un hit nunca sirve memoria revocada ni omite la autorización; un error nunca se cachea como vacío.
- **Independencia:** misma sesión/turno; efímera, sin disco ni reindexado.
- **No objetivos:** no caché distribuida, TTL inventado ni caché global de strings. Abierto: elegir revisión disponible sin coste mayor que el propio recall.

### X-SUP-03 — Dedup de presentación sólo sobre fuentes seleccionadas

- **Problema:** en modo query una memoria puede desaparecer por deduplicarla contra un perfil que el modelo nunca verá: la base de dedup debe ser el conjunto seleccionado, no todo lo recuperado.
- **Contrato hipotético:** `select_visible(selected_sources, candidates) -> SelectionReport` con IDs/revisiones conservados y descartes con causa; la igualdad textual normalizada es sólo fallback declarado.
- **Fuente de evidencia (clone):** `packages/tools/src/tools-shared.ts:196-205,244-316` (`deduplicateMemoriesForMode`: static > dynamic > search; en `query` excluye fuentes de perfil del conjunto de dedup — comentario explícito); `packages/tools/src/shared/memory-client.ts:134-182` (`normalizeMemoryFact`: no dedup semántico); `packages/livekit-sdk-python/src/supermemory_livekit/utils.py:117-159` (dedup espejo).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` antes de `inject_recall_block`; no tocar `l1_dedup` ni el juicio de persistencia.
- **Acciones:**
  1. Extensión determinista de la selección previa al render sobre la ruta existente.
  2. Modo sombra que reporta descartes con causa.
  3. Habilitar por caller; sin reindexado ni migración persistente.
- **Dependencias:** X-SUP-01 conserva IDs por fuente; X-SUP-02 incluye el modo en la clave; coordinar con budget/ledger OpenViking; C7.
- **FMEA:** prioridad estática tapa versión nueva → recuerdo incorrecto (detectar revisión/conflicto); texto normalizado colapsa hechos distintos → pérdida (usar IDs).
- **Criterios de aceptación propuestos:** query conserva un hecho también presente en perfil; full evita repetición; revisiones contradictorias distinguibles; counts/razones sin contenido.
- **Criterio negativo:** ningún resultado se oculta por duplicarlo contra una fuente que no se inyectará; `AmbiguousIdentity` conserva la evidencia, nunca fusiona por minúsculas.
- **Independencia:** es presentación, no dedup semántico ni pesos nuevos.
- **No objetivos:** no nuevo `l1_dedup` ni adapter por integración que duplique política. Abierto: reemplazo frente a contradicción.

### X-SUP-04 — Readiness por proyección y recibo consultable

- **Problema:** consultar tras agregar una conversación antes de que exista memoria/perfil produce falsos vacíos; un documento aceptado no equivale a derivado listo.
- **Contrato hipotético:** `IngestReceipt {source_id, revision, accepted, durable_evidence, projections}` y `status(receipt, auth) -> ReceiptSnapshot`; cada proyección `Pending | Ready | Stale | Failed | Unsupported`; la durabilidad desconocida se declara.
- **Fuente de evidencia (clone):** `apps/mcp/src/server/tools/output-schemas.ts:10-21` (`status` vs `dreamingStatus`: «done» no equivale a memorias buscables); `apps/mcp/src/server/client/index.ts:40-56,113-129`; `apps/mcp/src/server/tools/list-documents.ts:33-61`; `packages/tools/src/conversations-client.ts:119-139` (`add` devuelve `{id, status:"queued"}`; `dreaming:"instant"`); degradación ante 402 (`packages/livekit-sdk-python/.../memory.py:485-497`).
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` y `vanta-memory/src/core/conversation/l0_recorder.rs` (V4/V5); exposición SDK/MCP y DTO Studio; `WriteReceipt`/`ErasureReceipt` existentes.
- **Acciones:**
  1. Correlacionar `WriteReceipt` con eventos del worker por fuente/revisión.
  2. Estado aditivo por proyección; históricos `Unknown`; backfill sólo verificable.
- **Dependencias:** contrato durable/atomicidad de publicación; `WriteReceipt`/`ErasureReceipt`; worker/checkpoints [V5]; contrato transversal de recibos (Agentmemory/OpenViking); C9/C13/C14; X-SUP-06.
- **FMEA:** respuesta vieja → falso ready (comparar revisión); polling ilimitado → coste infinito (cancelación/deadline del caller); error oculto → falsa salud (outcome explícito).
- **Criterios de aceptación propuestos:** pending, ready separado por proyección, extracción fallida, restart, borrado y revisión concurrente; lag/attempts/estados sin payload; sin periodicidad inventada.
- **Criterio negativo:** `done`/accepted nunca se presenta como memorias listas; un timeout nunca se declara fallo definitivo ni ready.
- **Independencia:** estado aditivo por fuente/revisión; desactivar la exposición mantiene los trabajos funcionando.
- **No objetivos:** status honesto, no exactly-once ni certificación de durability del SaaS; no scheduler paralelo ni sleep fijo.

### X-SUP-05 — Olvido exacto con plan previo autorizado y revisionado

- **Problema:** una petición de olvido no debe destruir recuerdos por mera similitud ni permitir que el modelo autorice; preview→apply no es una transacción visible si el mundo cambia entre ambas.
- **Contrato hipotético:** `plan_forget(auth, refs_or_text) -> ForgetPlan {scope, refs_and_revisions, policy_revision}` y `apply(plan, confirmation) -> ForgetReport {removed, stale, denied, failed}`; `StalePlan | PartialFailure | Denied` explícitos; ID exacto preferido; distinguir fuente/derivado/backup.
- **Fuente de evidencia (clone):** `apps/mcp/src/server/tools/forget-memory.ts:41-150` (IDs 1-100 deduplicados, o contenido→5 candidatos→igualdad textual exacta→forget; sin match devuelve candidatos con similitud% sin borrar); `apps/mcp/src/server/client/index.ts:283-306` (`forgetByIds` `maxRetries:0`); `packages/tools/src/tools-shared.ts:64-169` (preview dryRun, deleteDocument con count===1); `packages/tools/src/claude-memory.ts:547-575`.
- **Seam VantaDB:** `vanta-memory/src/utils/erasure.rs` (V6); invalidación de X-SUP-02, proyecciones X-SUP-04 y ledger; lo ya entregado al LLM no puede desentregarse.
- **Acciones:**
  1. Extender erasure con plan + revisión.
  2. Filtrar scope antes del preview.
  3. Invalidar X-SUP-02/X-SUP-04 al aplicar.
  4. Plan no destructivo primero; tombstones compatibles; sin prometer purga física remota.
- **Dependencias:** autorización/revisiones/inventario de derivados; erasure existente [V6]; confirmación contra revisión; C5/C6/C13; X-SUP-02/X-SUP-04. El plan no sustituye permiso.
- **FMEA:** cambio preview→apply → borrado de revisión nueva (detectar mismatch y replanificar); objetivo fuera de scope → daño (filtrar antes del preview); caché conserva el dato → reinyección (invalidación).
- **Criterios de aceptación propuestos:** por ID, texto no exacto, cero matches, borrado parcial, plan stale, revocación y consulta posterior; counts/IDs opacos sin texto a olvidar.
- **Criterio negativo:** nunca se borra por similitud; un plan stale nunca se aplica sobre una revisión nueva; ningún rollback resucita contenido purgado.
- **Independencia:** plan exacto gobernado sobre el erasure existente; el adapter SaaS es sólo opcional.
- **No objetivos:** no segunda política de borrado fuera del motor ni olvido físico remoto acreditado. Abiertos: confirmación y backups en spec.

### X-SUP-06 — Outcome de captura separado de la respuesta

- **Problema:** una llamada de voz o tool loop termina correctamente y el usuario cree que la preferencia quedó recordada aunque el guardado sigue pendiente o falló; ninguno de los wrappers observados devuelve un recibo al host.
- **Contrato hipotético:** `capture_response(auth, HostEventId, session_revision, ResponseOutcome) -> Result<CaptureHandle, CaptureError>` y `wait(handle, cancel) -> Captured | Deferred | Failed | CancelledWait`; errores `SourceInterrupted | QueueFull | Denied | RevisionConflict` no son éxito; no mantener mutex durante red/LLM.
- **Fuente de evidencia (clone):** `packages/tools/src/vercel/index.ts:182-204,246-284` (`saveMemoryAfterResponse` sin await); `packages/tools/src/vercel/middleware.ts:151-188`; `packages/livekit-sdk-python/.../memory.py:399-454,512-527` (cola con requeue + drain); `packages/pipecat-sdk-python/.../service.py:278-325,499-502`; `packages/cartesia-sdk-python/.../agent.py:239-240,486-494` (task sin drain); `packages/tools/src/conversations-client.ts:119-139` (conversation-id→document id, `dreaming:"instant"`, retries 0).
- **Seam VantaDB:** `vanta-memory/src/core/conversation/l0_recorder.rs` (`L0Recorder::record_turn` con cursor/replay, V4); `vanta-memory/src/services/pipeline_worker.rs` publica el recibo (V5).
- **Acciones:**
  1. Acordar ID estable con el adapter.
  2. Mapear a `record_turn` existente.
  3. Publicar el recibo vía worker.
  4. Exponer `flush/status` al cerrar sesión.
  5. Reintentar con identidad estable y revisión.
- **Dependencias:** `L0Recorder` con cursor/replay [V4]; X-SUP-04 publica estado de derivado; X-SUP-02 se invalida al publicar revisión; lifecycle fiable del host o captura explícita; C8/C9/C13.
- **FMEA:** cierre de proceso antes de la promesa → captura incierta (detectar pendiente y recuperar); stream parcial tratado como completo → dato falso (outcome de origen); full history reintentado con ID nuevo → duplicación (ID estable + revisión).
- **Criterios de aceptación propuestos:** stream normal/cancelado, error de red tras respuesta, cierre con pendientes, reenvío y revocación; outcomes/counts sin transcripción.
- **Criterio negativo:** una respuesta completada nunca se presenta como memoria capturada; cancelar la espera no borra un commit confirmado ni duplica un reintento.
- **Independencia:** confirmación honesta en un adapter; el recibo de captura no es un ledger de lo ya entregado al modelo.
- **No objetivos:** no exactly-once de efectos externos, prompts nuevos ni SDK obligatorio; no reemplazar L0 por concatenación completa. Abiertos: conservar respuesta parcial y contrato de cierre del host.

### X-SUP-07 — Error de auth tipado: inválido vs backend caído

- **Problema:** un JWKS o `/session` caído hace que el cliente descarte una credencial válida o confunda indisponibilidad con memoria vacía.
- **Contrato hipotético:** `authorize_remote(request, credential_ref, deadline) -> Result<AuthorizedScope, AuthOutcome>` con `InvalidCredential | Denied | BackendUnavailable | Cancelled`; scope con revisión de política y caducidad; nunca token en serialización pública.
- **Fuente de evidencia (clone):** `apps/mcp/src/server/auth/index.ts:68-207` (`TransientAuthError`, `transientAuthErrorFor`, validadores que re-lanzan transitorios); `apps/mcp/src/server/index.ts:132-154,209-224` (`resolveAuthUser`: invalid→401 `invalid_token` vs transient→JSON-RPC `-32001`+503+`Retry-After:5`); `apps/mcp/src/server/auth/rbac.ts:3-36`; `apps/mcp/src/server/client/index.ts:482-538` (`handleError` por status).
- **Seam VantaDB:** spec/transports SDK-MCP — extender la traducción de errores del transporte existente, no convertir `InjectionPolicy` en un servicio OAuth nuevo.
- **Acciones:**
  1. Revisar la traducción de errores de transporte actual.
  2. Resolver el principal en el transporte.
  3. Reautorizar en delivery.
  4. Caché efímera por identidad+policy sin TTL importado; outcome no sensible al host.
- **Dependencias:** `InjectionPolicy` y transports existentes; X-SUP-01 conserva `Denied`≠`Empty`; X-SUP-02 no omite auth en hit; revisión jurídica del proveedor; C3/C11.
- **FMEA:** outage → invalid_token → logout destructivo (detectar la clase y conservar la credencial); scope viejo → lectura indebida (invalidar/revalidar); error remoto con secreto → fuga (códigos/redacción).
- **Criterios de aceptación propuestos:** 401/403, timeout, JWKS inválido, issuer/audience ajeno, revocación con hit y cancelación; sin contenido en trazas.
- **Criterio negativo:** la indisponibilidad del backend de auth nunca se traduce a credencial inválida ni a fail-open de contenido.
- **Independencia:** en borde/transporte, reutilizando la auth existente.
- **No objetivos:** copiar la auth comercial, introducir OAuth al núcleo ni certificar RBAC completo; no se traslada la regex de token como verificador.

### X-SUP-08 — Frontera de adapter SaaS fuera del núcleo (condicionada)

- **Problema:** un caso de producto podría requerir traducir contratos remotos a primitivas Vanta sin introducir dependencia de red en el motor embebido; mientras no exista ese caso, sólo procede el límite de especificación.
- **Contrato hipotético:** boundary de adapter opcional que traduce `add/search/profile/forget/status` a primitivas Vanta; declara qué no acredita: durabilidad, índices ni pipeline interno del servicio.
- **Fuente de evidencia (clone):** `apps/mcp/src/server/client/index.ts:241-280,482-538` (un cliente HTTP por namespace + tipos de resultado; sólo el contrato observable del cliente); `packages/tools/src/tools-shared.ts:26-49`.
- **Seam VantaDB:** spec — superficie de adapter fuera del core; sin implementación en el motor.
- **Acciones:**
  1. Especificar el contrato de adapter: qué traduce y qué no acredita.
  2. No implementar hasta caso de producto aprobado + consentimiento + contrato comercial.
- **Dependencias:** caso de producto aprobado; consentimiento y revisión jurídica; X-SUP-04/06/07 definen los recibos que el adapter traduciría; C11.
- **FMEA:** meter egress en el core → dependencia de versión, precio, retención y caída externa; confundir el contrato observable del cliente con acreditación de internos del servicio.
- **Criterios de aceptación propuestos:** — el informe no define aceptación funcional (fila «Condicionado»; la decisión requiere caso de producto, consentimiento y contrato comercial antes de escribir código).
- **Criterio negativo:** el adapter nunca introduce dependencia de red obligatoria en el motor ni se presenta como paridad self-hosted/SaaS.
- **Independencia:** condicionada — no procede sin caso de producto, consentimiento y contrato comercial.
- **No objetivos:** internals SaaS imaginados, scores/thresholds/templates importados, olvido físico remoto ni consola comercial como dependencia.

## Notas de alcance

- **Filas = fichas A–G + frontera condicionada §1.** Las demás filas «Rechazar»
  de §1 (internals privados supuestos, segundo dashboard) no generan candidata
  por definición del informe.
- **Sin fila para benchmark (§7):** la adaptación MemoryBench reutiliza el
  protocolo de [METHOD](../agent-memory-2026-11-07/METHOD.md) y converge en C2
  (evaluadores confiables); el `clear` no-op del provider y `recallAtK=hitAtK`
  quedan documentados como advertencias, no como unidad de trabajo.
- **Sin filas para:** namespace por defecto (`sm_project_default`), escape de
  delimitadores `<user_memories>`/`readonly`, sesiones de upload de un solo uso,
  rutas `/ns/{ns}/graph` no publicadas en el SDK, filtros `isForgotten/isLatest`
  de presentación ni el encoding `cm_` de ClaudeMemoryTool — son evidencia de
  contrato ya cubierta por las fichas o decisiones de seguridad que el informe
  marca como «no trasladar» sin mecanismo propuesto nuevo.
- **Rechazos del informe (§5) no generan filas:** internals SaaS imaginados,
  scores/thresholds/templates importados, olvido físico remoto, consola comercial
  como dependencia, self-hosted/SaaS como equivalencia.
- Prefijo `X-SUP` reservado a Supermemory. Estados `⬜ Candidata` por definición:
  nada de esta campaña entra al triage sin fila propia conforme a la taxonomía
  oficial de `docs/dev/Backlog.md`.
