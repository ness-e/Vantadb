---
title: "Brecha de definición de producto (revisión definición vs repo shippeado, 2026-09-24)"
type: research
status: active
tags: [vantadb, research, producto, definicion, frontera, naming]
last_reviewed: 2026-09-24
aliases: []
related: [VISION.md, GO_TO_MARKET.md, EXPERIMENTAL_FEATURES.md]
---

# Brecha de definición de producto (2026-09-24)

> **Origen:** revisión del owner (chat externo, 2026-09-24) contrastando la definición oficial del producto contra lo que el repo realmente shippea (v0.7.0 `develop`). Formalizada y verificada contra el repo en esta fecha.
> **Conclusión:** el problema no es que falte definición — hay dos documentos explícitos (`SPEC.md` y `docs/user/operations/EXPERIMENTAL_FEATURES.md`). El problema es que **ninguna de las dos gobierna ya lo que se construye**: no tienen mecanismo de mantenimiento por release y el repo creció por superficies.
> **Arreglo propuesto:** fase **P55 — Frontera de producto** (DEF-01..08) + correcciones inmediatas aplicadas el 2026-09-24 a los claims falsos de `EXPERIMENTAL_FEATURES.md`.

## 1. Hay dos productos distintos compitiendo en la misma definición

- `README.md` dice: "embedded database engine for AI agents, local RAG pipelines, and edge applications".
- `SPEC.md` (contrato del MVP) dice: "memoria automática en agentes de código (OpenCode, Claude Code, Cursor, Codex) con UN comando y CERO configuración".
- Son targets distintos (builder de RAG/edge vs dev usando agentes de código) con éxito medible distinto. Nada en la definición resuelve cuál manda cuando compiten.
- **Resuelto parcialmente por decisión owner 2026-09-24:** los 3 tracks ICP (AI-IDEs vía MCP / devs local-LLM y privacidad / frameworks) se desarrollan a profundidad; el motor (memoria embebida gobernada) es el núcleo único y cada track es una puerta de entrada, no un producto separado. Cierre documental → **DEF-01** (alinear SPEC/README/VISION en una sola jerarquía).

## 2. La frontera de producto quedó desactualizada y ya no es verificable

`EXPERIMENTAL_FEATURES.md` se declara para "v0.1.x" (`last_reviewed: 2026-07-01`) mientras el changelog está en 0.7.0. Claims falsos verificados:

| Claim del doc | Realidad verificada 2026-09-24 |
|---|---|
| "Agent metacognition and automatic memory consolidation — Deferred" | `dream_consolidate`/`promote` + auto-consolidación MCP shippeados en 0.6.x (gobernanza manual pendiente → VER-07) |
| "Production PyPI publication and signed installers — Deferred" | `pip install vantadb-py` + npm `vantadb` publicados con attestations (firmas de binarios siguen pendientes) |
| "Unicode folding, stopwords, stemming — Deferred" | Advanced tokenizer es default (stemming + stopwords + ASCII folding) |
| "IQL parser/evaluator — Archived (2024-06-10)" | `/api/v2/query` vivo (`src/server/router.rs`) + SELECT/INSERT probados por E2E (REST-06); lo archivado fue la evaluación LISP legada |
| Desktop (Tauri), web console y vanta-proxy | No aparecen en ninguna categoría de la frontera |

- Una definición que no se puede confrontar contra el código deja de ser un límite y pasa a ser un documento histórico.
- `scripts/validate-docs-coverage.ps1` ya existe — la frontera puede chequearse en CI contra features Cargo y rutas reales → **DEF-03**.
- Correcciones inmediatas de los 4 claims falsos aplicadas en este cambio (2026-09-24); regeneración completa + categorías labs → **DEF-02**.

## 3. El alcance real desbordó el MVP declarado

MVP declarado: embedded + WAL + búsqueda + export/import + CLI/Python. Realidad changelog 0.6.0+: app desktop con bundles macOS/Linux, i18n y lentes; proxy con cost tracking, virtual keys, redacción de PII y caché semántica; providers OpenAI/Ollama/litellm; 87 tools MCP; server con JWT y rate limiting; WASM/Node/TS.

- Nada de eso es "envoltorio opcional del núcleo" — es otra categoría de producto (AI gateway + GUI) que consume el presupuesto de la promesa central ("1 comando, 0 config").
- Decisión de categorización (core-promise vs labs) → **DEF-07**.

## 4. Promesas del README que el código no cumple al pie de la letra

- "Fjall (default) o fallback a RocksDB" — **no hay fallback**: RocksDB requiere feature de compilación (no default) y `VANTADB_BACKEND` solo reconoce `rocksdb`/`memory` (`src/config.rs`). Es alternativa opt-in, no fallback automático.
- Exactamente el tipo de claim que un usuario comprueba y desconfía → **DEF-06** (reconciliación README↔BENCHMARKS↔código).

## 5. La evidencia de rendimiento se auto-descalifica

- Benchmark base: 10K vectores, un solo hilo; el propio README excluye BM25 por dar un outlier degenerado (p50 0.0035 ms).
- El "benchmark competitivo SIFT-1M" es before/after propio (2.14×–2.80×), sin recall@k ni comparación externa.
- Para un claim de "hybrid retrieval competitivo" no hay un solo número comparable externamente. O se publica comparativa reproducible con recall y memoria (**VER-08/VER-09**), o el claim baja a "en progreso" (**DEF-06**).

## 6. La promesa central es el componente más frágil del repo

- "Cero configuración" depende de ORT ≥1.27 + descarga de modelo; historial de incidencias: FIND-100 (abort por dylib incompatible), EMB-10→EMB-18 en un solo release. El propio `SPEC.md` lo admite como riesgo.
- La definición promete justo lo que más incidencias tuvo, sin SLO de instalación (tiempo máximo, telemetría de éxito, fallback visible) → **DEF-08**.

## 7. Churn de nombres y API en pre-1.0

- 9 artefactos con nombre propio (vantadb, vantadb-py, vantadb-node, vantadb-ts, vantadb-server/vanta-server según doc, vantadb-mcp, vanta-cli, vanta-proxy, vanta-memory); repo `ness-e/Vantadb` vs marca VantaDB; alias `VantaDB` eliminado en 0.6.0 (breaking en minor); `vantadb_py`→`vantadb` con deprecación.
- Cada rename parte tutoriales, búsqueda y código de terceros. Congelar nombres hasta 1.0 cuesta cero → **DEF-04** (coordinado con API-01..09, que ya es el último breaking pre-lanzamiento).

## 8. Falta lo que más define un producto

- No hay "por qué esto y no SQLite+sqlite-vec / LanceDB / Chroma / Qdrant embedded" en la frontera (solo en blog), ni north-star métrico — los success criteria de `SPEC.md` son de campaña (<30 min al primer recall), no de producto.
- Sin eso, cada tarea nueva se decide por entusiasmo, no por frontera → **DEF-05** (north-star + success criteria) + `COMPARISON.md` capa memory-as-a-service (P54/VER-09).

## 9. Plan de corrección (mapping a filas)

| Hallazgo | Fila | Fase |
|---|---|---|
| 1. Dos productos | DEF-01 | P55 |
| 2. Frontera stale | DEF-02, DEF-03 (+ fixes inmediatos 2026-09-24) | P55 |
| 3. Alcance desbordado | DEF-07 | P55 |
| 4. Claims README | DEF-06 | P55 |
| 5. Benchmarks auto-descalificados | VER-08, VER-09 (+ DEF-06) | P52 |
| 6. Zero-config frágil | DEF-08 | P55 |
| 7. Naming churn | DEF-04 | P55 |
| 8. Sin why-not-X ni north-star | DEF-05 (+ P54 ICP) | P55 |

## 10. Referencias

- `README.md`, `SPEC.md`, `docs/user/operations/EXPERIMENTAL_FEATURES.md` (v0.1.x stale; corregido 2026-09-24), `docs/user/COMPARISON.md`, `docs/user/operations/BENCHMARKS.md`, `docs/dev/strategy/GO_TO_MARKET.md`, `docs/dev/vision/VISION.md`.
- Plan de ejecución: `docs/dev/plans/2026-09-24-post-investigacion-integral.md` (§P55).
- Investigación de mercado que motiva la corrección: `docs/dev/strategy/VantaDB-Analisis-Arquitectura-Producto-Competencia.md` (§6 productos de memoria, §7 BDs multi-modelo).
