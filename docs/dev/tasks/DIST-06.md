---
title: "TASK DIST-06: Estrategia de los 11 crates publish = false"
kind: task
description: "Decisión por crate (publicar/no/canal + motivo) en PUBLISH.md §crates + invariante verificable con release-plz.toml. Doc-only; sin cambios de versión."
---

# TASK DIST-06: Estrategia de los 11 crates `publish = false`

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 19, F0)
- **Fuente:** DELTA 2026-09-30 — la ambigüedad "qué se publica y por dónde" costó FIND-230; 11 crates `publish = false` (verificado 2026-09-30; hoy **10** post-DIST-01).
- **Esfuerzo:** 🟢 1d | **Appetite:** max 1d | **Prioridad:** 🟡
- **Tipo:** Release/Packaging (docs; sin código, sin cambios de manifest)
- **Creado:** 2026-10-04T13:05Z | **last-synced:** 2026-10-04T13:05Z
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE — `ses_ef9145f3fffezcgeYmCTiLD2Pt`; commit local `docs:`, sin push)
- **Campaign ID:** master-plan-0.9.0-20261004
- **Ejecutor:** vanta-lead (dominio release/packaging)
- **Incógnitas (uphill):** 0 abiertas (todas resueltas en DISCOVERY — ver Investigation Notes)
- **Pendientes (downhill):** 0 — steps 2-4 ✅ (cierre completo)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Release drivers leen `PUBLISH.md` (17 referrers: RULES.md, llms.txt, check-npm-versions.mjs, verify-release.ps1, DIST-01/05, FIND-142/143/144/230…); `release-plz.toml` ← `.github/workflows/release.yml` + PUBLISH.md + vanta-memory docs (solo lectura, NO se modifica) |
| Callees | Los 10 `Cargo.toml` con `publish = false` (solo lectura), `release-plz.toml` (solo lectura), workflows de release (release-wheels/npm/npm-node/adapters/binaries), Freeze List (EXPERIMENTAL_FEATURES.md) |
| Implicaciones | Doc-only aditivo: sección nueva en PUBLISH.md. Sin cambio de comportamiento, versiones, release-plz ni manifests. Los comandos del invariante quedan como verificación re-ejecutable. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/workflow/PUBLISH.md` (192L, fresco post-DIST-05), `release-plz.toml` (76L), los 11 Cargo.toml del alcance: `vanta-memory` (80L), `vanta-proxy` (40L), `vantadb-mcp` (54L), `vantadb-python` (33L), `vantadb-server` (79L), `vantadb-node` (38L), `vantadb-ffi-core` (15L), `fuzz` (48L), `providers/{openai,ollama,litellm}` (23L c/u); root `Cargo.toml:760-814` (workspace members/exclude/default-members) + sección `[package] exclude`; `.opencode/rules/release-ci.md` (42L); `docs/dev/tasks/DIST-01.md` + `DIST-05.md`; `.github/workflows/release-binaries.yml`, `providers-ci.yml`, `release-adapters.yml`; `scripts/docs/check-npm-versions.mjs` (TARGETS); `docs/dev/strategy/DISTRIBUTION.md`; `docs/user/operations/EXPERIMENTAL_FEATURES.md` §Freeze List (owner 2026-10-01); sección 0.8.0 de `docs/CHANGELOG.md`.
- **Archivos referenciados hacia dentro (imports/deps):** `vantadb-mcp` ← `vantadb-server/Cargo.toml:27` (path dep; root crate NO depende — solo lo excluye del package en `Cargo.toml:50,54`); `vantadb-ffi-core` ← `vantadb-python:26`, `vantadb-node:26`, `vantadb-wasm:23`; `vanta-memory` ← `vantadb-mcp:27`, `vanta-proxy:33`, `vantadb-python:30`; `vantadb-server` ← binarios del release; `vantadb-node`/`providers/*`/`fuzz` = workspaces aislados/excluidos (no vistos por release-plz).
- **Archivos que referencian a los editados (referencias entrantes):** `PUBLISH.md` ← 17 archivos (ver Blast Radius) — gana una sección; ningún consumidor parsea su contenido (links/texto). `release-plz.toml` NO se edita. `docs/pipeline-state.json` referrer NO se toca (prohibido).
- **Veredicto impacto:** **BAJO** — aditivo puro: 1 sección nueva en 1 doc. Sin consumidores rotos, sin cambios de comportamiento. Los 2 comandos del invariante son re-ejecutables (rg sobre manifests + release-plz.toml).

## Contrato

> Del plan (Task 19). Doc-only; **sin cambios de versión ni de release-plz.toml** (pre-mortem #2: churn mínimo).

1. **Decisión documentada POR crate** (publicar/no publicar/canal + motivo) en `PUBLISH.md` §crates para los 11 del alcance: 10 con `publish = false` hoy + `vanta-memory` (referencia a DIST-01, sin duplicar) — ⬜ Step 2
2. **Invariante de consistencia con `release-plz.toml` verificable por comando** — ⬜ Step 3:
   - (a) `rg -n "^\s*publish = false" --glob "**/Cargo.toml"` → exactamente las 10 filas de la tabla
   - (b) `rg -n "^\[\[package\]\]|^name = |^\s*(release|publish) = " release-plz.toml` → solo `vantadb` / `vantadb-wasm` / `vanta-memory`
3. **Política revisable con fecha** (pre-mortem #1): reviewed 2026-10-04 + review triggers (1.0.0 freeze lift, primer npm publish de `vantadb-node`, decisión PyPI de providers, bootstrap `vanta-memory`) — ⬜ Step 2
4. **Gates docs verdes**: `check-links.mjs` + `check-docs.mjs` + `gen-index.mjs --check` → 0 — ⬜ Step 3

## Spec (SDD — decisiones de release, no feature-add)

> Phase 1b: no agrega símbolos públicos de producto (1 sección de doc). Se llena igual: hay decisiones de release con evidencia.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive la decisión | A) **tabla por crate en `PUBLISH.md` §crates** (contrato del plan; 1 lugar, junto al flujo de release) / B) ADR (sobre-proceso: es política operativa, no arquitectura) / C) comentarios por manifest (disperso — no elimina la ambigüedad) | A | ✅ decidido-por-evidencia: contrato Task 19 lo fija; FIND-230 nació de ambigüedad en docs de release |
| 2 | ¿Cambiar manifests/release-plz? | A) **no — documentar el estado real** (los canales ya existen y funcionan; tocar `publish` rompería el build de release) / B) promover algunos crates a crates.io (sin consumidores externos → mantenimiento sin beneficio) | A | ✅ decidido-por-evidencia: Freeze List 2026-10-01 (proxy: no publicar hasta 1.0.0), READMEs de providers (build local), workflows reales (PyPI/npm/binaries), `cargo publish` imposible con `publish = false` |
| 3 | Tratamiento de `vanta-memory` | A) **referencia a DIST-01** (fila en tabla + remisión) / B) re-decidir (duplica el trabajo ya aprobado) | A | ✅ decidido-por-evidencia: pre-mortem #3 + DIST-01 ✅ (publishable + hold `release=false`) |
| 4 | ¿Incluir `vantadb` + `vantadb-wasm` (publishables, no en los 11)? | A) **sí, mini-tabla de contexto** (el invariante con release-plz.toml los involucra) / B) no (invariante incompleto) | A | ✅ decidido-por-evidencia: CMD (b) devuelve 3 overrides, 2 son crates publishables; sin ellos la tabla no explica el workspace |
| 5 | ¿Gate mecánico nuevo (script)? | A) **no — 2 comandos `rg` documentados** (el contrato pide "verificable por comando/inspección"; un script es scope creep) / B) script + CI gate (over-engineering; ponytail ladder) | A | ✅ decidido-por-evidencia: contrato del plan ("verificable por inspección + dry-run de los que pasen a publicar"); ningún crate pasa a publicar → sin dry-run nuevo |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) **sin push** (política owner: push diferido al final del plan) y sin tags; (2) `release-plz.toml` / versiones / `docs/CHANGELOG.md` intactos (solo lectura); (3) WIP ajeno intacto (`opencode.jsonc`, plan file, `docs/pipeline-state.json`); (4) los comandos (a)/(b) del contrato deben seguir dando **10 filas / 3 overrides** — si otro cambio los altera, la tabla §crates debe actualizarse en el mismo PR; (5) los canales documentados son los reales HOY (no promesas).
- **Comandos de verificación:** los 4 del Contrato (2 rg + 3 gates docs; ver Step 3).
- **Deuda pendiente:** ninguna. La política es revisable por diseño (fecha + triggers en el doc).

## Deuda técnica (Regla 6)

Sin deuda. Doc-only, sin código nuevo ni cambios de config.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-4 ✅ (tabla completa + invariante por comando + política fechada + gates docs). |
| **Commit** | Commit atómico `docs:` (conventional), verificación mecánica (nunca auto-reporte), sin archivos fuera del blast radius. |
| **Release** | n/a — doc de política interna; no publica ni cambia versiones (justificado en Notas). |

## Herramientas necesarias

- `rg` (inventario + comandos del invariante) · `cargo metadata` (members; solo lectura)
- `node scripts/docs/check-links.mjs` / `check-docs.mjs` / `gen-index.mjs` — gates docs
- `campaign_verify_cmd` / `campaign_update_task_state` — verificación y estado
- `pwsh dev-tools/ocr-review.ps1` — OCR delegation (cierre)
- `codegraph_codegraph_explore` / `codebase-memory-mcp` (coverage del índice)

**Skills cargadas (SDP):** `ci-cd-and-automation` (pinned CI/release) · `git-workflow-and-versioning` (pinned) · `security-and-hardening` (pinned) · `source-driven-development` (base) · `documentation-skill` (obligatoria: edita docs/) · `documentation-and-adrs` (decisión documentada) · `shipping-and-launch` (canales de distribución/rollout). Base auto: campaign-executor · progreso. SDP v3 detectó "Python SDK" por keywords de filename (falso positivo — tarea docs/release); se excluyeron `test-driven-development`/`incremental-implementation` (sin código).

## Investigation Notes (DISCOVERY — 2026-10-04)

1. **Inventario real (hoy):** `rg "^\s*publish = false"` → **10 crates**: `vantadb-mcp`, `vantadb-ffi-core`, `vantadb-server`, `vantadb-python`, `vanta-proxy` (workspace members); `vantadb-node` (standalone, no member); `providers/{openai,ollama,litellm}` (no members, por MSVC linker crash); `fuzz` (excluido del workspace). + `vanta-memory` (era el 11º; DIST-01 lo hizo publicable). El plan lista 11 (verificado 2026-09-30, pre-DIST-01).
2. **Canales reales por crate (verificados):** `vantadb-python` → PyPI `vantadb-py` wheels (`release-wheels.yml`, tag `v*.*.*`, OIDC); `vantadb-server` → binarios GitHub Release (`release-binaries.yml`, 5 targets, `vanta-cli` + `vantadb-server`); `vantadb-node` → npm `vantadb-node` (`release-npm-node.yml`, tag `node-v*.*.*`; **nunca publicado**, 404 documentado); `vantadb-mcp` → library-only (sin bin target; lo sirve `vantadb-server --mcp`, espath dep de `vantadb-server:27`); `vantadb-ffi-core` → hoja interna (dep de python/node/wasm); `providers/*` → source install `maturin develop` (READMEs; CI build+test only); `fuzz` → dev tool; `vanta-proxy` → source only (Freeze List).
3. **Freeze List (owner 2026-10-01, `EXPERIMENTAL_FEATURES.md`):** `vanta-proxy` "Frozen; **not published until 1.0.0**" (DIST-18 re-scoped); `vantadb-server` frozen except scheduler-host; providers "Maintenance active… no new features". Es la decisión de negocio YA tomada → DIST-06 la documenta, no la re-abre (no Gate D).
4. **Semántica release-plz (docs oficiales, fetch 2026-10-04):** `release = false` → "Release-plz ignores all packages" (no update/publish/tag/release). `publish = false` (en release-plz.toml) → salta `cargo publish` pero **sigue creando tags** (no usado en el repo). `git_only`: "Packages with `publish = false` in their Cargo.toml are also released (tagged)" — solo aplica en git_only (no es el modo del repo). Fuente: https://release-plz.dev/docs/config (verificado live).
5. **Evidencia empírica repo (release 0.8.0):** el merge del Release PR `ef2e33bd` tocó solo `Cargo.lock`, `Cargo.toml` (workspace version), `CHANGELOG.md`, 2 docs de API y PUBLISH.md — **ningún** crate `publish = false` recibió bump/changelog/tag propio. Consistente con: release-plz procesa solo publicables; el changelog es el de `vantadb` (`changelog_path`).
6. **FIND-230 (contexto):** el tren npm publicó wasm y salteó `vantadb` (TS quedó en versión vieja) en un run verde; el fix fue el gate `check-npm-versions` + regla de bump en la misma rama del Release PR. DIST-06 elimina la ambigüedad de fondo: qué canal produce qué crate.
7. **PyPI providers (live, 2026-10-04):** `pypi.org/pypi/vantadb-openai|vantadb-ollama|vantadb-litellm/json` → **404** en los 3 → no publicados; source-only confirmado.
8. **`vanta-memory` (DIST-01):** publicable en manifest + `release = false` (hold) hasta bootstrap de Trusted Publishing (owner). Referencia: `docs/dev/tasks/DIST-01.md` §Decisión release-plz. **No se duplica.**
9. **Comandos del invariante probados en DISCOVERY:** (a) da exactamente los 10; (b) da exactamente `vantadb` (changelog_path), `vantadb-wasm` (release=false), `vanta-memory` (release=false). El regex anclado `^\s*publish` evita el falso positivo del comentario en `vanta-memory/Cargo.toml:13`.
10. **Nota de consistencia:** `release-plz.toml` tiene `[workspace] publish = true` + overrides; los `[[package]]` deben ser subconjunto de la tabla §crates (vantadb/vantadb-wasm/vanta-memory) — queda como invariante documentado.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — inventario, canales y semántica release-plz resueltos por evidencia |
| Pendientes de ejecución (downhill) | 3 steps (2-4) |
| % completado | 25% (Step 1 ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** no toca trust boundaries, input de usuario, auth ni dependencias. El doc describe canales de distribución (superficie supply-chain): se documenta el estado real con enforcement ya existente (OIDC, `publish = false`). No aplica `security-and-hardening` más allá del pin SDP. Justificado.
- **PERFORMANCE:** N/A — sin hot paths.

## Steps

### Step 1 — Discovery + inventario + task file — ✅
- **Archivos:** (lectura) los 11 Cargo.toml, `release-plz.toml`, `PUBLISH.md`, reglas, workflows, Freeze List; (escritura) `docs/dev/tasks/DIST-06.md`
- **Acción:** inventario exacto (10+1), canales reales, semántica release-plz (docs live), evidencia 0.8.0, Freeze List, DIST-01/05 releídos; task file canónico con Gate Regla 0.
- **Verify:** `rg "^\s*publish = false"` = 10; CMD (b) = 3 overrides; PyPI providers 404; task file existe con Impacto mapeado.
- **Estado:** ✅

### Step 2 — `PUBLISH.md` §crates (core)
- **Archivos:** `docs/dev/workflow/PUBLISH.md`
- **Acción:** sección nueva `## Crates — publish decision per crate (DIST-06)`: tabla de los 11 (decisión + canal + motivo + producido por), mini-tabla de contexto (vantadb, vantadb-wasm), invariante release-plz + comandos (a)/(b), política fechada con triggers de revisión. Insertada entre §Tag namespace table y §Cascadas.
- **Verify:** los 4 comandos del Contrato (Step 3).
- **Estado:** ✅ DONE — sección insertada entre §Tag namespace table y §Cascadas (64 líneas netas, 0 deleciones); F1/F3 del review aplicados y re-verificados.

### Step 3 — Verify: invariante + gates docs
- **Archivos:** (verificación)
- **Acción:** re-ejecutar CMD (a) y (b) contra la tabla; gates docs (check-links / check-docs / gen-index --check); markdownlint del archivo si aplica.
- **Verify:** CMD (a) = 10 filas de la tabla; CMD (b) = 3 overrides ⊆ tabla; gates exit 0.
- **Estado:** ✅ DONE — CMD (a)=10 · CMD (b)=3 ⊆ tabla · check-links/check-docs/gen-index --check exit 0 al momento de la verificación (pre-WIP ajeno DOCS-F1; ver Notas) · markdownlint 0 issues.

### Step 4 — Cierre: OCR + review P2-01 + commit local
- **Archivos:** `docs/dev/tasks/DIST-06.md` (+ anteriores)
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (advisory); review por agente distinto (`vanta-review`, contexto fresco); commit local `docs: DIST-06 — …` (NUNCA push).
- **Verify:** OCR sin Critical/High · veredicto reviewer registrado · `git status` limpio post-commit (salvo WIP ajeno).
- **Estado:** ✅ DONE — OCR: 0 reviewable (docs `.md` fuera del scope del reviewer de código → sin Critical/High aplicables) · review APPROVE · commit local único `docs: DIST-06` (hash en RESULTADO §7; sin push).

## Dependencias

- F0 — DIST-01 ✅ (vanta-memory publicable + hold) y DIST-05 ✅ (verificación post-release) releídos; sin bloqueantes. nextTask: DOCS-F1.

## Review (GATE — agente distinto, P2-01)

> Tier risk-based: paths tocados (`docs/dev/workflow/PUBLISH.md`, `docs/dev/tasks/DIST-06.md`) NO matchean globs adversariales → tier **fast**; se eleva a review fresco `vanta-review` (decisión de release con invariante).

- **Revisor:** `vanta-review` (contexto fresco, read-only, no participó de la implementación — sesión `ses_ef9145f3fffezcgeYmCTiLD2Pt`).
- **Enfoque:** decisión release/packaging por crate; consistencia con `release-plz.toml`; accuracy de claims citados; scope discipline.
- **Cómo se probó:** CMD (a)/(b) re-ejecutados (10 filas / 3 overrides) · 3 gates docs exit 0 al momento del review · `git show ef2e33bd --stat` 6/6 contra el texto (delta F1) · live: PyPI providers 404 / `vantadb-py` 200 / npm `vantadb-node` 404 / crates.io `vantadb` 0.8.0 · docs oficiales release-plz · Freeze List · members/overrides del workspace.
- **Checklist anti-hábitos tóxicos:** ✅ sin salidas inventadas (todo re-derivado por el reviewer) · ✅ sin done-sin-verificar (gates corridos; fallos externos reportados, no ocultados) · ✅ sin fallos ignorados · ✅ alcance respetado (WIP ajeno intacto; staging selectivo de 4 paths).
- **Veredicto:** ✅ **APPROVE** — 0 Critical/High/Medium; F1 (claim enumerativo incompleto) y F3 (nit del comentario CMD b) aplicados y re-verificados en delta; F4 (boundary del invariante ante members nuevos) y F5 (race DOCS-F1) informativos, sin acción en este commit.

## Notas

- **Push:** diferido al final del plan (instrucción owner 2026-10-04) — commits locales.
- **Churn mínimo (pre-mortem #2):** `release-plz.toml` NO se toca; solo se documenta la consistencia y sus comandos. Ningún crate cambia de estado de publicación.
- **Índices generados:** `docs/index.md` + `llms.txt` regenerados con `gen-index.mjs --write` (artefacto derivado, nunca hand-edit): el diff es 100% mecánico y **absorbe también las entradas pendientes de DIST-05** (1504→1506 docs, workflows 37→38, task files 1056→1058) — verificado por el reviewer (delta atribuible 1:1).
- **Race externo documentado (F5):** durante el cierre apareció WIP ajeno de DOCS-F1 (`docs/dev/tasks/DOCS-F1.md` sin frontmatter + `scripts/docs/check-links.mjs` modificado con budget 0). Con ese WIP los gates globales dan exit 1 (`missingFrontmatter=1`; budget 0 con drain en curso) — **no atribuible a DIST-06**; el commit stagea solo los 4 paths de esta tarea y NO re-regenera el índice (absorbería DOCS-F1). La verificación de DIST-06 fue verde contra su estado propio.
- **F4 (info, próximo review de la política):** los 2 comandos no detectan un member nuevo publicable sin `publish=false` ni override — aceptado por Spec #5 (sin gate nuevo); anotado para el próximo review de la política (triggers ya listados en el doc).
- **Plan file / Backlog:** NO tocados por instrucción del orquestador (el orquestador sincroniza el plan y corre el pase de progreso).
- **FIND-230:** DIST-06 cierra la ambigüedad de fondo; el gate mecánico del tren npm (FIND-230) sigue siendo el enforcement específico de npm.

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4
PROXIMO_STEP: ninguno
COMMIT_HASH: docs: DIST-06 — commit local único (este archivo viaja en él; hash real en el RESULTADO del cierre)
ARCHIVOS: docs/dev/workflow/PUBLISH.md · docs/dev/tasks/DIST-06.md · docs/index.md · llms.txt
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | spec decidida-por-evidencia; sin pregunta; verify 1er intento; race DOCS-F1 documentado (externo)
SKILLS_CARGADAS: ci-cd-and-automation · git-workflow-and-versioning · security-and-hardening · source-driven-development · documentation-skill · documentation-and-adrs · shipping-and-launch (base auto: campaign-executor, progreso)
```
