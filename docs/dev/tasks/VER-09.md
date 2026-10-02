---
title: "VER-09 — Head-to-head Mem0/Zep/Letta con protocolo publicado (absorbe EXE-02)"
kind: task
---

# VER-09: Head-to-head Mem0/Zep/Letta con protocolo publicado (absorbe EXE-02)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 47, Fase F6 — wave F6, co-batch con EXE-01; LEÍDO ENTERO: contrato verbatim, riesgos, stop conditions)
- **Fuente:** plan Task 47 · `docs/dev/Backlog.md:913` (EXE-02 absorbido; fila madre P50 conservada `:947`) · `docs/dev/research/validacion/01-competidores-memoria-agentes-ia.md` · `docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md:447,:531,:553`
- **Esfuerzo:** 🔴 1-2sem · **Prioridad:** 🟠 · **Tipo:** Mixto (Python evals + docs; 0 código de producto)
- **Turns estimados:** 30-60
- **Creado:** 2026-09-30 · **last-synced:** 2026-09-30
- **Branch:** develop · **Commit:** lo hace el LEAD (wave F6: sub-agentes NO commitean)
- **Estado:** ⏳ IN PROGRESS — **trabajo ✅ completo (6/6 steps) + fixes review F1–F6 aplicados y re-verificados**; cierre = LEAD (commit local). Este sub-agente no commitea ni se auto-revisa (mandato wave F6).
- **Incógnitas (uphill):** 0 (resueltas en DISCOVERY/ejecución: mem0-raw ejecutable; Zep sin key; Letta no mapea)
- **Pendientes (downhill):** 0 steps ejecutables por este agente

## Contexto verificado del bloque (copiar, no re-derivar)

- **Harness VER-08 ✅** (`0405eeda`): `evals/memory_harness.py` + `calibration.py` + `evals/data/` (recall_all@5 0.7617; ECE) — **se reusa íntegro, sin fork**.
- **Gap:** **0 runners de competidores** (`memory_harness.py:138-141` corre solo el binding vantadb; `--backend` :501; `benchmarks/competitive_bench.py:1-5` = LanceDB/Chroma/Qdrant/Milvus, NO memory systems) + harness **fuera de CI** (`lurkr-informational.yml:58`). El slice = **capa de runners (`evals/runners/`) con las MISMAS métricas** + protocolo publicado + post (`docs/user/blog/`).
- **EXE-02 absorbido** (Backlog:913/947). **Hardware fijado en el contrato.**
- **Skip conditions:** runner no pineable o costo LLM excesivo → skip documentado (no bloquear).
- Dep: VER-08 ✅. **El P2-01 incluirá diseño revisado por `vanta-audit` (protocolo)** — el protocolo queda explícito en este task file (§Protocolo) y en `evals/runners/README.md`.
- **NO tocar:** `src/**` de producción (evals-only aditivo); regiones co-batch EXE-01 (workflows demos — solo LEER). Edits quirúrgicos.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | F6 anuncio (nota: gate de anuncio NOTION-SYNC:57 exige VER-09 publicado); EXE-02 absorbido (ya no corre por separado); docs públicas (`BENCHMARKS.md` §19/§Planificado, `COMPARISON.md`) |
| Callees | `evals/memory_harness.py` (import: `run_harness`, `_load_questions`, `sha256_file`, `_total_ram_gb` — NO se edita) · `evals/data/longmemeval_s_subset.json` (sha256 pinneado) · SDKs externos pineados: `mem0ai==2.2.1` + `fastembed==0.8.1` + `qdrant-client==1.19.1` (instalados local en la verificación; declarados en el protocolo) · `zep-cloud==3.30.0` (solo `--check`; ejecución requiere `ZEP_API_KEY`) |
| Implicaciones | 100% aditivo: `evals/runners/**` nuevo + docs. NO toca `src/`, workflows, plan file, Backlog, `CONSTRAINTS.md`, `perf-bench.yml`. Comparte `BENCHMARKS.md`/`COMPARISON.md` con la región DEF-06 (ya cerrada) — edits quirúrgicos al final del doc (append §19.x + bullets §Planificado). Riesgo: los números publicados deben ser reproducibles (Regla 11) o no publicarse |

## Impacto mapeado (Regla 0)

> Gate cumplido antes de la primera edición.

- **Archivos leídos (completos):** `evals/memory_harness.py` (559L) · `docs/user/operations/BENCHMARKS.md` (1251L) · `docs/user/COMPARISON.md` (225L) · `evals/README.md` (38L) · `evals/data/README.md` (49L) · `docs/dev/tasks/VER-08.md` (293L) · `.gitignore` (patrón evals) · `docs/user/blog/benchmarks_vs_lancedb_chroma.md` (patrón de post + frontmatter)
- **Archivos referenciados hacia dentro (imports):** `memory_harness.py` importa `calibration.py` (path local) y el paquete `vantadb` (opcional). El driver nuevo importa `memory_harness` (sin fork). `evals/runners/` no existía (0 referencias entrantes previas).
- **Archivos que referencian a los editados (referencias entrantes):** `rg "memory_harness" .github/` → solo `lurkr-informational.yml` referencia `evals/agent/**` (no tocado). `BENCHMARKS.md` §17/§19 citan `memory_harness.py` (intacto). `COMPARISON.md:17,:169-171` declara la capa pendiente (se actualiza a "protocolo + runners publicados; corridas nativas pendientes de keys/budget"). Plan Tasks 45-47 citan `evals/data/` (intacto).
- **Veredicto impacto:** **bajo** — aditivo puro; ningún contrato de producto cambia. Riesgo único: exactitud de los números publicados (mitigado: Regla 11 — comando + sha256 + hardware + reports gitignored; lo no medido se declara).

## Contrato

> **Verbatim del plan (Task 47, F6):**

"head-to-head publicado: Mem0/Zep/Letta corridos con el MISMO harness VER-08 (mismo dataset/top_k/hardware fijado; 0 forks del harness) Y protocolo completo publicado (versiones pineadas por sistema + stack LLM/embedding/reranking + juez determinista + comandos de reproducción, Regla 11) Y win/loss honesto por capacidad (recall@k, p50/p99, ingest QPS, tokens — pares accuracy+tokens) en `BENCHMARKS.md` §19/Planificado Y post técnico con metodología (draft en `docs/user/blog/`) Y `COMPARISON.md` :17/:169 actualizado (capa pendiente → corrida) Y EXE-02 absorbido: 0 claims vs competidores sin la tabla propia"

**Stop conditions del bloque (aplicables):** runner no pineable/instalable → correr los disponibles + disposición documentada por sistema + FIND (no bloquear); costo LLM excesivo → subset reducido + declarar n (Regla 11); head-to-head no cierra antes del anuncio → publicar harness + protocolo + resultados propios y diferir el win/loss (documentado, no silencioso); rabbit hole: leaderboard público / track 10M → NO.

## Spec (SDD — decisiones del protocolo, auditables por vanta-audit)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Arquitectura del runner layer | fork del harness / driver que importa `run_harness` + adapters al contrato `put/get/search/close` | ✅ **0 forks**: `evals/runners/` nuevo; `memory_harness.py` intacto; el driver importa `run_harness` y comparte el contrato de store (doc: `memory_harness.py:133-171`) |
| 2 | Entorno de ejecución real (verificado 2026-09-30) | correr todo / skip documentado | ✅ **Sin keys LLM, sin Docker, sin Ollama** (verificado: env, docker, :11434). Mem0 raw-mode es ejecutable **sin ningún LLM** (embedder local fastembed); Zep/Letta requieren key/servicio externo → `--check` los reporta SKIP con motivo |
| 3 | Config de Mem0 a correr | native (extracción LLM, default producto) / raw (`infer=False`) / no correr | ✅ **raw mode declarado** (`infer=False`, 0 llamadas LLM, 0 tokens) + embedder `fastembed` `BAAI/bge-small-en-v1.5` (384d, local ONNX) + qdrant-client embebido (path local). Native (gpt-4o-mini pinneado) = **pending** (needs provider key + budget; estimado declarado, no medido). Raw ≠ default de producto — **declarado en todos los artefactos** |
| 4 | Config de Zep | Zep Cloud (key+créditos) / Graphiti self-host (Neo4j/FalkorDB+LLM) / skip | ✅ **Skip documentado**: sin `ZEP_API_KEY`; Graphiti exige Neo4j/Docker + keys (no disponibles). Mapping implementado contra `zep-cloud==3.30.0` (thread.create → add_messages chunked ≤4000 chars con metadata `session_id` → `graph.search(scope="episodes")`); costo estimado declarado (subsets reducidos + n) |
| 5 | Config de Letta | mapear al contrato put/search / skip con rationale | ✅ **Skip arquitectónico documentado**: Letta 0.33.x es un *agent harness* (memoria gestionada por el agente: MemFS/dreaming; `letta --backend local` + provider), no un store put/search. El contrato no mapea; protocolo QA-level (LLM-in-the-loop) diferido → FIND |
| 6 | Juez | determinista VER-08 / LLM GPT-4o | ✅ Determinista session-level heredado (recall_all@k headline + recall_any@k secundaria; `memory_harness.py:11-23`). Sin cambios |
| 7 | Métricas comparables | mismas métricas para todos | ✅ Mismas: recall@k, p50/p95/p99, ingest QPS, write-quality (solo stores con get exacto: vantadb + mem0-raw), abstención proxy, token-economy, aislamiento. **ECE/calibración se anula para externos** (scores no son probabilidades calibradas; driver lo reemplaza por nota declarada) |
| 8 | Protocolo de publicación | narrativa / protocolo pineado | ✅ Protocolo pineado en §Protocolo de este task + `evals/runners/README.md`: versiones, stacks, dataset sha256, top_k=5, hardware, comandos, límites de no-comparabilidad |
| 9 | Dataset de la corrida | full split local (470 non-abs) / subset commiteado | ✅ Subset commiteado = smoke reproducible offline; split completo = corrida de medición. **Full-500 de mem0-raw no factible (≈0.85 s/doc → ≈5.6 h)** → stop condition aplicada: **slice estratificado declarado `--sample-stratified 10`** (60 preguntas, 10 por tipo — el dataset está ordenado por bloques de tipo; un `--limit` simple cubre 1 tipo = cherry-picking involuntario). n y hardware declarados por corrida |
| 10 | Presupuesto LLM | n/a raw / estimar nativos | ✅ **0 tokens en la corrida publicada** (raw mode). Estimaciones declaradas como estimaciones (no medidas) para native/Zep: full split mem0-native ≈ 24k extracciones ≈ orden $10-50; Zep full ≈ 790k créditos (free tier 10k) → subsets. Decisión de gasto = owner |
| 11 | Cierre de artefactos | qué se publica | ✅ `BENCHMARKS.md` §19.x (tabla + protocolo + límites), `COMPARISON.md` :17/§7 (capa → corrida con límites), post draft `docs/user/blog/head_to_head_mem0_zep_letta.md`, `evals/runners/README.md`, reports JSON gitignored. EXE-02 absorbido: 0 claims sin la tabla propia |

## Protocolo (pineado — input del review vanta-audit)

**Harness:** `evals/memory_harness.py` (VER-08, intacto) — juez determinista session-level:
`recall_all@k` (headline; todas las `answer_session_ids` ⊆ top-k) + `recall_any@k`
(secundaria); top_k=5; dataset LongMemEval-S (`xiaowu16/longmemeval-cleaned`, MIT;
subset commiteado sha256 `b8a994ba…`; full split sha256 `d6f21ea9…`); hardware en el reporte.

**Sistemas y versiones pineadas (2026-09-30):**

| Sistema | Pin | Config de la corrida | LLM | Estado |
|---|---|---|---|---|
| vantadb | 0.7.0 (PyO3) | `text_query` híbrido BM25+HNSW, memory backend | — | corre (referencia) |
| mem0 OSS | `mem0ai==2.2.1` | **raw** (`infer=False`) + `fastembed==0.8.1` `BAAI/bge-small-en-v1.5` (384d) + `qdrant-client==1.19.1` local (`path`) | **ninguno** (0 llamadas) | corre |
| mem0 OSS | `mem0ai==2.2.1` | native (extracción `gpt-4o-mini`, embedder fastembed, qdrant local) | OpenAI key | **pending** (keys/budget owner) |
| Zep | `zep-cloud==3.30.0` | thread/messages chunked + episode search | Zep Cloud (créditos) | **skip** (sin key) |
| Letta | `letta==0.33.8` | n/a (agent harness) | provider | **skip** (arquitectura + sin provider) |

**Límites de no-comparabilidad (declarar siempre):** (1) mem0-raw ≠ default de producto de
mem0 (extracción deshabilitada) — capa de almacenamiento/recuperación, no su pipeline de
extracción; (2) vantadb corre text-only en este run vs semántico+BM25 de mem0-raw — stacks
declarados por sistema, no igualados; (3) Zep/Letta no corridos; (4) write-quality solo
vantadb/mem0-raw; ECE solo vantadb; (5) token-economy = palabras (proxy); (6) n y hardware
declarados por corrida; (7) ninguna cifra se toma de leaderboards de terceros.

**Comandos (Regla 11):** `python evals/runners/head_to_head.py --check` ·
`--self-test` (offline) · `--systems vantadb,mem0 --label h2h-5q` ·
`--data datasets/longmemeval/longmemeval_s_cleaned.json --label h2h-full500` (cache local).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `evals/memory_harness.py` y `evals/calibration.py` NO se editan (0 forks; el driver los importa).
  2. `src/**` de producción y `vanta-proxy/**` intactos; regiones co-batch EXE-01 (workflows) solo lectura.
  3. Reportería honesta (Regla 11): todo número citable cita comando + dataset sha256 + hardware; `potential impact` ≠ medición; raw-mode etiquetado donde aparezca.
  4. Los skips son documentados con motivo y FIND — nunca silenciosos; nunca fabricar números.
  5. Reports JSON gitignored (patrón); la fuente citable es BENCHMARKS §19 + comando.
  6. `Backlog.md`, plan file, `CONSTRAINTS.md`, `perf-bench.yml`, `desktop/**`, `opencode.jsonc` PROHIBIDOS (LEAD).
- **Comandos de verificación:**
  - `python evals/runners/head_to_head.py --self-test` → OK (checks offline con SDKs fake)
  - `python evals/runners/head_to_head.py --check` → tabla de disponibilidad (vantadb ✅; mem0 raw ✅ vía pin local; zep/letta SKIP documentado)
  - `python evals/runners/head_to_head.py --systems vantadb,mem0 --label h2h-<n>` → filas reales + JSON
  - `python evals/memory_harness.py --self-test` → OK (13 checks — regresión del harness intacto)
  - `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write` → verde
  - `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps
- **Deuda pendiente:** runs nativos (mem0-native/Zep/Letta QA-level) = lane owner con keys/budget (FINDs); re-review P2-01 del protocolo (LEAD, vanta-audit).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR: sin deuda** — aditivo de evaluación + docs; no toca código de producto. `mem0ai/fastembed` instalados en el user-env local SOLO para la verificación (declarado; no es dependencia del repo).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato verificado por cláusula (§Verificación por cláusula) |
| **Commit** | Lo ejecuta el LEAD (wave F6: commits = LEAD). Commit sugerido: `feat(ver09): head-to-head runner layer + protocol + mem0 raw-mode run (F6)` |
| **Release** | No aplica (evals/docs, sin release ni semver surface) |

## Herramientas necesarias → Skills (SDP)

| Skill | Fase | Justificación |
|-------|------|---------------|
| performance-optimization | BUILD | el harness ES medición (Regla 9/11); comparativas de latencia/QPS |
| source-driven-development | BUILD | APIs/versiones/licencias de 3 SDKs externos verificadas contra docs oficiales (17 fuentes §Investigation) |
| test-driven-development | BUILD | `--self-test` offline con SDKs fake ANTES de la corrida real |
| doubt-driven-development | BUILD | verificación adversarial del protocolo/mapeos (placeholder de vanta-audit) |
| documentation-skill | BUILD | BENCHMARKS/COMPARISON/blog/READMEs (sintaxis + frontmatter + gates) |

**SKILLS_CARGADAS:** performance-optimization, source-driven-development, test-driven-development, doubt-driven-development, documentation-skill (+ base campaign-executor/progreso automáticas; coordinated-web-search para research — cargada).

## Investigation Notes

### Pinnabilidad y licencias (verificado 2026-09-30, fuentes oficiales)

| SDK | Pin verificado | Licencia | Fuente |
|---|---|---|---|
| `mem0ai` | 2.2.1 (PyPI; resuelve en Python 3.14; instalado ✅) | Apache-2.0 | pypi.org/pypi/mem0ai/json · docs.mem0.ai |
| `fastembed` | 0.8.1 (Apache-2.0; instalado ✅; modelo `BAAI/bge-small-en-v1.5` descargado de HF/Qdrant) | Apache-2.0 | pypi.org/pypi/fastembed/json |
| `qdrant-client` | 1.19.1 (traído por mem0ai; local mode `path`) | Apache-2.0 | mem0 docs qdrant (parámetro `path`) |
| `zep-cloud` | 3.30.0 (PyPI) | (SDK cliente; sin `license_expression` en PyPI) | help.getzep.com · pypi |
| `letta` | 0.33.8 (PyPI win_amd64; "Letta Code" — agent harness) | Apache-2.0 | pypi.org/pypi/letta/json · docs.letta.com |
| Graphiti (Zep OSS) | 0.30.2 | Apache-2.0 | pypi · help.getzep.com/graphiti |

### APIs pineadas (contra docs oficiales — URLs en §Fuentes)

- **mem0**: `Memory.from_config` (config `llm/embedder/vector_store`); `add(messages, user_id=…, metadata=…, infer=False)`; `search(query, filters={"user_id": …}, limit=k)` → `{"results": [{id, memory, metadata, score, …}]}`; `get_all(filters={"user_id": …})`. Probado en vivo: add→search→get_all OK (score 0.31; metadata `session_id` preservada). Límite docs: OSS v3 exige entidades DENTRO de `filters` (top-level `user_id` lanza ValueError) — el adapter usa `filters` con fallback.
- **Zep**: `Zep(api_key=…)`; `thread.create(thread_id, user_id)`; `thread.add_messages(thread_id, messages=[Message(role, content, metadata, name)])` (**≤30 msgs/call, ≤4,096 chars/msg** → chunking declarado); `graph.search(user_id, query, scope="episodes", limit)` (query ≤400 chars); metadata de mensaje = metadata de episodio (proyecta a artefactos) → filtro `episode_metadata_filters`. Reranker default RRF.
- **Letta**: arquitectura actual = harness de agentes (CLI `@letta-ai/letta-code` / pip `letta`; `letta server --backend local` + provider). Sin primitive put/search. No mapea.
- **Entorno de ejecución:** sin `OPENAI_API_KEY`/`ZEP_API_KEY`/`LETTA_API_KEY` (verificado); sin Docker; Ollama no responde; pip/red OK; full split cache local presente (277,383,467 B).

### Instalaciones locales realizadas (declarar)

`pip install --user mem0ai==2.2.1 fastembed==0.8.1` (trae qdrant-client 1.19.1, sqlalchemy, grpcio, …) + `zep-cloud==3.30.0`. Modelo fastembed descargado a `%TEMP%\fastembed_cache`. NO son dependencias del repo (evals-only; el runner falla con skip claro si no están).

### Verificación contra SDKs instalados (no solo docs)

- **mem0 2.2.1:** firmas inspeccionadas en el paquete real — `add(messages, *, user_id, metadata, infer=True)`, `search(query, *, top_k=20, filters, threshold=0.1)`, `get_all(*, filters, top_k=20)` → 3 bugs de integración detectados y corregidos ANTES de la corrida publicable (ver Step 3). Embedding ONNX de sesiones largas ≈ 840-950 ms/doc (cuello medido con cProfile: `onnxruntime.run` 866 ms) — es la causa del subset reducido.
- **zep-cloud 3.30.0:** import + constructores verificados en vivo (`thread.create(*, thread_id, user_id)`; `add_messages(thread_id, *, messages)`; `graph.search(*, user_id, query, scope, limit)`; `Episode{content, metadata, score, uuid}`) → el adapter Zep queda verificado contra el SDK real, ejecutable al proveer `ZEP_API_KEY`.
- **Letta 0.33.8:** PyPI describe "Letta Code: stateful agents in your terminal" (harness de agentes, MemFS) → confirmado que no mapea al contrato put/search.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 (las 3 resueltas: mem0 raw ejecutable; Zep skip sin key; Letta no mapea) |
| Pendientes de ejecución (downhill) | 0 al cierre |
| % completado | 100% al cierre |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** aplica como higiene de datos y red: (a) keys nunca hardcodeadas ni commiteadas (el adapter lee env; dummy key solo in-process, jamás impresa); (b) mem0 telemetry OFF best-effort (`MEM0_TELEMETRY=False`); (c) datasets solo redistribuibles (mitad VER-08, sin cambios); (d) el runner externo escribe solo en temp/local dirs. No toca trust boundaries del producto.
- **PERFORMANCE:** aplica como *medición* (Regla 9/11): latencias/QPS comparados con el mismo harness; baseline citado = §19 full-500 (vantadb) + §13/§11. No se modifica ningún hot path.

## Steps

- Step 1: DISCOVERY (contrato + entorno + APIs + licencias + SDP) — ✅
- Step 2: Task file (este archivo) — ✅
### Step 3: Runner layer `evals/runners/` (adapters + driver + self-test offline) — ✅
- **Archivos:** `evals/runners/adapters.py` · `evals/runners/head_to_head.py` · `evals/runners/README.md`
- **Verify:** `python evals/runners/head_to_head.py --self-test` → `OK (10 checks)` ✅ (fakes estrictos: firmas keyword-only + asserts de kwargs desconocidos, chunking lossless, skips, E2E con `run_harness`)
- **Fixes descubiertos por la corrida real (TDD empírico):** `get_all` de mem0 default `top_k=20` → el adapter pide `top_k=10_000` (write-quality pasó de 0.39 a 1.0 en el debug 53/53); `search` usa `top_k=` (no `limit=`, que iba a `**kwargs`); qdrant `path: ":memory:"` (sin residuos de disco ni fsync; in-memory declarado); `supports_write_quality` por `getattr` (VantaStore no lo declara).
### Step 4: Corridas reales (vantadb + mem0-raw) — ✅
- Smoke `h2h-5q` (subset commiteado, adapter corregido): ✅ recall 1.0/1.0 ambos; vantadb p50 0.57 ms / 269.4 QPS; mem0 p50 106.2 ms / 1.2 QPS / **wq 1.0000** (posible tras fix `get_all top_k`).
- **Full-500 NO factible** con mem0-raw en esta máquina: ~0.85 s/doc (embedding ONNX bge-small de sesiones de 2-18KB medido 840-950 ms; 23,867 sesiones ≈ 5.6 h) → **stop condition aplicada: subset reducido + n declarado**.
- **Corrida publicable `h2h-strat10`** (split completo cacheado, sha256 `d6f21ea9…`): `--sample-stratified 10` = 60 preguntas (10 por cada uno de los 6 tipos; el dataset está ordenado por bloques de tipo → un `--limit` simple cubría UN tipo = cherry-picking involuntario). **Medido:**
  - vantadb 0.7.0: recall_all@5 **0.7167** (43/60) · recall_any@5 0.8833 · q p50/p99 **0.65/1.09 ms** · ingest **243.9 QPS** · wq 1.0 · tokens 10,119 · isolation 0/300 · ECE 0.1167→0.0002 (T\*=6.83)
  - mem0 2.2.1 raw: recall_all@5 **0.9333** (56/60) · recall_any@5 0.9833 · q p50/p99 550.0/663.7 ms · ingest 1.3 QPS · wq 1.0 · tokens 10,019 · isolation 0/300 · ECE N/A (declarado)
  - by type (all, vantadb → mem0): knowledge-update 0.7→1.0 · multi-session **0.2→0.8** · assistant 0.9→1.0 · preference 0.8→0.9 · single-user 1.0→1.0 · temporal 0.7→0.9 — **mem0 gana todos los tipos; derrota publicada como tal**
- **Verify:** reports `evals/runners/report_h2h_h2h-strat10_{vantadb,mem0}.json` (gitignored; regenerables con el comando del README) + tabla en BENCHMARKS §19 (subsección Head-to-head).
### Step 5: Publicación (BENCHMARKS §19 + COMPARISON + README runner + post + evals/README + .gitignore) — ✅
- **Archivos:** `docs/user/operations/BENCHMARKS.md` (subsección §19 "Head-to-head — mem0 OSS (raw mode) vs vantadb" + §Planificado VER-09 → ENTREGADO) · `docs/user/COMPARISON.md` (:17 nota 2026-09-30 + §7 cierre) · `docs/user/blog/head_to_head_mem0_zep_letta.md` (nuevo, `status: draft`) · `evals/runners/README.md` · `evals/README.md` · `.gitignore`
- **Verify:** `check-links` exit 0 · `check-docs` exit 0 · `gen-index.mjs --write` (3 stale actualizados: docs/index.md, docs/user/index.md, llms.txt) · `validate-docs-coverage.ps1` → 0 gaps · `markdownlint-cli2` sobre los 4 docs tocados → **0 issues**.
### Step 6: Validación final (self-tests + gates docs + RESULTADO) — ✅
- **Verify:** `head_to_head --self-test` → OK (10 checks) · `memory_harness --self-test` → OK (13 checks — harness intacto) · `calibration --selftest` → OK (6 checks) · `head_to_head --check` → vantadb ok 0.7.0 / mem0 ok 2.2.1 (raw) / zep skip 3.30.0 / letta skip (arquitectura) · `py_compile` ×2 → 0.
- Reportes smoke y strat verificados como gitignored (`git check-ignore` OK; `__pycache__` ignorado).

## Dependencias

- **Prerrequisitos:** VER-08 ✅ (`0405eeda`); pin `mem0ai==2.2.1` instalado local ✅; full split cache local ✅.
- **Consumidores:** F6 anuncio (EXE-03 owner coordina timing); hallazgos FIND (LEAD inserta).
- **nextTask:** cierre de campaña (certify/ship) — LEAD.

## Verificación por cláusula (contrato verbatim → evidencia)

**Entorno de la corrida (Regla 11):** Windows 11 (10.0.26200) · Intel 12th Gen (12 logical) · 31.78 GB RAM · Python 3.14.7 · máquina compartida bajo carga · disco C: ~1.7 GB libres al correr · single-thread. Pins: vantadb 0.7.0 · mem0ai 2.2.1 + fastembed 0.8.1 + qdrant-client 1.19.1 (local, `:memory:`) · zep-cloud 3.30.0. Dataset full split sha256 `d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442`; subset commiteado `b8a994ba…`.

| # | Cláusula del contrato | Evidencia | Estado |
|---|----------------------|-----------|--------|
| 1 | Mem0/Zep/Letta corridos con el MISMO harness VER-08 (0 forks) | driver importa `run_harness` (`head_to_head.py:34-40` import; `:149-155` llamada); `memory_harness.py` intacto (selftest 13/13). **mem0 corrido** (strat10 + smoke); **Zep/Letta skip documentado** por stop condition (sin `ZEP_API_KEY` / contrato no mapea) | ✅ (forma stop-condition: "correr los disponibles + disposición documentada") |
| 2 | Protocolo completo publicado (versiones pineadas + stack + juez determinista + comandos) | `evals/runners/README.md` §Protocolo + BENCHMARKS §19 subsección Head-to-head + `--check` | ✅ |
| 3 | Win/loss honesto por capacidad (recall@k, p50/p99, ingest QPS, tokens) | tabla strat10: mem0 gana recall en todos los tipos (0.9333 vs 0.7167; multi-session 0.8 vs 0.2); vantadb gana latencia ≈850× e ingest ≈187×; tokens ≈; write-quality/isolation tie | ✅ (derrota incluida) |
| 4 | Post técnico con metodología (draft en `docs/user/blog/`) | `docs/user/blog/head_to_head_mem0_zep_letta.md` (`status: draft`) | ✅ |
| 5 | COMPARISON.md :17/:169 actualizado (capa pendiente → corrida) | :17 nota "Memory-as-a-service layer (updated 2026-09-30)"; §7 cierre con la derrota explícita ("lost the recall cells") | ✅ |
| 6 | EXE-02 absorbido: 0 claims vs competidores sin la tabla propia | todo claim vs Mem0 sale de §19.x (tabla propia); Zep/Letta sin números (disposición + FIND); referencias absorbidas citadas (`Backlog:913/947`) | ✅ |

## Findings / disposiciones (para LEAD — Backlog prohibido en este agente)

| ID propuesto | Descripción | Dueño sugerido |
|---|---|---|
| FIND-* (a crear) | Zep Cloud runner listo y verificado contra `zep-cloud==3.30.0` pero no ejecutado: requiere `ZEP_API_KEY` + créditos (full ≈790k créditos vs free tier 10k → subset reducido + n declarado). Comando en `evals/runners/README.md` | owner / vanta-tuner |
| FIND-* (a crear) | Letta: protocolo QA-level (LLM-in-the-loop) diferido — es un agent harness (MemFS/dreaming); el contrato put/search no mapea | vanta-tuner / vanta-audit |
| FIND-* (a crear) | mem0 native (extracción LLM, `gpt-4o-mini` pinneado) pendiente de keys/budget del owner; el adapter ya lo soporta (`--mem0-mode native`); costo = estimación declarada, no medida | owner / vanta-tuner |
| FIND-* (a crear) | Slice estratificado sin celdas `_abs`: extender `--sample-stratified` para incluir abstención en la v2 del protocolo | vanta-tuner |
| Nota (no FIND) | Full-500 mem0-raw ≈5.6 h en esta máquina (0.85 s/doc medido) — corrida del owner cuando haya hardware/ventana | owner |

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-audit (LEAD orquesta commit + review; este sub-agente NO se auto-revisa — mandato wave F6).
- **Protocolo explícito a auditar (diseño + honestidad):**
  1. **Mapeo mem0-raw:** `infer=False` (0 llamadas LLM; 0 tokens) + fastembed local `bge-small-en-v1.5` + qdrant `:memory:` (declarado); contra el SDK real: `get_all(top_k=10_000)` (default era 20 — causaba wq 0.39), `search(top_k=)` (no `limit=`), threshold default 0.1 conservado (declarado); ECE anulada con nota para externos.
  2. **Mapeo Zep:** chunking ≤4000 chars/mensaje (límite duro de API), metadata `session_id` (proyecta a episodios), `graph.search(scope="episodes", limit=k)`; write-quality N/A declarada; verificado contra las firmas del SDK real (`thread.create`/`add_messages`/`graph.search`).
  3. **Letta:** skip arquitectónico documentado (no es un store put/search).
  4. **Muestreo:** estratificado 10/tipo (anti cherry-picking; el archivo está ordenado por bloques de tipo); auditar que no quede sesgo de selección adicional.
  5. **Honestidad:** "raw ≠ default de mem0" etiquetado en BENCHMARKS, post, runners README y reports; la derrota de recall publicada también en COMPARISON §7; ningún número de terceros usado como fuente.
- **Cómo se probó (evidencia mecánica, no auto-reporte):** `--self-test` (10 checks, fakes estrictos con firmas keyword-only) · `--check` · corridas reales regenerables (comandos exactos) · reports JSON con sha256 + hardware (gitignored) · gates docs verdes (Steps 5-6).
- **Veredicto:** ✅ **APPROVE** (ronda 1, vanta-audit, 2026-09-30): **0 Critical/High** · 1 Medium latente + 3 Low + 2 Info → **fixes F1–F6 aplicados y re-verificados** (§Fixes review). Delta-review/commit = LEAD.

## Fixes review (vanta-audit — APPROVE ronda 1, batch F1–F6, 2026-09-30)

| # | Fix aplicado | Evidencia (comando → resultado) |
|---|--------------|----------------------------------|
| **F1** (Medium latente) | `ZepStore.search` lee `uuid_` (campo REAL de `zep_cloud.types.Episode` 3.30.0 — verificado: `model_fields` incluye `uuid_`, no `uuid`); `adapters.py:345`. Fake del selftest expone `self.uuid_` + nuevo assert del fallback sin metadata (`"ep-uuid"`) | `head_to_head --self-test` → OK (10 checks) — con el nombre viejo el fallback daría `"?"` y el assert falla (cobertura real, ya no enmascarado) |
| **F2** (Low) | `config_note` en `Mem0Store.config_summary()`: `"raw mode = extraction disabled; NOT mem0's default product pipeline"` (y su variante native) | `adapters.py:267`; el campo viaja en `system_config` de los reports nuevos |
| **F3** (Low) | README: comando publicable = `--sample-stratified 10`; nota explícita "`--limit N` sobre el split completo NO es publicable (bloques de tipo)" | `evals/runners/README.md` (bloque Comandos) |
| **F4** (Low) | `check_availability` mem0: `_has_module("fastembed")` → skip claro; `Memory.from_config` envuelto en try/except `ImportError` → `RunnerSkipped` | `--self-test` check 7 simula fastembed ausente (`table2` → skip con motivo "fastembed"); `--check` normal → mem0 ok |
| **F5** (Info) | `≈185×` → `≈187×` (ratio exacto 2226.156 s / 11.89 s = 187.2×) | BENCHMARKS §19.x · post · task file §Verificación cláusula 3 |
| **F6** (Info) | Cita corregida: `:34-40` (import) + `:149-155` (llamada a `run_harness`) | task file §Verificación cláusula 1 |

**Re-verify batch (comandos → resultado):** `head_to_head --self-test` → OK(10) · `--check` → vantadb ok / mem0 ok(raw) / zep skip / letta skip · `memory_harness --self-test` → OK(13) · `calibration --selftest` → OK(6) · gates docs verdes (ver §Steps 5-6 + re-run del cierre).

## Notas

- Wave F6 co-batch con EXE-01 (workflows demos): no se tocan sus archivos; `perf-bench.yml`/`Backlog.md` prohibidos.
- EXE-02 absorbido: sus claims pasan a la tabla propia (§19) — no duplicar.
- Post técnico sigue el patrón `benchmarks_vs_lancedb_chroma.md` (frontmatter + metodología + honestidad) y `BLOG_SERIES_PLAN.md`.
- **Entorno (declarar):** para la verificación se instaló localmente `mem0ai==2.2.1`, `fastembed==0.8.1` (+qdrant-client 1.19.1) y `zep-cloud==3.30.0` (`pip install --user`) — NO son dependencias del repo (el runner skipea con motivo claro si faltan). Se purgó el pip cache (~232 MB) y se eliminó un report parcial del primer intento (`h2h-s50`) para liberar disco (C: ~1.7 GB libres al correr — declarado como condición del entorno). Modelos fastembed en `%TEMP%\fastembed_cache`.
- **`git status` al cierre:** solo archivos de VER-09 (+ generados de docs) y la región co-batch EXE-01 (no tocada por este agente; commits = LEAD). Reports JSON y `__pycache__` verificados como gitignored (`git check-ignore` OK).
- **Nota para el LEAD:** `docs/index.md`/`docs/user/index.md`/`llms.txt` fueron regenerados con `gen-index --write` al cierre de esta tarea — incluirlos en el commit del batch F6 (pueden incluir también altas de EXE-01).
