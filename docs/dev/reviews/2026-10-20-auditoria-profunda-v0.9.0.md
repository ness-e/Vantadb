---
title: "Auditoria profunda de VantaDB v0.9.0 - 141 hallazgos (multi-subagente)"
kind: review
status: active
description: "Revision read-only de 12 areas del sistema con 8 subagentes: 2 P0, 39 P1, 59 P2, 41 P3. El ledger de una linea por hallazgo esta en el informe companion"
tags: [vantadb, audit, reviews, release, docs, security]
---

# VantaDB v0.9.0 — DEEP AUDIT REPORT

**Repo:** `el repo`
**Branch auditada:** `release-plz-fix-cargo-token` @ `2e8152a3` (9 commits por delante de `origin/main`)
**Versión del árbol:** 0.9.0
**Modo:** READ-ONLY absoluto. No se creó/modificó/borró ningún fichero del repo; no se ejecutó ningún comando git de escritura; `gh` solo en lectura (`api GET`).
**Salida:** todo en ``audit-v090-2026-10-20/`\` (el informe principal, el ledger de hallazgos, los informes por subagente (S2, S3, S5, S6, S7), notas del orquestador).

---

## 1. Resumen ejecutivo

### 1.1 Conteos por severidad

| Severidad | # | Significado |
|---|---|---|
| **P0** | **2** | Bloqueantes del release 0.9.0 en vuelo (1 causa raíz + su reprint en el changelog) |
| **P1** | **39** | Alto: claim falso que engaña al usuario, comando que falla, o gate que no gatea |
| **P2** | **59** | Medio: drift documental con impacto real, o bug de correctitud acotado |
| **P3** | **41** | Bajo: conteos, anchors, nomenclatura, cobertura ausente |
| **TOTAL** | **141** | |

### 1.2 Conteos por área

| Área | Alcance | P0 | P1 | P2 | P3 | Total |
|---|---|---|---|---|---|---|
| 1 · Núcleo Rust | `src/` (engine, wal, storage, index, graph, sdk, config, error, cli) | 0 | 1 | 4 | 3 | 8 |
| 2 · Bindings | python / node / wasm | 0 | 1 | 7 | 2 | 10 |
| 3 · Embeddings | `src/llm.rs`, `src/physical_plan/vector.rs`, `embeddings/` | 0 | 3 | 8 | 6 | 17 |
| 4 · Providers | `providers/*` | 0 | 0 | 0 | 0 | 0 ⚠️ |
| 5 · Adapters | `integrations/*` (9 pkgs) — auditados vía sus READMEs (§3.7) | 0 | 0 | 0 | 0 | 0 ⚠️ |
| 6 · MCP + Server | `vantadb-mcp/`, `vantadb-server/`, openapi | 0 | 0 | 7 | 5 | 12 |
| 7 · Skills de usuario | `skills/`, `.opencode/skills/vantadb*` | 0 | 5 | 6 | 4 | 15 |
| 8 · Docs | `docs/**` (incluye los P0) | 2 | 14 | 5 | 6 | 27 |
| 9 · READMEs (+ READMEs de adapters/providers) | `README.md`, `README_ES.md`, sub-paquetes | 0 | 13 | 15 | 12 | 40 |
| 10+11+12 · Harness / CI / release en vuelo | `.opencode/`, `.github/workflows/`, `scripts/`, `release-manifest.json` | 0 | 2 | 7 | 3 | 12 |
| **Total** | | **2** | **39** | **59** | **41** | **141** |

⚠️ S1, S4 y S8 no produjeron informe **a tiempo**. Estado a 2026-10-20: **S1 (CI/release) y S4 (providers/adapters) sí lo entregaron más tarde** y sus hallazgos están incorporados; **S8 (harness `.opencode/`) no entregó** — esa área la cubrió la wave W0 del orquestador (`audit-deep-w0-harness-20261020-033416.md`). Ver §5.2.

⚠️ Límites de verificabilidad de toda la campaña (declarados por la propia auditoría): **solo verificación estática** (no había `cargo` en el PATH → la suite no se ejecutó) y **sin segundo autor** (el tool `task` no existía; las 11 waves las ejecutó el orquestador, y W10 fue auto-verificación). Ver `docs/dev/reviews/audit-deep-sintesis-20261020-033416.md` §1 y el backlog `FIND-410`.

### 1.3 Veredicto

**Sí hay P0, y es del release 0.9.0 en vuelo.** `SECURITY.md:26` publica como capacidad activa del producto un cifrado en reposo AES-256-GCM que el core **no** aplica (único call site de `with_cipher` = un `#[test]` en `src/storage/vfile.rs:929`). El propio changelog del 0.9.0 (`docs/CHANGELOG.md:39`) afirma que se retiró el claim de cifrado de la documentación — no se retiró. Un documento de política de seguridad que miente sobre la garantía que protege los datos es el tipo de cosa que no puede entrar en un tag. (2 P0 = 1 causa raíz + su reprint en el changelog del release.)

Fuera de eso, **el release 0.9.0 no está bloqueado por bugs de código**: el motor de versión (`sync-workspace-version.mjs`) está verificado verde (`--check` → 22/22 en sync, `--self-test` → 12/12), `release-manifest.json` con 15 registries coincide 1:1 con el fan-out real, el guard `dependabot[bot]` está aplicado solo a jobs no-publicadores, y la ruta OIDC→token de crates.io está implementada. Los P1 de mayor riesgo para el release son gates que no gatean (AUD-01, AUD-02), no fallos de publish.

El patrón dominante del repo es **deriva documental**: 60+ afirmaciones concretas en `.md` que el código contradice (versiones, contadores de tools, variables de entorno, defaults, workflows inexistentes, flags inexistentes). Varias están en archivos que el propio repo declara "fuente de verdad" (`docs/api/MCP.md`, `skills/vantadb-mcp/references/api-reference.md`, `docs/user/operations/CONFIGURATION.md`).

---

## 2. P0 — Bloqueantes del release 0.9.0

### D-01 | P0 | seguridad / root-doc
- **Ubicación:** `SECURITY.md:26`
- **Qué afirma/rompe:** "**AES-256-GCM at-rest encryption** behind the `encryption` feature flag (`src/crypto.rs`)", listado bajo "## Security Practices" como capacidad activa. `src/storage/vfile.rs:439` (`with_cipher`) solo se llama desde `src/storage/vfile.rs:929`, dentro de un `#[test]` (`#[cfg(feature="encryption")] #[test]`, mod `tests` en `:517`). El propio doc del método (`:436-438`) dice que el storage layer *debería* usar el cipher. `docs/user/operations/CONFIGURATION.md:75` afirma lo contrario: *"Not yet applied to core storage files — WAL/HNSW/VantaFile/backend-KV/snapshots remain plaintext today (FIND-249); the current consumer is the `vanta-proxy` envelope."* `vantadb-server/Cargo.toml:21` no habilita la feature `encryption`.
- **Por qué importa:** Un revisor de seguridad — o un usuario que elige VantaDB por datos sensibles — lee `SECURITY.md` y concluye que la base está cifrada en reposo. No lo está. Es una garantía de seguridad falsa en el primer fichero que un auditor de seguridad abre. Contradice además el changelog del propio release: `docs/CHANGELOG.md:39` — *"SRV-10 - cifrado en reposo honesto: aviso key-sin-feature, feature opt-in encryption, **docs sin claim**"* — el 0.9.0 afirma haber retirado el claim y `SECURITY.md` lo sigue portant.
- **Fix:** Reescribir `SECURITY.md:26` con la redacción de `CONFIGURATION.md:75`: *"Primitivas AES-256-GCM existen detrás de la feature `encryption` (`src/crypto.rs`) y las usa el envelope de `vanta-proxy`; **los ficheros de storage del core todavía no están cifrados en reposo** (FIND-249)."* Corregir también el doc de módulo `src/crypto.rs:8`, que es la otra mitad del claim falso.
- **Cómo verificar:** `Select-String -Path SECURITY.md -Pattern "at-rest"` → solo menciones condicionadas; `Select-String -Path src -Recurse -Filter *.rs -Pattern "with_cipher"` → solo el call site de test hasta que aterrice FIND-249.

### D-01b | P0 | release 0.9.0 (consolidado de D-01)
- **Ubicación:** `docs/CHANGELOG.md:39` + `SECURITY.md:26`
- **Qué afirma/rompe:** El release 0.9.0 publica en su changelog que el claim de cifrado en reposo fue retirado de la documentación, mientras el fichero de política de seguridad de la raíz lo sigue emitiendo. El tag v0.9.0 queda con una garantía de seguridad no cumplida y con su propio changelog desincronizado del artefacto publicado.
- **Por qué importa:** Un changelog que afirma una corrección que no ocurrió es peor que no tener changelog: dirige al revisor a cerrar el ticket.
- **Fix:** Corregir `SECURITY.md:26` (ver D-01) antes del tag, o cambiar la línea del changelog para que describa el estado real.
- **Cómo verificar:** `git show v0.9.0:SECURITY.md | Select-String "at-rest"` vs `docs/CHANGELOG.md:39`.

---

## 3. P1 — Altos (39)

### 3.1 Núcleo Rust (1)

#### VDB-01 | P1 | vector index / distance
- **Ubicación:** `src/index/distance/kernels.rs:107,143`; `src/index/distance/mapper.rs:175`; `src/index/flat.rs:44`; `src/index/search/layer.rs:50`; `src/error.rs:146-151`; `src/sdk/serialization/mod.rs:611`; `src/sdk/api/memory.rs:2489`
- **Qué afirma/rompe:** `euclidean_distance_squared_f32` devuelve `0.0` cuando `a.len() != b.len()` (los kernels SIMD cortocircuitan). `calculate_similarity` lo niega → score `0.0`, que es el **mejor score posible** en Euclidean (vectores idénticos también dan 0.0). `Error::DimensionMismatch { expected, got }` existe pero **nunca se construye** en `src/`, y el `put` copia el vector del llamante sin validar longitud. Consecuencias: el path flat (≤ `flat_threshold` = 10 000 nodos) y el path HNSW devuelven el candidato mal dimensionado **por delante** de los válidos; el path HNSW-inline (`src/index/graph/core.rs:215-231`) computa `-(q_norm² + node_norm²)`, un valor grande negativo — **los tres backends discrepan para la misma entrada**.
- **Por qué importa:** Resultados de búsqueda silenciosamente wrong (top-1 incorrecto) sin error ni warning. Con el default `Cosine` el mismatch da 0.0 (score neutro plausible), así que el bug se esconde hasta que alguien cambia a `Euclidean`. La existencia de `DimensionMismatch` sin uso muestra que el invariante se quiso y nunca se implementó.
- **Trigger:** `put(ns,"a",payload,vector=[0.,1.,0.])` (3d) y `put(ns,"b",payload,vector=[1.,0.,0.,0.,0.])` (5d); `search(..., query_vector=[1.,0.,0.], distance_metric=Euclidean, top_k=1)` → devuelve "b" con score 0.0; la respuesta correcta es "a" (−2.0).
- **Fix:** (1) Que los kernels devuelvan `f32::NAN` en mismatch y propagar NAN — `src/index/graph/types.rs:164-176` ya fija NaN por debajo de todo score finito y `src/index/search/nearest.rs:192` ya descarta NaN. (2) Validar la dimensión en el `put` con `Error::DimensionMismatch` (ya existe) y en el query vector contra la dimensión del índice. (3) Test de regresión: corpus mixto + Euclidean + top_k=1 debe devolver el registro de dimensión correcta.
- **Cómo verificar:** `cargo test --lib sdk::search::tests::euclidean_dimension_mismatch_ranks_matching_dim_first`.

### 3.2 Embeddings (3)

#### S3-B4 | P1 | default del provider
- **Ubicación:** `docs/api/EMBEDDINGS.md:58` vs `src/config.rs:394`, `src/config.rs:1108-1111`; `src/llm.rs:65-86`; `src/cli_handlers/config.rs:497`
- **Qué afirma/rompe:** *"With `embed-local` active, default `VANTADB_EMBEDDING_PROVIDER` is `local` (default model `embeddings/models/multilingual-e5-small/onnx` [384d]…)"*. El default real es `"ollama"` tanto en `LlmCfg::default()` (`src/config.rs:394`) como en el parser de env (`src/config.rs:1110`), y en la plantilla TOML generada (`src/cli_handlers/config.rs:497`). `get_embedding_provider()` (`src/llm.rs:67-69`) enruta a `OllamaProvider`, que hace POST a `http://localhost:11434/api/embed`.
- **Por qué importa:** Un usuario que compila con `embed-local` y no fija el provider espera inferencia local (offline, sin red) y recibe llamadas a Ollama → fallo de red o, peor, dependencia silenciosa de un servicio externo.
- **Trigger:** Compilar con la feature `embed-local` y no exportar `VANTADB_EMBEDDING_PROVIDER` → `embed()` postea a `localhost:11434`.
- **Fix:** O cambiar el default del código a `local` cuando la feature está activa (feature-gated), o corregir el doc a *"default is `ollama` regardless of `embed-local`; set `VANTADB_EMBEDDING_PROVIDER=local` to use the bundled ONNX model"*. La primera opción es la que el usuario espera.
- **Cómo verificar:** `Select-String -Path src/config.rs -Pattern "embedding_provider"` → el default y la rama feature-gated.

#### S3-B2 | P1 | `VANTADB_ALLOW_DUMMY_EMBEDDINGS` no-op
- **Ubicación:** `src/llm.rs:192-197` vs `src/llm.rs:200-217`; `src/config.rs:1114-1124`; `src/config.rs:395`
- **Qué afirma/rompe:** `LlmCfg::default()` fija `strict_embeddings: true` (`src/config.rs:395`). `LocalOnnxProvider::new` construye su `LlmCfg` desde `Default::default()` (`src/llm.rs:193-196`), así que **nunca lee el entorno** y `allow_dummy` es siempre `false` (`src/llm.rs:208`). Solo `LocalOnnxProvider::from_llm_cfg(&Config::default().llm_cfg())` honra `VANTADB_STRICT_EMBEDDINGS` / `VANTADB_ALLOW_DUMMY_EMBEDDINGS`, y ese constructor se llama desde un único sitio: `src/physical_plan/vector.rs:66`.
- **Por qué importa:** La variable de entorno que la documentación y el propio `src/error.rs:430`灾 presentan como la palanca para permitir fallback sintético **no hace nada** en el camino principal. Un operador que fija la variable para desbloquear un arranque ve el mismo fallo, sin diagnóstico.
- **Trigger:** `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` + modelo ONNX ausente + llamada a `LocalOnnxProvider::new(...)` → sigue fallando en modo estricto.
- **Fix:** Que `LocalOnnxProvider::new` resuelva el env (o que su firma acepte la `LlmCfg`) y que el camino de error cite la variable que sí funciona.
- **Cómo verificar:** Test que fija `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` y comprueba que `LocalOnnxProvider::new` produce embeddings dummy.

#### S3-B1 | P1 | IQL vector search traga fallos de embedding
- **Ubicación:** `src/physical_plan/vector.rs:57,67-68,150-151,161-162`
- **Qué afirma/rompe:** El brazo vectorial del plan físico IQL se traga los errores del proveedor de embeddings y continúa como si el corpus tuviera 0 vectores → 0 hits, sin error.
- **Por qué importa:** Una consulta de búsqueda IQL devuelve "no hay resultados" cuando el problema real es que el proveedor de embeddings está caído o el modelo no está cargado. El usuario depura la query en lugar del pipeline de embeddings.
- **Trigger:** Provider de embeddings caído + `SELECT ... WHERE field ~ "text", min = score` → 0 filas, sin error.
- **Fix:** Propagar el error de embedding como `Err` (o marcar el resultado con un campo `embeddings_error`).
- **Cómo verificar:** Test con un provider mock que devuelve `Err` → el search debe propagar el error, no devolver un set vacío.

### 3.3 Bindings (1)

#### S3-A1 | P1 | node / ts — divergencia cross-backend
- **Ubicación:** `vantadb-node/src/lib.rs:1533`, `vantadb-node/src/lib.rs:1506`; `vantadb-ts/src/native.ts:525-540`; `vantadb-ts/src/types.ts:228`; `vantadb-ts/src/vantadb.ts:714`
- **Qué afirma/rompe:** El SDK TS declara `exclude_superseded?: boolean` en `SearchRequest` (`types.ts:228`) y lo honra **solo** en el backend WASM (`vantadb.ts:714`). La ruta nativa lo descarta: `_buildSearchRequest` (`native.ts:525-540`) nunca copia el campo, y el parser de Node lo fixa a `false`.
- **Por qué importa:** El mismo código TS devuelve resultados distintos según el backend: con WASM excluye los supersedidos, con Node no. Un usuario que migra de WASM a Node ve más resultados y cree que hay un bug de datos. La SearchParity/contract tests no lo detectan porque el campo no está en el request nativo.
- **Trigger:** Mismo corpus, misma query con `exclude_superseded: true`: WASM → N-1 hits, Node → N hits.
- **Fix:** Propagar el flag en `_buildSearchRequest` y en el parser NAPI; añadir el caso a la tabla de paridad de `docs/api/SEARCH_PARITY.md`.
- **Cómo verificar:** Test de paridad Py/Node/WASM con `exclude_superseded: true` → mismo número de hits.

### 3.4 MCP + Server (5)

#### S5-B1 | P1 | conteo de tools del skill
- **Ubicación:** `.opencode/skills/vantadb-mcp/SKILL.md:8`, `:124`, `:127` (y espejo `skills/vantadb-mcp/SKILL.md`)
- **Qué afirma/rompe:** "The MCP server exposes **85 tools** (47 core + 7 `skill_*` + 8 `code_*` + 6 `wiki_*` + 1 `context_assemble` + 5 `scene_*` + 6 `thread_*` + 5 `dream_*`)". Real: **82 listados / 88 definidos / 49 core** (`vantadb-mcp/src/handlers/tools.rs:83-1053` = 49 base; 33 extendidos listados; `config.rs:10-13` y el test `vantadb-mcp/tests/mcp_tests.rs:4961` `assert_eq!(full_tools.len(), 82)`).
- **Por qué importa:** Es la primera frase del skill, repetida en el heading que el agente trata como "fuente de verdad". Un agente que presupuesta tokens de schema o un usuario que compara `tools/list` contra el doc concluye que el servidor está roto.
- **Fix:** `82 listed (88 defined)`, `49 core`, `9 code_*` (3 listados), `other 33 listed / 39 defined`.
- **Cómo verificar:** `cargo test -p vantadb-mcp test_mcp_tool_profiles` → 82/39/38/22.

#### S5-B2 | P1 | tools absorbidas presentadas como disponibles
- **Ubicación:** `.opencode/skills/vantadb-mcp/SKILL.md:133`; `references/api-reference.md:77-91`
- **Qué afirma/rompe:** La tabla lista 8 tools `code_*` (`code_search, code_explore, code_callers, code_callees, code_impact, code_node, code_status, code_files`) como Available MCP Tools. Desde WIRE-02, `code_callers`/`code_callees`/`code_impact`/`code_node`/`code_status`/`code_files` son **dispatch-only absorbidos**: nunca aparecen en `tools/list` en ningún perfil (`handlers/tools.rs:1315-1331`, `:1190-1239`); resuelven a `code_explore`/`capabilities`/`code_search`. Y el skill **omite `code_index`**, que sí es una tool listada en `full` (`code_index.rs`, cableada en `handlers/tools.rs:1074`).
- **Por qué importa:** Un agente que sigue el skill llama `code_impact`/`code_node` y no las encuentra; y nunca aprende que existe `code_index`, con lo que nunca indexa un repo al grafo aunque la tool esté disponible.
- **Fix:** Separar la fila en "3 listadas (`code_search`, `code_explore`, `code_index`)" y "6 dispatch-only absorbidas — llama a su canónica", y añadir `code_index` al conjunto listado (espejo de `docs/api/MCP.md:216-227`, que sí documenta el set absorbido).
- **Cómo verificar:** `Select-String -Path .opencode/skills/vantadb-mcp/SKILL.md -Pattern 'code_index'` → 1 hit; `code_callers` solo dentro de la nota dispatch-only.

#### S5-B3 | P1 | el smoke test MCP no puede pasar
- **Ubicación:** `.opencode/skills/vantadb-mcp/scripts/test-mcp.py:58-62` (`EXPECTED_TOOLS = {"full": (85,85), "dev": (36,36), "memory": (20,20)}`), `:21-26` (docstring), `:112-117` (env del hijo), `:165` (selección de perfil), `:286` (return 1)
- **Qué afirma/rompe:** Tres defectos que se acumulan: (1) los conteos esperados están obsoletos (real 82/38/22, + `agent` 39); el script cita como fuente de verdad `mcp_tests.rs::test_mcp_tool_profiles`, que afirma 82/39/38/22; (2) `profile = os.environ.get("VANTADB_MCP_PROFILE","full")` decide el bucket esperado pero el script **nunca exporta** la variable al hijo (`env=None` para `vanta-cli`, `dict(os.environ, VANTADB_STORAGE_PATH=.)` para `vantadb-server`) → el servidor usa su default `agent` (39 tools) y el script compara 39 contra `[85,85]` → drift → exit 1; (3) `agent` no es clave de `EXPECTED_TOOLS`, así que `VANTADB_MCP_PROFILE=agent` hace abortar en `:166-171`.
- **Por qué importa:** Es la **única verificación ejecutable** que el skill da al usuario ("Test the MCP server"). Falla contra un servidor sano, y su propio mensaje (`:227-229`) culpa al binario → manda a recompilar en bucle. Tampoco puede detectar la deriva real para la que existe.
- **Fix:** `EXPECTED_TOOLS = {"agent": (39,39), "full": (82,82), "dev": (38,38), "memory": (22,22)}`; exportar `VANTADB_MCP_PROFILE=profile` en el env del hijo en ambas ramas; default esperado `"agent"`. Mejor: importar los conteos del test Rust.
- **Cómo verificar:** `VANTADB_MCP_BIN=target/debug/vantadb-server.exe python .opencode/skills/vantadb-mcp/scripts/test-mcp.py` → exit 0, "5/5 passed". Nota: `scripts/validate-docs-coverage.ps1:194` excluye este fichero del hash-mirror, por eso la deriva no se detectó.

#### S5-B4 | P1 | perfil por defecto y conteos del skill
- **Ubicación:** `.opencode/skills/vantadb-mcp/references/configuration.md:125-131`
- **Qué afirma/rompe:** "Accepted values: `full` (default, 85 tools), `dev` (36: …), `memory` (20: …)". Real (`vantadb-mcp/src/config.rs:10,22,27,30,33`): default **`agent` (39)**, `full` 82, `dev` 38, `memory` 22. El perfil `agent` — el que recibe todo usuario que no haga nada — **no se menciona en absoluto**. El skill tampoco explica el gate de perfil: un tool conocido fuera de perfil devuelve `-32601` "Tool not found: `<name>` (not in profile `<profile>`)" (`handlers/tools.rs:1368-1374`).
- **Por qué importa:** El usuario que pone `VANTADB_MCP_PROFILE=dev` ve 38 tools, no 36; el que lee "default = full" espera `wiki_ingest`/`skill_*`/`code_index` y no están → "tool not found" sobre tools que el skill dice que existen.
- **Fix:** Reescribir la sección con la tabla real (agent 39 default / full 82 / dev 38 / memory 22), añadir la fila `agent` con su membresía, y documentar el comportamiento `-32601 … (not in profile …)`.
- **Cómo verificar:** Contrastar cada número con `vantadb-mcp/src/config.rs:10-34` y `handlers/tools.rs:1097-1308`.

#### S5-B6 | P1 | plantillas de config con `~`
- **Ubicación:** `skills/vantadb-mcp/assets/{claude-desktop-config.json,cursor-config.json,opencode-config.json}`; `skills/vantadb-mcp/assets/install/*` (copia `.opencode` idéntica por hash)
- **Qué afirma/rompe:** Las plantillas que el usuario copia a su cliente MCP usan `~/.vantadb` como `--db`. El propio `docs/api/MCP.md` (sección "Starting the server") advierte: **"Never use `~` in these JSON configs"** — los clientes spawnean el proceso sin shell, `~` llega literal y la apertura falla.
- **Por qué importa:** El artefacto distribuible manda al usuario al camino que el mismo repo declara roto. El fallo aparece como "no such file `~/.vantadb`", no como un error de path.
- **Fix:** Plantillas con ruta absoluta (`C:/Users/<you>/.vantadb`, `/Users/<you>/.vantadb`) o con placeholder explícito `<ABSOLUTE_PATH>` + nota.
- **Cómo verificar:** `Select-String -Path skills/vantadb-mcp/assets/**/*.json -Pattern "~"` → 0 hits.

### 3.5 Skills de usuario (1 adicional ya contado arriba; ver §3.4)

### 3.6 Docs (14)

#### D-02 | P1 | `VantaConfig` no existe
- **Ubicación:** `docs/user/operations/CONFIGURATION.md:15,17,176-177,185,187` (+ `GRAFANA_SETUP.md:55`, `EDITOR_INTEGRATIONS.md:352`, `hardening.md:98`)
- **Qué afirma/rompe:** Toda la referencia normativa de configuración usa `VantaConfig` (21 ocurrencias). El tipo público es `vantadb::Config` (`src/lib.rs:177`). Búsqueda de `VantaConfig` en `src/`, `vantadb-server/`, `vantadb-python/`, `vantadb-mcp/`, `vanta-memory/` → **cero hits**; el símbolo se eliminó en 0.6.0, como dice el propio repo en `docs/api/EMBEDDED_SDK.md:15`.
- **Por qué importa:** Es la **referencia normativa de configuración** (`docs/api/VERSIONING.md:44` apunta ahí el contrato del CLI). El snippet Rust no compila y cada búsqueda de campo lleva a un tipo inexistente. Es el mayor drift de tipo del árbol: 79 ficheros de doc siguen diciendo `VantaConfig`; las docs de API se actualizaron, las de user/operations no.
- **Fix:** Renombrar global `VantaConfig` → `Config` en esos ficheros; §1 → "## 1. `Config` Reference".
- **Cómo verificar:** `Select-String -Path docs/api,docs/user -Recurse -Pattern "VantaConfig"` → 0 hits; compilar el snippet de `CONFIGURATION.md:184-195`.

#### D-03 | P1 | versión del registro MCP
- **Ubicación:** `docs/api/MCP.md:676`
- **Qué afirma/rompe:** "| version | tracks `[workspace.package].version` (`0.5.0` as of this doc) |". `Cargo.toml:4` → `0.9.0`; `server.json:6` → `"0.9.0"`; y el **mismo fichero** dice en `:11` "Current MCP implementation version: 0.9.0".
- **Por qué importa:** Autocontradictorio dentro de un fichero, y el valor es 4 minors atrasado. El `sync-workspace-version.mjs` solo reescribe la línea de la cabecera (`transformMcpMd`), por lo que esta deriva se repite en cada release.
- **Fix:** `0.9.0 as of this doc` — y añadir `:676` al patrón de rewrite del script.
- **Cómo verificar:** `Select-String -Path docs/api/MCP.md -Pattern "0\.5\.0"` → 0 hits.

#### D-04 | P1 | quickstart pineado a 0.8.0
- **Ubicación:** `docs/user/QUICKSTART.md:5,11,54-55,113,143,161`
- **Qué afirma/rompe:** "This quickstart validates the current **v0.8.0** boundary from pre-built artifacts"; `pip install vantadb-py==0.8.0`; `pip install ./dist/vantadb_py-0.8.0-*.whl`; "TestPyPI carries the same version (`0.8.0`)". El árbol es 0.9.0 (`Cargo.toml:4`, `docs/CHANGELOG.md:18`).
- **Por qué importa:** El glob `vantadb_py-0.8.0-*.whl` es un glob literal, no un rango: no matchea el wheel 0.9.0 que produce el build → el comando documentado falla. El pin `==0.8.0` instala dos minors por detrás del crate que el resto del repo documenta.
- **Fix:** Version-agnóstico: `pip install ./dist/vantadb_py-*.whl` y `pip install "vantadb-py>=0.9.0"`.
- **Cómo verificar:** el glob resuelve contra un `dist/` real.

#### D-05 | P1 | procedencia de benchmarks rota
- **Ubicación:** `docs/user/operations/BENCHMARKS.md:20,22`
- **Qué afirma/rompe:** "These metrics are run under the heavy certification suite (**`heavy-certification.yml`**, job `stress-protocol`)" y "same invocation as the `stress-protocol` job in `heavy-certification.yml`". **No existe** `heavy-certification.yml` en `.github/workflows/` (20 ficheros; solo existe `docs/dev/workflow/heavy-certification-50.md`, prosa). El job está en `.github/workflows/certification.yml:89-103` con la línea `run:` byte-idéntica a la que el doc cita.
- **Por qué importa:** El doc nombra el fichero exacto que un lector debe abrir para re-verificar los números; no existe → la cadena de provenance que exige la Regla 11 está rota. El *comando* es correcto, por eso sobrevivió.
- **Fix:** `:20` y `:22` → `certification.yml`.
- **Cómo verificar:** `Select-String -Path .github/workflows/*.yml -Pattern "stress_protocol"` → `certification.yml:89`.

#### D-06 | P1 | recall certificado atribuido al dataset equivocado
- **Ubicación:** `docs/user/operations/BENCHMARKS.md:28`
- **Qué afirma/rompe:** "| **Recall@10 (Block 1)** | 10K vectors, 128d, Cosine | **0.9560** | Certified |". El harness real se titula literal *"BLOCK 1 - GROUND TRUTH RECALL (**50K**/128D)"* (`tests/certification/stress_protocol.rs:221`) y usa `gen_dataset(50_000, DIMS, SEED)` (`:207`) con config `m=16, ef_s=250` que no comparte con el índice de 10K. Las filas "Scaling Recall" (`:29-31`) sí atribuyen bien 10K/50K/100K porque vienen del Block 2.
- **Por qué importa:** El número certificado se atribuye a otro tamaño de dataset. Un lector que lo compare contra la cifra 10K de un competidor está comparando workloads distintos. El marcador "Certified" agrava la atribución errónea.
- **Fix:** `:28` → "50K vectors, 128d, Cosine"; si el valor vino de otra corrida, re-ejecutar y republicar.
- **Cómo verificar:** `Select-String -Path tests/certification/stress_protocol.rs -Pattern "BLOCK 1"` → `(50K/128D)`.

#### D-07 | P1 | variables de entorno muertas en EMBEDDINGS.md
- **Ubicación:** `docs/api/EMBEDDINGS.md:129-130`
- **Qué afirma/rompe:** `VANTA_EMBEDDING_PROVIDER=local` / `VANTA_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx`. Ninguna de las dos se lee: solo aparecen en comentarios de deprecación (`src/config.rs:296,691`). Las vivas son `VANTADB_EMBEDDING_PROVIDER` (`src/config.rs:1110`) y `VANTADB_LOCAL_MODEL` (`src/config.rs:1090`). `docs/user/operations/CONFIGURATION.md:106-120` lista las `VANTA_*` en la tabla *"legacy env vars are deprecated and no longer read by VantaDB"*. El propio `docs/api/EMBEDDINGS.md:45` usa la forma correcta `VANTADB_*`.
- **Por qué importa:** Bloque copy-paste; ponerlas tiene **efecto cero**: el provider se queda en `ollama` y el path del modelo en su default hardcoded. Misconfiguración silenciosa, no error. El doc se contradice 84 líneas antes.
- **Fix:** `:129` → `VANTADB_EMBEDDING_PROVIDER=local`; `:130` → `VANTADB_LOCAL_MODEL=…`.
- **Cómo verificar:** `Select-String -Path src -Recurse -Filter *.rs -Pattern "VANTA_EMBEDDING_PROVIDER|VANTA_LOCAL_MODEL"` → solo la tabla de deprecación.

#### D-08 | P1 | contador de tools en el backlog
- **Ubicación:** `docs/dev/Backlog.md:356` (FIND-98)
- **Qué afirma/rompe:** "Binario `vanta-cli` instalado stale (**79 vs 85 tools**)" / "fuente 87→85 tras API-04 dedup". Real: 82 listados (88 definidos − 6 absorbidos), según `vantadb-mcp/tests/mcp_tests.rs:4961` y `vantadb-mcp/src/config.rs:27`.
- **Por qué importa:** La fila del backlog es el action item de un defecto de paridad, y está abierta. Quien la ejecute comparará contra 85 y encontrará una discrepancia de 3 tools que no existe, perseguindo un "API-04 dedup to 85" fantasma. Los números correctos son 79 vs 82.
- **Fix:** `(79 vs 82 tools)` y "fuente 82 (88 definidas − 6 absorbidas WIRE-02)".
- **Cómo verificar:** `cargo test -p vantadb-mcp test_mcp_tool_profiles`.

#### D-09 | P1 | contador que contradice su propia cita
- **Ubicación:** `docs/dev/Backlog-negocio.md:101` (DX-10)
- **Qué afirma/rompe:** "pipe verificado: **80 tools listadas - 86 definidas - 6 absorbidas** WIRE-02, `docs/api/MCP.md:198`". La línea citada dice "**82** listed tools in 8 families (88 defined - 6 WIRE-02 absorbed…)" y su tabla de familias (`:200-209`) suma 82 y 88.
- **Por qué importa:** Cita la fuente correcta y aun así la contradice → evidencia de que el backlog se mantiene a mano y no se re-deriva. Quien abra la línea citada tiene la respuesta al instante.
- **Fix:** "82 tools listadas - 88 definidas - 6 absorbidas".
- **Cómo verificar:** la aritmética es autoconsistente: 88 − 6 = 82.

#### D-10 | P1 | default de `max_blocking_threads`
- **Ubicación:** `docs/user/operations/CONFIGURATION.md:37` vs `src/config.rs:1050-1052` (`default_max_blocking = available_parallelism().map(|n| n.get()*2).unwrap_or(16)`), aplicado en `src/config.rs:1209`
- **Qué afirma/rompe:** Default documentado **16**. El real es **2 × nº de CPUs lógicos**; 16 es solo el fallback cuando `available_parallelism()` falla. El mismo default de `wal_buffer_size` y `max_connections` (`src/config.rs:1214`, `default_max_blocking * 2`) deriva de él.
- **Por qué importa:** En un runner de 32 cores el default real es 64, no 16 — 4× en el pool de bloqueos, que a su vez dimensiona el pool de queries HTTP. Un operador dimensiona contra "16" y se equivoca por 4×.
- **Fix:** Default: `2 × available_parallelism()` (fallback 16).
- **Cómo verificar:** `Select-String -Path src/config.rs -Pattern "available_parallelism"`.

#### D-11 | P1 | default de `wal_buffer_size`
- **Ubicación:** `docs/user/operations/CONFIGURATION.md:64` vs `src/config.rs:344` (`wal_buffer_size: None`) y `src/config.rs:1406-1413` (`parse_env_or::<u32>("VANTADB_WAL_BUFFER_SIZE", 0)…filter(|&n| n>0)` → 0 → `None`)
- **Qué afirma/rompe:** Default documentado **65536 (64KB)**. El real es `None` (default del OS). La fila se contradice a sí misma: la columna Default dice 64 KB, la columna Description dice "None = OS default".
- **Por qué importa:** Con 4 shards por defecto, la diferencia 256 KB vs 0 es un cambio real de comportamiento de I/O del WAL.
- **Fix:** Default: `None` (OS default).
- **Cómo verificar:** `Select-String -Path src/config.rs -Pattern "wal_buffer_size"` → `:344` y `:1406` dan `None`.

#### D-12 | P1 | prerrequisito de versión del CLI
- **Ubicación:** `docs/api/MCP.md:21`
- **Qué afirma/rompe:** "The VantaDB CLI (`vanta-cli` — **0.5.0**) installed and on PATH". El workspace es 0.9.0.
- **Por qué importa:** Prerrequisito pineado 4 releases atrás, en la lista "Getting Started → Requirements". Un usuario en 0.5.0 obtiene un `tools/list` pre-WIRE-02/pre-API-04 — sin las familias `code_*`/`wiki_*`/`skill_*` que el resto del documento describe.
- **Fix:** "(`vanta-cli` — **0.9.0**)" o "≥ 0.9.0".
- **Cómo verificar:** `Select-String -Path docs/api -Recurse -Pattern "0\.5\.0"` → 0 hits.

#### D-13 | P1 | tabla de versiones soportadas congelada
- **Ubicación:** `SECURITY.md:3-11` (+ `:13` "Patch releases are published for the latest minor version")
- **Qué afirma/rompe:** `0.5.x` Active development · `0.4.x` Security patches only · `< 0.4` Not maintained. La versión actual es **0.9.0** (`Cargo.toml:4`; `docs/CHANGELOG.md:18`), con cuatro minors publicados desde 0.5.0.
- **Por qué importa:** Esta tabla es el **contrato de soporte de parches de seguridad**. Tal como está, 0.6.x–0.9.x quedan sin clasificar, y 0.4.x aparece como "recibe parches" mientras la línea 13 dice que los parches van a la minor actual (0.9.x) — el propio fichero se contradice. Un usuario en 0.9.0 no puede saber si está soportado; un usuario en 0.4.x cree que lo está.
- **Fix:** Reescribir para la línea 0.9.x y enunciar la política en términos de "minor actual" (no una lista congelada) para que no vuelva a pudrirse.
- **Cómo verificar:** las minors de la tabla coinciden con `Cargo.toml`.

#### D-14 | P1 | metadatos de cita
- **Ubicación:** `CITATION.cff:9-10` (y la instrucción en `:3`)
- **Qué afirma/rompe:** `version: 0.5.0` / `date-released: 2026-08-01`. El propio fichero dice: *"Al publicar una release nueva, actualizar `version` + `date-released`"*.
- **Por qué importa:** El botón "Cite this repository" de GitHub renderiza `CITATION.cff` literal. Un usuario que cita un resultado de la era 0.9.0 citaría la 0.5.0 — cuatro releases y un año antes. La procedimiento está documentada y no se aplicó en cuatro releases → no hay gate que la fuerce.
- **Fix:** `version: 0.9.0`, `date-released: 2026-10-20`; añadir el fichero a un check de CI.
- **Cómo verificar:** comparar `version` con `Cargo.toml`.

#### D-15 | P1 | env vars reales sin documentar
- **Ubicación:** `docs/user/operations/CONFIGURATION.md:19-92` y `:122-129`
- **Qué afirma/rompe:** El doc se presenta como la referencia completa de env vars ("All configuration fields available in `VantaConfig` (Rust) **and via environment variables**"). Seis variables leídas por el código no aparecen en ninguna de las dos tablas: `VANTADB_QUIET` (`src/hardware/mod.rs:303`), `VANTADB_SILENT` (`src/hardware/mod.rs:304`), `VANTADB_PROFILE_PATH` (`src/hardware/mod.rs:99,732`), `VANTADB_SCHEDULER_INTERVAL_MS` (`vantadb-server/src/scheduler.rs:70`), `VANTADB_INGEST_CONFIG` (`vantadb-server/src/scheduler.rs:93`; `vantadb-mcp/src/wiki.rs:389`), `VANTADB_SYNC_MODE` (`src/cli_handlers/config.rs:216`; la fila `:42` dice Env Var "-"). Las tres primeras además las **escribe** el propio binario (`src/bin/vanta-cli.rs:34`, `vantadb-server/src/main.rs:55`).
- **Por qué importa:** `VANTADB_INGEST_CONFIG` selecciona el TOML de ingest que leen tanto el scheduler como el wiki ingester del MCP: un operador no puede descubrirlo, ni apuntarlo a un path no-default, ni auditar qué config honra el servidor. `VANTADB_SCHEDULER_INTERVAL_MS` controla la cadencia del pase en background.
- **Fix:** Añadir las 5 filas a la tabla "Environment Variables Outside `VantaConfig`" (`:124-129`) y corregir `:42`.
- **Cómo verificar:** diff de los `env::var("...")` de `src/` + `vantadb-server/` + `vantadb-mcp/` contra las dos tablas — todo hit debe aparecer.

### 3.7 READMEs (13)

> Detalle completo en el informe del subagente S7. Resumen con la misma evidencia:

- **S7-01 | P1 | `README.md:329,349,359`; `README_ES.md:314,334,344`** — Documenta `vanta-cli server start --port 8080`. No existe ese subcomando: `src/cli.rs:338` es una variante clap plana (`Server { http: bool, mcp: bool, port, host, require_auth, allow_insecure, dashboard_dir }`). clap aborta con `error: unexpected argument 'start'`. El mismo README lo dice bien en `:310` (`vanta-cli server --http`). *Fix:* reemplazar por `vanta-cli server --http` en los 6 sitios. *Verificar:* `cargo run --bin vanta-cli -- server --help` (no aparece `start`).
- **S7-02 | P1 | `README.md:9,10`; `README_ES.md:9,10`** — Badges a `gate-docs.yml` y `sec-codeql.yml`, workflows que **no existen** (el de docs es `docs-gates.yml`; no hay workflow CodeQL con ese nombre). *Fix:* repoint o eliminar. *Verificar:* `Get-ChildItem .github/workflows -Name`.
- **S7-03 | P1 | `README.md:219`; `README_ES.md:218`** — Enlace a `actions/workflows/heavy-certification.yml` (404); el real es `certification.yml`. *Verificar:* listar workflows de `origin/main`.
- **S7-04 | P1 | `README.md:139`; `README_ES.md:138`** — Cita `ci-examples.yml` como suite de smoke; los ejemplos los corre `.github/workflows/ci-fleet.yml:162-180`. *Fix:* `ci-examples.yml` → `ci-fleet.yml`.
- **S7-05 | P1 | `integrations/README.md:48`** — Justifica los pins contra `.github/workflows/adapters-compat.yml`, fichero inexistente. *Verificar:* `Get-ChildItem .github/workflows -Name | Select-String adapter`.
- **S7-06 | P1 | `vantadb-node/README.md:15`** — Enlaza `docs/dev/tasks/TS-12.md`, que no existe en `docs/dev/tasks/`, `complete/` ni `closed/`. *Verificar:* `Get-ChildItem -Recurse -Filter TS-12.md` → vacío.
- **S7-07 | P1 | `integrations/README.md:12-14`** — Afirma "ninguno está en PyPI (404 x7, verificado 2026-10-06)". **Verificado live hoy**: `vantadb-crewai|dspy|haystack|langchain|letta|llamaindex|mem0` → HTTP 200, todos en **0.5.0**. *Fix:* reescribir el bloque de estado. *Verificar:* `GET https://pypi.org/pypi/vantadb-crewai/json` → 200, `info.version=0.5.0`.
- **S7-08 | P1 | `integrations/{crewai,dspy,haystack,langchain,letta,llamaindex,mem0}/README.md:7-9`** — Banner "**Not on PyPI yet.**" falso en los 7; contradice la sección "Install from PyPI" del mismo fichero.
- **S7-09 | P1 | `providers/{litellm,ollama,openai}/README.md:7-9`** — Banner "Not on PyPI yet" falso: `vantadb-litellm|ollama|openai` → 200, **0.8.0**.
- **S7-10 | P1 | `integrations/mem0/README.md:32-35,41,43`** — El quickstart llama `store.search("vector database")` (real: `search(query, vectors, top_k=…)`, `vectors` posicional requerido — `integrations/mem0/vantadb_mem0/vectorstore.py:172`) e indexa `r["payload"]` (`OutputData` con `__slots__` sin `__getitem__` → `TypeError`). La API table documenta `search(query, k=4)` y `list(user_id=None, limit=100)`; ninguna de las dos existe (`vectorstore.py:172,278`).
- **S7-11 | P1 | `integrations/ollama/README.md:22,28-38`; `integrations/openai/README.md:22,28-39`** — El "From PyPI" instala el **provider Rust** (`providers/ollama/vantadb_ollama.pyi`: `embed/store/search/get/list/delete/list_namespaces`, sin `add_texts`/`similarity_search`/`page_content`) y el quickstart de abajo usa la API del **adaptador Python** (`integrations/ollama/vantadb_ollama/vectorstore.py:15`, `EmbeddingVectorStore`). `TypeError: VantaDBOllama.__init__() got an unexpected keyword argument 'model'`. Colisión de nombre PyPI **y** de módulo — exactamente lo que FIND-273 quería evitar. *Fix:* separar los dos quickstarts.
- **S7-12 | P1 | `vantadb-python/README.md:120`; `vantadb-ts/README.md:237`** — Enlace a `../docs/user/QUICKSTART.md#4-real-embeddings-optional`; la sección real es `## 7. Optional: Local Embeddings (\`embed-local\`)` (`docs/user/QUICKSTART.md:251`). *Verificar:* `Select-String -Path docs/user/QUICKSTART.md -Pattern "^## "`.
- **S7-13 | P1 | `README.md:70` vs `README_ES.md:66-68`** — EN: "`import vantadb_py` still works but emits a `DeprecationWarning`". ES: "sigue disponible sin cambios". El código emite el warning (`vantadb-python/vantadb_py/__init__.py`). Dos behaviours distintos publicados en dos idiomas.

### 3.8 CI / release (2)

#### AUD-01 | P1 | `integrations/**` no dispara los checks requeridos → merge bloqueado
- **Ubicación:** `.github/workflows/ci-fleet.yml:11-54` (`paths:` de `push`/`pull_request`); `.github/workflows/docs-gates.yml:16-58`
- **Qué afirma/rompe:** El ruleset `main` (id 16844590, `enforcement: active`, `strict_required_status_checks_policy: true`) exige, entre otros, `"Cross-language conformance (Py/Node/WASM)"` (job `parity` de `ci-fleet.yml`) y 6 checks de docs de `docs-gates.yml` ("Internal links resolve", "Frontmatter schema, kind agreement, orphans", "Generated indexes are current", "actionlint (workflows lint + inventory gate)", "Check npm package versions match workspace", "Check API Docs Version"). `integrations/**` está en los `paths:` de `ci-rust.yml:20,38` pero **no** en los de `ci-fleet.yml` ni `docs-gates.yml` (verificado: `Select-String -Path .github/workflows/*.yml -Pattern "^\s+- 'integrations/"` → solo `ci-rust.yml:20,38` y `demo-acceptance.yml:32-33,52-53`).
- **Por qué importa:** Un PR que toque **solo** `integrations/**` (bump de adaptador, fix de adaptador) dispara `ci-rust.yml` pero ni `ci-fleet.yml` ni `docs-gates.yml` → los 7 checks requeridos **nunca reportan** → el merge a `main` queda bloqueado para siempre. Es exactamente la clase de bug que el commit `0253a0c7` de esta misma rama dice arreglar ("un PR que solo tocara CI/docs nunca lo disparaba… el merge quedaba bloqueado para siempre — PR #242 y #257"), replicada para `integrations/**`. Y los adapters son 9 de los paquetes publicables del monorepo.
- **Fix:** Añadir `- 'integrations/**'` (y `- 'vanta-proxy/**'`, `- 'dev-tools/**'`, `- 'examples/**'` a `ci-fleet.yml`) y `- 'integrations/**'` a `docs-gates.yml`. Alternativa más robusta: un job "always-green" que emita todos los checks requeridos.
- **Cómo verificar:** abrir un PR que toque solo `integrations/langchain/pyproject.toml` y comprobar que reportan los 7 checks; o `gh api repos/ness-e/Vantadb/rulesets/16844590` →RequiredStatusChecks y comparar con los `paths:`.

#### AUD-02 | P1 | el gate de semver no gatea
- **Ubicación:** `.github/workflows/ci-rust.yml:94-136` (job `semver-checks`); `.github/workflows/ci-rust.yml:135`
- **Qué afirma/rompe:** El comentario del job dice: *"RELEASE-01: gate so a breaking change in the public API can never publish"* y `:96` *"release-plz.toml has semver_check = true, but nothing ran cargo-semver-checks as a CI gate — this job closes that gap"*. Pero el paso que hace el check lleva `continue-on-error: true # CATEGORY: INFORMATIONAL` (`:135`): `cargo semver-checks -p vantadb --release-type minor`. Solo falla si la *herramienta* no está (`cargo semver-checks --version`, `:132`).
- **Por qué importa:** El gate que la documentación (`docs/dev/workflow/ci-rust-10.md:25`, `docs/dev/avance/activo/operaciones.md:61` "gate pre-publish obligatorio") presenta como el que impide publicar breaking changes **no puede**. Un breaking change )-> el job queda verde -> el publish continúa. La Regla 2 de AGENTS.md prohíbe `continue-on-error` salvo CATEGORY tags, y la categoría usada aquí ("INFORMATIONAL") contradice el propósito declarado del job.
- **Fix:** Quitar `continue-on-error` del paso `:135` (o mover el check a un job *blocking* y dejar el paso informativo como `INFORMATIONAL` con otro nombre). Alternativa: la Regla 11/`release-plz.toml:43` (`semver_check = true`) ya da el veto real en el path de release — si es así, documentarlo como tal en vez de prometer un gate de CI que no existe.
- **Cómo verificar:** `Select-String -Path .github/workflows/ci-rust.yml -Pattern "continue-on-error"` y comprobar que el paso de semver NO lo lleva; un PR con breaking change debe dejar el check en rojo.

### 3.9 Adapters (5) — verificados por S7 (mismo bloque §3.7, S7-05..S7-11)

### 3.10 Área 4 (Providers) — sin hallazgos P1 propios; los P1 de providers están en §3.7 (S7-09, S7-11, S7-26)

---

## 4. P2 y P3 (60 + 41)

Ledger completo con evidencia `ruta:línea` en **el ledger de hallazgos**. Resumen por bloque:

### 4.1 P2 — 59 hallazgos

| Bloque | IDs | Count | Núcleo |
|---|---|---|---|
| Núcleo Rust | VDB-02..VDB-05 | 4 | escala de score Euclidean divergente por backend; bitset truncado a u32; WAL escribe registros >10 MB que el lector descarta; `MAX_K` sin clampear en el SDK → `Vec::with_capacity(top_k)` |
| Bindings | S3-A2..S3-A8 | 7 | `__all__` divergente; API de módulo fantasma; mapa de excepciones; resultados fabricados en TS; sparse descartado en WASM; gaps de capacidades en Node |
| Embeddings | S3-B3, B5..B11 | 8 | hint de recuperación inútil; `embed_texts` error duro en un camino y fallback en otro; nombre de modelo hardcodeado; health check satisfecho por otro modelo; benchmark que no infiere; brazo IQL limitado a 5 candidatos; `VECTOR_SEARCH()` inexistente; endpoint Ollama equivocado |
| MCP + Server | S5-A1..A3, A5..A8 | 7 | heading MCP.md que contradice su cuerpo; `thread_*` sin documentar; claim de `data.code` falso en 5 factories; 9 schemas de respuesta que no coinciden con el wire; `/dashboard` documentado como autenticado pero público; ruta HTTP viva no documentada |
| Skills usuario | S5-B5, B7..B10, B13 | 6 | env var `VANTA_DB` inexistente; api-reference omite 11 tools; 9 enlaces rotos; versión de protocolo MCP equivocada; setup script con `VANTADB_VERSION="0.5.0"`; 4 env vars MCP sin documentar |
| Docs | D-16..D-20 | 5 | flag `--allow-dummy-embeddings` inexistente; ejemplo `vantadb.VantaDB`; "2h" vs `timeout-minutes: 180`; `VANTADB_SYNC_MODE` ausente; ADR dice "dense-only" con sparse nativo |
| READMEs | S7-14..S7-28 | 15 | `src/sdk.rs` inexistente; anchors Quick Links rotos; sección North Star ausente en ES; definición de North Star divergente; flags del wizard ausentes en ES; `VantaDB` vs `Client`; `new Client()` con constructor privado; Node 18 vs `engines >=22.19`; `vantadb_python/` vs `vantadb_py/`; ADR-041 vs ADR-0047; firmas inexistentes en haystack/mem0/crewai; `cursor` vs `next_cursor`; `Config` no existe en Python |
| Harness / CI / Release | AUD-03, AUD-04, AUD-05, AUD-07, AUD-10, AUD-11, AUD-13 | 7 | cabecera OIDC obsoleta en `release.yml`; AGENTS.md afirma que ningún workflow usa `secrets.*`; policy `never-published` muerta en `check-npm-versions.mjs`; gates de cobertura parciales; `CONSTRAINTS.md` no está en la raíz; colisión de nombre PyPI `vantadb-openai`/`vantadb-ollama`; sync de manifiestos con skip silencioso |

### 4.2 P3 — 41 hallazgos

| Bloque | IDs | Count | Núcleo |
|---|---|---|---|
| Núcleo Rust | VDB-06..VDB-08 | 3 | `Commit` espurio en WAL; divergencia de `last_accessed` entre paths; error de lectura tragado en el pre-delete |
| Bindings/Embeddings | S3-A9, A10, B12..B17 | 8 | tablas cross-binding desfasadas; sin superficie de embeddings en Py/TS; "9 modelos" vs 10 en el manifest; `verify.py` no verifica lo que el doc dice; rutas relativas al CWD; `embed_batch` secuencial |
| MCP | S5-A9..A13, B14..B17 | 9 | header `x-vanta-user-key` sin declarar; claim de enforcement más ancho que el gate; `server.json` no instalable; `opencode.jsonc` con "85 tools perfil full"; panic inline → cuelgue; "2 resources" vs "2+2"; mirror `.opencode` sin assets; `docs/api/MCP.md` llamado "stub" |
| Docs | D-21..D-26 | 6 | ADR apunta a `src/node.rs` inexistente; 4.88x vs 5.08x; dos grafías de `status:`; `llms.txt` sin generador; ejemplos de versión atrasados; docstring del smoke test obsoleto |
| READMEs | S7-29..S7-40 | 12 | "File" vs "VantaFile"; badge Nightly ausente en ES; `vantadb_py` vs `vantadb`; firma posicional de crewai; knobs HNSW no expuestos; "Hybrid Search" sobre llamada solo-vector; rutas sin resolver; falta README de `vantadb-mcp`; pin 0.8.0; "tres conectores" indefinidos; sync/async inconsistente |
| Harness | AUD-08, AUD-09, AUD-17 | 3 | "196 skills en 197 dirs" vs 198/197 reales; autorreferencia de línea rota en AGENTS.md; `{path}` vs `{*path}` en el dashboard |

---

## 5. Coverage

### 5.1 Mapa subagente → área

| Subagente | Área cubierta | Estado | Informe |
|---|---|---|---|
| S1 | 11 · CI/release + 12 · release 0.9.0 en vuelo | **Sin informe** (log 0 B) | — |
| S2 | 1 · Núcleo Rust (`src/`) | **Completo** (28 KB, 8 hallazgos) | el informe del subagente S2 |
| S3 | 2 · Bindings + 3 · Embeddings | **Completo** (58 KB, 27 hallazgos) | el informe del subagente S3 |
| S4 | 4 · Providers + 5 · Adapters | **Sin informe** (log 0 B) | — |
| S5 | 6 · MCP + Server + 7 · Skills de usuario | **Completo** (57 KB, 30 hallazgos) | el informe del subagente S5 |
| S6 | 8 · Docs (`docs/**`) | **Completo** (55 KB, 26 hallazgos) | el informe del subagente S6 |
| S7 | 9 · READMEs | **Completo** (56 KB, 40 hallazgos) | el informe del subagente S7 |
| S8 | 10 · `.opencode/` harness | **Sin informe** (log 0 B) — **cubierto por la wave W0** del orquestador (`audit-deep-w0-harness-20261020-033416.md`) | — |

### 5.2 Áreas SIN pase profundo de subagente (estado real a 2026-10-20)

Las tres áreas listadas aquí **no quedaron sin cubrir**: el estado se corrigió cuando los informes
llegaron tarde y cuando las waves del orquestador cubrieron lo que faltaba. Lo que sí queda abierto
no es "auditar X" sino **verificar en runtime** y **revisar con otro autor**.

1. **Área 11/12 — CI y release 0.9.0 en vuelo.** ~~S1 no produjo informe~~ → **S1 sí entregó**
   (`subagents/S1.md`, CI + release mechanics, 12 hallazgos) y la wave **W8** cubrió tooling,
   scripts y workflows. Cobertura adicional del orquestador: `release-manifest.json` contrastado
   1:1 con el fan-out de `release.yml:182-188`; `sync-workspace-version.mjs` ejecutado (`--check`
   → 22/22, `--self-test` → 12/12); `release-plz.toml`, `dependabot.yml` y los rulesets `main`/
   `develop` leídos vía `gh api`. Pendiente de ejecución (no de lectura): `scripts/verify-release.ps1`
   (necesita red y escribe un informe).
2. **Área 4 — Providers (`providers/{openai,ollama,litellm}`, `vanta-proxy/`).** ~~S4 no produjo
   informe~~ → **S4 sí entregó** (`subagents/S4.md`, providers + adapters, 16 hallazgos) y la wave
   **W4** cubrió embeddings + providers + integrations. Lo que la auditoría **no** afirma es haber
   medido en runtime la redacción de API keys, los budgets, el streaming SSE ni los timeouts/
   retries de los providers: son afirmaciones estáticas (código leído), no de ejecución.
3. **Área 10 — `.opencode/` harness.** S8 no entregó informe, pero la wave **W0** lo cubrió a
   fondo (`audit-deep-w0-harness-20261020-033416.md`): inventario real (198 dirs / 197 con
   `SKILL.md`, 11 agents, 13 commands, 14 rules, 24 references), coherencia comando↔skill↔agent↔rule,
   `permission: subagent` de los 11 agentes, índice de `rules/README.md` vs ficheros reales y
   `skills-index.json`. Pendiente de esa wave: `plugins/ponytail.ts`, `mcp/`, `mcp-profiles/`,
   `configs/`, `locks/` (sin auditoría profunda).

**Límite transversal de toda la campaña (no por área):** la verificación fue **estática** —no había
`cargo`/`rustc`/`nextest` en el PATH, así que `cargo nextest run --profile audit -p vantadb` no se
ejecutó— y **no hubo segundo autor**: el tool de subagentes (`task`) no existía en la sesión, el
orquestador ejecutó las 11 waves y W10 (verificación adversarial) fue auto-verificación. Los
hallazgos de comportamiento (orden de locks, O(n²), carreras) son hipótesis estáticas hasta que se
confirmen en runtime. Ver `docs/dev/reviews/audit-deep-sintesis-20261020-033416.md` §1 y el
backlog `FIND-410`.

### 5.3 Cobertura por área (detalle)

| Área | Profundidad | Notas |
|---|---|---|
| 1 · Núcleo Rust | Alta (subagente S2, 8 hallazgos) | Módulos `src/`: engine, wal, storage, index (flat/hnsw/distance), node/bitset, sdk/search, sdk/serialization, config, error, cli, txns. **No cubiertos en profundidad:** `src/parser/` (IQL grammar), `src/physical_plan/`, `src/agentic/`, `src/graphrag/`, `src/shred/`, `src/skills/`, `src/tui/`, `src/metrics/`, `src/serialization/`, `src/utils/`, `src/hardware/` (salvo lo citado), `src/crypto.rs` ( salvo la llamada), `src/llm.rs` (salvo lo citado), `src/embedding_health.rs` (salvo lo citado), `src/backends/`, `src/vfile_mmap`, `src/edge_index`, `src/shred`, `src/bin/`. `cargo` no está en PATH en este entorno → **ningún hallazgo fue verificado por compilador**; todos son source-level. |
| 2 · Bindings | Alta (S3, 10 hallazgos) | Paridad de superficie, `__all__`, stubs .pyi, TS/native, WASM sparse, gaps de Node. **No cubiertos:** `vantadb-ffi-core/` (no analizado), `vantadb-node/index.cjs`, `vantadb-ts/src/native.ts` completo, `vantadb-ts/examples/`. |
| 3 · Embeddings | Alta (S3, 16 hallazgos) | Providers, batching, cachés, dims, strict/dummy, manifest de modelos, benchmark. **No cubiertos:** `embeddings/download.py` y `embeddings/models/` en detalle, `setup-embeddings.ps1` (26 KB) más allá de 1 claim. |
| 4 · Providers | **Baja — sin subagente** | Ver §5.2. |
| 5 · Adapters | Media (via READMEs, S7; el pase de API pública de los 9 paquetes no ocurrió) | Se validaron firmas documentadas vs implementación (mem0, haystack, crewai, ollama, openai) y packaging. **No cubiertos:** `__init__.py` de los 9 paquetes, `pyproject.toml` completo de cada uno, `tests/` de los adaptadores, versionado de los 9 (confirmado 0.9.0 por el sync check). |
| 6 · MCP + Server | Alta (S5, 18 hallazgos) | Conteo de tools verificado por extracción mecánica sobre `handle_tools_list` + los 8 módulos de extend (**82 listados / 88 definidos / 49 base**), perfiles (agent 39 / full 82 / dev 38 / memory 22), absorbed_canonical, errores, openapi vs rutas (**37 paths, 1:1** con `src/server/router.rs`), `/dashboard` fuera del auth, ruta MCP-proxy no documentada. |
| 7 · Skills de usuario | Alta (S5, 16 hallazgos) | `skills/vantadb-mcp/**` y `skills/vantadb/**` + espejo `.opencode`. Verificado que **todos** los nombres de tool que el skill menciona existen en la impl. No cubiertos: `skills/vantadb/scripts/install-vantadb.sh` en detalle, `assets/hooks/**`. |
| 8 · Docs | Alta (S6, 27 hallazgos) | 1329 ficheros md escaneados; los checkers del repo (`check-links.mjs`, `check-docs.mjs`, `gen-index.mjs --check`, `validate-docs-coverage.ps1`) ejecutados read-only: **todos verdes** — lo que confirma que los gates existentes no cubren estas clases de deriva. Muestreado: `docs/dev/tasks/` (634), `docs/dev/avance/`, `docs/dev/research/`, ADRs. |
| 9 · READMEs | Alta (S7, 40 hallazgos) | EN + ES + 15 sub-README, leídos completos; además una tabla de ~35 claims **verificados correctos** (benchmarks, badges que sí resuelven, quickstarts ejecutables, comandos CLI). |
| 10 · `.opencode/` | Media-baja | Ver §5.2. |
| 11 · CI/release | Media (orquestador) | Ver §5.2. |
| 12 · Release 0.9.0 en vuelo | Media-alta (orquestador) | Los 9 commits, línea por línea; los mecanismos de publish, el guard de dependabot (verificado: aplicado **solo** a jobs no-publicadores), el sync de manifiestos (ejecutado y verde), el manifest de 15 registries, `semver_check` en `release-plz.toml:43` (sí vigente — es el veto real, a diferencia del job de CI), el path OIDC→token de crates.io, el fan-out de 7 workflows y sus inputs (`publish`, `package`, `publish_pypi` — todos existen). |

### 5.4 Limitaciones de la auditoría (estado a 2026-10-21)

> **Actualizado 2026-10-21.** Las tres limitaciones de la corrida original ya no aplican:
> `cargo` **sí** está disponible (`~/.cargo/bin`, 1.95.0 + nextest 0.9.133) y todos los hallazgos
> de esta campaña se han confirmado con compilador, `clippy`, `nextest` y `cargo fmt`; **S1 y S4
> sí entregaron** su informe (losRaw están versionados en `docs/dev/reviews/audit-subagent-s1..s7-*.md`)
> y **W0 cubrió el harness** que S8 no llegó a auditar. La cobertura real es **11/11 waves + 7
> subagentes** (§5.3 y el indice de esta carpeta).
>
> Limitaciones que **siguen vigentes**:
>
> - Los 12 modulos del core listados como "no cubiertos en profundidad" en §5.3 **siguen sin
>   auditar** (dos de ellos los esta tocando la tanda de cifrado en reposo): filas
>   `FIND-502`..`FIND-506` del backlog.
> - La campana fue **read-only sobre el codigo**: no ejecuto el gate de release
>   (`scripts/verify-release.ps1` escribe un informe y hace polling de registries) ni los
>   benchmarks de coste; el p99 canonico lo mide la wave W7 del plan de cifrado.
> - **Verificacion adversarial de un solo autor**: la wave W10 la ejecuto el propio
>   orquestador, sin un contexto limpio independiente. Lo suple la review con contexto fresco
>   (`vanta-review`) que se aplico a los cambios de correctitud de esta campana.
