# Estado actual de VantaDB

## Diagnóstico ejecutivo

VantaDB quedó en una posición mucho mejor definida de la que tenía al inicio del ciclo. El snapshot auditable más reciente del repo que compartiste corresponde a `main` en el commit `43cd3dc` y ya describe un producto enfocado: motor embebido de memoria persistente, recuperación vía WAL, recuperación y rebuild de índices derivados, búsqueda vectorial HNSW por cosine, filtros estructurados por metadata, export/import JSONL, boundary estable en `src/sdk.rs` y surface operativa en Rust, Python y CLI. Además, la documentación de milestone, changelog y Python SDK ya lo ubica con `text_query` texto-only vía BM25 y con el modo híbrido `text_query + vector` todavía diferido hasta que exista RRF/planner. En otras palabras: el proyecto ya no está en fase de infraestructura básica; ya entró en fase de retrieval utilizable, aunque todavía no cerró su contrato de búsqueda híbrida. fileciteturn0file0

La conclusión dura, pero correcta, es esta: **el proyecto ya es serio como core embebido**, pero **todavía no es serio como producto de hybrid search completo**. Esa distinción importa porque define qué puedes afirmar, qué debes seguir construyendo y qué debes dejar de prometer. fileciteturn0file0

## Cómo quedó el producto

Hoy VantaDB debe describirse sin humo: no como “multimodel universal”, no como plataforma enterprise, y no como motor híbrido completo. Debe describirse como un **embedded persistent memory engine** con namespaces first-class, records canónicos, búsquedas vectoriales, filtros estructurados y una ruta lexical BM25 texto-only ya incorporada a la API de memoria. La CLI embebida, el SDK de Rust y el binding de Python ya cubren `put/get/delete/list/search`, rebuild manual, export/import y métricas operativas. El servidor existe, pero no es el boundary principal del producto. fileciteturn0file0

El salto técnico importante de esta fase es que el bloque textual dejó de ser una simple estructura interna y ya se convirtió en un path de búsqueda lexical real. Según el diff más reciente que compartiste, BM25 texto-only ya corre sobre postings persistentes con TF y estadísticas persistidas por término, documento y corpus por namespace; además, el path híbrido sigue rechazado de forma explícita mientras no exista RRF/planner. Esa decisión es correcta. Habilitar híbrido sin contrato de ranking habría sido un error de producto, no una victoria técnica.

## Fortalezas reales

La mayor fortaleza del proyecto no es una feature aislada; es la arquitectura. Tienes una línea de diseño coherente: records canónicos como source of truth, índices derivados reconstruibles desde esos records, repair-on-open, export/import centrado en estado canónico, y un boundary estable en `src/sdk.rs` que evita exponer internals peligrosos al SDK de Python. Eso reduce deuda técnica futura, evita acoplamiento innecesario y hace que recovery, rebuild y paridad entre superficies sean defendibles. fileciteturn0file0

La segunda fortaleza es la validación. No te quedaste en “compila en mi máquina”. En la evidencia visible hay ejecución satisfactoria de `cargo fmt --check`, suites Rust para `memory_api`, `memory_export_import`, `derived_indexes`, `derived_index_recovery`, `derived_index_prefix_scan`, `operational_metrics`, `text_index_recovery`, `memory_brutality`, build del wheel vía `maturin` y `pytest` del SDK Python. Los logs mostrados confirman green en recovery, rebuild, export/import, filtros, restart y smoke de volumen. Eso no te da marketing; te da algo más valioso: credibilidad técnica. fileciteturn0file2

También es una fortaleza que el repositorio ya haya corregido su narrativa. La documentación insiste en limitar los claims a lo que existe hoy: memoria persistente embebida, vector retrieval por cosine, filtros estructurados, índices derivados persistentes, repair-on-open y Python source-install, dejando explícitamente fuera la historia de “hybrid completo”, PyPI listo y posicionamiento enterprise. Esa honestidad no es cosmética; es disciplina de producto. fileciteturn0file0

## Debilidades y riesgos

La debilidad principal ya no es infraestructura. Es **cierre de producto**. Mientras `text_query + query_vector` siga bloqueado, VantaDB todavía tiene dos rutas de retrieval útiles, pero no una sola historia de búsqueda integrada. Tienes vector search. Tienes BM25 texto-only. No tienes todavía una experiencia híbrida cerrada. Si intentas vender “hybrid search” antes de resolver RRF/planner, repites exactamente el error de sobreclaim que la documentación ya había corregido antes. fileciteturn0file0

La segunda debilidad es calidad lexical. Incluso con BM25 ya activo, el sistema sigue con una tokenización austera y sin varias capas que un usuario comparará tarde o temprano: stemming, stopwords, Unicode folding, phrase queries, posiciones y snippets. No digo que debas implementarlo ya; digo que esa deuda existe y que hoy no puedes competir en percepción contra motores full-text maduros si te comparan en esa dimensión. Tu ventaja actual no está allí. fileciteturn0file0

La tercera debilidad es distribución y packaging. El Python SDK sigue en modo source-install y la deuda de PyPI/wheels/signing sigue abierta. Eso no bloquea el desarrollo del motor, pero sí bloquea parte de la adopción externa y de la narrativa comercial. Y la cuarta debilidad es operativa: JSONL export/import está correctamente documentado como flujo simple, no como sistema robusto de backup o framework de migración. Está bien así, pero hay que respetar ese límite. fileciteturn0file0

## Fortalezas y debilidades traducidas a oportunidad

La oportunidad más inteligente para VantaDB no es intentar ganarle a motores de búsqueda textual maduros en su terreno ni presentarse como una gran plataforma “todo en uno”. La oportunidad real, por inferencia del surface actual, es dominar el espacio de **motor embebido, durable, local-first**, con buen boundary para Rust/Python, recuperación fuerte, rebuild explícito, namespaces, filtros estructurados y una historia clara de memoria persistente. Ese posicionamiento es mucho más defendible con lo que ya existe. fileciteturn0file0

Dicho sin rodeos: tu ventaja competitiva potencial no está en vender amplitud; está en vender **claridad y confiabilidad**. Si conviertes BM25 + vector en una búsqueda híbrida pequeña, determinista y bien definida para embedded use cases, ahí sí empiezas a tener una propuesta distinta. Si te dispersas en snippets, packaging o benchmarks antes de cerrar ese contrato, desperdicias el momentum técnico.

## Siguiente fase prioritaria

El siguiente paso correcto **no** es phrase search. **No** es stemming. **No** es PyPI. **No** es Euclidean/SIFT. El siguiente paso correcto es una fase **Hybrid Retrieval v1**: planner mínimo, ejecución dual BM25 + ANN, y fusión por RRF con ordenamiento determinista y respeto estricto por namespace/filtros. Ese es el mayor hueco funcional entre el estado actual y un producto mucho más coherente. fileciteturn0file0

Mi recomendación ejecutiva es simple. Cierra primero el contrato de búsqueda. Define de forma explícita tres modos: text-only, vector-only e híbrido. Mantén BM25 texto-only como está. Mantén ANN como está. En el caso híbrido, genera candidatos por ambas rutas, fusiona con RRF, aplica desempate estable y deja telemetría suficiente para auditar el comportamiento. Solo después de eso tiene sentido ir a calidad lexical avanzada, distribución externa y benchmarks más agresivos. fileciteturn0file0 fileciteturn0file2

## Roadmap inmediato

1. **Cerrar el contrato híbrido.** Habilitar `text_query + query_vector`, introducir planner mínimo y devolver resultados fusionados con RRF bajo un contrato auditable y determinista. fileciteturn0file0

2. **Endurecer operabilidad y pruebas.** Añadir métricas de hybrid path, candidates fused, decisiones del planner y cobertura Rust/Python para namespace isolation, metadata filters, deterministic ordering, import/export, reopen y compatibilidad en read-only. fileciteturn0file0 fileciteturn0file2

3. **Subir la calidad de search.** Solo después, avanzar a phrase queries, posiciones, snippets, Unicode folding, stopwords y stemming. Antes de eso son mejoras visibles, pero no son la brecha principal. fileciteturn0file0

4. **Cerrar distribución y benchmark con verdad.** Cuando el contrato híbrido exista, entonces sí tiene sentido abordar PyPI/wheels/signing y evaluar Euclidean/SIFT como benchmark serio y no como maquillaje. fileciteturn0file0

## Prompt final para Codex

La propia documentación del repo deja RRF/planner como el siguiente corte técnico natural, mientras packaging y benchmarking competitivo siguen explícitamente diferidos. El prompt para continuar debe atacar exactamente esa brecha y nada más. fileciteturn0file0

```text
Quiero que implementes la siguiente fase de VantaDB: Hybrid Retrieval v1 sobre la base ya existente de BM25 texto-only y ANN vectorial.

Contexto operativo:
- El repo ya tiene:
  - canonical memory records con identidad namespace + key
  - WAL/recovery
  - derived indexes para namespace y metadata filters
  - índice textual persistente
  - BM25 text-only para `text_query`
  - vector search existente
  - Rust SDK + Python SDK + CLI embebida
- Hoy `text_query + query_vector` sigue diferido/bloqueado y esa es la brecha principal del producto.
- No quiero claims prematuros ni features cosméticas.
- No normalices ni reviertas archivos sucios no relacionados.
- Mantén el boundary estable en `src/sdk.rs` y cambios aditivos en el Python SDK.

Objetivo:
Habilitar búsqueda híbrida real para memoria persistente usando planner mínimo + RRF entre resultados lexicales BM25 y resultados vectoriales ANN, respetando namespace y metadata filters.

Alcance exacto:
1. `src/sdk.rs`
   - Permitir tres modos:
     - vector-only
     - text-only
     - hybrid text+vector
   - Reemplazar el error explícito actual de `text_query + query_vector` por ejecución híbrida.
   - Mantener comportamiento actual cuando solo haya texto o solo vector.
   - Definir ordenamiento determinista final: score fusionado desc, luego desempate estable por identidad lógica.
   - No romper contratos públicos existentes; solo cambios aditivos o compatibles.

2. Planner mínimo
   - Si hay `text_query` y no hay vector: usar BM25.
   - Si hay vector y no hay `text_query`: usar ANN actual.
   - Si hay ambos: ejecutar ambas rutas y fusionar con RRF.
   - Si `top_k == 0`: devolver vacío.
   - Si filtros excluyen todo: devolver vacío.
   - No introducir heurísticas complejas ni auto-tuning en esta fase; quiero un planner simple, auditable y testeable.

3. Fusión RRF
   - Implementar Reciprocal Rank Fusion con constante configurable a nivel interno.
   - Fusionar por identidad lógica `namespace + key`.
   - Si un candidato aparece en una sola lista, igual participa.
   - Aplicar truncamiento razonable de candidatos antes de fusionar, con budgets internos claros y documentados.
   - No usar una combinación ad-hoc de scores crudos BM25 y cosine; debe ser RRF por ranking.

4. `src/text_index.rs`
   - Exponer helpers mínimos que hagan falta para recuperar postings/stats del path lexical existente.
   - Mantener el principio actual: nada de serializar internals del índice textual en export/import; todo sigue derivado de registros canónicos.
   - No agregar phrase queries, posiciones, snippets, stemming, stopwords ni Unicode folding en esta fase.

5. `src/metrics.rs` y `vantadb-python/src/lib.rs`
   - Agregar métricas operativas para hybrid retrieval, como mínimo:
     - hybrid_query_ms
     - hybrid_candidates_fused
     - planner_hybrid_queries
     - planner_text_only_queries
     - planner_vector_only_queries
   - Exponerlas en `operational_metrics()` sin venderlas como claims de eficiencia pública.

6. Tests
   - Actualizar/agregar cobertura en:
     - `tests/memory_api.rs`
     - `tests/operational_metrics.rs`
     - `tests/text_index_recovery.rs`
     - `vantadb-python/tests/test_sdk.py`
   - Casos obligatorios:
     - text-only sigue funcionando
     - vector-only sigue funcionando
     - text+vector ya no falla y devuelve resultados fusionados
     - namespace isolation
     - metadata filters aplicados correctamente en hybrid
     - deterministic ordering
     - import/export + reopen no rompen hybrid
     - read_only no intenta repair indebido en paths no permitidos
     - error explícito solo para casos realmente no soportados

7. Docs y changelog
   - Actualizar:
     - `docs/architecture/TEXT_INDEX_DESIGN.md`
     - `docs/operations/RELIABILITY_GATE.md`
     - `docs/operations/NEXT_5_TASKS.md`
     - `docs/operations/REPO_CHECKLIST.md`
     - `docs/operations/PYTHON_SDK.md`
     - `CHANGELOG.md`
   - Ajustar narrativa: ahora sí existe hybrid retrieval v1 con RRF/planner simple, pero siguen diferidos:
     - phrase queries
     - snippets
     - stemming
     - stopwords
     - Unicode folding
     - ranking/debug avanzado
     - claims competitivos fuertes
     - PyPI/wheels/signing
     - Euclidean/SIFT como claim de marketing

Criterios de aceptación:
- `search_memory(..., text_query="...", query_vector=[])` sigue usando BM25.
- `search_memory(..., text_query=None, query_vector=[...])` sigue usando ANN actual.
- `search_memory(..., text_query="...", query_vector=[...])` funciona con RRF y no lanza el error anterior.
- Los resultados híbridos respetan namespace/filtros y son deterministas.
- No se reescanean payloads completos innecesariamente si ya existen postings/stats suficientes.
- El export/import sigue serializando solo registros canónicos.
- Todos los cambios pasan tests Rust/Python y no rompen la API actual.

Validación requerida:
- `cargo fmt --check`
- `cargo test text_index --lib`
- `cargo test --test memory_api --test memory_export_import --test derived_indexes --test derived_index_recovery --test derived_index_prefix_scan --test operational_metrics --test text_index_recovery`
- `cargo test --test memory_brutality -- --nocapture`
- `python -m maturin build --manifest-path .\\vantadb-python\\Cargo.toml --out .\\target\\wheels`
- reinstalar el wheel generado resolviendo la ruta real, sin wildcard ambiguo de PowerShell
- `python -m pytest vantadb-python/tests/test_sdk.py -v`

Entrega esperada:
- Resumen corto por archivo modificado
- Decisiones técnicas adoptadas
- Riesgos remanentes
- Resultado de validación ejecutada
- Nota explícita de lo que queda diferido después de esta fase
```