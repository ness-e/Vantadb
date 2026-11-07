---
title: "Dify: configuración efectiva y trazabilidad del retrieval"
kind: research
status: proposed
description: "Estudio de arquitectura RAG, persistencia, filtros, cachés, diagnóstico web y adaptación selectiva a VantaDB con evidencia congelada"
tags: [research, memory, engineering]
---

# Dify: configuración efectiva y trazabilidad del retrieval

## Índice

- [0. Identidad, arquitectura y flujos](#0-identidad-arquitectura-y-flujos)
- [1. Recomendaciones](#1-arquitectura-stack-flujo-y-recomendación)
- [2. Fichas Rust propuestas](#2-mecanismos-mínimos-extraíbles-y-seams-vanta)
- [3. Incidencias y seguridad](#3-riesgos-estáticos-y-separación-de-hipótesis)
- [4. Prerrequisitos](#4-prerrequisitos-vanta)
- [5. Rechazos](#5-no-trasladar)
- [6. Método y fuentes](#6-alcance-licencia-y-ledger)
- [7. Benchmarks](#7-benchmarks-y-runners-realmente-leídos)
- [8. Matriz](#8-matriz-capacidades-vanta)
- [9. Negativos sustentados](#9-deuda-sugerida-sin-falsos-bugs)
- [10. UI y Studio](#10-interfazescritorio-e-ideas-studio)
- [11. Cobertura y continuidad](#11-cobertura-dependencias-y-validación-pendiente)

## 0. Identidad, arquitectura y flujos

**Corte de lectura: 2026-11-07.** Código primario en
`C:\Users\Eros\dify`, HEAD comprobado sin actualizar:
`65b4e17d6ebeff39c7e66c832649afff01666adf`. Clon shallow; no extrapolar
historia ni todas las ramas. Metadatos congelados en
[clone-manifest](clone-manifest.json): 160 588 estrellas, no archivado.
No son métricas de calidad. El alcance es RAG/Knowledge y su consola, no todo
el runtime de agentes, plugins, sandbox o despliegue SaaS.

**Licencia por superficie.** [LICENSE:1-22][S9] no es Apache-2.0 sin más:
añade autorización comercial para operar código Dify en un entorno multitenant
(tenant = workspace), y prohíbe retirar/modificar logo/copyright al usar el
frontend (`web/` o imagen web). Declara protección del diseño interactivo.
Esto no acredita alcance jurídico ni concesión para copiarlo. Backend sin UI
no elimina la condición multitenant; traducción a Rust no elimina derivación.
Recomendaciones de este informe son contratos abstractos, no trasplantes de UI.
SDKs, plugins, modelos y bibliotecas terceras conservan licencias propias.

### 0.1 Mapa de responsabilidades y modelos

| Superficie | Responsabilidad observada | Persistencia/frontera |
|---|---|---|
| `api/controllers/console/datasets/` | Validación HTTP y permisos de consola; hit testing | Sesión del request; no dueño del índice |
| `api/services/knowledge/` | Referencias tenant/dataset/document, puertos de indexación y metadata | Repositorios SQLAlchemy; transacciones por fase |
| `api/tasks/document_indexing*.py` | Admisión, entrega Celery y estado de trabajo | SQL antes de llamadas lentas; cola separada |
| `api/core/rag/datasource/`, `retrieval/` | Elección de ramas, filtros, dedup y postproceso | Índices/proveedores; no transacción distribuida acreditada |
| `api/core/rag/embedding/` | Invocación y cachés de vectores | Redis query cache; tabla SQL `Embedding` |
| `api/models/dataset.py` | Dataset, documento, segmento, consulta y binding | Fuente SQL de identidad/config/estado |
| `web/app/components/datasets/` | Configuración, hit testing, documentos y resultados | Estado React y query cache; no autoridad de permisos |
| `docker/`, `dify-agent/`, `cli/`, `e2e/` | Despliegue y otros productos/herramientas presentes | Inventario; no auditados funcionalmente aquí |

Stack leído en S9: backend Python 3.13/Flask, Celery, Redis, SQLAlchemy y
PostgreSQL; web TypeScript/React con scripts Next/vinext. No inferir de esos
paquetes qué servicio se habilitó en una instalación concreta. Las búsquedas
vectoriales dependen de adaptadores; el nombre Dify no identifica un algoritmo ANN.

`Dataset` lleva tenant, técnica de indexación y configuración retrieval;
`Document` vincula dataset/origen/regla y separa `indexing_status`, pausa,
habilitación y archivo; `DocumentSegment` identifica fragmentos y nodos de
índice; `DatasetQuery` guarda la consulta de prueba (N1, S7). Un segmento no es
una memoria factual consolidada ni un checkpoint de ejecución. El documento
canónico y su vector derivado tienen ciclos distintos. `completed` tampoco
equivale a habilitado: la proyección de display puede producir deshabilitado o
archivado. Ningún span o fila de historial constituye ground truth de relevancia.

### 0.2 Tres recorridos concretos

**A. Ingestar una política y hacerla consultable.** En N2:51-130 el worker busca
dataset/documentos, aplica admisión cloud cuando corresponde y confirma
`PARSING` en una fase SQL breve. Después construye referencias desde registros
leídos, libera la transacción y llama `DocumentIndexingService.run`.
N3:108-153 resuelve documento, comprueba pausa, extrae, marca splitting,
transforma, admite espacio si se solicitó, cuenta tokens y persiste segmentos.
Solo entonces `load` escribe los índices y `complete_indexing` marca final.
Se puede reanudar desde splitting/indexing por entradas distintas.
**Fallo:** inexistente se omite; pausa se propaga; excepción registra error.
Si borraron el documento durante load se intenta `discard`; ese cleanup es
best-effort y no demuestra atomicidad entre SQL e índice. Éxito requiere
interpretar estado persistido, no solamente ACK Celery. No se ejecutó ingesta.

**B. Probar una consulta restringida por año.** N4 valida login, tenant, permiso
`DATASET_PIPELINE_TEST`, dataset y payload. S2:119-164 resuelve override →
config dataset → default; economy fuerza keyword. Resuelve filtro metadata a
IDs documentales; condición válida sin IDs devuelve vacío sin ampliar búsqueda.
S1:815-924 ejecuta ramas elegibles en threads con contexto propagado, reúne
errores, deduplica híbrido y pasa postproceso opcional. S2 guarda `DatasetQuery`
con `flush` en ruta interna y formatea segmentos/documentos.
**Fallo:** filtro numérico con dato mal tipado puede fallar en SQL (issue §3),
modelo no disponible puede omitir rerank según S3; no interpretar ninguno como
«no hay evidencia». La consulta de prueba puede costar embedding/rerank y
escribir historial; no es una lectura sin efectos. El query cache N5 almacena
vectores, no autorizaciones ni respuestas.

**C. Actualizar política mientras llegan consultas.** N6:36-93 marca parsing,
confirma y limpia nodos previos mediante `IndexProcessor.clean`; la limpieza
incluye keywords/hijos. Si falla, hace rollback de esa sesión, recarga documento
y persiste estado para continuar su tratamiento. El resto de esa tarea no
se ha auditado completo: no se promete reemplazo indivisible. Una consulta
concurrente puede observar transición de estado/datos derivados; hace falta
reproducción para determinar visibilidad en cada backend. Metadata se actualiza
en transacción N7:155-198, incluyendo bindings y modo parcial/completo; eso es
otra operación, no reembedding automático acreditado. El cache por texto/modelo
puede reutilizar un vector si el texto no cambió, sin validar vigencia de fuente.
UI debe mostrar revisión/estado y no vender «guardar» como «índice actualizado».

### 0.3 Base local revalidada

La referencia Vanta de este análisis es **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**,
no `c561158` dirty (historia). Se leyeron V1–V4: híbrido lexical/dense/sparse
con RRF, recall gobernado, assembler y progreso ya existen. `RecallConfig`
mantiene `core_search=false` por defecto: modo `Hybrid` no demuestra uso del
core híbrido por todas las interfaces. No proponer otro buscador o pipeline.
Studio se leyó en worktree separado `develop-desktop` @
`0ad38c542` (SHA abreviado observado; enlaces por revisión en §10), sin editarlo.

## 1. Arquitectura, stack, flujo y recomendación

**Dictamen:** extraer contratos de configuración efectiva y diagnóstico
redactado, no una plataforma de workflows. La composición de búsqueda/rerank
enseña fronteras útiles, pero no prueba superioridad sobre el híbrido de Vanta.
La licencia impide tratar el repositorio como Apache-2.0 sin condiciones.

- Repositorio: `langgenius/dify`; clon `C:\Users\Eros\dify`.
- SHA congelado: `65b4e17d6ebeff39c7e66c832649afff01666adf`.
- Snapshot: **160 588 estrellas**, `archived=false`, licencia API `NOASSERTION`.
- Fuente: [github-candidates.json](github-candidates.json) y [clone-manifest.json](clone-manifest.json).
- `LICENSE:1-22` declara Apache 2.0 modificado con condiciones adicionales.
- No se actualizaron metadatos, stars ni el snapshot de código.

El manifiesto API leído declara Python 3.13, Flask, SQLAlchemy a través del
código, Celery, Redis, PostgreSQL y dependencias de tracing/modelos.
La web usa React/TypeScript y tiene scripts Next/vinext; no se ejecutaron.
El inventario también contiene `dify-agent` y runtime separado: esta ficha sigue
el retrieval RAG de `api/`, no certifica toda la arquitectura de agentes.

Flujo leído: selección de datasets disponibles por tenant → filtros metadata →
estrategia single/multiple → retrieval por texto o attachment → búsqueda
keyword/semántica/fulltext según método → dedup híbrido → postproceso
ponderado o modelo reranker → documentos/segmentos para contexto y referencias.
`RetrievalService` propaga contexto de tracing a sus pools de threads.
Hit testing usa ese servicio, permite configuración de prueba y registra consultas.
Estado documental, configuración retrieval y spans son superficies separadas.

Vanta ya tiene BM25/HNSW/RRF/sparse, provenance, sesiones, supersession, dedup,
assembler/compaction, scheduler/checkpoints e ingestors/auditoría.
Studio Tauri/React existe en `develop-desktop`, incluido diagnóstico retrieval.
La base histórica `c561158` no acredita defectos actuales; §0.3 y §10
registran la relectura acotada de seams en la nueva base, no una auditoría total.

**Orden recomendado, no backlog aprobado:** (1) ampliar explain efectivo
sobre reportes existentes (A); (2) separar inspección de contenido y exportación
(B); (3) estados por revisión de ingesta/reindexado (C); (4) tipado de metadata
en la frontera de adaptación (D); (5) caché de embedding identificable (E).
C/D/E no son nuevos nombres del mismo DTO: gobiernan, respectivamente,
publicación de derivados, validez del predicado y reutilización de cómputo.
No depende toda la lista de resolver una hipótesis histórica de checkpoint.

## 2. Mecanismos mínimos extraíbles y seams Vanta

### A. Plan efectivo de retrieval y resultado de cada etapa

**Problema:** configuración solicitada, resuelta y aplicada no son equivalentes.
Unidad mínima: resolución/explicación del plan, no pools Flask ni proveedores.
S2:119-164 elige override, configuración del dataset o default y aplica
excepción economy → keyword; el método mostrado en una pantalla no basta.
S1:815-877 enruta keyword, semántica y fulltext según método.
S1:891-924 deduplica híbrido y aplica postproceso o threshold alternativo.
S3 permite ausencia de runner cuando no hay modelo válido o autorización.

**Seam Vanta:** configuración de retrieval híbrido existente → reporte de
búsqueda y assembler; SDK/MCP/servidor como frontera de validación, no core cloud.
**DTO hipotético, no API actual:** `EffectiveRetrievalPlan { scope,
config_revision, requested_mode, applied_stages, score_domains, exclusions }`;
`StageOutcome { stage, status: Skipped|Applied|Failed, reason, candidate_count }`.
Operaciones propuestas: `resolve_plan`, `validate_capabilities`, `execute_stage`,
`record_outcome`. Los scores conservan dominio, no se fusionan por nombre.

**Acciones:** cotejar planes/reportes actuales; preservar configuración efectiva
con referencia a revisión; distinguir deshabilitado de no disponible; mostrar
fallback sin prometer que mantenga orden o calidad equivalentes.
No crear valores de evaluación ni copiar los defaults top-k del producto.
**Dependencias:** metadata/config versionada, límites de trabajo y cancelación
existentes; proveedores LLM/rerank solo adaptadores opcionales.
**FMEA:** configuración inválida → etapa omitida silenciosamente; score de
distancia usado como similitud → filtrado invertido; error parcial → falso vacío;
override mutante → diagnóstico irreproducible. Mitigar con enums/dominios,
validación previa y revisión inmutable, no con diccionarios sin procedencia.
**Aceptación pendiente manual:** mismo plan explica qué se ejecutó; deshabilitar
rerank no requiere proveedor; fallo de etapa es distinguible de cero hits;
consulta de diagnóstico no modifica por accidente configuración persistente.
**No objetivo:** reemplazar RRF por suma ponderada ni integrar la plataforma.

### B. Diagnóstico de recuperación con privacidad explícita

**Problema:** explicar retrieval sin convertir observabilidad en copia secundaria
de todos los documentos. Unidad: evento de recuperación y política de exportación.
S4:20-65 serializa contenido, metadata, score e ID; S4:86-105 coloca consulta y
documentos en atributos de span. Eso acredita payload, no destino efectivo.
S2 registra consultas de hit testing mediante `DatasetQuery` y `session.flush`.
S5 expone historial de consultas y resultados, diferenciando fuente externa.

**Seam Vanta:** auditoría/diagnóstico retrieval y actividad existentes →
RetrievalLens/MemoryLens de Studio; exportador externo opt-in fuera del core.
**DTO hipotético:** `RetrievalDiagnostic { trace_id, scope, config_revision,
source_refs, stage_outcomes, visibility: MetadataOnly|PreviewAuthorized,
redaction_revision }`. El DTO contiene referencias; previews se resuelven aparte.
Operaciones: `capture_metadata`, `authorize_preview`, `redact`, `export_opt_in`.
Retención de auditoría y trazas no debe heredar la del historial conversacional.

**Acciones:** comprobar si la auditoría actual ya cubre el evento; clasificar
metadata sensible; mantener identidad estable sin exportar texto por defecto;
explicar qué consultas se guardan y quién puede inspeccionarlas.
**Dependencias:** scope, autenticación/autorización, provenance, política de
retención y canal de actividad existentes; sin requisito de OpenTelemetry remoto.
**FMEA:** traza cruza tenant → fuga; referencia firmada expira → vista rota;
documento retirado → preview histórico no autorizado; exportador falla →
consulta bloqueada. Mitigar con resolución autorizada actual, redacción previa
y separación entre éxito del retrieval y fallo de observabilidad.
**Aceptación pendiente:** metadata-only no contiene texto/secretos; retirar
acceso impide preview; error de exportación es visible sin perder resultados;
se identifica revisión de configuración sin guardar credenciales.
**No objetivo:** construir nuevo dashboard de métricas o un juez de respuestas.

### C. Publicación y retirada de derivados por revisión de ingesta

**Problema de agente:** una política sustituida no debe reaparecer porque un
worker antiguo terminó después de una actualización o un borrado. Fuente:
`DocumentIndexingService.run`/`_run` [N3], worker de actualización
[N6] y `clean_document_task` [N8]. Son fases y compensaciones, no una
transacción distribuida. N8 obtiene IDs antes de limpiar índices y separa
limpieza vectorial, SQL, attachments y blobs; registra fallos de storage.
Unidad extraíble: recibo de materialización/retirada por revisión, no Celery.

**Delta y seam:** Vanta ya planea/persiste L1 y aplica dedup/supersession
([V3]); ya tiene `TaskCheckpointManager` y scheduler [V4]. El dueño real de
ingesta wiki es `vanta-memory/src/ingest/worker.rs::{begin,execute,ingest_body}`
[V5], no el escritor L1: `begin:70-90` obtiene `(namespace,slug,run_id)`;
`ingest_body:167-309` escanea, extrae fuera de escritura y publica páginas
mediante callback `WikiStore::put_page`. `execute:95-154` llama después a
`WikiStore::complete` o marca `fail` best-effort. `CommitReport` distingue
`written/merge_errors` [V6]; terminar el run no acredita que cada página entró.
`WikiStore::{begin_processing,complete,fail}:162-217` [V7] incrementa versión y
valida run_id al terminal; `put_page:287-320` [V8] persiste una página canónica
`(namespace,slug,path)` en InternalMetadata **sin argumento run_id**. La
comprobación terminal no es fencing de cada write ni transacción de todas las
páginas/derivados. C ampliaría **ese callback/publicador y sus recibos** con
revisión esperada, no trasladaría publicación a L1 o a otro scheduler.
La ingesta documental directa es otra ruta: `Ingestor::ingest` [V9] produce
`MemoryInput` con clave `{file}#{chunk}` y provenance; su consumidor escribe
mediante SDK [V10], no automáticamente mediante el worker wiki.
No presentar `put_batch`, ACK de job o estado visual como commit
de todos los índices y almacenes externos.
**Contrato Rust propuesto:** `ProjectionReceipt { source_id, source_revision,
attempt_id, config_revision, state, derived_refs, cleanup_outcome }`;
`publish(expected_revision, receipt) -> Published|Superseded|RetryableFailure`.
`state` separaría staging, published y withdrawal-pending. Scope proviene del
caller autorizado, nunca del texto ingerido. No son tipos existentes.

**Pasos/ownership/commit:** (1) extender `IngestReport/CommitReport` del worker
y callback hacia `WikiStore::put_page`, conservando run_id/versión del wiki;
(2) identificar revisión/intent antes de escribir derivados; (3) persistir
progreso con checkpoints existentes; (4) comprobar que la revisión sigue
vigente antes de publicar; (5) retirar derivados antiguos con recibo de cada
recurso; (6) reconciliar intentos interrumpidos sin revivir tombstones.
El primario y el recibo durable son dueños de identidad; el vector no lo es.
El punto de publicación deberá usar la garantía real disponible en Vanta:
no suponer CAS ni atomicidad multialmacén sin especificarlas y acreditarlas.
`ExecutionConflict/NotFound/Invalid` y errores de storage no se convierten en
«sin derivados». El run_id ya existente identifica intento, no por sí solo
revisión de cada fuente; ese vínculo en ProjectionReceipt es **propuesto**.
No mantener locks del motor durante embeddings/red; lectores fijan una
revisión visible. Rollback: desactivar publicación nueva y conservar la última
revisión válida; reindexado solo cuando cambian texto/modelo/config relevante.

**Dependencias/alternativas:** provenance y supersession existentes; C consume
la identidad de cómputo de E y el schema de D solo cuando aplican.
Alternativa mínima: ampliar reporte del ingestor sin nueva persistencia si
ya existe un recibo equivalente. Rechazar doble scheduler/outbox paralelo.
**FMEA:** retry tardío → publicación obsoleta → contexto contradictorio;
detectar conflicto de revisión y rechazar. Cleanup parcial → datos huérfanos;
detectar recibos pendientes y reconciliar. Borrar fila de fuente antes de
inventariar derivados → pérdida de ownership; conservar referencias mínimas.
**Aceptación manual futura:** actualizar durante ingesta; borrar durante load;
reentregar intento; fallar un derivado después del primario; reiniciar entre
fases. Verificar identidad, revisión visible, error parcial y ausencia de
resurrección, sin registrar contenido. No se ejecutaron estos casos.
**No objetivos:** exactly-once de herramientas externas, transacción global,
copiar KnowledgeFS, ni sustituir el lifecycle de memoria factual de Vanta.

### D. Metadata tipada antes de persistir o compilar filtros

**Problema:** una sola fecha con representación incompatible puede invalidar
la consulta de toda una base. [N7:155-198] resuelve IDs y guarda valores en una
transacción; [N9:1694-1760] convierte campos number/time mediante `as_float`.
El issue 45638 conecta esas dos superficies; 41597 añade la salida de LLM.
Unidad mínima: validación de schema/valor y representación canónica, no el
prompt que adivina condiciones ni un parser SQL copiado.

**Delta/seam concreto:** entrada existente
[`MemoryInput.metadata:182-194`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types/record.rs#L182-L194),
[`MemoryMetadata/Fields:500-504`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types.rs#L500-L504)
y `Value::{Int,Float,Bool,DateTime,List*,Null}` [V11].
`src/wiki/ingestors.rs::JsonIngestor::ingest`, `CsvIngestor::ingest`,
`json_to_sdk_value:309-351` y `parse_csv_value:355-369` [V12] ya producen
metadata tipada; JSON anidado se omite, listas mixtas se vuelven strings en
ese helper y fechas textuales no se convierten automáticamente a DateTime.
El SDK diferencia DateTime de String [V11]; no inventar un schema SQL global.
La entrada de escritura es `Embedded::{put,put_batch}` [V10];
`validate_metadata:252-266` [V13] rechaza claves reservadas/NUL, **no valida
schema documental por campo ni unidad temporal**. D ampliaría validación del
adapter **antes** de convertir/asignar `MemoryInput.metadata`, reutilizando
Value; compilaría filtros con esos mismos valores en [V1], conservando
`RecallConfig/InjectionPolicy` [V2] como gobernanza, no validador de schema.
El diferencial es schema/revisión/semántica por fuente, no “añadir tipos” al
SDK que ya los tiene ni atribuir el cast PostgreSQL a Vanta.
**Contrato propuesto:** `ValidatedMetadataPatch { source_revision,
schema_revision, typed_values, update_mode }`;
`validate_patch(schema, raw) -> Result<ValidatedMetadataPatch, MetadataError>`.
Errores distinguibles: campo desconocido, tipo incompatible, fecha ambigua,
revisión obsoleta. `null`, ausencia y eliminación son operaciones distintas.
El modelo puede sugerir predicados; no autoriza ni decide qué datos borrar.

**Implementación futura:** inventariar tipos y operadores existentes; acordar
unidad/zona horaria en el adaptador; validar todo el batch antes del primer
efecto; persistir según la atomicidad documentada; compilar únicamente valores
validados; inspeccionar datos heredados sin corregirlos silenciosamente.
Ownership: schema pertenece al dueño autorizado de la colección/fuente, no al
LLM. Concurrencia: capturar revisión de schema y rechazar patch que compite con
rename/cambio de tipo. Migración requiere inventario/quarantine de valores
inválidos; desactivación devuelve error explícito, nunca elimina el filtro.
Identidad de destino: `(namespace,key)` del MemoryInput, no nombre de campo ni
posición del batch; source_revision/schema_revision serían aditivos propuestos.
`put_batch:1280-1350` [V10] valida shape/dimensión/cuotas antes del primer
commit; después confirma **por chunk**, no toda la importación. D valida
schema del conjunto antes de delegar, pero no promete revertir chunks ya
aplicados por fallo posterior; conserva Error::Validation y parcialidad.

**Deps/alternativas:** no bloquea B; C solo depende de D si ingiere metadata
tipada. Preferir extender validadores existentes; alternativa adapter-only si
el core admite metadata libre. No imponer schema global a todo registro.
**FMEA:** validar solo nuevas escrituras → legado sigue rompiendo lectura;
detectar con inventario. Normalización implícita → fecha/valor alterados;
exigir formato declarado. Filtro inválido omitido → ampliación de resultados;
fallar cerrado en el borde, mantener ACL separada. Batch mixto → parcialidad
no declarada; validar antes o devolver recibo parcial explícito.
**Aceptación manual:** número, timestamp, null y campo ausente; fecha textual
ambigua; batch mixto; schema cambiado concurrentemente; legado mal tipado.
Comprobar error accionable sin SQL/secretos y sin ampliación de scope.
**No objetivos:** generar nuevos prompts, copiar el corrector del issue,
inferir equivalencia de filtros entre todos los backends ni nuevos thresholds.

### E. Caché de embedding con identidad y correspondencia posicional

**Problema:** un vector inválido no debe desplazar la asociación de los demás
textos; reutilización de cómputo tampoco acredita vigencia/autorización.
[N5:24-133] consulta por provider/model/hash, conserva slots por índice de
entrada, normaliza cada vector y deja `None` al descartar NaN. Captura
`IntegrityError` al persistir y retorna la lista de slots. Query cache usa
Redis y TTL deslizante de 600 s; es comportamiento upstream, no default Vanta.
El comentario «batches of 10» no describe el loop efectivo: el tamaño sale
de `MAX_CHUNKS` del schema del modelo, con fallback 1.

**Delta/seam:** `core_embedding_hook` y `local_embedding_hook` en L1 [V3]
ya proporcionan extensión de embeddings. El delta sería hacer comprobable
la identidad del cálculo y resultado por entrada en el adapter de modelo;
no introducir Redis/SQL/pickle ni una segunda caché global dentro del core.
**Contrato Rust propuesto:** `EmbeddingBatchResult { model_revision,
input_kind, entries: Vec<EmbeddingEntry> }`, cada entry lleva `input_id`,
`content_digest` y `Ready(vector)|Invalid(reason)|ProviderError(code)`.
Clave de caché incluiría fingerprint efectivo de modelo, dimensiones,
preprocesado y tipo de input cuando afectan al vector; namespace de caché
según política de aislamiento. Dimensión/finiteza se validan antes del índice.

**Pasos/ownership/commit:** cotejar cachés existentes; resolver identidad
antes de calcular; devolver un outcome por input sin compactar slots; publicar
cache entries válidas después de validación; asociarlas al recibo C si aplica.
La caché es derivado descartable, no memoria canónica. Miss concurrente puede
repetir cómputo: coalescing, si se añade, vive en el adapter y no retiene locks
del motor durante I/O. Cambio de modelo invalida namespace de cómputo;
rollback ignora la caché nueva sin reescribir memorias válidas.
**Dependencias/alternativas:** ninguna exigencia de proveedor/LLM para motor
local. Alternativa segura: sin caché; alternativa preferida: ampliar la ya
existente. No deducir que provider/model string identifica pesos inmutables.
**FMEA:** omitir vector intermedio y zip compacto → fuente equivocada;
detectar IDs/longitud. Modelo homónimo cambiado → vector incompatible;
fingerprint/dimensión. Cache compartida → canal de información/acceso
impropio; política de aislamiento y autorización fuera de la caché.
**Aceptación manual:** batch válido, NaN central, respuesta corta del proveedor,
modelo cambiado, miss simultáneo, cancelación y fallo de persistencia de caché.
Los inputs posteriores mantienen identidad; fallo de caché no finge commit
de ingesta. Diagnóstico registra hit/miss/causa sin texto ni vector.
**No objetivos:** copiar serialización pickle, dedup semántico, eviction
arbitraria ni promesa de aceleración sin medición.

## 3. Riesgos estáticos y separación de hipótesis

- **Código:** S4 puede poner consulta/contenido/metadata en spans. El destino,
  redacción global y sampling efectivos no se auditaron; no se afirma fuga.
- **Hipótesis de privacidad:** exportar esos spans a tercero ampliaría frontera
  de confianza documento → proveedor de observabilidad; requiere configuración.
- **Código:** S3 devuelve `None` en ausencia de modelo o InvokeAuthorizationError.
  S1 contiene filtro alternativo de score si el híbrido carece de rerank runner.
  No es prueba de error actual ni de equivalencia semántica del fallback.
- **Código:** S1 usa pools anidados y `cancel()` de futures. No se acredita que
  una llamada de SDK ya iniciada se interrumpa; el coste residual es hipótesis.
- **Código:** hit testing persiste consultas; no es una operación puramente
  de lectura aunque la interfaz la presente como prueba de recuperación.
- **Frontera de autorización:** helper de bajo nivel resuelve por dataset ID;
  selección de datasets del caller recibe tenant. No se infiere bypass de
  permisos sin recorrer controladores, repositorios y verificación completa.
- **Hipótesis:** attachment/documento debe conservar ownership al expandir
  referencias; S1:928-956 verifica tenant y dataset en la ruta leída.

No CVEs atribuidas. La ampliación sí consultó incidencias públicas; la lectura
previa sin red no bastaba para este eje. No se auditó seguridad global/plugins.

### Incidencias públicas efectivamente leídas

Consulta **2026-11-07 UTC**: búsqueda REST GitHub y lectura HTML mediante
`websearch.fetchWebContent`. Son reportes, no reproducciones propias.

| Fuente y estado observado | Síntoma/versión reportada | Mecanismo y discriminación |
|---|---|---|
| [#45638: non-numeric metadata](https://github.com/langgenius/dify/issues/45638), open; creada 2026-11-05 | Dify 1.18.0 Docker: update acepta `abc` como number; retrieval retorna 500 | N7/N9 corroboran escritura no tipada + cast. Distinguir patch inválido de dato legado: validar escritura no repara legado |
| [#41597: automatic time filter](https://github.com/langgenius/dify/issues/41597), open en búsqueda REST; creada 2026-09-01 | 1.16.1; autor cita `6bc7f01261`: LLM devuelve fecha textual | N9:1643-1655 pasa nombres sin tipos; cast posterior. Prompt propuesto por autor no prueba fix ni sustituye validación determinista |

En #45638 la página enlaza **PR #45639** y un comentario de bot propone
regresión de batch/nullable/legado. No se inspeccionó el diff ni su integración:
no afirmar «corregido». #41597 trae un patch local del autor, no evidencia de
merge. Versiones citadas pertenecen al reporte; el SHA analizado es otro.
Aplicabilidad Vanta: **D**, si adaptadores admiten metadata externa; no
atribuir `as_float` ni esquema SQLAlchemy al core Rust. **DIFY-2:** #45638/
#41597 no acreditan E (caché/cardinalidad de embeddings), que conserva su
evidencia independiente `CacheEmbedding` [N5], no esas incidencias.
Reproducción discriminante futura: comparar API manual/automática con los mismos
datos tipados y mal tipados; separar fallo de conversión de fallo de permisos.

**Defecto estático del runner, no issue remoto:** S8/N10:38 y 498-503 consideran
`error` un evento terminal que marca `request_success=True`. Así, éxito Locust
puede significar cierre del protocolo con error de workflow. No invalida por
sí solo tiempos de transporte ni toda cifra publicada; impide llamarlo tasa
de éxito de agente. No se ejecutó ni se corrigió el runner.

## 4. Prerrequisitos Vanta

Revalidar el estado actual antes de crear tareas, no usar ausencia de prueba
como prueba de ausencia. Mantener sparse/BM25/HNSW/RRF existentes.
Precisar qué capa puede introducir overrides y quién fija sus límites.
Conservar distinción entre score de cada etapa y orden final.
Verificar reportes existentes de assembly, auditoría y diagnóstico Studio.
Definir ownership/retención de una consulta guardada y previews revocables.
Revisión legal de licencia/archivo/diseño antes de derivar expresión o UI.
No copiar componentes visuales por asumir que todo está bajo Apache estándar.

## 5. No trasladar

No importar marketplace, plugin daemon, workflows visuales, billing/cloud ni
monolito web/API como dependencia del core Rust embebido.
No convertir Celery/Redis/SQL/pools Flask en nuevos requisitos de scheduler.
No reemplazar RRF por weighting sin evidencia específica del caso de uso.
No registrar texto completo en trazas por conveniencia.
No equiparar índice completo, documento habilitado y fuente autorizada:
el modelo de estado leído los distingue y una UI debe mantener esa separación.
No copiar defaults, límites o prompts de pruebas como contrato de producto Vanta.

## 6. Alcance, licencia y ledger

Lectura estática acotada; **no auditoría completa de Dify** ni análisis de todas
las rutas de autorización, datasets, herramientas, modelos o despliegues.
La primera lectura fue local; esta ampliación incorpora consultas públicas
de §3. Sin ejecución de scripts del proyecto, instalaciones, apps/tests/builds/
benchmarks, servicios, stage, commits o push. No se inició ni detuvo runtime.
Clon superficial sin otras ramas/tags/historia; omisiones LFS según manifiesto.
`NOASSERTION` en GitHub no es ausencia de licencia.
LICENSE exige autorización comercial para uso multi-tenant salvo permiso escrito
y limita retirar/modificar logo/copyright del frontend; contiene además una
declaración sobre protección del diseño interactivo. No es asesoría jurídica.

| ID | Ruta/rango leído con enlace inmutable | Uso |
|---|---|---|
| S1 | [retrieval_service.py:93-173](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/datasource/retrieval_service.py#L93-L173), [203-267](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/datasource/retrieval_service.py#L203-L267), [792-956](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/datasource/retrieval_service.py#L792-L956) | Dispatch, dedup, postproceso, attachments |
| S2 | [hit_testing_service.py:65-192](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/services/hit_testing_service.py#L65-L192) | Config efectiva y log de consulta |
| S3 | [data_post_processor.py:1-134](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/data_post_processor/data_post_processor.py#L1-L134) | Modelo, weighting y reorder opcional |
| S4 | [otel/parser/retrieval.py:1-105](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/extensions/otel/parser/retrieval.py#L1-L105) | Datos serializados de tracing |
| S5 | [hit-testing/index.tsx:1-165](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/app/components/datasets/hit-testing/index.tsx#L1-L165) | Permisos UI, historial, cambios y resultado |
| S6 | [dataset_retrieval.py:82-155](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/retrieval/dataset_retrieval.py#L82-L155) y [470-525](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/retrieval/dataset_retrieval.py#L470-L525) | Caller tenant, estrategia y filtros |
| S7 | [models/dataset.py:100-132](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/models/dataset.py#L100-L132) | Contrato de estado documental |
| S8 | [sse_benchmark.py:1-210](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L1-L210), [364-448](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L364-L448), [718-750](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L718-L750) | Runner SSE y reporte |
| S9 | [LICENSE:1-22](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/LICENSE#L1-L22), [api/pyproject.toml:1-75](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/pyproject.toml#L1-L75), [web/package.json:1-55](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/package.json#L1-L55) | Licencia y stack |

Lectura adicional: `retrieval_service.py:1-92,175-202,269-320`;
inventario de runners/tests/tracing. No se leyeron todos los workers ni el
runtime Go; su presencia no acredita comportamiento que esta ficha no describe.

## 7. Benchmarks y runners realmente leídos

S8 es un **stress runner de transporte SSE/workflow**, basado en Locust.
No mide recall, memoria longitudinal ni fidelidad de citas.
Entradas: API key a través de `ConfigHelper`, ruta workflow, preguntas de fichero
o lista incorporada; POST con `response_mode=streaming`.
Métricas declaradas: conexiones, eventos, tasas, TTFE, duración de stream,
latencias inter-evento, percentiles y categorías de error.
El tracker conserva muestras acotadas de ciertas distribuciones; no representa
necesariamente toda la historia de una ejecución prolongada.
El reporte exporta JSON local y estadísticas agregadas de Locust.
Dependencias: Python, requests/Locust, helper de configuración y Dify operativo,
workflow publicado y sus proveedores/herramientas; no basta disponer del clon.
Efectos: ejecuta workflows reales, puede generar coste LLM, logs, mensajes o
mutaciones de herramientas; escribe `scripts/stress-test/reports/`.
Se amplió lectura de `ConfigHelper` y del tratamiento de terminales [N10];
no se certifica el parser completo ni todo setup/cleanup.
TTFE no equivale a primer token útil; éxito de stream no mide corrección.

S2/S5 ofrecen hit testing interactivo con registro de consulta, no corpus golden
ni benchmark de calidad por sí mismos. Los tests localizados no se ejecutaron.
No se creó harness, prompt/juez, threshold, configuración ni sampling.
No resultados propios; comparación futura bajo [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

### Defaults, poblaciones y adaptación discriminante

[N10] permite precisar la unidad: **intento HTTP/stream**, no pregunta de
dataset anotado. Config ambiente: ruta `/v1/workflows/run`, connect timeout
10 s, read timeout 60 s; terminales `workflow_finished,error`. `WAIT_TIME=0`
usa espera constante cero; cualquier otro valor toma `between(1,3)`, no ese
valor como demora. Pregunta elegida por `random.choice`; no semilla fijada en
la ruta leída. Archivo opcional: una pregunta no vacía por línea. Fallback:
cinco preguntas generales incorporadas, sin respuestas gold/split.
No es corpus de memoria ni prueba de calidad con ground truth.

`locust.conf:4-28` declara host localhost:5001, 10 usuarios, spawn-rate 2,
duración 1m, headless y `reset-stats=false`. Son defaults upstream, no
configuración propuesta. Invocaciones/entorno pueden cambiarlos; no se conoce
una ejecución efectiva. No hay warmup separado garantizado por ese archivo.
Modelos/embeddings/tokenizer/budgets pertenecen al workflow publicado y no se
fijan en este runner. Una misma etiqueta de endpoint no iguala esos costes.

`connection_started` cuenta intentos antes de validar HTTP. `record_ttfe`
solo ocurre al recibir primer evento; streams sin eventos quedan fuera de
su distribución. `stream_metrics` también requiere `event_count>0`.
Por tanto TTFE/duración no tienen como denominador todos los intentos.
S8:156-175 usa ventanas de muestras acotadas; S8:235-245 calcula p50 mediana
y p95 con `statistics.quantiles(n=20, method="inclusive")`, no percentiles
de toda la historia. Tasas overall usan contadores/tiempo transcurrido;
las tasas de ventana y percentiles no comparten necesariamente población.
Errores JSON pueden contarse sin hacer fallar inmediatamente el stream;
terminal `error` puede ser «success» de transporte (§3).
No interpretar los contadores como exactitud, disponibilidad de fuente o
éxito lógico del workflow.

`ConfigHelper` lee `setup/config/stress_test_state.json` y admite secciones
legacy. Contiene campos de auth/API key: no ejecutar setup sobre una instancia
compartida ni copiar su estado a evidencia pública. No se leyeron valores
secretos. El runner consume API key; no exige regenerarla para cada consulta.
Requests/SSE, cola del workflow, modelos/herramientas y persistencia del
producto están dentro de la latencia end-to-end; import/index build/juez
no son fases instrumentadas de calidad aquí.

**Adaptación futura, no harness nuevo:** solo un adapter del transporte
workflow podría mapear entrada `inputs.question`, eventos, terminal lógico,
error operativo e identidad de petición a la pista producto/agente de METHOD.
Para retrieval de Vanta el seam es su API de búsqueda/explain, no este
protocolo workflow. Mantener intentadas/completadas/fallidas y fases separadas
según METHOD/BENCHMARK-NOTES; no transferir prompts, usuarios, defaults ni
éxito Locust a un score de memoria. No se autoró ni ejecutó adaptación.

## 8. Matriz capacidades Vanta

| Capacidad | Estado baseline | Diferencial a revalidar |
|---|---|---|
| BM25/HNSW/RRF/sparse | Existe | No reemplazar por stack Dify |
| Provenance, sesiones, supersession, dedup | Existe | Referencias diagnósticas y autorización actual |
| Assembler/compaction | Existe | Etapas aplicadas y score domain: no acreditado integralmente |
| Scheduler/checkpoints, ingestors/auditoría | Existe | No incorporar workers/colas equivalentes |
| Publicación de ingesta | Existe worker/CommitReport/WikiStore V5–V8 | C extiende callback por página y revisión de fuente; terminal run_id no prueba fencing global |
| Metadata tipada SDK/ingestors | Existe Value/MemoryInput, V9–V13 | D añade validación schema por fuente, no tipos/SDK paralelo; batch commits por chunk |
| Studio: actividad y diagnóstico retrieval | Existe | Config solicitada vs efectiva: parcial por verificar |
| Exportación metadata-only con política revisionada | No acreditado | Baseline no certifica todas las integraciones |
| Marketplace/workflows/cloud dentro del core | No aplicable | Fuera del objetivo embebido |

## 9. Deuda sugerida sin falsos bugs

Preguntas abiertas: ¿un reporte conserva la configuración realmente aplicada?,
¿es visible cuándo se omite reranking?, ¿los previews respetan retirada de acceso
y las trazas tienen redacción independiente? No se afirma que Vanta carezca de ello.
El contraste sugiere revisar contratos en una tarea posterior autorizada.
No se heredan hallazgos históricos ni se crean bugs por omisión de esta lectura.

## 10. Interfaz/escritorio e ideas Studio

**Código verificado:** pantalla web de hit testing con consultas anteriores,
resultados internos/externos, modificación de retrieval y estado loading/error;
la UI calcula permiso de ejecución, sin que eso sustituya controles backend.
No se acredita un shell desktop nativo en las superficies inspeccionadas.
Tampoco se ejecutaron UI, navegación o comprobaciones de accesibilidad.
Para Studio existente: explicar plan efectivo, etapa omitida y score domain,
con preview autorizado y exportación explícita en RetrievalLens/actividad.
Inspirarse en los conceptos no autoriza copiar diseño o componentes licenciados.
No crear consola alternativa, runtime web obligatorio ni editor visual de flows.

### Canvas, inspector, ejecución y credenciales: fuente real

| Superficie inspeccionada | Lo que acredita | Límite e idea para Studio |
|---|---|---|
| [workflow/index.tsx:35-120][U1] y 350-420 | ReactFlow, nodeTypes y hooks; error de import/restauración; no resetear UI durante run/listening | Estado de editor distinto de estado ejecutado; no portar colaboración/canvas |
| [panel/index.tsx:1-75][U2] | NodePanel, EnvPanel, historial dinámico y layout del inspector | Inspección existente de Studio, no nuevo diseñador |
| [run/status.tsx:1-100][U3] | Consulta detalles de pausa solo con run ID/estado pausado; razones HITL y enlace a tracing parcial | Pausa no es fallo ni «memoria pendiente»; evitar convertir warning de etapa en éxito total |
| [credential-selector.tsx:1-110][U4] | ID/nombre de credencial, selector controlado, opción añadir, marca enterprise | No prueba cifrado ni backend de secretos; no copiar restricciones visuales como ACL |

El recorrido observado es editar/configurar → ejecutar/previsualizar →
seleccionar run/etapa → consultar estado/errores. La configuración del nodo y
la del proveedor son entidades distintas. El selector transporta referencias,
pero su lectura no acredita redacción de todos los formularios o exports.
Errores y outputs pueden contener texto sensible: resolver preview con permisos
actuales, no enviar todo el span a cualquier usuario del canvas.

En Studio se inspeccionó **`feature-freez/desktop/src/components/`**, no una
ruta `desktop/` supuesta: `lens/retrieval/RetrievalLens.tsx:1-130` ya ofrece
consulta, vector-picker, filtros, ScoreBars y contexto; `ConnectionPanel.tsx:
1-75` ya admite perfiles/health. Revisión completa:
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`. RetrievalLens reusa filtros y
obtiene orden del servidor; además tiene filtrado visual de resultados.
No certificar desde esos componentes toda la ejecución backend ni exactitud
global de filtros. A/B extenderían el explain/config efectiva en esas vistas,
sin duplicar consola ni almacenar secretos en el reporte.
Enlaces inmutables:
[RetrievalLens][VS1], [ConnectionPanel][VS2].

## 11. Cobertura, dependencias y validación pendiente

**Autonomía y split plan:** este informe supera 300 líneas por mecanismos
distintos, no por duplicar plantillas. Si se aprueba trabajo, separar futura
spec de explain/diagnóstico (A/B), lifecycle de derivados (C), metadata (D) y
adapter de embeddings (E); mantener aquí evidencia/decisión comparativa.
No se crean esos archivos ni se modifican índices/contratos/backlog.

| Eje | Cobertura conseguida | Laguna que no debe ocultarse |
|---|---|---|
| 0/1 arquitectura y prioridades | SQL/fuente, índices, Celery, Graphon y consola; A→B, C/D/E independientes según adapter | No todo runtime/plugins/KnowledgeFS |
| 2 extracción | Cinco fichas; C/D localizan worker/publicador y entrada metadata SDK V5–V13 | CAS/revisión de fuente propuestos, no implementación ni aceptación ejecutada |
| 3 issues/seguridad | Dos incidencias leídas y defecto de runner separado | PR #45639 no auditada; no pentest |
| 4/5 contratos/rechazos | Seams 97c9623, sin motor/low-code nuevo | Atomicidad efectiva debe acreditarse por operación |
| 6 método/licencia | Apache modificado y confianza por frontera | No dictamen jurídico de obra derivada |
| 7 benchmarks | Runner SSE, helper, defaults, denominadores y efectos | No resultados propios ni benchmark de calidad localizado en muestra |
| 8/9 matriz/negativos | Capacidades existentes preservadas; preguntas no elevadas a bugs | No auditoría completa de Vanta HEAD posterior |
| 10 producto | Canvas/panel/run/credencial + Studio real | Sin UI ejecutada, accesibilidad o cliente nativo Dify certificados |

**Compleción de A/B:** ambos usarían [V1] para búsqueda y [V2] para recall/
assembly; B publica metadata al dueño de diagnóstico, no conserva el texto
por sí mismo. Capturar config revisionada antes de ejecutar A; un override
concurrente no reescribe el recibo del run previo. B resuelve previews con
autorización actual y registra aparte su revocación. Persistencia/export de
diagnóstico no es commit del retrieval. Si se añade un campo, migración
aditiva/lectura backward-compatible y feature opt-in; rollback ignora ese
campo sin reordenar resultados. Alternativa: extender explain existente;
rechazar registro paralelo de runs y usar solo diagnóstico efímero si basta.
A no depende de B ni de C/D/E para funcionar; B puede existir sin exportador.
Errores propuestos: configuración inválida, etapa no disponible, cancelado,
preview denegado y fallo de persistencia de diagnóstico; no un `None` ambiguo.
Aceptar manualmente configuración ausente/override/economy, reranker ausente,
cancelación y exportador fallido; distinguir solicitado/resuelto/aplicado.

**Flujo workflow/state, complemento del mapa:** [N11:153-219] construye
`Engine` Graphon con grafo/runtime state, command channel y número configurable
de workers, añade execution context, límites y observabilidad opcional.
`run` produce eventos, ignora `GenerateTaskStoppedError` en este borde y
convierte otras excepciones en `GraphRunFailedEvent`. El filtro de stream
puede restaurarse al reanudar; no es la memoria factual ni una garantía
de checkpoint/replay de efectos externos. No se traslada ese executor:
las oportunidades aquí usan scheduler/checkpoints y context engine existentes.
API de Knowledge/hit testing (§0.2) y streaming de ejecución son contratos
diferentes, aunque ambos aparezcan en consola.

### Inventario adicional de fuentes fijadas

Los rangos siguientes complementan S1–S9; todos N/U usan el SHA Dify de §0.
No se leyó exhaustivamente cada archivo fuera del rango declarado.

| ID | Archivo/símbolo/rango | Cobertura |
|---|---|---|
| N1 | [models/dataset.py:150-206][N1], 374-435, 554-600, 670-700 | Identidad Dataset/Document/Segment/Query |
| N2 | [document_indexing_task.py:51-140][N2] | `_document_indexing`, admisión/fases SQL |
| N3 | [execution.py:77-178][N3] | `DocumentIndexingService`, extracción/load/error |
| N4 | [hit_testing.py:38-77][N4] | Entrada de consola y guards |
| N5 | [cached_embedding.py:24-133][N5] | `CacheEmbedding`, caché y slots |
| N6 | [document_indexing_update_task.py:36-110][N6] | Limpieza/reinicio de derivados |
| N7 | [metadata_repository.py:155-198][N7] | `update_documents`, batch transaccional |
| N8 | [clean_document_task.py:20-150][N8] | Inventario/cleanup por recursos |
| N9 | [dataset_retrieval.py:1631-1760][N9] | Metadata automática y filtros tipados |
| N10 | [sse_benchmark.py:364-549][N10]; helper 1-125; locust.conf 1-37 | Runner, terminales y defaults |
| N11 | [workflow_entry.py:153-219][N11] | Engine/eventos/límites |
| V1 | [hybrid.rs][V1] | Motor híbrido existente, revisión 97c9623 |
| V2 | [auto_recall.rs:302-335][V2]; context_engine/engine.rs:93-218,228-268 | `RecallConfig`, reportes/assembly ya presentes |
| V3 | [l1_writer.rs:131-176][V3], 289-330,389-430 | Hooks, plan/persist y dedup existentes |
| V4 | [task_checkpoint.rs:122-190][V4]; services/scheduler.rs:196-280 | Lifecycle de tarea/scheduler existentes |
| V5 | [worker.rs:70-309][V5], `begin/execute/ingest_body` | Run, callback publicador y error parcial |
| V6 | [merge.rs:34-49][V6], `CommitReport/MergeEntryError` | Written frente a errores por página |
| V7/V8 | [store.rs:162-217][V7], [287-320][V8] | Guard terminal por run_id frente a put_page individual |
| V9/V12 | [ingestors.rs:45-89][V9], [298-369][V12]; ingest JSON/CSV 122-227 | Input/provenance y conversión tipada |
| V10 | [memory.rs:1223-1350][V10] | Put, put_certified y validación/commit por chunk |
| V11/V13 | [types.rs:109-147][V11], [validate_metadata:252-266][V13] | Value tipado frente a validación de claves, no schema de campo |

**DIFY-1, cierre documental propuesto:** C/D ya identifican owner, identidad,
errores y frontera de commit; FMEA/aceptación anteriores se aplican a esos
símbolos, no a una ruta genérica futura por descubrir. La UI sólo mostraría
recibos/resultados reales; CAS, revisión de fuente y publicación global siguen
contratos propuestos a especificar, sin afirmar implementación ni autoaceptar.

**Búsquedas/fallos externos:** DuckDuckGo/metasearch para issues dio
resultados insuficientes; REST `/search/issues?q=repo:langgenius/dify
is:issue metadata filter numeric` sí devolvió cuerpos/estados. Fetch posterior
`/repos/langgenius/dify/issues/{45638,41597}` recibió rate limit; fallback
HTML de 45638 respondió. Para 41597 se usa el cuerpo/estado de search REST,
no un timeline íntegro. No inventar fecha de cierre/merge.
No se reinició ningún MCP, servicio o aplicación ante esos fallos.

[S9]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/LICENSE#L1-L22
[N1]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/models/dataset.py#L150-L206
[N2]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/tasks/document_indexing_task.py#L51-L140
[N3]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/services/knowledge/indexing/execution.py#L77-L178
[N4]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/controllers/console/datasets/hit_testing.py#L38-L77
[N5]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/embedding/cached_embedding.py#L24-L133
[N6]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/tasks/document_indexing_update_task.py#L36-L110
[N7]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/repositories/knowledge/metadata_repository.py#L155-L198
[N8]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/tasks/clean_document_task.py#L20-L150
[N9]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/retrieval/dataset_retrieval.py#L1631-L1760
[N10]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L364-L549
[N11]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/workflow/workflow_entry.py#L153-L219
[U1]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/app/components/workflow/index.tsx#L35-L120
[U2]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/app/components/workflow/panel/index.tsx#L1-L75
[U3]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/app/components/workflow/run/status.tsx#L1-L100
[U4]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/app/components/header/account-setting/model-provider-page/model-auth/credential-selector.tsx#L1-L110
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/hybrid.rs#L1-L100
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L302-L335
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L131-L176
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L190
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L70-L309
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L34-L49
[V7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L162-L217
[V8]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L287-L320
[V9]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L45-L89
[V10]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1223-L1350
[V11]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types.rs#L109-L147
[V12]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L298-L369
[V13]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/mod.rs#L252-L266
[VS1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L130
[VS2]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/ConnectionPanel.tsx#L1-L75
