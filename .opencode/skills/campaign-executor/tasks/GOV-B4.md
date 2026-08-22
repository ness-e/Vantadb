# GOV-B4 — Regeneración completa openapi.yaml + gate de paridad

**Estado:** ⏳ IN PROGRESS
**Plan:** docs/plans/2026-08-22-doc-governance-plan.md (Task 12)
**Contrato:** count(paths en openapi.yaml con método) == count(routes registradas en cli_server.rs); script `scripts/check_openapi_parity.mjs` (node stdlib-only) extrae rutas del .rs vía regex `.route("...")` y compara contra el yaml; exit 0 paridad, exit 1 listando diffs.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/cli_server.rs:195-284` — bloque completo del router (`app_with_cors`): 33 llamadas `.route()` (32 protected + `/health` public). Rutas adicionales en `cli_server.rs:1709-1710` (`/dashboard`, `/dashboard/{*path}` — branch sin dashboard-dir). Total archivo: 35 paths distintos.
- `docs/api/openapi.yaml` (495L completas) — estado actual: solo 3 paths (`/health`, `/metrics`, `/api/v2/query`). Version `0.5.0` == workspace (`Cargo.toml:646`).
- `.github/workflows/gate-docs-21.yml` (82L completas) — 3 jobs: lint-markdown, check-format, check-api-version (:56-81, valida `openapi.yaml` version vs workspace vía `grep -m1 '^  version:'`).
- Handlers core leídos: `records_put`(:1098, body=`VantaMemoryInput`), `records_put_batch`(:1109, `Vec<VantaMemoryInput>`), `records_search`(:1305, body=`SearchPageRequest` = flatten `VantaMemorySearchRequest` + cursor/limit), `export_v2`(:1988, `ExportRequest{path,namespace,filter}`), `import_v2`(:2006, `ImportRequest{records,path,format}`). Tipos fuente: `src/sdk/types.rs:131` (VantaMemoryInput), `src/sdk/serialization/vector_types.rs:12` (VantaMemorySearchRequest).

**Referencias entrantes:** `gate-docs-21.yml:68` grep del campo version (mantener formato `  version:` con 2 espacios bajo info). HTTP_API.md referencia conceptual (no se regenera en esta tarea).

**Referencias salientes:** ninguna rota — el yaml es terminal (consumido por gate y humanos).

**Veredicto:** impacto contenido a 3 archivos (yaml regenerado, script nuevo, gate extendido). `src/` READ-ONLY.

## Steps

### ✅ Step 1: Enumerar rutas del router (Regla 0, conteo exacto)
- 35 `.route()` calls totales; **29 paths `/api/v2/*`**, 6 no-v2 (`/health`, `/metrics`, `/conversation/add`, `/skill/listing`, `/dashboard`, `/dashboard/{*path}`)
- Operaciones path+método: **40** (ver desglose en RESULTADO final)

### ✅ Step 2: Leer fuentes (hecho arriba)

### ✅ Step 3: Regenerar docs/api/openapi.yaml (contract-first, schemas core-only)
- 35 paths / 40 operaciones; version "0.5.0" preservada con formato del gate (`grep -m1 '^  version:'` sigue matcheando)
- Schemas detallados: QueryRequest/QueryResponse, RecordInput (VantaMemoryInput), SearchPageRequest (flatten VantaMemorySearchRequest + cursor/limit), ExportRequest, ImportRequest. Resto: envelopes genéricos (Ack/RecordEnvelope/ListPage/SearchPage) + GraphTraversalBody compartido
- x-experimental: true en /dashboard, /dashboard/{path}, /conversation/add, /skill/listing

### ✅ Step 4: scripts/check_openapi_parity.mjs — exit 0 verificado (+ test negativo exit 1)
- Scanner de parens multi-line safe para `.route("...", get(h).post(h2))`; normaliza `{*name}`→`{name}`
- Lector yaml mínimo por indentación (2-space paths, 4-space methods) — stdlib-only

### ✅ Step 5: gate-docs-21.yml — step "Check OpenAPI/router parity" agregado a job check-api-version (check-api-version intacto); triggers extendidos con src/cli_server.rs y scripts/**

### ✅ Step 6: Verify
- `node scripts/check_openapi_parity.mjs` → exit 0 (35↔35 paths, 40↔40 ops)
- Test negativo (yaml mutado en copia temp) → exit 1 listando missing/extra ✅
- `python -c yaml.safe_load` → OK (pyyaml disponible); 27 $refs, 0 rotos
- `npx markdownlint-cli2 "docs/api/*.md"` → 0 issues

## Context Save Point
- Conteo autoritativo: ver RESULTADO del agente. Normalización `{*name}`→`{name}` para comparar.
