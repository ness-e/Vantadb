# HARD-01: Rails de breaking changes (semver-checks + public-api + docs de migración)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 1, Fase F0)
- **Fuente:** plan file Task 1 (`:50-72`) + investigación R1 2026-09-26 (rec. #2/#3/#4/#5/#6/#7: cargo-semver-checks, cargo-public-api, DEPRECATIONS/COMPATIBILITY/UPGRADE, checklist 1.0 estilo DuckDB)
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** Mixto (CI + test snapshot + docs)
- **Ruta:** vanta-lead
- **Turns estimados:** 15-25
- **Creado:** 2026-09-26T19:53 · **last-synced:** 2026-09-26T19:53
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 abierta (formato final del snapshot/exclude-list) — debe bajar a 0 para ✅
- **Pendientes (downhill):** 7 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `release-plz.toml:22` (`semver_check=true`, ya activo) ← `release.yml`; `docs/api/VERSIONING.md` ← `CONTRIBUTING.md:95`, `docs/user/operations/UPGRADE.md:13`, docs/api refs; `.github/workflows/ci-rust.yml:92-128` (job `semver-checks`, RELEASE-01) ← required checks de `main` |
| Callees | `release-plz` (upstream), `cargo-semver-checks`, `cargo-public-api`/crate `public-api` (tooling nuevo CI/dev), `scripts/validate-docs-coverage.ps1`, `tests/api/public_api.rs` (nuevo), `Cargo.toml` (`[[test]]` patrón `:440,444,539` + eventual dev-dep) |
| Implicaciones | NO es feature-add: 0 símbolos `pub` nuevos, 0 cambio de comportamiento runtime. Afecta CI (step nuevo de snapshot: fast si <5min medido, si no → nightly coord. HARD-02). `VERSIONING.md` conserva sus 11 superficies — solo gana enforcement + links. `UPGRADE.md` ya existe (ver Investigation Notes). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `release-plz.toml` (42 L), `docs/api/VERSIONING.md` (102 L), `docs/user/operations/UPGRADE.md` (117 L), `.github/workflows/ci-rust.yml` (667 L), `CONTRIBUTING.md` (297 L)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `Cargo.toml:440,444,539` (registro `[[test]]` para `tests/api/*` — sin auto-discovery en subdirs); `scripts/validate-docs-coverage.ps1` (gate docs); `cliff.toml` (changelog vía release-plz); `tests/api/` (3 targets existentes: `structured_api_v2`, `openapi_yaml_parity`, `python`)
- **Archivos que referencian a los editados (referencias entrantes):** `CONTRIBUTING.md:95` → VERSIONING.md; `VERSIONING.md:102` → UPGRADE.md; `CI_POLICY.md:61` describe el job `semver-checks`; `release-plz.toml:29` → `docs/CHANGELOG.md`; release.yml consume release-plz
- **Veredicto impacto:** medio — todo aditivo (snapshot + test + CI step + docs); riesgo real = ensuciar CI si el snapshot no tiene exclude-list acotada (riesgo #1 del plan) → mitigación en Step 2/4.

## Contrato
"`cargo semver-checks check-release` exit 0 (o findings triados y documentados) Y `cargo test -p vantadb --test public_api` verde con snapshot commiteado Y `DEPRECATIONS.md` + `COMPATIBILITY.md` + `UPGRADE.md` (en `docs/user/operations/`) existen con 0 links rotos (validate-docs-coverage verde)"

## Spec (SDD — Phase 1b: NO feature-add — no agrega símbolos/contratos públicos; decisiones documentadas igual)

> Phase 1b: sin señales (`pub fn`/endpoint/binding/componente nuevos) → Spec no obligatoria, pero se documentan las decisiones ya resueltas por evidencia (plan + código real) y la abierta.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Formato del snapshot | A) golden file `public-api.txt` (`--simplified`) + test de diff (estable, portable) / B) rustdoc JSON crudo (verboso, ruidoso) | A | ✅ decidido-por-evidencia: pre-mortem F1 del plan (`:63`) + R1; ref: plan `:55` |
| 2 | Scope del job semver en CI | A) mantener main-only (develop pre-release = ruido; comentario de diseño existente) + verificación de presencia obligatoria / B) correr también en PRs a develop | A | ✅ decidido-por-evidencia: `ci-rust.yml:99-103` (13 majors acumulados pre-release) |
| 3 | ¿Gate nuevo en fast o nightly? | A) fast si <5min / B) nightly si >5min (coord. HARD-02) | regla | ✅ decidido-por-evidencia: stop condition del plan (`:64`) — medir y elegir |
| 4 | Ubicación de UPGRADE | A) `docs/user/operations/UPGRADE.md` (existente, linkeado desde `VERSIONING.md:102`) / B) `docs/UPGRADE.md` (ruta del plan, stale) | A | ✅ decidido-por-evidencia: archivo A existe en disco (117 L) |
| 5 | Exclude-list del snapshot | A) exclude mínima documentada en header del `.txt` / B) sin excludes (ruido) | A | ⛔ abierta (uphill #1) — se resuelve en Step 2 con el output real |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** release-plz sigue siendo el ÚNICO que toca versiones/CHANGELOG/tags (AGENTS.md Regla 7) — este task NO versiona nada; `semver_check = true` (`release-plz.toml:22`) no se degrada; las 11 superficies de `VERSIONING.md:36-48` no se alteran (solo se referencian DEPRECATIONS/COMPATIBILITY); si el gate nuevo >5min NO entra al fast gate (stop condition → nightly).
- **Comandos de verificación:** Contrato (3 comandos) + `pwsh scripts/validate-docs-coverage.ps1` + `pwsh dev-tools/verify.ps1`.
- **Deuda pendiente:** si appetite >3d → entregar snapshot + DEPRECATIONS y diferir COMPATIBILITY (stop condition del plan `:64`), registrado en Notas.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HARD-01: …` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HARD-02 |

```
=== RECITATION ===
Objetivo activo: HARD-01 — rails de breaking changes (semver-checks + public-api + docs migración)
Estado: pending
Última acción: task file creado (Plan: master-roadmap Task 1)
Resultado: —
Próxima acción: Step 1 — baseline `cargo semver-checks check-release` + triage
Contrato: ver ## Contrato
Invariantes: release-plz único versionador; semver_check intacto; 11 superficies intactas; >5min → nightly
Deuda: ninguna al abrir
Próxima tarea si completa: HARD-02
last-synced: 2026-09-26
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — aditivo (snapshot + test + docs + CI step); no introduce suppressions, stubs ni dependencias de runtime (cargo-public-api es tooling CI/dev).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (3 comandos) verde + capa determinista (`cargo fmt --check`, `cargo clippy -p vantadb -- -D warnings`, `cargo nextest run -p vantadb --test public_api`) |
| **Commit** | Atómico(s) por bloque (snapshot/test · CI · docs), conventional (`test:`/`ci:`/`docs:` según toque), verificación mecánica previa, **sin push** (Regla 7 — push al final del plan) |
| **Release** | N/A en este task (no versiona ni publica); los rails se consumen en el corte **0.8.0 (F3)**: re-verificar 1 vez ahí. Justificado por diseño. |

## Herramientas necesarias

- cargo-semver-checks (`cargo-semver-checks --version`), cargo-public-api (+ toolchain `nightly` para rustdoc JSON — el job CI ya usa nightly, `ci-rust.yml:116`), `campaign_verify_cmd`, `pwsh scripts/validate-docs-coverage.ps1`, codegraph_explore (blast radius)
- Leer ANTES de editar: `.opencode/rules/release-ci.md` (área release/CI) + `.opencode/rules/api-contract.md` (semver/superficies públicas)

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `ci-cd-and-automation` · `git-workflow-and-versioning` · `documentation-and-adrs` · `api-and-interface-design` · `test-driven-development` · `systematic-debugging` · `doubt-driven-development` · **PINNED (policy):** `ci-cd-and-automation`, `git-workflow-and-versioning`, `documentation-and-adrs`, `api-and-interface-design`, `test-driven-development`, `systematic-debugging`

## Investigation Notes

- `ci-rust.yml:92-128` YA tiene job `semver-checks` (RELEASE-01) con `if: main || base_ref == main`. El gap real NO es "no corre en PR" (corre en PRs a main) sino: (a) no corre en PRs a develop — por diseño documentado `:99-103`; (b) sin verificación de presencia (`--version`) → riesgo de skip silencioso (pre-mortem F2); (c) sin snapshot `cargo-public-api`.
- `release-plz.toml:22`: `semver_check = true` confirmado.
- `UPGRADE.md` existe en `docs/user/operations/UPGRADE.md` (117 L, plantilla en `:98-110`, última entrada 0.6.1) — el plan cita `docs/UPGRADE.md` (stale). Trabajo = poblar/verificar, no crear.
- `tests/api/` tiene 3 targets registrados (`Cargo.toml:440,444,539`) → `public_api` debe registrarse igual (`[[test]] name/path`); no hay auto-discovery de subdirs.
- `Cargo.toml` sin dev-deps de `public-api` (grep 0 hits) → decidir crate vs invocación CLI, validando contra docs oficiales (source-driven).
- `VERSIONING.md:89-94` ya define la política de deprecación (≥1 MINOR antes de remoción) → `DEPRECATIONS.md` la referencia (no duplica), lista deprecaciones vigentes (hoy: ninguna declarada) + formato alias con fecha de remoción.
- R1: checklist 1.0 con exit criteria verificables (estilo DuckDB) → sección en `COMPATIBILITY.md`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — formato final del snapshot/exclude-list (se resuelve con evidencia del output real en Step 2) |
| Pendientes de ejecución (downhill) | 7 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No toca trust boundaries ni input de usuario; no agrega deps de runtime (tooling CI/dev). Tool en CI vía action pinneada (RULES.md §3: SHA + `# vX.Y.Z`). Sin secrets.
- [x] **PERFORMANCE** — N/A hot path. Sí toca presupuesto de CI wall-time: el step nuevo debe caber <5min o moverse a nightly (stop condition); medir y registrar.

## Steps

### Step 1: Baseline semver-checks + triage + decisión de lints
- **Archivos:** `release-plz.toml` (verificación), `Cargo.toml` (solo si la config de lints lo requiere — validar sintaxis contra docs oficiales)
- **Acción:** verificar presencia (`cargo-semver-checks --version`); correr `cargo semver-checks check-release`; triar cada finding (breaking aceptado por MINOR en 0.x → documentar en COMPATIBILITY; false positive → exclude documentado). Registrar baseline en Notas del task.
- **Verify:** `cargo semver-checks --version` OK + `cargo semver-checks check-release` exit 0 o findings listados con disposición
- **Estado:** ⬜ PENDING

### Step 2: Snapshot public-api + exclude-list (resuelve uphill #1)
- **Archivos:** `tests/api/public-api.txt` (nuevo), `Cargo.toml` (dev-dep solo si se usa la crate `public-api`)
- **Acción:** generar snapshot (`cargo +nightly public-api -p vantadb --simplified` o formato acordado contra docs oficiales); aplicar exclude-list mínima (items experimentales/ruidosos) documentada en header del archivo.
- **Verify:** generación reproducible: 2 corridas del mismo comando → `git diff --exit-code tests/api/public-api.txt` (0 drift)
- **Estado:** ⬜ PENDING

### Step 3: Test `public_api` (RED→GREEN)
- **Archivos:** `tests/api/public_api.rs` (nuevo), `Cargo.toml` (`[[test]] name = "public_api"` + `path`, patrón `:440,444`)
- **Acción:** test que regenera la superficie y compara contra el snapshot commiteado (RED: inducir drift trivial → falla con diff legible; GREEN: sin drift → pasa; documentar cómo actualizar el snapshot a propósito).
- **Verify:** `cargo nextest run -p vantadb --test public_api` verde + RED confirmado con drift inducido (luego revertido)
- **Estado:** ⬜ PENDING

### Step 4: Wiring CI (presencia semver + snapshot)
- **Archivos:** `.github/workflows/ci-rust.yml` (job `semver-checks:92-128` + step snapshot)
- **Acción:** (a) step de presencia `cargo-semver-checks --version` obligatorio (no `continue-on-error`); (b) decidir/documentar scope develop-PRs (default evidencia: mantener main-only `:99-103`); (c) ubicar el snapshot test — fast si <5min medido, si no → nightly (coord. HARD-02, stop condition).
- **Verify:** YAML válido (actionlint si disponible; fallback parse) + comando exacto del step corrido en local + wall-time registrado (<5min o nightly)
- **Estado:** ⬜ PENDING

### Step 5: `docs/api/DEPRECATIONS.md` + `docs/api/COMPATIBILITY.md` (nuevos)
- **Archivos:** `docs/api/DEPRECATIONS.md` (nuevo), `docs/api/COMPATIBILITY.md` (nuevo), `docs/api/VERSIONING.md` (links)
- **Acción:** DEPRECATIONS: política de referencia (link a `VERSIONING.md:89-94`, sin duplicar) + formato de alias con fecha de remoción + listado vigente (hoy vacío — declararlo). COMPATIBILITY: matriz por las 11 superficies (`VERSIONING.md:36-48`) + checklist 1.0 con exit criteria verificables (R1 rec. #7).
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` verde + links resuelven (0 rotos)
- **Estado:** ⬜ PENDING

### Step 6: `UPGRADE.md` poblado/verificado
- **Archivos:** `docs/user/operations/UPGRADE.md`, `docs/api/VERSIONING.md` (link existente `:102` — verificar)
- **Acción:** verificar estructura contra 0.7.x real (entrada 0.7.0 si hubo cambios consumer-facing; si no, declarar "sin breaking"); respetar separación CHANGELOG = release notes / UPGRADE = guía (pre-mortem F3).
- **Verify:** `rg "Upgrading to 0.7" docs/user/operations/UPGRADE.md` presente (o declaración "sin breaking 0.7") + links válidos
- **Estado:** ⬜ PENDING

### Step 7: Cierre (contrato + DoD + commit local)
- **Archivos:** — (todo el set)
- **Acción:** correr contrato completo; `pwsh dev-tools/verify_changed.ps1` (o `verify.ps1`); escribir recitation; commit(s) locales por bloque (NUNCA push — Regla 7); learnings vía `campaign_memory_write`.
- **Verify:** Contrato (3 comandos) + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings`
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (paralela en F0). Coordina con **HARD-02** si el gate >5min (recepción en nightly) y con **DEF-04** (naming freeze referencia DEPRECATIONS). Siguiente: HARD-02.

## Review (GATE — agente distinto, P2-01)

> Placeholder. Lo completa un agente DISTINTO al implementador al ejecutar la tarea (vanta-review/vanta-audit; fallback sin subagentes: doubt-driven-development degradado + escalado al owner). Sin review registrado, NO se marca ✅ COMPLETED.

- **Revisor:** ⬜ (designar al ejecutar — distinto del implementador)
- **Enfoque:** ¿el approach de rails es correcto? ¿snapshot con excludes bien acotada (no CI rojo crónico)? ¿docs de migración reales (no checklist decorativo)?
- **Cómo se probó:** ⬜ (pegar salidas reales: semver-checks, RED/GREEN del snapshot, validate-docs-coverage)
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⬜ pendiente

## Notas
- Consume R1 (rec. #2/#3/#4/#5/#6/#7). `feat!` → MINOR automático en 0.x ya lo maneja release-plz — NUNCA tocar versiones/CHANGELOG/tags a mano.
- Si se decide excluir lints semver o fijar scope CI, registrar decisión (Regla 5: ADR corto o `campaign_memory_write`) con evidencia.
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) como input del reviewer al cierre (Critical/High bloquean commit).
- DoD Release N/A justificado: task de rails/docs; se consume en el corte 0.8.0 (F3).
