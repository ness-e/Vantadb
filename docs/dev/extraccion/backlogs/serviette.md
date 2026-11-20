---
title: "Backlog de extracción — Serviette"
kind: research
status: active
description: "Cuatro candidatas del informe Serviette (fingerprint de derivados, manifest/retirada, lifecycle de parse y evidencia documental) como filas del esquema canónico de 10 columnas"
tags: [extraccion, backlog, serviette]
---

# Backlog de extracción — Serviette

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/serviette.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `X-SRV-01` | 🟠 | Fingerprint de reproducibilidad de derivados | `src/wiki/chunker.rs` · `src/wiki/state.rs` · `src/wiki/ingestors.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: fingerprint de derivación (parser, splitter, embedding, normalización, versión de implementación) persistido por generación; el replay incompatible se rechaza con diagnóstico y rebuild explícito en vez de continuar. · Por qué: calcular bajas con un splitter distinto al de las altas produce derivados huérfanos que ya no se pueden retirar; una rotación de clave no debe invalidar la generación. · Acción mínima: identificar IDs/versiones existentes, persistir fingerprint con la generación y comprobar compatibilidad antes de cualquier replay; secretos fuera del digest. · Trazabilidad: informe §2/Ficha A · evidencia `serviette/indexer/fingerprint.py:145-225` (clon `C:\Users\Eros\serviette`). | X-SRV-02 · C4 · C14 | Serialización canónica estable, registro de fuentes, migración y procedencia; un ingestor existente, sin segundo pipeline |
| `X-SRV-02` | 🟠 | Manifest de derivados y retirada incremental por revisión | `vanta-memory/src/ingest/merge.rs` · `vanta-memory/src/core/record/lifecycle.rs` · `vanta-memory/src/utils/task_checkpoint.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: manifest por revisión que materializa el conjunto publicado de IDs derivados y permite retirarlos sin releer los bytes de una fuente ya borrada; una revisión vieja nunca elimina la sucesora. · Por qué: la baja de un documento debe poder deshacer sus derivados aunque el original ya no exista; una actualización parcial no puede dejar el índice mezclando generaciones. · Acción mínima: contrastar el manifest actual, almacenar el conjunto de IDs publicado por generación y retirar/publicar por revisión con fence. · Trazabilidad: informe §2/Ficha B · evidencia `serviette/indexer/graph.py:1-35`, `serviette/indexer/graph.py:550-660`, `serviette/indexer/graph.py:976-1028`. | X-SRV-01 · X-SRV-03 · C4 · C14 | Transacción e índices consistentes; supersession/lifecycle existentes; scheduler reutilizado, embedding opt-in |
| `X-SRV-03` | 🟡 | Lifecycle de ingesta con outcome de parse y cancelación observable | `vanta-memory/src/ingest/worker.rs` · `vanta-memory/src/ingest/callback.rs` · `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: outcome tipado por etapa (`ParsedEmpty`/`Parsed`/`ParseFailed`/`ProjectionPending`/`CancelRequested`) que distingue fuente vacía de parser fallido y solicitud de cancelación de cancelación efectiva. · Por qué: un `ParseError` convertido en texto vacío hace desaparecer un documento y el consumidor confunde ausencia con contenido irrelevante; `solicitado ≠ cancelado` y `parseado ≠ visible`. · Acción mínima: mapear los enums actuales a transiciones y resultados parciales, persistir sólo lo que el checkpoint soporte realmente y propagar a SDK/Studio con migración aditiva `unknown` para legado. · Trazabilidad: informe §11.3/Candidata C · evidencia `serviette/indexer/graph.py:550-668`, `serviette/indexer/graph.py:976-1028`, `serviette/server/main.py:97-143`, `serviette/up.py:371-449`. | X-SRV-02 · C9 · C16 | Worker/callback/scheduler/checkpoints existentes; B aporta visibilidad de derivados; C16 presenta fases en Studio |
| `X-SRV-04` | 🟡 | Evidencia documental con cobertura declarada y presupuesto compartido | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: vista de evidencia por documento/revisión con spans incluidos y omitidos, unidad de presupuesto declarada, cobertura y modo comparativo ligado a revisión. · Por qué: comparar revisiones extensas no debe presentar un prefijo como si representase todo el documento ni gastar el presupuesto dos veces en solapes; un cap de caracteres no es un presupuesto de tokens. · Acción mínima: resolver IDs/revisión autorizados, preservar spans al recortar y emitir coverage tras render sobre la ruta de assembly existente. · Trazabilidad: informe §11.4/Candidata D · evidencia `serviette/server/documents.py:1-65`, `serviette/server/main.py:517-630`, `serviette/server/main.py:335-422`. | X-SRV-03 · C4 · B · C7 · C14 · C16 | C4/B para revisiones de fuente; C7 presupuesto compartido; C14 stale; C16 UI; independiente del dataflow upstream |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/serviette.md` (§2 fichas A/B + §11.2, §11.3 candidata C y §11.4 candidata D). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `84e92a3` (`C:\Users\Eros\serviette`); las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD.

### X-SRV-01 — Fingerprint de reproducibilidad de derivados

- **Problema:** calcular bajas con un splitter distinto al usado para las altas produce derivados huérfanos que ya no se pueden retirar; el drift de configuración/librería invalida el replay sin diagnóstico y una rotación de clave no debe invalidar la generación.
- **Contrato hipotético:** `DerivationFingerprint { parser, splitter, embedding, normalization, implementation_version }`; `inspect_compatibility(source_id, expected_revision, stored, current) -> Result<ReplayCompatibility, FingerprintError>` con salida `Compatible | RebuildRequired | UnknownEvidence`; `begin_generation`, `rebuild_required`. Errores: `CorruptFingerprint | UnknownLegacy | RevisionConflict | ReadFailed`. Invariantes: digest sin secretos y mismo espacio vectorial, no sólo misma dimensión.
- **Fuente de evidencia (clone):** `serviette/indexer/fingerprint.py:145-225` (`build_fingerprint`/`check_fingerprint`: config y versiones condicionan replay, drift requiere confirmación; la reescritura automática del fingerprint ilegible en `:199-204` es el anti-patrón a no copiar); `serviette/indexer/graph.py:721-761` (`_metadata_key_json` excluye sólo `seen_at`; `CHUNK_ID_SCHEME=2`: ID SHA256 = metadata de versión + separador nulo + texto — excluir la observación evita IDs nuevos por sondeo).
- **Seam VantaDB:** `src/wiki/chunker.rs::chunk_text` (:19), `src/wiki/ingestors.rs::Ingestor` (:45-89) y `src/wiki/state.rs::WikiState` — extender metadata del ingestor/estado existente; no crear un segundo pipeline de ingesta.
- **Acciones:**
  1. Inventariar identidad actual: IDs/versiones existentes en chunker, ingestors y estado.
  2. Definir serialización canónica estable de los campos del fingerprint (secretos fuera del digest).
  3. Persistir el fingerprint ligado a la generación.
  4. Comprobar compatibilidad antes de cualquier replay; legado `unknown` exige evidencia o rebuild explícito.
  5. Rechazar replay incompatible con diagnóstico y ofrecer rebuild explícito en Studio. Rollback: deshabilitar producción nueva conservando el fingerprint anterior.
- **Dependencias:** serialización canónica estable, registro de fuentes, migración y procedencia; un ingestor existente; X-SRV-02 consume la generación para el manifest; C4/C14 (revisiones). Ningún servicio obligatorio.
- **FMEA:** librería cambia silenciosamente → IDs distintos al replay → chunks huérfanos (detectar fingerprint y forzar decisión, no sobrescribir corrupto como compatible); identidad incluye credenciales → filtración (digest sin secretos); fingerprint ilegible → falsa compatibilidad (fallar cerrado conservando diagnóstico).
- **Criterios de aceptación propuestos:** cambio de splitter exige nueva generación; rotar clave no invalida la generación; restart idéntico mantiene IDs; fingerprint corrupto no habilita replay. Adversa: drift, corrupto, rotación de clave. Observabilidad: digest/revisión/estado, sin valores de config sensibles. Pendiente de ejecución manual.
- **Criterio negativo:** un fingerprint ilegible o en drift nunca se reescribe ni se trata como compatible; ningún secreto ni timestamp de observación entra al digest de identidad.
- **Independencia:** aplicable a un único ingestor existente sin manifest completo ni dataflow upstream; es base de X-SRV-02.
- **No objetivos:** fijar versiones globales por capricho, versionar el entorno entero ni reproducir Pathway; la lista canónica de campos/model identity queda abierta.

### X-SRV-02 — Manifest de derivados y retirada incremental por revisión

- **Problema:** la baja de un documento debe poder deshacer sus derivados aunque el original ya no exista; una actualización parcial no puede dejar el índice mezclando generaciones y una revisión vieja nunca puede eliminar la sucesora.
- **Contrato hipotético:** `SourceDelta { scope, source_id, revision, kind }` y `DerivedManifest { revision, chunk_ids, fingerprint }`; `plan_delta(SourceDelta, expected_revision, manifest) -> Result<DeltaPlan, DeltaError>` y `apply(plan) -> DeltaPublication` con IDs retirados/publicados y pendientes; operaciones `publish_revision`, `retract_revision`, `inspect_pending`. Errores: `Denied | MissingManifest | RevisionConflict | ParseFailed | ProjectionPending | Cancelled`. Invariante: vacío confirmado distinto de lectura fallida.
- **Fuente de evidencia (clone):** `serviette/indexer/graph.py:1-35` (dataflow y retractions: no releer bytes eliminados para deshacer derivados), `serviette/indexer/graph.py:550-660` (`parse_document`, `split_text`, `make_id`; splitter puro sólo en whitelist; flatten y embedding), `serviette/indexer/graph.py:976-1028` (`run_indexer`: verifica fingerprint antes de preparar backend y corre `pw.run` con persistencia).
- **Seam VantaDB:** `vanta-memory/src/ingest/merge.rs::commit` (:239-289), `vanta-memory/src/core/record/lifecycle.rs`, `vanta-memory/src/utils/task_checkpoint.rs::TaskCheckpointManager` — extender provenance/manifest existentes; una transacción sólo si cubre la unidad, nunca nuevo sink/runtime/WAL.
- **Acciones:**
  1. Contrastar el manifest actual y capturarlo antes de perder la fuente.
  2. Almacenar el conjunto de IDs publicado por generación.
  3. Validar `expected_revision` y planificar el delta.
  4. Publicar/retirar por ese conjunto y revisión con fence sobre el backend existente.
  5. Proyectar/retirar derivados y avanzar el checkpoint. Campos aditivos; no inventar manifest legacy. Rollback: detiene trabajo nuevo, termina/repara operaciones ya iniciadas.
- **Dependencias:** transacción e índices consistentes; supersession/lifecycle existentes; scheduler reutilizado (reconciliación sin Redis/Pathway en core); embedding opt-in; X-SRV-01 (fingerprint de la generación); X-SRV-03 consume los estados.
- **FMEA:** baja repetida elimina revisión nueva → fence por revisión (retry viejo no elimina sucesora); parse fallido confundido con documento vacío → estado de error distinto y conservación de la generación previa según política explícita; update parcial deja índice mixto → publicación de manifest consistente por revisión.
- **Criterios de aceptación propuestos:** alta/edición/baja/replay; retirada sin bytes fuente; parser fallido conserva diagnóstico y no finge éxito; revisión vieja no borra nueva. Adversa: cancel/replay/crash/sucesora. Observabilidad: operación/fase/conteos/error, sin texto.
- **Criterio negativo:** una revisión vieja nunca elimina la sucesora; un parse fallido nunca retira derivados válidos como si la fuente estuviera legítimamente vacía.
- **Independencia:** requiere X-SRV-01 (fingerprint) y transacción/índices consistentes; una operación identificada por fuente/revisión con serialización/CAS existente.
- **No objetivos:** watch universal de todos los conectores ni semántica distribuida exactly-once no demostrada; no exactly-once de red; visibilidad y retención de generaciones abiertas.

### X-SRV-03 — Lifecycle de ingesta con outcome de parse y cancelación observable

- **Problema:** una captura aceptada no aparece en recall y el usuario necesita saber si está vacía, falló el parser, espera índice o pidió cancelar; un `ParseError` convertido en texto vacío hace desaparecer un documento y el consumidor confunde ausencia con contenido irrelevante.
- **Contrato hipotético:** `record_stage(operation_id, source_revision, expected_phase, outcome) -> Result<IngestStageView, StageError>`; outcome tipado `ParsedEmpty | Parsed | ParseFailed | ProjectionPending | CancelRequested`; errores `RevisionConflict | OutOfOrder | StorageFailed | UnknownOperation`. Invariantes: `solicitado ≠ cancelado`, `parseado ≠ visible`, `health ≠ ready`.
- **Fuente de evidencia (clone):** `serviette/indexer/graph.py:550-668` (`make_parse_udf`: `ParseError` devuelve vacío y log; `split_text("")` produce cero chunks indistinguibles de una fuente legítimamente vacía), `serviette/indexer/graph.py:976-1028` (runtime/persistencia del indexer), `serviette/server/main.py:97-143` (`_IndexChangeTracker` — stats heurísticas, no máquina durable), `serviette/up.py:371-449` (supervisión de procesos: cancelación del supervisor, no API por documento ni rollback).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs::{begin, execute}`, `vanta-memory/src/ingest/callback.rs::{IngestProgress, ProgressTracker::update_progress}`, `vanta-memory/src/utils/task_checkpoint.rs::{advance, fail}`; presentación propuesta en Studio sin atribuir JobChip/jobs al pin (SE-1, informe §12.4).
- **Acciones:**
  1. Inventariar los enums actuales de progreso y resultado.
  2. Mapear transiciones y resultados parciales a los outcomes propuestos.
  3. Persistir sólo lo que el checkpoint soporte realmente.
  4. Propagar a SDK/Studio con migración aditiva `unknown` para legado.
  5. Reconciliar la fase al reintentar. Rollback: oculta la vista nueva, nunca reetiqueta un fallo como éxito.
- **Dependencias:** worker/callback/scheduler/checkpoints existentes; C9 transversal; X-SRV-02 aporta visibilidad de derivados; revisión (X-SRV-01/C4); C16 presenta fases en Studio. No exige modelo ni cambio del motor.
- **FMEA:** parser excepciona → vacío → corpus parece completo (detectar outcome antes de publicar y exponer el fallo); cancel tardío → commit realizado → UI dice revertido (detectar applied IDs/revisión y mostrar parcial); stats regresan tras delete → timestamp falso de frescura (separar observación local de checkpoint autoritativo, sin convertir wall clock en revisión).
- **Criterios de aceptación propuestos:** normal captura→parse→índice; adversa: vacío real, parser roto, fallo al persistir estado, cancel antes/después de chunk y dos runs. Telemetría: IDs/fases/conteos/latencia/error class, sin nombres ni texto.
- **Criterio negativo:** `ParseFailed` nunca se presenta como `ParsedEmpty`; `CancelRequested` nunca se anuncia como cancelación efectiva ni como rollback de lo ya aplicado.
- **Independencia:** un parser y worker existente; ejecutable en paralelo a X-SRV-01/X-SRV-02 sobre el worker actual; no requiere Pathway ni LLM.
- **No objetivos:** no detener procesos globales para cancelar un documento ni añadir cola/scheduler adicional. Abiertos: retención de errores y garantías del checkpoint al crash; no se presume durabilidad de jobs de UI.

### X-SRV-04 — Evidencia documental con cobertura declarada y presupuesto compartido

- **Problema:** comparar revisiones extensas no debe presentar un prefijo como si representase todo el documento ni gastar el presupuesto dos veces en solapes; un cap de caracteres no es un presupuesto de tokens.
- **Contrato hipotético:** `document_evidence(source_refs, expected_revisions, scope, remaining_budget) -> Result<DocumentEvidenceView, EvidenceViewError>`; salida: spans incluidos/omitidos, unidad de presupuesto, coverage y modo. Errores: `Denied | AmbiguousSource | StaleRevision | ReadFailed | BudgetExhausted`. Invariante: no llamar completo a truncado ni usar el nombre como autorización.
- **Fuente de evidencia (clone):** `serviette/server/documents.py:1-65` (catálogo, lectura de documento y diff con presupuesto de caracteres; dos versiones se tratan como dos documentos), `serviette/server/main.py:517-630` (`_answer_documents`: contextos por documento/comparación; `RagResponse.notice/mode`), `serviette/server/main.py:335-422` (`_search`: presupuesto por request en modo documental).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` — `CandidatePage.sources`, `RecallScope`/`InjectionPolicy`, `assemble_with_recall`/`IntegratedContext`; inspección de fuente en Studio. No otro context assembler.
- **Acciones:**
  1. Resolver IDs/revisión autorizados.
  2. Cargar sólo lo autorizado.
  3. Preservar spans al recortar.
  4. Ensamblar en la ruta de assembly existente.
  5. Emitir coverage tras render. Persistencia opcional de la vista (no copia del documento); legado `unknown`. Rollback: desactiva la comparación y conserva retrieval ordinario.
- **Dependencias:** C4/X-SRV-02 para revisiones de fuente; C7 presupuesto compartido; C14 stale; C16 UI; X-SRV-03 (outcomes). Independiente del dataflow upstream y del backend externo de Serviette.
- **FMEA:** versión homónima → mezcla documentos → diff incorrecto (detectar identidad no única y exigir selección); cap de caracteres tratado como tokens → ventana desborda (declarar estimador y presupuesto real existente).
- **Criterios de aceptación propuestos:** normal: dos revisiones autorizadas; adversa: truncado, Unicode, versión retirada, lectura IO y presupuesto insuficiente. Observar IDs/revisiones/span counts/coverage — nunca preguntas, diferencias ni contenido.
- **Criterio negativo:** una vista truncada nunca se declara completa; el nombre del documento nunca sirve como autorización de lectura.
- **Independencia:** lectura de una o dos fuentes sin chat nuevo; última en el orden propuesto — sólo si se necesita comparación documental.
- **No objetivos:** pesos/umbrales de evaluación, nuevo diff semántico ni modelo obligatorio. Abiertos: política de conservación/cobertura por formato y ubicación de summaries.
