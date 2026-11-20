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
- [11. Dossier ampliado de flujos y evidencia](#11-dossier-ampliado-de-flujos-y-evidencia)
- [12. Fichas operativas y cierre](#12-fichas-operativas-y-cierre)

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
- **Contrato Rust propuesto:** único `CodeEvidenceImport` C12 de
  [CONSOLIDATED](CONSOLIDATED.md#c12--codeevidenceimport-contrato-canónico-único-de-evidencia-code-memory).
  `also_seen` produce evidencias asociadas separadas de una misma relación;
  origen, dirección y referencias se preservan. El score reportado no se
  convierte automáticamente a confianza derivada de Vanta.
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
- **Contrato propuesto:** generación de C12, con manifiesto de completitud,
  digest del worktree y bajas propiedad del productor como control de aplicación.
  No se crea un segundo `CodeGraphSnapshot` persistente. Una generación
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
  GF-1: una negativa por nombre ambiguo también es salida textual medible,
  no contexto útil. Conservar `resolution_state` y no promover longitud baja
  a éxito de recuperación.
- **Aceptación:** límites y omisiones se ven en texto y DTO; homónimos no se
  resuelven por orden del grafo; dirección inversa cambia el resultado explicado.
  Refusal ambiguo, no-match sentinel, error y subgrafo útil permanecen distintos
  en el recibo/inspector; no cambiar el benchmark para forzar esta aceptación.
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

La ampliación §11 incorpora incidencias remotas leídas; no se reprodujeron vulnerabilidades.
`security.py` contiene validación URL, resolución de direcciones, límites de
descarga/archivo, rutas de grafo y redacción/sanitización: son controles localizados,
no certificación de SSRF/XSS. `serve_http` ofrece bearer/API key y restricciones
Host; annotations MCP no sustituyen ACL ni aislamiento por tenant.
Un archivo local malicioso sigue siendo entrada no confiable, incluso offline.

## 4. Prerrequisitos VantaDB

1. Baseline revalidado `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, limpio
   antes de edición. `c561158`/dirty es histórico; revalidar FIND-299 antes de tarea.
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
Promedia salidas distintas del sentinel exacto `No matching nodes found.`:
puede incluir **rechazo por ambigüedad**, no solo matches útiles. Un ratio alto
puede coincidir con baja cobertura o negativas cortas; es longitud estimada,
no todo “contexto útil reducido” ni ahorro de tokens facturados.

[test_benchmark_real_query.py:35–74](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/tests/test_benchmark_real_query.py#L35-L74)
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
| Ratio de reducción del benchmark | No aplicable como score de utilidad Vanta | GF-1: longitud de salidas no-sentinel, incluidas negativas ambiguas; no acredita QA ni cobertura |
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

## 11. Dossier ampliado de flujos y evidencia

### 11.1 Mapa, persistencia y fronteras

Esta continuación conserva las fichas previas. Clon de lectura:
`C:\Users\Eros\graphify`, SHA `16218bfc80d4ca2799734d045039f7ed1a5fa9fb`
según el manifest congelado; consulta pública adicional el **2026-11-07**.
No se regeneró el grafo del clon: investigar su corrección exige fuente primaria.

| Superficie | Mecanismo observado y frontera |
|---|---|
| `detect.py`, `extract.py`, `extractors/` | Descubrimiento y extracción; no se revisaron todos los parsers |
| `ids.py`, `build.py`, `validate.py` | Normalización, nodos/aristas/evidencias, validación de campos; validación de forma no prueba verdad |
| `symbol_resolution.py`, resolvers especializados | Resolución interarchivo localizada; cobertura de dispatch completa no acreditada |
| `cache.py`, `watch.py` | Cache por contenido/versión, reconciliación, exclusión de escritores y actualización de derivados |
| `serve.py`, `affected.py` | Consulta y recorrido; CLI/MCP no son autorización de lectura del filesystem |
| `export.py`, `exporters/html.py`, `wiki.py` | JSON, HTML e hipertextos derivados, no otro primario de memoria |
| `ingest.py`, `llm.py`, `security.py` | Entrada externa/enriquecimiento/control de contenido; son fronteras distintas del AST local |

El modelo incluye nodos identificados por strings, atributos `source_file`,
origen y relaciones con evidencia; `also_seen` no es otra llamada confirmada.
NetworkX sirve al cálculo; JSON `nodes`/`links`, flags directed/multigraph,
comunidades e hiperedges sirven al intercambio. No convertir un hiperedge en
todas las parejas de llamadas posibles. Caches, manifest, etiquetas y HTML
son proyecciones con ciclos de vida diferentes; `built_at_commit` no captura
por sí solo ediciones dirty. No se acreditó ACL multitenant ni memoria de sesión
equivalente a Vanta en estas superficies.

Licencias por componente: [LICENSE](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/LICENSE)
es Apache-2.0; [NOTICE](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/NOTICE)
declara porciones anteriores bajo MIT, retenidas en `LICENSE-MIT`.
No equivale a elección libre de MIT para cualquier archivo nuevo. Gramáticas,
NetworkX, renderizador web y proveedores requieren inventario propio si se
redistribuyen; no se revisaron todas sus licencias transitivas.

### 11.2 Traza de escritura, consulta, update/delete y commit

1. **Admisión:** `_rebuild_lock` en
   [watch.py:258–350](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/watch.py#L258-L350)
   ofrece bloqueo por repo; caller no bloqueante recibe false y puede dejar
   trabajo pendiente. POSIX verifica inode/device después de adquirir flock.
   En plataforma sin locks o filesystem Windows sin soporte existe degradación
   a `yield True`: no anunciar exclusión universal. No iniciar este watcher desde Vanta.
2. **Reconstrucción:** `_rebuild_code` coordina extracción y reconciliación.
   [_reconcile_existing_graph:1087–1138](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/watch.py#L1087-L1138)
   propaga fallo de lectura del baseline en vez de tomarlo por vacío.
   Missing y unreadable son estados distintos; el caller termina con fallo,
   no con un grafo vacío exitoso.
3. **Ownership y bajas:** [1324–1451](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/watch.py#L1324-L1451)
   conserva archivos vivos excluidos sin regla explícita; AST reemplaza su
   capa de fuentes reconstruidas, no toda explicación semántica que no puede
   regenerar. Delete explícito y cambio de filtro no son lo mismo. Una colisión
   AST/semántica con evidencia independiente conserva el ID semántico y remapea
   los endpoints AST nuevos; conflicto no resoluble falla, no sobrescribe.
4. **Publicación:** [3247–3398](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/watch.py#L3247-L3398)
   construye temporal, compara baseline y aplica guard de shrink. Antes de
   avanzar JSON marca HTML stale; después escribe reporte, labels y firmas,
   manifiesto, HTML/callflow opt-in y por último stamp de build.
   Son varias escrituras, **no una transacción global de todos los productos**.
   Fallos de manifest se capturan y callflow puede quedar sin actualizar;
   `True` del rebuild no certifica igualdad de cada derivado.
5. **Durabilidad:** [paths.py:84–205](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/paths.py#L84-L205)
   intenta replace; fallback por permisos/cross-device copia a temporal,
   aparta destino y renombra, con restauración best-effort. Hay ventana entre
   renames, no atomicidad universal. `_atomic_replace` documenta ausencia de
   fsync: rename visible no acredita power-loss durability.
6. **ACK de inputs:** [_current_build_state/_write_build_state:1970–2080](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/watch.py#L1970-L2080)
   incorpora root, conjunto escaneado, exclusiones, modo y epoch, no solo mtime.
   No emite stamp si falta JSON completo o HTML está stale. El check de extremos
   `{`/`}` es deliberadamente barato: no valida toda corrupción interna.
7. **Consulta:** [_query_graph_text:1489–1565](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/serve.py#L1489-L1565)
   acepta seeds externos; referencia ambigua se rehúsa y unresolved se informa.
   Sin seeds obtiene términos, puntúa candidatos y renderiza BFS presupuestado.
   `No matching nodes` no demuestra que el cuerpo de código no contenga la respuesta.
   Consulta no publica cambios; cancelación/timeout del transporte y lectura
   durante cada fallback de escritura no quedaron acreditados end-to-end.

### 11.3 Incidencias públicas leídas y alcance

Búsqueda keyless DDG Graphify issues: cero resultados. Comparación Bing/DDG
del grupo Understand-Anything también vacía; API GitHub pública devolvió cuota
agotada para ese grupo. El fallback HTML permitió leer los cuerpos siguientes,
no solo sus títulos. La revisión independiente completa metadata por API
(consulta 2026-11-07, [recibo](reviews/memory-code-remaining.md#graphify)):
#5051 open, creada 2026-11-06T15:23:45Z; #4273 closed, creada
2026-10-10T03:04:15Z y cerrada 2026-10-10T21:36:10Z. No se deduce un fix/merge.

| Fuente | Reporte efectivamente leído | Discriminación y aplicación |
|---|---|---|
| [#5051](https://github.com/Graphify-Labs/graphify/issues/5051) | En 0.9.102/Windows/Python 3.12, pregunta “credit history transactions” recupera tests/docs pero no `Ledger.entries` cuyo cuerpo contiene términos | Problema de admisión de candidato, no necesariamente ranking final. `serve.py:1489–1565` permite seeds explícitos: alternativa es Vanta hybrid → refs → recorrido, no reescribir BM25. No se revisó todo scorer ni corrección posterior |
| [#4273](https://github.com/Graphify-Labs/graphify/issues/4273) | En 0.9.82 y según autor 0.8.40/macOS, unlock+unlink deja waiter sobre inode antiguo y permite otro holder; aporta repro A/B/C | Fuente fijada ya comprueba inode/device y reintenta (`watch.py:310–350`), por lo que **no atribuir literalmente el mecanismo histórico sin guard al snapshot actual**. No reproducción ni garantía de todas las ramas Windows |

Las cifras SWE-bench y comparación BM25 de #5051 son claims del autor, no
medición propia ni equivalencia QA; sus poblaciones no autorizan reconstruir
un denominador único sin runner. Las referencias a #4982/#4983/#858 no se
cuentan como leídas. No se verificó merge de PR correctivo alguno.
Casos manuales futuros: código cuyo cuerpo y nombre divergen, seeds ambiguos,
dos rebuilds y un waiter, filesystem sin locks. En Vanta son condiciones de
adopción, no bugs heredados por compartir el concepto de watcher.

### 11.4 Lectura complementaria del benchmark y UI

`benchmark.py:32–103` usa cinco preguntas genéricas por defecto; lista vacía
cae también en esos defaults por `questions or _SAMPLE_QUESTIONS`. No hay
dataset etiquetado, split, juez, answerer, embedding o semilla de muestreo.
El denominador de `avg_query_tokens` es **len(per_question)** tras excluir
solo el texto exacto `No matching nodes found.`:
[benchmark.py:36-43,81-95](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/benchmark.py#L36-L95).
[serve.py:1532-1538](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/serve.py#L1532-L1538)
devuelve negativa textual ante un nombre exacto ambiguo; esa salida entra
en tokens y ratio aunque no contenga subgrafo útil. Solo si todas devuelven
sentinel se obtiene el objeto error por falta de entradas. Excepciones de
loader/query no tienen recibo por pregunta en esa función. Se lee JSON,
calcula e imprime; no se localiza allí reset de corpus ni llamada LLM.
No mide latencia, coste de construcción, cache del cliente o exactitud.

Adaptación bajo METHOD/BENCHMARK-NOTES ya fijados: resolver candidatos con el
retrieval Vanta existente y pasar referencias C12 a la consulta es el seam;
conservar output/estado real (incluidos refusals), no-match y errores
corresponde al protocolo común, no a modificar este runner. FMEA: negativa
corta → ratio grande → falsa utilidad; detectar por estado separado de
longitud. No se midió frecuencia ni se recalculó score alguno. No se diseñó
harness, pregunta adicional, configuración ni umbral.

El HTML sí tiene flujo implementado:
[showInfo/focusNode:256–297](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/exporters/html.py#L256-L297)
selecciona nodo → obtiene vecinos → muestra tipo/comunidad/fuente/grado →
permite foco en vecino. Escapa campos y usa listener delegado en vez de
onclick interpolado. Son controles localizados, no auditoría XSS. El
marcador `.graph.html.stale` prueba que vigencia de vista y JSON son distintas.
No se abrió UI ni se acreditó aplicación desktop/SaaS desde esta fuente.

## 12. Fichas operativas y cierre

### 12.1 Terminación de A–C y prioridades

Todas las funciones Rust de este cierre son **propuestas no implementadas**.
El único DTO persistente de importación sigue siendo C12 `CodeEvidenceImport`;
los planes/recibos/vistas son resultados de aplicación. No ampliar C12 por
inferencia ni crear un segundo primario JSON/NetworkX.

| Ficha / escenario de agente | Alternativas y delta | Ownership, concurrencia, cancelación y rollback propuestos |
|---|---|---|
| A: explicar por qué existe una relación | Extender scanner propio, adapter opcional o resolver completo. Adapter preserva evidencia rival que `add_edge` no admite como metadata | Validar originales/endpoints antes de publicar; key por productor/repo/revisión/evidencia; cancelación previa deja anterior, posterior no implica undo. Reimport converge y rollback cambia generación visible, nunca memoria humana |
| B: recordar evidencia entre actualizaciones | Full snapshot conservador frente a delta con tombstones; hash Vanta ya existe, el delta es coverage/origen/versionado | Reconcile condicionado a generación y autoridad; error no es baja. Migración legacy sin dueño queda diagnóstico; job viejo no pisa nuevo. Observar pending/rechazos/lag sin paths en etiquetas |
| C: responder impacto sin fingir exhaustividad | Expandir `code_search`/`code_impact`, no otro retriever; query con seeds explícitos si nombres no bastan | Lectura sobre generación fija, sin commit. Cancelación o presupuesto produce incomplete; paginar solo misma generación. Desactivar adapter mantiene consultas Vanta y no reduce gates |

Orden propuesto: **primero A/B y D** (evitar pérdida/promoción de evidencia);
**después C/F** (contexto correcto y relaciones no colapsadas); **E/G** cuando
exista pipeline y UI aprobados. No requiere rehacer motor ni bloquear consultas
read-only hasta completar un watcher.

### 12.2 Ficha D — Reconciliación con ownership por capa

- **Caso/fuente:** un agente añade explicación manual; refresh AST no debe
  borrarla. Unidad `watch.py:1324–1451`, §11.2: reemplazo por fuente **y origen**.
- **Delta/seam:** `index_repo:152–327` ya reconcilia hash/símbolos; `MemoryInput`
  ya expresa procedencia. Falta acreditar ownership por productor/capa importada.
  Extender coordinador existente, no llevar código al ingestor documental.
- **Contrato:** `plan_reconcile(previous, incoming: &CodeEvidenceImport,
  coverage) -> Result<ReconcilePlan, ImportError>`; inputs de control no son
  otro schema de importación. Error `UnknownCoverage/Conflict/BadEndpoint`
  no habilita borrado.
- **Pasos:** capturar original → separar owner AST/explicación → validar bajas
  explícitas y fuentes fallidas → preview → publicar generación → cleanup propio.
  Full reimport sin bajas es alternativa mínima si export no declara coverage.
- **Dependencias/rollback:** A/B, namespaces y publicación controlada; conservar
  manifest anterior. Ninguna migración deduce ownership de nombre/peso.
- **FMEA:** ID igual reemplaza evidencia humana → detectar origen distinto y
  rechazar/alias explícito; archivo excluido parece borrado → coverage; delete
  deja dangling → verificar endpoints. Cancelación no publica plan incompleto.
- **Aceptación manual:** AST+semántica sobreviven refresh; falla de lectura no
  borra; baja propia no toca otra fuente; métricas por altas/bajas/rechazos.
- **No objetivos:** merge bidireccional, recuperar evidencia nunca exportada o
  asumir que explicación conservada sigue semánticamente fresh.

### 12.3 Ficha E — Publicación de derivados y ACK comprobable

- **Caso/unidad:** agente abre HTML anterior junto a JSON nuevo; fuente
  `watch.py:3247–3398,1970–2080` y `paths.py:84–205`.
- **Delta/seam:** jobs/checkpoints y hashes Vanta existen; distinguir generación
  primaria visible de derivados listos no exige nuevo scheduler.
- **Contrato:** `publish_code_projection(expected_generation, prepared_refs)
  -> Result<ProjectionReceipt, PublishError>`; receipt referencia C12 y separa
  publicación, estado de derivados y cleanup, no promete fsync por proyección.
- **Pasos/alternativas:** preparar → validar → publicar condicionalmente →
  marcar proyecciones pending → actualizar cada una → ACK de inputs.
  Alternativa mínima: no importar HTML, generar vista read-only desde Vanta.
- **Ownership/deps:** solo writer autorizado por repo/productor; D y contrato
  de commit real. Cancelar antes no adelanta ACK; después se reconcilia recibo.
  Rollback vuelve a generación retenida y marca derivados incompatibles.
- **FMEA/aceptación:** fallo entre JSON y UI → detectar generación distinta y
  mostrar stale; retry no duplica; cleanup fallido sigue visible. Medir estados,
  no contenido. No copiar fallback rename como garantía transaccional.
- **No objetivos:** atomicidad filesystem universal, exactly-once o portar HTML.

### 12.4 Fichas F/G — Relaciones paralelas y navegación explicable

- **F, caso/fuente:** `imports` y `calls` entre mismos nodos deben sobrevivir
  recluster. [to_json:415–474](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/export.py#L415-L474)
  preserva `original_links`, multigraph y `_src/_tgt`. Delta: adapter A no
  colapsa por pareja; Vanta ya tiene labels, no necesita otro multigraph.
  Puerto propuesto `map_relations(&CodeEvidenceImport) -> Result<RelationPlan,_>`.
  Validar dirección/tipo → mapear endpoints → asociar evidencias → publicar por B.
  Alternativa: rechazar export ambiguo, no adivinar orden. Depende de A/D;
  cancelación descarta plan, rollback retira solo generación propia.
  FMEA: simplificación de NetworkX pierde relación inversa → comparar lista
  original y endpoints; aceptar manualmente ida/vuelta y dos tipos distintos
  sin duplicados tras retry. No calibrar confidence desde defaults numéricos.
- **G, caso/fuente:** agente inspecciona evidencia y vecinos, no topología como
  certeza. `exporters/html.py:256–297` sustenta selección/inspector; Studio ya
  tiene diagnóstico retrieval, no hay que crear otra aplicación.
  Puerto propuesto `inspect_code_evidence(auth, generation, selection) ->
  Result<CodeView,_>`, sin escritura. Autorizar → resolver C12 → mostrar
  fuente/origen/revisión/límites → navegar con budget existente.
  Links versionados son alternativa mínima al panel; depende de A/C y trabajo
  frontend separado, no de E si la vista consulta datos canónicos.
  FMEA: selección vieja pisa nueva → descartar respuesta anterior; texto hostil
  → render seguro; ruta fuera de root → denegar. Aceptación manual: stale,
  no-match, rechazo ambiguo y error distintos; ratio no es utilidad ni confianza;
  cancelación UI no cancela job compartido.
  Desactivar panel no borra datos. Sin portar vis.js ni prometer a11y validada.
  Seam Studio releído con `git show`:
  [feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228),
  búsqueda explain y get/vecinos/audit existentes (2026-11-07), no bytes dirty
  ni un inspector de refusals ya implementado.

### 12.5 Contraste final, negativos y límites

Vanta baseline fijo `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, relectura de
`code_index.rs:152–327`, `record.rs:184–224`, `graph.rs:113–151`: hay scanners,
hash, reconciliación y grafo; **parcial** para evidencia code rica, **no
acreditado** para publicación por productor/cobertura, **no aplicable** portar
NetworkX/UI. El seam conserva memoria/provenance/budgets/Studio existentes.
Negativo local acotado: hash del archivo se escribe antes de todos sus símbolos;
fallo posterior podría dejar hash nuevo con símbolos incompletos y el retry
saltaría por hash igual. Es una hipótesis derivada del orden de `index_repo`,
no pérdida reproducida ni diagnóstico de atomicidad de todo Vanta.
Aceptación futura: fallo entre file put y symbol put debe impedir ACK completo.

Cobertura adicional leída: rangos enlazados de watch/paths/export/serve,
`benchmark.py` completo, LICENSE/NOTICE y cuerpos #5051/#4273. No se revisaron
todos los resolvers, ingestores remotos, seguridad web, transportes ni recovery.
La revisión completó estado/fecha de incidencias; persisten licencias
transitivas y pruebas de concurrencia/crash como lagunas explícitas.
Recibo [GF-1](reviews/memory-code-remaining.md#graphify): denominador/refusal
propagados a C, benchmark, matriz y UI; pendiente de closure independiente.
No ejecución ni aceptación final.
Si se aprueba pasar a specs, separar A/D/importación, B/E/ciclo de vida,
C/F/consulta y G/Studio; mantener este dossier con TOC como evidencia histórica.
