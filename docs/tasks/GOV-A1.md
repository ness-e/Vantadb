# GOV-A1: Medición openapi parity docs/api/openapi.yaml vs src/server/routing.rs

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Creado:** 2026-09-02T20:00
- **last-synced:** 2026-09-02T20:00
- **Estado:** ✅ COMPLETED
- **Wave:** Wave1 paralelo MAX 3 con A2,A3 (depende Wave0 GOV-T01..T03 ✅)
- **No tocar:** RES-02..05 (aislados P38 — no tocados)

## Blast Radius
- **Callers:** `scripts/check_openapi_parity.mjs` (CI gate-docs-21), `docs/api/HTTP_API.md` (deriva de openapi.yaml), `docs/api/openapi.yaml` itself
- **Callees:** `src/server/routing.rs` (facade) → `src/server/router.rs` (37 paths, 44 ops reales) + `src/server/handlers.rs` (handlers), `dev-tools/validate_doc_snippets.py` (no afectado directamente, pero share docs/api)
- **Implicaciones:** Fix docs-only. No tocar Rust. Cambios: `scripts/check_openapi_parity.mjs` RS_FILE + mensaje + `docs/api/openapi.yaml` (remover /fast /slow). Riesgo: si se elimina mal path, gate-docs falla. Mitigación: `node scripts/check_openapi_parity.mjs` debe pasar + `cargo check` verde.

## Contrato
`node scripts/check_openapi_parity.mjs` → `Parity OK` (0 extra, 0 missing, 0 methodDiff) AND `cargo check --workspace` exit 0
- Medición canónica: Router 37 paths / 44 ops, OpenAPI 39 paths / 46 ops → gap = 2 paths (`/fast`, `/slow` x-experimental no implementados)
- Tras fix: ambos 37 paths / 44 ops

## Herramientas
- codegraph_explore "openapi parity routing"
- node scripts/check_openapi_parity.mjs
- cargo check --workspace

## Steps

### Step 1: Medir gap 35 vs 40 (real 39 vs 37)
- **Archivos:** `docs/api/openapi.yaml`, `src/server/router.rs`, `scripts/check_openapi_parity.mjs`
- **Acción:** Contar paths/ops reales: router 37/44 (app_with_cors) vs openapi 39/46. Script actual apunta a src/cli_server.rs obsoleto → reporta 0/39. Documentar gap: /fast + /slow son extra en yaml sin handler.
- **Verify:** `node scripts/check_openapi_parity.mjs` muestra FAIL con extra /fast /slow (o 0 si apunta a cli_server.rs) + `node check_routes.mjs` confirma 37 vs 39
- **Estado:** ✅ COMPLETED — medición: Router 37/44, OpenAPI 39/46, gap = /fast, /slow (x-experimental sin handler). Script pre-fix: 0 paths (RS_FILE obsoleto).

### Step 2: Completar openapi o documentar exclusiones (docs-only, ponytail)
- **Archivos:** `docs/api/openapi.yaml` (remover /fast /slow), `scripts/check_openapi_parity.mjs` (RS_FILE src/server/router.rs, mensaje Router (src/server/router.rs))
- **Acción:** (ponytail: diff mínimo docs-only)
  1. Editar scripts/check_openapi_parity.mjs: RS_FILE = src/server/router.rs, comentario y log actualizados (5 ediciones)
  2. Editar docs/api/openapi.yaml: eliminar bloques `  /fast:` y `  /slow:` completos (44 líneas), dejando 37 paths que matchean router
  3. No añadir handlers Rust, no tocar src/server/routing.rs
- **Verify:** `node scripts/check_openapi_parity.mjs` → Parity OK (37 paths, 44 ops) + `cargo check --workspace` ✅
- **Estado:** ✅ COMPLETED — verify 2026-09-02: Parity OK + cargo check --workspace Finished dev

## Dependencias
- GOV-T01..T03 ✅ (Wave0)
- No depende de RES-02..05 (prohibido tocar)

## Notas
- Task original plan decía 35/40, medición real 39/37: números plan desactualizados tras split routing.rs. Documentar cifra canónica 37/44 aquí.
- codegraph_explore confirma handlers en src/server/handlers.rs, routing es facade.
- Ponytail: no crear nueva abstracción parity, reutilizar script existente con fix 1 línea.

## Context Save Point
- **Fecha:** 2026-09-02T20:00
- **Branch:** main (detached? verify git status)
- **CI pendiente:** gate-docs-21 parity
- **Decisiones:** Fix docs-only: remover /fast /slow de yaml (no implementar handlers vacíos)
- **Problemas conocidos:** Ninguno tras fix
- **Próxima tarea:** GOV-A2 (Wave1 paralelo)
