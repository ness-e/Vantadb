# evals/runners — head-to-head (VER-09)

Capa de **runners de competidores** para el harness VER-08: mapea SDKs externos al
MISMO contrato de store que consume `evals/memory_harness.py` (`put/get/search/close`),
de modo que `run_harness` calcula **métricas idénticas** para todos los sistemas
(recall_all@k / recall_any@k session-level, p50/p99, ingest QPS, write-quality,
abstención proxy, token-economy, aislamiento). **0 forks del harness** — el driver
importa `run_harness`; `memory_harness.py`/`calibration.py` no se editan.

> Resultados publicados: `docs/user/operations/BENCHMARKS.md` §19 · narrativa:
> `docs/user/blog/head_to_head_mem0_zep_letta.md`. Este README es el protocolo operativo.

## Comandos (Regla 11)

```bash
# self-test offline (SDKs fake, sin red, sin vantadb)
python evals/runners/head_to_head.py --self-test

# disponibilidad por sistema (no corre nada)
python evals/runners/head_to_head.py --check            # --strict: exit 1 si algún sistema skipea

# corrida real (subset commiteado, smoke)
python evals/runners/head_to_head.py --systems vantadb,mem0 --label h2h-5q

# corrida publicable sobre el split local (cache 277 MB, gitignored) — estratificada
python evals/runners/head_to_head.py --systems vantadb,mem0 \
    --data datasets/longmemeval/longmemeval_s_cleaned.json --sample-stratified 10 --label h2h-strat10

# NOTA: `--limit N` sobre el split completo NO es publicable — el archivo está ordenado
# por bloques de tipo y un slice plano cubre un solo tipo; usá `--sample-stratified N`
# para comparativas (10/tipo = 60 preguntas cubre los 6 tipos).
```

Reports: `evals/runners/report_h2h_<label>_<system>.json` (gitignored — regenerar con
los comandos de arriba; la fuente citable es BENCHMARKS §19 + comando + sha256).

## Protocolo (pineado — verificá contra docs antes de correr)

- **Harness / juez:** `evals/memory_harness.py` (VER-08): juez **determinista**
  (sin LLM) session-level; headline `recall_all@k` (todas las `answer_session_ids`
  ⊆ top-k), secundaria `recall_any@k`; `top_k=5`; hardware registrado en el reporte.
- **Dataset:** LongMemEval-S — sub-muestra commiteada (MIT, sha256 `b8a994ba…`) o
  split completo local (`sha256 d6f21ea9…`). Mismo dataset/top_k/hardware para todos.
- **Sistemas y configuración declarada:**

| Sistema | Pin | Config corrida | LLM | Write-quality | Estado |
|---|---|---|---|---|---|
| vantadb | 0.7.0 (PyO3, memory backend) | `text_query` híbrido BM25+HNSW | — | sí | corre |
| mem0 OSS | `mem0ai==2.2.1` | **raw** (`infer=False`) + `fastembed==0.8.1` (`BAAI/bge-small-en-v1.5`, 384d) + `qdrant-client==1.19.1` embebido in-memory (`path: ":memory:"`); search `top_k`, threshold default 0.1; spaCy ausente (BM25 sin lemma, warning declarado) | **ninguno** (0 llamadas) | sí | corre |
| mem0 OSS | `mem0ai==2.2.1` | native (extracción `gpt-4o-mini`) | OpenAI key | sí | **pending** (owner: keys + budget) |
| Zep | `zep-cloud==3.30.0` | `thread.create` + `add_messages` (chunked ≤4000 chars/msg, ≤30/call, metadata `session_id`) → `graph.search(scope="episodes", limit=k)` | Zep Cloud (créditos) | N/A (sin get exacto) | **skip** (sin `ZEP_API_KEY`) |
| Letta | `letta==0.33.8` | — (agent harness: MemFS/dreaming, no put/search) | provider | N/A | **skip arquitectónico** |

- **Skips:** documentados con motivo en `--check` y en el reporte — nunca silenciosos
  (stop condition del plan: runner no pineable o costo LLM excesivo → skip documentado).
- **ECE/calibración:** se reporta SOLO para vantadb; en externos el driver la anula con
  nota (sus scores no son probabilidades calibradas).

## Límites de no-comparabilidad (declarar siempre)

1. **mem0 raw ≠ mem0 de fábrica**: la extracción LLM está deshabilitada
   (`infer=False`) — se compara la capa de almacenamiento/recuperación, no el
   pipeline de extracción. Los números native quedarán en su propia fila.
2. **Stacks de recuperación distintos y declarados**: vantadb corre text-only
   (BM25+HNSW vía `text_query`) vs mem0-raw semántico+BM25 (fastembed). No se
   "igualan" embeddings — cada sistema corre su mejor config local declarada.
3. **Zep/Letta no corridos** (sin key / contrato no mapea). Presentes solo en la
   tabla con su disposición.
4. **Unidades de ingesta**: Zep recibe chunks (límite duro de API); mem0/vantadb
   reciben la sesión completa. mem0-raw embebe con un modelo de 512 tokens de
   contexto (truncación interna del modelo — declarada); el BM25 de vantadb indexa
   el texto completo.
5. **write-quality** es fidelidad write→read exacta (solo stores con get por key);
   **token-economy** es conteo de palabras (proxy); **abstención** es proxy por umbral.
6. **Slice declarado**: cada tabla publica su n y su origen (subset commiteado o
   `--limit N` sobre el split). No mezclar slices ni comparar contra el headline
   full-500 de §19 sin declararlo.
7. Ningún número de terceros/leaderboards se usa como fuente — todo es corrida local
   con comando reproducible.

## Entorno de verificación (2026-09-30, máquina del owner)

- Win 11 · Intel 12th Gen (12 logical) · 31.78 GB RAM · Python 3.14.7 · disco C:
  ~1.7 GB libres al momento de la corrida (declarado: latencias sensibles a carga).
- Instalado localmente para la verificación (`pip install --user`), **no** es
  dependencia del repo: `mem0ai 2.2.1`, `fastembed 0.8.1` (+ `qdrant-client 1.19.1`).
  El runner falla con skip claro si no están importables.
- Modelo fastembed cacheado en `%TEMP%\fastembed_cache`; `MEM0_DIR` se redirige al
  dir de corrida (best-effort hygiene) y la telemetría de mem0 se apaga
  (`MEM0_TELEMETRY=False`).

## Disposiciones / FINDs (para LEAD)

- **Zep Cloud runner**: ejecutable solo con `ZEP_API_KEY` + créditos (full split
  ≈ 790k créditos vs free tier 10k → subset reducido + n declarado). La alternativa
  self-host (Graphiti + Neo4j/FalkorDB + LLM keys) no está disponible en este entorno.
- **Letta QA-level**: protocolo LLM-in-the-loop diferido (FIND) — su memoria es
  agente-gestionada, no un store.
- **mem0 native**: corrida pendiente de keys/budget (orden de magnitud estimado,
  no medido — declarar como estimación).
