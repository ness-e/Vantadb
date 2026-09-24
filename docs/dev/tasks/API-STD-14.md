# Task API-STD-14 — Web-checklist: 4 bloques ítem-por-ítem

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes — cascada keyless→webfetch; MetaSearch/Argus no disponibles en este entorno)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Convertir los 4 bloques del usuario en veredicto trazable por ítem (confirma/matiza/descarta/1-fuente).

## 2. Fuentes verificadas esta sesión (segunda capa donde fue posible)

- **S1** OpenAPI design-first + single-source + CI validators: https://learn.openapis.org/best-practices.html (+ https://www.openapis.org/what-is-openapi, https://swagger.io/specification/)
- **S2** RFC 7807 problem+json (`type/title/status/detail/instance`): https://www.rfc-editor.org/rfc/rfc7807.html — ⚠️ **obsoleta por RFC 9457**: https://www.rfc-editor.org/rfc/rfc9457.html
- **S3** Stripe cursor (`limit/starting_after/ending_before/has_more`, sin totales): https://docs.stripe.com/api/pagination
- **S4** GitHub (`Link` headers, `per_page≤100`, ETag/`304`, sort estable): https://docs.github.com/en/rest/using-the-rest-api/using-pagination-in-the-rest-api (+ best-practices)
- **S5** cursor-vs-offset (has_more, `next_cursor:null`, 400 cursor inválido, no mezclar estilos): https://apidog.com/blog/cursor-vs-offset-pagination/
- **S6** PyO3 (cdylib, `PyResult`, maturin): https://pyo3.rs/
- **S7** MCP tools (`name` 1-128, `A-Za-z0-9_-.`, case-sensitive, único; `inputSchema` JSON Schema obligatorio; `outputSchema`; `isError`; orden determinista; server MUST validar inputs + rate-limit; human-in-loop SHOULD): https://modelcontextprotocol.io/docs/concepts/tools
- **S8** Clap 4 derive (`#[derive(Parser)]`, `-n/--name`, semver, WG-CLI): https://docs.rs/clap
- **S9** NAPI-RS (`#[napi]`, `.node` por target, Node-API ABI): https://napi.rs/

## 3. Tabla veredicto (ítem | veredicto | fuentes | aplica VantaDB)

| # | Ítem del bloque usuario | Veredicto | Fuentes | Aplica |
|---|---|---|---|---|
| 1 | Nativo dentro de cada lenguaje (snake Rust, camel TS), estándar en fronteras | **CONFIRMA** | S7 (case-sensitive, charset) + S8/S9 (convenciones propias) + código (`removeEdge` camel, `graph_bfs` snake) | SÍ — regla frontera-vs-interior |
| 2 | JSON frontera en camelCase; CLI/MCP en kebab/snake | **CONFIRMA parcial** | S7 (nombres admiten `-_.`, ejemplos `getUser`, `admin.tools.list`) + S4 (query params) | SÍ con matiz: MCP admite camel y kebab; fijar 1 por superficie |
| 3 | `js_name`/traducción automática PyO3/NAPI | **1-FUENTE** (S9 confirma `#[napi]` pero no `js_name`; S6 confirma PyO3) | S6 + S9 + código | SÍ — verificar `js_name` en ejecución |
| 4 | Options-object / Builder / kwargs `*`, no posicionales largos | **CONFIRMA** (doctrina) | S7 (schemas objeto con required) + S5 (envelope limpio) | SÍ — `put_batch`/`search` caso testigo |
| 5 | Tipos base + codegen single-schema (JSON Schema/Protobuf) | **CONFIRMA** | S1 (single source of truth, no duplicar) + S7 (JSON Schema 2020-12) | SÍ — 1 schema → Rust/Python/TS |
| 6 | Glosario verbos get(fetch pesado)/create/delete/update | **1-FUENTE** | bloque + código (get/fetch mezclados hoy) | SÍ — glosario propio, verificar uso industria en ejecución |
| 7 | `Result→excepción` sin panic, code+message+context | **CONFIRMA** | S7 (protocol errors JSON-RPC vs execution `isError`, actionable retry) + S6 (`PyResult`) + código (`to_js_err`) | SÍ — `to_js_err` como patrón |
| 8 | URLs plurales, no verbos | **CONFIRMA** | S1 (diseño describible) + S3/S4 (recursos) | SÍ — H1 testigo |
| 9 | kebab vs snake en URLs: elegir 1 | **CONFIRMA** | S7 (charset, no espacios) | SÍ — kebab (recomendado bloques) |
| 10 | JSON `application/json` везде | **CONFIRMA** | S2 (media types) + S7 | SÍ |
| 11 | Error envelope idéntico con code+message+details | **CONFIRMA** | S2 (`type/title/status/detail/instance` + extensiones) | SÍ — RFC **9457** (no 7807, obsoleta) |
| 12 | OAuth2/OIDC, JWT RS256, `Authorization: Bearer`, `x-api-key` partners | **1-FUENTE** | bloque + código (auth headers proxy `server.rs:297`) | SÍ con verificación pendiente en ejecución |
| 13 | Verbos HTTP correctos | **CONFIRMA** | S3/S4 (GET listas, POST creación) | SÍ |
| 14 | Versionado en URL `/v2` | **CONFIRMA** | S3 (`/v2` namespace distinto), S4 (`X-GitHub-Api-Version`) | SÍ — `/api/v2` total |
| 15 | Códigos nativos (200/201/400/401/403/404/429/500) | **CONFIRMA** | S2 (status MUST = response) + S5 (400 cursor inválido) | SÍ — H3 testigo |
| 16 | `?page&limit&sort=-price`, filtros | **MATIZA** | S3 (cursor `starting_after`, NO page numbers) + S4 (`per_page`, `Link`) + S5 (offset solo admin interno) | Cursor opaco + `limit` + `has_more` (NO `?page=` público) |
| 17 | OpenAPI viva / Swagger, API-first | **CONFIRMA** | S1 (design-first "cannot be stressed strongly enough", validators en CI) | SÍ — YAML owner + parity en CI |
| 18 | RFC 7807 | **MATIZA** | S2 → obsoleta por **RFC 9457** | SÍ pero 9457 |
| 19 | ISO 8601 UTC `…Z` | **1-FUENTE** | bloque + código (`created_at_ms` u64 — decidir: ms vs string ISO) | SÍ — verificar en ejecución |
| 20 | CORS estricto, TLS 1.3 | **1-FUENTE** | bloque + código (`tower-http` cors por auditar) | SÍ — auditar `Allow-Origin` en 16 |
| 21 | Rate-limit 100/min + `X-RateLimit-*/Retry-After/429` | **CONFIRMA parcial** | S7 (servers MUST rate limit) + S4 (best-practices rate) | SÍ — formato headers a verificar en ejecución |
| 22 | ETag/`304`/`Cache-Control`/`Idempotency-Key` | **CONFIRMA parcial** | S4 (ETag/`304` explícito) | SÍ ETag/304; idempotencia a verificar |
| 23 | API Gateway (Kong/AWS/Apigee/Traefik) | **DESCARTA** | S1–S9 (nada para embebido 1-proceso) | NO — monolito local, over-engineering |
| 24 | Correlation-id + logs JSON | **1-FUENTE** | bloque + código (`request_id` test existe: `tests/request_id.rs`) | SÍ — `X-Correlation-ID` barato |
| 25 | Zero-copy Arrow/Protobuf vs JSON local | **MATIZA** | S6 (pyo3-arrow existe en ecosistema) | Solo vectores hot-path Y con benchmark Regla 9; JSON basta resto |
| 26 | PyO3 / NAPI-RS (no WASM en servidor) | **CONFIRMA** | S6 + S9 + código (`native.ts` ya usa napi) | SÍ — NAPI server, WASM browser |
| 27 | FFI `unsafe` encapsulado | **CONFIRMA** (doctrina) | S6 (guía) + Regla 4 (`// SAFETY:`) | SÍ — ya es regla |
| 28 | MCP prompts/resources/tools + Pydantic/Zod/Schemars | **CONFIRMA** | S7 (3 primitivas en spec, schemas estrictos) | SÍ — separar + Schemars |
| 29 | CLI POSIX + Clap/Typer/Commander + `--json` | **CONFIRMA** | S8 (Clap estándar) | SÍ — Clap ya en uso, `--json` global |
| 30 | Single-ownership Rust + Unix-socket/IPC vs TCP | **MATIZA** | S9 (in-process primero) + código (sin IPC hoy) | Ownership SÍ; socket solo con benchmark |
| 31 | IQL AST JSON a Python/TS | **1-FUENTE** | bloque + código (parser nom central existe) | SÍ — exponer AST plano |

**Descartado neto:** gateway (23). **Matizados:** 16 (cursor, no page), 18 (9457), 25/30 (medir antes). **1-fuente (re-verificar en ejecución):** 3, 6, 12, 19, 20, 24, 31.

## 4. DoD

- [x] Contrato ✅ (31 filas, 9 fuentes citadas, veredicto por ítem) · Task file sync · Recitation: web-checklist completo

## Context Save Point

API-STD-14 DONE. Next: API-STD-15 (síntesis + Gate P). Deuda: 7 ítems 1-fuente. WIP: ninguno.
