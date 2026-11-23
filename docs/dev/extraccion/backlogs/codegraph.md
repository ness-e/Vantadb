---
title: "Backlog de extracción — CodeGraph"
kind: research
status: active
description: "Candidatas del informe CodeGraph: frescura revisionada de la evidencia entregada, publicación por generación de fuente, referencias pendientes, límite dinámico y puerto visual"
tags: [extraccion, backlog, codegraph]
---

# Backlog de extracción — CodeGraph

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/codegraph.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-CGR-01 | 🔴 | Evidencia entregada ligada a revisión verificable | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: cada rango de evidencia entregado lleva source_id, revisión y digest de la generación que sustentó la respuesta, con estados verified/stale/unchecked. Por qué: un contexto no debe presentar rangos de una revisión sobre bytes de otra; un deadline vencido no certifica frescura. Acción mínima: mapear revision/provenance actuales, llevar el digest de la generación recuperada hasta el assembler y verificar contribuyentes con presupuesto acotado; nunca renderizar rango antiguo como texto actual. Trazabilidad: informe §Unidad A · evidencia `src/mcp/answer-freshness.ts:1-66`, `ui/src/lib/adapter.ts:1-125` | X-CGR-02 (usa su catálogo de generación), C12, C14 | Lectores de fuente autorizados, retención/acceso a fuente, path guard y límites de trabajo |
| X-CGR-02 | 🔴 | Publicación recuperable de generación por fuente | `vanta-memory/src/ingest/merge.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: reingesta de una fuente publica una generación nueva mediante el commit vigente, conservando última evidencia utilizable cuando el extractor falla y reemparejando relaciones entrantes de forma trazable. Por qué: borrar evidencia válida o perder relaciones entrantes aunque solo cambió una fuente rompe provenance y supersession. Acción mínima: inventariar atomicidad actual, extender el coordinador existente con validación de relaciones/límites antes del commit e invalidación por revisión; diferenciar extractor fallido de fuente vacía. Trazabilidad: informe §Unidad B · evidencia `src/extraction/index.ts:3319-3480`, `src/index.ts:1110-1169` | X-CGR-01, X-CGR-03, C12, C1, C9 | IDs estables de fuente/relación, retención de generación previa, escritura condicional/checkpoint existente |
| X-CGR-03 | 🟠 | Referencias pendientes reintentables con contexto suficiente | `vantadb-mcp/src/code_index.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: las refs sin resolver conservan refName/refKind/posición y estado bound/ambiguous/pending/failed/unsupported, de modo que borrar una definición no convierte callers en «nunca existieron». Por qué: `all_indexed_symbols` solo reúne archivos procesados y `references` no lleva contrato de unresolved; un rebind ambiguo puede unir sobrecargas equivocadas. Acción mínima: extender `index_repo` y el consumo de `code_callers`/`code_callees` con reconcile por generación y reporte de resolución, sin segundo resolver obligatorio. Trazabilidad: informe §Ficha C · evidencia `src/db/schema.sql:19-143`, `src/extraction/index.ts:3896-3940` | X-CGR-02, C12 | Generación/identidad de C12, coverage del productor, IDs portables de referencia |
| X-CGR-04 | 🟡 | Límite dinámico declarado como resultado, no arista inventada | `vantadb-mcp/src/code.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: cuando un recorrido de `code_impact` termina en dispatch no resoluble, la respuesta declara el límite con razón, candidatos y coverage en lugar de «no hay camino». Por qué: la ausencia de relación confirmada no es ausencia de llamada; un candidato heurístico nunca puede participar como call confirmado. Acción mínima: extender la respuesta y el assembly existente con stopping_ref/razón/cobertura; shortlist opcional solo con lector de fuente autorizado y presupuesto. Trazabilidad: informe §Ficha D · evidencia `src/graph/dynamic-boundary-report.ts:1-109` | X-CGR-01 (frescura del cuerpo leído), C12 | Tipado de relaciones existente, presupuesto/cancelación del host; no depende de LLM |
| X-CGR-05 | 🟡 | Puerto visual de evidencia con errores y drift explícitos | `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: la vista de evidencia en Studio distingue not-found/refused/drift/current y desactiva marcas por línea indexadas cuando se muestran bytes actuales fuera de la revisión. Por qué: mostrar otro símbolo bajo un rango antiguo presenta evidencia equivocada como actual; un error no debe parecer pantalla vacía. Acción mínima: proponer `read_code_view`/`open_evidence` sobre la API existente con resolución de revisión y render de estado/errores; descartar respuestas de selección de generación anterior. Trazabilidad: informe §Ficha E · evidencia `ui/src/lib/adapter.ts:1-125`, `ui/src/App.svelte:1-80` | X-CGR-01, X-CGR-02, C12, C16 | Frescura/revisión de X-CGR-01, Studio existente en `develop-desktop`, trabajo frontend separado |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/codegraph.md` (Unidad A, Unidad B,
> Fichas C/D/E y §12.1). Todos los contratos son **hipotéticos** — propuestas de
> comportamiento para VantaDB, no APIs existentes ni puertos de código upstream.
> Las referencias `archivo:línea` son del clon `C:\Users\Eros\codegraph` en SHA
> `f72c9d2efb14454ff4f9fd7b8460abd82f19a287`.

### X-CGR-01 — Evidencia entregada ligada a revisión verificable

- **Problema:** un resultado indexado puede apuntar a líneas que ya contienen otro símbolo o texto; un deadline vencido no significa que la fuente esté fresca. VantaDB ya hashea al indexar; el delta es verificar **lo entregado**, no lo indexado.
- **Contrato hipotético (tipos propuestos):** `EvidenceSlice { source_id, revision, range, content_digest }`; `Freshness` distingue `verified`/`stale`/`unchecked`. Interfaz de aplicación `verify_sources(VerificationRequest) -> VerificationReport` que usa lectores de fuente autorizados; el dominio puro interpreta el resultado. `EvidenceSlice`, recibos e informes de consulta son vistas de aplicación de `CodeEvidenceImport` (C12), no otros DTO persistentes. No se afirma que estos sean tipos actuales de VantaDB.
- **Fuente de evidencia:** `src/mcp/answer-freshness.ts:1-66` (C5, `validateAnswerFiles` con cantidad/bytes/tiempo acotados; distingue `stale` de `unchecked`), `ui/src/lib/adapter.ts:1-125` (C9, `SourceRequest.ondrift`), caller `src/mcp/tools.ts:3114-3203` (solo verifica con watcher before/after degraded, sin error y con contribuyentes — no es verificación universal byte-a-byte de todo MCP).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs`; provenance/ingestors existentes; diagnóstico de retrieval de Studio.
- **Acciones:**
  1. Mapear revision/provenance actuales del pipeline de entrega.
  2. Llevar el digest de la generación recuperada hasta el assembler.
  3. Verificar contribuyentes con presupuesto de trabajo acotado, sobre la generación inmutable y bytes de la misma lectura.
  4. No renderizar rango antiguo como texto actual; exponer alternativa «bytes actuales» rotulada sin marcas indexadas.
  5. Publicar el informe solo si corresponde a esa generación; el rollback desactiva la verificación sin anunciar `verified`.
  6. Emitir métricas por estado/causa, nunca snippets.
- **Dependencias:** lectores de fuente autorizados, retención/acceso a fuente, lectura consistente, path guard y límites de trabajo; catálogo de generación de X-CGR-02; contrato C12.
- **FMEA:** TOCTOU entre hash y lectura; timeout confundido con fresh; hash de bytes distinto del texto normalizado; symlink que sale de la raíz; presupuesto agotado que oculta contribuyentes sin comprobar.
- **Criterios de aceptación propuestos:** cambio de fuente produce `stale` o lectura explícitamente actual; error/timeout produce `unchecked`, no `verified`; rango antiguo nunca se aplica silenciosamente a otro texto; scope/root no se amplían; todos los contribuyentes quedan enumerados.
- **Criterio negativo:** un timeout o error de lectura nunca puede reportarse como `verified`; la verificación no escribe ni reindexa.
- **Independencia:** depende del catálogo/mapping de generación de X-CGR-02 pero no de instalar un watcher; es una verificación de lectura, no una pipeline de ingesta.
- **No objetivos:** copiar constantes upstream (200 archivos/8 MiB/250 ms); garantizar snapshot de filesystem sin soporte; reindexar automáticamente sin permiso.

### X-CGR-02 — Publicación recuperable de generación por fuente

- **Problema:** una reingesta puede borrar evidencia válida o perder relaciones entrantes aunque solo haya cambiado una fuente; borrar evidencia válida o perder relaciones entrantes rompe provenance y supersession.
- **Contrato hipotético (tipos propuestos):** `CodeEvidenceImport` (C12 de CONSOLIDATED): identidad `repo/revision/producer/schema_version/generation`; refs, relation, direction; evidencia `Ast|Heuristic|Model|Unknown` con span/hash/reported_weight. La generación, revisión y productor pertenecen a su identidad; registros, relaciones y evidencia no constituyen un segundo `ImportBundle`. Interfaz `publish_generation(request) -> Result<Receipt, PublishError>`: la extracción produce candidatos y la aplicación coordina el commit; el recibo describe visibilidad/limpieza, no otro almacén.
- **Fuente de evidencia:** `src/extraction/index.ts:3319-3480` (C2, `storeExtractionResult`: no sustituye evidencia válida por grammar-load failure; captura aristas entrantes e identidades previas antes de borrar; bundle pequeño en transacción, grande en bloques de 2000 con yields — sin atomicidad global), `src/index.ts:1110-1169` (C3, lock/reapertura del índice; base reemplazada sin estado devuelve contención, no sync sobre handle obsoleto), `src/extraction/index.ts:3503-3515` (file record al final).
- **Seam VantaDB:** `vanta-memory/src/ingest/merge.rs`, writer/provenance, scheduler/checkpoints existentes. Evitar una segunda cola o almacenamiento paralelo.
- **Acciones:**
  1. Inventariar la atomicidad actual del coordinador de ingesta.
  2. Preparar la nueva generación como staging lógico.
  3. Validar relaciones/endpoints y límites antes del commit de visibilidad.
  4. Publicar mediante el commit vigente con CAS de generación (solo el producer/repo dueño publica y baja).
  5. Invalidar referencias por revisión; reemparejar relaciones entrantes de forma trazable.
  6. Recuperar cleanup/reintentos; diferenciar extractor fallido de fuente realmente vacía.
- **Dependencias:** IDs estables de fuente y relación, retención de generación previa, escritura condicional/checkpoint existente, C12. Si la API no permite batch indivisible, staging lógico y puntero quedan pendientes de diseño — no es garantía asumida.
- **FMEA:** marker vacío presentado como importación correcta; chunk parcial visible como completo; dos writers que publican generaciones invertidas; reattach por nombre ambiguo que une sobrecargas equivocadas; borrado que deja refs colgadas.
- **Criterios de aceptación propuestos:** extracción fallida conserva la última evidencia utilizable con estado explícito; retry converge; relaciones entrantes se mantienen o invalidan de forma trazable, nunca desaparecen silenciosamente; cleanup pendiente no finge operación íntegra. Aceptación manual adicional del informe: reader concurrente solo ve la generación publicada; cancelación antes del commit no adelanta baseline; repetir el lote converge; fallo de cleanup es visible sin borrar memorias humanas.
- **Criterio negativo:** una publicación vieja nunca pisa una nueva (CAS de generación); un marker vacío no cuenta como éxito de importación.
- **Independencia:** depende de la identidad y del writer real de VantaDB, no del RMW global de checkpoints por analogía; VantaDB ya reconcilia símbolos, esto extiende el coordinador existente — no es una nueva ingesta.
- **No objetivos:** afirmar que VantaDB ya falla en estos casos; copiar el mecanismo de IDs por línea ni los chunks upstream; migrar datos legacy sin owner por inferencia (el modo inicial sería diagnóstico sin bajas).

### X-CGR-03 — Referencias pendientes reintentables con contexto suficiente

- **Problema:** borrar una definición sin tocar callers no debe convertir sus referencias en «nunca existieron»; un rebind ambiguo puede unir sobrecargas equivocadas.
- **Contrato hipotético (tipos propuestos):** `reconcile_refs(import: &CodeEvidenceImport, expected_generation) -> Result<ResolutionReport, ReconcileError>`. El report separa `bound`/`ambiguous`/`pending`/`failed`/`unsupported`; los candidatos no son edges confirmados. `WrongRevision`, `MissingSource`, `IncompleteManifest` y `Conflict` no se traducen a cero callers.
- **Fuente de evidencia:** `src/db/schema.sql:19-143` (C1, `unresolved_refs` conserva nombre original, tipo, posición, archivo/lenguaje, candidatos y estado pending/failed/name_tail; el resolver elimina refs resueltas y conserva failed para retry), `src/extraction/index.ts:3896-3940` (`removeTracked` comprueba existencia física y resucita refs originales de edges entrantes antes de `deleteFile`; edges legacy/sintetizados sin `refName` pueden caer — inventar short-name quitaría contexto receptor/qualifier).
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs` (`all_indexed_symbols` solo reúne archivos procesados; `references` no lleva contrato de unresolved); extender `index_repo` y el consumo de `code_callers`/`code_callees`, no crear un segundo resolver obligatorio.
- **Acciones:**
  1. Mapear refs originales del productor.
  2. Invalidar bindings propios cuyo target cambió.
  3. Pedir evidencia al productor para refs sin material suficiente.
  4. Validar receptor/import antes de publicar.
  5. Publicar la resolución dentro de la generación.
  6. Reintentar solo pendientes relevantes; el legacy sin callee original queda `unknown`, no se reconstruye desde display name.
- **Dependencias:** generación/identidad de X-CGR-02 y C12, coverage del productor, IDs portables de referencia. Alternativas del informe: full reimport si no hay ref portable; índice invertido incremental solo después. Rollback: volver al mapping previo marcado por revisión.
- **FMEA:** homónimos tras delete → rebind equivocado → detectar candidatos múltiples y retener la ambigüedad; retry infinito → jobs repetidos → estado observable y retry gobernado; otro productor borra el endpoint → ownership estricto.
- **Criterios de aceptación propuestos:** delete/reintroducción, overloads y line shift mantienen identidad/evidencia o explican `unknown`; retry idempotente; logs solo con conteos por estado/generación.
- **Criterio negativo:** los errores de reconcile (`WrongRevision`, `MissingSource`, `IncompleteManifest`, `Conflict`) nunca se traducen a «cero callers»; un candidato ambiguo nunca se promociona a edge confirmado.
- **Independencia:** no exige un segundo resolver ni portar parsers; depende de X-CGR-02/C12 y de la cobertura del productor.
- **No objetivos:** inferir dispatch en runtime; recuperar información que el export nunca conservó.

### X-CGR-04 — Límite dinámico declarado como resultado, no arista inventada

- **Problema:** cuando un recorrido de `code_impact` termina en dispatch no resoluble, «no hay camino» es demasiado fuerte: la ausencia de relación confirmada no es ausencia de llamada.
- **Contrato hipotético (tipos propuestos):** `explain_boundary(snapshot, seeds, authorized_source_reader, budget) -> Result<BoundaryReport, QueryError>` con `stopping_ref`, razón, candidatos y coverage. El reporte es efímero; reutiliza C12 para refs a evidencia.
- **Fuente de evidencia:** `src/graph/dynamic-boundary-report.ts:1-109` (define `BoundarySite`/`BoundaryCandidate`/continuations, lectura acotada de cuerpos y representación compartida MCP/viewer; declara explícitamente que no muta el grafo).
- **Seam VantaDB:** `vantadb-mcp/src/code.rs` (`code_impact` ya tiene depth/direction; no acredita reporte de cierre semántico). Extender la respuesta y el assembly existente, no añadir calls especulativas.
- **Acciones:**
  1. Fijar la generación de consulta.
  2. Recorrer las relaciones admitidas.
  3. Informar dónde termina la evidencia (stopping_ref + razón + coverage).
  4. Shortlist opcional de candidatos solo con lector de fuente autorizado y presupuesto.
  5. Presupuestar la salida; sin escrituras ni commit.
- **Dependencias:** X-CGR-01 (frescura del cuerpo leído), tipado de relaciones existente, presupuesto/cancelación del host; no depende de LLM. La shortlist es opcional si el productor conserva dispatch key.
- **FMEA:** regex que confunde strings con llamadas → candidato erróneo → rótulo heurístico y fuente; hub que agota el presupuesto → omission explícita; cuerpo stale → rechazo/unchecked por X-CGR-01.
- **Criterios de aceptación propuestos:** un callback desconocido devuelve límite declarado; una llamada real se separa del candidato; cancelación y query vacía no prueban ausencia. Métricas de límites/omisiones, sin código en logs.
- **Criterio negativo:** un candidato heurístico nunca puede participar como call confirmado; la cancelación devuelve `incomplete`, no un path inexistente.
- **Independencia:** «solo avisar unsupported» es el mínimo sostenible; desactivar la shortlist deja el límite informado, por lo que la candidata es útil incluso sin lector de fuente.
- **No objetivos:** copiar constantes upstream, inventar edges, seleccionar tests.

### X-CGR-05 — Puerto visual de evidencia con errores y drift explícitos

- **Problema:** mostrar otro símbolo bajo un rango antiguo presenta evidencia equivocada como actual; un error de lectura no debe parecer pantalla vacía.
- **Contrato hipotético (tipos propuestos):** `read_code_view(auth, generation, selection) -> Result<CodeView, ViewError>` y `open_evidence(ref)` sobre la API existente; la vista distingue `not-found`/`refused`/`drift`/`current`.
- **Fuente de evidencia:** `ui/src/lib/adapter.ts:1-125` (C9, `GraphAdapter`/`ApiFailure`/`SourceRequest.ondrift` — mismo lector con transportes diferentes, no obliga a HTTP), `ui/src/App.svelte:1-80` (C10, vistas Symbol/File/Map/Flow con canal live y recarga al cambiar el índice). Límite: `src/bin/viewer-gate.ts:1-29` exige `CODEGRAPH_UI=1`; `ui/package.json` es private (Svelte5/XYFlow/Vite) — fuente presente ≠ viewer lanzado o paquete publicado.
- **Seam VantaDB:** `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` — Studio Tauri/React ya existe en `develop-desktop` con diagnóstico de retrieval según VANTA-REFERENCE; se amplía, no se crea otra consola.
- **Acciones:**
  1. Autorizar el repo antes de leer.
  2. Resolver la referencia vía C12.
  3. Obtener bytes/rango de la revisión indexada.
  4. Renderizar estado y errores explícitos.
  5. Desactivar marcas por línea indexadas cuando se eligen bytes actuales fuera de la revisión.
  6. Descartar respuestas de una selección de generación anterior; cancelar navegación no cancela el indexador compartido.
- **Dependencias:** frescura/revisión de X-CGR-01 y publicación de X-CGR-02, C12/C16, Studio existente, trabajo frontend separado. Alternativa mínima del informe: enlaces a fuente.
- **FMEA:** symlink/ruta ajena → denegación; error presentado como pantalla vacía; anotaciones indexadas aplicadas a bytes actuales stale; selección vieja que pisa una nueva.
- **Criterios de aceptación propuestos:** symlink/ruta ajena se deniega; un error nunca parece pantalla vacía; stale no aplica anotaciones indexadas a bytes actuales; una selección vieja no pisa una nueva. Observabilidad: errores por clase, sin roots ni contenido sensible.
- **Criterio negativo:** una selección de generación anterior nunca sobrescribe la vista de la selección vigente; desactivar el panel no toca la evidencia persistida.
- **Independencia:** requiere X-CGR-01/X-CGR-02 para la semántica de revisión; la variante mínima (enlaces a fuente) no requiere el adapter visual completo.
- **No objetivos:** importar Svelte, otro servidor loopback o el producto Pro; declarar un endpoint de Studio ya disponible; acreditar app desktop nativa, a11y o seguridad frontend.

<!-- audit: 2026-11-23 normalizada referencia residual B -> X-CGR-02 en X-CGR-03 -->
