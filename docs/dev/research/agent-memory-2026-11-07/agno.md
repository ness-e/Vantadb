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
  migrar datos sin owner mediante decision explicita, no usuario global.
- **FMEA:** `"default"` compartido en despliegue multiusuario; namespace
  compartido confundido con publico; confundir agente originador con lector
  autorizado. Es riesgo condicionado a integracion, no exploit demostrado.
- **Aceptacion:** mismo user local en tenants distintos no se cruza;
  compartir requiere accion trazable; borrado respeta audience y fuentes;
  recall y extraccion usan el mismo scope validado.
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

## 3. Riesgos e issues

| Nivel | Evidencia | Lectura prudente |
|---|---|---|
| Codigo | clear y despues upserts en optimizacion [A1] | No se ve transaccion global en ese metodo; revisar adapter |
| Codigo | agent/team de UserMemoryStore son auditoria [A3] | No son scopes privados por agente |
| Codigo | PROPOSE/HITL no soportados en ese store [A3] | No presumir aprobacion por configurar el enum |
| Codigo | strategy resume contenido y combina topics [A2] | Consolidacion cambia identidad y puede perder granularidad |
| Hipotesis | Historial hostil condiciona herramientas de memoria | Validar autorizacion fuera del modelo |

No se leyeron issues upstream ni se afirma un defecto vigente a partir de
incidencias historicas. La ruta concurrente y todas las DBs quedan sin
reproduccion. Dedup exacta no equivale a dedup semantica ni exactly-once.

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
