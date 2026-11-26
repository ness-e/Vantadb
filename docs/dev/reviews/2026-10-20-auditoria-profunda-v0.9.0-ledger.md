---
title: "Auditoria profunda v0.9.0 - ledger de hallazgos (141)"
kind: review
status: active
description: "Una linea por hallazgo de la auditoria profunda: ID, severidad, area, ruta:linea y resumen. Companion del informe principal"
tags: [vantadb, audit, reviews, release]
---

# VantaDB v0.9.0 — FINDINGS LEDGER

Una línea por hallazgo. `ID | severidad | area | ruta:línea | resumen`.
Severidad: P0 = bloquea el release 0.9.0 en vuelo · P1 = alto · P2 = medio · P3 = bajo.

## P0 (1)

- `D-01 | P0 | seguridad/root-doc | SECURITY.md:26` — Afirma cifrado AES-256-GCM en reposo "behind the `encryption` feature flag (`src/crypto.rs`)"; `with_cipher` solo se llama desde un `#[test]` (`src/storage/vfile.rs:929`), `src/config.rs`/`docs/user/operations/CONFIGURATION.md:75` dice que el storage core sigue en texto plano (FIND-249). Falsa garantía de seguridad. | FIND-410 (cerrado) |
- `D-01b | P0 | release 0.9.0 | docs/CHANGELOG.md:39` — El changelog del 0.9.0 afirma "docs sin claim" de cifrado en reposo, pero `SECURITY.md:26` sigue portant el claim; el release 0.9.0 publica una garantía de cifrado que el código no cumple.

## P1 (39)

- `VDB-01 | P1 | rust-core | src/index/distance/kernels.rs:107,143; src/index/distance/mapper.rs:175; src/index/flat.rs:44; src/index/search/layer.rs:50; src/error.rs:146-151` — La dimensión del vector nunca se valida; con `Euclidean` un vector de dimensión distinta puntúa 0.0 (el MEJOR score) y outrankea los candidatos válidos. `Error::DimensionMismatch` existe pero nunca se construye. | FIND-411 (cerrado) |
- `S3-A1 | P1 | bindings node/ts | vantadb-node/src/lib.rs:1533,1506; vantadb-ts/src/native.ts:525-540; vantadb-ts/src/types.ts:228; vantadb-ts/src/vantadb.ts:714` — `exclude_superseded` se honra solo en WASM; la ruta nativa lo descarta silenciosamente. | FIND-412 (cerrado) |
- `S3-B1 | P1 | embeddings | src/physical_plan/vector.rs:57,67-68,150-151,161-162` — La búsqueda vectorial IQL se traga los fallos de embedding y devuelve 0 hits sin error. | FIND-413 (cerrado) |
- `S3-B2 | P1 | embeddings | src/llm.rs:192-197 vs :200-217; src/config.rs:1114-1124,395` — `LocalOnnxProvider::new` construye `LlmCfg` desde `Default::default()` y nunca lee el entorno: `VANTADB_ALLOW_DUMMY_EMBEDDINGS` es no-op en el camino principal. | FIND-414 (cerrado) |
- `S3-B4 | P1 | embeddings/docs | docs/api/EMBEDDINGS.md:58 vs src/config.rs:394,1108-1111; src/llm.rs:65-86; src/cli_handlers/config.rs:497` — El doc dice que con `embed-local` el provider por defecto es `local`; el default real es `ollama` siempre. | FIND-415 (cerrado) |
- `S5-B1 | P1 | user-skill | .opencode/skills/vantadb-mcp/SKILL.md:8,124,127` — "85 tools (47 core + …)"; real = 82 listados / 88 definidos / 49 core. | FIND-416 (cerrado) |
- `S5-B2 | P1 | user-skill | .opencode/skills/vantadb-mcp/SKILL.md:133; references/api-reference.md:77-91` — Presenta 6 `code_*` absorbidos (dispatch-only, nunca en `tools/list`) como disponibles y omite `code_index`, que sí está listado. | FIND-417 (cerrado) |
- `S5-B3 | P1 | user-skill | .opencode/skills/vantadb-mcp/scripts/test-mcp.py:58-62,165` — El smoke test MCP nunca puede pasar: espera 85/36/20 (real 82/38/22) y no exporta `VANTADB_MCP_PROFILE` al hijo, así que compara el perfil `agent` (39) contra el bucket `full`. | FIND-418 (cerrado) |
- `S5-B4 | P1 | user-skill | .opencode/skills/vantadb-mcp/references/configuration.md:125-131` — Declara `full` como default (85 tools), `dev` 36, `memory` 20; real = `agent` 39 default, `full` 82, `dev` 38, `memory` 22; `agent` no se menciona. | FIND-419 (cerrado) |
- `S5-B6 | P1 | user-skill | .opencode/skills/vantadb-mcp/assets/{claude-desktop-config.json,cursor-config.json,opencode-config.json}` | skills/vantadb-mcp/assets/install/*` — Las plantillas de config usan `~`, que el propio skill dice que no funciona. | FIND-420 (cerrado) |
- `D-02 | P1 | docs/config | docs/user/operations/CONFIGURATION.md:15,17,176-177,185,187` — Toda la referencia normativa usa `VantaConfig`, un tipo que no existe (el público es `vantadb::Config`, `src/lib.rs:177`); el snippet Rust no compila. | FIND-421 (cerrado) |
- `D-03 | P1 | docs/api | docs/api/MCP.md:676` — "tracks `[workspace.package].version` (`0.5.0` as of this doc)"; el workspace es 0.9.0 y la propia línea 11 del mismo archivo dice 0.9.0. | FIND-422 |
- `D-04 | P1 | docs/user | docs/user/QUICKSTART.md:5,11,54-55,113,143,161` — Todo el quickstart está pineado a v0.8.0 mientras el árbol es 0.9.0; el glob `vantadb_py-0.8.0-*.whl` no matchea el wheel 0.9.0. | FIND-423 |
- `D-05 | P1 | docs/benchmarks | docs/user/operations/BENCHMARKS.md:20,22` — Cita `heavy-certification.yml` como procedencia de los números; ese workflow no existe (es `certification.yml:89`). | FIND-424 (cerrado) |
- `D-06 | P1 | docs/benchmarks | docs/user/operations/BENCHMARKS.md:28` — Atribuye Recall@10 "Certified" a 10K vectores; el harness real es "BLOCK 1 - GROUND TRUTH RECALL (50K/128D)" (`tests/certification/stress_protocol.rs:221`). | FIND-425 (cerrado) |
- `D-07 | P1 | docs/api | docs/api/EMBEDDINGS.md:129-130` — `VANTA_EMBEDDING_PROVIDER` / `VANTA_LOCAL_MODEL` no se leen en ningún sitio (solo comentarios de deprecación, `src/config.rs:296,691`); las variables vivas son `VANTADB_*`. | FIND-426 (cerrado) |
- `D-08 | P1 | docs/dev | docs/dev/Backlog.md:356` — FIND-98 dice "79 vs 85 tools"; real 79 vs 82. | FIND-427 |
- `D-09 | P1 | docs/dev | docs/dev/Backlog-negocio.md:101` — Cita `docs/api/MCP.md:198` como fuente y da 80/86; esa línea dice 82/88. | FIND-428 |
- `D-10 | P1 | docs/config | docs/user/operations/CONFIGURATION.md:37` — `max_blocking_threads` default documentado 16; real = 2 × `available_parallelism()` (`src/config.rs:1050-1052`), 16 es solo el fallback. | FIND-429 (cerrado) |
- `D-11 | P1 | docs/config | docs/user/operations/CONFIGURATION.md:64` — `wal_buffer_size` default 65536 documentado; real = `None` (`src/config.rs:344,1406-1413`); la fila se contradice a sí misma. | FIND-430 (cerrado) |
- `D-12 | P1 | docs/api | docs/api/MCP.md:21` — Prerrequisito "`vanta-cli` — 0.5.0" con el workspace en 0.9.0. | FIND-431 |
- `D-13 | P1 | security/root-doc | SECURITY.md:3-11` — Tabla "Supported Versions" congelada en 0.5.x/0.4.x mientras la versión es 0.9.0; 0.6–0.9 quedan sin clasificar. | FIND-386 (cerrado) |
- `D-14 | P1 | citation/root-doc | CITATION.cff:9-10` — `version: 0.5.0` / `date-released: 2026-08-01`; el propio archivo instruye actualizarlo. | FIND-389 (cerrado) |
- `D-15 | P1 | docs/config | docs/user/operations/CONFIGURATION.md:19-92,122-129` — 6 env vars leídas por el código (`VANTADB_QUIET`, `VANTADB_SILENT`, `VANTADB_PROFILE_PATH`, `VANTADB_SCHEDULER_INTERVAL_MS`, `VANTADB_INGEST_CONFIG`, `VANTADB_SYNC_MODE`) no aparecen en ninguna de las dos tablas. | FIND-432 (cerrado) |
- `S7-01 | P1 | readme | README.md:329,349,359; README_ES.md:314,334,344` — Documenta `vanta-cli server start`; no existe ese subcomando (`src/cli.rs:338` es una variante plana con `--http/--mcp`). | FIND-433 (cerrado) |
- `S7-02 | P1 | readme | README.md:9,10; README_ES.md:9,10` — Badges apuntan a `gate-docs.yml` y `sec-codeql.yml`, workflows inexistentes. | FIND-448 (lote, cerrado) |
- `S7-03 | P1 | readme | README.md:219; README_ES.md:218` — Enlaza a `actions/workflows/heavy-certification.yml` (404); el real es `certification.yml`. | FIND-434 (cerrado) |
- `S7-04 | P1 | readme | README.md:139; README_ES.md:138` — Cita `ci-examples.yml` como suite de smoke; los ejemplos los corre `ci-fleet.yml:162-180`. | FIND-435 (cerrado) |
- `S7-05 | P1 | readme/integrations | integrations/README.md:48` — Justifica los pins contra `.github/workflows/adapters-compat.yml`, fichero inexistente. | FIND-436 (cerrado) |
- `S7-06 | P1 | readme/node | vantadb-node/README.md:15` — Enlaza `docs/dev/tasks/TS-12.md`, que no existe en ninguna de las tres ubicaciones canónicas. | FIND-437 (cerrado) |
- `S7-07 | P1 | readme/integrations | integrations/README.md:12-14` — Afirma "ninguno está en PyPI (404 x7, verificado 2026-10-06)"; los 7 están publicados en 0.5.0. | FIND-438 (cerrado) |
- `S7-08 | P1 | readme/integrations | integrations/{crewai,dspy,haystack,langchain,letta,llamaindex,mem0}/README.md:7-9` — Banner "Not on PyPI yet" falso en los 7 adaptadores. | FIND-439 (cerrado) |
- `S7-09 | P1 | readme/providers | providers/{litellm,ollama,openai}/README.md:7-9` — Banner "Not on PyPI yet" falso en los 3 providers (publicados en 0.8.0). | FIND-440 (cerrado) |
- `S7-10 | P1 | readme/integrations | integrations/mem0/README.md:32-35,41,43` — El quickstart llama `store.search("q")` (real: `search(query, vectors, top_k=…)`) e indexa `r["payload"]` (OutputData no es subscriptable). | FIND-441 (cerrado) |
- `S7-11 | P1 | readme/integrations | integrations/ollama/README.md:22,28-38; integrations/openai/README.md:22,28-39` — El quickstart instala el provider Rust por PyPI y luego usa la API del adaptador Python (nombres PyPI y de módulo en colisión). | FIND-448 (lote, cerrado) |
- `S7-12 | P1 | readme | vantadb-python/README.md:120; vantadb-ts/README.md:237` — Enlace a `QUICKSTART.md#4-real-embeddings-optional`; la sección real es `## 7. Optional: Local Embeddings`. | FIND-448 (lote, cerrado) |
- `S7-13 | P1 | readme | README.md:70 vs README_ES.md:66-68` — EN dice que `import vantadb_py` emite `DeprecationWarning`; ES dice que sigue "sin cambios". | FIND-448 (lote, cerrado) |
- `AUD-01 | P1 | CI/release | .github/workflows/ci-fleet.yml:36-54; .github/workflows/docs-gates.yml:38-58` — `integrations/**` no está en los `paths:` de ningún workflow que produce checks requeridos por el ruleset de `main` → un PR que solo toque adapters no dispara "Cross-language conformance (Py/Node/WASM)" ni los 6 checks de docs → merge bloqueado para siempre. | FIND-449 (lote, cerrado) |
- `AUD-02 | P1 | CI/release | .github/workflows/ci-rust.yml:94-136` — El job "Semver Checks (public API)" que dice "gate so a breaking change in the public API can never publish" tiene `continue-on-error: true` (línea 135) en el paso que hace el check → un breaking change NO puede bloquear el publish. | FIND-449 (lote, cerrado) |

## P2 (60)

- `VDB-02 | P2 | rust-core | src/index/flat.rs:44-51; src/index/search/nearest.rs:196-204; src/sdk/search/vector.rs:54-58,204-208; src/sdk/search/page.rs:264-269` — La escala de score Euclidean difiere por backend (`-d²` flat vs `-d` HNSW/prefilter): `range.min_score` filtra conjuntos distintos según tamaño de DB y selectividad. | FIND-442 (lote, cerrado) |
- `VDB-03 | P2 | rust-core/bitset | src/node/bitset.rs:75-77,101-112; src/storage/ops.rs:163; src/sdk/search/lexical.rs:208,221` — `set_bit` trunca a u32 y `to_u128()` persiste solo 128 bits: las máscaras construidas desde ids de 128 bits nunca matchean y los ids distintos colisionan. | FIND-442 (lote, cerrado) |
- `VDB-04 | P2 | rust-core/WAL | src/wal.rs:407-413,458-459,638,939,1067-1070,356-367,656-670` — El escritor WAL no limita el tamaño del registro pero el lector tiene un techo duro de 10 MB: un registro grande se clasifica como corrupto, se salta en el replay y puede truncarse. | FIND-442 (lote, cerrado) |
- `VDB-05 | P2 | rust-core/SDK | src/config.rs:51; src/server/handlers.rs:371-374; src/sdk/search/vector.rs:21,137; src/sdk/search/sparse.rs:46` — `MAX_K` solo se aplica en la superficie HTTP; el SDK pasa `top_k` sin clampear a `Vec::with_capacity(top_k)` → alloc enorme → abort de proceso. | FIND-442 (lote, cerrado) |
- `S3-A2 | P2 | bindings/python | vantadb-python/vantadb/__init__.py:18-28 vs vantadb-python/vantadb_py/__init__.py:52-74` — Los `__all__` de los dos paquetes difieren: el star-import depende de cuál se importe. | FIND-443 (lote, cerrado) |
- `S3-A3 | P2 | bindings/python | docs/api/PYTHON_SDK.md:678; vantadb-python/vantadb/__init__.pyi:15,28; vantadb-python/src/lib.rs:2854-2857` — El spec documenta una API de nivel de módulo que no está exportada (fn nativa existe pero no se reexporta). | FIND-443 (lote, cerrado) |
- `S3-A4 | P2 | bindings/python | vantadb-python/src/convert.rs:876-904 vs docs/api/ERROR_HANDLING.md:311-323` — El mapeo real de clases de excepción no coincide con la tabla canónica. | FIND-443 (lote, cerrado) |
- `S3-A5 | P2 | bindings/ts | vantadb-ts/src/native.ts:940-947` — Un método nativo no soportado devuelve un resultado fabricado en vez de fallar. | FIND-443 (lote, cerrado) |
- `S3-A6 | P2 | bindings/ts | vantadb-ts/src/native.ts:817-825,1144-1162` — Métodos nativos no soportados devuelven vacío/payload fabricado silenciosamente. | FIND-443 (lote, cerrado) |
- `S3-A7 | P2 | bindings/wasm | vantadb-wasm/src/lib.rs:1297,1403,1550,180-215; docs/api/WASM_API.md` — La consulta sparse se descarta en silencio; el doc WASM no menciona sparse en absoluto. | FIND-443 (lote, cerrado) |
- `S3-A8 | P2 | bindings/node | vantadb-node/src/lib.rs; docs/api/BINDINGS_NAMESPACES.md:179-197` — El binding Node carece de `generate_snippet`, `audit_text_index`, `operational_metrics`, `graph_page_rank`, `graph_degree_centrality` y `search_vector`; la diferencia no está documentada. | FIND-443 (lote, cerrado) |
- `S3-B3 | P2 | embeddings | src/error.rs:429-431; src/llm.rs:561-568` — El hint de recuperación sugiere una variable sin efecto en el camino real (ver S3-B2). | FIND-444 (lote, cerrado) |
- `S3-B5 | P2 | embeddings/mcp | vantadb-mcp/src/handlers/tools.rs:4122-4126,4098-4111,3072-3074,4174-4186` — `embed_texts` con `model` da error duro mientras el camino por defecto hace fallback. | FIND-444 (lote, cerrado) |
- `S3-B6 | P2 | embeddings/mcp | vantadb-mcp/src/handlers/tools.rs:3075-3081` — `embed_texts` reporta un nombre de modelo hardcodeado, no el real. | FIND-444 (lote, cerrado) |
- `S3-B7 | P2 | embeddings | src/embedding_health.rs:187-198; vantadb-mcp/src/handlers/tools.rs:3907-3918` — El chequeo "existen los ficheros del modelo?" puede ser satisfecho por los ficheros de OTRO modelo. | FIND-444 (lote, cerrado) |
- `S3-B8 | P2 | embeddings/bench | benchmarks/embed_bench.py:233-263` — El benchmark nunca ejecuta inferencia. | FIND-444 (lote, cerrado) |
- `S3-B9 | P2 | embeddings | src/physical_plan/vector.rs:80; src/planner.rs:427-432` — El brazo vectorial de IQL está limitado a 5 candidatos sea cual sea el LIMIT. | FIND-444 (lote, cerrado) |
- `S3-B10 | P2 | docs | docs/api/EMBEDDINGS.md:163; docs/user/QUICKSTART.md:264; docs/user/tutorials/05-embedding-integrations.md:112 vs src/parser/grammar.rs:69-80; docs/api/IQL.md:275` — `VECTOR_SEARCH('...')` no existe en ningún lenguaje del repo. | FIND-445 (lote, cerrado) |
- `S3-B11 | P2 | docs | src/llm.rs:767 vs docs/api/EMBEDDINGS.md:46` — El doc dice `POST /api/embeddings`; el código hace `POST /api/embed`. | FIND-445 (lote, cerrado) |
- `S5-A1 | P2 | docs/api | docs/api/MCP.md:408 vs :410 vs :200-209` — El heading "Extended Tool Families (38 defined; 32 listed)" contradice su propio cuerpo (39/33) y la tabla (82/88). | FIND-445 (lote, cerrado) |
- `S5-A2 | P2 | docs/api | docs/api/MCP.md:208 y :469-481` — Los 6 `thread_*` se cuentan en la tabla de familias pero no se documentan en ninguna sección. | FIND-445 (lote, cerrado) |
- `S5-A3 | P2 | docs/api | docs/api/MCP.md:676 vs server.json:6; Cargo.toml:805` — Claim de versión del registro MCP desfasado 4 minors. | FIND-445 (lote, cerrado) |
- `S5-A5 | P2 | docs/api | docs/api/MCP.md:105-108 vs vantadb-mcp/src/error.rs:21-27,123-129` — "Every error response carries … `error.data.code`" es falso para las 5 factorías JSON-RPC estándar (`plain()` → `data: null`, sin clave `data`). | FIND-445 (lote, cerrado) |
- `S5-A6 | P2 | docs/api | docs/api/openapi.yaml:283,320,355,385,413,670,703,1304,1708,2044-2072` — 9 response schemas declaran un envoltorio `{success,error,data}` que los handlers no emiten. | FIND-445 (lote, cerrado) |
- `S5-A7 | P2 | docs/api + server | docs/api/openapi.yaml:107-108,128-129; docs/api/HTTP_API.md:893-894 vs src/server/router.rs:173,314,343-345,372-383` — `/dashboard` se documenta como autenticado (`bearerAuth`) pero se monta FUERA del middleware de auth (público en loopback). | FIND-445 (lote, cerrado) |
- `S5-A8 | P2 | mcp | vantadb-mcp/src/server.rs:246,113-168; scripts/check_openapi_parity.mjs:16` — La ruta HTTP viva `/api/v2/mcp/proxy` no está en openapi.yaml ni en HTTP_API.md, y el gate de paridad solo escanea `src/server/router.rs`. | FIND-446 (lote, cerrado) |
- `S5-B5 | P2 | user-skill | .opencode/skills/vantadb-mcp/references/configuration.md:17,195` — Nombra un env var inexistente `VANTA_DB` para el flag `--db`. | FIND-447 (lote, cerrado) |
- `S5-B7 | P2 | user-skill | .opencode/skills/vantadb-mcp/references/api-reference.md:5,8-13,15,19-58` — La "fuente de verdad" del contrato MCP omite 11 herramientas reales. | FIND-447 (lote, cerrado) |
- `S5-B8 | P2 | user-skill | .opencode/skills/vantadb-mcp/SKILL.md:584-590,594,668` — 9 enlaces relativos rotos en la copia que el agente carga realmente (`.opencode/skills/`). | FIND-447 (lote, cerrado) |
- `S5-B9 | P2 | user-skill | .opencode/skills/vantadb-mcp/references/mcp-protocol.md:11` — Declara "Current version: 2024-11-05" como protocolo vigente. | FIND-447 (lote, cerrado) |
- `S5-B10 | P2 | user-skill | .opencode/skills/vantadb-mcp/scripts/setup-vantadb.sh:8,20-26` — Pinea `VANTADB_VERSION="0.5.0"` y resuelve el repo root equivocadamente desde la copia `.opencode`. | FIND-447 (lote, cerrado) |
- `S5-B13 | P2 | user-skill | .opencode/skills/vantadb-mcp/references/configuration.md:121-139` — Omite 4 env vars MCP reales, incluida la ACL de inyección y el sink de auditoría. | FIND-447 (lote, cerrado) |
- `D-16 | P2 | docs/api | docs/api/EMBEDDINGS.md:60` — Menciona el flag `--allow-dummy-embeddings`, que no existe en ningún binario (`src/cli.rs:15-38`). | FIND-445 (lote, cerrado) |
- `D-17 | P2 | docs/user | docs/user/blog/ollama_vantadb_local_memory.md:45` — El ejemplo usa `vantadb.VantaDB`; el SDK expone `Client` (lo detecta `scripts/docs/check-doc-examples.mjs`). | FIND-445 (lote, cerrado) |
- `D-18 | P2 | docs | CONTRIBUTING.md:164-165 vs .github/workflows/certification.yml:94` — "Heavy Certification up to 2h"; el job `stress-protocol` tiene `timeout-minutes: 180` (3 h). | FIND-445 (lote, cerrado) |
- `D-19 | P2 | docs/config | docs/user/operations/CONFIGURATION.md:42 vs src/cli_handlers/config.rs:216` — La fila `sync_mode` afirma "no env var"; el código lee `VANTADB_SYNC_MODE`. | FIND-445 (lote, cerrado) |
- `D-20 | P2 | docs/adr | docs/dev/architecture/adr/ADR-0011-sparse-vectors-native.md:5 (+ llms.txt) — El frontmatter afirma "dense-only vector search"; los sparse vectors son nativos (`src/node/vector_data.rs:30`). | FIND-445 (lote, cerrado) |
- `S7-14 | P2 | readme | README.md:208; README_ES.md:207; vantadb-node/dts-header.d.ts` — Apunta a `src/sdk.rs`, que no existe (es `src/sdk/`). | FIND-448 (lote, cerrado) |
- `S7-15 | P2 | readme | README.md:50; README_ES.md:46` — Quick Links apuntan a `#optional-server-mode` / `#modo-servidor-opcional`; los anchors reales son `#server-mode--self-healing-daemon` / `#modo-servidor-y-demonio-self-healing`. | FIND-448 (lote, cerrado) |
- `S7-16 | P2 | readme | README.md:240-248 vs README_ES.md:239-246` — Falta por completo la sección "North Star" en la traducción ES. | FIND-448 (lote, cerrado) |
- `S7-17 | P2 | readme | README.md:230 vs README_ES.md:229; docs/api/PROXY.md:233` — EN y ES definen la North Star de forma distinta ("put + search" vs "recall exitoso"). | FIND-448 (lote, cerrado) |
- `S7-18 | P2 | readme | README.md:288-295 vs README_ES.md:277-280; scripts/install.sh:48` — ES omite los flags `--modules`/`-Modules`/`--dry-run`/`-DryRun` y describe un wizard que no existe. | FIND-448 (lote, cerrado) |
- `S7-19 | P2 | readme/ts | vantadb-ts/README.md:245 vs vantadb-ts/src/vantadb.ts:181,247,1765-1773` — Documenta `VantaDB.create(config?)`; el export real es `Client`. | FIND-448 (lote, cerrado) |
- `S7-20 | P2 | readme/wasm | vantadb-wasm/README.md:119-121,154,158,210 vs vantadb-ts/src/vantadb.ts:185` — Tres grafías distintas del entry point, una de ellas `new Client()` con constructor `private`. | FIND-448 (lote, cerrado) |
- `S7-21 | P2 | readme | vantadb-ts/README.md:381-382 vs vantadb-ts/package.json:6-8; vantadb-node/README.md:25` — La tabla de runtimes dice "Node 18–22.11"; `engines.node` es `>=22.19`; el propio README usa 22.12 en otro sitio. | FIND-448 (lote, cerrado) |
- `S7-22 | P2 | readme/python | vantadb-python/README.md:165` — `black tests/ vantadb_python/`; el directorio real es `vantadb_py/`. | FIND-448 (lote, cerrado) |
- `S7-23 | P2 | readme/python | vantadb-python/README.md:25 vs docs/dev/architecture/adr/ADR-0047-anti-stutter.md` — Cita ADR-041 (renombres de variantes de error) como ADR anti-stutter; el correcto es ADR-0047. | FIND-448 (lote, cerrado) |
- `S7-24 | P2 | readme/integrations | integrations/haystack/README.md:45 vs integrations/haystack/vantadb_haystack/vectorstore.py:371` — Documenta `count_documents(filters)`; la firma real no tiene `filters`. | FIND-448 (lote, cerrado) |
- `S7-25 | P2 | readme/integrations | integrations/mem0/README.md:41,43 vs integrations/mem0/vantadb_mem0/vectorstore.py:172,278` — Documenta `search(query, k=4)` y `list(user_id=None, limit=100)`; ninguna de las dos firmas existe. | FIND-448 (lote, cerrado) |
- `S7-26 | P2 | readme/providers | providers/ollama/README.md:49 vs providers/ollama/src/python.rs:321; providers/litellm/README.md:49; providers/openai/README.md:49` — Documenta la clave `cursor`; el código emite `next_cursor`. | FIND-448 (lote, cerrado) |
- `S7-27 | P2 | readme/wasm | vantadb-wasm/README.md:327 + docs/user/QUICKSTART.md` — Referencia a `QUICKSTART.md §"4. Real Embeddings"`, sección que no existe (es §7). | FIND-448 (lote, cerrado) |
- `S7-28 | P2 | readme | README.md:74; README_ES.md:71 vs vantadb-python/vantadb_py/vantadb_py.pyi` — Nombra `Config` como símbolo canónico de Python; no existe en el SDK Python (es un tipo de TypeScript). | FIND-448 (lote, cerrado) |
- `AUD-03 | P2 | CI/release | .github/workflows/release.yml:1-3 vs :66` — La cabecera sigue diciendo "publish to crates.io via Trusted Publishing (OIDC)… Requires: crates.io Trusted Publisher" mientras el job publica con `secrets.CARGO_REGISTRY_TOKEN` (y pide `id-token: write` en :27 sin usarlo). | FIND-449 (lote, cerrado) |
- `AUD-04 | P2 | harness/docs | .opencode/AGENTS.md (sección "Secrets de CI Necesarios") vs .github/workflows/release.yml:66, release-adapters.yml:176, release-providers.yml:280, release-wheels.yml` — Afirma "ningún workflow referencia `secrets.*` para publicar (`gh secret list` vacío, correcto). No crear estos tokens"; hay 4 workflows de publish que sí los usan, y ahora el publish a crates.io **exige** `CARGO_REGISTRY_TOKEN`. | FIND-450 (lote, cerrado) |
- `AUD-05 | P2 | CI/scripts | scripts/docs/check-npm-versions.mjs:19-24 vs TARGETS (líneas ~50-70) y self-test (~250) | La cabecera dice que `vantadb-node` "NEVER published" con policy `never-published`; `TARGETS` usa `policy: 'workspace'`, y el self-test sigue ejercitando la rama muerta `never-published`. | FIND-449 (lote, cerrado) |
- `AUD-07 | P2 | CI/scripts | scripts/validate-docs-coverage.ps1:64-65 (SDK), :176-181 (MCP) | El gate de cobertura cubre 32 de ~142 métodos públicos del SDK (4 de ~50 ficheros de `src/sdk/`) y 49 de las 82 tools MCP — las extendidas se贡献 vía `tools.extend()` (handlers/tools.rs:1072-1079) y nunca se comprueban. | FIND-449 (lote, cerrado) |
- `AUD-10 | P2 | harness | .opencode/AGENTS.md:37,41; SKILLS-MANIFEST.md (referencias a CONSTRAINTS.md) | Remiten a `CONSTRAINTS.md` en la raíz como quality bar; el fichero no existe ahí (está en `docs/specs/CONSTRAINTS.md`). | FIND-450 (lote, cerrado) |
- `AUD-11 | P2 | packaging | integrations/openai/pyproject.toml:6; integrations/ollama/pyproject.toml:6 vs providers/openai/Cargo.toml:2; providers/ollama/Cargo.toml:2` — Colisión de nombre de proyecto PyPI: `vantadb-openai` y `vantadb-ollama` existen como adaptador Python y como provider Rust; solo la matriz de `release-adapters.yml:35` los mantiene separados. | FIND-451 (lote, cerrado) |
- `AUD-13 | P2 | CI/release | .github/workflows/release.yml:120-124 | Si `gh pr list` no ve todavía el PR de release-plz, el sync de manifiestos imprime "skipping manifest sync" y `exit 0` → los 22 manifiestos quedan sin sincronizar sin ninguna señal de fallo. | FIND-449 (lote, cerrado) |

## P3 (41)

- `VDB-06 | P3 | rust-core/txn | src/storage/engine/txn.rs:281-287,140-145; src/storage/engine/init.rs:532-540` — `commit_transaction` de un id desconocido o ya comiteado escribe un `Commit` espurio en el WAL y devuelve `Ok(())`. | FIND-442 (lote, cerrado) |
- `VDB-07 | P3 | rust-core/storage | src/storage/ (dos rutas de escritura) | La copia en WAL y la almacenada divergen en `last_accessed` en direcciones opuestas tras crash+replay; afecta al orden de evicción LRU. | FIND-442 (lote, cerrado) |
- `VDB-08 | P3 | rust-core/delete | src/storage/engine/delete.rs:38,75-112` — La sonda pre-delete descarta el error (`self.get(id).ok().flatten()`): un fallo de lectura se confunde con "nodo no existe" y los contadores/índices quedan desincronizados. | FIND-442 (lote, cerrado) |
- `S3-A9 | P3 | docs | docs/api/BINDINGS_NAMESPACES.md:261,313,317,372 | Tablas de superficie cross-binding desfasadas ("45 public methods", "49 pyclass methods").` | FIND-445 (lote, cerrado) |
- `S3-A10 | P3 | docs | bindings python/ts | No hay superficie de embeddings en Python ni TS pese a documentarse en `docs/api/EMBEDDINGS.md`. | FIND-445 (lote, cerrado) |
- `S3-B12 | P3 | docs | embeddings/manifest.json (10 modelos) vs docs/api/EMBEDDINGS.md:89,94,105,133,145-149; embeddings/README.md:22,36,87; embeddings/download.py:69` — "9 modelos" en todas partes; el manifest tiene 10 (`embeddinggemma-2` incluido). | FIND-445 (lote, cerrado) |
- `S3-B13 | P3 | docs | docs/api/EMBEDDINGS.md:97 vs embeddings/verify.py:1-20,141-203 | El doc dice que `verify.py` hace "ONNX vs HF > 0.98"; el script solo imprime el cosine, nunca lo asserta, y no hay baseline HF. | FIND-445 (lote, cerrado) |
- `S3-B14 | P3 | embeddings | embeddings/sanity_embed.py | Resuelve rutas relativas al CWD.` | FIND-444 (lote, cerrado) |
- `S3-B15 | P3 | setup | setup-embeddings.ps1 | El claim "falls back to dummy" contradice el default estricto.` | FIND-451 (lote, cerrado) |
- `S3-B16 | P3 | docs | docs/api/EMBEDDINGS.md §`LocalOnnxProvider` internals` | Internos der drifted. | FIND-445 (lote, cerrado) |
- `S3-B17 | P3 | embeddings | src/llm.rs (embed_batch) | `embed_batch` es un bucle secuencial, no una llamada batched. | FIND-444 (lote, cerrado) |
- `S5-A9 | P3 | docs/api | docs/api/openapi.yaml:1764-1772; src/server/middleware.rs:166-176 | El header `x-vanta-user-key` (tercer nivel de identidad, con 401 que lo nombra) no está declarado como security scheme ni documentado. | FIND-445 (lote, cerrado) |
- `S5-A10 | P3 | docs/api | docs/api/MCP.md:665 vs scripts/validate-docs-coverage.ps1:176-181 | El claim "tool coverage … enforced mechanically" es más ancho que lo que el gate realmente cubre (49 de 82).` | FIND-445 (lote, cerrado) |
- `S5-A11 | P3 | mcp/registry | server.json:1-22 | Válido contra el schema pero no instalable (sin `packages[]`/`remotes[]`) y con `websiteUrl` apuntando a `blob/develop` (rama móvil). | FIND-446 (lote, cerrado) |
- `S5-A12 | P3 | harness | opencode.jsonc:69 | La entrada MCP del propio repo declara "85 tools perfil full"; el launcher nunca fija `VANTADB_MCP_PROFILE` y el default es `agent` (39). | FIND-450 (lote, cerrado) |
- `S5-A13 | P3 | mcp | vantadb-mcp/src/server.rs:499-526,571-635 | Un panic en `initialize`/`tools/list`/`prompts/*` (inline, fuera de `spawn_blocking`) aborta la task sin escribir respuesta → cuelgue silencioso del cliente. | FIND-446 (lote, cerrado) |
- `S5-B14 | P3 | user-skill | .opencode/skills/vantadb-mcp/SKILL.md:594,668 | Llama "stub" a `docs/api/MCP.md` en ambas direcciones, aunque es la referencia completa. | FIND-447 (lote, cerrado) |
- `S5-B15 | P3 | user-skill | .opencode/skills/vantadb-mcp/references/mcp-protocol.md:7 | Repite el conteo obsoleto de 85 tools.` | FIND-447 (lote, cerrado) |
- `S5-B16 | P3 | user-skill | .opencode/skills/vantadb-mcp/ vs skills/vantadb-mcp/ | El mirror `.opencode` no tiene los dos directorios de assets (hooks/, install/). | FIND-447 (lote, cerrado) |
- `S5-B17 | P3 | user-skill | .opencode/skills/vantadb-mcp/SKILL.md:8,437; docs/api/MCP.md:494 | "2 resources" vs "2 static + 2 dynamic" dentro del mismo fichero.` | FIND-447 (lote, cerrado) |
- `D-21 | P3 | docs/adr | docs/dev/architecture/adr/ADR-0011-sparse-vectors-native.md:36` | Dice que `SparseVector` está en `src/node.rs`; vive en `src/node/vector_data.rs:30` y `src/node.rs` ya no existe. | FIND-445 (lote, cerrado) |
- `D-22 | P3 | docs/benchmarks | docs/user/operations/BENCHMARKS.md:34-36 | 6.1/1.2 = 5.08x, pero el factor publicado es 4.88x (calculado pre-redondeo).` | FIND-445 (lote, cerrado) |
- `D-23 | P3 | docs/dev | docs/dev/tasks/ (634 ficheros) | `status:` usa dos grafías para el mismo estado: `in_progress` (11) e `in-progress` (6). | FIND-445 (lote, cerrado) |
- `D-24 | P3 | root-doc | llms.txt:1-14 | Los conteos son correctos (475 páginas / 634 tasks / 1109 total) pero no existe generador para `llms.txt` → se mantiene a mano. | FIND-445 (lote, cerrado) |
- `D-25 | P3 | docs/api | docs/api/VERSIONING.md:17-19 | Los ejemplos de versión de la política (`0.5.0 → 0.6.0`) van 4 releases atrasados. | FIND-445 (lote, cerrado) |
- `D-26 | P3 | user-skill | skills/vantadb-mcp/scripts/test-mcp.py:26-28,60-61 | El docstring declara 85/36/20 citando `mcp_tests.rs::test_mcp_tool_profiles` como fuente, que afirma 82/39/38/22. | FIND-447 (lote, cerrado) |
- `S7-29 | P3 | readme | README.md:201 vs README_ES.md:200 | EN dice backend "File", ES dice "VantaFile" — dos backends distintos en tablas equivalentes.` | FIND-448 (lote, cerrado) |
- `S7-30 | P3 | readme | README_ES.md:9-10 vs README.md:8-11 | Falta el badge Nightly en la traducción ES.` | FIND-448 (lote, cerrado) |
- `S7-31 | P3 | readme | README.md:139; README_ES.md:138 vs examples/python/mem0_integration.py:1 | Señala `vantadb_py` (deprecado) como nombre canónico; los ejemplos importan `vantadb`. | FIND-448 (lote, cerrado) |
- `S7-32 | P3 | readme/integrations | integrations/crewai/README.md:44 vs integrations/crewai/vantadb_crewai/vectorstore.py:32 | La API table da una firma posicional que no existe (`embedding` es el primer parámetro, `db_path`/`namespace` son keyword-only). | FIND-448 (lote, cerrado) |
- `S7-33 | P3 | readme | README.md:203; README_ES.md:202 vs src/config.rs | Afirma que M/ef_construction/ef_search son configurables; no están expuestos en `Config`. | FIND-448 (lote, cerrado) |
- `S7-34 | P3 | readme | README.md:121-122; README_ES.md:120-121 vs src/sdk/search/debug_ops.rs:241-245 | Etiqueta como "Hybrid Search" una llamada solo-vector (sin `text_query`), que toma el brazo denso. | FIND-448 (lote, cerrado) |
- `S7-35 | P3 | readme/server | vantadb-server/README.md:5 | Referencia `DEPLOYMENT_GUIDE.md` sin ruta (real: `docs/user/operations/DEPLOYMENT_GUIDE.md`). | FIND-448 (lote, cerrado) |
- `S7-36 | P3 | readme/proxy | vanta-proxy/README.md:16 | Referencia `docs/api/PROXY.md` ambigua desde el subpaquete. | FIND-448 (lote, cerrado) |
- `S7-37 | P3 | readme/mcp | vantadb-mcp/ | El paquete no tiene README, siendo una de las cuatro superficies del producto.` | FIND-448 (lote, cerrado) |
- `S7-38 | P3 | docs/user | docs/user/QUICKSTART.md:~57-60 | Pinea `vantadb-py>=0.8.0` sin reconciliar con el árbol 0.9.0. | FIND-445 (lote, cerrado) |
- `S7-39 | P3 | readme | vantadb-node/README.md:3; vantadb-wasm/README.md:3; vantadb-ts/README.md:5; vantadb-python/README.md:3 | "tres conectores de lenguaje activos" nunca definidos; conviven con "cuatro superficies" y "4+ paquetes".` | FIND-448 (lote, cerrado) |
- `S7-40 | P3 | readme/ts | vantadb-ts/README.md:206-214 vs vantadb-ts/src/vantadb.ts:746 | Ejemplo sin `await` junto a otros que sí lo usan, sin fijar el contrato sync/async. | FIND-448 (lote, cerrado) |
- `AUD-08 | P3 | harness | SKILLS-MANIFEST.md:3,7; .opencode/AGENTS.md:184 | Declaran "196 skills en 197 dirs"; en disco hay 198 dirs y 197 con SKILL.md.` | FIND-450 (lote, cerrado) |
- `AUD-09 | P3 | harness | .opencode/AGENTS.md:137 | Autorreferencia rota: "La tabla de Path Resolution (línea 78)" — la línea 78 pertenece a la tabla "Intención → comando" (la real está en 99-108).` | FIND-450 (lote, cerrado) |
- `AUD-17 | P3 | docs/api | docs/api/openapi.yaml:121 vs src/server/router.rs:380 | El path parametrizado del dashboard usa `{path}` en el spec y `{*path}` (axum 0.8) en el código. | FIND-445 (lote, cerrado) |
