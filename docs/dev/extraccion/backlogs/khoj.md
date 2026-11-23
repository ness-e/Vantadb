---
title: "Backlog de extracción — Khoj"
kind: research
status: active
description: "Siete candidatas del informe Khoj: gramática de filtros, recibo de sync, citas por revisión, política de memoria, parser acotado, identidad de caché y ack por operación"
tags: [extraccion, backlog, khoj]
---

# Backlog de extracción — Khoj

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/khoj.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-KHO-01 | 🟡 | Gramática de filtros empujada al plan de consulta | `src/physical_plan/filter.rs`, `src/sdk/search/{hybrid,vector,explain}.rs` | 🟡 | 🟠 P1 | ⬜ Candidata | Traducción opcional de lenguaje de cliente a filtro tipado del AST existente con explicación de lo resuelto · Postfiltrar tras topK pierde vecinos válidos; errores de sintaxis, fecha ambigua o predicado no soportado deben rechazarse, no degradar a búsqueda sin filtro · Acción mínima: parser puro sin I/O que produce texto libre + filtro + explicación; explain confirma pushdown · **Trazabilidad:** `informe §3 ficha A · evidencia database/adapters/__init__.py:2090-2178, routers/helpers.py:1521-1530` | X-KHO-06 incluye el AST en su identidad de caché; converge con C7 de `CONSOLIDATED.md` | AST/filtros del motor existentes; `core_search` opt-in en el snapshot (informe §5.1) |
| X-KHO-02 | 🔴 | Recibo de sync por fuente, revisión y fingerprint de derivación | `vanta-memory/src/ingest/worker.rs::{run_with_progress,IngestReport}`, `vanta-memory/src/ingest/callback.rs::{IngestProgress,ProgressTracker}`, `vanta-memory/src/ingest/auto_sync.rs` | 🟠 | 🔴 P0 | ⬜ Candidata | `SyncReceipt` durable por fuente/revisión/destino con estados Unchanged/Applied/Pending/Skipped/Failed/Deleted · "Sync completo" no puede confirmarse por nombres enviados mientras fallan batches; timeout/rate-limit no son documento vacío ni tombstone · Acción mínima: extender el manifest/progreso actual con fingerprint source/parser/model y reconciliación por ID exacto · **Trazabilidad:** `informe §3 ficha B · evidencia routers/api_content.py:544-627, processor/content/text_to_entries.py:142-263, interface/obsidian/src/utils.ts:227-248 · issue khoj-ai/khoj#1363` | X-KHO-03 cita su revisión; X-KHO-05 limita parsers; X-KHO-07 muestra su estado; converge con C9/C13 de `CONSOLIDATED.md` | Worker, run_id, progreso y `AutoSyncScheduler` existentes; legado migra como desconocido |
| X-KHO-03 | 🟡 | Evidencia de entrega por turno con cita a revisión autorizada | `vanta-memory/src/context_engine/engine.rs::{assemble_inner,assemble_with_recall}`, `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 | 🟠 P1 | ⬜ Candidata | `ContextEvidence` por turno: source_id/revisión, clase corpus/history/memoria/externo, incluido/descartado/compactado y entrega parcial/final · Una respuesta que mezcla corpus, web y memorias necesita trazabilidad de qué entró al modelo; stream terminado no prueba escritura durable · Acción mínima: manifest tras el presupuesto, persistido mínimo en sesión, navegable a revisión autorizada · **Trazabilidad:** `informe §3 ficha C · evidencia search_type/text_search.py:141-164, routers/api_chat.py:985-998,1449-1470` | X-KHO-02 propaga identidad; X-KHO-04 y X-KHO-06 condicionan política/scope; converge con C8 de `CONSOLIDATED.md` | Provenance e `IntegratedContext`/`CompactionReport` existentes; reautorización al abrir tras delete/revocación |
| X-KHO-04 | 🟠 | Política efectiva de memoria y edición recuperable | `vanta-memory/src/core/record/l1_writer.rs::{plan_write,persist_planned,apply_dedup_batch}` | 🟠 | 🟠 P1 | ⬜ Candidata | `MemoryPolicyDecision` por principal/operación/revisión de política y `ReviseFact` con expected_revision e ID estable · Editar un hecho de agente no default puede retirarlo del recall al perder la asociación; desactivar memoria no es borrar y fallo de embed no es hecho eliminado · Acción mínima: resolver política → planificar reemplazo con CAS → conservar o declarar cambio de asociación · **Trazabilidad:** `informe §3 ficha D · evidencia database/adapters/__init__.py:1623-1695,2292-2378, routers/api_memories.py:18-114, routers/helpers.py:1048-1066` | X-KHO-02 aporta commit derivado; X-KHO-03/X-KHO-06 la consumen; converge con C6/C10 de `CONSOLIDATED.md` | Dedup/supersession/checkpoints existentes; política revalidada antes de publicar |
| X-KHO-05 | 🟠 | Terminación acotada y cancelación real de parsers de ingesta | `vanta-memory/src/ingest/worker.rs`, `src/wiki/sources.rs::scan_local_sources`, `src/wiki/chunker.rs` | 🟠 | 🔴 P0 | ⬜ Candidata | `ParseOutcome` por archivo con Unsupported/NonProgress/LimitExceeded/Cancelled; toda iteración consume entrada o reduce medida finita · Una nota hostil puede bloquear el sync de todo un vault; timeout HTTP no mata un hilo ocupado · Acción mínima: verificar progreso/cancelación en los parsers invocados y reportar fallo por archivo sin publicar derivados incompletos · **Trazabilidad:** `informe §3 ficha E · evidencia processor/content/markdown/markdown_to_entries.py:21-124 (bucle 86-101) · issue khoj-ai/khoj#1437` | X-KHO-02 informa cobertura; X-KHO-07 no confirma de más; converge con C9 de `CONSOLIDATED.md` | Límites definidos por política existente; workers aislados no publican tras cancelación |
| X-KHO-06 | 🟠 | Identidad de caché bajo principal, política, modelo y revisión | `vanta-memory/src/core/hooks/auto_recall.rs::InjectionPolicy`, `vanta-memory/src/core/record/l1_writer.rs::default_tenancy`; cachés de host/adapters (inventariar) | 🟡 | 🔴 P0 | ⬜ Candidata | `RecallCacheIdentity` = principal + scope autorizado + filtro normalizado + fingerprint embed/ranking + revisión corpus/política; Stale/Forbidden/UnknownRevision fuerzan miss · Un hit cross-agent o post-delete puede entregar contexto indebido · Acción mínima: unir autorización antes del hit y propagar revisión/invalidación desde X-KHO-02/X-KHO-04; rollback bypass sin relajar ACL · **Trazabilidad:** `informe §3 ficha F · evidencia routers/helpers.py:1514-1519,1567-1569,3177-3179` | X-KHO-01 aporta AST; X-KHO-02 revisión; X-KHO-04 política; converge con C3/C15 de `CONSOLIDATED.md` | Inventario de caché real; principal/política del servidor, no del caller |
| X-KHO-07 | 🟡 | Capacidad y confirmación por operación en transportes de cliente | `feature-freez/desktop/src/transport.ts`, `feature-freez/desktop/src/vanta.ts`, `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx::runSearch` | 🟡 | 🟡 P2 | ⬜ Candidata | `OperationAck` por conexión/operación/revisión con accepted/applied/partial/failed, capability revision y soporte de cancelación · Retorno de IPC no acredita indexación; un transporte sin edit/cancel debe declararlo, no hacer fallback silencioso · Acción mínima: descriptor por conexión enlazado a X-KHO-02, mostrando fase/proyección; reconectar consulta estado con clave idempotente · **Trazabilidad:** `informe §3 ficha G · evidencia interface/desktop/main.js:159-610 (sync 600-602 sin await), interface/obsidian/src/utils.ts:104-248` | X-KHO-02 receipts; X-KHO-05/X-KHO-06; converge con C15/C16 de `CONSOLIDATED.md` | Transportes Studio existentes; host conserva operación y alcance elegido |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/agent-memory-2026-11-07/khoj.md` (§3 fichas A–G). Todos los contratos son **hipotéticos** — propuestas de comportamiento para VantaDB, no APIs existentes ni puertos de código upstream. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\khoj` (rama `master`) en SHA `ae229ca894c0b80ad84664afcfdde523b5e87057` (prefijos `src/khoj/` para Python y `src/` para `interface/` abreviados); las rutas VantaDB citan el snapshot del informe y requieren revalidación contra HEAD. Prioridad documental del informe §2: P0 = B/E/F; P1 = A/C/D; P2 = G.

### X-KHO-01 — Gramática de filtros empujada al plan de consulta

- **Problema:** un agente pide notas de una fecha/ruta sin perder vecinos válidos; postfiltrar tras topK pierde candidatos y un error de sintaxis no puede degradar silenciosamente a búsqueda sin filtro.
- **Contrato hipotético (tipos propuestos):** `ParsedSearch` recibe texto y zona/instante de referencia explícitos; devuelve texto libre + filtro tipado + explicación de lo resuelto. `InvalidSyntax`, `AmbiguousDate`, `UnsupportedPredicate` rechazan sin degradar a búsqueda sin filtro; el filtro nunca amplía el scope autorizado.
- **Fuente de evidencia (clone):** `database/adapters/__init__.py:2090-2178` (`apply_filters`/`search_with_embeddings` empuja predicados al queryset antes de coseno/topK); `routers/helpers.py:1521-1530` (`execute_search` elimina términos estructurados para el embedding).
- **Seam VantaDB:** `src/physical_plan/filter.rs`, `src/sdk/search/{hybrid,vector,explain}.rs` y `RecallScope` — traducción de lenguaje de cliente sobre filtros/AST existentes, no filtro/índice nuevo.
- **Acciones:**
  1. Traducir el lenguaje de cliente al AST existente con un parser puro, sin I/O, cancelable antes de consultar.
  2. Validar campos y predicados soportados.
  3. Pasar a búsqueda y explain; confirmar pushdown, no solo resultados filtrados visualmente.
  4. Rotular en UI filtro de servidor vs visual.
  5. Si se guardan búsquedas, persistir versión de gramática y referencia temporal; legado sigue la API estructurada; rollback deshabilita el parser sin alterar índices.
- **Dependencias:** AST/filtros del motor existentes; `core_search` opt-in en el snapshot (informe §5.1); X-KHO-06 incluye el AST en su identidad de caché; converge con C7 de `CONSOLIDATED.md`. Alternativas: API estructurada existente (preferida para SDK); postfiltrar topK (rechazado como equivalente semántico).
- **FMEA:** fecha local frente a UTC → resultados distintos (detectar el rango resuelto y exponerlo); error de parsing silencioso → ampliar corpus (detectar tokens no consumidos y rechazar); no resolver «ayer» de nuevo en cada página ni reinterpretar consulta guardada con reloj distinto.
- **Criterios de aceptación propuestos:** ruta+fecha normal preserva citas y límite; glob escapado, negación, zona ambigua y sintaxis rota tienen resultado explícito; explain confirma pushdown.
- **Criterio negativo:** un predicado inválido o ambiguo nunca degrada a búsqueda sin filtro ni amplía el scope autorizado.
- **Independencia:** P1 documental; implementable sobre filtros existentes sin depender de otras candidatas — X-KHO-06 la consume, no al revés.
- **No objetivos:** copiar literal el DSL/regex de Khoj; lenguaje SQL; pesos nuevos de ranking.

### X-KHO-02 — Recibo de sync por fuente, revisión y fingerprint de derivación

- **Problema:** «sync completo» no puede confirmarse por nombres enviados mientras fallan batches; timeout/rate-limit no son documento vacío ni tombstone.
- **Contrato hipotético (tipos propuestos):** `SyncReceipt(source_id, revision, target, derivation_fingerprint, operation_id)` devuelve `Unchanged | Applied | Pending | Skipped(reason) | Failed(stage) | Deleted`; durable por fuente/revisión/destino.
- **Fuente de evidencia (clone):** `routers/api_content.py:544-627` (indexer devuelve nombres recibidos); `processor/content/text_to_entries.py:142-263` (`update_embeddings` puede capturar fallos por batch); `interface/obsidian/src/utils.ts:227-248` (confirma por substring de nombre); issue [khoj-ai/khoj#1363](https://github.com/khoj-ai/khoj/issues/1363) (necesidad operativa de estado por archivo, no benchmark).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs::{run_with_progress, IngestReport}`, `vanta-memory/src/ingest/callback.rs::{IngestProgress, ProgressTracker}` y `vanta-memory/src/ingest/auto_sync.rs::AutoSyncScheduler` — run_id/busy/late packet guard ya existen; el delta es el manifest durable por fuente/revisión/destino, no otra cola ni otro checkpoint.
- **Acciones:**
  1. Registrar el plan de sync.
  2. Derivar sin borrar la revisión activa.
  3. Confirmar por ID exacto (no por nombre).
  4. Activar y limpiar la proyección antigua.
  5. Guardar pendientes/tombstone; migrar mtime/estados legacy como desconocidos; rollback deja lectura de revisión anterior y conserva tombstones/reintentos.
- **Dependencias:** worker, run_id, progreso y `AutoSyncScheduler` existentes; X-KHO-05 limita parsers; X-KHO-06 invalida caché; X-KHO-03 cita su revisión; converge con C9/C13 de `CONSOLIDATED.md`. Alternativas: mtime+nombre (insuficiente para confirmación); reindexar todo (coste y ventana destructiva).
- **FMEA:** batch parcialmente fallido pero nombre confirmado → falsa cobertura (detectar IDs sin commit y devolver parcial); cambio de parser/modelo con hash igual → derivado obsoleto (comparar fingerprint y rederivar solo afectados); delete concurrente gana a derivación vieja; cancelación detiene nuevas fases y registra efectos ya confirmados — no supone matar el executor remoto.
- **Criterios de aceptación propuestos:** dos syncs iguales no emiten mutación innecesaria; rename, fuente borrada, fallo en un batch, restart y proveedor 429 conservan clasificación y retry; archivo vacío legítimo no borra salvo operación explícita.
- **Criterio negativo:** un nombre enviado nunca equivale a contenido indexado; timeout/rate-limit no se representan como documento vacío ni tombstone; exclusión por fuente+destino y expected revision, no lock global.
- **Independencia:** P0 documental; delta sobre progreso/scheduler existentes, sin depender de otras candidatas — X-KHO-03/X-KHO-06/X-KHO-07 la consumen.
- **No objetivos:** conectores SaaS nuevos ni bytes/modelos descargados; SSRF, permisos de raíz y secretos sí son fronteras a resolver, no autorización para diferirlos.

### X-KHO-03 — Evidencia de entrega por turno con cita a revisión autorizada

- **Problema:** una respuesta que mezcla notas privadas, web y recuerdos necesita trazabilidad de qué entró al modelo; una referencia no prueba soporte factual y un stream terminado no prueba escritura durable.
- **Contrato hipotético (tipos propuestos):** `ContextEvidence` recibe referencias autorizadas y resultado de assembly; devuelve `source_id/revision`, clase `corpus | history | user-memory | externo`, estado `incluido | descartado | compactado` y entrega `parcial | final`. `SourceUnavailable`, `PolicyChanged`, `MissingRevision` no inventan evidencia.
- **Fuente de evidencia (clone):** `search_type/text_search.py:141-164` (`collate_results` conserva ruta/URI/heading); `routers/api_chat.py:985-998` (recupera contexto interrumpido) y `:1449-1470` (agenda persistencia de referencias separadas por clase).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs::{assemble_inner, assemble_with_recall}` y `vanta-memory/src/core/hooks/auto_recall.rs::perform_auto_recall_governed` — provenance e `IntegratedContext`/`CompactionReport` ya existen; extender con entrega final.
- **Acciones:**
  1. Propagar identidad/revisión desde X-KHO-02.
  2. Generar el manifest después del presupuesto.
  3. Confirmar lo enviado y persistir mínimo en sesión.
  4. Navegar en Studio a revisión autorizada.
  5. Legado = unknown; rollback deja citas históricas legibles sin inventar revisiones ni eliminar provenance actual.
- **Dependencias:** provenance e `IntegratedContext`/`CompactionReport` existentes; X-KHO-02 propaga identidad; X-KHO-04 y X-KHO-06 condicionan política/scope; reautorización al abrir tras delete/revocación; converge con C8 de `CONSOLIDATED.md`. Alternativas: guardar hits brutos (no prueba entrega) o todo el prompt (riesgo privado) — manifest mínimo por turno/revisión es el candidato.
- **FMEA:** ruta renombrada → cita equivocada (detectar identidad/revisión, no resolver por filename actual); contexto hostil → instrucciones (preservar taint/`InjectionPolicy` existentes); append parcial al interrumpir — no marcar completo porque terminó el WebSocket; hash/puntero no permite recuperar payload sin permiso.
- **Criterios de aceptación propuestos:** nota+online+memoria quedan distinguibles; respuesta interrumpida muestra parcial; fuente borrada no expone contenido; presupuesto excluye evidencia sin presentarla como utilizada. Logs de IDs/revisiones, no extractos por defecto.
- **Criterio negativo:** ninguna cita referencia contenido que no fue entregado ni fuente sin autorización vigente; `SourceUnavailable`/`PolicyChanged`/`MissingRevision` no fabrican evidencia.
- **Independencia:** P1 documental; extensión del assembly existente que requiere la identidad de X-KHO-02 para revisiones — el manifest por turno es su núcleo propio.
- **No objetivos:** juez de grounding ni score de veracidad; el manifest explica entrada/salida del sistema, no el pensamiento interno del modelo.

### X-KHO-04 — Política efectiva de memoria y edición recuperable

- **Problema:** editar un hecho de agente no default puede retirarlo del recall al perder la asociación (upstream: `update_memory` borra antes de re-embedding, cambia el ID y llama `save_memory` sin `agent`); desactivar memoria no es borrar y un fallo de embed no es un hecho eliminado.
- **Contrato hipotético (tipos propuestos):** `MemoryPolicyDecision` recibe principal, operación y revisión de política; devuelve `allowed | denied` + motivo. `ReviseFact(id, expected_revision, replacement)` devuelve nueva revisión/ID estable o conflicto; `EmbedFailed` conserva el hecho activo, no éxito vacío.
- **Fuente de evidencia (clone):** `database/adapters/__init__.py:1623-1695` (`ConversationAdapters` combina server mode + preferencia), `:2292-2378` (`UserMemoryAdapters`: `agent=None` por defecto crea fila sin asociación; agentes no default leen solo sus memorias); `routers/api_memories.py:18-114` (`update_memory:70-114` y `:83-103` — elimina y guarda sin agent); `routers/helpers.py:1048-1066` (`ai_update_memories` invoca delete para extracción).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs::{plan_write, persist_planned, apply_dedup_batch}` — sesiones, dedup/supersession y writer existentes; integrar preferencia/política efectiva y revisión de edición sobre ese flujo, sin copiar el CRUD destructivo.
- **Acciones:**
  1. Resolver la política efectiva por operación (capture/recall/edit/delete).
  2. Planificar el reemplazo con CAS por revisión — no last-writer-wins silencioso.
  3. Validar propiedad de IDs sugeridos por LLM (el LLM no autoriza IDs ni borra otra tenancy).
  4. Derivar, persistir/activar y marcar supersession mediante el flujo existente; conservar la asociación o exigir cambio explícito.
  5. Versionar política y outcome, no la respuesta raw; legado mantiene preferencia explícita; rollback suspende nuevos updates sin resucitar borrados ni cambiar el opt-in.
- **Dependencias:** dedup/supersession/checkpoints existentes; política revalidada antes de publicar; X-KHO-02 aporta commit derivado; X-KHO-03/X-KHO-06 la consumen; converge con C6/C10 de `CONSOLIDATED.md`. Alternativas: interruptor solo UI (no enforcement); disabled=purga automática (semántica peligrosa).
- **FMEA:** modelo inválido confundido con `NoFacts` → pérdida (outcome tipado + retry); disable durante extracción → persistencia no deseada (detectar revisión de política y denegar antes del commit); edición exitosa sin asociación → hecho fuera del recall del agente original (comparar asociación anterior/propuesta y conservarla o exigir cambio explícito); cancelación previa al efecto no borra; posterior devuelve applied.
- **Criterios de aceptación propuestos:** servidor disabled domina preferencia; default/user override explícitos; desactivar no borra implícitamente; edición fallida, concurrente o con ID ajeno conserva invariantes y produce error distinguible; distinguir ID de otro usuario de otra asociación del mismo usuario; una edición conserva asociación o informa su cambio intencional y efecto en recall. Auditoría de decisiones sin hechos sensibles.
- **Criterio negativo:** `EmbedFailed` nunca reporta el hecho como eliminado ni éxito vacío; ninguna operación omite la revalidación de política antes de publicar.
- **Independencia:** P1 documental; delta sobre writer/dedup existentes — alimenta a X-KHO-03/X-KHO-06 sin requerirlas.
- **No objetivos:** prompts/rúbricas nuevos; copiar default-agent global por usuario como permiso universal; retención legal requiere decisión separada.

### X-KHO-05 — Terminación acotada y cancelación real de parsers de ingesta

- **Problema:** una nota hostil puede bloquear el sync de todo un vault (regex de detección más amplia que la de split y bucle sin bound); un timeout HTTP no mata un hilo ocupado por una librería bloqueante.
- **Contrato hipotético (tipos propuestos):** `ParseOutcome(source_revision, parser_version, cancellation)` devuelve fragmentos o `Unsupported | NonProgress | LimitExceeded | Cancelled`; toda iteración consume entrada o reduce una medida finita. Los límites los define la política existente, no números inventados aquí.
- **Fuente de evidencia (clone):** `processor/content/markdown/markdown_to_entries.py:21-124` (`process_single_markdown_file`, bucle `:86-101`); issue [khoj-ai/khoj#1437](https://github.com/khoj-ai/khoj/issues/1437) (refiere exactamente el SHA fijado; no reproducción propia).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs` (ya coordina y emite progreso), que invoca `src/wiki/sources.rs::scan_local_sources` y `src/wiki/chunker.rs::chunk_text` antes de `ingest/merge.rs::commit`; evaluar bound/cancelación del parser invocado, no modificar HNSW ni scheduler.
- **Acciones:**
  1. Identificar ciclos recursivos en los parsers invocados.
  2. Asegurar una medida de progreso por iteración.
  3. Comprobar cancelación en fronteras.
  4. Reportar fallo por archivo sin activar derivados.
  5. Persistir solo versión/outcome; legado puede seguir el parser previo solo si es explícitamente seguro; rollback deshabilita el formato problemático, no reintroduce procesamiento ilimitado.
- **Dependencias:** límites definidos por política existente; workers aislados no publican tras cancelación; X-KHO-02 informa cobertura; X-KHO-07 no confirma de más; converge con C9 de `CONSOLIDATED.md`. Alternativas: timeout HTTP (no mata hilo); límite monotónico por parser; proceso aislado para parsers no cooperativos.
- **FMEA:** heading sin título → bucle → CPU/cola agotada (detectar no progreso y cortar); límite excesivo → rechazo válido (mostrar razón y conservar el original, no borrar); dropping de future no prueba parada de librería bloqueante; no retener lock de escritura al parsear.
- **Criterios de aceptación propuestos:** Markdown normal conserva headings/line refs; heading vacío/tab, documento largo y cancelación terminan con resultado explícito; otros archivos continúan sin publicar derivados incompletos.
- **Criterio negativo:** ningún parser itera sin consumir entrada ni reducir la medida finita; un archivo hostil no detiene el sync del resto ni publica derivados incompletos.
- **Independencia:** P0 documental; evaluación acotada de los parsers invocados por el worker existente — no requiere otras candidatas; X-KHO-02/X-KHO-07 consumen sus outcomes.
- **No objetivos:** copiar parser GPL/AGPL; nuevo formato; afirmar bug de Vanta sin encontrar el mismo ciclo y reproducirlo por separado.

### X-KHO-06 — Identidad de caché bajo principal, política, modelo y revisión

- **Problema:** un hit cross-agent o post-delete puede entregar contexto indebido (upstream cachea por usuario sin agent/model/corpus revision en clave); TTL solo no sustituye revocación.
- **Contrato hipotético (tipos propuestos):** `RecallCacheIdentity` recibe `principal_ref`, scope autorizado, filtro normalizado, fingerprint embed/ranking y revisión de corpus/política; devuelve hit solo si la identidad coincide. `Stale`, `Forbidden`, `UnknownRevision` fuerzan miss/denegación, nunca éxito con resultado anterior.
- **Fuente de evidencia (clone):** `routers/helpers.py:1514-1519,1567-1569` (`execute_search` usa caché por usuario sin agent/model/corpus revision en clave); `routers/helpers.py:3177-3179` (`configure_content` invalida al configurar). El informe declara no haber leído toda la invalidación: riesgo, no exploit probado.
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs::InjectionPolicy` (autoriza inyección), `vanta-memory/src/core/record/l1_writer.rs::default_tenancy` (sella tenancy); verificar las cachés del host/adapters que envuelven recall — no instalar caché nueva en el motor por analogía.
- **Acciones:**
  1. Inventariar la caché real en la ruta elegida.
  2. Unir la autorización antes del hit.
  3. Propagar revisión/invalidación desde X-KHO-02/X-KHO-04.
  4. No persistir payloads si no hace falta.
  5. Migración descarta caché sin versión; rollback bypass caché — nunca relaja ACL.
- **Dependencias:** inventario de caché real; principal/política del servidor, no del caller; X-KHO-01 aporta AST; X-KHO-02 revisión; X-KHO-04 política; converge con C3/C15 de `CONSOLIDATED.md`. Alternativas: no cachear contenido autorizado (simple/seguro); invalidar por usuario (grueso).
- **FMEA:** hit cross-agent → contexto indebido (detectar fingerprint mismatch y reautorizar); cambio de modelo que altera dimensión → error (comparar schema/fingerprint; no asumir score comparable); cambio concurrente invalida publicación/hit; cancelar consulta no escribe entrada parcial ni mueve watermark.
- **Criterios de aceptación propuestos:** consulta repetida válida mantiene permiso; delete, cambio de agente/política/embedding o parser no reutilizan hit viejo; caché legacy y cancelación producen miss seguro sin texto sensible en logs.
- **Criterio negativo:** `Stale`/`Forbidden`/`UnknownRevision` nunca devuelven éxito con el resultado anterior; ningún hit omite la autorización del principal resuelto por el servidor.
- **Independencia:** P0 documental, condicionada a que exista caché real y necesidad (informe: «candidato si existe cache y necesidad»); consume identidad de X-KHO-01/X-KHO-02/X-KHO-04.
- **No objetivos:** mejora de performance prometida; benchmark nuevo; afirmar fuga de Vanta por parecido de clave upstream.

### X-KHO-07 — Capacidad y confirmación por operación en transportes de cliente

- **Problema:** retorno de IPC no acredita indexación (upstream llama `syncData` sin `await`); un transporte sin edit/cancel debe declararlo, no hacer fallback silencioso.
- **Contrato hipotético (tipos propuestos):** `OperationAck(connection_id, operation_id, revision)` → `accepted | applied | partial | failed`, capability revision y soporte de cancelación. `UnsupportedTransport`, `Disconnected`, `PermissionDenied` no disparan fallback silencioso a otro destino.
- **Fuente de evidencia (clone):** `interface/desktop/main.js:159-610` (`syncData` invocado sin retornar `await` en `:600-602`); `interface/obsidian/src/utils.ts:104-248` (espera batches y actualiza mtime por nombres — semánticas diferentes por superficie).
- **Seam VantaDB:** `feature-freez/desktop/src/transport.ts`, `feature-freez/desktop/src/vanta.ts` y conexiones existentes; `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx::runSearch` usa `search({explain:true})`; extender capacidades/ack/error de los transportes existentes, no crear otra app.
- **Acciones:**
  1. Mapear los comandos reales por conexión.
  2. Enlazar el ID de operación al recibo de X-KHO-02.
  3. Mostrar fase/proyección y no emitir éxito desde el retorno de IPC.
  4. Conservar selección local mientras hay fallo recuperable.
  5. Versionar prefs/capacidades — no tokens en memoria; legado = unknown; rollback rotula capacidad desconocida.
- **Dependencias:** transportes Studio existentes; host conserva operación y alcance elegido; X-KHO-02 receipts; X-KHO-05/X-KHO-06; converge con C15/C16 de `CONSOLIDATED.md`. Alternativas: booleano `connected` (no acredita capacidad); misma firma para todo (puede ocultar unsupported).
- **FMEA:** acks fuera de orden → falso éxito (comparar operation/revision); path retornado por fuente hostil → apertura externa (validar esquema y raíz, confirmar egress/acción); desconectar UI no borra ni cancela implícitamente; reconectar consulta estado — no reenvía sin clave idempotente; elegir otra conexión no retargetea trabajo.
- **Criterios de aceptación propuestos:** conexión soportada funciona; transporte sin edit/cancel lo declara; offline, respuesta tardía, restart y cambio de conexión no confirman otra operación ni abren rutas arbitrarias.
- **Criterio negativo:** ningún retorno de IPC se presenta como indexación aplicada; `UnsupportedTransport`/`Disconnected`/`PermissionDenied` no disparan fallback silencioso.
- **Independencia:** P2 documental; descriptor por conexión sobre transportes existentes — enlaza a X-KHO-02 para recibos pero su descriptor/capability revision es un delta propio.
- **No objetivos:** portar Electron/Obsidian/TWA; prometer paridad por compartir API.

<!-- audit: 2026-11-23 completadas rutas VantaDB/Studio@0ad38c542 y precisados prefijos src/khoj/ y src/ del clon -->
