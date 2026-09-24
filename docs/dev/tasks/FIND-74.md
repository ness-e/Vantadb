# FIND-74 — requirements + enlaces + decisión TS

> Campaign: `6ab26f3f-cf16-4416-9255-c18cca0bcaf0` · Plan: `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 25, Wave8)
> Estado: ⬜ PENDING → IN PROGRESS (post-Task 16 FIND-67 por QUICKSTART compartido — YA cumplido b5d9b3f8, no pisarlo)
> Appetite: 4h · Esfuerzo: 🟢 · Prioridad: 🟢 · Ruta: vanta-docs
> Branch: `develop` · Commit previsto: `docs: FIND-74 — ...` (solo archivos propios)
> nextTask: FIND-86/72 paralelos Wave8, luego Wave9 (orquestador decide)
> Sin símbolos públicos nuevos → sin Spec técnica (solo docs, tabla Spec excepción docs-only abajo).

## 1. TAREA: objetivo + contrato exacto + acceptance criteria

**Objetivo:** Fijar ejemplo instalable roto por deriva de versión + enlazar ejemplos desde puertas de entrada + cerrar decisión TS sin mover archivos.

**Contrato exacto (plan Task 25:369-374):**
- `vantadb-py>=0.5.0` en `examples/demo/requirements.txt` (hoy `>=0.4`, deriva confirmada)
- 1 línea QUICKSTART→examples en `docs/user/QUICKSTART.md` (hoy 0 hits `examples|demo|colab`, verificado)
- 1 línea README→demo/colab en `README.md` (hoy Quick Links sin fila examples, verificado)
- Decisión TS documentada: 0 `.ts` en `examples/` confirmado; referenciar es salida válida, no mover ejemplos (pre-mortem plan: mover rompe links)

**Acceptance criteria:**
- `Get-Content examples/demo/requirements.txt` primera línea = `vantadb-py>=0.5.0`
- `Select-String -Path docs/user/QUICKSTART.md -Pattern "examples/README"` ≥1 hit (1 línea añadida, 6 hunks FIND-67 intactos)
- `Select-String -Path README.md -Pattern "examples/README"` ≥1 hit (1 línea añadida)
- Decisión TS con evidencia: conteo `.ts` en `examples/` = 0 + `vantadb-ts/examples/` existe con 3 `.mjs` + `examples/README.md:4,34-40` ya referencia (no mover)
- `git diff --check` limpio; `git status` solo archivos propios; commit `docs:` con task ID

**Deriva de versión (Regla 11 — cada versión con fuente):**
- Tag canónico: `v0.5.0` (fuente: `git tag --list` → `v0.5.0` único; `git log --oneline -15` muestra `b5d9b3f8 docs: FIND-67` ya en historia)
- Workspace: `Cargo.toml [workspace.package] version = "0.5.0"` (fuente: `Get-Content Cargo.toml | Select-String version` + `git show v0.5.0:Cargo.toml`)
- Python SDK: `vantadb-python/pyproject.toml: version = "0.5.0"`, `name = "vantadb-py"`, `requires-python >=3.11` (fuente: lectura directa)
- Demo ya migrado a 0.5.0 en docs: `examples/demo/README.md:20` (`vantadb-py ≥ 0.5.0`) + `:27` (`pip install "vantadb-py>=0.5.0"`) → `requirements.txt:1` (`>=0.4`) es el outlier
- Colateral NOTICED (no se toca): `examples/demo/demo.py:17` docstring `Requires: vantadb-py>=0.4` — misma deriva, archivo fuera de Archivos clave del plan → Gate C: ticket FIND futuro, no scope-creep

## 2. ARCHIVOS: clave / relacionados / prohibidos

**Clave (con :línea — leídos completos antes de editar, Regla 0):**
- `examples/demo/requirements.txt:1` — `vantadb-py>=0.4` → `>=0.5.0` (4 líneas totales, leído entero)
- `examples/README.md:1-50` — leído entero (50 líneas); `:4` ya dice `TS viven en vantadb-ts/examples/` + `:5` link QUICKSTART + `:29-30` demo/colab + `:34-40` tabla TS; NO requiere edición (decisión ya documentada, Ponytail 0 líneas)
- `docs/user/QUICKSTART.md:1-214` — leído entero (214 líneas, post-FIND-67 `b5d9b3f8` con 6 hunks: `:6` fecha, `:12` v0.5.0, `:90` wheel, `:142-144` search, `:179-183` audit nota, `:195` VANTADB_*); añadir 1 línea SIN revertir hunks (append al final, no tocar hunks)
- `README.md:55-70,104-138` — leído Quick Links + 5-Minute Quickstart; `:60` ya enlaza QUICKSTART; sin fila examples → añadir 1 fila

**Relacionados (callers/callees vía lectura directa + rg):**
- Callers de `examples/README.md`: `examples/demo/README.md:33,44` (rutas demo propias), plan file `:370,:424` (cita FIND-74), `docs/user/QUICKSTART.md:5` (link inverso ya existe desde examples → QUICKSTART, falta QUICKSTART → examples)
- Callers de QUICKSTART: `README.md:60` (`[5-Minute Quickstart](docs/user/QUICKSTART.md)`), `examples/README.md:5` (`[QUICKSTART](../../user/QUICKSTART.md)`), `SUPPORT.md:11`, plan FIND-67/FIND-74
- Callees: `vantadb-ts/examples/` (3 `.mjs` + 3 subdirs: `langchain/`, `llamaindex/`, `vercel-ai/`) — destino TS referenciado, no tocado
- `Get-ChildItem -Recurse -Filter *.md | Select-String "QUICKSTART"` → callers conocidos (README, README_ES, SUPPORT, AGENTS, skills) — ningún import de código depende del .md; cambio texto no rompe build
- `Get-ChildItem -Recurse -Filter *.md | Select-String "examples/demo"` → solo `examples/demo/README.md:33,44` + plans + tasks legacy — sin links frágiles que romper

**Prohibidos (WIP ajeno que NO se toca — verificado `git status --short` + `git stash list`):**
- `.opencode` (M submodule), `Justfile` (M), `completions/_vanta-cli*` (M ×3), `desktop/src-tauri/Cargo.lock` (M), `docs/pipeline-state.json` (M) — todos M en worktree ajeno, NO `git add`
- Untracked ajenos: `.github/workflows/ocr-delegate.yml`, `dev-tools/ocr-review.ps1`, `reparacion.bat`, `docs/dev/plans/2026-09-15-find-correcciones.md` (plan file, solo recitation del orquestador), `docs/dev/plans/archive/*.budget.json`
- `stash@{0..14}` (15 stashes, verificado `git stash list`) — no pop/drop
- Archivos FIND-86 (`vanta-memory/`) y FIND-72 (`benchmarks/`) — paralelos Wave8, disjuntos, no colisionan
- `examples/demo/demo.py:17` — deriva gemela NOTICED, fuera de contrato, no se toca (Gate C → FIND futuro)
- `examples/demo/__pycache__/`, `examples/python/__pycache__/` — artefactos, no commitear

## 3. DEPENDENCIAS: waves + orden + paralelas

- Wave8 (plan `:424`): `FIND-74 + FIND-86 + FIND-72` disjuntos por archivo (74 docs/examples, 86 `vanta-memory/`, 72 `benchmarks/`) → paralelizables, no colisionan
- Orden plan: post-Task 16 FIND-67 por QUICKSTART compartido — YA cumplido (`b5d9b3f8` en `git log`, 6 hunks verificados con `git show b5d9b3f8 -- docs/user/QUICKSTART.md`); no pisarlo (append-only en QUICKSTART)
- Previa: FIND-80 ✅ (`e3260652` head actual), Wave0-7 DONE 24/30 (dado contexto tarea: Wave0-7 DONE)
- Next: FIND-86/72 paralelos (orquestador), luego Wave9
- Sin dependencias bloqueantes (plan: `Sin dependencias bloqueantes`)

## 4. REFERENCIAS: fuentes con Regla 11

- Tag `v0.5.0`: `git tag --list` → `v0.5.0` (único); `git show v0.5.0:examples/demo/requirements.txt` → `vantadb-py>=0.4` (deriva ya en tag, fix post-tag en develop)
- FIND-67: commit `b5d9b3f8 docs: FIND-67 — QUICKSTART a 0.5.0` (verificado `git show --stat` + `git show -- docs/user/QUICKSTART.md` 6 hunks); QUICKSTART actual `last_reviewed: 2026-09-15`, `:12` v0.5.0, `:90` wheel 0.5.0, `:195` VANTADB_* — no revertir
- `examples/README.md` existente: 50 líneas, `:4` TS fuera del árbol + `:34-40` tabla TS + `:29-30` demo/colab (fuente: lectura directa completa)
- `Cargo.toml [workspace.package] version = "0.5.0"` (fuente: `Get-Content Cargo.toml | Select-String version` + `git show v0.5.0:Cargo.toml`)
- `vantadb-python/pyproject.toml: name vantadb-py, version 0.5.0, requires-python >=3.11` (fuente: lectura directa)
- `examples/demo/README.md:20,27` ya en 0.5.0 (fuente: lectura directa) → requirements.txt outlier
- `vantadb-ts/examples/`: 3 `.mjs` (`langchain-rag.mjs`, `llamaindex-rag.mjs`, `vercel-ai-memory.mjs`) + 3 dirs (fuente: `Get-ChildItem vantadb-ts/examples`)
- 0 `.ts` en `examples/`: `(Get-ChildItem -Path examples -Recurse -Include *.ts -File | Measure).Count` = 0 (fuente: comando directo)
- Regla 11: cada versión/path arriba lleva fuente file:línea o comando; sin URLs inventadas. Si símbolo público nuevo → tabla Spec (no se espera, solo docs — confirmado 0 `pub fn`/tool/endpoint en diff previsto)

## 5. SKILLS: lista SDP (campaign_discover_skills_v2 phase=BUILD + keywords, ≤8)

Comando: `campaign_discover_skills_v2 archivosClave="examples/demo/requirements.txt, examples/README.md, docs/user/QUICKSTART.md, README.md" phase="BUILD" contractKeywords=["examples-requirements","version-drift","ts-decision","quickstart-link"] maxSkills=8` → 8 candidatas + 2 keyword-mapped (total 13, filtradas a 8 por score).

| Skill | Cuándo aplica (1 línea) |
|-------|-------------------------|
| `campaign-executor` (base, score 1.0) | Pipeline PLAN→ACT→VERIFY + task file + recitation en cada transición |
| `incremental-implementation` (lifecycle BUILD) | 1 step vertical delgado: requirements → QUICKSTART → README, cada edit verificado antes del siguiente |
| `test-driven-development` (lifecycle BUILD) | No lógica nueva, pero verify mecánico por edit (rg + diff-check) como Red→Green por archivo |
| `context-engineering` (lifecycle BUILD) | Sesión con 4 archivos + 6 hunks FIND-67 que no se pueden pisar — empaquetar solo hunks relevantes |
| `source-driven-development` (lifecycle BUILD) | Cada versión/path con fuente file:línea (Regla 11), no memoria del modelo |
| `doubt-driven-development` (lifecycle BUILD) | Descartada tras evaluar: sin stakes prod/seguridad (docs-only, 0 código) — registrada como no-aplica |
| `frontend-ui-engineering` (lifecycle BUILD) | Descartada: no toca `web/` — registrada como no-aplica |
| `api-and-interface-design` (lifecycle BUILD) | Descartada: sin API pública nueva — registrada como no-aplica |
| `documentation-and-adrs` (keyword-mapped) | Decisión TS mover-vs-referenciar con tradeoff + gotchas inline si aplican |
| `writing-guidelines` (keyword-mapped + detect type docs) | Voz/tono de las 2 líneas añadidas (conciso, sin reescritura) |

Cargadas en sesión: `documentation-and-adrs`, `writing-guidelines`, `writing-plans`, `spec-driven-development`, `source-driven-development`, `incremental-implementation` (+ base auto: `campaign-executor`, `progreso`, `ponytail full`).
`SDP: documentation-and-adrs + writing-guidelines + writing-plans + spec-driven-development + source-driven-development + incremental-implementation (+ base campaign-executor/progreso/ponytail; doubt/frontend-ui/api descartadas por docs-only)`
Tipo auto-detectado: `docs` (`campaign_detect_task_type` → skills `[writing-guidelines, writing-plans]`, checks `[scripts/validate-docs-coverage.ps1]`).

## 6. HERRAMIENTAS+MCP: comandos exactos

**Lectura/verificación (sin red, validación = lectura + diff):**
- `Get-Content examples/demo/requirements.txt` — deriva `:1`
- `Get-Content Cargo.toml | Select-String -Pattern 'version'` + `Get-Content vantadb-python/pyproject.toml | Select-String -Pattern 'version|name'` — fuentes versión
- `Get-ChildItem -Path examples -Recurse -Include *.ts -File | Measure-Object` — conteo TS (= 0)
- `Get-ChildItem vantadb-ts/examples` — destino TS existe
- `Get-ChildItem examples -Recurse -File | Select-Object FullName` — árbol examples (sin `.ts`)
- `Select-String -Path README.md -Pattern "QUICKSTART|examples/demo|colab"` + `Select-String -Path docs/user/QUICKSTART.md -Pattern "examples|demo|colab"` — gaps de enlace (= 0 antes del fix)
- `git diff --check` — whitespace errors (= 0 exigido)
- `git status --short` — solo archivos propios al commitear
- `git show b5d9b3f8 -- docs/user/QUICKSTART.md` — 6 hunks FIND-67 que no se tocan
- `Test-Path` enlaces: `Test-Path examples/README.md`, `Test-Path examples/demo/demo.py`, `Test-Path examples/colab/vantadb_quickstart.ipynb`, `Test-Path vantadb-ts/examples/langchain-rag.mjs` — todos True exigido
- `pip install --dry-run` NO — sin red por instrucción tarea; validación = lectura + diff (requirements es pin floor `>=`, no lock; CI `ci-examples-12.yml` instala real)
- `scripts/validate-docs-coverage.ps1` si aplica (check tipo docs según `campaign_detect_task_type`)

**MCP:**
- `campaign_detect_task_type` ✅ usado (tipo `docs`) — evidencia arriba §5
- `campaign_discover_skills_v2` ✅ usado (8 skills + justificaciones) — evidencia §5
- `campaign_verify_cmd` — bug exit -1 conocido (plan Riesgos globales) → fallback bash directa y anotarlo en RESULTADO (precedente FIND-63/FIND-67)
- `campaign_update_task_state` — `in-progress` antes de ACT, `completed` al cierre con recitation canónica 6 claves
- `campaign_emit_event` — solo si cambia dirección (Gate D/V/C disparado; no se espera)
- `codegraph_explore` — solo si indexa docs; docs `.md` no son símbolos → grep directo aplicado (justificado: grafo para código, no para 4 archivos docs)
- `cargo` con `-j 2` si tocara Rust — no debe (docs-only, 0 líneas Rust)

## 7. INVESTIGACIÓN CÓDIGO: blast radius (descubierto en DISCOVERY arriba)

- Grafo: `requirements.txt:1` → `examples/demo/README.md:20,27` (ya 0.5.0, outlier confirmado) → `examples/demo/demo.py:17` (colateral NOTICED, no tocado) → `examples/README.md:16-32` (tabla Python, sin versión, no tocada) → `docs/user/QUICKSTART.md` (0 links a examples, gap) → `README.md` Quick Links (0 fila examples, gap)
- TS: `examples/` (0 `.ts`, 16 archivos: 1 README + 1 ipynb + 2 demo + 9 python + 3 rust + pycache) → `examples/README.md:4,34-40` (referencia a `vantadb-ts/examples/`) → `vantadb-ts/examples/` (3 `.mjs` + 3 dirs, existe) → decisión: referenciar (mover rompe links + CI `ci-examples-12.yml` + imports relativos `../vantadb-ts/examples/`)
- Implicaciones: no romper links (append-only, rutas relativas existentes intactas); riesgo único: pisar 6 hunks FIND-67 en QUICKSTART (mitigación: 1 línea al final, `git diff docs/user/QUICKSTART.md` debe mostrar +1/-0 en ese archivo)
- Riesgo colisión Wave8: FIND-86 (`vanta-memory/`) y FIND-72 (`benchmarks/`) disjuntos — verificado sin solapamiento de paths

## 8. INVESTIGACIÓN PROBLEMA: ejemplo instalable + TS sin hogar

- **Deriva `>=0.4` vs SDK 0.5.0:** `requirements.txt:1` permite instalar `vantadb-py 0.4.x` (floor 0.4, sin upper bound) mientras `examples/demo/README.md` documenta `>=0.5.0` y el SDK publica `0.5.0` (pyproject + workspace + tag). Instalación con floor viejo = APIs `search` vs `search_memory` (FIND-67 halló rename flat `*_memory` removidos) y `Client` vs `VantaDB` (ADR-041 anti-stutter, README raíz `:84-87`) pueden fallar según versión instalada. Fix: floor `>=0.5.0` (1 línea, sin pins lock — pins lock son para FIND-72/84, no aquí).
- **TS sin hogar (0 `.ts` confirmado):** `examples/` no contiene TypeScript por diseño (conteo 0 verificado); los ejemplos TS viven en `vantadb-ts/examples/` (3 `.mjs` + adapters). Tradeoff mover-vs-referenciar:
  - Mover (`examples/ts/` o copiar `.mjs`): Pros = un solo árbol; Contras = rompe links (`examples/README.md:4,34-40`, `vantadb-ts` package paths, CI `ci-examples-12.yml`, `TS_SDK.md`), duplica fuente de verdad, diverge en 1 mes → RECHAZADO (pre-mortem plan).
  - Referenciar (actual + ratificado): Pros = 0 links rotos, 1 fuente de verdad (`vantadb-ts/`), `examples/README.md` ya lo hace (`:4` + tabla `:34-40`); Contras = dos árboles que descubrir (mitigado con las 2 líneas de enlace QUICKSTART↔examples↔README) → ELEGIDO. Salida válida per plan.
- **Decisión escrita dónde:** `examples/README.md:4,34-40` ya la contiene → no requiere edición (Ponytail 0 líneas); esta sección del task file es el registro de la decisión con tradeoff (documentation-and-adrs: decidir por evidencia, no re-debatir en 6 meses).

## 9. INVESTIGACIÓN INTERNET: no se espera (todo local)

- Todo verificable en repo (tag, Cargo.toml, pyproject, árbol examples, `vantadb-ts/examples/`, FIND-67 commit). Sin `webfetch`/`websearch` necesario.
- Regla: si ambigüedad irresoluble en repo (p.ej. wheel path de Release sin evidencia CI) → marcar `[cita NO VERIFICADA]` y anotar en deuda (TSYS-13). No se espera: wheel no es parte de este contrato (era FIND-67, ya cerrado con evidencia CI).
- GATE CITAS (TSYS-13): esta tarea no produce evidencia con URLs citadas (0 URLs en contrato; links añadidos son relativos `../examples/README.md`, `examples/README.md`, no requieren HEAD).

## 10. VALIDACIÓN+CIERRE: verify contrato + full + OCR + DoD + Gates + commit

- **Verify contrato (mecánico, por archivo):** `Get-Content requirements.txt` (= `>=0.5.0`) + `Select-String "examples/README" QUICKSTART` (≥1) + `Select-String "examples/README" README` (≥1) + conteo `.ts` (= 0) + `Test-Path` 4 enlaces (= True) + `git diff --check` (= 0) + `git diff --stat` (solo propios: requirements + QUICKSTART + README + este task file)
- **Verify full relevante (docs-only, no Rust):** `git diff --check` + `scripts/validate-docs-coverage.ps1` (check tipo docs; si requiere red/CI pesado → documentar skip con motivo, no rojo falso) + `campaign_verify_cmd "git diff --check"` intentado (bug exit -1 → fallback bash anotado)
- **OCR delegation:** `pwsh dev-tools/ocr-review.ps1` si existe en disco (untracked ajeno — solo lectura/ejecución, NO `git add`); Critical/High = bloquea commit; Medium → fila FIND-*; Low se descarta. Si el script no corre (sin red/API) → advisory documentado, no bloquea docs-only (precedente FIND-67 self-review + hooks).
- **DoD 3 niveles:** Task (contrato 4 puntos + diff-check) / Commit (atómico `docs: FIND-74 — ...`, solo propios, hooks verdes) / Release (N/A docs — release-plz ignora `docs:`, sin semver/changelog)
- **Reviewer distinto P2-01:** self-review con checklist anti-hábitos + verificación mecánica; reviewer distinto al push vía vanta-lead (orquestador decide; precedente FIND-63/FIND-65)
- **Gates D/V/C activos:** D evaluado abajo (no disparado, blast radius 4 docs, sin símbolos nuevos); V solo si 2 fallas mismo-error (no se espera, edits de 1 línea); C con `git status` fuera de blast radius → dejar sin commit (WIP ajeno listado §2)
- **RESULTADO §7 obligatorio** al final de la invocación + **Context Save Point** abajo en este file
- **Commit conventional `docs:` con task ID (solo archivos propios):** `git add examples/demo/requirements.txt docs/user/QUICKSTART.md README.md docs/dev/tasks/FIND-74.md` + `git commit -m "docs: FIND-74 — ..."` (NO `.opencode/`, `Justfile`, `completions/`, `Cargo.lock`, `pipeline-state.json`, plan file, `demo.py`, stash)
- **Backlog→avance NO tocar** (race paralelo, orquestador) + **push vía vanta-lead**

## SDP

`campaign_discover_skills_v2 archivosClave="examples/demo/requirements.txt, examples/README.md, docs/user/QUICKSTART.md, README.md" phase="BUILD" contractKeywords=["examples-requirements","version-drift","ts-decision","quickstart-link"] maxSkills=8` → 8 skills (ver tabla §5).
`SDP: documentation-and-adrs + writing-guidelines + writing-plans + spec-driven-development + source-driven-development + incremental-implementation (+ base campaign-executor/progreso/ponytail; doubt/frontend-ui/api descartadas por docs-only)`
SKILLS_CARGADAS base sesión: campaign-executor, progreso, ponytail(full).

## Gate D (question-gates.md)

Blast radius = 4 archivos docs (efectivo: 3 editados + 1 verificado sin edición), ~3 líneas totales. Sin símbolos públicos nuevos (`pub fn`/tool/endpoint: no — 0 líneas código), sin hot path (`vector/`/`engine.rs`: no), sin API pública, contrato no ambiguo (4 puntos con línea exacta + conteo 0 verificado), docs-fix (no feature-add) → Gate D **no disparado**, sin `question` (harness sin `question` en este rol; si disparara → RESULTADO con BLOQUEO para orquestador). Gate spec mecánico: tipo `docs` → sin sección `## Spec` técnica requerida; tabla excepción docs-only abajo justifica. Familia aprobada en plan (Gate P 2026-09-15: 27 DO) suprime D individual para fix/docs sin superficie nueva (question-gates.md Anti-abuso).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `README.md:60` → QUICKSTART; `examples/README.md:5` → QUICKSTART; `examples/demo/README.md` (ya 0.5.0); plan Task 25 |
| Callees | `vantadb-ts/examples/` (3 `.mjs`, existe); `examples/colab/vantadb_quickstart.ipynb` (existe); `examples/demo/demo.py` (NOTICED, no tocado) |
| Implicaciones | Solo docs/texto. Prohibidos §2 intactos. QUICKSTART append-only (6 hunks FIND-67 intactos). Sin código Rust/Python. |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):** `examples/demo/requirements.txt` (4 líneas); `examples/README.md` (50 líneas); `docs/user/QUICKSTART.md` (214 líneas + `git show b5d9b3f8 -- docs/user/QUICKSTART.md` 6 hunks); `README.md` (349 líneas, foco `:55-70` Quick Links + `:104-138` Quickstart); `examples/demo/README.md` (79 líneas); `examples/demo/demo.py:1-20` (cabecera versión); `Cargo.toml` (workspace.package) + `vantadb-python/pyproject.toml` (versión); `git log --oneline -15` + `git tag --list` + `git status --short` + `git stash list` (WIP ajeno mapeado).
- **Archivos referenciados hacia dentro (lo que los editados citan):** `vantadb-py>=0.5.0` (PyPI/TestPyPI/source `vantadb-python/`); `examples/README.md` (demo + colab + TS tabla); `docs/user/QUICKSTART.md` (CLI + binding 0.5.0); `vantadb-ts/examples/*.mjs`.
- **Archivos que referencian a los editados (referencias entrantes):** `rg QUICKSTART` (README, README_ES, SUPPORT, AGENTS, skills) + `rg examples/demo` (demo README + plans/tasks) — ningún import de código depende de los .md; cambiar texto no rompe build. CI `ci-examples-12.yml` ejecuta ejemplos (floor `>=0.5.0` compatible con SDK 0.5.0 instalado desde fuente en CI).
- **Veredicto impacto:** mínimo. 3 archivos editados (~3 líneas) + 1 verificado sin edición + este task file. Riesgo único: pisar hunks FIND-67 → mitigado append-only + `git diff` revisado (+1/-0 en QUICKSTART). WIP ajeno (`git status` 7M + 4??) excluido del commit.

## Contrato

`vantadb-py>=0.5.0` en requirements + 1 línea QUICKSTART→examples + 1 línea README→demo/colab + decisión TS referenciar (0 `.ts` confirmado). Verificación mecánica §10.

## Spec (SDD — excepción docs-only, question-gates.md §"Contenido válido")

> Excepción única: tarea 100% docs/markdown sin decisiones técnicas abiertas → `sin decisiones técnicas` + lista de archivos tocados. La decisión TS mover-vs-referenciar se registra por evidencia abajo (un solo camino viable con tradeoff explícito §8).

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | TS mover-vs-referenciar | mover (un árbol, rompe links/CI) vs referenciar (0 rotos, 2 árboles por descubrir, mitigado con enlaces) | referenciar (pre-mortem plan) | ✅ decidido-por-evidencia (ref: `examples/README.md:4,34-40` + conteo 0 + `vantadb-ts/examples/` existe) |
| — | Sin decisiones técnicas abiertas en versiones/enlaces (floor + 2 líneas, evidencia file:línea §1/§4) | — | — | ✅ N/A-docs-only + 1 decisión registrada arriba |

Archivos tocados: `examples/demo/requirements.txt`, `docs/user/QUICKSTART.md`, `README.md` (+ este task file). `examples/README.md` verificado sin edición.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no tocar código Rust/Python (solo docs/texto); no tocar prohibidos §2 (WIP ajeno, stash, plan file, FIND-86/72); no revertir 6 hunks FIND-67 (QUICKSTART append-only); no mover ejemplos TS (referenciar); Regla 11: 0 claims sin fuente.
- **Comandos de verificación:** §6 + §10 (conteo 0 + Test-Path + diff-check + Select-String post-edit).
- **Deuda pendiente:** `examples/demo/demo.py:17` deriva gemela → Gate C (FIND futuro, no inline por archivo distinto); `§7 embed-local` con modelo real no revalidado (heredado FIND-67, fuera de scope).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda. Docs-only, ~3 líneas texto, no introduce deuda nueva. Colateral `demo.py:17` NOTICED (no scope-creep).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | 4 puntos §1 + `git diff --check` 0 + Test-Path 4/4 + conteo 0 + Select-String post-edit ≥1 en QUICKSTART y README |
| **Commit** | Atómico `docs: FIND-74 — ...`, solo 4 archivos propios (§10), hooks verdes |
| **Release** | N/A docs (release-plz ignora `docs:`; sin semver/changelog). Justificado: 0 código |

## Herramientas necesarias

- `campaign_detect_task_type` ✅, `campaign_discover_skills_v2` ✅, `campaign_update_task_state` (in-progress → completed), `campaign_verify_cmd` (intentar, bug -1 → fallback)
- Comandos §6 (lectura + diff, sin red). `cargo -j 2` no aplica (docs-only).

**Skills cargadas (SDP):** ver tabla §5. Lifecycle genéricas no-aplicables registradas con motivo (no omitidas en silencio).

## Investigation Notes

- **Claim:** `requirements.txt` deriva `>=0.4` vs SDK 0.5.0. **Evidencia:** `examples/demo/requirements.txt:1` (`vantadb-py>=0.4`) + `Cargo.toml [workspace.package] version = "0.5.0"` + `vantadb-python/pyproject.toml version = "0.5.0"` + `git tag --list` (`v0.5.0`) + `examples/demo/README.md:20,27` (ya `>=0.5.0`). **Confianza:** alta.
- **Claim:** QUICKSTART sin enlace a examples. **Evidencia:** `Select-String -Path docs/user/QUICKSTART.md -Pattern "examples|demo|colab"` = 0 hits. **Confianza:** alta.
- **Claim:** README sin fila examples. **Evidencia:** `Select-String -Path README.md -Pattern "examples/README"` = 0 hits; Quick Links `:55-70` sin examples. **Confianza:** alta.
- **Claim:** 0 `.ts` en `examples/`, TS vive en `vantadb-ts/examples/`. **Evidencia:** conteo = 0 + `Get-ChildItem vantadb-ts/examples` (3 `.mjs` + 3 dirs) + `examples/README.md:4,34-40` (referencia existente). **Confianza:** alta.
- **Claim:** FIND-67 intacto. **Evidencia:** `git show b5d9b3f8 --stat` (2 files) + 6 hunks en QUICKSTART + `git log --oneline` lo lista. **Confianza:** alta.
- **Claim:** Wave8 disjunta. **Evidencia:** paths 74 (examples/docs/README) vs 86 (`vanta-memory/`) vs 72 (`benchmarks/`) sin solapamiento + `git status` sin esos archivos modificados. **Confianza:** alta.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — evidencia completa, sin web research, decisión TS cerrada por pre-mortem plan |
| Pendientes de ejecución (downhill) | 0 — Step 1 ✅ (verify + self-review ✅, commit pendiente) |
| % completado | 100% (1/1 steps; commit + recitation + RESULTADO pendientes) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No aplica (docs-only, sin trust boundaries/input/auth/deps/storage/FFI/red; `requirements.txt` floor bump no es nueva dependencia ni bump lock — solo floor compatible con SDK ya publicado 0.5.0). Justificado: 0 líneas código.
- [x] **PERFORMANCE** — No aplica (sin hot paths: no `vector/`, `engine.rs`, search/ingestión, serialización). Justificado: docs-only.

## Steps

### Step 1: 3 edits mínimos + verify mecánico + review + commit (único step, ~100 líneas tocadas incluyendo task file)
- **Archivos:** `examples/demo/requirements.txt:1`, `docs/user/QUICKSTART.md` (append 1 línea final), `README.md` (1 fila Quick Links) + este task file (sync estados)
- **Acción (mínimo diff, sin reescritura, sin mover ejemplos):**
  1. `requirements.txt:1` `vantadb-py>=0.4` → `vantadb-py>=0.5.0` ( resto del file intacto, comentario sentence-transformers intacto).
  2. `docs/user/QUICKSTART.md` append final (post `EXPERIMENTAL_FEATURES.md` línea, sin tocar 6 hunks): `> For runnable examples beyond this quickstart, see [examples/](../../examples/README.md) (demo + Colab).`
  3. `README.md` Quick Links +1 fila tras `| Follow a tutorial | [Tutorials](docs/user/tutorials/) |`: `| Run runnable examples | [Demo + Colab](examples/README.md) |`
  4. TS: 0 ediciones en `examples/README.md` (ya `:4,34-40`); decisión registrada §8 de este file.
  5. NO tocar: `demo.py:17` (NOTICED → FIND futuro), prohibidos §2, plan file, Backlog/avance (orquestador).
- **Verify:** §10 (Select-String post-edit + Test-Path + conteo + diff-check + diff-stat solo propios) + `campaign_verify_cmd "git diff --check"` ✅ passed 1.1s (sin bug -1 esta vez) + `scripts/validate-docs-coverage.ps1` 0 gaps ✅ + OCR delegate `default` sin Critical/High ✅.
- **Estado:** ✅ DONE (2026-09-16: requirements `>=0.5.0` + QUICKSTART:216 + README:65 verificados; QUICKSTART diff +2/-0, hunks FIND-67 intactos; commit pendiente)

## Dependencias

- Previa: FIND-67 ✅ (`b5d9b3f8`), FIND-80 ✅ (`e3260652` head). Paralelas Wave8: FIND-86, FIND-72 (disjuntos). Next: Wave9 (orquestador).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** self-review (implementador) con checklist anti-hábitos + verificación mecánica abajo; reviewer distinto pendiente al push vía vanta-lead (orquestador decide; precedente FIND-63/FIND-65 cerró con self-review + hooks).
- **Enfoque:** ¿floor con fuente triple (tag v0.5.0 + workspace.package 0.5.0 + pyproject 0.5.0 + demo/README ya 0.5.0)? Sí. ¿QUICKSTART +2/-0 (6 hunks FIND-67 intactos, append final)? Sí (`git diff -- docs/user/QUICKSTART.md` solo +blank +blockquote). ¿README +1 fila Quick Links? Sí (`:65`). ¿TS 0 + referencia existente sin mover? Sí (conteo 0 + `vantadb-ts/examples/` 3 mjs + `examples/README.md:4,34-40`). ¿sin scope-creep (`demo.py:17` NOTICED intacto, prohibidos intactos)? Sí (`git diff --stat` propios solo 3 + task file; WIP ajeno excluido). ¿Regla 11 por claim? Sí (§1/§4/Investigation Notes).
- **Cómo se probó:** lectura + diff (sin red por instrucción tarea); `campaign_verify_cmd "git diff --check"` passed 1.1s; `scripts/validate-docs-coverage.ps1` 0 gaps; `ocr delegate rule` → Rule Group `system/default` (Correctness/Security/Performance/Maintainability/Test Coverage genéricos, sin hallazgos Critical/High en 1-liners docs); CI `ci-examples-12.yml` instala real (no `--dry-run` sin red).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve (self; distinto-revisor opcional en push vía vanta-lead)

## HALLAZGOS (para orquestador, Backlog NO tocado por race Wave8)

1. **NOTICED (no tocado):** `examples/demo/demo.py:17` docstring `Requires: vantadb-py>=0.4` — misma deriva que `requirements.txt:1`. Archivo fuera de Archivos clave del plan → Gate C: candidato FIND futuro (1 línea), no inline (archivo distinto).
2. **NOTICED (entorno):** `campaign_verify_cmd` bug exit -1 vacío (plan Riesgos) → fallback bash documentado en RESULTADO.
3. **Verificado sin edición:** `examples/README.md:4,34-40` ya satisface decisión TS (referenciar) — 0 líneas a ese archivo es salida válida Ponytail, no omisión.

## Notas

- Ponytail full: mínimo diff docs (~3 líneas repo + task file), sin reescritura, sin mover ejemplos, sin pins lock.
- Backlog→avance NO tocar (race paralelo, orquestador) + push vía vanta-lead.
- Regla 11: cada versión/path con fuente (§1/§4/Investigation Notes). Sin `[cita NO VERIFICADA]` (todo local verificado).
- Commit: `docs:` con ID, solo archivos propios (requirements + QUICKSTART + README + este task file).

## Referencias

- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles
- `.opencode/task-system/prompts/question-gates.md` — Gates D/V/C
- `docs/dev/plans/2026-09-15-find-correcciones.md` Task 25 — contrato y contexto verificado

## Context Save Point

- **Fecha:** 2026-09-16
- **Branch:** develop (head `e3260652`)
- **CI pendiente:** no (docs-only; validate-docs-coverage 0 gaps ✅; `campaign_verify_cmd git diff --check` passed 1.1s ✅; OCR delegate `default` sin bloqueos ✅; push vía vanta-lead)
- **Decisiones:** floor `>=0.5.0` con triple fuente; QUICKSTART append-only (+2/-0, hunks intactos); README +1 fila Quick Links (`:65`); TS referenciar (0 ediciones a `examples/README.md`); `demo.py:17` NOTICED no tocado
- **Problemas conocidos:** WIP ajeno 7M+4?? excluido del commit; stash@{0..14} intactos; `pip --dry-run` omitido sin red (validación lectura+diff per instrucción)
- **Próxima tarea:** commit atómico → `campaign_update_task_state completed` → RESULTADO §7 → orquestador (FIND-86/72, Backlog/avance, push vanta-lead)
