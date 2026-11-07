---
title: "Graphiti: temporalidad de hechos y recuperación por superficies"
kind: research
status: proposed
description: "Lectura estática de episodios, invalidación temporal y búsqueda de Graphiti para ideas independientes en Rust"
tags: [research, memory, engineering]
---

# Graphiti: temporalidad de hechos y recuperación por superficies

## Identidad y dictamen

- Repositorio: `getzep/graphiti`; clon `C:\Users\Eros\graphiti`.
- SHA congelado: `747b480ec6b12ecb718f1e2bc588b20ee36ea9c2`, rama `main`.
- Snapshot: **32 614 estrellas**, Apache-2.0 según
  [inventario](github-candidates.json); revisión según [manifest](clone-manifest.json).
- Método: [METHOD](METHOD.md). Lectura offline, sin ejecutar código ni consultar issues.
- Vanta: base del estudio `c561158`; HEAD comunicado `167435a9e`.
  No se reaudita Vanta ni se presentan defectos históricos como actuales.

**Dictamen:** estudiar separación entre tiempo del hecho y tiempo de conocimiento,
y contratos de recuperación por tipo de evidencia. No portar el framework,
sus drivers ni su pipeline LLM completo. BM25/HNSW/RRF, procedencia,
supersession, dedup y ensamblado **ya existen** en Vanta.

## 1. Arquitectura, tecnologías, flujo y recomendaciones

**Código:** biblioteca Python con modelos Pydantic, clientes LLM/embeddings,
drivers de grafos y un orquestador `Graphiti`. `pyproject.toml:1-70` declara
Python >=3.10, Neo4j, OpenAI y extras FalkorDB/Neptune, entre otros [G0].
Los adaptadores REST/MCP no son el dominio temporal.

Flujo leído de `add_episode`, no reconstrucción imaginaria del producto [G1]:

1. Recupera episodios previos con `reference_time`, `group_id` y tipo de fuente;
   si se suministran UUID previos, usa la ruta explícita de carga.
2. Reutiliza episodio por UUID o construye uno con contenido, fecha de creación
   y `valid_at=reference_time`.
3. Extrae nodos y resuelve identidades; después extrae/resuelve relaciones.
4. Combina relaciones resueltas e invalidadas. Para atributos/resumen de nodos
   pasa solo `new_edges`, evitando volver a resumir hechos existentes.
5. Delega persistencia a `_process_episode_data`; comunidades se actualizan
   después, únicamente si se solicitan.

La secuencia acredita orquestación, **no** una transacción universal entre
extracción remota, persistencia y comunidades. No se inspeccionó su atomicidad
en todos los drivers.

`EntityEdge` conserva `episodes`, `valid_at`, `invalid_at`, `expired_at` y
`reference_time` [G2]. El primero enlaza evidencia; los timestamps no son
intercambiables: fin de validez del hecho no equivale a cuándo se invalidó
en el sistema. Los comentarios distinguen planes futuros de fechas efectivas.

La invalidación compara intervalos y permite que un candidato posterior
invalide el hecho recién llegado, importante para ingestión fuera de orden.
Los intervalos que no se solapan se excluyen de contradicción temporal [G3].
El helper recibe candidatos contradictorios: **no descubre por sí solo**
la incompatibilidad semántica.

`search` valida grupos, puede reutilizar un vector y ejecuta brazos de
relaciones, entidades, episodios y comunidades [G4]. La configuración enumera
BM25, coseno y BFS según superficie; rerankers incluyen RRF, MMR, distancia,
menciones y cross-encoder [G5]. No implica que toda consulta ejecute todos.

**Recomendación Vanta:** conservar el retrieval actual; añadir, si se justifica,
semántica temporal explícita y explicación de qué superficie produjo cada
candidato. LLM para interpretar contradicciones como componente opcional,
no requisito de una comparación de intervalos.

## 2. Unidades mínimas candidatas a reimplementación independiente

### Unidad A — decisión temporal pura antes de publicar supersession

- **Problema:** confundir llegada tardía, fin de validez y momento de revisión
  puede hacer que el hecho más recientemente escrito parezca el más actual.
- **Fuente mínima:** campos de `EntityEdge` [G2] y comparación/invalidation
  en `_apply_temporal_invalidation` / `resolve_edge_contradictions` [G3].
- **Seam Vanta:** writer y dedup de L1,
  `vanta-memory/src/core/record/l1_writer.rs` y `l1_dedup.rs`;
  complementar la supersession existente, no reemplazar su modelo.
- **DTO HIPOTÉTICO Rust, no API actual:** `FactInterval { valid_from,
  valid_until, reference_time }`; `TemporalDecision { keep, supersede,
  reason, evidence_ids }`. Timestamps opcionales representan desconocimiento,
  no una fecha artificial.
- **Interfaz HIPOTÉTICA:** `resolve_intervals(request) -> Result<Decision,
  TemporalError>`; entrada contiene hechos ya autorizados y una relación de
  contradicción acreditada por el llamador.
- **Acciones concretas:** inventariar timestamps actuales; fijar significado
  de extremos y UTC; implementar comparación pura; enlazar decisión a fuente;
  publicar con el mecanismo de escritura vigente; exponer razón en auditoría.
- **Relaciones/dependencias:** provenance identifica fuentes; dedup decide
  identidad; supersession registra sustitución; scheduler puede reprocesar
  hechos tardíos. Ninguna de estas responsabilidades debe duplicarse.
- **FMEA:** fechas extraídas incorrectas invalidan evidencia válida;
  timestamps desconocidos tratados como cero ordenan falsamente; igualdad
  de fechas puede dejar conflicto sin decidir; cambio concurrente puede
  publicar sobre una revisión obsoleta.
- **Aceptación funcional propuesta:** llegada tardía no vence a un hecho
  posterior solo por insertarse después; intervalos disjuntos no se invalidan;
  fecha desconocida produce decisión explícita; toda sustitución mantiene
  evidencia e identifica la revisión sobre la que se decidió.
- **Fuera de unidad:** inferir verdad del mundo, reconstruir historial
  perdido o traducir los prompts upstream.

### Unidad B — recuperación tipada con procedencia del brazo

- **Problema:** entidad, hecho y episodio son unidades distintas; mezclar sus
  scores como si fueran probabilidades oculta por qué se entregó el contexto.
- **Fuente mínima:** agregador `search` [G4], configuraciones y `SearchResults`
  con listas/scores por superficie [G5].
- **Seam Vanta:** `vanta-memory/src/context_engine/engine.rs` y recall L1;
  reutilizar BM25/HNSW/RRF y presupuesto compartido del assembler.
- **DTO HIPOTÉTICO:** `RecallCandidate { id, revision, evidence_refs,
  origin, rank, temporal_state }`; `origin` distingue literal, derivado y
  expansión de relación. No se propone copiar las cuatro listas Python.
- **Interfaz HIPOTÉTICA:** `retrieve_surfaces(ScopedRecall) -> RecallBundle`,
  con resultados y estado de cada brazo, separado del renderizado.
- **Acciones:** mapear tipos actuales; conservar origen durante fusión/dedup;
  aplicar permisos antes de expansión; limitar trabajo; enviar al assembler
  candidatos con referencias, no una cadena opaca preparada por el backend.
- **Dependencias:** Unidad A solo para vistas temporales; evidencia y scopes
  son obligatorios incluso sin temporalidad; embeddings y reranking opcionales.
- **FMEA:** expansión cruza scope; mismo hecho consume presupuesto varias
  veces; fallo de un brazo se interpreta como cero relevancia; score de
  reranker se compara directamente con BM25 sin contrato.
- **Aceptación propuesta:** cada candidato tiene origen y evidencia;
  brazo indisponible se distingue de búsqueda vacía; expansión no amplía
  autorización; duplicados no inflan contexto; sin embeddings sigue
  disponible la ruta textual ya existente.
- **No objetivo:** introducir Neo4j, un segundo RRF o nuevas constantes
  de benchmark para aparentar una mejora.

## 3. Defectos, riesgos e issues: niveles de evidencia

- **Defecto estático del runner leído:** `add_episode_context` es lista de
  strings ([G6], líneas 40-68). En el bucle del juez cada `episodes` es un string;
  `message=episodes[0]` y `previous_messages=episodes[1:]` pasan primer carácter
  y resto, no mensaje completo e historial ([G6], líneas 155-170).
  El efecto sobre puntuaciones no fue medido.
- **Límite estático:** `zip(..., strict=False)` puede truncar desajustes del
  baseline; el denominador sigue siendo número de resultados candidatos.
  No atribuir a ese score recall de hechos ni exactitud de respuestas.
- **Código:** `search` normaliza lista de grupos vacía a `None` ([G4], líneas 155-156).
  **Hipótesis de seguridad condicionada:** si una API usa vacío para representar
  «ningún grupo permitido», podría ampliar una consulta. No se auditó si
  los adaptadores permiten esa combinación ni sus controles de autorización.
- **Hipótesis:** fallos entre guardar episodio y actualizar comunidades pueden
  dejar derivados temporalmente desalineados. No se reproduce ni se declara
  corrupción persistente del producto.
- **Issues:** no investigados; comentarios y guías del clon no sustituyen una
  auditoría de incidencias vigente.

## 4. Prerrequisitos Vanta

Confirmar semántica actual de supersession y tiempo antes de diseñar cambios.
Definir fechas desconocidas y lecturas históricas, identidad estable de fuente
y control de revisión al publicar. Asegurar que filtros sobrevivan a expansiones.
No exigir LLM ni red para almacenar o recuperar evidencia literal.
Las rutas Vanta citadas son seams candidatos, no una reauditoría del HEAD actual.

## 5. Qué no trasladar

No importar drivers remotos por necesitar relaciones temporales.
No igualar contradicción detectada por LLM con verdad validada.
No portar comunidades, saga, prompts, clientes y API como un bloque indivisible.
No interpretar `group_id` como política de acceso suficiente.
No sustituir assembler/compaction ni BM25/HNSW/RRF existentes.

## 6. Método, licencia y límites

Apache-2.0 en metadata y manifiesto Python; estudiar ideas no concede excepción
a notices, patentes o licencias de dependencias/modelos [G0].
Una reimplementación independiente en Rust necesita especificación propia:
cambiar lenguaje no elimina derivación. Clon superficial sin otras ramas,
tags, historial previo, submódulos inicializados ni descarga LFS.
No se probaron servicios, concurrencia, calidad de extracción o paridad de drivers.

## 7. Runners upstream leídos

`eval_cli.py:7-40` llama opcionalmente baseline y siempre evaluación [G7].
`eval_e2e_graph_building.py:32-180` usa LongMemEval **oracle**, toma
`haystack_sessions`/fechas y limita mensajes con un contador por usuario
que abarca sus sesiones. Es evaluación de **construcción de grafo** contra
baseline, no QA LongMemEval ni benchmark ANN [G6].

Dependencias: pandas, Graphiti, Neo4j y cliente OpenAI con modelo nombrado
`gpt-6-luna` en el código; no se verificó disponibilidad real de ese modelo.
Escribe `baseline_graph_results.json` y `candidate_graph_results.json`, ingiere
grupos baseline/candidate y llama al juez LLM. La ruta del dataset es relativa
al directorio de ejecución. El score promedia «candidate no peor» por usuario;
no mide precisión independiente del baseline ni todos los errores operativos.
No se acredita cleanup/cierre del cliente en el cuerpo leído.

El defecto de construcción del contexto del juez limita interpretar resultados.
No se ejecutó, reparó ni rediseñó nada: prompts, muestreo y configuración
upstream permanecen intactos. Comparación futura únicamente según
[protocolo común](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Capacidad | Estado Vanta usado aquí | Diferencia investigada |
|---|---|---|
| BM25/HNSW/RRF | Existe | origen tipado y expansión |
| Fuentes/supersession/dedup | Existe | intervalos y llegada tardía |
| Assembly/compaction con presupuesto | Existe | evidencia de varios brazos |
| Scheduler/checkpoints/ingestors | Existe | reprocesamiento temporal |
| Temporalidad del dominio | Parcial para este contraste | supersession no acredita equivalencia bitemporal |
| Contrato bitemporal completo | No acreditado | no se inspeccionó Vanta para certificarlo |
| Drivers Neo4j/FalkorDB como requisito | No aplicable | no necesarios para extraer principios |
| Calidad equivalente a claims upstream | No acreditado | no hubo mediciones |

## 9. Deuda sugerida, no bugs Vanta inventados

Preguntas para una futura tarea: ¿se distingue fecha efectiva de fecha de
conocimiento?, ¿qué ocurre con evidencia tardía y fecha desconocida?,
¿se conserva origen tras fusionar?, ¿se comunica fallo parcial de retrieval?
Primero contrastar contratos actuales y cambios posteriores a `c561158`.
La propuesta es verificar composición, no declarar ausencias ni abrir tareas
duplicadas a partir de defectos históricos.

## 10. UI/escritorio y aplicación a Studio

En las superficies inspeccionadas hay biblioteca, REST/MCP y evaluaciones;
no se localizó fuente de un dashboard operativo propio en la búsqueda acotada
de componentes TSX/Vue/Svelte. No demuestra ausencia de interfaces externas
ni convierte Neo4j Browser en UI propia de Graphiti.
Studio Tauri/React ya existe en `develop-desktop`: una aplicación pertinente
sería mostrar intervalos, fuente e invalidación en memoria/diagnóstico actuales,
no crear otro escritorio ni portar un visor Neo4j.

### Ledger de lectura efectiva

| Ref. | Archivo y rangos leídos | Evidencia |
|---|---|---|
| G0 | `pyproject.toml:1-115` | stack, extras y licencia |
| G1 | `graphiti_core/graphiti.py:1432-1552` | ingestión de episodio |
| G2 | `graphiti_core/edges.py:263-299` | tiempo y procedencia |
| G3 | `graphiti_core/utils/maintenance/edge_operations.py:714-785` | comparación/invalidación |
| G4 | `graphiti_core/search/search.py:70-220` | agregación y scopes |
| G5 | `graphiti_core/search/search_config.py:20-150` | tipos y métodos |
| G6 | `tests/evals/eval_e2e_graph_building.py:1-180` | runner y defecto del contexto |
| G7 | `tests/evals/eval_cli.py:1-40` | entrada CLI |

También se leyó el README del dataset y las instrucciones AGENTS/CLAUDE.
No cubiertos: todos los drivers, filtros completos, implementaciones de cada
reranker, transacciones, auth REST/MCP y todos los tests. Pendiente manual:
concurrencia, recuperación, alcance autorizado y reproducción de los runners.
Sin tests, builds, servicios, benchmarks, red, validadores, commits ni push.

[G0]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/pyproject.toml#L1-L115
[G1]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L1432-L1552
[G2]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/edges.py#L263-L299
[G3]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/utils/maintenance/edge_operations.py#L714-L785
[G4]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/search/search.py#L70-L220
[G5]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/search/search_config.py#L20-L150
[G6]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_e2e_graph_building.py#L1-L180
[G7]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_cli.py#L1-L40
