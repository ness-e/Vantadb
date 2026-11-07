---
title: "Revisión independiente de Agno, AutoGen, Agents SDK y Flowise"
kind: research
status: proposed
description: "Ledgers adversariales de cuatro informes, contraste estático de fuentes y correcciones verificables sin aceptación final"
tags: [research, memory, engineering]
---

# Revisión independiente de Agno, AutoGen, Agents SDK y Flowise

## Índice

- [Alcance y recibos](#alcance-y-recibos)
- [Agno](#agno)
- [AutoGen](#autogen)
- [OpenAI Agents Python](#openai-agents-python)
- [Flowise](#flowise)
- [Cierre de los cuatro recibos](#cierre-de-los-cuatro-recibos)

Si se aprueba convertir hallazgos en trabajo activo, separar por proyecto y
enlazar los recibos originales; no se crean specs, índices o contratos aquí.

## Alcance y recibos

Revisor independiente de la autoría. Fecha de lectura: 2026-11-07.
Único archivo editado: esta revisión; sin staging/commit/push, ejecución de
proyectos, tests, servicios ni cambios de harness. Los casos adversos de los
informes son aceptación **futura**, no pruebas ejecutadas.
Contratos leídos: [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md),
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md), [REVIEW-COVERAGE](../REVIEW-COVERAGE.md)
y el brief `C:\Users\Eros\AppData\Local\Temp\vantadb-research-review-brief.md`.
Se verificó antes de planificar que este recibo no existía.

Base Vanta contrastada mediante `git show` de
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, no HEAD mutable.
Studio se leyó en el worktree separado, HEAD
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`.
No se infiere ausencia global desde muestras ni se exige runtime como bloqueo
universal. Los veredictos son propuestas al responsable, no aceptación final.

## Agno

**Recibo durable:** informe [agno.md](../agno.md), leído íntegro, líneas
1–624, SHA-256
`c34ed65266cd9d714dad3586569357c62ba5abac183f7c1429741e395f6fb00a`.
Clon `C:\Users\Eros\agno`, HEAD verificado
`f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68` (A-pin).
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10 ejes

| Eje | Estado | Sección y contraste |
|---|---|---|
| 0. Arquitectura/identidad/flujos | parcial | §§1–2/11.1: sesión/hechos/run/knowledge correctamente separados; update F3 no hace la lectura scoped que se afirma, A-1 |
| 1. Recomendaciones | sustentado | §§1/2: A–E son deltas de consolidación, mapping, aprobación, cobertura y capacidades, no nuevo motor |
| 2. Fichas Rust | parcial | §§2/11.2: A–E cubren los once campos al combinar fichas y complemento; el supuesto de update scoped debe corregirse antes de derivar B/E |
| 3. Incidencias upstream | sustentado | §11.3: cuerpos y comentarios #11143/#9983 releídos; fecha/estado/revisión afectados y PR sólo propuesto correctamente distinguidos |
| 4. Prerrequisitos Vanta | sustentado | §§4/11.2: RecallScope/RecallConfig/InjectionPolicy y tenancy existentes; CAS de grupo no acreditado, condicionado por candidato |
| 5. Rechazos/alternativas | sustentado | §§2/5: rechaza clear-then-upsert, IAM/runtime paralelo y LLM obligatorio; alternativas por ficha |
| 6. Advertencias | sustentado | §§6/11: muestra acotada, ID no commit, API no cierre de seguridad de las tools |
| 7. Benchmarks | sustentado | §§7/11.4: segundos/MiB, 10 warmups, entrypoint 1000, tiempo/memoria en invocaciones distintas; excepciones sin resultado agregado, sin reset de DB |
| 8. Matriz | parcial | §§8/11.5: baseline Vanta y WriteReceipt precisos; matizar seguridad por recurso con el camino update de A-1 |
| 9. Negativos Vanta | sustentado | §§9/11.5: hipótesis de cobertura, no bugs inferidos por analogía ni ausencia de memoria híbrida/budgets |
| 10. UI/escritorio | sustentado | §§10/11.1/11.5: router HTTP local, os.agno.com externo y Studio real, sin frontend SaaS ni render probados |

### Hallazgos

**A-1 — Alta: F3 atribuye a update una comprobación de propiedad inexistente
en el camino citado.** Informe líneas 157–159: «La tool update obtiene la
memoria por ID/usuario y combina valores». En A-pin,
[`manager.py:1386–1416`](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/memory/manager.py#L1386-L1416)
construye un `UserMemory` y llama directamente a `db.upsert_user_memory`;
no hay lectura de la fila ni validación de ownership previa.
[`sqlite.py:2699–2725`](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/sqlite/sqlite.py#L2699-L2725)
resuelve conflicto sólo por `memory_id`, actualiza contenido/topics/input/
agent/team y no añade condición por user. Pasar user en el objeto insertado
no demuestra que el update sobre una fila existente lo respete.
**Consecuencia:** el dossier sobreatribuye aislamiento a un camino destructivo;
con DB compartida e ID ajeno conocido, esa llamada puede modificar una fila de
otro user. Es deducción estática del camino directo sin wrapper, no exploit REST
reproducido ni afirmación sobre todos los adapters.
**Corrección verificable:** sustituir F3 por el flujo real, describir conflicto
por ID frente a autorización, y actualizar B/E y la matriz con esta frontera.
Conservar como validación manual futura el control de ID propio/ajeno/inexistente
y el contraste con wrapper autorizado; no declarar test pasado.

### Evidencias efectivamente contrastadas y límites

- A-pin: `memory/manager.py` 220–247, 377–432, 576–739, 803–873,
  1343–1452; `memory/strategies/summarize.py` 35–125;
  `learn/stores/user_memory.py` 55–155 y `entity_memory.py` 426–505,
  1300–1328; `db/sqlite/sqlite.py` 1060–1235, 2672–2741, 2849–2867.
  Se confirma UUID retornado aun cuando `_upsert_db_memory` devuelve texto de
  error; sin await de AsyncBaseDb y sin comprobar retorno. Clear global sí
  está acreditado, distinto de clear_user_memories. Consolidación tiene
  commits separados clear/upsert; no rollback global demostrado.
- Modelos y producto: `db/schemas/memory.py` 8–65, `session/agent.py` 15–70,
  `knowledge/knowledge.py` 85–151, router `os/routers/memory/memory.py` 55–142,
  `pyproject.toml` 1–85 y README 20–80. LICENSE Apache-2.0 leído; no licencia
  del SaaS inferida ni auditoría de todas las dependencias.
- Benchmark: `eval/performance.py::PerformanceResult.compute_stats`,
  `_measure_time/_measure_memory`, defaults 203–246 y `run` 497–632;
  `cookbook/09_evals/performance/instantiate_agent.py` completo.
  No dataset QA, denominador de preguntas ni resultados propios. Warmups
  también ejecutan la función; GC no resetea estado persistente y otras
  funciones pueden tener efectos/red/coste; telemetría true por defecto.
- Web, `websearch.fetchWebContent`, HTML completo leído:
  [#11143](https://github.com/agno-agi/agno/issues/11143) Open, 2026-10-18,
  commit reportado `0aa5924`, comentario #11154; y
  [#9983](https://github.com/agno-agi/agno/issues/9983) Open, 2026-09-05,
  v3.0.6 `d1a388446e1b44b20498c772e91303588e2734cf`, comentarios #9986.
  Coincide mecanismo en el pin; PR mencionado no prueba merge ni CVE.
  No hubo acceso fallido propio a estas dos páginas.
- Vanta pin: `l1_writer.rs::persist_planned/log_and_mark` 289–355;
  `auto_recall.rs` 130–185, 270–340; `engine.rs` 197–218, 252–330;
  `src/attestation.rs` 678–755; `task_checkpoint.rs` 90–160;
  adapters `integrations/openai/.../vectorstore.py` 15–70,
  `vantadb_shared/__init__.py` 31–163 y LangChain checkpointer 232–296/store
  39–65, 97–177. Son recursos distintos. WriteReceipt es SHA-256/relectura,
  **no firma digital**; MutationReceipt permanece **PROPUESTO**.
- Studio: MemoryLens 1–85, RetrievalLens 1–95, ActivityPanel 1–65;
  se confirma superficie existente, no render/runtime.

No se auditaron wrappers auth completos, todos los drivers, providers,
relationships ni frontend SaaS. No se midieron durabilidad, aislamiento
end-to-end, calidad, latencia ni coste. La laguna no convierte una hipótesis
en bug Vanta ni impide corregir documentalmente A-1.

## AutoGen

**Recibo durable:** [autogen.md](../autogen.md), leído íntegro, líneas 1–517,
SHA-256 `ba3afcd302a9c10cd787ae3a69624cfe89d1996bf067e3dd174f3f6fda571831`.
Clon `C:\Users\Eros\autogen`, HEAD verificado
`d07c667a3239ea4ab7af62a53b0f4ff205a1f23d` (G-pin).
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10 ejes de AutoGen

| Eje | Estado | Sección y contraste |
|---|---|---|
| 0. Arquitectura/identidad/flujos | parcial | §§1/2/11.1: estado/contexto/memoria/subscriptions separados correctamente; G-1 corrige terminal de error y topología Chroma |
| 1. Recomendaciones | sustentado | §§1/11.2: adapter pequeño condicionado al coste de soportar proyecto no mantenido; sucesor fuera de alcance |
| 2. Fichas Rust | sustentado | A–E §§2/11.2–11.4 cubren once campos, errores/ownership/migración/FMEA y aceptación futura; extienden seams existentes |
| 3. Incidencias upstream | sustentado | §11.5: #6793/#5688 cuerpo/conversación releídos; históricos, no bugs actuales del pin ni prueba de Vanta |
| 4. Prerrequisitos Vanta | sustentado | §§4/11.2: scopes, MIME, recibos y checkpoint por recurso; sin imposición de nuevo runtime |
| 5. Rechazos/alternativas | sustentado | §§5/11.2–11.4: no portar ejecutores, pickle no confiable, trimming ni ACL por modelo |
| 6. Advertencias | sustentado | §§6/11: snapshot durante run inconsistente, restore no atómico, lecturas no ejecución, MIT/CC por componente |
| 7. Benchmarks | parcial | §§7/11.6: fórmula conocida/missing correctos, pero falta población de tareas sin directorio, G-2 |
| 8. Matriz | sustentado | §§8/11.7: backend Vanta ya híbrido/quotas/dedup/context; binding AutoGen no acreditado, no negación global |
| 9. Negativos Vanta | sustentado | §§9/11.7: sólo lagunas de mapping/export/ledger; prefijo protegido documentado, no promesa absoluta de budget |
| 10. UI/escritorio | sustentado | §§10/11.7: web Gatsby/FastAPI/WS real, auth condicional y ownership; no desktop ni seguridad total inferidos |

### Hallazgos de AutoGen

**G-1 — Media: el camino de fallo no emite el TaskResult que describe F1;
Chroma tampoco es necesariamente remoto.** Informe líneas 278–281.
En G-pin, [`_base_group_chat.py:550–578`](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_base_group_chat.py#L550-L578),
`GroupChatTermination.error` provoca `raise RuntimeError` antes del yield
`TaskResult`; sí ejecuta cleanup en finally. En
[`_chromadb.py:179–180,246–262`](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py#L246-L262),
el default es PersistentChromaDBVectorMemoryConfig/PersistentClient local;
HttpClient es otra configuración. **Consecuencia:** un consumidor podría
esperar resultado terminal inexistente o asumir frontera de commit/red equivocada.
**Corrección verificable:** bifurcar salida normal/error/cancelación, reservar
TaskResult a terminación normal, y separar backend local/HTTP. No requiere
ejecutar el runtime para corregir estos hechos estáticos.

**G-2 — Media: Missing/Total no cubren tareas enteramente ausentes de runlogs.**
Informe líneas 433–442 explica correctamente successes/(successes+failures),
pero no delimita el universo de `Total`. En G-pin,
[`tabulate_cmd.py:127–191`](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/tabulate_cmd.py#L127-L191)
recibe sólo runlogs, enumera directorios existentes y forma filas desde ellos;
no reconcilia el JSONL esperado. Padding agrega None para repeticiones de
filas presentes, no para una tarea sin directorio. Líneas 214–261 agregan esa
población, no todas las tareas planificadas/intentadas.
**Consecuencia:** imprimir Missing y Total no acredita cobertura completa de
intentos; subsiste sesgo de supervivencia aun sin eliminar explícitamente None.
**Corrección verificable:** declarar unidad tarea×repetición y universo de
directorios descubiertos, distinguir missing conocido/tarea no representada
y dejar reconciliación como evidencia futura bajo METHOD, sin cambiar fórmula,
scorer, muestreo, configuración o harness. No invalidar globalmente GAIA.

### Evidencias contrastadas y límites de AutoGen

- G-pin: README 14–31 y LICENSE/ LICENSE-CODE (CC BY 4.0/MIT);
  agentchat pyproject 1–38 (0.7.5/core==0.7.5, Python >=3.10).
  Discrepancia entre archived=false congelado y README «no mantenido»
  correctamente conservada; no sustitución por Microsoft Agent Framework.
- `autogen-core/memory/_base_memory.py` 13–140,
  `_list_memory.py` 104–180; `autogen-ext/memory/chromadb/_chromadb.py`
  170–430; `_assistant_agent.py` 1028–1055, 1580–1645.
  `add` UUID, clear colección, close recurso y on_reset contexto son distintos.
  Token de cancelación no se consulta en ListMemory ni en el query mostrado
  de Chroma; update_context no recibe token. No se acredita rollback.
- `_chat_completion_context.py` 49–78 y `_token_limited_chat_completion_context.py`
  19–94; `_base_group_chat.py` 247–264, 483–577, 657–835;
  `_single_threaded_agent_runtime.py` 431–468; `_swarm_group_chat.py` 47–115;
  `messages.py::BaseMessage.dump` 45–58. Se confirma save_state sin DB externa
  ni suscripciones runtime; restore participante→manager puede ser parcial.
- `agbench/run_cmd.py::run_cli` 869–962, `tabulate_cmd.py` 1–110, 110–265,
  GAIA `init_tasks.py` 1–110 y `custom_tabulate.py` 1–170. Docker/nativo,
  repeat/parallel=1, subsample=None, descargas y efectos/red/pago correctamente
  separados de calidad de memoria; ausencia de reset de stores externos no
  equivale a afirmar que Docker no aísla filesystem.
- Studio: frontend `package.json`, `views/playground/api.ts` 1–115;
  FastAPI `routes/runs.py` 13–80 y `ws.py` 20–155. Session API usa user_id;
  WS valida run/status y owner/admin si auth habilitada. No auditoría global.
- HTML público leído con `websearch.fetchWebContent`:
  [#6793](https://github.com/microsoft/autogen/issues/6793), abierto
  2025-07-11/cerrado 2025-07-15, reporta 0.6.4 y datetime; hilo #6797.
  [#5688](https://github.com/microsoft/autogen/issues/5688), abierto
  2025-02-24/cerrado 2025-02-26, dev main sin SHA, #5727/dd0781a.
  Serializador MIME y model_dump(mode="json") sí están en G-pin.
  No se comprobó diff/merge histórico de los PRs ni se extrapola arreglo universal.
- Seams Vanta/Studio releídos según inventario Agno de esta revisión:
  EmbeddingVectorStore, VantaDBCheckpointer, TaskCheckpointManager,
  assemble_with_recall/IntegratedContext, RecallScope/RecallConfig/
  InjectionPolicy, l1_writer y WriteReceipt; deltas A–E no recrean esas capas.

Límites: sin .NET, runtime distribuido completo, sucesor, todos los templates,
descargas GAIA, workbenches ni auth global. Tiempo AgBench de logs no demuestra
latencia retrieval; no scores, presupuestos pagados ni aislamiento medidos.

## OpenAI Agents Python

**Recibo durable:** [openai-agents-python.md](../openai-agents-python.md),
leído íntegro, líneas 1–632, SHA-256
`01a093ec66b034fedabff9a455359a63dbba90ea66d3d8d37e0336bb1fbbffc5`.
Clon `C:\Users\Eros\openai-agents-python`, HEAD verificado
`27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9` (O-pin).
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance.**
No es aceptación final ni certificación de todos los paths del SDK.

### Ledger 0 + 10 ejes de Agents SDK

| Eje | Estado | Sección y contraste |
|---|---|---|
| 0. Arquitectura/identidad/flujos | sustentado | §§1/11.1–11.2: Session historial, RunState interrupciones, trace observación, tool efecto externo y handoff vista separados; tres recorridos y fronteras explícitos |
| 1. Recomendaciones | sustentado | §§1/11.3: ownership/publicación antes que runtime, endpoint LLM o dedup nuevo |
| 2. Fichas Rust | sustentado | A/B más complemento y C/D/E numeradas: once campos cubiertos, interfaces PROPUESTAS, alternativas y dependencias por candidato |
| 3. Incidencias upstream | sustentado | §12.1: #4646 y #4990 cuerpos/comentarios contrastados; pin no tratado como revisión histórica; Closed no equivale a fix |
| 4. Prerrequisitos Vanta | sustentado | §§4/11.3–11.6: identificar CAS real, ownership y backend; spill/assembler/checkpoints/WriteReceipt existentes |
| 5. Rechazos/alternativas | sustentado | §5 y fichas: fail-closed frente a repetir tool, no clear/add como ACID, no proveedor obligatorio ni runtime Rust |
| 6. Advertencias | sustentado | §§3/6/12.4: settlement no timeout ni exclusión, muestra y backends limitados, pruebas propuestas no ejecutadas |
| 7. Benchmarks | sustentado | §§7/12.2: validador de filas financieras, no accuracy/recall; excepción o row_count, generación separada, sin medición propia |
| 8. Matriz | sustentado | §12.3: capacidades Vanta vigentes y delta por recurso, no heredadas de c561158 |
| 9. Negativos Vanta | sustentado | §§9/12.3: ownership/adapter son hipótesis; no ausencia de híbrido/quotas/recibos ni tokens exactos desde caracteres |
| 10. UI/escritorio | sustentado | §§10/12.4: Graphviz local/topología, tracing SaaS externo y Studio existente; no licencia/render de SaaS acreditados |

### Hallazgos y precaución residual de Agents SDK

No se encontraron discrepancias bloqueantes en los mecanismos efectivamente
contrastados. No se convierte esta conclusión en paridad de runtime.
**Precaución no bloqueante:** F2 (líneas 350–359) resume la reconciliación de
un append **ya ofrecido**. En O-pin `resume_pending_session_write:1459–1505`
también distingue `held=True`: batch retenido por gate, nunca ofrecido,
puede atravesar el checkpoint sin settlement hasta salida legal; no confundirlo
con fallo ambiguo de append. `_PendingSessionWrite` lo documenta expresamente.
Si una spec posterior amplía D a todo pending, debe conservar esa distinción
y no publicar antes del guardrail. No se exige ampliar esta investigación
a todo el run loop ni modificar prompts/harness para resolver esa cautela.

### Evidencias contrastadas y límites de Agents SDK

- O-pin: pyproject 1–87, LICENSE MIT, README 1–30, `run.py::Runner.run`
  279–343; `memory/session.py` 21–145,
  `sqlite_session.py` 261–358, 447–537. `await_session_mutation` espera
  asentamiento incluso ante cancelaciones repetidas, no tiene timeout,
  retry, transacción o exclusión global. SQLite commit local no cubre tools.
- `session_persistence.py` 965–1035, 1444–1625, 1873–1962 y
  `run_state.py::_PendingSessionWrite` 189–237. Se confirma inclusión de
  NextStepInterruption y RunAgain, comparación de tail, wrong-session,
  ambigüedad/partial fail-closed y obligación de serializar copias por el host.
  Mismo session_id no demuestra misma DB. No auditoría completa de
  serialización/validación de RunState ni de guardrails del run loop.
- `openai_responses_compaction_session.py` 90–184, 559–655;
  `encrypt_session.py` 189–214; SQLAlchemy 572–615; Redis 698–778;
  Conversations 122–195; Dapr 494–512. Wrapper inmediato conserva transforms;
  atomic replace sin CAS de snapshot no prueba entrada fresca. Batch remoto
  >20 implica requests separados; Dapr conserva dos deletes independientes.
- `handoffs/__init__.py` 72–179 y `result.py::cancel` 941–995:
  input_items no redacta por sí solo nested history; streaming anterior no es
  revocable; after_turn no es deshacer herramientas. No filtros de handoff
  en server-managed conversation según contrato local leído.
- Tracing: `run_config.py` 53–60, 414–443; `tracing/processors.py`
  45–94, 698–778; `tracing/setup.py` 17–70. Sensitive default true/env,
  export endpoint externo, queue 8192/batch128/delay5s y drops correctos;
  inicialización lazy/atexit no acreditan entrega durable o privacidad total.
- `dataroom_metric_extract/evals.py` **completo**, 1–315, y Makefile 1–150:
  JSONL/CSV→schema→duplicados/claves/cantidad/sección/valor/unidad→row_count.
  No denom QA, split, embeddings, generación ni juez en ese archivo; datos
  EXPECTED_ROWS no deben entrar como hechos de memoria. El artefacto default
  está anclado al directorio del tutorial. Sin reset/efecto DB del validador.
- HTML público releído vía `websearch.fetchWebContent`:
  [#4646](https://github.com/openai/openai-agents-python/issues/4646),
  abierto 2026-08-25/cerrado 2026-08-26 en #4654; reporte v0.22.0
  `4df9ecfae1761ca6fea67cc5a20b383c1d492024` y
  `36976b1854534c0f184ed955afd542493abbb5d9`, NextStepInterruption,
  publicación postefecto y resultados de guardrails. No se afirma merge
  independiente ni exactamente-una-vez; el pin ya contiene la rama descrita.
  [#4990](https://github.com/openai/openai-agents-python/issues/4990),
  abierto/cerrado 2026-09-13, main `fbd2dbca`, Dapr 1.16.0;
  cuerpo, #4991 draft y respuesta mantenedor leídos: bookkeeping/created_at,
  no corrupción acreditada de historial; pin sigue con dos awaits.
- `extensions/visualization.py` 1–65 acredita DOT/topología local.
  Seams Vanta compartidos contrastados antes, más `context_engine/spill.rs`
  1–160 en 97c9623: preservación de originales y recall explícito existentes.
  No nuevo historial, WAL, scheduler ni firma; MutationReceipt PROPUESTO.
  Studio leído en las tres lentes ya inventariadas.

Lagunas declaradas: schemas/serialización RunState completos, todos los stores,
sandbox/MCP, guardrails/streaming enteros, ocho documentos financieros y
frontend SaaS no inspeccionados. No hay scores, crash safety, seguridad,
costes o latencias medidos; las fuentes pendientes no se presentan como
ausencia ni como pruebas fallidas.

## Flowise

**Recibo durable:** [flowise.md](../flowise.md), leído íntegro, líneas 1–864,
SHA-256 `4c622e01f98d8e3e21ba6519f86172930a513f4273351b03893f901eef269be1`.
Clon `C:\Users\Eros\Flowise`, HEAD verificado
`9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb` (F-pin).
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10 ejes de Flowise

| Eje | Estado | Sección y contraste |
|---|---|---|
| 0. Arquitectura/identidad/flujos | sustentado | §§0/11.1: definición, Execution, mensajes, runtime state, BullMQ y proyección remota separados; tres trazas con commits independientes |
| 1. Recomendaciones | sustentado | §§1/2: referencia histórica, A–E deltas de adapters, no portar monolito/enterprise/RRF/runtime |
| 2. Fichas Rust | sustentado | §2 más §11.2 y E §11.3 cubren once campos, inputs/outputs/errors/ownership/migración, FMEA y aceptación futura; firmas no implementadas |
| 3. Incidencias upstream | sustentado | §11.4: #4968 configuración descartada como bug, #4433 histórico contrastado con persistState en pin |
| 4. Prerrequisitos Vanta | sustentado | §§4/11.2: fusión existente, provenance, config y recibos/checkpoints actuales antes de nuevos envelopes |
| 5. Rechazos/alternativas | sustentado | §§2/5: no copiar seenScores/canvas/IAM/comercial, no nuevo WAL/scheduler ni LLM para merge |
| 6. Advertencias | sustentado | §§3/6/11: auth por recurso no certifica todo, cancel no rollback, licencia/archivo/lectura estática y omisiones explícitas |
| 7. Benchmarks | parcial | §§7/11.5: fórmula pass correcta pero población simple vs LLM y unidad de latencia agregada incompletas, F-1/F-2 |
| 8. Matriz | sustentado | §§8/11.7: Vanta híbrido/scopes/budgets/receipts/adapters y Studio existentes, deltas por fuente/operación |
| 9. Negativos Vanta | sustentado | §§9/11.7: preguntas de cobertura, no bugs demostrados; RMW de checkpoint condicionado a caller, no carrera declarada |
| 10. UI/escritorio | sustentado | §§10/11.6: ReactFlow v2 y REST reales, web no desktop, status no durable, ampliar lentes Studio existentes |

### Hallazgos de Flowise

**F-1 — Media: el porcentaje no agrega indistintamente evaluadores simples y
LLM.** Informe §11.5, líneas 745–751: «resultados de evaluadores acumulados»
es insuficiente tras mencionar ambos tipos. En F-pin,
[`evaluations/index.ts:270–315`](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L270-L315)
alimenta passCount/failCount/errorCount **sólo** con `runAdditionalEvaluators`
y `additionalConfig.simpleEvaluators`. La rama LLM persiste otro
`resultArray` en `newRun.llmEvaluators` sin sumarlo a esos contadores.
`EvaluatorRunner.ts:13–51` recorre outputs de flows y evaluadores seleccionados;
si hay error genera Error por evaluador, no una única pregunta fallida.
**Consecuencia:** passPcnt puede ser -1 sin criterios simples aunque haya
resultados LLM, o reflejar criterios operacionales en vez del juez/QA que
un lector atribuya al porcentaje. **Corrección verificable:** nombrar la
población fila×flow×evaluador simple efectivamente emitido y separar LLM,
errores de prediction y fallos globales. Conservar fórmula y criterios
existentes; no crear un nuevo score, denominador, juez ni harness.

**F-2 — Media: falta delimitar la latencia medida y el denominador de
averageLatency cuando hay varios flows.** Informe §11.5, líneas 738–744,
describe latencia API end-to-end pero no el agregado que sí produce S8.
En F-pin,
[`EvaluationRunner.ts:135–180`](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/evaluation/EvaluationRunner.ts#L135-L180)
inicia reloj antes del POST y lo termina después de procesar métricas
Agentflow y awaits `getCostMetrics`; es intervalo del cliente, no sólo HTTP.
En
[`evaluations/index.ts:196–202,321–325`](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L321-L325)
totalTime suma latencias de cada evaluationRow/flow, pero averageLatency
divide por `result.rows.length` (filas de dataset), igual que totalRuns.
**Consecuencia:** con varios flows, el agregado representa suma media por
fila, no media por solicitud; cambiar cantidad de flows cambia la magnitud
sin demostrar regresión del motor. **Corrección verificable:** documentar
intervalo cliente, unidad ms/string y fórmula exacta por fila/flow con fallos,
sin renombrar ni corregir código upstream, sin nueva normalización/scoring
y sin solicitar ejecución como bloqueo universal. No se atribuye error a
Vanta ni se invalida el runner completo.

### Evidencias contrastadas y límites de Flowise

- F-pin: LICENSE.md (reparto comercial/terceros/Apache-2.0),
  package.json 1–85, server package 1–38 y UI package 1–85.
  Versión declarada 3.1.4 no prueba distribución instalada del reportante.
  Archivo y NOASSERTION no se transforman en licencia única ni uso autorizado
  del RBAC enterprise o de IdentityManager.
- `buildChatflow.ts` 481–505, 990–1145: clave externa, workspace→org,
  cuota y cola con waitUntilFinished, no ack inmediato al encolar;
  `BaseQueue.ts` 1–110 y `PredictionQueue.ts` 65–110: nombre jobId no
  opción jobId, removeOnFail=true y concurrency=100000 defaults confirmados.
  No se ejecutaron workers, Redis o servicios.
- `buildAgentflow.ts` 1575–1720, 1776–1868, 2020–2240, 2363–2415:
  consulta previa por session+flow+workspace y restauración Start opt-in;
  STOPPED/error-after-STOPPED reanudables, no toda ERROR/TERMINATED;
  abort cooperativo clasificado por texto; update Execution/SSE antes de
  mensajes finales. No CAS/replay exactly-once ni transacción tools+DB+Redis.
  `BufferMemory.ts` 86–150: lectura DB cronológica, add/clear delegados.
- `documentstore/index.ts` 319–354, 399–470, 1270–1444, 1489–1576,
  1995–2077: catálogo/chunks/archivos/historial separados del delete vectorial,
  upsert remoto antes de historial/telemetría/status, iteración no atómica.
  `DocStoreVector.ts` 40–173, Retriever 138–218, RRF 65–96 y constructor
  RRFRetriever 103–137 confirman config persistida, última asignación de docs,
  identidad por contenido/fallback UUID y seenScores; no recall medido.
- `tracingEnv.ts` 1–111: cache lifetime, UI activa prevalece, UI false no
  veto al entorno; borra flags LangChain tras adoptar config LangSmith.
  No recomendar esa mutación global de env para Vanta.
- Rutas documentstore 8–79 y predictions 1–18; entidades ChatFlow 1–85,
  Execution 1–48, ChatMessage 1–78, DocumentStore 1–43;
  chat-messages service 110–180. Ownership por recurso observado, no
  auditoría integral de middleware/IAM ni vulnerabilidad multi-workspace.
- `EvaluationRunner.ts` 1–235 completo; servicio evaluaciones 174–340,
  `EvaluatorRunner.ts` 1–165. Dataset/flow/evaluadores/config del producto,
  sesión opcional compartida, sin reset universal, errores por fila y
  efectos reales/pago/red posibles; no ejecutar. No se leyó todo
  EvaluatorRunner ni prompts/jueces LLM, ni se acredita ausencia de benchmark.
- UI: `Canvas.jsx` 219–254, 709–740; chatflows API 1–65;
  DocumentStoreStatus 1–100 y EvaluationResultSideDrawer 1–115.
  Serializar definición no publica efectos; colores/drawer no certifican
  durabilidad ni paridad con Cloud; no interfaz ejecutada.
- HTML público `websearch.fetchWebContent`:
  [#4968](https://github.com/FlowiseAI/Flowise/issues/4968), abierto
  2025-07-29/cerrado 2025-07-31, autor reconoce configuración
  overrideConfig.sessionId, sin SHA/versión exactos; no atribuir PR/fix.
  [#4433](https://github.com/FlowiseAI/Flowise/issues/4433), abierto
  2025-05-15/cerrado 2025-05-27 en #4512, Cloud sin SHA/versión;
  esta revisión sí leyó todo el texto extraído hasta cierre, no adjuntos:
  mantenedor anuncia toggle Start el 27.05. No se inspeccionó merge/diff
  del PR. Pin acredita restauración condicional, no state siempre efímero.
  Banner GitHub confirma archivo 2026-08-13; no se actualiza inventario.
- Vanta pin/Studio: seams comunes ya contrastados en recibo Agno;
  A usa fusión/provenance/assembler, B diagnóstico/actividad, C checkpoint,
  D recibo/write/generation log, E tenancy/bindings existentes. No duplican
  motor, IAM, scheduler, recibos o desktop. Garantía remota queda condicionada.

Limitaciones: no todas las funciones de buildAgentflow, nodos/proveedores,
record managers, ACL enterprise/global, transportes de cancelación completos,
adjuntos ni juez LLM completo. Persistencia observada en source no certifica
reinicio/crash/aislamiento ni seguridad del producto. Pendientes manuales
futuros no son resultados ejecutados y no bloquean corregir F-1/F-2.

## Cierre de los cuatro recibos

| Informe leído íntegro | Recibo independiente | Veredicto propuesto | Pendiente documental concreto |
|---|---|---|---|
| Agno | [Agno](#agno) | cambios requeridos | A-1: update no hace lookup/ownership por usuario en el camino citado |
| AutoGen | [AutoGen](#autogen) | cambios requeridos | G-1: error terminal y Chroma local/HTTP; G-2: población descubierta de AgBench |
| OpenAI Agents Python | [Agents SDK](#openai-agents-python) | sin hallazgos bloqueantes en este alcance | Mantener límites; pending held no equivale a append ambiguo ya ofrecido |
| Flowise | [Flowise](#flowise) | cambios requeridos | F-1: porcentaje sólo de criterios simples; F-2: intervalo y agregado de latencia por fila |

Los cuatro recibos se guardaron por proyecto, no desde un ID agregado.
Contienen arquitectura + diez ejes, fuentes fijadas, mecanismos, correcciones,
evidencia no leída y límites. Los hashes identifican exactamente los informes
leídos; cambios posteriores de autoría requieren releer el delta afectado y
su evidencia, no transferir este veredicto automáticamente.

El responsable decide correcciones y aceptación final; esta revisión no cambia
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md), informes de autoría ni contratos.
No se propone ejecutar como requisito universal ni editar harness/prompts/
rúbricas/scoring/umbrales/configuraciones para resolver estos hallazgos.
No se acreditó un nuevo defecto de Vanta por analogía con upstream.

**Operaciones realizadas:** lecturas locales (`sed`/`nl`/`cat`/`grep` y
lectura de salidas truncadas), `git rev-parse`/`git show` sólo lectura para
identidad/evidencia congelada, SHA-256 de informes y consultas web públicas
de cuerpos/conversaciones. Edición exclusivamente de este archivo mediante
patches menores de 300 líneas. TOC incorporado al superar 300 líneas.
Sin comandos de validación dinámica, apps/tests/builds/benchmarks,
instalaciones, servicios, arranques/paradas, git writes ni delegación anidada.

**Límite de cierre:** no certifica ejecución/crash/durabilidad remota,
seguridad completa de runtimes/ACL, UI visual, calidad semántica ni métricas
comparativas. Las comprobaciones adversas descritas siguen siendo futuras y
manuales, sin resultados atribuidos a esta revisión.
