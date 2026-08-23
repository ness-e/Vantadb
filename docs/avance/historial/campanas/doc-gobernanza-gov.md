# Gobernanza Documental — entradas GOV

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### GOV-B4: openapi.yaml completo (~29 paths desde cli_server.rs) + gate paridad
- **Fuente:** Plan `docs/plans/2026-08-22-doc-governance-plan.md` (Task 12)
- **Fecha:** 2026-08-22
- **Objetivo:** Regenerar `docs/api/openapi.yaml` contract-first cubriendo TODAS las rutas del router HTTP (`src/cli_server.rs`, READ-ONLY) y agregar gate CI de paridad router↔spec.
- **Resultado:** ✅ `docs/api/openapi.yaml` regenerado: 35 paths / 40 operaciones (29 bajo `/api/v2/*`), version "0.5.0" sincronizada al workspace, tags por dominio (Query/Records/Search/Graph/Maintenance/Threads/System), schemas detallados solo core (query, records CRUD, search, export/import — wire shapes desde `VantaMemoryInput`/`VantaMemorySearchRequest`/`ExportRequest`/`ImportRequest`), envelopes genéricos para el resto, `x-experimental: true` en `/dashboard`, `/dashboard/{path}`, `/conversation/add`, `/skill/listing`. `scripts/check_openapi_parity.mjs` (nuevo, node stdlib-only): scanner de parens extrae `.route()` multi-line del .rs (normaliza `{*name}`→`{name}`), lector yaml por indentación, exit 0 en paridad / exit 1 listando missing+extra+method-diffs (test negativo verificado). `.github/workflows/gate-docs-21.yml`: step "Check OpenAPI/router parity" en job check-api-version (check-api-version intacto) + triggers extendidos (`src/cli_server.rs`, `scripts/**`). Verify: parity exit 0 ✅; pyyaml safe_load OK ✅; 27 $refs sin rotos ✅; markdownlint docs/api/*.md 0 issues ✅. Commit pendiente del lead (git prohibido en la invocación).
- **Ids:** `GOV-B4`
