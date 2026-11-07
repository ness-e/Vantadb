---
title: "Informe estático — khoj-ai/khoj"
kind: research
status: proposed
description: "Evaluación estática de Khoj (asistente/segunda memoria personal) a profundidad ampliada: indexación, búsqueda, conversaciones y clients"
tags: [research, memory, khoj]
---

# Informe estático — khoj-ai/khoj

> **Identidad fijada:** `khoj-ai/khoj`, rama `master`, clon
> `C:\Users\Eros\khoj`, SHA **`ae229ca894c0b80ad84664afcfdde523b5e87057`**.
> Captura 2026-11-07 de [clone-manifest.json](clone-manifest.json);
> se conserva metadata, sin fetch/pull ni actualización de estrellas/versiones.
> Paquetes web/Desktop/Obsidian declaran `2.0.0-beta.28`; no equivale a
> versión desplegada en Cloud. Referencia Vanta:
> **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**.
> **Autoría estática ampliada:** lectura de código y fuentes públicas;
> no ejecución, revisión independiente ni aceptación final.

## Índice

1. [Arquitectura y mapa](#1-arquitectura-y-mapa)
2. [Recomendaciones](#2-recomendaciones)
3. [Fichas candidatas](#3-fichas-candidatas)
4. [Incidencias y riesgos](#4-incidencias-y-riesgos)
5. [Prerrequisitos Vanta](#5-prerrequisitos-vanta)
6. [Que no trasladar](#6-que-no-trasladar)
7. [Advertencias metodológicas](#7-advertencias-metodológicas)
8. [Benchmarks](#8-benchmarks)
9. [Matriz](#9-matriz)
10. [Negativos Vanta](#10-negativos-vanta) · [Escritorio/UI](#11-escritorio--ui)
11. [Fuentes y cobertura](#12-fuentes-y-cobertura)

## 1. Arquitectura y mapa

### 1.1. Producto, stack, despliegue y licencia

Asistente con corpus personal, chat con herramientas, agentes y **memoria de
hechos derivada de conversaciones**. No se reduce a corpus+historial ni es
una implementación de motor vectorial propia. [pyproject.toml][package]
declara Python >=3.10,<3.13, FastAPI, Django 5.1.15, pgvector 0.2.4,
sentence-transformers 3.4.1 y Torch 2.6; `khoj.main:run` es entry point CLI.
Gunicorn es dependencia `prod`; Uvicorn también figura en dependencias.

| Dominio | Fuente / frontera |
|---|---|
| `src/khoj/main.py`, `configure.py`, `app/` | Arranque, middleware, rutas y tareas; importar settings puede iniciar pgserver si se habilita |
| `database/models/__init__.py`, `adapters/__init__.py` | Django ORM, documentos/chunks, conversaciones, agentes y UserMemory |
| `processor/content/` | Markdown, org, PDF, imágenes, plaintext, DOCX; GitHub/Notion conectores server-side |
| `search_type/text_search.py`, `search_filter/` | Embedding, candidatos coseno, filtros SQL y rerank; no acreditar BM25 por nombre de carpeta |
| `routers/api_content.py`, `api.py`, `api_chat.py`, `helpers.py` | Sync, search, chat HTTP/WebSocket, memoria, límites y telemetría |
| `src/interface/web` | Next/React exportado para servir desde backend; fuente de UI, no solo landing SaaS |
| `src/interface/desktop`, `obsidian`, `emacs`, `android` | Electron, plugin, cliente Emacs y TWA Android: §11 distingue implementación |
| `tests/evals/eval.py`, `tests/`, `pytest.ini` | Evaluador de QA/IR real y tests; no confundirlos |

[settings.py][settings]:121-185 usa **PostgreSQL**, opcional pgserver embebido;
la afirmación previa «SQLite/Postgres» no queda sustentada por esta revisión.
[docker-compose.yml][compose] declara pgvector:pg15, servidor, sandbox Terrarium,
SearxNG y computer; el socket Docker para computer está comentado y el operador
requiere opt-in. Usa imágenes `latest`, volúmenes DB/config/modelos y ejemplos
de credenciales: **no es un manifiesto endurecido ni un despliegue ejecutado**.
No trasladar a Vanta esos servicios como requisito de almacenamiento.

Licencia raíz [AGPL v3][license]; package Python declara AGPL-3.0-or-later.
Desktop y Obsidian declaran **GPL-3.0-or-later** en sus packages; Obsidian
incluye LICENSE GPL v3. Web package es privado sin declaración propia leída:
no inferir permiso diferente de la licencia raíz. Modelos, Android, assets y
dependencias requieren revisar obligaciones por separado. Reescribir en Rust
no garantiza exención de obligaciones de obra derivada.

### 1.2. Modelo persistente y selección de modelos

[models][models]:760-864 separa:

- `FileObject`: texto completo y propietario user/agent; `Entry`: raw/compiled,
  heading, ruta/URL, source/type, hash, corpus UUID, `search_model` y vector.
  `file_object` tiene CASCADE; `EntryDates` agrega fechas extraídas.
- `Conversation`: mensajes y agente; `PublicConversation` es copia compartida,
  no prueba de ACL idéntica a una sesión privada.
- `UserMemory`: user, agent opcional, raw, embeddings y search_model, distinto
  de chunks del corpus. `ServerChatSettings.memory_mode` y preferencia del
  usuario gobiernan activación; desactivar no equivale a borrar.
- `SearchModelConfig` defaults: `thenlper/gte-small`, reranker
  `mixedbread-ai/mxbai-rerank-xsmall-v1`, umbral bi-encoder 0.18; permite
  endpoints de inferencia configurables. `ChatModel` declara default
  `gemini-2.5-flash`. Son **defaults de schema**, no modelo efectivo certificado.
  OpenAI-compatible, Anthropic y Google tienen rutas en helpers; no pesos propios.

### 1.3. Sync e indexación: confirmaciones y errores

1. [Obsidian updateContentIndex][obs-sync]:104-248 filtra archivos elegibles,
   usa mtime frente a lastSync, agrupa por bytes/items y envía vacíos para deletes.
   Regenerate borra tipos habilitados **antes** de subir. [api.ts][obs-api]
   hace PATCH autenticado; error 429 se comunica, no se trata como corpus vacío.
2. [PUT/PATCH content][content]:75-116 exige authenticated y dependencia de
   límites; PUT pasa `regenerate=True`, PATCH False. `indexer`:544-627 separa
   formatos por detección, ejecuta `configure_content` en executor y espera;
   fallo global devuelve 500; 200 devuelve **nombres enviados por tipo**, no
   un ledger durable por archivo indexado correctamente.
3. [configure_content][configure-content]:3009-3181 llama
   `text_search.setup`→procesador por tipo, captura errores por familia y limpia
   query cache del usuario al terminar. Sin documentos del cliente puede usar
   configuración GitHub/Notion. Obsidian no es un tipo de parser: envía archivos.
4. [MarkdownToEntries.process][markdown]:21-50 interpreta texto `""` como delete,
   extrae secciones, divide por tamaño, llama `TextToEntries.update_embeddings`.
   Los nombres `max_tokens`/256 usan tokenizer propio: no certificar equivalencia
   con tokenizer del LLM. `process_single_markdown_file`:86-101 contiene bucle
   sin bound ante ciertos headings (§4), no un error ya corregido por asumirlo.
5. [update_embeddings][ingest]:142-263 calcula hash del compiled, evita embeber
   hashes existentes por user/type, genera embeddings nuevos, actualiza
   FileObject, bulk_create por batch, añade fechas y borra hashes obsoletos.
   Regenerate borra primero. Excepción de bulk_create se loguea y continúa:
   **no hay transacción global observada** que revierta texto, batches y deletes.
   Dedupe por contenido no acredita identidad de revisión del modelo de embedding.

`indexer` no muestra cancelación cooperativa del parser en executor; cancelar
HTTP no mata un hilo ocupado. Retorno del servidor y `lastSync` son fronteras
distintas: Obsidian confirma nombres mediante `response.includes(file.path)`,
no por ID tipado; omisiones, substrings y parse fallido merecen discriminación.
`GithubToEntries.process_repo`:63-84 descarga formatos y comparte extracción
Markdown; su espera de rate limit (`43-48`) es otra frontera bloqueante.

### 1.4. Search, chat, fuentes y tenancy

[GET /search][api]:46-77 toma user autenticado y llama
[execute_search][search]:1484-1574. Comprueba acceso al agente **antes** de cache;
cache por UUID de usuario y clave query/n/type/r/distance/dedupe. Esa clave
no incluye agent/model/revisión de corpus; hay invalidación al configurar
contenido, no garantía observada para toda mutación o cambio de política.

Elimina términos de filtro antes de embeber, mantiene consulta original para
[EntryAdapters.apply_filters][filters]:2090-2147: ownership `user OR agent`,
palabras raw, globs escapados por ruta y fechas extraídas mediante EntryDates.
[search_with_embeddings][filters]:2150-2178 aplica filtros **antes** de distancia,
orden y limit. [text_search.query][text-search]:99-164 obtiene hasta 10 candidatos
y collate conserva URI/ruta/heading; después dedupe/rerank y n final.
No inferir lexical/BM25 de `WordFilter`: aquí es predicado SQL sobre raw.

[event_generator][chat]:676-1486 mantiene monitor HTTP/WebSocket e interrupt,
evento de cancelación, contexto parcial y persistencia de interrupciones.
Chat selecciona comandos/fuentes; corpus, online/code/operator/research y
UserMemory no son la misma colección. `979-983` combina memoria reciente y
semántica por ID cuando está habilitada. `1449-1482` agenda guardado de
conversación con `asyncio.create_task` **antes de END_RESPONSE**: stream
terminado no prueba escritura durable. Presencia de cancelación no acredita
que todas las herramientas/proveedores hayan detenido efectos externos.

[UserAuthenticationBackend][auth]:98-228 acepta credenciales de varias vías;
en `anonymous_mode` usa usuario default autenticado. Esto debe distinguirse de
usuarios separados en servidor compartido. En EntryAdapters sin user ni agent
retorna vacío; union user/agent exige autorización del caller, no permiso
inferido del filtro ni del identificador de agente.

### 1.5. Actualización, borrado y memoria automática

[Delete content][content]:235-282 borra entradas y después FileObject scoped por
user; no demuestra purga de mensajes/copia pública/caches/almacenamiento externo.
Los cambios de archivo y re-embed tienen ventanas parciales descritas en §1.3.

[ai_update_memories][memory-extract]:1038-1067 verifica memoria habilitada,
extrae create/delete y llama adapters. En edición manual (`api_memories.py:83-103`)
el update reemplaza sin propagar `agent` (`agent=None` por defecto en
`UserMemoryAdapters:2311-2326`), por lo que un hecho de agente no default
puede perder su asociación — ver ficha D. Parse inválido en `extract_facts_from_query`
devuelve lista vacía (`1024-1034`); logs incluyen memoria/raw respuesta: no
adoptar esa observabilidad sin política de datos.
El caller leído `processor/conversation/utils.py::save_to_conversation_log`:
541-620 guarda mensajes mediante `ConversationAdapters.save_conversation` y
después llama `ai_update_memories`, salvo automatizaciones. El fallo en generar
memoria no se equipara a fallo del guardado de mensajes ya efectuado.
[UserMemoryAdapters][memories]:2292-2378 guarda embeddings por memoria, recupera
recientes o cercanas y scoped user+agent; default agent puede ver todas las
memorias del mismo usuario, agente no default solo las suyas.
[api_memories.update_memory][memory-api]:70-114 **borra antes de save_memory**,
que puede fallar al embeber; cambia ID. Delete/update verifican user.
No hay supersession transaccional acreditada por tener CRUD/create-delete.

## 2. Recomendaciones

| Prioridad documental | Mecanismo / valor incremental | Dependencia |
|---|---|---|
| P0 | B: recibo de sync por revisión/proyección, no nombres enviados | Estados/progreso/scheduler ya existentes |
| P0 | E: terminación/cancelación de parsers para corpus hostil | Bound computacional e aislamiento del host |
| P0 | F: cache bajo política/modelo/corpus autorizado | Tenancy existente, invalidaciones por mutación |
| P1 | A: gramática opcional→filtro tipado con explicación | AST/filtros del motor, no ranking nuevo |
| P1 | C: referencias por revisión y entrega/contexto parcial | Provenance y context assembly actuales |
| P1 | D: política de memoria efectiva y edición recuperable | Dedup/supersession/checkpoints ya implementados |
| P2 | G: operaciones y capacidades de cliente inequívocas | Transportes Studio/adapters existentes |

No se propone clonar Khoj como producto ni reservar IDs de backlog.
Las referencias C3/C7/C8/C9/C11 de la ficha previa son relaciones conceptuales,
no tipos Rust ni tareas aprobadas.

## 3. Fichas candidatas

### A — `SearchFilter` declarativo empujado al plan de consulta

- **Caso/fuente:** agente pide notas de una fecha/ruta sin perder vecinos válidos.
  [apply_filters/search_with_embeddings][filters]:2090-2178 empuja predicados
  al queryset antes de coseno/topK; [execute_search][search]:1521-1530 elimina
  términos estructurados para embedding. Todas las fichas fijan
  **`ae229ca894c0b80ad84664afcfdde523b5e87057`**.
- **Delta/seam:** Vanta ya tiene filtros/plan en `src/physical_plan/filter.rs`,
  búsqueda `src/sdk/search/{hybrid,vector,explain}.rs` y `RecallScope`.
  Añadir traducción de lenguaje de cliente, no filtro/índice nuevo.
- **Alternativas:** API estructurada existente (preferida para SDK); parser
  opcional para humanos; postfiltrar topK (rechazado como equivalente semántico).
- **Contrato Rust propuesto, no implementado:** `ParsedSearch` recibe texto y
  zona/instante de referencia explícitos; devuelve texto libre+filtro tipado y
  explicación. `InvalidSyntax`, `AmbiguousDate`, `UnsupportedPredicate` rechazan
  sin degradar a búsqueda sin filtro; el filtro nunca amplía scope autorizado.
- **Pasos/persistencia/migración:** traducir a AST existente; validar campos;
  pasar a búsqueda y explain; UI rotula filtro de servidor versus visual.
  Si se guardan búsquedas, persistir versión de gramática y referencia temporal;
  legado sigue API estructurada; rollback deshabilita parser, no altera índices.
- **Ownership/cancelación:** caller autorizado construye scope; parser puro,
  sin I/O, cancelable antes de consultar. No resolver «ayer» de nuevo en cada
  página ni reinterpretar consulta guardada con reloj distinto.
- **Dependencias/FMEA:** F incluye AST en identidad de cache. Fecha local frente
  UTC → resultados distintos: detectar rango resuelto y exponerlo; error de
  parsing silencioso → ampliar corpus: detectar tokens no consumidos y rechazar.
- **Criterios manuales futuros:** ruta+fecha normal preserva citas y límite;
  glob escapado, negación, zona ambigua y sintaxis rota tienen resultado explícito;
  explain confirma pushdown, no solo resultados filtrados visualmente.
- **No-goals:** no copiar literal DSL/regex de Khoj, lenguaje SQL ni pesos nuevos.

### B — Ingestores por conector con manifest de revisión

- **Caso/fuente:** host espera que «sync completo» haga consultable una nota.
  [indexer][content]:544-627 devuelve nombres recibidos; [update_embeddings][ingest]:
  142-263 puede capturar fallos por batch; [Obsidian][obs-sync]:227-248 confirma
  por substring de nombre. #1363 muestra necesidad operativa, no benchmark.
- **Delta/seam:** `vanta-memory/src/ingest/worker.rs::{run_with_progress,
  IngestReport}`, `callback.rs::{IngestProgress,ProgressTracker}` y
  `auto_sync.rs::AutoSyncScheduler` ya existen; run_id/busy/late packet guard
  también. Delta: manifest durable **por fuente/revisión/destino** y error,
  no una nueva cola observable ni otro checkpoint.
- **Alternativas:** mtime+nombre (barato, insuficiente para confirmación);
  reindexar todo (coste y ventana destructiva); extender ingesta actual con
  fingerprint source/parser/model y reconciliación selectiva.
- **Contrato Rust propuesto:** `SyncReceipt(source_id, revision, target,
  derivation_fingerprint, operation_id)` devuelve `Unchanged`, `Applied`,
  `Pending`, `Skipped(reason)`, `Failed(stage)` o `Deleted`.
  Timeout/rate-limit no se representan como documento vacío/tombstone.
- **Pasos/persistencia:** registrar plan; derivar sin borrar revisión activa;
  confirmar por ID exacto; activar y limpiar proyección antigua; guardar
  pendientes/tombstone. Migrar mtime/estados legacy como desconocidos; rollback
  deja lectura de revisión anterior y conserva tombstones/reintentos.
- **Ownership/cancelación/commit:** exclusión por fuente+destino y expected
  revision, no lock global; delete concurrente gana a derivación vieja.
  Cancelación detiene nuevas fases, registra efectos ya confirmados; no supone
  matar executor remoto ni revertir transacción inexistente.
- **Dependencias/FMEA:** E limita parsers; F invalida cache; C cita revisión.
  Batch parcialmente fallido pero nombre confirmado → falsa cobertura:
  detectar IDs sin commit y devolver parcial. Cambio de parser/modelo con hash
  igual → derivado obsoleto: comparar fingerprint y rederivar solo afectados.
- **Criterios futuros:** dos syncs iguales no emiten mutación innecesaria;
  rename, fuente borrada, fallo en un batch, restart y proveedor 429 conservan
  clasificación y retry; archivo vacío legítimo no borra salvo operación explícita.
- **No-goals:** no conectores SaaS nuevos ni bytes/modelos descargados; SSRF,
  permisos de raíz y secretos **sí son fronteras**, no autorización para diferirlos.

### C — Cita al origen en resultados conversacionales

- **Caso/fuente:** respuesta mezcla notas privadas, web y recuerdos de usuario.
  [collate_results][text-search]:141-164 conserva ruta/URI/heading; [chat][chat]:
  985-998 recupera contexto interrumpido y 1449-1470 agenda persistencia de
  referencias separadas por clase. Referencia no prueba soporte factual.
- **Delta/seam:** provenance e `IntegratedContext`/`CompactionReport` ya existen
  en `vanta-memory/src/context_engine`; extender
  `engine.rs::{assemble_inner,assemble_with_recall}` y
  `core/hooks/auto_recall.rs::perform_auto_recall_governed` con entrega final.
- **Alternativas:** guardar hits brutos (no prueba entrega), guardar todo prompt
  (riesgo privado), manifest mínimo por turno/revisión (candidato).
- **Contrato Rust propuesto:** `ContextEvidence` recibe referencias autorizadas
  y resultado de assembly; devuelve source_id/revision, clase corpus/history/
  user-memory/externo, incluido/descartado/compactado y entrega parcial/final.
  `SourceUnavailable`, `PolicyChanged`, `MissingRevision` no inventan evidencia.
- **Pasos/persistencia/migración:** propagar identidad desde B; generar manifest
  después del presupuesto; confirmar lo enviado y persistir mínimo en sesión;
  Studio navega a revisión autorizada. Legado = unknown; rollback deja citas
  históricas legibles sin inventar revisions ni eliminar provenance actual.
- **Ownership/cancelación:** host dueño del envío; append parcial al interrumpir,
  no marcar completo porque terminó WebSocket. Concurrente delete/revocación
  reautoriza apertura; hash/puntero no permite recuperar payload sin permiso.
- **Dependencias/FMEA:** B freshness, D política y F scope. Ruta renombrada →
  cita equivocada: detectar identidad/revisión, no resolver por filename actual.
  Contexto hostil → instrucciones: preservar taint/InjectionPolicy existentes.
  Logs de IDs/revisiones, no extractos por defecto.
- **Criterios futuros:** nota+online+memoria quedan distinguibles; respuesta
  interrumpida muestra parcial; fuente borrada no expone contenido; presupuesto
  excluye evidencia sin presentarla como utilizada.
- **No-goals:** no juez de grounding ni score de veracidad; el manifest explica
  entrada/salida del sistema, no pensamiento interno del modelo.

### D — Política efectiva y edición recuperable de memoria

- **Caso/fuente:** usuario desactiva memoria o edita un hecho sensible.
  [ConversationAdapters][memory-policy]:1623-1695 combina server mode+preferencia;
  [ai_update_memories][memory-extract]:1048-1066 lo verifica;
  [update_memory][memory-api]:70-114 borra antes de re-embedding y cambia ID.
  **Alcance de agente en edición (leído):** `api_memories.py:83-103` carga la
  memoria por user/id, la elimina y llama
  `save_memory(user=user, memory=new_content)` **sin agent**;
  `UserMemoryAdapters:2311-2326` tiene `agent=None` por defecto y crea la
  fila nueva sin asociación. En `2301-2306` y `2342-2345`, agentes no default
  leen solo sus memorias; el default lee todas las del usuario.
  `delete_memory:2357-2363` filtra user/id, **no agent**;
  `helpers.py:1064-1066` lo invoca para deletes sugeridos por extracción.
  Consecuencia estática: editar desde settings un hecho asociado a un agente
  no default puede retirarlo del recall de ese agente y convertirlo en
  memoria sin asociación, además del cambio de ID. Lección de contrato:
  autorización por operación (lectura user+agent vs edición/borrado user/id);
  sin fuga entre usuarios acreditada.
- **Delta/seam:** Vanta ya posee sesiones, dedup/supersession y writer
  `core/record/l1_writer.rs::{plan_write,persist_planned,apply_dedup_batch}`;
  integrar preferencia/política efectiva y revisión de edición sobre ese flujo.
  No copiar CRUD destructivo ni recrear mecanismo cognitivo.
- **Alternativas:** interruptor solo UI (no enforcement); disabled=purga
  automática (semántica peligrosa); distinguir capture/recall/edit/delete y
  política efectiva del servidor (candidato).
- **Contrato Rust propuesto:** `MemoryPolicyDecision` recibe principal,
  operación y revisión de política; devuelve allowed/denied+motivo.
  `ReviseFact(id, expected_revision, replacement)` devuelve nueva revisión/ID
  estable o conflicto; `EmbedFailed` conserva hecho activo, no éxito vacío.
- **Pasos/persistencia:** resolver política; planificar reemplazo; validar
  propiedad de IDs sugeridos por LLM; derivar; persistir/activar y marcar
  supersession mediante flujo existente. Versionar política y outcome, no raw
  respuesta. Legado mantiene preferencia explícita; rollback suspende nuevos
  updates sin resucitar borrados ni cambiar opt-in del usuario.
- **Ownership/concurrencia/cancelación:** política revalidada antes de publicar;
  CAS por revisión, no last-writer-wins silencioso. Cancelación previa al efecto
  no borra; posterior devuelve applied. LLM no autoriza IDs ni borra otra tenancy.
- **Dependencias/FMEA:** B commit derivado, C origen, F cache invalidation.
  Modelo inválido confundido con NoFacts → pérdida: outcome tipado, retry.
  Disable durante extracción → persistencia no deseada: detectar policy revision
  y denegar antes de commit. Auditoría de decisiones sin hechos sensibles.
  Edición exitosa sin asociación → hecho fuera del recall del agente original:
  comparar asociación anterior/propuesta y conservarla o exigir cambio explícito.
- **Criterios futuros:** servidor disabled domina preferencia; default/user
  override explícitos; desactivar no borra implícitamente; edición fallida,
  concurrente o con ID ajeno conserva invariantes y produce error distinguible.
  Distinguir ID de otro usuario de otra asociación del mismo usuario; una
  edición conserva asociación o informa su cambio intencional y efecto en recall.
- **No-goals:** no prompts/rúbricas ni copiar default-agent global por usuario
  como permiso universal; retención legal requiere decisión separada.

### E — Parser con progreso acotado y cancelación real

- **Caso/fuente:** nota hostil bloquea sync de todo el vault.
  [process_single_markdown_file][markdown]:86-101 y [#1437](https://github.com/khoj-ai/khoj/issues/1437)
  muestran regex de detección más amplia que regex de split y bucle sin bound.
  Issue refiere exactamente el SHA fijado; no reproducción propia.
- **Delta/seam:** `ingest/worker.rs` ya coordina y emite progreso; el worker
  invoca `src/wiki/sources.rs::scan_local_sources` y
  `src/wiki/chunker.rs::chunk_text` antes de `ingest/merge.rs::commit`.
  Evaluar bound/cancelación del parser invocado, no modificar HNSW o scheduler.
- **Alternativas:** timeout del HTTP (no mata hilo); límite monotónico por
  parser; proceso aislado para parsers no cooperativos. Preferir progreso
  comprobable y aislamiento solo donde lo exija la librería.
- **Contrato Rust propuesto:** `ParseOutcome(source_revision, parser_version,
  cancellation)` devuelve fragmentos o `Unsupported`, `NonProgress`,
  `LimitExceeded`, `Cancelled`; toda iteración consume entrada o reduce medida
  finita. Límites los define política existente, no números inventados aquí.
- **Pasos/persistencia:** identificar ciclos recursivos; asegurar medida de
  progreso; comprobar cancelación en fronteras; reportar fallo por archivo B
  sin activar derivados. Persistir solo versión/outcome; legado puede seguir
  parser previo solo si explícitamente seguro. Rollback deshabilita formato
  problemático, no reintroduce procesamiento ilimitado.
- **Ownership/concurrencia:** host conserva cancel token; dropping future no
  prueba parada de librería bloqueante. Workers aislados no publican un commit
  si la revisión fue cancelada; no retener lock de escritura al parsear.
- **Dependencias/FMEA:** B informa cobertura y G cliente no confirma de más.
  Heading sin título → bucle → CPU/cola agotada: detectar no progreso y cortar;
  límite excesivo → rechazo válido: mostrar razón y conservar original, no borrar.
- **Criterios futuros:** Markdown normal conserva headings/line refs; heading
  vacío/tab, documento largo y cancelación terminan con resultado explícito;
  otros archivos continúan sin publicar derivados incompletos.
- **No-goals:** no copiar parser GPL/AGPL, nuevo formato ni afirmar bug Vanta
  sin encontrar el mismo ciclo y reproducirlo por separado.

### F — Identidad de cache bajo principal, modelo y revisión

- **Caso/fuente:** misma consulta tras cambiar agente, contenido o política.
  [execute_search][search]:1514-1519,1567-1569 usa cache por usuario sin agent/
  model/corpus revision en clave; [configure_content][configure-content]:3177-3179
  invalida al configurar. No se leyó toda invalidación: riesgo, no exploit probado.
- **Delta/seam:** Vanta ya autoriza inyección en `auto_recall.rs::InjectionPolicy`,
  sella tenancy en `l1_writer.rs::default_tenancy` y busca con filtros existentes.
  Verificar caches del host/adapters que envuelven recall, no instalar cache
  nueva en motor por analogía.
- **Alternativas:** no cachear contenido autorizado (simple/seguro); invalidar
  por usuario (grueso); clave de scope+política+modelo+revisión (candidato si
  existe cache y necesidad). TTL solo no sustituye revocación.
- **Contrato Rust propuesto:** `RecallCacheIdentity` recibe principal_ref,
  scope autorizado, filtro normalizado, fingerprint embed/ranking y revisión de
  corpus/política; devuelve hit solo si identidad coincide. `Stale`, `Forbidden`,
  `UnknownRevision` fuerzan miss/denegación, nunca éxito con resultado anterior.
- **Pasos/persistencia:** inventariar cache real; unir autorización antes de hit;
  propagar revisión/invalidation desde B/D; no persistir payloads si no hace falta.
  Migración descarta cache sin versión; rollback bypass cache, no relaja ACL.
- **Ownership/cancelación:** principal/política del servidor, no caller libre;
  cambio concurrente invalida publicación/hit. Cancelar consulta no escribe
  entrada parcial ni mueve watermark.
- **Dependencias/FMEA:** A aporta AST, B corpus, D política. Hit cross-agent →
  contexto indebido: detectar fingerprint mismatch y reautorizar. Modelo cambia
  dimensión → error: comparar schema/fingerprint; no asumir score comparable.
- **Criterios futuros:** consulta repetida válida mantiene permiso; delete,
  cambio de agente/política/embedding o parser no reutilizan hit viejo; caché
  legacy y cancelación producen miss seguro sin texto sensible en logs.
- **No-goals:** no mejora de performance prometida, benchmark nuevo ni afirmar
  fuga Vanta por parecido de clave upstream.

### G — Capacidad y confirmación por operación de cliente

- **Caso/fuente:** UI muestra sync terminado al volver IPC, aunque sigue HTTP.
  [Desktop main][desktop]:600-602 llama `syncData` sin retornar await;
  [Obsidian][obs-sync]:104-248 espera batches y actualiza mtime por nombres.
  Ambas superficies reales, pero con semánticas diferentes.
- **Delta/seam:** Studio ya tiene `desktop/src/transport.ts`, `vanta.ts` y
  conexiones; `RetrievalLens.tsx::runSearch` usa `search({explain:true})`.
  Extender capacidades/ack/error de transportes existentes, no crear otra app.
- **Alternativas:** booleano connected (no acredita capacidad); misma firma
  para todo (puede ocultar unsupported); descriptor por conexión/operación con
  resultado discriminado B y cancelación soportada explícita.
- **Contrato Rust propuesto:** `OperationAck(connection_id, operation_id,
  revision)` → accepted/applied/partial/failed, capability revision y soporte
  cancel. `UnsupportedTransport`, `Disconnected`, `PermissionDenied` no disparan
  fallback silencioso a otro destino.
- **Pasos/persistencia/migración:** mapear comandos reales; enlazar ID a B;
  mostrar fase/proyección y no emitir éxito desde retorno de IPC; conservar
  selección local mientras hay fallo recuperable. Versionar prefs/capacidades,
  no tokens en memoria; legado unknown. Rollback rotula capacidad desconocida.
- **Ownership/cancelación:** host conserva operación y alcance elegido;
  desconectar UI no borra ni cancela implícitamente; reconectar consulta estado,
  no reenvía sin clave idempotente; elegir otra conexión no retargetea trabajo.
- **Dependencias/FMEA:** B receipts, E parser, F policy. Acks fuera de orden →
  falso éxito: comparar operation/revision; path retornado por fuente hostil →
  apertura externa: validar esquema y raíz, confirmar egress/acción.
- **Criterios futuros:** conexión soportada funciona; transporte sin edit/cancel
  lo declara; offline, respuesta tardía, restart y cambio de conexión no confirman
  otra operación ni abren rutas arbitrarias.
- **No-goals:** no portar Electron/Obsidian/TWA ni prometer paridad por compartir API.

## 4. Incidencias y riesgos

Páginas públicas completas leídas el 2026-11-07. Estado observado del issue no
certifica versión publicada, ni un enlace Development demuestra merge.

| Incidencia / estado | Reporte y contraste estático | Aplicabilidad |
|---|---|---|
| [#1437 Markdown indexing never finishes](https://github.com/khoj-ai/khoj/issues/1437), **open**, abierto 2026-09-26 | Autor refiere SHA completo de este informe, heading sin título en nota larga y bucle regex sin progreso. Código `markdown_to_entries.py`:86-101 concuerda. Enlaza #1440; PR/diff no leídos, no afirmar merge ni ejecución propia | E: garantizar terminación real del parser; timeout HTTP solo no libera executor. No prueba fallo de parser Vanta |
| [#1363 Obsidian no per-file sync status](https://github.com/khoj-ai/khoj/issues/1363), **open**, abierto 2026-07-01 | Reporte Windows/Docker/vault de más de 500 notas; pide indexed/skipped/failed por archivo. Fuente actual tiene progreso de batches/summary, no recibo durable por revisión acreditado | B/G: confirmación exacta y cobertura, no inferir que todo sync falla ni negar mejoras presentes |
| [#759 Force Sync never resolving](https://github.com/khoj-ai/khoj/issues/759), abierto 2024-05-26, **closed completed** 2024-09-29 | Maintainer dice añadir aviso al superar cuota y limitar formatos; no commit corrector identificado ni prueba de causa única del bloqueo. Respuesta del autor agradece, no reproducción verificable | Diferenciar cuota, lote, formato y parser; B/E/G. Cierre no demuestra solución universal de los bloqueos posteriores |

**Riesgos estáticos propios, no issues atribuidos:** cache no declara agent/
model/revisión en clave; update memory destruye antes de re-embebir; sync puede
retornar nombres enviados pese a fallo de entrada/batch; guardado de chat es
asíncrono respecto al final del stream. Cada riesgo tiene detección/mitigación
en A–G. Revisar auth anónima, paths/URLs, secretos de conector, logs de raw y
contenido hostil sin afirmar vulnerabilidad explotable no demostrada.

## 5. Prerrequisitos Vanta

1. **A/F:** mapear filtros de motor/explain y caller real de recall. Hay
   BM25/HNSW/RRF, pero `RecallConfig.core_search` default false en el snapshot:
   no atribuir la ruta híbrida a toda consulta de memoria.
2. **B/E/G:** reutilizar worker, `ProgressTracker` best-effort, run_id y
   `AutoSyncScheduler` antes de proponer manifest más fino. No reemplazar
   checkpoint de ejecución por documento ni viceversa.
3. **C/D/F:** identificar principal, política y revisión en writer/recall/
   assembly; distinguir provenance de generación frente a entrega al modelo.
   Verificar errores de la frontera antes de exigir «atomicidad» universal.
4. **Studio:** mapear capacidades Tauri/HTTP/WASM existentes y permisos al
   navegar fuentes. Gate de red/autenticación de Vanta ya existe según
   [referencia](VANTA-REFERENCE.md); no importar modo anonymous como default.

Ninguno es bloqueo universal ni defecto Vanta confirmado. No se implementa
ningún contrato candidato ni se altera backlog.

## 6. Que no trasladar

- Plataforma Python/Django, chat/operador/sandbox/billing/servicios Docker como
  dependencia del motor Rust: resuelven producto, no primitives.
- Delete-before-embed, vacío como señal de delete, ack por substring y excepción
  logueada como sustituto de receipt: semánticas ambiguas/destructivas.
- Top10 fijo/umbral/reranker de Khoj como defaults Vanta; dimensiones, score de
  distancia y modelo no son intercambiables con BM25/RRF o memoria factual.
- Cache usuario-only o default-agent all-memories como permiso multiagente
  universal. Namespace y selección UI tampoco autorizan lectura/escritura.
- Electron/Obsidian/TWA como productos nuevos ni código GPL/AGPL incorporado
  sin revisar obligaciones. Reimplementar ideas no garantiza exención legal.

## 7. Advertencias metodológicas

Se preservan observaciones válidas de conectores/filtros/citas/clientes, pero
se corrigen tres simplificaciones: DB observada es PostgreSQL; sí hay UserMemory
y extracción automática; sí existe runner `tests/evals/eval.py`. La ampliación
lee funciones, no infiere comportamiento por carpetas. Clon shallow no demuestra
historia causal ni qué se desplegó en SaaS.

Tests `test_memory_settings.py` describen política e intención de scoping;
no se ejecutaron. `pytest.ini` usa Django settings y `--reuse-db`: importar
settings/arrancar suite no es inocuo. Defaults de modelos no acreditan modelo
efectivo. Resultados reportados por un autor upstream siguen siendo reportes.
No inventar SSRF/RCE o fuga Vanta desde un path/flag parecido; se requieren caller,
precondiciones y reproducción discriminante que esta tarea no ejecuta.

## 8. Benchmarks

### 8.1. Entry point, datos y preparación existentes

**Sí hay runner**: [tests/evals/eval.py][eval], lectura `main`:623-707,
`parse_args`:605-620. CLI `--dataset` default `frames`; opciones `frames_ir`,
`simpleqa`, `gpqa`, `math500`, `--output`. No se ejecutó ni se diseñó otro.

| Familia / loader leído | Unidad y split / preparación | Evaluador real |
|---|---|---|
| FRAMES (`185-205`) | `google/frames-benchmark`, test, Prompt/Answer/reasoning_types; shuffle opcional y muestra opcional | Gemini compara respuesta con gold |
| FRAMES IR (`112-183`, `638-643`) | KB `parasail-ai/frames-benchmark-wikipedia` train; archivo con IDs de pregunta+URL codificada, texto no vacío; compara archivos ya indexados y PATCH faltantes en batches | `evaluate_response_for_ir`:393-431; gold es ID de prompt, no respuesta textual |
| SimpleQA (`208-248`) | CSV OpenAI público, problem/answer; normaliza a Prompt/Answer. **reasoning_types lee metadata de la primera fila para todas** | Gemini; agregado por tema puede quedar mal etiquetado |
| GPQA (`251-320`) | `Idavidrein/gpqa`, gpqa_diamond train; choices barajadas con seed si provista; letra correcta conocida por loader | Match MCQ (`434-464`), no judge Gemini |
| MATH500 (`323-344`) | `HuggingFaceH4/MATH-500` test; problem/answer/subject normalizados | Gemini; fallback juez flash-lite en esta rama |

No revisión de dataset fijada en `load_dataset`, ni dev/holdout propio
congelado acreditado. El nombre FRAMES IR codifica IDs gold para **juez**;
como nombre de archivo también se envía al sistema indexado: posible canal
de anotación que debe registrarse, no asumir aislamiento gold/retrieval.
No se descargaron datasets ni validaron sus licencias/contenidos completos.

### 8.2. Modelos, defaults y cadena de llamadas

`KHOJ_URL` default localhost:42110; bearer opcional; modo default `"default"`
prefijado al prompt por runner; `get_agent_response`:347-375 POST `/api/chat`
con `create_new=True`, timeout 1200 s; devuelve response/usage/references.
El modelo answerer y embedder efectivos los decide servidor, **no el runner**.
Juez general default `gemini-2.5-flash`, MATH500 fallback
`gemini-2.5-flash-lite`; `KHOJ_LLM_SEED` se envía al juez, no prueba
determinismo del answerer. `DATASET_SEED` opcional; randomize false por defecto.
SAMPLE_SIZE opcional; batch size configurable o 10 sin muestra, y default
**inválido para muestras pequeñas (leído, no ejecutado):**
`eval.py:44-48,664` calcula
`BATCH_SIZE=int(os.getenv("BATCH_SIZE", int(SAMPLE_SIZE)/10 if SAMPLE_SIZE else 10))`;
con `SAMPLE_SIZE` 1..9 sin override el resultado es **0** y
`range(0, dataset_length, BATCH_SIZE)` tiene paso cero — el runner falla
**antes de emitir preguntas**: es default inválido, no fallo de calidad del
answerer ni recall cero. Separado de None/dropna y del fallo en futures.
ThreadPoolExecutor usa workers por defecto;
cada batch corre secuencialmente con sleep 1 s, 3 s en general.
No se inventa presupuesto de tokens ni concurrencia efectiva del servidor.

### 8.3. Métricas, denominador y fallos que sesgan lectura

- **Diferencia de predicado población indexable vs gold (leído):**
  `eval.py:152-157` (`index_frames_kb`) omite textos vacíos al indexar;
  `93-105` (`get_articles_by_prompt_id`) filtra solo `frames_prompt_id`, y
  `408-410` usa `len(articles)` como denominador **sin** el filtro notempty
  ni contraste de hashes/revisión de archivos ya indexados. El denominador
  gold **puede** incluir artículos que ese camino no subió: ocurrencia real
  y magnitud **desconocidas** (sin conteos); pendiente conciliar inventario
  de fuentes en ejecución futura. No se cambia fórmula, dataset, scoring ni
  muestreo.
- IR cuenta archivos únicos cuyas listas de IDs contienen el gold;
  precision divide por archivos recuperados, recall por artículos gold del
  dataset filtrado. Calcula F1 para explicación **pero retorna recall** (`428`).
  El agregado llamado Overall Accuracy es media de recalls, no accuracy QA/F1
  ni recall-all de sesiones de Vanta. `0/0` devuelve 1 en helper (`378-385`).
- Gemini recibe pregunta, respuesta y gold; decision se convierte a 1 solo si
  string upper es TRUE. Error/503 detectado devuelve None. No es prueba de
  éxito de agente ni juez calibrado por esta investigación.
- Timeout/API error devuelve respuesta vacía; `process_batch`:535-538 asigna
  None. `main`:679-684 **dropna** excluye fallos/vacíos del denominador. El
  running_total_count incrementa solo para decision no None. Reportar número
  intentado, emitido y evaluable por separado según el método ya fijado.
- `process_batch`:565 divide por running_total_count sin guardia. Si aún es
  cero tras un error, puede lanzar ZeroDivisionError y cortar ese batch.
  `wait(futures)` no llama `future.result()` en `main`:675-676, por lo que esa
  excepción puede dejar preguntas no emitidas. Es inferencia del flujo, no
  reproducción. `len(results)` puede diferir del dataset pretendido.
- No se mide abstención como categoría separada; vacío no es abstención válida.
  Seeds/caches/config del servidor pueden cambiar resultados; create_new
  separa conversaciones, no garantiza reset completo de memoria/corpus/cache.

### 8.4. Coste, efectos y adaptación limitada

Coste suma `usage.cost` de answerer más tokens del juez por tabla helper;
usage ausente aporta cero. No garantiza facturación completa de embeddings,
indexación, búsqueda web, retries, llamadas fallidas o herramientas. `timer`
de carga y sleeps no son latencias p50/p95/p99 por fase; sin warmup/medición
separada acreditados. Se escriben CSV y resumen, se loguean prompts/gold/
respuestas/referencias y se hacen llamadas pagadas y descargas.
FRAMES IR **muta corpus por PATCH**, consulta archivos existentes sin reset
aislado y crea conversaciones; no purga propia garantizada. No iniciar ese
runner contra usuario/corpus compartido ni importar settings para validarlo.

Adaptación futura solo bajo [METHOD](METHOD.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md), sin prompts, rúbricas, scoring, configs
ni harness nuevos: un adapter al sistema Vanta que aloje answerer tendría
que mapear query→respuesta/usage/references y source IDs→archivos gold en
FRAMES IR. El motor Vanta por sí solo no cumple contrato de chat/agente.
Para retrieval preservar identidad de fuentes y distinguir recall por artículo
de sesiones/turnos; no comparar con LoCoMo sintético/fallback ni con canonical
P99 como si midieran lo mismo. Corregir un denominador requeriría autorización
posterior; aquí solo se documenta su comportamiento, no se modifica.

## 9. Matriz

Vanta snapshot **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**;
no «estado actual» de cambios ajenos. [VANTA-REFERENCE](VANTA-REFERENCE.md)
es referencia de capacidades, no certificación de ejecución.

| Capacidad | Khoj SHA fijado | Vanta snapshot / evidencia | Decisión |
|---|---|---|---|
| Sync incremental | Existe mtime/hash+bulk; receipt por revisión parcial | Existe `ingest/worker.rs`, `auto_sync.rs`, wiki run_id | B extiende manifest, no scheduler nuevo |
| Filtro por fecha/ruta/palabra | Existe SQL pre-limit | Existe filtros `src/physical_plan/filter.rs`, SDK search | A gramática/explain opcional |
| Retrieval híbrido | No acreditado BM25/RRF en vía leída; coseno/rerank sí | Existe BM25/HNSW/RRF, `src/sdk/search` | No importar motor ni negar lexical Vanta |
| Provenance y citas | Existe ruta/URI/heading, revisión fina parcial | Existe record provenance/context diagnostics | C entrega/revisión, no provenance desde cero |
| Memoria automática | Existe extracción UserMemory; update destructivo | Existe writer/dedup/supersession/lifecycle | D política/outcome sobre lo existente |
| Tenancy | Lectura user/agent; edición/delete user/id; editar puede perder asociación de agente | Existe `RecallScope`, `InjectionPolicy`, agent/team L1 | D conserva o declara cambio de asociación; F cache ligada a política; verificar cada operación |
| Sesiones/checkpoints | Existe chat e interrupciones; no mismo recurso | Existe sesiones, TaskCheckpointManager/pipeline checkpoint | No sustituir estado de tarea por hechos |
| Budgets/quotas/compaction | Límites de corpus/modelos; equivalencia no acreditada | Existe recall/compaction, quotas `src/config.rs`:760-762 | No copiar cuotas ni prometer token exacto |
| Parser bounded | No acreditado en Markdown; #1437 y bucle | Terminación de todo parser no acreditada por muestra | E verificación específica, no bug local declarado |
| UI/clients | Existe Electron, Next, plugin, Emacs; Android TWA | Studio existe en worktree, retrieval/escenas/skills | G extiende conexiones, no apps nuevas |
| Bench QA/IR | Existe eval.py con fallos de denominador | Existen canonical P99/LongMemEval/LoCoMo, niveles distintos | No tabla competitiva ni harness nuevo |

## 10. Negativos Vanta

Ninguna lectura acredita bug nuevo de Vanta por analogía. Deltas **condicionados**
a verificar caller: recibo fino por fuente/proyección, política efectiva visible,
cache invalidation y evidencia de entrega. No negar progreso/ACL/budgets/dedup
existentes. `TaskCheckpointManager` advierte at-least-once por efecto antes de
advance; eso no demuestra corrupción del WAL ni atomicidad universal.

Observación acotada de Studio en §11: filtros visuales posteriores al topK no
equivalen a pushdown, pero pueden ser intencionales si se rotulan así. Requiere
decisión UX, no «arreglar motor» desde una diferencia de interfaz.

## 11. Escritorio / UI

### 11.1. Clientes cuya implementación se leyó

| Superficie | Código y flujo efectivo | Límite / consecuencia |
|---|---|---|
| Web Next/React | `web/app/search/page.tsx::search`:656-675 GET `/api/search`; filtros de archivo construidos en query; lista de archivos paginada `678-694` | catch loguea error sin reset visible de loading en ese bloque; no ejecución propia ni diagnóstico de toda UI |
| Web chat | `web/app/chat/page.tsx`:492-536 usa mismo host `/api/chat/ws`, manda q/conversation_id/stream, localización/imágenes/archivos opcionales | Selección de fuente y contenido enviado importan; no asumir que toda respuesta usa solo corpus local |
| Web settings/memories | `web/app/settings/page.tsx`:660-730 GET/DELETE/PUT `/api/memories` y PATCH `/api/user/memory`; actualiza lista solo tras response.ok | Edit devuelve ID nuevo y puede perder asociación, retirando el hecho del recall del agente original; sigue perteneciendo al usuario. Disable no purga hechos; la pantalla no sustituye política server-side |
| Desktop Electron | `desktop/main.js::createWindow`:443-497 BrowserWindow con preload, `nodeIntegration:true`, HTML local de settings; cron invoca sync; `preload.js`:32-76 expone files/folders/host/token/sync IPC | Existe main/preload, a diferencia de AnythingLLM shell no disponible en su checkout. No inferir seguridad del aislamiento ni explotación sin analizar navegación/HTML |
| Desktop sync | `pushDataToKhoj`:159-275 reúne archivos/carpetas, usa mtime, vacíos para deletes, PUT/PATCH por grupos y lastSync; IPC sync `600-602` no retorna await | Retorno de invoke no acredita indexación; estado global por grupo no manifest por revisión |
| Desktop abrir archivo | `renderer.js::makeFileElement`:33-68 usa path y openFileAPI; `main.js`:605-610 pasa ruta a shell.openPath | Frontera OS explícita, no copiar apertura arbitraria a Studio; `setWindowOpenHandler`:473-492 confirma enlaces externos salvo prefijo hostURL |
| Obsidian plugin | `main.ts`:99-110 comando sync y `156-171` timer→`utils.ts::updateContentIndex`→`api.ts::uploadContentBatch`; configura formatos, progreso batches y Notice | No backend embebido; usa URL/token configurados. Conteo batch no coverage exacta; §1.3/B |
| Emacs Lisp | `khoj.el::khoj--send-index-update-request`:432-455 multipart PUT/PATCH bearer; `khoj--query-search-api-and-render-results`:715-730 GET search con tipo/rerank/n | Cliente real, no paridad funcional Desktop/Obsidian certificada |
| Android | `android/twa-manifest.json`:1-15 declara package dev.khoj.app, host app.khoj.dev, standalone | **Trusted Web Activity**, no motor nativo/offline acreditado; no se inspeccionaron todos Gradle/Java |

La distribución Cloud y el self-host pueden servir la misma UI sin tener
idénticos modelos, auth o permisos. No se encontró evidencia leída de una app
iOS nativa: no inventarla por mencionar móviles en README/issues.

### 11.2. Transferencia a Studio, no producto nuevo

Referencia de Studio en [VANTA-REFERENCE](VANTA-REFERENCE.md):
worktree `C:\Users\Eros\VantaDB Proyect\develop-desktop`, revisión observada
`0ad38c542` (no se recalculó metadata). Lectura concreta:

- `desktop/src/components/lens/retrieval/RetrievalLens.tsx`:54-164 carga
  `listPage({limit:200})` para picker vectorial, ejecuta
  `search({query,embedding,top_k,explain:true,search_profile})`, y aplica filtros
  visuales posteriores; no re-rankea en cliente. Proponer A explicando
  visual/postfilter frente a filtro de servidor, sin negar filtros del motor.
- `desktop/src/vanta.ts`:779-846 usa `transport.call` para escenas, persona,
  skills y generación; hay memoria y diagnóstico, no un Studio por construir.
- B/C/D/G se insertan como información de operación, revisión, fuente y
  política en inspector/conexión existentes. Capacidad unsupported debe
  mostrarse, no llamar a otra conexión en secreto. UI evita almacenar raw
  prompts/secretos para «demostrar» cobertura.

No se ejecutó UI, backend, empaquetado, red ni navegación nativa. Estos seams
no certifican igualdad de Tauri/HTTP/WASM o disponibilidad visual de un contrato
que todavía es propuesta.

## 12. Fuentes y cobertura

### 12.1. Inventario fijado de código

Los enlaces fijan **`ae229ca894c0b80ad84664afcfdde523b5e87057`**.
Los rangos son superficies de lectura, no afirmación de auditoría exhaustiva.

[package]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/pyproject.toml
[license]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/LICENSE
[settings]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/app/settings.py#L121-L185
[compose]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/docker-compose.yml
[models]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/models/__init__.py#L544-L864
[content]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api_content.py#L52-L627
[api]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api.py#L46-L105
[configure-content]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/helpers.py#L3009-L3181
[markdown]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/processor/content/markdown/markdown_to_entries.py#L21-L124
[ingest]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/processor/content/text_to_entries.py#L142-L263
[filters]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/adapters/__init__.py#L2090-L2178
[search]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/helpers.py#L1484-L1574
[text-search]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/search_type/text_search.py#L99-L164
[chat]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api_chat.py#L676-L1486
[auth]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/configure.py#L98-L228
[memory-extract]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/helpers.py#L996-L1067
[memory-caller]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/processor/conversation/utils.py#L541-L620
[memories]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/adapters/__init__.py#L2292-L2378
[memory-policy]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/adapters/__init__.py#L1623-L1695
[memory-api]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api_memories.py#L18-L114
[obs-sync]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/obsidian/src/utils.ts#L104-L248
[obs-api]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/obsidian/src/api.ts
[desktop]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/desktop/main.js#L159-L610
[eval]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L34-L707

Fuentes UI adicionales fijadas, complementarias a los rangos de §11:
[web search](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/web/app/search/page.tsx#L656-L694),
[web settings](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/web/app/settings/page.tsx#L660-L730),
[web chat](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/web/app/chat/page.tsx#L492-L536),
[preload](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/desktop/preload.js#L12-L77),
[renderer](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/desktop/renderer.js#L33-L83),
[Emacs](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/emacs/khoj.el#L432-L455),
[Android TWA](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/android/twa-manifest.json),
[memory settings tests](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/test_memory_settings.py#L1-L80).
La cadena de persistencia→extracción se leyó también en
[save_to_conversation_log][memory-caller], no se infiere desde una función sin caller.

### 12.2. Cobertura por eje y límites pendientes

| Eje solicitado | Leído/documentado | No leído/no demostrado |
|---|---|---|
| Recomendaciones/arquitectura | §1-2, packages, ORM/pgvector, API y compose | Todos deployments Cloud y configs reales |
| Extracción/Rust | A–G con contratos hipotéticos, seams, alternativas, FMEA/criterios | Ninguna implementación o aceptación dinámica |
| Incidencias | #1437/#1363/#759 completos, estado/fecha y revisión cuando consta | PR #1440/diff y causa histórica exacta de #759 |
| Prerrequisitos | Worker/progress/scheduler/checkpoint Vanta, política/recall §5 | Todos transports y caches de hosts |
| Rechazos | §6 y no-goals de cada ficha | Auditoría legal total de modelos/dependencias |
| Advertencias | §7 distingue defaults, tests y reportes | Reproducción adversarial |
| Benchmarks | eval.py loaders/answerer/jueces/denominador/efectos §8 | Datasets completos, resultado/coste real, servidor efectivo |
| Matriz | §9 con snapshot y fuentes por capacidad | Paridad total de productos |
| Negativos Vanta | §10 y preguntas discriminantes | Ningún bug propio confirmado |
| Escritorio/UI | §11 código Next/Electron/plugin/Emacs y manifest TWA, Studio | Runtime/UI completa/IPC security review/Android nativo |

**Acceso público:** `websearch.fetchWebContent` entregó listado GitHub
`issues?q=is%3Aissue+index+sync` y las tres páginas concretas; no se atribuye
estado desde snippets de búsqueda. Router `coordinated-web-search` cargado
antes de red; documentación skill y contratos locales leídos. Intento inicial
`database/models.py` no existía: se usó el paquete real
`database/models/__init__.py`, sin inferir ausencia de modelos.

**Relaciones/continuidad:** B produce revisiones para C/F/G; E limita B;
A normaliza filtro de F; D resuelve política antes de cache e inyección.
Persistencia de receipts y manifests no exige retener contenido sensible.
Los criterios normales/adversos A–G son **validación manual futura**, no
tests/harness creados ni ejecutados.

**Cierre durable de autoría:** preservadas las ideas válidas de filtros,
conectores y referencias; completados flujos reales, memoria automática,
tenancy, clientes y benchmark antes omitido. Se corrigieron simplificaciones,
sin convertir issues upstream en diagnóstico Vanta. No apps/tests/builds/
benchmarks/servicios/install, fetch/pull ni git write. **No revisión independiente
ni aceptación**; decisiones y validación posteriores corresponden a otra fase.
Al superar 300 líneas, futura fase autorizada podría separar specs de sync/
parser (B/E), consulta/política/evidencia (A/C/D/F) y adapters Studio (G).
No se crean otros documentos ahora; este informe conserva autonomía.
