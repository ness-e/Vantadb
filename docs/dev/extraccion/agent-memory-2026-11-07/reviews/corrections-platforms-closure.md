---
title: "Reconfirmación independiente de correcciones: plataformas y frameworks"
kind: research
status: proposed
description: "Cierre documental propuesto de once informes y diecisiete hallazgos históricos, con evidencia estática y límites por proyecto"
tags: [research, memory, engineering]
---

# Reconfirmación independiente de correcciones: plataformas y frameworks

## Alcance, identidad y criterio

Revisión independiente de la autoría de los once informes, sobre sus cuerpos
corregidos completos y el delta recibido, no aceptación final. Único archivo
editable: este recibo. Antes de planificar se comprobó que no existía.
Rigen [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md) y
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md). Antecedentes:
[plataformas](platforms-frameworks.md), [adiciones](frameworks-additions.md) y
[restantes](platforms-retrieval-remaining.md). Sus líneas históricas no se
confunden con las actuales citadas debajo.

Entrada: `%TEMP%/vantadb-fix-platforms.md`, once
`vantadb-fix-platforms-<nombre>.BASE.md` y diff real
`vantadb-fix-platforms.diff`: **1169 líneas**, SHA256 corroborado
`638858E6A1E366C56E0DA2D4E631BDE443CF9B6992F8988E5081614B860A9EAB`.
Las bases son las recibidas antes del rework, no una reconstrucción de HEAD.
La declaración del autor en su recibo no reemplaza el contraste de fuentes.

Referencia motor/adapters **V** =
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; Studio **S** =
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`, worktree separado.
Las lecturas de grafo sólo orientan; para atribuir hechos a V/S se usan cuerpos
de `git show`, sin mantenimiento de índices. No se confunden bytes dirty
con los del commit.

Estados de cada eje: **sustentado**, **parcial**, **no acreditado** o
**no aplicable justificado**, relativos a esta muestra estática, no al producto
entero. Disposición histórica: **cerrado** = rectificación documental verificada;
**residual** = parte sustantiva corregida con resto explícito;
**no resuelto** = persiste la contradicción. Ninguno significa test pasado,
seguridad certificada, adopción aprobada ni aceptación final del responsable.

## Índice

- [Dify](#dify)
- [Langflow](#langflow)
- [LangChain](#langchain)
- [LangGraph](#langgraph)
- [Agno](#agno)
- [AutoGen](#autogen)
- [Flowise](#flowise)
- [LightRAG](#lightrag)
- [PageIndex](#pageindex)
- [Serviette](#serviette)
- [Meilisearch](#meilisearch)
- [Disposición conjunta y límites](#disposición-conjunta-y-límites)

## Dify

**Recibo:** [dify.md](../dify.md), cuerpo actual 1–752 y delta contra BASE;
pin `65b4e17d6ebeff39c7e66c832649afff01666adf`.
**Veredicto propuesto:** sin bloqueantes documentales de los dos hallazgos
históricos en la muestra; propuestas C/D aún no son garantías implementadas.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Identidad/arquitectura/flujos | sustentado | 26–120: SQL/fuente, Celery, derivados, consulta y actualización separados; Graphon 668–677 no es memoria factual. |
| 1. Recomendaciones | sustentado | 122–162: A/B diagnóstico, C publicación, D schema y E cómputo; no nuevo motor/runtime. |
| 2. Fichas | sustentado | 164–399, 653–666, 708–712: owner concreto de C/D, contratos propuestos, errores, FMEA y aceptación futura. |
| 3. Incidencias | sustentado | 423–442: metadata D; E se apoya en CacheEmbedding, no #45638/#41597. PR no inspeccionada sigue sin fix certificado. |
| 4. Prerrequisitos | sustentado | 240–284, 305–356, 450–459: worker/SDK actuales, schema y revisión por fuente propuestos; no WAL/scheduler paralelo. |
| 5. Rechazos | sustentado | 461–470: no marketplace, Celery, ponderaciones ni texto completo por defecto. |
| 6. Método/licencia | sustentado | 36–43, 472–499, 714–720: Apache modificado, multitenancy/frontend y terceros delimitados; no licencia uniforme. |
| 7. Benchmarks | sustentado | 501–570: intento SSE, muestras condicionadas a eventos, error terminal puede marcar transporte exitoso; no QA. |
| 8. Matriz | sustentado | 572–584: worker/CommitReport y metadata tipada ya existen; delta por callback/schema, no negar capacidades V. |
| 9. Negativos Vanta | sustentado | 586–592: preguntas sobre configuración/previews, no bugs deducidos por analogía. |
| 10. UI | sustentado | 594–632: web hit-testing/canvas/run/credenciales y RetrievalLens S; visual no otorga autorización ni certifica commit. |

### DIFY-1 y DIFY-2

- **DIFY-1 — cerrado documentalmente.** C 240–292 identifica
  `worker::{begin,execute,ingest_body}`, callback `WikiStore::put_page`,
  `(namespace,slug,path)`, terminal `run_id`, errores y CommitReport.
  Contraste V: [worker:244–309][v-worker], [merge:34–49][v-merge],
  [store:287–320][v-page] y [terminal:162–217][v-wiki-terminal].
  `put_page` no recibe run_id: el terminal no se vende como fencing por write.
  D 305–354 distingue `MemoryInput.metadata`, `Value`, JSON/CSV tipados y
  [validate_metadata:252–266][v-validate] (claves reservadas/NUL, no schema).
  [put_batch:1280–1350][v-batch] valida antes, confirma por chunk.
  FMEA/aceptación 285–292/347–354, matriz 580–581 y cierre 708–712
  mantienen parcialidad/revisión propuesta; UI 601–630 no promete commit global.
- **DIFY-2 — cerrado documentalmente.** 437–440 restringe ambos reportes a D.
  [CacheEmbedding:24–133][dify-cache] acredita slots/modelo/hash de E; una
  incidencia de metadata no acredita cardinalidad de embeddings. URLs históricas:
  [#45638](https://github.com/langgenius/dify/issues/45638) y
  [#41597](https://github.com/langgenius/dify/issues/41597).
  No se atribuye merge de #45639 ni reproducción propia.

**Límites:** no CAS global, plugins, todos los índices/permisos ni seguridad
runtime acreditados. Revisión de fuente, publicación y schema son diseño futuro;
no es motivo para exigir ejecución universal antes de cerrar estos errores
documentales.

## Langflow

**Recibo:** [langflow.md](../langflow.md), cuerpo 1–688 y delta BASE;
pin `60999cc9a7350e44dde3ef641baae864e10861f0`.
**Veredicto propuesto:** sin bloqueantes documentales de LF-1/LF-2 en la
muestra, sin certificar backends, UI ejecutada ni adopción de A–E.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 26–164: ingesta sin lease durante embedding, revalidación al write, routing y cursor separados; N definidos en 673–682. |
| 1. Recomendaciones | sustentado | 166–214: A/B antes de C/D/E según necesidad, no canvas ni SQLite como sustituto de HNSW. |
| 2. Fichas | sustentado | 216–437: cinco mecanismos, contratos/errores propuestos, ownership, migración, FMEA y aceptación futura; V1–V5/VS1 ahora resolubles. |
| 3. Incidencias | sustentado | 463–487: #13264/#14994 cerrados y #15800 propuesta; comentarios de fix no equivalen a diff/merge verificado. |
| 4. Prerrequisitos | sustentado | 282–311, 489–498: explain, VectorStore/conexiones, L1 y TaskCheckpointManager existentes antes del delta. |
| 5. Rechazos | sustentado | 500–509: no ACL desde filtro tolerante, marketplace o scheduler paralelo. |
| 6. Método/licencia | sustentado | 511–537, 626–671: MIT raíz/terceros, fuentes, lagunas, dependencias, manual futuro y split materializados. |
| 7. Benchmarks | sustentado | 539–582: runner completo, semilla/defaults, self-match, primera consulta incluida, máximo no percentil, aserción previa al JSON. |
| 8. Matriz | sustentado | 584–595: capacidades V existentes, historial por item fuente N9/N10 y propuesta de proyección, no commit por modal. |
| 9. Negativos Vanta | sustentado | 597–603: interrogantes de contrato, sin bugs ni carencias globales inventadas. |
| 10. UI | sustentado | 605–624: spans distintos de runs/items; polling sólo de página cargada; Studio S existente. |

### LF-1 y LF-2

- **LF-1 — cerrado documentalmente.** 626–688 añade §11 y definiciones
  N1–N10/V1–V5/VS1 con pin/ruta/rango; corrige referencias compuestas
  no navegables. El historial se respalda en cuerpos separados:
  [modal:14–222][lf-modal] muestra item/status/error y run/job;
  [sección:15–86][lf-runs] sólo hace polling si la página 1/limit 10 contiene
  runs no terminales. No se acredita censo global de jobs ni persistencia
  backend desde ese polling. [operation:282–316][lf-operation] contrasta
  routing/owner/generación bajo lock; [_advance_cursor:838–879][lf-cursor]
  puede retornar al desaparecer la sesión. Los límites 121–164 y 663–671
  no anuncian exactly-once ni commit vectorial+SQL común.
  FMEA/aceptación E 428–435 y matriz 593 conservan la distinción visual/durable.
- **LF-2 — cerrado documentalmente.** [sqlite_kb.py completo:1–114][lf-runner]:
  seed 20261001, batch 250, k10, rows10000, dims384/1536/3072, runs5.
  Cronómetro de importación incluye construcción de vectores/documentos;
  mediana incluye primera consulta. `all/one_percent` son dos casos del mismo
  vector, no población QA. AssertionError evita JSON de esa dimensión, aunque
  pueda haber JSON previo. 573–582 concreta adapter a `MemoryInput/put_batch`
  y `Embedded::search` existentes sin cambiar defaults, score o harness;
  no hereda atomicidad SQLite al batch Vanta.

**Límites:** enlaces restaurados no son lectura exhaustiva de sus intervalos
amplios ni de todos los backends/guards/licencias; aceptación de delete/migrate,
retry/purge y privacidad sigue futura. No se exige runtime para verificar
estas correcciones documentales.

## LangChain

**Recibo:** [langchain.md](../langchain.md), cuerpo 1–562 y delta BASE;
pin `d9caf083577c43a91c5bf6bd1f7dfc289581184d`.
**Veredicto propuesto:** LC-1/LC-2 corregidos documentalmente; conformidad
de la combinación de versiones y garantías de mutación siguen por validar,
no se presenta fallo del adapter como ausencia de transacciones del motor.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 26–162: mensajes/historial/grafo/vector store separados; mutación con IDs y parcialidad ahora trazada. |
| 1. Recomendaciones | sustentado | 78–92: adapters actuales antes de funciones nuevas; no LangChain Rust. |
| 2. Fichas | sustentado | 164–383: A–F y contraste temporal incluyen pasos, errores, rollback/límites, FMEA y aceptación futura. |
| 3. Incidencias | sustentado | 385–406: #41323/#38867 separados por mecanismo/revisión; no fix desde cierre ni CVE inventada. |
| 4. Prerrequisitos | sustentado | 408–424: por candidata, context engine, approval de capturas y checkpoint existentes; no presupuesto de storage confundido con tools. |
| 5. Rechazos | sustentado | 426–437: no buffer/índice temporal classic, framework ni historial como checkpoint. |
| 6. Método/licencia | sustentado | 26–76, 439–446, 519–543: MIT por superficie y terceros, muestra y split; legacy ahora contrastado, no descartado por etiqueta. |
| 7. Benchmarks | sustentado | 448–473: conversión de 20 schemas warm/cold; rondas del fixture desconocidas, no QA/recall o resultado Rust. |
| 8. Matriz | sustentado | 475–491: mutación existe/conformidad parcial y CompositeScoring opt-in; no nuevo motor ni inversión cosine ya descartada. |
| 9. Negativos | sustentado | 493–502: append/grupos/score son hipótesis; conserva excepción de prefijo protegido. |
| 10. UI | sustentado | 504–517: LangSmith externo documentado, no frontend OSS inspeccionado; delta Studio propuesto con permisos actuales. |

### LC-1 y LC-2

- **LC-1 — cerrado documentalmente.** 132–154 y F 330–354 distinguen
  [Core add/update:234–291][lc-add] del override Vanta.
  [_build_key:84–85][v-lc-key] = UUID5(texto:posición);
  [add_texts/add_documents/delete:414–495][v-lc-mutations] = puts/deletes
  individuales, UUID4 faltante en documentos, colisión posible de kwargs.ids
  con `ids=ids`, validación de listas no vacías antes de embedding, sin
  prevalidación global posterior de cardinalidad. Excepción puede dejar prefijo
  aplicado; retry/cancel no son undo. Metadata no se fusiona en este adapter.
  FMEA 342–346, aceptación 347–354 y matriz 480 preservan esa frontera;
  UI 513–517 no cambia recibo por estado visual.
- **LC-2 — cerrado documentalmente.** 356–383 contrasta
  [TimeWeightedVectorStoreRetriever:110–198][lc-time-buffer]:
  touch sólo del buffer, copias profundas y extensión de `memory_stream`
  **antes** de persistir, tanto sync como async. Fallo del store no revierte
  buffer; reinicio no acredita reconstrucción. Frente a
  [CompositeScoring:65–144][v-scoring] existente se rechaza portar buffer,
  pesos e índice posicional. Sólo se propone una capa experimental de
  identidad/touch con ownership, permisos, fallo y efímero/durable explícitos.
  FMEA/aceptación 378–383, rechazo 432–435, matriz 481 y cierre 538–539
  no convierten esa capa en implementación.

**Límites:** no todas las versiones Core/bindings, cancelación por proveedor,
transacciones backend ni frontend SaaS auditados. Cerrar LC-1 no declara
conformidad universal, y cerrar LC-2 no adopta scoring ni nuevos parámetros.

## LangGraph

**Recibo:** [langgraph.md](../langgraph.md), cuerpo 1–574 y delta BASE;
pin `e5e37b5d70dbfd93c7006bd90a658c6179a95946`.
**Veredicto propuesto:** LG-1 cerrado como especificación documental futura;
autorización efectiva end-to-end continúa **no acreditada**, no vulnerabilidad
reproducida ni condición para aceptar automáticamente el informe.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 25–137: saver/checkpoint y BaseStore distintos; diferencias de adapter y versiones no se ocultan. |
| 1. Recomendaciones | sustentado | 27–29, 73–76: conformidad antes de extender; no Pregel en Rust. |
| 2. Fichas | sustentado | 139–342: A–F con ownership/errores/migración/FMEA; B/E/F incorporan explícitamente copia previa a revocación. |
| 3. Incidencias | sustentado | 357–383: #9563 SQLite y #8855 scans, no bug heredado por Vanta ni merge/fix inferido. |
| 4. Prerrequisitos | sustentado | 196–200, 235–245, 302–312, 385–392: host autoriza antes de decode y antes de resume/entrega/efecto. |
| 5. Rechazos | sustentado | 394–401: ni namespace como autenticación ni clases Python no confiables como memoria Rust. |
| 6. Método/licencia | sustentado | 403–427, 533–574: MIT por componente/terceros; fuente, muestra, lagunas y split separados de validación ejecutada. |
| 7. Benchmarks | sustentado | 429–474: pyperf, fake ReAct, durability exit, sin fsync/QA; guard objeto contra strings no acredita first-event medido. |
| 8. Matriz | sustentado | 476–494: TTL rechazado, index parcial, namespaces por segmento; permiso al resume explícitamente no acreditado end-to-end. |
| 9. Negativos | sustentado | 496–512: mecanismos del adapter y escenario LG-1, no pérdida/exploit ni ausencia de checkpoints. |
| 10. UI | sustentado | 514–531: producto web externo documentado, no frontend/auth auditados; Studio propuesto muestra denegación/redacción sin dispatch por inspección. |

### LG-1

**Cerrado documentalmente.** Contraste V:
[saver:89–118][v-lg-load], [put/put_writes/delete_thread:232–313][v-lg-write]
y [store:97–174][v-lg-store]. El saver serializa checkpoint completo;
eliminar un objeto de BaseStore opera sobre otra familia. Por ello un padre
retenido puede conservar una copia, pero estos cuerpos **no demuestran**
que un host real entregue datos revocados. La responsabilidad/política está
ahora explícita en B 196–200, E 302–312 y F 335–341: autorizar acceso al
checkpoint antes de decode y revalidar antes de resume, recall/nueva entrega
o efecto; denegación o redacción explícita, nunca autorización desde el LLM.
FMEA y aceptación futuras incluyen revocación/retirada con padre previo;
matriz 492, negativos 501–503 y UI 530–531 propagan el mismo alcance.
No se promete sanear backups ni convertir delete_thread en borrado universal.

**Límites:** ese caso es criterio funcional futuro, no exploit encontrado ni
test pasado. Tampoco se acreditan CAS de writes, compatibilidad moderna,
crash safety, codec seguro universal o auth de todos los transports.

## Agno

**Recibo:** [agno.md](../agno.md), cuerpo 1–653 y delta BASE;
pin `f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68`.
**Veredicto propuesto:** A-1 cerrado documentalmente, sin certificar
aislamiento REST/todos los drivers ni aprobar A–E.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 25–180: hechos, sesión, run y knowledge distintos; F3 ahora traza upsert sin lookup/ownership. |
| 1. Recomendaciones | sustentado | 80–99: B/E mapping/capacidades antes de herramientas destructivas; no AgentOS en Rust. |
| 2. Fichas | sustentado | 183–365, 510–558: A–E con contratos propuestos, alternativas, commits, FMEA y aceptación propia/ajena/inexistente. |
| 3. Incidencias | sustentado | 560–574: #11143/#9983, await y clear global acotados; no CVE/score adoptado ni PR como fix. |
| 4. Prerrequisitos | sustentado | 255–299, 382–390: RecallScope/InjectionPolicy/tenancy existentes; MemoryScope conceptual no tipo actual; CAS de grupo no prometido. |
| 5. Rechazos | sustentado | 392–399: no usuario global en servicio multiusuario, clear-then-upsert ni IAM paralelo. |
| 6. Método/licencia | sustentado | 401–420, 642–653: Apache-2.0 y terceros/muestra diferenciados; no todos los wrappers/auth auditados. |
| 7. Benchmarks | sustentado | 422–437, 576–592: 10 warmups, 1000 del entrypoint, tiempo/memoria en invocaciones distintas; excepción sin resultado agregado, GC no reset DB. |
| 8. Matriz | sustentado | 594–605: scope parcial por operación; WriteReceipt existente SHA256/relectura, no firma; MutationReceipt propuesto. |
| 9. Negativos Vanta | sustentado | 452–458, 642–650: lagunas de contrato, no bugs nuevos ni ausencia de híbrido/budgets/dedup. |
| 10. UI | sustentado | 460–467, 500–508, 629–640: HTTP local y SaaS externo separados; componentes Studio S reales, denegación UI propuesta. |

### A-1

**Cerrado documentalmente.** [manager.py:1386–1416][agno-update] construye
UserMemory y llama a upsert sin leer memoria existente ni comprobar propietario.
[sqlite.py:2699–2725][agno-upsert] usa conflicto por memory_id y set de
contenido/topics/input/agent/team/feedback sin condición user: user en INSERT
no autoriza UPDATE. F3 157–169 elimina la afirmación contraria y acota
la deducción a **DB compartida + ID ajeno conocido + camino directo sin wrapper
autorizado**. No es exploit REST, conclusión de todos los adapters ni semántica
de delete. Propagación verificada: B 217–231, E 356–364, matriz 600,
UI futura 639–640 y cierre 642–644. La aceptación propia/ajena/inexistente
y contraste con wrapper es futura, no resultado pasado.

**Límites:** no auditoría integral de wrappers/AgentOS/knowledge/providers,
consolidación crash-safe ni SaaS. No se recomienda patch upstream; se registra
la frontera para diseñar un adapter Vanta autorizado, si se aprueba.

## AutoGen

**Recibo:** [autogen.md](../autogen.md), cuerpo 1–545 y delta BASE;
pin `d07c667a3239ea4ab7af62a53b0f4ff205a1f23d`.
**Veredicto propuesto:** G-1/G-2 cerrados documentalmente; sin certificar
runtime, restore o cobertura de intentos externos a runlogs.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 25–64, 239–313: memoria/contexto/subscriptions separados; local/HTTP y terminal normal/error/cancel rectificados. |
| 1. Recomendaciones | sustentado | 27–40, 315–321: adapter pequeño condicionado a soporte histórico, no sucesor intercambiado por AutoGen. |
| 2. Fichas | sustentado | 66–130, 323–414: A–E, contratos propuestos, lifecycle, preflight, FMEA y aceptación futura; no backup distribuido prometido. |
| 3. Incidencias | sustentado | 416–429: datetime/MIME históricos contrastados con mecanismos al pin, no fix universal ni CVE. |
| 4. Prerrequisitos | sustentado | 149–156, 337–359: MIME/scope/ownership y recibos existentes; no presupuestos duplicados. |
| 5. Rechazos | sustentado | 158–165: no ListMemory de producción, ejecutores Rust, trimming causal no validado o pickle no confiable. |
| 6. Método/licencia | sustentado | 167–190, 258–269, 536–545: código MIT/documentación CC, muestra y omisiones; sin equivalencia automática de SaaS/deps. |
| 7. Benchmarks | sustentado | 192–207, 431–472: AgBench Docker/nativo, fórmula intacta y Missing sólo de población descubierta, no censo JSONL. |
| 8. Matriz | sustentado | 499–508: híbrido/quotas/context/dedup/WriteReceipt existen; adapter Memory no acreditado, no ausencia global. |
| 9. Negativos Vanta | sustentado | 510–516: lagunas de mapping/export/reinyección, no bugs demostrados; prefijo protegido no garantiza caber. |
| 10. UI | sustentado | 228–235, 474–497: React/Gatsby/REST/WS reales, auth condicional; UI propuesta no lee cleanup como commit. |

### G-1 y G-2

- **G-1 — cerrado documentalmente.** F1 277–290 ahora separa
  [PersistentClient/HttpClient:246–262][ag-chroma] y
  [terminación:543–578][ag-terminal]: RuntimeError por error ocurre **antes**
  del yield TaskResult; finally limpia cola/estado también en error.
  Cancelación no promete TaskResult ni undo. FMEA/aceptación 340–354
  conservan OutcomeUnknown y parcialidad; UI 495–497 y cierre 536–538
  mantienen normal/error/cancel separados. No se adjudica commit remoto a
  un backend local ni atomicidad a cleanup.
- **G-2 — cerrado documentalmente.** 449–462 contrasta
  [tabulate_cmd.py:127–191,214–261][ag-tabulate]: filas desde directorios
  existentes, padding None sólo para filas presentes; sin directorio no hay
  tarea en la tabla. Missing/Total no son censo autorizado externo.
  Fórmula successes/(successes+failures) y media temporal sobre no-null
  preservadas. Inventario/reconciliación es aceptación documental futura,
  no harness/denominador nuevo ni resultado propio; no invalida globalmente GAIA.

**Límites:** .NET, todos los templates, auth integral, subscriptions/restores
atómicos, stores externos y datos descargados no acreditados. Leer el scorer
no prueba aislamiento de gold en cada escenario.

## Flowise

**Recibo:** [flowise.md](../flowise.md), cuerpo 1–889 y delta BASE;
pin `9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb`.
**Veredicto propuesto:** F-1/F-2 cerrados documentalmente; no se corrige
código/scoring ni se certifica el juez LLM, el runtime o la seguridad enterprise.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 26–141, 556–613: definición/Execution/mensajes/state/proveedor separados; cola espera trabajo, no ACK listo. |
| 1. Recomendaciones | sustentado | 143–182: referencia archivada, no dependencia estratégica; identidad/outcome/config antes de nuevas funciones. |
| 2. Fichas | sustentado | 184–416, 615–708: A–E con once campos, errores, FMEA y aceptación futura; tipos no implementados. |
| 3. Incidencias | sustentado | 710–726: #4968 configuración descartada como bug; #4433 histórico frente a persistState opt-in al pin; límites de conversación explícitos. |
| 4. Prerrequisitos | sustentado | 443–452, 641–668: RRF/tenancy/checkpoints/WriteReceipt existentes, no IAM/WAL/scheduler nuevo. |
| 5. Rechazos | sustentado | 454–463: no seenScores/canvas/comercial/LLM para merge ni status UPSERTED como transacción global. |
| 6. Método/licencia | sustentado | 465–493, 606–613, 872–889: reparto Apache/enterprise/terceros, archivo, muestra y omisiones. |
| 7. Benchmarks | sustentado | 495–519, 728–792: simple vs LLM, intervalo cliente, agregado fila×flow/filas y fallos corregidos sin normalización nueva. |
| 8. Matriz | sustentado | 521–531, 821–834: capacidades V preservadas, deltas por fuente/binding/recibo; sin falsa ausencia de híbrido o budgets. |
| 9. Negativos Vanta | sustentado | 836–844: RMW de checkpoint condicionado a caller, no carrera efectiva declarada ni bug heredado de 2025. |
| 10. UI | sustentado | 542–552, 794–819: canvas serializa definición, no ACID; lentes Studio S, propuesta de unidades/outcomes no UI certificada. |

### F-1 y F-2

- **F-1 — cerrado documentalmente.** §7 y 759–790 separan
  [contadores simples:270–325][f-counters] de `newRun.llmEvaluators`.
  [EvaluatorRunner:13–51][f-simple] recorre outputs de flows y evaluadores
  simples seleccionados: población **fila×flow×evaluador simple emitido**.
  Error de prediction puede generar Error por evaluador; fallo global puede
  impedir agregado. pass/(pass+fail+error)*100 queda intacto, -1 si no hay
  casos simples aunque sí LLM. No es score de juez/QA homogénea. Aceptación
  documental/UI futura 786–790 mantiene -1 distinto de cero.
- **F-2 — cerrado documentalmente.** [EvaluationRunner:135–180][f-clock]
  inicia reloj **antes de POST** y termina **después** de métricas Agentflow
  y awaits getCostMetrics; intervalo cliente en ms/string, no sólo HTTP.
  [servicio:196–202][f-sum] suma **todos** los flows por fila, incluidos
  runData con error; [321–325][f-counters] divide por `result.rows.length`
  y totalRuns es filas. Así averageLatency es suma media por fila, **no
  media por request**, percentil o coste exclusivo de motor. 745–758 y
  786–790 propagan fórmula/unidad/fallos a aceptación e interpretación UI,
  sin alterar criterio, denominador, scoring ni harness.

**Límites:** juez/prompts completos, auth comercial/global, todos los nodos/
drivers y ejecución visual no auditados. FMEA de A–E sigue futura y no cambia
por cerrar dos errores de descripción de métricas; no hay medición propia.

## LightRAG

**Recibo:** [light-rag.md](../light-rag.md), cuerpo 1–647 y delta BASE;
pin `aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a`.
**Veredicto propuesto:** LR-1 cerrado documentalmente; jobs/cancel/progreso
atribuibles a Studio al pin siguen no acreditados, no “Studio ausente”.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 26–74, 284–324, 492–526: almacenes/proyecciones separados y multimodalidad real; global=relaciones, no comunidades Leiden. |
| 1. Recomendaciones | sustentado | 28, 61, 326–330: enriquecimiento opt-in, antes linaje/retirada; no reemplazar retrieval actual. |
| 2. Fichas | sustentado | A–D 76–116/332–371, E/F 373–438, G 528–566: contratos propuestos y once campos/FMEA/aceptación futura. |
| 3. Incidencias | sustentado | 131–154, 483–490: #3659 RFC, #3978 tabla histórica y actividad truncada; no merge/fix universal inferido. |
| 4. Prerrequisitos | sustentado | 156–162, 192–200: RecallScope/InjectionPolicy y linaje existentes, permisos transitivos y presupuesto compartido. |
| 5. Rechazos | sustentado | 164–170: nombres/rutas/pesos no identidad/confianza; sin nuevos cuatro stores, RRF ni assembler. |
| 6. Método/licencia | sustentado | 172–200, 631–639: MIT por archivo, parsers/modelos/deps aparte; muestra, omisiones y futura separación no implementación. |
| 7. Benchmarks | sustentado | 202–252: RAGAS denominador por métrica/no NaN y éxito bool(metrics); offline lexical no ejecuta grafo, no score propio. |
| 8. Matriz | sustentado | 254–265, 445–453, 596–606: capacidades V existentes, cobertura integral parcial; UI LS1, jobs no acreditados al pin. |
| 9. Negativos Vanta | sustentado | 267–269: comprobar linaje/identidad/degradación, no ausencia global ni bug nuevo; WriteReceipt SHA256 sin firma 590–594. |
| 10. UI | sustentado | 608–629: API web cancel-request ≠ undo; MemoryLens S real, JobChip/jobs retirados como evidencia shipped. |

### LR-1

**Cerrado documentalmente.** Reconfirmado mediante `git ls-tree` del pin
Studio S: existe [MemoryLens:57–412][s-memory], no JobChip/jobs en las rutas
citadas ni sus equivalentes feature-freez. En worktree separado los dos
archivos continúan **`??`**; no se usan sus cuerpos como fuente inmutable.
614–625 corrige procedencia/rangos y deja cancelación entre chunks, IDs
parciales y registro durable como propuesta. Propagación: matriz 606,
FMEA/aceptación 626–629 y límites 635–637; no sólo cambio de enlace.
[cancelPipeline:1132–1145][lr-cancel] devuelve solicitud/not_busy, no rollback.
MemoryLens tiene escenas/persona/skills/genlog y get(anchorId), no ledger
de entrega LLM ni prueba de estado durable de jobs.

**Límites:** parsers completos, todos los roles/backends, crash safety y auth
end-to-end no certificados. Runners leídos no ejecutados; fallback y números
RAGAS no se convierten en éxito de memoria longitudinal.

## PageIndex

**Recibo:** [pageindex.md](../pageindex.md), cuerpo 1–508 y delta BASE;
pin `2ac8694bf8fe89a3840dda9bdb88db5119b77fc4`.
**Veredicto propuesto:** PI-1 cerrado documentalmente; spans/publicación
end-to-end y jobs UI no certificados.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 25–61, 281–336: local/cloud, dos parsers, árbol/páginas/meta, chat y delete separados; local no significa offline. |
| 1. Recomendaciones | sustentado | 27–29, 246–250: intervalos/generación como delta condicionado, no vectorless contra híbrido. |
| 2. Fichas | sustentado | A/B 79–141/338–358, C/D 360–429: contratos propuestos, errores/FMEA/migración/aceptación, sin deadline remoto ya acreditado. |
| 3. Incidencias | sustentado | 431–444: cinco reportes, cache anormal vs writer normal, lock POSIX, path no desplegado y exploit #585 descartado con evidencia actual. |
| 4. Prerrequisitos | sustentado | 164–171, 344–358: inventario previo, backend real y WriteReceipt existente sin firma, no transacción multiarchivo asumida. |
| 5. Rechazos | sustentado | 173–180: ni ordinal como ID ni JSON como WAL, nuevos prompts o loop de agente en core. |
| 6. Método/licencia | sustentado | 182–195, 289–295, 491–498: MIT/modelos/PDFs aparte, SDK checkout vs notebook discrepantes, lectura no corrida. |
| 7. Benchmarks | sustentado | 197–229, 446–461: notebook NVIDIA no FinanceBench claim; accuracy sobre preguntas y coste sobre respuestas, cache/config/errores explícitos. |
| 8. Matriz | sustentado | 231–242, 463–470: V2 vigente al pin, híbrido/budgets/checkpoints preservados; PS1 real, jobs no acreditados. |
| 9. Negativos Vanta | sustentado | 244–250, 487–489: spans/generaciones sin cobertura global, no ausencia de provenance/receipts ni bug nuevo. |
| 10. UI | sustentado | 252–259, 472–486: SaaS anunciado no frontend leído; MemoryLens S real, árbol/spans/generación y cancel propuestos. |

### PI-1

**Cerrado documentalmente.** Mismo contraste de árbol/status Studio S de LR-1:
[MemoryLens:57–412][s-memory] versionado bajo feature-freez; JobChip/jobs
sólo `??` bajo desktop del worktree, no prueba inmutable. 470–486 retira
PS2/PS3 y garantía de cancelación entre chunks, preserva Studio real,
añade FMEA cancel visual ≠ retirada y aceptación futura de parcialidad/restart.
497–498 conserva límite de procedencia. Cambio adicional 238 retira
“V2 histórico” porque su URL ya fija V=97c9623; no nuevo contexto engine.

**Límites:** exactitud global de spans, coherencia de todos los derivados,
algoritmos/parsers completos, todas las lanes y auth SaaS no certificados.
FinanceBench sigue claim externo; notebook y outputs guardados no son resultado
propio ni baseline NumPy equivalente al híbrido actual.

## Serviette

**Recibo:** [serviette.md](../serviette.md), cuerpo 1–491 y delta BASE;
pin `84e92a33728cf843026f4011d86568b403e38fb4`.
**Veredicto propuesto:** SE-1 cerrado documentalmente, no certificación
de retraction de todos los sinks ni durabilidad del registro UI.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 26–72, 212–271: dataflow/memoización/retract, procesos y sinks separados; frescura documental no memoria de agente. |
| 1. Recomendaciones | sustentado | 28–31, 445–446: fingerprint/manifest antes de replay; C paralelo al worker actual, no Pathway/LLM obligatorios. |
| 2. Fichas | sustentado | A/B 74–114/273–286, C/D 288–358: once campos/FMEA/aceptación futura; C ya no depende de JobChip shipped. |
| 3. Incidencias | sustentado | 362–382: #53/#46 de 2023 explícitamente históricos; parse→vacío es código actual sin issue específico localizado, no CVE. |
| 4. Prerrequisitos | sustentado | 131–137, 277–286: fingerprint/revisión/manifest, semántica backend y budgets existentes, no replay inferido de logs. |
| 5. Rechazos | sustentado | 139–145: no Pathway/core obligatorio, BM25 RAM duplicado o catálogo como autobiografía. |
| 6. Método/licencia | sustentado | 147–154, 472–479: MIT wrapper ≠ Pathway xpack/extras/modelos; no todos los accessors/auth/runtime auditados. |
| 7. Benchmarks | sustentado | 156–176, 384–422: FRAMES judged, gold vacío=1, pool puede abortar; realtime reset destructivo/PSS sólo indexer, no HNSW complete ni coste cero. |
| 8. Matriz | sustentado | 178–188, 424–440: híbrido/linaje/budgets/WriteReceipt existen; SS1 real, jobs al pin no acreditados. |
| 9. Negativos Vanta | sustentado | 190–195, 441–444: fingerprint/retraction integral no acreditados ≠ ausentes; no bug nuevo ni conector 2023 trasladado. |
| 10. UI | sustentado | 197–203, 448–470: HTML same-origin, sin AbortController en llamada leída; MemoryLens S y cancel/progreso sólo propuestos. |

### SE-1

**Cerrado documentalmente.** Árbol/status Studio S reconfirmados como en LR-1:
[MemoryLens:57–412][s-memory] existe bajo feature-freez; JobChip/jobs sólo
`??` en desktop del worktree, sin bytes atribuibles al pin. Corrección
transversal comprobada en C 295–308 (ya no seam JobChip implementado),
matriz 435, UI 457–470 y cierre 479. FMEA/aceptación C 315–325 y UI
468–470 mantienen cancel solicitado ≠ rollback, error ≠ vacío y
restart/parcialidad pendientes, no garantías dirty trasladadas al SHA.

**Límites:** no ejecución de FRAMES/realtime/compose ni prueba de todos los
accessors, persistencia Pathway, fetchers/SSRF, permisos o licencias transitivas.
La presencia de código incremental no certifica transacción global o entrega
factual del answerer.

## Meilisearch

**Recibo:** [meilisearch.md](../meilisearch.md), cuerpo 1–661 y delta BASE;
pin `c18c81973ad5d3e126f4583cdd16a9ec1719aa4a`.
**Veredicto propuesto:** ME-1/ME-2 cerrados documentalmente; E3 **no
acreditado por prohibición expresa**, sin bloqueo del resto ni acceso a forge.
No se modifica ni se recomienda modificar el proyecto upstream.

### Ledger 0 + 10 actualizado

| Eje | Estado | Evidencia actual y límite |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | 27–126, 334–363: task/document/batch y commits queue/index/stats separados; filtro/score no ACL/probabilidad. |
| 1. Recomendaciones | sustentado | 128–150, 595–597: trazabilidad/explicación acotadas a Vanta, no motor Meili ni ratios como RRF. |
| 2. Fichas | sustentado | A–C 166–221/365–425 y D 427–462: once campos/FMEA/aceptación futura, incluyendo fases/modelo ME-2; tipos propuestos. |
| 3. Incidencias | no acreditado por restricción | 466–485, 641–646: listado histórico no cuerpos; no forge, mirror/bypass o solicitud de completar lo prohibido. |
| 4. Prerrequisitos | sustentado | 241–249, 371–409: worker/callback/checkpoints/WriteReceipt existentes, no ledger UI durable desde job/ACK. |
| 5. Rechazos | sustentado | 251–256: no ranking editorial, colas duplicadas, filtro como permiso o succeeded como fsync universal. |
| 6. Método/licencia | sustentado | 258–273, 336–343, 638–646: MIT AND BUSL/EE y dependencias aparte, lectura estática limitada, no seguridad universal. |
| 7. Benchmarks | sustentado | 495–575: movies y dos workloads embedding, precommands fuera de captura, runs solicitados/completados y fallos/reset/caches explícitos. |
| 8. Matriz | sustentado | 577–588: V híbrido/filtros/budgets/recibos existen; settings-add/ingest distintos; Studio MS1 no jobs al pin. |
| 9. Negativos Vanta | sustentado | 590–597: uniformidad por binding no acreditada ≠ ausencia; no bug nuevo reproducido/duplicación de WAL. |
| 10. UI | parcial | 599–636: launcher CDN tag v0.4.2 no commit resuelto/zip equivalente, Studio MemoryLens S real; jobs sólo propuestos. |

### ME-1 y ME-2

- **ME-1 — cerrado documentalmente.** 588 y 624–646, árbol/status Studio S
  reconfirmados: [MemoryLens:57–412][s-memory] versionado en feature-freez;
  JobChip/jobs sólo `??` del worktree. MS2/MS3 y supuestas garantías de
  cancel/repoll/IDs parciales retiradas. FMEA/aceptación 633–636 separa
  task succeeded/cancel visual de publicación/undo y conserva restart
  y parcialidad como futuro, no comportamiento shipped.
- **ME-2 — cerrado documentalmente.** Lectura local al pin de
  [settings-add:1–65][me-settings] y [ingest-hf:1–61][me-ingest]:
  **5 runs/4 threads**, mismo asset SHA256
  `d215e395e4240f12f03b8f1f68901eac82d9e7ded5b462cbf4a6b8efde76c6c6`.
  [runner:40–85][me-phases] ejecuta preparación antes de start_report.
  Settings-add mide añadir embedder a documentos existentes; ingest-hf mide
  insertar con embedder preparado. Default declarado
  [hf.rs:60–75][me-hf] no sustituye identidad efectiva: [settings:2126–2150][me-resolve]
  resetea revision si model está Set. 514–538 conserva modelo/bytes/cache/red/
  CPU-GPU desconocidos, población y errores; no cinco preguntas ni pago cero.
  D FMEA/aceptación 453–459 y matriz 587 propagan esa distinción. No se mide
  ni se cambia runner/scoring/configuración.

**Límites:** E3 de **incidencias** no es la referencia E3 de `hybrid.rs`;
ésta sí es lectura de código, no permiso de consultar issues. Sin forge
en esta reconfirmación. Recovery completo, EE, auth/proveedores y frontend
completo no certificados; bytes de modelo/asset no descargados.

## Disposición conjunta y límites

### Diecisiete disposiciones históricas y FMEA de la corrección

La disposición **propuesta**, en todos los casos, corresponde al error/laguna
documental identificado en las reviews, **no** a aceptar la candidata ni a
implementar su mitigación. Las secciones anteriores dan informe/líneas,
SHA/rango/URL, propagación a FMEA/aceptación/matriz/UI y límite por proyecto.
Esta tabla conserva además disparador, mecanismo, consecuencia, detección y
mitigación de cada hallazgo; no añade requisitos, scoring, backlog ni IDs nuevos.

| Finding / disposición propuesta | Disparador → mecanismo → consecuencia | Detección → mitigación documental y resto no certificado |
|---|---|---|
| DIFY-1 — cerrado documentalmente | Ingesta/schema descritos desde owner equivocado → atribuir fencing/validación a otra capa → diseño o promesa de atomicidad incorrectos | Worker→callback→put_page y conversores/put_batch → C/D ahora localizan owner y commit real; revisión esperada/schema siguen propuestas, no CAS global. |
| DIFY-2 — cerrado documentalmente | Usar issue de metadata para cache → mecanismo no corresponde → recomendación E sin evidencia discriminante | Contrastar cuerpo/ruta de metadata frente a CacheEmbedding → limitar issues a D, sostener E con código propio; PR/fix universal no certificados. |
| LF-1 — cerrado documentalmente | Referencias/§11 ausentes → no se puede seguir claim a código → ledger/UI sin trazabilidad | Resolver N/V/VS y leer modal/página/backend → inventario/cierre materializados, polling limitado declarado; no censo global ni durabilidad desde UI. |
| LF-2 — cerrado documentalmente | Omitir defaults/población/self-match → latencia sintética tomada por QA/percentil → comparación inválida | Leer runner completo y reloj/assert → explicitar primera query, máximo, abort y adaptación a APIs actuales; ninguna corrida ni equivalencia de ranking. |
| LC-1 — cerrado documentalmente | Editar/reintentar sin ID estable o fallar a mitad → puts/deletes por entrada → identidad distinta o prefijo aplicado | Seguir Core/overrides/keys → documentar IDs, validación, parcialidad y retry/cancel; conformidad de versión/backend no certificada. |
| LC-2 — cerrado documentalmente | Portar touch/buffer como memoria durable → modifica RAM antes del write → divergencia buffer/store y falsa recuperación | Orden de memory_stream/last_accessed_at frente al store → rechazar buffer/pesos, reconocer scoring V y limitar touch a propuesta autorizada. |
| LG-1 — cerrado documentalmente | Revocar/retirar con checkpoint previo → copia retenida independiente del store → posible reinyección no autorizada según host | Saver vs BaseStore → caso futuro en B/E/F exige autorización/revalidación y denegación/redacción; no fuga reproducida ni borrado universal. |
| A-1 — cerrado documentalmente | DB compartida + ID ajeno conocido sin wrapper → conflicto sólo por ID → update puede mutar fila ajena | Closure UserMemory y SQL ON CONFLICT → sustituir lookup/ownership ficticios y propagar casos propio/ajeno/inexistente; no conclusión REST/todos los drivers. |
| G-1 — cerrado documentalmente | Inferir remoto/éxito terminal desde stream → topología omitida y RuntimeError previo → promesa falsa de commit/TaskResult | Config Chroma y yield/finally → separar local/HTTP y normal/error/cancel en flujo/UI; cleanup no es rollback. |
| G-2 — cerrado documentalmente | Intento sin directorio → tabulador nunca crea fila → Missing/Total aparentan censo completo | Seguir os.listdir/padding/agregado → universo descubierto explícito, reconciliación futura sin cambiar fórmula; cobertura externa no acreditada. |
| F-1 — cerrado documentalmente | Mezclar jueces LLM con counters simples → denominar todos como passPcnt → score/población falsos | Dos ramas de servicio y EvaluatorRunner → simples emitidos separados de LLM, -1 sin simples; juez completo y fallos globales no homologados. |
| F-2 — cerrado documentalmente | Rotular reloj como HTTP/media request → awaits incluidos y suma flows/filas → atribución/normalización erróneas | Reloj antes POST/después metrics y fórmula totalTime/rows → declarar intervalo cliente/suma media por fila; sin percentil o normalización nueva. |
| LR-1 — cerrado documentalmente | Atribuir archivos ?? a S → jobs no recuperables por SHA → falsa UI shipped/cancel durable | Árbol S y status del worktree → retirar garantías/enlaces dirty, preservar MemoryLens y aceptación futura de parcialidad/restart. |
| PI-1 — cerrado documentalmente | Cancel visual leído como retirada/publicación → jobs ?? atribuidos al pin → citas/generación parecen vigentes sin evidencia | Árbol S y matriz/UI → PS2/PS3 retirados, spans/generación/cancel como propuestas; no certificado de publicación integral. |
| SE-1 — cerrado documentalmente | C depende de JobChip “existente” → confundir solicitud/resultado/ready → parse fallido o commit parcial ocultos | C/matriz/UI y árbol S → seam worker/checkpoint real, presentación futura, ParseFailed distinto de vacío; jobs durables no acreditados. |
| ME-1 — cerrado documentalmente | Task succeeded/cancel visual y jobs ?? → UI inferida como commit/undo → resultado operativo mal representado | Árbol S/MS1 y matriz/UI → retirar MS2/MS3, outcome desde owner y casos futuros antes/después apply; frontend CDN sigue a tag, no SHA/zip equivalente. |
| ME-2 — cerrado documentalmente | Confundir preparación de embedder con ingesta medida → descarga/cómputo en fase distinta → coste/latencia/modelo atribuidos erróneamente | Dos JSON + precommands/start_report + defaults/resolver → separar workloads, runs/asset y modelo efectivo desconocido; caches/red/CPU-GPU y fallos explícitos, nada medido. |

**Resultado propuesto de las 17 disposiciones:** 17 cierres **documentales**,
sin contradicción histórica sustantiva pendiente localizada en esta muestra;
0 cierres funcionales/aceptaciones de implementación. Los límites indicados
no se eliminan ni se convierten en capacidades existentes.
No se fuerza un “residual” artificial para algo ya rectificado, ni se usa
cerrado para ocultar lo que sigue parcial/no acreditado en las matrices.

**Excepción E3 de Meilisearch:** no acreditado por prohibición expresa,
fuera de esos 17 findings; no se reabre forge ni se exige suplir cuerpos por
mirrors, listados o títulos. No impide proponer cierre de ME-1/ME-2 ni del
resto del recibo. Su eje 3 continúa no acreditado y la UI externa a tag, parcial.

### Verificación estática final y límites operativos

- Once cuerpos completos, **6970 líneas**, contrastados con BASE y diff real;
  no revisión limitada a párrafos añadidos. Once ledgers 0+10 explícitos,
  **121 celdas** de cobertura documental, no puntuación de productos.
- Comprobación textual acotada a este recibo: once secuencias 0–10 sin
  duplicados, 17 disposiciones únicas, 36 definiciones de referencia sin
  referencias usadas indefinidas, enlaces relativos existentes y anchors
  internos resolubles. No es un gate global de documentación ni render probado.
- Recalculados SHA256 de las once BASE: todos coinciden con el recibo de
  autoría. Regeneración **en memoria**, por `git diff --no-index --no-ext-diff
  --text` de las once parejas: idéntica línea a línea al diff congelado,
  **1169 líneas**, SHA256 `638858E6A1E366C56E0DA2D4E631BDE443CF9B6992F8988E5081614B860A9EAB`.
  Exit 1 de esas llamadas significa diferencias BASE→informe, no fallo.
  Los once informes siguen correspondiendo a esa entrega; no se editaron.
- Sólo se escribió este recibo mediante patches absolutos menores de 300
  líneas. No se modificó proposedTOC, informes, contratos, código, metadatos,
  runners ni configuración. Cambios concurrentes ajenos permanecen intactos.
  Este recibo está ignorado por `.gitignore:268`; no se cambió la regla ni se
  forzó staging. Git utilizado sólo para lectura/diff; sin commit/push.
- Contrastes discriminantes estáticos, no auditoría exhaustiva de repos,
  licencias transitivas, proveedores, auth, crash safety ni UIs completas.
  “Sustentado” evalúa la corrección documental **dentro de esa cobertura**,
  no elimina los parciales/no acreditados de capacidades del informe.
  Fechas/estados de incidencias conservan la observación histórica declarada;
  no se presentan como un censo o una nueva consulta actual de todo el forge.
- Casos normales/adversos son criterios futuros por seam/candidata, no
  resultados ejecutados ni exigencia de runtime universal para este cierre.
  No apps/tests/builds/benchmarks/instalaciones/servicios, descargas de
  modelos/assets, gates globales de docs ni arranques/paradas.
- Se mantienen BM25/HNSW/RRF, sesiones/provenance/dedup/supersession,
  compactación/presupuesto compartido, personas/escenas/skills, scheduler/
  checkpoints, wiki/ingestors, auditoría, auth fuera de loopback y Studio
  existentes. No se duplican motor, WAL, IAM, índices o dashboard.
  WriteReceipt conserva SHA256/relectura, sin firma; MutationReceipt,
  ProjectionReceipt y DTOs nuevos siguen propuestas.

**Decisión reservada al responsable:** aceptar, corregir o conservar
residuales de estos veredictos propuestos. Este recibo no otorga aceptación
final ni autorización de implementación, ejecución o cambios upstream.

[me-settings]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/workloads/embeddings-settings-add.json#L1-L65
[me-ingest]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/workloads/embeddings-movies-subset-hf.json#L1-L61
[me-phases]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/xtask/src/bench/workload.rs#L40-L85
[me-hf]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/vector/embedder/hf.rs#L60-L75
[me-resolve]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/vector/settings.rs#L2126-L2150

[s-memory]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412
[lr-cancel]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag_webui/src/api/lightrag.ts#L1132-L1145

[f-clock]: https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/evaluation/EvaluationRunner.ts#L135-L180
[f-sum]: https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L196-L202
[f-counters]: https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L270-L325
[f-simple]: https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/EvaluatorRunner.ts#L13-L51

[ag-chroma]: https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py#L246-L262
[ag-terminal]: https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_base_group_chat.py#L543-L578
[ag-tabulate]: https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/tabulate_cmd.py#L127-L261

[agno-update]: https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/memory/manager.py#L1386-L1416
[agno-upsert]: https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/sqlite/sqlite.py#L2699-L2725

[v-lg-load]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L89-L118
[v-lg-write]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L232-L313
[v-lg-store]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/store.py#L97-L174

[lc-add]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/vectorstores/base.py#L234-L291
[lc-time-buffer]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain/langchain_classic/retrievers/time_weighted_retriever.py#L110-L198
[v-lc-key]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L84-L85
[v-lc-mutations]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L414-L495
[v-scoring]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/scoring.rs#L65-L144

[lf-modal]: https://github.com/langflow-ai/langflow/blob/60999cc9a7350e44dde3ef641baae864e10861f0/src/frontend/src/pages/MainPage/pages/knowledgePage/components/IngestionRunDetailModal.tsx#L14-L222
[lf-runs]: https://github.com/langflow-ai/langflow/blob/60999cc9a7350e44dde3ef641baae864e10861f0/src/frontend/src/pages/MainPage/pages/knowledgePage/components/IngestionRunsSection.tsx#L15-L86
[lf-operation]: https://github.com/langflow-ai/langflow/blob/60999cc9a7350e44dde3ef641baae864e10861f0/src/backend/base/langflow/services/knowledge_base_storage/runtime.py#L282-L316
[lf-cursor]: https://github.com/langflow-ai/langflow/blob/60999cc9a7350e44dde3ef641baae864e10861f0/src/backend/base/langflow/services/memory_base/task.py#L838-L879
[lf-runner]: https://github.com/langflow-ai/langflow/blob/60999cc9a7350e44dde3ef641baae864e10861f0/scripts/benchmark/sqlite_kb.py#L1-L114

[v-worker]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L244-L309
[v-merge]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L34-L49
[v-page]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L287-L320
[v-wiki-terminal]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L162-L217
[v-validate]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/mod.rs#L252-L266
[v-batch]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1280-L1350
[dify-cache]: https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/embedding/cached_embedding.py#L24-L133
