---
title: "Verificación y pruebas"
kind: concept
status: active
description: "Cómo se verifica el repo: suites de test registradas, snapshots, paridad, fuzz y gates de CI — inventario, no ejecución."
---

# Verificación y pruebas

## Suites registradas

`[HECHO]` `Cargo.toml` lista suites `[[test]]` por área (están diseñadas para verificar; no se ejecutaron en esta sesión): core (`regression_certification`, `basic_node`, `hnsw`, `graph`, `vector_scale_check`), storage (`mutations`, `core_invariants`, `wal_resilience`, `durability_recovery`, `chaos_integrity`, `crash_injection`, `multi_process_lock`), API (`structured_api_v2`, `openapi_yaml_parity`, `public_api`, `python_sdk_boundary`), server (`request_id`, `server_auth_rotation`, `rbac_namespace` — feature `server`), migración/temporal (`schema_v2_migration`, `chaos_migration`, `time_travel`), certificación (`stress_protocol`, `hnsw_validation`, `hnsw_recall_certification`, `sift_validation`, `hybrid_retrieval_quality`, `competitive_bench`, `hardware_profiles`).

`[HECHO]` El mapa de tests por área es [TEST_MAP](../../dev/operations/TEST_MAP.md); caos/failpoints en [chaos-testing](../../dev/operations/chaos-testing.md) y fuzz en [FUZZING](../../dev/operations/FUZZING.md) (`cargo +nightly fuzz run`, Linux).

## Snapshots y paridad

`[HECHO]` [`tests/api/public-api.txt`](../../../tests/api/public-api.txt) se regenera con `VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api` (comando documentado en la cabecera del propio snapshot). `[HECHO]` Paridad cross-binding: escenario `tests/parity/scenario.json` en Python/WASM/Node + comparador `dev-tools/parity-compare.mjs` (CI: job `parity` en [`.github/workflows/ci-fleet.yml`](../../../.github/workflows/ci-fleet.yml#L191-L286)).

## Gates CI

`[HECHO]` Workflows: `ci-rust.yml`, `ci-gate.yml`, `docs-gates.yml` (docs: check-docs/check-links/gen-index --check), `certification.yml`, `nightly.yml`, `platform-ops.yml`, más la familia `release-*` ([tooling](../modules/tooling.md)).

## Advertencia

`[INFERENCIA]` Esta página inventaria la maquinaria de verificación; no certifica que esté verde — los resultados los produce la ejecución del usuario/CI.

## Fuentes

`Cargo.toml` · `tests/` · `fuzz/` · `dev-tools/` · `.github/workflows/` · [TEST_MAP](../../dev/operations/TEST_MAP.md)
