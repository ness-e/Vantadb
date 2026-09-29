---
title: "SCH-08: Corte 0.8.0 — migration guide + CHANGELOG + release notes"
kind: task
description: "UPGRADE.md §'Upgrading to 0.8.0' publicado (campos v2 + semántica valid/transaction + AS OF + pasos de migración/backfill + backup) Y auditoría release-plz del tramo (remedio al bump) Y release notes revisadas; merge/release = lane owner."
---

# SCH-08: Corte 0.8.0 — migration guide + CHANGELOG + release notes

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 33 (F3) · **Origen:** plan L865-890; ADR-0046 §Plan de migración/§D1d/§D7/§D4c
- **Fuente del prompt:** sub-agente vanta-docs (orquestador pipeline) — wave F3.6 (única en vuelo); branch `develop`
- **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴 · **Tipo:** docs/release-readiness (guide + audit + notes; sin merge/publish)
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ⏳ IN PROGRESS — guía + auditoría release-plz + notes draft ✅; pendiente **commit (LEAD)** · **Incógnitas (uphill):** 0 · **Pendientes (downhill):** §Pendientes
- **Gate D (question-gates):** pre-respondido por el orquestador — alcance LOCAL explícito (push/merge/release bloqueados por política owner: push solo al completar el plan + OK; PRs a main solo desde develop + OK). Sin question al usuario.
- **Gate V (question-gates):** no disparado (0 fallas de verify; 0 gates P/D/V/C disparados).

## Contrato (verbatim del plan, L875)
> "UPGRADE.md §'Upgrading to 0.8.0' publicado (campos v2 + semántica valid/transaction + `AS OF` + pasos de migración/backfill + backup pre-upgrade) Y entrada `[0.8.0]` en `docs/CHANGELOG.md` con breaking changes marcados (`feat!`/`BREAKING CHANGE`) Y release notes del GitHub release (release-plz) revisadas Y corte ejecutado SOLO vía release-plz (Release PR merge — nunca tag/versión/CHANGELOG a mano, Regla 7), con verificación post-release de artefactos (crates/wheels/npm)"

**Alcance LOCAL de esta ejecución (orquestador):** (1) guía §0.8.0; (2) auditoría de commits + remedio exacto del bump (sin reescribir historia, sin push, sin tocar `docs/CHANGELOG.md` a mano); (3) contenido de release notes para revisión post-push; (4) handoff del lane release (owner). Merge/publish = owner.

**Cláusulas a verificar (matriz de cierre — parte local):**

| # | Cláusula | Superficie | Evidencia |
|---|----------|-----------|-----------|
| C1 | §0.8.0 con campos v2 + ejes valid/transaction + `AS OF` | `docs/user/operations/UPGRADE.md` | sección publicada + gates docs |
| C2 | Pasos de migración/backfill + backup pre-upgrade (deterministas) | UPGRADE.md vs ADR-0046 §Migration / `src/cli_handlers/migrate.rs` | comandos `vanta-cli migrate plan/check/run` + orden expand→backfill→bump |
| C3 | Auditoría de commits del tramo: bump que propondría release-plz + breaking marcados | commits `7af34366`,`932b1211`,`83d65518`,`b90c494b`,`aa111979` + cierres | §Auditoría (probes ejecutados; `release-plz update` → **0.8.0**) |
| C4 | Remedio exacto del bump (commit marcador `feat!:` + `BREAKING CHANGE:`; NO reescribir historia) | mensaje redactado completo | §Auditoría §R1 |
| C5 | Release notes (contenido para el GitHub release) revisadas | draft + checklist de review del Release PR | §Release notes |
| C6 | Corte SOLO vía release-plz; CHANGELOG nunca a mano | `release-plz.toml` + `.github/workflows/release.yml` | §Handoff; `docs/CHANGELOG.md` intacto en el working tree |
| C7 | Handoff post-push del lane release (Release PR → merge → artefactos) | pasos numerados | §Handoff |

## Re-baseline (verificado 2026-09-29, HEAD `2fc033f8`; worktree sucio de otras sesiones intacto)

- **Corte = 0.8.0** (re-baseline ya ratificado en SCH-01/ADR-0046; `docs/CHANGELOG.md:15` `[0.7.0] - 2026-09-25` shipped; el lugar del placeholder `UPGRADE.md:67-71` lo confirma).
- **Superficies del corte ya en `develop`:** SCH-02..07 ✅ (commits `7af34366` schema v2, `932b1211` AS OF + confidence, `83d65518` cuarentena/abstención, `b90c494b` suite determinismo/chaos, `aa111979` 8 superficies + docs/api).
- **Semver acumulado (vs v0.7.0):** 9 familias de lints en `.local-semver-sch07.txt` (196 checks: 187 pass / 9 fail / 57 skip) — todas intencionales del corte (`feat!`/`feat` sin marcar, ver §Auditoría).
- **`docs/CHANGELOG.md`:** `[Unreleased]` vacío; generado por release-plz (nunca a mano). Frontmatter (`kind: changelog`) fue añadido por `d23e1224` (consolidación docs, 2026-09-28) DESPUÉS del último release → riesgo detectado en §Auditoría F2.
- **Working tree al abrir:** WIP ajeno presente (`perf-bench.yml`, `CONSTRAINTS.md`, `desktop/README.md`, `opencode.jsonc(+bak)`, `.local-semver-sch07.txt`) + **dos archivos del cierre SCH-07 sin commitear** (`docs/user/operations/CONFIGURATION.md` +2 filas de config, `vantadb-server/tests/e2e.rs` +112) → INTOCADOS por esta task; el LEAD debe incluirlos en el commit de cierre (son la cláusula C6-adjunta de SCH-07).
- **`v0.7.0` NO es ancestro de `develop`** (`merge-base --is-ancestor` exit 1; tag en `58a41ad8` sobre main). release-plz calcula el rango de commits `v0.7.0..HEAD` por tag → sin impacto para el bump (212 commits en rango), pero relevante para entender el PR vigente (§Handoff).

## Blast Radius

| Dirección | Archivos |
|-----------|----------|
| **Edita** | `docs/user/operations/UPGRADE.md` (§0.8.0 nueva + placeholder retirado) · `docs/api/DEPRECATIONS.md` (registro `VantaHeader`→`Header`, since 0.8.0) · `docs/dev/tasks/SCH-08.md` (este archivo) · `.local-release-plz-sch08.md` (evidencia local, no versionada) |
| **NO toca (explic.)** | `docs/CHANGELOG.md` (release-plz) · `release-plz.toml` (remedio R2 **propuesto**, no aplicado) · `.github/workflows/release.yml` · `docs/api/COMPATIBILITY.md` (cierre post-release → handoff) · plan file (LEAD) · Backlog (prohibido) · WIP ajeno |
| **Referencias hacia dentro** | `UPGRADE.md` ← `COMPATIBILITY.md:75` ("consumer-facing write-up ships with 0.8.0 in UPGRADE.md"), ADR-0046 §D4c, `docs/api/VERSIONING.md` |
| **Referencias entrantes** | Release PR (release-plz) consume los breaking markers; ICP-01/02 (F5) y VER-* consumirán la guía |
| **Implicaciones** | Docs-only + 1 fila de registry: ningún gate de código afectado; gates docs re-ejecutados. El corte real (merge/publish) queda en lane owner. |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `docs/user/operations/UPGRADE.md` (144L) · `docs/CHANGELOG.md` (head) · `release-plz.toml` (42L) · `docs/api/COMPATIBILITY.md` (111L) · `docs/api/VERSIONING.md` (118L) · `docs/api/DEPRECATIONS.md` (43L) · `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md` (602L) · `docs/dev/tasks/SCH-02..07.md` (§Contrato/§Verificación) · `docs/dev/workflow/RULES.md` §8/§9 · `.opencode/rules/release-ci.md` · `.local-semver-sch07.txt` (252L).
- **Leídos rangos clave:** `src/cli_handlers/migrate.rs` (:1-220 — comandos/orden/dry-run/force/JSON) · `src/schema.rs` (:1-60 — `CURRENT_SCHEMA_VERSION=2`, `MIN_COMPAT_VERSION=1`, comentario de normalización) · `src/cli.rs` (:406-431 `MigrateCommand`) · `src/binary_header.rs:151-152` (`#[deprecated(since="0.8.0")]`) · `src/lib.rs:173-174` (re-export del alias) · `docs/api/IQL.md` (AS OF/versión) · `docs/api/EMBEDDED_SDK.md` (campos v2, import) · `scripts/docs/check-docs.mjs` (frontmatter gating).
- **Referencias hacia dentro:** ADR-0046 §D1d alcance + §D7 + §D4c (deltas observables) + §Plan de migración (comandos); SCH-02..07 §Verificación (qué cambió por consumidor); `COMPATIBILITY.md:53-76` (7 familias del API wave); `.local-semver-sch07.txt` (9 familias finales).
- **Veredicto impacto:** bajo (docs) — pero la **auditoría sí toca el corte**: sin el commit marcador R1, el changelog/release notes no marcan los breaking del tramo (contrato VERSIONING "scan mecánico"). Registrado con remedio exacto.

## Spec (docs/release-readiness — decisiones por evidencia)

| # | Decisión | Elegido | Evidencia |
|---|----------|---------|-----------|
| 1 | Ubicación §0.8.0 | nueva `### Upgrading to 0.8.0` **antes** de 0.7.0 (historial newest-first), reemplaza el placeholder | patrón del archivo (`:44-66`) |
| 2 | Qué cubre la guía | campos v2 + ejes valid/transaction + `AS OF` + migración/backfill + backup + tabla de breaking (schema + API wave) | contrato L875 + `COMPATIBILITY.md:72-75` (el write-up 0.8.0 ES esta sección) |
| 3 | Comandos de migración | `vanta-cli migrate plan/check/run --format records|all [--dry-run]` (orden expand→backfill→bump; idempotente) | `src/cli_handlers/migrate.rs:207-230` + ADR-0046 §Plan |
| 4 | Release notes | **no** crear fuente nueva: draft de revisión en este task file; la fuente mecánica es el body del GitHub release de release-plz (`{{ changelog }}`) | rabbit hole del plan: "nunca como fuente"; `release-plz.toml` sin `git_release_body` |
| 5 | `DEPRECATIONS.md` | registrar `VantaHeader`→`Header` (policy: el PR que marca registra; el corte cierra el consumidor) | plan L871 "(si aplica)" + `src/binary_header.rs:151` + `VERSIONING.md` §Deprecation policy |
| 6 | `release-plz.toml` (frontmatter) | **remedio propuesto R2** (no aplicado; lane release) | probe §Auditoría F2 |
| 7 | Commit marcador | **remedio propuesto R1** (LEAD commitea; historia intacta) | pre-mortem F1 del plan + §Auditoría |

## Invariantes de dominio (handoff — MUST)
- **`docs/CHANGELOG.md` jamás se edita a mano** (solo release-plz); sin tags/versiones manuales (Regla 7).
- El commit marcador R1 **no reescribe historia**: es un commit NUEVO con sintaxis conventional válida (`feat(schema)!:` + footer `BREAKING CHANGE:`).
- La guía no promete exactitud numérica de artefactos hasta post-release (los pasos de verificación quedan en el handoff).
- Nothing aquí cambia código: si un hallazgo toca `src/**`, se registra como FIND/pendiente (no se arregla).

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: 0.** Docs-only (+1 registro de deprecación). Hallazgos nuevos → §Pendientes (FIND candidates: cli.rs help string desactualizado; `llm.rs` deprecations sin `since`; `COMPATIBILITY.md` cierre post-release).

## Definition of Done (3 niveles)
- **Task:** C1-C7 con evidencia (§Verificación + §Auditoría + §Release notes + §Handoff); gates docs verdes.
- **Commit:** `feat(schema)!: … (SCH-08)` — **mensaje marcador R1 completo en §Auditoría**; lo ejecuta el LEAD.
- **Release:** N/A local (release-plz, lane owner — §Handoff).

## Herramientas necesarias
- `release-plz` CLI local (v0.3.160) — probes en clones scratch, sin push ni token (modo `update` + `--registry-manifest-path`); `git log`/`merge-base` para la auditoría.
- Gates docs: `node scripts/docs/gen-index.mjs --check` · `check-links.mjs` · `check-docs.mjs` · `pwsh scripts/validate-docs-coverage.ps1`.
- **SDP (v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · **pin `documentation-skill`** (obligatoria para `docs/**`) · `documentation-and-adrs` · `writing-guidelines` · `deprecation-and-migration` (pin storage/schema). Sin candidatos extra tras discovery.

## Steps

| # | Step | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | DISCOVERY (plan L865-890 entero, ADR-0046, task files SCH-02..07, UPGRADE/CHANGELOG/release-plz.toml, semver file) | ✅ | §Re-baseline + §Impacto |
| 2 | Auditoría release-plz: patrón de commits + probes empíricos + remedios R1/R2 | ✅ | §Auditoría (probes A-D) |
| 3 | Escribir `UPGRADE.md` §0.8.0 (contrato) + retirar placeholder | ✅ | `docs/user/operations/UPGRADE.md` |
| 4 | Registrar deprecación `VantaHeader` (si aplica) | ✅ | `docs/api/DEPRECATIONS.md` |
| 5 | Release notes draft + checklist de review del Release PR | ✅ | §Release notes |
| 6 | Handoff lane release (post-push) | ✅ | §Handoff |
| 7 | Gates docs + verificación | ✅ | §Verificación |
| 8 | Cierre (LEAD): commit con mensaje R1 + review P2-01 | ⬜ | pendiente LEAD |

## Auditoría release-plz (evidencia — 2026-09-29)

**Pregunta (pre-mortem F1 del plan):** "ningún commit del tramo usa `feat!` → release-plz propone PATCH". ¿Qué bump propone release-plz para el corte?

**Patrón de commits (verificado mecánicamente):**

| Métrica (rango `v0.7.0..HEAD`, 212 commits) | Valor |
|---|---|
| Commits con breaking **válido** (`type!:` / `type(scope)!:`) | **1** — `aae39059 feat!: TS API a objetos` |
| Commits con forma **malformada** `feat!(scope):` (bang antes del scope) | **8** — API-02..07 + 2 de API-01 (`f6c395ef`,`a6f6a70b`,`f0c3f95f`,`bbcd9360`,`94009297`,`caf063ff`,`23ef7f63`,`f86584f6`) |
| Footers `BREAKING CHANGE:` exactos | **0** |
| Tramo F3 (`41c88c2b..HEAD`: SCH-02..07 + cierres) con `feat!`/footer | **0** — todos `feat(scope):` planos |

**Probes ejecutados (clones scratch en `%TEMP%/opencode/vantaplz`, sin push, sin token; `docs/CHANGELOG.md` del repo real intacto):**

| # | Probe | Resultado |
|---|-------|-----------|
| A | `release-plz update --registry-manifest-path <checkout v0.7.0>` (repo completo, clone limpio) | **`vantadb: next version is 0.8.0` (MINOR)** ✅ — el bump sale del único commit breaking válido (`aae39059`); con `semver_check` real: "✓ API compatible changes" (0.x MINOR licencia los breaks) |
| B | Mini-repo controlado (git_only; 1 commit por rama desde `v0.1.0`) | `feat:` → **0.1.1 (PATCH)** · `fix:` → 0.1.1 · `feat!(api):` malformado → **0.1.1 (PATCH — NO se reconoce como breaking)** · `feat(api)!:` válido → **0.2.0 (MINOR)** |
| C | Changelog generado por release-plz (probe A) | Los 8 `feat!(scope):` aparecen como **líneas crudas** en `### Added` (sin scope ni `[**breaking**]`); solo `aae39059` recibe `[**breaking**]`. Los commits F3 (schema v2, AS OF, cuarentena…) aparecen como features normales — **sin marca de breaking** |
| D | `[changelog] header` (remedio R2) en el clone + re-run | Frontmatter `title/kind` **preservado**; entrada `## [0.8.0] - …` insertada tras `[Unreleased]` ✅ |

**Hallazgos:**

- **F1 — Bump correcto pero frágil.** release-plz propone **0.8.0** (correcto), pero el MINOR depende de **un solo commit** bien marcado (`aae39059`, del API wave). Los 8 `feat!(scope):` malformados **no cuentan como breaking** (probe B) y el tramo F3 **no tiene ningún marcador** (0 `feat!`, 0 footer). Si ese único commit se pierde en el tránsito a main (squash/rebase selectivo), release-plz propondría **PATCH 0.7.1** con breaking dentro → violación del contrato VERSIONING (PATCH = compatible). Precedente real: el PR release-plz (hoy cerrado, #225) propuso `chore(vantadb): release v0.7.1` (`ea36c74b`, 2026-09-25) cuando no había marcador en rango.
- **F2 — release-plz strippea el frontmatter de `docs/CHANGELOG.md`.** El header por defecto reemplaza todo lo anterior a `## [Unreleased]` — incluido el frontmatter añadido por `d23e1224` (post-0.7.0). `check-docs.mjs` **gatea** `missingFrontmatter` (exit 1) ⇒ el Release PR tal cual rompe el gate docs al mergear. Verificado en probe A (removido) y remediado/verificado en probe D.
- **F3 — Sintaxis convencional malformada recurrente.** `feat!(scope):` no es conventional commit válido (el `!` va tras el paréntesis: `feat(scope)!:`). No se reescribe historia: se documenta para futuros commits + R1 cubre el rango.

**Remedios exactos (lane LEAD/owner):**

**R1 — Commit marcador (redactado completo; commit NUEVO, historia intacta, `git commit` normal del cierre SCH-08 — los archivos de este cierre viven bajo la raíz del paquete `vantadb`, así que cuentan para su changelog):**

```text
feat(schema)!: 0.8.0 — schema v2 (bitemporal + confidence + quarantine) + migration guide (SCH-08)

BREAKING CHANGE: 0.8.0 ships the single schema-v2 cut (ADR-0046). Consumer impact:

- On-disk schema v2: records gain valid_at_ms/invalid_at_ms, confidence fields
  (class/score/last_validated/derived_from) and quarantine fields; JSONL exports
  switch to schema_version 2 (v1 exports remain importable: schema_version 1 or 2
  accepted, >2 rejected). The storage header bumps to 2 when
  `vanta-cli migrate run --format all|schema` completes; 0.7.x binaries then
  refuse the directory (TooNew). Migration is deterministic + idempotent:
  backup -> `vanta-cli migrate run --format records` -> `--format all`.
- Rust SDK: MemoryRecord/MemoryInput/MemoryListOptions/MemorySearchRequest/Query
  gain public fields (struct literals must be updated); Embedded::import_records/
  import_file gain a `quarantine: bool` parameter; VantaHeader renamed to Header
  (deprecated alias kept); QueryResult::Write.node_id serializes u128 as a
  decimal string.
- IQL_VERSION 1 -> 2 (adds AS OF; v1 statements keep parsing). Search/list add
  opt-in as_of_ms/valid_window/min_confidence/include_quarantined; defaults are
  unchanged.
- Cargo: feature "server" no longer enables "cli" (use features = ["server", "cli"]).
- CLI / HTTP / MCP / TS bindings: API standardization wave (global --json,
  canonical routes/tool names, object-shaped TS API, u128-as-string).
- Data semantics: backfilled records get confidence = 1.0 (was an implicit
  node-level default of 0.5), moving eviction weights and LLM prompt scores
  (ADR-046 D4c).

Docs: docs/user/operations/UPGRADE.md § Upgrading to 0.8.0;
docs/api/COMPATIBILITY.md § Pre-release deltas; docs/api/DEPRECATIONS.md.
```

> Nota: cualquier mensaje del cierre sirve siempre que use `feat(<scope>)!:` válido + footer `BREAKING CHANGE:` (el subject aparece en el changelog; el footer alimenta el resumen de breaking changes del PR body de release-plz). PROHIBIDO `feat!(<scope>):`.

**R2 — `release-plz.toml` (appendar al final del archivo; preserva el frontmatter en las regeneraciones; verificado en probe D):**

```toml
# Changelog header — must stay byte-compatible with the docs frontmatter
# (scripts/docs/check-docs.mjs gates on title+kind): release-plz rewrites
# everything above `## [Unreleased]`, so the frontmatter must live here
# (otherwise the next release strips it from docs/CHANGELOG.md).
[changelog]
header = """---
title: Changelog
kind: changelog
description: All notable changes to this project will be documented in this file
---

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

"""
```

> Contexto: el frontmatter actual del CHANGELOG fue añadido por `d23e1224` (2026-09-28) después del último release-plz run; sin R2, el próximo run lo elimina y `check-docs` falla. `[changelog] header` debe ir AL FINAL del TOML (una tabla captura las claves siguientes; `changelog_update` quedó fuera del bloque en el probe fallido).

## Release notes — draft de revisión (GitHub release 0.8.0)

> **Fuente mecánica:** el body del GitHub release lo genera release-plz (`{{ changelog }}` = entrada `[0.8.0]`). Este draft NO es fuente: es la revisión curada para (a) el checklist del Release PR y (b) editar el body en GitHub si el owner lo desea (post-merge). El CHANGELOG no se toca.

**Título:** `v0.8.0` (release-plz: `vantadb-v0.8.0` por default de workspace multi-paquete; el tag es `v0.8.0`).

**Highlights (sugerido para el body revisado):**
- **Schema v2 — bitemporal records**: valid-time windows (`valid_at_ms`/`invalid_at_ms`) separate from transaction time; deterministic, idempotent v1→v2 migration via `vanta-cli migrate`.
- **Point-in-time queries**: IQL v2 `AS OF` + `as_of_ms`/`valid_window` params (opt-in; defaults unchanged).
- **Confidence per record**: asserted/derived with `derived_from`, `min(parents)×0.9`, `min_confidence` filter.
- **Quarantine**: default-excluded from retrieval, sticky, never auto-promotes, 30-day review signal.
- **Selective abstention** (opt-in) + trust-aware retrieval.
- **API standardization wave**: canonical CLI/HTTP/MCP/TS surfaces + all 8+ surfaces carry the v2 wire.
- **Docs**: UPGRADE.md §0.8.0 (migration), IQL/HTTP/MCP/SDK references updated in the same cut.

**Breaking changes (resumen; lista completa en el changelog):** schema v2 on-disk + export v2 (v1 imports siguen válidos); header v2 (0.7.x rechaza dirs migrados); campos nuevos en structs del SDK + arity de `import_records/import_file`; `VantaHeader`→`Header` (alias deprecado); `server` ya no implica `cli`; CLI/HTTP/MCP/TS normalization; `confidence=1.0` en backfill (eviction/prompts).

**Upgrade:** `docs/user/operations/UPGRADE.md` § Upgrading to 0.8.0 (backup → migrate → verify → rollback). Publicado en este mismo changeset.

**Checklist de review del Release PR (post-push; bloquea el merge):**
- [ ] Versión: `0.7.0 → 0.8.0` (MINOR; NUNCA 0.7.1 — si sale PATCH, falta R1: corregir y regenerar).
- [ ] Entrada `[0.8.0]` presente con al menos una línea `[**breaking**]` (R1 aplicado).
- [ ] Frontmatter `title/kind` intacto en `docs/CHANGELOG.md` (R2 aplicado; si falta → NO mergear sin R2).
- [ ] Release link `[0.8.0](…/compare/v0.7.0...v0.8.0)` presente.
- [ ] Evidence de semver-check: "✓ API compatible changes" (0.x MINOR licencia los 9 breaks aceptados; en 0.x el job es informativo, no un fail).
- [ ] Sin ediciones manuales del CHANGELOG (Regla 7); sin tags manuales.

## Review P2-01 (cerrado 2026-09-29)

> **Revisor:** `ses_f123b4fdeffew7UivC2bXx860O` (fresco ≠ autor `ses_f126411a5ffeC5y8xCoikg1gmT`) — ronda 1 **❌ CHANGES REQUIRED** (1 Required: cita del PR release-plz; +2 Optional +3 Nits) → fixes aplicados (exactos) + R2 → **delta ✅** (commit `b9296909` con mensaje R1 verbatim; check-docs verde; sin reescritura).

## Handoff — lane release (owner/LEAD; post-push)

1. **LEAD (local):** commit de cierre con mensaje **R1** (+ archivos del cierre SCH-07 que quedaron sin commitear: `CONFIGURATION.md`, `vantadb-server/tests/e2e.rs`); aplicar **R2** si el owner lo aprueba; review P2-01 (tier adversarial por `docs/api/**`).
2. **Owner:** `push develop` (OK explícito) → PR `develop → main` (solo desde develop + OK) → merge.
3. **release-plz en main** (`.github/workflows/release.yml`; `secrets.RELEASE_PLZ_TOKEN || secrets.GITHUB_TOKEN`, OIDC para publish): se dispara `release-plz-pr` → **regenera el Release PR**. ⚠️ El PR release-plz vigente es **#228** (`origin/release-plz-2026-09-25T19-00-22Z`, `c1841523`); el anterior **#225** (`origin/release-plz-2026-09-25T15-38-39Z`, propuesta **v0.7.1** `ea36c74b`) quedó **cerrado**. Ambos son **anteriores al corte F3**: NO mergearlos; el run nuevo debe supersederlos (si no se actualiza, cerrarlo y re-disparar). `release_always = false` ⇒ el release ocurre SOLO al mergear el Release PR.
4. **Verificación del Release PR:** checklist §Release notes (versión/breaking/frontmatter/link). Si no refleja 0.8.0 → **NO mergear**; corregir commits (R1) y regenerar (nunca CHANGELOG a mano).
5. **Merge del Release PR (owner):** release-plz tagea `v0.8.0`, crea el GitHub release y publica `vantadb` en crates.io (Trusted Publishing OIDC). Downstream `release: published`: **release-wheels** (PyPI `vantadb-py` + TestPyPI), **release-adapters**, **release-npm-61/62** (`vantadb`/`vantadb-wasm`), **release-npm-node**, **release-binaries** (`vanta-cli`, `vantadb-server`).
6. **Verificación post-release de artefactos:** `pip index versions vantadb-py` · `npm view vantadb version` (+ `vantadb-node`, `vantadb-ts`, `vantadb-wasm`) · crates.io `vantadb` 0.8.0 · binarios en el release de GitHub; editar el body del GitHub release con el draft §Release notes si se desea (el body mecánico ya está).
7. **Cierre docs post-release (vanta-docs, task nueva o follow-up):** `COMPATIBILITY.md` §Pre-release deltas → cerrada ("shipped in 0.8.0"; criterio 1.0 exige la sección vacía); `UPGRADE.md` §0.8.0 `Released: pending` → fecha real; `DEPRECATIONS.md` sigue activa (removal target 0.9.0).

## Verificación (evidencia 2026-09-29)

| Comando | Resultado |
|---|---|
| `node scripts/docs/check-docs.mjs` | ✅ exit 0 — `GATING: all clear` (1655 docs; orphans=4/1426 report-only; `SCH-08.md` linkeado por el índice regenerado) |
| `node scripts/docs/check-links.mjs` | ✅ exit 0 — 53/58 links rotos **dentro de presupuesto** (0 nuevos; anchors nuevos resueltos) |
| `node scripts/docs/gen-index.mjs --check` (inicial) | ⚠️ stale (`docs/index.md`, `llms.txt`) — incluye entradas SCH-06/07 pendientes de la wave |
| `node scripts/docs/gen-index.mjs --write` → `--check` | ✅ ejecutado (diffs generados: `docs/index.md` + `llms.txt`, parte del changeset) → check final **exit 0** |
| `pwsh -NoProfile scripts/validate-docs-coverage.ps1` | ✅ exit 0 — **0 gaps** (31+69+35+44+51+47 ítems ok; incluye las 2 filas de `CONFIGURATION.md` sin commitear de SCH-07) |
| `release-plz update` (probe A, clone scratch) | ✅ `next version is 0.8.0`; diff propuesto: `Cargo.toml` 0.7.0→0.8.0 + `Cargo.lock` + `docs/CHANGELOG.md` (~+229/−6) |
| Probe D (`[changelog] header`) | ✅ frontmatter preservado (remedio R2 verificado) |
| Auditoría commits | ✅ 212 commits en rango; 1 breaking válido / 8 malformados / 0 footers; tramo F3: 0 |
| `git status` (repo real) | ✅ `docs/CHANGELOG.md` intacto; solo los archivos de la task (+2 generados del índice, +1 evidencia local); WIP ajeno intacto |
| Artefactos scratch | `%TEMP%\opencode\vantaplz\{update.log,changelog-proposed.diff,update-headerfix.log}` + `.local-release-plz-sch08.md` (local, no versionado) |

## Review (GATE P2-01)
- ⬜ **PENDIENTE — LEAD.** Tier: **adversarial** (el diff toca `docs/api/**` según la tabla risk-tiered; reviewer fresco ≠ autor; sin self-review por mandato del orquestador). Insumos: §Auditoría (probes A-D), §Verificación, §Release notes.
- Nota: la task NO ejecuta `campaign_verify_cmd` (rol docs, sin cambios de código); gates docs completos sí.

## Pendientes (§Pendientes)
- **LEAD:** commit con R1 + review P2-01 + (decisión owner) R2; incluir los 2 archivos de SCH-07 sin commitear.
- **Owner lane:** todo el §Handoff (push/PR/Release PR/merge/artefactos).
- **FIND candidates (no registrados: Backlog prohibido en esta task):**
  - `src/cli.rs:416` — help string de `migrate run --format` lista "(vfile, index, wal, schema, all)" sin `records` (el error real de `migrate.rs:185` sí lo lista) → 1 línea, visible en `--help`.
  - `src/llm.rs:713,798,909` — constructores `#[deprecated]` sin `since` ni registro en `DEPRECATIONS.md` (fuera del corte; revisar si son superficie pública).
  - Sintaxis convencional `feat!(scope):` (8 commits del API wave) — no reescribir; considerar lint/nota en CONTRIBUTING.
- Post-release docs (§Handoff paso 7).

## Context Save Point
- Task ejecutada de punta a punta (steps 1-7 ✅). Entregables: guía §0.8.0 publicada, auditoría con remedios R1/R2 redactados y verificados empíricamente, release notes draft + checklist, handoff. Sin commit (LEAD). Working tree ajeno intacto.

## Recitation
```
=== RECITATION ===
Objetivo activo: SCH-08 — Corte 0.8.0: migration guide + CHANGELOG + release notes
Estado: in-progress (steps 1-7 ✅; commit + review P2-01 = LEAD)
Última acción: guía §0.8.0 escrita (UPGRADE.md) + deprecación VantaHeader registrada + auditoría release-plz (probes: bump=0.8.0; 8/9 breaking malformados; frontmatter strip detectado + fix verificado) + release notes draft + handoff owner
Resultado: PARTIAL
Próxima acción: LEAD — commit con mensaje R1 (§Auditoría) + review P2-01 adversarial + aplicar R2 si owner aprueba; owner lane: push → PR develop→main → Release PR (checklist) → merge → artefactos
Contrato: C1 ✅ §0.8.0 · C2 ✅ migración/backfill/backup vs ADR-0046 · C3 ✅ auditoría (212 commits; bump 0.8.0) · C4 ✅ R1 redactado · C5 ✅ notes draft + checklist · C6 ✅ CHANGELOG intacto (release-plz únicamente) · C7 ✅ handoff post-push
Invariantes: CHANGELOG nunca a mano; sin reescribir historia; sin push/merge (owner); WIP ajeno intacto; sin tags/versiones manuales
Deuda: ninguna nueva (saldo Regla 6 = 0); FIND candidates en §Pendientes (cli.rs help; llm deprecations sin since)
Próxima tarea si completa: cierre de F3 (gate) → F4 (VER-07/01/05/06…)
last-synced: 2026-09-29
=== END RECITATION ===
```
