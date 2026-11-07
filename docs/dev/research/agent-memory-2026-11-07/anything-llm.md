---
title: "Informe estático — Mintplex-Labs/anything-llm"
kind: research
status: proposed
description: "Evaluación estática de Anything-LLM (plataforma RAG/agentes) a profundidad ampliada: memoria, workspaces, vectores y UI"
tags: [research, memory, anything-llm]
---

# Informe estático — Mintplex-Labs/anything-llm

> **Identidad fijada:** Mintplex-Labs/anything-llm, rama `master`, clon
> `C:\Users\Eros\anything-llm`, SHA **`308b27662b2f6fd4947791d61c797945c609fcbc`**.
> Captura 2026-11-07; paquete raíz 1.19.0. Se conserva la selección original de
> [clone-manifest.json](clone-manifest.json), sin recalcular metadata.
> **Autoria estática ampliada:** código y fuentes públicas leídos el 2026-11-07;
> ningún resultado de ejecución, aceptación final o revisión independiente.
> VantaDB se contrasta con **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**,
> no con cambios concurrentes ni con defectos de informes históricos.

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
11. [Fuentes, cobertura y continuidad](#12-fuentes-cobertura-y-continuidad)

## 1. Arquitectura y mapa

### 1.1. Producto, despliegue y límites de licencia

Es una plataforma de documentos, conversaciones, memorias y agentes, no un motor
vectorial implementado de cero. El checkout público reúne servidor compartido,
collector y SPA. **No contiene el launcher Electron de Desktop**: el inventario
`git ls-files '*desktop*' '*electron*'`, el package raíz y el mapa del README
no acreditan main/preload/IPC de ese producto. Sí hay referencias a Desktop,
modelos de dispositivos y código compartido: no confundirlos con su shell.
La [documentación oficial Desktop][desktop-doc] distingue aplicación single-player
y servidor Docker multiusuario; la diferencia de red se detalla en §11.

Licencia raíz [MIT][license]; `server/package.json`, `collector/package.json` y
`frontend/package.json` declaran MIT. Esto no licencia automáticamente modelos
descargados, proveedores, binarios Electron, assets ni dependencias transitivas.
`embed` y `browser-extension` son **gitlinks no inicializados**, no implementaciones
leídas: commits respectivos `a3a2fea57cfc753698110dd0973a607e3209f634` y
`385d36c0807221e6674d1325ef06505746c49ceb`, identificados con `git ls-tree`.
Su licencia/código y el empaquetado Desktop quedan pendientes; no se descargaron.
Reescribir en Rust no elimina por sí mismo posibles obligaciones de derivación.

### 1.2. Mapa, stack y modelos

| Unidad | Implementación inspeccionada / responsabilidad |
|---|---|
| `server/` | Node >=18.12.1, Express 4.21, Prisma 5.3.1; auth, modelos, REST, streaming, LLM y vector adapters |
| `collector/` | Otro Express, parsers/OCR y extracción; el servidor lo llama por HTTP firmado, no escribe directamente la tabla workspace |
| `frontend/` | React 18.2, Vite, React Router, fetch-event-source; onboarding, documento seleccionado/staged/embedding y citas |
| `server/utils/vectorDbProviders/` | Adapters; aquí se trazó **LanceDB 0.15.0**, local en `STORAGE_DIR/lancedb`, no un servicio externo obligatorio |
| `server/jobs/` | Workers de embedding, sync vigilado y extracción de memorias; estados operacionales distintos de respuesta conversacional |
| `docker/`, `cloud-deployments/` | Despliegue servidor; no se inspeccionaron todas sus variantes ni se levantaron |
| `embed`, `browser-extension` | Dependencias separadas no presentes materialmente; no inferir su seguridad desde el servidor |

Modelos concretos de [schema.prisma][schema]:

- `workspace_documents` (27-39): entero interno, `docId` UUID único,
  `docpath`, `workspaceId`, metadata JSON serializada, `pinned`, `watched`.
- `document_vectors` (113-119) relaciona `docId` con `vectorId`; **no almacena**
  el vector. LanceDB guarda vector, texto y metadata por chunk, con UUID nuevo.
- `workspaces` (121-152): slug único, historial default 20, topN 4, umbral 0.25,
  `chatProvider/chatModel`, prompt y modo chat/query. El vector backend se elige
  por configuración del servidor; no está demostrado un vector store distinto
  por workspace. Se corrige esa generalización de la ficha anterior.
- `workspace_threads` (154-168) enlaza workspace y usuario; `workspace_chats`
  persiste prompt/respuesta y referencias. Un thread no crea una colección aparte.
- `document_sync_queues`/`document_sync_executions` mantienen calendario y runs.
- `memories` (431-446) almacena contenido por usuario y scope workspace/global.
  [Memory][memory] limita global a 5, workspace a 20 y selección inyectada a 5.
  **Sí hay extracción automática Observer→Reflector**: no reducir toda su memoria
  a historial+vectores ni acreditar por ello supersession cognitiva completa.

Los modelos de chat y embeddings se seleccionan por proveedor, no son pesos
propios de AnythingLLM. OpenAI/Ollama y otros adaptadores se ven en dependencias
y selección; no se comprobó equivalencia de todos. Embedding y generación son
fases diferentes; un LLM de chat funcional no acredita un documento indexado.

### 1.3. Ingesta y confirmación real

1. En la SPA [DocumentSettings][picker-ui] separa selección y `movedItems`.
   `moveSelectedItemsToWorkspace` (44-55) **solo prepara**; `updateWorkspace`
   (57-76) llama `Workspace.modifyEmbeddings` y después abre seguimiento.
2. [Upload][workspace-api] (121-161) valida petición/rol, recibe multipart,
   comprueba `Collector.online()` y llama `processDocument`. Si está caído,
   responde 500 con razón. Orden de campos multipart importa para metadata.
3. [CollectorApi.processDocument][collector-api] (114-146) hace POST `/process`
   con `X-Integrity` y signer, espera JSON y devuelve `{success:false,reason}`
   en error. La extracción produce documentos procesados; **el chunking para
   vectores observado está en el adapter Lance**, no en `vectors.js`.
4. `/workspace/:slug/update-embeddings` (234-294) resuelve workspace autorizado,
   aplica deletes y distingue embedder nativo de no nativo. Nativo:
   `embedFiles` devuelve tras despachar worker; **HTTP 200 no es indexación completa**.
   Otro embedder: espera `Document.addDocuments`, pero puede devolver HTTP 200
   con mensaje de documentos fallidos.
5. [Document.addDocuments][documents] (85-248) lee JSON, crea `docId`, emite
   progreso, llama adapter y después crea fila Prisma. Falta de archivo,
   vectorización y persistencia tienen ramas separadas. Si falla la fila,
   intenta compensar vectores y mapping; el cleanup también puede fallar.
6. [Lance.addDocumentToNamespace][lance-write] (318-420) reutiliza cache con
   IDs nuevos o divide texto según chunk size y límite del embedder; solape
   fallback 20. Genera embeddings, escribe tabla, cache y mapping Prisma.
   Son confirmaciones separadas, **no transacción distribuida**.

[EmbeddingWorkerManager][progress] (17-155) agrupa workers por slug y emite SSE,
con replay en memoria solo mientras el worker sigue activo; limpia historial
tras `all_complete`. Exit inesperado emite error terminal. No es un journal
durable recuperable tras reinicio. Su `removeQueuedFile` se refiere a cola, no
acredita rollback de un archivo cuyo proveedor ya está procesando.

### 1.4. Consulta, contexto y derivados

[streamChatWithWorkspace][chat] (93-307) instala abort al desconectar cliente,
lee historial por usuario/workspace/thread, obtiene documentos pinned completos
y archivos parseados, y ejecuta similarity search en namespace `workspace.slug`.
`query` sin namespace/vectores o sin contexto devuelve negativa persistida;
`chat` puede continuar sin documentos. Son contratos distintos, no igual calidad.

[Lance.performSimilaritySearch][lance-query] (423-472) embebe consulta, busca
coseno y opcionalmente rerankea. La vía simple (190-229) toma topN antes de
aplicar umbral/excluir pinned: puede retornar menos de topN, sin que eso sea
un fallo del motor. `score` representa conversión de distancia, no confianza
factual ni score intercambiable con BM25/RRF de Vanta.

`fillSourceWindow` añade contexto histórico, pero `sources` visibles conserva
los hits actuales más pinned/parseados (chat 224-240). **Contexto entregado y
citas mostradas pueden diferir por diseño**. Se comprimen mensajes según proveedor.
El prompt se persiste antes de la respuesta (273-287); abandonar un stream no
implica eliminarlo. La presencia de abort en el conector no prueba cancelación
de embeddings, de todos los tools ni atomicidad de la respuesta final.

### 1.5. Update/delete y memoria automática

`Document.removeDocuments` (251-276) busca `docpath+workspaceId`, borra vectores
y después fila/mapping. [Lance.deleteDocumentFromNamespace][lance-write]
(297-315) necesita el mapping Prisma para encontrar IDs: perderlo no equivale
a haber borrado toda la proyección. Desasociar un workspace tampoco acredita
purga del JSON global ni de copias de conversaciones.

[syncQueue/reembedDocument][sync] (97-252) compara contenido; si cambia, fija un
`published` común para documento y chunks pinned, **borra antes de re-embebir**
y propaga a otros workspaces por filename. Guarda texto nuevo incluso si un
workspace falló; el siguiente sync puede verlo sin cambios y no reintentar ese
embedding. El código y el test mock lo dicen explícitamente: no extrapolar una
garantía de refresh atómico ni trasladar a Vanta ese patrón destructivo.

[processGroup][extraction] (65-181) filtra grupos inactivos/con mínimo de chats,
ejecuta Observer y Reflector, valida/aplica resultados mediante
`Memory.applyExtractedMemories` (345-439), con transacción Prisma y ownership
en updates de IDs propuestos por LLM. El `finally` marca chats procesados incluso
en error. Conteos/cupos leídos antes de la transacción no prueban serialización
entre workers. No se midió esa carrera; es una frontera a discriminar.

### 1.6. Autorización, API y consentimiento

[validatedRequest][auth] (8-110) distingue multiusuario de single-user:
en este último deja pasar en desarrollo o sin `AUTH_TOKEN`/`JWT_SECRET`.
En multiusuario decodifica JWT, carga usuario y rechaza suspendidos.
[validWorkspaceAndThreadSlug][scope] (23-47) combina workspace autorizado,
`user_id` y `workspace_id` del thread; un slug/namespacing no sustituye esta
comprobación. El developer API tiene otra frontera (`validApiKey.js`):
no se certifica igualdad de permisos con todas las rutas SPA/mobile/embed.
El HTTP firmado server→collector tampoco constituye autorización de usuario final.

[DataHandling][consent] (7-35) muestra `ProviderPrivacy` y el botón avanza
directamente a survey. [ProviderPrivacy][privacy] (21-70) consulta `System.keys`,
separa destino LLM, embedding y vector store, y declara proveedor desconocido.
En estos componentes **no se encontró recibo persistido de consentimiento**:
son información contextual, no prueba de aceptación auditable ni revocación.
El renderer de citas [SourceTypeCircle][citation] (60-90) puede pedir favicons
a Google para una URL: incluso con LLM local hay que revisar egress de la UI.

## 2. Recomendaciones

| Prioridad documental | Delta concreto | Motivo / dependencia |
|---|---|---|
| P0 | A: distinguir aceptación, indexación y recuperación verificable en Studio | El 200 del worker nativo y el 200 parcial muestran por qué un spinner/contador no basta |
| P0 | B: freshness por revisión y proyección, sin delete-before-success | Sync puede avanzar fuente aunque falte el vector de un workspace |
| P0 | C: scope de transporte y capacidad de entorno explícitos | Desktop single-player y servidor multiusuario no son intercambiables |
| P1 | D: relación contexto/citas con revisión | `fillSourceWindow` demuestra que lista de fuentes no equivale a ledger de entrega |
| P1 | E: consentimiento contextual al cambiar proveedor/destino | La pantalla de privacidad es útil; no demuestra consentimiento persistido |
| P1 | F: separar extracción vacía, fallida y aplicada | `finally` de memoria no puede convertirse en confirmación universal |
| P2 | G: selección completa paginada en operaciones de corpus | Preparar UI no es mutación; carpeta seleccionada no equivale a primera página |

Se conservan las intuiciones anteriores (separación de procesamiento, scope,
adaptadores y configuración), pero no se propone otro collector, motor, scheduler,
producto de chat ni widget. Los nombres transversales C3/C8/C9/C11/C13/C15/C16
son relaciones conceptuales, no autorización ni tipos Rust existentes.

## 3. Fichas candidatas

### A — Documento→colección con estado de ingestión observable

- **Caso agente y fuente:** un host debe saber si puede usar el archivo recién
  enviado; [workspace API][workspace-api]:234-294 y [worker manager][progress]:91-155
  aceptan trabajo antes de terminar. SHA de todas las fichas:
  `308b27662b2f6fd4947791d61c797945c609fcbc`; enlaces fijados al final.
- **Delta/seam:** Vanta ya tiene `ingest/worker.rs`, `ingest/auto_sync.rs` y
  `utils/task_checkpoint.rs::TaskCheckpointManager`. Extender recibos del worker,
  no introducir otra cola; el checkpoint de tarea conserva pasos, no ejecuta jobs.
  `IngestReport`, `run_with_progress` y `ProgressTracker` ya informan fases;
  `AutoSyncScheduler` ya evita busy y descarta paquetes de runs antiguos.
  El delta evaluable es persistencia/lectura por documento y proyección, no
  «añadir observabilidad» desde cero.
- **Alternativas:** solo SSE (simple, pierde reinicios); polling del primario
  (no prueba proyección); recibo durable por documento/revisión (candidato).
- **Contrato Rust propuesto, no implementado:** `IngestReceipt` recibe identidad
  de fuente+revisión+destino+operación idempotente; devuelve fase, revisión
  visible y error tipado (`Parse`, `Embed`, `Persist`, `Cancelled`, `Conflict`).
  `Accepted` no implica `Searchable` ni obliga a ejecutar una consulta pagada.
- **Pasos/persistencia:** mapear fases reales; persistir transición después de
  cada efecto confirmado; publicar snapshot por API y mostrarlo en Studio.
  Migrar registros previos como estado desconocido, no éxito inventado.
  Desactivar emisión nueva mantiene lectura y reintentos existentes.
- **Ownership/commit/cancelación:** un coordinador por identidad/revisión;
  correlacionar eventos con intento, rechazar progreso de un intento obsoleto.
  Desconectar SSE no cancela trabajo; cancelación explícita antes/después del
  efecto debe devolver el estado observado, no prometer deshacer embeddings.
- **Dependencias/FMEA:** depende de B para freshness y G para selección completa.
  Crash tras vectorizar/antes de recibo → reintento duplicado; detectar por
  identidad de proyección, reconciliar antes de repetir. SSE viejo → falso Ready;
  detectar mismatch de revisión y consultar estado durable. Logs sin texto.
- **Criterios manuales futuros:** envío normal acaba con revisión consultable;
  caída del parser, proveedor y persistencia quedan distinguibles; reinicio y
  dos envíos iguales no pierden ni duplican trabajo; cancelar no produce Ready.
- **No-goals/abierto:** no servicio collector nuevo ni progreso porcentual
  ficticio; queda decidir qué frontera acredita disponibilidad de cada índice.

### B — Freshness por proyección y reemplazo recuperable

- **Caso/fuente:** fuente compartida por dos workspaces cambia; uno falla al
  embeber. [syncQueue][sync]:97-112,124-260 borra antes de añadir y guarda fuente
  aun con fallos; el test `sync-watched-documents.test.js`:125-158 lo conserva.
- **Delta/seam:** ya hay índices y provenance; integrar revisión de derivación
  en `vanta-memory/src/ingest/{worker,auto_sync}.rs` y escritura existente, no
  reemplazar WAL ni diagnosticar un defecto local desde este patrón upstream.
- **Alternativas:** delete-before-add rechazado; transacción única solo si
  ambos efectos están en el mismo dominio; staged revision+activación cuando
  el proveedor sea externo. La elección requiere capacidades del adapter.
- **Contrato Rust propuesto:** entrada `(source_id, expected_revision,
  target, derived_revision)`; salida revisión activa por destino y `Pending`,
  `Applied` o `Failed`. Error de un destino no avanza su watermark.
  `RevisionConflict` evita publicar una derivación anterior a una edición nueva.
- **Pasos/migración/rollback:** registrar identidad y revisión sin mutar fuentes;
  derivar nueva proyección, confirmar y activar; después retirar la vieja.
  Persistir pendientes y tombstones antes de cleanup. Legado: revisión
  desconocida hasta reconciliar; rollback desactiva activación nueva, conserva
  revisión anterior y no resucita documentos borrados.
- **Ownership/cancelación:** serializar activación por fuente/destino, no todas
  las ingestas; cancelación durante embedding conserva pendiente y última
  revisión activa; borrar gana frente a una derivación que termine tarde.
- **Dependencias/FMEA:** A observa intentos; C autoriza destino. Pérdida de
  mapping → huérfanos: detectar inventario contra IDs, reconciliar antes de
  purgar. Texto nuevo/vector viejo → inconsistencia: mostrar ambas revisiones.
  Proveedor sin versionado → no prometer atomicidad; exigir degradación explícita.
- **Criterios futuros:** cambio normal actualiza todos los destinos; un fallo
  deja uno atrasado y reintentable; edición concurrente no publica versión vieja;
  delete concurrente y reinicio no reactivan contenido. Sin inspeccionar payloads.
- **No-goals:** no transacción distribuida universal, nuevo índice ni garantía
  de purga física de backups/citas históricas; retención exige decisión separada.

### C — Scope autorizado y capacidades por transporte

- **Caso/fuente:** un host cambia Desktop local por servicio compartido.
  [auth][auth]:8-110, [scope][scope]:23-47 y [Desktop oficial][desktop-doc]
  demuestran contratos distintos. El widget/gitlinks no se usan como evidencia.
- **Delta/seam:** `RecallScope`, `InjectionPolicy`,
  `perform_auto_recall_governed` en `core/hooks/auto_recall.rs` ya gobiernan
  inyección; bindings/MCP y `desktop/src/transport.ts` deben informar capacidad
  real de su conexión. `MemoryScope` no se acredita como tipo existente.
- **Alternativas:** namespace como ACL (rechazado); confiar solo en UI
  (rechazado); resolver principal y política en frontera servidor y transportar
  capacidades declaradas sin convertirlas en autorización.
- **Contrato Rust propuesto:** `AuthorizedRecallRequest { principal_ref,
  requested_scope, policy_revision }` validado por host; salida resultados
  autorizados+denegaciones redacted. Errores `Unauthenticated`, `Forbidden`,
  `UnsupportedCapability`; la configuración cliente nunca amplía privilegios.
- **Pasos/persistencia:** inventariar llamadas reales; propagar principal;
  validar antes de buscar/expandir citas; exponer capacidades. Versionar política
  y configuración de conexión sin almacenar bearer tokens en registros de memoria.
  Migración de legado explícita; rollback falla cerrado para uso compartido.
- **Concurrencia/cancelación:** ligar petición a revisión de política; revocación
  durante streaming exige detener nuevas entregas, no prometer borrar lo enviado.
  Cancelar request libera consulta, no cambia pertenencia ni borra memoria.
- **Dependencias/FMEA:** D debe reautorizar citas; E verifica egress. Scope
  omitido → fuga: detectar petición sin principal y denegar; entorno Desktop
  expuesto a LAN → alcance ampliado: advertir capacidad real, no asumir auth.
- **Criterios futuros:** principal válido ve solo su alcance; usuario suspendido,
  thread ajeno, ruta alternativa y política revocada no entregan contenido;
  soporte desconocido produce error, no fallback silencioso.
- **No-goals:** no servicio IAM, otro protocolo ni copiar single-user bypass.

### D — Cita de evidencia frente a contexto efectivamente entregado

- **Caso/fuente:** un agente necesita explicar una respuesta con backfill.
  [streamChatWithWorkspace][chat]:224-240 amplía contexto histórico sin añadir
  esas fuentes a las citas actuales; [Citation][citation] presenta metadata.
- **Delta/seam:** provenance e `IntegratedContext`/`CompactionReport` existen;
  extender `context_engine/engine.rs::assemble_inner` y
  `core/hooks/auto_recall.rs::perform_auto_recall_governed` con trazabilidad de
  entrega; `memory_generation_log` registra generación, no ese mismo evento.
- **Alternativas:** lista de hits (insuficiente); guardar prompt completo
  (riesgo de privacidad); manifest de referencias y revisiones más motivos de
  exclusión/compactación, preferido como candidato.
- **Contrato Rust propuesto:** entrada ensamblado y referencias autorizadas;
  `DeliveryManifest` devuelve source ID, revisión, origen current/history/pin,
  estado incluido/compactado/excluido. No implica que el modelo leyó o verificó
  la afirmación. Errores `MissingRevision`, `PolicyChanged`, `SourceUnavailable`.
- **Pasos/persistencia:** vincular referencias al ensamblado final después de
  presupuesto; persistir manifest mínimo ligado al turno; Studio abre la revisión,
  no el archivo actual por nombre. Legado = desconocido; rollback desactiva nuevos
  manifests sin eliminar provenance ni historial.
- **Ownership/concurrencia/cancelación:** host dueño del manifest; cerrar solo
  al conocer lo enviado; cancelación conserva entrega parcial etiquetada.
  Borrado/revocación obliga reautorizar navegación y evita rescatar contenido.
- **Dependencias/FMEA:** B suministra revisión, C permisos. Cita vieja tras
  refresh → detectar mismatch y mostrar no disponible; fuente recuperada pero
  descartada por presupuesto → no mostrarla como prueba usada. Contadores sin texto.
- **Criterios futuros:** current+history+pins quedan diferenciados; compactación
  no inventa fuentes; interrupción muestra parcial; borrado no revela payload
  desde inspector. Verificación factual requiere otro mecanismo.
- **No-goals:** no judge LLM, score de confianza ni almacenar prompts íntegros.

### E — Consentimiento ligado al destino efectivo

- **Caso/fuente:** documentos privados cambian de embedder local a remoto.
  [DataHandling][consent]:14-35 informa pero no persiste aceptación;
  [ProviderPrivacy][privacy]:29-46 distingue tres proveedores.
- **Delta/seam:** proveedores opcionales ya existen; extender configuración del
  host/adapters de embedding y conexiones Studio (`ConnectionSidebar.tsx`,
  `transport.ts`), no insertar consentimiento dentro de BM25/HNSW.
- **Alternativas:** aviso genérico al instalar (caduca); bloquear toda red
  (limita uso legítimo); confirmación por finalidad/destino efectivo.
- **Contrato Rust propuesto:** `EgressDecision` recibe principal, operación,
  destino normalizado y revisión de configuración; devuelve permiso/denegación
  con referencia a recibo. `ConsentRequired`, `DestinationChanged`, `UnknownPolicy`
  no se convierten en permiso por disponibilidad del proveedor.
- **Pasos/persistencia:** inventariar salidas incluidas favicon/telemetría;
  mostrar destino y clase de datos antes de transmisión; guardar recibo mínimo
  versionado; invalidarlo al cambiar endpoint/finalidad. Migración sin recibo
  pide nueva autorización; rollback no habilita egress previamente denegado.
- **Ownership/cancelación:** host valida justo antes de enviar; edición
  concurrente invalida decisión; revocar impide envíos futuros, no borra datos
  ya transmitidos ni garantiza cancelación del proveedor remoto.
- **Dependencias/FMEA:** C identifica actor. Endpoint custom clasificado como
  local → fuga: comparar destino resuelto y avisar desconocido; recibo viejo →
  detectar revision mismatch. Auditar decisiones sin secretos ni documentos.
- **Criterios futuros:** destino estable aceptado funciona; cambio de hostname,
  embedder o política exige nueva decisión; rechazar no transmite; abrir citas
  no produce egress oculto sin política.
- **No-goals:** no asesoría legal ni afirmar que la pantalla upstream demuestra
  cumplimiento; jurisdicción y retención de recibos requieren decisión del owner.

### F — Extracción automática con resultado no ambiguo

- **Caso/fuente:** batch de chats sin hechos frente a modelo caído.
  [processGroup][extraction]:119-181 distingue candidatos nulos/vacíos en logs
  pero marca procesados en `finally`; [Memory.applyExtractedMemories][memory]
  valida ownership de updates y aplica transacción Prisma.
- **Delta/seam:** Vanta ya extrae, deduplica y marca contradicciones;
  extender `core/record/l1_writer.rs::{plan_write,persist_planned,
  apply_dedup_batch}` y checkpoint/generation log existentes; no nuevo Reflector.
- **Alternativas:** booleano processed (ambiguo); reintentar todo (duplica coste);
  resultado tipado por ventana y revisión con cursor confirmado por efecto.
- **Contrato Rust propuesto:** `ExtractionOutcome` recibe IDs/revisiones de chats,
  configuración y scope; salida `NoFacts`, `Applied(ids)`, `RetryableFailure`,
  `Rejected` o `Cancelled`. `NoFacts` exige llamada válida, no timeout ni parse
  fallido. Ningún ID sugerido por LLM autoriza editar otro propietario.
- **Pasos/persistencia:** delimitar ventana; registrar intento; validar salida
  y propiedad antes de `plan_write`; persistir y confirmar cursor; reconciliar
  efecto/cursor tras crash. Legado conserva significado desconocido; rollback
  suspende extracción nueva sin revertir hechos ya aplicados automáticamente.
- **Ownership/concurrencia:** exclusión por ventana/scope y revisión esperada;
  cancelación antes del commit no marca éxito; posterior devuelve aplicado.
  Cupos revisados antes del plan deben revalidarse donde se publica.
- **Dependencias/FMEA:** A observa, C autoriza, D enlaza origen. Timeout marcado
  vacío → pérdida de memoria: detectar error tipado, no avanzar. Cursor atrasado
  tras commit → duplicados: reconciliar IDs/dedup ya existente; no guardar raw LLM.
- **Criterios futuros:** ventana sin hechos avanza como NoFacts; error temporal
  queda pendiente; respuesta hostil/ID ajeno se rechaza; dos workers y reinicio
  no duplican hechos ni exceden política sin señal explícita.
- **No-goals:** no prompts nuevos, consenso factual, ni copiar cupos upstream.

### G — Selección de corpus completa antes de mutación

- **Caso/fuente:** usuario marca una carpeta mayor que la página visible.
  [picker][picker-ui]:44-76 resuelve selección antes de stage y POST;
  el hook completo [`useDocumentPicker.js`][picker-hook]:643-689 es la
  frontera de expansión (ruta verificada:
  `frontend/src/components/Modals/ManageWorkspace/Documents/hooks/`).
- **Delta/seam:** Studio ya lista y busca; `RetrievalLens.tsx`:82-104 carga una
  página para elegir vector y 121-129 filtra resultados visualmente. Eso no
  prueba defecto del motor ni acredita selección masiva completa.
- **Alternativas:** seleccionar solo visibles (válido si se rotula); recorrer
  páginas sin snapshot (riesgo de drift); manifest de IDs/revisiones para una
  operación confirmada. Reutilizar listPage y mutaciones oficiales.
- **Contrato Rust propuesto:** `SelectionPlan(scope, filter, cursor_revision)`
  devuelve IDs/revisiones y alcance visible/todo; errores `SelectionChanged`,
  `Forbidden`, `Cancelled`. El botón de stage no ejecuta ingest/delete.
- **Pasos/persistencia/migración:** distinguir visible/todo; resolver páginas;
  confirmar plan; enviar operación idempotente A; persistir solo manifest de
  operación si cruza reinicios. No migración del motor; rollback UI conserva
  selección visible explícita, nunca amplía silenciosamente el lote.
- **Ownership/cancelación:** principal autoriza cada ID al commit; cambio
  concurrente devuelve conflicto o resultado por ID; cancelar expansión no muta.
- **Dependencias/FMEA:** A y C. Página faltante → omisión: reconciliar count/cursor;
  documento nuevo durante paginado → scope drift: exigir revisión o avisar
  selección no congelada. Progreso sin nombres sensibles por defecto.
- **Criterios futuros:** carpeta multipágina se resuelve completa o se declara
  parcial; buscar y seleccionar visibles no incluye ocultos; cancelación y
  revocación no disparan borrado ni embedding.
- **No-goals:** no nuevo explorador ni convertir filtro visual en filtro de motor.

## 4. Incidencias y riesgos

Fuentes públicas leídas el 2026-11-07; estado de la página, no certificación
de todas las distribuciones ni atribución de la introducción histórica del fallo.

| Incidencia / estado observado | Evidencia y límite | Aplicabilidad concreta |
|---|---|---|
| [#3958 Weaviate embeddings persist after removal](https://github.com/Mintplex-Labs/anything-llm/issues/3958), abierto 2025-06-05, cerrado completed 2025-06-30 | El reporte mezcla persistencia tras delete y cache desaprovechada. Comentario 2025-06-29 **no reproduce delete**, sí `skipCache` invertido; cierre refiere `d1d68af`. No se leyó ese commit histórico completo | B: inspeccionar mapping/compensación y semántica de cache por backend. No afirmar que todos borran mal ni que el cierre arregló ambos síntomas |
| [#6806 Failed parsed-attachment move or embed deletes attachment](https://github.com/Mintplex-Labs/anything-llm/issues/6806), abierto/cerrado **not planned** 2026-10-22 | Autor propone #6807; maintainer declara rechazo+re-upload como UX intencional. En SHA fijado `workspaceParsedFiles.js`:110-174 sí borra fila en finally antes de confirmar embed | A/B: separar rechazo final de fallo recuperable y explicar pérdida de selección; no diagnosticar Vanta ni afirmar merge de #6807 |

**Riesgos estáticos adicionales:** mapping SQL y tabla Lance tienen commits
separados (§1.3/1.5); backfill y citas visibles no son la misma población (§1.4);
raw respuestas/hechos aparecen en logs de extracción (§1.5); provider privacy
no es enforcement de egress (§1.6). FMEA por mecanismo en A–G.

## 5. Prerrequisitos Vanta

1. **A/B:** contrastar estado wiki/run y `ingest/merge.rs::commit` antes de
   añadir campos. Snapshot Vanta ya posee `begin_processing→complete/fail`,
   progreso best-effort, scheduler y checkpoints; precisar qué metadatos faltan.
2. **C/E:** inventario de principals y salidas por transporte/adapters;
   revisar enforcement de `InjectionPolicy` sin declarar todos los transports
   cubiertos por su existencia. No copiar bypass de single-user.
3. **D/F:** mapear source/revision/cursor a `assemble_inner`, generation log y
   escritura L1; distinguir token estimado de límite del proveedor. Preservar
   prefijo protegido: puede superar presupuesto (`engine.rs`:197-218).
4. **G:** verificar semántica de `listPage` y alcance visible/todo de Studio.
   No implementar contratos mientras no haya decisión y validación manual.

## 6. Que no trasladar

- **Rechazar** plataforma de chat/collector y ORM como dependencia del motor:
  son producto/orquestación, no primitivas de almacenamiento Rust.
- **Rechazar** delete-before-success y processed universal en finally como
  garantías; respetar intención UX upstream sin adoptar su semántica.
- **No portar** widget ni browser extension no materializados, shell Electron
  no leído, cada proveedor o parámetros de chunk/rerank por mera disponibilidad.
- **No equiparar** cita a verificación, workspace a tenant, score a confianza,
  ni añadir llamadas LLM obligatorias al retrieval.

## 7. Advertencias metodológicas

Lectura estática del clon shallow, no resultado experimental. Rutas concretas
frontend sí leídas; queda descartada la afirmación previa «frontend solo
estructura». Dos issues leídos no constituyen auditoría de seguridad exhaustiva.
La documentación pública Desktop puede evolucionar fuera del SHA de servidor:
se etiqueta como documentación, no como código Electron inspeccionado.

No se recalculó metadata, inicializaron submódulos ni consultaron secretos.
No atribuir a Vanta defectos por analogía. C1/C3/C8/C9/C11/C13/C15/C16 son
referencias conceptuales de campaña, no un backlog aprobado ni contratos
implementados. MIT por componente no licencia automáticamente todos sus assets.

## 8. Benchmarks

### 8.1. Runner efectivamente leído y lo que no mide

`package.json::scripts.test` → Jest 29.7; `jest.config.cjs`:1-6 solo ignora
node_modules. El test [sync-watched-documents.test.js][sync-test]:1-164 importa
el job, reemplaza collector/vector database/source writes por mocks y usa
`OLD_CONTENT`, `NEW_CONTENT`, timestamp fijo y tres workspaces. La unidad es
caso programado/parametrizado de refresh, **no pregunta, sesión o retrieval hit**.
No hay split, ground truth semántico, answerer ni juez. El denominador es el
conjunto de casos seleccionados por Jest, no un corpus de calidad.

Los casos verifican fallo por workspace sin reintento y coherencia de published;
excepciones/expectaciones fallidas hacen fallar el caso, no reciben score cero
de una rúbrica de memoria. Modelos, semillas, tokenizer, coste por token,
latencias p50/p99 y recuperación semántica **no se miden**; mocks no acreditan
corrección de Lance/Weaviate reales. No se ejecutó el runner.

### 8.2. Efectos y adaptación, sin harness nuevo

El test aislado simula escritura/embedding; `yarn test` global descubriría otros
tests cuyo aislamiento no se auditó. `prisma:reset` del package raíz trunca DB;
`setup` instala/migra y `dev` inicia tres procesos: **no ejecutarlos**.
No se localizó runner comparable de memoria en package scripts, `server/jobs`,
`server/__tests__` o nombres bench/eval inspeccionados; no es ausencia global.

Para una evaluación futura se conserva el protocolo de
[METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md), sin diseñar prompts,
scoring, configs ni suite. Adaptación posible del caso existente: sustituir
interfaces mock de `deleteDocumentFromNamespace/addDocumentToNamespace` por
frontera de ingesta Vanta y conservar source/destino/revisión/error por operación.
El test upstream no sería benchmark de HNSW ni evaluación cognitiva. No usarlo
para publicar velocidad, recall o ahorro de coste; requiere runner ya existente
apropiado al nivel (motor/retrieval/memoria/answer/éxito de agente).

## 9. Matriz

Referencia Vanta **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**;
fuentes [VANTA-REFERENCE](VANTA-REFERENCE.md) y lecturas locales señaladas.
«Parcial» califica el delta documentado, no un producto defectuoso.

| Capacidad | AnythingLLM fijado | Vanta fijado / fuente | Decisión incremental |
|---|---|---|---|
| Extracción/ingesta | Existe: collector→document→adapter | Existe: `ingest/worker.rs::run_with_progress`, `IngestReport`, wiki estados | A parcial: recibo durable por proyección, no otra cola |
| Sync y revisión | Existe; refresh separado por workspace | Existe: `ingest/auto_sync.rs::AutoSyncScheduler`, hashes/run_id | B parcial: evaluar manifest destino/revisión y reconciliación |
| Búsqueda | Existe Lance coseno/rerank | Existe BM25/HNSW/RRF/filtros: `src/sdk/search`, `src/index` | No importar motor; comprobar ruta recall default |
| Provenance/citas | Existe metadata; backfill no igual citas | Existe: `core/record`, `context_engine::IntegratedContext` | D parcial: entrega final y revisión, no provenance desde cero |
| Memoria automática | Existe Observer→Reflector→SQL | Existe dedup/supersession: `l1_writer.rs`, `l1_dedup.rs`, lifecycle | F parcial: outcome/cursor explícitos |
| Sesiones y tenancy | Existe workspace/thread/user | Existe `RecallScope`, `InjectionPolicy`, `l1_writer::default_tenancy` | C: auditar integración por transporte; no IAM nuevo |
| Budgets/cuotas | Cupos memory e historial, no misma unidad | Existe recall budget/compaction, namespace quotas `src/config.rs`:760-762 | No copiar límites; no negar enforcement sin leer caller |
| Scheduler/checkpoints | Workers y tablas sync; SSE no durable | Existe `TaskCheckpointManager`, scheduler, pipeline checkpoint | Reusar; no confundir checkpoint con memoria factual |
| UI | SPA fuente leída; Desktop shell no acreditado aquí | Studio existe en worktree; retrieval y memoria leídos | G/E/D como extensión, no otra app |
| Consentimiento durable por destino | No acreditado en componentes leídos | No acreditado en seams Studio muestreados | E propuesta condicionada, no ausencia global |

## 10. Negativos Vanta

El contraste sugiere **preguntas discriminantes**, no defectos probados:
¿un recibo identifica revisión de cada derivado?, ¿cita visible distingue
recuperado de entregado?, ¿rechazo recuperable se diferencia de vacío?,
¿las conexiones cambian política de egress?, ¿selección masiva informa su alcance?
Seams reales en A–G y criterios futuros permiten contestarlas sin inventar
carencias generales. Ya existen ingesta observable, ACL, compaction y dedup.
El `core_search` default false requiere especificar qué vía se compara.

## 11. Escritorio / UI

**AnythingLLM:** SPA React→`DocumentSettings`→`resolveSelection`→stage→
`Workspace.modifyEmbeddings`→SSE. `resolveSelection`:643-689 carga carpeta
completa con `System.localFiles(..., NO_LIMIT)` cuando faltan páginas, deduplica
IDs y excluye ya embebidos; no acredita snapshot concurrente ni paginado acotado.
Chat→citations/sidebar muestra metadata, no prueba qué texto consumió el LLM.
Onboarding→DataHandling→ProviderPrivacy informa destinos sin recibo leído.
Desktop es **Electron según el comentario de compatibilidad leído en
`frontend/src/utils/folderUpload.js`:3-15**; no se atribuye a este árbol código
main/preload no encontrado. Server, collector,
SPA, API/mobile y gitlinks embed/extension son fronteras diferentes.

La documentación oficial dice Desktop single-player, backend loopback por
defecto; network discovery expone API en `0.0.0.0` tras reiniciar, **no SPA**;
desaconseja Internet por ausencia de cuentas/auth delante de Desktop. Docker
soporta multiusuario y widgets. No generalizar esa ausencia al servidor validado.

**Studio existente:** worktree `C:\Users\Eros\VantaDB Proyect\develop-desktop`,
referencia observada `0ad38c542` en VANTA-REFERENCE (no se recalculó metadata).
Se leyó `desktop/src/components/lens/retrieval/RetrievalLens.tsx`:54-164:
estado de consulta/topK, `listPage` para picker, `search({explain:true,
search_profile})` y filtros visuales posteriores sin reordenar ranking.
`desktop/src/vanta.ts`:779-846 usa `transport.call` para escenas, persona y
skills; no es una pantalla vacía por falta de carpeta desktop en develop.
Extensiones propuestas: inspeccionar recibo/revisión al abrir registro,
rotular alcance visible/todo, enlazar manifest de entrega y destino de conexión.
No se arrancó frontend ni se promete paridad Tauri/HTTP/WASM por un wrapper.

## 12. Fuentes, cobertura y continuidad

### 12.1. Código y fuentes públicas

Todos los enlaces de código siguientes fijan el SHA completo del informe;
rangos orientan lectura, no implican auditoría del archivo entero.

[license]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/LICENSE
[schema]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/prisma/schema.prisma#L27-L168
[memory]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/models/memory.js#L345-L439
[workspace-api]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/endpoints/workspaces.js#L121-L295
[collector-api]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/collectorApi/index.js#L114-L146
[documents]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/models/documents.js#L85-L276
[lance-write]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/vectorDbProviders/lance/index.js#L297-L420
[lance-query]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/vectorDbProviders/lance/index.js#L423-L472
[progress]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/EmbeddingWorkerManager.js#L17-L175
[chat]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/chats/stream.js#L93-L307
[sync]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/jobs/sync-watched-documents.js#L97-L260
[extraction]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/jobs/extract-memories.js#L65-L181
[picker-ui]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/components/Modals/ManageWorkspace/Documents/index.jsx#L10-L94
[auth]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/middleware/validatedRequest.js#L8-L110
[scope]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/utils/middleware/validWorkspace.js#L23-L47
[consent]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/pages/OnboardingFlow/Steps/DataHandling/index.jsx#L7-L35
[privacy]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/components/ProviderPrivacy/index.jsx#L21-L70
[citation]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/components/WorkspaceChat/ChatContainer/ChatHistory/Citation/index.jsx#L60-L90
[sync-test]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/server/__tests__/jobs/sync-watched-documents.test.js#L1-L164
[picker-hook]: https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/components/Modals/ManageWorkspace/Documents/hooks/useDocumentPicker.js#L643-L689
[desktop-doc]: https://docs.anythingllm.com/installation-desktop/overview

Evidencia de compatibilidad Electron, no del shell:
[folderUpload.js](https://github.com/Mintplex-Labs/anything-llm/blob/308b27662b2f6fd4947791d61c797945c609fcbc/frontend/src/utils/folderUpload.js#L3-L15).

### 12.2. Cobertura por eje y lagunas durables

| Eje solicitado | Leído / documentado | No leído / no demostrado |
|---|---|---|
| Recomendaciones/arquitectura | Packages, schema, fronteras y priorización §1-2 | Todos los providers/despliegues |
| Extracción/Rust | Flujos y A–G con contratos propuestos/seams/FMEA | Implementación y aceptación runtime |
| Incidencias | #3958 y #6806 completos con discusión/estado §4 | Diff histórico d1d68af y PR #6807 completos; no afirmar merge |
| Prerrequisitos | Snapshot Vanta, worker/auto-sync/checkpoint y contexto §5 | Enforcement de cada SDK |
| Rechazos | §6, alternativas y no-goals A–G | Auditoría legal de dependencias/modelos |
| Advertencias | §7 y distinción documento/código/reporte | Reproducción de fallos |
| Benchmarks | Jest config, test sync, package scripts §8 | Suite global, calidad semántica, coste/latencia reales |
| Matriz | §9 con fuente por capacidad | Paridad de todas las variantes |
| Negativos Vanta | Preguntas discriminantes §10 | Ningún bug local demostrado |
| Escritorio/UI | SPA flujo, privacidad/citas; Studio código §11 | Shell Electron/IPC, gitlinks embed/extension, ejecución UI |

**Accesos:** websearch DuckDuckGo para issues embedding/delete devolvió cero;
metasearch dio resultados ajenos y algunos proveedores 403/timeouts; no se usaron
como evidencia. Fallback `websearch.fetchWebContent` a listado GitHub, dos issues
y Desktop oficial sí entregó contenido. El intento de leer
`server/jest.config.cjs` no encontró archivo; se leyó el config raíz correcto.

**Cierre de autoría:** se preservó la arquitectura válida y se sustituyeron
fichas parciales/afirmaciones contradictorias por siete mecanismos trazados,
incidencias leídas, benchmark acotado y comparación sin reinventar capacidades.
No apps/tests/builds/benchmarks/servicios/install, fetch/pull ni git write.
Propuestas no implementadas; **no revisión independiente ni aceptación**.
Por superar 300 líneas, una futura fase autorizada puede separar especificaciones
de ingesta/freshness (A/B/G), gobernanza/entrega (C/D/E) y extracción (F);
este informe permanece autónomo y no crea esos documentos ahora.
