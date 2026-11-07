---
title: "AutoGen: interfaz de memoria y persistencia de estado separadas"
kind: research
status: proposed
description: "Lectura del AutoGen historico, Memory/ListMemory/RAG, contexto y AgBench sin confundirlo con Microsoft Agent Framework"
tags: [research, memory, engineering]
---

# AutoGen: interfaz de memoria y persistencia de estado separadas

## Indice

- [1. Dictamen y estado del producto](#1-dictamen-y-estado-del-producto)
- [2. Flujo y fichas candidatas](#2-flujo-y-fichas-candidatas)
- [3. Riesgos e issues](#3-riesgos-e-issues)
- [4. Prerrequisitos](#4-prerrequisitos)
- [5. No trasladar](#5-no-trasladar)
- [6. Evidencia y cobertura](#6-evidencia-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz](#10-interfaz)
- [11. Arquitectura ampliada, contratos y cierre](#11-arquitectura-ampliada-contratos-y-cierre)

## 1. Dictamen y estado del producto

**Propuesta:** estudiar un adapter pequeño de `Memory`, no adoptar el runtime
historico ni presentar al sucesor como si fuera el codigo inspeccionado.

- Repo `microsoft/autogen`, `C:\Users\Eros\autogen`.
- HEAD local `d07c667a3239ea4ab7af62a53b0f4ff205a1f23d`.
- **61 738 estrellas**, licencia detectada **CC-BY-4.0**, `archived=false`
  en [metadata congelada](github-candidates.json).
- **Discrepancia explicita:** el README de ese HEAD declara archivado/no
  mantenido, sin bug/security fixes, y recomienda Microsoft Agent Framework.
  No se modifica el snapshot ni se infiere estado actual de la API desde
  un badge. Se estudia AutoGen, no el codigo del sucesor.
- Licencias locales: `LICENSE` contiene CC BY 4.0; `LICENSE-CODE` contiene
  MIT. Revisar artefacto/archivo: el identificador del inventario no resume
  todas las licencias del monorepo.
- Checkout verificado: HEAD resoluble, status tracked limpio y fuentes
  presentes; ningun codigo/dependencia ejecutado.

Stack observado: Python, Pydantic, `autogen-core` con mensajes/contexto y
`Memory`; `autogen-agentchat` con agentes; `autogen-ext` con adapters RAG;
Studio web React/Gatsby; AgBench separado. No es un motor embebido Rust.

## 2. Flujo y fichas candidatas

### Flujo real

`AssistantAgent -> Memory.update_context(model_context) -> query -> insercion
de contexto -> MemoryQueryEvent -> modelo`. El contrato `Memory` tambien
expone add, clear y close [G1]. No exige extraccion/consolidacion automatica.
`ListMemory.query` ignora consulta/cancelacion y devuelve todo; `update_context`
añade todas las memorias como `SystemMessage` cronologico [G2].

`ChromaDBVectorMemory` consulta a partir del ultimo mensaje, obtiene k resultados,
convierte distancias a scores y conserva IDs/score en metadata [G4].
El add genera UUID: reintentar no es dedup por identidad de origen.
El modelo puede consultar/integrar memorias, pero guardar contexto de agente
es otra operacion: `AssistantAgent.save_state` guarda `llm_context`, y
`load_state` lo restaura; no exporta automaticamente la DB de memoria [G3].
`on_reset` limpia contexto, no necesariamente el store.

### Ficha A: adapter Memory sin apropiarse del context engine

- **Objetivo:** conectar Vanta como store sin crear AutoGen en Rust.
- **Unidad minima:** `Memory`/`MemoryContent`/`MemoryQueryResult`,
  26-130 [G1], y `ChromaDBVectorMemory.update_context/query`,
  310-333 y 355-413 [G4].
- **Seam:** binding Python y retrieval existente BM25/HNSW/RRF;
  `vanta-memory/src/context_engine/engine.rs::assemble_with_recall`,
  para que el adapter no duplique budgets.
- **Contrato Rust propuesto:** `RecallEnvelope { query, tenant, scope,
  evidence, score_kind, budget_consumed }`; el adapter devuelve
  `MemoryContent.metadata` con IDs/versiones y una proyeccion contextual.
- **Pasos/dependencias:** fijar version archivada soportada; mapear MIME;
  separar query de render; validar scope antes de retrieval; preservar raw
  IDs y provenance; declarar operaciones de lifecycle soportadas.
- **FMEA:** contenido hostil promovido a SystemMessage, cancelacion ignorada,
  duplicacion de memoria inyectada y filtro solo despues de k.
  El modelo no autoriza scopes ni cambia jerarquia de confianza.
- **Aceptacion:** consulta aislada por tenant; resultados trazables; contexto
  respeta budget global y rol de dato; MIME no soportado falla explicitamente;
  retries de add conservan identidad si el caller aporta operation ID.
- **No objetivos:** portar runtime, convertir todos los hechos en instrucciones.

### Ficha B: manifiesto de sesion con referencias a memoria

- **Objetivo:** que guardar/restaurar agente no parezca backup de memoria.
- **Unidad minima:** `AssistantAgent.save_state/load_state`, 1630-1639 [G3],
  `ChatCompletionContext.save_state/load_state`, 66-74 [G5].
- **Seam:** sesiones, provenance y task checkpoints ya presentes en Vanta.
- **Contrato Rust propuesto:** `SessionManifest { schema_version,
  conversation_revision, memory_scope_refs, checkpoint_ref }`; snapshot
  del contexto y referencias a store se validan por separado.
- **Pasos/dependencias:** inventariar ownership de cada recurso; decidir
  que se exporta y que solo se referencia; registrar revisiones; restaurar
  con validacion de tenant/schema; diagnosticar referencias faltantes.
- **FMEA:** contexto antiguo con memoria nueva, referencias borradas,
  restauracion que pisa nuevas escrituras, backup que aparenta incluir una
  DB externa; no se infiere atomicidad entre componentes.
- **Aceptacion:** restaurar no cruza scope; ausencia/version incompatible es
  error visible; clear de conversacion no elimina hechos salvo orden explicita;
  export distingue incluidos/referenciados/omitidos.
- **No objetivos:** checkpoint distribuido exactly-once ni nuevo backup engine.

### Ficha C: budget y reinyeccion idempotente

- **Objetivo:** impedir que el numero de stores multiplique silenciosamente
  tokens o memoria repetida en contexto.
- **Unidad minima:** `ListMemory.update_context`, 104-129 [G2],
  `AssistantAgent._update_model_context_with_memory`, 1028-1053 [G3],
  `TokenLimitedChatCompletionContext.get_messages`, 57-77 [G6].
- **Seam:** presupuesto compartido, compactacion, spill/dedup y reporte
  de context assembly existentes en Vanta.
- **Contrato Rust propuesto:** `InjectionKey { evidence_id, revision,
  placement }` y decision de reinyeccion dentro del reporte actual.
  El presupuesto considera tools, instrucciones y todas las memorias juntas.
- **Pasos/dependencias:** separar contexto crudo de vista; registrar IDs
  inyectados; agrupar llamadas/resultados; elegir prioridad bajo presupuesto;
  actualizar solo evidencia revisada, no append de todo el store.
- **FMEA:** append repetido, recorte de mensaje del medio que deja tool
  result huerfano, exposicion de instrucciones hostiles y falsa seguridad
  por limitar solo la vista. No copiar literalmente el algoritmo upstream.
- **Aceptacion:** dos assemble sin nueva evidencia no duplican IDs; vista
  valida para proveedor; originals intactos; budget efectivo y omisiones
  quedan explicados, sin afirmar calidad QA a partir de tokens ahorrados.
- **No objetivos:** nueva politica de benchmark ni resumidor obligatorio.

## 3. Riesgos e issues

| Nivel | Evidencia | Interpretacion |
|---|---|---|
| Documentacion local [G0] | README declara fin de mantenimiento | Desaconseja nueva dependencia estrategica del runtime |
| Codigo [G2] | ListMemory retorna todo e ignora query/cancelacion | No es RAG semantico ni almacenamiento durable |
| Codigo [G3] | save_state exporta contexto, no store de memoria | No equivale a backup de hechos |
| Codigo [G4] | add usa UUID nuevo | No ofrece idempotencia por fuente |
| Codigo [G6] | Vista elimina mensajes desde el medio hasta caber | No certifica coherencia de todos los grupos de tools |
| Hipotesis | Stores sucesivos consultan contexto ya mutado | Orden puede influir; requiere caso discriminante |

`UserProxyAgent` documenta bloqueo hasta respuesta y recomienda terminar/
devolver control para humanos lentos [G10]. Esto no prueba aprobacion durable
de cada herramienta ni exactly-once. Los issues públicos se amplían en §11;
la afirmación inicial «no consultados» ya no describe este informe.
No se traslada un riesgo condicionado a una vulnerabilidad demostrada.

## 4. Prerrequisitos

Revalidar Vanta actual: memoria, sesiones, checkpoints, scheduler,
supersession, dedup, provenance y shared budget ya existen segun
[METHOD](METHOD.md). Acordar scope, MIME, schema/version y semantica de
clear versus delete/close. Definir si el adapter archivado justifica coste
de soporte frente a integraciones vigentes. Una evaluacion del sucesor seria
otro alcance; no se presupone por recomendacion del README.

## 5. No trasladar

No adoptar todo ListMemory como memoria de produccion ni instrucciones del
modelo como ACL. No portar la arquitectura multiagente o ejecutores de
codigo al motor. No copiar trimming del medio sin validar causalidad.
No confundir autogen, forks o Microsoft Agent Framework.
Licencias mixtas: revisar codigo, documentacion y dependencias; conservar
notices/atribucion aplicables. Traduccion a Rust no exime de derivacion.

## 6. Evidencia y cobertura

Ledger de lectura acotada con URLs inmutables:

- [G0: README, declaracion de archivo y sucesor](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/README.md#L14-L31).
- [G1: Memory, 13-130](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-core/src/autogen_core/memory/_base_memory.py#L13-L130).
- [G2: ListMemory, 104-172](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-core/src/autogen_core/memory/_list_memory.py#L104-L172).
- [G3: AssistantAgent, 1028-1053 y 1626-1639](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/agents/_assistant_agent.py#L1028-L1053).
- [G4: ChromaDBVectorMemory, 304-419](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py#L304-L419).
- [G5: ChatCompletionContext, 49-74](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-core/src/autogen_core/model_context/_chat_completion_context.py#L49-L74).
- [G6: TokenLimitedChatCompletionContext, 19-94](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-core/src/autogen_core/model_context/_token_limited_chat_completion_context.py#L19-L94).
- [G7: agbench/run_cmd, 1-100](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/run_cmd.py#L1-L100).
- [G8: agbench/tabulate_cmd, 19-103](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/tabulate_cmd.py#L19-L103).
- [G9: GAIA loader, 1-105](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/benchmarks/GAIA/Scripts/init_tasks.py#L1-L105).
- [G10: UserProxyAgent, 37-56](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/agents/_user_proxy_agent.py#L37-L56).
- [G11: GAIA scorer, 16-90](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/benchmarks/GAIA/Scripts/custom_tabulate.py#L16-L90).
- [G12: Studio package.json, 1-61](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio/frontend/package.json#L1-L61).
- [Licencia del codigo](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/LICENSE-CODE)
  y [licencia raiz](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/LICENSE).

**No examinado:** Redis/Mem0/Canvas, .NET, runtime de equipos completo,
todos los ejecutores, seguridad de Studio, sucesor y cadena de dependencias.
**Pendiente manual:** aislamiento y budget del adapter, restauracion y
crash safety. No ejecucion de clones/tests/scripts ni nuevos servicios.

## 7. Benchmarks existentes

AgBench [G7] carga escenarios JSONL y templates, soporta Docker y modo nativo.
Importa Docker/subprocess/credenciales Azure y crea directorios de resultados.
El loader GAIA [G9] descarga `gaia-benchmark/GAIA` desde Hugging Face y genera
casos de validation/test por niveles, prompts y expected answers.
El tabulador base [G8] busca marcadores de exito/completado y runtime en logs;
devuelve `None` si no completo. El scorer GAIA [G11] normaliza strings/numeros;
es distinta pista de exito de agente/respuesta, no recall de memoria.

Dependencias: dataset/credenciales, pandas, Docker o host, modelos y herramientas
de los escenarios. Riesgos: codigo generado, red, gasto, escrituras, secretos en
ENV/logs; **no usar modo nativo como prueba inocua**. No se ejecuto.
No hay score local y cifras del proveedor no cuentan como reproduccion.
Comparacion futura solo segun [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks);
no se crea runner, sampling, rubric, grader ni configuracion.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Contraste |
|---|---|---|
| BM25/HNSW/RRF, provenance | Existe | Mejor backend posible, no nuevo indice |
| Sesiones/checkpoints | Existe | Documentar separacion del store |
| Context assembly/compaction/shared budget | Existe | Adapter debe consumirlo, no append ilimitado |
| Dedup/supersession/scheduler | Existe | Evitar UUID nuevo por retry como unico contrato |
| Binding AutoGen Memory | No acreditado aqui | Propuesta opcional, no prioridad por estrellas |
| Replay exactly-once de herramientas | No acreditado | save_state tampoco lo prueba |
| Runtime AutoGen embebido | No aplicable | Riesgo de dependencia archivada |

## 9. Debilidades sugeridas

El contraste motiva hacer visible que un backup de conversacion no incluye
necesariamente toda memoria externa, y mostrar reinyecciones/provenance y
tokens por fuente. No demuestra ausencia de esas capacidades en Vanta.
Revalidar codigo actual antes de formular un hallazgo o tarea.

## 10. Interfaz

**Disponible localmente:** fuente de AutoGen Studio en
`python/packages/autogen-studio/frontend`, con React/Gatsby y paginas
build/deploy/gallery; [G12] acredita stack, no funcionamiento o seguridad.
Es UI web, no escritorio nativo; **escritorio nativo no localizado**.
No se inicio Studio. Tampoco se atribuye al sucesor su UI.
Vanta ya tiene Studio Tauri en `develop-desktop`; no hace falta clonarlo.

## 11. Arquitectura ampliada, contratos y cierre

### 11.1 Base fija, mapa y tres recorridos

Ampliación de autoría 2026-11-07; preserva G0–G12 y A–C. Base comparativa:
**Vanta `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**; no el historial
c561158 ni futuros cambios concurrentes. [VANTA-REFERENCE](VANTA-REFERENCE.md),
[METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md) son contratos.
AutoGen conserva SHA `d07c667a3239ea4ab7af62a53b0f4ff205a1f23d`, clon
superficial del manifiesto; no fetch/pull ni actualización de metadata.

```text
host Python → autogen-agentchat AssistantAgent / BaseGroupChat / Swarm
  → autogen-core runtime (AgentId, topics, queues, cancellation, tracing)
  → ChatCompletionContext → Memory.update_context → model/tools
  → save_state agentes/manager → mapping del host
autogen-ext → ChromaDBVectorMemory y proveedores/modelos/workbenches opcionales
autogen-studio → frontend Gatsby/React → FastAPI REST + WS → DB de runs/mensajes
agbench → escenarios JSONL/templates → Docker/host → logs → tabulador/scorer
```

**Modelos y dependencias.** `MemoryContent` contiene content tipado MIME y
metadata, `MemoryQueryResult` lista resultados, `UpdateContextResult` explica
la inyección [G1]. `AssistantAgentState.llm_context` no contiene todo backend
de Memory [G3]. `TeamState.agent_states` usa nombres portables, manager
incluido; Swarm guarda thread/current_turn/current_speaker [G13/G15].
Runtime guarda mapping por AgentId de agentes instanciados, **no suscripciones**
[G14]. Envelope/traza transporta causalidad, no es memoria de hechos.
`autogen-agentchat/pyproject.toml:1-36` fija Python >=3.10/core==0.7.5;
core/Pydantic y protocolos no hacen obligatorios Chroma/OpenAI/Studio/Docker.
Licencias: MIT para código `LICENSE-CODE`, CC BY 4.0 documento raíz; servicios,
modelos, datasets, imágenes y dependencias conservan condiciones propias.
No inferir licencia de datos/modelos desde MIT ni exención por portar a Rust.

**F1 captura/orquestación.** Host llama `BaseGroupChat.run_stream`; verifica
`_is_running`, inicializa runtime/topics y envía `GroupChatStart` con token
[G13:483-537]. Agente añade mensajes y ejecuta `_update_model_context_with_memory`
[G3] → cada store query/update → MemoryQueryEvent → modelo → tool.
Workbench recibe `call_id`/cancel token; retorna `FunctionExecutionResult`
con is_error y mismo ID [G3:1580-1624]. Herramienta desconocida da resultado
de error, no hecho verdadero. `ListMemory.add` modifica lista; Chroma add
usa UUID y garantías del backend [G2/G4]. El default
`PersistentChromaDBVectorMemoryConfig` abre **PersistentClient local**;
`HttpChromaDBVectorMemoryConfig` abre **HttpClient remoto**
([_chromadb.py:170-180](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py#L170-L180),
[246-262](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-ext/src/autogen_ext/memory/chromadb/_chromadb.py#L246-L262)).
Topología, I/O y frontera de commit dependen de esa selección. Ninguna
transacción envuelve tool externa, memoria, context y stream.
`GroupChatTermination.error` provoca **RuntimeError antes de TaskResult**;
sin error se rompe el bucle y se emite TaskResult; cancelación del future
propaga sin promesa de ese terminal. `finally` ejecuta cleanup incluso en fallo
([_base_group_chat.py:543-578](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_base_group_chat.py#L543-L578)).
El consumidor debe separar normal/error/cancelado; cleanup no es rollback
ni consumir un evento demuestra commit durable.

**F2 recall/resume.** Store consulta y renderiza evidencia antes de inferir;
`ListMemory` ignora query y cancelación y hace append SystemMessage [G2].
Chroma obtiene resultados/score desde último mensaje [G4], distinto de guardar
el contexto. Para reiniciar: host obtiene mapping de `save_state`, persiste
por cuenta propia y reconstruye agentes/modelos/memory; `load_state` valida
TeamState y rechaza equipo corriendo [G13:748-834]. Restore itera participantes
antes del manager; falta de un estado posterior puede dejar agentes previos
ya restaurados. No se acredita rollback global del restore. Runtime
save/load [G14:431-464] no guarda suscripciones ni reconstruye cada servicio.
Memorias externas nuevas/borradas pueden divergir del snapshot del equipo.

**F3 handoff/update/delete/cancel.** Swarm valida destino miembro, selecciona
último HandoffMessage y guarda speaker/thread [G15:47-112]; pasar control no
autoriza otro tenant ni realiza retrieval. Reset limpia thread/turno del
manager y `AssistantAgent.on_reset` limpia contexto, **no Chroma ni hechos**
[G3/G15]. `Memory.clear` es operación distinta y close libera recursos [G1/G2].
El contrato Memory base no promete update/delete individual: un adapter debe
declarar soporte, no fingirlo borrando todo. Cancelar run enlaza token a future
de salida; docstring advierte estado potencialmente inconsistente y recomienda
terminación cooperativa [G13:247-264,543-577]. Pause/resume RPC puede ser no-op
si participante no lo implementa [G13:657-746]. Reinicio no repite herramientas
ya aplicadas sin política del host; save_state no es WAL ni replay exactly-once.

### 11.2 Prioridad y complemento contractual de A–C

**P0 A** aislamiento y rol de evidencia; **P1 C** presupuesto/reinyección;
**P1 B/E** export/restauración explicables; **P2 D** handoff referenciado sólo
si hay consumidor. La condición de mantenimiento G0 desaconseja dependencia
estratégica nueva: comparar coste de adapter opcional contra soporte vigente,
sin estudiar aquí código del sucesor Microsoft Agent Framework.

Firmas Rust **PROPUESTAS**, preservando las formas válidas del borrador:

```rust
fn query_memory(req: RecallRequest, cancel: &CancelToken)
    -> Result<RecallEnvelope, AdapterError>;
fn capture_memory(req: CaptureRequest, op: OpId, cancel: &CancelToken)
    -> Result<MutationObservation, AdapterError>;
fn export_session(req: ExportRequest) -> Result<SessionManifest, AdapterError>;
fn restore_session(m: SessionManifest, expected: Revision)
    -> Result<RestoreReport, AdapterError>;
fn plan_injection(view: &RecallEnvelope, context: &ContextSnapshot)
    -> Result<InjectionPlan, AdapterError>;
```

Inputs propuestos incluyen actor autenticado, scope validado, IDs/revisiones,
MIME y budget existente. `RecallEnvelope` conserva score_kind y provenance,
no declara distancia homogénea entre backends. `InjectionKey` usa ID/revisión/
placement, no igualdad textual. Errores distinguen `ScopeDenied`,
`UnsupportedMime`, `StaleRevision`, `MissingReference`, `CancelledBeforeWrite`
y `OutcomeUnknown`; cancelación posterior a commit no significa NotApplied.

| Campo complementario | A adapter Memory | B SessionManifest | C InjectionKey/budget |
|---|---|---|---|
| Delta real | Traducir contrato Memory a binding existente | Distinguir incluido/referenciado/omitido en export | Explicar reinyección y omisiones por evidencia |
| Alternativa/decisión | Adapter Python pequeño; no nuevo runtime Rust ni índice | Manifiesto refs antes que backup distribuido | Extender assembler/report actual, no segundo tokenizador |
| Seam real | `integrations/vantadb_shared/__init__.py::EmbeddingVectorStore`, binding Python, NV1 | `integrations/langchain/vantadb_langchain/checkpointer.py::VantaDBCheckpointer`, `TaskCheckpointManager`, NV2/NV4 | `assemble_with_recall`, `IntegratedContext/CompactionReport`, NV3 |
| Persistencia/pasos | Mapear MIME→autorizar antes de top-k→captura op_id→consultar→render gobernado | Inventariar owners→serializar versión→persistir refs→prevalidar todo→restore explícito | Vista separada→identificar grupos tool→elegir con budget global→emitir reporte |
| Migración/rollback | Metadata opcional versionada, legacy Unknown; quitar adapter no borra hechos | No declarar DB externa incluida; legado sin refs marcado incompleto; rollback no pisa nuevos hechos | Sin reindex; reiniciar ledger de vista por generación, no reescribir raw |
| Dependencias | Scope/provenance/budgets existentes; independiente de B | A si incluye memory refs, E para serialización; sin scheduler propio | A aporta IDs, B aporta generación; no exige export de todo store |
| FMEA | Reintento UUID→duplicado→op_id; filtro tardío→escape/underfill→scope previo | Snapshot live→mezcla de generaciones→quiescencia/preflight; referencia borrada→error visible | Append repetido→inflación→InjectionKey; recorte tool huérfano→rechazo de grupo |
| Aceptación futura | MIME normal trazable; tenant ajeno negado; cancel/timeout conserva estado conocido | Restore normal conserva ownership; faltante/schema incompatible falla antes de alterar; no promete atomicidad remota | Dos ensamblajes sin cambios no duplican; budget/protected-prefix explicados; tool-call/result no se separan |
| Observabilidad | IDs/revisiones/código/latencia, sin texto | Incluidos/refs/faltantes/schema, sin volcar conversación | Conteos/budget/omisión/revisión, sin payload |
| Mínimo/no-goals/abierto | Sólo Memory; no runtime; abierto backend soportado | Manifiesto, no backup engine; abierto snapshot inter-recurso | Vista, no nueva compaction; abierto accounting por provider |

`WriteReceipt` **ya existe** en NV5: SHA-256/relectura, no firma digital ni
prueba de commit de Chroma/tools. `MutationObservation`/MutationReceipt C13
son **PROPUESTOS**; no confundir aceptación/aplicación/durabilidad/visibilidad.
No otro WAL, scheduler, índice ni política de scoring.

### 11.3 D — Referencias de memoria en handoff sin transferencia de privilegios

1. **Problema agente:** especialista recibe control y debe saber qué evidencia
   autorizada sustentaba tarea sin duplicar transcript o ampliar audiencia.
2. **Fuente/unidad:** `SwarmGroupChatManager.validate_group_state/select_speaker/
   save_state` [G15:47-112], no portar su selección de speakers.
3. **Delta:** paquete de refs/revisiones sobre handoff del host, no runtime.
4. **Alternativas:** transcript íntegro barato pero expone más; re-recall evita
   copia pero cambia evidencia. Decisión: refs autorizadas con missing explícito.
5. **Rust propuesto:** `prepare_handoff(actor: &Actor, target: AgentRef,
   refs: &[VersionedRef], rev: Revision) -> Result<HandoffMemoryRefs, AdapterError>`;
   errores ScopeDenied/UnknownTarget/StaleReference. Destino no obtiene ACL por ID.
6. **Seam:** `auto_recall.rs::InjectionPolicy/perform_auto_recall_governed` NV4;
   checkpoint actual conserva sólo referencia al paquete, no tareas nuevas.
7. **Pasos:** identificar refs→validar actor/destino→persistir versión en
   checkpoint→reautorizar al consumir→resolver refs→assemble existente.
   Legado sin refs significa no suministrado; desactivar no borra memoria.
8. **Dependencias:** A/B para IDs/generación; independiente de consolidación.
9. **FMEA:** target malicioso pide scope mayor→denegación; evidencia sustituida
   entre entrega/lectura→StaleReference; retry duplica contexto→InjectionKey C.
   Detectar sólo IDs/decisiones, mitigar con ACL vigente y provenance.
10. **Aceptación futura:** handoff normal usa mismas revisiones; revocación
    impide leer; delete/supersession no resucita fuente; cancelación prepublicación
    no crea paquete visible, después no revierte herramienta.
11. **Mínimo/no objetivos/abierto:** refs opt-in; no scheduler ni transferencia
    automática de IAM; abierto quién decide compatibilidad del destino.

### 11.4 E — Export tipado prevalidado

1. **Problema:** export parece exitoso pero archivo queda truncado por datetime/
   enum; restore parcial modifica agentes antes de descubrir recurso faltante.
2. **Fuente:** G13:748-834, `MemoryContent.serialize_mime_type` G1:40-45,
   `BaseMessage.dump` G16; incidencias concretas abajo.
3. **Delta:** preflight y manifest de schema/recursos en export existente.
4. **Alternativas:** pickle/jsonpickle acoplan clases y son inapropiados para
   entrada no confiable; preferir formatos tipados del host, no importar Python.
5. **Rust PROPUESTO:** `validate_export(m: &SessionManifest,
   resources: &ResourceInventory) -> Result<ValidatedManifest, ExportError>`;
   `encode_export(v: ValidatedManifest) -> Result<Vec<u8>, ExportError>`.
   Errores UnsupportedSchema/Unserializable/MissingResource/ScopeDenied.
   Validación no asegura atomicidad de restore ni verdad del contenido.
6. **Seam real:** NV2 `VantaDBCheckpointer`, NV4 TaskCheckpointManager y bindings.
7. **Pasos:** inventario→validar todos refs/schema→serializar en memoria/buffer
   controlado→publicar con mecanismo de archivo existente→restore tras preflight.
   Versionar envelope, conservar importador previo; no migrar raw a pickle.
8. **Dependencias:** B manifiesto, no D ni modelos; sin nuevo motor de backup.
9. **FMEA:** dato nuevo no serializable→export falla antes de sustituir archivo;
   load parcial→reportar recursos aplicados, no éxito global; schema hostil→
   parse tipado/size limit existente. Detectar código/schema, no contenido.
10. **Aceptación futura:** fechas/MIME roundtrip; schema desconocido no altera
    estado; snapshot corriendo declara consistencia no acreditada.
11. **Mínimo/no-goals/abierto:** preflight/compatibilidad, no snapshot distribuido;
    abierto contrato de quiescencia del host.

### 11.5 Issues públicos, revisiones y correcciones acreditadas

Leídos HTML/cuerpo/conversación el **2026-11-07** tras búsqueda web sin resultados
útiles y fallback al buscador de issues GitHub; no afirmar fallback agotado.

| Incidencia | Fecha/estado/revisión reportados | Causa, contraste y decisión |
|---|---|---|
| [#6793](https://github.com/microsoft/autogen/issues/6793), JSON dumping of save state on teams fails because of datateimes | 2025-07-11; Closed 2025-07-15; Python 0.6.4 | Datetime `created_at` no serializable; hilo atribuye arreglo a #6797. G16 en pin usa `model_dump(mode="json")`: no presentar como bug actual. No se inspeccionó diff/merge de PR independientemente |
| [#5688](https://github.com/microsoft/autogen/issues/5688), MemoryMimeType not JSON serializable | 2025-02-24; Closed 2025-02-26; main dev sin SHA exacto | Hilo enlaza #5727/dd0781a; G1 en pin tiene field_serializer de MIME. E protege evolución, no vuelve a arreglar enum ya cubierto |

Cerrar issue no basta para acreditar arreglo universal: aquí se contrasta
el mecanismo concreto con código fijado, sin certificar todos los tipos.
No copiar workaround `str(state)`+ast ni deserialización ejecutable.
No se acredita CVE ni vulnerabilidad de Studio desde estos reportes.

### 11.6 AgBench completo en la frontera de métricas

Lectura adicional G17–G19: CLI `run_cli:869-962` default repeat=1/parallel=1,
subsample=None, Docker por defecto; nativo advertido y rechazado en Windows.
No cambiar esos defaults ni proponer configuración nueva. JSONL por tarea
contiene id/template/substitutions; GAIA loader [G9:25-105] descarga dataset,
separa validation/test/niveles, escribe pregunta y expected_answer por separado.
No se auditó cada escenario para certificar que agente no accede a gold.
Modelo/embedding/tokenizer/seed/cache/budget dependen del template/config,
no hay valor universal acreditado.

GAIA `scorer:127-158` busca `FINAL ANSWER:` y usa normalización numérica/listas/
strings, sin juez LLM; falta archivo/answer devuelve None. Tabulador
`default_tabulate:148-255` distingue True/False/None y calcula tasa
**successes/(successes+failures)**, excluyendo missing de ese denominador
aunque imprime Missing/Total. Tiempo medio excluye null. No comparar esa
tasa con intentadas totales ni con recall@k. Sus any/all entre repeticiones
no son recall-any/all de sesiones LongMemEval. No se oculta esa diferencia.
**Universo descubierto, no censo autorizado externo:** `tabulate_cmd.py:
127-191` enumera directorios existentes de runlogs y sus repeticiones; las
filas de tarea sólo nacen de esos directorios. Padding agrega None para
repeticiones de filas presentes, **no para una tarea sin directorio**.
`214-261` agrega tarea×repetición representada, sin reconciliar el JSONL
planificado. Missing cuenta huecos conocidos en esa tabla, no todos los
intentos ausentes; Total tampoco acredita cobertura de lo planificado.
Fallo anterior a crear directorio puede quedar fuera de toda la tabla
(sesgo de supervivencia), sin invalidar globalmente GAIA.
Fuente inmutable:
[tabulate_cmd.py:127-261](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/tabulate_cmd.py#L127-L261).
Aceptación documental futura: distinguir directorio con None, repetición
ausente y tarea enteramente no representada contra inventario autorizado.
No se cambia fórmula, scorer, denominador, configuración ni harness.
Reset: carpetas por tarea/repetición y Docker proporciona entorno controlado;
modo nativo puede contaminar siguientes intentos con instalaciones/archivos.
No acredita reset de stores externos/servicios ni revoca efectos pagados.

Pytest declarado en pyproject es prueba funcional, **no AgBench** ni benchmark
de memoria. Adaptación futura: adapter A consume Memory y conserva baseline
agente/modelo/herramientas/scorer del escenario, bajo protocolo METHOD;
no portar ejecutor Docker al motor ni crear harness/prompts/rúbricas/umbrales.
Efectos: descargas, containers, código generado, DB/logs, llamadas pagadas y
secretos en entorno; no ejecutar aquí. No hay score propio ni superioridad.

### 11.7 UI real, matriz y ledger de cierre

Studio tiene código web local, no shell desktop acreditado. Flujo leído:
`SessionAPI.createSession/listSessions/getSessionRuns/deleteSession` [G20]
→ REST FastAPI; `create_run` [G21] vincula sesión/user y status CREATED →
WS `/runs/{run_id}` [G22] verifica run/status, autentica **si auth_manager
habilitado**, comprueba owner/admin y atiende start/stop/input_response.
`start` lanza tarea del manager; `stop` llama `stop_run`; desconexión pasa
por disconnect. No acreditar rollback de tools ni cierre durable desde ese
finally. REST get_run filtrado por ID en ese módulo no permite concluir fuga:
faltan middleware/política global. Tampoco trasladar user_id de cliente como ACL.
No se ejecutó frontend ni se certificó su seguridad.

Oportunidad Studio Vanta **existente**, no clon de teambuilder:
pin `0ad38c542b39a589035b0e17fdedb4c09dca24b9`, cuerpos releídos con git show,
no rangos del worktree:
[ActivityPanel:1-65](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L1-L65)
expone audit paginado/estado no configurado;
[MemoryLens:57-412](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412)
usa ScenesPanel/PersonaPanel/SkillsPanel/GenlogPanel y get del anchor;
[RetrievalLens:1-130](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L130)
muestra scores/filtros. G-1: UI propuesta distingue error/cancel de TaskResult
normal y no usa cleanup como commit. Añadir included/referenced/omitted, generación y decisiones de
inyección a estas superficies; no confundir genlog/traces con ledger de entrega.

| Capacidad | Upstream, estado/fuente | Vanta 97c9623, estado/fuente | Delta/decisión |
|---|---|---|---|
| Hechos y memoria conversacional | Distintos, G1–G5 | Existe, NV1/NV4 | Adapter A mantiene distinción |
| Snapshot de equipo/runtime | Parcial, G13/G14 sin suscripciones | Checkpoints existentes NV2/NV4 | B/E describen recursos, no portar runtime |
| Reinyección/budget global | ListMemory append total G2; contexto limitado G6 | Existe, NV3/NV4 | C adapta identidad, no nuevo assembler |
| Handoff con destino | Existe, G15 | Governance de recall existente NV4; adapter AutoGen no acreditado | D opcional y reautorizado |
| BM25/HNSW/RRF y cuotas | No provistos por protocolo Memory G1 | Existen, VANTA-REFERENCE | Usar backend, no recrearlos |
| Dedup/supersession/provenance | UUID por add G4 no garantiza idempotencia | Existe L1 NV4 | Reutilizar al capturar |
| Recibo | Retornos Memory no certifican durabilidad G1 | Existe WriteReceipt NV5 | No llamarlo firma ni inventar garantía remota |
| UI | Web local G20–G22 | Studio desktop existente | Datos en lentes existentes |

**Negativos Vanta:** ninguno nuevo demostrado. Lagunas concretas: mapping de
Memory/MIME, export inter-recurso y ledger de reinyección no quedaron acreditados
por estas lecturas. Son hipótesis a discriminar, no defectos. No reaprovechar
#6793/#5688 como bugs presentes ni acusar falta de budget, checkpoints, scheduler,
quotas, dedup, provenance o integraciones LangChain ya existentes.
El prefijo protegido puede exceder budget (NV3), excepción documentada, no
razón para prometer límite de tokens absoluto desde un contador de caracteres.

Fuentes nuevas **leídas**; todas upstream fijadas al SHA del informe:

- [G13, BaseGroupChat:483-577](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_base_group_chat.py#L483-L577), además `247-264,657-834`, run/cancel/pause/save/load.
- [G14, runtime:431-464](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-core/src/autogen_core/_single_threaded_agent_runtime.py#L431-L464), tracer config `253-256`.
- [G15, Swarm manager:47-112](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_swarm_group_chat.py#L47-L112).
- [G16, BaseMessage.dump:45-57](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-agentchat/src/autogen_agentchat/messages.py#L45-L57).
- [G17, AgBench CLI:869-962](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/run_cmd.py#L869-L962).
- [G18, agregación:148-255](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/src/agbench/tabulate_cmd.py#L148-L255).
- [G19, scorer:127-162](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/agbench/benchmarks/GAIA/Scripts/custom_tabulate.py#L127-L162).
- [G20, SessionAPI:1-115](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio/frontend/src/components/views/playground/api.ts#L1-L115).
- [G21, runs REST:13-65](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio/autogenstudio/web/routes/runs.py#L13-L65).
- [G22, websocket:20-132](https://github.com/microsoft/autogen/blob/d07c667a3239ea4ab7af62a53b0f4ff205a1f23d/python/packages/autogen-studio/autogenstudio/web/routes/ws.py#L20-L132).
- [NV1, EmbeddingVectorStore:31-163](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/vantadb_shared/__init__.py#L31-L163).
- [NV2, VantaDBCheckpointer:232-296](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L232-L296).
- [NV3, assemble_with_recall:252-326](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L326).
- [NV4, InjectionPolicy/RecallConfig:133-335](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L133-L335); también `core/record/l1_writer.rs::persist_planned/log_and_mark:289-330`, `utils/task_checkpoint.rs::TaskCheckpointManager:122-125`.
- [NV5, WriteReceipt y límites:678-745](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L678-L745).

**Cierre propio:** G-1/G-2 precisados en F1, §11.6 y UI: topología/terminal
y universo descubierto no se convierten en garantías de commit/censo completo.
A–E desarrollados, sin autoaceptación ni revisión independiente.
No leído: .NET, runtime distribuido completo, todos workbenches/DB/auth middleware,
sucesor, todos templates GAIA y datos PDF/descargas del dataset; no se afirma
lectura exhaustiva ni crash-safety medida. Grafo Vanta sólo lectura; sin índices
nuevos. Validaciones normales/adversas siguen manuales, no se ejecutaron.
Por >300 líneas, futura fase aprobada separaría specs adapter/inyección y
manifest/restore, conservando aquí investigación y fuentes. No se crean specs,
código, contratos activos, backlog, commits o servicios.
