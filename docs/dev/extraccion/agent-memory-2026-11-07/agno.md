---
title: "Agno: memoria extraida, aprendizaje y sesiones"
kind: research
status: proposed
description: "Contraste de MemoryManager, LearningStore, scopes y consolidacion con la memoria y el contexto existentes en VantaDB"
tags: [research, memory, engineering]
---

# Agno: memoria extraida, aprendizaje y sesiones

## Indice

- [1. Recomendacion y arquitectura](#1-recomendacion-y-arquitectura)
- [2. Flujo y fichas candidatas](#2-flujo-y-fichas-candidatas)
- [3. Riesgos e issues](#3-riesgos-e-issues)
- [4. Prerrequisitos](#4-prerrequisitos)
- [5. No trasladar](#5-no-trasladar)
- [6. Evidencia y cobertura](#6-evidencia-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz](#10-interfaz)
- [11. Cierre de contratos y evidencia](#11-cierre-de-contratos-y-evidencia)

## 1. Recomendacion y arquitectura

**Propuesta:** estudiar scopes y consolidacion versionada, manteniendo fuera
del motor el procesamiento LLM. No adoptar la plataforma de agentes completa.

- Repo `agno-agi/agno`, `C:\Users\Eros\agno`.
- HEAD `f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68`, verificado localmente.
- **43 089 estrellas**, Apache-2.0, no archivado en el
  [snapshot congelado](github-candidates.json).
- Al comenzar: HEAD resoluble, archivos presentes y status tracked limpio.
- Stack observado: Python, dataclasses/Pydantic, modelos enchufables,
  protocolos DB sync/async, agentes/evaluaciones y rutas AgentOS.

Hay al menos dos superficies de memoria que conviven en este HEAD:
`MemoryManager` y `learn/stores/*`. No inferir que un ejemplo de una configura
automaticamente la otra. `AgentSession` guarda runs, datos de sesion, estado y
summary; `UserMemory` guarda hechos extraidos y metadatos de usuario/agente.
La sesion conversacional no es el indice de memoria ni una garantia de replay.

### Mapa, modelos y fronteras

Ampliacion de autoria del 2026-11-07, no revision independiente ni aceptacion.
La base comparativa es **VantaDB `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**,
no las revisiones historicas del estudio inicial; consultar
[VANTA-REFERENCE](VANTA-REFERENCE.md). La metadata de estrellas queda congelada.

| Capa | Codigo en el SHA Agno anterior | Modelo / frontera |
|---|---|---|
| API Python de memoria | `libs/agno/agno/memory/manager.py` [A1, A10] | Crear desde mensajes, CRUD explicito, seleccion y optimizacion |
| Aprendizaje | `learn/stores/user_memory.py`, `entity_memory.py` [A3, A4] | Procesar experiencias y recuperar hechos; no son el mismo store |
| Modelo de hechos | `db/schemas/memory.py:8-57` [A5] | `UserMemory`: texto, UUID opcional, topics, user/agent/team, input, feedback, timestamps |
| Conversacion | `session/agent.py:15-65` [A6] | `AgentSession`: runs, estado/datos y summary; unidad distinta del hecho |
| Persistencia | `db/base.py`, `db/sqlite/sqlite.py` [A9, A11] | Contratos BaseDb/AsyncBaseDb y SQLAlchemy; commit del adapter, no del modelo |
| Entidades | `EntityMemoryStore.recall/_novel_facts` [A4, A12] | Namespace, identidad y tipo; hechos vivos, archivo y relaciones |
| Knowledge | `knowledge/`, `vectordb/` | Superficie separada; no auditada integralmente ni equiparada a UserMemory |
| HTTP/AgentOS | `os/routers/memory/memory.py` [A13] | FastAPI; scope resuelto desde request, DB local/async/remota |
| Evaluacion | `eval/performance.py`, cookbook [A7, A8] | Instanciacion y asignaciones Python, no calidad de recuerdo |

Stack concreto: `libs/agno/pyproject.toml:1-85` [A14] declara Python
`>=3.9,<4`, Pydantic/settings, httpx, h11, rich y typing-extensions.
AgentOS agrega FastAPI/Uvicorn, SQLAlchemy async, JWT y telemetria como extras.
Los proveedores de modelo y sus SDKs son otra dependencia: leer/escribir un
hecho programaticamente no necesita inferencia; extraer, seleccionar de modo
`agentic` y resumir si la necesitan. No convertir ese coste en requisito del
motor Vanta. SQLite es el backend recorrido, no prueba de paridad con Postgres,
Mongo, bases remotas ni con todos los modos de aprendizaje.

**Semantica:** session conserva conversacion; checkpoint conserva avance;
replay reanuda/repite ejecucion; handoff cambia ejecutor/contexto;
retrieval selecciona evidencia y memory contiene hechos/experiencias.
Ninguna de las cinco primeras operaciones sustituye un indice de retrieval.
Guardar estado no garantiza efectos externos exactamente una vez: una
herramienta puede haber cobrado/enviado antes de perderse la confirmacion.
No se audito aqui la implementacion durable completa anunciada por AgentOS.

### Orden de oportunidades y deuda evitada

1. **P0, seguridad del adapter:** B y E: identidad explicita y capacidad real
   antes de exponer herramientas destructivas. Los issues leidos justifican
   esta prioridad, no una nueva DB ni un nuevo runtime.
2. **P1, integridad del ciclo de vida:** A, conservar originales y publicar
   reemplazo condicionado; C solo si el producto necesita aprobacion de hechos.
3. **P2, contexto explicable:** D, directorio acotado de entidades con
   cobertura declarada; extender retrieval/assembler, no construir ANN propio.

Antes de proponer otra integracion se leyeron en la base Vanta
`integrations/openai/vantadb_openai/vectorstore.py:15-64` y
`integrations/vantadb_shared/__init__.py:31-163` [AV1, AV2].
`VantaDBOpenAI` ya aporta embeddings y almacenamiento compartido;
**no es** un BaseDb Agno ni un contrato de aprobaciones.
`integrations/langchain/vantadb_langchain/checkpointer.py:232-296` y
`store.py:39-65,97-177` [AV3, AV4] ya separan checkpoints y store.
La recomendacion es extender binding/adapter existente si se aprueba el delta;
no portar AgentOS ni presentar esos adapters como aislamiento multiusuario
automatico. Un namespace elegido por el caller no demuestra autorizacion.

## 2. Flujo y fichas candidatas

### Flujo real

En [A1], `create_user_memories` normaliza mensajes, usa usuario `"default"`
si falta identidad, lee memorias existentes, pide `create_or_update_memories`
y refresca desde DB. `search_user_memories` ofrece `last_n`, `first_n` o
`agentic`; esta ultima seleccion usa modelo, no implica indice ANN.
`optimize_memories` delega estrategia y, con `apply=True`, limpia memorias
del usuario y hace upsert de las optimizadas.

En [A3], `UserMemoryStore.process` solo extrae automaticamente en modo
`ALWAYS`; `recall` obtiene por usuario; `extract_and_save` forma texto de
conversacion y herramientas de extraccion para el modelo. El propio contrato
aclara que agent/team son auditoria, no aislamiento de memorias del usuario.
`PROPOSE`/`HITL` emiten advertencia de no soporte en este store.

En [A4], `EntityMemoryStore` separa namespace de usuario/compartido,
recall de directorio y entidades expandidas; `_novel_facts` descarta duplicados
exactos antes de la posible decision de supersession. No se auditó todo el
grafo de relaciones ni todos sus adapters.

### Tres recorridos completos y caminos de fallo

**F1 — escritura desde experiencia.** Caller identifica usuario y entrega
`Message` → `MemoryManager.create_user_memories:377-430` valida DB/modo e
input → `read_from_db` obtiene hechos → `create_or_update_memories` construye
las tools → modelo propone operaciones → closure `add_memory/update_memory`
en `_get_db_tools:1343-1452` → `SqliteDb.upsert_user_memory:2672-2741`
realiza INSERT/ON CONFLICT y termina `sess.begin()` → refresco desde DB.
Persistir cada hecho no es commit atomico de toda la extraccion.
El UUID identifica el hecho, no necesariamente la experiencia ni el intento.
Si falta DB devuelve texto de advertencia; input invalido y DB async en la
ruta sync producen ValueError; closures capturan ciertos errores como texto.
Una respuesta del modelo o UUID devuelto no certifica persistencia.
El caller es dueño del trabajo LLM; no se identifico en este recorrido
cancelacion transaccional de todos los efectos ni version CAS de la lectura.
Dos extracciones pueden decidir sobre el mismo conjunto previo.
Derivados: hechos/topics/input; eventos y summary de sesion no se borran
automaticamente por este upsert.

**F2 — consulta y entrega contextual.** Caller → `search_user_memories`
(`manager.py:603-650`) → modo `last_n/first_n` consulta DB ordenada, o
`_search_user_memories_agentic:666-739` selecciona mediante LLM → lista de
`UserMemory`. No hay equivalencia entre esa seleccion y distancia ANN.
En aprendizaje, `UserMemoryStore.recall:98-116` consulta por usuario,
mientras `EntityMemoryStore.recall:426-500` exige user si namespace es user,
pide directorio hasta cap+1, expande identidad/tipo solicitado, hace matching
de nombres/aliases y busquedas lexicas intercaladas hasta top-k; añade nombres
relacionados. Archivados se excluyen del recall descrito, no de toda busqueda.
Sin user devuelve None con warning, no evidencia de inexistencia.
El consumidor todavia debe ensamblar/budgetar el contexto: resultados
recuperados no prueban entrega al modelo ni uso en una respuesta.
Una lectura concurrente no acredita snapshot de todas las consultas;
directorio parcial no autoriza afirmar que una entidad no existe.

**F3 — update/delete/consolidacion.** La tool `update_memory`
([manager.py:1386-1416](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/memory/manager.py#L1386-L1416))
construye `UserMemory(memory_id, user_id, memory, topics, input, agent_id,
team_id)` y llama directamente a `db.upsert_user_memory`: **no obtiene la fila,
no combina sus valores previos ni valida ownership**. SQLite
([sqlite.py:2699-2725](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/sqlite/sqlite.py#L2699-L2725))
resuelve conflicto sólo por `memory_id`; actualiza contenido/topics/input/
feedback/agent/team sin condición por user. El user del objeto de inserción
no restringe el update de una fila existente. **Deducción estática acotada:**
con DB compartida, ID ajeno conocido y acceso directo sin wrapper autorizado,
ese camino puede modificar contenido de otro user; no es exploit REST
reproducido ni conclusión sobre todos los adapters. Delete pasa ID y user al
adapter y requiere su propio contraste, no hereda la semántica del upsert.
`optimize_memories:803-873` lee fuentes → estrategia
`SummarizeStrategy.optimize:44-120` resume y combina topics → si `apply=False`
devuelve propuesta; si True limpia las fuentes del user y hace upserts
individuales. Cada operacion SQLite tiene su propia frontera de commit,
no hay transaccion envolvente acreditada en el manager. Fallar entre clear y
ultimo upsert puede dejar reemplazo parcial; cancelar el LLM antes del apply
no es igual a cancelar despues del clear; escritor concurrente requiere
revalidar revisiones antes de publicar.
Ademas, la tool `clear_memory:1433-1441` llama a `db.clear_memories()` sin user:
en SQLite `2849-2867` hace `table.delete()` global. **No confundir** esa tool
con `clear_user_memories`, ni con borrar una sesion. Riesgo concreto condicionado
a permitir la tool y compartir DB; alcance demostrado aqui por lectura, sin PoC.

### Ficha A: consolidacion como revision, no borrado previo

- **Objetivo:** evaluar una consolidacion segura sobre la compactacion,
  dedup y supersession que Vanta ya tiene.
- **Unidad minima:** `MemoryManager.optimize_memories`, 803-870 [A1],
  y `SummarizeStrategy.optimize`, 44-119 [A2].
- **Seam existente:** `vanta-memory/src/context_engine/engine.rs`,
  `assemble/assemble_with_recall`, y `src/core/record/l1_dedup.rs` dentro de
  `vanta-memory`; scheduler/checkpoints para trabajo diferido.
- **Contrato Rust propuesto:** `ConsolidationPlan { scope, source_ids,
  source_revisions, replacement, provenance }`; aplicar mediante commit
  condicionado a las revisiones; nunca limpiar fuentes antes del reemplazo.
- **Pasos/dependencias:** enumerar fuentes y registrar presupuesto; generar
  propuesta fuera del motor con LLM opcional; validar alcance; persistir
  reemplazo y supersession atomicamente o con recuperacion explicita.
- **FMEA:** crash entre clear y upsert; escritor concurrente borrado por una
  consolidacion vieja; perdida de evidencia; resumen que mezcla usuarios.
  [A1] muestra clear seguido de bucle; la perdida efectiva exige fallo y
  semantica concreta del adapter, no se reprodujo.
- **Aceptacion:** fallos entre etapas dejan fuente o reemplazo recuperable;
  revision obsoleta no pisa datos nuevos; cada hecho consolidado conserva
  referencias; cancelar no elimina originales.
- **No objetivos:** sustituir context assembly ni exigir LLM al storage.

### Ficha B: scope tipado y routing de memoria

- **Objetivo:** separar compartir deliberadamente de compartir por omision.
- **Unidad minima:** `UserMemoryStore`, 57-149 y `_build_memories_id`,
  930-932 [A3]; `EntityMemoryStore.recall`, 426-463 [A4].
- **Seam:** sesiones/personas/escenas, filtros del retrieval y provenance
  existentes de Vanta; usar adapters Python, no copiar AgentOS.
- **Contrato Rust propuesto:** `MemoryScope { tenant, owner, audience,
  namespace }` con owner obligatorio para memoria personal y audience
  explicita para compartir. Agent/team pueden ser provenance sin ser ACL.
- **Pasos/dependencias:** inventariar scopes actuales; definir mapping de
  identidad del caller; aplicar filtros antes de retrieval y consolidacion;
  autorizar también el recurso existente por ID antes de update/delete/merge;
  migrar datos sin owner mediante decision explicita, no usuario global.
- **FMEA:** `"default"` compartido en despliegue multiusuario; namespace
  compartido confundido con publico; confundir agente originador con lector
  autorizado. En F3, conflicto por ID sin condición de user permite mutar una
  fila ajena en DB compartida sin wrapper: detectar discrepancia actor/owner
  y rechazar antes del efecto. No equiparar namespace o user de inserción a ACL.
- **Aceptacion:** mismo user local en tenants distintos no se cruza;
  compartir requiere accion trazable; borrado respeta audience y fuentes;
  recall y extraccion usan el mismo scope validado. Casos futuros: ID propio
  permite update autorizado; ID ajeno deniega sin cambio; ID inexistente
  distingue insert permitido de update no encontrado; repetir con wrapper
  autorizado y dos users compartiendo DB. Ninguno fue ejecutado.
- **No objetivos:** nuevo sistema general de identidad ni framework multiagente.

### Ficha C: propuestas de memoria y aprobaciones verificables

- **Objetivo:** evitar que una opcion declarativa de HITL aparente una barrera
  de aprobacion donde solo hay extraccion automatica.
- **Unidad minima:** `UserMemoryStore.__post_init__/process`,
  76-149 [A3], y `UserMemory` con input/feedback/agent/team, 8-43 [A5].
- **Seam:** provenance, supersession y checkpoints de tareas de Vanta,
  con aprobacion en adapter o Studio existente.
- **Contrato Rust propuesto:** `MemoryProposal { evidence_ids, scope,
  operations, extractor_version, state }`; transiciones
  propuesta/aprobada/aplicada/rechazada con revision e identidad de actor.
- **Pasos/dependencias:** hacer explicita la capacidad HITL; almacenar la
  propuesta sin incorporarla al recall de hechos; decidir y aplicar con
  clave idempotente; devolver recibo vinculado a las fuentes.
- **FMEA:** prompt injection en hechos, doble aprobacion, edicion de fuentes
  despues de aprobar y fallo tras aplicar. El resumen LLM no prueba verdad.
- **Aceptacion:** rechazada nunca entra a recall; retry no duplica memoria;
  aprobacion sobre revision vieja se invalida; audit distingue fuente,
  inferencia y actor. La evidencia cruda no se reemplaza por resumen.
- **No objetivos:** imponer HITL a todo write ni portar el ciclo de agentes.

### Contratos ejecutables en una futura especificacion de A-C

Los nombres siguientes son **propuestas documentales**, no tipos implementados.
No copiar otro `MemoryScope`: B debe mapear `RecallScope`, `RecallConfig`,
`InjectionPolicy` y sellos `default_tenancy/plan_write` existentes [AV5].

| Ficha | Entrada → salida / errores propuestos | Commit, ownership y rollback |
|---|---|---|
| A | `plan(scope, ids+versions, replacement, op_id)` → revision aplicada y IDs preservados; `StaleSources`, `ScopeDenied`, `UnsupportedAtomicApply`, `OutcomeUnknown` | Caller posee propuesta; worker/checkpoint posee aplicacion; cancelar preparacion no escribe; tras commit se consulta op_id, no se repite clear |
| B | `resolve_scope(authenticated_actor, requested_scope, operation)` → scope autorizado; `MissingOwner`, `ScopeDenied`, `LegacyUnmapped` | Identidad viene del host, nunca del texto LLM; validar tambien delete/merge antes de mutar; fallo no altera scope previo |
| C | `decide(proposal_id, expected_revision, actor, decision, op_id)` → estado/receipt; `StaleProposal`, `EvidenceChanged`, `AlreadyDecided`, `ScopeDenied` | Decision y aplicacion diferenciadas; rechazo no publica hecho; cancelacion de UI no revoca un commit confirmado |

**Delta A:** `l1_dedup.rs`, `l1_writer.rs::plan_write/persist_planned` ya tienen
merge, versiones y fuentes; revisar esos pasos antes de ofrecer CAS del grupo.
El assembler/spill conserva contenido, pero no demuestra por si solo commit
versionado de consolidacion L1. Pasos: inventariar fuentes/revisiones; preparar
fuera del lock; revalidar; publicar con mecanismo backend real; registrar recibo;
habilitar por caller; desactivar nuevas propuestas conservando historial.
Alternativa barata: conservar originales y solo marcar sustitucion; rechazar
clear-then-upsert. Migracion: no reindexar hechos invariantes; cambios de
contenido/vector siguen la ruta existente y no anuncian proyeccion lista antes.

**Delta B:** ampliar el mapping de identidad del adapter si falta, no crear IAM.
Pasos: inventario de auth/scope; mapear audience a politicas existentes;
resolver legado explicitamente; aplicar la misma decision a recall y write;
registrar solo identidad/scope/decision, no PII cruda.
Alternativas: una DB por cliente simplifica aislamiento pero multiplica gestion;
filtro posterior a top-k no es sustituto de autorizacion previa.

**Delta C:** checkpoints ya guardan progreso; lo nuevo seria estado de
aprobacion de un hecho con revision de evidencia, no otro scheduler.
Pasos: negociar capacidad (E); persistir propuesta fuera del recall de hechos;
resolver actor; comprobar evidencia/scope; aplicar por A o write existente;
conservar relacion decision/recibo y permitir desactivar nuevas propuestas.
No migrar hechos antiguos a «aprobados» sin evidencia de quien los aprobo.

**FMEA discriminante A-C:** resumen tardio → revision distinta → conflicto sin
borrado; prompt injection → pide ampliar scope → rechazo por politica externa;
doble click → mismo op_id → mismo receipt; actor revocado → rechazo al aplicar,
aunque aprobara antes; fallo al notificar → resultado desconocido consultable.
Aceptacion manual futura: conservar fuente o reemplazo en cada corte, no mezclar
tenants, no publicar rechazados y distinguir aplicado/conflicto/desconocido
sin registrar contenidos sensibles. Son condiciones funcionales, no pruebas
ejecutadas ni nuevos criterios de benchmark. A depende de revisiones/commit;
B puede existir sin LLM; C depende de B/E y solo de A para operaciones compuestas.

### Ficha D: directorio de entidades con cobertura explicita

- **Caso agente:** asistente identifica «Acme» y debe distinguir no encontrado
  de no explorado por presupuesto, sin volcar todas las fichas al modelo.
- **Fuente minima:** mismo SHA Agno; `EntityMemoryStore.recall:426-500`
  [A12], `_novel_facts:1300-1322` [A4]. Cap+1 revela truncamiento; expansion
  por identidad precede a matching y busqueda lexica intercalada.
- **Delta/seam:** Vanta ya tiene indices y filtros; agregar a su reporte
  contextual cobertura del directorio, no otro BM25 ni tabla paralela de hechos.
  Seams: `context_engine/engine.rs::assemble_with_recall`,
  `core/hooks/auto_recall.rs::RecallConfig` y metadata/provenance L1 [AV5].
- **Contrato Rust propuesto:** `EntityRecallView { scope, source_revision,
  directory_refs, expanded_refs, truncated, continuation }`;
  `read_entity_view` devuelve esa vista o `ScopeDenied`, `StaleCursor`,
  `InvalidEntityRef`. Un match lexical no prueba identidad real.
- **Pasos:** mapear identidad/tipo/aliases existentes; listar con limite y
  senal de truncamiento; expandir IDs autorizados; ensamblar con budget global;
  reportar omitidos; mantener vista derivable, desactivable sin borrar hechos.
- **Alternativas:** lista exhaustiva solo para conjuntos pequenos; retrieval
  puro evita coste de directorio pero no sustenta negativos sobre cobertura.
  No importar todo el grafo de entidades ni sus heuristicas sin necesidad.
- **Dependencias:** B para scope; version/paginacion existente; assembler.
  Independiente de C y de inferencia; A debe invalidar referencias sustituidas.
- **FMEA:** directorio truncado → falso negativo → marcar parcial; alias
  ambiguo → mezcla de personas → referencias tipadas y evidencia separada;
  borrado concurrente → expansion ausente → `StaleCursor`, no ficha inventada.
  Caller posee cursor, motor posee lectura; no hay commit externo por recall.
- **Aceptacion futura:** cap exacto y cap+1 se distinguen; archivado no se
  expande en el modo declarado; mismo nombre/tipos distintos no se conflan;
  cancelacion devuelve error o vista completa declarada, nunca negativo global.
- **No-goals:** nuevo entity runtime, resolver verdad ontologica con un LLM,
  sustituir dedup/supersession existentes o configurar otro benchmark.

### Ficha E: capacidades negociadas y recibos de mutacion

- **Caso:** adapter declara HITL/async/clear-scoped pero el metodo elegido
  no lo soporta o devuelve ID aunque no persista; no aceptar exito nominal.
- **Fuente:** `UserMemoryStore.__post_init__/process:76-149` [A3],
  `MemoryManager.add_user_memory/_upsert_db_memory:220-247,576-585` [A10],
  incidencias 11143/9983 abajo. No es una extraccion de algoritmo de busqueda.
- **Delta/seam:** validar capacidades al conectar el binding Python y
  `integrations/vantadb_shared`, conservando `TaskCheckpointManager` [AV5]
  para seguimiento; no envolver todos los metodos sync con async ficticio.
- **Contrato Rust propuesto:** `MutationCapability { scoped_delete,
  conditional_replace, proposal_decision }` y `MutationReceipt { op_id,
  committed_refs, outcome }`; `Unsupported`, `NotApplied`, `OutcomeUnknown`,
  `ScopeDenied` son resultados distintos. No afirmar que hoy existen.
- **Pasos:** inventariar metodos/endpoints; declarar soporte; rechazar al inicio
  combinaciones invalidas; esperar frontera backend; emitir recibo verificable;
  mapearlo a SDK; versionar formato, sin migrar IDs de hechos.
- **Alternativas:** comprobacion estatica del adapter suele bastar; no construir
  un descubridor dinamico universal ni bucle de retries dentro del storage.
- **Dependencias:** B para operaciones scoped; A/C consumen recibos.
  Ownership del intento en host; persistencia en backend; cancelar despues de
  commit no convierte exito en «no aplicado» ni revierte efectos de tools.
- **FMEA:** coroutine no awaited → ID sin write → error de capacidad temprano;
  timeout despues de commit → retry duplicado → op_id; feature no soportada
  → warning ignorado → rechazo explicito; upsert por ID ajeno sin ownership
  → modificación cruzada → exigir autorización de recurso en B, no inferirla
  de un recibo de éxito. Observar codigo/ID/estado, no texto.
- **Aceptacion futura:** unsupported no cambia datos; error operativo no se
  devuelve como exito; op_id repetido no genera identidad nueva; close no borra.
  E debe declarar si update autoriza owner, si sólo hace upsert por ID y cómo
  distingue ID propio/ajeno/inexistente; no anunciar aislamiento desde el DTO.
- **No-goals:** exactly-once de sistemas externos, nuevo worker o proveedor LLM.

## 3. Riesgos e issues

| Nivel | Evidencia | Lectura prudente |
|---|---|---|
| Codigo | clear y despues upserts en optimizacion [A1] | No se ve transaccion global en ese metodo; revisar adapter |
| Codigo | agent/team de UserMemoryStore son auditoria [A3] | No son scopes privados por agente |
| Codigo | PROPOSE/HITL no soportados en ese store [A3] | No presumir aprobacion por configurar el enum |
| Codigo | strategy resume contenido y combina topics [A2] | Consolidacion cambia identidad y puede perder granularidad |
| Hipotesis | Historial hostil condiciona herramientas de memoria | Validar autorizacion fuera del modelo |

Se leyeron las incidencias públicas detalladas en §11; la frase anterior del
borrador «no se leyeron issues» queda corregida. La ruta concurrente y todas
las DBs quedan sin reproducción. Dedup exacta no equivale a dedup semántica
ni exactly-once.

## 4. Prerrequisitos

Revalidar el estado actual de Vanta antes de abrir tareas: ya hay dedup con
juicio LLM opcional, supersession, sesiones, scheduler, checkpoints y
context assembly con presupuesto compartido. Distinguir retencion de eventos,
hechos y resumen; acordar consentimiento para persistir PII y acceso por
tenant. Precisar atomicidad de aplicar consolidacion sin bloquear inferencia.
La memoria de agentes puede ser durable; el proceso que extrae no debe
convertirse en dependencia obligatoria del Rust embebido.

## 5. No trasladar

No usar un resumen unico como reemplazo universal de eventos/evidencia.
No trasladar el usuario global por defecto a un servicio multiusuario.
No replicar routing de agentes, proveedor/modelo ni APIs de plataforma en
el storage. No confundir memoria de usuario con knowledge vector DB.
Apache-2.0 y las dependencias requieren revision de notices/patentes y
licencias por archivo si se adopta material; Rust no es una exencion.

## 6. Evidencia y cobertura

[METHOD](METHOD.md) es autoridad para baseline, evidencia y comparacion.
Ledger de **lectura acotada**, no auditoria integral:

- [A1: MemoryManager, 377-432, 603-650 y 803-873](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/memory/manager.py#L377-L432).
- [A2: SummarizeStrategy.optimize, 44-120](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/memory/strategies/summarize.py#L44-L120).
- [A3: UserMemoryStore, 57-150, 734-785 y 930-934](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/learn/stores/user_memory.py#L57-L150).
- [A4: EntityMemoryStore, 1-40, 426-465 y 1300-1328](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/learn/stores/entity_memory.py#L426-L465).
- [A5: UserMemory, 8-57](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/schemas/memory.py#L8-L57).
- [A6: AgentSession y serializacion, 15-65](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/session/agent.py#L15-L65).
- [A7: PerformanceResult y medicion, 27-78 y 248-280](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/eval/performance.py#L27-L78).
- [A8: runner de instanciacion, 1-30](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/cookbook/09_evals/performance/instantiate_agent.py#L1-L30).
- [A9: base DB, 20-65, contexto de contratos](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/base.py#L20-L65).

**No examinado:** implementaciones completas DB/knowledge/vector DB, runner de
agente, modelos, AgentOS auth, rutas UI, relaciones de entidades completas
y suites de integracion. **Pendiente manual:** fallos de consolidacion,
concurrencia, routing efectivo del agente y aislamiento extremo a extremo.
Sin ejecucion de clones, scripts, dependencias ni servidores.

## 7. Benchmarks existentes

[A8] instancia `Agent` con un system message y lo entrega a `PerformanceEval`;
no carga LoCoMo/LongMemEval y no mide recall. [A7] registra tiempos en segundos
y memoria en MiB, media/min/max/desviacion/mediana/p95; usa `Timer` y
`tracemalloc` con baseline para memoria Python. No equiparar esto a RSS total
de Rust ni a coste de extraccion.

Dependencias: Agno/Python y la funcion evaluada; el caso de instanciacion no
llama explicitamente a un proveedor, pero otras funciones pueden llamar LLM,
escribir DB y emitir telemetria. No se auditaron esas rutas por completo.
Efectos potenciales: CPU, RAM, salida y, segun funcion, red/coste/datos.
No hay resultado medido local ni se trasladan cifras comerciales.
La futura comparacion sigue solo [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks):
instanciacion y memoria de proceso no certifican memoria del agente.
No se crea rubric, juez, harness ni configuracion.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Contraste |
|---|---|---|
| Sesiones y provenance | Existe | Separar evento, hecho y summary |
| Dedup/supersession | Existe | Revision atomica de consolidacion por verificar |
| Compactacion y presupuesto compartido | Existe | No sustituir por resumen sin presupuesto |
| Scheduler/checkpoints | Existe | Aprovechar para tareas de consolidacion |
| BM25/HNSW/RRF | Existe | Agentic selection no los reemplaza |
| Scopes de aprendizaje explicitos | Parcial en este contraste | Revalidar mapping actual y ACL |
| Propuesta/aprobacion de cada memoria | No acreditado aqui | No inferirlo del soporte general de checkpoints |
| Framework AgentOS dentro de Rust | No aplicable | Mantener adapter/servicio opcional |

## 9. Debilidades sugeridas

Conviene verificar que el estado actual pueda explicar **por que se recuerda
un hecho**, quien puede leerlo y que fuentes sobreviven a su consolidacion.
Tambien que una capacidad no soportada sea error explicito, no solo opcion
nominal. Son preguntas de contrato e inspeccion; no acreditan bugs actuales
de Vanta ni justifican recrear memoria o presupuestos ya implementados.

## 10. Interfaz

En el indice local existen rutas AgentOS de memoria/sesiones/evaluaciones;
su existencia no acredita que el frontend comercial este en el mismo repo
ni que sus permisos se hayan auditado. **Frontend grafico local y escritorio
nativo: no localizados en las superficies examinadas**. No se iniciaron
servicios. Para Vanta, Studio ya vive en `develop-desktop`: evolucionar
inspeccion de scopes, fuentes y propuestas alli, no añadir otro dashboard.

## 11. Cierre de contratos y evidencia

### 11.1 Persistencia de sesión/run, conocimiento y producto externo

No trasladar el nombre histórico **AgnoStorage** a una API inventada del pin:
la superficie comprobada es `BaseDb/AsyncBaseDb` [A9] y `SqliteDb` [A11].
Storage de conversación, hechos extraídos y knowledge son recursos distintos:

| Recurso | Identidad y comportamiento acreditado | Lo que no acredita |
|---|---|---|
| Sesión | `AgentSession.session_id`, runs/datos/summary [A6] | Backup de todos los hechos o índices |
| Run | `SqliteDb.get_run/upsert_run`, tabla runs, `run_id/session_id/user_id`, preserva `run_index` [A15] | Replay de herramientas externas exactamente una vez |
| Continuación | `update_run_if_matches` bloquea escritura antes de leer y compara expected; `cancel_pending_run` condiciona estado [A15] | Que cancelar un run deshaga efectos o borre memoria |
| Hechos | `UserMemory.memory_id`, usuario y procedencia [A5/A11] | Que agent/team sean por sí mismos ACL |
| Knowledge | `Knowledge.vector_db`, contents DB, readers/storage sources [A16] | Que borrar sesión purgue documentos/vectores |

**Resume/handoff real en la frontera de persistencia:** un continuador presenta
`run_id`, sesión, usuario y campos esperados; `update_run_if_matches:1116-1160`
adquiere lock SQLite, lee fila y ejecuta update en `sess.begin()`. Devuelve
false si cambió estado/falta fila. La transferencia de ownership es condicional,
no una búsqueda de hechos. Reiniciar requiere además recuperar el run y
reconstruir dependencias del host; este tramo no certifica todo AgentOS.
Los commits de hechos, runs y servicios de herramientas siguen separados.

Knowledge [A16] declara `max_results=10`, `isolate_vector_search=False` por
compatibilidad y reindexación necesaria si se añade `linked_to`. No extender
ese default a una acusación de fuga: depende del adapter, permisos y despliegue.
También declara retries de embedding desactivados por coste/rate limits.
El motor Vanta no debe adoptar reintentos que vuelvan a facturar un documento
entero como consecuencia implícita de un timeout de persistencia.

AgentOS tiene **API HTTP local**, no frontend local acreditado: [A13]
`create_memory` resuelve DB/scope con request y reemplaza user con el efectivo;
rechaza user ausente, espera AsyncBaseDb cuando corresponde y comprueba retorno.
Rutas leídas incluyen POST/GET/PATCH/DELETE `/memories` y optimize; el router
no demuestra autorización de todos los transports ni inmunidad de las tools
del manager. El [README pin](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/README.md#L20-L80)
anuncia **AgentOS UI alojada externamente en os.agno.com**, con trazas,
conversaciones y knowledge. Eso es documentación del proveedor, no código
frontend inspeccionado, licencia de SaaS ni aplicación de escritorio.

### 11.2 Complemento contractual A–E

Estas firmas son Rust **PROPUESTO**, pseudointerfaces sobre tipos del adapter;
no implementaciones aprobadas ni sustitutos de los tipos existentes:

```rust
// Tipos locales propuestos: AuthorizedScope, Revision, OpId y AdapterError.
fn plan_consolidation(s: &AuthorizedScope, refs: &[VersionedRef],
    replacement: ProposedFact, op: OpId) -> Result<ConsolidationPlan, AdapterError>;
fn apply_consolidation(p: ConsolidationPlan, cancel: &CancelToken)
    -> Result<MutationObservation, AdapterError>;
fn resolve_scope(actor: &AuthenticatedActor, request: ScopeRequest,
    action: MemoryAction) -> Result<AuthorizedScope, AdapterError>;
fn decide_proposal(id: ProposalId, rev: Revision, actor: &AuthenticatedActor,
    decision: Decision, op: OpId) -> Result<ProposalOutcome, AdapterError>;
fn read_entity_view(s: &AuthorizedScope, q: EntityRequest,
    cursor: Option<EntityCursor>) -> Result<EntityRecallView, AdapterError>;
fn negotiate_memory(c: &BackendCapabilities, required: &MutationCapability)
    -> Result<CapabilityAgreement, AdapterError>;
```

`MemoryScope` del boceto B sigue siendo **nombre conceptual**, no un tipo
Vanta existente: mapear a `RecallScope/InjectionPolicy` y tenancy L1 [AV5].
El recibo conceptual E tampoco reemplaza **`WriteReceipt` existente** [AV6]:
éste vincula contenido/version/store/WAL con hash SHA-256 y relectura, **no
firma digital**, no prueba de todo derivado ni de commit remoto. Si se usa
el candidato transversal **MutationReceipt C13**, es solamente PROPUESTO;
mantener Accepted/Applied/Durable/Visibility como observaciones independientes,
Unknown donde no existe evidencia. UUID retornado no llena esas dimensiones.

| Ficha | Persistencia/migración/desactivación y abierto específico |
|---|---|
| A | Plan referenciado por checkpoint existente; originales conservados; publicar sólo sobre revisiones aún válidas. Legado sin revisión no obtiene CAS ficticio. Desactivar nuevas consolidaciones conserva fuentes/reemplazos. Abierto: garantía de grupo efectiva del backend |
| B | Mapping en adapter/host, no copiar IAM. Legado sin owner queda sin mapping hasta decisión autorizada; no migrar a público. Desactivar adapter no cambia ACL del primario. Abierto: audience compartida por caso de producto |
| C | Propuestas separadas del recall de hechos; checkpoint guarda decisión/actor/revisión y referencia a recibo. Nunca marcar hechos viejos «aprobados». Desactivar aprobación impide nuevas propuestas, no revierte commits. Abierto: qué operaciones requieren intervención humana |
| D | Vista derivable, cursor ligado a revisión/scope; sin índice nuevo. Cambios de aliases invalidan cursor, no reescriben hechos. Rollback quita vista y mantiene retrieval existente. Abierto: identidad canónica externa y resolución de ambigüedad |
| E | Declaración de capacidades versionada por adapter y observaciones en actividad/checkpoint. Legacy Unknown; rechazo de unsupported antes del write. Desactivar negociación no permite fingir atomicidad. Abierto: consulta por op_id realmente ofrecida por proveedor |

**FMEA adicional D/E:** cursor previo a borrado → referencia ya inexistente →
vista parcial o StaleCursor detectables; revalidar scope antes de expandir.
Wrapper sync sobre async → coroutine sin await → ID sin dato; negociación
rechaza combinación y no reintenta ciegamente. Estado aplicado + notificación
cancelada → cliente cree rollback → duplicación; inspeccionar op_id/recibo
sin volver a ejecutar el efecto. Métricas propuestas: códigos de decisión,
conflicto, truncamiento, capacidades y tiempos por fase; **sin payload**,
topics/PII ni prompts por defecto. Aceptación futura D normal/adversa: entidad
exacta se expande, alias ambiguo no une personas, cursor stale no da negativo
global. E: combinación válida conserva frontera de commit; unsupported no
muta y cancelación posterior al commit conserva observación conocida.

### 11.3 Incidencias públicas leídas el 2026-11-07

| Fuente / fecha / estado observado | Causa y alcance verificado | Decisión |
|---|---|---|
| [#11143](https://github.com/agno-agi/agno/issues/11143), abierta 2026-10-18, Open | Reporta CRUD sync con AsyncBaseDb sin await en `0aa5924`. En pin [A10], add retorna ID y `_upsert_db_memory` llama método sin await y convierte excepciones en texto | E valida capacidad; no trasladar devolución de UUID como commit. Comentario menciona PR #11154: no se acredita merge/fix |
| [#9983](https://github.com/agno-agi/agno/issues/9983), abierta 2026-09-05, Open | Reporte en v3.0.6 `d1a388446e1b44b20498c772e91303588e2734cf`; manager tool global coincide con [A10/A11] en pin estudiado | B/E: destructive scope externo al modelo. #9986 es corrección propuesta referida en hilo, no merge acreditado |

Se leyó cuerpo y conversación visible, no sólo título. El autor de #9983
propone CWE/CVSS/advisory: **no se adopta como CVE oficial ni score certificado**.
SQLite acredita delete global; los demás drivers mencionados en issue quedan
como reporte upstream, no auditoría propia. Configuración discriminante:
DB compartida y tool clear habilitada; no afirmar que toda instalación expone
ese camino. La API REST con scope [A13] no corrige automáticamente la closure.
Acceso inicial `websearch.fetchWebContent` a ambos endpoints REST de GitHub
devolvió rate limit; fallback HTML de ambos issues sí permitió leer evidencia.

### 11.4 Benchmark: defaults, población y efectos

Lectura adicional [A17] completa `PerformanceEval.run:497-632` y defaults
`203-246`: runtime/memory true, 10 warmups, 50 iteraciones; el entrypoint A8
sobrescribe a **1000**, no 50. Ejecuta primero warmups, luego iteraciones de
tiempo y separadamente de memoria. Con ambos activos la función se ejecuta
dos veces por iteración medida; **no es latencia+memoria del mismo intento**.
La población son invocaciones de `func`, sin dataset, preguntas, split ni
denominador QA. Estadísticos se calculan sobre listas completadas [A7].
Excepción de `func` propaga antes de formar resultado; no hay tasa de fallos
agregada que permita asumir 100% de éxito. No reset genérico de estado/DB
entre llamadas; `gc.collect()` no borra memoria de agente. Puede guardar
archivo/resultados DB y telemetría está true por defecto.
Separa este microbenchmark Python de pytest funcional y de QA/RAG/LongMemEval.
Adaptación Vanta: reutilizar pistas existentes de METHOD/evals, medir la misma
operación concreta; tracemalloc no es portable como RSS Rust. No crear runner,
prompts, scoring, rubricas, umbrales ni configuración. Ningún resultado nuevo.

### 11.5 Matriz auditada, fuentes y cierre propio

| Capacidad | Upstream / fuente | Vanta 97c9623 / fuente | Delta y decisión |
|---|---|---|---|
| Hechos vs sesión/run | Existe, A5/A6/A15 | Existe, AV3/AV5 | Manifiesto de recursos, no replay como memoria |
| Consolidación recuperable | Parcial, clear/upsert A1 | Dedup/supersession existentes AV5; CAS de grupo no acreditado | A, condicionado al backend |
| Scope/tenant | Parcial por operación, A3/A13/A16; update F3/SQLite por ID sin ownership | Existe, InjectionPolicy/plan_write AV5 | B/E: autorizar recurso antes de upsert; no generalizar a REST/todos los drivers |
| Budget/recall híbrido | Selección agentic A1; knowledge A16 | Existe, assemble_with_recall/RecallConfig AV5 | D sólo añade cobertura |
| Aprobación de hechos | No soportada en UserMemoryStore A3 | No acreditada por checkpoints AV5 | C opcional |
| Recibo de escritura | ID no certifica commit A10 | Existe WriteReceipt AV6 | E negocia capacidades, no inventar firma |
| Quotas/scheduler/checkpoints | Servicios distintos del hecho | Existe, VANTA-REFERENCE §capacidades | Reutilizar; no runtime/WAL nuevo |
| Inspección | REST A13 + UI externa README | Studio existente AS1–AS3 | Extender superficies reales |

Fuentes que faltaban como definiciones del borrador, todas en el pin Agno:
[A10](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/memory/manager.py#L220-L247)
incluye además `576-585,1343-1452`;
[A11](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/sqlite/sqlite.py#L2672-L2741), además `2849-2867`;
[A12](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/learn/stores/entity_memory.py#L426-L500);
[A13](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/os/routers/memory/memory.py#L59-L141);
[A14](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/pyproject.toml#L1-L85);
[A15](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/db/sqlite/sqlite.py#L1060-L1235);
[A16](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/knowledge/knowledge.py#L86-L150);
[A17](https://github.com/agno-agi/agno/blob/f45ccee83dcd332c6a2cf46b0a0add3e7f7d3e68/libs/agno/agno/eval/performance.py#L497-L632).

Seams Vanta, no nuevos componentes:
[AV1](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/openai/vantadb_openai/vectorstore.py#L15-L64),
[AV2](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/vantadb_shared/__init__.py#L31-L163),
[AV3](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L232-L296),
[AV4](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/store.py#L97-L177),
[AV5](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L330)
y `context_engine/engine.rs::assemble_with_recall:252-326`,
`core/hooks/auto_recall.rs::RecallConfig:302-335`,
`utils/task_checkpoint.rs::TaskCheckpointManager:122-125`;
[AV6](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L678-L745).

Studio: cuerpos del pin separado `0ad38c542b39a589035b0e17fdedb4c09dca24b9`
releídos mediante `git show`, no rangos del worktree.
[MemoryLens:57-412](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412)
(AS1, escenas/persona/skills/genlog y get del anchor),
[RetrievalLens:1-130](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L130)
(AS2, filtros/scores) y
[ActivityPanel:1-65](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L1-L65)
(AS3, audit paginado). Oportunidad concreta: mostrar scope,
source refs, cobertura y conflictos en esas superficies, sin inspector nuevo.
No afirmar render probado ni equivalencia entre generación-log y entrega LLM.
B/E deberían mostrar denegación por recurso/ID ajeno, no éxito desde un upsert;
es aceptación UI futura de A-1, no funcionalidad ya implementada.

**Cierre propio:** A–E preservados; A-1 rectificado en F3 y propagado a B/E,
FMEA, aceptación futura y matriz. La propiedad en update queda condicionada
al wrapper/backend concretos, no acreditada por pasar user al DTO.
Diez ejes cubiertos dentro
de lectura estática, no aceptación final. Grafo consultado sólo en lectura;
sin reindexar ni ejecutar apps/tests/builds/benchmarks. No hay bug nuevo de
Vanta acreditado: CAS de grupo, aprobaciones y receipts externos son hipótesis
de cobertura. Excluidos falsos positivos: budgets inexistentes, IAM ausente,
dedup inexistente y WriteReceipt como firma. Sin auditoría exhaustiva de
providers, UI SaaS, auth transport completa ni crash físico. Por superar 300
líneas, una futura fase aprobada separaría specs de scope, publicación y
propuestas; aquí no se crean specs/backlog ni archivos de producto.
