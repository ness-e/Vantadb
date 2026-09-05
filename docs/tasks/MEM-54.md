# MEM-54 — Skills CRUD en server HTTP (H5)

## Meta
- **Plan:** `docs/plans/2026-08-22-vanta-ultima-milla.md` — Task 4 (P33)
- **Estado plan al inicio:** ⬜ PENDING
- **Archivos clave:** `src/cli_server.rs` (rutas POST/PUT/PATCH/DELETE skills)
- **Contrato:** "tests D19: create/update/patch/delete vía HTTP con expected_version optimistic lock (patrón MEM-06) + owner check 404 sin filtrar existencia"

## Impacto mapeado (Regla 0)
**Archivos leídos completos (secciones relevantes):**
- `src/cli_server.rs` — imports (:29-36), router (`app_with_cors` :198-301, rutas `/skill/listing` :256), `vanta_error_status` (:877-894, NotFound→404, ExecutionConflict→409), `run_db_op` (:1035-1049), handlers skill_listing (:2637-2697), snapshots_create (patrón AxumPath :2710), tests helpers `cors_test_state`/`spawn_app`/oneshot (:2870+, :3316+)
- `src/skills.rs` — SkillStore completo: get_head(:64), create(:151, idempotencia content-hash), update(:209), patch(:247), delete(:292, retorna Ok(false) si no existe), require_head(:358→NotFound), check_version(:366→ExecutionConflict)
- `src/sdk/types.rs` — SkillCreateInput(:807)/SkillUpdateInput(:828)/SkillPatchInput(:840)/SkillWriteResult(:896) todos Serialize+Deserialize
- `vantadb-mcp/src/skills.rs` — patrón `require_owned`(:201): owner mismatch → mismo 404 que missing
- `docs/api/openapi.yaml` + `scripts/check_openapi_parity.mjs` — CI gate (gate-docs-21.yml): toda ruta `.route(` debe existir en yaml con mismos métodos

**Referencias entrantes:** ninguna hacia los handlers nuevos (aditivo). Router es punto único.
**Referencias salientes:** SkillStore::create/update/patch/delete ya estables (MEM-06).
**Veredicto de impacto:** cambio aditivo en `src/cli_server.rs` (+rutas/+handlers/+tests) y `docs/api/openapi.yaml` (+paths). Cero riesgo sobre skills core ni otros crates.

## Steps
1. ✅ Rutas + handlers HTTP (POST /api/v2/skills; PUT/PATCH/DELETE /api/v2/skills/{skill_id}) con query params owner_agent+expected_version; owner-mismatch → 404 idéntico a missing (anti-enumeración); stale expected_version → 409 vía ExecutionConflict existente.
2. ✅ Tests D19 en mod tests (oneshot): roundtrip CRUD + idempotencia + 409 lock + 404 owner-mismatch vs missing indistinguibles.
3. ✅ OpenAPI parity: paths nuevos en docs/api/openapi.yaml; `node scripts/check_openapi_parity.mjs`.
4. ✅ Verify mecánico: cargo check/test/fmt/clippy -p vantadb --all-targets.

## Context Save Point
(ninguno aún)

