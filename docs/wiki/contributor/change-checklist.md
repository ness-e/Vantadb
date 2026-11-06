---
title: "Checklist de cambio"
kind: concept
status: active
description: "Qué actualizar junto a un cambio según su tipo: snapshots, matrices, OpenAPI, índices generados y ADRs."
---

# Checklist de cambio

Regla base ([OPERATING-MODEL](../../dev/OPERATING-MODEL.md) + SURFACE_MANIFEST): un cambio de API pública sin su contrato actualizado está incompleto.

## Por tipo de cambio

`[HECHO]` Superficie Rust pública → regenerar [`tests/api/public-api.txt`](../../../tests/api/public-api.txt) (`VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api`, cabecera del snapshot) + revisar `docs/api/EMBEDDED_SDK.md`. `[HECHO]` Endpoint HTTP → `docs/api/openapi.yaml` + `docs/api/HTTP_API.md` + gate `openapi_yaml_parity`. `[HECHO]` Método de binding → fila en `docs/api/BINDINGS_NAMESPACES.md` (matriz W1/diferencias) + doc `*_SDK.md` del binding + evidencia same-PR; si es capacidad W1 nueva debe existir en los 4 bindings o declarar la excepción.

`[HECHO]` Versión → solo release-plz (`release-plz.toml`); nunca a mano (post-mortem 0.9.0 referenciado en AGENTS). `[HECHO]` Nuevo `.md` en docs → frontmatter `title`+`kind` (kind por ruta vía deriveKind) + `description`; luego `node scripts/docs/gen-index.mjs --write`. `[HECHO]` Decisión arquitectónica → ADR nuevo MADR con ≥2 opciones y consecuencias negativas; nunca editar un ADR aceptado (usar `supersedes`).

`[INFERENCIA]` Changelog y `llms.txt`/índices son generados: no editarlos a mano; el diff correcto llega regenerando.

## Antes del PR

`[HECHO]` `node scripts/docs/check-docs.mjs`, `check-links.mjs`, `gen-index.mjs --check` son los gates de docs (`docs-gates.yml`); `verify_changed.ps1` es el quick check local documentado. Los ejecuta el usuario/CI.

## Fuentes

[contracts](../developer/contracts.md) · [verification](../developer/verification.md) · [SURFACE_MANIFEST](../../api/SURFACE_MANIFEST.md) · `scripts/docs/`
