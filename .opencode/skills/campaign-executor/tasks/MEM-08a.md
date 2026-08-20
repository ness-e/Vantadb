# MEM-08a: F4 Fundación crate vanta-memory

## Metadata
- **Plan file:** docs/plans/2026-08-18-vanta-memory.md
- **Creado:** 2026-08-20T19:00
- **last-synced:** 2026-08-20T19:00
- **Estado:** ✅ COMPLETED (scaffold creado; `cargo check -p vanta-memory` ✅ + 2/2 smoke 2026-08-20)

## Contrato
"`cargo check -p vanta-memory` pasa; crate es workspace member"

## Steps
- **Step 1 (scaffold):** `vanta-memory/Cargo.toml` (workspace inheritance, features `llm-driver` default off + `mock` para tests, deps serde/serde_json/thiserror/tracing + `vantadb` default-features=false); `vanta-memory/src/lib.rs` (doc + 6 módulos esqueleto: core/utils/services/adapters/offload/gateway + `name()`); `vanta-memory/tests/smoke.rs` (2 tests D19); registrado en `Cargo.toml` raíz `[workspace] members` (NO default-members — experimental, política CI_POLICY.md). — ✅
- **Step 2 (verify):** `cargo check -p vanta-memory` ✅ (33s); `cargo nextest run -p vanta-memory` ✅ 2/2 (crate_links, llm_driver_feature_is_opt_in). — ✅

## Dependencias
- Ninguna (F4 base).

## Notas
- `vanta-memory` es crate experimental → fuera de default-members (igual que server/mcp/wasm). `cargo check --workspace` lo cubre pero no bloquea CI core.
- Los 7 warnings de `cargo check` son de `vantadb` core (pre-existentes), no del scaffold.
- Siguiente: MEM-08b (contratos + trait LLMRunner) — se delega a vanta-worker con pipeline-full.md.