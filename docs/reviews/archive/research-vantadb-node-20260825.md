# Research: vantadb-node — binding nativo napi-rs vs estado del arte

**Fecha:** 2026-08-25 · **Comando:** `/research vantadb-node` · **Modo:** read-only
**Competidor principal:** `@lancedb/lancedb`

## Score global: 4.8/10 — potencial alto bloqueado por distribución

| Dimensión | Score | Justificación |
|-----------|------:|---------------|
| DX de onboarding | **2** | NO publicado en npm (E404). Instalación solo compilando source. Sin README. |
| Completitud funcional | 6 | Core memoria+grafo+search completo (~30 métodos); faltan versioning/supersede/vacuum/purge/count que sí tienen python/MCP |
| Performance | 4* | Cero benchmarks; el "A/B interno vs WASM" prometido nunca se midió (*no medible = score por ausencia) |
| Robustez | 7 | close() con drain de in-flight, spawn_blocking, errores mapeados desde VantaError |
| Seguridad | 7 | Validadores parse_* en boundaries, u128 como decimal strings (safe integers) |
| Docs & ejemplos | **1** | Sin README, sin docs/api/NODE_SDK.md; index.d.ts con docstrings es lo único |
| Observabilidad | 3 | capabilities() existe; sin métricas/logging expuestos |
| Testabilidad | 4 | 8 tests (graph 5 + persistence 3) para ~30 métodos públicos |
| Paridad inter-bindings | 5 | Núcleo paritario; herramientas de ciclo de vida (versions/supersede/purge/vacuum) ausentes |
| Diferenciación | 7 | Único en su nicho: memoria semántica persistente + grafo dirigido + hybrid search en un addon nativo embebido |

## 1. Usuarios objetivo

Devs Node.js/backend que quieren engine embebido sin WASM: agentes Node con memoria
persistente local, CLIs, servicios pequeños que evitan un servidor dedicado.

## 2. Estándares del ecosistema npm (bindings nativos 2026)

El estándar de facto para distribuir addons nativos (validado contra napi.rs oficial
y la práctica de @lancedb/lancedb): **root package + paquetes por-plataforma como
`optionalDependencies`** (`napi create-npm-dirs` → `napi artifacts` → `napi prepublish`).
npm resuelve la plataforma automáticamente; cero toolchain en la máquina del usuario;
cero descargas postinstall. Alternativas inferiores: compilar source (pide toolchain)
o descargar binario en postinstall (fallos de red/redes privadas).

## 3. Competidores (datos npm reales, semanales)

| Paquete | Downloads/sem | Licencia | Nota |
|---------|--------------:|----------|------|
| `sqlite-vec` | 1.866.412 | MIT | Extensión sqlite; vector-only, sin grafo/memoria |
| `@orama/orama` | 1.303.371 | Apache-2.0 | Search engine JS puro (full-text + vector); sin persistencia nativa |
| `@lancedb/lancedb` | **998.960** | Apache-2.0 | **El espejo directo**: Rust core + napi-rs + prebuilds por plataforma (8 targets incl. musl), Arrow tipado fuerte, `engines >=18`, cpu/os declarados |
| `hnswlib-node` | 124.003 | — | Wrapper hnswlib; vector-only |
| `vectra` | 35.103 | — | JSON-local vector store (ligero, sin nativo) |
| `usearch` | 28.615 | Apache-2.0 | Índice ANN multi-lenguaje |

**Posición:** el nicho de VantaDB-node (memoria semántica + grafo + TTL + hybrid en
un solo addon embebido) no tiene competidor directo; los líderes de downloads son
vector-only o search-only. La brecha no es producto — es **distribución**.

## 4. Estado actual (evidencia interna)

- `package.json`: dual ESM/CJS ✓, types ✓, license ✓, **5 targets declarados**
  (win-msvc, lin-gnu, lin-arm64, darwin-x64, darwin-arm64) — pero sin musl.
  **Falta:** `engines`, `os`/`cpu`, repository, optionalDependencies de plataformas.
- `src/lib.rs` (817L): ~30 métodos públicos exportados + helpers privados. Calidad:
  validación en boundary (`parse_*`), u128→decimal strings, drain en close().
- `index.d.ts`: autogenerado con TSDoc completo ✓ — pero parámetros complejos como
  `any` (put/search/list/filter).
- Tests: **8 tests** (graph.test.ts ×5, persistence.test.ts ×3). Sin tests de
  search/explain_search/put_batch/list_namespaces/capabilities/close-drain.
- Distribución real: **un único `.node` win32-x64 local**. `git grep vantadb-node
  .github/workflows` = **0 resultados**: no hay pipeline de build/release.
- npm: `vantadb-node` → **E404** (nunca publicado).
- Docs: sin README, sin `docs/api/NODE_SDK.md`; sí aparece en BINDINGS_NAMESPACES.md.
- Historial: BND-03/BND-05/BND-FIXES/BND-07 en tasks/ (creación y fixes del binding).

## 5. Matriz de gaps vs competidor principal

| Capacidad | vantadb-node | @lancedb/lancedb |
|-----------|:---:|:---:|
| Publicado en npm con prebuilds multiplataforma | ❌ | ✅ (optionalDeps ×8) |
| README + docs + typedoc | ❌ | ✅ (docs.lancedb.com) |
| `engines`/`os`/`cpu` declarados | ❌ | ✅ |
| Tipado fuerte de requests (no `any`) | ❌ | ✅ (Arrow schema) |
| Memoria semántica con namespaces/TTL/versioning | ✅ | ❌ |
| Grafo dirigido + traversal filtrada | ✅ | ❌ |
| Hybrid search (vector+BM25) + explain | ✅ | ⚠️ (vector+SQL filter) |
| Benchmarks públicos reproducibles | ❌ | ⚠️ (parciales) |

> **Nota Regla 11:** los downloads/sem de la sección 3 provienen de
> `api.npmjs.org/downloads/point/last-week/<pkg>` (consulta del 2026-08-25,
> reproducible con ese comando). Cualquier claim de performance de competidores
> citado en este informe debe considerarse "claim sin evidencia" hasta medirlo
> con nuestro propio bench (PERF-BENCH-01).

---

## Método y fuentes

- **Internet:** api.npmjs.org (downloads + registry metadata de
  `@lancedb/lancedb`, `sqlite-vec`, `hnswlib-node`, `@orama/orama`, `usearch`,
  `vectra`); napi.rs/docs/deep-dive/release (modelo de distribución
  optionalDependencies); deepwiki lancedb nodejs-packaging; README oficial
  microsoft/playwright-cli n/a — fuentes primarias GitHub.
- **Interno:** lectura directa de `vantadb-node/{package.json, index.d.ts,
  src/lib.rs (817L), tests/}`; grep CI workflows (0 menciones);
  BINDINGS_NAMESPACES.md; tasks BND-03/05/FIXES/07.
- **No consultado (gap declarado):** issues/discussions de competidores,
  crates.io stats, benchmarks de terceros.

---

## Apéndice de hallazgos (H-NN) — entrada de la Fase D

| ID | Hallazgo | Categoría sugerida | Severidad | Esfuerzo | Ref |
|----|----------|--------------------|-----------|----------|-----|
| H-01 | **Nunca publicado en npm** — el paquete no existe para usuarios reales; sin workflow CI de build/release napi (0 menciones en .github/workflows) | AGREGAR | 🔴 Crítico | 🔴 | package.json, .github/workflows |
| H-02 | Sin README.md (declarado en `files` del package.json — npm publicaría roto) | APLICAR | 🟠 Alto | 🟢 | vantadb-node/ |
| H-03 | Falta `engines` (node ≥18), `os`, `cpu` en package.json (estándar LanceDB/napi) | APLICAR | 🟢 Medio | 🟢 | package.json |
| H-04 | Target linux **musl ausente** — Docker/Alpine (mayoría de despliegues Node) sin cobertura; LanceDB lo incluye | AGREGAR | 🟠 Medio | 🟡 | package.json napi.targets |
| H-05 | Tipado débil: `put(record: any)`, `search(request: any)`, `list(options: any)` — competidor usa schemas fuertes | MEJORAR | 🟠 Medio | 🟡 | index.d.ts |
| H-06 | Cobertura de tests: 8 tests para ~30 métodos; sin cubrir search/explain_search/put_batch/capabilities/drain-de-close | MEJORAR | 🟠 Medio | 🟡 | tests/*.test.ts |
| H-07 | Sin docs de producto: README + docs/api/NODE_SDK.md inexistentes; BINDINGS_NAMESPACES.md es la única mención | MEJORAR | 🟡 Menor | 🟡 | docs/api/ |
| H-08 | Gap de paridad API: faltan `versions`, `supersede`, `vacuum`, `compact_wal`, `purge_expired`, `count`, `delete_by_filter`, `similar_to_key`, `search_with_method`, `search_multi` (presentes en python SDK y MCP) | AGREGAR | 🟠 Medio | 🔴 | src/lib.rs |
| H-09 | Sin benchmark A/B node-native vs ts-WASM (la razón de existir de este módulo) — decisión de adopción ciega | OPTIMIZAR | 🟡 Menor | 🟡 | benches (inexistente) |
| H-10 | ESTRATEGIA: posicionar node-native vs ts-WASM — ¿matriz de decisión pública para usuarios (¿cuándo usar cada uno?) o mantener WASM como primario? | ESTRATEGIA | 🟠 Medio | 🟢 | README futuro |

## Score global: 4.8/10 · Veredicto: producto sólido INVISIBLE — la brecha es distribución, no capacidad.
