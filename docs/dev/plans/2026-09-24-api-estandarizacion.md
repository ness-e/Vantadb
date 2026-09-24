# Plan de Ejecución: Estandarización 11 APIs VantaDB — Investigación + Síntesis + Validación

> **Inicio:** 2026-09-24
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** Conversación 2026-09-24 (auditoría 11 superficies) + fallos verificados ses_f2b6280b* — no parte de `docs/dev/Backlog.md` (ad-hoc informativo → normativo)
> **Autonomous:** false
> **Modo:** PLAN (read-only, sin cambios de código — solo investiga, sintetiza y valida)
> **Contexto libre:** Nadie usa el proyecto ni los paquetes — breaking changes ilimitados permitidos, dejando todo funcional para el siguiente release. MCP local (`vanta-cli server --mcp` en opencode) debe actualizarse si hay cambios.
> **SDP:** coordinated-web-search, api-design-principles, codebase-memory, systematic-debugging, writing-plans, progreso (fase PLAN)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 8 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 5 (router web-cascade, paridad OpenAPI real, AST bindings, IQL_VERSION, MCP local refresh) · ⬇️ downhill = 8 tasks con contrato mecánico definido

Fases: F0 inventario → F1 investigación profunda 11 superficies (funcionamiento+uso+código) → F2 estándares industria vía web-cascade → F3 síntesis qué estandarizar → F4 re-validación adversarial + plan de implementación.

## Tasks

### Task 1: API-STD-01 — Inventario funcional 11 superficies + mapa conjunto vs individual

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** `src/lib.rs:9-53`, `Cargo.toml:744-765` (workspace members), `docs/api/` (19 files), `docs/api/VERSIONING.md:30-50`, `docs/api/BINDINGS_NAMESPACES.md:19-28`, `vantadb-python/`, `vantadb-ts/`, `vantadb-node/`, `vantadb-wasm/`, `vantadb-server/`, `vantadb-mcp/`, `vanta-memory/`, `vanta-proxy/`, `src/parser/`, `src/bin/vanta-cli.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `src/lib.rs:19` Embedded existe; workspace 7 members `Cargo.toml:744-752`; `docs/api/` 19 ficheros verificados por glob; `VERSIONING.md:35-38` contrato 4 superficies; `BINDINGS_NAMESPACES.md:79,134,188` conteos 47/43/44.
- **Gate Justificación:** Sin inventario no hay plan serio — fija las 11 (7 SDK/API + IQL + CLI + proxy + vanta-memory) y el funcionamiento conjunto (core Rust como single-owner, bindings como vistas) vs individual.
- **Gate Result:** ✅ DO
- **Contrato:** `ls docs/api/*.md | Measure-Object Count -eq 19` Y `cargo metadata --format-version 1 | grep -c vantadb-(python|server|mcp|wasm)` >= 4 Y plan file lista las 11 con entry-point + docs + funcionamiento en 1 línea cada una
- **Task file:** `docs/dev/tasks/API-STD-01.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio — inventario mecánico, causa-efecto claro
- **Top 3 riesgos:**
  1. Ruta renombrada (ej. `.agents/` eliminado) contamina inventario
  2. Confundir feature (GraphRAG/embeddings) con superficie API
  3. Docs a la deriva se toman como verdad sin verificar código
- **Pre-mortem:**
  - **Fallo 1:** se cuenta `providers/` o `integrations/` como API y el plan se hincha
  - **Fallo 2:** se omite `vantadb-node` por creer que es parte de `vantadb-ts`
  - **Fallo 3:** inventario solo-docs sin entry-points de código → F1 parte ciega
- **Stop conditions:** appetite >1d → abortar a DEFER; premisa invalidada (superficie no existe) → re-triaje; budget tool-calls agotado → registrar Notas
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | Superficie fantasma en docs | verificar cada una con glob+read | 1 iteración sin green |
  | 🟢×🟡 | `.opencode/` repo separado contamina paths | resolver solo workspace VantaDB | diff toca `.opencode/` |
  | 🟢×🟢 | Conteo docs/api cambia (19→N) | re-glob en VERIFY | contrato falla |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Entregable: tabla 11 filas (superficie | entry-point código | doc canónica | funcionamiento 1 línea | individual vs conjunto).
  - Registrar hallazgos nuevos vía `prompts/findings.md` (FIND-*).

### Task 2: API-STD-02 — Investigación profunda bindings (Python + TS + Node + WASM): funcionamiento, uso y código

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🔴
- **Archivos clave:** `vantadb-python/src/lib.rs:538-540,648-649,701-712,1017-1018,1241-1255,1379,1473-1498,1563-1577`, `vantadb-python/src/types.rs:329`, `vantadb-ts/src/vantadb.ts:423,451,482-502,601-611,638,714,822-926,1135-1454`, `vantadb-ts/src/types.ts:124-147`, `vantadb-wasm/src/lib.rs:585-1035,1157,1202-1262,1383-1896`, `vantadb-node/src/lib.rs:104-120,136-149,403-613`, `vantadb-node/index.d.ts:44-447`, `docs/api/BINDINGS_NAMESPACES.md`, `docs/api/PYTHON_SDK.md`, `docs/api/TS_SDK.md`, `docs/api/NODE_SDK.md`, `docs/api/WASM_API.md`
- **Verificación real:** ✅ CÓDIGO-REAL — fallos ya verificados: `get/delete` semántica opuesta (`lib.rs:1563` vs `vantadb.ts:482`); `score→distance` sin invertir valor (`vantadb.ts:605-611` vs `types.ts:143-147`); `put_batch` columnar vs array (`lib.rs:701` vs `:1157` TS/WASM); TS traversals `number[]` (`:1327-1331`) vs `number|bigint` insert (`:1135-1139`); `importRecords` bucle JS (`:900-926`); Node sin export/import (`index.d.ts:243-447`); doc drift `supersede` (`BINDINGS_NAMESPACES.md:215` vs `lib.rs:1445` WASM).
- **Gate Justificación:** Grupo más roto (15 hallazgos) — requiere análisis funcionamiento (cómo se usa cada binding hoy) + código (firmas reales) + blast radius antes de proponer unificación.
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "fn (get|delete|search|put_batch|search_multi|bulk_import)" vantadb-python/src/lib.rs vantadb-wasm/src/lib.rs` documentado en task file Y matriz 4 bindings × 20 métodos con firma real Y cada fallo del listado con repro/quote `file:line`
- **Task file:** `docs/dev/tasks/API-STD-02.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — requiere experto bindings (PyO3/NAPI/wasm-bindgen) para decidir forma canónica
- **Top 3 riesgos:**
  1. Proponer firma canónica sin medir blast radius (callers externos inexistentes pero tests sí)
  2. `u128` >2^53 rompe JSON en 3 bindings a la vez
  3. Alias legacy con usuarios cero pero tests que los fijan
- **Pre-mortem:**
  - **Fallo 1:** se asume JSON como frontera cuando Arrow/zero-copy sería mejor para vectores
  - **Fallo 2:** se unifica nombre pero no semántica (el bug `score/distance` se re-etiqueta, no se arregla)
  - **Fallo 3:** Node se deja fuera por "experimental" y la matriz queda coja
- **Stop conditions:** appetite >3d → scope a solo matriz+repro, síntesis a API-STD-06; rabbit hole FFI → abortar y registrar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Cambio firma rompe `tests/api/python.rs` + `sdk_serialization` | mapear tests antes de proponer | 2 iteraciones sin green |
  | 🟡×🟡 | `u128` como number en TS | proponer string canónico + test >2^53 | repro falla |
  | 🟢×🔴 | PyO3 panic en frontera | auditar `expect/unwrap` en bindings | clippy deny |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Por superficie: qué hace, cómo se usa (ejemplo mínimo real del README), cómo está implementado (binding tool: PyO3/NAPI/wasm-bindgen), qué está roto.
  - Delegar DISCOVERY pesado a `vanta-research` si se necesita (digest ≤500 palabras + RESULTADO).

### Task 3: API-STD-03 — Investigación profunda red (HTTP server + OpenAPI + MCP + proxy): funcionamiento, uso y código

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🔴
- **Archivos clave:** `src/server/router.rs:149-351`, `src/server/handlers.rs:252-1322`, `docs/api/openapi.yaml:36-1935`, `docs/api/HTTP_API.md`, `tests/api/openapi_yaml_parity.rs:90-144`, `vantadb-mcp/src/handlers/tools.rs:101-1939`, `vantadb-mcp/src/validation.rs:477-489`, `vantadb-mcp/src/error.rs:68-95`, `docs/api/MCP.md`, `vanta-proxy/src/server.rs:112-117,297-300,741-840`, `vanta-proxy/src/config.rs:12-272`, `docs/api/PROXY.md`, `.opencode/rules/server-mcp.md`
- **Verificación real:** ✅ CÓDIGO-REAL — verbos en URL (`router.rs:197-207,218-219`); fuera de `/api/v2` (`:149,232-233` vs `VERSIONING.md:37`); `201` vs YAML `200` (`handlers.rs:252,263` vs `openapi.yaml:275,347`); paginación mixta (`handlers.rs:1129-1138` offset vs `:375-382` cursor); `RecordInput` sobre-requerido (`openapi.yaml:1721-1738` vs `record.rs:35-54`); alias MCP doble (`tools.rs:309` vs `:373`); `GET /snapshot` sin auth (`server.rs:744,816-840`); self-loop default (`config.rs:272`).
- **Gate Justificación:** Segunda fuente mayor de deuda — define si OpenAPI pasa a API-first y si MCP/proxy convergen al mismo error/paginación/auth.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb --test openapi_yaml_parity` estado documentado Y tabla ruta-a-ruta (router vs YAML vs handler status) Y cada fallo red con `file:line` + request/response real
- **Task file:** `docs/dev/tasks/API-STD-03.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — OpenAPI vs impl, auth, rate-limit requieren decisión experta
- **Top 3 riesgos:**
  1. YAML y router derivan a la vez y nadie es fuente de verdad
  2. Auth (`/snapshot`) se clasifica como feature y se difiere
  3. MCP `bulk_import_stream` bypasea validación por diseño y se normaliza sin querer
- **Pre-mortem:**
  - **Fallo 1:** se propone gateway centralizado para un monolito embebido (over-engineering)
  - **Fallo 2:** se estandariza paginación cursor sin migrar `threads/audit` que usan offset
  - **Fallo 3:** `spaceId` camelCase se cambia sin grep de clientes
- **Stop conditions:** appetite >3d → partir en HTTP vs MCP; premisa invalidada (ruta no existe) → SKIP parcial
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Cambio auth rompe MCP local | test MCP local antes/después | MCP no responde |
  | 🟡×🟡 | Rate-limit `tower_governor` vs proxy `parsed but unused` | unificar en un solo punto | 2 fuentes divergen |
  | 🟢×🟡 | CORS `*` en prod | auditar `tower-http` cors config | grep `Allow-Origin` |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Por superficie: funcionamiento (quién la consume: agentes/CLI/LLM), uso (ejemplo curl/tool-call real), código (Axum router → handler → SDK).

### Task 4: API-STD-04 — Investigación profunda core (Rust SDK + IQL + CLI + vanta-memory): funcionamiento, uso y código

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** `src/sdk/api/memory.rs:29-44`, `src/sdk/types/record.rs:13-114`, `src/sdk/types/graph.rs:18-29`, `src/error.rs:184-287`, `src/binary_header.rs:20`, `src/lib.rs:167`, `docs/api/EMBEDDED_SDK.md`, `src/parser/grammar.rs:47-56,92-121,349-439`, `src/parser/lexer.rs:41-144`, `src/parser/mod.rs:170-225,683-1225`, `src/sdk/api.rs:328-331`, `docs/api/IQL.md`, `src/cli.rs:43-321`, `src/cli_handlers/crud.rs:53-553`, `src/cli_handlers/search.rs:21-344`, `src/cli_handlers/server.rs:21-349`, `vanta-memory/src/adapters/standalone/llm_runner.rs:104-112`, `vanta-memory/src/lib.rs:15-18`, `docs/api/VANTA_MEMORY.md:20-104`, `.opencode/rules/core-engine.md`, `.opencode/rules/query-dsl.md`
- **Verificación real:** ✅ CÓDIGO-REAL — stutter `VantaHeader` (`binary_header.rs:20`); `Generic(ChainedError)` (`error.rs:275-276`); `node_id` sin serde (`graph.rs:18-24`); IQL 6 docs vs 7 parser (`IQL.md:16-26` vs `grammar.rs:349`); bug `==` (`grammar.rs:47-56` + test `:170-176`); int→Float (`lexer.rs:138-144`); CLI `--in` vs `--out` (`cli.rs:123-124,109-110`); cajas `╭` + truncado (`crud.rs:178-235`, `search.rs:140-142`); `cmd_put` bypass SDK (`crud.rs:53-55`); L0-L3 core-only (`BINDINGS_NAMESPACES.md:250-259`).
- **Gate Justificación:** Core es single-owner del estado — si su error/IQL/CLI no se fijan, los bindings heredan el drift.
- **Gate Result:** ✅ DO
- **Contrato:** `rg -n "IQL_VERSION|language version" src/ vanta-memory/` (=0 hoy) documentado Y tabla IQL statement-a-statement (docs vs parser vs test) Y CLI flag-a-flag (nombre + salida json/humano) Y cada fallo core con `file:line`
- **Task file:** `docs/dev/tasks/API-STD-04.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — IQL sin versión + parser nom: solo emerge probando; probe-sense-respond, steps cortos
- **Top 3 riesgos:**
  1. Fijar IQL case-insensitive rompe queries que usan minúsculas como alias
  2. `--json` global rompe scripts que parsean cajas humanas
  3. Exponer `vanta-memory` en bindings requiere nuevo Rust binding (D42) — scope explosion
- **Pre-mortem:**
  - **Fallo 1:** se propone Protobuf/Arrow para CLI humano (frontera equivocada)
  - **Fallo 2:** se elimina `FROM` o `MATCH` y se rompen tests `mod.rs:1106-1225`
  - **Fallo 3:** `TokenEstimator chars/3` se fija sin benchmark (Regla 9)
- **Stop conditions:** appetite >3d → IQL a DEFER con flag; rabbit hole nom-combinators → abortar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | `parse_literal` int→float pierde >2^53 | test `Int` inalcanzable primero | repro falla |
  | 🟡×🟡 | Single-quote vs double-quote | decidir 1 + error claro | `api.rs:328-331` |
  | 🟢×🔴 | RW en lecturas CLI (`ensure_indexes_current`) | auditar lock order (Regla 8) | deadlock |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Incluir `docs/dev/architecture/adr/041_anti_stutter.md:43-64` (stutter) y Regla 4 (dual API `put_batch` P2-5).

### Task 5: API-STD-05 — Web research industria (cascada obligatoria, multi-fuente verificada)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/skills/coordinated-web-search/SKILL.md`, `.opencode/skills/api-design-principles/` (assets `api-design-checklist.md`, references `rest-best-practices.md`, `graphql-schema-design.md`)
- **Verificación real:** 🟡 VERIFICAR — en DISCOVERY ejecutar router cascada: 1) keyless (agent-search/webfetch/Jina) → 2) Argus (`argus_search_web`, `extract_content`) → 3) MetaSearchMCP (`compare_engines`) → 4) fallbacks (recover_url→Jina→Playwright). Citar URLs fuente siempre (Regla AGENTS.md:14).
- **Gate Justificación:** El usuario aportó 4 bloques web sin URLs — hay que verificarlos contra fuentes oficiales (OpenAPI OAS, OAuth2/OIDC, RFC 7807, ISO 8601, MCP spec Anthropic, PyO3/NAPI-RS, Clap) y descubrir qué más estandarizar (casing fronteras, Options-object, glosario verbos get/fetch/create, Result→excepción con code+message+context, rate-limit headers, ETag/idempotency-key, correlation-id, logs JSON).
- **Gate Result:** ✅ DO
- **Contrato:** task file cita ≥12 URLs oficiales (≥2 capas por hecho: keyless+argus o compare_engines) Y tabla estándar→fuente→aplica-a-VantaDB(sí/no/por qué) Y marca qué del aporte del usuario se confirma, se matiza o se descarta
- **Task file:** `docs/dev/tasks/API-STD-05.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — sintetizar 4 bloques + oficiales requiere criterio, pero no experimentación
- **Top 3 riesgos:**
  1. Un solo motor (sesgo) se toma como verdad
  2. Se recomienda gateway (Kong/AWS) para embebido local (no aplica)
  3. GraphQL se propone donde REST/MCP ya cubren (scope creep)
- **Pre-mortem:**
  - **Fallo 1:** MetaSearchMCP wedgea (timeouts todos engines) y se inventan resultados en vez de restart
  - **Fallo 2:** se cita blog SEO como estándar en vez de RFC/spec oficial
  - **Fallo 3:** casing JSON (camelCase vs snake_case) se decide por gusto sin mirar frontera MCP/LLM
- **Stop conditions:** todo-falla-red → solo webfetch+Playwright; si ni eso, registrar y DEFER; nunca inventar URLs
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | MCP wedgeado | restart opencode, seguir keyless | timeout todos engines |
  | 🟡×🟢 | Paywall/docs JS | Jina → Playwright real | extract falla |
  | 🟢×🟡 | Fuentes no oficiales | exigir spec/RFC/GitHub | duda → validar |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Queries mínimas: `OpenAPI 3.1 API-first`, `RFC 7807 problem+json`, `ISO 8601 UTC`, `MCP Anthropic tools/resources/prompts + JSON Schema`, `PyO3 exceptions mapping`, `NAPI-RS js_name camelCase`, `Clap POSIX --json`, `cursor pagination Stripe/GitHub`, `rate limit headers Retry-After`, `OAuth2 JWT RS256 vs API keys`.
  - Delegar a `vanta-research` (read-only, digest ≤500 + RESULTADO) para la fase pesada.

### Task 6: API-STD-06 — Síntesis: qué estandarizar / cambiar / mejorar (nombres, firmas, errores, REST, OpenAPI, CLI/IQL + extras web)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Archivos clave:** outputs API-STD-02/03/04/05 (task files), `docs/api/VERSIONING.md`, `CONSTRAINTS.md`, `.opencode/rules/api-contract.md`, `.opencode/rules/server-mcp.md`, `.opencode/rules/js-ecosystem.md`, `.opencode/rules/query-dsl.md`, `.opencode/rules/python-bindings.md`
- **Verificación real:** 🟡 VERIFICAR — en DISCOVERY cruzar matriz fallos×estándares; sin código nuevo, solo decisiones. Requiere `question` tool (Gate P) para 🔴/ambiguos antes de fijar DO.
- **Gate Justificación:** Entrega central del pedido (punto 1 y 2 del usuario): lista normativa por eje + qué más (casing fronteras camelCase vs kebab CLI/MCP, Options-object/kwargs, glosario get/fetch/create/update, error code+message+context, tipos base codegen desde 1 schema, auth JWT/API-keys, versionado, RFC 7807, ISO 8601, rate-limit/ETag/idempotency/correlation-id, zero-copy Arrow donde aplique, MCP JSON Schema estricto, IQL AST JSON).
- **Gate Result:** ✅ DO
- **Contrato:** task file contiene tabla EJE→DECISIÓN→ALCANCE (11 superficies marcadas)→FUENTE (URL o `file:line`)→BREAKING(sí/no) Y `question` Gate P registrado para cada 🔴 Y cero decisiones sin fuente
- **Task file:** `docs/dev/tasks/API-STD-06.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟧 complejo — decisiones con trade-offs (compat vs limpieza) solo emergen al cruzar datos; NO planificar implementación aquí
- **Top 3 riesgos:**
  1. Se fija camelCase global y se rompe Python snake_case idiomático (frontera vs interior confundidos)
  2. Se prohíben sinónimos sin glosario de migración (get/fetch/retrieve coexisten)
  3. Se declara OpenAPI-first sin owner del YAML (deriva de nuevo en 1 PR)
- **Pre-mortem:**
  - **Fallo 1:** síntesis copia los 4 bloques del usuario sin filtrar lo inaplicable (gateway, GraphQL)
  - **Fallo 2:** se propone Protobuf/gRPC para bindings locales donde JSON+codegen basta (over-engineering, ponytail)
  - **Fallo 3:** decisiones breaking sin marcar `feat!:` (Regla 7 release-plz)
- **Stop conditions:** sin consenso en 🔴 tras 1 ronda `question` → DEFER con dueño; appetite >1d → partir por ejes
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Breaking sin `feat!:` | marcar cada una | review |
  | 🟡×🟡 | Estándar inaplicable a embebido | columna no-aplica con porqué | propuesta gateway |
  | 🟢×🟡 | ADR sin autor humano (Regla 5) | IA solo evidencia | decisión tradeoff |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Salida en inglés si es normativa técnica (Doc Language Split), resumen ES en task file ok.
  - Cada decisión con consecuencias + deuda (Regla 5/6).

### Task 7: API-STD-07 — Re-validación adversarial 11 superficies (segunda pasada: no más errores)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟠
- **Archivos clave:** TODOS los de API-STD-02/03/04 + `tests/` (`tests/api/`, `tests/sdk_serialization.rs`, `tests/query_result_*.rs`), `scripts/validate-docs-coverage.ps1`, `dev-tools/verify_changed.ps1`, `.opencode/task-system/prompts/subagent-recovery.md` (SARL)
- **Verificación real:** ✅ CÓDIGO-REAL — segunda pasada con contexto fresco: por superficie, re-grep firmas, re-correr parity tests, fuzz básico parser/CLI (`--help` × comando, ejemplo YAML inexistentente), revisar `unwrap/expect` en fronteras (deny clippy), `unsafe` sin `// SAFETY:` (Regla 4). Quien implementó no se auto-audita (P2-01 → `vanta-review`/`vanta-audit`).
- **Gate Justificación:** Punto 3 del usuario — validar lo conseguido y asegurar que no quedan errores/fallas/mal funcionamiento antes de definir implementación.
- **Gate Result:** ✅ DO
- **Contrato:** `dev-tools/verify_changed.ps1` verde documentado Y `scripts/validate-docs-coverage.ps1` estado anotado Y por superficie veredicto (✅ sin hallazgos nuevos | FIND-* creados) Y OCR `pwsh dev-tools/ocr-review.ps1` sin Critical/High o FIND-* abiertos
- **Task file:** `docs/dev/tasks/API-STD-07.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟨 complicado — checklist conocido pero 11 superficies exigen rigor, no invención
- **Top 3 riesgos:**
  1. Reviewer reusa contexto del investigador (no es fresco → pierde su función)
  2. Parity test en verde se toma como prueba de ausencia (cobertura ≠ exhaustividad)
  3. Fuzz encuentra crash y se clasifica como feature (minimizar)
- **Pre-mortem:**
  - **Fallo 1:** se re-verifica solo lo listado y no se buscan clases nuevas (auth, u128, paginación)
  - **Fallo 2:** `count` sin DB exit 0 se marca "ok" por ser edge menor
  - **Fallo 3:** INCOMPLETE se marca FAILED sin SARL (RESUME→RETRY→STRATEGY→ESCALATE)
- **Stop conditions:** fuzz crash real → estabilizar primero (Regla 8 caos); appetite >3d → por waves (bindings | red | core)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Deadlock RW-lecturas CLI | auditoría concurrencia (Regla 8) | toca dashmap/Tokio |
  | 🟡×🟡 | Flaky test se oculta con `continue-on-error` | prohibido (Regla 2) → Issue `flaky` | test intermitente |
  | 🟢×🔴 | UB FFI PyO3/WASM | exigir `// SAFETY:` + Miri | `unsafe` nuevo |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Routing: `vanta-review` (verdict approve/changes-required) + `vanta-audit` (solo reporta, nunca fix).
  - Todo hallazgo nuevo → FIND-* + fila en Risk Register.

### Task 8: API-STD-08 — Plan de implementación (cómo aplicar todo, breaking libre, MCP local, release siguiente)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** outputs API-STD-06/07, `docs/dev/operations/CI_POLICY.md`, `.opencode/rules/release-ci.md`, `.opencode/rules/open-core-licensing.md` (no tocar `vantadb-pro`), `CONTRIBUTING.md` (conventional commits), `vanta-mcp-local.ps1`, `opencode.jsonc` (MCP local), `docs/api/VERSIONING.md` (nuevo contrato 11 superficies), `docs/dev/Backlog.md` (filas nuevas si se aprueba ejecución)
- **Verificación real:** 🟡 VERIFICAR — sin código; produce waves de ejecución, cada una con verify cmd, `feat!:` markers, doc-sync (Regla 3 mismo-PR `docs/api/`), y refresh MCP local (`vanta-cli server --mcp` restart + `opencode.jsonc` check).
- **Gate Justificación:** Cierra el pedido: cómo se va a implementar lo definido, en qué orden, con qué gates, dejando todo funcional para el siguiente release aunque se rompa todo.
- **Gate Result:** ✅ DO
- **Contrato:** task file contiene waves ordenadas (0 fundación: tipos base+error envelope+codegen → 1 bindings → 2 HTTP/OpenAPI-first → 3 MCP/proxy/auth → 4 IQL versionado → 5 CLI `--json` → 6 docs+VERSIONING+MCP local) Y cada wave con verify (`verify_changed`/`verify.ps1`/`openapi_yaml_parity`/MCP smoke) Y rollback plan por wave
- **Task file:** `docs/dev/tasks/API-STD-08.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**
- **Cynefin:** 🟦 obvio una vez fijadas decisiones — secuenciar waves por dependencias
- **Top 3 riesgos:**
  1. Wave bindings y HTTP en paralelo con dependencias ocultas (contrato compartido)
  2. MCP local no se refresca y el agente trabaja contra tools viejas
  3. Release-plz publica breaking como patch por commit mal tipado
- **Pre-mortem:**
  - **Fallo 1:** se ejecutan waves sin `feat!:` y el changelog miente
  - **Fallo 2:** `docs/api/` se deja detrás del código (Regla 3) y el drift reaparece día 1
  - **Fallo 3:** se añade gateway/Protobuf por entusiasmo web sin appetite (ponytail ultra)
- **Stop conditions:** decisión 🔴 sin dueño → BLOQUEADO con `question`; scope > release → partir en 2 releases
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Deuda neta >0 por PR (Regla 6) | pagar con P2-5/P2-8 | PR con deuda nueva |
  | 🟡×🟡 | MCP local stale | `vanta-mcp-local.ps1` + restart opencode | tools viejas |
  | 🟢×🔴 | Tag/version manual (prohibido Regla 7) | solo release-plz | edit Cargo.toml version |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Orden sugerido (a confirmar en API-STD-06): tipos base + error RFC7807 + casing fronteras → bindings (get/score/put_batch/u128) → OpenAPI-first + REST + paginación → MCP nombres/validación/errores → proxy auth/endpoints → IQL_VERSION + case → CLI `--json` + flags → VERSIONING 11 superficies + docs + MCP local + `/audit quick` + `/ship` GO/NO-GO.
  - No ejecutar implementación en este plan (PLAN read-only). Ejecución = `/pipeline task API-STD-0X` o `/pipeline run` tras aprobar.
