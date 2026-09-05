# GOV-TK2: Release 0.6.0 para exponer ~80 tools MCP (gap skill_*/code_*/wiki_*)

## Metadata
- **Plan file:** docs/Backlog.md (sección GOV-TK)
- **Fuente:** Backlog GOV-TK line 456: "Release para que el binario MCP exponga las 18 tools skill_*/code_*/wiki_* (skill ya documenta 33; binario publicado tiene 15)"
- **Esfuerzo:** 🟡 1d (release workflow + verificación)
- **Prioridad:** 🔴 Alta (bloquea multi-sesión real; skill documenta 33, publicado 15, código ~80)
- **Tipo:** Release / CI
- **Turns estimados:** 15-30
- **Creado:** 2026-09-02T18:45:00
- **last-synced:** 2026-09-02T18:45:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 3 (D5 decisión owner GO/NO-GO; estado CHANGELOG; secretos CI activos)
- **Pendientes (downhill):** 7 steps de ejecución

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub Actions: `release-plz`, `release-binaries-63.yml`, `release-wheels-60.yml`, `release-npm-61.yml`, `release-npm-node.yml` |
| Callees | `release-plz.toml`, `cliff.toml`, `Cargo.toml` [workspace.package], `docs/CHANGELOG.md` |
| Implicaciones | 1) Bump semver 0.5.0 → 0.6.0 (minor: 284 feat commits). 2) Publica `vantadb` 0.6.0 a crates.io + binarios `vanta-cli`/`vantadb-server` a GitHub Releases (~80 tools). 3) Publica `vantadb-python`/`vantadb-wasm`/`vantadb-ts`/`vantadb-node` a PyPI/npm. 4) **D5 diferido** — requiere decisión owner explícita vía `/ship` antes de mergear Release PR. 5) FIND-MCP-001 (vantadb-mcp/tests/context_tests.rs:70) **no bloquea** Fast Gate (`vantadb-mcp` fuera de default-members) pero puede fallar `cargo check --workspace --tests` local. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `release-plz.toml`, `cliff.toml`, `docs/CHANGELOG.md` (vacío desde v0.5.0?), `.github/workflows/release-plz.yml`, `.github/workflows/release-binaries-63.yml`
- **Archivos referenciados hacia dentro:** `Cargo.toml` [workspace.package] version, `vantadb-mcp/src/lib.rs`, `vanta-cli/src/bin/vanta-cli.rs`
- **Archivos que referencian a los editados:** GitHub Actions que disparan en tag push (`push.tags: ['v*']`)
- **Veredicto impacto:** ALTO — release es acción pública irreversible; requiere pre-flight completo (Regla 7) + gate `/ship` + verificación `cargo semver-checks` + `cargo deny check` + `dev-tools/verify.ps1`

## Contrato
"Release 0.6.0 publicado: (1) `cargo publish` vantadb 0.6.0 exit 0, (2) GitHub Release v0.6.0 creado con binarios `vanta-cli` (3 targets) + `vantadb-server`, (3) `vanta-cli --version` = 0.6.0 expone ≥50 tools (memory_*, search_*, collection_*, graph_*, skill_*, code_*, wiki_*, scene_*, thread_*, embed_texts, snapshot_*, maintenance_*), (4) `cargo semver-checks --baseline-rev main` pasa, (5) `dev-tools/verify.ps1` verde previo a push."

## SDP — Skills cargadas
- **campaign-executor** (base) — orquestación
- **git-workflow-and-versioning** — conventional commits, semver, release-plz flow
- **ci-cd-and-automation** — quality gates, workflow patterns
- **shipping-and-launch** — pre-launch checklist, staged rollout, rollback plan
- **progreso** — migración tarea completada
- **ponytail (full)** — ya activo (modo lazy)

**SDP: sin candidatos adicionales** (release task usa skills base de CI/CD + git workflow).

## Investigation Notes
- Workspace version 0.5.0 = tag v0.5.0 (correcto, release-plz bumps).
- 284 feat commits desde v0.5.0 → release-plz bump minor → 0.6.0.
- D5 decisión: "Release triage semver 0.6.0 (D5 - diferido hasta decisión del owner)" — NO proceder sin `/ship` GO.
- `vantadb-mcp` fuera de `default-members` (Cargo.toml:677-678) → FIND-MCP-001 no bloquea Fast Gate.
- Código actual ~80 tools (tools.rs 50 directas + context delegadas 30 skill/code/wiki/scene/thread) vs publicado 15.
- Secretos CI requeridos: `CARGO_REGISTRY_TOKEN`, `NPM_TOKEN`, `TEST_PYPI_API_TOKEN` (GitHub Secrets).
- Uncommitted: docs/Backlog.md, .opencode submodule, completions, REDDIT_POSTS.md, planes nuevos — **limpiar antes de PR develop→main**.

## Invariantes de dominio
- **Invariantes a preservar:** 
  1. Release-plz gestiona versionado/CHANGELOG — NUNCA editar manualmente Cargo.toml version ni docs/CHANGELOG.md.
  2. Pre-push gate: `dev-tools/verify.ps1` completo (fmt + clippy + nextest + deny) verde ANTES de push develop→main.
  3. `/ship` gate: decisión GO/NO-GO con fan-out (audit/tuner/docs) ANTES de mergear Release PR.
  4. Rollback plan: 1 línea revert `default-members` si release rompe CI (P47 STABLE-09).
- **Comandos de verificación:** `dev-tools/verify.ps1`; `cargo semver-checks --baseline-rev main`; `cargo deny check`; `cargo nextest run --profile audit -p vantadb --build-jobs 2`; `git log --oneline -5` (verificar conventional commits).
- **Deuda pendiente:** ninguna (release no introduce deuda — solo publica estado actual).

## Deuda técnica (Regla 6)
Saldo neto: **Sin deuda** — release publica código existente, no modifica.

## Definition of Done
| Nivel | Gate |
|-------|------|
| **Task** | Release 0.6.0 live (crates.io + GitHub Releases) + binario expone tools skill_*/code_*/wiki_* |
| **Commit** | Release PR mergeado con conventional commit `chore: release v0.6.0` (auto release-plz) |
| **Release** | `/ship` GO + pre-launch checklist §2a completa + rollback plan documentado |

## Herramientas necesarias
- `git` (push, PR, merge, tag)
- `release-plz` (auto bump + changelog)
- `cargo` (semver-checks, deny, publish dry-run)
- `dev-tools/verify.ps1` (pre-flight)
- GitHub CLI `gh` (PR create/review/merge, secrets check)
- `codegraph_explore` (blast radius verificación)

## Steps

### Step 1: Pre-flight — limpiar working tree + verify completo
- **Archivos:** working tree
- **Acción:** `git stash push -u -m "pre-release stash"` (guardar trabajo en progreso: docs/Backlog.md, .opencode submodule, completions, REDDIT_POSTS, planes nuevos) → `dev-tools/verify.ps1` completo (6 pasos: fmt, clippy, nextest, deny, semver-checks dry-run, audit)
- **Verify:** `dev-tools/verify.ps1` exit 0 + `cargo semver-checks --baseline-rev main` exit 0
- **Estado:** ⬜ PENDING

### Step 2: Confirmar D5 GO — /ship gate
- **Archivos:** n/a
- **Acción:** Ejecutar `/ship` (fan-out audit/tuner/docs → GO/NO-GO). Registrar decisión en task file. Si NO-GO → STOP y documentar razón.
- **Verify:** `/ship` output "GO" + rollback plan escrito
- **Estado:** ⬜ PENDING

### Step 3: PR develop → main
- **Archivos:** git
- **Acción:** `git push origin develop` (ya limpio del step 1) → `gh pr create --base main --head develop --title "Release 0.6.0" --body "Auto-generated by release-plz. 284 feat commits since v0.5.0."`
- **Verify:** PR creado, CI Fast Gate verde (<5 min)
- **Estado:** ⬜ PENDING

### Step 4: Release-plz crea Release PR
- **Archivos:** `.github/workflows/release-plz.yml` (trigger on push to main)
- **Acción:** Merge PR develop→main → release-plz action detecta push a main → analiza conventional commits → bump 0.5.0→0.6.0 → actualiza Cargo.toml [workspace.package] + docs/CHANGELOG.md → crea Release PR "chore: release v0.6.0"
- **Verify:** Release PR visible en GitHub, `cargo publish --dry-run -p vantadb` exit 0 en CI
- **Estado:** ⬜ PENDING

### Step 5: Revisar y mergear Release PR
- **Archivos:** Release PR diff
- **Acción:** Revisar Release PR: version 0.6.0 correcta, CHANGELOG curado, sin cambios manuales en Cargo.toml version. Merge Release PR → release-plz taguea `v0.6.0` → dispara workflows RELEASE (binaries, wheels, npm).
- **Verify:** Tag `v0.6.0` creado; `release-binaries-63.yml` + `release-wheels-60.yml` + `release-npm-61.yml` + `release-npm-node.yml` + `release-adapters-62.yml` en verde
- **Estado:** ⬜ PENDING

### Step 6: Verificar publicación live
- **Archivos:** crates.io, GitHub Releases, PyPI, npm
- **Acción:** Esperar workflows RELEASE completados (5-10 min). Verificar: `cargo search vantadb --limit 1` → 0.6.0; `gh release view v0.6.0` → binarios adjuntos; `vanta-cli --version` (descargar binario) = 0.6.0; `pip index versions vantadb` → 0.6.0; `npm view vantadb-ts version` → 0.6.0.
- **Verify:** Todas las publicaciones confirmadas; `vanta-cli server --mcp --db test` expone tools skill_list, code_search, wiki_search, scene_read, thread_create
- **Estado:** ⬜ PENDING

### Step 7: Post-release — restaurar working tree + skill progreso
- **Archivos:** working tree (stash)
- **Acción:** `git stash pop` (restaurar docs/Backlog.md, .opencode, planes, etc.) → ejecutar `skill progreso` (Trigger 1: elimina GOV-TK2 de Backlog.md, migra a docs/avance/activo/ci-cd.md o operations.md)
- **Verify:** GOV-TK2 removido de Backlog.md; progreso registrado; `git status` limpio salvo trabajo del plan activo
- **Estado:** ⬜ PENDING

## Dependencias
- **Bloqueante externo:** Decisión D5 owner GO/NO-GO (Step 2)
- **Precondición técnica:** `dev-tools/verify.ps1` verde + `cargo semver-checks` verde (Step 1)
- **Post-condición:** `release-plz` workflows completados (Steps 4-5)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-audit` (leaf, no implementa) — seguridad supply chain + verificación gates
- **Enfoque:** ¿release-plz config correcto? ¿secrets CI vigentes? ¿semver-checks pasa? ¿binarios compilados con custom-allocator (Regla release-ci #1)?
- **Cómo se probó:** evidencia de verify.ps1 verde pre-merge + semver-checks + deny check + CI Fast Gate verde en PR + release workflows verde
- **Checklist anti-hábitos:** [ ] no saltar verify "porque es release" [ ] no editar version manual [ ] no mergear sin /ship GO
- **Veredicto:** ⬜ pendiente

## Notas
- GOV-TK2 premisa original (18 tools skill_/code_/wiki_) **ampliada**: código actual expone ~80 tools (50 directas + 30 delegadas skill/code/wiki/scene/thread/embed/snapshot/maintenance). Release 0.6.0 cierra gap completo.
- D5 diferido: "release triage semver 0.6.0 (D5 - diferido hasta decisión del owner)". El task file formaliza el release; /ship gate formaliza la decisión.
- Si /ship da NO-GO: tarea → 🟡 DEFER con justificación en Notas. No forzar.
- Rollback: si v0.6.0 rompe → `git tag -d v0.6.0 && git push origin :refs/tags/v0.6.0` + `cargo yank` (no recommended) + revertir develop/main state. Documentado en P47 STABLE-09.

## Context Save Point
- **Fecha:** 2026-09-02T18:45:00
- **Branch:** develop
- **CI pendiente:** sí (verify.ps1 + semver-checks pre-PR)
- **Decisiones:** D5 deferido → /ship gate obligatorio; release-plz auto-bump minor (284 feat)
- **Problemas conocidos:** FIND-MCP-001 (vantadb-mcp test roto, fuera de default-members); uncommitted work requiere stash
- **Próxima tarea si completa:** MCP-35 (plan activo Wave 1) o GOV-TK3 (drift yaml↔real)