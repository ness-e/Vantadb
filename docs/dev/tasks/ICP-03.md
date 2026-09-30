---
title: "ICP-03: Track frameworks — one-pager + adapters PyPI (MKT-18f) + importadores (VER-05) + demo CI"
kind: task
description: "One-pager del track frameworks + demo CI dedicada dev→prod (mismo grafo, solo cambia el checkpointer) + instalación de adapters desde PyPI o disposición Alpha documentada + entrada COMPARISON.md + métrica installs/semana documentada (pypistats, baseline 0)"
---

# ICP-03: Track frameworks — one-pager + adapters PyPI (MKT-18f) + importadores (VER-05) + demo CI

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 44, Fase F5 — wave F5.2, co-batch DEF-06)
- **Fuente:** Backlog `ICP-03` (P54, L968) · master Task 44 · `docs/dev/strategy/GO_TO_MARKET.md` §Vertical 2 (:165-178)
- **Esfuerzo:** 🟡 1sem (re-baseline: VER-05 ✅ + guías ✅ + smokes de frameworks ya en CI → slice real = one-pager + demo dedicada dev→prod + COMPARISON + métrica; publish = lane owner)
- **Prioridad:** 🟠
- **Tipo:** CI/CD + Docs + demo Python (sin símbolos públicos nuevos, sin cambios de core)
- **Turns estimados:** 7
- **Creado:** 2026-09-30T00:00
- **last-synced:** 2026-09-30
- **Estado:** ⏳ IN PROGRESS (implementación 7/7 ✅ + review ronda 1 ✅ APPROVE + batch post-review aplicado/re-verificado; ⬜ pendiente de proceso: commit local = LEAD)
- **Incógnitas (uphill):** 0 — publish MKT-18f pendiente = stop condition aplicada (disposición Alpha + instalación local + FIND switch PyPI); sin rabbit holes (no adapters nuevos).
- **Pendientes (downhill):** 0 de código — batch post-review ✅ (Opt1/Opt2/Nit SDK/Nit2); pendiente solo el cierre de proceso (commit local, LEAD)
- **Iteraciones:** 1 (ejecución en wave F5.2; 0 retries de verify)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | CI (`ci-frameworks-demo.yml` nuevo) · F6 anuncio (vertical 2) · COMPARISON.md (§9) · MKT-18f (handoff publish: el one-pager/demo documentan el switch a PyPI cuando se publique) · DEF-06 co-batch (no tocar claims/README/BENCHMARKS) |
| Callees | `integrations/langchain/vantadb_langchain/{checkpointer,store,vectorstore}.py` (INTG-01; `VantaDBCheckpointer`, `VantaDBStore`) · `integrations/test_pins.py` (gate FIND-84) · `.github/workflows/ci-examples.yml` (patrón de wheel+smokes) · `docs/user/tutorials/migrating-from-{mem0,zep,letta}.md` (VER-05) · `docs/api/MEMORY_INTERCHANGE_FORMAT.md` |
| Implicaciones | (1) Demo **offline y determinista**: el nodo del grafo es Python puro (no LLM, sin tokens, sin red externa); prueba el pipeline de checkpointing (put/get/list/resume cross-proceso), no calidad de modelo. (2) La demo **no cambia el adapter** (solo lo consume) — el pin refresh de los adapters es lane owner (Task 41/MKT-18f). (3) `vanta-py` deprecation warning de `import vantadb_py` en adapters = deuda pre-existente, no de esta task. (4) `COMPARISON.md` coopera con la wave (sección cualitativa al final, sin números — Regla 11); NO se toca README/BENCHMARKS (DEF-06). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `integrations/langchain/vantadb_langchain/checkpointer.py` (350L) · `integrations/langchain/vantadb_langchain/__init__.py` (25L) · `integrations/langchain/README.md` (105L) · `integrations/langchain/pyproject.toml` (39L) · `integrations/langchain/tests/conftest.py` (3L) · `integrations/README.md` (92L) · `integrations/test_pins.py` (40L) · `examples/python/langgraph_checkpoint.py` (295L) · `.github/workflows/ci-examples.yml` (146L) · `.github/workflows/ci-ai-ides-demo.yml` (80L) · `docs/user/AI_IDES.md` (90L) · `docs/user/COMPARISON.md` (194L) · `docs/dev/tasks/ICP-01.md` (211L) · `docs/dev/tasks/MKT-18f.md` (51L) · `docs/user/tutorials/migrating-from-mem0.md` (tramo 1-50) · `docs/dev/strategy/GO_TO_MARKET.md` (tramo 150-194) · `.opencode/rules/release-ci.md` (42L) · `.opencode/references/clean-code-clean-architecture.md` §Apéndice V (:646-697) · `scripts/docs/check-doc-examples.mjs` (tramos 100-463) · `scripts/validate-docs-coverage.ps1` (header)
- **Archivos referenciados hacia dentro (imports/dependencias):** `checkpointer.py` importa `vantadb_py` + `langchain_core.runnables` + `langgraph.checkpoint.base` (pin `langgraph-checkpoint>=2,<5`) · `integrations/langchain/__init__.py` re-exporta `VantaDBCheckpointer`/`VantaDBStore` con guard try/ImportError · demo nueva dependerá de `langgraph` (paquete runtime) + `vantadb_langchain` instalado · workflow nuevo dependerá de `.github/actions/rust-setup` + `PyO3/maturin-action` + `actions/setup-python` (pins SHA espejo de `ci-examples.yml`)
- **Archivos que referencian a los editados (referencias entrantes):** `docs/user/index.md` + `llms.txt` (generados ← `gen-index.mjs`; FRAMEWORKS.md se indexará) · `COMPARISON.md` ← `README.md`, `BENCHMARKS.md` ("single canonical source") · `integrations/README.md` ← docs del track · `ci-examples.yml:123` ya instala `./integrations/langchain` (no se toca) · `MKT-18f` (handoff: el post-release debe quitar "Not on PyPI yet" ×9 y el one-pager debe actualizar el bloque PyPI)
- **Veredicto impacto:** **bajo y acotado** — archivos nuevos (`examples/langgraph_dev_to_prod/{demo.py,README.md}`, `.github/workflows/ci-frameworks-demo.yml`, `docs/user/FRAMEWORKS.md`, task file) + 1 edición quirúrgica (`COMPARISON.md` §9, mismo patrón §7/§8 de ICP-01/02). Sin código core, sin bindings, sin deps nuevas del workspace, sin símbolos públicos. Regiones de co-batch intactas (`README.md` raíz/BENCHMARKS/claims = DEF-06; `vanta-proxy/**`/`src/sdk/importers/**` = solo lectura).

## Contrato

> **Verbatim del plan (Task 44):** "one-pager del track (frameworks: LangGraph/CrewAI/DSPy/Haystack/LlamaIndex; aislamiento multi-usuario; migración desde Mem0/Zep/Letta) Y demo CI verde de 'dev→prod sin cambiar código' (LangGraph: mismo grafo, solo cambia el checkpointer a VantaDB; test/workflow que falla si el código de negocio difiere) Y instalación de adapters desde PyPI verde (`pip install vantadb-langchain` + smoke) o disposición documentada si MKT-18f queda Alpha Y entrada en `COMPARISON.md` Y métrica installs/semana documentada con método (PyPI stats) — baseline 0 hasta publicar"

Desglose verificable (5 cláusulas):
1. **One-pager** publicado en `docs/user/FRAMEWORKS.md`: frameworks (LangGraph/CrewAI/DSPy/Haystack/LlamaIndex + los 9 adapters), aislamiento multi-usuario (namespaces), migración desde Mem0/Zep/Letta (guías VER-05), instalación, límites honestos.
2. **Demo CI verde** dev→prod: mismo grafo, solo cambia el checkpointer (InMemorySaver → `VantaDBCheckpointer`); corre en workflow PR-blocking; **falla si el código de negocio difiere** (fingerprint de fuentes + assert de sección negocio libre de backend + equivalencia de comportamiento + control negativo dev + aislamiento de threads).
3. **Instalación de adapters desde PyPI** verde **o disposición documentada** (MKT-18f = Alpha: 9/9 → 404 live 2026-09-29, re-verificado `vantadb-langchain` 404 2026-09-30) → disposición Alpha + instalación local en demo/CI/one-pager + FIND para el switch a PyPI cuando publique.
4. **Entrada en `COMPARISON.md`** (§9 — cualitativa, links, sin números).
5. **Métrica installs/semana** documentada con método (pypistats API/CLI, shape verificado) + baseline 0 hasta publicar + Regla 11 (solo números reales).

**Stop conditions activas (plan :1167):** MKT-18f Alpha → demo CI con instalación local + FIND switch PyPI (NO intentar publicar) · renombrar paquetes (colisión) → disposición por adapter + FIND · rabbit hole: adapter nuevo (Pydantic AI/AutoGen) → NO.

## Spec (tabla de decisiones — gate spec-first)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Forma de la demo | A) script Python único `demo.py` con modo driver + `_child` (subprocess ×5: dev-first, dev-resume, prod-first, prod-resume, prod-other-thread) / B) test pytest / C) extender `examples/python/langgraph_checkpoint.py` (smoke existente) / D) Rust test como ICP-01/02 | A | ✅ decidido-por-evidencia: probar **persistencia cross-proceso** exige procesos separados (el valor real del checkpointer en prod); plain `python` sin pytest (CI thin); B no aporta sobre A y añade dep; C rompe el invariante de `ci-examples` ("examples importan solo vantadb_py + stdlib") y el smoke ya corre; D no corresponde (frameworks = Python). Patrón ICP-01/02: artefacto dedicado + workflow dedicado |
| 2 | Check "código de negocio no difiere" | A) fingerprint sha256 de `inspect.getsource(respond|build_graph)` + assert de tokens prohibidos en la sección negocio (`InMemorySaver`/`VantaDBCheckpointer`/`vantadb`/`dev`/`prod`/`mode`) + `assert` de equivalencia de comportamiento (dev-first == prod-first) comparado por el driver / B) hash del archivo entero (no distingue secciones) / C) solo revisión manual | A | ✅ decidido-por-evidencia: A es mecánico y localiza la violación (una sección negocio que conozca el backend rompe el assert; un fork dev/prod rompe la equivalencia de fingerprints). C no es gate; B da falsos positivos por wiring |
| 3 | Instalación del adapter en CI | A) `pip install --no-deps ./integrations/langchain` + `pip install "langgraph>=1,<2"` explícito / B) `pip install ./integrations/langchain` normal (resuelve pins stale: `vantadb-py<0.7.0` + `langchain-core<1`) | A | ✅ decidido-por-evidencia: B con el pin `vantadb-py>=0.5.0,<0.7.0` **reemplaza el wheel local 0.7.0 por 0.6.x de PyPI** y `langchain-core<1` choca con `langgraph` 1.x (que exige `langchain-core>=1.4.7` — `importlib.metadata.requires('langgraph')`); el refresh de pins es lane owner (Task 41/MKT-18f, finding compartido) — NO se tocan pyproject en esta task. A corre contra las versiones vigentes reales |
| 4 | Cláusula PyPI (MKT-18f ⬜) | A) disposición Alpha documentada + instalación local (stop condition del plan) + FIND switch / B) intentar publish (PROHIBIDO — lane owner con tokens) | A | ✅ decidido-por-evidencia: 404 live 9/9 (2026-09-29) + stop condition plan :1167; el one-pager documenta ambos caminos (hoy local / post-release PyPI) |
| 5 | Métrica installs/semana | A) método documentado (pypistats API `https://pypistats.org/api/packages/<pkg>/recent` + CLI `pypistats`) + baseline 0 (404 = no publicados) / B) script propio / C) número inventado | A | ✅ decidido-por-evidencia: contrato pide "documentada con método… baseline 0"; API verificada live (shape `{"data":{"last_day",…,"last_week",…}}`); B = scope creep (no hay qué medir); C viola Regla 11 |
| 6 | Entrada COMPARISON.md | A) §9 "Track note — agent frameworks" cualitativa con links (patrón §7/§8) / B) fila en tabla §1 | A | ✅ decidido-por-evidencia: §1 compara vector DBs; §7/§8 ya establecieron el patrón de sección por track; cooperación de wave declarada en ICP-01 (083) |
| 7 | Workflow demo | A) `ci-frameworks-demo.yml` nuevo (ubuntu-only, PR-blocking, pins SHA, timeout, permissions read, concurrency) + step pins gate `test_pins.py` / B) sumar a `ci-examples.yml` | A | ✅ decidido-por-evidencia: `ci-examples` declara "examples importan solo vantadb_py + stdlib" (comentario :78-83) y sus steps son por archivo — la demo necesita langgraph; patrón `ci-ai-ides-demo.yml`/`icp02-privacy-demo.yml`; ubuntu-only como ICP-01 (no es comportamiento platform-specific) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) la demo es **determinista y offline** — sin LLM, sin tokens, sin red externa (loopback tampoco); el nodo del grafo es Python puro. (2) NO tocar pyproject/integrations prod (pins = lane owner) ni `src/sdk/importers/**` ni `vanta-proxy/**` (co-batch). (3) NO tocar `README.md` raíz / `BENCHMARKS.md` / claims (DEF-06 en vuelo). (4) COMPARISON §9 sin números de performance ni claims de competidores (Regla 11). (5) Los gates docs no pueden empeorar: 0 links rotos nuevos, 0 violaciones schema, `gen-index --check` exit 0. (6) El workflow nuevo sin `continue-on-error` y con pins SHA.
- **Comandos de verificación:** `$env:PYTHONPATH="integrations\langchain"; python examples/langgraph_dev_to_prod/demo.py` (local; en CI el adapter va pip-instalado) · `actionlint .github/workflows/ci-frameworks-demo.yml` · `python -m pytest integrations/test_pins.py -q` · `node scripts/docs/check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check` · `node scripts/docs/check-doc-examples.mjs` · `pwsh scripts/validate-docs-coverage.ps1` · `npx markdownlint-cli2 <docs tocados>`.
- **Deuda pendiente:** pin refresh adapters (`vantadb-py<0.7.0`, `langchain-core<1`) — Task 41/owner · disposición PyPI → switch cuando MKT-18f publique (FIND propuesto) · `integrations/README.md` §"Suites + shim temporal" quedó **stale** (FIND-94 migrado en `880cd0f3`; shim ya no existe — verificar y proponer FIND) · `import vantadb_py` deprecation warning en adapters (pre-existente).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# ICP-03: Track frameworks` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ (S2…S7) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | F6 (anuncio) — wave F5.2 cierra con ICP-03 + DEF-06 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≈0 → positivo. Añade cobertura CI (nuevo workflow PR-blocking que no existía) + docs del track; no toca código de producción ni añade deps al workspace. Paga: cierra el cuadro de distribución del track (el gap "demo dedicada" del re-baseline Task 44) y documenta el finding de pins con evidencia nueva (`langgraph` 1.x exige `langchain-core>=1.4.7` — refuerza la decisión del owner en Task 41).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 5/5 verificable: one-pager + demo CI verde (falla si el código difiere) + disposición Python (Alpha documentada) + COMPARISON §9 + métrica con método/baseline · task file sync · gates docs verdes |
| **Commit** | 1 commit atómico `feat(icp03):`/`docs:` según diff final (lo ejecuta el LEAD — esta wave NO commitea) · verificación mecánica registrada · **review fresco P2-01 = LEAD (no self-review)** |
| **Release** | N/A justificado: docs + demo CI + workflow; sin superficie pública nueva (no participa del contrato de release). El switch a PyPI del one-pager/demo se actualiza post-`adapters-v*` (FIND) |

## Herramientas necesarias
- `python` local (demo: driver + 5 procesos hijo) · `pytest` (`integrations/test_pins.py`)
- `actionlint` (lint del workflow nuevo) · `npx markdownlint-cli2` (docs tocados)
- `node scripts/docs/{check-links,check-docs,gen-index,check-doc-examples}.mjs` · `pwsh scripts/validate-docs-coverage.ps1`
- CodeGraph/CBM: usados en discovery (codegraph no cubre Python `integrations/` con esta granularidad → read directo verificado con evidencia de código)

**Skills cargadas (SDP v3 + obligatorias del área, 2026-09-30):** `documentation-skill` · `ci-cd-and-automation` (pinned) · `git-workflow-and-versioning` (pinned) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering`
- SDP v2 output: taskType "CI/CD / DevOps"; base: `campaign-executor`+`progreso` (auto vía MCP) + `doubt-driven-development`; pinned: `ci-cd-and-automation`, `git-workflow-and-versioning`.
- `documentation-skill` — obligatoria (docs/**: frontmatter, links GitHub-first, kind↔ruta).
- `source-driven-development` — verificar APIs (langgraph `StateGraph`/`InMemorySaver`, pypistats, PyPI 404) contra fuentes, no memoria.
- `doubt-driven-development` — gate de duda en las decisiones no triviales de la demo (fingerprint, --no-deps, controles negativos). Contexto subagente: ciclo degradado + review fresco P2-01 por LEAD.
- `ci-cd-and-automation` + `git-workflow-and-versioning` (pinned) — workflow nuevo bajo `RULES.md` (pins SHA, timeout, permissions, sin silencios).
- `incremental-implementation` / `test-driven-development` / `context-engineering` — slices verticales; la demo es su propio test ejecutable.

## Investigation Notes

- **Paso 0 (re-baseline verificado 2026-09-30):**
  - **Importadores ✅** (VER-05 `23c74a52`): `src/sdk/importers/{mem0,zep,letta}.rs` + `docs/api/MEMORY_INTERCHANGE_FORMAT.md` + 3 guías `docs/user/tutorials/migrating-from-{mem0,zep,letta}.md` (solo LEER para citar).
  - **Adapters NO publicados:** 9/9 → 404 (2026-09-29, MKT-18f); re-verificado hoy: `https://pypi.org/pypi/vantadb-langchain/json` → **404**. `vantadb-py` 0.7.0 live (200). Stop condition activa.
  - **Smokes de frameworks ya existen:** `ci-examples.yml:127-146` corre `langgraph_checkpoint.py`, `crewai_memory.py`, `dspy_retriever.py`, `haystack_documentstore.py`, `mem0_integration.py`, `semantic_kernel_memory.py` + instala el adapter langchain in-repo (:117-123) — pero NO hay demo dedicada dev→prod (gap del plan).
  - **Checkpointer real:** `integrations/langchain/vantadb_langchain/checkpointer.py` (`VantaDBCheckpointer`, `BaseCheckpointSaver`, namespaces `langgraph.checkpoints`/`langgraph.writes`). Probe local (2026-09-30): funciona con langgraph 1.2.11 + langchain-core 1.6.2 + langgraph-checkpoint 4.2.0 (entorno global del dev). Suite del adapter: **47 passed**.
  - **Pin stale (finding compartido Task 41):** `integrations/langchain/pyproject.toml` declara `vantadb-py>=0.5.0,<0.7.0` (excluye core 0.7.0) y `langchain-core>=0.3,<1` (excluye langchain-core 1.x que **langgraph 1.x exige ≥1.4.7** — evidencia `importlib.metadata.requires('langgraph')`). Decisión de bump = owner (Task 41/MKT-18f); mi demo instala `--no-deps` + `langgraph>=1,<2` explícito.
  - **FIND-94 migrado (`880cd0f3`)**: adapters prod usan `vanta.Client` + `memory.*`; shim eliminado. **`integrations/README.md` §"Suites + shim temporal" quedó stale** (describe shim/FIND-94 pendiente) → FIND candidato (doc-drift).
  - **FIND-84 pins gate:** `integrations/test_pins.py` (10 tests, offline) — pasa; `vantadb-py` exenta por techo propio.
  - **Entorno local para verificar la demo:** Python 3.14 + langgraph 1.2.11 + langchain-core 1.6.2 + `vantadb_py` editable (workspace) → `PYTHONPATH=integrations\langchain` alcanza para correr la demo sin instalar nada.
  - **Métrica — método validado:** `https://pypistats.org/api/packages/vantadb-py/recent` → `{"data":{"last_day":N,"last_month":N,"last_week":N},"package":…,"type":"recent_downloads"}` (fetch live OK). Baseline adapters = 0 (404).
- **Gap del track:** falta one-pager + demo dedicada + entrada COMPARISON + métrica (VER-05 ✅ y smokes ya cubiertos).
- **Naming/co-batch:** NO tocar `README.md` raíz/BENCHMARKS/claims (DEF-06), `vanta-proxy/**`, `src/sdk/importers/**` (solo LEER). `COMPARISON.md` cooperado (entrada al final; §7 ICP-01, §8 ICP-02 → ICP-03 = §9).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — publish = stop condition aplicada; sin deps externas nuevas |
| Pendientes de ejecución (downhill) | 0 — 7/7 steps ✅ + batch post-review ✅ (proceso: commit = LEAD) |
| % completado | 100% implementación/verificación |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado (cierre): sin input de usuario nuevo ni trust boundaries; la demo no abre red (ni loopback) y no toca secretos; el workflow usa `permissions: contents: read` + pins SHA de terceros + `--no-deps` (no re-resuelve deps fuera de lo declarado); sin deps nuevas del workspace. Checklist `security-and-hardening` aplicable: satisfecho (sin shell de usuario, sin logging de secretos, sin endpoints).
- [x] **PERFORMANCE** — evaluado: sin hot paths, sin claims numéricos (Regla 9/11 no disparan); demo local ~15s (5 procesos Python cortos); sin cambios de algoritmo.

## Steps

### Step 1: DISCOVERY + task file + Regla 0
- **Archivos:** `docs/dev/tasks/ICP-03.md` (nuevo)
- **Acción:** plan Task 44 leído completo; re-baseline (importadores ✅, adapters 404, smokes existentes, pin stale, entorno local); Gate D no disparado (contrato claro del plan, sin símbolos públicos nuevos, blast radius acotado); Spec table llena; Regla 0 mapeada.
- **Verify:** task file creado; `campaign_discover_skills_v2` ejecutado (taskType CI/CD/DevOps) ✅.
- **Estado:** ✅ COMPLETED (2026-09-30)

### Step 2: Demo `examples/langgraph_dev_to_prod/demo.py` + verificación local
- **Archivos:** `examples/langgraph_dev_to_prod/demo.py` (nuevo, driver + `_child`)
- **Acción:** sección negocio (`respond` + `build_graph`) backend-free; wiring (`make_checkpointer` dev/prod); fingerprint mecánico; driver: dev-first → dev-resume (control: NO recuerda) → prod-first (equivalencia de reply con dev) → prod-resume (SÍ recuerda, cross-proceso) → prod-other-thread (aislamiento); exit 1 con mensaje si cualquier propiedad falla.
- **Verify:** `$env:PYTHONPATH="integrations\langchain"; python examples/langgraph_dev_to_prod/demo.py` → **DEMO PASSED, 5/5 procesos, 6/6 checks `[ok]`, exit 0** ✅ · **Control negativo ejecutado**: copia con `_ = "dev"` dentro de `respond` → **exit 1** (`business code references ['dev'] — it must stay backend/mode-free`) ✅ · Entorno: Python 3.14 + langgraph 1.2.11 + langchain-core 1.6.2 + langgraph-checkpoint 4.2.0 + `vantadb_py` editable (develop).
- **Re-verify (ronda 1 review — Opt 1 + Nit 2):** wording corregido: fingerprint = *same-file display invariant* (no gate), gates reales = token assert backend-free (con control negativo) + equivalencia `dev-first == prod-first` (docstring + comentario + check label renombrado a "all 5 processes ran the same script (fingerprint — display invariant)"); `mode/phase/thread` del child documentados como contexto del CHILD_RESULT. Re-corrido: **DEMO PASSED 6/6, exit 0** + control negativo **exit 1** ✅.
- **Estado:** ✅ COMPLETED (2026-09-30)

### Step 3: README del demo + workflow `ci-frameworks-demo.yml`
- **Archivos:** `examples/langgraph_dev_to_prod/README.md` (nuevo) · `.github/workflows/ci-frameworks-demo.yml` (nuevo)
- **Acción:** README corto (qué prueba, cómo correr, límites); workflow PR-blocking (push [main, develop] + PR [main], paths: demo/integrations/langchain/test_pins/vantadb-python/src/Cargo/workflow), ubuntu, wheel maturin + `langgraph>=1,<2` + adapter `--no-deps` + demo + pins gate, timeout 30, pins SHA, permissions read, concurrency, sin `continue-on-error`.
- **Verify:** `actionlint .github/workflows/ci-frameworks-demo.yml` → **exit 0** ✅ · `python -m pytest integrations/test_pins.py -q` → **10 passed** ✅ · reglas `RULES.md` chequeadas (pins SHA + timeout + permissions + concurrency + sin silencios) ✅.
- **Re-verify (ronda 1 review — Nit SDK):** README del demo con `python -m pip install vantadb-py` explícito (paso 1/3 del run-block; antes solo en prosa) + expected tail actualizado al label nuevo del demo.
- **Estado:** ✅ COMPLETED (2026-09-30)

### Step 4: One-pager `docs/user/FRAMEWORKS.md`
- **Archivos:** `docs/user/FRAMEWORKS.md` (nuevo)
- **Acción:** problema (gap dev InMemorySaver → prod PostgresSaver) → valor (mismo código; embedded; namespaces multi-usuario) → instalación (Alpha local hoy; PyPI post-`adapters-v*`; referencia `release-adapters.yml`) → demo dev→prod (link + comando) → migración Mem0/Zep/Letta (links guías VER-05) → métrica installs/semana (método pypistats + baseline 0) → límites honestos (no PyPI; pins stale con evidencia; testing status).
- **Verify:** desc frontmatter 190 chars (≤200) ✅ · check-links (0 nuevos; 45/58) + check-docs GATING all clear ✅ · check-doc-examples dentro de budget (0 entradas nuevas) ✅ · **9/9 nombres PyPI → 404 re-verificado live hoy** (loop `Invoke-WebRequest`; + `vantadb-langchain` 404 vía webfetch) ✅ · **método pypistats validado live** (`https://pypistats.org/api/packages/vantadb-py/recent` → `{"data":{"last_day","last_week","last_month"}}`) ✅.
- **Re-verify (ronda 1 review — Opt 2 + Nit SDK):** baseline aclarado — "0 for the adapters" (el core `vantadb-py` se publica y se mide aparte; nadie lee 0 como "todo el proyecto"); bloque `--no-deps` con `python -m pip install vantadb-py` explícito (el SDK 0.7.0 de PyPI tiene la API que usa el demo: `Client(db_path, memory_limit_bytes, read_only, backend)` + `memory.list(cursor=…)→next_cursor`, verificado en el tag `v0.7.0`).
- **Estado:** ✅ COMPLETED (2026-09-30)

### Step 5: Entrada `COMPARISON.md` §9
- **Archivos:** `docs/user/COMPARISON.md` (edición quirúrgica al final)
- **Acción:** sección "§9 Agent frameworks — same code in dev and prod (ICP-03)" (cualitativa, links a FRAMEWORKS.md/demo/workflow/tutorials; sin números ni claims de competidores — Regla 11; mismo patrón §7/§8).
- **Verify:** check-links OK (0 nuevos) ✅ · check-docs GATING all clear ✅ · markdownlint 0 issues ✅.
- **Estado:** ✅ COMPLETED (2026-09-30)

### Step 6: Gates docs (regenerados + validaciones)
- **Archivos:** `docs/user/index.md` + `docs/index.md` + `llms.txt` (regenerados por `gen-index --write`)
- **Acción:** correr la secuencia de gates docs y arreglar lo que emerja.
- **Verify:** `gen-index --write` + `--check` → **exit 0** ✅ (FRAMEWORKS.md indexado en docs/index.md + docs/user/index.md + llms.txt) · `validate-docs-coverage` → **0 gaps** ✅ · `check-doc-examples` → within budget (1/1, pre-existente `blog/ollama_vantadb_local_memory.md`; 0 nuevos) ✅ · `markdownlint` → **0 issues** en FRAMEWORKS/COMPARISON/ICP-03 ✅ · `check-links` final → 45/58 (0 nuevos) ✅ · `check-docs` final → GATING all clear ✅.
- **Estado:** ✅ COMPLETED (2026-09-30)

### Step 7: Cierre — evidencia por cláusula + FINDs + RESULTADO
- **Archivos:** `docs/dev/tasks/ICP-03.md` (sync final)
- **Acción:** evidencia por cláusula en §Notas; FINDs propuestos (switch PyPI; pin refresh compartido Task 41; README stale shim); recitation; RESULTADO §7. NO commit · NO self-review (LEAD).
- **Verify:** comandos registrados + task file sync ✅.
- **Estado:** ✅ COMPLETED (2026-09-30) — proceso pendiente: review fresco P2-01 + commit local = LEAD

## Dependencias
- **MKT-18f ⬜** (publish = lane owner; local ✅ `3815afa3`): stop condition → disposición Alpha + FIND switch. NO publicar desde esta task.
- **VER-05 ✅** (`23c74a52`, importadores + guías) · **INTG-01 ✅** (`d7281744`, checkpointer/store) · **INTG-02 ✅** (`876df446`) · **FIND-84** (gate pins) · **VER-03/04/02** ✅ (F4, otras verticales) · **ci-examples.yml** (smokes ya verdes).
- Downstream: F6 anuncio (vertical 2) · DEF-06 (co-batch; COMPARISON coopera) · post-`adapters-v*` (switch PyPI del one-pager/demo).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED. **Esta wave: el LEAD coordina review fresco (no self-review).**

- **Revisor:** review fresco P2-01 (LEAD) — ronda 1 completada.
- **Enfoque:** ¿la demo prueba "mismo código" de verdad (fingerprint + controles negativos, no keyword disfrazado)? ¿el workflow falla si la demo falla (sin silencios)? ¿la disposición Alpha es honesta (404 verificable) sin sobre-prometer? ¿COMPARISON sin números (Regla 11)? ¿la métrica no inventa números?
- **Input del implementador (advisory, no sustituye review):** control negativo ejecutado (exit 1 con mensaje); display de los 5 procesos con fingerprint idéntico; 9/9 404 live; método pypistats validado live; gates docs verdes.
- **Ronda 1 (review fresco, 2026-09-30):** ✅ **APPROVE** — 0 Critical/Required; 2 Optional + 3 Nits. El owner exigió arreglar los hallazgos accionables → batch aplicado y re-verificado (§Notas).
- **Veredicto:** ✅ APPROVE (ronda 1) — batch de opcionales/nits aplicado; pendiente solo el commit local (LEAD)

## Notas
- **Evidencia del contrato (cláusula → verificación):**
  1. **One-pager** → `docs/user/FRAMEWORKS.md` (indexado en `docs/index.md` + `docs/user/index.md` + `llms.txt` regenerados; `gen-index --check` exit 0; desc 190/200).
  2. **Demo CI** → `examples/langgraph_dev_to_prod/demo.py` (5 procesos frescos; 6/6 checks; **DEMO PASSED exit 0** local) + `.github/workflows/ci-frameworks-demo.yml` (actionlint exit 0; PR-blocking; sin `continue-on-error`) + **control negativo ejecutado** (sección negocio con `"dev"` → exit 1). El "falla si el código de negocio difiere" se verifica por: fingerprint de fuente igual en todos los procesos + assert de tokens prohibidos en la sección negocio + equivalencia de comportamiento dev-vs-prod + control dev (no persiste) + control aislamiento de threads.
  3. **Instalación de adapters / disposición Alpha** → MKT-18f ⬜ = stop condition aplicada: **9/9 nombres → 404 re-verificados live 2026-09-30**; instalación local documentada y usada en demo/CI (adapter `--no-deps` por pins stale); FIND propuesto para el switch a PyPI post-`adapters-v*`.
  4. **COMPARISON.md** → §9 "Agent frameworks — same code in dev and prod (ICP-03)" (cualitativa; links; sin números — Regla 11).
  5. **Métrica** → `FRAMEWORKS.md` §Adoption metric: método pypistats (API URL + shape verificado live + CLI) y baseline 0 hasta publicar; sin números inventados.
- **FINDs propuestos (los registra el LEAD/orquestador en Backlog — esta task no edita `Backlog.md`):**
  1. **Switch PyPI del track frameworks (ICP-03):** cuando `adapters-v*` publique, el demo/workflow/one-pager pasan de instalación local + `--no-deps` a `pip install vantadb-langchain` (+ quitar la nota Alpha). Trigger: tag `adapters-v*` (checklist MKT-18f paso 3).
  2. **Pin refresh adapters (compartido con Task 41, evidencia nueva):** `langchain-core>=0.3,<1` no admite langchain-core 1.x que **langgraph 1.x exige (≥1.4.7)**; `vantadb-py>=0.5.0,<0.7.0` excluye el core vigente 0.7.0 → instalación limpia desde fuente requiere `--no-deps` (documentado en demo/CI/one-pager). Decisión de bump = owner pre-tag.
  3. **`integrations/README.md` stale:** la sección "Suites + shim temporal (…pendiente FIND-94)" describe un shim y una migración ya cerrados (FIND-94 migrado en `880cd0f3`; `vantadb_test_shim.py` no existe). Corregir en barrido docs o en el PR del primer release de adapters.
  - *Nota:* el alias deprecado `vantadb_py` que usan los adapters ya está trackeado en `FIND-118` (no se duplica).
- **Doubt (degradado — subagente, sin reviewer fresco anidado):** revisadas las decisiones no triviales contra el contrato: (a) fingerprint tapa forks realistas dev/prod; ceiling documentado en `demo.py` (`ponytail:` en `business_fingerprint`); (b) control negativo ejecutado para el assert de sección negocio; (c) la cláusula PyPI se resolvió por stop condition (no intento de publish); (d) instalación CI anti-frágil (`--no-deps` + versiones explícitas) con el why en el header del workflow. **Review fresco P2-01 = LEAD (no self-review).**
- **Batch post-review (ronda 1 — ✅ APPROVE, 2 Optional + 3 Nits; el owner exigió arreglar los hallazgos accionables):**
  1. **Opt 1 (wording):** fingerprint re-etiquetado como *same-file display invariant* (no puede fallar: los 5 hijos ejecutan el mismo archivo); los gates reales = token assert backend-free (con control negativo) + equivalencia de comportamiento `dev-first == prod-first`. Corregido en docstring + comentario in-code + check label (`demo.py`) y en tabla/prosa del README del demo.
  2. **Opt 2 (nota):** `FRAMEWORKS.md` §Adoption metric aclara que el 0 es de los adapters y que el core `vantadb-py` se publica y se mide aparte (nadie lee 0 como "todo el proyecto").
  3. **Nit SDK:** `python -m pip install vantadb-py` explícito en el run-block del README del demo y en el bloque `--no-deps` de FRAMEWORKS (verificado que el wheel 0.7.0 de PyPI tiene la API usada: `Client(db_path, memory_limit_bytes, read_only, backend)` + `memory.list(cursor=…)→next_cursor` → tag `v0.7.0`).
  4. **Nit 2:** campos `mode/phase/thread` del child documentados como contexto del `CHILD_RESULT` (se mantienen — útiles para debug de corridas crudas).
  5. **Extra de consistencia (no listado en el batch):** fila del check 3 ("Fresh runs see the request they just wrote") añadida a la tabla del README del demo — la tabla listaba 5 filas y el demo imprime 6 checks.
  - **Re-verify del batch:** demo → **DEMO PASSED exit 0** (6/6 `[ok]`) · control negativo → **exit 1** · markdownlint (FRAMEWORKS + README demo + ICP-03) → **0 issues** · check-docs → **GATING all clear** · check-links → **45/58, 0 nuevos** · gen-index --check → **exit 0**. ✅
- **Wave note:** `COMPARISON.md` §9 cooperó con la wave (patrón §7/§8); los co-batch DEF-06 mantienen sus regiones (README raíz/BENCHMARKS/claims solo leídos). `gen-index --write` regeneró índices compartidos incluyendo el task file de DEF-06 (co-batch, el LEAD commitea el conjunto).
- Creado por vanta-docs (subagente) desde el master roadmap — wave F5.2, co-batch con DEF-06.
