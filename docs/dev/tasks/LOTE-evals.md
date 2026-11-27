---
title: "LOTE evals y benchmarks - 8 tareas, especificacion completa"
kind: task
status: active
description: "Especificacion ejecutable de FIND-664, 667, 668, 669, 670, 674, 675 y 676: contrato, verificacion, comandos, dependencias y skills por tarea."
id: LOTE-evals
---

# LOTE evals/benchmarks — 8 tareas

**Owner:** 4 días hasta el 01-dic. Nada diferido. **Un `FIND` nuevo se resuelve en el acto.**

## Orden recomendado

```
664 (el CI no mide)  →  667 (LoCoMo)  →  669 (soak)  →  668 (latencia híbrida)
   →  670 (p99 cifrado)  →  674 / 675 / 676 (benchmarks nuevos)
```

**Por qué `664` primero:** mientras el CI "haga skip silencioso", **cualquier medición que produzcas aguas abajo es sospechosa**. Arreglar el medidor antes que lo medido es el criterio de toda la sesión.

---

## FIND-664 — El CI de benchmarks no mide: paniquea o salta en verde

**Severidad:** 🟠 Alta · **Esfuerzo:** 1 día · **Estado:** Pendiente

**Cuatro roturas verificadas:**
1. `canonical_p99`, `param_sweep`, `hybrid_queries` y `tokenizer_bench` requieren `data/benchmark/{glove-100,sift-128,beir-scifact}` generados por `scripts/download_ground_truth.py`, pero **CI solo ejecuta `download_benchmark_datasets.sh`** ⇒ paniquean con `expect` ocultos **tras `continue-on-error`**.
2. Los jobs `sift-validation` y `competitive_bench` **no descargan SIFT** ⇒ **retornan temprano pasando en verde sin validar nada**.
3. `tests/certification/glove_dimension_scale.rs` **no tiene entrada `[[test]]`** en `Cargo.toml` ⇒ código muerto nunca compilado, y GloVe-300d no se cachea.
4. `certification.yml:483,490` escribe `metricas/benchmarks/nightly_density_results.txt` pero **sube `nightly_density_results.txt` en raíz**.

**Dónde.** `.github/workflows/nightly.yml:493,498` · `.github/workflows/certification.yml:147,159,416,432,446,483,490` · `Cargo.toml` · `tests/certification/glove_dimension_scale.rs`.

**Contrato.** Cablear **provisión verificada** con gate `TBH-01` **fail-loud**, registrar `glove_dimension_scale`, y corregir la ruta del artefacto.

⚠️ **Éste es el más importante del lote.** Los puntos (1) y (2) son **`FIND-787`/`FIND-794` en otra capa**: un job que se salta y pasa verde. El contrato es que **falle** en vez de saltar, nombrando qué falta. Un `continue-on-error` que tapa un `expect` es un gate que no existe.

**Verificación.** Quitar el `continue-on-error`: el job debe **fallar** nombrando el dataset ausente. Y el artefacto subido debe existir en la ruta que declara.
**Comandos:** `actionlint` exit 0 con y sin shellcheck, **cronometrado** (cuelga si un `run:` gana una sentencia: medido 33 h bloqueado).

---

## FIND-667 — `eval_locomo.py` no soporta el esquema oficial de LoCoMo

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Qué rompe.** El loader espera un diccionario plano **propio** (`question_id`, `haystack_sessions`, `answer_session_ids`) que **solo produce** `generate_synthetic_locomo`. Pasar el `locomo10.json` oficial (`conversation` con `session_N`/`dia_id` + `qa` con `category` 1..5) lanza **`KeyError`**. Además carece de `--strict-embeddings` y **cae a sintético** (`FakeLocomoStore`).

**Dónde.** `evals/eval_locomo.py:218-225,580-610,826-856` · `docs/user/operations/BENCHMARKS.md:2546`.

**Contrato.** (a) **Adaptador canónico** del schema `snap-research/LoCoMo`; (b) soporte del filtro limpio **`locomo-fixed`** (estándar AMB 2026: excluye QA con ground-truth ausente o contradictorio); (c) `query_embed()` **BGE** + `--strict-embeddings`; (d) publicar la tabla real de **§23.4**.

**Verificación.** Con `locomo10.json` oficial: **0 `KeyError`** y las 5 categorías parseadas. Con `--strict-embeddings` y sin modelo: **falla** en vez de caer a sintético.

---

## FIND-668 — `search_hybrid` es 16×–270× más lento que `search_vector`

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Qué rompe.** Sobre **2K–10K registros** por materializar **payloads y snippets completos antes o durante la fusión**, y serializar objetos pesados a través de **N-API** y `serde-wasm-bindgen`. Números: `vantadb-node` **115–1869 ms** vs `1–7 ms` vectorial; `vantadb-ts`/WASM **180–291 ms**.

**Dónde.** `vantadb-node/src/lib.rs` · `vantadb-wasm/src/lib.rs` · `src/sdk/search/` · `docs/user/operations/BENCHMARKS.md:379,1043`.

**Contrato.** Proyectar **solo `(doc_id, score)`** durante BM25+HNSW+RRF; materializar payload/snippet **únicamente para el `top_k` final**; minimizar copias JS↔Rust; **re-certificar §15/§16** con `bench-abi.mjs` y `vantadb-ts/bench/bench.mjs`.

**Verificación.** Medición **antes/después** con el mismo corpus y el mismo `k`. El número es el entregable; sin número, no hay cierre.
⚠️ **No edites `vantadb-node/src/**` si no puedes medir** — es de otro carril en la zona Rust. Si sólo vas a medir, entrega la medición y el contrato, y **dilo**.

---

## FIND-669 — Deriva lineal en `soak_test.py`

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Qué rompe.** Intercalar `supersede` cada 20 turnos y `forget(mode="certified")` cada 100 provoca **crecimiento lineal**: p50 **0.9 → 63.6 ms** (`166 ms` p95) y RSS **36.8 → 126.5 MiB** en 1 000 turnos (**70×** p50, **3.4×** RSS), con **0 fugas de datos**.

**Dónde.** `evals/soak_test.py` · `evals/results/soak_test_1000t.json:33-190` · `src/sdk/api/memory.rs`.

**Contrato.** Perfilar **cuál** de los tres es el cuello: (a) acumulación de **tombstones** HNSW/BM25; (b) **re-escaneo** de cadenas `superseded`; (c) **re-escritura** en `certified` forget. Aplicar poda/compactación **incremental**. Añadir **gate de pendiente máxima** en un soak extendido de **10 000** turnos.

**Verificación.** La latencia p50 del turno 1000 **no** es mayor que la del turno 100 por más de un factor declarado; RSS estable. El gate falla si la pendiente supera el umbral.

---

## FIND-670 — p99 de cifrado `+30.7 %` a `+59.2 %` contra un objetivo de `<15 %`

**Severidad:** 🟠 Alta · **Esfuerzo:** 3-5 días

**Qué rompe.** La navegación HNSW sobre `vfile` cifrado ejecuta **1 desencriptación AEAD por cada vector candidato visitado** (`~ef_search × hops`), violando el presupuesto **incluso con caché caliente**.

**Dónde.** `benches/encrypted_p99.rs` · `src/storage/vfile.rs` · `docs/user/operations/BENCHMARKS.md:1920-2028`.

**Contrato — dos caminos, elegí y medí:**
- (a) **vectores cuantizados SQ8/F16 en memoria** para el paseo del grafo, con desencriptación AEAD **exacta solo en el re-ranking final** (`top_ef`); o
- (b) **caché LRU** de vectores desencriptados gobernada por `memory_limit_bytes`, con **`zeroize` en eviction y drop**.

**Verificación.** Re-medir **§22 sobre GloVe-100 (25K×100d)**: p99 dentro de `<15 %` o el número real declarado. ⚠️ **Si no puedes correr el benchmark, mide el mecanismo** (¿qué sigue descifrando por candidato?) y deja la fila con el contrato correcto — **no inventes un número**.

---

## FIND-674 — Benchmark estándar de MCP Server y Code Mode

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Qué falta.** La puerta principal ICP-01 (AI-IDEs vía MCP + Code Mode + Proxy) **no tiene ninguna medición** en `BENCHMARKS.md`. Metodología: *ProMCP* (ACL Findings 2026, `10.18653/v1/2026.findings-acl.1967`) y *MCP-Universe* (`arXiv:2508.14704`).

**Entregable.** `benchmarks/mcp_profile_bench.py`, determinista, midiendo:
1. **bytes y tokens exactos** de inyección de esquemas `tools/list` en **`full` (88 tools) vs `brain` vs `code`** (`vanta_dispatch`) — cuantificando la reducción de contexto;
2. **latencia JSON-RPC p50/p95/p99** de `context_assemble`, `memory_recall` y `memory_put` por **stdio y HTTP**;
3. **overhead en el wire** de `vanta-proxy` (`WriteBack::track` asíncrono).

**Verificación.** El script corre y produce una tabla reproducible. **Determinista**: misma semilla, mismo resultado.
**Dónde:** `vantadb-mcp/` · `vanta-proxy/` · `docs/user/operations/BENCHMARKS.md`.

---

## FIND-675 — Benchmark de Borrado Certificado GDPR y Churn HNSW

**Severidad:** 🟠 Alta · **Esfuerzo:** 2-3 días

**Contexto.** *Ghost Vectors* (`arXiv:2606.18497`, 2026) demuestra que en motores HNSW estándar los embeddings borrados con *soft-delete* **siguen físicamente recuperables** en índice/mmap (riesgo GDPR Art. 17 / HIPAA). VantaDB **ya implementa** `forget(mode="certified")` con recibo criptográfico, pero **carece de benchmark público**.

**Entregable.** `benchmarks/certified_unlearning_bench.py`:
1. **escaneo forense de bytes residuales** en `vstore`, WAL y SSTables tras `soft`, `hard` y `certified` — **`0 %` de recuperación en `certified`**;
2. **degradación y recuperación** de `Recall@10`, QPS y latencia p99 tras borrar/reinsertar el **10 %, 30 % y 50 %** del corpus HNSW.

**Verificación.** El `0 %` de `certified` es una **aserción**, no un reporte: si sale distinto de 0, es un bug, no un resultado.
**Relación:** ver `FIND-669` (misma familia: tombstones y churn).

---

## FIND-676 — Benchmark E2E del ciclo cognitivo L0→L3

**Severidad:** 🟠 Alta · **Esfuerzo:** 3-5 días

**Qué falta.** Todos los evals actuales (`eval_longmemeval.py`, `memory_harness.py`) ejercitan **solo `db.search` plano (L1 crudo)**, dejando sin certificar el **diferenciador cognitivo**: `context_assemble`, `thread_send`, `memory_supersede`, `vanta_scene` (L2), `vanta_dream` (L3).

**Metodología.** *MemoryAgentBench* (ICLR 2026, `arXiv:2507.05257`), *HaluMem* (EMNLP 2026, `10.18653/2026.emnlp-main.220`), *PrecisionMemBench* (AMB), *COLLABMEM* (`10.18653/2026.emnlp-main.1194`).

**Entregable.** Arnés determinista que evalúe:
1. **resolución de contradicciones** y *selective forgetting* vía `supersede` — **tasa de intrusión de hechos obsoletos en top-K = `0 %`**;
2. **ganancia de compresión/precisión** de escenas L2 y axiomas L3 sobre L1 crudo;
3. **aislamiento y compartición controlada** entre namespaces multi-agente.

**Verificación.** La intrusión `0 %` es aserción. Y la ganancia L2/L3 sobre L1 es **un número**, no una afirmación.
**Dónde:** `vanta-memory/` · `evals/` · `docs/user/operations/BENCHMARKS.md`.

---

## Proceso del lote

1. **Leer** la fila + su task file + `BENCHMARKS.md` §correspondiente antes de tocar.
2. **Medir primero**: `FIND-664` antes que nada, porque mientras el CI mienta, todo lo demás es sospechoso.
3. **Un commit por fila**, con el helper y salida capturada.
4. **Review P2-01** una ronda por lote.
5. **`FIND` nuevo**: se resuelve en el acto si está en `evals/`, `benches/` o los workflows de bench.

## Gates de validación

| Gate | Criterio |
|---|---|
| Python | `ruff check` + `ruff format --check` sobre lo tocado |
| YAML | `actionlint` exit 0 con y sin shellcheck, **cronometrado** |
| Benchmarks | correr el script y pegar la tabla; **sin corrida no hay cierre** |
| Docs | `check-docs` / `check-links` exit 0 |
| Commit | helper, códigos `7`/`8`/`9`, `-ExpectAbsent` declarado |

## Riesgos

- **Disco**: 13 GB y dos carriles Rust. Un benchmark con dataset (GloVe/SIFT) puede pedir GB. **Medí el tamaño antes de descargar** y decí cuánto ocupa.
- **`continue-on-error` en benchmarks**: si lo encontrás, es el bug, no la solución.
- **Números sin corrida**: un benchmark "entregado" sin ejecutar es exactamente lo que `FIND-664` documenta. **No lo hagas.**
