---
title: "TASK DESKTOP-43: Auto-update vía tauri-plugin-updater — defer registrado con trigger + checklist de habilitación"
kind: task
description: "Defer del auto-update (stop condition del plan): sin claves de firma (DEFER HITL H-08/DEVOPS-10) y decisión de distribución pública abierta → trigger explícito + checklist de habilitación + trust model documentado; sin tocar código ni pipeline de releases"
---

# TASK DESKTOP-43: Auto-update vía tauri-plugin-updater — defer registrado con trigger + checklist de habilitación

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 33, F0 expandido)
- **Fuente:** Backlog P46 `DESKTOP-43` (nota :712; fila removida al registrar el defer) + INV-desktop-prod H-10 (`docs/dev/reviews/archive/research-desktop-prod-20260825.md:113`) + `docs/dev/research/installer-personalizado/RESEARCH.md` §F3 (:152)
- **Esfuerzo:** 🟡 1-2d | **Appetite:** max 2d | **Prioridad:** 🟡
- **Tipo:** Docs (defer; **sin cambios de código** — no feature-add: no agrega símbolos/contratos públicos)
- **Turns estimados:** 15-30 (consumidos ~12)
- **Creado:** 2026-10-04 (DISCOVERY) | **last-synced:** 2026-10-04
- **Estado:** ✅ COMPLETED (rama defer del contrato: trigger + checklist + trust model; sin tocar código ni pipeline de releases)
- **Campaign ID:** master-plan-0.9.0-20261004 (taskId `33`)
- **Incógnitas (uphill):** 0 abiertas — resueltas en DISCOVERY: (1) ¿decisión de distribución pública tomada? **NO** (Backlog P46 nota :712 + master plan :953 + `desktop/README.md` "No public installer yet") → stop condition; (2) ¿claves/secrets de firma disponibles? **NO** (`gh secret list` → solo `RELEASE_PLZ_TOKEN`; firma DIFERIDA por HITL, `wontfix.md:103`) → rama defer sancionada por la stop condition (el E2E simulado era viable con keypair de prueba, pero dejaría un plugin inerte y cambiaría el trust model antes de la decisión de distribución — "no construir a ciegas")
- **Pendientes (downhill):** 5 steps (5 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (entrantes) | Nadie consume el updater (no existe). Referencias documentales: master plan Task 33, Backlog P46 (nota :712), research installer §F3, `INSTALLER_SMOKE_CHECKLIST.md:52`, `wontfix.md:103` (firma) |
| Callees (salientes) | N/A — el defer no agrega deps ni código; el checklist referencia docs oficiales Tauri v2 y `tauri-action` (no ejecuta nada) |
| Implicaciones | **Cero cambios en código/CI/release pipeline** (contrato: "sin tocar el pipeline de releases si se difiere"). Diff = docs: task file nuevo, `UPDATER_ENABLEMENT.md` nuevo, sección en `desktop/ARCHITECTURE.md`, registros (wontfix/avance/Backlog), generados (`docs/index.md`, `llms.txt`) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/desktop/ARCHITECTURE.md` (122L) · `docs/dev/avance/decisiones/wontfix.md` (98L) · `docs/dev/avance/activo/desktop.md` (455L) · `docs/dev/desktop/INSTALLER_SMOKE_CHECKLIST.md` (140L) · `desktop/src-tauri/Cargo.toml` (64L) · `desktop/src-tauri/tauri.conf.json` (69L) · `desktop/src-tauri/capabilities/default.json` (13L) · `.github/workflows/desktop.yml` (248L) · `desktop/README.md` (§Installer status :124-129) · `docs/dev/reviews/archive/research-desktop-prod-20260825.md` (§H-08/H-10 vía grep) · `docs/dev/research/installer-personalizado/RESEARCH.md` (:45,:61,:63,:115,:152 vía grep) · `docs/dev/tasks/DESKTOP-41.md` (formato canónico de cierre)
- **Lecturas puntuales:** `docs/dev/Backlog.md` §P46 (:690-718, machine-managed) · `.opencode/task-system/prompts/task.md` (formato) · `scripts/docs/lib.mjs` §deriveKind (:210-236) · `scripts/docs/check-docs.mjs`
- **Referencias hacia dentro (imports/deps):** `desktop/README.md:133` → `docs/dev/desktop/ARCHITECTURE.md`; `docs/user/desktop/{README,GUIDE}.md` → ARCHITECTURE.md; `docs/dev/avance/README.md` → `wontfix.md` + `activo/desktop.md`; `docs/index.md`/`llms.txt` (generados) indexan todo
- **Referencias entrantes a los editados:** `wontfix.md` ← Backlog (:712), ADR-0048, avance/README, investigaciones.md, master plan; `activo/desktop.md` ← Backlog, `INSTALLER_SMOKE_CHECKLIST.md:104-105`, avance/README, meta.md, master plan; `ARCHITECTURE.md` ← desktop/README.md, user/desktop/*, docs/index.md, llms.txt
- **Veredicto impacto:** **BAJO** — ediciones aditivas en docs; ningún contrato de código, CI o release se toca. Efectos: (a) nuevos archivos entran a los índices generados (`gen-index --write`), (b) links nuevos deben resolver (check-links), (c) fila Backlog removida — el registro DEFER queda en `wontfix.md` (canónico) + enablement doc para no perder trazabilidad. Gate D evaluado: **no dispara** (sin símbolos públicos nuevos; el plan sanciona la rama defer por stop condition)

## Contrato

> Del plan (Task 33): "updater configurado y probado (update simulado end-to-end: manifest JSON + firma + app que detecta/aplica) **o** decisión de defer registrada con trigger explícito (decisión de distribución pública) + checklist de habilitación; sin tocar el pipeline de releases si se difiere."
>
> **Stop condition aplicada (plan):** "decisión de distribución no tomada → defer registrado con trigger + checklist (contrato cumplido)."

Entregables de esta tarea (rama defer):

1. **Decisión de defer registrada** con trigger explícito: `wontfix.md` §DEFER (registro canónico; trigger = decisión de distribución pública del desktop).
2. **Checklist de habilitación:** `docs/dev/desktop/UPDATER_ENABLEMENT.md` (claves → wiring → CI release-mode → validación E2E → trust model; anclado a docs oficiales Tauri v2).
3. **Trust model documentado** (pre-mortem 3): sección §Distribution & updates en `docs/dev/desktop/ARCHITECTURE.md` (estado actual + impacto del futuro canal de updates).
4. **Registros de cierre:** avance/desktop.md + fila Backlog removida; pipeline de releases **intacto**.

**Comandos de verificación:** `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · `node scripts/docs/gen-index.mjs --check` · `pwsh scripts/validate-docs-coverage.ps1` · markdownlint de los archivos tocados + review P2-01.

## Spec (SDD — no feature-add; decisiones resueltas por evidencia)

> Gate mecánico: la tarea NO agrega símbolos/contratos públicos (no toca código) → feature-add = false. Se documentan las decisiones técnicas del DISCOVERY:

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Rama del contrato (E2E simulado vs defer) | A) wirear updater + E2E simulado con clave de prueba (agrega plugin inutilizable sin firma real; cambia trust model sin decisión de distribución) / B) defer con trigger + checklist (camino listo documentado; no construye a ciegas) | B | ✅ decidido-por-evidencia: stop condition del plan + claves/secrets ausentes (`gh secret list`) + firma ya DIFERIDA por HITL (`wontfix.md:103`, extiende DEVOPS-10) + Gate Justificación ("no construir a ciegas") |
| 2 | Dónde registrar el defer | A) `wontfix.md` §DEFER (canónico, precedente DEVOPS-10) + doc de habilitación durable / B) ADR nuevo (más pesado; el plan pide "ADR corto **o** sección en ARCHITECTURE.md" solo para trust model) | A | ✅ decidido-por-evidencia: `wontfix.md` = "Registro de decisiones de no implementar (WONTFIX) y diferimientos (DEFER)… criterio de re-apertura" |
| 3 | Endpoint de manifests (para la habilitación) | A) JSON estático `latest.json` en GitHub Releases (tauri-action lo genera; patrón research) / B) server dinámico (hosting + mantenimiento) | A | ✅ decidido-por-evidencia: research §F3 (:63,:115) + docs oficiales Tauri v2 updater (static JSON + GitHub Releases example); queda como default documentado en el checklist |
| 4 | Trust model (pre-mortem 3) | A) documentar el cambio futuro sin aplicarlo (sección ARCHITECTURE) / B) nada | A | ✅ decidido-por-evidencia: pre-mortem 3 del plan ("habilitar updater cambia el modelo de confianza → documentar"); aplicado como sección + pointer al ADR-en-PR-de-habilitación |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) no tocar `opencode.jsonc`, master plan (`docs/dev/plans/2026-10-04-master-plan-0.9.0.md`), `docs/pipeline-state.json`, `providers/**` (PROV-12 en vuelo); (2) **no push** (Regla 7 — commit local); (3) no inventar claves de firma ni decidir la distribución pública — el defer preserva esa decisión para el owner; (4) no tocar el pipeline de releases (`desktop.yml`, workflows) ni código del desktop (contrato del defer); (5) si se re-abre: trigger = decisión de distribución pública; el checklist es la fuente única de pasos.
- **Comandos de verificación:** `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · `node scripts/docs/gen-index.mjs --check` · `pwsh scripts/validate-docs-coverage.ps1` — todos exit 0.
- **Deuda pendiente:** la implementación del updater queda **diferida con trigger** (registrada — no deuda oculta). Al habilitarse: claves + secrets + wiring + CI release-mode + E2E (checklist).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto cero:** diff docs-only; no introduce código, deps ni artefactos. La ausencia de auto-update (deuda de producto) queda **registrada como DEFER con trigger** — visible, no oculta.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato (defer + trigger + checklist + trust model) + gates docs verdes | ⏳ |
| **Commit** | Atómico docs-only, `docs(desktop):`, pathspec propio (WIP ajeno fuera) | ⏳ |
| **Release** | n/a — docs-only; no toca artefactos, versiones ni pipeline. Justificado | ✅ justificado |

## Herramientas necesarias

- `node scripts/docs/*.mjs` (gates docs) · `pwsh scripts/validate-docs-coverage.ps1` · `pwsh dev-tools/ocr-review.ps1`
- Web: docs oficiales Tauri v2 updater (fetched — ver Investigation Notes)
- codegraph/CBM: N/A (sin código)

**Skills cargadas (SDP):** ci-cd-and-automation (dominio CI/release) · shipping-and-launch (decisión de distribución) · documentation-and-adrs (defer + registros) · documentation-skill (MUST docs) · source-driven-development (checklist anclado a docs oficiales) · doubt-driven-development (review adversarial) · git-workflow-and-versioning (commit/pathspec). Base auto: campaign-executor/progreso/ponytail.

## Investigation Notes

**Evidencia del DISCOVERY (2026-10-04):**

1. **Updater ausente:** `rg tauri-plugin-updater desktop/src-tauri` → 0 refs de código. `Cargo.toml` solo deep-link + single-instance; `tauri.conf.json` `plugins` solo deep-link; `capabilities/default.json` sin permisos updater.
2. **Claves/secrets de firma ausentes:** `gh secret list` → solo `RELEASE_PLZ_TOKEN` (2026-10-02). Sin `TAURI_SIGNING_PRIVATE_KEY[_PASSWORD]`. La firma de instaladores está **DIFERIDA por decisión HITL** (`wontfix.md:103`, H-08: "sin certificado; extiende DEVOPS-10 — Azure Trusted Signing cuando release público lo requiera").
3. **Decisión de distribución pública NO tomada:** Backlog (fila `DESKTOP-43` :716 removida al registrar el defer; nota vigente en :712) "⏸️ Bloqueada… Desbloquear tras decisión de distribución pública"; master plan Task 33 (:953) "con la decisión de distribución aún abierta"; `desktop/README.md` "**No public installer yet** … no public release channel / download page".
4. **Firma del updater obligatoria e indeshabilitable:** docs oficiales Tauri v2 updater ("Tauri's updater needs a signature… This cannot be disabled") — sin claves el updater no funciona; el E2E real exige el canal firmado.
5. **Endpoint propuesto (para la habilitación):** JSON estático en GitHub Releases (`latest.json` generado por `tauri-action`), per research §F3 + docs Tauri.
6. **Conclusión:** stop condition del plan → **defer** con trigger (decisión de distribución pública) + checklist. Sin BLOQUEO: el contrato se cumple por la rama defer (sancionada por el plan y el prompt del orquestador).

**Fuentes (verificadas por webfetch, 2026-10-04):** [Tauri v2 — Updater](https://v2.tauri.app/plugin/updater/) (signing obligatorio, `createUpdaterArtifacts`, `pubkey`/`endpoints`, static JSON, permissions `updater:default`) · `docs/dev/research/installer-personalizado/RESEARCH.md` (:45,:63,:115,:152).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — ¿distribución? NO tomada (→ defer) · ¿claves? NO disponibles (→ defer) |
| Pendientes de ejecución (downhill) | 5 steps |
| % completado | 20% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No aplica al diff (docs-only; sin trust boundaries, sin deps, sin secrets). El checklist **documenta** el cambio futuro de trust model (firma obligatoria, custodia de clave privada, pubkey embebida) para que la habilitación lo trate con el rigor correspondiente.
- [x] **PERFORMANCE** — No aplica (sin código, sin hot paths, sin serialización).

## Steps

### Step 1 — Task file canónico + decisión de defer
- **Archivos:** `docs/dev/tasks/DESKTOP-43.md` (nuevo)
- **Acción:** crear el task file con DISCOVERY completo (evidencia, contrato, rama defer), steps atómicos y gates.
- **Verify:** archivo existe + `node scripts/docs/check-docs.mjs` (kind `task` OK)
- **Resultado:** ✅ creado; `check-docs` verde; indexado (1522 docs, +2 propios)
- **Estado:** ✅

### Step 2 — Checklist de habilitación
- **Archivos:** `docs/dev/desktop/UPDATER_ENABLEMENT.md` (nuevo)
- **Acción:** checklist paso a paso (claves → wiring → CI release-mode → validación E2E → trust model) anclado a docs oficiales; trigger de reapertura explícito.
- **Verify:** `node scripts/docs/check-links.mjs` + `check-docs.mjs`
- **Resultado:** ✅ creado (kind `howto`); check-links/check-docs verdes; links relativos resueltos
- **Estado:** ✅

### Step 3 — Trust model / estado actual en ARCHITECTURE
- **Archivos:** `docs/dev/desktop/ARCHITECTURE.md`
- **Acción:** sección "Distribution & updates": canal actual (none, unsigned build-only) + updater diferido (pointer al checklist) + nota de trust model para la habilitación.
- **Verify:** `check-links` (link al checklist resuelve) + `check-docs`
- **Resultado:** ✅ sección agregada (22 líneas); link `./UPDATER_ENABLEMENT.md` resuelve
- **Estado:** ✅

### Step 4 — Registros de cierre
- **Archivos:** `docs/dev/avance/decisiones/wontfix.md` (§DEFER) · `docs/dev/avance/activo/desktop.md` (cierre) · `docs/dev/Backlog.md` (fila DESKTOP-43 removida + nota P46)
- **Acción:** registrar el defer con trigger + checklist; cierre en avance; remover fila del Backlog (el registro vive en wontfix/avance).
- **Verify:** `rg "DESKTOP-43" docs/dev/Backlog.md` → 0 filas de tarea; wontfix contiene trigger; avance contiene entrada
- **Resultado:** ✅ fila Backlog removida (queda solo la nota P46 con el registro); wontfix §DEFER con trigger + link al checklist; avance/desktop.md con entrada de cierre
- **Estado:** ✅

### Step 5 — Verify full + OCR + review P2-01 + commit LOCAL + cierre campaña
- **Archivos:** `docs/index.md` + `llms.txt` (generados), este task file (sync), commit
- **Acción:** `gen-index --write`; gates docs; OCR delegation; review P2-01 (vanta-review, contexto fresco); commit **LOCAL** `docs(desktop):` (pathspec propio); `campaign_update_task_state(taskId 33, completed)` con review payload; RESULTADO §7.
- **Verify:** gates verdes + veredicto review + commit local
- **Resultado:** ✅ gates verdes (8/8) + OCR 0 reviewables + review P2-01 **APPROVE** (vanta-review, 2 rondas: M1/M2 → fixes → L3) + commit LOCAL `docs(desktop):` con pathspec propio
- **Estado:** ✅

### Evidencia de verificación (post-implementación, 2026-10-04)

| Gate | Comando | Resultado |
|------|---------|-----------|
| check-links | `node scripts/docs/check-links.mjs` | ✅ exit 0 (2.1s) |
| check-docs | `node scripts/docs/check-docs.mjs` | ✅ exit 0 (2.5s) — kinds `task`/`howto` OK |
| gen-index | `node scripts/docs/gen-index.mjs --write` + `--check` | ✅ in sync (1522 docs, +2 propios) |
| docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | ✅ exit 0 (1.6s) |
| avance coverage | `pwsh scripts/check-avance-coverage.ps1` | ✅ exit 0 (1.6s) |
| fmt | `cargo fmt --check` | ✅ exit 0 (8.1s) — sin Rust tocado; gate barato |
| markdownlint | `npx markdownlint-cli2 <5 archivos>` | ✅ exit 0 (18.6s) |
| OCR delegation | `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ 0 reviewables (docs excluidos por diseño) — Critical/High 0 |

## RESULTADO (§7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5 total steps
PROXIMO_STEP: ninguno — orquestador: Task 33 registrado `completed` en campaña (taskId 33, payload review APPROVE); push diferido al owner (Regla 7)
COMMIT_HASH: ver git log — `docs(desktop):` LOCAL, sin push (+ `docs(task):` de este archivo)
ARCHIVOS: docs/dev/tasks/DESKTOP-43.md · docs/dev/desktop/UPDATER_ENABLEMENT.md · docs/dev/desktop/ARCHITECTURE.md · docs/dev/avance/decisiones/wontfix.md · docs/dev/avance/activo/desktop.md · docs/dev/Backlog.md · docs/index.md · llms.txt
VERIFY_CONTRATO: pasa (rama defer) — defer registrado con trigger + checklist de habilitación + trust model; gates docs 8/8 verdes; pipeline de releases y código intactos
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(plan Task 33 ✅ DO) D:no(rama defer sancionada por stop condition; sin símbolos públicos) V:no(verde) C:no(sin colaterales; WIP ajeno fuera del commit)
SKILLS_CARGADAS: ci-cd-and-automation, shipping-and-launch, documentation-and-adrs, documentation-skill, source-driven-development, doubt-driven-development, git-workflow-and-versioning (SDP v3; base auto: campaign-executor/progreso/ponytail)
```

## Dependencias

- **DESKTOP-41** (✅) — instaladores NSIS/MSI + smoke checklist; contexto del canal (unsigned, build-only).
- **DESKTOP-42** (✅) — bundles macOS/Linux + CI matrix (build-only).
- **DESKTOP-24/25** (✅) — empaquetado + CI desktop.
- **DEVOPS-10 / H-08** — firma diferida por HITL (premisa del defer).
- **Decisión de distribución pública** (owner) — trigger de reapertura.
- **En vuelo (no tocar):** PROV-12 (`providers/**` + workflows), master plan (modificado en el tree — pathspec), `opencode.jsonc`.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` — sesión fresca `ses_ef5c370f2ffeFMnoLIDDcHstAg` (2026-10-04), contexto distinto al implementador (P2-01).
- **Enfoque:** adversarial — legitimidad del defer vs contrato/stop condition; realidad de la evidencia (`rg`/`gh`/archivo:línea); exactitud técnica del checklist vs docs oficiales Tauri v2 + `action.yml` del SHA pineado; gates mecánicos; scope discipline.
- **Cómo se probó:** 6 gates re-ejecutados exit 0 (check-links/check-docs/gen-index/validate-docs-coverage/check-avance-coverage/markdownlint); `rg tauri-plugin-updater desktop/src-tauri` → 0 refs de código; `gh secret list` → solo `RELEASE_PLZ_TOKEN`; `action.yml@1deb371b` → input `uploadUpdaterJson` (no `includeUpdaterJson`); refs de línea verificadas; ronda 2: fixes M1/M2/L1/L2/N1/N2/O1 re-verificados + `rg` sin stale (`:98`/`§UX`/casing); L3 (`:75`) aplicado post-verdicto y re-verificado (`rg includeUpdaterJson docs/` → 0 matches).
- **Checklist anti-hábitos tóxicos:** sin outputs inventados ✅ (todo claim re-verificado; M/L de precisión, no fabricación) · sin verificación salteada ✅ · sin "done" sin criterios ✅ (DoD multi-nivel) · parcial no reportado como OK ✅ · sin self-review ✅.
- **Veredicto:** ✅ **APPROVE** (R1: CHANGES-REQUIRED M1/M2 + L1/L2 → fixes aplicados → R2: APPROVE; L3 Low cerrado post-verdicto con `rg` en 0).

## Notas

- **(DISCOVERY) Decisión clave:** la rama defer está sancionada por la stop condition del plan y por el prompt del orquestador ("el contrato se cumple igual — stop condition sancionada"); no se reportó BLOQUEO porque el contrato se cumple por esa rama (el E2E simulado con keypair de prueba era viable, pero habría dejado un plugin inerte y un trust model cambiado sin decisión de distribución — "no construir a ciegas").
- **(DISCOVERY) Sin claves:** `gh secret list` (2026-10-02) solo `RELEASE_PLZ_TOKEN`; sin `TAURI_SIGNING_*`. La firma está DIFERIDA por HITL (H-08/DEVOPS-10) — no se generaron claves ni se decidió por el owner.
- **(S2) NOTICED BUT NOT TOUCHING:** `desktop/README.md` §Installer status ("No public installer yet") sigue correcto — no hay canal público; el defer no lo cambia.
- **(S2) NOTICED BUT NOT TOUCHING:** no se creó ADR nuevo — el plan pedía "ADR corto **o** sección en ARCHITECTURE.md" para el trust model; se eligió sección + ADR diferido al PR de habilitación (Regla 5).
- **(S3) validate_scope advisory:** los paths nuevos (`docs/dev/tasks/DESKTOP-43.md`, `docs/dev/desktop/UPDATER_ENABLEMENT.md`, `llms.txt`) reportan "OUTSIDE declared blast radius" — el blast radius del plan lista archivos de código del updater; la rama defer es docs-only por contrato. Advisory, no bloqueante.
- **(S3) OCR:** 0 archivos reviewables del diff propio (docs excluidos por diseño) — Critical/High 0.
- **Learning para la habilitación:** el input del tauri-action pineado es `uploadUpdaterJson` (no `includeUpdaterJson`) — verificado contra `action.yml@1deb371b`; el checklist lo documenta.
