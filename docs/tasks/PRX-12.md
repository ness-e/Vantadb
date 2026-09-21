# PRX-12 — Compat suite contra releases de coding agents

- **Estado:** ✅ COMPLETED (S0-S4 ✅ — commit pendiente de hash)
- **Plan:** `docs/plans/2026-09-09-backlog.md` Task 6 (Wave1, 5/6 ✅, disjunta: tests/fixtures)
- **Appetite:** max 2d · **Esfuerzo:** 🟡 1-2d · **Prioridad:** 🟡 Media
- **Archivos clave:** `vanta-proxy/tests/fixtures/`, CI
- **Contrato:** fixtures requests/responses reales (Claude Code/Codex/OpenCode) + tests regresión en cada PR ✅ + `cargo test -p vanta-proxy` 0 failed
- **Branch:** develop
- **SDP:** test-driven-development, incremental-implementation, source-driven-development, context-engineering (SDP v2 BUILD; frontend-ui-engineering/api-and-interface-design descartadas — sin UI ni API nueva; doubt-driven sin stakes de prod)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vanta-proxy/tests/proxy_wire.rs:1-250` (patrón setup/seeded_engine/post_json),
  `vanta-proxy/tests/tool_loop.rs:1-80` (patrón mock scripted), `vanta-proxy/Cargo.toml` (reqwest+tokio
  disponibles para tests), `.github/workflows/ci-rust-10.yml` (test job corre nextest audit en cada
  push/PR con paths `vantadb-*/**`), `.config/nextest.toml` (audit no excluye binarios vanta-proxy).
- **Referencias hacia dentro:** `prx12_compat.rs` → `vanta_proxy::config`, `vanta_proxy::server`,
  `vantadb::storage::StorageEngine`, `vantadb::entity::EntityStore` (mismo grafo que proxy_wire/tool_loop).
- **Referencias entrantes:** ninguna (archivo de test nuevo + fixtures JSON; CI los recoge por glob).
- **Veredicto:** impacto ADITIVO PURO — 8 archivos nuevos, 0 ediciones en `src/`, 0 cambios CI
  (el job `test` de ci-rust-10 ya corre todo `vanta-proxy/tests/` en cada PR), 0 símbolos `pub` nuevos.
  Gate D: no disparado (sin hot path, sin API pública, sin blast radius >10 archivos).

## Spec-lite (decisiones)

| # | Decisión | Justificación por evidencia |
|---|----------|----------------------------|
| 1 | Fixtures = shapes protocolares representativos, NO capturas live | No existe tráfico live en el repo; stop condition prohíbe inventar → README declara proveniencia explícita |
| 2 | 3 pares req/resp: Messages (Claude Code), Responses (Codex), Chat (OpenCode) | Mapeo agente→protocolo de las suites wire existentes (proxy_wire cubre las 3 rutas) |
| 3 | Sin cambios en CI | `ci-rust-10.yml` job `test` + paths `vantadb-*/**` ya corren el nuevo binario en cada PR; nextest audit no lo excluye |
| 4 | Sanitización: 0 secrets en disco; credenciales solo en memoria de test | Pre-mortem #1; grep pre-commit en verify |

## Steps

- [x] **S0 DISCOVERY** — tipo=proxy, patrón proxy_wire, CI ya cubre, Gate D no-disparado.
- [x] **S1 Fixtures** — 6 JSON + `fixtures/README.md` (proveniencia, sanitización, procedimiento de actualización).
- [x] **S2 Test regresión** — `tests/prx12_compat.rs` (3 round-trip verbatim + 3 contratos de campos).
- [x] **S3 Verify mecánico** — `cargo test -p vanta-proxy` 148 passed 0 failed (142 PRX-09 + 6 nuevos) + fmt 0 + clippy 0 + grep secrets 0 hits. Hallazgo S3: `codex_responses_request.json` con `}` extra (`function"}}]` → `function"}]`, diagnosticado con bracket-matching) — corregido, 6/6 verde.
- [x] **S4 Commit** — solo archivos propios + recitation + RESULTADO.

## Verificación (S3 — 2026-09-09 ✅)

- `cargo test -p vanta-proxy --test prx12_compat` — ✅ 6 passed 0 failed
- `cargo test -p vanta-proxy` (148 passed 0 failed: 142 PRX-09 sin regresión + 6 nuevos) — ✅
- `cargo fmt --check` — ✅ (1 ajuste auto-aplicado en closure de mock)
- `cargo clippy -p vanta-proxy --all-targets -- -D warnings` — ✅ 0 warnings
- `grep secrets en fixtures/` 0 hits — ✅

## Notas

- NOTICED BUT NOT TOUCHING: `ci-rust-10.yml` solo dispara en PRs a `main`, no a `develop`
  (línea 20-21) — decisión de CI policy, dueña vanta-lead; no scope de este task.
- WIP ajeno intacto: `.opencode`, `opencode.jsonc`, `docs/Backlog.md`, `docs/avance/*`,
  `Investigacion-plan.md`, `docs/plans/2026-09-09-backlog.md` (untracked) — no stagear.
