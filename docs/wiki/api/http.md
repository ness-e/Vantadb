---
title: "API HTTP (`vantadb-server`)"
kind: reference
status: active
description: "Servidor HTTP con axum: endpoints verificados, auth/RBAC por namespace, maintenance, snapshots, skills y modo MCP."
---

# API HTTP (`vantadb-server`)

Audiencia: operadores del modo servidor. Referencia canónica: [HTTP_API](../../api/HTTP_API.md) + contrato `docs/api/openapi.yaml` (gate `tests/api/openapi_yaml_parity.rs`).

## Naturaleza

`[HECHO]` `vantadb-server` es el binario distribuido por GitHub Release ([`vantadb-server/src/lib.rs:1-12`](../../../vantadb-server/src/lib.rs#L1-L12)); la lógica HTTP vive en el módulo `server` del core tras la feature `server` (`src/lib.rs:145-146`) con el router en [`src/server/router.rs`](../../../src/server/router.rs) (`app` en `:120`, `app_with_cors` en `:131`). El scheduler del crate hostea el puente de conversación de `vanta-memory` vía `on_storage_ready` (ADR-0054, `vantadb-server/src/scheduler.rs`). El mismo binario sirve MCP con `--mcp` (ver [mcp](mcp.md)).

## Endpoints (verificados en HTTP_API.md)

`[HECHO]` Salud/métricas: `GET /health`, `GET /metrics`, `GET /api/v2/health`, `GET /api/v2/metrics`, `GET /api/v2/audit`. Registros: `POST /api/v2/records`, `POST /api/v2/records/batch`, `GET|DELETE /api/v2/records/{ns}/{key}` (+`/versions`), `DELETE /api/v2/records?namespace=&filter=`, `GET /api/v2/list`. Búsqueda: `POST /api/v2/search`, `GET /api/v2/autocomplete`, `POST /api/v2/query` (IQL). Portabilidad: `POST /api/v2/export`, `POST /api/v2/import`. Mantenimiento: `POST /api/v2/maintenance/{flushes,compactions,index-rebuilds}`, `DELETE /api/v2/maintenance/expired-records`. Snapshots: `GET|POST /api/v2/snapshots[/{name}]`. Dominios: threads CRUD, `POST /api/v2/conversations`, skills CRUD con `expected_version`, `POST /api/v2/mcp/proxy`, `GET /dashboard`.

## Auth y aislamiento

`[HECHO]` JWT HS256 offline (ADR-0039; `src/server/jwt.rs`, `src/server/middleware.rs`); RBAC por namespace con grants enforced por endpoint — tabla verificada en HTTP_API §RBAC (`src/rbac.rs`, tests `tests/rbac_namespace.rs`, `tests/server_auth_rotation.rs`). `POST /api/v2/search` es operación de lectura (grant `read`). TLS opcional tras la feature `tls` (axum-server+rustls, `Cargo.toml:194`).

## Semántica compartida

`[HECHO]` `score` mayor-es-mejor; la señal de abstinencia (`abstained`/`abstention_reason`) viaja en `SearchPageV2` — único transporte binding-level con página junto a MCP (BINDINGS_NAMESPACES §SCH-07). `[HECHO]` Los límites de recursos devuelven error explícito en `POST /records[batch]` e import JSONL; el bulk `.vdbdump` está exento en 0.9.0 (HTTP_API).

## Fuentes

`vantadb-server/` · `src/server/` · [HTTP_API](../../api/HTTP_API.md) · `docs/api/openapi.yaml` · [ERROR_HANDLING](../../api/ERROR_HANDLING.md) · ADR-0039/0054
