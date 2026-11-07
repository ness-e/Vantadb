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
de cada herramienta ni exactly-once. **Issues: no consultados**.
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
