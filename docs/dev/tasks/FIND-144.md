# FIND-144 — Durable workflow rules + CI_POLICY refresh + AGENTS pointer

> **Plan:** `docs/dev/plans/2026-09-21-workflows-repair.md` (Wave 5, tras FIND-143)
> **Type:** docs (campaign_detect_task_type: Documentation)
> **SDP:** documentation-and-adrs (curada; keywordMapped) + writing-guidelines (base) — resto lifecycle (frontend-ui, api-design, TDD, doubt, source, context) N/A docs-only. Keywords: [docs, rules, markdownlint, policy]. Phase BUILD.
> **Estado:** ⬜ PENDING → ⏳ IN PROGRESS (S1)
> **Gate D:** NO dispara — blast radius 3 archivos docs-only, sin hot path/API pública/símbolos nuevos, contrato cerrado, feature-add N/A (reglas, no código). Motivo ≤6 palabras: docs-only, alcance cerrado.
> **Gate P:** owner aprobó plan 2026-09-21 + reglas a `docs/dev/workflow/RULES.md` (decisión 3 vinculante).

## Objetivo

Crear `docs/dev/workflow/RULES.md` (nuevo) + refresh `docs/dev/operations/CI_POLICY.md` (conteo/triggers) + 1 fila puntero en `AGENTS.md` raíz. Contrato: lint verdes; una regla verificable por cada hallazgo Alta; 0 links rotos.

## Contrato (ley)

- `npx markdownlint-cli2 "docs/dev/workflow/**/*.md" "docs/dev/operations/CI_POLICY.md"` → 0 issues.
- Una regla verificable por cada hallazgo Alta (triggers, timeouts, pins, permissions, publish, anti-patrones).
- 0 links rotos (grep `](docs/workflow` + `](docs/operations` resuelven; sin URLs externas nuevas).
- `git diff --check` limpio. Commit `docs: FIND-144 — ...` selectivo. NO PUSH.
- PROHIBIDOS intactos: `.github/workflows/` (cero edits YAML), `src/`, `web/src/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, stash GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation vía MCP), `docs/CHANGELOG.md`, `docs/api/openapi.yaml`, `docs/api/MCP.md`, `C:/Users/Eros/.vantadb*`, secretos.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `docs/dev/operations/CI_POLICY.md` (479L entero) — §1 Fast Gate `ci-rust.yml`, §§3-10 triggers, línea 17 conteo `26` stale (2026-09-15).
- `AGENTS.md` raíz (16L entero) — tabla Entry points existe (4 filas), cabecera prohíbe duplicar.
- `.opencode/rules/README.md` (111L entero) — formato R2 (Scope/Status/Fuentes + Must/Must-not/Por qué).
- `docs/dev/workflow/README.md` (90L), `TRIGGERS.md` (71L), `PUBLISH.md` (88L), `FAQ.md` (59L), `RUNBOOK.md` (frontmatter) — convención frontmatter + matriz triggers post-FIND-134/139/140/141/146 + FAQ §"Where do the durable rules live?" apunta a RULES.md (FIND-144).
- `SPEC.md` raíz (108L), `.opencode/references/definition-of-done.md` (144L) — DoD standing + docs timeless.
- `.github/workflows/` disco: 27 files (ls 2026-09-22); `git ls-files` 27. Commit FIND-142 `97a3a03c` (42 files, 14 renames R).
- `.markdownlint-cli2.yaml` — MD013/MD041/etc off; `docs/dev/archive/**` ignorado.

**Referencias hacia dentro (qué cita el cambio):**
- RULES.md cita filenames FINALES del disco (27): `ci-rust.yml`, `ci-rustdoc.yml`, `ci-web.yml`, `ci-examples.yml`, `ci-gate.yml`, `gate-docs.yml`, `sec-codeql.yml`, `providers-ci.yml`, `adapters-compat.yml`, `chaos.yml`, `fuzz.yml`, `perf-bench.yml`, `heavy-certification.yml`, `heavy-bench-nightly.yml`, `bench-canonical-p99-informational.yml`, `arch-metrics-informational.yml`, `ocr-delegate.yml`, `ocr-nightly.yml`, `opencode.yml`, `desktop.yml`, `release.yml`, `release-wheels.yml`, `release-npm-61.yml`, `release-npm-node.yml`, `release-adapters.yml`, `release-binaries.yml`, `release-sbom.yml`.
- EXCEPCIÓN: `release.yml`, `release-npm-61.yml`, `release-npm-node.yml` NO renombrados (trusted publishers externos exigen filename; commit 97a3a03c).
- CI_POLICY refresh cita `docs/dev/workflow/TRIGGERS.md` como matriz canónica + `docs/dev/workflow/README.md` inventario 27.
- AGENTS.md fila cita `docs/dev/workflow/RULES.md` (puntero, sin duplicar contenido).

**Referencias entrantes (quién cita el cambio):**
- `docs/dev/workflow/FAQ.md:54-59` ya apunta a `docs/dev/workflow/RULES.md` (FIND-144) — RULES.md lo satisface, no hay que editar FAQ.
- `docs/dev/workflow/README.md:related` no lista RULES.md (4 entries) — fuera de alcance (tocar solo lo pedido; no reescribir related en masa).
- `docs/dev/workflow/*.md` detalle (20 files con nombres viejos `ci-rust-10.md` etc. sin renombrar) — fuera de alcance, no citarlos desde RULES.md.

**Veredicto de impacto:** BAJO, docs-only, reversible (`git revert` 1 commit). Sin código/CI/locks. Blas radius: 1 nuevo + 2 edits. Sin Gate V previsto.

## Discrepancia conteo (dado vs disco)

- Tarea dice refresh 26→28. Disco dice **27** (`ls` + `git ls-files` 27). `docs/dev/workflow/README.md:15-16` explica: eran 28 en FIND-128, FIND-137 fusionó `rustdoc-70.yml` → `ci-rustdoc.yml` (commit 4b0686b0) → 27. CI_POLICY línea 17 dice 26 (stale 2026-09-15). **Decisión: usar 27 (verdad del disco)**, anotar 28 como auditoría vieja pre-FIND-137. FAQ:59 ya anticipa `26 → 27`.

## Steps (~100 líneas c/u, reversibles)

- [x] **S1 DISCOVERY** — task file este + `campaign_update_task_state in-progress` + validación `campaign_validate_command`. ✅ este file.
- [x] **S2 RULES.md** — crear `docs/dev/workflow/RULES.md` (frontmatter workflow-index + Scope/Status/Fuentes + 7 reglas Must/Must-not/Por qué con bueno/malo + excepción publishers + verificación mecánica por regla). ~150L inglés. Grep 0 old-names. ✅ creado.
- [x] **S3 CI_POLICY** — refresh mínimo: L17 `26`→`27` + fecha + nota rustdoc-merge/renames; §§3-8 triggers reales (web push main+develop, gate-docs main+develop, fuzz +PR paths, perf-bench +develop, bench-nightly +PR, release tabla +wheels/npm-node rows, npm-61 sin "push main paths"). Nada más. ✅ 7 edits.
- [x] **S4 AGENTS.md** — 1 fila puntero en tabla Entry points → `docs/dev/workflow/RULES.md`. Verificar tabla existe antes (sí, 4 filas → 5). ✅.
- [x] **S5 VERIFY** — `npx markdownlint-cli2 "docs/dev/workflow/**/*.md" "docs/dev/operations/CI_POLICY.md"` + `git diff --check` + grep old-names + grep links. ✅ lint 0/22, diff-check limpio, old-names 0, links resuelven. Retry ladder; 2 fallas mismo-error → Gate V.
- [ ] **S6 CIERRE** — `git add` selectivo (3 paths + task file) + commit `docs: FIND-144 — ...` sin push + `campaign_update_task_state completed` + RESULTADO §7.

## Context Save Point

- S1 hecho: discovery + task file + disco 27 + CI_POLICY L17 + AGENTS tabla OK + SDP + Gate D no.
- Próximo: S2 crear RULES.md (ver detalle tarea §1 para contenido por tema).
- Reanudar: leer este file + `git status --short` + `ls .github/workflows/` y seguir desde primer ⬜.

## Gates evaluados

- P: no (owner aprobó plan 2026-09-21, decisión 3).
- D: no (docs-only, alcance cerrado).
- V: pendiente (si 2 fallas mismo-error en S5).
- C: pendiente (cierre S6).
