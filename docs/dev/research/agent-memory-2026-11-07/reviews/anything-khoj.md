---
title: "Revisión independiente — AnythingLLM y Khoj"
kind: research
status: proposed
description: "Contraste adversarial estático por proyecto, con ledger de cobertura, recibos de fuentes y veredictos propuestos sin aceptación final."
tags: [research, memory, engineering]
---

# Revisión independiente — AnythingLLM y Khoj

## Índice

- [Alcance y contratos](#alcance-y-contratos)
- [AnythingLLM](#anythingllm)
  - [Ledger AnythingLLM](#ledger-0--diez-ejes)
  - [Hallazgos AnythingLLM](#hallazgos-y-corrección-verificable)
  - [Fuentes públicas AnythingLLM](#recibos-públicos-persistentes)
- [Khoj](#khoj)
  - [Ledger Khoj](#ledger-khoj-0--diez-ejes)
  - [Hallazgos Khoj](#hallazgos-khoj-ordenados-por-gravedad)
  - [Fuentes públicas Khoj](#recibos-públicos-khoj)
  - [Veredicto Khoj](#lagunas-y-veredicto-khoj)

## Alcance y contratos

Revisión independiente de autoría, realizada el 2026-11-07. Único archivo
modificado: esta revisión. No se editaron informes, código, configuración,
harnesses, prompts, scoring ni registros de aceptación.
Contratos leídos: [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md),
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md) y
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md), además del brief temporal asignado.
El snapshot Vanta comprobado con `git rev-parse HEAD` es
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; Studio:
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`.

Lectura estática exclusivamente: no ejecución de aplicaciones, tests, builds,
benchmarks, instalaciones ni servicios; ningún runtime iniciado/detenido.
No staging, commit, push, fetch ni cambios Git/configuración.
Se cargaron `documentation-skill` y `coordinated-web-search`, se consultó el
catálogo de skills y se listaron servidores/herramientas MCP antes de la red.
Los recibos siguientes preservan contenido comprobado, no solo IDs de herramienta.

## AnythingLLM

### Recibo de lectura e identidad

Informe [anything-llm.md](../anything-llm.md), **629 líneas leídas completas**,
incluidas fichas A–G, cobertura y definiciones de enlaces.
Clon `C:\Users\Eros\anything-llm`; HEAD comprobado:
`308b27662b2f6fd4947791d61c797945c609fcbc` (en adelante SHA A).
`package.json` confirma 1.19.0 y MIT; `LICENSE` leído confirma MIT.
`git ls-tree HEAD embed browser-extension` confirma gitlinks
`a3a2fea57cfc753698110dd0973a607e3209f634` y
`385d36c0807221e6674d1325ef06505746c49ceb`: no código local inspeccionado.

### Ledger 0 + diez ejes

Sin puntuación agregada. «Sustentado» se limita al alcance documental contrastado.

| Eje | Estado | Sección y evidencia / límite |
|---|---|---|
| 0. Arquitectura y tres flujos | sustentado | §1: API→worker→document→Lance, consulta→contexto/citas, refresh/delete; commits separados y compensación limitada explícitos. Recibos locales abajo. |
| 1. Recomendaciones | sustentado | §2: A–G priorizados; no otro motor, collector o scheduler. Distingue aceptación de disponibilidad y citas de entrega. |
| 2. Fichas Rust | sustentado | §3 A–G: los once campos del contrato están presentes, aunque agrupados; contratos etiquetados propuestos, alternativas, seam/delta, migración/rollback, concurrencia, FMEA y aceptación futura. No implementados. |
| 3. Incidencias upstream | sustentado | §4: #3958 y #6806 releídos completos con discusión/estado. Cierre no se convierte en fix universal; d1d68af y #6807 no auditados. |
| 4. Prerrequisitos Vanta | sustentado | §5 y A–G: IngestReport/progreso/run_id, scheduler, checkpoint, L1 y contexto ya existen; auditoría de transportes condicionada, no carencia global. |
| 5. Rechazos | sustentado | §6: producto completo, delete-before-success, bypass single-user, shell/gitlinks no leídos; costes y alternativas también en fichas. |
| 6. Advertencias | sustentado | §7: shallow, estático, dos issues no exhaustivos; documentación Desktop separada del código. |
| 7. Benchmarks | sustentado | §8: Jest/config/test mock, unidad caso, no QA/recall/juez, efectos de scripts destructivos. Adaptación documental, no suite nueva. Nota menor de rango abajo. |
| 8. Matriz | sustentado | §9: capacidades Vanta existentes preservadas y deltas condicionados; no atribuye RRF a toda ruta recall. |
| 9. Negativos Vanta | sustentado | §10: preguntas discriminantes y no defectos por analogía. `core_search=false` confirmado en auto_recall.rs:325-335. |
| 10. UI/escritorio | sustentado | §11: SPA real, Electron como producto/compatibilidad, shell no leído; staging separado de mutación; Studio existente con filtros visuales. |

### Hallazgos y corrección verificable

**Baja — precisión del anclaje de fuentes, sin contradicción de comportamiento.**
Informe `anything-llm.md:245,482-489,595` cita el test hasta L158, pero
`server/__tests__/jobs/sync-watched-documents.test.js:155-164` contiene el segundo
run y termina la comprobación de estado `"exited"` después de ese límite.
`anything-llm.md:400,544-546` da el hook como ruta abreviada sin enlace SHA propio.
La implementación sí se localizó en
`frontend/src/components/Modals/ManageWorkspace/Documents/hooks/useDocumentPicker.js:643-689`;
la abreviatura no acredita por sí sola la ruta completa.
**Consecuencia:** el lector debe reconstruir un enlace o ampliar el rango para
repetir el contraste; no invalida la conclusión.
**Corrección verificable:** ampliar `[sync-test]` hasta L164 y enlazar el hook
completo al SHA A con L643-L689. No editar lógica, runner ni el informe desde
esta revisión.

No se identificaron hallazgos bloqueantes en las afirmaciones contrastadas.
La compensación de `Document.addDocuments` no debe ampliarse interpretativamente:
solo se intenta en el fallo de crear la fila después de vectorizar; no revierte
cualquier fallo posible de `Lance.addDocumentToNamespace`. El informe ya limita
su garantía a esa rama y explicita confirmaciones separadas.

### Evidencia local contrastada

Rutas relativas al clon SHA A; los símbolos y rangos siguientes fueron leídos:

- `server/endpoints/workspaces.js:234-294`: deletes primero; nativo despacha;
  no nativo responde 200 con mensaje parcial. No son pruebas de Searchable.
- `server/models/documents.js:85-276`: vectorizar antes de fila; compensación
  L192-218 puede fallar; delete vector→fila→mapping sin transacción conjunta.
- `server/utils/vectorDbProviders/lance/index.js:190-229,297-420`: topN antes de
  umbral/exclusión; borrado depende de mapping; tabla→cache→mapping separados.
- `server/utils/EmbeddingWorkerManager.js:17-87,91-170`: replay solo con worker
  activo, borrado del buffer diez segundos después de all_complete; no journal.
- `server/jobs/sync-watched-documents.js:90-260`: delete/reinsert; fuente avanza
  incluso con retornos vectorized=false; siguiente comparación puede omitir retry.
  Test `server/__tests__/jobs/sync-watched-documents.test.js:1-164` y
  `jest.config.cjs:1-6` leídos, **no ejecutados**.
- `server/utils/chats/stream.js:93-155,180-307`: negativa query sin vectores;
  backfill entra en contextTexts, no en sources actuales; prompt se guarda antes
  de compressMessages. Citas no acreditan el contexto final entregado.
- `server/jobs/extract-memories.js:65-181` y `server/models/memory.js:345-439`:
  finally marca procesados; validación/ownership y transacción de hechos sí
  existen; excepciones de apply se capturan y devuelven contadores, no certifican
  NoFacts. Cupos externos a la transacción no prueban serialización.
- `server/models/workspaceParsedFiles.js:110-174`: finally elimina attachment
  aun con fallo; embedding ocurre después de move. Coincide con #6806.
- `server/utils/middleware/validatedRequest.js:8-110` y
  `validWorkspace.js:23-47`: bypass single-user condicionado, multiusuario con
  JWT/usuario/suspensión, thread ligado a usuario y workspace.
- UI: `Documents/index.jsx:10-94` y hook L643-689 del directorio indicado;
  `DataHandling/index.jsx:7-35` solo navega; `ProviderPrivacy/index.jsx:21-70`
  consulta tres proveedores. **No hay consentimiento persistido acreditado por
  defecto en esos componentes.** `Citation/index.jsx:60-90` construye favicon
  Google; `frontend/src/utils/folderUpload.js:3-15` menciona Electron, no su IPC.
- Vanta: `vanta-memory/src/ingest/worker.rs:22-119` y
  `auto_sync.rs` (guards/run_id), `core/record/l1_writer.rs:176-184,289-294,389+`,
  `core/hooks/auto_recall.rs:133-142,288-335,453-456`,
  `context_engine/engine.rs:115-119,228-232`; confirman seams preexistentes.
  Studio `desktop/src/components/lens/retrieval/RetrievalLens.tsx:54-164`:
  picker de una página, filtros posteriores visuales y search explain server-side.

### Recibos públicos persistentes

Lecturas MCP `websearch.fetchWebContent` del 2026-11-07; respuestas sin truncar:

- [Issue #3958](https://github.com/Mintplex-Labs/anything-llm/issues/3958):
  abierto 2025-06-05; comentario 2025-06-29 no reproduce borrado y sí inversión
  skipCache; cierre completed 2025-06-30 referencia d1d68af. No se verificó el
  diff histórico ni se atribuye ese fallo a todos los adapters o a Vanta.
- [Issue #6806](https://github.com/Mintplex-Labs/anything-llm/issues/6806):
  abierto y cerrado not planned 2026-10-22; autor enlaza #6807, maintainer
  especifica rechazo/eliminación y re-upload como UX intencional. No prueba merge.
- [Desktop oficial](https://docs.anythingllm.com/installation-desktop/overview):
  single-player; por defecto 127.0.0.1; network discovery cambia a 0.0.0.0 con
  reinicio, expone API y no SPA; desaconseja Internet por falta de cuentas/auth.
  Docker sí admite multiusuario/widgets/UI web. Es documentación viva, no fuente
  Electron fijada ni autorización para iniciar/reiniciar nada.

### Lagunas y veredicto propuesto

No se auditaron todos los providers, auth de cada transporte, dependencias,
shell Electron, gitlinks, licencias transitivas ni garantías tras crash.
No se ejecutó UI ni se midió recuperación, calidad, coste o rendimiento.
Los once campos de ficha acreditan preparación documental, no diseño aprobado.
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance**, con la
corrección menor de anclajes registrada. **No aceptación final**: corresponde
al responsable principal y no autoriza implementar A–G.

## Khoj

### Recibo de lectura e identidad Khoj

Informe [khoj.md](../khoj.md), **725 líneas leídas completas**, incluidas
fichas A–G, benchmark y fuentes. Clon `C:\Users\Eros\khoj`; HEAD comprobado:
`ae229ca894c0b80ad84664afcfdde523b5e87057` (en adelante SHA K).
`pyproject.toml` confirma Python >=3.10,<3.13, Django 5.1.15, pgvector 0.2.4,
sentence-transformers 3.4.1, Torch 2.6 y AGPL-3.0-or-later.
El backend PostgreSQL y pgserver opcional se comprobaron leyendo settings,
**sin importarlo**. Los paquetes/licencias de clientes se distinguen del motor.

### Ledger Khoj 0 + diez ejes

| Eje | Estado | Sección y evidencia / laguna |
|---|---|---|
| 0. Arquitectura y tres flujos | parcial | §1 traza indexación→consultas→delete/reinsert y chat→UserMemory. Falta la pérdida de asociación agent durante edición manual y la asimetría del delete automático; hallazgo medio primero. PostgreSQL/pgvector sí acreditados. |
| 1. Recomendaciones | sustentado | §2: siete mecanismos, prioridad y dependencias específicas; no copia plataforma Python ni servicios al motor. |
| 2. Fichas Rust | parcial | §3 A–G contiene los once campos, contratos propuestos/no implementados, delta y seams reales, alternativas, FMEA, rollback y criterios futuros. D necesita incorporar la consecuencia de editar/cambiar scope, no solo pérdida de ID/contenido. |
| 3. Incidencias upstream | sustentado | §4: #1437/#1363/#759 releídos íntegros; SHA afectado solo cuando consta; ningún cierre/PR se convierte en solución universal. |
| 4. Prerrequisitos Vanta | sustentado | §5: worker/progreso/run_id/scheduler y L1/context assembly existentes; capacidades del caller/transportes pendientes, no bloqueo universal. |
| 5. Rechazos | sustentado | §6 y no-goals A–G: producto, defaults/ranking, vacío-delete y contratos de clientes rechazados como sustitutos universales. |
| 6. Advertencias | sustentado | §7: tests no ejecutados, settings con efectos, Cloud distinto del clon y ausencia no inferida desde carpeta. |
| 7. Benchmarks | parcial | §8 distingue IR recall de F1 impreso y QA/juez; dropna/None/futures/costes bien señalados. Falta fallo derivado del default BATCH_SIZE y diferencia entre artículos indexables y gold de IR; hallazgos medios segundo/tercero. |
| 8. Matriz | parcial | §9 correcta en motores, clientes y niveles de benchmark; fila tenancy necesita matizar lectura user+agent frente a edición/borrado solo user. |
| 9. Negativos Vanta | sustentado | §10 no convierte riesgos Khoj en bugs Vanta; descarta equivalencia checkpoint/WAL y postfiltro/pushdown. |
| 10. UI/escritorio | sustentado | §11: código real Next, Electron main/preload/IPC, plugin, Emacs y TWA; no Android offline/iOS nativo inventados. Ack IPC no significa sync aplicado. |

### Hallazgos Khoj ordenados por gravedad

**Media — la edición de UserMemory cambia también su scope de agente.**

- **Informe:** `khoj.md:169-174`, ficha D `294-327`, matriz `584` y UI `611`.
  Se documentan delete-before-save y cambio de ID, pero no el cambio de
  asociación. La descripción general de scoping user+agent requiere separación
  por operación, no extenderla del retrieval al CRUD.
- **Fuente/mecanismo:** [api_memories.py:83-103][k-memory-api] carga una memoria
  por user/id, la elimina y llama `save_memory(user=user,memory=new_content)`
  **sin agent**. [UserMemoryAdapters:2311-2326][k-memory-adapters] tiene
  `agent=None` por defecto y crea la nueva fila sin asociación. En L2301-2306
  y L2342-2345, agentes no default leen solo sus memorias; default lee todas
  las del usuario. Además, delete_memory L2357-2363 filtra user/id, **no agent**;
  helpers.py:1064-1066 lo invoca para deletes sugeridos por extracción.
- **Consecuencia estática:** editar desde settings un hecho asociado a un agente
  no default puede retirarlo del recall de ese agente y convertirlo en memoria
  sin asociación, además del cambio de ID ya señalado. La autorización probada
  en edición/delete es por usuario, no aislamiento mutacional por agente.
  **No** demuestra fuga entre usuarios ni explotación por Vanta o por un LLM.
- **Corrección verificable:** añadir al flujo/tabla el parámetro omitido y el
  efecto en pull/search; separar autorización user de asociación agent. Ampliar
  FMEA/criterios documentales de D para conservar o cambiar explícitamente el
  scope al editar y discriminar IDs de otra asociación. No implementar aquí
  ningún contrato ni atribuir semántica defectuosa al writer de Vanta.

**Media — el default derivado de BATCH_SIZE puede impedir una muestra pequeña.**

- **Informe:** `khoj.md:520-524,536-544` describe muestra opcional y batch
  derivado, pero solo enumera el fallo aritmético posterior de running accuracy.
- **Fuente/mecanismo:** [eval.py:44-48,664][k-eval] calcula
  `int(os.getenv("BATCH_SIZE", int(SAMPLE_SIZE) / 10 if SAMPLE_SIZE else 10))`.
  Con SAMPLE_SIZE entero positivo entre 1 y 9 y sin override, el resultado es
  cero; `range(0,dataset_length,BATCH_SIZE)` tiene paso cero. Es inferencia
  determinista de fuente, **no ejecución**.
- **Consecuencia:** una validación futura mínima no llega a emitir preguntas;
  no es un fallo del answerer ni una observación de calidad cero. Afecta al
  alcance de los defaults documentados, no invalida todos los resultados Khoj.
- **Corrección verificable:** documentar fórmula exacta, precondición de paso
  válido y momento del fallo, separado de None/dropna y del fallo dentro de
  futures. No se cambia default/configuración, harness ni tamaño de muestra.

**Media — los filtros de población gold e ingesta difieren en FRAMES IR.**

- **Informe:** `khoj.md:501,528-532` explica archivos con texto no vacío y recall
  por artículos gold, pero no relaciona ambas poblaciones.
- **Fuente/mecanismo:** [eval.py:93-105,152-157,408-410][k-eval]:
  index_frames_kb omite textos vacíos; get_articles_by_prompt_id solo filtra
  `frames_prompt_id`, y `len(articles)` se usa como denominador sin ese filtro.
  Tampoco contrasta hashes/revisión de archivos que ya estaban indexados.
- **Consecuencia:** el denominador **puede** incluir artículos que este camino
  no subió. No se inspeccionó el dataset completo y **no se afirma** que ocurra
  con una pregunta concreta ni se calcula sesgo o recall máximo.
- **Corrección verificable:** explicitar filtros distintos y desconocimiento
  de su diferencia real; dejar pendiente conciliación del inventario de fuentes
  de esa ejecución futura, sin cambiar fórmula, dataset, scoring ni muestreo.

### Evidencia local Khoj y controles positivos

Rutas del clon SHA K, leídas sin importar módulos ni ejecutar el proyecto:

- `src/khoj/app/settings.py:121-185`: backend PostgreSQL, credenciales por env;
  USE_EMBEDDED_DB puede crear directorio/arrancar pgserver/extensión al importar.
- `src/khoj/database/models/__init__.py:544-582,760-864`: modelos de búsqueda,
  FileObject/Entry/EntryDates/UserMemory, vectores y asociación nullable agent;
  defaults de schema no certifican configuración efectiva.
- `src/khoj/processor/content/markdown/markdown_to_entries.py:21-124` y
  `text_to_entries.py:35-141,142-263`: tokenizer es `text.split()` (palabras),
  no tokenizer LLM; regex de presencia y de split no son equivalentes y el
  while carece de bound. Regenerate borra primero; batches fallidos se capturan
  y luego se borran hashes obsoletos; no rollback global observado.
- `src/khoj/routers/api_content.py:544-627`: executor esperado; 500 global o 200
  con nombres de entrada, no comprobación por documento persistido.
- `src/khoj/routers/helpers.py:1484-1574` y
  `database/adapters/__init__.py:2090-2178`: autorización agent antes de cache;
  clave por usuario/query/n/type/r/distance/dedupe, sin agent/model/revisión.
  WordFilter es SQL raw__icontains; filtros antes de coseno/order/limit.
  `search_type/text_search.py:99-164`: hasta diez candidatos, dedupe/collate;
  no se acreditó BM25/RRF en esa vía ni se generaliza ausencia global.
- `src/khoj/routers/helpers.py:996-1067`,
  `processor/conversation/utils.py:541-620` y
  `database/adapters/__init__.py:1623-1695,2292-2378`:
  guardar mensajes precede extracción; política efectiva condiciona auto-update;
  parse inválido produce create/delete vacíos; retrieval combina recientes y
  semánticas. No equivale a supersession atómica ni checkpoint de tarea.
- `src/khoj/routers/api_chat.py:676-790,970-1005,1350-1486`: evento de
  cancelación, monitor y guardado parcial; historial/referencias interrumpidas;
  save normal por create_task antes de END_RESPONSE, no durabilidad certificada.
- **Eval:** `tests/evals/eval.py:1-726` leído completo. IR devuelve recall L428
  aunque imprime F1; precision/recall 0/0→1; QA usa Gemini, GPQA MCQ; None se
  excluye en dropna; wait(futures) no inspecciona result; excepción puede truncar
  batch. create_new no resetea memoria/corpus. FRAMES IR hace PATCH persistente;
  CSV/resumen, descarga y llamadas pagadas son efectos, no resultados propios.
- **Coste:** `src/khoj/utils/helpers.py:960-980` y
  `utils/constants.py:53-57`: helper usa tarifas estáticas y cero para modelo
  desconocido; eval L499-501 solo pasa prompt/candidate tokens. Total Cost no
  acredita factura, thought/cache/embeddings/herramientas/retries completos.
- **UI:** `src/interface/desktop/main.js:159-275,443-497,590-615`,
  `preload.js:12-77`, `renderer.js:33-83`: main/IPC reales; syncData no retornado
  por handler; shell.openPath es frontera OS. nodeIntegration/innerHTML no
  bastan para declarar exploit sin caller/navegación/entrada hostil.
  `web/app/search/page.tsx:656-694`, `chat/page.tsx:492-536`,
  `settings/page.tsx:660-730`: search, WebSocket y CRUD/política reales.
- `src/interface/obsidian/src/utils.ts:104-248` y `api.ts:1-30`:
  borrar antes de regenerate, batches, PATCH bearer, confirmación por substring,
  no receipt exacto. `src/interface/android/twa-manifest.json:1-15` acredita TWA,
  no runtime nativo offline. No confundir Cloud/producto con fuente local.
- Vanta/Studio: seams y snapshots del recibo AnythingLLM se mantienen;
  filtros ya existen y RetrievalLens postfiltra visualmente. Los nombres
  ParsedSearch/SyncReceipt/ContextEvidence/MemoryPolicyDecision/ParseOutcome/
  RecallCacheIdentity/OperationAck son **propuestas del informe**, no tipos
  acreditados como implementados en Vanta.

### Recibos públicos Khoj

MCP `websearch.fetchWebContent`, 2026-11-07, tres páginas completas sin truncar:

- [#1437](https://github.com/khoj-ai/khoj/issues/1437): **open**, 2026-09-26;
  reporte refiere exactamente SHA K y nota larga con heading vacío/tab;
  describe while sin progreso, test en thread y fix preparado. Enlace Development
  #1440 no prueba merge; diff/PR y ejecución no verificados. El código local
  concuerda con el mecanismo, no acredita todos los clientes/configuraciones.
- [#1363](https://github.com/khoj-ai/khoj/issues/1363): **open**, 2026-07-01;
  Windows 11, Docker/WSL2 y vault >500 notas según autor; solicita estado por
  archivo/coverage/export. No hay SHA afectado exacto ni fix identificado;
  progreso por batches en código no satisface por sí solo ese recibo durable.
- [#759](https://github.com/khoj-ai/khoj/issues/759): abierto 2024-05-26,
  cerrado **completed** 2024-09-29; maintainer cita aviso de cuota y selección
  de formatos. Autor dice que lo probará, no que reprodujo arreglo. No commit
  corrector ni causa única acreditados; no se extrapola a #1437 o a Vanta.

### Lagunas y veredicto Khoj

No se leyó PR #1440 ni historia correctora de #759; no se descargaron datasets
ni se verificaron revisiones/solapamientos reales de sus poblaciones.
No se auditó cada invalidación de cache, proveedor, transporte, deployment Cloud
ni seguridad completa de Electron/Android/Emacs. Lectura de UI no demuestra
interacción, cancelación integral, rendimiento o permisos en runtime.
**Veredicto propuesto: cambios requeridos**, por los tres hallazgos medios
documentales, sin negar los controles positivos contrastados.
**No aceptación final**, ni autorización para corregir código o ejecutar casos.

[k-memory-api]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api_memories.py#L83-L103
[k-memory-adapters]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/adapters/__init__.py#L2292-L2378
[k-eval]: https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L34-L707

La revisión supera 300 líneas para conservar dos recibos autónomos en el único
archivo autorizado. Si se autoriza una fase posterior, puede separarse por
proyecto manteniendo los ledgers y enlaces; no se crean esos archivos ahora.

### Control de cierre y accesos

Ambos ledgers y recibos se persistieron al terminar cada contraste; revisión
releída completa y enlaces internos comprobados por lectura, no por validación
global. Los comandos usados fueron exclusivamente de lectura:
`Get-Content`, `Select-String`, `Get-ChildItem`/`Get-Item`, conversión del JSON
del manifiesto, `git rev-parse`, `git ls-tree` y `git status` scoped.
`git check-ignore -v` confirmó que `.gitignore:268` ignora `docs/dev/research/`:
este recibo existe en disco, pero status normal no lo muestra. El responsable
deberá decidir su incorporación/preservación en una fase autorizada; no se
modificó ignore, índice ni se creó commit.
La consulta inicial de búsqueda localizada fue seguida por lectura directa
de informes y fuentes; no se toma el resultado de búsqueda como evidencia única.

Incidencias operativas resueltas: `rg` no estaba disponible y se sustituyó por
Select-String; la función abreviada R colisionó con el alias Invoke-History
y sus llamadas fallaron antes de leer fuentes, por lo que se repitieron como
ReadRange. Un intento de `Get-Content` a
`frontend/src/hooks/useDocumentPicker.js` de AnythingLLM falló por ruta
inexistente; se localizó y leyó la ruta completa indicada arriba. Ningún fallo
de acceso se convirtió en prueba de ausencia funcional. Las seis lecturas
públicas finales sí entregaron contenido; no queda una página inaccesible
presentada como leída.
