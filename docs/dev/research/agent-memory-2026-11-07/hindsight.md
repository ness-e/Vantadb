---
title: "Hindsight: consolidacion con evidencia y recuperacion multifuente"
kind: research
status: proposed
description: "Analisis estatico de retain, recall, reflect y consolidacion de Hindsight, con propuestas incrementales para VantaDB"
tags: [research, memory, engineering]
---

# Hindsight: consolidacion con evidencia y recuperacion multifuente

Estudio estático, no certificación. Repositorio `vectorize-io/hindsight`,
checkout `C:\Users\Eros\hindsight`, rama `main`, commit
`eb963cb5a929d92c081371df64c2d5c2fe3351af`, comprobado mediante
`git rev-parse HEAD` y presencia del código.
Snapshot: **59 509 estrellas**, MIT, no archivado; exclusivamente
[inventario congelado](github-candidates.json) y [manifest](clone-manifest.json).
No se consultaron nuevamente contadores. Rige [METHOD](METHOD.md).

**Contenido:** [1. Arquitectura](#1-recomendaciones-para-vantadb-y-contexto-arquitectonico-del-proyecto) ·
[2. Extracción](#2-partes-extraibles-flujo-real-lectura-de-codigo-y-mapeo-a-rustvantadb) ·
[3. Riesgos](#3-defectos-riesgos-e-issues-transferibles-con-su-nivel-de-evidencia) ·
[4. Prerrequisitos](#4-prerrequisitos-en-vantadb-antes-de-incorporar-la-idea) ·
[5. No trasladar](#5-componentes-y-practicas-que-no-deben-trasladarse) ·
[6. Método](#6-advertencias-metodologicas) · [7. Benchmarks](#7-benchmarks-existentes-runner-entrada-metricas-dependencias-y-riesgos) ·
[8. Matriz](#8-matriz-existeparcialno-acreditadono-aplicable-frente-a-vantadb) ·
[9. Contraste](#9-debilidades-de-vantadb-sugeridas-por-el-contraste-no-bugs-inventados) ·
[10. Interfaz](#10-escritorio-interfaz-e-inspeccion-implementado-externo-o-no-localizado).

Ampliación de autoría: se conserva la lectura anterior y se cierran lagunas
de API, temporalidad, incidencias y AMB. Comparación exclusivamente contra
[VANTA-REFERENCE](VANTA-REFERENCE.md), SHA
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, no contra el HEAD concurrente.
Las fichas siguen [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md); no son specs
aceptadas ni implementación. Al exceder 300 líneas, una fase posterior podrá
separar fichas de publicación/retrieval de este dossier; aquí se conservan
juntas por la exclusividad de escritura, sin crear documentos adicionales.

## 1. Recomendaciones para VantaDB y contexto arquitectonico del proyecto

**Dictamen:** estudiar las invariantes de consolidación, no portar el producto.
El diferencial está en controlar la vigencia de las fuentes mientras un LLM
trabaja, explicar la fusión y separar conocimiento observado de documentos
deliberadamente sintetizados. Vanta ya tiene procedencia, supersession,
deduplicación, reflexión, escenas y scheduler: no son capacidades nuevas.

| Capa | Implementación observada | Frontera |
|---|---|---|
| Distribución | `hindsight-api/pyproject.toml` depende de `hindsight-api-slim` | El paquete principal no contiene el motor |
| Dominio | Python, Pydantic, async; `engine/retain`, `search`, `reflect`, `consolidation` | LLM y embeddings son servicios, no storage |
| Persistencia | Store `memories/base.py` y adaptador `memories/pg`; PostgreSQL y compatibilidad Oracle | El recorrido inspeccionado es principalmente SQL/PG |
| Trabajo diferido | `worker/poller.py`, operaciones persistidas, estados terminales y defer/retry | Un retain aceptado no equivale a consolidación completa |
| Interfaces | FastAPI/MCP, clientes generados; CLI Rust; control plane Next.js | CLI Rust no implica motor Rust embebido |

El README presenta aprendizaje biomimético; el código inspeccionado acredita
un pipeline concreto, no una propiedad cognitiva ni calidad medida.

### Mapa de objetos, API y fronteras

```text
HTTP/MCP + RequestContext → MemoryEngine
  retain → documentos/chunks → extracción → facts world/experience
         → store/índices → operación diferida → observations con fuentes
  recall → scope + tiempo → recall_unified → fusión → rerank → evidencia
  reflect → herramientas de mental models/observations/facts → respuesta
  refresh → candidato de página → resultado preservado/escrito/fallido
control plane Next.js → proxy SDK → misma API por banco
```

`models.py::Document:104-122` usa clave compuesta `(id, bank_id)`;
`MemoryUnit:125-183` separa UUID, documento, embedding, metadata y fact_type.
`event_date` es compatibilidad; `occurred_start/end` representan cuándo pasó,
`mentioned_at` cuándo se mencionó, `created_at/updated_at` persistencia/revisión
[H12]. No inferir bitemporalidad transaccional completa de estos campos.
Entidades y enlaces enriquecen facts; una página mental no sustituye la
evidencia ni el historial de revisión de una observation.

| Operación | Contrato leído en `api/http.py` [H13] | Semántica y fallo |
|---|---|---|
| Retain | `RetainRequest:1354-1402`, `api_retain:9465-9538`; `POST /v1/default/banks/{bank_id}/memories` | items; sync por defecto; `operation_id` UUID idempotente solo para async, conflicto declarado 409 si pertenece a otra operación |
| Recall | `RecallRequest:584-660`, `api_recall:5455-5518` | query, types, tags/grupos, budget, max_tokens, trace, query_timestamp; fecha ISO inválida produce 400 antes del motor |
| Reflect | `ReflectRequest:1569-1628`, `run_reflect_agent` [H4] | genera respuesta/esquema usando herramientas; no equivale al ranking de recall ni al refresh persistente |
| Delete | `api_delete_document:7556-7583` | delega documento+banco al motor; cero documentos borrados produce 404; errores de validación/auth se conservan |
| Operaciones | `api_list_operations:7594-7605` | pending/processing/completed/failed/cancelled; retain, consolidation y refresh son trabajos distintos |

`RequestContext:15-74` [H12] transporta key, tenant, flags internos y señal de
cancelación cooperativa. Un `bank_id` en la consulta **no autentica** al caller;
los bypass internos/MCP requieren confiar en su construcción. La muestra
acredita dependencias precheck/admission en recall/retain, no una auditoría de
todas las extensiones. Tags `any/all` incluyen untagged; variantes strict no;
`exact` compara conjuntos y `exact + []` selecciona global [H13:641-652].
Ese vocabulario es filtro de dominio, no ACL transportable a Vanta.

**Priorización, no roadmap aprobado:** primero A/E/F (vigencia, separación
de ámbitos y errores de extracción); después B/D/C (diagnóstico, tiempo y
publicación de derivados); G solo ante un requisito real de razonamiento.
No se atribuyen ganancias de calidad/latencia sin evaluación equivalente.

## 2. Partes extraibles, flujo real, lectura de codigo y mapeo a Rust/VantaDB

### Flujo observado

1. **Retain:** `retain_batch` coordina entradas/documentos; `_extract_and_embed`
   llama extracción, recibe facts/chunks/usage, aplica filtro opcional antes
   de embeddings y aumenta el texto con fechas [H1].
2. **Identidad semántica:** `world` y `experience` son facts extraídos.
   Las **observations** son conocimiento consolidado con fuentes; no son
   sinónimo de facts originales. Los **mental models** son páginas/síntesis
   configurables consultadas por reflect, tampoco sinónimo de observations.
   La nomenclatura histórica de documentación/migraciones no reemplaza
   estas fronteras actuales [H3][H4].
3. **Persistencia/procedencia:** documentos, chunks, memory units, entidades
   y enlaces pertenecen al store. Las lecturas enriquecen observaciones con
   source facts/chunks; el fragmento de borrado devuelve además la referencia
   del original para que el llamador pueda gestionarlo [H6].
4. **Consolidación:** agrupa por scopes; relee fuentes vivas bajo bloqueo y
   detecta `updated_at` cambiado antes de aplicar la decisión del LLM.
   El comentario distingue explícitamente el store SQL de stores externos,
   que no proporcionan necesariamente esa misma protección [H3].
5. **Recall:** resuelve ventana temporal y delega brazos semántico, BM25,
   grafo y temporal al store; la fusión conserva ranks y scores por brazo.
   El orden de las listas tiene significado; hay un límite previo por brazo
   y una alternativa interleave usada para problemas de deduplicación [H2][H5].
6. **Reflect:** bucle con herramientas: buscar/leer mental models, buscar
   observations y recall de facts; no es simplemente devolver vecinos.
   Presupuestos de herramientas y freshness son parte de la interfaz [H4].
7. **Actualización:** los modelos de refresh distinguen full/delta, fallback,
   contenido preservado y candidato fallido; un fallo no debe confundirse
   con una página vacía escrita correctamente [H7].

**Update documental discriminante:** el delta en
`retain/orchestrator.py:4133-4177,4280-4335` [H14] relee hash antes de la
extracción para evitar gasto redundante; eso solo estrecha la carrera.
La transacción vuelve a bloquear el documento y contrasta el hash base;
si cambió devuelve `False` y cae al streaming/full retain. El ramo externo
debe respaldarse con su compare-and-set, no con el mero preflight.
Actualizar texto, retag y borrar implican invalidación de observaciones
derivadas; [H3] detecta edición y [H6] muestra borrado SQL, no la eliminación
física de todos los adjuntos/backups. Una caída después de facts pero antes
de consolidar deja evidencia disponible y conocimiento derivado retrasado;
no se promete una sola transacción que abarque LLM, SQL, blob y caches.

**Aislamiento y garantías:** las funciones SQL leídas reciben `bank_id`;
`delete_document_rows` filtra documento+banco y elimina enlaces antes del
cascade [H6]. Eso no demuestra autorización HTTP completa. El worker distingue
defer, retry solicitado y fallo terminal; no reintenta indiscriminadamente toda
excepción [H8]. No se acredita exactamente-una-vez de extremo a extremo ni
borrado físico de backups, archivos y derivados con esta muestra.

### Ficha candidata A — decisión de consolidación ligada a revisiones vivas

- **Problema/unidad:** evitar que un resultado LLM tardío resucite evidencia
  borrada o ya modificada. Extraer la invariante de
  `_filter_live_source_memories` y `_sources_changed_since_read`,
  `consolidator.py:706-751` [H3], no su dependencia SQL.
- **Seam existente:** [MemoryRecord](../../../../vanta-memory/src/core/abstractions/types.rs)
  `71-138` tiene fuentes/version/supersession; [writer](../../../../vanta-memory/src/core/record/l1_writer.rs)
  `write_memory:131-158` y `apply_dedup_batch:389` ya persisten decisiones.
- **Hipótesis de contrato Rust:** `SourceRevision { id, version }`,
  `ConsolidationProposal { sources, output }` y resultado
  `Applied | StaleSources | DeletedSources`; aplicar solo si revisiones y
  ámbito siguen coincidiendo. Estos nombres **no existen acreditados**.
- **Diferencia incremental:** una referencia de procedencia describe origen;
  una precondición de commit verifica que ese origen todavía sea válido.
- **Pasos/dependencias:** revalidar el writer actual; identificar la primitiva
  condicional/atómica adecuada del SDK; capturar revisiones antes del LLM;
  validar al commit; reencolar solo conflictos recuperables con checkpoint.
- **FMEA:** check separado del write permite TOCTOU; locks durante el LLM
  provocan contención; reintentos sin invalidación repiten decisiones obsoletas.
- **Aceptación funcional:** borrar o editar una fuente mientras se calcula
  una síntesis impide publicar el candidato antiguo; repetir el mismo trabajo
  no duplica ni pierde procedencia; namespaces diferentes no comparten fuentes.
  **No objetivo:** introducir PostgreSQL o mantener locks durante inferencia.

### Ficha candidata B — explicación y balance previo de brazos de retrieval

- **Problema/unidad:** un brazo expansivo puede consumir todo el pool de rerank.
  `cap_per_source:8-26` y `reciprocal_rank_fusion:29-109`,
  `search/fusion.py` [H2], conservan ranking y scores separados.
- **Seam existente:** [recall_candidates](../../../../vanta-memory/src/core/record/l1_reader.rs)
  `321-380` ya tiene pools keyword/vector y `rrf_merge`; METHOD acredita RRF
  también en el core. No proponer otra implementación paralela de RRF.
- **Hipótesis Rust:** `ArmCandidate { id, arm, rank, raw_score }`,
  `FusionTrace { contributions, exclusions }`; presupuesto por brazo configurado
  en la capa dominio y resultado compatible con el diagnóstico existente.
- **Incremento/pasos:** inventariar trazas actuales; ampliar la traza que ya
  exista; imponer límites antes de fusionar, registrar truncamiento y medir
  únicamente con el protocolo de METHOD. Dependencias: IDs estables y scope.
- **FMEA:** normalizar scores de distintas métricas como si fueran iguales,
  duplicados dentro de un brazo, límites que excluyan evidencia rara.
- **Aceptación funcional:** un ID presente en dos brazos aparece una vez con
  dos contribuciones; brazo ausente se explica; cambiar orden de entrada no
  cambia su identidad. **No objetivo:** afirmar mejora de recall o copiar
  constantes upstream como configuración de evaluación.

### Ficha candidata C — contrato de resultado de refresh, no solo texto

- **Problema/unidad:** distinguir «sin nueva evidencia», «candidato rechazado»
  y «escrito». `RefreshMode`, `ModeFallbackReason`, `RefreshOutcome` y
  `RefreshOperationOutcome`, `mental_model_refresh.py:27-65` [H7].
- **Seam existente:** [reflexión](../../../../vanta-memory/src/core/reflection/mod.rs)
  y [consolidación de escenas](../../../../vanta-memory/src/core/scene/auto_consolidate.rs);
  [scheduler](../../../../vanta-memory/src/services/scheduler.rs) ya recupera
  tareas y ejecuta mantenimiento (`110-145`).
- **Hipótesis Rust:** `RefreshOutcome { status, source_revision, preserved_id,
  reason }`; conservar contenido previo cuando no haya candidato válido.
  No se afirma que Vanta carezca actualmente de todos estos estados.
- **Incremento/pasos:** comparar resultados ya emitidos; añadir solo estados
  faltantes; persistirlos en generation log/checkpoint; exponerlos en Studio.
  Dependencias: ficha A para vigencia, API de logging existente.
- **FMEA:** éxito de ejecución no implica éxito de publicación; un resultado
  vacío puede borrar contenido útil; historial puede revelar datos borrados.
- **Aceptación funcional:** refresh vacío/fallido conserva la versión válida,
  queda auditable y no se presenta como nuevo conocimiento; el operador puede
  distinguir preservación de escritura. **No objetivo:** un agente reflect
  generalista ni copiar prompts de inferencia/evaluación.

### Registro de cobertura

Lectura de cuerpos/rangos, no auditoría íntegra de cada archivo:

| Evidencia | Archivo y rangos examinados | Qué acredita |
|---|---|---|
| [H1] | `retain/orchestrator.py:1378-1485,1740-1820` | extracción, filtro, redacción |
| [H2] | `search/fusion.py:1-210` | RRF, cap y alternativa interleave |
| [H3] | `consolidation/consolidator.py:655-776` | scopes, locks, revisión de fuentes |
| [H4] | `reflect/agent.py:1-95,503-592` | herramientas y contrato jerárquico |
| [H5] | `search/retrieval.py:1-130,129-248` | brazos y ventana temporal |
| [H6] | `memories/pg/documents.py:1-90,321-348,435-550` | identidad, delete y retag |
| [H7] | `mental_model_refresh.py:1-65` | resultados y causas de fallback |
| [H8] | `worker/poller.py:1286-1430` | retry, defer, estado y métricas |
| [H9] | `hindsight_system_evals/amb.py:1-123`, `target.py:1-62` | delegación runner y ciclo local |
| [H10] | `retrieval_metrics.py:1-85` | unidad documental y métricas |
| [H11] | `observation-history-view.tsx:1-100` | historial de texto, tags y fuentes |

Además: licencia raíz, manifests y README/CLAUDE como documentación;
`retain/fact_extraction.py:1-75` solo imports/estructura inicial, **no** auditoría
de todo su prompt/parser. No examinados: matriz completa Oracle, auth/tenancy
de extensiones, todas las migraciones, object storage, rerankers/proveedores,
ejecutor completo de mental models y SDKs generados.

## 3. Defectos, riesgos e issues transferibles, con su nivel de evidencia

- **Código:** el store externo no tiene necesariamente `updated_at` ni locks
  equivalentes: la garantía de [H3] no se puede generalizar por interfaz.
- **Código:** el worker explica que una operación manejada internamente como
  fallo puede contabilizarse como `success=true` en su métrica de throughput;
  el estado DB es la autoridad [H8:1296-1308]. No usar esa serie como tasa de éxito.
- **Hipótesis:** un ordinal de brazo equivocado produciría atribución errónea
  en [H2]; requiere recorrido del llamador, no se declara defecto reproducido.
- **Seguridad:** texto retenido y síntesis son datos no confiables. Filtrar
  facts antes del embed ayuda, pero no prueba protección frente a todas las
  inyecciones ni autorización por banco.
- **Issues públicos:** el intento histórico `list_issues` falló; la reanudación
  sí leyó cuerpos públicos mediante `websearch.fetchWebContent` y REST keyless.
  El registro discriminado al final sustituye la anterior laguna, sin convertir
  comentarios del código en prueba de una corrección publicada.

## 4. Prerrequisitos en VantaDB antes de incorporar la idea

Antes de implementar, revalidar el estado vigente; este contraste usa solamente
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`. Confirmar invariantes de commit,
revisionado y propagación del scope en writer, scheduler y derivados. Acordar
qué significa borrar una fuente con outputs todavía vigentes y quién posee
la política. Las llamadas LLM deben permanecer fuera del core puro y ser
opcionales; reutilizar checkpoints, generation log y diagnósticos actuales.

## 5. Componentes y practicas que no deben trasladarse

No importar el monorepo, servidor Python/SQL ni el CLI como supuesto motor
embebido. No reemplazar el modelo L0/L1/persona/escena con nomenclatura
biomimética incompatible. No confundir observación consolidada con verdad;
no dejar retries genéricos ocultar publicación parcial. No copiar prompts,
parámetros, presupuestos ni esquemas de evaluación a partir de resultados
comerciales. El modelo mental es un derivado revocable, no la fuente original.

## 6. Advertencias metodologicas

Solo lectura estática; sin servidores, dependencias, tests ni benchmarks.
Clon superficial de una rama, sin tags/historial anterior/submódulos recursivos
ni LFS descargado; el manifest no registra punteros LFS para esta revisión.
No se reconstruye cuándo se introdujo/corrigió un defecto.
MIT raíz, copyright Vectorize AI 2025: conservar notices y revisar dependencias
y archivos concretos antes de adoptar. Una traducción Rust no elimina derivación.

## 7. Benchmarks existentes: runner, entrada, metricas, dependencias y riesgos

`hindsight-system-evals/.../amb.py` [H9] **no** contiene LoCoMo/LongMemEval ni su
juez: prepara un checkout de `vectorize-io/agent-memory-benchmark`, ejecuta su
CLI y devuelve el exit code. `AMB_REF` es `main`; imprime el SHA resuelto,
pero no congela permanentemente el experimento. El runner recibe dataset,
split y target; el backend remoto conserva el modelo del servidor, no el
que el cliente de evaluación suponga. Localmente inicia servidor y lo detiene
en `finally`; no ejecutar aquí. Los comentarios sobre limpieza de bancos
remotos no prueban que AMB haga esa limpieza: se debe inspeccionar su adaptador.

También hay métricas retrieval locales [H10]: colapsar units al documento
fuente antes de cortar; nDCG de ganancia lineal, MAP, recall, precisión, MRR
y hit tienen denominadores distintos. Eso no es exactitud de respuesta.
Costes: extracción, embeddings, rerank, reflect, respuesta/juez externo y
arranque deben separarse; no se ha obtenido factura ni resultado.
Dependencias: Python/uv, servicio/DB, proveedores y checkout AMB externo.
Errores del subproceso se propagan; semántica completa por pregunta, scoring
y cleanup de AMB se amplían abajo con revisión remota separada. Comparación futura exclusivamente
según [protocolo del responsable](METHOD.md#protocolo-de-comparacion-de-benchmarks),
sin nuevos prompts, umbrales o harness.

## 8. Matriz existe/parcial/no acreditado/no aplicable frente a VantaDB

| Capacidad | Estado Vanta | Incremento que merece comprobar |
|---|---|---|
| BM25/HNSW/RRF y dedup | Existe | balance y explicación por brazo |
| Fuentes, versiones y supersession | Existe | precondición contra edición/borrado concurrente |
| Reflexión, persona y escenas | Existe | resultado de refresh y freshness coherente |
| Scheduler/checkpoints | Existe | publicación condicionada y estados recuperables |
| Motor PostgreSQL/Oracle | No aplicable | no sustituir storage embebido |
| Paridad de locks con stores externos de Hindsight | No acreditado | no es una propiedad de Vanta ni de toda interfaz upstream |
| Borrado de todos los derivados bajo concurrencia | No acreditado en este estudio | revalidar rutas actuales, no asumir bug |

## 9. Debilidades de VantaDB sugeridas por el contraste, no bugs inventados

Preguntas abiertas: ¿qué revisión exacta valida una consolidación tardía?,
¿la traza distingue retrieval vacío de brazo fallido?, ¿reflexión preservada
y publicada tienen estados diferentes?, ¿borrado invalida también vistas
y caches? Las primitivas ya existen; la hipótesis es cobertura/integración
desigual, no ausencia de arquitectura. No se trasladan defectos históricos
de OpenViking ni de otro HEAD al worktree de la fase inicial.

## 10. Escritorio, interfaz e inspeccion: implementado, externo o no localizado

Fuente de control plane **presente**. `ObservationHistoryView` [H11] modela
texto/tags/fechas previos y nuevas fuentes; `diffWords` usa una matriz LCS:
útil para trazabilidad, con coste cuadrático para textos grandes. No se abrió
la aplicación ni se validó autorización/flujo completo. Vanta Studio ya tiene
memoria, actividad y diagnóstico en `develop-desktop` según METHOD; proponer
extender sus vistas existentes, no crear una UI paralela.

**Pendiente manual:** comprobar retain→consolidación→recall→reflect real;
carreras edición/borrado; restauración/retry sin duplicados; auth de bancos;
persistencia y coste de derivados; reproducción AMB y obligaciones por archivo.
Validación documental automatizada y cualquier commit quedan para el responsable:
en esta tarea fueron expresamente prohibidos.

[H1]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/retain/orchestrator.py#L1378-L1485
[H2]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/search/fusion.py#L8-L176
[H3]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py#L655-L776
[H4]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/reflect/agent.py#L503-L592
[H5]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/search/retrieval.py#L129-L248
[H6]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/memories/pg/documents.py#L321-L348
[H7]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/mental_model_refresh.py#L1-L65
[H8]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/worker/poller.py#L1286-L1430
[H9]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-system-evals/hindsight_system_evals/amb.py#L1-L123
[H10]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-system-evals/hindsight_system_evals/retrieval_metrics.py#L1-L85
[H11]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-control-plane/src/components/observation-history-view.tsx#L1-L100

## Complemento de autoría: contratos completos y evidencia discriminante

Este complemento completa A–C sin sustituir sus interfaces y desarrolla D–G,
ya priorizados en §1. Índice adicional: [fichas](#fichas-dg-y-complementos-ac),
[incidencias](#incidencias-públicas-leídas), [AMB](#amb-lectura-del-runner-real),
[matriz](#matriz-trazable-y-seam-visual). No es revisión independiente.

### Fichas D–G y complementos A–C

Todos los nombres de interfaz siguientes son **PROPUESTOS**. El ámbito usa
`RecallScope` existente, nunca un supuesto `MemoryScope`. No se propone motor,
runtime, scheduler, WAL ni protocolo de evaluación adicionales.

**A, complementos del contrato de publicación con fuentes vivas.**
Alternativas: solo provenance documenta pero no impide TOCTOU; locks durante LLM
son inaceptables; se elige validación condicional breve en el writer existente.
Firma propuesta `apply_consolidation(scope: &RecallScope, p: ConsolidationProposal)
-> Result<ApplyOutcome, ConsolidationError>`: inputs con `(id,version)` y
output `Applied { id, version } | StaleSources | DeletedSources`; errores
`Unauthorized`, `Storage`, `Cancelled`. No publicar tras cancelación previa al
commit; tras commit devolver identidad durable, no fingir rollback de LLM.
Versionar metadata de propuesta y checkpoint sin reindexar vectores; lectores
antiguos ignoran campos opcionales; rollback desactiva publicación y conserva
fuentes. FMEA: edición concurrente→check/write separados→síntesis obsoleta;
detectar por conflicto de revisiones y mitigar dentro de la misma sección
condicional. Contadores `applied/stale/deleted/storage/cancelled`, sin texto.
Abierto: primitiva condicional adecuada; no atribuir atomicidad nueva a
`write_memory`, cuyo persist/log son etapas separadas [V1].
La protección específica de tags no es solo el check de fuentes: el pin
[H-tags] lee `current_tags` mediante `lock_observation_tags`, conserva ese
estado bajo bloqueo y lo une con tags de fuentes antes de rewrite. Para A,
FMEA adicional: snapshot de recall anterior al LLM → perder tags añadidos o
resucitar retirados → revalidar fuentes **y** metadata actual en publicación.
Aceptación futura separa edición de fuentes de edición de tags durante LLM;
no afirmar que una revisión de fuentes cubre ambas. No se añade otro runtime.

**B, complementos de explicación por brazo.**
Firma propuesta `trace_fusion(candidates: &[ArmCandidate], selected: &[String])
-> Result<FusionTrace, TraceError>`; errores de identidad/ordinal inválido,
invariante de una identidad con múltiples contribuciones y score bruto no
normalizado entre métricas. Se prefiere extender diagnóstico a otra fusión;
no cambiar constantes ni presupuesto de evaluación. Persistencia: traza
efímera por defecto, campos opcionales API; rollback omite la traza, conserva
orden actual. FMEA: duplicado intrabrazo→doble crédito→atribución engañosa;
detectar contribuciones repetidas y rechazarlas. Cancelación descarta traza
incompleta, no se vuelve un fallo de escritura. Aceptación adversa: brazo
fallido no equivale a vacío; observabilidad con cantidades, tiempos y enums,
no query ni payload. Abierto: contrato de trace existente del core vs legacy.

**C, complementos de resultado de refresh.**
Firma propuesta `finish_refresh(expected_version: u64, result: RefreshOutcome)
-> Result<RefreshOutcome, RefreshError>`; estados preservado/publicado/rechazado
y errores `Stale`, `Cancelled`, `Storage`, sin reemplazar una página válida por
vacío. Alternativa solo log textual no sirve al consumidor; se elige resultado
tipado en reflexión y generation log existentes [V3]. Migración aditiva de
estado/causa, sin reindexar fuentes; rollback deja de producir nuevos estados
y conserva versiones válidas. FMEA: excepción tras persistencia→retry ciego→
duplicado; detectar por identidad/revisión y checkpoint, mitigar con consulta
de resultado antes de repetir. Aceptación futura: reinicio entre persistencia
y log conserva la decisión durable o muestra reconciliación pendiente, no éxito
falso. Métricas por outcome sin contenido. Abierto: visibilidad del estado
entre writer/log; no se promete transacción global.

**D — ventana temporal explícita y semántica de fechas (11 campos).**
1. Problema: un agente pregunta «qué prefería entonces»; fecha de ingesta no
   debe reemplazar fecha del hecho ni «ahora» del caller.
2. Fuente/unidad: [H5] `retrieve_all_fact_types_parallel:129-248` resuelve
   `TemporalWindow` antes de `recall_unified`; [H12] separa ocurrió/mencionó.
3. Delta: Vanta ya tiene metadata/filtros y budgets; interesa contrato temporal
   auditable, no otro índice ni atribuir ausencia de filtros.
4. Alternativas: metadata opaca es compatible pero ambigua; extender request
   de recall con ventana validada es preferible a parser LLM obligatorio.
5. Contrato: `resolve_time(input: TemporalRecallInput) -> Result<ResolvedWindow,
   TemporalError>` propuesto; entrada referencia UTC y ventana opcional;
   salida origen explicit/inferred/none; errores rango invertido/fecha
   inválida. Ventana explícita no se reinterpreta y no autoriza acceso.
6. Seam: `RecallConfig` y `perform_auto_recall_governed` [V2], más filtros del
   SDK ya existentes; compatibilidad del modo legacy debe comprobarse.
7. Pasos: identificar campos disponibles, normalizar fechas al borde, conservar
   ausentes, aplicar filtro en ruta autorizada, propagar diagnóstico. Metadata
   aditiva/versionada; no backfill especulativo, rollback ignora campos nuevos.
8. Dependencias: B explica brazo temporal; A condiciona síntesis histórica;
   reloj del caller y provenance, sin proveedor obligatorio.
9. FMEA: timezone omitida→conversión local→hecho fuera de ventana; detectar
   fecha ambigua, rechazar/explicitar zona, no completar silenciosamente.
10. Aceptación futura: normal respeta límites; adverso distingue evento sin
    fecha de evento fuera de rango; cancelación no publica derivados;
    métricas por causa/origen, nunca timestamps personales en logs.
11. Mínimo: entrada explícita; no objetivos bitemporalidad MVCC ni nuevo modelo
    cognitivo; abierto tratamiento contractual de fechas desconocidas.

**E — aislamiento del lote por ámbito resuelto (11 campos).**
1. Problema: dos fuentes autorizadas separadamente no deben mezclarse en el
   mismo prompt por compartir tags originales.
2. Fuente: [H3] `_consolidation_batch_key:655-694`, `_scope_sort_key:697-703`:
   combined/scope/fanout se distinguen y el orden de locks es total.
3. Delta: sellos agent/team e `InjectionPolicy` existen [V1][V2]; aquí el límite
   es la **entrada al LLM de consolidación**, no solo salida de recall.
4. Alternativas: filtrar respuesta llega tarde; serializar todo desperdicia
   paralelismo; elegir agrupación canónica y locks breves ordenados.
5. Contrato propuesto `partition_consolidation(items: &[ScopedSource])
   -> Result<Vec<ScopedBatch>, ScopeError>`; input revisiones+ámbitos resueltos,
   output lotes homogéneos; errores scope inválido/no autorizado; una fuente
   fanout no cambia ámbito por tomar tags del primer elemento.
6. Seam: `plan_write`/`default_tenancy` [V1] y consolidación de escenas [V3].
7. Pasos: resolver ámbitos antes de inferencia, ordenar claves, capturar
   revisiones, ejecutar fuera de locks, publicar por A; checkpoint incluye
   versión del particionado, sin reindexar. Rollback pausa esos jobs, no mezcla.
8. Dependencias: A vigencia, C outcomes; permisos vienen del caller autorizado.
9. FMEA: ámbito vacío interpretado distinto→lote compartido→exfiltración al
   proveedor; detectar igualdad de scopes canónicos y mitigar fail-closed.
10. Aceptación futura: fuentes de tenants distintos nunca comparten lote;
    scopes solapados adquieren locks en mismo orden; cancelación libera
    ownership; métricas cardinalidad/denegados sin tags ni textos.
11. Mínimo: particionado y validación; no ACL paralela; abierto fanout permitido.

**F — validación de extracción y contabilidad de fallos (11 campos).**
1. Problema: lote con resultado duplicado/no solicitado no puede anunciar
   ingesta completa ni descartar evidencia sin diagnóstico.
2. Fuente: [H1] `_extract_and_embed:1378-1450`, `RetainFactFilter.review:1453-1485`;
   issue #6439 detalla mapeo `custom_id` y su límite de evidencia.
3. Delta: writer Vanta admite embedding opcional fallido [V1]; preservar esa
   degradación legítima, distinguirla de extracción incoherente.
4. Alternativas: abortar todo pierde parciales válidos; ignorar errores oculta
   pérdida; elegir resultado por item y publicación de válidos autorizados.
5. Contrato propuesto `validate_extraction(expected: &[SourceRevision],
   returned: &[ExtractionItem]) -> Result<ExtractionBatchOutcome, ExtractError>`;
   salida aceptados+rechazados por ID, errores duplicado/desconocido/malformed;
   ningún resultado crea identidad que no fue solicitada.
6. Seam: `plan_write`/`write_memory` [V1] y generation log [V3].
7. Pasos: validar antes de embed, conservar correlación al filtrar, publicar
   válidos, registrar parcial; metadata opcional, sin reindexar. Rollback
   suspende extractor nuevo, no elimina registros previamente válidos.
8. Dependencias: E scopes, A revisión, C outcome y proveedor externo opcional.
9. FMEA: índice cambia al filtrar→embedding de otro fact→corrupción semántica;
   detectar correspondencia ID/dimensión y mitigar zip validado por identidad.
10. Aceptación: normal un ID por resultado; adversos duplicados, omitidos,
    unknown y cancelación antes de persistir; conteos por causa sin payload.
11. Mínimo: validación determinista; no prompt/evaluador nuevo; abierto
    política de retry parcial y quién autoriza retención de errores.

**G — razonamiento con herramientas y cancelación (11 campos).**
1. Problema: respuesta necesita combinar observación y evidencia original,
   sin convertir un texto generado en un hecho automáticamente.
2. Fuente: [H4] `run_reflect_agent:503-592` y estructura inicial de herramientas;
   [H12] `RequestContext.raise_if_cancelled:68-74`.
3. Delta: reflexión/context assembly ya existen [V3]; solo adapter opcional
   que diferencia consulta de publicación.
4. Alternativas: usar recall directamente para preguntas simples; reservar
   herramientas para requisito explícito; rechazar un nuevo runtime agentic.
5. Contrato propuesto `reflect_evidence(request: ReflectEvidenceRequest)
   -> Result<ReflectEvidenceResult, ReflectError>`; scope+fuentes+budget;
   salida respuesta con IDs/versiones y estado, errores denied/cancelled/
   provider/budget; ninguna herramienta amplía permisos ni persiste por defecto.
6. Seam: reflexión existente [V3], `assemble_with_recall` [V4].
7. Pasos: mapear herramientas a lectura autorizada, propagar cancelación,
   retornar citas; solo log de estado opcional, sin migración de fuente ni
   reindexación; rollback deshabilita adapter y conserva recall directo.
8. Dependencias: A si publicación solicitada aparte; B/D evidencia explicable.
9. FMEA: desconexión→LLM continúa y retry duplica trabajo→agotamiento;
   detectar operación abandonada y mitigar cancelación cooperativa/admisión
   existente, sin prometer cancelar factura de una solicitud remota enviada.
10. Aceptación futura: citas resuelven dentro del scope; fuente borrada impide
    publicación; cancelar libera recursos locales; métricas llamadas/outcome
    sin prompts, respuesta ni claves del proveedor.
11. Mínimo/no objetivo: adapter de lectura, no reflect autónomo generalista;
    abierto utilidad real y contrato de salida por SDK.
El cierre de #2122 por #2127 (commit abreviado `07c85da`, Release v0.8.2
declarado en la página) y el token actual [H12] acotan la vigencia del reporte:
G no parte de ausencia global de cancelación. Aceptación pendiente por etapa/
proveedor/transporte; no se reprodujo desconexión ni se verificó el artefacto
de aquella versión.

### Incidencias públicas leídas

Consulta de cuerpos API y comentario de #6440 el **2026-11-07**; las fechas
y estados siguientes son observación remota, no metadata recalculada del clon.

| Incidencia / fecha / estado observado | Mecanismo y revisión reportada | Fix acreditado y transferencia |
|---|---|---|
| [#6440](https://github.com/vectorize-io/hindsight/issues/6440), creada 2026-11-07, open | Reporta `OracleOps.bulk_insert_links`: vecino ANN borrado entre búsqueda y enlace causa FK_ML_TO; imagen v1.1.0 revisión abreviada a5a619b y v1.2.0 según autor | Autor declara fix en fork; comentario ofrece cherry-pick, **no merge upstream acreditado**. No generalizar a PG ni Vanta; aplicar A al enlace/derivado y examinar orden de locks |
| [#6439](https://github.com/vectorize-io/hindsight/issues/6439), 2026-11-07, open | Reporta `fact_extraction.py`, diccionario por custom_id sobrescribe duplicado; revisión «main hoy», sin SHA del reportante | Patch local mencionado, no PR/fix publicado acreditado. Motiva F; confirmar parser concreto antes de llamarlo defecto del snapshot |
| [#4831](https://github.com/vectorize-io/hindsight/issues/4831), 2026-09-27, closed 2026-10-05 | Usuario reporta pérdida de status/entity tags en consolidation updates de v0.9.2 | Mantenedor 2026-10-05 declara fix en main: [#4893](https://github.com/vectorize-io/hindsight/pull/4893) descarta lote con fuentes editadas; [#4897](https://github.com/vectorize-io/hindsight/pull/4897) conserva tags actuales bajo bloqueo. [H-tags] confirma `lock_observation_tags/current_tags`, distinto del guard [H3]. No prueba paridad runtime global |
| [#2122](https://github.com/vectorize-io/hindsight/issues/2122), 2026-06-11, closed ese día por [#2127](https://github.com/vectorize-io/hindsight/pull/2127) | v0.7.1: recall abandonado continúa, retries compiten con consolidación | Página atribuye commit `07c85da` y Release v0.8.2; [H12] confirma token cooperativo actual. No se verificó artefacto de release ni cancelación inmediata de CPU/proveedor; G no supone el bug histórico vigente |

Son reportes upstream, no reproducciones propias; sus cifras operativas no
se usan como benchmarks. Los bodies están en
`https://api.github.com/repos/vectorize-io/hindsight/issues/{numero}`;
el comentario leído es
[#6440 comentario](https://github.com/vectorize-io/hindsight/issues/6440#issuecomment-6713083153).
No se inspeccionaron commits del fork ni timelines completos.
Relectura pública correctiva 2026-11-07: cuerpos y seguimiento de #4831/#2122
vía `websearch.fetchWebContent`, completos; declaraciones de fixes/versión
son evidencia upstream, no tests propios ni auditoría de los PR completos.

### AMB: lectura del runner real

Complemento remoto por raw/REST, sin checkout/fetch/instalación/ejecución:
`vectorize-io/agent-memory-benchmark` revisión resuelta
**`28b644bd4dd653c0207c7507280439390c829155`**. Es otra revisión y otro
repositorio, no el runner histórico de `hindsight-dev`. La URL errónea
`src/memory_bench/llm.py` devolvió 404; se resolvió leyendo `llm/__init__.py`
y `llm/gemini.py`, no se abandonó la búsqueda.

| Superficie leída a ese SHA | Entrada/defaults, unidad, denominador y efectos |
|---|---|
| [EvalRunner.run](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/runner.py#L51-L183) | `initialize→load_queries/documents→prepare(reset=not skip_ingestion)`; query/category/unit/doc limits; oracle carga gold explícitamente, no es baseline retrieval; skip usa resultados previos y altera alcance/coste |
| [Ejecución y agregado](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/runner.py#L185-L475) | Unidad aislada ingesta antes de responder; consultas default 4 concurrentes, unidades default 1 salvo proveedor; QA usa juez, retrieval tiene scorer aparte. Accuracy booleana `correct/len(results)` o media de scores no nulos; no equivale recall por sesión |
| [Provider Hindsight](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/memory/hindsight.py#L157-L299) | Banco derivado de store_dir, sufijo por unidad; `_create_bank` intenta delete y captura excepción antes de create. **Reset destructivo remoto**; observations desactivadas salvo `AMB_ENABLE_OBSERVATIONS`. No asumir configuración del dashboard publicado |
| [Espera de operaciones](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/memory/hindsight.py#L299-L420) | Cuenta operaciones, reintenta fallidas y falla si no logra completar; aceptar retain no basta para medir recall tras ingesta |
| [Recall sync](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/memory/hindsight.py#L899-L939) | Tras tres excepciones devuelve vacío; colapsa fallo operativo a no evidencia. Dedup por ID de fact, no por chunk; no deducir que k=10 limite realmente el cuerpo que construye `_recall_kwargs` |
| [HTTP provider](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/memory/hindsight.py#L1278-L1301) | Endpoint default localhost:8888; claves y concurrencia por entorno; modelo de extracción/embedding del servidor no queda fijado por este cliente |
| [LongMemEval](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/dataset/longmemeval.py#L1-L110) | JSON cleaned, auto-descarga o LONGMEMEVAL_DATA_PATH; documento por sesión/pregunta, unidad question_id; pregunta tiene answer/type/date y haystack separado |
| [LoCoMo](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/dataset/locomo.py#L51-L110), [carga](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/dataset/locomo.py#L245-L338) | Unidad conversación; docs por sesión con fecha; excluye categoría 5 adversarial, no toda pregunta difícil; no confundir `locomo` con `locomo-fixed` |
| [RAGMode](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/modes/rag.py#L29-L79) | Cronómetro retrieval rodea async_retrieve, excluye generación/juez; concatena documentos y genera respuesta estructurada |
| [Modelos](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/llm/__init__.py#L21-L40), [Gemini](https://github.com/vectorize-io/agent-memory-benchmark/blob/28b644bd4dd653c0207c7507280439390c829155/src/memory_bench/llm/gemini.py#L19-L87) | Answer provider default groq, juez default gemini; Gemini default gemini-2.5-flash-lite, temperatura 0, seed solo si OMB_SEED; modelo Groq concreto no fijado en la muestra |

Fallo no transitorio de `_process_one` se propaga por gather y puede impedir
resumen final; guardado parcial por unidad no equivale a una fila por cada
intento. Contexto vacío se marca incorrecto para QA. `_save` persiste preguntas,
answers y gold: contiene payload sensible, **no** es telemetría segura.
Costes incluyen extracción/embeddings/rerank remotos y llamadas answer/juez;
no se midieron importes. Cachés/skip/reset deben conservarse en la comparación;
no atribuir una corrida warm/skip al coste cold.

Adaptación futura: la entrada `Document(id,content,user_id,timestamp)` debe
mapearse a ingesta de sesión y provenance Vanta; `user_id` a ámbito autorizado,
no a autorización implícita; `retrieve` a `RecallConfig`/SDK real y salida a
evidencias con IDs de fuente. Reutilizar `evals/eval_locomo.py` y el método
existente **sin diseñar harness/prompts/rúbricas/configuración nuevos**.
No trasladar reset por banco a namespaces compartidos. Fallos, modalidad
efectiva core/legacy y degradación sin embedding deben quedar visibles.

### Matriz trazable y seam visual

Las fuentes V1–V4 son Vanta al SHA fijo, no el índice CodeGraph de HEAD.

| Capacidad upstream | Vanta: estado y fuente | Delta / decisión |
|---|---|---|
| Consolidación con fuentes vivas | Existe provenance/version/supersession, parcial garantía condicional [V1] | A; no certificar CAS por `persist_planned` |
| Preservación de tags concurrentes | Upstream confirmado en [H-tags]; paridad de publicación Vanta no acreditada | A: distinguir guard de fuentes y merge desde metadata actual; no heredar bug de v0.9.2 |
| Cancelación de recall/reflect abandonado | Upstream token [H12] y fix declarado #2127; cobertura Vanta por transporte no certificada | G: propagación/recursos por etapa, no ausencia global ni cancelación instantánea de proveedor |
| Recall multibrazo | Existe `recall_candidates:321-381`; [lector](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_reader.rs#L321-L381) | B; ampliar explicación, no RRF nuevo |
| Temporalidad explícita | Parcial metadata/filtros; no acreditada paridad occurred/mentioned [V1][V2] | D; sin backfill de fechas inventadas |
| Bancos y ámbitos de consolidación | Existe tenancy/injection [V1][V2], no acreditada equivalencia de particionado LLM | E; no banco como ACL automática |
| Reflejar vs publicar | Existe `Reflector`/`reflect_session` [V3] | C/G; conservar lectura sin escritura implícita |
| Validación de extracción por item | Parcial pipeline/write y degradación opcional [V1] | F; no declarar bug actual de correlación |
| Presupuesto/context assembly | Existe [V2][V4] | Integrar, no coordinador alternativo |
| Recibo de escritura | Existe `WriteReceipt`, [attestation](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L657-L735) | Hash SHA-256 **no firma**; MutationReceipt C13 sigue solo propuesta |

UI upstream web real: [POST proxy recall](https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-control-plane/src/app/api/recall/route.ts#L1-L65)
elige banco, transmite tags/tiempo/budget al SDK y devuelve cuerpo completo;
errores se convierten en 500 en ese proxy. `ObservationHistoryView` [H11]
inspecciona revisiones, no se acredita aquí desktop nativo.
Studio real leído al SHA separado **`0ad38c542b39a589035b0e17fdedb4c09dca24b9`**:
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409,488-550`
selecciona sesión/tabs y consulta genlog/anchors;
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:52-89`
llama `auditEvents` y distingue error/no configurado;
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228`
implementa search con explain y get/vecinos/audit. Cuerpos releídos mediante
`git show`, no bytes dirty de `desktop/...`. Extender esas superficies con
outcome/revisiones, sin guardar payload ni montar otro control plane. Es
snapshot de UI distinto del motor, no prueba de paridad visual ejecutada.
FMEA/aceptación UI: tags actuales/revisión distintos del snapshot consultado
deben verse como conflicto, no borrarse desde preview; cancelación solicitada
no significa proveedor ya detenido. No hay JobChip/jobs acreditados en el pin.
Recibo [H-1/H-2](reviews/memory-additions.md#hindsight): correcciones aplicadas
a contrato, vigencia, matriz y UI; reconfirmación independiente pendiente.
Fuentes Studio inmutables de los cuerpos releídos (2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

[H-tags]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py#L3445-L3495

**Licencia por componente:** [LICENSE raíz MIT](https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/LICENSE#L1-L21)
cubre la fuente propia leída del engine, control plane y wrapper; SDKs
generados/proveedores y PostgreSQL/pgvector/Oracle/modelos tienen obligaciones
separadas no auditadas individualmente. No se transfiere licencia del servicio
Oracle ni de pesos desde MIT del wrapper. AMB y sus datasets requieren revisión
propia antes de distribución; aquí solo se leyó su código remoto.

**Cobertura/cierre de autoría:** ampliados siete mecanismos, API/modelos,
errores/cancelación/fronteras de commit, cuatro incidencias, runner AMB real,
matriz y UI. No examinados exhaustivamente Oracle, permisos de extensiones,
retención física/backups, todos los modelos/configs ni dataset binario. No se
ejecutó nada del producto, no hay medición ni aceptación independiente.
Si se autoriza después, separar contratos de publicación A/C/E/F y adapters
B/D/G en specs; conservar este dossier como evidencia y decisiones, no crear
esas specs dentro de esta tarea.

[H12]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/models.py#L15-L183
[H13]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/api/http.py#L5455-L5518
[H14]: https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/retain/orchestrator.py#L4133-L4335
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L117-L315
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L302-L335
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/reflection/mod.rs#L118-L186
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L228-L329
