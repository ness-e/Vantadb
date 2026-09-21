# REST-02 — `/api/v2/metrics` JSON (métricas del motor en shape JSON)

> **Plan:** `docs/plans/2026-08-19-vanta-studio-fase4.md` — Wave 1
> **Estado:** ✅ COMPLETO (worktree listo, sin commit — verify/commit del lead)
> **Fecha:** 2026-08-19

## Contrato

- `GET /api/v2/metrics` → JSON con métricas del motor (mismo shape que `namespace_stats`/`VantaMetrics` existente: `VantaOperationalMetrics` + `VantaNamespaceStatsMap`)
- CORS igual que el resto de endpoints (ruta en el router `protected` → hereda auth + rate-limit + CORS outermost)
- Documentado en `docs/api/HTTP_API.md`
- `vanta_metrics` deja de estar en la lista de rechazos de `vanta-http-map.ts`

## Impacto mapeado (Regla 0)

- **Leídos completos:** `src/cli_server.rs` (app_with_cors L141-235, metrics_endpoint L506-520, tests L2606+), `src/sdk/api.rs` (namespace_stats L1393, operational_metrics L1192), `src/sdk/types.rs` (VantaOperationalMetrics L331-406), `desktop/src/vanta-http-map.ts`, `desktop/src/vanta-http-map.test.ts`, `docs/api/HTTP_API.md`
- **Referencias entrantes:** `desktop/src/vanta.ts` → `metrics()` llama `vanta_metrics` (antes rechazado); FEAT-02 consume `/api/v2/metrics`
- **Referencias salientes:** cli_server.rs usa `crate::sdk::{VantaNamespaceStatsMap, VantaOperationalMetrics}`, `run_db_op`, `cors_layer`
- **Veredicto:** cambio aditivo (nueva ruta + handler + test), no toca endpoints existentes, no toca `src/wal.rs`/`src/vector/`/`src/storage/`

## Steps

- [x] 1. Verificar fuente real de métricas: `VantaOperationalMetrics` (src/sdk/types.rs) + `namespace_stats` (src/sdk/api.rs) — existe
- [x] 2. Ruta `/api/v2/metrics` en router protected de `app_with_cors` (src/cli_server.rs L196)
- [x] 3. Handler `metrics_v2` con `run_db_op` → `{ metrics, namespaces }` (L537-549)
- [x] 4. Test `metrics_v2_returns_json_operational_snapshot` (L2609+)
- [x] 5. Mapping `vanta_metrics` en `vanta-http-map.ts` (GET /api/v2/metrics, transform → `.metrics`) + test
- [x] 6. Documentar en `docs/api/HTTP_API.md`
- [x] 7. Verify: `cargo test --features server` verde
- [x] 8. Verify: node:test `vanta-http-map.test.ts` verde (vanta_metrics ya no es rechazo)
- [x] 9. Verify: curl `/api/v2/metrics` → 200 JSON con campos esperados

## Verify

- `cargo test --features server` — ✅ (ver output en sesión)
- `node --test src/vanta-http-map.test.ts` (desktop/) — ✅
- curl local con DB temp — ✅

## Notas

- NO commit — vanta-lead hace verify + commit (Regla del plan: sub-agentes no commitean)
- `MetricsV2Response` expone `namespaces` (VantaNamespaceStatsMap) además de `metrics` — FEAT-02 (Índices/salud) lo consume
- Worktree ya contenía el trabajo parcial de un intento previo; se validó contra contrato y se completó verify