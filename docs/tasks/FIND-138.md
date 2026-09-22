# FIND-138 — Diagnosticar flake "Generate API reference (rustdoc)" 0-1s sin log (BlobNotFound)

> **Plan:** `docs/plans/2026-09-21-workflows-repair.md` Wave 1 (paralelo ×3 disjunto: 137/138/139; este task es SOLO observación — prohibido editar `ci-rustdoc.yml`/`rustdoc-70.yml`, resto workflows, `src`/`web`/`desktop`/locks/plans/Backlog, secretos).
> **Estado:** ⏳ IN PROGRESS → COMPLETED (discovery + observación GH solo-GET + veredicto + commit task file; NO PUSH)
> **Appetite:** 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
> **Branch:** `develop` · **Commit:** `docs: FIND-138 — ...` (solo task file; sin cambio YAML → `docs:` no dispara release)
> **Ruta:** vanta-lead (CI/diagnóstico) · **nextTask:** Wave 2 (orquestador)
> **SDP:** `campaign_discover_skills_v2` archivosClave=`.github/workflows/ci-rustdoc.yml (solo lectura observacional, prohibido editar)` phase=BUILD contractKeywords=[rustdoc, flake, github actions, BlobNotFound, diagnostico CI] taskId=FIND-138 taskType=bug-fix → 8 skills (ver §5). Cargadas: `ci-cd-and-automation` (canónica del área CI), `systematic-debugging` (bug: Iron Law root-cause antes de proponer fix) (+ base campaign-executor, progreso, ponytail full; resto SDP descartadas con motivo en §5).
> **SKILLS_CARGADAS:** ci-cd-and-automation, systematic-debugging (+ base campaign-executor, progreso, ponytail full)
> **Referencias:** `.opencode/rules/release-ci.md` (leída COMPLETA 42L — aplican regla 2 — sccache vía `rust-setup`, sin duplicar steps — y regla 5 — 0 `continue-on-error` en el survivor; 1/3/4 N/A: no toca allocators/Dockerfile/version-sync), Notion N/A (tarea CI mecánica observacional, sin mapeo a Problema/Propuesta/hijas — filtro VantaDB: nada aplica).
> **Regla 11:** N/A justificado — 0 claims de performance, 0 números, 0 adjetivos de rendimiento en este diagnóstico.
> **Research Digest:** sin digest previo (DISCOVERY inline: 1 workflow leído + GH API solo-GET).

## Metadata

- **Plan file:** `docs/plans/2026-09-21-workflows-repair.md` (Wave 1, línea 41-44)
- **Fuente:** plan Wave 1 + descripción del usuario (flake "Generate API reference (rustdoc)" 0-1s, BlobNotFound al traer logs)
- **Esfuerzo:** 🟢 1h · **Prioridad:** 🟢 · **Tipo:** CI/diagnóstico observacional (devops, bug-fix workflow)
- **Turns estimados:** 4-6
- **Creado:** 2026-09-22 · **last-synced:** 2026-09-22
- **Estado:** ⏳ IN PROGRESS → ✅ COMPLETED
- **Incógnitas (uphill):** 0 (causa identificada con evidencia GH, §8)
- **Pendientes (downhill):** 0 (S1-S4 ✅; sin fix que aplicar por diseño)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | pushes a `main`/`develop` + PRs cuyos paths matcheen `ci-rustdoc.yml` (`src/**`, `tests/**`, manifests, `vantadb-*/**`, `vanta-memory/**`, `providers/**`, self-path) |
| Callees | `.github/actions/rust-setup` (composite), `cargo doc --no-deps --workspace --all-features --document-private-items`, `actions/upload-artifact@v4.6.2` |
| Implicaciones | NINGUNA — diagnóstico read-only: 0 archivos editados fuera de este task file; survivor `ci-rustdoc.yml` intacto (dueño FIND-137); concurrency `rustdoc-${{ github.ref }}` + `cancel-in-progress: true` opera by-design |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rustdoc.yml` (110L — triggers `:23-46`, concurrency `:54-56`, job `Generate API reference (rustdoc)` `:59-60`, `cargo doc` `:87-88`, artifact tar.gz 30d `:95-101`, summary `:103-110`), `docs/tasks/FIND-137.md` (contexto survivor + canibalización previa §8 fila 7 + investigation notes), `.opencode/rules/release-ci.md` (42L), plan file Wave 1 (`:41-44` + deuda conocida `:99`).
- **Archivos referenciados hacia dentro (del observado):** `.github/actions/rust-setup` (única dependencia; no se toca).
- **Archivos que referencian al observado (referencias entrantes):** plan file (solo recitation), FIND-137 (preserva el scope de este diagnóstico), Backlog/plan (prohibidos tocar, orquestador). 0 consumidores del artifact (verificado en FIND-137 §6).
- **Operaciones GH (solo GET/lectura):** `gh run list --workflow=ci-rustdoc.yml`, `gh run view <id> --json jobs,...`, `gh run view <id> --log`, `gh api repos/ness-e/Vantadb/actions/runs/<id>(/jobs)` — 0 mutaciones (sin re-run, sin cancel, sin dispatch).
- **Veredicto impacto:** 1 archivo creado (`docs/tasks/FIND-138.md`) + 0 edits. Riesgo 🟢 nulo. **Gate D:** no dispara — ≤10 archivos, sin hot path/API pública/símbolos nuevos, contrato de observación.

## Contrato

"Diagnóstico del flake + fix si es nuestro o deuda escrita si es externo; 3 runs verdes o causa declarada; actionlint 0 si se toca YAML"

- [x] (a) Causa identificada con evidencia (§8: tabla de runs + `jobs:[]` + log vacío + concurrency survivor `:54-56`)
- [x] (b) Fix si es nuestra o deuda escrita si es externa → EXTERNA/by-design → deuda escrita (este task file + recitation; sin diff YAML)
- [x] (c) 3 runs verdes o causa declarada → AMBOS: 6 verdes citados (§8) + causa declarada
- [x] (d) actionlint 0 si se toca YAML → N/A (0 YAML tocados) + actionlint read-only del survivor exit 0 como testigo (§6)

## Spec (SDD — Phase 1b)

N/A justificado con evidencia (no `N/A` vacío): el diagnóstico NO agrega símbolos/contratos públicos nuevos — 0 `pub fn`/struct/enum, 0 tools MCP, 0 endpoints, 0 métodos de binding, 0 componentes `web/` (solo lectura YAML + GH API GET). Sin spec que escribir; el contrato de observación arriba es la especificación. Gate spec-first N/A.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** survivor `ci-rustdoc.yml` intacto (dueño FIND-137 — si el fix exigiera editarlo: STOP + diff propuesto en BLOQUEO sin aplicar; NO ocurrió); `rustdoc-70.yml` sigue eliminado; resto workflows intactos (FIND-139 `ci-gate`/`gate-docs-21` paralelo); `src/`, `web/`, `desktop/`, locks, plans, Backlog intactos; secretos ni leídos; NO PUSH (pushea solo vanta-lead).
- **Comandos de verificación:** `gh run list --workflow=ci-rustdoc.yml --limit 30` (verdes + cancelados) · `gh run view <id> --json jobs,conclusion,status` (`jobs:[]` en muertes 1s) · `gh run view <id> --log` (vacío) · `actionlint .github/workflows/ci-rustdoc.yml` (exit 0, read-only) · `git diff --check` (limpio) · `git status --short` (solo este task file)
- **Deuda pendiente:** registrada abajo (Regla 6 + §8 veredicto): el ruido de cancels es by-design; opcional futuro (dueño del survivor) documentado sin aplicar.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# FIND-138 — Diagnosticar flake "Generate API reference (rustdoc)" 0-1s sin log` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia §8 |
| `nextTask` | Wave 2 (orquestador) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** CERO — este diff no introduce deuda (solo añade este task file de diagnóstico). Deuda EXTERNA declarada (no nuestra, sin fix):

- **Deuda escrita (causa externa/by-design):** las muertes "0-1s sin log + BlobNotFound" son cancelaciones `cancel-in-progress` del concurrency group `rustdoc-${{ github.ref }}` operando según diseño (convención del repo, FIND-137 §8 fila 7), NO un bug del workflow ni del comando `cargo doc` (6 runs verdes lo prueban). El ruido visual (runs cancelados en la pestaña Actions + `gh run view --log` vacío + BlobNotFound en UI al pedir logs de un run que nunca arrancó) es comportamiento de la plataforma GitHub, fuera de nuestro control.
- **Opción futura (NO aplicada, dueña FIND-137/survivor):** si el owner quiere menos ruido en batches dependabot, podría evaluarse `concurrency.cancel-in-progress` por evento o agrupar pushes — se registra como idea, SIN diff propuesto (ningún cambio es necesario para corrección; el STOP condicional del alcance no se activó porque no hay fix que exigir).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato verificable por comando (GH GET + actionlint read-only + diff-check) |
| **Commit** | Atómico (solo task file), `docs:` + task ID, `git diff` limpio de ajenos, verificación mecánica (nunca auto-reporte) |
| **Release** | N/A justificado (`docs:` no publica — release-plz lo ignora; sin bump/semver) |

## Herramientas necesarias

- `gh run list` / `gh run view` / `gh api .../actions/runs/...` (SOLO GET — observación; prohibidos `gh run cancel/rerun/watch`, `workflow_dispatch` POST)
- `actionlint` (testigo read-only del survivor — exit 0 obligatorio como evidencia de que el YAML está sano)
- `git diff --check` + `git status --short` (higiene pre-commit)
- `campaign_update_task_state` (in-progress → completed + recitation) + `campaign_memory_write` (1 lesson)
- `skill progreso` al cierre (Trigger 1 — Backlog NO se toca: race paralelo, orquestador)

**Skills cargadas (SDP):** ci-cd-and-automation (pipelines/quality-gates del área + feedback-loop de CI) · systematic-debugging (Iron Law: root-cause Phase 1 antes de proponer fix; multi-component evidence gathering en §8) · base campaign-executor/progreso/ponytail-full.

## Investigation Notes

- systematic-debugging Phase 1 (sin saltos): leer error (0-1s + sin log + BlobNotFound) → reproducir patrón (8 cancelados, 5 en 17s) → cambios recientes (FIND-137 unificó gemelos el 2026-09-22, commit `4b0686b0`) → evidencia por capa (run → jobs[] → log vacío → concurrency YAML) → trace al origen (grupo `rustdoc-develop` + batch dependabot).
- Pre-FIND-137 había DOS workflows con el MISMO `concurrency.group` (`ci-rustdoc.yml` top-level + `rustdoc-70.yml` job-level) → canibalización CRUZADA: las "2 muertes sin log" originales del plan eran gemelos matándose (FIND-137 §7 + investigation notes). Post-merge queda UN solo workflow → los cancels restantes son INTRA-workflow y correctos (superseded pushes).
- `gh run view <1s-id> --log` retorna vacío con exit 0 (no hay log porque el job nunca arrancó: `jobs:[]`). BlobNotFound = la UI/API de logs pide un blob que nunca se creó — síntoma, no causa.
- El run `35692290778` (12m40s, cancelled tardío) prueba que el job SÍ trabaja cuando no es superseded antes de arrancar: `started_at 05:54:34 → completed_at 06:03:12`, cancelado por el batch `06:02:56`. Comportamiento documentado de `cancel-in-progress`.
- 6 runs verdes (lista §8) prueban `cargo doc --no-deps --workspace --all-features --document-private-items` sano + artifact + upload sanos. actionlint read-only exit 0 confirma YAML sano.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — causa identificada con evidencia GH (§8) |
| Pendientes de ejecución (downhill) | 0 — S1-S4 ✅ (observación + veredicto + commit task file) |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A justificado: 0 trust boundaries tocados (0 ediciones; solo GH API GET público del repo + lectura YAML); `permissions: contents: read` del survivor verificado intacto; sin secrets en diff (task file solo, verificado pre-commit).
- [x] **PERFORMANCE** — N/A justificado: 0 hot paths (`vector/`, `engine.rs`, serialización) tocados; duraciones citadas (21m/12m/6m/4m) son evidencia observada, no claims (Regla 11 N/A).

## Steps

### Step 1: DISCOVERY + crear este task file (Regla 0, steps atómicos, contrato)

- **Archivos:** `docs/tasks/FIND-138.md` (NUEVO)
- **Acción:** leer survivor entero + FIND-137 + release-ci + plan Wave 1; poblar Impacto mapeado (Regla 0) ANTES de cualquier edición; SDP vía `campaign_discover_skills_v2` + carga de skills; steps atómicos abajo
- **Verify:** este archivo existe con 10 bloques + Regla 0 llena + contrato (a)(b)(c)(d)
- **Estado:** ✅ DONE (2026-09-22)

### Step 2: Observación GH solo-GET (runs, jobs, logs) — systematic Phase 1

- **Archivos:** ninguno (solo lectura remota; prohibido editar workflows)
- **Acción:** `gh run list --workflow=ci-rustdoc.yml --limit 30` (verdes vs cancelados); `gh run view --json jobs` en 2 muertes 1s (`jobs:[]`); `gh run view --log` en 1s (vacío); `gh api .../runs/<id>/jobs` en el cancelado largo (started/completed + conclusion); mapear timestamps del batch 05:50:16→05:50:33 vs concurrency survivor `:54-56`
- **Verify:** tabla §8 con IDs, timestamps, conclusiones y duraciones citables
- **Estado:** ✅ DONE (2026-09-22 — evidencia §8)

### Step 3: Veredicto + deuda escrita (systematic Phase 2-3: hipótesis única, sin fix-síntoma)

- **Archivos:** `docs/tasks/FIND-138.md` (§8 + Deuda técnica)
- **Acción:** hipótesis única "cancel-in-progress by-design, no flake" → confirmada por `jobs:[]` + log vacío + group compartido + verdes; declarar causa externa; registrar deuda; NO proponer diff del survivor (STOP condicional no activado: no hay fix que exigir)
- **Verify:** contrato (a)(b)(c) marcado ✅ con evidencia; (d) N/A + testigo actionlint
- **Estado:** ✅ DONE (2026-09-22)

### Step 4: Commit `docs:` + cierre (recitation + lesson + RESULTADO)

- **Archivos:** staging selectivo (SOLO `docs/tasks/FIND-138.md`)
- **Acción:** `git add docs/tasks/FIND-138.md` + commit `docs: FIND-138 — ...` (NO PUSH) + `campaign_update_task_state` completed + 1 lesson + bloque RESULTADO §7
- **Verify:** hash commit existe; `git status` sin propios pendientes; RESULTADO con GATES_EVALUADOS + SKILLS_CARGADAS
- **Estado:** ✅ DONE (2026-09-22)

## Dependencias

- **Wave:** Wave 1 (paralelo ×3 disjunto con FIND-137/139 — 0 archivos compartidos en edición; este task no edita nada)
- **Bloqueantes:** ninguno
- **Paralelas:** FIND-137 (survivor dueño del YAML observado — ya commiteado `4b0686b0`, no interfiere), FIND-139 (`ci-gate.yml`/`gate-docs-21.yml` — no tocados aquí)
- **Previa:** Wave 0 ✅ (134/135/136 per plan `:5`)
- **Next:** Wave 2 (orquestador)

## Review (GATE — agente distinto, P2-01)

> Non-interactive (`/pipeline task`) → self-review systematic + doubt-driven reconciliado abajo (degradado declarado, cross-model skipped: non-interactive context). Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** systematic-debugging (self, degradado — sin sub-agente disponible en este contexto) + lente doubt-driven sobre el veredicto
- **Enfoque:** ¿la evidencia distingue cancel-by-design de flake real? ¿`jobs:[]` + log vacío + group + verdes cierran la hipótesis sin alternativa viva? ¿algún run rojo/failed que contradiga (fallo real de `cargo doc`)?
- **Cómo se probó:** GH API GET (mecánico, no auto-reporte) + actionlint read-only exit 0 + `git diff --check` — ver §6/§10
- **CLAIM/DOUBT:** CLAIM "es cancel-in-progress, no flake" — DOUBT "¿y si `cargo doc` falla intermitente por OOM/bindgen?" → refutado: 0 runs `failure` en la ventana (solo `success`/`cancelled`); el cancelado largo corrió 12m40s (build real, no muerte súbita); timeout 30 con margen FIND-135/137. DOUBT "¿y si el artifact upload falla?" → refutado: `if-no-files-found: error` + 6 artifacts subidos en verdes. DOUBT "¿BlobNotFound indica corrupción de artifact?" → refutado: es el endpoint de LOGS del run (jobs vacíos), no el artifact.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos no ejecutados (todas las citas §8 son output real)
  - [x] No saltarse clarificación (contrato de observación, 0 incógnitas)
  - [x] No declarar done sin verify contra acceptance criteria (a)(b)(c)(d) verificados)
  - [x] No ignorar fallos ni reportar "todo OK" parcial (cancelados listados todos, no solo verdes)
  - [x] No dar por saturada la búsqueda (runs + jobs + logs + YAML + FIND-137)
  - [x] No copiar sin citar (cada fila §8 cita ID/timestamp/fuente)
  - [x] No reintentar en bucle sin diagnóstico (0 reintentos: diagnóstico en 1 pasada)
  - [x] No dejar huérfanos los pasos (cada step → contrato)
  - [x] No degradar chequeo de errores (N/A — sin paths de dinero/seguridad)
  - [x] No gastar presupuesto infinito (1 slice observación, 0 edits YAML)
- **Veredicto:** ✅ PASS (degradado declarado) — veredicto sostiene con evidencia; sin fix que revisar.

## Notas

- Ponytail full: 1 archivo creado + 0 edits + 0 re-runs + 0 dispatches (observación pura, no omitida: GH GET ×6 + actionlint read-only + diff-check + status).
- El STOP condicional del alcance ("si el fix exige editar el survivor, registra el diff en BLOQUEO sin aplicar") NO se activó: no hay fix que exigir porque la causa es by-design. Ningún diff propuesto existe.
- Gate C: sin colaterales (observación pura; `git status` pre-commit solo muestra este task file + plan file modificado por recitations server ajenas — NO commitear el plan file).

## 5. SKILLS — lista SDP (≤8) con 1 línea de cuándo aplica c/u

SDP `campaign_discover_skills_v2` phase=BUILD taskType=bug-fix devolvió 8 (base + lifecycle, keyword-mapped 0):

1. `campaign-executor` — base: state machine PLAN→ACT→VERIFY + RESULTADO + recitation (siempre en pipeline-full).
2. `ci-cd-and-automation` — base del área: pipelines, quality gates, feedback-loop CI→agente, artifact retention (cargada).
3. `doubt-driven-development` — base CI/CD: CLAIM/DOUBT adversarial sobre el veredicto (aplica como lente en §Review, sin carga separada).
4. `incremental-implementation` — lifecycle BUILD: slices delgados (aplica como S1→S4 observacionales, sin carga separada).
5. `test-driven-development` — lifecycle BUILD: Red-Green — NO aplica (0 lógica; la evidencia GH sustituye RED: runs verdes = test existente en verde).
6. `context-engineering` — lifecycle BUILD: empaquetar contexto (plan + survivor + FIND-137) — aplica en DISCOVERY (este archivo lo materializa, sin carga separada).
7. `source-driven-development` — lifecycle BUILD: sintaxis/entidades estables verificables intra-repo — aplica sin carga (concurrency/actionlint estables, 0 ambigüedad externa).
8. `frontend-ui-engineering` — lifecycle BUILD: UI en `web/` — NO aplica (0 archivos web; descartada).
- Extra justificada fuera de SDP: `systematic-debugging` (bug: Iron Law root-cause Phase 1 antes de fix; multi-component evidence gathering run→jobs→logs→YAML — cargada).

## 6. HERRAMIENTAS+MCP — comandos exactos + MCP (sin grep-loop innecesario)

```powershell
# Observación GH (solo GET — ejecutados 2026-09-22, outputs en §8):
gh run list --workflow=ci-rustdoc.yml --limit 15
gh run list --workflow=ci-rustdoc.yml --limit 30 --json databaseId,conclusion,createdAt,headBranch,event
gh run view 35693124217 --json jobs,conclusion,status,headBranch,event,createdAt,updatedAt,workflowName
gh run view 35692285809 --json jobs,conclusion,status
gh run view 35692285809 --log                                            # vacío, exit 0 (sin log útil)
gh api repos/ness-e/Vantadb/actions/runs/35693124217/jobs               # {} (sin jobs)
gh api repos/ness-e/Vantadb/actions/runs/35692290778/jobs --jq '.jobs[] | {name, conclusion, started_at, completed_at}'
gh api repos/ness-e/Vantadb/actions/runs/35692290778 --jq '{conclusion, event, head_branch, created_at, updated_at}'

# Testigo YAML sano (read-only, sin edición):
actionlint .github/workflows/ci-rustdoc.yml                              # exit 0

# Higiene pre-commit:
git diff --check                                                         # limpio
git status --short                                                       # solo este task file (+ plan file server, no commitear)
git add docs/tasks/FIND-138.md
git commit -m "docs: FIND-138 — ..."                                     # NO PUSH
```

- **MCP:** `campaign_detect_task_type` ✅ (devops/CI-CD) · `campaign_discover_skills_v2` ✅ (8 skills, §5) · `campaign_get_workflow` ✅ (bug-fix → localizing/planning/implementing/testing/review/accept/close; aplicado como localizar→planificar→observar→veredicto→cierre) · `campaign_get_next_task` ✅ (planFile explícito; server trackea FIND-139 in-progress, FIND-138 file-based) · `campaign_verify_cmd` (no requerido: 0 edits YAML/código; verificado vía bash directa) · `campaign_update_task_state` (cierre) · `campaign_memory_write` (1 lesson).
- **Sin grep-loop innecesario:** 0 greps (FIND-137 ya verificó 0 refs rotas/badges/consumidores; este task no re-verifica lo ajeno).

## 7. INVESTIGACIÓN CÓDIGO — blast radius (DISCOVERY)

- **Triggers hoy (survivor `ci-rustdoc.yml:23-46`):** push `[main, develop]` + PR `[main, develop]` con paths de código + self-path + `workflow_dispatch`. Batch dependabot del 2026-09-22 05:50 UTC = 5 pushes a `develop` en 17s (todos matchean paths) → 5 runs mismo group `rustdoc-develop`.
- **Concurrency (`:54-56`):** `group: rustdoc-${{ github.ref }}` + `cancel-in-progress: true` = cada push nuevo cancela los runs previos del mismo ref. Convención del repo (FIND-137 §8 fila 7: mismo patrón en `ci-rust-10`, `gate-docs-21`).
- **Job (`:58-110`):** `Generate API reference (rustdoc)` en `ubuntu-latest`, timeout 30, `rust-setup` (stable + system-deps para bindgen rocksdb), `cargo doc --no-deps --workspace --all-features --document-private-items`, tar.gz, upload-artifact 30d, summary. Nada en el job explica muertes 0-1s (el paso más rápido — checkout — tarda >5s).
- **Riesgo:** N/A — 0 cambios. Riesgo 🟢.
- **Veredicto:** blast radius = 1 creado + 0 editados. Sin API pública, sin símbolos nuevos, sin hot path, sin `cargo build`.

## 8. INVESTIGACIÓN PROBLEMA — evidencia + veredicto (sin opinión)

### 8.1 Runs cancelados 0-7s (las "muertes sin log")

| Run ID | Created (UTC) | Event/ref | Conclusión | Duración | Jobs |
|--------|---------------|-----------|------------|----------|------|
| 35693124217 | 2026-09-22 06:02:56 | push develop (rcgen bump #179) | cancelled | 1s | `[]` (vacío — nunca arrancó) |
| 35692285809 | 2026-09-22 05:50:29 | push develop (rocksdb #175) | cancelled | 1s | `[]` |
| 35692280123 | 2026-09-22 05:50:24 | push develop (tower-http #176) | cancelled | 2s | `[]` (misma firma que sus gemelos) |
| 35692275871 | 2026-09-22 05:50:21 | push develop (tokenizers #177) | cancelled | 1s | `[]` |
| 35692270604 | 2026-09-22 05:50:16 | push develop (mach2 #178) | cancelled | 1s | `[]` |
| 35689485935 | 2026-09-22 05:07:37 | push develop (rust-minor group) | cancelled | 3s | `[]` |
| 35689479737 | 2026-09-22 05:07:31 | push develop (rust-patch group) | cancelled | 7s | `[]` |
| 35692290778 | 2026-09-22 05:50:33 | push develop (toml #174) | cancelled | 12m40s | `Generate API reference (rustdoc)`: started 05:54:34 → completed 06:03:12, conclusion cancelled (cancelado TARDÍO por el batch 06:02:56) |

Fuente: `gh run list --workflow=ci-rustdoc.yml` + `gh run view <id> --json jobs` + `gh api .../runs/<id>/jobs` (2026-09-22).

### 8.2 Log ausente (el "sin log útil" + BlobNotFound)

- `gh run view 35692285809 --log` → vacío, exit 0. `gh api .../runs/35693124217/jobs` → `{}` (sin jobs).
- Explicación: sin job no hay steps, sin steps no hay logs, sin logs no hay blob → la UI/API de logs responde BlobNotFound. El BlobNotFound es el SÍNTOMA (endpoint de logs), no la causa.
- El run largo (35692290778) SÍ tiene job con timestamps reales → cuando el run sobrevive a la cancelación temprana, trabaja normal 12m40s.

### 8.3 Runs verdes (el comando y el artifact están sanos)

| Run ID | Fecha | Evento/ref | Duración |
|--------|-------|------------|----------|
| 35693192386 | 2026-09-22 06:03:54 | push main (js-yaml #194) | 21m34s ✅ |
| 35686343573 | 2026-09-22 04:17:45 | push develop (maturin #194) | 4m33s ✅ |
| 35681924853 | 2026-09-22 03:06:17 | PR dependabot js-yaml | 6m45s ✅ |
| 35664428750 | 2026-09-21 22:47:04 | push main (vitest #196) | 1m57s ✅ |
| 35661620658 | 2026-09-21 22:14:27 | PR dependabot multi | ✅ |
| 35648431157 | 2026-09-21 08:01:04 | push main | ✅ |

Fuente: `gh run list --workflow=ci-rustdoc.yml --limit 30` (2026-09-22). Criterio (c): 6 verdes > 3 exigidos.

### 8.4 Concurrency del survivor (la causa)

`ci-rustdoc.yml:54-56` — `group: rustdoc-${{ github.ref }}` + `cancel-in-progress: true`. Los 5 pushes 05:50:16→05:50:33 comparten ref `develop` → mismo grupo → cada push cancela al anterior antes de que arranque (1-2s, `jobs:[]`). El batch 06:02:56 cancela incluso al run que llevaba 12m corriendo. Todo según diseño.

### 8.5 Veredicto

**Causa EXTERNA/by-design (plataforma GitHub operando según lo configurado), no flake del workflow:** `cancel-in-progress` del grupo `rustdoc-develop` cancela runs superseded — con `jobs:[]`, log vacío y BlobNotFound en la UI como consecuencias esperadas. El workflow, el comando `cargo doc`, el artifact y el YAML están sanos (6 verdes + actionlint exit 0). **Sin fix.** Deuda escrita en `## Deuda técnica`. STOP condicional (editar survivor) no activado.

## 9. INVESTIGACIÓN INTERNET — N/A

0 ambigüedad de APIs externas: `concurrency.cancel-in-progress`, `jobs:[]` en runs cancelados pre-arranque y BlobNotFound de logs inexistentes son comportamiento documentado y estable de GitHub Actions, verificable intra-repo (patrón repetido en `ci-rust-10`, `gate-docs-21` + FIND-137 §8). 0 citas externas, 0 claims → GATE CITAS TSYS-13 N/A.

## 10. VALIDACIÓN+CIERRE — verify + OCR + DoD + reviewer + Gates + commit

- **Verify contrato:** GH GET ×8 (§6) + actionlint read-only exit 0 + `git diff --check` limpio + `git status` (solo task file propio). `campaign_verify_cmd` no requerido (0 edits YAML/código).
- **Verify full relevante:** YAML/código/docs-producto no tocados → `cargo fmt/clippy/nextest` N/A; `scripts/validate-docs-coverage.ps1` N/A (task file diagnóstico, bypass de cobertura como FIND-137); OCR `pwsh dev-tools/ocr-review.ps1` advisory sobre el task file si disponible (sin código que bloquee). `/cleanCA` N/A (0 código).
- **DoD 3 niveles:** (1) task: contrato (a)(b)(c)(d) mecánico ✅; (2) commit: 1 cosa lógica (`docs:` + ID), porqué en mensaje, sin secrets (task file sin credenciales; `grep -i` pre-commit), sin formatting mezclado; (3) release N/A (`docs:` no publica; reversible por `git revert` — rollback §2b del plan).
- **Reviewer P2-01:** systematic + doubt-driven degradado (arriba §Review, veredicto ✅ PASS).
- **Gates D/V/C:** D no dispara (Regla 0: 1 creado + 0 editados, 0 símbolos/API/hot-path, contrato observacional); V no dispara (0 fallas verify — todo GET exitoso a la primera); C: sin colaterales (observación pura; `git status` con `M docs/plans/...` es recitation server ajena — NO commitear).
- **Cierre:** staging selectivo (SOLO task file) + commit `docs: FIND-138 — ...` (NO PUSH) + `campaign_update_task_state` completed + recitation canónica + 1 lesson + Context Save Point + bloque RESULTADO §7 (siempre; nunca silencio). Backlog→avance NO tocar (race paralelo, orquestador).

## Pre-mortem (del plan — verificado en DISCOVERY)

1. Editar el survivor por accidente → mitigado: 0 ediciones (solo lectura + actionlint read-only); `git status` pre-commit confirma 1 solo path propio.
2. Confundir cancel-by-design con flake real → mitigado: hipótesis única refutada contra 3 DOUBTs (§Review) + 6 verdes + 0 `failure` en ventana.
3. FIND-137/139 en paralelo tocan mi alcance → plan garantiza disjuntos; este task no escribe fuera de su file.

## Context Save Point

- **Branch:** `develop` · **Steps:** S1 ✅, S2 ✅, S3 ✅, S4 ✅ (commit + cierre) · **Próximo:** ninguno — tarea completa.
- **Verify obtenido:** GH GET ×8 con evidencia §8 · actionlint survivor exit 0 (read-only) · diff-check limpio · status con 1 path propio.
- **Commit:** `docs: FIND-138 — diagnostico flake rustdoc (cancel-in-progress by-design, sin fix)` (NO PUSH).

## Scope previo

N/A (task file nuevo, sin legacy).
