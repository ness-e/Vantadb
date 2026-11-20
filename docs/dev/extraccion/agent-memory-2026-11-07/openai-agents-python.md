---
title: "OpenAI Agents Python: propiedad del historial y reemplazo recuperable"
kind: research
status: proposed
description: "Análisis estático de sesiones, cancelación y compactación con contratos mínimos independientes para VantaDB"
tags: [research, memory, engineering]
---

# OpenAI Agents Python: propiedad del historial y reemplazo recuperable

## Índice

- [1. Recomendación y arquitectura](#1-recomendación-y-arquitectura)
- [2. Evidencia y unidades mínimas independientes](#2-evidencia-y-unidades-mínimas-independientes)
- [3. Riesgos, defectos e hipótesis](#3-riesgos-defectos-e-hipótesis)
- [4. Prerrequisitos Vanta](#4-prerrequisitos-vanta)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura)
- [7. Runners encontrados y realmente leídos](#7-runners-encontrados-y-realmente-leídos)
- [8. Matriz frente a Vanta](#8-matriz-frente-a-vanta)
- [9. Carencias sugeridas, no bugs certificados](#9-carencias-sugeridas-no-bugs-certificados)
- [10. UI, escritorio y evidencia externa](#10-ui-escritorio-y-evidencia-externa)
- [11. Ampliación de arquitectura y contratos](#11-ampliación-de-arquitectura-y-contratos)
- [12. Incidencias, matriz y cierre propio](#12-incidencias-matriz-y-cierre-propio)

## 1. Recomendación y arquitectura

**Dictamen:** estudiar propiedad de mutaciones y publicación de compactación,
no convertir un SDK de orquestación en dependencia del motor de memoria.
Clone: `C:\Users\Eros\openai-agents-python`.
SHA congelado: `27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9`.
Snapshot: **30 539 estrellas, MIT**, según [inventario](github-candidates.json).
Preparación en [manifiesto](clone-manifest.json); no hubo consulta de estrellas.

Stack [O0]: Python >=3.10, openai, Pydantic, MCP, HTTP/WebSocket;
extensiones opcionales incluyen SQLAlchemy, Redis, MongoDB y cifrado.
La versión declarada en el archivo no acredita una publicación verificada.
`Runner.run` acepta agente, entrada/RunState, sesión y continuidad remota [O1].
Su contrato documenta ciclo modelo → salida final / handoff / herramientas.
La firma distingue `session`, `conversation_id` y `previous_response_id`;
no son tres nombres para un único historial propietario.

```text
host → Runner → agente/modelo → herramientas/handoff → siguiente turno
             ↘ entrada e items de sesión → backend Session
Session → get latest-N cronológico / add batch / pop tail / clear
decorador compaction → reemplazo atómico si disponible
                     → fallback clear/add con restauración
retry → comprobar sufijo esperado → pop → restaurar si diverge
```

La lectura profundiza en persistencia, no en todos los caminos del Runner.
SQLite ordena por ID, filtra JSON ilegible y amplía ventana cuando hace falta
para obtener N items válidos [O3]. El batching y las mutaciones se ejecutan
fuera del loop mediante thread y esperan asentamiento ante cancelación [O4].
Ese comportamiento no garantiza atomicidad de todos los backends externos.

**Vanta:** base histórica `c561158253bb88192590fa406c61e0a27dd7df8d`;
las conclusiones de ampliación se fijan exclusivamente a
**`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**, no al HEAD concurrente.
Se conservan las capacidades del [contrato](METHOD.md#vantadb-como-referencia):
ANN/BM25/sparse/RRF, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints/ingestors/auditoría y Studio.
Seams históricos no equivalen a una auditoría del HEAD actual.

## 2. Evidencia y unidades mínimas independientes

| Ref. | Ruta y rango efectivamente leído | Evidencia |
|---|---|---|
| O0 | `pyproject.toml:1-87` | Dependencias y extras |
| O1 | `src/agents/run.py:279-343` | Firma y contrato del ciclo público |
| O2 | `src/agents/memory/session.py:21-108` | Cancelación, snapshot y protocolo |
| O3 | `src/agents/memory/sqlite_session.py:261-358` | Insert, lectura y orden |
| O4 | `src/agents/memory/sqlite_session.py:447-520` | Append, commit y pop validado |
| O5 | `src/agents/run_internal/session_persistence.py:1873-1962` | Sufijo y restauración best-effort |
| O6 | `src/agents/memory/openai_responses_compaction_session.py:559-650` | Reemplazo y restauración ante cancelación |
| O7 | `examples/sandbox/tutorials/dataroom_metric_extract/evals.py:1-315` | Validador de artefactos leído completo |
| O8 | `src/agents/extensions/visualization.py:1-65` | Nodos Graphviz por identidad |
| O9 | `Makefile:1-135`; `LICENSE:1-21`; `README.md:5,21` | Superficies de verificación, licencia y tracing anunciado |

La referencia interna de persistencia también fue leída como guía de intención,
no se usa como prueba sustitutiva de O2-O6.

### Candidata A — rewind condicionado a propiedad del sufijo

- **Problema:** un retry no puede deshacer escrituras ajenas por semejanza de
  contenido ni interpretar cancelación como rollback.
- **Unidad mínima:** identidad del intento + posición/generación del tail +
  comparación y retirada atómicas dentro del backend.
- **Fuente:** O5 verifica serializaciones y restaura pops si hay divergencia;
  O4 muestra validación dentro de la transacción de un pop.
- **Seam Vanta:** captura idempotente y estado de pipeline declarados en [V1],
  más locks por sesión/retry del worker ya existente [V2].
- **Contrato hipotético:** `AttemptSuffix { session, attempt_id, expected_tail,
  owned_ids }` y `RewindOutcome { removed, conflict, outcome_unknown }`.
- **Acción 1:** inventariar si el host ya conserva identidad de intento y turnos.
- **Acción 2:** separar items históricos de los aportados por la ejecución;
  contenido duplicado no es una identidad.
- **Acción 3:** si se necesita rollback, definir compare/remove bajo la frontera
  de escritura real; no componer lectura y pops públicos como CAS inexistente.
- **Acción 4:** devolver conflicto sin eliminar items ajenos; preservar errores
  de restauración en diagnósticos autorizados.
- **Dependencias:** ID estable, orden total por sesión, operación backend
  apropiada y semántica clara de timeout/cancelación.
- **FMEA — interleaving:** otro escritor añade entre comprobar y retirar;
  la comprobación debe gobernar la mutación, no sólo una lectura anterior.
- **FMEA — repetición legítima:** dos turnos dicen lo mismo y se borra el antiguo;
  la propiedad no puede deducirse únicamente del texto.
- **FMEA — cancelación tras commit:** retry vuelve a añadir contenido aplicado;
  conservar resultado conocido/desconocido e idempotencia existente.
- **Aceptación propuesta:** sólo se elimina el sufijo del intento esperado;
  conflicto deja intacta la historia superviviente.
- **Aceptación negativa:** backend sin garantía necesaria la declara; no se
  anuncia exactly-once ni rollback fuerte con helpers best-effort.
- **Independencia:** útil para retry de ingesta/captura sin compactación B.
- **No objetivos:** portar RunState entero, handoffs o conversaciones OpenAI.

### Candidata B — compactación con publicación y recuperación explícitas

- **Problema:** reemplazar contexto no debe vaciar historial durable si falla
  el guardado, ni sobreescribir una generación que cambió mientras se resumía.
- **Unidad mínima:** snapshot versionado + reemplazo condicionado + receipt.
  O6 distingue capacidad atómica del backend y fallback recuperable.
- **Seam Vanta:** assembler puro [V3], tipos/reportes [V4] y spill reversible
  existente que conserva originales antes de sustituirlos [V5].
- **Contrato hipotético:** `CompactionCommit { source_generation, replacement,
  preserved_refs }` y `CompactionPublish { applied_generation, conflict }`.
- **Acción 1:** revisar integración actual entre assembler, spill y persistencia.
- **Acción 2:** mantener cómputo puro fuera del commit; validar que el snapshot
  sigue siendo aplicable justo al publicar.
- **Acción 3:** reutilizar transacción/batch existente para el puntero visible;
  fallback sólo con garantías y restauración explícitas.
- **Acción 4:** publicar éxito después de confirmar el backend, incluso si la
  notificación al llamador termina cancelada.
- **Dependencias:** revisión monotónica, retención de originales, autorización,
  contrato de spill y marcadores de compactación existentes.
- **FMEA — resumen tardío:** reemplaza mensajes posteriores; condicionar a
  generación/identidad de entrada y devolver conflicto.
- **FMEA — clear/add parcial:** sesión queda vacía; preferir una frontera
  atómica real antes que confiar en compensación no durable.
- **FMEA — restauración cancelada:** reparación en background no asentada
  permite leer vacío; definir propiedad y finalización de esa recuperación.
- **Aceptación propuesta:** historial previo o reemplazo válido, nunca vacío
  accidental anunciado como éxito; los originales siguen siendo recuperables.
- **Aceptación de contexto:** preservar tool-call/result, prefijo protegido y
  presupuesto compartido; no duplicar las reglas del assembler.
- **Independencia:** no requiere eliminación de sufijo ni API de rewind A.
- **No objetivos:** usar endpoint de compactación OpenAI como requisito,
  nuevos prompts, thresholds o política de retención.

## 3. Riesgos, defectos e hipótesis

**Hecho O2:** `await_session_mutation` difiere cancelación hasta finalizar la
mutación y no añade timeout, transacción, retry ni exclusión mutua.
Su docstring advierte que una operación bloqueada puede demorar cancelación
indefinidamente. Es trade-off explícito, no un deadlock reproducido.

**Hecho O5:** la comprobación del sufijo y los pops son llamadas separadas;
restauración es best-effort y puede fallar. El helper genérico no acredita
una operación atómica entre todos esos pasos ni exclusión interproceso.
**Hipótesis:** escritores concurrentes podrían interferir sin coordinación del
backend/host. No se afirma que el Runner permita tal interleaving en toda ruta.

**Hecho O6:** capacidad atómica se resuelve sobre el tipo inmediato del wrapper
para no saltarse transformaciones como cifrado. Copiar sólo el backend interno
puede vulnerar esa frontera: es un riesgo de adaptación, no defecto upstream.
El fallback intenta restaurar tanto por excepción como por cancelación.
No se ha certificado recuperación tras crash de proceso o fallo físico.

Fronteras revisadas: historial persistido → modelo, intento → historial común,
wrapper → backend y cancelación → resultado de commit.
No se revisaron sandbox/MCP/credenciales/tracing end-to-end.
No se adjudican CVEs, incidentes ni bugs actuales a Vanta.

## 4. Prerrequisitos Vanta

1. Revalidar propiedad de sesión e identidad de intentos sobre el HEAD vigente.
2. Revisar operaciones reales de batch/CAS antes de exponer contratos fuertes.
3. Preservar spill, provenance, supersession y los reportes existentes.
4. Separar vista para modelo de historial durable: filtrar contexto no debe
   borrar retrospectivamente capturas válidas.
5. Definir quién posee tareas de limpieza y cómo observa su resultado el host.
6. Comprobar bindings/SDKs antes de ampliar errores o receipts públicos.

## 5. Qué no trasladar

No introducir Python, Responses API ni telemetría remota obligatoria al motor.
No copiar clear/add como si fuera transacción ni blindar cancelación sin límites.
No usar fingerprints de contenido como autoridad para borrar historia ajena.
No eliminar audit trail al compactar ni mezclar propietario local/remoto.
No importar modelo de agente, herramientas o sandbox para resolver persistencia.
No convertir el protocolo mínimo Session en garantía automática de ACID.

## 6. Método, licencia y cobertura

Lectura estática y autoría de esta ficha; la ampliación §11–12 añade fuentes públicas.
Sin ejecución upstream, instalación, tests, builds, benchmarks o servicios;
sin commits, push ni modificación de los clones. La restricción offline del
borrador queda sustituida por consultas públicas expresamente autorizadas.
MIT raíz [OL] y snapshot coinciden; conservar avisos si se reutiliza código.
Extras, servicios, datos y modelos requieren análisis de condiciones separado.

Cobertura O0-O9, más seams históricos V1-V5, no auditoría exhaustiva.
Omitidos: mayoría de `run.py`, streaming/resume completos, todos los stores,
implementación del endpoint compaction, herramientas, MCP, sandbox y exporters.
El manifiesto limita historial, ramas/tags, submódulos y LFS.
La guía interna de persistencia es intención; pruebas de garantía se apoyan
en los tramos de código concretos, sin inferir comportamiento no recorrido.

## 7. Runners encontrados y realmente leídos

**O7** es un runner/validador determinista de artefactos financieros del tutorial
`dataroom_metric_extract`, no un benchmark de memoria ni rendimiento del SDK.
Lee JSONL o CSV y valida `FinancialMetricBatch`.
Entrada esperada codificada: filas por archivo, nombre de métrica, período y
segmento; ocho rutas de fuentes txt/PDF y valores/unidades fijados.
Comprueba duplicados, cantidad exacta, sección de filing, claves, valor y unidad.
Salida: `EvalSummary.row_count` o excepción; no accuracy porcentual, recall,
latencia ni consumo de tokens. La igualdad numérica observada es exacta.

Dependencias directas: Python, biblioteca estándar y schemas locales/Pydantic.
El validador no invoca modelos, juez remoto ni sandbox; producir el artefacto
es otra fase con dependencias no acreditadas por este archivo.
No se inspeccionaron los ocho documentos para certificar sus valores.
No se llama FinanceBench ni se le atribuyen resultados de QA de otro proyecto.
No se diseñó ni modificó ningún prompt, juez, eval, threshold o configuración.

Se localizó también `examples/sandbox/tutorials/repo_code_review/evals.py`,
pero no se leyó su implementación: no se le atribuyen métricas.
Se revisó `Makefile:1-135` [OM]: tests e integration-tests no constituyen por
su nombre un benchmark. La búsqueda por nombres bench/perf y el recorrido
acotado no localizaron runner de latencia/durabilidad de sesiones.
Eso es **no localizado en cobertura**, no prueba de inexistencia global.
Comparación futura: [protocolo](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a Vanta

| Capacidad | Estado Vanta | Delta defendible |
|---|---|---|
| Sesiones/provenance/dedup/supersession | Existe | No crear un segundo historial |
| Assembler/compaction/presupuesto | Existe; V3/V4 histórico | B trata publicación, no resumen nuevo |
| Spill reversible | Existe en base; V5 | Conservarlo y revalidar integración |
| Worker/retry/locks/checkpoints | Existe; V2/propietario | A debe usar identidad existente |
| Rewind backend atómico por intento | No acreditado aquí | Inventario antes de proponer API |
| Cancelación uniforme en todos los hosts | Parcial: contratos/seams, no trazado actual | No asumir falta ni paridad completa |
| ANN/BM25/sparse/RRF/auditoría | Existe | SDK no justifica reemplazar motor |
| Conversación remota OpenAI como requisito | No aplicable | Propiedad del host/adaptador |

## 9. Carencias sugeridas, no bugs certificados

La pregunta útil es si el contrato distingue «cancelado», «no aplicado» y
«aplicación desconocida», y si sólo el dueño puede revertir sus items.
Otra es si compactación publica sobre la generación que realmente resumió.
Estas preguntas no prueban que Vanta incumpla ninguno de esos invariantes.
La respuesta podría ser documentación/reportes, sin nuevo componente.
A y B requieren autorización posterior y revalidación de cambios concurrentes.

## 10. UI, escritorio y evidencia externa

README anuncia tracing y muestra imagen externa de Agents Tracing UI [OU].
No se inspeccionó fuente frontend de esa plataforma ni se acredita escritorio.
O8 sí es código local: construye identificadores DOT por identidad de agentes,
tools y servidores MCP; evita confluir nodos que comparten etiqueta.
Graphviz es visualización de topología, no inspector durable de sesiones.
Vanta Studio Tauri/React ya existe en `develop-desktop`; los receipts/conflictos
serían datos para esa UI, no motivo para desarrollar un escritorio paralelo.
Runtime necesario: **ninguno**; no iniciar/reiniciar procesos.

[O0]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/pyproject.toml#L1-L87
[O1]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run.py#L279-L343
[O2]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/memory/session.py#L21-L108
[O3]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/memory/sqlite_session.py#L261-L358
[O4]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/memory/sqlite_session.py#L447-L520
[O5]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run_internal/session_persistence.py#L1873-L1962
[O6]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/memory/openai_responses_compaction_session.py#L559-L650
[O7]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/examples/sandbox/tutorials/dataroom_metric_extract/evals.py#L1-L315
[O8]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/extensions/visualization.py#L1-L65
[OM]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/Makefile#L1-L135
[OL]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/LICENSE#L1-L21
[OU]: https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/README.md#L5-L21
[V1]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/core/mod.rs#L1-L65
[V2]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/services/mod.rs#L1-L17
[V3]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/engine.rs#L1-L70
[V4]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/types.rs#L1-L65
[V5]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/spill.rs#L1-L65

## 11. Ampliación de arquitectura y contratos

### 11.1 Mapa, modelos y backends realmente inspeccionados

Se conservan A/B y O0–O9. Esta ampliación del 2026-11-07 usa pin upstream
`27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9`; no cambia manifiestos ni usa
historia shallow como prueba de introducción de un defecto. Base Vanta NV1–NV5
de §12 y [VANTA-REFERENCE](VANTA-REFERENCE.md), no seams históricos V1–V5.

```text
host → Agent + RunConfig → Runner → model/tools/guardrails/handoff
  → RunResult / RunResultStreaming → RunState → aprobación/resume
  → session_persistence → Session protocolo (historial de items)
      SQLite local / SQLAlchemy / Redis / Dapr / Conversations remoto
      wrappers EncryptedSession / ResponsesCompactionSession
  → tracing provider → BatchTraceProcessor → exporter externo
Vanta adapter propuesto: Session y refs autorizadas → binding/SDK existente
```

`TResponseInputItem` es historial tipado del proveedor: mensajes, llamadas/
resultados y otros items. `RunState` conserva decisiones/interrupciones,
contadores y `_PendingSessionWrite` [O13/O14], no una DB de hechos aprendidos.
`Session.session_id/get_items/add_items/pop_item/clear_session` [O2] no
incluye ACL, CAS distribuido, vector retrieval ni ownership universal.
`RunContextWrapper` transporta contexto del host; contenido de tool/LLM no
se convierte en autorización. Handoff cambia agente/contexto, no tenant.
Traza registra ejecución, no es memoria semántica ni commit de sesión.

| Backend/wrapper | Evidencia leída | Garantía y frontera real |
|---|---|---|
| SQLite | O3/O4 | Orden por ID, transacciones locales; thread+settlement no equivalen a transacción con tools |
| SQLAlchemySession | O15:572-610 | Serializa payload, `sess.begin`, lock de sesión, insert+updated_at; `_await_mutation` con retry SQLite existente |
| RedisSession | O16:698-775 | Lock de instancia; `_mutate_items`; replacement sólo en tipo concreto, false en subclass; no verifica stale input por sí mismo |
| OpenAIConversationsSession | O17:122-191 | Writes por lotes de hasta 20, no atómicos entre requests; multi-request sin retries automáticos y settlement del request en vuelo |
| DaprSession | O18:494-509 | Dos deletes secuenciales bajo lock local; no settlement compuesto en ese método del pin |
| ResponsesCompactionSession | O6/O19 | Decorador Responses-only; snapshot/sufijo nativo si backend lo soporta; fallback clear/add+restore no ACID universal |
| EncryptedSession | O20:189-211 | Compaction pasa callbacks read/decrypt/encrypt; no saltar wrapper mediante forwarding al store interno |

MongoDBSession, AsyncSQLiteSession y AdvancedSQLiteSession existen en inventario,
pero sus implementaciones completas no se revisaron aquí; no acreditarles
paridad por nombre. Redis/SQLAlchemy/Dapr son extras, no dependencias de todo
Session. MIT raíz no licencia automáticamente SDKs/modelos/servicios remotos,
datasets ni claves; toda adopción derivada conserva notices aplicables.

Compaction O19:90-184 declara `model="gpt-4.1"`, modo auto, hook default
10+ candidatos y rollback item budget None. Son defaults del pin, no
recomendaciones Vanta ni métricas; item budget no es bytes/tokens ni límite
del historial. No envuelve ConversationsSession, que posee historial remoto.
LLM/Responses compact permanece opt-in en adapter, nunca requisito del motor.

### 11.2 Tres flujos y fronteras de fallo

**F1 captura/orquestación.** `Runner.run` [O1] recibe input o RunState,
Session opcional y continuidad remota separada. Modelo genera output/tools/
handoff; call/result e interrupciones se convierten a items. Al persistir
resume, `save_resumed_turn_items:965-1026` [O13] compone held items con outputs,
registra pending write para `NextStepRunAgain | NextStepInterruption` y los
envía por ruta canónica. `_PendingSessionWrite` [O14:189-231] retiene session_id,
batch, fingerprints previos, count, response/store y estado acknowledged.
Tool puede haber aplicado efecto antes de fallar `add_items`; no volver a
invocarla para reparar publicación. Runtime/host posee RunState; Session
posee historial; proveedor externo posee efecto. Sin commit conjunto.

**F2 recall/resume.** `get_items(limit)` pide últimos N cronológicos,
no búsqueda de hechos. `resume_pending_session_write:1444-1619` [O13] exige
Session original/ID y acceso serializado por aplicación, verifica tail exacto:
si unchanged puede append; si committed no repite; cambiado/ambiguo/partial
falla cerrado. Locks in-process no serializan copias deserializadas ni prueban
identidad backend; mismo session_id en otra DB no basta. RunState conserva
checkpoint tipado, no registra por sí solo todo runtime/conexiones/tools.
Resume luego de reinicio exige host que reconstruya esos recursos y controle
las copias. No ejecutar inferencia ni siguiente herramienta sobre publicación
incierta. No llamar este recorrido «memoria a largo plazo».

**F3 handoff/update/delete/cancel.** `HandoffInputData` separa input_history,
pre_handoff_items, new_items e input_items [O10:72-179]. `input_items` puede
filtrar próxima vista preservando new_items en Session; nested history usa
los otros tres campos, por lo que filtrar sólo input_items **no redacta nested**.
Server-managed conversations no soporta esos filtros; items ya streameados
no se retiran. Compaction O6 resuelve hook en **tipo inmediato** para no saltar
cifrado; si no hay atomic replace, restaura después de excepción/cancelación.
Pop/clear mutan conversación, no purgan todos los hechos derivados o logs.
Conversations clear borra remoto y luego ID local [O17], no todas las trazas.
Streaming `cancel("immediate")` limpia tareas/queues; `"after_turn"` deja
terminar LLM/tools/persistencia del turno y detiene siguiente; docstring exige
seguir consumiendo stream para completar cancelación [O11:941-987].
Ninguno promete rollback de pago/email ya enviado. Error propagado y write
confirmado pueden coexistir; `await_session_mutation` no añade timeout/CAS.

### 11.3 A/B — Contratos completos sobre seams actuales

Prioridad **P0 A** propiedad del sufijo, **P0 D** resultado pendiente de tool,
**P1 B** publicación segura de compaction, **P1 C/E** vista y telemetría.
Preservar interfaces del borrador; estas firmas son **Rust PROPUESTO**:

```rust
fn rewind_attempt(s: &AuthorizedSession, req: AttemptSuffix,
    cancel: &CancelToken) -> Result<RewindOutcome, SessionMutationError>;
fn prepare_compaction(s: &AuthorizedSession, snap: ContextSnapshot,
    replacement: Vec<ContextItem>) -> Result<CompactionCommit, CompactionError>;
fn publish_compaction(s: &AuthorizedSession, commit: CompactionCommit,
    cancel: &CancelToken) -> Result<CompactionPublish, CompactionError>;
```

`AttemptSuffix` conserva session/attempt_id/expected_tail/owned_ids; añadir
expected revision y op_id sólo al envelope del adapter. `CompactionCommit`
conserva source_generation/replacement/preserved_refs. Errores propuestos
ScopeDenied/StaleGeneration/NotOwner/UnsupportedAtomicMutation/OutcomeUnknown,
sin inferir rollback desde timeout. Salidas explican aplicado/conflicto/
desconocido y evidencia disponible, no bool fsync inventado.

| Campo | A rewind de sufijo | B publicación de compaction |
|---|---|---|
| Problema/fuente | Retry deshace ajenos, O4/O5 | Clear+add puede perder contexto, O6/O19 |
| Delta | Propiedad de operación en adapter, no dedup nuevo | Integrar publicación versionada con spill existente, no summarizer nuevo |
| Alternativas | No ofrecer rewind si backend no ofrece frontera suficiente; abortar es más seguro que pops públicos | Transacción/backend ya existente; conservar originales y marcar reemplazo antes que clear compensado |
| Seam real | `core/record/l1_writer.rs::persist_planned/log_and_mark`, `utils/task_checkpoint.rs::TaskCheckpointManager`, NV4 | `context_engine/engine.rs::assemble_with_recall`, `spill.rs`, `IntegratedContext/CompactionReport`, NV3 |
| Pasos/persistencia | Inventariar ID intento→registrar ownership checkpoint→validar scope/rev→mutar bajo frontera real→emitir observación | Snapshot→prepare fuera del lock→revalidar→guardar originals/spill→publicar puntero/reemplazo→reportar |
| Migración | Legacy sin ownership no rebobina por parecido de texto; no reindex | Versionar refs/envelope, no reindex si contenido/vector invariantes |
| Rollback | Quitar API/adapter no borra historial; conflicto no elimina | Desactivar nuevas compactaciones, originals preservados; rollback no pisa generación nueva |
| Dependencias | Identidad/revisión/auth y backend; no requiere B | Snapshot/spill/commit/auth; no requiere rewind A |
| FMEA | Escritor intercalado→sufijo ajeno→conflicto por revisión; repetición textual→borra anterior→IDs de ownership | Resumen tardío→pisa nuevos→StaleGeneration; restauración cancelada→vacío visible→propietario de reparación explícito |
| Aceptación futura | Normal sólo sufijo propio; ajeno/missing/conflicto intactos; cancel postcommit conserva outcome | Normal originals o replacement válido; corte entre pasos no se anuncia éxito; wrapper cifrado no se bypass |
| Observabilidad/no-goals/abierto | IDs/rev/error sin payload; no exactly-once; abierto backend CAS real | Generación/refs/etapas sin texto; no endpoint OpenAI requerido; abierta atomicidad publicable |

Vanta ya tiene **WriteReceipt** NV5 (hash SHA-256, no firma) y context reports.
No sustituirlos por DTO homónimo; **MutationReceipt C13 sólo PROPUESTO** sería
envelope externo con Accepted/Applied/Durable/Visibility independientes y
Unknown si falta prueba. No WAL, runtime ni scheduler nuevo.

### 11.4 C — Vista de handoff separada del historial persistido

1. **Caso:** triage pasa a especialista con contexto mínimo autorizado, conserva
   evidencia cruda para auditoría sin filtrar sólo la vista equivocada.
2. **Fuente/unidad:** HandoffInputData/input_filter [O10:72-179]; no su runtime.
3. **Delta Vanta:** adjuntar decisión de compartición/revisiones al assembler
   ya existente, no crear retrieval ni transferir permisos desde modelo.
4. **Alternativas:** transcript completo expone más; sólo resumen puede ocultar
   fuentes; decisión: refs autorizadas + vista, originals intactos.
5. **Rust PROPUESTO:** `project_handoff(actor: &Actor, target: AgentRef,
   history: &ContextSnapshot, policy: &InjectionPolicy)
   -> Result<HandoffView, HandoffViewError>`; salida refs/generación/omitidos.
   ScopeDenied/UnsupportedRemoteFilter/StaleGeneration/InvalidToolGroup.
6. **Seam real:** NV3 `assemble_with_recall` y NV4 InjectionPolicy.
7. **Pasos:** identificar recursos→autorizar→seleccionar/ref redactar→verificar
   grupos call/result→registrar decisión→entregar. No borrar raw; legado
   sin recibo de entrega significa Unknown. Rollback quita proyección, no hechos.
8. **Dependencias:** provenance/budget existentes; B sólo si se compacta; independiente
   de inferencia y de framework de agentes.
9. **FMEA:** nested ignora vista redactada→fuga de contexto→detectar source fields
   incluidos y bloquear nested incompatible; stream previo no revocable→filtrar
   antes de emitir, no prometer borrado retroactivo.
10. **Aceptación futura:** destino normal sólo ve autorizados; revocación/sesión
    remota no soportada produce error explícito; tool-result huérfano rechazado;
    métricas de IDs/counts/decisiones, **sin payload**.
11. **Mínimo/no-goals/abierto:** vista opt-in; no SDK handoff en Rust; abierto
    quién posee redacción y qué transports permiten verificar entrega.

### 11.5 D — Publicación recuperable de resultados de herramientas

1. **Caso agente:** herramienta cobró con éxito pero guardar output falló;
   reanudar no puede cobrar de nuevo ni saltar incertidumbre de publicación.
2. **Fuente:** `_PendingSessionWrite` [O14:189-231],
   `save_resumed_turn_items` y `resume_pending_session_write` [O13:965-1026,
   1444-1619]; distinta de A, que retira sufijo, y B, que reemplaza contexto.
3. **Delta:** ligar resultado/guardrail/aprobación a captura existente del host,
   no añadir ejecución de tools al motor Vanta.
4. **Alternativas:** fail-closed explícito si backend no reconcilia; nunca
   reejecutar efecto ni inferir commit por texto. Envelope mínimo si sí puede.
5. **Rust PROPUESTO:** `settle_tool_publication(session: &AuthorizedSession,
   pending: &PendingToolPublication, cancel: &CancelToken)
   -> Result<PublicationOutcome, PublicationError>`.
   Entrada op_id/call_id/approval refs/backend ref/revisión/batch refs;
   salida Applied/AlreadyApplied/BlockedUnknown con revisión. Errores
   WrongBackend/StaleHistory/AmbiguousHistory/ScopeDenied/UnsupportedEvidence.
6. **Seam:** adapter Python/host; NV4 TaskCheckpointManager, plan_write/
   persist_planned y generation log para referencias, no ledger nuevo universal.
7. **Pasos:** registrar intent+resultado ya realizado→persistir pending en
   checkpoint existente→intentar append→comparar frontera exacta autorizada→
   confirmar antes de siguiente efecto. Versionar envelope; legacy desconocido
   no adquiere exactly-once. Desactivar bloquea continuaciones pendientes,
   conserva datos; reparar por operador/host, no borrar evidencia.
8. **Dependencias:** identidad/auth, exclusión real del host y backend; C si
   filtra vista; B si publicación dispara compaction. Sin dependencia de LLM.
9. **FMEA:** commit-then-raise→retry duplica→reconcile por op/revisión; copia
   RunState en dos workers→doble acción→host serializa; mismo ID en otra DB→
   falso match→backend ref; partial remote batch→ambigüedad→fail closed.
10. **Aceptación futura:** fallo previo no ejecuta tool, resultado confirmado se
    publica una vez; timeout postcommit no repite; estado ambiguo bloquea
    aprobación restante. Observación códigos/IDs/etapas sin output de tool.
11. **Mínimo/no-goals/abierto:** una publicación por intento del adapter; no
    transacción distribuida ni scheduler; abierta garantía backend de compare.

### 11.6 E — Observabilidad sin convertir trazas en memoria ni recibos

1. **Problema:** diagnóstico de tool/recall puede exfiltrar inputs/outputs o
   confundir span exportado con resultado persistido.
2. **Fuente:** `RunConfig` O21:53-60,414-443; `BackendSpanExporter` O22:45-94;
   `BatchTraceProcessor` O22:698-778 y setup O23:16-66.
3. **Delta:** mapping de metadata/outcome en diagnóstico ya existente y
   consentimiento de exportación; no nuevo motor de telemetría.
4. **Alternativas:** logs locales de códigos bastan inicialmente; exporter
   opt-in sólo si hay destino/retención aprobados; no copiar defaults sensibles.
5. **Rust PROPUESTO:** `describe_observation_policy(actor: &Actor,
   config: &EffectiveConfig) -> Result<ObservationPolicyView, PolicyError>`;
   `observe_publication(event: &PublicationMetadata, policy: &ObservationPolicyView)
   -> Result<ObservationStatus, ObservationError>`. Salida Local/Exported/
   Dropped/Disabled, no `Durable`; DeniedDestination/UnsupportedRedaction.
6. **Seam:** `IntegratedContext/CompactionReport` NV3, generación-log NV4 y
   ActivityPanel de Studio existente; exporter en adapter/host, fuera del motor.
7. **Pasos:** inventariar campos→clasificar sensibles→resolver configuración
   efectiva→registrar metadata sin payload→exportar sólo opt-in→mostrar drops.
   Versionar política, no migrar trazas antiguas a hechos. Desactivar exporter
   no borra historial ni revierte writes; retención externa requiere otro owner.
8. **Dependencias:** auth/config y redacción; independiente de A/B, observa D.
9. **FMEA:** trace sensible default→egreso de datos→vista de destino/política;
   cola llena→span perdido→drop count, nunca error del write primario; flush
   timeout→no evidencia externa→Unknown sin repetir herramienta.
10. **Aceptación futura:** flujo normal emite IDs/timing/códigos; input de tool
    con secreto no aparece por defecto; exporter caído no transforma write
    aplicado en no aplicado; métricas de dropped/disabled, sin texto.
11. **Mínimo/no-goals/abierto:** metadata local primero; no port de threads/
    scheduler/exporter OpenAI; abierto contrato de retención/destino del host.

O21 tiene tracing_disabled false y default sensible leído de env con `"true"`.
Esto describe upstream, no política recomendada. O22 usa endpoint por defecto
`https://api.openai.com/v1/traces/ingest`, credentials del entorno y reintentos;
cola en memoria default 8192, batch128, delay5s, drop si llena. O23 inicializa
lazy y registra shutdown atexit: importar módulo no prueba egreso inmediato,
ni atexit garantiza export durable frente a crash. No certificar privacidad
de todos campos desde un flag, ni proponer nueva configuración en esta tarea.

## 12. Incidencias, matriz y cierre propio

### 12.1 Fuentes públicas efectivamente leídas

Consulta **2026-11-07**, HTML de cuerpo/conversación tras búsqueda DDG/Bing
sin resultados útiles y fallback al buscador de issues GitHub exitoso:

| Incidencia / fecha / estado | Revisión y mecanismo | Contraste y aplicabilidad |
|---|---|---|
| [#4646](https://github.com/openai/openai-agents-python/issues/4646), 2026-08-25, Closed con relación #4654 | Reportado v0.22.0 `4df9ecfae1761ca6fea67cc5a20b383c1d492024` y main `36976b1854534c0f184ed955afd542493abbb5d9`: tras tool aprobada y Session append ambiguo, NextStepInterruption no retenía publicación/guardrails | Pin O13 ahora registra pending para RunAgain **e Interruption**. No presentarlo como bug actual universal ni inferir exactamente una vez. PR relacionada, merge/diff no leídos independientemente. D retiene lección de frontera postefecto |
| [#4990](https://github.com/openai/openai-agents-python/issues/4990), 2026-09-13, Closed ese día | Reportado main `fbd2dbca`; Dapr clear con dos awaits puede dejar metadata si cancela entre ambos | Pin O18 conserva dos deletes directos. Mantenedor confirma ventana pero cierra por no acreditar consecuencia relevante: historial lee messages independientemente. No exagerar a pérdida de conversación ni atribuir fix; #4991 sólo draft referido |

No se ejecutaron reproducciones ni se inspeccionaron todos los caminos de
guardrails. Issue cerrado no prueba arreglo; un comentario con pytest pasado
no es resultado de esta investigación. El alcance Dapr es bookkeeping/fecha
de creación potencialmente antigua, no exploit ni corrupción de todo store.
La recomendación no es portar la solución de un PR ni volver a arreglar el
camino RunAgain histórico: conservar outcome pendiente en integración concreta.

### 12.2 Benchmarks, QA y adaptación sin protocolo nuevo

O7 `__main__:305-315` tiene `--artifact-path` default
`output/financial_metrics.jsonl`; `load_metrics:218-237` lee JSONL ignorando
líneas vacías o CSV y valida tipos. `validate_outputs:240-302` crea claves
por source_file/metric_name/period/segment, rechaza duplicados, compara
cantidad exacta con EXPECTED_ROWS, secciones, valores y unidades; devuelve
row_count sólo si todo pasa. No es un denominador de accuracy, ni tolerancia
numérica/abstención/recall. Falla con excepción, no tasa agregada sobre
artefactos intentados. Cantidad de EXPECTED_ROWS no se convierte en preguntas
de memoria. Datos son expectativas financieras fijadas en código, no split
LongMemEval ni runner comparativo de Session.

El **validador aislado** lee artefacto y escribe stdout, no limpia DB ni llama
LLM/juez; la **producción del artefacto** del tutorial sandbox sí es otra fase
con modelo/herramientas/archivos/red potenciales no medida por O7. Reset,
semilla, cache, embeddings, coste y latencia de esa generación no quedan
acreditados. Pytest/Makefile son funcionales, no certificación longitudinal.
Adaptación futura concreta: adapter Session→binding captura historial y
refs de herramientas; mantener invariantes de fuente/duplicados de O7 sólo
en ese tutorial, y usar evals existentes LoCoMo/LongMemEval para retrieval con
las advertencias fake/sintético/unidad de BENCHMARK-NOTES. No portar expected
answers a memoria, ni crear harness, prompts, rúbricas, scoring, umbrales,
muestreo o configuraciones. Sin ejecución ni scores locales.

### 12.3 Matriz upstream/Vanta 97c9623 con decisión

| Capacidad | Upstream / fuente | Vanta estado y fuente | Delta/decisión |
|---|---|---|---|
| Historial y resume | Session O2, RunState O14, distintos | Existe, checkpoints/sesiones NV2/NV4 | Manifest/adapter, no llamar replay «hechos» |
| Retrieval híbrido y budgets | Session no ofrece vector search O2 | Existe NV3/NV4 y VANTA-REFERENCE | Usar core/recall; no motor nuevo |
| Dedup/provenance/supersession | Fingerprints recuperación O13 no son dedup semántica | Existe L1 NV4 | A/D añaden ownership/publicación, no dedup |
| Rewind por intento | Best-effort helper O5; backend específico O4 | No acreditado CAS de sufijo inter-recurso | A condicionado a garantía real |
| Compactación | Snapshot/sufijo/fallback O6/O19 | Existe assembler/spill NV3 | B publicación versionada; no resumidor nuevo |
| Handoff redactado | Parcial O10; no filtros en server-managed | Governance de inyección existente NV4 | C por transporte, no privilegios del modelo |
| Recibo | Session add None no prueba durable O2 | WriteReceipt existente NV5, hash no firma | C13 sólo propuesta para observaciones externas |
| Scheduler/quotas/checkpoints | No son contrato Session O2 | Existen, VANTA-REFERENCE | Reutilizar, no otro runtime/WAL |
| SDK integración OpenAI | SDK agente/historial O1/O2 | Vectorstore existente NV1, no demuestra adapter Session | Extender binding sólo si aprobado |
| UI | SaaS tracing anunciado OU, Graphviz local O8 | Studio existente, worktree separado | Mostrar outcome/refs en lentes actuales |

**Negativos Vanta:** sólo hipótesis de cobertura A/D y compatibilidad del
adapter Session, no defecto demostrado. Se descartan falsos positivos:
ausencia global de memoria, de compactación, de provenance/budget, de cuotas
o de recibos. `RecallConfig` usa presupuesto por caracteres y ruta core
configurable: no prometer tokens exactos ni recall híbrido por defecto en
todas rutas; prefijo protegido de assembler puede exceder budget.
Problemas históricos c561158 o issue antiguo no se trasladan al pin.

### 12.4 Ledger, UI y cierre de autoría

Fuentes adicionales upstream leídas y fijadas al SHA:

- [O10, handoffs:72-179](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/handoffs/__init__.py#L72-L179).
- [O11, cancel:941-987](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/result.py#L941-L987).
- [O13, save_resumed_turn_items:965-1026](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run_internal/session_persistence.py#L965-L1026) y [resume_pending_session_write:1444-1619](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run_internal/session_persistence.py#L1444-L1619).
- [O14, PendingSessionWrite:189-231](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run_state.py#L189-L231); modelo RunState/approvals `842-1118,1380-1420` sólo inventario parcial de símbolos, no auditoría completa.
- [O15, SQLAlchemy add/pop:572-610](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/extensions/memory/sqlalchemy_session.py#L572-L610).
- [O16, Redis add/replace/pop/clear:698-775](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/extensions/memory/redis_session.py#L698-L775); `_mutate_items` interno completo no auditado.
- [O17, Conversations add/clear:122-191](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/memory/openai_conversations_session.py#L122-L191).
- [O18, Dapr clear:494-509](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/extensions/memory/dapr_session.py#L494-L509).
- [O19, compaction constructor:90-184](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/memory/openai_responses_compaction_session.py#L90-L184).
- [O20, encrypted callbacks:189-211](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/extensions/memory/encrypt_session.py#L189-L211).
- [O21, sensible default:53-60](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run_config.py#L53-L60) y [RunConfig:414-443](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/run_config.py#L414-L443).
- [O22, exporter:45-94](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/tracing/processors.py#L45-L94) y [batch/drop:698-778](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/tracing/processors.py#L698-L778).
- [O23, setup lazy:16-66](https://github.com/openai/openai-agents-python/blob/27c75c5f9c5fd957d28c04fb7e9eeacaad4630a9/src/agents/tracing/setup.py#L16-L66).

Seams reales en **Vanta 97c9623**, no DTOs inventados:
[NV1, VantaDBOpenAI:15-64](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/openai/vantadb_openai/vectorstore.py#L15-L64),
[NV2, VantaDBCheckpointer:232-296](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L232-L296),
[NV3, assemble_with_recall:252-326](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L326),
[NV4, L1 persist_planned/log_and_mark:289-330](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L330)
y `core/hooks/auto_recall.rs::InjectionPolicy/RecallConfig:133-335`,
`utils/task_checkpoint.rs::TaskCheckpointManager:122-125`;
[NV5, WriteReceipt:678-745](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L678-L745).

**UI:** tracing alojado por proveedor no tiene fuente frontend inspeccionada
en este clon ni licencia atribuible al SDK MIT. Graphviz O8 sí tiene código
local para topología, no desktop ni inspector de historial durable.
En Studio Vanta `develop-desktop` (referencia `0ad38c542`) se leyeron
`desktop/src/components/activity/ActivityPanel.tsx:1-65::ActivityPanel`,
`components/lens/retrieval/RetrievalLens.tsx:1-95::RetrievalLens` y
`components/memory/MemoryLens.tsx:1-85::ScenesPanel`: audit paginado,
scores/filtros/contexto y bridge de memoria. Oportunidad concreta: resultado
pending/conflict/Unknown y revisiones en actividad/inspector existente, sin
clonar SaaS ni inventar otra app. No se inició ninguna UI.

**Cierre propio:** cinco mecanismos A–E con dependencias/no-goals y diez ejes
documentados, preservadas ampliaciones. No leído exhaustivamente: run loop
streaming completo, schemas/serialización RunState, todos stores, sandbox,
proveedores/jueces ni frontend SaaS; no certificar su seguridad o durabilidad.
Validación dinámica normal/adversa pendiente manual. Sin apps/tests/builds/
benchmarks/instalaciones/servicios, cambios de código, índices, git writes ni
revisión independiente. >300 líneas: futura fase aprobada separaría specs
de mutación/publicación y proyección/observabilidad; aquí no se crean.
La aceptación documental final corresponde al responsable, no al autor.
