---
title: "Contratos del proyecto"
kind: concept
status: active
description: "Qué es contrato estable y dónde se verifica mecánicamente: API pública, wire formats, paridad, namespaces y versionado."
---

# Contratos del proyecto

## Contratos con gate mecánico

`[HECHO]` Mapa de autoridad (SURFACE_MANIFEST §Authority map): API Rust → [`tests/api/public-api.txt`](../../../tests/api/public-api.txt) (gate `tests/api/public_api.rs`); HTTP → `docs/api/openapi.yaml` (gate `tests/api/openapi_yaml_parity.rs`); bindings → `docs/api/BINDINGS_NAMESPACES.md`; superficie/estado → `docs/api/SURFACE_MANIFEST.md` (generado); versiones → `Cargo.toml` workspace + `package.json`/`pyproject.toml` (gates `check-npm-versions.mjs`, `tests/version_coherence.rs`); serialización SDK → `tests/sdk_serialization.rs`; paridad cross-binding → `dev-tools/parity-compare.mjs` + `tests/parity/scenario.json`.

## Contratos normativos sin gate

`[HECHO]` Casing por payload (Gate P), score-vs-distance por transporte, taxonomía de dominios memory/graph/wiki/system, campos v2 wire (SCH-07) y excepciones declaradas — todo en BINDINGS_NAMESPACES. `[HECHO]` ADRs inmutables una vez aceptados (MADR; `docs/dev/architecture/adr/README.md` generado).

## Regla de cambio

`[INFERENCIA]` Cambiar una firma pública sin actualizar su contrato (snapshot/OpenAPI/matriz/doc api) deja el cambio incompleto: las gates correspondientes fallarán o la deriva quedará invisible — ver [change-checklist](../contributor/change-checklist.md).

## Fuentes

[SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md) · [BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md) · `tests/api/public-api.txt` · `docs/api/openapi.yaml` · [COMPATIBILITY](../../api/COMPATIBILITY.md) · [VERSIONING](../../api/VERSIONING.md)
