---
title: "Revisión independiente de seis estudios de memoria y código"
kind: research
status: proposed
description: "Recibos adversariales de Cognee, Graphiti, MemPalace, CodeGraph, Graphify y Understand Anything, sin aceptación final"
tags: [research, memory, engineering]
---

# Revisión independiente de seis estudios de memoria y código

## Índice

- [Alcance y método](#alcance-y-método)
- [Cognee](#cognee)
- [Graphiti](#graphiti)
- [MemPalace](#mempalace)
- [CodeGraph](#codegraph)
- [Graphify](#graphify)
- [Understand Anything](#understand-anything)
- [Cierre documental](#cierre-documental)

## Alcance y método

Revisión independiente de autoría, 2026-11-07. Los antecedentes
[memory](memory.md) y [code-memory](code-memory.md) solo contienen recibos de
Mem0 y CodebaseMemory respectivamente: no se duplican ni se atribuye a ellos
revisión de estos seis proyectos.
Rigen [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md),
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md) y
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md).

Vanta se contrasta con `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`;
Studio con `0ad38c542b39a589035b0e17fdedb4c09dca24b9`, rutas
`feature-freez/desktop/...`, no con bytes locales movidos a `desktop/...`.
Un recibo se guarda al terminar cada proyecto. «Sustentado» significa
compatibilidad documental en la muestra, no certificación exhaustiva.
Alta: altera decisión de integración/evaluación; media: laguna material para
derivar una tarea; baja: precisión sin efecto sustancial aislado. Sin score.
Todos los veredictos son propuestas al responsable principal, no aceptación final.
No se ejecutan apps/tests/builds/benchmarks/instalaciones/servicios ni cambios
de runtime, git, índices, contratos o evaluadores.

## Cognee

**Informe íntegro:** [cognee.md](../cognee.md), L1–905.
**Veredicto propuesto: cambios requeridos.**
Clon `C:\Users\Eros\cognee`, HEAD comprobado
`8853378eae5366addb064e758e560a456146ae1a` (en adelante C).

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/tres flujos | sustentado | §1–2: add autorizado, persistencia grafo→vector, query híbrida y delete/update con fronteras separadas contrastados |
| 1. Recomendaciones | sustentado | L103–111: A–H condicionadas, readiness/ownership distintos de otra cola o RRF |
| 2. Fichas Rust y delta | sustentado | L233–573: ocho mecanismos con los once campos, errores, invariantes, alternativas, migración/rollback y aceptación; todos propuestos |
| 3. Incidencias | sustentado | L594–615: cuerpos/comentarios #3235/#2291/#1366 releídos; cierre no se presenta como fix del pin |
| 4. Prerrequisitos Vanta | sustentado | §4 y fichas: writer/checkpoints/worker/RecallConfig/InjectionPolicy reales; publicación atómica no presumida |
| 5. Rechazos | sustentado | §5: no auto-install, plataforma completa, globals como IAM ni graph-first como transacción |
| 6. Método/límites | sustentado | §6/§10: cobertura parcial, distinción propiedad/ledger y no prueba de explotación |
| 7. Benchmarks | parcial | §7: LoCoMo defaults/reset/unscored claros; BEAM no describe defaults ni denominador efectivo (CO-1) |
| 8. Matriz | sustentado | §8: híbrido, dedup, provenance, scheduler, budgets y Studio reconocidos; aislamiento equivalente no acreditado, no negado |
| 9. Negativos propios | sustentado | L765–779: delete-before-put propio es estático; demás gaps son preguntas, no bugs por analogía |
| 10. UI/Studio | parcial | §10: flujo web real y what-if visual correctamente separados; atribución de rutas Studio al SHA incorrecta (CO-2) |

### Hallazgos Cognee

1. **CO-1 — Media — BEAM incompleto, no invalidación de sus resultados.**
   Informe L685–701/856–860: leer REPORT y citar agregador no describe el runner
   local. En C,
   [run_sweep.py L148–177,309–404](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/run_sweep.py#L309-L404)
   usa corpus ya ingerido, split 100K/conversación 0, una repetición, concurrencia
   4 y exige `--config-json-path` sin preset distribuido; 10M exige preguntas
   exportadas. Con `questions_path`, split solo es metadata. En
   [aggregate_cross_run.py L24–85,92–114](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/aggregate_cross_run.py#L24-L114)
   solo scores numéricos entran en medias; `question_count` cuenta entradas,
   no ese denominador. **Consecuencia:** fácil confundir configuración del
   reporte Modal con una ejecución local reproducible o contar no puntuadas
   como evaluadas. **Corrección:** describir variantes, selección y denominador
   por métrica; identificar modelos/config desconocidos y fases/coste excluidos
   al consultar corpus existente. No diseñar ni cambiar scoring/configuración.
2. **CO-2 — Media — identidad de evidencia Studio.** Informe L828–837 fija
   `0ad38c542...` pero cita `desktop/src/...`. `git show` del pin confirma
   `feature-freez/desktop/src/components/memory/MemoryLens.tsx L381–409`
   (`GenlogPanel`, query de generación y anchors). **Mecanismo/consecuencia:**
   ruta de worktree AM atribuida al commit rompe reproducibilidad, aunque el
   comportamiento citado sí tenga antecedente. **Corrección:** citar las tres
   rutas `feature-freez/desktop/...` y sus rangos del commit; cualquier lectura
   actual, separada y rotulada modificada, sin atribuirle ese SHA.

### Fuentes y límites Cognee

- C: `api/v1/add/add.py L313–408`; `tasks/storage/add_data_points.py L307–363`;
  `modules/retrieval/hybrid_retriever.py L120–208`;
  `datasets/datasets.py L457–575`; `update/incremental.py L1122–1210`;
  `improve/improve.py L219–284`. Confirman commits separados, no crash safety.
- Evaluación: driver LoCoMo L64–119; búsqueda de filtros en `answer.py`;
  `aggregate.py L65–190`; BEAM `run_sweep.py L1–275,299–445` y
  `aggregate_cross_run.py L1–158`. No ejecución ni recálculo de claims.
- UI: `searchDataset.ts L1–40` (POST, referencias y timeout);
  `useBusinessQaSurface.ts L46–82` (manual answer sin node IDs);
  Studio `git show` arriba. No navegación ni paridad SaaS.
- Fuentes públicas leídas: API oficial de
  [#3235](https://api.github.com/repos/topoteretes/cognee/issues/3235),
  [#2291](https://api.github.com/repos/topoteretes/cognee/issues/2291),
  [#1366](https://api.github.com/repos/topoteretes/cognee/issues/1366) y sus
  `/comments` completos (4/2/12). Primer intento de expandir el array PowerShell
  de comentarios falló; se releyó con iteración explícita. No diffs causales.
- Seams compartidos: CodeGraph read-only y writer propio; referencia canónica
  para contexto/tenancy/checkpoints. Sin mantenimiento de grafos.
  Restan ACL/handlers completos, conectores, SBOM/datasets, recovery multiproceso
  y todos los modelos. No se exige dinámica como bloqueo documental automático.

## Graphiti

**Informe íntegro:** [graphiti.md](../graphiti.md), L1–834.
**Veredicto propuesto: cambios requeridos.**
Clon `C:\Users\Eros\graphiti`, HEAD comprobado
`747b480ec6b12ecb718f1e2bc588b20ee36ea9c2` (G).

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/tres flujos | sustentado | §0–1: episodio/entidad/hecho/saga separados; add, query y retirada recorridos con commits parciales |
| 1. Recomendaciones | sustentado | §2: tiempo efectivo e identidad de ocurrencia incrementales, no otro híbrido o scheduler |
| 2. Fichas Rust y delta | sustentado | §3.1–3.6: cinco mecanismos, once campos y tabla adicional de commit/cancelación; pseudotipos explícitos, no API existente |
| 3. Incidencias | sustentado | §4: #1865/#1676/#1876 y PR #1699 contrastados; routing MCP del pin ya distinto del reporte histórico |
| 4. Prerrequisitos Vanta | sustentado | §5.1: valid_at ya existe, writer delete→put y error posterior al commit; no bloqueo universal de lógica pura/UI |
| 5. Rechazos | sustentado | §5.2: no Neo4j, saga como checkpoint, comunidades sin caso ni port completo |
| 6. Método/límites | sustentado | §0/4/9: scope no es auth, reporte no reproducción; intervalos de aristas no historial íntegro de nodos |
| 7. Benchmarks | parcial | §6: oracle, contexto string, zip y denominador bien descritos; reset/acumulación y adaptación concreta incompletos (GI-2) |
| 8. Matriz | sustentado | §7.1: híbrido, valid_at, dedup, contexto/checkpoints propios reconocidos; no acreditado no significa ausente |
| 9. Negativos propios | sustentado | §7.2: lectura writer propio compatible; RMW y stale permanecen hipótesis separadas |
| 10. UI/Studio | parcial | §8: no confunde REST/Zep con escritorio; routes/codec atribuidos al pin no existen allí (GI-1) |

### Hallazgos Graphiti

1. **GI-1 — Media — navegación Studio no anclada al snapshot.**
   Informe L713–718,768,834 enlaza `0ad38c542/desktop/src/app/routes.ts`.
   `git cat-file -e` falla para esa ruta y `git ls-tree -r` del SHA completo
   tampoco contiene `feature-freez/desktop/src/app/routes.ts`.
   Sí contiene `feature-freez/desktop/src/App.tsx`, cuyo cuerpo monta
   `WorkspaceShell`; esto no acredita por sí mismo el codec `?s/?ns/?inspect`
   afirmado. **Consecuencia:** un seam de navegación potencialmente nuevo/local
   se presenta como evidencia del pin. **Corrección:** separar lectura AM
   actual de la inmutable y citar ruta/rangos reales; conservar el antecedente
   App/WorkspaceShell sin adjudicarle rutas/codec no presentes.
2. **GI-2 — Media — estado persistente y portabilidad del eval pendientes.**
   Informe L616–620,640–653 describe escrituras y protocolo común, pero no
   explicita que el runner
   [eval_e2e_graph_building.py L32–180](https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_e2e_graph_building.py#L32-L180)
   reutiliza grupos `lme_oracle_experiment_user_N_baseline/candidate`, escribe
   episodios sin UUID suministrado y no contiene limpieza/restauración.
   `eval_cli.py L25–35` puede repetir candidate sin reconstruir baseline.
   **Consecuencia:** repetir contra el mismo destino no implica corpus limpio;
   su juez consume `AddEpisodeResults` Graphiti y no resultados Vanta directamente.
   **Corrección:** documentar ausencia de reset, acumulación posible y exclusiones
   de coste; identificar cómo representar episodios/fuentes/entidades/aristas
   Vanta o declarar no portable ese eval comparativo. No construir adapter,
   cambiar juez ni corregir aquí el defecto de contexto ya bien identificado.

### Fuentes y límites Graphiti

- G: `graphiti.py L1505–1587,2174–2251`; `edge_operations.py L715–786,925–980`;
  `search/search.py L95–225`; MCP `graphiti_mcp_server.py L188–219,795–855`.
  Confirman soporte plural, rebuild fallible, fast path sin tiempo, embedding
  condicional y routing MCP por grupo; no atomicidad ni auth global.
- Leídos completos CLI y runner del eval; `LICENSE` y manifest base;
  `git show` de `l1_writer.rs L481–560` confirma `record_input/valid_at_ms`.
  Studio pin: `App.tsx` completo y listado de rutas; no se leyó el codec AM como
  sustituto de evidencia inmutable.
- Red oficial: cuerpos y estado de
  [#1865](https://api.github.com/repos/getzep/graphiti/issues/1865),
  [#1676](https://api.github.com/repos/getzep/graphiti/issues/1676),
  [#1876](https://api.github.com/repos/getzep/graphiti/issues/1876), comentarios
  de #1876, y cuerpo/metadatos de
  [PR #1699](https://api.github.com/repos/getzep/graphiti/pulls/1699).
  Open/open/closed y merge `08834609936e85a1bbed4158773b1f4641f48d6f`
  son compatibles con el informe; merge no certifica todos los backends.
- No se inspeccionaron diff causal del PR, todos los drivers, IAM, datos
  completos oracle ni precisión LLM. No se confirma disponibilidad comercial
  de nombres de modelo, costes, concurrencia real o borrado físico.

## MemPalace

**Informe íntegro:** [mempalace.md](../mempalace.md), L1–737.
**Veredicto propuesto: cambios requeridos.**
Clon `C:\Users\Eros\mempalace`, HEAD comprobado
`f8e35362189bd0a5abf4c0df1dd5bf90ba059176` (M).

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/tres flujos | sustentado | §0–2: raw/closet/KG/native scan separados; escritura, query directa y delete lógico/físico contrastados |
| 1. Recomendaciones | sustentado | §1: preservar evidencia y capabilities; no convertir claim R@5 en QA ni reemplazar memoria L1 |
| 2. Fichas Rust y delta | parcial | A–F cubren once campos, contratos propuestos, FMEA y rollback; C/F no completan seam de publicación real (MP-1) |
| 3. Incidencias | sustentado | §3: #2752 leído de nuevo; #2958 propuesta abierta, no fix fusionado; parentless explicado por código del pin |
| 4. Prerrequisitos Vanta | parcial | §4 y C/F: capacidades existentes reconocidas, pero tipos de wiki no acreditan commit de fuente/grafo |
| 5. Rechazos | sustentado | §5: no AAAK lossless, metáfora obligatoria ni nuevo exact scan en lugar de HNSW/BM25/RRF |
| 6. Método/límites | sustentado | §6/11: backend nativo no prueba paridad; scope no auth, develop no release, muestra no ausencia global |
| 7. Benchmarks | sustentado | §7: defaults raw/session, user-only, relleno, SKIP y denominadores distintos, reset, cache y adaptación de IDs separados de QA |
| 8. Matriz | sustentado | §8: deltas condicionados, no negación de L0, provenance, ingestors o contexto propios |
| 9. Negativos propios | sustentado | §9: composición y model identity quedan preguntas, no fallos Vanta inventados |
| 10. UI/Studio | sustentado | §10: Landing.vue solo marketing; flujo Studio propuesto remite a referencia canónica, no le atribuye un desktop MemPalace |

### Hallazgos MemPalace

1. **MP-1 — Media — seam C/F detenido en DTOs de wiki.**
   Informe L247–248,261–275,427–429,449–458 presenta `CandidatePage`/
   `CommitReport` como integración para generaciones de fuente e import de hechos.
   En Vanta fijado,
   [merge.rs L16–49,228–288](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L228-L288)
   esos tipos son página `wiki/...` y lista de paths/errores; `commit` recibe
   closures `read_page/write_page` ligados por el worker a `WikiStore`.
   No recibe revisión/generación ni publica por sí mismo un grafo o fuente L0.
   El import upstream
   [knowledge_graph.py L94–193](https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/knowledge_graph.py#L94-L193)
   sí opera entidades/triples y `target.commit()`: son dominios diferentes.
   **Consecuencia:** una tarea derivada puede extender el DTO equivocado o
   presumir una frontera SDK inexistente en el rango citado.
   **Corrección:** delimitar C/F a páginas wiki, si es la intención, o seguir
   caller/closure y puerto de persistencia de fuentes/relaciones correspondiente;
   señalar explícitamente qué publicación/revisión es contrato nuevo no
   acreditado. Conservar los once campos y la propuesta; no implementarla.

### Fuentes y límites MemPalace

- M: `rust_exact.py L110–199` (filtro/snapshot/hidratación y conflicto);
  `palace/writes.py L150–228`; `searcher/query.py L315–375`;
  `knowledge_graph.py L94–193,616–730`; `tools_write.py L577–625,697–727`.
  Confirman parentless intencional, delete por handle o fila, supersede bajo
  `with conn`, no transacción global de KG+colecciones.
- Runner `longmemeval_bench.py L146–243,2907–3005,3120–3165,3220–3258,3265–3370`;
  defaults y efectos compatibles con §7. No lectura exhaustiva de variantes
  hybrid/diary ni resultados, corpus completo o disponibilidad de modelos.
  `Landing.vue L1–29` acredita composición promocional, no inspector.
- API `/issues/2752` falló con HTTP 504; fallback keyless
  [HTML #2752](https://github.com/MemPalace/mempalace/issues/2752) leído completo
  mostró Open, versión 3.10.0 y conteos alegados. API
  [#2958](https://api.github.com/repos/MemPalace/mempalace/issues/2958)
  mostró cuerpo y `pull_request.merged_at=null`; no se usa como código del pin.
- Restan todos los backends remotos/auth, binding PyO3 íntegro, trazado completo
  de cleanup a KG/caches y licencias de modelos/datasets. Ningún resultado de
  pruebas ajenas se declara ejecutado; no se pide dinámica para cerrar MP-1.

## CodeGraph

**Informe íntegro:** [codegraph.md](../codegraph.md), L1–556.
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance.**
No es aceptación final. Clon `C:\Users\Eros\codegraph`, HEAD comprobado
`f72c9d2efb14454ff4f9fd7b8460abd82f19a287` (CG).

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/tres flujos | sustentado | §1/11.1–11.2: bundle pequeño transaccional/grande chunked, consulta/contexto, removeTracked y referencias pendientes contrastados |
| 1. Recomendaciones | sustentado | §1: revisión de evidencia y publicación coherente, no portar producto ni confundir tooling con motor Vanta |
| 2. Fichas Rust y delta | sustentado | A–E/§12: once campos distribuidos entre ficha y tabla complementaria; C12 propuesto, publicación no presumida, error/cancelación/rollback explícitos |
| 3. Incidencias | sustentado | §11.3: tres cuerpos releídos, Open y cero comentarios; #3251/#3238 reportan mismo pin, #3228 es lectura estática |
| 4. Prerrequisitos Vanta | sustentado | §4/12: code_index/index_repo/code_impact/MemoryInput reales; generation CAS y metadata por relación no se declaran implementados |
| 5. Rechazos | sustentado | §5: no catálogo completo, daemon, otro FTS o AST como dispatch completo |
| 6. Método/licencia/límites | sustentado | §6/11: MIT componente raíz, gramáticas/kernel pendientes; helper no implica enforcement global |
| 7. Benchmarks | sustentado | §7/11.4: runner/scorer contrastados; MRR dependiente del orden expected, nombres/homónimos, defaults, errores y denominadores separados de QA |
| 8. Matriz | sustentado | §8/12.5: índice/grafo Vanta existen, equivalencia AST no acreditada; traza no cierre semántico |
| 9. Negativos propios | sustentado | §9/12.5: let _ en remove/delete/add_edge y contador contrastados al pin; no se extrapola a writer global |
| 10. UI/Studio | sustentado | §10/12.4: Svelte real, live reload y DTO drift; no desktop ni Pro auditado; Studio es producto existente |

### Hallazgos CodeGraph

No se encontraron discrepancias materiales adicionales en los caminos
contrastados. Se conserva el defecto estático del scorer ya identificado por
el autor, **no** se presenta como nuevo hallazgo del informe ni se corrige.
El texto inicial «caller no leído» (§1) queda precisado por §11.2, no se
evalúa aisladamente ignorando esa continuación. La muestra no acredita una
auditoría de seguridad, conformance de todos los extractores o rendimiento.

### Fuentes y límites CodeGraph

- CG: `extraction/index.ts L3319–3468,3896–3940` confirma grammar failure,
  reattach, transacción pequeña y baja con refs; `context/index.ts L283–340`
  confirma query→subgrafo→bloques→resumen; `answer-freshness.ts L1–66` completo
  y caller `mcp/tools.ts L3114–3203` confirman guard solo con watcher degraded,
  contribuyentes y ausencia de error. Oversize usa stamp, no digest íntegro.
- CLI de evaluación/scorer completos, `runner.ts L1–123`,
  `scoring.ts L1–82`: temporización antes de scoring, open fuera, throw aborta;
  resultados JSON no fijan SHA del target. No modelos/juez ni reset del corpus
  en el loop leído; se consulta índice existente, no se midió construcción.
- `ui/src/App.svelte L1–80`, `adapter.ts L1–125`: `$effect` inicia live,
  tick recarga stats/entries/trails; `SourceRequest.ondrift` diferencia lectura
  actual y rango indexado. Son código/contrato, no prueba de interacción.
- Vanta `git show` de `code_index.rs L191–240,286–328`, writer ya contrastado
  y C12 en [CONSOLIDATED](../CONSOLIDATED.md#c12--codeevidenceimport-contrato-canónico-único-de-evidencia-code-memory):
  generación y provenance por relación siguen propuestas. C13 tampoco es
  implementación; no se confunde `WriteReceipt`/hash SHA256 con firma.
- API oficial: cuerpos/fechas/estado de
  [#3251](https://api.github.com/repos/colbymchenry/codegraph/issues/3251),
  [#3238](https://api.github.com/repos/colbymchenry/codegraph/issues/3238),
  [#3228](https://api.github.com/repos/colbymchenry/codegraph/issues/3228).
  No PR correctivo atribuido ni repro propia. Sin fallos de acceso en esta lectura.
- Restan kernel/gramáticas/resolver íntegros, implementación de todos los views,
  auth/transportes y validación TOCTOU/crash. Estas limitaciones ya están
  declaradas y no se convierten en bloqueo automático por no ejecutar.

## Graphify

**Informe íntegro:** [graphify.md](../graphify.md), L1–482.
**Veredicto propuesto: cambios requeridos.**
Clon `C:\Users\Eros\graphify`, HEAD comprobado
`16218bfc80d4ca2799734d045039f7ed1a5fa9fb` (GF).

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/tres flujos | sustentado | §1/11.1–11.2: AST/semantic, query, reconcile/bajas y publicaciones separadas trazados; no transacción global |
| 1. Recomendaciones | sustentado | §1/12: adapter opcional con evidencia rival, no NetworkX ni otro motor |
| 2. Fichas Rust y delta | sustentado | A–G/§12: once campos entre ficha y terminación; C12 único propuesto, errores/rollback/ownership explícitos |
| 3. Incidencias | parcial | §11.3: cuerpos correctos, estado/fechas faltantes recuperados en esta revisión; no atribuye bug histórico al guard actual |
| 4. Prerrequisitos Vanta | sustentado | §4/12.5: index_repo/MemoryInput/add_edge reales; metadata completa por arista no admitida por esa API |
| 5. Rechazos | sustentado | §5: no daemon/parsers/URL arbitraria ni estimador como tokens facturados |
| 6. Método/licencia/límites | sustentado | §6/11.1: Apache/porciones MIT separados, AST no truth ni offline universal; no ejecución |
| 7. Benchmarks | parcial | §7/11.4: estimador/defaults/exclusión no-match claros; rechazo por ambigüedad también cuenta como output (GF-1) |
| 8. Matriz | sustentado | §8/12.5: hash/scanners/grafo propios existen; equivalencia AST/coverage no acreditada |
| 9. Negativos propios | sustentado | §9/12.5: hash antes de símbolos y referencias heurísticas contrastados; crash/retry siguen hipótesis |
| 10. UI/Studio | sustentado | §10/11.4: HTML showInfo→vecinos→focus real; no desktop, SaaS ni seguridad acreditados; Studio existente |

### Hallazgos Graphify

1. **GF-1 — Media — «con match» no es el denominador efectivo completo.**
   Informe L204,351–358 llama población a consultas con match. En GF,
   [benchmark.py L36–43,81–95](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/benchmark.py#L36-L95)
   solo excluye texto exactamente igual a `No matching nodes found.`; cualquier
   otro texto cuenta como tokens. Pero
   [serve.py L1532–1538](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/serve.py#L1532-L1538)
   devuelve rechazo textual cuando la pregunta es nombre exacto ambiguo.
   **Mecanismo/consecuencia:** una negativa sin subgrafo útil entra en
   `per_question` y en ratio de reducción; el denominador no garantiza consultas
   respondidas ni matches útiles. No se midió frecuencia ni efecto agregado.
   **Corrección:** documentar ambas ramas y rotular el ratio como longitud de
   salidas no-sentinel, pudiendo incluir refusals; remitir al registro de estados
   de METHOD, sin modificar runner, preguntas, scoring ni umbrales.

### Fuentes y límites Graphify

- GF: `build.py L253–312`; `watch.py L310–350,1087–1138,1324–1364,3247–3335`;
  `serve.py L1489–1565`; `benchmark.py L1–121` completo;
  `exporters/html.py L256–297`. Confirman evidencia plural, guard de inode,
  baseline fail-closed, preservación semántica, HTML stale antes de JSON y
  query con rechazo ambiguo. No se auditó todo fsync/fallback en esta muestra.
- Vanta: `code_index.rs` del pin ya contrastado; C12 propuesto y API simple
  de relaciones no equivalen a import enriquecido implementado.
- APIs públicas leídas, sin fallo:
  [#5051](https://api.github.com/repos/Graphify-Labs/graphify/issues/5051)
  **Open**, creado 2026-11-06T15:23:45Z;
  [#4273](https://api.github.com/repos/Graphify-Labs/graphify/issues/4273)
  **Closed**, creado 2026-10-10T03:04:15Z, cerrado 2026-10-10T21:36:10Z.
  Estos metadatos completan la laguna autoral L330–344/479, no prueban PR/fix.
  Cuerpos compatibles con el informe; guard del pin impide generalizar el
  mecanismo POSIX antiguo sin su discriminación. Claims SWE-bench no medidos.
- Restan resolvers/ingest remoto completos, licencias transitivas, auth
  transports, XSS completo y recovery. No se adoptan cifras publicadas ni se
  confunde persistencia visible con durabilidad o selección segura de tests.

## Understand Anything

**Informe íntegro:** [understand-anything.md](../understand-anything.md), L1–520.
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance.**
No es aceptación final. Clon `C:\Users\Eros\Understand-Anything`, HEAD comprobado
`a6ea4024767df85ce4f99140d4f12ecfc5a81eb6` (UA).

| Eje | Estado | Sección y evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/tres flujos | sustentado | §1/11: builder/persistencia, query→expansión, update/delete y sidecars contrastados; no commit compuesto supuesto |
| 1. Recomendaciones | sustentado | §1/12: enriquecimiento opcional, Unknown cuando falta origen; no promover weight a confianza ni call graph runtime |
| 2. Fichas Rust y delta | sustentado | A–F/§12: once campos y terminación con ownership/rollback; C12 único propuesto, source root separado de output; precisión menor UA-1 |
| 3. Incidencias | sustentado | §11.3: tres cuerpos releídos, #739 histórico discriminado del pin, #758 sin merge probado; metadatos #167 completados abajo |
| 4. Prerrequisitos Vanta | sustentado | §4/12: index_repo/MemoryInput/add_edge/assembly reales; generación/ownership por productor aún propuestos |
| 5. Rechazos | sustentado | §5: no pipeline de agentes, servidor/dashboard/Fuse ni persistencia JSON paralela |
| 6. Método/licencia/límites | sustentado | §6/11: MIT raíz y transitivas pendientes, documentación del flujo no ejecución, scope/root no auth multitenant |
| 7. Benchmarks | sustentado | §7/11.4: entrypoint→runBenchmark, defaults/rango, fases, degraded exit 0, max RSS por worker y duración antes de cleanup contrastados |
| 8. Matriz | sustentado | §8/12.5: code-index/grafo propios existen; paridad AST y publicación completa no acreditadas, no negadas globalmente |
| 9. Negativos propios | sustentado | §9/12.5: ignores y orden hash/symbol del pin contrastados; retry fallido sigue hipótesis, no repro |
| 10. UI/Studio | sustentado | §10/12.5: App fetch/token→HTTP→validate→store, foco/tour y source allowlist reales; no escritorio ni symlink safety supuestos |

### Hallazgos Understand Anything

1. **UA-1 — Baja, no bloqueante — nombre de tipo propio impreciso.**
   Informe L424 escribe `MemoryInput::Derived`. En Vanta fijado,
   [record.rs L184–221](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types/record.rs#L184-L221),
   `MemoryInput` es struct; la clase es el campo
   `confidence_class: Option<ConfidenceClass>`, con `derived_from` y sin
   confidence explícita para Derived. **Consecuencia:** notación que parece
   API invocable pero no lo es; la semántica descrita en §4 sí es correcta.
   **Corrección:** nombrar el campo/clase real o decir «MemoryInput con clase
   Derived»; no añadir otra variante, tipo o contrato. No invalida D ni exige
   compilar para demostrar la diferencia.

### Fuentes y límites Understand Anything

- UA, prefijo `understand-anything-plugin/`: `types.ts L61–146`;
  `analyzer/graph-builder.ts L102–207`; `context-builder.ts L25–110`;
  `staleness.ts L472–508`; `persistence/index.ts L19–144`;
  `skills/understand/SKILL.md L47–73` y
  `merge-batch-graphs.py L233–276`. Rutas de types/analyzer/staleness/
  persistence bajo `packages/core/src`, contexto bajo `src`.
  Confirman weight fijo, cap solo de seeds, pérdida de edges incidentes y
  conservación de layers/tour, escrituras separadas y reasignación PROJECT_ROOT.
  Nada demuestra que todos los callers carezcan de guard adicional.
- Runner UA: `scripts/benchmark-large-repo.mjs L1–64` completo;
  `scripts/lib/large-repo-benchmark.mjs L182–283,1671–1741,1938–2058`
  y constante DEFAULT_CONCURRENCY. No gold, QA o juez en ese recorrido;
  `llm.invoked=false` no certifica todo el pipeline de producto.
- UI: `packages/dashboard/src/App.tsx L144–194`;
  `GraphView.tsx L98–155,191–239`; `vite.config.ts L125–184`.
  RAF/timer cleanup no cancelan extracción; allowlist léxica+stat/read no
  acredita aislamiento físico. No se abrió visor ni empaquetado viewer.
- API oficial de
  [#167](https://api.github.com/repos/Egonex-AI/Understand-Anything/issues/167):
  Closed, creado 2026-05-21T17:14:29Z, cerrado 2026-05-24T13:12:59Z;
  su único [comentario](https://github.com/Egonex-AI/Understand-Anything/issues/167#issuecomment-4513808773)
  atribuye lectura literal de inherit, no aporta diff causal.
  [#758](https://api.github.com/repos/Egonex-AI/Understand-Anything/issues/758)
  Open desde 2026-11-05T09:10:50Z;
  [#739](https://api.github.com/repos/Egonex-AI/Understand-Anything/issues/739)
  Open desde 2026-10-23T02:39:26Z. Cuerpos/estados compatibles con el
  informe; se completa la laguna de metadatos de #167, no se afirma fix.
- Vanta `record.rs L184–225` y `code_index.rs` del pin; no se prueba una API
  de import C12 implementada. Restan fingerprints/schema íntegros, agentes,
  todos los helpers/parsers, seguridad física, aborts y delivery/recovery
  completo del benchmark. No conformance/runtime extrapolados.

## Cierre documental

Se guardaron seis recibos después de leer íntegramente sus respectivos
informes, sin reutilizar los de Mem0/CodebaseMemory como cobertura.
Propuestas: cambios requeridos para Cognee, Graphiti, MemPalace y Graphify;
sin hallazgos bloqueantes en este alcance para CodeGraph y Understand Anything.
Seis hallazgos medios y uno bajo; ningún score de aceptación.
Decisión final y correcciones de autoría corresponden al responsable principal.

Solo se creó/editó este archivo de revisión. Las lecturas fueron de
PowerShell/Get-Content/Select-String, Git read-only (`show`, `rev-parse`,
`ls-tree`, `cat-file`, `status`, `check-ignore`), grafo read-only y fuentes
públicas, con los fallos de acceso indicados en cada recibo. Los `LICENSE`
raíz de M/CG/UA y `LICENSE`/`NOTICE` GF se contrastaron; no es SBOM transitiva.
No se tocaron informes fuente, contratos, backlog, harnesses o configuración;
no apps/tests/builds/benchmarks/installs/servicios, validación global ni
staging/commit/push, conforme al encargo específico.

El worktree contiene modificaciones ajenas concurrentes en informes y otros
documentos; `git status` no distingue autoría. No se revierte ni se incorpora
ninguna. Este recibo está bajo la exclusión `docs/dev/research/` de
`.gitignore:268`: que no aparezca en status no implica que no esté guardado.
Su eventual versionado requiere decisión del responsable, no git writes aquí.
