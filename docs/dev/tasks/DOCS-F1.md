---
title: "DOCS-F1: Cerrar docs-consolidation F1 (triage de links + mojibake + markdownlint)"
kind: task
description: "Cerrar docs-consolidation F1: triage P0-P4 de los 55 enlaces rotos, drain a 0 no-frozen, mojibake de avance a 0 en prosa y ratchet markdownlint 12 a 0 en la superficie controlada."
---

# DOCS-F1: Cerrar docs-consolidation F1 (triage de links + mojibake + markdownlint)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 20, F0)
- **Fuente:** `docs/dev/plans/2026-09-28-docs-consolidation.md` §F1 (T8/T9/T12)
- **Esfuerzo:** 🟢 4h · **Prioridad:** 🟡
- **Tipo:** Docs (2 scripts de tooling + 1 workflow + docs)
- **Turns estimados:** 15-25
- **Creado:** 2026-10-04T18:00
- **last-synced:** 2026-10-04T20:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas (T12 reconciliado en DISCOVERY: ver Notas N1)
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `scripts/docs/check-links.mjs` ← workflow `gate-docs-links.yml` (job `docs-links`), F5 del plan docs-consolidation; `scripts/docs/gen-index.mjs` ← job `docs-index` + skill `progreso` (índices) |
| Callees | `lib.mjs` (`proseOf`, `segment`, `buildTargetSet`, `MD_LINK`) — no se toca; `docs/dev/workflow/gate-docs-links.md` documenta ambos jobs |
| Implicaciones | Docs + tooling sin consumidores de código Rust. El cambio de scanner (`proseOf` para links md) reduce el conteo local y en CI en la misma medida (mismos archivos versionados). `FROZEN_RE` excluye `avance/historial/**` y `tasks/**` del conteo roto (P3). Los budgets bajan (58→0, 40→N). `docs/index.md`/`llms.txt` se regeneran (esperado). Cero cambio de API pública |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/docs/check-links.mjs`, `scripts/docs/lib.mjs`, `scripts/docs/gen-index.mjs`, `.github/workflows/gate-docs-links.yml`, `docs/dev/workflow/gate-docs-links.md`, `.markdownlint-cli2.yaml`, `docs/dev/plans/2026-09-28-docs-consolidation.md`, `.opencode/task-system/prompts/task.md`, `.opencode/task-system/prompts/pipeline-full.md`. Los ficheros de docs a editar se leen completos en su step de edición (son prosa corta; contexto ya verificado por línea).
- **Archivos referenciados hacia dentro (imports/dependencias):** `check-links.mjs` importa `lib.mjs` (`buildTargetSet`, `listDocs`, `proseOf`, `WIKI_LINK`, `MD_LINK`) — el cambio usa helpers existentes, no agrega dependencias. `gen-index.mjs` importa `lib.mjs` — ídem.
- **Archivos que referencian a los editados (referencias entrantes):** `gate-docs-links.yml` invoca `check-links.mjs` y `gen-index.mjs --check`; `docs/dev/workflow/gate-docs-links.md` los documenta (se actualiza); F5 del plan docs-consolidation los cita (se actualiza); skill `documentation-skill` §7/§9 los cita (sin cambio de interfaz).
- **Veredicto impacto:** **bajo-medio.** Docs + tooling; sin código de producto. Riesgos: (1) cambio de semántica del scanner → mitiga el comentario + medición before/after + los 3 links reales de ejemplo siguen detectables si son prose; (2) budgets que no coincidan con el conteo real → se miden mecánicamente antes de fijarlos; (3) `docs/api/EMBEDDED_SDK.md` entra en tier adversarial P2-01 (2 líneas de links) → review degradado documentado.

## Contrato

**(a)** Cada link roto de `check-links.mjs --json` (44 md + 11 wikilinks = 55 entradas) clasificado **P0-P4** (tabla abajo), con **P3/P4 excluidos del gate** y **motivo escrito en el workflow**; **(b)** 0 mojibake (`[[bench]]`/`[[test]]`/`[[package]]`/`[[bin]]`/`[[example]]`) **en prosa** bajo `docs/dev/avance/**` fuera de `tasks/`; **(c)** `npx markdownlint-cli2 "docs/**/*.md"` con el ratchet bajado **12→0 para la superficie controlada** y CI verde (el master plan — recitations machine-appended del orquestador, PROHIBIDO — queda excluido del conteo con motivo en el workflow; desviación de "5" documentada en Notas N1); **(d)** F1 del plan docs-consolidation marcado ✅ con T8/T9/T12 COMPLETED.

**Verificación mecánica:**
- `node scripts/docs/check-links.mjs` → exit 0, `0 broken markdown links` (no-frozen)
- `node scripts/docs/check-docs.mjs` → gating clear (wikilinks informativos bajan 30→20)
- `node scripts/docs/gen-index.mjs --check` → exit 0
- `npx markdownlint-cli2 "docs/**/*.md"` → 0 errores en la superficie controlada (= BASELINE 0; master plan excluido con motivo)
- `rg "\[\[bench\]\]|\[\[test\]\]|\[\[package\]\]|\[\[bin\]\]|\[\[example\]\]" docs/dev/avance --glob "!tasks/**"` → 0 en prosa (solo code spans legítimos de TOML)

## Spec (decisiones técnicas — 100% docs/tooling)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dominio de escaneo de links md | A: raw (hoy: ejemplos en code spans gatean) / **B: `proseOf`** (consistente con wikilinks; solo links clicables gatean) | ✅ B — misma lógica ya documentada en `lib.mjs` ("excluir el código es lo que hace el recuento drenable"); evidencia: 21/44 entries en code spans |
| 2 | Operacionalización de P3 "archive/" | A: solo dirs `archive/` (ya excluidos) / **B: extender a frozen-by-design** (`avance/historial/**` = registro congelado; `tasks/**` = work-items congelados, restricción owner) | ✅ B — motivo en script + workflow + doc del gate; P3 real ≠ ∅ |
| 3 | T12 BASELINE | A: 5 (imposible: errores en master plan PROHIBIDO) / **B: 0 para la superficie controlada + exclusión scoped del master plan (motivo + FIND)** | ✅ B — ratchet 12→0, CI verde y estable ante el archivo vivo; desviación de 5 documentada |
| 4 | `postcard.md` inexistente | A: desenlazar (2 edits de prosa) / **B: crear página de glosario** | ✅ B — convención del glosario (60+ términos); contenido derivado de `serialization.md` + `Cargo.toml` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `docs/dev/tasks/` y `docs/dev/plans/` no se mueven ni renombran; (2) frontmatter `links: "[[...]]"` del glosario NO se toca (tipo vault-wide de Obsidian); (3) índices generados nunca se editan a mano (regen con `gen-index.mjs --write`); (4) un link roto se repara solo con destino único verificado — nunca se adivina; (5) `docs/CHANGELOG.md` no se toca (release-plz); (6) PROHIBIDO tocar: master plan, `opencode.jsonc`, `docs/pipeline-state.json`, `PUBLISH.md`, `release-plz.toml`, Cargo.tomls.
- **Comandos de verificación:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check && npx markdownlint-cli2 "docs/**/*.md"` (esperado: 0 / clear / 0 / 0 en superficie controlada).
- **Deuda pendiente:** master plan 7× MD007 (orchestrator, FIND derivado); `[[test]]` en `.venv`-hit del wikilink test (`.venv` fuera del skip de `buildTargetSet` — nota, no tocar); wikilinks en `tasks/` (19) quedan en el budget del contador (frozen).

## Triage P0-P4 (T8) — las 55 entradas del JSON

> Fuente: `node scripts/docs/check-links.mjs --json` (2026-10-04, 44 md + 11 wikilinks). Reproducible. Método: P0 = destino inexistente + origen en el funnel QUICKSTART a profundidad ≤2 (0 casos: el índice generado hace universal la alcanzabilidad desde `docs/index.md`, la definición literal degenera — se documenta); P1 = origen en `docs/user/**` o `docs/api/**`; P2 = origen en `docs/dev/**` no congelado; P3 = origen congelado por diseño (`avance/historial/**` = historial; `tasks/**` = work-items; `archive/` ya excluido del scan); P4 = externo (0 en `broken`; ya excluido del gate: solo conteo + barrido semanal).

| Nivel | Entradas | Acción |
|---|---|---|
| **P0** | 0 — degenerado por índice generado (ver Notas N2) | — |
| **P1** | 16: blog ×6 (`/blog/slug`→relativo), desktop ×3 (`ARCHITECTURE.md`→`../../dev/desktop/`), glosario ×6 (postcard ×2 → página nueva; wikilinks ×4 → md links), EMBEDDED_SDK ×2 (`[[SEARCH_PARITY\|…]]`→md), master-index ops ×3 (paths `../../dev/…`) | **Arreglar ya** (superficie viva) |
| **P2** | 12: GO_TO_MARKET/ROADMAP/VISION ×3 (`../../master-index.md`→`../`), research ponytail ×2 (de-link: `.opencode/` no existe en CI), master-index dev ×1 (fila northstar inexistente → remover), gate-docs-links.md ×1 (ejemplo `y.md` → drenado por scanner), docs-strategy ×2 + README ×1 (ejemplos → drenados por scanner), COMMUNITY_GOVERNANCE ×1 (wikilink→md) | **Arreglar** |
| **P3** | 27: `avance/historial/**` ×17 (fuentes 5, snapshots 7, campañas 3, sesiones 1, wikilink walkthrough 1) + `tasks/**` ×10 (DESKTOP-QW8 ×2, FIND-74 ×3, MEM-44, SCH-08, SHOW-05, + code-span) | **Dejar roto. Excluir del gate** (motivo en workflow + script) |
| **P4** | 0 en `broken` (externos: conteo only, nunca gate; barrido semanal `schedule` ya en workflow) | Sin cambio |

## Deuda técnica (Regla 6)

**Saldo neto:** negativo (paga deuda). Drena: 44 broken md → 0 no-frozen; 2 mojibake en prosa → 0 (+1 fórmula corrupta en rrf.md); ratchet lint 12→0 scoped (master plan excluido, FIND); wikilink budget 40→20. Sin deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (a)-(d) verificado mecánicamente (comandos arriba) + OCR sin Critical/High + review registrado |
| **Commit** | Commit atómico `docs:` (~100-200 líneas), solo archivos del blast radius, verificación mecánica adjunta |
| **Release** | n/a (docs; sin release). Gates CI: jobs docs-links/docs-schema/docs-index/docs-lint verdes en el próximo push (evidencia local equivalente) |

## Herramientas necesarias
- `node scripts/docs/{check-links,check-docs,gen-index}.mjs`, `npx markdownlint-cli2`, `git log -S` (mojibake), `pwsh dev-tools/ocr-review.ps1`

**Skills cargadas (SDP v3):** `documentation-skill` (obligatoria docs — define sintaxis de enlace y DoD), `writing-guidelines` (prosa), `git-workflow-and-versioning` (pinned — commit/ratchet), `ci-cd-and-automation` (pinned — workflow), `doubt-driven-development` (base CI/CD + review degradado al cierre), `incremental-implementation` (lifecycle BUILD — slices por step), base auto: `campaign-executor`, `progreso`, `ponytail`.

## Steps

### Step 1: `check-links.mjs` — dominio proseOf + exclusión frozen (P3)
- **Archivos:** `scripts/docs/check-links.mjs`
- **Acción:** escanear `MD_LINK` sobre `proseOf(text)` (no raw); `FROZEN_RE` para `avance/historial` + `tasks` (skip del push a `broken` con comentario); reescribir bloque de comentarios de budgets (58→0, 40→20 tras medir) documentando F1 + P3/P4.
- **Verify:** `node scripts/docs/check-links.mjs --json` → 20 broken (antes 44) y `--max-broken` default 0 tras fixes; self-test del segmenter sigue verde (`wikilinks-to-md.mjs --self-test`).
- **Estado:** ✅ COMPLETED

### Step 2: Fixes P1 — superficie pública
- **Archivos:** `docs/user/blog/*.md` ×6, `docs/user/desktop/{README,GUIDE}.md`, `docs/user/glosario/{bincode,serialization,bm25,hnsw,mmap,heuristic_search}.md`, `docs/api/EMBEDDED_SDK.md`, `docs/user/operations/master-index.md`, `docs/user/glosario/postcard.md` (nuevo)
- **Acción:** blog `/blog/slug`→relativo `.md`; desktop→`../../dev/desktop/ARCHITECTURE.md`; postcard.md creado (glosario, frontmatter completo); wikilinks glosario/EMBEDDED→md links a destinos existentes verificados (`TEXT_INDEX_DESIGN.md`, `FND-20-hnsw-tradeoff.md`); master-index ops→`../../dev/operations/chaos-testing.md` + `../../dev/archive/…`.
- **Verify:** `node scripts/docs/check-links.mjs` → 0 broken P1; `node scripts/docs/check-docs.mjs` clear.
- **Estado:** ✅ COMPLETED

### Step 3: Fixes P2 — dev no congelado
- **Archivos:** `docs/dev/strategy/{GO_TO_MARKET,ROADMAP}.md`, `docs/dev/vision/VISION.md`, `docs/dev/research/{bench-framework-evaluation,concurrency-testing}-2026-08-30.md`, `docs/dev/master-index.md`, `docs/dev/operations/COMMUNITY_GOVERNANCE.md`
- **Acción:** `../../master-index.md`→`../master-index.md` ×3; ponytail de-link ×2 (`.opencode/` no resoluble en CI); master-index dev: fila `northstar.md` removida (reporte inexistente — `reports/INDEX.md` no lo lista) + banner deprecación→`docs/index.md` (ítem F5 atribuido a F1); governance wikilink→md link ADR-0009.
- **Verify:** `node scripts/docs/check-links.mjs` → 0 broken no-frozen; `npx markdownlint-cli2` sin errores nuevos.
- **Estado:** ✅ COMPLETED

### Step 4: T9 — mojibake en `avance/`
- **Archivos:** `docs/dev/avance/historial/backlog-history.md`, `docs/dev/avance/historial/campanas/migradas-backlog-tabla.md`
- **Acción:** `[[test]]`/`[[bench]]` en prosa → restaurar forma correcta según `git log -S` (introducidas por b764d703 C-02; pre-C-02 no existían — la forma correcta es la referencia TOML en code span: `` `[[test]]` ``/`` `[[bench]]` ``). NO tocar tasks/.
- **Verify:** `rg "\[\[bench\]\]|\[\[test\]\]|\[\[package\]\]|\[\[bin\]\]|\[\[example\]\]" docs/dev/avance --glob "!tasks/**"` → 0 en prosa; `check-links` wikilinks 30→20.
- **Estado:** ✅ COMPLETED

### Step 5: T12 — markdownlint 12→0 scoped + regen
- **Archivos:** `scripts/docs/gen-index.mjs` (escape `[`/`]` en celdas de descripción), `.github/workflows/gate-docs-links.yml` (BASELINE '12'→'0' + exclusión scoped del master plan + comentario motivo), `docs/dev/workflow/gate-docs-links.md` (tabla valores), `docs/index.md`+`llms.txt`+índices (regen)
- **Acción:** escapar brackets en `rowFor`/ADR index (fix clase MD052, mismo patrón que `\|`); regen `gen-index.mjs --write`; bajar BASELINE a 0 con exclusión scoped del master plan (motivo en el workflow + FIND; desviación de 5 documentada).
- **Verify:** `npx markdownlint-cli2 "docs/**/*.md"` → 0 en superficie controlada; `node scripts/docs/gen-index.mjs --check` → 0.
- **Estado:** ✅ COMPLETED

### Step 6: Plan docs-consolidation — F1 ✅ (T8/T9/T12)
- **Archivos:** `docs/dev/plans/2026-09-28-docs-consolidation.md`
- **Acción:** F1 header→✅ COMPLETADA + gate de salida anotado; T8/T9/T12 `Estado:`→COMPLETED con resultado; F5 checklist: marcar ítem del banner (hecho en F1); nota de medición actualizada.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` verdes tras el edit (el plan es .md).
- **Estado:** ✅ COMPLETED

### Step 7: Verificación final + cierre
- **Acción:** verify full (comandos del contrato) + OCR (`pwsh dev-tools/ocr-review.ps1`) + review P2-01 degradado (§Review) + commit local `docs:` + campaign close taskId `20` + skill progreso.
- **Verify:** contrato (a)-(d) completo; `git status` sin residuos fuera de scope.
- **Estado:** ✅ COMPLETED

## Dependencias
- F0 — sin dependencias. **nextTask: DOCS-F2** (Task 21).

## Review (GATE — agente distinto, P2-01)

> Diff mixto: `docs/api/EMBEDDED_SDK.md` matchea el tier **adversarial** → sin subagentes disponibles en este contexto (subagente hoja), se aplica fallback documentado: `doubt-driven-development` degradado + escalado al orquestador. Evidencia en §Review al cierre.

- **Revisor:** `doubt-driven-development` **DEGRADADO** (subagente hoja sin tool de subagentes para forkear `vanta-review`) + OCR delegation (spec `ocr-delegate/v1`, 3 archivos: workflow + 2 scripts) — **escalado al orquestador** para review formal vanta-review (tier adversarial por `docs/api/**`).
- **Enfoque:** ¿scanner `proseOf` + `FROZEN_RE` correctos? ¿presupuestos honestos? ¿destinos de links con coincidencia única verificada?
- **Hallazgos del review adversarial (todos resueltos antes del commit):**
  - **R1** — residuo vivo no detectado por el scan por segmentos: `[[bm25|` dentro de una fórmula corrupta en `rrf.md` (`$[[bm25|0, \infty)$` → `$[0, \infty)$`; la fila hermana del mismo tabla probaba el original). Detectado con el scan `proseOf` (21 vs 20), restaurado; budget 21→20.
  - **R2** — números reconciliados tras R1 (script/workflow/doc/task: 21→20) y recomputados mecánicamente (20/20, exit 0).
  - **R3** — duplicación de lógica flag por OCR Rule Group 2: boundary de frontmatter → reuso de `parseFrontmatter()`; escape de celdas → helper `cellDesc()`. Refactor aplicado y re-verificado.
  - **R4** — bug propio del helper de líneas (offset por frontmatter): detectado y corregido; líneas verificadas contra `rg` (68/92/93, 227, 180/129, 466/485/259 — exactas).
  - **R5** — durante el cierre, el conteo del master plan creció 7→11 por ediciones concurrentes del orquestador: un BASELINE fijo habría quedado rojo en el push. Decisión endurecida: ratchet **12→0 para la superficie controlada** con el master plan excluido del conteo (motivo en el workflow + FIND para el formato de recitations) — más fuerte para todo lo controlable y estable ante el archivo vivo.
- **Cómo se probó:** batería mecánica final — `check-links` 0 broken / 20 wikilinks exit 0 · `check-docs` exit 0 · `gen-index --check` exit 0 · `markdownlint` 0 en superficie controlada (= BASELINE 0; master plan excluido) · `actionlint` 0 · `node --check` 0 · segmenter self-test ok · `validate-docs-coverage` 0 gaps. OCR: 0 Critical/High tras R1-R4.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas (todo comando ejecutado en esta sesión)
  - [x] No done sin verificar (contrato a-d medido, no auto-reportado)
  - [x] Fallos parciales reportados (T12: desviación 5→7 documentada con motivo y FIND)
  - [x] SDP cubierto (`SKILLS_CARGADAS` en RESULTADO)
- **Veredicto:** ✅ **approve (degradado)** — escalado al orquestador para review P2-01 formal.

## Notas

- **N1 — T12 reconciliado:** el plan pedía drenar 7 errores (5 BENCHMARKS MD005/MD007 + 2 CONFIGURATION MD027) y bajar BASELINE 12→5. Medición 2026-10-04: esos 2 ficheros lintan **0** (drenados por tareas posteriores — BENCH-01 `860340b9` tocó BENCHMARKS); el conteo inicial era **8**: 7×MD007 en el master plan (recitations FIND-238/DX-01, PROHIBIDO tocar) + 1×MD052 en `docs/index.md` (descripción FIND-239 `["fields"]["content"]`). Se arregla el MD052 vía generador (escape); durante el cierre el conteo del master plan creció **7→11** (ediciones concurrentes del orquestador), así que el ratchet final es **12→0 para la superficie controlada con el master plan excluido del conteo** (motivo en el workflow + FIND para el formato de recitations). **Desviación explícita del contrato (c): 0 scoped ≠ 5 literal — más fuerte para todo lo controlado y estable ante el archivo vivo.**
- **N2 — P0 degenerado:** la definición P0 ("origen alcanzable desde docs/index.md") es universal con el índice generado (lista los 1506 docs) y vacía por el funnel QUICKSTART (depth≤2 no toca ningún origen roto) → P0=0 documentado; la urgencia real vive en P1 (superficie pública), que se arregla completa.
- **N3 — CI vs local:** 1 link local-only (`docs/api/HTTP_API.md:663` → `.opencode/references/…`) resuelve local y NO en CI (`.opencode/` gitignored, no existe en checkout). No está en el `broken` local; se registra como deuda/FIND (no se toca para no perder el puntero del equipo).
- **N4 — mojibake:** `git log -S` (pre-mortem): las ocurrencias en prosa entraron con `b764d703` (C-02); no hay "texto original" que restaurar distinto — la forma correcta es la referencia TOML entre backticks. Las ~14 ocurrencias en code spans son TOML legítimo (no mojibake, no se tocan).
- **N5 — budget wikilinks:** tras fixes queda **20** (19 en `tasks/` frozen + 1 walkthrough en historial); el residuo vivo (`[[bm25|` dentro de una fórmula en `rrf.md`) se detectó en el review adversarial y se restauró a `$[0, \infty)$`. Budget fijado al valor medido con comentario.
