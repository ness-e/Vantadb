---
title: "OpenAI Agents Python: propiedad del historial y reemplazo recuperable"
kind: research
status: proposed
description: "Análisis estático de sesiones, cancelación y compactación con contratos mínimos independientes para VantaDB"
tags: [research, memory, engineering]
---

# OpenAI Agents Python: propiedad del historial y reemplazo recuperable

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
HEAD concurrente comunicado por propietario `167435a9e`.
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

Sólo lectura estática offline y authoring de esta ficha.
Sin ejecución upstream, instalación, tests, builds, benchmarks o servicios;
sin red, commits, push ni modificación de los clones.
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
