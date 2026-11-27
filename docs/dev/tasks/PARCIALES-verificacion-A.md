---
title: "PARCIALES-A: verificación de nueve filas parciales del Backlog contra HEAD"
kind: task
status: completed
description: "Siete de las nueve filas parciales siguen vivas pero con líneas desplazadas, una cita un archivo que nunca contuvo lo que dice, y otra ya la resolvió otro carril 68 días después de escribirse: tabla con veredicto, evidencia en HEAD y commit que fecha cada error."
---

# PARCIALES-A: verificación de nueve filas parciales del Backlog contra HEAD

Medición pura: sin compilar, sin tocar `src/**`. La evidencia es lectura de `HEAD`
(`12e6d7c`, 2026-11-27) más `git log -S` / `git show` / `git blame`.

Las nueve filas marcadas `🟡 Parcial`: `FIND-120`, `FIND-121`, `FIND-122`,
`FIND-158`, `FIND-159`, `FIND-290`, `EGO-13`, `EGO-15`, `DEAD-01`.

La marca `re-verificado <fecha>` del Backlog no significa nada. En esta tanda
**una** de esas marcas citó una línea que había dejado de existir **dos días
antes** de la re-verificación (`EGO-15`), y otra describía como pendiente un
cache que otro carril ya había entregado.

## Tabla de veredictos

| ID | Afirmación de la fila | Veredicto | Evidencia en `HEAD` | Commit que fecha el error | Qué queda |
|---|---|---|---|---|---|
| `FIND-120` | `src/wiki/sources.rs:82` "hoy solo `.md`"; `code.rs` 8 tools `code_*`, BFS-1, `code_files` NO soportado; hooks 4 clientes + tests; `setup-embeddings.ps1` es el wizard a extender; falta scan gitignore-aware + chunking por símbolo + escena repo-map | **PARCIAL-REAL** | **Viva:** `vantadb-mcp/src/code.rs:39` define 8 tools (`:42,61,79,97,115,135,153,165`); `:13-16` documenta BFS profundidad-1; `code_files` es stub "not supported" (`:326-327`). Hooks: `skills/vantadb-mcp/assets/hooks/` tiene 4 clientes (`claude`, `codex`, `cursor`, `opencode`) + `tests/test-hooks.ps1`. No hay scan gitignore-aware en `src/` (0 hits `gitignore`/`WalkBuilder`). El chunker es **por caracteres**, no por símbolo: `src/wiki/chunker.rs:19` es `chunk_text(text, target_chars, overlap_chars)`. El wizard no ofrece "indexar este repo" (`scripts/setup-embeddings.ps1`, 707 líneas, descarga/verifica modelos). **Cambiado:** `scan_local_sources` sigue filtrando `ext == "md"` (`sources.rs:39`), pero ya existe un registro multi-formato con trait pluggable: `default_ingestors()` = Txt/Json/Csv/Md (`ingestors.rs:599-606`) y `scan_ingestable_sources` (`:622`); `sources.rs:82` ya no contiene el filtro | `42fb7c86a` (2026-10-05, MEMG-08) añadió el registro de ingestores — **17 días después** de la fila (2026-09-18); el `.md`-only era cierto al escribirse. El path `setup-embeddings.ps1` vive hoy en `scripts/` | (a) scan gitignore-aware (respeto de `.gitignore`, denylist de `dist`/`node_modules`/locks); (b) chunking por símbolo (tree-sitter) — hoy `chunk_text` por chars; (c) escena `repo-map`; (d) hook de re-indexación incremental; (e) oferta "indexar este repo" en el wizard. Solapa con `FIND-299` (MGR-22 v1) |
| `FIND-121` | `src/storage/engine/cache.rs:21-34` = `CacheLayer` (volatile LRU + text_stats + cardinality + warmer, `pub(crate)` sellado) + `volatile_cache_entries`; "verificar en DISCOVERY que no exista ya cache texto→vector ni query→hits"; propuesta LRU+TTL | **PARCIAL-REAL** | **Cita exacta:** `src/storage/engine/cache.rs:21` es el doc "Sealed `pub(crate)`", `:22-33` la struct con `volatile`, `text_stats`, `text_ns`, `cardinality_stats`, `warmer`. `volatile_cache_entries` vive en `src/metrics/core/mod.rs:679`. **Pero la mitad "texto→vector" ya existe:** `EmbedCache` (`src/llm.rs:230-262`), acotado por `WARM_CACHE_CAP = 256` (`:216`), FIFO, contenedor por modelo (`WarmLocal`, `:267-284`) — y **rechaza el TTL por diseño** con razón escrita (`:234-237`: `embed` es determinista en `(model, family prefix, text)`, un TTL sólo forzaría recómputo). Query→hits **no** existe (0 hits de `result_cache`/`search_cache`) | `82077466c` (2026-11-25, "keep the embedding session warm and cache by (model, text) (FIND-706)") — **68 días después** de la fila (2026-09-18) | El slice 1 (`embed-cache`) lo entregó `FIND-706`, y con la decisión **contraria** a la prescripción: sin TTL y sin LRU. Quedan los slices 2–3: cache de recall (`query+top_k+filters → hits`, que sí necesita invalidación por namespace) y proxy-context. La fila debe reescribirse: pedir "TTL" contradice el código que la resuelve |
| `FIND-122` | "paths híbridos BM25+HNSW+RRF (`src/index/search/layer.rs`, `src/api/scores.rs`)"; propuesta "reglas + `search_profile` primero" | **CAMBIADA** | **La cita principal nunca fue cierta:** `src/index/search/layer.rs:1` es "HNSW layer traversal — the core greedy beam-search loop. Hot path."; `git log -S bm25 -- src/index/search/layer.rs` devuelve **vacío** (jamás contuvo fusión). La fusión híbrida vive en `src/planner.rs`, `src/physical_plan/scan.rs` y `src/search_profile.rs` (constantes `RRF_K`, `CANDIDATE_MULTIPLIER`, `MIN/MAX_CANDIDATE_BUDGET`, `:20-30`). `src/api/scores.rs` sí es correcto (`rrf_contribution`, `:17`). **Y el "search_profile primero" ya está:** `SearchProfileMode { Keyword, Vector, Hybrid }` (`search_profile.rs:35-43`) + `SearchProfileConfig` (`:50`) con `dense/lexical/sparse_weight` (`:73,80,87`). No hay router **automático** por tipo de query ni selección provider/runner por idioma/fuente | Cita falsa desde el origen (2026-09-18): el archivo no tuvo nunca el contenido citado | Lo que queda no es el storage híbrido (existe) sino el **routing automático**: dada una query, elegir keyword/vector/grafo/temporal y provider/runner sin que el caller pase `mode`. `search_profile` hoy es una perilla explícita por request, no un router. Sigue gateado por `EXE-02` (harness LoCoMo), que ya no está en el Backlog: **verificar su estado antes de implementar** |
| `FIND-158` | (a) `GET /records/{ns}/{key}/versions` sin paginar; (b) `export/import` verbales, "nota YAML vs REST plurales"; (c) desktop TS con tipos de cursor numéricos; (d) fan-out multi-namespace aplica cursor por-namespace | **PARCIAL-REAL** | **(a) VIVA:** `src/server/handlers.rs:286-289` — `RecordsVersionsParams` sólo tiene `version: Option<u64>`; ni `limit`, ni `cursor`, ni `has_more`. **(b) CAMBIADA:** los paths siguen siendo verbales (`router.rs:277-278`, `openapi.yaml:697,732`), pero la "nota YAML" que la fila describe **no existe** (0 hits de `naming`/`REST convention` en `openapi.yaml`/`HTTP_API.md`). **(c) VIVA y demostrable sin `desktop/`:** `vantadb-node/index.d.ts:148` tipa `next_cursor?: number` en la **salida**, mientras `:120` acepta `number \| string` en la entrada y el backend WASM emite strings decimales (documentado en `vantadb-ts/src/types.ts:159-161`) — el tipo numérico está en la salida, no en el desktop. **(d) VIVA con mitigación:** el fan-out usa un cursor global (`list_records.rs:107`) y pide cada namespace con `cursor: None` (`:116`), pero los namespaces capados ya se **reportan** en `truncated_namespaces` (`:126`, `handlers.rs:449-453`) — el quirk dejó de ser silencioso | — | (a) paginar `versions` con el esquema único (`limit` + `cursor` opaco de `src/server/pagination.rs:20-41`); (b) decidir: renombrar a recurso plural o documentar la excepción; (c) corregir la **salida** `next_cursor` a `string` en `vantadb-node/index.d.ts:148` y en el docstring `lib.rs:499`; (d) decidir si el cursor de fan-out pasa a ser compuesto o queda con `truncated_namespaces` como contrato |
| `FIND-159` | (a) Schemars codegen single-source **DEFER**; (b) strictness/tipado de las **38 tools extendidas**; (c) envelopes `internal_error` sin `data.code` en fallos envueltos | **PARCIAL-REAL** | **(a) VIVA (como DEFER):** `schemars` no está en `vantadb-mcp/Cargo.toml` ni en el workspace, y 0 hits en `vantadb-mcp/src` — sigue diferido, y la fila lo dice. **(b) CAMBIADA:** "38" es el tamaño del **perfil `dev`** (`docs/api/MCP.md:276`), no el de las tools extendidas: son **41** (35 listadas + 6 absorbidas `code_*`), sobre 96 definidas / 90 listadas (`MCP.md:971`). **(c) VIVA exacta:** `internal_error` (`error.rs:82-84`) delega en `plain` (`:21-27`) que fija `data: Value::Null`, y `to_json` **omite** `data` cuando es null (`:124-125`). Contraste: `not_found` (`:60-67`) y `resource_limit` (`:72-79`) sí llevan `code` + `retriable`; `from_domain` (`:108-114`) siempre emite `data.code` | — | (b) fijar el universo real (41 extendidas / 96 definidas) y tipar por familia; (c) dar a `internal_error` (y a `invalid_request`, `:87-89`, que la fila **no menciona**) el mismo `data.code` que el resto, para que un fallo envuelto siga siendo ramificable por un cliente LLM; (a) sigue diferido por decisión |
| `FIND-290` | (a) "sin consumidor in-repo" **superado**: `vantadb-node` importa `perform_auto_recall_scored` (`lib.rs:45`) y lo invoca (`:1164`); (b) lecciones sin promotion gated a L1 (declarado fuera de scope en `reflection/mod.rs:245`); (c) ningún scheduler corre `reflect_session` | **PARCIAL-REAL** | **Las tres citas son exactas en `HEAD`:** `:45` importa (`perform_auto_recall_governed, perform_auto_recall_scored, …`) y `:1164` invoca. `reflection/mod.rs:245` dice literal "(the lesson is session-local until an explicit promotion - FIND-290)" y `:6` declara "promotion stays explicit and out of scope". `reflect_session` (`:351`) **sólo** tiene llamadores en `vanta-memory/tests/reflection.rs` — ningún caller productivo. Matiz: la invocación está bajo `if owned.trust_aware` (`lib.rs:1163`) | — | (b) promotion de lecciones con quality gate + provenance (patrón `promote_dream_run` de VER-07); (c) wiring del pase al ciclo de mantenimiento (WIRE-15). El residual (a) de la fila —"activar composite por config de host"— está **más avanzado de lo que dice**: `trust_aware` **es** esa perilla de host (`memoryRecall({trust_aware: true})`), así que (a) ya cerró |
| `EGO-13` | Python ya zero-copy (PyBuffer, PERF-15); Node sin `Float32Array::as_slice` (diferido en `EGO-03.md:113`); acción: napi `Float32Array::as_slice()` + PyO3 `PyReadonlyArray1<f32>` en Python | **PARCIAL-REAL** | **Python entregado:** `vantadb-python/src/lib.rs:988` (`PyBuffer::<f32>::get`) + `:1008` (`buf.as_slice(py)`), `convert.rs:198,213`, y la vista `types.rs:19-38`. **Node pendiente:** `get_f32_vec` (`vantadb-node/src/lib.rs:1929-1960`) recorre `Value::Array` elemento a elemento con `as_f64()` + validación de finitud; 0 ocurrencias de `Float32Array` en todo `vantadb-node/src`. **Path inexistente:** `vantadb-py/` no existe — el crate es `vantadb-python/` | `cb22b4636` (2026-10-21, "purga 70 task files completados") borró `docs/dev/tasks/EGO-03.md` — **3 días después** de la fila (2026-10-18); el veredicto citado existía y era correcto (`EGO-03.md:113`: "diferir épico EGO-13 a slice propio") | Sólo queda la mitad **napi**: rediseñar firmas a `Either<Value, Float32Array>` en `put`/`put_batch`/`search`/`graphrag`, más el bench de binding before/after (Regla 9) que `EGO-03.md:109` ya identificó como slice propio. La parte PyO3 de la acción **está de más**: Python usa el buffer protocol (`PyBuffer`), y `vantadb-python/Cargo.toml:15` no habilita la feature `numpy` de pyo3, así que `PyReadonlyArray1` no es la vía instalada. `_ = 39` asserts en `tests/test-hooks.ps1`, no 50 como decía `FIND-120` |
| `EGO-15` | `DEFAULT_MAX_DEGREE_PER_NODE = 128` en `pipeline.rs:71` / `expand.rs:13`; "ninguna superficie pública expone `max_degree_per_node`/`min_edge_weight`"; `max_total_nodes` no existe; el seed sale de `embedded.search` (`seed.rs:25`) ⇒ 0 nodos / 0 aristas | **PARCIAL-REAL** | **Defaults exactos:** `expand.rs:13` (`pub const DEFAULT_MAX_DEGREE_PER_NODE: usize = 128;`) y `pipeline.rs:71` (`max_degree_per_node: DEFAULT_MAX_DEGREE_PER_NODE`). `max_total_nodes` **no existe** (0 hits repo-wide). **Superficie de binding vacía:** `graphrag_search` (`vantadb-node/src/lib.rs:1013-1019`) acepta sólo `namespace`, `query`, `query_vector`, `abort`. **El seed:** `find_seeds` llama `embedded.search(request)` en `seed.rs:31` (no `:25`). **Pero "ninguna superficie pública" es falso:** `GraphRagPipeline::with_max_degree_per_node` / `with_min_edge_weight` son API pública del SDK (`pipeline.rs:96,101`; registradas en `tests/api/public-api.txt:3151,3153`) | `8f3b7039d` (2026-11-20, FIND-607) insertó el bloque `GR-04 / FIND-608` (`seed.rs:15-18`) y empujó `embedded.search` de `:25` a `:31` — **2 días antes** de la marca "re-verificado 2026-11-22", que igual siguió citando `:25`. El cap 128 llegó en `647f4f27c` (2026-11-03) | (a) exponer `max_degree_per_node`/`min_edge_weight` en la superficie `graphragSearch` de **cada binding** (hoy sólo el SDK Rust los tiene) y documentarlos; (b) decidir si `find_seeds` siembra también desde los nodos del grafo o se documenta que exige registros de memoria (causa medida en `seed.rs:31`); (c) `max_total_nodes` |
| `DEAD-01` | Barrido acotado a `cli/cli_handlers/bin` (DX-13 Slice 4); falta el reporte repo-wide periódico; flags sin consumidores como `FIND-210`/`FIND-282` | **VIVA** | No existe reporte repo-wide periódico de dead-code: ningún plan, script de `dev-tools/`/`scripts/` ni fila del Backlog lo instancia. Las dos filas citadas siguen abiertas: `FIND-210` (`Backlog.md:289`, "`OriginCollisionTracker` sin callers", `🆕 Pendiente`) y `FIND-282` (`:231`, "`governance.source` sin consumidor", `🆕 Pendiente`). La deuda mecánica es medible: **81** `#[allow(dead_code)]` en **33** archivos de `src/`, y `dead_code` no está en `deny` (`Cargo.toml:761-762` sólo declara `missing_docs = "allow"`) | — (nunca se cerró nada: el defecto es la ausencia del barrido) | El barrido repo-wide con dueño por ítem. **Pero la premisa del paréntesis no se verifica:** `DX-13` en `docs/dev/avance/activo/bindings.md:1249` es "Regeneración de completions de CLI", y "DX-13 Slice 4" como barrido de dead-code no aparece en ningún documento salvo en esta misma fila — la fila se cita a sí misma como evidencia |

## Lo que sorprendió

1. **El patrón se invirtió, pero no del todo.** En las hijas de `FIND-607` el
   error era citar código *borrado*. Aquí **7 de 9** describen defectos que
   todavía existen — y 6 de esas 7 citan **líneas desplazadas** (no código
   borrado). El síntoma cambia, la causa es la misma: nadie releyó `HEAD`.
   La única cita que apunta a un archivo inexistente es `EGO-03.md:113`,
   purgado el 2026-10-21.

2. **`FIND-121` pide verificar que no exista un cache que otro carril ya
   entregó — y lo entregó al revés de lo prescrito.** La fila propone
   "LRU+TTL". `EmbedCache` (`src/llm.rs:230`) llegó **68 días después** con
   FIFO, sin LRU y **sin TTL**, y dejó el motivo escrito: `embed` es
   determinista, un TTL sólo fuerza recómputo. La prescripción y la
   implementación son incompatibles por diseño, y la fila no lo sabe.

3. **`FIND-122` no tiene drift: tiene una cita que nació falsa.**
   `git log -S bm25 -- src/index/search/layer.rs` devuelve **vacío**. Ese
   archivo jamás contuvo fusión híbrida; es el beam-search de HNSW desde el
   primer día. No es que el código se moviera — es que la fila citó un
   archivo que nunca tuvo lo que decía. Y encima `SearchProfileMode`
   (`Keyword`/`Vector`/`Hybrid`) ya existe, así que la mitad de la
   "propuesta" está shipped y nadie lo anotó.

4. **`FIND-158(c)` se demuestra sin `desktop/`.** El defecto que la fila ubica
   en el TS del desktop vive también —y se puede probar aquí— en el binding
   publicado: `vantadb-node/index.d.ts:148` tipa `next_cursor?: number` como
   **salida** mientras `:120` acepta `number | string` como **entrada** y el
   backend WASM emite strings decimales. El tipo miente en el artefacto que
   los usuarios descargan, no sólo en un cliente interno.

5. **`EGO-15` exagera en la dirección que más cuesta.** "Ninguna superficie
   pública expone `max_degree_per_node`" es falso: `with_max_degree_per_node`
   es API pública del SDK desde `pipeline.rs:96`, registrada en
   `public-api.txt:3151`. Lo que falta es la exposición en **cada binding**.
   Leído literal, un implementador puede concluir que hay que crear el
   setter; existe, y duplicarlo rompería el contrato público.

6. **La marca "re-verificado 2026-11-22" de `EGO-15` citó una línea que había
   muerto dos días antes.** `8f3b7039d` (2026-11-20) insertó el bloque
   `GR-04 / FIND-608` y empujó `embedded.search` de `seed.rs:25` a `:31`. La
   re-verificación del 22 copió `:25`. Es el mismo fallo que las hijas de
   `FIND-607` en miniatura, dentro de una fila que se declara verificada.

7. **`FIND-290` es la única fila honesta de la tanda** — y aun así subestima
   su propio avance: dice que "activar composite por config de host" sigue
   pendiente, cuando `trust_aware` **es** esa perilla y el consumidor ya la
   usa (`lib.rs:1163-1164`). Se equivoca hacia el lado conservador.

8. **`FIND-159` omite un gemelo del defecto que denuncia.** `internal_error`
   sin `data.code` es real; `invalid_request` (`error.rs:87-89`) tiene
   exactamente el mismo agujero y no aparece en la fila. Arreglar sólo el
   primero deja el segundo intacto.

9. **El incidente del 2026-11-26 contamina toda la arqueología.**
   `72643d9df` borró 2365 paths y `19f759980` los restauró. Ahora *cualquier*
   `git log -S` devuelve esos dos commits como si hubieran introducido el
   símbolo. Filtrar por fecha es obligatorio: sin eso, todo aparece fechado
   el 2026-11-26. Este documento fecha los errores con el commit **real**
   (2026-10-05, 2026-11-03, 2026-11-20, 2026-11-25), no con la restauración.

10. **`DEAD-01` se cita a sí misma como evidencia.** "Barrido acotado a
    cli/cli_handlers/bin (DX-13 Slice 4)" no aparece en ningún documento del
    repo salvo en la propia fila; `DX-13` en el registro de avance es la
    regeneración de completions de CLI. El defecto (falta de barrido
    repo-wide) es real, pero el "ya se hizo un barrido acotado" no tiene
    traza verificable.

## Resumen contable

**1 VIVA · 0 REFUTADAS · 1 CAMBIADA · 7 PARCIAL-REAL** (9 filas).

- **VIVA:** `DEAD-01`.
- **CAMBIADA:** `FIND-122` (cita falsa desde el origen + `SearchProfileMode` ya existe).
- **PARCIAL-REAL:** `FIND-120`, `FIND-121`, `FIND-158`, `FIND-159`, `FIND-290`, `EGO-13`, `EGO-15`.

Ninguna fila resultó completamente refutada. El daño de esta tanda no es
"describe código que no existe" sino **"describe el código correcto con la
línea equivocada"** — más barato de arreglar (re-citar, no re-escribir) y más
peligroso de ignorar, porque una línea desplazada se lee como código vivo.

Filas que además necesitan **reescritura de alcance**, no sólo de cita:
`FIND-121` (TTL contradice `EmbedCache`), `FIND-122` (el router automático es
lo único que falta; el storage híbrido ya está), `EGO-15` (el setter existe en
SDK; falta en bindings) y `FIND-290` (el residual (a) ya cerró).
