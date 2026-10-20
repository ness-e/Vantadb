---
title: "Deep audit W3 - Server and MCP"
kind: review
status: active
description: "Auditoria read-only del servidor HTTP (axum) y del MCP server: paridad OpenAPI, auth/RBAC, SSE/rate limit/health, y cobertura de la skill de usuario frente a las 49 tools MCP."
tags: [audit, reviews, server-mcp]
---

# Deep audit W3 - Server + MCP

> **Wave:** W3 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W3)
> **Ejecucion:** subagentes `vanta-audit` + `vanta-review` no disponibles (sin tool `task`).

## Scope

**Cubierto:**

| Area | Ruta | Detalle medido |
|---|---|---|
| HTTP server (core) | `src/server/{router,routing,handlers,middleware,jwt,bootstrap,state,errors,pagination,conversation,self_healing,telemetry,list_records}.rs` | 17 ficheros |
| HTTP server (crate) | `vantadb-server/src/{lib,main,server,scheduler}.rs` | 4 ficheros, 340 lineas |
| MCP server | `vantadb-mcp/src/**` (25 `.rs`) | 24 353 lineas, 49 tools declaradas |
| OpenAPI | `docs/api/openapi.yaml` + `scripts/check_openapi_parity.mjs` | - |
| Skill de usuario MCP | `skills/vantadb-mcp/SKILL.md` | 24 ficheros en `skills/vantadb-mcp/` |

**Excluido con motivo:** `.vantadb-mcp/` (directorio de datos local, gitignored en `.gitignore:205`,
no es codigo); `vanta-proxy` (W2).

## Metodologia

- Ejecucion del gate de paridad: `node scripts/check_openapi_parity.mjs` (exit 0).
- Lectura de `src/server/router.rs` (412 lineas) y `src/server/bootstrap.rs` para el modelo de auth.
- Conteo y extraccion de la lista de tools MCP desde `vantadb-mcp/src/handlers/tools.rs:82`
  (`handle_tools_list`) y el mapa de alias en `tools.rs:1319`.
- Diff de la lista de tools MCP contra las mentioned en `skills/vantadb-mcp/SKILL.md`.
- Sin ejecucion de servidores (no se levanto el binario).

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-408` | High | **2 de las 49 tools MCP no son alcanzables desde la skill de usuario** | `vantadb-mcp/src/handlers/tools.rs:82` (lista de tools), `skills/vantadb-mcp/SKILL.md` | `handle_tools_list` declara **49 tools unicas**. Comparacion con el texto de `skills/vantadb-mcp/SKILL.md`: 47 documentadas. Faltan **`memory_reinforce`** y **`memory_verify_certificate`**. La segunda es la superficie de gobernanza/atestacion (verificacion de certificados de borrado certificado) - un agente que solo sigue la skill **no puede** verificar un certificado de borrado, que es justo el caso de uso de confianza. El alias legacy `collection_list` si esta correctamente declarado y mapeado (`tools.rs:1319` -> `memory_list_namespaces`), asi que no es un falso positivo de este patron. | Anadir las 2 tools a `skills/vantadb-mcp/SKILL.md` con su esquema de argumentos, y anadir un gate en `dev-tools/validate_doc_snippets.py` o en `scripts/docs/check-docs.mjs` que compare la lista de tools de `handle_tools_list` con la skill y falle si difieren. | 2-4h |
| `W3-02` | Info | Paridad OpenAPI: **VERDE y exacto** | `docs/api/openapi.yaml`, `scripts/check_openapi_parity.mjs`, `src/server/router.rs` | `node scripts/check_openapi_parity.mjs` -> "Router (src/server/router.rs): 37 paths, 44 operations (33 paths under /api/v2/*) / OpenAPI (docs/api/openapi.yaml): 37 paths / Parity OK: openapi.yaml matches the registered router exactly." exit 0. El gate esta cableado en `docs-gates.yml`. Ademas existe un test de paridad en Rust: `tests/api/openapi_yaml_parity.rs` (target `openapi_yaml_parity`, `Cargo.toml:451-452`). Doble verificacion. | Sin accion. | - |
| `W3-03` | Info | Modelo de auth solido: fail-closed, con的要求 de loopback y proteccion de key rotation | `src/server/bootstrap.rs:40-93`, `src/server/router.rs:163-164` | `bootstrap.rs:42` rechaza `alt_api_key` sin `api_key` primaria. `bootstrap.rs:58` rechaza `require_auth` sin `api_key` ni `jwt_secret`. `bootstrap.rs:75-83` rechaza enlazar a host no-loopback sin api_key (con mensaje accionable). `router.rs:113` permite el header `AUTHORIZATION` en CORS. `router.rs:294` calcula el burst del rate limiter en modo conservador cuando hay auth activa vs dev. `SECURITY.md` declara comparacion constant-time con `subtle::ConstantTimeEq`. | Sin accion. | - |
| `W3-04` | Info | Rate limit con categoria `INFORMATIONAL` y tests propios | `src/server/router.rs:29-54`, `src/server/router.rs:290-309` | `rate_limit_period_ms` (rpm -> periodo), `rate_limit_burst` (burst segun auth activa), `rate_limit_error_response` (handler de `GovernorError`). Tests inline: `rate_limit_period_ms_is_sane` (linea 391), `rate_limit_burst_dev_mode_full`, `rate_limit_burst_auth_mode_conservative`, `cors_layer_none_when_empty`. | Sin accion. | - |
| `W3-05` | Info | El MCP expone un modo proxy 1:1 contra el writer HTTP, con fallback local | `vantadb-mcp/src/server.rs:110-175`, `vantadb-mcp/src/server.rs:244`, `vantadb-mcp/src/server.rs:658-679` | `dispatch_request_proxy` reenvia `tools/call` al writer vivo via proxy y usa `tools/list` local; `dispatch_request` (sin proxy) ejecuta localmente contra `StorageEngine`. `server.rs:174` documenta "the full `app_with_cors` router for proxy parity (1:1 tools via HTTP)". Dos modos con la misma superficie de tools. | Sin accion. | - |
| `W3-06` | Low | `docs/api/MCP.md` no se pudo validar por conteo automatico de tools | `docs/api/MCP.md` | El gate `check_openapi_parity.mjs` cubre el HTTP server pero **no** existe un gate equivalente que compare `handle_tools_list` contra `docs/api/MCP.md`. La deriva real detectada en W3-01 esta precisamente en el canal de la skill, no del doc. | Anadir `scripts/check_mcp_parity.mjs` con el mismo patron que `check_openapi_parity.mjs`: extraer la lista de tools de `tools.rs` y contrastarla con `docs/api/MCP.md` + `skills/vantadb-mcp/SKILL.md`. | 2-4h |
| `W3-07` | Info | MCP: 25 modulos con separacion por dominio (axioms, code, code_index, context, dreams, governance, metrics, proxy, scenes, skills, temporal, threads, validation, wiki) | `vantadb-mcp/src/` | 24 ficheros en `vantadb-mcp/src/` + 4 en `vantadb-mcp/src/handlers/`. Dominios separados y `handlers/{initialize,prompts,resources,tools}.rs` para el protocolo MCP. 22 ficheros de test en `vantadb-mcp/tests/`. | Sin accion. | - |

## Veredicto

**PASS CON OBSERVACIONES**

El server HTTP y el MCP son solids. La paridad OpenAPI esta **exacta y verificada por doble via**
(gate de Node + test de Rust), el modelo de autenticacion es *fail-closed* con guardas explicitas
(key rotation, `require_auth` sin credenciales, binding no-loopback sin api_key) y el rate limit tiene
tests propios. El unico hallazgo High es de **alcanzabilidad de la superficie MCP**: dos tools -
incluida `memory_verify_certificate`, que es la via de verificacion de borrado certificado - existen en
el servidor pero no en la skill que leen los agentes de usuario, de modo que la superficie real (49
tools) y la documentada (47) han divergido sin que ningun gate lo detecte. No se encontro ningun
problema de seguridad en el path de auth.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `W3-01` | High | `vantadb-mcp/src/handlers/tools.rs:82` |
| `W3-02` | Info | `docs/api/openapi.yaml:1` |
| `W3-03` | Info | `src/server/bootstrap.rs:42` |
| `W3-04` | Info | `src/server/router.rs:29` |
| `W3-05` | Info | `vantadb-mcp/src/server.rs:658` |
| `W3-06` | Low | `docs/api/MCP.md:1` |
| `W3-07` | Info | `vantadb-mcp/src/handlers/tools.rs:1` |
