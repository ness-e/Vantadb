---
title: "graphify: grafo explicable como entrada opcional de memoria"
kind: research
status: proposed
description: "Contraste estático de procedencia, identidad, invalidación y consultas de Graphify con los seams existentes de VantaDB"
tags: [research, memory, engineering]
---

# graphify: grafo explicable como entrada opcional de memoria

## Índice

- [1. Dictamen y arquitectura](#1-dictamen-y-arquitectura)
- [2. Fichas de adaptación](#2-fichas-de-adaptación)
- [3. Riesgos y evidencia negativa](#3-riesgos-y-evidencia-negativa)
- [4. Prerrequisitos VantaDB](#4-prerrequisitos-vantadb)
- [5. No trasladar](#5-no-trasladar)
- [6. Método y ledger](#6-método-y-ledger)
- [7. Benchmarks reales y límites](#7-benchmarks-reales-y-límites)
- [8. Matriz de brechas](#8-matriz-de-brechas)
- [9. Debilidades sugeridas de VantaDB](#9-debilidades-sugeridas-de-vantadb)
- [10. Interfaz y escritorio](#10-interfaz-y-escritorio)

## 1. Dictamen y arquitectura

**Propuesta:** admitir un export Graphify versionado como productor opcional de
evidencias, sin integrar NetworkX ni otro catálogo de parsers en Vanta.
El patrón más interesante es preservar evidencias rivales de una misma arista,
no usar “semántico” como sinónimo de verdadero.

- Repo `Graphify-Labs/graphify`, rama congelada `v8`; HEAD comprobado
  `16218bfc80d4ca2799734d045039f7ed1a5fa9fb`.
- [Snapshot](github-candidates.json): **143 653 estrellas**,
  `2026-11-07T06:57:16.2284649Z`, licencia detectada Apache-2.0.
- Stack observado: Python, tree-sitter, NetworkX, diccionarios/JSON, librería/CLI,
  skill para agentes, MCP stdio/HTTP y exportaciones HTML/wiki/Obsidian/Cypher.
  `llm.py` conserva enriquecimiento y etiquetado opcionales; el selector
  `GRAPHIFY_BACKEND=none` bloquea backends LLM en rutas inspeccionadas.
- Flujo: detección → extracción AST/documental → resolución interarchivo →
  validación → build/merge → comunidades/análisis → JSON/visores/consultas.
  `_origin` separa AST/semantic; `EXTRACTED`, `INFERRED`, `AMBIGUOUS` son clases,
  no probabilidades calibradas. La modalidad AST local no prueba que toda
  ejecución de la skill, ingestión URL o proveedor LLM sea offline.

Vanta ya posee wiki ingestors con procedencia, búsqueda híbrida, grafo y
`code_index`/`code_*`; [code_index.rs](../../../../vantadb-mcp/src/code_index.rs),
líneas 193–315 y 514–520, contiene incluso rutas para Python/TS/JS además de Rust.
No son todavía evidencia de paridad AST con Graphify. El scope canónico es
`FIND-299` (en `Backlog.md` (registro local, no versionado por git)), línea 285, y
`mgr-22-repo-map` (registro historico, sin destino localizable en el repo; ver FIND-299 en `Backlog.md` (registro local, no versionado por git)), no un nuevo core de código.
El CodeGraph externo del entorno de desarrollo tampoco implementa por sí mismo
memoria de producto: se necesitan contrato y persistencia canónica explícitos.

## 2. Fichas de adaptación

### Ficha A — Importación con evidencia por arista

- **Problema/unidad mínima:** `edge_evidence` y `_fold_also_seen`,
  `graphify/build.py:253–312` (G1), preservan confianza, score y localización
  cuando colapsan duplicados de igual relación. No reducirlo a un peso.
- **Seam existente:** [MemoryInput](../../../../src/sdk/types/record.rs),
  líneas 184–224, y [Embedded::add_edge](../../../../src/sdk/api/graph.rs),
  líneas 113–151. La API de aristas no admite el objeto de evidencia completo.
  Guardar evidencia como memoria canónica asociada a la relación.
- **DTO Rust propuesto:** `ImportedCodeRelation { repo_id, revision,
  producer_schema, source_ref, target_ref, relation, direction,
  evidence: Vec<SourceEvidence>, unresolved }`;
  `SourceEvidence { origin, confidence_class, reported_score: Option<f32>,
  path, span, digest, verification }`. `reported_score` no se convierte
  automáticamente a confidence derivada de Vanta.
- **Pasos/dependencias:** fijar SHA y root → leer JSON permitido → validar
  endpoints, dirección y fuentes → importar AST y semantic como evidencias
  separadas → mapear a IDs Vanta → publicar lote idempotente.
  Depende de schema del conector, identidad de repo y soporte de evidencia asociada.
- **FMEA:** merge destructivo oculta evidencia contradictoria; grafo no dirigido
  invierte callers/callees; stub externo se interpreta como código disponible.
  Mantener `also_seen`, dirección y clase external/unresolved.
- **Aceptación:** reimportación no duplica evidencias; dos fuentes de una relación
  sobreviven; aristas inversas no se fusionan; un stub no ofrece snippet inventado.
- **No objetivos:** replicar build/cluster, deduplicación semántica o exportadores.

### Ficha B — Identidad e invalidación como contrato de productor

- **Unidad:** `normalize_id`/`DeclIdRegistry`, `graphify/ids.py:51–140` (G2);
  `_AST_CACHE_SCHEMA`, fingerprints de gramática/prompt y guards de mtime en
  `cache.py` (G3); colas `_queue_pending`/`_drain_pending` de `watch.py:33–70` (G4).
  Normalizar Unicode de forma idempotente evita que productor y consumidor
  diverjan; no garantiza persistencia de identidad al mover o renombrar archivos.
- **Seam:** hash/manifiesto por archivo de Vanta `code_index.rs:193–235,476–486`;
  `FIND-299` contempla watcher. No iniciar otro watcher desde este conector.
- **Contrato propuesto:** `CodeGraphSnapshot { repo_id, source_revision,
  worktree_digest, extractor_version, schema_version, generation,
  nodes, relations, removed_refs, completeness }`. Una generación
  incompatible se rechaza o migra explícitamente; no se “arregla” silenciosamente.
- **Pasos/dependencias:** conservar ID externo original y clave interna separada;
  exigir manifest completo o delta con tombstones; invalidar por cambio de fuente,
  gramática/modelo/prompt; validar entrantes a archivos cambiados antes de publish.
- **FMEA:** mismo mtime/tamaño no asegura mismos bytes; normalización casefold
  confunde símbolos distintos; pérdida de evento deja snapshot obsoleto;
  mover archivo parece alta+baja. Registrar alias/move solo con evidencia,
  nunca unir entidades por similitud de nombre.
- **Aceptación:** schema desconocido no escribe parcialmente; actualización de
  gramática no reutiliza evidencia antigua como fresh; delete elimina relaciones
  propiedad del productor y no borra memoria manual; un move conserva trazabilidad.
- **No objetivos:** trasladar todos los caches, workers o hooks a Vanta.

### Ficha C — Consultas que declaran lo que no cubren

- **Unidad:** `_query_terms`, `_bfs`, `_subgraph_to_text`, `_query_graph_text`
  en `serve.py:317,1110,1168,1489` (G5); `resolve_seed_outcome` y
  `affected_nodes`, `affected.py:302–408` (G6).
  Desambiguar formas de referencia y respetar dirección antes del recorrido.
- **Seam:** `code_search` y `code_impact`, [code.rs](../../../../vantadb-mcp/src/code.rs),
  líneas 12–20 y 272–301; Vanta ya ensambla contexto con presupuesto.
- **DTO propuesto:** `CodeContextSlice { query, normalized_terms,
  revision, seeds, evidence_keys, omitted, truncated,
  traversal_direction, max_hops, resolution_state }`.
  Una consulta no resuelta no se presenta como “sin dependencias”.
- **Pasos/dependencias:** resolver ID/nombre cualificado → conservar alternativas
  ambiguas → recuperar vecindario → deduplicar por evidencia → presupuestar salida
  completa con avisos → entregar a context assembly canónico.
- **FMEA:** BFS recortado parece cierre exhaustivo; stemming/casefold pierde
  identidad; una relación semantic débil cambia blast radius.
  Diferenciar `related` de `calls` y corte por presupuesto de ausencia.
- **Aceptación:** límites y omisiones se ven en texto y DTO; homónimos no se
  resuelven por orden del grafo; dirección inversa cambia el resultado explicado.
- **No objetivos:** autorizar exclusión de tests desde `affected`; no se acreditó
  aquí un selector seguro con cierre transitivo y cobertura runtime.

## 3. Riesgos y evidencia negativa

| Hallazgo estático | Alcance de la afirmación | Mitigación transferible |
|---|---|---|
| `build_from_json(... directed=False)` por defecto (G1) | El modelo de almacenamiento puede ser no dirigido; no afirmar que todo export pierde dirección | Leer marcadores/dirección y rechazar interpretación ambigua |
| `EXTRACTED/INFERRED/AMBIGUOUS`, `also_seen` (G1/G7) | Etiquetas y evidencia existen; no prueba calibración o completitud | Conservar origen, no promocionar a certeza |
| Evidence binding marca símbolos fabricados `unverified` (G8) | El símbolo no se elimina automáticamente | Consumidor distingue verificado de pendiente |
| Cache usa schema y fingerprints (G3) | La cabecera arquitectónica menciona una versión anterior; el código fija 39 | Tomar versión real del productor; no confiar en prosa desactualizada |
| `benchmark.py` excluye consultas sin match (G9) | Sesgo verificable de denominador del ratio de reducción | No reutilizar su ratio como eficacia QA |

No se consultaron issues remotos ni se reprodujeron vulnerabilidades.
`security.py` contiene validación URL, resolución de direcciones, límites de
descarga/archivo, rutas de grafo y redacción/sanitización: son controles localizados,
no certificación de SSRF/XSS. `serve_http` ofrece bearer/API key y restricciones
Host; annotations MCP no sustituyen ACL ni aislamiento por tenant.
Un archivo local malicioso sigue siendo entrada no confiable, incluso offline.

## 4. Prerrequisitos VantaDB

1. Revalidar árbol concurrente y FIND-299 antes de tarea: baseline HEAD
   `c561158253bb88192590fa406c61e0a27dd7df8d`; comentarios antiguos no describen
   necesariamente el código vigente (lectura histórica del worktree inicial).
2. Definir namespace por repo/productor, revisión, esquema y ownership de
   tombstones. IDs Graphify normalizados no se convierten directamente a IDs u128.
3. Respetar metadatos planos y límites de clave/payload; conservar procedencia
   existente en [wiki/ingestors.rs](../../../../src/wiki/ingestors.rs), líneas 1–27.
   No enviar código por el ingestor documental que lo excluye.
4. Una confianza declarada por upstream no elude reglas `Derived`/`derived_from`
   de Vanta. La memoria fuente debe existir antes de la evidencia derivada.
5. Acordar snapshot/delta y invalidación interarchivo; conectar al ensamblado
   existente, no crear presupuesto paralelo ni prometer mejora sin comparación.

## 5. No trasladar

No añadir NetworkX, todo tree-sitter, proveedores LLM, ingestión arbitraria URL,
instalación de skills/hooks ni otro daemon al motor. No copiar el número estimado
de tokens como contador real ni usar un grafo visual como prueba de ejecución.
Licencia local Apache-2.0 y presencia de `LICENSE-MIT`/`NOTICE` requieren revisión
por archivo y dependencias antes de copiar/adaptar. **Reescribir en Rust no es una
exención de atribución, avisos u obligaciones sobre material derivado.**

## 6. Método y ledger

Conforme a [METHOD](METHOD.md): solo lectura de fuente local fijada; tests leídos
pero no ejecutados, sin servicios nuevos ni indexación. No se alteraron MCPs
existentes. No se crearon prompts, rubric, grader, harness ni configuración de eval.

| ID | Fuente inmutable y alcance leído |
|---|---|
| G1 | [build.py:253–327](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/build.py#L253-L327), [1524](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/build.py#L1524): evidencia y constructor |
| G2 | [ids.py:51–140](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/ids.py#L51-L140): normalización, salts y colisiones |
| G3 | [cache.py:41–86](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/cache.py#L41-L86), [235–284](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/cache.py#L235-L284): schema y timestamps |
| G4 | [watch.py:1–70](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/watch.py#L1-L70): cambios pendientes y lock contention |
| G5 | [serve.py:317–354](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/serve.py#L317-L354), [2856–2876](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/serve.py#L2856-L2876): términos y dispatch presupuestado |
| G6 | [affected.py:302–408](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/affected.py#L302-L408): recorrido acotado y referencia |
| G7 | [validate.py:1–90](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/validate.py#L1-L90): campos obligatorios y clases |
| G8 | [test_evidence_binding.py:1–55](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/tests/test_evidence_binding.py#L1-L55), [llm.py:2128–2155](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/llm.py#L2128-L2155): verificación y backend opcional |
| G9 | [benchmark.py:1–103](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/benchmark.py#L1-L103): comparación corpus/contexto |
| G10 | [security.py:83–105](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/security.py#L83-L105), [330–372](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/security.py#L330-L372): controles localizados |
| G11 | [export.py:303–448](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/export.py#L303-L448): export y dirección |

**No examinado exhaustivamente:** `extractors/`, todos los resolvers multiidioma,
`always_on/`, `pi/`, providers, importadores externos, red, generación completa de
HTML y seguridad del navegador. No se verificaron indexación, move/rename,
migración de caches viejos, interoperabilidad MCP o runtime local-only.

## 7. Benchmarks reales y límites

El runner localizado es `graphify.benchmark.run_benchmark` (G9), invocado por la
superficie `graphify benchmark`. Entrada: JSON de grafo, `corpus_words` opcional
y preguntas; requiere Python/NetworkX y loader/query de `serve.py`.
Usa respuesta real de `_query_graph_text` en modo BFS, profundidad 3 y presupuesto
2000, pero **estima** tokens como caracteres/4 y corpus como palabras ×100/75;
sin palabras de corpus usa nodos ×50. No registra consumo de modelo ni llamadas.
Promedia solo preguntas con match; un ratio alto puede coincidir con baja cobertura.

[test_benchmark_real_query.py:35–75](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/tests/test_benchmark_real_query.py#L35-L75)
comprueba salida real, corte por presupuesto y exclusión de no-match; son
tests sintéticos leídos, no resultados ejecutados en esta investigación.
[BENCHMARKS.md:1–85](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/BENCHMARKS.md#L1-L85)
publica claims de LoCoMo/LongMemEval/ERPNext y menciona suites `memory/`/`crosstool/`.
No se localizaron ni inspeccionaron esos runners en las superficies revisadas;
no se acredita reproducción end-to-end de esas tablas.

Para Vanta rige [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks), no
el denominador del ratio upstream: separar motor/retrieval/QA/producto, congelar
modelo y contexto, incluir no-match/fallos, medir tokens de cliente y llamadas
totales. No afirmar ahorro real, superioridad QA ni cifras nuevas.

## 8. Matriz de brechas

| Capacidad | Vanta | Delta propuesto |
|---|---|---|
| Memoria, procedencia, búsqueda y wiki | Existe, baseline METHOD | Conservar como canónico |
| Indexado código y relaciones | Existe slice/scanners; paridad AST no acreditada | Importación opcional, no parser core |
| Evidencias múltiples por relación | Parcial: memorias/metadata y aristas simples | Registro asociado equivalente a `also_seen` |
| Estado ambiguo/no verificado | No acreditado como contrato code completo | No confundir con ausencia de símbolo |
| Hash/reconciliación incremental | Existe por archivo; watcher completo pendiente de revalidar | Manifiesto de generación del productor |
| Cierre transitivo seguro para selección de tests | No acreditado | `affected` es asesor, no gate |
| Visor Graphify dentro de Studio | No aplicable | UI externa, Studio separado |

## 9. Debilidades sugeridas de VantaDB

Vanta `code_index.rs:310–315` asigna peso 1 a coincidencias textuales; no representa
automáticamente evidencia AST/LSP. Comparar solo símbolos acumulados tras saltar
archivos sin cambios (`193–202`) sugiere revisar relaciones interarchivo y
reconciliación, sin convertirlo en bug vigente sin reproducción. La cabecera
Rust-only ya diverge de `extract_symbols_for_lang`: revalidar backlog y fuente,
no copiar la narrativa histórica. Otra mejora sería devolver incomplete/unknown
cuando el presupuesto no permite observar todo el vecindario.

## 10. Interfaz y escritorio

Hay fuente de exportación HTML (`export.py`), callflow/tree HTML y wiki/Obsidian;
no se acredita escritorio nativo. Son productos visuales de un snapshot:
comunidades y cercanía gráfica no equivalen a llamadas ni trazas runtime.
`to_json(... built_at_commit=...)` y dirección preservada son ideas útiles para
mostrar revisión y abrir evidencia. No se abrió ningún visor ni se validó XSS,
accesibilidad o empaquetado. Vanta Studio vive en `develop-desktop`; un conector
no implica portar Graphify UI ni negar las pantallas de memoria ya existentes.
