# Task API-STD-07 — INDIVIDUAL (6/11) HTTP server + OpenAPI

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual red: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Servidor Axum (feature `server`): router (`router.rs:149-351`) → handlers (`handlers.rs`) → core SDK. Rutas `/api/v2/*` + legacy sin versionar. Consumen: CLI (`cmd_server`), MCP (vía hijo), agentes externos. YAML (`docs/api/openapi.yaml`) deriva de la impl (no API-first).

## 3. Uso (ejemplos mínimos)

```bash
curl -X POST localhost:8096/api/v2/records -H 'Content-Type: application/json' \
  -d '{"namespace":"docs","key":"a","payload":"hello"}'
curl 'localhost:8096/api/v2/search?namespace=docs&top_k=5' -X POST -d '{...}'
```

## 4. Código (re-verificado 2026-09-24, grep directo)

- **H1 Verbos en URL CONFIRMADO:** `POST /api/v2/maintenance/purge|compact|flush` (`:197-205`), `/rebuild-index` (`:247`), `/export|/import` (`:244-245`), `POST /conversation/add` + `GET /skill/listing` (`:218-219`), `POST /threads/{id}` mensaje (`:213`).
- **H2 Versionado parcial CONFIRMADO + MATIZADO:** `/health` (`:149`) y `/metrics` (`:232`) sin versionar PERO con gemelos `/api/v2/health` (`:155`), `/api/v2/metrics` (`:233`). Sin gemelo: `/conversation/add`, `/skill/listing`, `/dashboard` (`:304-348`, público loopback D12).
- **H3 `201` vs YAML `200` CONFIRMADO:** `CREATED` en 6 sitios (`handlers.rs:252,263,1213,1322,1433,1521`) vs YAML `"200"` (`:275,347,1203,1344`).
- **H4 Paginación mixta CONFIRMADO** (auditoría previa): offset (`:1129-1138`) vs cursor (`:375-382`), `versions` solo `?version` vs YAML `?limit`.
- **H5 YAML drifts CONFIRMADOS** (auditoría previa, para re-chequeo en 16): `RecordInput` required-6 (`:1721-1738`) vs `Option` core; ejemplo Lisp (`:1727`); enum case (`:1815-1818`); `next_cursor string|null` (`:1933-1955`); tag `Skills` sin declarar (`:36-50` vs `:1406+`).
- Parity test existe: `tests/api/openapi_yaml_parity.rs` (contrato 16: correrlo y anotar).

## 5. Veredicto + implicaciones

5/5 confirmados (H2 matizado: hay gemelos v2 parciales — migrar resto, no crear de cero). Propuestas 15: recursos plurales sin verbos, `/api/v2` total, unificar status+YAML (owner YAML), sub-recurso `/threads/{id}/messages`, cursor único, `RecordInput` opcional. Blast radius: router+handlers+YAML+tests parity+MCP local.

## 6. DoD

- [x] Contrato ✅ (grep + citas) · Task file sync · Recitation: HTTP ficha completa

## Context Save Point

API-STD-07 DONE. Next: API-STD-08 (MCP). Deuda: ninguna. WIP: ninguno.
