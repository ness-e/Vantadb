---
title: "Backlog de extracción — Dify"
kind: research
status: active
description: "Candidatas del informe Dify: plan efectivo de retrieval, diagnóstico con privacidad, publicación de derivados por revisión, metadata tipada y caché de embedding con identidad"
tags: [extraccion, backlog, dify]
---

# Backlog de extracción — Dify

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/dify.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-DIF-01 | 🟠 | Plan efectivo de retrieval con resultado por etapa | `vanta-memory/src/context_engine/engine.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: el reporte de búsqueda declara configuración solicitada, resuelta y aplicada por etapa (Skipped/Applied/Failed con razón y candidatos) y dominio de cada score. Por qué: una excepción economy → keyword o un reranker ausente no deben presentarse como el método mostrado; un fallback no promete orden ni calidad equivalentes. Acción mínima: cotejar planes/reportes actuales, preservar la configuración efectiva con referencia a su revisión y distinguir deshabilitado de no disponible. Trazabilidad: informe §Ficha A · evidencia `api/services/hit_testing_service.py:65-192`, `api/core/rag/datasource/retrieval_service.py:792-956` | X-DIF-02, C7, C4 | Metadata/config versionada, límites de trabajo y cancelación existentes; proveedores solo como adaptadores opcionales |
| X-DIF-02 | 🟠 | Diagnóstico de recuperación con privacidad explícita | `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: el evento de diagnóstico transporta referencias y metadatos con visibilidad MetadataOnly/PreviewAuthorized y exportación externa opt-in separada del core. Por qué: observabilidad no debe convertirse en copia secundaria de todos los documentos; spans con consulta/contenido y previews tras retirar acceso amplían la frontera de confianza. Acción mínima: comprobar si la auditoría actual ya cubre el evento, clasificar metadata sensible y resolver previews con autorización actual registrando su revocación. Trazabilidad: informe §Ficha B · evidencia `api/extensions/otel/parser/retrieval.py:1-105`, `api/services/hit_testing_service.py:65-192` | X-DIF-01, C8, C16 | Scope y autorización existentes, provenance, política de retención y canal de actividad; sin requisito de OpenTelemetry remoto |
| X-DIF-03 | 🔴 | Publicación y retirada de derivados por revisión de ingesta | `vanta-memory/src/ingest/worker.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: cada publicación/retirada de derivados lleva source_id, revisión de fuente esperada, attempt_id y recibo por recurso; un intento con revisión obsoleta se rechaza en lugar de revivir contenido sustituido. Por qué: `put_page` persiste sin run_id y la comprobación terminal de `WikiStore` no es fencing de cada write; un worker antiguo que termina después de una actualización puede publicar derivados obsoletos. Acción mínima: extender `IngestReport`/`CommitReport` y el callback hacia `WikiStore::put_page` conservando run_id/versión, validar vigencia antes de publicar y reconciliar intentos interrumpidos. Trazabilidad: informe §Ficha C · evidencia `api/services/knowledge/indexing/execution.py:77-178`, `api/tasks/document_indexing_update_task.py:36-110`, `api/tasks/clean_document_task.py:20-150` | X-DIF-04, X-DIF-05, C9, C14 | Provenance/supersession y checkpoints existentes, garantía de commit real disponible; no suponer CAS ni atomicidad multialmacén sin acreditarla |
| X-DIF-04 | 🟠 | Metadata validada por schema antes de persistir o compilar filtros | `src/wiki/ingestors.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: la escritura y el filtrado de metadata pasan por validación de schema/valor con representación canónica, revisión de schema y errores distinguibles (campo desconocido, tipo incompatible, fecha ambigua). Por qué: una sola fecha o número con representación incompatible puede invalidar la consulta de toda una base (issues #45638/#41597); `validate_metadata` actual rechaza claves reservadas/NUL pero no valida schema de campo ni unidad temporal. Acción mínima: ampliar la validación del adapter antes de asignar `MemoryInput.metadata` reutilizando `Value`, validar todo el batch antes del primer efecto y compilar filtros solo con valores validados. Trazabilidad: informe §Ficha D · evidencia `api/repositories/knowledge/metadata_repository.py:155-198`, `api/core/rag/retrieval/dataset_retrieval.py:1631-1760` | X-DIF-03, C4, C5 | Schema perteneciente al dueño autorizado de la fuente; no imponer schema global ni corregir legado silenciosamente |
| X-DIF-05 | 🟡 | Caché de embedding con identidad comprobable y correspondencia posicional | `vanta-memory/src/core/record/l1_writer.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: el adapter de modelo devuelve un outcome por entrada (Ready/Invalid/ProviderError) con input_id y content_digest, y la clave de caché incluye fingerprint de modelo, dimensiones y preprocesado. Por qué: un vector inválido no debe desplazar la asociación de los demás textos y la reutilización de cómputo no acredita vigencia ni autorización. Acción mínima: cotejar cachés existentes en los hooks de embedding de L1, resolver identidad antes de calcular y no compactar slots; dimensión/finiteza se validan antes del índice. Trazabilidad: informe §Ficha E · evidencia `api/core/rag/embedding/cached_embedding.py:24-133` | X-DIF-03, C15 | Ninguna exigencia de proveedor externo para motor local; caché como derivado descartable, nunca memoria canónica |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/dify.md` (Fichas A–E de §2 y
> compleción de A/B en §11). Todos los contratos son **hipotéticos** — propuestas
> de comportamiento para VantaDB, no APIs existentes ni puertos de código
> upstream. Las referencias `archivo:línea` son del clon `C:\Users\Eros\dify` en
> SHA `65b4e17d6ebeff39c7e66c832649afff01666adf`; las referencias `V*` fijan el
> snapshot VantaDB `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y `VS*` Studio en
> `develop-desktop` @ `0ad38c542`. La licencia upstream es Apache-2.0 modificada
> (condiciones multitenant y de frontend): contratos abstractos, no trasplantes.

### X-DIF-01 — Plan efectivo de retrieval con resultado por etapa

- **Problema:** configuración solicitada, resuelta y aplicada no son equivalentes: una excepción economy → keyword o un reranker ausente no deben presentarse como el método mostrado; un fallback no promete orden ni calidad equivalentes.
- **Contrato hipotético (tipos propuestos):** `EffectiveRetrievalPlan { scope, config_revision, requested_mode, applied_stages, score_domains, exclusions }`; `StageOutcome { stage, status: Skipped|Applied|Failed, reason, candidate_count }`. Operaciones propuestas: `resolve_plan`, `validate_capabilities`, `execute_stage`, `record_outcome`. Los scores conservan dominio, no se fusionan por nombre. Errores propuestos: configuración inválida, etapa no disponible, cancelado — no un `None` ambiguo.
- **Fuente de evidencia:** `api/services/hit_testing_service.py:65-192` (S2; `:119-164` resuelve override → config dataset → default y aplica excepción economy → keyword), `api/core/rag/datasource/retrieval_service.py:792-956` (S1; `:815-877` enruta keyword/semántica/fulltext según método, `:891-924` deduplica híbrido y aplica postproceso o threshold alternativo), `api/core/rag/data_post_processor/data_post_processor.py:1-134` (S3; permite ausencia de runner cuando no hay modelo válido o autorización).
- **Seam VantaDB:** `vanta-memory/src/context_engine/engine.rs` (assembler y reportes existentes, V2) y `src/sdk/search/hybrid.rs` (V1); SDK/MCP/servidor como frontera de validación. Alternativa del informe: extender el explain existente — rechazar registro paralelo de runs.
- **Acciones:**
  1. Cotejar planes/reportes actuales del retrieval híbrido.
  2. Capturar la configuración efectiva con referencia a su revisión antes de ejecutar (un override concurrente no reescribe el recibo del run previo).
  3. Declarar por etapa `Skipped`/`Applied`/`Failed` con razón y candidatos.
  4. Distinguir deshabilitado de no disponible; mostrar fallback sin prometer orden o calidad equivalentes.
  5. No crear valores de evaluación ni copiar los defaults top-k del producto.
- **Dependencias:** metadata/config versionada, límites de trabajo y cancelación existentes; proveedores LLM/rerank solo como adaptadores opcionales. No depende de X-DIF-02 ni de X-DIF-03/04/05 para funcionar.
- **FMEA:** configuración inválida → etapa omitida silenciosamente; score de distancia usado como similitud → filtrado invertido; error parcial → falso vacío; override mutante → diagnóstico irreproducible. Mitigar con enums/dominios, validación previa y revisión inmutable, no con diccionarios sin procedencia.
- **Criterios de aceptación propuestos (manuales):** mismo plan explica qué se ejecutó; deshabilitar rerank no requiere proveedor; fallo de etapa es distinguible de cero hits; consulta de diagnóstico no modifica por accidente configuración persistente. Casos del informe: configuración ausente/override/economy, reranker ausente, cancelación y exportador fallido; distinguir solicitado/resuelto/aplicado.
- **Criterio negativo:** una etapa omitida o fallida nunca se reporta como aplicada; el score de cada etapa conserva su dominio y no se reinterpreta para pasar un umbral.
- **Independencia:** funciona sin las demás candidatas; solo necesita explain/config versionada sobre el retrieval existente.
- **No objetivos:** reemplazar RRF por suma ponderada ni integrar la plataforma Dify; pools Flask ni proveedores como unidad extraíble.

### X-DIF-02 — Diagnóstico de recuperación con privacidad explícita

- **Problema:** explicar retrieval sin convertir observabilidad en copia secundaria de todos los documentos; spans con consulta/contenido y previews tras retirar acceso amplían la frontera de confianza.
- **Contrato hipotético (tipos propuestos):** `RetrievalDiagnostic { trace_id, scope, config_revision, source_refs, stage_outcomes, visibility: MetadataOnly|PreviewAuthorized, redaction_revision }` — el DTO contiene referencias; los previews se resuelven aparte. Operaciones: `capture_metadata`, `authorize_preview`, `redact`, `export_opt_in`. Errores propuestos: preview denegado, fallo de persistencia de diagnóstico.
- **Fuente de evidencia:** `api/extensions/otel/parser/retrieval.py:1-105` (S4; `:20-65` serializa contenido/metadata/score/ID, `:86-105` coloca consulta y documentos en atributos de span — acredita payload, no destino efectivo), `api/services/hit_testing_service.py:65-192` (S2; registra consultas de hit testing mediante `DatasetQuery` y `session.flush`), `web/app/components/datasets/hit-testing/index.tsx:1-165` (S5; historial de consultas y resultados, diferenciando fuente externa).
- **Seam VantaDB:** `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` (VS1, `:1-130`: ya ofrece consulta, vector-picker, filtros, ScoreBars y contexto) y auditoría/diagnóstico retrieval + canal de actividad existentes; exportador externo opt-in fuera del core.
- **Acciones:**
  1. Comprobar si la auditoría actual ya cubre el evento de diagnóstico.
  2. Clasificar metadata sensible; mantener identidad estable sin exportar texto por defecto.
  3. Resolver previews con la autorización actual y registrar aparte su revocación.
  4. Explicar qué consultas se guardan y quién puede inspeccionarlas.
  5. Separar éxito del retrieval de fallo de observabilidad; persistencia/export de diagnóstico no es commit del retrieval.
- **Dependencias:** scope y autorización existentes, provenance, política de retención y canal de actividad; sin requisito de OpenTelemetry remoto. Publica metadata al dueño de diagnóstico, no conserva el texto por sí mismo; retención de auditoría/trazas no hereda la del historial conversacional.
- **FMEA:** traza cruza tenant → fuga; referencia firmada expira → vista rota; documento retirado → preview histórico no autorizado; exportador falla → consulta bloqueada. Mitigar con resolución autorizada actual, redacción previa y separación entre éxito del retrieval y fallo de observabilidad.
- **Criterios de aceptación propuestos:** metadata-only no contiene texto/secretos; retirar acceso impide preview; error de exportación es visible sin perder resultados; se identifica revisión de configuración sin guardar credenciales.
- **Criterio negativo:** un span exportado nunca prueba autorización ni sustituye al recibo durable; la exportación externa es opt-in y separada del core.
- **Independencia:** puede existir sin exportador y sin X-DIF-01; extiende las vistas de Studio ya existentes sin duplicar consola ni almacenar secretos en el reporte.
- **No objetivos:** construir nuevo dashboard de métricas o un juez de respuestas; registrar texto completo en trazas por conveniencia.

### X-DIF-03 — Publicación y retirada de derivados por revisión de ingesta

- **Problema:** una política sustituida no debe reaparecer porque un worker antiguo terminó después de una actualización o un borrado: `put_page` persiste sin `run_id` y la comprobación terminal de `WikiStore` no es fencing de cada write ni transacción de todas las páginas/derivados.
- **Contrato hipotético (tipos propuestos):** `ProjectionReceipt { source_id, source_revision, attempt_id, config_revision, state, derived_refs, cleanup_outcome }`; `publish(expected_revision, receipt) -> Published|Superseded|RetryableFailure`. `state` separa staging, published y withdrawal-pending. Scope proviene del caller autorizado, nunca del texto ingerido. El vínculo `run_id`→revisión de cada fuente en `ProjectionReceipt` es propuesto: el `run_id` existente identifica intento, no por sí solo revisión de fuente.
- **Fuente de evidencia:** `api/services/knowledge/indexing/execution.py:77-178` (N3, `DocumentIndexingService.run`/`_run`: fases extract/splitting/transform/load, cleanup best-effort sin atomicidad SQL↔índice demostrada), `api/tasks/document_indexing_update_task.py:36-110` (N6; `:36-93` marca parsing, confirma y limpia nodos previos con `IndexProcessor.clean`; rollback de sesión y continuar — no reemplazo indivisible acreditado), `api/tasks/clean_document_task.py:20-150` (N8; obtiene IDs antes de limpiar índices y separa limpieza vectorial, SQL, attachments y blobs; registra fallos de storage).
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs` (V5: `begin:70-90` obtiene `(namespace,slug,run_id)`, `ingest_body:167-309` publica páginas vía callback `WikiStore::put_page`, `execute:95-154` llama `complete`/`fail`), `src/wiki/store.rs` (V7 `begin_processing/complete/fail:162-217` valida run_id al terminal; V8 `put_page:287-320` persiste `(namespace,slug,path)` sin `run_id`), `vanta-memory/src/ingest/merge.rs` `CommitReport` (V6 `:34-49`, distingue `written`/`merge_errors`). La ingesta documental directa (`src/wiki/ingestors.rs::Ingestor::ingest`, V9) es otra ruta: produce `MemoryInput` `{file}#{chunk}` y escribe vía SDK, no por el worker wiki.
- **Acciones:**
  1. Extender `IngestReport`/`CommitReport` del worker y el callback hacia `WikiStore::put_page`, conservando run_id/versión del wiki.
  2. Identificar revisión/intento antes de escribir derivados.
  3. Persistir progreso con los checkpoints existentes.
  4. Comprobar que la revisión sigue vigente antes de publicar; un intento con revisión obsoleta se rechaza.
  5. Retirar derivados antiguos con recibo por recurso.
  6. Reconciliar intentos interrumpidos sin revivir tombstones. Rollback: desactivar publicación nueva y conservar la última revisión válida; reindexado solo cuando cambian texto/modelo/config relevante.
- **Dependencias:** provenance/supersession y checkpoints existentes (V3, V4); garantía de commit real disponible — no suponer CAS ni atomicidad multialmacén sin especificarlas y acreditarlas. Consume la identidad de cómputo de X-DIF-05 y el schema de X-DIF-04 solo cuando aplican. No mantener locks del motor durante embeddings/red; lectores fijan una revisión visible.
- **FMEA:** retry tardío → publicación obsoleta → contexto contradictorio → detectar conflicto de revisión y rechazar; cleanup parcial → datos huérfanos → detectar recibos pendientes y reconciliar; borrar fila de fuente antes de inventariar derivados → pérdida de ownership → conservar referencias mínimas.
- **Criterios de aceptación propuestos (manuales):** actualizar durante ingesta; borrar durante load; reentregar intento; fallar un derivado después del primario; reiniciar entre fases. Verificar identidad, revisión visible, error parcial y ausencia de resurrección, sin registrar contenido.
- **Criterio negativo:** `ExecutionConflict`/`NotFound`/`Invalid` y errores de storage nunca se convierten en «sin derivados»; `put_batch`, ACK de job o estado visual no se presentan como commit de todos los índices y almacenes.
- **Independencia:** el primario y el recibo durable son dueños de identidad; el vector no lo es. Alternativa mínima: ampliar el reporte del ingestor sin nueva persistencia si ya existe un recibo equivalente; rechazar doble scheduler/outbox paralelo.
- **No objetivos:** exactly-once de herramientas externas, transacción global, copiar KnowledgeFS ni sustituir el lifecycle de memoria factual de VantaDB.

### X-DIF-04 — Metadata validada por schema antes de persistir o compilar filtros

- **Problema:** una sola fecha o número con representación incompatible puede invalidar la consulta de toda una base (issues #45638 y #41597, open al 2026-11-07); `validate_metadata` actual rechaza claves reservadas/NUL pero no valida schema de campo ni unidad temporal.
- **Contrato hipotético (tipos propuestos):** `ValidatedMetadataPatch { source_revision, schema_revision, typed_values, update_mode }`; `validate_patch(schema, raw) -> Result<ValidatedMetadataPatch, MetadataError>`. Errores distinguibles: campo desconocido, tipo incompatible, fecha ambigua, revisión obsoleta. `null`, ausencia y eliminación son operaciones distintas. Identidad de destino: `(namespace,key)` del `MemoryInput`, no nombre de campo ni posición del batch. El modelo puede sugerir predicados; no autoriza ni decide qué datos borrar.
- **Fuente de evidencia:** `api/repositories/knowledge/metadata_repository.py:155-198` (N7, `update_documents`: resuelve IDs y guarda valores en una transacción, bindings y modo parcial/completo), `api/core/rag/retrieval/dataset_retrieval.py:1631-1760` (N9; `:1694-1760` convierte number/time mediante `as_float`, `:1643-1655` pasa nombres sin tipos a salida de LLM). Issues públicas [#45638](https://github.com/langgenius/dify/issues/45638) y [#41597](https://github.com/langgenius/dify/issues/41597).
- **Seam VantaDB:** `src/wiki/ingestors.rs` (`JsonIngestor::ingest`/`CsvIngestor::ingest` V9 `:45-89`; `json_to_sdk_value:309-351` y `parse_csv_value:355-369` V12 — ya producen metadata tipada; JSON anidado se omite, listas mixtas → strings, fechas textuales no se convierten a `DateTime`), `src/sdk/serialization/mod.rs::validate_metadata:252-266` (V13), `src/sdk/types.rs::Value` (V11) y `MemoryInput.metadata` (`src/sdk/types/record.rs:182-194`), `src/sdk/api/memory.rs::put_batch:1280-1350` (V10: valida shape/dimensión/cuotas antes del primer commit; después confirma **por chunk**, no toda la importación). `RecallConfig`/`InjectionPolicy` (V2) quedan como gobernanza, no validador de schema.
- **Acciones:**
  1. Inventariar tipos y operadores existentes (`Value::{Int,Float,Bool,DateTime,List*,Null}`).
  2. Acordar unidad/zona horaria en el adaptador.
  3. Ampliar la validación del adapter antes de asignar `MemoryInput.metadata`, reutilizando `Value`.
  4. Validar todo el batch antes del primer efecto; persistir según la atomicidad documentada.
  5. Compilar filtros solo con valores validados.
  6. Inspeccionar datos heredados sin corregirlos silenciosamente (inventario/quarantine de valores inválidos); desactivación devuelve error explícito, nunca elimina el filtro.
- **Dependencias:** schema perteneciente al dueño autorizado de la fuente/colección, no al LLM; capturar revisión de schema y rechazar patch que compite con rename/cambio de tipo. No bloquea X-DIF-02; X-DIF-03 solo depende de esta candidata si ingiere metadata tipada. `put_batch` confirma por chunk: la validación de schema del conjunto se hace antes de delegar, pero no promete revertir chunks ya aplicados por fallo posterior — conserva `Error::Validation` y parcialidad.
- **FMEA:** validar solo nuevas escrituras → legado sigue rompiendo lectura → detectar con inventario; normalización implícita → fecha/valor alterados → exigir formato declarado; filtro inválido omitido → ampliación de resultados → fallar cerrado en el borde, manteniendo ACL separada; batch mixto → parcialidad no declarada → validar antes o devolver recibo parcial explícito.
- **Criterios de aceptación propuestos (manuales):** número, timestamp, null y campo ausente; fecha textual ambigua; batch mixto; schema cambiado concurrentemente; legado mal tipado. Comprobar error accionable sin SQL/secretos y sin ampliación de scope.
- **Criterio negativo:** un filtro con valor no validado nunca se compila ni se omite silenciosamente; una fecha textual no se convierte a `DateTime` sin formato declarado.
- **Independencia:** preferir extender validadores existentes; alternativa adapter-only si el core admite metadata libre. No imponer schema global a todo registro.
- **No objetivos:** generar nuevos prompts, copiar el corrector del issue, inferir equivalencia de filtros entre todos los backends ni nuevos thresholds; no añadir tipos al SDK que ya los tiene ni atribuir el cast PostgreSQL a VantaDB.

### X-DIF-05 — Caché de embedding con identidad comprobable y correspondencia posicional

- **Problema:** un vector inválido no debe desplazar la asociación de los demás textos; la reutilización de cómputo no acredita vigencia ni autorización.
- **Contrato hipotético (tipos propuestos):** `EmbeddingBatchResult { model_revision, input_kind, entries: Vec<EmbeddingEntry> }`, cada entry con `input_id`, `content_digest` y `Ready(vector)|Invalid(reason)|ProviderError(code)`. Clave de caché con fingerprint efectivo de modelo, dimensiones, preprocesado y tipo de input cuando afectan al vector; namespace de caché según política de aislamiento. Dimensión/finiteza se validan antes del índice.
- **Fuente de evidencia:** `api/core/rag/embedding/cached_embedding.py:24-133` (N5, `CacheEmbedding`: consulta por provider/model/hash, conserva slots por índice de entrada, normaliza cada vector y deja `None` al descartar NaN; captura `IntegrityError` al persistir y retorna la lista de slots; query cache Redis con TTL deslizante de 600 s — comportamiento upstream, no default VantaDB; el tamaño de lote sale de `MAX_CHUNKS` del schema del modelo, con fallback 1, no «batches of 10» del comentario).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` — `core_embedding_hook`/`local_embedding_hook` (V3, `:131-176`) ya proporcionan extensión de embeddings; el delta es hacer comprobable la identidad del cálculo y el outcome por entrada en el adapter de modelo, no introducir Redis/SQL/pickle ni una segunda caché global dentro del core.
- **Acciones:**
  1. Cotejar cachés existentes en los hooks de embedding de L1.
  2. Resolver la identidad (modelo/dimensiones/preprocesado/input_kind) antes de calcular.
  3. Devolver un outcome por input sin compactar slots.
  4. Publicar cache entries válidas después de validación; asociarlas al recibo de X-DIF-03 si aplica.
  5. Coalescing de miss concurrente, si se añade, en el adapter — sin retener locks del motor durante I/O. Cambio de modelo invalida namespace de cómputo; rollback ignora la caché nueva sin reescribir memorias válidas.
- **Dependencias:** ninguna exigencia de proveedor externo para motor local; caché como derivado descartable, nunca memoria canónica. No deducir que provider/model string identifica pesos inmutables.
- **FMEA:** omitir vector intermedio y zip compacto → fuente equivocada → detectar IDs/longitud; modelo homónimo cambiado → vector incompatible → fingerprint/dimensión; caché compartida → canal de información/acceso impropio → política de aislamiento y autorización fuera de la caché.
- **Criterios de aceptación propuestos (manuales):** batch válido, NaN central, respuesta corta del proveedor, modelo cambiado, miss simultáneo, cancelación y fallo de persistencia de caché. Los inputs posteriores mantienen identidad; el fallo de caché no finge commit de ingesta. Diagnóstico registra hit/miss/causa sin texto ni vector.
- **Criterio negativo:** un outcome `Invalid`/`ProviderError` nunca se compacta ni desplaza la correspondencia posicional de los demás inputs; la caché nunca es memoria canónica ni fuente de autorización.
- **Independencia:** alternativa segura documentada: funcionar sin caché; alternativa preferida: ampliar la existente. No requiere proveedor/LLM.
- **No objetivos:** copiar serialización pickle, dedup semántico, eviction arbitraria ni promesa de aceleración sin medición.
