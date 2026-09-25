# FIND-137 — Unificar rustdocs caníbales (`ci-rustdoc.yml` vs `rustdoc-70.yml`) en uno solo

> **Plan:** `docs/dev/plans/2026-09-21-workflows-repair.md` Wave 1 (paralelo ×3 disjunto: 137/138/139; este task solo toca sus 3 archivos).
> **Estado:** ⬜ PENDING → IN PROGRESS (discovery + survivor-edit + git rm + verify + commit; NO PUSH)
> **Appetite:** 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
> **Branch:** `develop` · **Commit:** `ci: FIND-137 — ...` (solo tras verify mecánico; solo archivos propios: survivor + task file; el eliminado va vía `git rm`)
> **Ruta:** vanta-lead (CI) · **nextTask:** Wave 2 (orquestador)
> **SDP:** `campaign_discover_skills_v2` archivosClave=`ci-rustdoc.yml, rustdoc-70.yml, README.md` phase=BUILD contractKeywords=[github-actions, rustdoc, cargo-doc, actionlint] → 8 skills (ver §5). Cargadas: `ci-cd-and-automation` (canónica del área, sugerida por el detalle), `doubt-driven-development` (base CI/CD: CLAIM/DOUBT adversarial pre-commit) (+ base campaign-executor, progreso, ponytail full; `frontend-ui-engineering`/`api-and-interface-design`/`test-driven-development` descartadas — 0 UI, 0 API nueva, 0 lógica; `incremental-implementation`/`context-engineering`/`source-driven-development` aplican como slice único/contexto materializado/sintaxis estable sin carga separada).
> **SKILLS_CARGADAS:** ci-cd-and-automation, doubt-driven-development (+ base campaign-executor, progreso, ponytail full)
> **Referencias:** `.opencode/rules/release-ci.md` (leída COMPLETA 42L — aplican regla 2 — sccache vía `rust-setup`, no duplicar steps — y regla 5 — sin `continue-on-error` nuevo; 1/3/4 N/A: no toca allocators/Dockerfile/version-sync), `.opencode/references/definition-of-done.md` (leída COMPLETA 144L — DoD v1 baseline + checklist pre-commit), Notion N/A (tarea CI mecánica, sin mapeo a Problema/Propuesta/hijas — filtro VantaDB: nada aplica).
> **Regla 11:** N/A justificado — 0 claims de performance, 0 números, 0 adjetivos de rendimiento en este fix.
> **Research Digest:** sin digest previo (DISCOVERY inline, 2 archivos + README).

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-21-workflows-repair.md`
- **Fuente:** Backlog `FIND-137` (:229) + plan Wave 1
- **Esfuerzo:** 🟢 1h · **Prioridad:** 🟢 · **Tipo:** CI/YAML (devops)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-22 · **last-synced:** 2026-09-22
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 (decisión por evidencia, tabla §8)
- **Pendientes (downhill):** 3 steps (survivor-edit + git rm/verify + commit/cierre)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | pushes a `main/develop` + PRs a `main/develop` cuyos archivos matcheen `paths`; pestaña Artifacts del run (adoptantes pre-docs.rs) |
| Callees | `.github/actions/rust-setup` (composite: toolchain + sccache + system-deps), `cargo doc`, `actions/upload-artifact@v4.6.2` |
| Implicaciones | 1 workflow rustdoc en vez de 2; sin cambio de comportamiento público (0 símbolos `pub`, 0 endpoints, 0 bindings); artifact pasa a `api-reference-rust.tar.gz` 30d (nombre estable, formato comprimido); triggers = unión (superset estricto de ambos) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rustdoc.yml` (82L — triggers `:16-37`, concurrency top-level `:45-47`, job `:49-82`, `cargo doc --no-deps --workspace --all-features` `:74`, artifact `api-reference-rust`/`target/doc`/7d `:76-82`), `.github/workflows/rustdoc-70.yml` (85L — triggers `:3-24`, env `:26-28`, job `:33-85`, nightly+override `:45-63`, tar.gz/30d `:65-76`, summary `:78-85`), `README.md` (433L — badges `:8-10` sin rustdoc), `README_ES.md` (badges idem, verificado por grep), `.opencode/rules/release-ci.md` (42L), `.opencode/references/definition-of-done.md` (144L), `docs/dev/tasks/FIND-128.md:155-195` (matriz triggers + propuesta dedup), `docs/dev/tasks/FIND-95.md` (modelo task file CI).
- **Archivos referenciados hacia dentro (del survivor):** `.github/actions/rust-setup` (única dependencia externa); tras el merge el comentario stale `:59` "(same fix as rustdoc-70.yml)" se reescribe autocontenido (citaba al eliminado).
- **Archivos que referencian a los editados (referencias entrantes):** NINGUNO — verificado: `rg "ci-rustdoc\.yml|rustdoc-70\.yml"` → solo los 2 workflows (self-paths + comentario stale) + historial (CHANGELOG, plans archive, tasks FIND-92/95 — historia, no se toca) + Backlog `:229` (prohibido tocar, orquestador) + plan file (solo recitation); `rg "actions/workflow/status.*rustdoc"` en README/ES → 0 badges rustdoc (nada que actualizar); `rg "download-artifact|api-reference-rust|rustdoc-html"` → 0 consumidores del artifact; `ci-gate.yml` REQUIRED sin rustdoc; `docs/dev/workflow/` sin archivo rustdoc.
- **Veredicto impacto:** 1 archivo editado + 1 eliminado + 0 badges. Riesgo 🟢 mínimo. **Gate D:** no dispara — ≤10 archivos, sin hot path/API pública/símbolos nuevos, contrato mecánico.

## Contrato

"1 solo workflow rustdoc (`ci-rustdoc.yml` survivor), 0 referencias rotas (grep), `actionlint` exit 0, survivor con lo mejor de ambos"

- [ ] `ls .github/workflows/` muestra `ci-rustdoc.yml` y NO `rustdoc-70.yml`
- [ ] Survivor: toolchain stable + `cargo doc --no-deps --workspace --all-features --document-private-items` (sin override nightly, sin `RUSTDOCFLAGS -Z`)
- [ ] Survivor triggers: push `[main, develop]` + PR `[main, develop]` + paths unión de código + self-path + dispatch
- [ ] Survivor artifact: `api-reference-rust.tar.gz`, `retention-days: 30`, `if-no-files-found: error` + summary step
- [ ] `actionlint` exit 0 sobre el survivor (y full si disponible)
- [ ] `git diff --check` limpio + `rg "rustdoc-70"` 0 hits fuera de historial
- [ ] README.md sin cambios (0 badges rustdoc — verificado, no hay nada que actualizar)

## Spec (SDD — Phase 1b)

N/A justificado con evidencia (no `N/A` vacío): la solución planeada NO agrega símbolos/contratos públicos nuevos — 0 `pub fn`/struct/enum, 0 tools MCP, 0 endpoints, 0 métodos de binding, 0 componentes `web/` (solo YAML: triggers + flags `cargo doc` + artifact). Tabla de decisiones sustituida por tabla comparativa §8 con justificación por-evidencia por ítem (archivo:línea). Gate spec-first N/A.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** resto workflows intactos (FIND-138 diagnostica el flake "Generate API reference" sobre el survivor vivo; FIND-139 `ci-gate`/`gate-docs-21` paralelos); `src/`, `web/`, `desktop/`, `reparacion.bat`, `.opencode/`, `completions/*`, `*.lock`, stash GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation server) intactos; `C:/Users/Eros/.vantadb*` y secretos ni leídos; NO PUSH (pushea solo vanta-lead).
- **Comandos de verificación:** `actionlint .github/workflows/ci-rustdoc.yml` (exit 0) · `git diff --check` (limpio) · `rg -n "rustdoc-70" .github/ README.md README_ES.md docs/dev/workflow/ docs/user/operations/` (0 hits) · `git status --short` (solo 3 paths: survivor M + eliminado D + task file)
- **Deuda pendiente:** ninguna (FIND-138 dirá si el flake era infra externa; renombres FIND-142 al final per decisión owner #4)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# FIND-137 — Unificar rustdocs caníbales en uno solo` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia §8 |
| `nextTask` | Wave 2 (orquestador) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — este diff ELIMINA deuda (duplicación de workflow + hack `rustup override` + artifact sin comprimir). No se introduce deuda nueva: 0 `unsafe`, 0 `clone()` hot-path, 0 suppressions, 0 skips.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato verificable por comando (actionlint + diff-check + greps) |
| **Commit** | Atómico (survivor-edit + `git rm` + task file), `ci:` + task ID, `git diff` limpio, verificación mecánica (nunca auto-reporte) |
| **Release** | N/A justificado (`ci:` no publica — release-plz lo ignora; sin bump/semver) |

## Herramientas necesarias

- `actionlint` (gate pre-publish YAML — exit 0 obligatorio)
- `git rm` (eliminación trackeada del workflow perdedor) + `git diff --check` (whitespace)
- `rg` (verificación 0 refs rotas — 3 greps puntuales, sin loop)
- `campaign_verify_cmd` (contrato; BUG exit -1 conocido → fallback bash directa + mención en RESULTADO)
- `campaign_update_task_state` (in-progress → completed + recitation) + `campaign_memory_write` (1 lesson)

**Skills cargadas (SDP):** ci-cd-and-automation (pipelines/quality-gates del área) · doubt-driven-development (CLAIM/DOUBT adversarial pre-commit sobre survivor-choice + diff) · base campaign-executor/progreso/ponytail-full.

## Investigation Notes

- Canibalización confirmada por texto: mismo `cargo doc --no-deps --workspace --all-features` (ci-rustdoc `:74` vs rustdoc-70 `:61`) + mismo `concurrency.group: rustdoc-${{ github.ref }}` + `cancel-in-progress: true` (ci-rustdoc `:45-47` top-level vs rustdoc-70 `:38-40` job-level) → cada push con paths coincidentes cancela al gemelo. Hallazgo O3 de FIND-95 (`colisión concurrency.group`) era este futuro FIND.
- FIND-128:8/27 + propuesta dedup (líneas 190/193): quitar `develop` de push en 7 workflows + alinear PR de rustdoc-70 — este task resuelve el caso rustdoc tomando el superset en vez del recorte (cobertura, no recorte: push `[main,develop]` de ci-rustdoc + PR `[main,develop]` de rustdoc-70).
- Flake FIND-138 ("Generate API reference 0s"): ese es el job-name de ci-rustdoc (`:51`) — razón adicional para que el survivor conserve ese archivo/nombre (FIND-138 paralelo no pierde su scope).
- `rust-toolchain.toml` pinea stable (citado en rustdoc-70 `:57-58` como motivo del override) — stable es el toolchain canónico del repo (MSRV 1.94.1, release-ci regla 3).
- `--document-private-items` es flag estable de `cargo doc` (no requiere nightly); lo nightly-only es `RUSTDOCFLAGS: --show-type-layout --enable-index-page -Z unstable-options` (rustdoc-70 `:63`) — se descarta con el override.
- `docs/**` + `*.md` (rustdoc-70 `:8,12,17,21`) no afectan la salida de `cargo doc` (inputs = fuentes Rust + manifests) — incluirlos genera runs con artifact idéntico (spam); se descartan, se conserva self-path renombrado.
- Internet N/A (detalle §7-9): 0 ambigüedad de APIs externas; fuente primaria = FND-17 + FIND-128/133 intra-repo.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — survivor y cada dimensión decididos por evidencia (§8) |
| Pendientes de ejecución (downhill) | 3 — S2 survivor-edit, S3 git rm + verify, S4 commit + cierre |
| % completado | 25% (S1 ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A justificado: 0 trust boundaries (sin input de usuario, auth, deps nuevas/bumps, storage, FFI, red); `permissions: contents: read` se conserva; sin secrets en diff (verificado pre-commit por grep).
- [x] **PERFORMANCE** — N/A justificado: YAML CI, no toca hot paths (`vector/`, `engine.rs`, serialización); timeout 30 adoptado por margen sobre build ~10min medido (FIND-133), no por benchmark.

## Steps

### Step 1: DISCOVERY + crear este task file (Regla 0, comparativa, steps)

- **Archivos:** `docs/dev/tasks/FIND-137.md` (NUEVO)
- **Acción:** leer ambos yml enteros + README badges + release-ci + DoD + FIND-128 matriz; poblar Impacto mapeado (Regla 0) ANTES de editar; tabla decisión §8 con evidencia por ítem
- **Verify:** este archivo existe con 10 bloques + Regla 0 llena + tabla §8
- **Estado:** ✅ DONE

### Step 2: Survivor-edit `ci-rustdoc.yml` (adopta lo mejor de rustdoc-70)

- **Archivos:** `.github/workflows/ci-rustdoc.yml`
- **Acción:** PR branches → `[main, develop]`; paths += self-path; comando += `--document-private-items`; timeout 15→30; +env determinista; tar.gz + `retention-days: 30` + summary; reescribir comentario stale `:59`; header FIND-137 (3 líneas)
- **Verify:** `git diff --stat` solo ese archivo + `actionlint` exit 0
- **Estado:** ✅ DONE (2026-09-22: filas §8 2-9 aplicadas — PR [main,develop], self-paths, `--document-private-items`, timeout 30, env determinista, tar.gz 30d + summary, comentario stale reescrito autocontenido, header FIND-137; `actionlint` survivor + full exit 0)

### Step 3: `git rm` perdedor + verify contrato

- **Archivos:** `.github/workflows/rustdoc-70.yml` (DELETE vía `git rm`)
- **Acción:** `git rm .github/workflows/rustdoc-70.yml`; correr §6 en orden (actionlint + diff-check + 3 greps + status)
- **Verify:** `ls` sin `rustdoc-70.yml`; `rg "rustdoc-70"` 0 hits fuera de historial; `git status --short` solo 3 paths propios
- **Estado:** ✅ DONE (2026-09-22: `git rm` exit 0, `ls` sin rustdoc-70.yml; actionlint survivor+full exit 0; diff-check limpio; greps: rustdoc-70 solo header survivor 2L histórico, badges 0, artifact solo survivor 5L)

### Step 4: Commit `ci:` + cierre (recitation + lesson + RESULTADO)

- **Archivos:** staging selectivo (survivor M + eliminado D + task file)
- **Acción:** `git add` 2 paths + commit `ci: FIND-137 — ...` (NO PUSH) + `campaign_update_task_state` completed + 1 lesson + bloque RESULTADO §7
- **Verify:** hash commit existe; `git status` limpio de propios; RESULTADO con GATES_EVALUADOS + SKILLS_CARGADAS
- **Estado:** ⬜ PENDING

## Dependencias

- **Wave:** Wave 1 (paralelo ×3 disjunto con FIND-138/139 — archivos disjuntos, sin bloqueantes)
- **Bloqueantes:** ninguno
- **Paralelas:** FIND-138 (flake diagnosis sobre el survivor vivo — este task le preserva el scope), FIND-139 (`ci-gate.yml`/`gate-docs-21.yml` — no tocados aquí)
- **Previa:** Wave 0 ✅ (134/135/136 pusheados per plan `:5`)
- **Next:** Wave 2 (orquestador)

## Review (GATE — agente distinto, P2-01)

> Non-interactive (`/pipeline task`) → self-review doubt-driven reconciliado abajo (degradado declarado, cross-model skipped: non-interactive context). Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** doubt-driven-development (self, degradado — sin sub-agente disponible en este contexto)
- **Enfoque:** ¿sobrevive el archivo correcto? ¿cada dimensión adopta lo mejor con evidencia? ¿alternativa (sobrevive rustdoc-70) refutada?
- **Cómo se probó:** actionlint exit 0 + git diff --check + greps 0-refs (mecánico, no auto-reporte) — ver §6/§10
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos no ejecutados
  - [x] No saltarse clarificación (contrato mecánico, 0 incógnitas)
  - [x] No declarar done sin verify contra acceptance criteria
  - [x] No ignorar fallos ni reportar "todo OK" parcial
  - [x] No dar por saturada la búsqueda (3 greps + README/ES + workflows + docs/)
  - [x] No copiar sin citar (toda fila §8 cita archivo:línea)
  - [x] No reintentar en bucle sin diagnóstico (Gate V si 2 fallas mismo-error)
  - [x] No dejar huérfanos los pasos (cada step → contrato)
  - [x] No degradar chequeo de errores (N/A — sin paths de dinero/seguridad)
  - [x] No gastar presupuesto infinito (1 slice, ~100 líneas)
- **Veredicto:** ⬜ PENDING (se registra en §10 tras el diff)

## Notas

- Ponytail full: 1 archivo editado + 1 `git rm` + 0 badges (verificado innecesario, no omitido); `docs/**`+`*.md` descartados con evidencia (spam de runs idénticos), no por gusto.
- Alternativa refutada (sobrevive `rustdoc-70.yml`): pierde push-main (main deriva sin reference), pierde nombre `ci-*` (FIND-142 renombra igual), conserva hack `rustup override` con estado global, y mata el scope de FIND-138 (job-name "Generate API reference" vive en ci-rustdoc `:51`).

## 5. SKILLS — lista SDP (≤8) con 1 línea de cuándo aplica c/u

SDP `campaign_discover_skills_v2` phase=BUILD devolvió 8 (base + lifecycle, keyword-mapped 0):

1. `campaign-executor` — base: state machine PLAN→ACT→VERIFY + RESULTADO + recitation (siempre en pipeline-full).
2. `doubt-driven-development` — base CI/CD: CLAIM/DOUBT adversarial pre-commit sobre survivor-choice + diff (cargada).
3. `incremental-implementation` — lifecycle BUILD: 1 slice delgado (edit + rm + verify) — aplica como slice único, sin sub-slices.
4. `test-driven-development` — lifecycle BUILD: Red-Green — NO aplica (0 lógica; verify mecánico actionlint+grep sustituye RED).
5. `context-engineering` — lifecycle BUILD: empaquetar contexto (plan + Backlog + ambos yml) — aplica en DISCOVERY (este archivo lo materializa).
6. `source-driven-development` — lifecycle BUILD: sintaxis `paths`/artifact estable — N/A (verificada intra-repo: gate-docs-21 + FIND-92/95; 0 ambigüedad).
7. `frontend-ui-engineering` — lifecycle BUILD: UI en `web/` — NO aplica (0 archivos web; descartada).
8. `api-and-interface-design` — lifecycle BUILD: APIs/boundaries — NO aplica (0 símbolos nuevos; descartada).
- Extra justificada fuera de SDP: `ci-cd-and-automation` (canónica del área CI + sugerida por el detalle: pipelines, quality gates, artifact retention).

## 6. HERRAMIENTAS+MCP — comandos exactos + MCP (sin grep-loop innecesario)

```powershell
# Verify contrato (post-edit, en orden):
actionlint .github/workflows/ci-rustdoc.yml              # exit 0 (contrato d)
actionlint                                               # full (si disponible en PATH)
git diff --check                                         # limpio (contrato)
rg -n "rustdoc-70" .github/ README.md README_ES.md docs/dev/workflow/ docs/user/operations/  # 0 hits (contrato b)
rg -n "rustdoc" README.md README_ES.md                   # 0 badges (evidencia: nada que actualizar)
rg -n "api-reference-rust|rustdoc-html" .github/workflows/  # solo survivor (0 consumidores externos)
git status --short; git diff --stat                      # solo 3 paths propios
git rm .github/workflows/rustdoc-70.yml                  # eliminado trackeado (contrato a)
```

- **MCP:** `campaign_detect_task_type` ✅ (devops/CI-CD) · `campaign_discover_skills_v2` ✅ (8 skills, §5) · `campaign_get_workflow` ✅ (null → C0 genérica) · `campaign_verify_cmd` (BUG exit -1 conocido → fallback bash directa + mención en RESULTADO) · `campaign_update_task_state` (in-progress ✅ → completed ⬜) · `campaign_memory_write` (1 lesson ⬜).
- **Sin grep-loop innecesario:** 3 `rg` puntuales bastan (refs-rotas + badges + artifact-consumers); codegraph/Cargo/Internet N/A per detalle §6-9.

## 7. INVESTIGACIÓN CÓDIGO — blast radius (DISCOVERY)

- **Triggers hoy:** ci-rustdoc push `[main,develop]` + PR `[main]` (paths código `:19-26`/`:28-36`); rustdoc-70 push `[develop]` + PR `[main,develop]` (paths docs-code `:5-13`/`:15-23`). Unión survivor = push `[main,develop]` + PR `[main,develop]`, paths código unión + self-path.
- **Implicaciones:** runs duplicados/canibalizados desaparecen (1 group, 1 workflow); artifact comprimido 30d sustituye raw 7d (adopters pre-docs.rs, header `:3-14` conservado); `docs/**`+`*.md` fuera (runs idénticos = spam); `RUSTDOCFLAGS -Z` + override fuera (nightly-only, hack con estado global).
- **Riesgo:** nombre de artifact cambia de formato (`target/doc/` → `api-reference-rust.tar.gz`, nombre lógico estable) — 0 consumidores internos (grep §6); adoptantes externos lo descargan por nombre (`api-reference-rust` estable) → re-trabajo = descomprimir, documentado en summary. Riesgo 🟢.
- **Veredicto:** blast radius = 1 edit + 1 delete + 0 badges. Sin API pública, sin símbolos nuevos, sin hot path, sin `cargo build`. Riesgo 🟢 mínimo.

## 8. INVESTIGACIÓN PROBLEMA — tabla comparativa + decisión con evidencia (sin opinión)

| # | Dimensión | `ci-rustdoc.yml` (línea) | `rustdoc-70.yml` (línea) | Gana + evidencia |
|---|-----------|--------------------------|--------------------------|------------------|
| 1 | Supervivencia (archivo) | `ci-*` convención Fast Gate (`ci-rust-10`, `ci-gate`); header RES-11/FND-17 | sufijo `-70` condenado por FIND-142 (renombres) | **ci-rustdoc** — evita rename+churn; FIND-138 scope (job-name `:51`) vive aquí |
| 2 | Toolchain | stable (`:62`, default rust-setup) = `rust-toolchain.toml` pin + MSRV 1.94.1 | nightly + `rustup override set nightly` (`:48,:60`, estado global que fuga a steps siguientes) | **stable** — reproducible; override es hack (el propio comentario `:57-58` admite el shadowing) |
| 3 | Flags `cargo doc` | `--no-deps --workspace --all-features` (`:74`), sin privados | + `--document-private-items` (`:61`, flag **estable**) + `RUSTDOCFLAGS -Z unstable-options` (`:63`, nightly-only) | **mezcla**: base + `--document-private-items`; `-Z` se descarta (ata al nightly por 2 flags cosméticos `type-layout`/`index-page`) |
| 4 | Artifact / retention | `api-reference-rust` ← `target/doc` raw, 7d (`:77-82`) | `rustdoc-html` ← tar.gz, 30d (`:71-76`) | **mezcla**: nombre `api-reference-rust` (estable, 0 consumidores que romper) + tar.gz (comprimido, `du -h` testigo) + 30d (adoptantes pre-docs.rs) |
| 5 | Triggers branches | push `[main,develop]` (`:18`) cubre post-merge main; PR `[main]` (`:28`) | push `[develop]` (`:5`, main deriva ciego); PR `[main,develop]` (`:15`, único con PR-develop — detalle + FIND-128:27) | **unión**: push `[main,develop]` + PR `[main,develop]` (superset estricto, 0 pérdida de cobertura) |
| 6 | Trigger paths | código completo: `src tests Cargo vantadb-* vanta-memory providers` (`:19-26`) | `src docs Cargo vanta-memory *.md` self (`:6-13`) — sin `tests/vantadb-*/providers` (cambios en esos crates no lo disparan aunque `cargo doc --workspace` los incluye) | **unión código** + self renombrado; `docs/**`+`*.md` fuera (no afectan salida `cargo doc` → runs idénticos) |
| 7 | Concurrency | top-level `:45-47` (convención repo: `gate-docs-21:12-14`, comentario `:42-44`) | job-level `:38-40` (mismo group → canibaliza igual) | **top-level** (convención; 1 group tras el merge = fin de la canibalización) |
| 8 | Timeout | 15 (`:53`) — 1.5× sobre build ~10min (FIND-133) | 30 (`:37`) — 3× margen cold-cache + bindgen rocksdb | **30** (margen FIND-135: duración real + margen; 15 arriesga timeout-flake) |
| 9 | Extras | header documentado, gate docstrings (`:9-11`) | env determinista (`:26-28`), summary step (`:78-85`) | **ambos**: env + summary adoptados; header conservado |
| 10 | Badges README | 0 badges rustdoc (`README:8-10`, `README_ES:8-10` — solo Rust CI/Docs/Security) | 0 badges | **nada que actualizar** (verificado, no omitido) |

**Decisión: sobrevive `ci-rustdoc.yml`** (9-1 en dimensiones, la única "derrota" —retention/artifact— se adopta del perdedor). El perdedor se elimina con `git rm`.

## 9. INVESTIGACIÓN INTERNET — N/A

0 ambigüedad de APIs externas (detalle §7-9: internet N/A). Sintaxis `paths`/artifact/actionlint estable y verificable intra-repo (gemelos FIND-92/95 + `gate-docs-21.yml`). 0 citas externas, 0 claims → GATE CITAS TSYS-13 N/A.

## 10. VALIDACIÓN+CIERRE — verify + OCR + DoD + reviewer + Gates + commit

- **Verify contrato:** los 6 comandos del §6 en orden. `campaign_verify_cmd` con bug exit -1 conocido → fallback bash directa + mención en RESULTADO.
- **Verify full relevante:** YAML-only → `cargo fmt/clippy/nextest` N/A (0 `.rs` tocados); `scripts/validate-docs-coverage.ps1` N/A (0 docs producto; este task file es/bypass de cobertura); OCR `pwsh dev-tools/ocr-review.ps1` advisory si disponible (Critical/High=bloquea, Medium→FIND nuevo, Low se descarta). `/cleanCA` N/A (0 código).
- **DoD 3 niveles:** (1) task: contrato mecánico + determinista aplicable (actionlint+diff-check; fmt/clippy/nextest N/A sin `.rs`); (2) commit: 1 cosa lógica (`ci:` + ID), porqué en mensaje, sin secrets (`grep -i "password\|secret\|api_key\|token"` en diff staged), sin formatting mezclado; (3) release/deploy N/A (`ci:` no publica; reversible por `git revert` + re-run del workflow — rollback §2b del plan).
- **Reviewer P2-01:** doubt-driven degradado (arriba §Review); `code-review-and-quality` como lente pre-commit sobre el diff.
- **Gates D/V/C:** D no dispara (Regla 0: ≤10 archivos, 0 símbolos/API/hot-path, contrato mecánico); V dispara solo si 2 fallas mismo-error en verify (→ `question`, sin respuesta → STOP); C: colaterales → `prompts/findings.md` (FIND nuevo, nunca solo anotado); `git status` con M fuera del blast radius → confirmar alcance antes de `git add` (selectivo: 2 paths + delete trackeado).
- **Cierre:** staging selectivo + commit `ci: FIND-137 — ...` (NO PUSH) + `campaign_update_task_state` completed + recitation canónica + 1 lesson + Context Save Point + bloque RESULTADO §7 (siempre; nunca silencio). Backlog→avance NO tocar (race paralelo, orquestador).

## Pre-mortem (del plan — verificado en DISCOVERY)

1. `git rm` del archivo equivocado → verificar `ls` pre/post + `git status` (D=deleted solo `rustdoc-70.yml`).
2. Badge/docs que citan al eliminado → 3 greps §6 pre-commit confirman 0 (única cita viva era el comentario stale `:59`, reescrito en el survivor).
3. FIND-138/139 en paralelo tocan mis archivos → plan garantiza disjuntos; `git status` pre-commit confirma 0 M ajenos.

## Context Save Point

- **Branch:** `develop` · **Steps:** S1 ✅, S2 ✅, S3 ✅, S4 ⏳ (commit + cierre) · **Próximo:** S4 staging selectivo + commit `ci:` (NO PUSH) + recitation + RESULTADO.
- **Verify previsto:** actionlint exit 0 · diff-check limpio · `rg rustdoc-70` 0 fuera de historial · status con 3 paths propios.

## Scope previo

N/A (task file nuevo, sin legacy).
