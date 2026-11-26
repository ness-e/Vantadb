---
title: "Backlog de extracción — OpenViking"
kind: research
status: active
description: "Candidatas F1–F10 del informe de extracción OpenViking más las unidades de escritorio separables derivadas del eje 10, con trazabilidad a la evidencia del clone b8e8895"
tags: [extraccion, backlog, openviking]
---

# Backlog de extracción — OpenViking

> **Origen:** informe `docs/dev/extraccion/OpenViking/OPENVIKING.md`
> (fichas §2 F1–F10 y §10 ampliado en segunda pasada; clone
> `C:\Users\Eros\Open Vikings\OpenViking` @ `b8e8895`).
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
> Las rutas `desktop/` se refieren al worktree `develop-desktop` de VantaDB;
> las rutas `openviking/…`/`web-studio/…` de la columna Descripción son
> evidencia del clone, no de este repo.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-OVK-01 | 🟠 | Identidad de contexto estable con navegación por subárbol y filtro por profundidad | `spec` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: `ContextId` opaco y estable desacoplado del path, con `parent`/`revision`/`scope` en el registro, `children/ancestors` paginados y pushdown de subárbol+depth sobre filtros existentes. Por qué: OV separa identidad URI de contenido y filtra por `PathScope` como AST empujado al motor — evidence para MEMG-25 (scope de consulta, no authz). Acción mínima: esquema de identidad + canonicalización en escritura/consulta + índice de prefijo. **Trazabilidad:** informe §F1 · evidencia `openviking/core/namespace.py:187-253`, `openviking/storage/scope_filter.py:36-68`, `openviking/storage/viking_vector_index_backend.py:2007`, `openviking/retrieve/hierarchical_retriever.py:204-223` | X-OVK-02/03/04/09; MEMG-25; FIND-320 | Esquema de registro; filtros del motor; spec MEMG-25 |
| X-OVK-02 | 🟠 | Derivados versionados por revisión (abstract/overview/source) con generación asíncrona | `spec` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: `Derivative {context_id, level, source_revision, generator_version, state}` publicado por CAS sobre `(context_id, level, source_revision)`; generación async con invalidación; estados Pending/Ready/Stale/Failed. Por qué: OV materializa sidecars `.abstract.md`/`.overview.md` bottom-up por directorio y los indexa por nivel — el diferencial es storage/indexación, no retrieval (el retriever es flat). Acción mínima: esquema + job de generación + publicación CAS. **Trazabilidad:** informe §F2 · evidencia `openviking/storage/queuefs/semantic_processor.py:1560-1724`, `openviking/core/context.py:34-39`, `openviking/storage/index_consistency.py:25-41` | X-OVK-01/03/06; issue #5873 | §4-A (reemplazo L1 indivisible) antes de borrados en cascada; X-OVK-01; jobs existentes |
| X-OVK-03 | 🟡 | Presupuesto de contexto con cuotas por categoría y contabilidad de descartes | `vantadb-mcp/src/context.rs`; `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: extender el ensamblado existente con `ContextBudget {total, reserved_response, category_quotas}`, redistribución de remanente y `discarded: [(item, reason)]` en el resultado. Por qué: el pipeline gather→budget→tiers→render de OV contabiliza explícitamente lo seleccionado y lo descartado; el estimator se declara, no se promete exactitud. Acción mínima: añadir cuotas/redistribución/registro de descartes al seam existente — nunca un assembler nuevo. **Trazabilidad:** informe §F3 · evidencia `openviking/retrieve/context_assembler/{pipeline,gather,budget,tiers,params,render,ledger,models,expansion,recall_preset,rewrite}.py` | X-OVK-01/02/04; solapamiento con `context_engine` en trabajo — revalidar | Estimator de tokens declarado; ítems con categoría/nivel (X-OVK-01/02 ayudan) |
| X-OVK-04 | 🟠 | Recibo de contexto entregado por sesión y revisión (ledger dedicado, actualización atómica) | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `DeliveryLedger` por sesión con entradas `{context_id, revision, state: Selected\|Rendered\|Delivered, turn}`, confirmación idempotente, ventana de turnos — estado dedicado, no el JSON global del checkpoint. Por qué: el ledger de OV es read-modify-write del JSON entero sin CAS (`load()` :98-108, `record()` :160-173) — el mecanismo exacto del lost-update #7170; además `detail=="uri"` no supprime (:126-130), distinción servido-completo vs servido-referencia que el diseño propio exige. Acción mínima: esquema de entradas + transición atómica (CAS o sección crítica compartida read+write). **Trazabilidad:** informe §F4 · evidencia `openviking/retrieve/context_assembler/ledger.py:98-108,126-133,160-173`, `openviking/session/session.py:3270-3286`, `openviking/session/session.py:1489-1498` | X-OVK-03; issue #7170; §9-B (checkpoint RMW) | §4-B solo como desbloqueo de promesas sobre estado compartido; X-OVK-01 (id+revisión); X-OVK-03 |
| X-OVK-05 | 🟡 | Operador declarativo y determinista de fusión post-`DedupDecision` | `vanta-memory/src/core/record/l1_dedup.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: enum cerrado `MergeOp = Patch\|Replace\|Sum\|Immutable\|Link` permitido por esquema, aplicado sobre campos tipados con precedencia `(authority, revision, policy, desempate)` y rechazo de mutaciones desconocidas/protegidas; guard anti-replay para `Sum`. Por qué: OV separa decidir si fusionar (LLM) de aplicar cómo (ops declarativos); `l1_dedup` ya decide — falta el álgebra de aplicación. Acción mínima: handlers por op + precedencia + rechazo explícito tras la decisión. Opcional, no Required. **Trazabilidad:** informe §F5 · evidencia `openviking/session/memory/merge_op/{base,factory,patch,patch_handler,replace,sum,link_merge,immutable}.py`, `openviking/system_assets/memory/memory_templates/builtin/*.yaml` | X-OVK-01; reformulación adversarial del "dedup ausente" | §4-A (merge = reemplazo); contrato de campos tipados por tipo de memoria |
| X-OVK-06 | 🟡 | Auditoría de consistencia registro canónico ↔ vector ↔ derivado por revisión | `src/cli_handlers/index.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: extender el checker existente a `ConsistencyExpectation {record_id, revision, expected_derivatives}` con reporte `missing\|orphan\|stale_revision\|invalid`; diagnóstico separado de reparación; reparación idempotente guardada por `current_revision`. Por qué: OV reconstruye expectativas desde el storage canónico y verifica `Eq(uri)` por nivel — eje contenido↔vector que la auditoría de text-index actual no cubre; regla adoptada: fallo de lectura ≠ ausencia (#7132). Acción mínima: expectativa por registro+revisión + auditoría diagnóstico-only. **Trazabilidad:** informe §F6 · evidencia `openviking/storage/index_consistency.py:14-50`; lección `issues/7132` | X-OVK-02 (revisiones); issues #5873/#7132 | X-OVK-02; jobs existentes; §4 si la reparación escribe L1 |
| X-OVK-07 | 🟠 | Máquina de estados observable de ingesta con reintentos acotados | `src/wiki/ingestors.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: `IngestItem {key, revision, state, timestamps, idempotency_key}` con estados `Captured→ExtractionQueued→Extracted→Indexing→Ready` + `Failed\|Retryable\|Cancelled`; ACK de captura ≠ ready; reanudación en cada frontera tras restart. Por qué: la propia UI de OV tiene que **inferir** los pasos del pipeline desde `result.queue_status` (`error_count`/`processed`) porque el backend no expone estados explícitos — síntoma directo de la carencia. Acción mínima: inventariar transiciones del scheduler existente + persistir estado + clasificar retryable/failed. **Trazabilidad:** informe §F7 · evidencia `openviking/storage/queuefs/` (add_resource/semantic/embedding/reindex/session_commit), `web-studio/src/features/workbench/tasks/lib/task-pipeline.ts:26-140`; lección `issues/7034` | X-OVK-09/14; issue #7034 | Claves deterministas existentes; persistencia de jobs |
| X-OVK-08 | 🟡 | Registro de experiencias estructuradas con estado de evidencia | `spec` | 🟠 1-2sem | 🟡 P2 | ⬜ Candidata | Qué: `Experience {situation, objective, preconditions, actions, outcome, evidence_refs, evidence_status: Claimed\|Observed\|Verified, failure_detail?, expiry?}` alimentando reflection/skills/scenes; política declarativa de promoción a skill con aprobación donde esté configurada; retrieval nunca ejecuta instrucciones. Por qué: OV extrae con ReAct y expone trayectorias/outcomes en UI (`/api/v1/agent-evolution/experiences/*`); el claim del agente debe distinguirse del resultado verificado por campo explícito. Acción mínima: esquema + ciclo de vida + ingestión con evidencia enlazada. **Trazabilidad:** informe §F8 · evidencia `openviking/session/memory/extract_loop.py`, `openviking/session/skill/dedup.py:15-48`, `web-studio/src/features/workbench/agent-experience/lib/api.ts:149-192` | X-OVK-04/07; FIND-290 (reflection); FIND-288; FIND-295 | §4 y X-OVK-04/07 para evidencia confiable; reflection/skills/scenes existentes |
| X-OVK-09 | 🟡 | Reingesta incremental por watches con manifest de revisiones | `src/wiki/ingestors.rs` | 🟠 1-2sem | 🟡 P2 | ⬜ Candidata | Qué: `Watch {root, excludes, debounce_ms, parser_version}` + `ChangeEvent {unit_id, kind: Upsert\|Rename\|Delete, ancestors, revision}` idempotentes; propagación a ancestros; `parser_version` dispara reproceso selectivo; conectores remotos separados del parsing (fetch ≠ parse). Por qué: la reingesta idempotente ya existe (claves deterministas); lo nuevo es la detección incremental con manifest y rename/delete de primera clase. Acción mínima: manifest de revisiones por unidad + debounce + eventos idempotentes. **Trazabilidad:** informe §F9 · evidencia `openviking/resource/watch_{manager,runtime,pagination,storage,store,store_file,store_mysql,types}.py`, `openviking/storage/resource_diff.py`, `openviking/service/resource_service.py` | X-OVK-01 (ancestros)/07; FIND-299; §0.7-H3 (SSRF en conectores) | X-OVK-01; X-OVK-07; §4 para escrituras seguras |
| X-OVK-10 | 🟡 | Inspector del ciclo captura→extracción→recall→inyección + onboarding de integraciones con consentimiento | `desktop/src/components/inspector/`; `desktop/src/components/activity/` | 🟠 1-2sem | 🟡 P2 | ⬜ Candidata | Qué: en Studio existente — timeline `Capture\|Extract\|Recall\|Inject` con evidencia real (read-only primero), memory diff + provenance, detección de integraciones con `IntegrationCandidate {kind, evidence, config_diff}` → preview → consentimiento → aplicación sin sobrescritura. Por qué: "instalado ≠ funcionando"; Helper documenta exactamente esa verificación de ciclo por sesión, y el patrón clave verificado en OV es **el pipeline persistiendo un artefacto de evidencia que la UI consume** (`memory_diff.json` en el propio VFS). Acción mínima: inspector read-only sobre datos reales + provenance visible. **Trazabilidad:** informe §F10 · evidencia `web-studio/src/features/workbench/sessions/components/memory-impact.tsx:52-368`, `web-studio/src/features/workbench/sessions/lib/memory-diff.ts:62-94`, `openviking/session/compressor_v3.py:1383-1403`, `web-studio/src/features/workbench/sessions/lib/api.ts:281-322`, `docs/en/agent-integrations/14-openviking-helper.md` [Claim] | X-OVK-04/07 (evidencia del ciclo); X-OVK-11/12/13/14 (descomposición) | X-OVK-04/07; API de lectura disponible; Studio existente |
| X-OVK-11 | 🟡 | Sonda de capabilities por contrato de lectura (no por reachability) | `desktop/src/` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: cada feature de Studio declara un endpoint probe y se muestra solo si la respuesta cumple el contrato de lectura (`supported\|unavailable\|forbidden\|unknown`); features de management además condicionadas al proveedor. Por qué: es la implementación práctica de "instalado ≠ funcionando" — un endpoint alcanzable no certifica que la página pueda operar. Acción mínima: tabla feature→probe + clasificador de respuesta/errores + ocultación declarativa. **Trazabilidad:** informe §10.2/§10.5 · evidencia `web-studio/src/shared/lib/studio-service.ts:40-89,97-171` | X-OVK-10 (descomposición de onboarding/inspector) | Cliente Studio existente; contratos de lectura del backend por feature |
| X-OVK-12 | 🟡 | Verificación de conexión con probe real e higiene de credenciales en Studio | `desktop/src-tauri/src/commands/connection.rs`; `desktop/src/` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: al configurar una conexión — probe de salud → modo de auth → rol → lectura de datos real antes de guardar; clasificación de credenciales (dato vs control); redacción de secretos en mensajes de error; rechazo fail-closed de rutas de control desde el plano de datos; en Tauri la clave maestra vive en el keyring del SO (no IndexedDB). Por qué: OV no acepta credenciales a ciegas y separa planos de credencial con rechazo previo al envío. Acción mínima: probe encadenado en el comando de conexión + saneado de errores + almacén seguro nativo. **Trazabilidad:** informe §10.2/§10.5 · evidencia `web-studio/src/features/connection/lib/onboarding.ts:52-56,155-346`, `web-studio/src/shared/api/ov-client/client.ts:126-146,213-266`, `web-studio/src/features/connection/lib/credential-storage.ts:1-40`, `web-studio/docs/admin-workbench-refactor.md` | X-OVK-10/11 | Comando `connection.rs` existente; keyring/capability de Tauri |
| X-OVK-13 | 🟢 | Vista de impacto por consolidación (diff add/update/delete con before/after) | `desktop/src/components/inspector/` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | Qué: por evento de consolidación/dedup, mostrar operaciones `add\|update\|delete` con `before`/`after`, tipo de memoria y enlace al registro — agrupadas por evento, no por registro suelto (el Inspector actual ya tiene diff de historial por registro; esto es por **evento de extracción**). Por qué: MemoryImpact hace auditable qué escribió la extracción en cada commit; requiere que el pipeline persista el artefacto de evidencia (registro interno, no archivo en VFS como en OV). Acción mínima: definir artefacto de diff por consolidación + vista read-only. **Trazabilidad:** informe §10.3-2/§10.5 · evidencia `web-studio/src/features/workbench/sessions/components/memory-impact.tsx:52-241,305-368`, `web-studio/src/features/workbench/sessions/lib/memory-diff.ts:62-94`, `openviking/session/compressor_v3.py:385-414,1383-1403` | X-OVK-10; converge con `historial-diff.ts` existente | Artefacto de evidencia persistido por consolidación (X-OVK-04/07 lo alimentan) |
| X-OVK-14 | 🟢 | Superficie visual del pipeline de jobs por tipo con estados reales | `desktop/src/components/activity/` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | Qué: diagrama serial/paralelo por tipo de job con estados `pending\|running\|completed\|failed` leídos del backend, no inferidos por heurística de contadores. Por qué: OV reconstruye el pipeline en UI desde `queue_status` porque el backend no expone estados (anti-patrón a evitar, no a copiar); con X-OVK-07 la vista es directa. Acción mínima: componente de pasos por `job_type` alimentado por el estado explícito del job. **Trazabilidad:** informe §10.3-4/§10.5 · evidencia `web-studio/src/features/workbench/tasks/lib/task-pipeline.ts:26-40,46-267`, `web-studio/src/features/workbench/monitoring/components/queue-status-card.tsx:18-44` | X-OVK-10; contraejemplo `web-studio/src/features/workbench/monitoring/lib/parse-status.ts:19-40` (tablas ASCII) | X-OVK-07 (estados explícitos en backend) |

## Fichas completas

> Expansión de cada candidata a partir del propio informe `OpenViking/OPENVIKING.md` (fichas F1–F10 de §2 y ampliación §10 para X-OVK-11..14). Todos los contratos son **hipotéticos** (status `proposed`): no describen APIs existentes de VantaDB y requieren spec propia antes del backlog. La evidencia `archivo:líneas` cita el clone upstream fijado en `b8e8895` (`C:\Users\Eros\Open Vikings\OpenViking`); las rutas VantaDB citan el snapshot del informe (`632f491`/`f6ed51d`) y requieren revalidación contra HEAD. AGPL/legal documentado: ningún port source-to-source. Studio ya existe en `develop-desktop`; no se crea otra consola.

### X-OVK-01 — Identidad lógica de contexto y navegación por subárbol

- **Problema:** la identidad de contexto debe ser estable e independiente del path físico, con navegación paginada por hijos/ancestros y filtrado por subárbol+profundidad; un rename/move no puede romper provenance ni la identidad.
- **Contrato hipotético:** `ContextId` opaco y estable; `ContextObject { id: ContextId, source_ref, parent: Option<ContextId>, revision, kind, scope }` mantiene revisión y relaciones **fuera** de la identidad; `resolve(id)` resuelve el registro existente sin duplicar payload; `children(id, page)`, `ancestors(id)`; canonicalización de path + metadatos de subtree/depth en el registro para filtro pushdown. Un rename/move actualiza `parent`/relaciones, no la identidad; un cambio de revisión no es un cambio de identidad.
- **Fuente de evidencia (clone):** `core/namespace.py:187-253` (scopes `resources`/`agent`/`user/{id}`/`session`/`temp`, `default_target_directories`), `storage/scope_filter.py:36-68` (AST `PathScope("uri", target, depth)` inyectado como `req.dsl`), `storage/viking_vector_index_backend.py:2007` (`visible_roots(ctx)` ACL), `retrieve/hierarchical_retriever.py:204-223`.
- **Seam VantaDB:** `spec` — namespaces planos actuales, registro de memoria/contexto existente, `FilterBitset`/filtros del motor e ingestors con claves deterministas; no cambia el modelo físico de almacenamiento.
- **Acciones:**
  1. Definir esquema de identidad y canonicalización.
  2. Persistir `parent/scope/depth` en registros.
  3. Índice de prefijo para scope-filter.
  4. Resolución id↔registro sin copia.
  5. Paginación estable.
- **Dependencias:** esquema de registro; filtros existentes; spec MEMG-25; FIND-320 (trust/scope boundary).
- **FMEA:** colisión de identidad entre tenants → id incluye scope; escape de subárbol por path no canónico → canonicalización obligatoria en escritura y consulta; rename que rompe provenance → identidad separada del path; profundidad inconsistente tras mover subárbol → recálculo o derivación al vuelo.
- **Criterios de aceptación propuestos:** colisión entre tenants, intento de escape por `..`/encoding, límites de profundidad, rename preserva provenance, paginación estable bajo inserciones concurrentes. Observabilidad: métricas de resolución id→registro (hit/miss), cardinalidad de scope, profundidad máxima.
- **Criterio negativo:** el scope nunca se presenta como autenticación ni enforcement de permisos (eso es frontera gobernada de lectura/escritura, independiente de la navegación y de todos los transports); ningún path no canónico escapa del subárbol.
- **Independencia:** base de X-OVK-02/X-OVK-09 (ancestros); no depende de otras fichas.
- **No objetivos:** scope no es auth; no cambia el modelo físico de almacenamiento.

### X-OVK-02 — Derivados versionados abstract/overview/source con generación asíncrona

- **Problema:** materializar por unidad de agrupación derivados L0-abstract/L1-overview/L2-source versionados por revisión de fuente y versión de generador, con generación asíncrona e invalidación — sin que una generación vieja sobrescriba una nueva.
- **Contrato hipotético:** `Derivative { context_id, level: Abstract|Overview|Source, source_revision, generator_version, state: Pending|Ready|Stale|Failed }`; publicación por CAS sobre `(context_id, level, source_revision)`; lectura progresiva fuente→overview→abstract.
- **Fuente de evidencia (clone):** `storage/queuefs/semantic_processor.py:1560-1724` (generación bottom-up; L0 = primera línea del L1 `:1560-1589`), `core/context.py:34-39` (`ContextLevel`), `storage/index_consistency.py:25-41` (`IndexExpectation.level` — niveles como registros separados del mismo índice).
- **Seam VantaDB:** `spec` — subsistemas existentes de wiki/scenes/jobs para generación asíncrona; registros en partición interna + `FilterBitset` por nivel; **no** replicar el lifecycle L0–L3 cognitivo (mapeo erróneo documentado).
- **Acciones:**
  1. Esquema derivado + revisiones.
  2. Job async de generación con invalidación por nueva revisión.
  3. Publicación CAS.
  4. Indexación del derivado con `level`.
  5. Lectura progresiva en consulta (fuente→overview→abstract según necesidad).
- **Dependencias:** **escrituras L1 seguras como prerequisito** antes de cualquier borrado en cascada de derivados (§4-A del informe: delete-before-put no indivisible puede perder datos); jobs existentes; X-OVK-01.
- **FMEA:** job lento deja derivados stale → marcar `Stale` por `source_revision`, nunca borrar fuente; actualización durante generación → descartar resultado viejo en CAS; fallo de generador → `Failed` reintentable, fuente intacta; borrado en cascada con L1 no atómico → prohibido hasta el prerequisito.
- **Criterios de aceptación propuestos:** generación concurrente de dos revisiones (gana la nueva); cancelación a mitad; la fuente nunca desaparece por fallo de derivado; invalidación correcta tras update; derivado huérfano detectable por auditoría (X-OVK-06). Observabilidad: estados por nivel, lag fuente→derivado listo, ratio stale, fallos por generador/versión.
- **Criterio negativo:** una generación vieja nunca publica sobre la nueva; ningún fallo de derivado borra la fuente.
- **Independencia:** adopta el esquema URI→niveles materializados, no `HierarchicalRetriever`; requiere el prerequisito L1 seguro y X-OVK-01.
- **No objetivos:** no es compresión ni summarización en hot path; el generador (LLM o no) es intercambiable; no es cascade de retrieval.

### X-OVK-03 — Presupuesto de contexto explícito (extensión del assembler existente)

- **Problema:** dotar al ensamblado de contexto existente de presupuesto total explícito, reserva de respuesta, cuotas por categoría, redistribución de remanente y contabilidad de descartes — sin crear un assembler nuevo.
- **Contrato hipotético:** `ContextBudget { total, reserved_response, category_quotas: Map<Category, usize> }`; resultado `AssembledContext { selected: Vec<Item>, discarded: Vec<(Item, DiscardReason)>, per_category_count }`; estimator de tokens **declarado** (nunca afirmar tokenización exacta); tier `explicit|detail`.
- **Fuente de evidencia (clone):** `retrieve/context_assembler/` completo: `pipeline.py`, `gather.py`, `budget.py`, `tiers.py`, `params.py`, `render.py`, `models.py`, `expansion.py`, `recall_preset.py`, `rewrite.py`, `ledger.py` — leer pipeline→gather→budget→tiers→params→render como flujo, sin portarlo mecánicamente.
- **Seam VantaDB:** `vantadb-mcp/src/context.rs` (`context_assemble`: recall + compaction + presupuesto compartido) y `vanta-memory/src/context_engine/engine.rs` (`assemble_with_recall` o equivalente — verificar nombre actual en worktree).
- **Acciones:**
  1. Leer el pipeline OV como referencia.
  2. Mapear `context.rs`/`engine.rs` actuales.
  3. Añadir presupuesto/cuotas/redistribución al flujo existente.
  4. Registrar descartes con razón.
  5. Citations/wrappers consistentes.
- **Dependencias:** estimator declarado; esquema de ítems con categoría y nivel (X-OVK-01/X-OVK-02 ayudan); ninguna dependencia de OV en runtime; revalidar solapamiento con el trabajo de `context_engine` observado en el worktree.
- **FMEA:** presupuesto diminuto → comportamiento definido con mínimos por categoría y degradación documentada; empates de score → desempate estable y determinista; Unicode/multibyte en estimación → medir en el estimador declarado, no en bytes; ítems superseded o sin permiso → excluidos antes de presupuestar; port mecánico del pipeline → prohibido.
- **Criterios de aceptación propuestos:** presupuesto exacto, presupuesto mínimo degenerado, redistribución de remanente documentada, empates estables entre runs, exclusión por auth/supersession previa al presupuesto, descartes con razón y conteo, estimator declarado en salida. Observabilidad: tokens estimados vs reservados, % de presupuesto por categoría, descartes por razón, items por tier.
- **Criterio negativo:** el estimator nunca se presenta como tokenización exacta; ningún ítem superseded o sin permiso entra en el presupuesto.
- **Independencia:** extensión del assembler existente; útil sin X-OVK-01/X-OVK-02 aunque se benefician mutuamente.
- **No objetivos:** no es compresión de prompts ni reranking; no cambia la API de recall existente.

### X-OVK-04 — Ledger de contexto entregado por sesión y revisión

- **Problema:** registrar qué contexto (id+revisión) se seleccionó, renderizó y entregó por sesión y ventana de turnos, con confirmación idempotente — evita reinyectar lo ya entregado y permite auditar la inyección; upstream perdía entregas por un RMW sin CAS ni lock (issue #7170).
- **Contrato hipotético:** `DeliveryLedger { session_id, entries: [{context_id, revision, state: Selected|Rendered|Delivered, turn}] }`; update atómico (CAS o sección crítica compartida sobre read+write); `confirm_delivery(entry)` idempotente; ventana de turnos configurable. La confirmación de entrega es semántica de aplicación: no es promesa de exactly-once de red.
- **Fuente de evidencia (clone):** `retrieve/context_assembler/ledger.py` (`_record_turn`, `_resolve_turn`); RMW verificado: `load()` lee el JSON entero (`:98-108`) y `record()` lo reescribe entero (`:160-173`) **sin CAS ni lock** — mecanismo del lost-update de #7170; degrada a ledger vacío si es ilegible (`status="new"`, `:104-107`) y a `write_failed` best-effort (`:171-173`); `detail=="uri"` (servido sin contenido) **no** suprime en turnos siguientes (`:126-130`); `session/session.py:3270-3286` (contexto para search), `session/session.py:1489-1498`.
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` — sesiones existentes y store propio con estado de ledger **dedicado por sesión** (no acoplado al JSON global del checkpoint); actualización atómica por CAS o sección crítica compartida. La carrera RMW histórica observada en `vanta-memory/src/utils/checkpoint.rs` (§4-B) debe resolverse antes de prometer garantías basadas en estado compartido.
- **Acciones:**
  1. Resolver la carrera RMW del checkpoint global (§4-B) antes de prometer garantías sobre estado compartido.
  2. Esquema de entradas por sesión+context_id+revisión en estado dedicado.
  3. Transiciones de estado atómicas.
  4. Consulta «¿qué se entregó ya en esta ventana?».
  5. Integración con el assembler (X-OVK-03).
- **Dependencias:** §4-B sólo en cuanto desbloquea promesas sobre estado compartido; el ledger usa estado dedicado por sesión, no el JSON global; X-OVK-01 (context_id+revision); X-OVK-03 (puntos de selección/render).
- **FMEA:** cancelación antes de entrega → no suprime (queda Selected/Rendered, no Delivered); dos llamadas concurrentes → ambas preservadas por la actualización atómica; nueva revisión del mismo contexto → no suprimida por entrega previa de la vieja; fallo a mitad de update → idempotencia en retry; ventana mal dimensionada → reinjección o supresión excesiva (parámetro observable); transport de confirmación → no garantiza exactly-once.
- **Criterios de aceptación propuestos:** dos recalls concurrentes preservan ambas entregas; cancel-before-delivery no suprime; nueva revisión no suprimida; confirmación repetida idempotente; ventana de turnos respeta límite; restart no pierde el ledger. Observabilidad: entregas por sesión, ratio supresión, conteo por estado, edad del ledger.
- **Criterio negativo:** `Selected`/`Rendered` nunca se reporta como `Delivered`; un ledger ilegible nunca degrada silenciosamente a vacío sin estado `new`/`write_failed` declarado.
- **Independencia:** bloqueado por §4-B (checkpoint RMW) para las garantías de estado compartido; el ledger en sí es estado dedicado por sesión.
- **No objetivos:** no es dedup semántico ni política de calidad; no persiste contenido, sólo referencias id+revisión. Lección: issue upstream #7170.

### X-OVK-05 — Merge declarativo post-`DedupDecision`

- **Problema:** aplicar la fusión una vez que `l1_dedup` ya decidió `update|merge` necesita un operador declarativo y determinista — nunca sustituir la decisión LLM.
- **Contrato hipotético:** enum cerrado `MergeOp = Patch | Replace | Sum | Immutable | Link` permitido por esquema del registro; campos tipados; precedencia explícita `(authority, revision, policy, tie_estable)`; rechazo de mutaciones desconocidas/incompatibles/protegidas; guard anti-replay para `Sum`.
- **Fuente de evidencia (clone):** `session/memory/merge_op/`: `base.py` (registry ABC), `factory.py`, `patch.py`, `patch_handler.py`, `replace.py`, `sum.py`, `link_merge.py`, `immutable.py`; plantillas por tipo en `system_assets/memory/memory_templates/builtin/*.yaml`.
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_dedup.rs` — pipeline de dos fases (keyword recall + LLM judge) que hoy aplica `update/merge` sin álgebra declarativa; el operador se inserta **después** de `DedupDecision`.
- **Acciones:**
  1. Definir el enum y el esquema por tipo de memoria.
  2. Handlers deterministas por op sobre campos tipados.
  3. Resolver precedencia y rechazo.
  4. Replay guard para `Sum`.
  5. Integrar tras la decisión LLM en `apply_dedup_batch` o equivalente.
- **Dependencias:** **reemplazo L1 seguro como prerequisito** (§4-A: delete-before-put no indivisible — un merge que elimina antes de reemplazar hereda la pérdida); contrato de campos tipados por tipo.
- **FMEA:** op desconocida o incompatible con el esquema → rechazo explícito, nunca aplicación a ciegas; replay de `Sum` → guard por idempotency key; mutación de campo protegido → rechazo; precedencia ambigua → regla declarada + desempate estable; merge sobre registro eliminado entre decisión y aplicación → recheck bajo la misma guard.
- **Criterios de aceptación propuestos:** replay de `Sum` no duplica; conflictos de precedencia resueltos por la regla declarada; campos protegidos intactos; tipos incompatibles rechazados; decisión LLM intacta (el operador sólo aplica). Observabilidad: merges por op, rechazos por razón, replays detectados, merges aplicados vs decididos.
- **Criterio negativo:** el operador nunca decide *si* fusionar (eso es el LLM/judge); ninguna mutación desconocida o protegida se aplica.
- **Independencia:** Optional, no Required; requiere el prerequisito L1 seguro (§4-A).
- **No objetivos:** no introduce plantillas YAML en VantaDB salvo que la spec lo justifique; no es dedup semántico.

### X-OVK-06 — Auditoría de consistencia registro ↔ vector ↔ derivado por revisión

- **Problema:** extender el checker existente al eje registro canónico ↔ vector ↔ revisión de derivado: detectar missing/orphan/stale/invalid por revisión — el checker actual audita postings/BM25, que no es el mismo eje.
- **Contrato hipotético:** `ConsistencyExpectation { record_id, revision, expected_derivatives }`; `AuditReport { missing, orphan, stale_revision, invalid }`; diagnóstico separado de reparación; jobs de reparación idempotentes guardados por `current_revision`.
- **Fuente de evidencia (clone):** `storage/index_consistency.py:14-50` — job que reconstruye expectativas desde storage canónico y verifica `Eq(uri)` por nivel en el índice.
- **Seam VantaDB:** `src/cli_handlers/index.rs` — `cmd_audit_index`, drift de text index, `cmd_repair_text_index`; el eje propuesto es contenido↔vector por revisión (delimitación de la revisión adversarial).
- **Acciones:**
  1. Mapear comandos actuales.
  2. Definir expectativa por registro+revisión.
  3. Auditoría diagnóstico-only.
  4. Reparación como job separado con guard de revisión.
  5. Tratar **fallo de lectura ≠ ausencia** (lección #7132).
- **Dependencias:** X-OVK-02 (revisiones de derivados); jobs existentes; §4 si la reparación escribe L1.
- **FMEA:** reparación concurrente con publicación → guard de `current_revision` (nunca reparar hacia una revisión vieja); cancelación a mitad de job → idempotente/reanudable; paginación grande → job por páginas con progreso persistido; lectura fallida reportada como ausente → error categorizado, no «missing».
- **Criterios de aceptación propuestos:** publicación concurrente durante auditoría; cancelación y reanudación por página; error de lectura clasificado distinto de ausencia; reparación no retrocede revisiones. Observabilidad: conteos por categoría (missing/orphan/stale/invalid), duración por página, reparaciones aplicadas/omitidas por guard.
- **Criterio negativo:** un fallo de lectura nunca se reporta como ausencia; el diagnóstico nunca muta; la reparación nunca retrocede a una revisión vieja.
- **Independencia:** reformulación del checker existente; útil con sólo X-OVK-02.
- **No objetivos:** no es rebuild completo ni GC de vectores.

### X-OVK-07 — Pipeline de ingesta con estados observables

- **Problema:** hacer observable el scheduler/ingesta existente con estados explícitos `Captured → ExtractionQueued → Extracted → Indexing → Ready` más `Failed | Retryable | Cancelled`, revisiones, timestamps e idempotencia; `ACK de captura ≠ ready`.
- **Contrato hipotético:** `IngestItem { key, revision, state, timestamps por transición, idempotency_key, error?, partial_result? }`; máquina de estados explícita; reintentos acotados; «lock/deferral» ≠ «ejecución fallida».
- **Fuente de evidencia (clone):** QueueFS = colas persistentes at-least-once (`storage/queuefs/`: `add_resource_msg.py`, `add_resource_processor.py`, `semantic_processor.py`, colas embedding/reindex/session_commit/external_task); jobs con ACK de captura.
- **Seam VantaDB:** `src/wiki/ingestors.rs` — scheduler/jobs existentes + ingestors wiki de claves deterministas.
- **Acciones:**
  1. Inventariar transiciones actuales del scheduler.
  2. Modelo de estados + persistencia.
  3. Reintentos acotados con clasificación retryable/failed.
  4. Reanudación en **todos** los límites tras restart.
  5. Superficie de consulta por estado.
- **Dependencias:** claves deterministas existentes; persistencia de jobs; VantaDB ya limita reintentos — mantenerlo (lección #7034).
- **FMEA:** at-least-once duplica trabajo → handlers idempotentes por `idempotency_key`; lock deferral reportado como fallo → estado `Retryable` separado; restart a mitad → reanudar en cada frontera Captured/Queued/Extracted/Indexing; retry infinito → tope explícito.
- **Criterios de aceptación propuestos:** restart en cada frontera reanuda correctamente; duplicado por reintento no duplica efecto; deferral por lock no marca `Failed`; tope de reintentos respeta límite; estado consultable en cualquier instante. Observabilidad: items por estado, latencia por transición, reintentos por clase, backlog, edad del item más viejo.
- **Criterio negativo:** un ACK de captura nunca se presenta como `Ready`; un deferral por lock nunca marca `Failed`.
- **Independencia:** sobre scheduler/jobs existentes; relacionado con watches (X-OVK-09).
- **No objetivos:** no cambia la semántica de las colas; no añade orquestador externo; no promete exactly-once.

### X-OVK-08 — Experiencias estructuradas del agente

- **Problema:** registrar experiencias (situación, objetivo, precondiciones, acciones, resultado, evidencia, fallo, expiración) que alimenten reflexión, skills y escenas — el claim del agente debe distinguirse del resultado verificado.
- **Contrato hipotético:** `Experience { situation, objective, preconditions, actions, outcome: Success|Failure|Partial, evidence_refs, evidence_status: Claimed|Observed|Verified, failure_detail?, expiry? }` — `evidence_refs` por sí solo no prueba el éxito; política **declarativa** de promoción experiencia→skill con evidencia y aprobación humana donde esté configurada (sin umbral numérico); retrieval nunca ejecuta instrucciones recuperadas.
- **Fuente de evidencia (clone):** `session/memory/extract_loop.py` (extracción ReAct, cap 32k tokens), skills persistentes y su procesamiento (`server/routers/skills.py`, `server/skill_ingest.py`, `utils/skill_processor.py`), dedup textual de session-skills (`session/skill/dedup.py:15-48`).
- **Seam VantaDB:** `spec` — reflection (FIND-290), skills, scenes/dreaming existentes; `l1_dedup` para consolidación; FIND-288, FIND-295 (hooks).
- **Acciones:**
  1. Esquema + ciclo de vida con expiración.
  2. Ingestión desde sesiones/escenas con evidencia enlazada.
  3. Política declarativa de promoción a skill con evidencia + aprobación humana configurable.
  4. Recuperación contextual.
  5. Barrido anti-secretos y anti-contaminación train/test.
- **Dependencias:** §4 y X-OVK-04/X-OVK-07 para evidencia confiable; reflection (FIND-290), hooks (FIND-295), FIND-288.
- **FMEA:** claim del agente tomado como verdad → `evidence_status` distingue afirmado/observado/verificado; experiencias fallidas descartadas → conservarlas (son útiles); instrucciones recuperadas ejecutadas automáticamente → prohibido por contrato (retrieval ≠ ejecución); secretos en experiencias → barrido previo a persistir; contaminación train/test → aislamiento por scope/expiry (riesgo real en benchmarks §7); promoción prematura a skill → política declarativa con evidencia y aprobación configurada.
- **Criterios de aceptación propuestos:** experiencia sin evidencia no marca `Verified`; experiencia fallida recuperable; instrucción recuperada nunca se ejecuta; barrido de secretos bloquea persistencia; promoción exige evidencia declarada y aprobación donde esté configurada. Observabilidad: experiencias por outcome, ratio claim/verificado, promociones, hits de retrieval.
- **Criterio negativo:** una instrucción recuperada nunca se ejecuta; ninguna experiencia se marca `Verified` sin evidencia.
- **Independencia:** conecta subsistemas existentes; depende de la evidencia confiable de X-OVK-04/X-OVK-07.
- **No objetivos:** no es replay de episodios ni RL; no ejecuta acciones; no es journaling de sesión crudo.

### X-OVK-09 — Watches incrementales de fuentes

- **Problema:** reingesta incremental por watches con manifest de revisiones, debounce, unidad de cambio + ancestros, exclusiones, permisos, rename/delete como eventos de primera clase, versión de parser y conectores remotos.
- **Contrato hipotético:** `Watch { root, excludes, debounce_ms, parser_version }`; `ChangeEvent { unit_id, kind: Upsert|Rename|Delete, ancestors, revision }`; eventos idempotentes; conector remoto separado del parsing (fetch ≠ parse).
- **Fuente de evidencia (clone):** `openviking/resource/watch_{manager,runtime,pagination,storage,store,store_file,store_mysql,types}.py`, `openviking/service/resource_service.py`, `openviking/storage/resource_diff.py`, parsers por tipo; `openviking/server/routers/resources.py` (prefijo `openviking/` abreviado en fichas `X-OVK-01..08`).
- **Seam VantaDB:** `src/wiki/ingestors.rs` — ingestors con claves deterministas (la reingesta idempotente ya existe); lo nuevo es la detección incremental por watch.
- **Acciones:**
  1. Manifest de revisiones por unidad.
  2. Debounce + coalescing de eventos.
  3. Propagación a ancestros (X-OVK-01).
  4. Rename/delete como eventos de primera clase.
  5. `parser_version` en el manifest para reprocesar al cambiar parser.
  6. Conectores remotos con validación de fuente.
- **Dependencias:** X-OVK-01 (ancestros), X-OVK-07 (estados), FIND-299 (watches ya en backlog); §4 para escrituras seguras.
- **FMEA:** tormenta de eventos → debounce + límite; auto-ingest (escribir sobre lo que el propio watch genera) → exclusión de artefactos derivados; symlink fuera del root → rechazo (lección `zip_safe`/`path_safety`); SSRF en conectores remotos → guard equivalente al TOCTOU de §0.7-H3, sin allowlist sin verificación IP; evento para revisión ya inválida → descartar por `source_revision`; permisos de origen cambian → re-evaluar scope.
- **Criterios de aceptación propuestos:** rename mantiene identidad+provenance; delete propaga; tormenta coalescida; evento idempotente ante reintento; parser nuevo dispara reproceso selectivo; symlink/SSRF rechazados. Observabilidad: eventos por tipo, debounce rate, lag watch→ready, reprocesos por `parser_version`.
- **Criterio negativo:** ningún symlink fuera del root ni SSRF sin verificación IP pasa; un evento para revisión inválida se descarta, nunca se aplica.
- **Independencia:** requiere X-OVK-01 (ancestros) y se beneficia de X-OVK-07; la reingesta idempotente ya existe.
- **No objetivos:** no es FUSE ni montaje; no es sync bidireccional; el fetch remoto no ejecuta parsers en el conector.

### X-OVK-10 — Inspector de evidencia y onboarding en Studio existente

- **Problema:** en VantaDB Studio existente (`develop-desktop`, Tauri v2): onboarding de integraciones con consentimiento + inspector del ciclo captura→extracción→recall→inyección con evidencia real; «instalado ≠ funcionando».
- **Contrato hipotético:** `IntegrationCandidate { kind, evidence, config_diff }` con detección → preview → consentimiento → aplicación sin sobrescritura; `TimelineEvent { stage: Capture|Extract|Recall|Inject, ref, ts }`; memory diff + provenance + presupuesto de descartes (X-OVK-03); sync selectivo por categoría.
- **Fuente de evidencia (clone):** `web-studio/src/features/workbench/sessions/components/memory-impact.tsx:52-368` + parser `web-studio/src/features/workbench/sessions/lib/memory-diff.ts:62-94` (MemoryImpact: diff add/update/delete con before/after por `archive_NNN`); productor del artefacto `openviking/session/compressor_v3.py:1383-1403` (`memory_diff.json` escrito en el propio VFS; leído por convención en `web-studio/src/features/workbench/sessions/lib/api.ts:281-322`); sonda de capabilities `web-studio/src/shared/lib/studio-service.ts:40-171`; onboarding con verificación real `web-studio/src/features/connection/lib/onboarding.ts:155-346`; credenciales dual fail-closed `web-studio/src/shared/api/ov-client/client.ts:213-266` + cifrado `web-studio/src/features/connection/lib/credential-storage.ts:1-40`; pipeline de tareas inferido `web-studio/src/features/workbench/tasks/lib/task-pipeline.ts:26-140` (evidencia de que el backend no expone estados explícitos → X-OVK-07); `docs/en/agent-integrations/14-openviking-helper.md` (perfiles cloud/selfhost, detección de agentes, setup MCP/hooks/plugins, inspección de sesiones, sync selectivo — **sólo documentado, no verificado en runtime**; el fuente de Helper no está en el checkout).
- **Seam VantaDB:** `desktop/src/components/inspector/` + `desktop/src/components/activity/` — VantaDB Studio existente (conexiones, memoria, skills, escenas, actividad, diagnóstico de retrieval): extenderlo, no crear producto nuevo.
- **Acciones:**
  1. Inspector de timeline sobre datos reales (read-only primero).
  2. Memory diff/provenance en UI.
  3. Detección de integraciones con preview + consentimiento, nunca escritura silenciosa.
  4. Sync selectivo.
  5. Superficie de errores reales de transporte/capabilities.
- **Dependencias:** X-OVK-04/X-OVK-07 para evidencia del ciclo; Studio existente; API de lectura ya disponible.
- **FMEA:** «instalado ≠ funcionando» → toda capacidad mostrada requiere evidencia real o se marca no-verificada; detección que sobrescribe config del usuario → preview+consentimiento obligatorio; tokens en UI/transporte → nunca persistir secretos, transporte seguro, errores reales propagados; diff con datos parciales → indicar cobertura.
- **Criterios de aceptación propuestos:** el timeline muestra sólo eventos con evidencia; la detección no muta nada sin consentimiento; errores de transporte visibles; sync selectivo respeta exclusión. Observabilidad (UI): eventos por stage, acciones de onboarding aceptadas/rechazadas.
- **Criterio negativo:** ninguna capacidad se muestra como funcionando sin evidencia real; ninguna detección escribe configuración sin consentimiento.
- **Independencia:** extensión de Studio existente; primera iteración read-only; depende de X-OVK-04/X-OVK-07 para la evidencia del ciclo.
- **No objetivos:** no es marketplace de integraciones; no es configurador remoto; no Helper propio ni atribuir a OV funciones que ya tiene Studio.

### X-OVK-11 — Sonda de capabilities por contrato de lectura (no por reachability)

- **Problema:** «instalado ≠ funcionando»: un endpoint alcanzable no demuestra que el backend expone el contrato de la feature; la UI no debe mostrar capacidades que fallarán al usarse.
- **Contrato hipotético:** tabla feature→probe donde cada sonda valida la **forma de la respuesta** del contrato de lectura (p. ej. `result.tasks` array, `items`+`total`, `is_healthy`+`components`), con estados `supported|unavailable|forbidden|unknown`; la feature se oculta/degrada salvo respuesta que cumple el contrato.
- **Fuente de evidencia (clone):** `web-studio/src/shared/lib/studio-service.ts:40-89,97-171` (`STUDIO_FEATURES` asigna a cada feature un endpoint probe; `classifyCapabilityResponse` valida la forma antes de mostrarla — comentario explícito «A reachable endpoint is not enough: verify the read contract»).
- **Seam VantaDB:** `desktop/src/` — sonda por feature sobre el cliente Studio existente.
- **Acciones:**
  1. Definir la tabla feature→probe con el contrato de lectura esperado de cada feature.
  2. Clasificar el resultado en `supported|unavailable|forbidden|unknown`.
  3. Gate de UI: ocultar/degradar salvo `supported`.
- **Dependencias:** cliente Studio existente; contratos de lectura del backend por feature; alimenta X-OVK-10/X-OVK-12.
- **FMEA:** endpoint que responde sin la forma válida → falso `supported` (validar shape, no sólo HTTP 200); feature de management condicionada al proveedor → sonda por feature, no global.
- **Criterios de aceptación propuestos:** feature oculta/degradada salvo respuesta que cumple el contrato de lectura; `forbidden`/`unknown` distinguibles del éxito.
- **Criterio negativo:** un endpoint alcanzable nunca muestra una feature sin contrato verificado; `unknown` nunca es `supported`.
- **Independencia:** implementable en Studio sin backend nuevo.
- **No objetivos:** no probing por reachability/ping ni sonda global única.

### X-OVK-12 — Verificación de conexión con probe real e higiene de credenciales en Studio

- **Problema:** no aceptar credenciales a ciegas: probe de salud → modo de auth → rol → lectura de datos real antes de guardar; separar planos de credencial (dato vs control) con rechazo pre-envío; en Tauri la clave vive en el keyring del SO, no en el storage del frontend.
- **Contrato hipotético:** `verifyStudioSetup` equivalente: probe encadenado health→`authMode`(dev/api_key/trusted)→rol (root/admin/user)→lectura real (`GET fs/ls` con `node_limit=1` o endpoint admin según caso); reconoce keys invertidas root↔user; redacta secretos en errores; rechazo fail-closed de rutas de control desde scope data (`PERMISSION_DENIED` pre-envío).
- **Fuente de evidencia (clone):** `web-studio/src/features/connection/lib/onboarding.ts:52-56,155-305,320-346` (`verifyStudioSetup` + perfiles multi-conexión cifrados), `web-studio/src/shared/api/ov-client/client.ts:126-146,213-266` (credenciales dual fail-closed con interceptores axios), `web-studio/src/features/connection/lib/credential-storage.ts:1-40` (AES-GCM-256, clave no exportable en IndexedDB, «encryption at rest, not an XSS/extension security boundary»), `web-studio/docs/admin-workbench-refactor.md`.
- **Seam VantaDB:** `desktop/src-tauri/src/commands/connection.rs` (comando de conexión existente), `desktop/src/` (UI de onboarding/conexión); en Tauri, keyring del SO en lugar de IndexedDB.
- **Acciones:**
  1. Probe encadenado en el comando de conexión (salud→authMode→rol→lectura real).
  2. Clasificación de credenciales y rechazo pre-envío de rutas de control desde scope data.
  3. Almacén seguro nativo (keyring/capability Tauri); saneado de errores sin secretos.
- **Dependencias:** comando `connection.rs` existente; keyring/capability de Tauri; X-OVK-10/X-OVK-11.
- **FMEA:** credencial aceptada a ciegas → probe real antes de persistir; secreto en mensaje de error → fuga (redacción); root key en plano de datos → escalada (rechazo pre-envío); credencial en sessionStorage → exposición (keyring SO).
- **Criterios de aceptación propuestos:** probe real antes de guardar; rechazo pre-envío de rutas de control desde scope data; errores sin secretos; credencial en keyring.
- **Criterio negativo:** la credencial nunca se guarda sin verificación real; la root key nunca hace fallback a peticiones de datos; ningún secreto aparece en errores ni storage del frontend.
- **Independencia:** comando existente extensible; no requiere backend nuevo.
- **No objetivos:** no IndexedDB/sessionStorage para credenciales en Tauri; no plano de control nuevo.

### X-OVK-13 — Vista de impacto por consolidación (diff add/update/delete con before/after)

- **Problema:** auditar qué escribió la extracción en cada commit: operaciones `add|update|delete` con `before`/`after` por **evento de extracción**, no por registro suelto (el Inspector actual ya tiene diff de historial por registro); requiere que el pipeline persista el artefacto de evidencia.
- **Contrato hipotético:** artefacto de diff por evento persistido por el pipeline (registro interno — en OV es `memory_diff.json` dentro del VFS) + vista read-only con operaciones, `memory_type`, URI y enlace al registro.
- **Fuente de evidencia (clone):** `web-studio/src/features/workbench/sessions/components/memory-impact.tsx:52-241,305-368` (vista agrupada por `archive_NNN` con link a `/agent-experience/$uri`), `web-studio/src/features/workbench/sessions/lib/memory-diff.ts:62-94` (parser del diff), `openviking/session/compressor_v3.py:385-414,1383-1403` (productor del artefacto; ignora memorias sin cambios), `web-studio/src/features/workbench/sessions/lib/api.ts:281-322` (lectura por convención `archive_{001..commit_count}` con `content/read?raw=true` — anti-patrón frágil a no adoptar).
- **Seam VantaDB:** `desktop/src/components/inspector/` — converge con `historial-diff.ts` existente sin duplicarlo.
- **Acciones:**
  1. Definir el artefacto de diff por consolidación persistido por el pipeline (registro interno, no convención de naming).
  2. Vista read-only por evento en Inspector enlazando al registro.
  3. Indicar cobertura cuando el artefacto sea parcial.
- **Dependencias:** artefacto de evidencia persistido por consolidación (X-OVK-04/X-OVK-07 lo alimentan); converge con `historial-diff.ts`.
- **FMEA:** adivinar artefactos por convención `archive_NNN` desde `commit_count` → acoplamiento frágil (no adoptar); diff con datos parciales → declarar cobertura.
- **Criterios de aceptación propuestos:** por evento se ven operaciones con before/after y enlace al registro; cobertura parcial declarada; primera iteración read-only.
- **Criterio negativo:** nunca adivinar artefactos por convención de naming; una cobertura parcial nunca se presenta como diff completo.
- **Independencia:** read-only sobre datos existentes; la vista completa depende del artefacto persistido.
- **No objetivos:** no duplicar el diff por registro existente; sin acciones en la primera iteración.

### X-OVK-14 — Superficie visual del pipeline de jobs por tipo con estados reales

- **Problema:** la UI de OV **infiere** pasos desde `task.result.queue_status` porque el backend no expone una máquina de estados explícita por ítem — anti-patrón a evitar; con estados explícitos (X-OVK-07) la vista es directa.
- **Contrato hipotético:** diagrama serial/paralelo por `job_type` con pasos y estados `pending|running|completed|failed` leídos del backend; sin heurística; job sin tipo conocido → `unknown`.
- **Fuente de evidencia (clone):** `web-studio/src/features/workbench/tasks/lib/task-pipeline.ts:26-40,46-267` (pasos y grupos serial/parallel por tipo: `externalParse→[semantic‖embedding]`, `session_commit`, `admin_reindex`, `connector_import`), `web-studio/src/features/workbench/monitoring/components/queue-status-card.tsx:18-44`; contraejemplo `web-studio/src/features/workbench/monitoring/lib/parse-status.ts:19-40` (endpoints observer devuelven tablas ASCII troceadas por formato — no adoptar).
- **Seam VantaDB:** `desktop/src/components/activity/` — superficie de actividad existente de Studio.
- **Acciones:**
  1. Componente de pasos por `job_type` alimentado por el estado explícito del job.
  2. Estados `pending|running|completed|failed` directos del backend; sin inferencia por contadores.
  3. Enlazar con la vista de cola de X-OVK-10.
- **Dependencias:** X-OVK-07 (estados explícitos en backend); X-OVK-10; superficie de actividad existente.
- **FMEA:** inferir pasos desde contadores de `queue_status` reproduce el anti-patrón → sólo estados explícitos; job sin tipo conocido → `unknown`, no diagrama inventado.
- **Criterios de aceptación propuestos:** el diagrama refleja estados reales del backend por `job_type`; sin estados explícitos muestra `unknown`.
- **Criterio negativo:** nunca reconstruir el pipeline por heurística de `queue_status`; las tablas ASCII nunca son contrato de diagnóstico.
- **Independencia:** depende de X-OVK-07 para estados fiables; vista independiente del resto de UI.
- **No objetivos:** no inferencia por heurística ni visualización que invente pasos no declarados por el backend.

<!-- audit: 2026-11-23 completado campo — en Dependencias de X-OVK-11, expandidas rutas truncadas con elipsis y corregida ruta openviking/server/routers/resources.py + prefijos web-studio/src/ en X-OVK-09/10 -->
