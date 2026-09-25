# Task API-STD-15 — Síntesis normativa + Gate P HITL

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Gate P log (ronda `question` 2026-09-24)

Primera ronda: usuario pidió investigación profunda por decisión antes de elegir. Segunda ronda con evidencia (Qdrant `score`, batch objetos, scroll-cursor; Stripe cursor; MCP spec; RFC 9457): **las 4 aprobadas con la recomendada** — score-todos, array-objetos, cursor+has_more, core-only memory.

## 2. Tabla normativa (EJE → DECISIÓN → ALCANCE → FUENTE → BREAKING)

| Eje | Decisión (Gate P donde 🔴) | Alcance 11 | Fuente | Breaking |
|---|---|---|---|---|
| Nombres/semántica search | `score` higher-is-better en search/híbrido; `distance` lower-is-better solo ANN crudo 🔴✅ | Rust/Py/TS/Node/WASM/HTTP/MCP | Qdrant docs + 5/6 superficies + `T1` | SÍ `feat!:` (TS) |
| Nombres graph | `graphBfs/graphDfs/…` + `roots: (number\|bigint)[]`; `graph_degree` canónico | TS/Node/WASM/Py | T2 + N5 | SÍ `feat!:` |
| `get/delete` | memory `namespace+key` en sub-cliente; nodo por id con nombre distinto (`getNode/deleteNode`) — Python renombra flat 🔴 (derivado, sin ronda: aplica glosario) | Py (+ resto ya ok) | P1 + glosario (ítem 6) | SÍ `feat!:` |
| Firmas batch | `put_batch([{…}])` array-objetos 🔴✅ | Py migra; resto ya | Qdrant upsert + 3/4 bindings | SÍ `feat!:` (Py) |
| Firmas search | objeto único `SearchRequest` + `method` inline; fuera `searchWithMethod` separado | Node alinea; TS ya; Py migra | ítem 4 + N4 | SÍ `feat!:` |
| Errores | `code+message+context`, sin panic; `to_js_err` como patrón; RFC 9457 en HTTP (`type/title/status/detail/instance`) | 11 | S7 + S2(9457) + W1 | SÍ `feat!:` |
| REST | plurales sin verbos; `/api/v2` total; `/threads/{id}/messages`; cursor opaco + `has_more` + `limit` 🔴✅; status=YAML | HTTP/proxy | S3/S4/S5 + H1-H4 | SÍ `feat!:` |
| OpenAPI | YAML owner + `openapi_yaml_parity` en CI; `RecordInput` opcional; case único; `next_cursor` 1 tipo | HTTP | S1 + H5 | SÍ `feat!:` |
| Casing | camelCase JSON/MCP; kebab CLI/tools-nuevos; nativo interior (snake Rust/Python, camel TS) | 11 | S7 + ítem 1-2 | SÍ `feat!:` |
| Tipos base | 1 schema → codegen; `u128→string` decimal (Node `:63-64` como modelo); ISO 8601 UTC (verificar ms-vs-string en ejecución) | 11 | S1 + N5 + ítem 5/19 | SÍ `feat!:` |
| Auth | `Authorization: Bearer` JWT + `x-api-key` partners; auth en `/snapshot` YA (seguridad) | HTTP/proxy/MCP | ítem 12 + X1 | SÍ `feat!:` |
| Rate-limit/cache/obs | rate-limit 1 punto + headers estándar; ETag/`304`; `X-Correlation-ID`; logs JSON | server/proxy | S7 + S4 + ítem 21/22/24 | Parcial |
| MCP | 1 nombre canónico (quitar alias doble); JSON Schema estricto (Schemars); `invalid_params` temprano; tipados; separar prompts/resources/tools | MCP | S7 + M1-M6 | SÍ `feat!:` |
| IQL | `IQL_VERSION` + 1 sintaxis documentada + case definido + `==`/int/string fixes + AST JSON | IQL+consumidores | Q1-Q5 + ítem 31 | SÍ `feat!:` |
| CLI | POSIX+Clap; flags simétricos; `--json` global completo; humano truncado solo TTY; `count` exit≠0; `put` vía SDK; lecturas RO | CLI | S8 + C1-C4 | SÍ `feat!:` |
| vanta-memory | core-only + API Rust estable; exponer post-release con demanda 🔴✅ | memory | D42/D43 + V1-V3 | NO (no expone) |
| IPC | in-process primero; TCP basta; socket solo con benchmark | 13 | S9 + P1/P2 | NO |
| Gateway/GraphQL | NO (descartados) | — | ítem 23 | — |

**Deuda a pagar (Regla 6):** P2-5 (`put_batch` dual) con la migración array-objetos; P2-8 con batch fundación.

## 3. DoD

- [x] Contrato ✅ (tabla 18 ejes, fuente por fila, breaking marcado) · Gate P log · Recitation: síntesis normativa lista

## Context Save Point

API-STD-15 DONE. Next: API-STD-16 (re-validación ~40 fallos). Gate P: 4/4 recomendadas aprobadas.
