---
title: "FIND-344 — publicación automatizada al MCP Registry"
kind: task
status: active
description: "Automatizar publish al MCP Registry: fix server.json, workflow CI owner-gated, gate offline."
tags: [ci, mcp, registry]
---

# FIND-344 — publicación automatizada al MCP Registry

Decisión owner: flujo CI automatizado, nunca manual. PROHIBIDO: push, tags,
PRs, publicar en registries. Review + commit los hace el orquestador.

## BLAST RADIUS (Discovery, por evidencia)

| Artefacto | Existe | Evidencia |
|---|---|---|
| MCP server | Sí, embebido en `vantadb-server --mcp` | `vantadb-server/src/main.rs:25` (`MCP_FLAG`), `run_stdio_server_auto` vía `vantadb-mcp` |
| Package npm/pypi/oci del MCP server | NO | `vantadb-mcp`: `publish = false`, "ships inside the server binary" (`docs/dev/workflow/PUBLISH.md:195`) |
| `.mcpb` / manifest bundle | NO | Sin `*.mcpb` en repo; binarios van a GitHub Release (`release-binaries.yml`) |
| `server.json` | Sí, inválido (1 error) | `description` 101 chars > `maxLength: 100` del schema draft-07 |
| Workflow registry | NO | 18 workflows, ninguno de MCP Registry |

## Schema (fuente oficial, leída 2026-10-16)

`https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json`
(draft-07). Hechos decisivos:

- `ServerDetail.required = [name, description, version]` — `packages[]` y
  `remotes[]` son **opcionales**. El fix de `description` basta para validar.
- `Package.required = [registryType, identifier, transport]` **solo si** hay
  entrada en `packages[]` (camelCase del schema, no `registry_type`).
  `fileSha256` (`^[a-f0-9]{64}$`) requerido para `registryType: mcpb`,
  `identifier` = URL al `.mcpb`.
- `name` exige exactamente un `/`: `io.github.ness-e/vantadb` cumple.

## Steps

- [x] S1 — Fix `server.json`: description 101 → 87 chars (límite 100).
- [x] S2 — Gate offline `scripts/validate-mcp-server.py` (stdlib, sin red,
  espeja el schema: required, name pattern, description 1..100, version
  pineada, packages/remotes si presentes, fileSha256 para mcpb).
- [x] S3 — Workflow `publish-mcp-registry.yml`: `workflow_dispatch`
  (`validate-only` default), SHA pins del repo, timeouts, `contents: read`
  (+ `id-token: write` solo en publish), environment `mcp-registry`
  (approval owner). Sin triggers en tags.
- [ ] S4 — Orquestador: review + commit (PROXIMO_STEP).

## GAP honesto (no inventado)

El único canal viable hoy sería **mcpb** (URL a releases + `file_sha256`),
pero **no existe ningún `.mcpb` construible con evidencia actual**: el server
viaja dentro del binario `vantadb-server` (5 targets, custom allocator) y el
formato bundle (manifest + binarios empaquetados) no está implementado. El
job `publish` del workflow lo declara y falla en rojo hasta que exista.
Publish real = owner: (1) construir `.mcpb`, (2) `mcp-publisher login`
(first-claim `io.github.*`), (3) flip del job a publish.

## Deuda / pendiente (orquestador)

- Re-validar `server.json` con `jsonschema` contra el schema descargado ✅ HECHO por orquestador: **0 errores** (librería jsonschema 4.25.1).
- Coherencia `docs/dev/workflow/PUBLISH.md` ✅ HECHA: fila MCP Registry + § dedicada (manual dispatch, owner-gated, GAP .mcpb).
- `server.json` `version` 0.8.0 vs workspace: verificar sync en el release
  que publique el `.mcpb` (regla release-ci §4).

## Verificación

```powershell
python scripts/validate-mcp-server.py
python -c "import json,jsonschema; s=json.load(open('<schema-descargado>')); v=jsonschema.Draft7Validator(s); print(list(v.iter_errors(json.load(open('server.json')))))"
git diff --check
```

## Save Point

Trabajo en working tree sin commitear: `server.json` (1 línea),
`.github/workflows/publish-mcp-registry.yml` (nuevo),
`scripts/validate-mcp-server.py` (nuevo), `docs/dev/workflow/PUBLISH.md`
(fila + § MCP Registry), `docs/dev/tasks/FIND-344.md`
(nuevo). GATES: D no disparado (sin publish/tag/invención) · V/C pendientes
del orquestador (review + commit).

## Review (Gate P2-01)
- **Revisor:** vanta-review, sesión distinta (veredicto 2026-10-16, re-ejecución: gate offline, pins, yaml, diff scoped).
- **Veredicto:** ✅ APPROVE FIND-344 — GAP honesto (.mcpb inexistente, publish en rojo hasta que exista), sin `packages[]` inventados, OIDC mínimo, commit selectivo 5 paths.
- **Estado:** ✅ COMPLETED.
