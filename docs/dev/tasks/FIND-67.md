# FIND-67 — QUICKSTART a 0.5.0 (+ legacy `WANTA_*`)

> Campaign: `6ab26f3f-cf16-4416-9255-c18cca0bcaf0` · Plan: `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 16, Wave5)
> Estado: ⬜ PENDING → IN PROGRESS (disjunto de FIND-68 PROXY.md y FIND-81 server/ en Wave5)
> Appetite: 4h · Esfuerzo: 🟡 · Prioridad: 🟠 · Ruta: vanta-docs
> Branch: `develop` · Commit previsto: `docs: FIND-67 — ...` (solo `docs/user/QUICKSTART.md` + este task file)
> nextTask: FIND-68 (Wave5, archivo disjunto)
> Sin símbolos públicos nuevos → sin Spec (solo docs, tabla Spec N/A justificada abajo).

## SDP

`campaign_discover_skills_v2 archivosClave="docs/user/QUICKSTART.md:6,12,90,188-189" phase="BUILD" contractKeywords=["quickstart-docs","version-boundary","env-var-rename","doc-revalidation"] maxSkills=8` →
base: campaign-executor, progreso (+ ponytail full auto) +
lifecycle: incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development (+ frontend-ui-engineering, api-and-interface-design descartadas por inaplicables a docs) +
keyword: documentation-and-adrs, writing-guidelines.
Cargadas: documentation-and-adrs, writing-guidelines, campaign-executor, progreso.
`SDP: documentation-and-adrs + writing-guidelines + campaign-executor + progreso (beyond base; frontend-ui/api descartadas por docs-only)`
SKILLS_CARGADAS base sesión: campaign-executor, progreso, ponytail(full).

## Gate D (question-gates.md)

Blast radius = 1 archivo docs (`docs/user/QUICKSTART.md`, 4 hunks). Sin símbolos públicos nuevos (`pub fn`/tool/endpoint: no), sin hot path, sin API pública, contrato no ambiguo (4 puntos con línea exacta), docs-fix (no feature-add) → Gate D **no disparado**, sin `question`. Gate spec mecánico: tipo auto-detectado `docs` (campaign_detect_task_type) → sin sección `## Spec` requerida; tabla N/A abajo justifica.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/user/QUICKSTART.md` es puerta de entrada: referenciado por `README.md` (por verificar en Step 2 con grep, solo lectura), `examples/README.md` (FIND-74 lo tocará después: 1 línea QUICKSTART→examples — dejar compatible, no bloquear) |
| Callees | `src/config.rs:935-957` (nombres env reales), `src/llm.rs:48-76` (factory `get_embedding_provider`), `release-wheels-60.yml:101,111,135,196` (wheel path real), `Cargo.toml:727-728` + `vantadb-python/pyproject.toml:8` (versión 0.5.0), `embeddings/manifest.json` (modelo default) |
| Implicaciones | Solo docs + comandos de revalidación en lectura/ejecución. Prohibido: código Rust/Python, `.opencode/`, `complets/`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, archivos FIND-68/FIND-81 |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):** `docs/user/QUICKSTART.md` (207 líneas, leído entero); `src/config.rs:925-959` (defaults env); `src/llm.rs:47-76` (factory + docstring `VANTADB_EMBEDDING_PROVIDER`); `.github/workflows/release-wheels-60.yml:97-136,192-196` (build `--out dist`, install `./dist/vantadb_py-*.whl`, attach Release); `Cargo.toml:727-728` (`[workspace.package] version = "0.5.0"`); `vantadb-python/pyproject.toml:1-12` (`name vantadb-py, version 0.5.0`); `docs/user/operations/CONFIGURATION.md:56,59,101,107` (tabla `VANTADB_LOCAL_MODEL`/`VANTADB_EMBEDDING_PROVIDER` + legacy mapping); `codegraph_explore "QUICKSTART VANTADB_ WANTA_"` (blast radius + fuente `src/llm.rs` verbatim).
- **Archivos referenciados hacia dentro (lo que el doc cita):** `vanta-cli` (`put/get/list/export/audit-index`), `vantadb-python` (`Client`, `search_memory`), `embeddings/download.py`, `embeddings/verify.py`, `embeddings/manifest.json`, wheel `./dist/vantadb_py-*.whl`, env `VANTADB_EMBEDDING_PROVIDER`/`VANTADB_LOCAL_MODEL`.
- **Archivos que referencian a los editados (referencias entrantes):** `rg "QUICKSTART" --type md` (por correr en Step 2; conocido: `examples/README.md`, `README.md`, plan FIND-67/FIND-74) — ningún import de código depende del .md; cambiar texto no rompe build.
- **Veredicto impacto:** bajo. 1 .md, 4 hunks (~6 líneas). Riesgo único: revalidación larga → timebox con comandos literales, sin embellecer. FIND-74 (Wave8) añadirá 1 línea después — dejar estructura compatible.

## Contrato

boundary 0.5.0 + wheel path real + `VANTADB_*` (sin legacy `WANTA_*`) + revalidación corriendo el quickstart + `last_reviewed` actualizada. Verificación mecánica: `rg "WANTA_" docs/user/QUICKSTART.md` = 0; `git diff --check` limpio; `scripts/validate-docs-coverage.ps1` (check tipo docs); comandos literales del quickstart verdes (timebox).

## Spec (SDD — N/A justificado, docs-only)

> N/A aceptable en tareas 100% docs sin decisiones técnicas (template task-definition.md:44). Sin símbolos nuevos, sin comportamiento nuevo.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| — | Sin decisiones técnicas abiertas: los 4 puntos tienen evidencia file:línea y el fix es mecánico (versión/path/rename/fecha) | — | — | ✅ N/A-docs-only (ref: Contrato + Investigation Notes) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** no tocar código Rust/Python (solo docs + comandos en lectura/ejecución); no tocar prohibidos (`.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, stash@{0}, FIND-68/FIND-81); no bloquear FIND-74 (estructura QUICKSTART compatible para su línea posterior); Regla 11: 0 claims sin fuente (cada versión/path con evidencia).
- **Comandos de verificación:** `rg "WANTA_" docs/user/QUICKSTART.md` (= 0) + `git diff --check` + `cargo run --bin vanta-cli -- --help` + `put/get/list/export/audit-index` literales + `python quickstart_memory.py` (binding 0.5.0 instalado) + `scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente:** ninguna prevista; colateral `docs/user/tutorials/05-embedding-integrations.md:42` con mismo legacy → NOTICED (no scope-creep), no se toca.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda. Fix docs-only, 0 líneas de código, no introduce deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | 4 puntos cerrados + `rg WANTA_` 0 + quickstart revalidado (comandos literales timebox) + `last_reviewed` hoy |
| **Commit** | Commit atómico `docs:`, solo `docs/user/QUICKSTART.md` + este task file, `git diff --check` limpio |
| **Release** | N/A docs (sin semver/changelog; release-plz ignora `docs:`). Justificado en Notas |

## Herramientas necesarias

- codegraph_explore (blast radius doc→código) ✅ usado
- campaign_detect_task_type (tipo docs) ✅ usado
- campaign_discover_skills_v2 (SDP) ✅ usado
- campaign_verify_cmd (bug exit -1 conocido → fallback bash directa y anotarlo)
- Comandos: comandos literales del quickstart + `git diff --check` + `rg "WANTA_" docs/user/QUICKSTART.md`

**Skills cargadas (SDP):** documentation-and-adrs (ADRs/plantillas, gotchas inline) + writing-guidelines (voz/tono docs) + campaign-executor (pipeline) + progreso (avance). Lifecycle genéricas (incremental/TDD/context/source/doubt) descartadas por docs-only salvo source-driven (evidencia file:línea ya aplicada).

## Investigation Notes

- **Claim:** tag actual es v0.5.0 (QUICKSTART dice v0.4.x → stale).
  **Evidencia:** `git tag --sort=-v:refname | Select -First 10` → `v0.5.0` primero; `git describe --tags --abbrev=0` → `v0.5.0`; `Cargo.toml:727-728` `[workspace.package] version = "0.5.0"`; `vantadb-python/pyproject.toml:8` `version = "0.5.0"`.
  **Confianza:** alta.
- **Claim:** wheel 0.1.1 del doc no existe; path real es `./dist/vantadb_py-*.whl` (CI) con versión 0.5.0.
  **Evidencia:** `release-wheels-60.yml:101` `--out dist --manifest-path ./vantadb-python/Cargo.toml`; `:111` `wheel="$(ls -t ./dist/vantadb_py-*.whl | head -n 1)"`; `:135` `path: ./dist/*.whl`; `:196` `files: ./dist/*.whl` (attach Release); `perf-bench-40.yml:68-69` `maturin build --out vantadb-python/dist` + `vantadb-python/dist/vantadb_py-*.whl`; local `./dist/*.whl` = `0.1.5, 0.4.0` (0.1.1 ausente, 0.5.0 aún no buildeado — patrón con versión 0.5.0, no archivo).
  **Confianza:** alta (path) / alta (versión por pyproject 0.5.0).
- **Claim:** nombres env actuales son `VANTADB_EMBEDDING_PROVIDER` / `VANTADB_LOCAL_MODEL` (FIND-89 migró código).
  **Evidencia:** `src/config.rs:936` `env::var("VANTADB_LOCAL_MODEL")`; `:956` `env::var("VANTADB_EMBEDDING_PROVIDER")`; `src/llm.rs:48,55,79` docstring+factory `VANTADB_EMBEDDING_PROVIDER`; `docs/user/operations/CONFIGURATION.md:56,59` tabla + `:101,107` mapping legacy→nuevo; `rg WANTA_` = solo `docs/user/QUICKSTART.md:188` + colateral `docs/user/tutorials/05-embedding-integrations.md:42` + plan file (fuera de scope).
  **Confianza:** alta.
- **Claim:** `last_reviewed` stale (2026-07-01) → actualizar a hoy tras revalidar.
  **Evidencia:** `docs/user/QUICKSTART.md:6` `last_reviewed: 2026-07-01`; hoy 2026-09-15 (date del entorno + plan file).
  **Confianza:** alta.
- **Claim:** revalidación viable sin red ni build pesado.
  **Evidencia:** `cargo run --bin vanta-cli -- --help` verde (compiló v0.5.0 en 4.28s, 22 comandos listados); `python -c "import vantadb_py"` → `0.5.0` (DeprecationWarning `import vantadb` futuro, fuera de scope); `embeddings/verify.py` + `download.py` existen.
  **Confianza:** alta.
- Colateral NOTICED (no scope-creep): `docs/user/tutorials/05-embedding-integrations.md:42` mismo `WANTA_*` legacy → NOTICED para FIND futuro, no se toca (disciplina Wave5).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado, evidencia completa, sin web research (todo local) |
| Pendientes de ejecución (downhill) | 0 — Step 1 ✅ + Step 2 ✅ |
| % completado | 100% (2/2 steps; verify mecánico + review self ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No aplica (docs-only, sin trust boundaries/input/auth/deps/storage/FFI/red). Justificado: 0 líneas de código, comandos de revalidación en lectura/ejecución sobre `./quickstart_data` temporal.
- [x] **PERFORMANCE** — No aplica (sin hot paths: no toca `vector/`, `engine.rs`, search/ingestión, serialización). Justificado: docs-only.

## Steps

### Step 1: Fix 4 puntos verificados (~6 líneas, 1 archivo)
- **Archivos:** `docs/user/QUICKSTART.md:6,12,90,188-189` (+ §5 `:142-144` y §6 `:179-183` — deriva hallada en revalidación, mismo archivo, ver HALLAZGOS)
- **Acción (mínimo diff, sin reescritura):**
  1. `:6` `last_reviewed: 2026-07-01` → `2026-09-15` (tras revalidar en Step 2 ✅).
  2. `:12` `v0.4.x MVP boundary` → `v0.5.0 MVP boundary` (evidencia tag + workspace.package + pyproject).
  3. `:90` `pip install ./path/to/vantadb_py-0.1.1-*.whl` → `pip install ./dist/vantadb_py-0.5.0-*.whl` + nota 4 líneas: wheels en GitHub Release (`release-wheels-60.yml:196`) y local `maturin build --out dist` (`:101`); `vantadb-python/dist/` como alt local (perf-bench-40.yml:68). Sin URL inventada.
  4. `:188` `WANTA_EMBEDDING_PROVIDER=local WANTA_LOCAL_MODEL=` → `VANTADB_EMBEDDING_PROVIDER=local VANTADB_LOCAL_MODEL=` (evidencia config.rs:936,956). NO tocado `docs/user/tutorials/05-*:42` (NOTICED).
  5. (HALLAZGO revalidación) `:142-144` `db.search_memory` → `db.search` (3 líneas; flat `*_memory` removidos sin aliases per `PYTHON_SDK.md:24`, `lib.rs:1243` `fn search` vigente).
  6. (HALLAZGO revalidación) §6 +4 líneas: fresh DB → `repair_recommended` hasta `rebuild-index` (verificado 2× → `passed: true`).
- **Verify:** `rg "WANTA_" docs/user/QUICKSTART.md` = 0 ✅ + `rg "search_memory" docs/user/QUICKSTART.md` = 0 ✅ + `rg "v0\.4|0\.1\.1"` = 0 ✅ + `git diff --check` limpio (solo warnings CRLF ajenos) ✅ + `git diff docs/user/QUICKSTART.md` = 23 líneas, 6 hunks ✅.
- **Estado:** ✅ DONE

### Step 2: Revalidación literal (timebox) + verify mecánico + review P2-01
- **Archivos:** ninguno nuevo (ejecución en `C:\Users\Eros\AppData\Local\Temp\opencode\find67-{reval,cli,py,literal}` + limpieza; repo sin artefactos: `git status` solo 2 archivos propios ✅)
- **Acción:** comandos TAL CUAL del doc (sin embellecer) ✅:
  - `cargo run --bin vanta-cli -- --help` → 22 comandos ✅ (build v0.5.0 4.28s)
  - `put/get/list` §3 → `✓ Record stored`, `Payload: local durable memory`, tabla con `memory-1` ✅
  - snippet §5 literal CON `metadata={"kind":"note"}` y `db.search` → `vector: ['vector','hybrid','text']`, `text: ['text']`, `hybrid: ['hybrid','vector','text']` ✅ (`LITERAL SNIPPET OK`)
  - `export` §6 → `Records exported: 1` (CLI-only) y `3` (py+CLI) ✅
  - `audit-index` §6 → `repair_recommended` en fresh DB (missing/incompatible) + `passed: true` tras `rebuild-index` (2×) ✅ → nota §6
  - `python embeddings/verify.py --check` §7 → `[check] OK ... [verify] PASS` ✅ (sin descarga)
  - `§7 embed-local` con modelo real NO corrido (691 MB download, fuera de timebox; comando solo renombrado, misma forma) — registrado en deuda.
- **Verify:** `rg "WANTA_"` = 0 ✅; `git diff --check` ✅; `scripts/validate-docs-coverage.ps1` → `0 gaps` ✅; `campaign_verify_cmd "git diff --check"` → exit -1 vacío (bug conocido plan Riesgos, fallback bash aplicado) ✅; review self (abajo) ✅ Approve.
- **Estado:** ✅ DONE

## Dependencias

- Previa: Wave4 DONE 15/29 (dado plan). Paralelas Wave5: FIND-68, FIND-81 (disjuntos, no tocar sus archivos).
- Orden inverso: FIND-74 (Wave8) tocará QUICKSTART después (1 línea QUICKSTART→examples) — dejar compatible, no bloquearlo.
- Next: Wave6 (orquestador decide).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** self-review (implementador) con checklist anti-hábitos + verificación mecánica; reviewer distinto pendiente al push vía vanta-lead (orquestador decide; precedente FIND-65 cerró con self-review + hooks).
- **Enfoque:** ¿los 4 puntos citan evidencia file:línea? Sí (tag/Cargo/pyproject; release-wheels-60.yml; config.rs+llm.rs; fecha entorno). ¿wheel path sin claims sin fuente (Regla 11)? Sí (patrón `./dist/`, sin URL inventada; local `0.1.5/0.4.0` documentado como evidencia de que `0.1.1` no existe). ¿sin scope-creep? Tutorial colateral intacto ✅; §5/§6 mismo archivo + requeridos por contrato de revalidación ✅. ¿sin reescritura (ponytail mínimo diff)? 23 líneas / 6 hunks ✅.
- **Cómo se probó:** revalidación Step 2 outputs reales (no auto-reporte) + `rg` triple-0 + `diff --check` + validate-docs-coverage 0 gaps + `git status` 2 archivos propios.
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
- **Veredicto:** ✅ approve (self; distinto-revisor opcional en push)

## HALLAZGOS (revalidación — para orquestador, Backlog NO tocado por race Wave5)

1. **§5 `search_memory` removido (ARREGLADO inline):** flat `*_memory` eliminados sin aliases (`PYTHON_SDK.md:24`, AST-010/012); `db.search` vigente (`lib.rs:1243`). Sin el rename el quickstart falla con `AttributeError`. Fix 3 líneas mismo archivo.
2. **§6 audit en fresh DB (ARREGLADO con nota):** `repair_recommended` hasta `rebuild-index` (2× → `passed: true`). Nota 4 líneas añadida.
3. **NOTICED (no tocado):** `docs/user/tutorials/05-embedding-integrations.md:42` mismo `WANTA_*` legacy → candidato FIND futuro.
4. **NOTICED (no tocado):** `§7 embed-local` con modelo real no ejecutado (691 MB, fuera de timebox); solo rename aplicado, misma forma de comando.
5. **NOTICED (entorno):** `campaign_verify_cmd` bug exit -1 vacío (confirmado 0.3s) → fallback bash; binding instalado emite `DeprecationWarning import vantadb_py → vantadb` (doc ya usa `import vantadb` ✅).

## Notas

- Ponytail full: mínimo diff docs (~6 líneas), sin reescritura, sin embellecer quickstart. `campaign_verify_cmd` con bug exit -1 conocido (plan Riesgos) → fallback bash directa y anotarlo en RESULTADO.
- Backlog→avance NO tocar (race Wave5, queda al orquestador) + push vía vanta-lead (instrucción tarea).
- Regla 11: cada versión/path del fix lleva fuente (ver Investigation Notes). Si ambigüedad en wheel path de releases sin evidencia en repo → marcar `[cita NO VERIFICADA]` (no se espera: evidencia CI completa).
- Commit: `docs:` con ID, solo archivos propios (`docs/user/QUICKSTART.md`, `docs/dev/tasks/FIND-67.md`).

## Referencias

- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles

## Context Save Point

- **Fecha:** 2026-09-15
- **Branch:** develop
- **CI pendiente:** no (docs-only; validate-docs-coverage 0 gaps ✅; push vía vanta-lead)
- **Decisiones:** fix mecánico 4 puntos con evidencia file:línea; §5 rename + §6 nota inline (requeridos por revalidación, mismo archivo); colateral tutorial → NOTICED no scope-creep; wheel como patrón `./dist/vantadb_py-0.5.0-*.whl` (no archivo, no URL inventada)
- **Problemas conocidos:** campaign_verify_cmd bug exit -1 (fallback bash documentado); `§7 embed-local` con modelo real no ejecutado (691 MB, fuera de timebox — NOTICED); `import vantadb_py` DeprecationWarning (doc ya canónico ✅)
- **Próxima tarea:** ninguna en FIND-67 (2/2 ✅). Siguiente plan: FIND-68 (Wave5, archivo disjunto)
