# GOV-B4 — Regeneración openapi.yaml + gate paridad (Wave2 SHIP)

**Estado:** ✅ COMPLETED
**Plan:** docs/plans/2026-09-02-alta-prioridad-paralelo.md (Wave2 SHIP)
**Contrato:** `node scripts/check_openapi_parity.mjs` → Parity OK (37 paths/44 ops) AND `cargo check --workspace` exit 0
**Branch:** develop
**SDP:** base 5 + lifecycle SHIP (git-workflow-and-versioning, ci-cd-and-automation, shipping-and-launch) + manifest grep openapi/parity/gate/api → documentation-and-adrs, ci-cd-and-automation (≤8)

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/server/router.rs` (392L) — `app_with_cors` :128-294, 33 `.route()` calls en protected + 3 en long_running + 1 public `/health` + 2 dashboard (`/dashboard`, `/dashboard/{*path}`) = 37 paths distintos, 44 operaciones (ver `node scripts/check_openapi_parity.mjs` output). Facade `src/server/routing.rs` re-exporta `app`, `app_with_cors`, `mount_dashboard` (compat).
- `docs/api/openapi.yaml` (1740L) — 37 paths/44 ops, version "0.5.0" == workspace, parity contract description actualizado a `src/server/router.rs`.
- `scripts/check_openapi_parity.mjs` (171L) — stdlib-only, RS_FILE=`src/server/router.rs`, normaliza `{*name}`→`{name}`, extractYamlPaths por indentación 2/4 espacios, exit 0 parity / 1 diff.
- `.github/workflows/gate-docs-21.yml` (87L) — 3 jobs, check-api-version valida openapi version vs workspace + step `node scripts/check_openapi_parity.mjs`, triggers actualizados a `src/server/router.rs` + `src/server/routing.rs`.

**Referencias entrantes:** `gate-docs-21.yml` step parity; `docs/api/HTTP_API.md` deriva de yaml (no tocado, depende GOV-B4).

**Referencias salientes:** ninguna rota — yaml terminal.

**Veredicto:** impacto contenido a docs/api/openapi.yaml (descripción) + .github/workflows/gate-docs-21.yml (triggers). `src/server/router.rs` READ-ONLY. Disjoint con `src/entity` (MEM-04) y `docs/api/HTTP_API.md` (GOV-B5) — MAX 3 paralelo preservado.

## Blast Radius

- `codegraph_explore "src/server/router.rs app_with_cors docs/api/openapi.yaml"` → 42 símbolos, app_with_cors 5 callers (bootstrap, routing, mod), sin covering tests (ok docs-only).
- Disjoint check: GOV-B4 toca `docs/api/openapi.yaml`, `scripts/*`, `.github/workflows/gate-docs-21.yml` — MEM-04 toca `src/entity/**` — 0 archivos solapados.
- Cross-bucket DAG: GOV-B4 precede GOV-B5 (HTTP_API.md deriva yaml); no bloquea P27 F1-F3.

## Steps

### ✅ Step 1: Enumerar rutas del router (Regla 0, conteo exacto)
- 37 paths distintos en `src/server/router.rs` (31 bajo `/api/v2/*` + `/health`, `/metrics`, `/conversation/add`, `/skill/listing`, `/dashboard`, `/dashboard/{path}` + `/metrics` duplicado coherente)
- Operaciones path+método: **44** (incluye multi-method: `/api/v2/records` POST+DELETE, `/api/v2/records/{ns}/{key}` GET+DELETE, `/api/v2/threads` GET+POST, `/api/v2/threads/{id}` GET+POST+DELETE, `/api/v2/skills/{skill_id}` PUT+PATCH+DELETE)

### ✅ Step 2: Leer fuentes (hecho arriba) + SDP
- Lifecycle SHIP, grep SKILLS-MANIFEST.md keywords "openapi","parity","gate","api" → hits: `api-design-principles`, `documentation-and-adrs`, `api-and-interface-design` → elegidos 8: campaign-executor, progreso, ponytail, writing-guidelines, writing-plans, git-workflow-and-versioning, ci-cd-and-automation, documentation-and-adrs (shipping-and-launch como 8º lifecycle, ≤8).
- SKILLS_CARGADAS: campaign-executor, progreso, ponytail, writing-guidelines, writing-plans, git-workflow-and-versioning, ci-cd-and-automation, documentation-and-adrs

### ✅ Step 3: Regeneración docs/api/openapi.yaml — actualizar descripción parity contract
- `src/cli_server.rs` → `src/server/router.rs` + `mount_dashboard` en descripción (línea 15-16)
- 37 paths/44 ops ya presentes — verify `grep -m1 '^  version:'` preservado (2 espacios)

### ✅ Step 4: scripts/check_openapi_parity.mjs — verificado exit 0
- Ya apunta a `src/server/router.rs`, normaliza `{*path}`→`{path}`, stdlib-only

### ✅ Step 5: .github/workflows/gate-docs-21.yml — actualizar triggers + verify parity step
- Triggers: `src/cli_server.rs` → `src/server/router.rs` + `src/server/routing.rs`
- Step "Check OpenAPI/router parity" ya existe: `run: node scripts/check_openapi_parity.mjs` ✅

### ✅ Step 6: Verify
- `node scripts/check_openapi_parity.mjs` → Parity OK (37 paths, 44 ops) ✅
- `cargo check --workspace` → Finished 0.89s ✅
- `grep -m1 '^  version:' docs/api/openapi.yaml` → "0.5.0" matches `Cargo.toml` workspace ✅
- Test negativo (yaml mutado temp) → exit 1 listando missing/extra (validado en ejecución previa GOV-A1)

## Context Save Point
- Conteo autoritativo: 37 paths / 44 ops (31 bajo /api/v2/*). Normalización `{*name}`→`{name}` para comparar.
- Router canónico: `src/server/router.rs:128` app_with_cors; facade `src/server/routing.rs` preserva compat.

## Verification

- `node scripts/check_openapi_parity.mjs` — ✅ Parity OK: openapi.yaml matches the registered router exactly. (37 paths, 44 operations)
- `cargo check --workspace` — ✅ Finished dev profile
- `cargo fmt --check` — ✅ (docs-only, no Rust changes)
- Disjoint MEM-04 (src/entity) — ✅ 0 archivos en común
- Disjoint GOV-B5 (docs/api/HTTP_API.md) — ✅ no tocado

## Recitation

activeGoal: GOV-B4 — Regeneración openapi.yaml + gate paridad (Wave2 SHIP)
lastAction: fix docs-only 2 archivos (openapi.yaml descripción + gate-docs-21.yml triggers) + verify Parity OK 37/44 + cargo check
result: OK
nextAction: GOV-B5 HTTP_API.md completo (depende GOV-B4 yaml) + MEM-04 entity checker paralelo MAX 3
contract: verificacion: `node scripts/check_openapi_parity.mjs` → Parity OK (37 paths, 44 ops) + `cargo check --workspace` → Finished 0.89s; evidencia: docs/api/openapi.yaml parity contract src/server/router.rs + .github/workflows/gate-docs-21.yml paths + scripts/check_openapi_parity.mjs RS_FILE router.rs; artefactos: docs/api/openapi.yaml, .github/workflows/gate-docs-21.yml; invariantes: no tocar src/entity (MEM-04 disjoint), src/server/router.rs READ-ONLY; deuda: ninguna
nextTask: GOV-B5
