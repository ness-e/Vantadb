# BND-08: pipeline npm napi-rs end-to-end en dry-run (SIN publicar)

## Metadata
- **Plan file:** docs/plans/2026-09-04-durability-release-readiness.md (Task 8, Wave 2)
- **Campaign ID:** a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
- **Creado:** 2026-09-05 (DISCOVERY pipeline-full, Task ID BND-08)
- **Estado:** ✅ COMPLETED (2026-09-05, commit `e9843100` — contrato 4/4)
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(node): prepublish verificado dry-run + checklist (BND-08)`
- **Scope APROBADO:** SOLO dry-run + checklist — PROHIBIDO publicar (decisión Gate P del usuario)
- **Reconciliación 2026-09-09 (plan `docs/plans/2026-09-08-backlog.md`, Task 4, Campaign `5b5a8ce1`):**
  scope nuevo (workflow + matrix 5 targets) YA EXISTE en git (`ed75cb0b`/`3af4c598`/`f4394b7c`:
  workflow 223L, matrix 7 targets ⊇ 5, artifacts + prepublish). Esta pasada: verify mecánico
  del contrato + reconcile, SIN rewrite del workflow (ponytail: reescribir 223L funcionales = riesgo
  sin beneficio). Invariantes heredados vigentes (nunca publish real, nunca stagear ajenos).

## Contrato
`npm pack` + prepublish artifacts OK + `npm publish --dry-run` verde + checklist de release escrita en `docs/plans/artifacts/bnd-08-publish-checklist.md` (pasos para el humano: OIDC/trusted-publisher config, tag, orden vs GOV-TK2). PROHIBIDO ejecutar `npm publish` real.

## SDP — Skills cargadas
- **campaign-executor** (base) — orquestación pipeline-full
- **progreso** (base) — sync backlog/avance al cierre
- **ponytail (full)** (base, ya activo) — ladder YAGNI, mínimo diff
- **source-driven-development** (lifecycle BUILD) — validar npm/napi-rs/OIDC contra docs oficiales si hay duda
- **incremental-implementation** (lifecycle BUILD) — slices delgados: pack → dry-run → checklist → commit
- **test-driven-development** (lifecycle BUILD) — N/A lógica nueva; verificación = comandos mecánicos del contrato
- **context-engineering** (lifecycle BUILD) — context pack por slice (rules → plan → source → error previo)
- **doubt-driven-development** (lifecycle BUILD) — stakes altos (publicación irreversible): doble-check de que ningún comando publica

**SDP: campaign-executor + progreso + ponytail + source-driven-development + incremental-implementation + test-driven-development + context-engineering + doubt-driven-development**

## Gates
- **Gate P:** no dispara (decisión ya tomada por el usuario: dry-run only, publish real NO).
- **Gate D (question-gates):** no dispara — blast radius 1 archivo nuevo (checklist, docs solo), 0 archivos productivos, 0 símbolos `pub` nuevos, contrato no ambiguo.
- **Gate V:** dispara solo si 2 fallas mismo-error en VERIFY → question al usuario.
- **Gate C:** al cierre, confirmar que `git status` no stagea ajenos (worktree con cambios de otras sesiones).

## Blast Radius
- Callers: `.github/workflows/release-npm-node.yml` job `publish` (lee `vantadb-node/package.json` version + `*.tgz`).
- Callees: `vantadb-node/package.json` (files: index.js/index.cjs/index.d.ts/*.node/README.md), `vantadb-node/index.js` (loader napi generado), `vantadb-node/index.d.ts` (contrato TS).
- Implicaciones: 0 cambios productivos. Solo LECTURA de workflow/package + 1 archivo NUEVO de docs (checklist). `npm pack`/`--dry-run` no tienen efectos de red de escritura.

## Impacto mapeado (Regla 0)
- **Leídos completos:** `.github/workflows/release-npm-node.yml` (223L: build matrix 7 targets, test, publish con OIDC `id-token: write`, environment `npm`, check-node skip-if-exists, smoke-test, attest provenance INFORMATIONAL, `npm publish ${dry_run}`), `vantadb-node/package.json` (59L: name `vantadb-node`, version `0.5.0`, files incluye `*.node`, napi targets 7, engines `node>=18`), `vantadb-node/Cargo.toml` (31L: standalone `[workspace]` vacío intencional R-3 js-ecosystem, `publish=false` crate Rust — solo se publica el paquete npm, no el crate), `vantadb-node/README.md` (68L: estado pre-npm, install desde source), `vantadb-node/index.js` (708L: loader napi auto-generado), `vantadb-node/index.d.ts` (381L: contrato TS), `.opencode/rules/release-ci.md` (42L), `.opencode/rules/js-ecosystem.md` (R-3 standalone intencional, R-4 op-gate), `tasks/GOV-TK2.md` (orden release global: D5 `/ship` GO requerido antes de cualquier publish real).
- **Referencias hacia dentro:** workflow `paths: vantadb-node/**` + tag `node-v*.*.*`; job publish descarga artefactos `vantadb-node-*.node` + `vantadb-node-js` (index.cjs/index.js/index.d.ts/package.json/README.md); `npm pack` empaqueta según `files`.
- **Referencias entrantes:** GOV-TK2 (release 0.6.0 global, BLOQUEADO hasta `/ship` GO) — BND-08 NO lo desbloquea, solo verifica el pipeline en seco. BND-09 gated por BND-08 (targets musl ya presentes en package.json + workflow matrix — verificado de paso, sin tocar).
- **Veredicto:** impacto DOC-ONLY + comandos read-only. Ningún edit en workflow/package/Cargo. Reversible por construcción (checklist es aditiva; `npm pack` genera `.tgz` local que NO se commitea; `--dry-run` no escribe en registry).

## Steps
### Step 1 — DISCOVERY: verificar pipeline + registry 404 + GOV-TK2 orden
- **Archivos:** workflow, package.json, GOV-TK2.md (solo lectura)
- **Acción:** confirmar job Publish con OIDC (`id-token: write`, `environment: npm`, `registry-url`), versión `0.5.0`, `npm view vantadb-node@0.5.0 version` → 404 esperado (no publicado), orden vs GOV-TK2 documentado para la checklist.
- **Verify:** lectura + `npm view` (red solo lectura, permitido; failure 404 = evidencia).
- **Estado:** ✅ (2026-09-05 — `npm view vantadb-node@0.5.0` → 404 Not Found, paquete nunca publicado)

### Step 2 — `npm pack` + prepublish artifacts OK
- **Archivos:** `vantadb-node/` (solo lectura + artefacto `.tgz` efímero, NO commitear)
- **Acción:** `npm pack` en `vantadb-node/` (usa build local existente: `index.cjs`/`index.js`/`index.d.ts` + `*.node` win32-x64-msvc presente). Verificar contenido del tarball (`files` respetados, `engines`, `os`/`cpu`).
- **Verify:** `npm pack` exit 0 + tarball lista archivos esperados.
- **Estado:** ✅ (2026-09-05 — exit 0 → `vantadb-node-0.5.0.tgz`, 6 files, shasum `30130fb5…`; `.tgz` borrado tras S3)

### Step 3 — `npm publish --dry-run` verde
- **Archivos:** ninguno (comando read-only contra registry)
- **Acción:** `npm publish --dry-run` (o sobre el `.tgz` como hace el workflow: `npm publish --dry-run *.tgz`). PROHIBIDO sin `--dry-run`.
- **Verify:** exit 0, output indica dry-run sin publicar.
- **Estado:** ✅ (2026-09-05 — exit 0 → `+ vantadb-node@0.5.0`, tag latest, aviso login solo dry-run; ningún publish real)

### Step 4 — checklist + verify + commit + cierre
- **Archivos:** `docs/plans/artifacts/bnd-08-publish-checklist.md` (NUEVO, único archivo del commit)
- **Acción:** escribir checklist (OIDC/trusted-publisher config, tag `node-v*`, orden vs GOV-TK2 `/ship` GO primero, pasos humáno + rollback). Luego `git add` SOLO ese archivo + commit `ci(node): prepublish verificado dry-run + checklist (BND-08)`. NO stagear `.opencode` ni ajenos. Actualizar plan Task 8 a COMPLETO (sin stagear, lo commitea el orquestador/lead según patrón de waves previas).
- **Verify:** `git status --short` muestra solo el checklist stageado/commiteado; contrato 4/4 verde.
- **Estado:** ✅ (2026-09-05 — checklist escrita + commit `e9843100` solo 1 archivo; ajenos intactos `M .opencode` + `ADR-038` sin stagear; pre-commit hook OK)

## Invariantes
1. NUNCA `npm publish` sin `--dry-run` (publicación irreversible).
2. NUNCA stagear/commitear archivos ajenos (`M .opencode`, trabajo de otras sesiones).
3. NUNCA editar workflow/package.json/Cargo.toml en esta tarea (verificación, no modificación).
4. NO commitear `.tgz` generados por `npm pack` (artefacto efímero → borrar al final).
5. release-plz gobierna versiones — no tocar versiones ni CHANGELOG.

## Deuda técnica (Regla 6)
Saldo neto: **sin deuda** — 0 código productivo, 1 doc aditiva.

## Notas
- `publish = false` en `vantadb-node/Cargo.toml` es del CRATE Rust (no se publica a crates.io); el artefacto npm `vantadb-node@0.5.0` es lo que verifica este task. No confundir.
- BND-09 (musl) queda desbloqueada en cuanto BND-08 verifique pipeline: targets musl ya presentes (package.json napi.targets ×2 musl + workflow matrix ×2 musl) — BND-09 verificará matriz CI, no este task.
- `attest-build-provenance` con `continue-on-error: true` lleva `# CATEGORY: INFORMATIONAL` (release-ci Regla 5 OK).

## Reconciliación 2026-09-09 — verify mecánico del contrato (plan 2026-09-08 Task 4)

**SDP:** `campaign_discover_skills_v2` BUILD → 8 skills (base: campaign-executor auto-vía-MCP,
source-driven-development; lifecycle: incremental-implementation, test-driven-development,
context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design).
Cargadas: doubt-driven-development + incremental-implementation (+ ponytail full activo, campaña base).
Descartadas con motivo: frontend-ui-engineering / api-and-interface-design (misfire lifecycle —
tarea CI/yaml, sin UI ni API nueva); test-driven-development N/A (sin lógica nueva; verify =
comandos mecánicos); context-engineering N/A (contexto = 3 archivos, ya empaquetado);
source-driven-development N/A (sin duda de API externa; napi build/attest ya fijados en workflow).
**SDP: doubt-driven-development + incremental-implementation (+ base auto)**

**Gates:** P no dispara (decisión publish-real-NO heredada, sigue vigente) · D no dispara
(0 archivos productivos editados; workflow intacto) · V no dispara (0 fallas verify) ·
C: `git status` confirma solo `docs/tasks/BND-08.md` propio stageado.

**Impacto mapeado (Regla 0):**
- **Leídos completos:** `.github/workflows/release-npm-node.yml` (223L, matrix 7, jobs build/test/publish),
  `vantadb-node/package.json` (59L, `files` incluye `*.node`, napi targets 7, engines node>=18),
  `vantadb-node/src/lib.rs` (`VantaDB::connect` lib.rs:62, verificado existencia previa).
- **Referencias hacia dentro:** job build → `npm run build` = `napi build --platform --release` (doble: cjs+esm);
  publish descarga artefactos `vantadb-node-*` + `vantadb-node-js`, verifica conteo ≥5, pack, skip-if-exists,
  smoke-test, attest, publish (o `--dry-run` si dispatch con input).
- **Referencias entrantes:** TS-12 BLOQUEADO por BND-08 (se desbloquea con este cierre); GOV-TK2 orden release
  global (checklist previa `docs/plans/artifacts/bnd-08-publish-checklist.md` sigue vigente).
- **Veredicto:** verify-only + 1 doc editada (este archivo). Cero cambios productivos. Reversible por construcción.

**Doubt (degradado, anunciado):** CLAIM "workflow existente satisface contrato sin rewrite".
Disproof intentada: (1) matrix 7≠5 → 7⊇5, quitar musl rompería BND-09 → mantener; (2) "workflow nuevo" →
existe desde `ed75cb0b`, duplicar = riesgo → verificar; (3) `create-npm-dirs` literal ausente →
estrategia single-package intencional (`files: *.node` empaqueta todos los .node; no hay per-platform
dirs que crear; artifacts+prepublish presentes) → documentar, no reescribir; (4) "CI verde" multi-OS
no ejecutable local sin riesgo (dispatch publicaría real si dry_run=false) → verificado lo verificable
local + CI real corre en push-a-main (trigger existente). Cross-model skipped: contexto no-interactivo
(pipeline run) — sin CLI externo sin autorización.

### Steps reconcile (todos ✅ 2026-09-09)
### Step R1 — workflow + matrix + artifacts + prepublish (lectura)
- **Verify:** yaml parse OK (7 targets: msvc/gnu/musl×2/gnu-aarch64/musl-aarch64/darwin×2) +
  jobs build (upload `*.node` + `vantadb-node-js`) / test / publish (verify-count≥5, pack,
  check-node skip-if-exists, smoke-test, attest INFORMATIONAL, publish/`--dry-run`).
- **Estado:** ✅ (matrix 7 ⊇ 5 contrato; `napi build --platform` vía `npm run build`)

### Step R2 — actionlint 0
- **Verify:** `actionlint .github/workflows/release-npm-node.yml` → exit 0, 0 findings.
- **Estado:** ✅

### Step R3 — `npm pack` incluye `*.node` + E404
- **Verify:** `npm pack --dry-run` → 6 files incl. `vantadb_native.win32-x64-msvc.node` (5.4MB),
  shasum `d2077dda…` (sin escribir `.tgz`); `npm view vantadb-node@0.5.0` → E404 (nunca publicado).
- **Estado:** ✅

### Step R4 — commit scope propio + cierre
- **Verify:** `git status --short` solo `M docs/tasks/BND-08.md` stageado; ajenos
  (`M .opencode`, `M opencode.jsonc`, `M vanta-memory/*`) intactos sin stagear.
- **Estado:** ✅ (commit `ci(node): BND-08 reconcile — contrato verificado mecánicamente`)

## Invariantes reconcile (extienden §Invariantes)
6. NUNCA `workflow_dispatch` manual del workflow (publicaría real con dry_run=false por defecto).
7. NUNCA reescribir workflow funcional para calzar nombres literales del contrato
   (`create-npm-dirs` es estrategia multi-package; la vigente es single-package intencional).
8. "CI verde" multi-OS queda a corrida CI real en push-a-main (trigger existente); lo verificable
   local está verde (yaml + actionlint + pack-dry-run).
