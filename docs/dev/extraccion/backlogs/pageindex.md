---
title: "Backlog de extracción — PageIndex"
kind: research
status: active
description: "Candidatas A–D del informe PageIndex: spans de fuente versionados, publicación coherente de generaciones documentales, navegación estructurada como adapter de recall y presupuesto de ejecución del host"
tags: [extraccion, backlog, pageindex]
---

# Backlog de extracción — PageIndex

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/pageindex.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-PGI-01 | 🟠 | Cita versionada con intervalo de fuente verificado | `vanta-memory/src/ingest/merge.rs::CandidatePage/commit:18-30,239-289`, `src/wiki/ingestors.rs::Ingestor:45-89` | 🟡 | 🟡 P2 | ⬜ Candidata | **Qué:** `resolve_span(source_id, expected_revision, span, RecallScope) -> ResolvedSpan` con `SourceSpan{source_revision, start, end, unit}` y `SectionRef{stable_source_id, span, parent}`; página física, etiqueta impresa y offsets de texto nunca intercambiables, y la introducción entre inicio de sección y primer hijo conserva ruta de acceso. **Por qué:** una jerarquía puede perder el texto previo al primer encabezado y una cita no debe cambiar al reindexar; sin evidencia de posición se reporta desconocido, no se fabrican coordenadas desde un resumen. **Acción mínima:** inventariar spans/referencias ya persistidos por ingestors, verificar intervalos antes de publicar estructura y conservar correspondencia de IDs al reconstruir; campo aditivo, legacy unknown hasta verificar original. **Trazabilidad:** informe §Candidata A + §11.2 · evidencia `pageindex/tree_optimize.py:172-266` (semántica unión/intro/renumeración), `pageindex/local_api.py:219-286` (chequeo de límites) | X-PGI-03 (la consume); C4/C5 de CONSOLIDATED | Identidad de documento/revisión, texto original retenido, parser identificable, namespaces y autorización de fuente; priorizar cuando se necesite inspección documental (informe §9) |
| X-PGI-02 | 🟠 | Publicación coherente de generaciones documentales | `vanta-memory/src/utils/task_checkpoint.rs::TaskCheckpointManager::{begin,advance,fail}`, `src/wiki/state.rs::WikiState`, `src/backend.rs:1-57` | 🟠 | 🟡 P2 | ⬜ Candidata | **Qué:** `publish_generation(source_id, expected_revision, artifacts, operation_id) -> GenerationPublication` con `DocumentGeneration{source_revision, tree_digest, text_digest, state}` y `PublicationReceipt{generation, visibility}`; staging separado de publicada y lectura de generación incompleta como diagnóstico, no borrado silencioso. **Por qué:** un lector no debe combinar árbol nuevo con páginas viejas ni interpretar una caché de manifiesto como fuente de verdad; replace individual por archivo no equivale a commit conjunto. **Acción mínima:** localizar generaciones y receipts actuales y ampliar antes que duplicar mecanismo; ligar checkpoint y artefactos a una sola identidad de operación; `WriteReceipt` existe (hash, no firma) — añadir solo la semántica no expresada. **Trazabilidad:** informe §Candidata B + §11.2 · evidencia `pageindex/local_store.py:1-186` (escritura atómica por archivo, sin fsync de directorio ni commit multiarchivo) | X-PGI-01, X-PGI-03; C6/C14 | Batch/orden real del backend, validación de digests, semántica WAL y coordinación con recolector/rebuild existentes; aplicable solo si el contrato actual no expresa una publicación que ya ocurre |
| X-PGI-03 | 🟡 | Navegación estructurada de secciones ligada a evidencia | `src/wiki/ingestors.rs::Ingestor`, `vanta-memory/src/core/hooks/auto_recall.rs::{RecallScope,perform_auto_recall_governed}`, `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` | 🟠 | 🟡 P2 | ⬜ Candidata | **Qué:** `list_sections(source_id, revision, cursor, scope)` y `read_section(section_ref, revision, remaining_budget, scope)` como adapter de recall: árbol paginado sin texto para orientar selección y rangos dirigidos para evidencia, con cursores ligados a revisión y conteo returned/omitted. **Por qué:** encontrar una sección en un informe largo sin volcar todo el PDF ni asumir que top-k de fragmentos captura su introducción; la estructura orienta y el contenido de página es evidencia distinta del resumen. **Acción mínima:** mapear X-PGI-01 → importar árbol con X-PGI-02 → validar ciclos/rangos → listar sin texto → resolver sección bajo ACL → render existente; persistir árbol como derivado con fingerprint, no duplicar fuente. **Trazabilidad:** informe §11.3 Candidata C · evidencia `pageindex/agent_tools.py:910-1008` (`_get_document_structure`) y `:1011-1127` (`_get_page_content`), `:1524-1570` (`_tool_specs` local) | X-PGI-01, X-PGI-02, X-PGI-04; C4/C7/C14/C16 | X-PGI-01 (identidad/spans), X-PGI-02 (frescura/generación), presupuesto existente C7, inspector C16; complementa — no sustituye — BM25/HNSW/RRF |
| X-PGI-04 | 🟡 | Presupuesto de ejecución y cancelación de navegación en el host | `vanta-memory/src/ingest/worker.rs::{begin,execute}`, `vanta-memory/src/utils/task_checkpoint.rs::TaskCheckpointManager` | 🟡 | 🟡 P2 | ⬜ Candidata | **Qué:** `navigate(request, run_context)` con remanente del host (identidad/revisión, cancelación) en el adapter: agotamiento impide llamadas nuevas y la salida declara uso observado/completitud/estado (`DeadlineExceeded`, `Cancelled`, `UsageUnknown`), separado del presupuesto de contexto del assembler. **Por qué:** un agente puede respetar tokens de contexto y seguir gastando turnos, retries y tiempo de herramientas después de que el caller abandone; timeout de request no es fin de run. **Acción mínima:** inventariar cancel/budget existentes, propagar remanente al adapter opt-in, finalizar recursos y emitir resultado parcial con razón; baseline híbrido sigue utilizable al desactivar. **Trazabilidad:** informe §11.4 Candidata D · evidencia `pageindex/local_chat.py:1052-1059` (`_run_kwargs`), `:1434-1501` (`_chat_events_agen`), `:162-227` (`_stream_sync`); issue #569 (max_turns vs deadline, sin fix acreditado) | X-PGI-03; C7/C9 | C7 distingue contexto/tiempo/coste, C9 owner/cancel, X-PGI-01 fija revisión; adapter opt-in, no bloquea X-PGI-03 determinista ni ingesta sin LLM |

## Fichas completas

> Expansión de la tabla anterior con el contenido del informe fuente. Todos los tipos, firmas Rust y errores son **propuestos** — no describen APIs existentes de VantaDB ni autorizan implementación. Las rutas de evidencia corresponden al pin upstream `2ac8694bf8fe89a3840dda9bdb88db5119b77fc4` (clon `C:\Users\Eros\PageIndex`); las de VantaDB, a la base `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`. `—` marca campos no documentados por el informe.

### X-PGI-01 — Cita versionada con intervalo de fuente verificado

- **Problema:** una jerarquía puede perder el texto entre el comienzo de una sección y su primer hijo, y una cita no debe cambiar al reindexar. Página física, etiqueta impresa y offsets de texto no son intercambiables.
- **Contrato hipotético (tipos propuestos):** `resolve_span(source_id, expected_revision, span, RecallScope) -> Result<ResolvedSpan, SpanError>` con salida texto autorizado + intervalo + revisión; `SourceSpan { source_revision, start, end, unit }` y `SectionRef { stable_source_id, span, parent }`. Errores propuestos: `Denied`, `InvalidInterval`, `MissingSource`, `StaleRevision`, `ReadFailed`. Una cita no acredita entailment.
- **Fuente de evidencia:** `pageindex/tree_optimize.py:172-266` (P3: semántica unión/intro/renumeración; el `id_map` de optimización no convierte ordinal en identidad durable) · `pageindex/local_api.py:219-286` (P2: chequeo de límites cuando hay start/end — no establece por sí solo orden ni validez de todos los nodos). Issue #648 como contexto de cache permutada/duplicada leída por posición.
- **Seam VantaDB:** `vanta-memory/src/ingest/merge.rs::{CandidatePage,commit}:18-30,239-289`, `src/wiki/ingestors.rs::Ingestor:45-89`, `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` (V1-V2).
- **Acciones:**
  1. Inventariar spans/referencias ya persistidos por ingestors.
  2. Distinguir página física, etiqueta impresa y offsets de texto; conversión explícita al adaptar formatos, no números ambiguos.
  3. Verificar intervalos antes de publicar estructura; rechazar inversión, fuera de rango o relación entre revisiones diferentes.
  4. Conservar correspondencia de IDs al reconstruir un árbol; no usar la numeración secuencial P3 como identidad eterna.
  5. Campo aditivo unidad/revisión; legacy `unknown` hasta verificar original — backfill solo con original verificable.
- **Dependencias:** identidad de documento/revisión, texto original retenido, parser identificable, namespaces y autorización de fuente. Priorizar cuando se necesite inspección documental (informe §9).
- **FMEA:** introducción omitida — búsqueda solo en hojas pierde evidencia previa al primer encabezado → representar cobertura sin duplicarla al ensamblar; página compartida — cortar antes del encabezado elimina texto útil → admitir solapamiento sin doble consumo de contexto; idiomas — la heurística de inicio P3 exige letras latinas → no elevarla a verdad estructural universal; cache permutada — posición sustituye identidad → página incorrecta → validar cobertura/unicidad/`page_index` como lectura, no rehacer la fuente automáticamente.
- **Criterios de aceptación propuestos:** cada referencia resuelve sobre la revisión citada; el texto introductorio conserva una ruta de acceso y límites válidos. Adversos: cache duplicada, rename, stale, Unicode, ACL. Log de IDs/revisión/unidad/omisiones sin texto.
- **Criterio negativo:** sin evidencia de posición se reporta desconocido; no se fabrican coordenadas a partir de un resumen del modelo ni se rehace la fuente por una lectura de cache anómala.
- **Independencia:** puede trabajar con documentos ya publicados, sin X-PGI-02.
- **No objetivos:** nuevo parser PDF, modelo/juez ni reemplazar retrieval híbrido; no OCR nuevo. Abierto: exactitud de offset y retención por decidir.

### X-PGI-02 — Publicación coherente de generaciones documentales

- **Problema:** un lector no debe combinar árbol nuevo con páginas viejas ni interpretar una caché de manifiesto como fuente de verdad; replace individual por archivo no equivale a commit conjunto.
- **Contrato hipotético (tipos propuestos):** `publish_generation(source_id, expected_revision, artifacts, operation_id) -> Result<GenerationPublication, PublicationError>` con salida generación visible, proyecciones pendientes y referencia de evidencia; `DocumentGeneration { source_revision, tree_digest, text_digest, state }` y `PublicationReceipt { generation, visibility }`. Errores propuestos: `RevisionConflict`, `IncompleteArtifacts`, `StorageFailed`, `Cancelled`, `ProjectionPending`. Commit primario ≠ búsqueda completa.
- **Fuente de evidencia:** `pageindex/local_store.py:1-186` (P4: flush/fsync/replace por archivo con limpieza del temporal; no se observa fsync de directorio ni atomicidad multiarchivo; `save_document` escribe árbol, páginas y meta en secuencia; lock POSIX ausente en Windows, best-effort documentado).
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs::TaskCheckpointManager::{begin,advance,fail}`, `src/wiki/state.rs::WikiState`, `src/backend.rs:1-57` (V3: backend particionado y metadatos internos de salud). `WriteReceipt` ya existe en `src/attestation.rs:654-738` (hash SHA-256, no firma) — añadir solo la semántica no expresada.
- **Acciones:**
  1. Localizar las generaciones y receipts actuales; ampliar antes que duplicar un mecanismo existente.
  2. Especificar qué puntero/registro hace visible una generación completa y cómo se distingue staging de publicada.
  3. Ligar checkpoint y artefactos a una sola identidad de operación.
  4. Definir relectura de una generación incompleta como diagnóstico, no borrado silencioso de la fuente primaria.
  5. Secuencia propuesta: preparar artefactos → verificar → registrar revisión esperada → publicar marcador existente si cubre la unidad → proyectar → diagnosticar.
- **Dependencias:** batch/orden real del backend, validación de digests, semántica WAL y coordinación con recolector/rebuild existentes. Aplicable solo si el contrato actual no expresa una publicación que ya ocurre.
- **FMEA:** crash intermedio — derivados de generaciones distintas → impedir exposición conjunta hasta verificar descriptor de publicación; último escritor — dos ingestas de la misma fuente sobrescriben estado válido → exigir identidad/revisión en la transición visible; error confundido con ausencia — una lectura fallida dispara reingesta destructiva → distinguir corrupción, permisos y documento inexistente.
- **Criterios de aceptación propuestos:** lectores ven una generación coherente o estado explícito no disponible; retry no vuelve a publicar una versión obsoleta; fallo conserva diagnóstico y fuente original. Adversos: dos writers, crash/cancel en frontera, error de IO; fase/revisión/operation/conteos sin payload.
- **Criterio negativo:** no se anuncia durabilidad más fuerte que la proporcionada por el backend; legado sin generación no se «certifica»; `ProjectionPending` no se presenta como commit.
- **Independencia:** no requiere árbol de secciones ni X-PGI-01.
- **No objetivos:** copiar almacenamiento plano, desplegar cloud ni nuevo consenso/docstore JSON. Abierto: unidad real de publicación y retención de staging.

### X-PGI-03 — Navegación estructurada de secciones ligada a evidencia

- **Problema:** encontrar una sección dentro de un informe largo sin volcar todo el PDF ni asumir que top-k de fragmentos captura su introducción; la estructura orienta la selección y el contenido de página es evidencia distinta del resumen.
- **Contrato hipotético (tipos propuestos):** `list_sections(source_id, revision, cursor, scope) -> Result<SectionPage, NavigationError>` y `read_section(section_ref, revision, remaining_budget, scope) -> Result<SectionEvidence, ...>`. Cursores ligados a revisión; salida incluye returned/omitted y padre/rango. Errores propuestos: `Denied`, `UnknownCursor`, `StaleRevision`, `MissingSection`, `ReadFailed`, `BudgetExhausted`.
- **Fuente de evidencia:** `pageindex/agent_tools.py:910-1008` (`_get_document_structure`: árbol sin texto, resúmenes y partes) · `:1011-1127` (`_get_page_content`: páginas concretas, enumera devueltas/restantes/fuera de rango) · `:1524-1570` (`_tool_specs` local). Umbrales upstream (20 páginas de guidance, `_CHAR_BUDGET=95000`) son defaults externos, no configuración propuesta.
- **Seam VantaDB:** `src/wiki/ingestors.rs::Ingestor` y `CandidatePage.sources` alimentan estructura; `vanta-memory/src/core/hooks/auto_recall.rs::{RecallScope,perform_auto_recall_governed}` y `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` consumen evidencia. Adapter de recall — complementa, no sustituye, BM25/HNSW/RRF.
- **Acciones:**
  1. Mapear X-PGI-01 → importar árbol con X-PGI-02 → validar ciclos/rangos/cobertura → listar sin texto → resolver sección bajo ACL → render existente.
  2. Persistir el árbol como derivado con fingerprint, no duplicar la fuente; migración aditiva, rebuild solo desde revisión identificada.
  3. Rollback apaga el adapter, no las fuentes.
- **Dependencias:** X-PGI-01 (identidad/spans, C4), X-PGI-02 (frescura/generación, C14), presupuesto existente C7, inspector C16. Tripletas/ontología no son requisito para una jerarquía documental.
- **FMEA:** homónimo/path truncado → documento equivocado → cita cruzada → detectar resolución no única y rechazar; árbol cíclico/hub → loop/coste → validar estructura y consumir el presupuesto ya existente, no un scheduler nuevo.
- **Criterios de aceptación propuestos:** normal árbol→página correcta; adversos: nombres iguales, cursor viejo, padre sin texto, revisión retirada, cancel entre lecturas. Observar fuente/revisión/nodos/bytes omitidos — no títulos ni contenido.
- **Criterio negativo:** el nombre/path descriptivo nunca reemplaza la identidad autorizada; la resolución ambigua se rechaza en lugar de elegir por heurística.
- **Independencia:** read-only sobre un documento; depende de X-PGI-01/X-PGI-02 para identidad y frescura, no de X-PGI-04.
- **No objetivos:** no crear carpetas globales, permisos por path ni MCP cloud obligatorio; no loop de razonador en el core. Abierto: política de selección del host y formato de árbol interoperable.

### X-PGI-04 — Presupuesto de ejecución y cancelación de navegación en el host

- **Problema:** un agente puede respetar tokens de contexto y seguir gastando turnos, retries y tiempo de herramientas después de que el caller abandone; timeout de request no es fin de run. El presupuesto de `assemble_with_recall` es de contexto — no prueba límite global de red/modelo.
- **Contrato hipotético (tipos propuestos):** `navigate(request, run_context) -> Result<NavigationRun, NavigationRunError>`; `run_context` lleva identidad/revisión, cancelación y remanente del host; salida evidencia + uso observado/completitud/estado. Errores propuestos: `DeadlineExceeded`, `Cancelled`, `ProviderFailed`, `UsageUnknown`, `StaleRevision`.
- **Fuente de evidencia:** `pageindex/local_chat.py:1052-1059` (`_run_kwargs`) · `:1434-1501` (`_chat_events_agen`: cancela al abandonar y cierra backend en finally) · `:162-227` (`_stream_sync`: señala cancelación y espera teardown — no prueba que el proveedor remoto deje de consumir). Issue #569 distingue `max_turns` de deadline (propuesta de timeout sin fix acreditado).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs::{begin,execute}` y `vanta-memory/src/utils/task_checkpoint.rs::TaskCheckpointManager` cuando es ingesta; en recall, caller de `perform_auto_recall_governed` — no persistir estado de ejecución como memoria.
- **Acciones:**
  1. Inventariar cancel/budget existentes → propagar remanente al adapter opt-in → finalizar recursos → emitir resultado parcial con razón.
  2. Lectura sin migración del primario; diagnóstico opcional con schema versionado.
  3. Rollback apaga navegación por modelo; el baseline híbrido sigue utilizable.
- **Dependencias:** C7 distingue contexto/tiempo/coste, C9 owner/cancel, X-PGI-01 fija revisión; adapter opt-in — no bloquea X-PGI-03 determinista ni la ingesta sin LLM.
- **FMEA:** timeout de request confundido con final de run → la tarea sigue → coste/threads → detectar llamadas tras cancel y teardown pendiente; propagar la señal al owner y no marcar concluido antes de conocer su límite efectivo.
- **Criterios de aceptación propuestos:** normal navegación acotada; adversos: modelo lento, herramienta bloqueada, abandono de stream y fallo de cierre. Observar turnos/tiempos/uso faltante/cancelación sin pregunta, respuesta ni secretos.
- **Criterio negativo:** el agotamiento impide llamadas nuevas pero no promete cancelar el pago del proveedor; un stream abandonado no se reporta como run concluido.
- **Independencia:** adapter opt-in sobre un protocolo; el baseline híbrido sigue utilizable al desactivar.
- **No objetivos:** no copiar Agents, nuevo runtime (Tokio paralelo), prompts ni políticas de precios. Abierto: límite de cancelación del proveedor y contrato de uso parcial deben explicitarse antes de implementar.

<!-- audit: 2026-11-23 completada ruta repo-relativa vanta-memory/src/context_engine/engine.rs en ficha X-PGI-01 -->
