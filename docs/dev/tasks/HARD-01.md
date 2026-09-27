# HARD-01: Rails de breaking changes (semver-checks + public-api + docs de migración)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 1, Fase F0)
- **Fuente:** plan file Task 1 (`:50-72`) + investigación R1 2026-09-26 (rec. #2/#3/#4/#5/#6/#7: cargo-semver-checks, cargo-public-api, DEPRECATIONS/COMPATIBILITY/UPGRADE, checklist 1.0 estilo DuckDB)
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** Mixto (CI + test snapshot + docs)
- **Ruta:** vanta-lead
- **Turns estimados:** 15-25
- **Creado:** 2026-09-26T19:53 · **last-synced:** 2026-09-27
- **Estado:** ⏳ IN PROGRESS — 6/7 ✅ + Step 7 🟡 PARCIAL (verificación ✅; commit local delegado al LEAD; ronda 2 de review pendiente)
- **Incógnitas (uphill):** 0 — resuelta (Step 2: `--simplified` -s + exclude-list vacía, evidencia 2 corridas 0 drift)
- **Pendientes (downhill):** 1 — commit local (LEAD) + Review P2-01 (agente distinto)

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
| `nextTask` | HARD-04 |

```
=== RECITATION ===
Objetivo activo: HARD-01 — rails de breaking changes (semver-checks + public-api + docs migración)
Estado: in-progress (6/7 ✅ + Step 7 🟡 PARCIAL; verificación completa ✅; commit local delegado al LEAD; review ronda 2 pendiente)
Última acción: snapshot+test+CI+3 docs completos y verificados (fmt/docs/public_api/clippy via MCP); re-run final check-release en curso
Resultado: PARTIAL (funcional completo; pendiente commit del LEAD + Review P2-01)
Próxima acción: Step 7 — LEAD: commit local en 3 bloques (test: snapshot+Cargo.toml · ci: ci-rust.yml · docs: DEPRECATIONS+COMPATIBILITY+VERSIONING+UPGRADE); luego Review P2-01 (vanta-review)
Contrato: check-release exit 100 con 6 findings TRIADOS y documentados (COMPATIBILITY §Pre-release deltas) · public_api verde con snapshot en disco (8480 líneas, hash 7BA24E…5DBC) · validate-docs-coverage ✅ 0 gaps
Invariantes: release-plz único versionador; semver_check intacto; 11 superficies intactas; >5min → job dedicado (no Fast Gate); sin push
Deuda: (1) check-release exit 100 en develop es esperado hasta el bump 0.8.0 (documentado; no se degradan lints); (2) toolchain nightly flotante del snapshot (drift de formato → pin si flakea); (3) límite tooling: campaign_verify_cmd normaliza exit≠0 a 1 (no observable el 100; evidencia raw/log) — para orquestador
Próxima tarea si completa: HARD-04
last-synced: 2026-09-27
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
| Incógnitas abiertas (uphill) | 0 — resuelta con evidencia real (Step 2) |
| Pendientes de ejecución (downhill) | 1 — commit local (LEAD) + Review P2-01 |
| % completado | ~95% (funcional completo; cierre de commit delegado) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No toca trust boundaries ni input de usuario; no agrega deps de runtime (tooling CI/dev). Tool en CI vía action pinneada (RULES.md §3: SHA + `# vX.Y.Z`). Sin secrets.
- [x] **PERFORMANCE** — N/A hot path. Sí toca presupuesto de CI wall-time: el step nuevo debe caber <5min o moverse a nightly (stop condition); medir y registrar.

## Steps

### Step 1: Baseline semver-checks + triage + decisión de lints
- **Archivos:** `release-plz.toml` (verificación), `Cargo.toml` (solo si la config de lints lo requiere — validar sintaxis contra docs oficiales)
- **Acción:** verificar presencia (`cargo-semver-checks --version`); correr `cargo semver-checks check-release`; triar cada finding (breaking aceptado por MINOR en 0.x → documentar en COMPATIBILITY; false positive → exclude documentado). Registrar baseline en Notas del task.
- **Verify:** `cargo semver-checks --version` OK + `cargo semver-checks check-release` exit 0 o findings listados con disposición
- **Estado:** ✅ COMPLETED
- **Evidencia:** version 0.49.0 ✅ (MCP passed). `check-release` exit 100: 6 lint families (190 pass / 6 fail / 57 skip), todas en la superficie CLI (#9) — `src/cli.rs` + `src/cli_handlers/` — de `f6c395ef` (API-07) + `8e55e853` (WIRE-10), post-0.7.0. Disposición: breaking aceptado bajo 0.x MINOR (rumbo 0.8.0) → `docs/api/COMPATIBILITY.md` § Pre-release deltas. Sin excludes ni lints degradados. `vantadb-wasm` skipped (npm, sin baseline crates.io). Log: `target/session-api01/semver-check-release.log`

### Step 2: Snapshot public-api + exclude-list (resuelve uphill #1)
- **Archivos:** `tests/api/public-api.txt` (nuevo), `Cargo.toml` (dev-dep solo si se usa la crate `public-api`)
- **Acción:** generar snapshot (`cargo +nightly public-api -p vantadb --simplified` o formato acordado contra docs oficiales); aplicar exclude-list mínima (items experimentales/ruidosos) documentada en header del archivo.
- **Verify:** generación reproducible: 2 corridas del mismo comando → `git diff --exit-code tests/api/public-api.txt` (0 drift)
- **Estado:** ✅ COMPLETED
- **Evidencia:** `tests/api/public-api.txt` generado (8480 líneas, 635 KB) con rustdoc nightly + public-api 0.52.2 `omit_blanket_impls(true)` (= `--simplified` -s), default features, exclude-list VACÍA (documentada en el header del .txt). Uphill #1 RESUELTA: formato = golden simplificado sin excludes; census: 1075 auto-trait + 911 derived impls (23%) aceptados a propósito (visibilidad completa, churn determinista). Reproducibilidad: corrida 1 (UPDATE, 364s) vs corrida 2 (GREEN, 81s) → 0 drift (hash SHA256 `7BA24E…5DBC` estable).

### Step 3: Test `public_api` (RED→GREEN)
- **Archivos:** `tests/api/public_api.rs` (nuevo), `Cargo.toml` (`[[test]] name = "public_api"` + `path`, patrón `:440,444`)
- **Acción:** test que regenera la superficie y compara contra el snapshot commiteado (RED: inducir drift trivial → falla con diff legible; GREEN: sin drift → pasa; documentar cómo actualizar el snapshot a propósito).
- **Verify:** `cargo nextest run -p vantadb --test public_api` verde + RED confirmado con drift inducido (luego revertido)
- **Estado:** ✅ COMPLETED
- **Evidencia:** RED: drift inducido en golden (línea 9) → exit 100 con diff legible (`line 9: snapshot: pub mod vantadb_drift_demo / current: pub mod vantadb`); restaurado (hash íntegro). GREEN: MCP verify passed (exit 0). Iteración RED: mensaje re-escrito de `assert_eq!` (volcaba 8k líneas) a `assert!` + diff acotado (≤10 líneas + conteo) — verificado con RED-2. Update flow: `VANTADB_PUBLIC_API_UPDATE=1`.

### Step 4: Wiring CI (presencia semver + snapshot)
- **Archivos:** `.github/workflows/ci-rust.yml` (job `semver-checks:92-128` + step snapshot)
- **Acción:** (a) step de presencia `cargo-semver-checks --version` obligatorio (no `continue-on-error`); (b) decidir/documentar scope develop-PRs (default evidencia: mantener main-only `:99-103`); (c) ubicar el snapshot test — fast si <5min medido, si no → nightly (coord. HARD-02, stop condition).
- **Verify:** YAML válido (actionlint si disponible; fallback parse) + comando exacto del step corrido en local + wall-time registrado (<5min o nightly)
- **Estado:** ✅ COMPLETED
- **Evidencia:** `.github/workflows/ci-rust.yml`: (a) step presencia `cargo semver-checks --version` (obligatorio, post-install); (b) scope develop-PRs = mantener main-only (default evidencia, comentario actualizado a COMPATIBILITY); (c) job nuevo `public-api-snapshot` (checkout pinneado, rust-setup nightly + nextest + sysdeps, timeout 45, `if` main/PR-main, concurrency group) — NO entra al Fast Gate: wall-time medido 364s primera corrida fría de rustdoc / 81s warm / 23s warm-cacheado (>5min frío → job dedicado). actionlint ✅ exit 0 (fix: `:` en name rompía YAML).

### Step 5: `docs/api/DEPRECATIONS.md` + `docs/api/COMPATIBILITY.md` (nuevos)
- **Archivos:** `docs/api/DEPRECATIONS.md` (nuevo), `docs/api/COMPATIBILITY.md` (nuevo), `docs/api/VERSIONING.md` (links)
- **Acción:** DEPRECATIONS: política de referencia (link a `VERSIONING.md:89-94`, sin duplicar) + formato de alias con fecha de remoción + listado vigente (hoy vacío — declararlo). COMPATIBILITY: matriz por las 11 superficies (`VERSIONING.md:36-48`) + checklist 1.0 con exit criteria verificables (R1 rec. #7).
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` verde + links resuelven (0 rotos)
- **Estado:** ✅ COMPLETED
- **Evidencia:** `docs/api/DEPRECATIONS.md` (registro: política referenciada, formato de alias con ventana de remoción, lista vigente VACÍA declarada) + `docs/api/COMPATIBILITY.md` (rails, matriz 11 superficies × gate mecánico, deltas pre-release, exit criteria 1.0, gaps honestos). `VERSIONING.md`: links a ambos en §Enforcement y §Deprecation policy (11 superficies intactas). MCP verify `validate-docs-coverage` ✅ 0 gaps (28 SDK / 60 config / 44 CLI / 51 Python / 47 MCP / 10 skills-mirror). **Link-check changeset (4 docs): 58 links relativos → 0 rotos + 0 anchors malos** (62 totales con externos; 2026-09-27 post-fix ronda 1 de review — supersede el subset 23/23 inicial).

### Step 6: `UPGRADE.md` poblado/verificado
- **Archivos:** `docs/user/operations/UPGRADE.md`, `docs/api/VERSIONING.md` (link existente `:102` — verificar)
- **Acción:** verificar estructura contra 0.7.x real (entrada 0.7.0 si hubo cambios consumer-facing; si no, declarar "sin breaking"); respetar separación CHANGELOG = release notes / UPGRADE = guía (pre-mortem F3).
- **Verify:** `rg "Upgrading to 0.7" docs/user/operations/UPGRADE.md` presente (o declaración "sin breaking 0.7") + links válidos
- **Estado:** ✅ COMPLETED
- **Evidencia:** entrada `### Upgrading to 0.7.0 (from 0.6.x)` añadida (released 2026-09-25; sin breaking de consumidor con evidencia `git diff v0.6.1 v0.7.0 -- src/` = 5 líneas internas; reorg web/docs documentada) + pointer a deltas pre-0.8.0 en COMPATIBILITY. `rg "Upgrading to 0.7"` ✅. `last_reviewed` → 2026-09-27. Separación F3 respetada: CHANGELOG = release notes, UPGRADE = guía de migración (sin duplicar).

### Step 7: Cierre (contrato + DoD + commit local)
- **Archivos:** — (todo el set)
- **Acción:** correr contrato completo; `pwsh dev-tools/verify_changed.ps1` (o `verify.ps1`); escribir recitation; commit(s) locales por bloque (NUNCA push — Regla 7); learnings vía `campaign_memory_write`.
- **Verify:** Contrato (3 comandos) + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings`
- **Estado:** 🟡 PARCIAL — verifies mecánicos ✅; commit local delegado al LEAD (instrucción del orquestador: el sub-agente NO commitea)
- **Evidencia:** fmt ✅ (MCP 4.5s) · docs ✅ (MCP 2.5s) · `cargo nextest -p vantadb --test public_api` ✅ (MCP 23s) · `cargo clippy -p vantadb --test public_api -- -D warnings` ✅ (MCP 86.7s — superset del DoD: lib + target del test nuevo) · check-release re-run final: ver §Notas. Commit(s) pendientes: `test:` (snapshot+test+Cargo.toml) · `ci:` (ci-rust.yml) · `docs:` (3 docs + VERSIONING).

## Dependencias
- Ninguna (paralela en F0). Coordina con **HARD-02** si el gate >5min (recepción en nightly) y con **DEF-04** (naming freeze referencia DEPRECATIONS). Siguiente: HARD-04.

## Review (GATE — agente distinto, P2-01)

> Placeholder. Lo completa un agente DISTINTO al implementador al ejecutar la tarea (vanta-review/vanta-audit; fallback sin subagentes: doubt-driven-development degradado + escalado al owner). Sin review registrado, NO se marca ✅ COMPLETED.

- **Revisor:** vanta-review — ronda 1 completada (sesión fresca, contexto distinto: `ses_f1e5eaf82ffeiJrZgx6UrRp3iK`)
- **Enfoque:** ¿el approach de rails es correcto? ¿snapshot con excludes bien acotada (no CI rojo crónico)? ¿docs de migración reales (no checklist decorativo)?
- **Cómo se probó:** evidencia en §Steps + logs `target/session-api01/` (semver-checks exit 100 raw ×2, RED/GREEN del snapshot, validate-docs-coverage 0 gaps, fmt/clippy ✅). Link-check changeset post-fix: 58 links relativos → 0 rotos, 0 anchors malos.
- **Ronda 1 (fresca, `ses_f1e5eaf82ffeiJrZgx6UrRp3iK`):** ❌ REQUEST CHANGES — 1 REQUIRED (link `UPGRADE.md:69` `../api/` → `../../api/`) + 3 optional (a: anchor `#pre-1.0…` → `#pre-10…`; b: claim cold-run reemplazado por corridas reales 149.7s/391.1s; c: conteo de links real 58→0). **Fix aplicado + re-check 0 rotos (2026-09-27)**; ronda 2 de confirmación pendiente.
- **Ronda 2 (fresca, mismo revisor `ses_f1e5eaf82ffeiJrZgx6UrRp3iK`):** ✅ APPROVE — link-check re-ejecutado (58 links relativos → 0 rotos + 0 anchors malos), anchor `#pre-10-stability-contract` validado contra el heading real, claim de tiempos == logs reales, sin regresiones (docs-only). (2026-09-27)
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
- **Veredicto:** ✅ APPROVE (ronda 2, `ses_f1e5eaf82ffeiJrZgx6UrRp3iK`) — ronda 1 ❌ (1 required + 3 optional) → fix aplicado y re-verificado (0 links rotos)

## Notas
- Consume R1 (rec. #2/#3/#4/#5/#6/#7). `feat!` → MINOR automático en 0.x ya lo maneja release-plz — NUNCA tocar versiones/CHANGELOG/tags a mano.
- Si se decide excluir lints semver o fijar scope CI, registrar decisión (Regla 5: ADR corto o `campaign_memory_write`) con evidencia.
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) como input del reviewer al cierre (Critical/High bloquean commit).
- DoD Release N/A justificado: task de rails/docs; se consume en el corte 0.8.0 (F3).

## Run log 2026-09-27 (implementador — evidencia por step en §Steps)

- **Resumen:** rails completos — snapshot `tests/api/public-api.txt` (8480 líneas) + test `public_api.rs` + job CI `public-api-snapshot` + presence check + `DEPRECATIONS.md` + `COMPATIBILITY.md` + `UPGRADE.md` 0.7.0. Contrato: check-release 100 triado+documentado · public_api ✅ · docs ✅.
- **Decisiones registradas:** (1) lints semver SIN cambios ni excludes — los 6 findings son breaking reales de la superficie CLI (API-07/WIRE-10) aceptados bajo 0.x MINOR; degradar debilitaría el gate (trigger de revisión: si CI rojo 2 corridas seguidas); (2) snapshot formato `-s` sin excludes (0 drift en 2 corridas); (3) CI job dedicado (no Fast Gate: >5 min cold medido — 364s primera rustdoc + deps); (4) scope develop-PRs = mantener main-only (default documentado, comentario actualizado).
- **Budget:** `campaign_verify_cmd` agotó 120 min wall-time por builds cold (semver ~75 min acumulados + rustdoc) → `campaign_budget_reset(HARD-01)` aplicado (motivo: builds cold, no falla de código); verifies re-ejecutados ✅.
- **Incidente check-release (resuelto — 2 causas):** (a) exportar `CARGO_TARGET_DIR` re-homó la caché interna de semver-checks → re-build largo; (b) `campaign_verify_cmd` normaliza exit≠0 a `exitCode:1` (probe: `pwsh exit 42` → 1) → el exit **100** real (findings) no es observable vía MCP. Evidencia autoritativa: corridas shell directas con `EXITCODE=100` y las 6 familias de findings (`target/session-api01/semver-check-release.log`; re-run MCP bare con stdout completo de findings). Deuda de tooling (normalización de exit codes MCP) → para el orquestador (no bloqueante).
- **Review P2-01:** ✅ APPROVE (ronda 2). Ronda 1 ❌ REQUEST CHANGES (1 required link `UPGRADE.md:69` + 3 optional) → fix aplicado y re-verificado (58 links → 0 rotos). Evidencia: §Review + logs `target/session-api01/`. OCR delegation: preview generado (7 archivos), delegate pendiente (advisory).
