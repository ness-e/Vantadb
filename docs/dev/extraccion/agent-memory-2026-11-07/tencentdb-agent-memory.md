---
title: "TencentDB Agent Memory: captura incremental y gobierno de activos"
kind: research
status: proposed
description: "Análisis estático de captura, checkpoints, gobierno y fronteras del hub TencentDB para ideas independientes en Rust"
tags: [research, memory, engineering]
---

# TencentDB Agent Memory: captura incremental y gobierno de activos

## Índice

- [Identidad y dictamen](#identidad-y-dictamen)
- [1. Arquitectura y flujos](#1-arquitectura-tecnologías-flujo-y-recomendaciones)
- [2. Unidades candidatas Rust A–G](#2-unidades-mínimas-para-reimplementación-independiente)
- [3. Incidencias y FMEA](#3-defectos-riesgos-e-issues-con-evidencia-diferenciada)
- [4. Prerrequisitos](#4-prerrequisitos-vanta)
- [5. Rechazos](#5-qué-no-trasladar)
- [6. Método y licencia](#6-método-licencia-y-cobertura)
- [7. Runners](#7-runners-upstream-datos-métricas-y-efectos)
- [8. Matriz](#8-matriz-frente-a-vantadb)
- [9. Negativos condicionados](#9-deuda-sugerida-no-defectos-actuales-de-vanta)
- [10. Interfaz real y Studio](#10-uiescritorio-y-studio)
- [11. Cobertura y cierre](#11-cobertura-relaciones-y-cierre-de-autoría)

## Identidad y dictamen

- Repositorio `TencentCloud/TencentDB-Agent-Memory`, clon
  `C:\Users\Eros\TencentDB-Agent-Memory`.
- SHA `2002196f2d7c22ea267cfcaf67c5a6fa861df681`; rama **`feat/server_team`**,
  no confundirla con una release estable.
- Snapshot congelado: **30 589 estrellas**, licencia detectada **NOASSERTION**
  en [candidatos](github-candidates.json). No se alteró esa captura.
- `LICENSE:1-27` local contiene concesión MIT y copyright Tencent 2026 [T0].
  La diferencia es entre detector y texto, no permiso para inventar otro snapshot.
- [Manifest](clone-manifest.json) y [METHOD](METHOD.md) fijan el alcance offline.
- Vanta: **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**, fijado en
  [VANTA-REFERENCE](VANTA-REFERENCE.md). `c561158`/`167435a9e` eran antecedentes,
  no la base del cierre. Studio separado: `0ad38c542`, no runtime certificado.
- Captura congelada 2026-11-07T06:56:04.6569066Z; no se recalculan metadata
  ni estrellas. Lectura estática, sin fetch/pull/reclone ni ejecutar proyectos.
- Etiquetas: **Código**, **reporte público**, **inferencia**, **propuesta** y
  **pendiente manual** no son intercambiables. Ningún tipo Rust nuevo descrito
  en este dossier se presenta como implementado.

**Dictamen:** no migrar el hub entero. Interesan la frontera captura/cursor y
el modelado explícito de compartir/importar activos. Vanta ya tiene las capas
de memoria, ingestors, scheduler, checkpoints, dedup, provenance y Studio.
El incremento útil es contractual, no reconstruir el pipeline en Rust.

## 1. Arquitectura, tecnologías, flujo y recomendaciones

**Documentación:** README presenta MemoryCore, hub/panel y proxy para compartir
memoria entre agentes; distingue activos Chat Memory, Skill, Wiki y Code-Graph.
La muestra de código acredita piezas, no todos los claims del producto.

**Código/manifiestos:** TypeScript ESM. MemoryCore declara pipeline L0→L1→L2→L3,
build con tsdown y pruebas Vitest [T1]. MemoryKnowledge es servicio independiente
con Hono, MCP, Drizzle/SQLite, AI SDK y dependencia `@colbymchenry/codegraph`
`^1.2.0` [T2]. No asumir que la dependencia resuelta corresponde al SHA del
clon CodeGraph estudiado en otra ficha.

Flujo de captura observado [T3][T4]:

1. `performAutoCapture` crea `CheckpointManager` y llama `captureAtomically`.
2. El callback recibe cursor de sesión, registra conversaciones y devuelve
   máximo timestamp/cantidad; sin mensajes devuelve `null`.
3. `captureAtomically` ejecuta callback dentro de `mutate`; solo con resultado
   avanza cursor y contadores. Puede usar un suelo temporal de arranque.
4. Indexación vectorial es una etapa posterior: hay capacidad de embedding
   diferido o camino síncrono, no una única transacción con L0.
5. Un registro opcional de promesas permite al propietario esperar trabajo
   pendiente. La notificación posterior al scheduler entrega array vacío:
   el runner debe leer storage, no depender del buffer de esta llamada.

La ampliación inspecciona `mutate`/locks [T11] y L0 [T12], cerrando esa laguna
inicial. El nombre `captureAtomically` **no** demuestra atomicidad crash-safe
entre JSONL, checkpoint, índices y extracción LLM; véase 1.2 y unidad C.

**Gobierno:** `ChatMemoryAgentRel` guarda compartir con equipo e IDs importados;
valida pertenencia al equipo, no autoimportar, no duplicados y máximo dos [T5].
El comentario reconoce almacenamiento provisional en `metadata_json`,
no columna dedicada. Lectura inválida cae al default de compartir `true`.
Es contrato específico de chat, no abstracción universal de todas las assets.
Además, ese helper provisional **no es evidencia de la autoridad efectiva de
las rutas**: el flujo BFF actual usa assets/bindings y un guard compartido
[T16][T17]. El default permisivo no se eleva a fuga de datos demostrada.

**Code-Graph:** el bridge carga paquete CodeGraph y su `ToolHandler`, resuelve
un módulo interno por plataforma y abre/indexa el proyecto [T6].
Es integración, no una implementación independiente del motor AST de Tencent.
La portabilidad del bridge depende también de empaquetado y exports upstream.

**Recomendación:** reutilizar scheduler y registros Vanta; explicitar qué
significa «capturado», «indexado» y «derivado disponible». Compartir contenido
debe ser política autorizada del backend, no inferencia de un toggle UI.

### 1.1 Mapa de modelos, almacenamiento y API

| Componente | Código/stack leído | Identidad, persistencia y frontera |
|---|---|---|
| MemoryCore | TS ESM, pipeline y puertos [T1][T3] | Conversación/sesión, L1, escenas/persona; plugin o servicio, no un único backend obligatorio |
| Captura L0 | `L0MessageRecord` [T12] | ID, sessionKey/sessionId, userId/agentId, timestamp y recordedAt; JSONL diario o `StorageAdapter.appendFile` |
| Checkpoint | runner_states/pipeline_states [T11] | Cursor por sesión dentro de un objeto compartido; JSON por archivo o storage inyectado |
| Store L1 | `IMemoryStore` consumido en recall [T13] | keyword/embedding/hybrid según config; calidad/consistencia de cada driver no auditadas |
| MemoryKnowledge | Hono/MCP, Drizzle/SQLite [T2][T15] | `WikiRow` y `CodeGraphRow` con service_id, team_id, owner, visibility, version, status, internal_status |
| Wiki | `WikiService` [T14] | Metadata separada de raw/sources, pages e index.db; path por service/team/wiki; worker LLM separado |
| Planificación KB | `BuildQueue`/`SerialQueue` [T18] | FIFO por asset en RAM, `onIdle`; no journal durable de jobs |
| MemoryPanel | Hono/JOSE/Zod, web React [T9][T16][T19] | BFF a kernel meta/datos; sesión suministra service-id/user-key, no browser directo a SQLite |
| Offload | hooks, state-reporter, reclaimer [T20][T21] | ToolPair, refs, JSONL, MMD y estado; contexto operacional, no checkpoint durable de herramienta |
| Code-Graph | bridge npm [T6] | Abre/indexa proyecto; AST y licencia/exports pertenecen también a dependencia externa |

`WikiStatus` añade draft a pending/processing/ready/failed; CodeGraph no usa
draft [T15]. `version` identifica generación de build, no es prueba de CAS en
SQLite. `internal_status` explica fase; no debe sobrevivir como fase activa a
un estado terminal. Tiempo de evento, cursor de captura y fecha de commit
resuelven problemas distintos. Los `serviceId` como primer parámetro del
contrato store ayudan a no omitir tenant; no prueban autorización del actor.

**API leída:** `recordConversation`/`performAutoCapture`/`performAutoRecall`;
servicio wiki create/ingest/sync/get/list/delete [T14]; BFF POST
`/api/v1/chat-memory/{agent-fixed,team-assets,layer,allocate,patch-scope}`
[T16][T19]. El README describe otros activos; no se da por auditada toda
la API `/v3`, OpenClaw ni el proxy. Los modelos de proveedor son configurables;
el bridge y un nombre en manifest no fijan modelo resuelto ni coste.

### 1.2 Escritura: qué confirma realmente el cursor

`recordConversation:274-313` [T12] serializa líneas y espera append. Si append
falla, **atrapa el error y devuelve los mensajes filtrados igualmente**.
El callback de `performAutoCapture:122-140` [T3] calcula maxTimestamp de ese
array; `captureAtomically` [T4] puede por tanto avanzar cursor aun si falló
esa escritura L0. Esto sí es un mecanismo estático acotado de confirmación
ambigua, más preciso que “no se inspeccionó storage”. No prueba pérdida total:
el store vectorial posterior puede conservar mensajes; hace falta discriminar
qué camino consume L1 y qué retry reconstruye L0.

`mutate:364-375` [T11] relee dentro del lock de archivo y distribuido opcional,
ejecuta callback y escribe checkpoint. Filesystem hace tmp+rename; storage
hace overwrite directo, no transacción entre recursos. En este caller
`new CheckpointManager(pluginDataDir, logger, storage)` [T3:96] no pasa
lockOptions: no afirmar que toda captura adquiere Redis por existir ese puerto.
La promesa de embedding diferido tiene ownership del registro opcional
`bgTaskRegistry:289-293`; la función puede retornar sin que el vector esté listo.

**Delta Vanta revalidado:** `L0Recorder::record_turn:274-320` del pin [V1]
ya discrimina mensajes al mismo milisegundo por key+contenido con `tie_break`.
No recomendar añadir ese mecanismo como si faltase. Sigue siendo relevante
auditar la composición entre recibo de L0, cursor y derivados, no copiar el
filtro timestamp-only ni IDs reloj/azar upstream.

### 1.3 Consulta: vacío, fallo y timeout no son equivalentes

`performAutoRecall:92-134` [T13] corre core contra timer, default
`cfg.recall.timeoutMs ?? 5000`. Timeout devuelve `RecallResult.error` con
contexto vacío y `partial=false`; `Promise.race` **no cancela** la operación
perdedora. No afirmar que ese deadline detiene una llamada pagada, un storage
read o un reporte posterior.

Core resuelve perfiles L2/L3 por scope team/agent, sin fallback explícito al
root no scoped (`163-175`). Para texto no vacío usa `searchMemories`,
estrategia default hybrid y `applyRecallBudget` (`177-203`); texto vacío
puede todavía inyectar persona/navegación. Persona y escenas se leen en
try/catch separados; esos fallos pueden tratarse como ausencia (`206-242`).
Wrapper `performAutoRecallInner:326-370` transforma excepción en resultado
estructurado; no acredita partial por cada error absorbido internamente.

Salida distingue prependContext/appendSystemContext, recuerdos/persona y
estrategia; no es recibo de entrega al modelo. Deben separarse tiempo total,
búsqueda, embeddings, lectura de perfiles y contexto finalmente usado.
Esta lectura no auditó `searchMemories` íntegro ni todos los drivers.

### 1.4 Update/delete y derivados Wiki: generación antes que nombre de job

`WikiService.create:317-330` [T14] crea metadata y directorio, no autoingest.
`ingest:355-390` devuelve not_found/busy/ok/queued; replace marca solo la
generación anterior y aumenta version antes de encolar. `BuildQueue` [T18]
serializa por asset, no globalmente; su estado se pierde al reiniciar.
`runBuild:1350-1425` chequea cancelación al entrar y después del worker;
publica ready/failed con `internal_status=null`, audita y ejecuta hook posterior.
Metadata ready no es el mismo commit que todos los archivos del worker.

`delete:423-462` puede cancelar en-flight, invalidar metadata y limpiar
conexión/index.db/directorio sin esperar worker. Cada cleanup atrapa errores:
retorno true no prueba purga física completa. `finishCancelled:494-530`
distingue replace (preservar raw/ready pages) y delete; no restablece draft
si una generación nueva ya posee el estado. `isDeleted` mira cancelación RAM
o ausencia de fila, no un fencing durable multirréplica.

`SqliteKnowledgeStore.markInterruptedAsFailed:547-559` [T15] convierte
pending/processing en failed al reinicio pero no limpia internal_status.
No se extrapola a “no persiste metadata nunca”: la ruta normal sí publica
estado y la incidencia pública §3 mezcla varios síntomas/versiones.

### 1.5 Priorización específica, no clonación de la plataforma

| Prioridad propuesta | Unidad/delta | Dependencia decisiva |
|---|---|---|
| P0 contractual | A recibo durable/cursor; C exclusión y resultado ambiguo | Comprobar callers/serialización Vanta, no instalar Redis |
| P0 de frontera | B acceso/préstamo/revocación, solo si colaboración es requisito | Actor confiable y guard backend; no JSON permisivo |
| P1 | D generación/cancelación/publicación de derivados | Scheduler/checkpoint y receipts existentes |
| P1 | E señal de capacidad efectiva de hook y G fallos por fase de recall | Instrumentar consumidor real, no otro assembler |
| P2 | F retención con cierre de referencias y fallos explícitos | Inventario de originales/derivados y política aprobada |

No se traslada BM25/RRF, personas/escenas, scheduler ni AST como “nuevos
mecanismos”: Vanta ya los tiene o son dependencias externas.

## 2. Unidades mínimas para reimplementación independiente

### Unidad A — contrato de captura con cursor y visibilidad por etapa

- **Problema:** confirmar captura puede confundirse con que toda indexación
  y extracción haya terminado, especialmente con embeddings diferidos.
- **Unidad fuente:** callback de `captureAtomically:741-768` [T4];
  resultados y separación de etapas de `performAutoCapture` [T3].
- **Seam Vanta:** `L0Recorder::record_turn:274-372` [V1] y sus
  `read_cursor/write_cursor` [V-cursor]; callers de captura [V-capture-caller].
  Checkpoint de tarea y merge wiki no publican ese cursor; scheduler solo
  coordina trabajo posterior, no confirma la captura por sí mismo.
- **DTO HIPOTÉTICO Rust:** `CaptureReceipt { source, session, cursor,
  durable_revision, lexical_state, embedding_state, derivation_state }`.
  No es un tipo acreditado del SDK ni una promesa de durabilidad actual.
- **Interfaz HIPOTÉTICA:** `capture_batch(ScopedBatch) -> CaptureReceipt`;
  un puerto de publicación posterior consume la revisión confirmada.
- **Acciones:** mapear confirmación actual; conservar el tie_break ya existente
  para timestamps iguales; distinguir publicación de mensajes/cursor y progreso L1;
  separar disponibilidad de evidencia e índice; registrar fallos recuperables;
  permitir al scheduler reintentar sobre IDs, no sobre buffers efímeros.
- **Dependencias/relaciones:** fuente/provenance y dedup ya presentes;
  cierre ordenado y cancelación para tareas diferidas; no crear segundo
  scheduler ni otro archivo de checkpoints por similitud upstream.
- **FMEA:** cursor avanza antes de write durable; mismo timestamp omite
  mensajes; retry duplica; cold-start floor pierde backlog deseado;
  cierre destruye índice mientras embedding sigue activo.
- **Aceptación funcional propuesta:** confirmación distingue persistencia de
  derivados; retry converge a las mismas evidencias; fallo de embeddings
  conserva recuperación textual; interrupción no hace aparecer un lote
  incompleto como íntegramente procesado; shutdown identifica trabajo pendiente.
- **No objetivo:** copiar IDs aleatorios o el suelo de arranque como política
  obligatoria para todos los ingestors Vanta.

### Unidad B — relación de préstamo con autorización y revisión

- **Problema:** permitir a un agente consultar activos de otro sin confundir
  pertenencia al equipo, visibilidad y equipamiento/importación.
- **Unidad fuente:** DTO y `validateImportedAgents`, lectura/escritura
  normalizada de relaciones en [T5].
- **Seam Vanta:** frontera de scopes de memoria/escenas/skills y adaptadores
  server/MCP; UI en Studio `develop-desktop`, no en el motor de índices.
- **DTO HIPOTÉTICO:** `AssetGrant { asset_id, owner_scope, reader_scope,
  asset_kind, revision, decision_source }`. `BorrowRequest` expresa actor,
  objetivo y revisión esperada. No se propone el límite de dos como universal.
- **Interfaz HIPOTÉTICA:** `authorize_borrow(request) -> Result<Grant,
  PolicyError>`; backend decide, UI solo muestra la decisión.
- **Acciones:** inventariar autorización actual; distinguir ver, usar y editar;
  validar pertenencia con identidad confiable; persistir cambios con revisión;
  aplicar el grant antes de búsqueda/expansión; revocar caches y derivados
  al retirar acceso; dejar auditoría explicable.
- **Dependencias:** catálogo de activos, identidad de actor, provenance,
  revisión y revocación. La Unidad A puede informar disponibilidad, no permisos.
- **FMEA:** default compartido abre datos; metadata corrupta habilita permisos;
  equipo cambia durante préstamo; UI oculta activo pero retrieval lo entrega;
  snapshot/cache conserva acceso revocado.
- **Aceptación propuesta:** actor de otro equipo no obtiene contenido;
  configuración inválida no amplía acceso; autoimportación y duplicados se
  rechazan; revocación afecta recall y preview; revisiones en conflicto no
  pisan concesiones; error explica política sin divulgar activos ajenos.
- **No objetivo:** trasladar el modelo provisional en JSON ni una plataforma
  de identidad/administración empresarial completa.

## 3. Defectos, riesgos e issues con evidencia diferenciada

- **Código:** `readChatMemoryRel` devuelve default con sharing `true` ante
  ausencia/JSON inválido ([T5], líneas 25-28,61-74). **Riesgo condicionado:** sería
  fail-open si esa relación fuese autoridad de acceso. No se auditó el
  enforcement de rutas ni se afirma una fuga reproducida.
- **Código:** captura atrapa fallo de L0 y continúa a etapas posteriores;
  scheduler puede ser notificado incluso sin mensajes ([T3], líneas 143-145,301-326).
  No equivale a pérdida probada: faltan contratos de caller, storage y retry.
- **Código:** ID de L0 vectorial incorpora reloj/aleatoriedad ([T3], líneas 42-44).
  No acredita idempotencia de reenvío; tampoco invalida la deduplicación
  que pueda existir en otra capa.
- **Código:** `resolveToolsPath` depende de un path interno por plataforma
  ([T6], líneas 25-38). **Hipótesis:** futuras versiones empaquetadas pueden romperlo
  aunque el API principal siga siendo compatible.
- **Defecto estático acotado:** el runner token importa
  `../src/offload/fast-token-estimate.ts` desde `src/offload/`, que resuelve
  a `src/src/offload/...`, ruta no presente ([T8], línea 4). No se ejecutó.
- **Issues:** cuerpos públicos leídos al cierre mediante fallback HTML después
  de rate-limit REST; registro fechado en §11. No se atribuye merge por enlaces.

## 4. Prerrequisitos Vanta

Revalidar contratos existentes antes de abrir implementación: persistencia L0,
semántica de checkpoints y publicación de derivados; identidad de equipo/actor
si realmente se necesita colaboración. Usar políticas explícitas de retención,
borrado y revocación. Mantener funciones puras de dominio separadas de HTTP,
proveedores LLM y drivers. No suponer que un hub multiagente exige varios servicios.
C depende del cursor `l0_cursor/<sesión>` y sus callers, no de `step/partial`
de `TaskCheckpointManager`; revisar exclusión en ese recurso sin nueva queue.

## 5. Qué no trasladar

No portar MemoryCore/Knowledge/Panel/Proxy juntos ni acoplar Vanta a OpenClaw,
Redis, COS, Tencent VectorDB o MongoDB. No duplicar BM25/HNSW/RRF.
No copiar `metadata_json` provisional como fuente de permisos.
No convertir «compartir true» en default del motor embebido.
No importar rutas internas de CodeGraph ni el protocolo de despliegue como
requisito para una unidad de memoria.

## 6. Método, licencia y cobertura

NOASSERTION congelado se conserva; MIT local es evidencia adicional [T0].
Revisar notices/archivos/dependencias antes de cualquier derivación:
Rust no elimina obligaciones ni hace independiente una traducción literal.
Clon de una rama superficial, sin historial completo, tags ni servicios.
No auditoría exhaustiva de auth, almacenamientos, wiki, proxy, borrado o cuotas.

## 7. Runners upstream, datos, métricas y efectos

Se leyó **`scripts/bench-checkpoint-lock.ts:1-212`** [T7]:
usa `CheckpointManager` real; entradas sintéticas `bench-sess-*` y preload,
no LoCoMo ni LongMemEval. Cronometra `markL1ExtractionComplete`, calcula QPS y
p50/p95/p99; escenarios de concurrencia y tamaño están definidos upstream.
Depende de Node/TypeScript y, con `REDIS_HOST`, `ioredis`/Redis real.
El cuerpo crea directorio temporal y pasa storage `undefined`: el comentario
sobre COS no acredita que este runner mida COS.

Efectos: crea/elimina directorio `cp-bench-*`, escribe checkpoints, crea locks
Redis con TTL y libera por owner mediante Lua; `finally` elimina directorio y
cierra Redis. No ejecutado. Es contención de checkpoint en una ejecución,
no prueba de escritores en procesos distintos, fencing ni calidad de memoria.
La conclusión de capacidad L1 usa supuestos comentados, no medición end-to-end.

También se leyó **`src/offload/benchmark-token-estimate.ts:1-89`** [T8]:
compara estimación con `js-tiktoken` `cl100k_base`, toma archivos multilingües
si existen y agrega escenarios JSON/código sintéticos. Calcula error relativo,
tiempos y speedup; el total es diferencia de sumas, no error absoluto medio
por documento. No acredita presupuesto para cualquier tokenizer/modelo.
Efectos visibles: lectura de corpus y salida a consola; hay un problema de
ruta de importación descrito arriba.

No se diseñaron ni modificaron prompts, métricas, umbrales, sampling o
configuraciones. Otros runners e2e/Mongo se localizaron, **no se analizaron**.
Claims de memoria del README no quedan certificados por estos microbenchmarks.
Comparación futura solo según [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Capacidad | Estado usado | Contraste útil |
|---|---|---|
| Captura/sesiones/provenance/dedup | Existe | recibo por etapa |
| Scheduler/checkpoints/ingestors | Existe | cursor/publicación y cierre |
| BM25/HNSW/RRF | Existe | no otro índice por portabilidad |
| Escenas/personas/skills | Existe | no importar toda la ontología |
| Pipeline con estados visibles entre capas | Parcial para este contraste | existen piezas; composición no certificada aquí |
| Préstamo gobernado de assets entre equipos | No acreditado | requisito primero, no bug supuesto |
| Tencent COS/Redis como requisito | No aplicable | elección de despliegue upstream |
| Calidad de captura/recall del producto | No acreditado | runners leídos no la miden |

## 9. Deuda sugerida, no defectos actuales de Vanta

Preguntar si una confirmación actual distingue raw/indexado/derivado; si hay
eventos estables frente a timestamps iguales; si shutdown drena trabajo
de ingestión; y si permisos/revocación se aplican también al contexto derivado.
Son verificaciones futuras, no afirmaciones de fallos de otro HEAD.
Evitar nuevas filas si ya hay tareas que cubren estas garantías.

## 10. UI/escritorio y Studio

**Código real:** `MemoryPanel/web/src/App.tsx:1-70` usa React, RouterProvider,
store de auth, LoginGate e i18n [T9]. `ChatMemoryPage/index.tsx:1-10` monta
`ChatMemoryPanel` dentro de `ResourcePage` [T10]; no solo capturas de marketing.
No se auditó su aplicación de permisos ni todas sus acciones.
No se localizó un empaquetado desktop propio en la muestra; el panel es web.

Studio Tauri/React ya tiene memoria, actividad, escenas, skills y diagnóstico.
Aplicar allí estados por etapa y explicación de grants/revocación cuando
existan contratos backend, no crear otro panel ni copiar su stack visual.

### Ledger de archivos/rangos efectivamente leídos

| Ref. | Ruta/rango | Superficie |
|---|---|---|
| T0 | `LICENSE:1-27` | MIT local frente a detector |
| T1 | `MemoryCore/package.json:1-70` | pipeline declarado y scripts |
| T2 | `MemoryKnowledge/package.json:1-80` | stack y dependencia CodeGraph |
| T3 | `MemoryCore/src/core/hooks/auto-capture.ts:1-180,285-347` | captura/capacidades/scheduler |
| T4 | `MemoryCore/src/utils/checkpoint.ts:735-768` | callback y avance del cursor |
| T5 | `MemoryPanel/src/panel/domain/chat-memory-governance.ts:1-107` | modelo/normalización |
| T6 | `MemoryKnowledge/src/engines/code/bridge.ts:1-140` | carga y apertura del índice |
| T7 | `MemoryCore/scripts/bench-checkpoint-lock.ts:1-212` | runner completo |
| T8 | `MemoryCore/src/offload/benchmark-token-estimate.ts:1-89` | estimación de tokens |
| T9 | `MemoryPanel/web/src/App.tsx:1-70` | shell UI |
| T10 | `MemoryPanel/web/src/pages/ChatMemoryPage/index.tsx:1-10` | composición de página |

README raíz leído `1-90`; listado de rutas no equivale a lectura de cuerpos.
Pendientes: enforcement auth/equipos completo, storage/cleanup exhaustivo,
calidad del pipeline y ejecución manual de runners. `mutate` sí leído en §1.2.
Sin scripts de proyectos, builds, tests, apps, instalaciones, servicios,
benchmarks, validadores documentales, commits ni push.

[T0]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/LICENSE#L1-L27
[T1]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/package.json#L1-L70
[T2]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/package.json#L1-L80
[T3]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/core/hooks/auto-capture.ts#L1-L347
[T4]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/utils/checkpoint.ts#L735-L768
[T5]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/src/panel/domain/chat-memory-governance.ts#L1-L107
[T6]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/src/engines/code/bridge.ts#L1-L140
[T7]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/scripts/bench-checkpoint-lock.ts#L1-L212
[T8]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/offload/benchmark-token-estimate.ts#L1-L89
[T9]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/web/src/App.tsx#L1-L70
[T10]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/web/src/pages/ChatMemoryPage/index.tsx#L1-L10

## 11. Cobertura, relaciones y cierre de autoría

### Completar contratos A/B y desarrollar C–H

Todas las firmas siguientes son **PROPUESTAS**, no APIs implementadas.
Vanta se contrasta solo en `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
[WriteReceipt](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L657-L735)
ya existe: binding/integridad SHA-256, **no firma**. A correlaciona su resultado
con captura/derivados; `MutationReceipt` C13 es solo propuesta, no tipo actual.

**A, complemento de los 11 campos.** Delta no es crear recibos desde cero:
es evitar que cursor confirme un append fallido [T12]. Alternativas: extender
L0 y checkpoints existentes frente a ledger paralelo (rechazado). Conservar
`capture_batch`, ampliando retorno a `Result<CaptureReceipt,CaptureError>`;
errores `InvalidEvent,Denied,AppendFailed,CheckpointFailed,Cancelled`;
source/session/identidad/revisión son entradas, y durable_revision solo sale
tras confirmar primario. `L0Recorder::record_turn:274-376` [V1] es seam real,
no usar su tie_break ya existente como supuesta novedad. Migración metadata
aditiva, históricos Unknown, sin reindexado raw intacto; rollback detiene nueva
captura sin borrar pendientes. FMEA append falla→callback devuelve mensajes→
cursor avanza; detectar receipt/cursor divergentes, mitigar no adelantar cursor
y reconciliar por ID. Cancelación antes del commit no confirma; después expone
resultado consultable. Aceptación normal/adversa: replay, timestamp empatado,
append fallido, caída entre primario/cursor y embedding fallido; contadores de
fase/error sin texto. Mínimo adapter; abierto commit del storage concreto.

**B, complemento de los 11 campos.** Delta frente a `InjectionPolicy`/
`RecallScope` [V2] es préstamo entre activos, no ACL desde cero. Alternativas:
extender autorización del transporte; rechazar JSON permisivo como autoridad.
`authorize_borrow` recibe actor confiable, asset/scope y expected_revision;
devuelve grant o `Denied,UnknownAsset,StalePolicy,Unavailable`, nunca distingue
existencia de activo ajeno. Seam `perform_auto_recall_governed` [V2], antes de
expandir fuentes; Studio solo presenta decisión. Migración grants aditivos,
legado no aumenta acceso; rollback revoca grants nuevos e invalida cachés,
no recupera contenido ya entregado. FMEA cambio de equipo→grant cacheado→
lectura indebida; detectar policy_revision y reautorizar en entrega. Cancelación
no concede permisos; métricas denied/stale sin IDs ajenos. Aceptación: editar
y usar son permisos distintos, revocación durante recall no entrega contenido;
abierto requisito colaborativo y autoridad real del backend.

### C — exclusión de RMW y publicación de cursor

1. Problema/riesgo: dos capturas de la misma sesión pueden competir al publicar
   cursor; no se declara carrera Vanta reproducida ni un cursor global.
2. Fuente mínima [T11] `mutate:364-375`: lock local por archivo, lock externo
   opcional, read→callback→write; tmp+rename no transacción multirrecurso.
3. Delta: el recurso es `l0_cursor/<sesión>`, payload `after_timestamp_ms`,
   leído/escrito por `L0Recorder`, no el checkpoint de tarea [V3].
4. Alternativa elegida: serializar writer/caller existente; no Redis obligatorio.
5. Contrato propuesto `advance_capture(expected: CursorRevision, next:
   ConfirmedCursor) -> Result<CursorRevision, CursorError>`; errores stale/
   storage/cancelled; next no retrocede ni refiere evidencia no confirmada.
6. Seam real: `L0Recorder::record_turn:274-372` [V1] llama
   `read_cursor:421-430` y `write_cursor:471-495` [V-cursor].
   Callers: `core/hooks/auto_capture.rs:131` y
   `services/conversation_hook.rs:61` [V-capture-caller].
   Persiste mensajes antes del cursor; un fallo intermedio puede dejar
   mensajes escritos sin nuevo cursor. `TaskCheckpointManager::advance`
   [V3:129-188] avanza `step/partial` de **otro recurso**; el helper global
   `utils/checkpoint.rs` tampoco es este cursor.
7. Pasos: delimitar exclusión por sesión en caller/recorder existente, confirmar A,
   revalidar el cursor leído y los mensajes confirmados antes de actualizar,
   reconciliar tras crash. Metadata revisionada; sin reindexado; rollback
   pausa callers nuevos, no restaura cursor antiguo sobre evidencia nueva.
8. Dependencias A; ownership de captura en caller/L0Recorder, no transferido
   al scheduler. TaskCheckpoint solo serviría progreso/reconciliación si se
   acredita vínculo explícito; no es guard de L0 ni requiere nueva queue.
9. FMEA dos capturas leen igual cursor→avance obsoleto/regresión posible;
   detectar conflicto por sesión, mitigar en publicación del **cursor L0**.
   Mensaje persistido y cursor fallido→replay→conservar tie_break y recibo
   parcial A; no “arreglar” `step/partial` esperando proteger otro recurso.
10. Aceptación futura normal/paralela, interrupción y lock perdido; cancelación
    libera guard y no anuncia avance; métricas wait/conflict sin payload.
11. Mínimo una frontera RMW; no WAL/scheduler nuevo; abierto primitiva CAS real.
La firma propuesta se conserva: `CursorRevision` no existe en el payload
actual y su política condicional no está acreditada por `write_cursor`.
Aceptación adicional: sesiones distintas no se bloquean entre sí; avanzar
checkpoint de tarea no cuenta como avance de captura; fallo de cursor no se
presenta como nada escrito ni como extracción L1 terminada.

### D — generación de derivado y materialización visible

1. Problema: build tardío resucita wiki borrada o pisa reemplazo más reciente.
2. Fuente [T14] `runBuild:1350-1425`, delete/finishCancelled; [T18] cola por
   asset en RAM. `ingestRunId` de progreso no equivale fencing durable.
3. Delta: Vanta ya tiene scenes/reflection/generation log y jobs [V4];
   interesa ligar publicación y diagnóstico a la revisión original.
4. Alternativas: extender job/checkpoint, no cola por asset adicional.
5. Contrato propuesto `publish_derived(source: SourceRevision, generation:
   u64, output: DerivedCandidate) -> Result<PublishOutcome,PublishError>`;
   Applied/Stale/Deleted/Cancelled y storage; versión vieja nunca publica.
6. Seam `reflect_session`/`ReflectionRun` [V4] y writer L1 [V5].
7. Pasos: capturar generación, inferir sin locks, revalidar, publicar, registrar
   estado; metadata aditiva, reindexar solo derivado cambiado. Rollback pausa
   publicación nueva y preserva última versión válida, no revierte purga.
8. A/C confirman origen/cursor; F invalida referencias tras retención.
9. FMEA cancelación solo RAM→restart pierde marker→output viejo; detectar
   revisión/tombstone en commit, no confiar en ausencia en cola local.
10. Aceptación: normal ready, replace/delete durante worker, crash tras archivos
    antes de metadata; cancelar no transforma fallo en ready. Métricas por fase.
11. Mínimo contrato de publicación; no nuevo motor wiki; abierto atomicidad
    metadata/archivos y compensación de outputs parciales.

### E — capacidad efectiva del hook, no «configurado = entregado»

1. Problema: hook no recibe mensajes mutables y compaction aparenta actuar.
2. Fuente [T20] `classifyPatchEffectiveness:65-80`: effective/missing_field/
   empty_messages/n-a según etapa; no acredita patch global del runtime.
3. Delta: Vanta ya tiene `IntegratedContext`/`CompactionReport` [V6];
   ampliar resultado del adapter, no assembler alternativo.
4. Alternativas: warning textual insuficiente; elegir outcome tipado.
5. Contrato propuesto `inspect_hook(stage: HookStage, observed: HookShape)
   -> HookCapabilityReport`; entradas sin contenido, salida estado+cantidad;
   malformed explícito, no afirmar entrega desde presencia de campo.
6. Seam `assemble_with_recall:252-329` [V6] y consumidor del host.
7. Pasos: detectar forma, ensamblar existente, confirmar aplicación del host,
   reportar degradación. Efímero/versionado, sin migración ni reindexado;
   rollback omite reporte nuevo, no modifica mensajes.
8. G da outcomes recall; A no sustituye recibo de entrega.
9. FMEA host ignora modificación→contexto ausente→respuesta sin evidencia;
   detectar ack/capacidad observada, mitigar fallback explícito autorizado.
10. Aceptación: forma válida/ausente/vacía y cancelación; no medir «ahorro»
    cuando no aplicado. Contadores forma/etapa sin mensajes.
11. Mínimo diagnóstico; no parche runtime; abierto ack concreto por host.

### F — TTL, retención y cierre de referencias

1. Problema: caducidad raw deja referencias o contexto derivado aún accesibles.
2. Fuente [T21] `reclaimOffloadData:62-130`: JSONL→refs→MMD→logs→registro,
   errores independientes; [T22] `MemoryCleaner.runOnce:69-155` limpia L0/L1
   por día local, luego DB con mínimos. Son políticas diferentes.
3. Delta: Vanta lifecycle/erasure existen [V7]; no confundir TTL con cuotas,
   budgets de recall ni borrado criptográfico.
4. Alternativas: extender sweep/erasure existente; rechazar TTL mágico global.
5. Contrato propuesto `plan_retention(scope: &RecallScope, now_ms: u64,
   policy: RetentionPolicy) -> Result<RetentionPlan,RetentionError>`;
   apply devuelve por superficie Deleted/Preserved/Pending/Failed;
   denied/stale/storage, jamás éxito físico por conteo incompleto.
6. Seam `erase_scope:191` y `emit_cascade_erasure_receipt:232` [V7], scheduler.
7. Pasos: inventario original/derivado/cache, preview, revalidar, borrar,
   invalidar, reconciliar fallos; metadata aditiva y reindexado de afectados.
   Desactivar detiene futuros sweeps, no restaura purgado.
8. D conoce generación; A conserva tombstone/replay; política y permisos mandan.
9. FMEA mtime o zona local mal interpretados→borrado prematuro; detectar
   cutoff/origen temporal y mitigar semántica explícita/revisión; fallo en refs
   tras raw borrado→huérfano, reportar pendiente en vez de éxito global.
10. Aceptación: activo preservado, vencido retirado, ref aún usada preservada,
    error parcial/restart/cancelación sin resurrección; métricas por superficie.
11. Mínimo retención de una clase aprobada; no garantías sobre backups ni
    importar mínimos/tres días upstream; abierto política UTC/local y legal.

### G — recall con outcomes y cancelación real del consumidor

1. Problema: timeout o lectura de persona fallida se confunden con vacío.
2. Fuente [T13] `performAutoRecall:92-134`, core 163–242, wrapper 326–370.
3. Delta: Vanta budgets, governed recall y hybrid ya existen [V2]; ampliar
   discriminación por fase, no nuevo ranker.
4. Alternativa: extender RecallResult, no reemplazar try/catch por fail-open.
5. Propuesto `recall_scoped(request: ScopedRecall, cancel: CancelSignal)
   -> Result<RecallPhaseReport,RecallError>`; Ready/Empty/TimedOut/Denied/
   Failed por fase; scope/revisión/budget preservados; timeout no prueba abort.
6. Seam `perform_auto_recall_governed`/`RecallConfig` [V2], assembly [V6].
7. Pasos: mapear errores, propagar cancelación cooperativa, ensamblar una vez,
   reautorizar delivery; DTO efímero, sin reindexado; rollback render compatible.
8. E observa aplicación; B autoriza fuentes; D no publica desde recall.
9. FMEA Promise.race pierde→operación continúa→retry amplifica coste;
   detectar in-flight abandonado, mitigar cancelación/admisión existente,
   no prometer devolución de coste de proveedor ya enviado.
10. Aceptación: normal, vacío, persona caída, query caída, timeout/revocación;
    Cancelled distinto de Empty; conteos/latencia/outcomes sin query.
11. Mínimo outcomes; no nuevos pesos ni benchmark; abierto cancelación por driver.

### H — dimensión del cliente y contrato de embeddings

1. Problema: modelo remoto rechaza `dimensions` o devuelve batch incompatible.
2. Fuente [T23] `_callApi:520-599` envía dimensions condicional, proxy QClaw,
   timer/retry, ordena por index y normaliza respuesta.
3. Delta: Vanta embeddings opcionales y validación vectorial existen [V5];
   interesa contrato de proveedor, no otro motor de embeddings.
4. Alternativas: adapter/configuración explícita; no autodetectar dimensión
   por truncar/pad ni convertir fallo a vector cero.
5. Propuesto `validate_embedding_batch(expected: EmbeddingBatchContract,
   response: Vec<EmbeddingItem>) -> Result<ValidatedBatch,EmbeddingError>`;
   modelo/revisión/dimensión/IDs de entrada, output misma correspondencia;
   errores dimension/missing/duplicate/nonfinite/denied/cancelled.
6. Seam `core_embedding_hook`/`EmbedFn` y `write_memory` [V5]; provider en borde.
7. Pasos: identificar modelo/config, validar orden/cardinalidad/dimensión,
   preservar fallback textual permitido, registrar degradación. Metadata
   aditiva; cambiar modelo requiere reembedding explícito separado, nunca
   mezclar vectores de espacios distintos. Rollback mantiene índice compatible.
8. G expone degradación; A confirma raw aunque vector falle; secretos del
   proxy/Remote-URL son configuración confiable, nunca texto del agente.
9. FMEA sort por index sin validar duplicados→vector de otro texto→recall falso;
   detectar unicidad/cardinalidad y mitigar rechazo antes de persistir.
10. Aceptación: normal, dimensiones incompatibles, duplicado/NaN/timeout/429;
    cancelación no publica vector parcial; métricas por error sin input/token.
11. Mínimo validación adapter; no modelo nuevo ni reindexación automática;
    abierto contrato de versiones y permisos del proveedor.

### Incidencias públicas efectivamente leídas

El 2026-11-07 REST search devolvió referencias; cuerpos individuales dieron
rate-limit 403. Fallback `websearch.fetchWebContent` leyó las secciones china/
inglesa visibles de estas páginas; extracción truncó texto final de requisitos,
no se presume lectura de comentarios/PRs ni merge.

| Fuente / estado y fecha | Reporte / revisión | Transferencia sin inventar fix |
|---|---|---|
| [#1937](https://github.com/TencentCloud/TencentDB-Agent-Memory/issues/1937), open, creada 2026-11-03 | `useCodeSources.ts`: inFlight retira ready pero no failed/missing; error card depende del mismo conjunto. Reporta rama actual sin SHA individual | Separar tarea pendiente de estado terminal visible en D/E. Enlaces #1942/#1960 son propuestas no leídas, **fix no acreditado** |
| [#1935](https://github.com/TencentCloud/TencentDB-Agent-Memory/issues/1935), open, creada 2026-11-03 | `DefaultAgentTemplateSection.tsx`: respuesta tardía equipo A pisa estado UI de B; sin secuencia/cancelación según reportante | B exige identidad/revisión en UI y backend; no afirmar fuga servidor ni bug Studio. #1941 enlazada no acredita merge |

No son reproducciones ni prueba de que todo el snapshot tenga esas carreras;
se conservan como mecanismos de diseño. Issue #1936 se localizó en búsqueda
pero su cuerpo no se leyó: no se atribuye mecanismo adicional.

### Benchmarks, materialización y matriz trazable

Se mantienen los runners reales de §7: checkpoint mide operaciones de
`markL1ExtractionComplete`, no mensajes del producto. `percentile:31-34` [T7]
selecciona índice sobre duraciones ordenadas; preload no es trabajo medido,
QPS usa operaciones terminadas sobre intervalo de esa corrida. No acredita
denominador de calidad ni éxito de agente; no comparar su p99 con recall.
Token estimate usa cl100k_base y escenarios disponibles/sintéticos, no un
tokenizer universal ni calidad de respuesta. Defaults efectivos por CLI,
fallos/reset y efectos permanecen upstream; no nuevos harness/prompts/rúbricas/
scoring/umbrales/configs. Otros e2e/Mongo no leídos siguen laguna, no ausencia.
Adapter futuro debe mapear `sessionKey/sessionId` a sesiones/provenance Vanta
y `markL1ExtractionComplete` a progreso de extracción existente solo tras
comprobar semántica [V3]; **no** a `write_cursor` L0 ni a C. No portar Redis/
COS ni ejecutar limpieza compartida. Comparación de retrieval solo mediante
[METHOD](METHOD.md)/[BENCHMARK-NOTES](BENCHMARK-NOTES.md), separando fases/costes.

La materialización observada no es un nuevo motor: [v2-router:2203-2252](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/gateway/v2-router.ts#L2203-L2252)
cuenta queued/running y materializa sets de sesiones por capa, incluye flush
en L1 e idle solo cuando ambos conteos son cero. **Idle no equivale a todos
los derivados válidos/publicados**; D relaciona estado de trabajo con generación.

| Capacidad upstream | Estado Vanta y fuente a SHA fijo | Delta / decisión |
|---|---|---|
| Captura/cursor | Existe `record_turn/read_cursor/write_cursor`, `l0_cursor/<sesión>` [V1][V-cursor] | A/C protegen captura, no `TaskCheckpointManager::advance`; no timestamp fix ya presente |
| Checkpoint de tarea/global | Existe `step/partial` [V3]; helper global es otro estado | No acredita exclusión o CAS de cursor L0; vínculo de reconciliación no demostrado |
| Recall híbrido/budget/scope | Existe [V2], core_search configurable no universal | G, no otro BM25/HNSW/RRF |
| Estados de jobs/materialización | Existe checkpoint [V3] y reflexión [V4], parcial vínculo por derivado | D/E, no scheduler nuevo |
| Préstamo de assets | No acreditada paridad específica; gobernanza recall existe [V2] | B solo ante requisito |
| TTL raw/offload | Parcial lifecycle/erasure [V7]; equivalencia de políticas no acreditada | F, no importar defaults |
| Dimensiones cliente | Existe hook opcional [V5]; contrato de cada proveedor no certificado | H adapter, fallback textual explícito |
| Cuotas de almacenamiento | Existe configuración según [referencia](VANTA-REFERENCE.md) | No confundir TTL ni budget de contexto |
| Tencent drivers/CodeGraph runtime | No aplicable como dependencia del core Rust | Rechazar paquete plataforma |

**UI/flujo real:** shell React [T9]→ChatMemoryPage [T10]→BFF chat-memory
[T16]→kernel assets/bindings. `allocate` consulta bindings y hace set completo;
`patch-scope:589-640` conserva bindings antiguos y comenta filtro de visibilidad
en lectura, no eliminación física de todas las relaciones. No se certifica
auth global a partir de ese comentario. Studio separado leído al SHA
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`:
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409,488-550`
selecciona sesión/tabs y consulta genlog/anchors;
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:52-89`
muestra audit paginado y errores;
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228`
implementa búsqueda con explain y contexto/audit. Cuerpos releídos con
`git show` al SHA completo, no bytes dirty de `desktop/...`.
Extender estas vistas con estados A/D y grants B, no otro panel/runtime.
FMEA/aceptación UI A/C: mostrar por separado mensajes confirmados, cursor L0
y progreso L1/tarea; no anunciar avance de captura desde `step/partial` ni
audit vacío desde error. JobChip/jobs no están acreditados en el pin.
Recibo [T-1/T-2](reviews/memory-additions.md#tencentdb-agent-memory):
correcciones propagadas, pendientes de reconfirmación independiente.
Fuentes Studio inmutables de esa lectura (2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

[V-cursor]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L421-L495
[V-capture-caller]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/conversation_hook.rs#L45-L78

**Licencias/cobertura:** MIT local [T0] cubre fuente propia leída Core/
Knowledge/Panel; detector NOASSERTION congelado se conserva. CodeGraph npm,
node-llama-cpp/pesos locales, servicios COS/VectorDB/Redis/Mongo y dependencias
frontend tienen obligaciones separadas no auditadas íntegramente. Reescribir
Rust no elimina derivación ni otorga licencia de pesos/SaaS.
Leídos adicionalmente checkpoint mutate, state-reporter 61–90, reclaimer
62–130, memory-cleaner 69–155, embedding 520–621, wiki runBuild 1350–1425,
sqlite firmas/markInterrupted y rutas BFF acotadas; no auditoría global de
drivers, materialización de archivos, auth o cleanup físico.
T17/T19 eran menciones de la ampliación previa sin ledger: aquí se remiten
a ruta BFF T16, **no se acredita un guard universal por esos alias**.

**Cierre propio:** ocho mecanismos con contratos/FMEA/aceptación futura;
ninguna prueba, app, build, benchmark, servicio, instalación o git write.
No aceptación ni revisión independiente. Por superar 300 líneas, separación
futura autorizada: specs captura/cursor A/C, publicación/retención D/F,
adapters/acceso/recall B/E/G/H; este dossier conserva evidencia y decisiones.

[T11]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/utils/checkpoint.ts#L150-L375
[T12]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/core/conversation/l0-recorder.ts#L274-L313
[T13]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/core/hooks/auto-recall.ts#L92-L242
[T14]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/src/store/wiki-service.ts#L1350-L1425
[T15]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/src/store/sqlite-store.ts#L547-L559
[T16]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/src/panel/http/routes/chat-memory.ts#L589-L640
[T17]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/src/panel/http/routes/chat-memory.ts#L773-L896
[T18]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/src/store/build-queue.ts
[T19]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/src/panel/http/routes/chat-memory.ts#L122-L185
[T20]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/offload/state-reporter.ts#L61-L90
[T21]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/offload/reclaimer.ts#L62-L130
[T22]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/utils/memory-cleaner.ts#L69-L155
[T23]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/core/store/embedding.ts#L520-L621
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L274-L376
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L302-L335
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L129-L188
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/reflection/mod.rs#L118-L186
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L52-L158
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L228-L329
[V7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/erasure.rs#L191-L243
