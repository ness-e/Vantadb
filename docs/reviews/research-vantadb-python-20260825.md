# INV-vantadb-python-01 — Investigación profunda: SDK PyO3 `vantadb-python`

> Generado por `/research vantadb-python` · 2026-08-25 · modo read-only.
> Registro: `.opencode/references/research-modules.md` fila 17. Plantilla: `research-module.md`.
> **Score global: 8.1 / 10**

## 1. Usuarios objetivo y flujo diario

**Devs Python/AI y frameworks de agentes** (LangChain/LangGraph, LlamaIndex,
AutoGen, CrewAI, DSPy, Haystack, Mem0 — ver ejemplos en `examples/python/`).
Flujo típico: `pip install` → quickstart <5 min → put/search híbrido → monitoreo
(`operational_metrics`) → cierre seguro. Fricciones esperadas del ecosistema:
tamaño de wheels nativas, compatibilidad de versión de Python, errores tipados
vs `ValueError` genérico, GIL durante búsquedas.

## 2. Estándares del ecosistema PyPI

- Distribución binaria nativa vía **maturin** (PyO3) es hoy el estándar de facto
  (lancedb, polars, pydantic-core). Wheels **abi3** (`cp311-abi3`) reducen la matriz de builds.
- `py.typed` + stubs `.pyi` son expectativa de paquetes serios (typing ecosystem).
- Clasificadores deben cubrir las versiones vigentes de CPython (3.11→3.14;
  3.14 estable desde octubre 2025 — fuente: pypi.org/project/python y release notes CPython).
- Trusted Publishing (OIDC) en PyPI es la recomendación actual de PyPA.

## 3. Competidores (fuentes oficiales PyPI)

| Competidor | Arquitectura | DX instalación | Licencia | Actividad | Adopción | Performance |
|---|---|---|---|---|---|---|
| `chromadb` (principal) | Embedded + client/server; auto-embedding opcional | `pip install chromadb`, API núcleo de 4 funciones | Apache-2.0 | Muy alta (releases frecuentes, Chroma Cloud) | Líder de adopción en agentes | Claims propios; sin baseline comparable citado aquí (**claim sin evidencia reproducible**) |
| `lancedb` | Embedded columnar (Lance), Rust vía PyO3 igual que nosotros | `pip install lancedb`; wheels AVX2 + variante `lancedb-compat` pre-Haswell; canal preview quincenal | Apache-2.0 | Releases estables cada ~2 semanas | Alta | Publica guías de tuning SIMD (pypi.org/project/lancedb) |
| `qdrant-client` (local mode) | Cliente del server con modo in-process para prototipos | `pip install qdrant-client` | Apache-2.0 | Alta | Alta | N/A local (modo dev) |
| `mem0ai` | Memory layer para agentes sobre stores externos | `pip install mem0ai`; requiere LLM/embeddings externos | Apache-2.0 | Muy alta (YC S24, npm+PyPI) | Alta y creciente | Publica research propio de token-efficiency (mem0.ai/research) |
| `sqlite-vec` | Extensión SQLite embebida | `pip install sqlite-vec`, wheels livianas (~293 KB, v0.1.9 mar 2026) | MIT/Apache | Media | Media-alta | Minimalista por diseño |
| `txtai` | Framework all-in-one (embeddings + workflows) | `pip install txtai` | Apache-2.0 | Alta, maduro | Media-alta | N/A |

**Diferenciación vs `chromadb`:** VantaDB-python es el único del set que combina
en un solo binario nativo: memoria persistente con namespaces + búsqueda híbrida
BM25↔HNSW con RRF + grafo dirigido con algoritmos (PageRank, DAG, BFS/DFS) +
TTL/supersede + auditoría + IQL + recuperación de wiki archivada — todo sin
servidor y con errores tipados. Chroma no tiene grafo ni supersede; LanceDB no
tiene texto híbrido nativo ni grafo; mem0 requiere store externo y LLM.

## 4. Estado actual de `vantadb-python`

- **API pública:** `VantaDB` (~50 métodos flat) + subclients generados por macro
  `forward_to_db!` (`MemoryClient`/`GraphClient`/`SystemClient`/`WikiClient`,
  `src/lib.rs:273-350`), `AsyncVantaDB` wrapper con semáforo (`vantadb_py/__init__.py:106`),
  `SearchRequest` dataclass (`__init__.py:57`), `connect()`, jerarquía tipada de
  9 excepciones + `VantaError` (MOD-20 ✅), migradores `migrate/chroma.py` y
  `migrate/lancedb.py`.
- **Tests:** 10 archivos — `stub_drift` (MOD-18 ✅), `typed_errors`, `close_concurrency`,
  `subclients`, `migration`, `async_smoke`, perf con marker `slow` excluido del gate
  default (`pyproject.toml:49-53`). Cobertura sólida y bien marcada.
- **Empaquetado:** maturin + abi3 cp311; CI multiplataforma
  (`.github/workflows/release-wheels-60.yml`: windows/macos/linux, manylinux 2_28,
  musllinux 1_2). PyPI: `vantadb-py` v0.5.0; doble import `vantadb`/`vantadb_py`
  documentado en README:12.
- **Docs:** `docs/api/PYTHON_SDK.md` fresca (34 KB, editada hoy) +
  `docs/api/BINDINGS_NAMESPACES.md`. README con quickstart funcional.
- **Historial relevante:** MOD-18/19/20/21 completados (commits `70016a20`,
  `dc65c242`, `9de39702`, `1002c301`, `f61cd4ae`). Deuda abierta: **P2-5**
  `put_batch` dual API (~53 líneas de branching, AGENTS.md tabla P2).
- **Performance:** sin números reproducibles publicados del SDK (Regla 11);
  existe `tests/test_perf_15_16.py` pero no hay baseline documentado en
  `docs/operations/BENCHMARKS.md`.

## 5. Framework de evaluación (score por dimensión)

| Dimensión | Score | Evidencia |
|---|---|---|
| DX onboarding | 8 | Quickstart <5min funciona; doble nombre import puede confundir |
| Completitud funcional | 8.5 | Superficie enorme; falta paridad `graph_filtered_traversal` (H-04) |
| Performance/overhead | 7 | GIL liberado, `put_batch_raw` zero-copy; sin números publicados (H-06) |
| Robustez | 8 | Tests de close-concurrency, op-gate drain, read_only |
| Seguridad | 8 | UB de punteros resuelto (P2-2 ✅), sin unsafe expuesto nuevo |
| Docs & ejemplos | 7.5 | PYTHON_SDK.md fresca; README con residuos ES (H-01) y sin posicionamiento diferencial (H-07) |
| Observabilidad | 8 | `operational_metrics`, `hardware_profile`, `audit_text_index` |
| Testabilidad | 9 | 10 suites, marker `slow`, anti-drift de stubs |
| Paridad inter-módulos | 8 | Falta `graph_bfs_filtered` que node/ts sí exponen (H-04) |
| Diferenciación | 8.5 | Único con memoria+híbrido+grafo+TTL embebido; mal contado (H-07) |

## 6. Matriz competencia resumida

| Feature | vantadb-py | chroma | lancedb | qdrant local | mem0 | sqlite-vec | txtai |
|---|---|---|---|---|---|---|---|
| Embebido sin servidor | ✅ | ✅ | ✅ | ⚠️ (dev) | ❌ | ✅ | ✅ |
| Híbrido vector+BM25 | ✅ RRF | ⚠️ | ❌ | ⚠️ server | ❌ | ❌ | ✅ |
| Grafo + algoritmos | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚠️ |
| TTL / supersede | ✅ | ❌ | ❌ | ❌ | ⚠️ | ❌ | ❌ |
| Errores tipados | ✅ | ⚠️ | ⚠️ | ✅ | ⚠️ | ❌ | ⚠️ |
| Async wrapper | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Stubs .pyi + py.typed | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Migradores desde competidores | ✅ chroma+lance | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

## 7. Gap analysis priorizado

- **Falta (P1):** paridad `graph_filtered_traversal` (H-04); soporte declarado 3.14 (H-03).
- **Mejorable (P1-P2):** README bilingüe inconsistente (H-01); posicionamiento
  diferencial ausente (H-07); higiene de artefactos locales (H-05); deuda P2-5 (H-02).
- **Optimizable (P2):** benchmarks reproducibles del SDK (H-06).
- **Quick wins (<1 día):** H-01, H-03, H-05, H-07, H-02 (🟢 1 hr según tabla P2).
- **Apuestas estratégicas (>1 semana):** H-09 (nombre de import), H-06 (suite de
  benchmarks comparativos contra chroma/lance con metodología propia).

## APÉNDICE — Inventario de hallazgos (entrada Fase D)

| ID | Hallazgo | Categoría sugerida | Severidad | Esfuerzo | Evidencia |
|----|----------|--------------------|-----------|----------|-----------|
| H-01 | Residuos en español en README pese a FIND-06 (`Abrir o crear…`, `busqueda-hibrida`) | MEJORAR | media-baja | 🟢 | `vantadb-python/README.md:37,52,63` |
| H-02 | Deuda P2-5: `put_batch` dual API (tuplas legacy + kwargs) | APLICAR | media | 🟢 | `vantadb-python/src/lib.rs` (flat `put_batch`; ref. AGENTS tabla P2) |
| H-03 | Classifiers/pyproject sin Python 3.14 (abi3 cp311 ya lo corre, falta declaración) | APLICAR | baja | 🟢 | `vantadb-python/pyproject.toml:22-26` |
| H-04 | Paridad: `graph_bfs_filtered`/`graph_filtered_traversal` no expuesto (node/ts sí) | AGREGAR | media | 🟡 | `vantadb-python/src/lib.rs:314-325` vs `vantadb-node/src/lib.rs:326-343` |
| H-05 | Artefactos locales sin exclusión explícita del módulo (`.pyd/.pdb/dist/probe_lock_db/.coverage`) — riesgo si amplían globs de maturin | MEJORAR | baja | 🟢 | `vantadb-python/` (disco, no tracked) + `pyproject.toml:37-43` |
| H-06 | Sin benchmarks reproducibles publicados del SDK (competidores publican los suyos) | AGREGAR | media | 🟡 | ausencia en `docs/operations/BENCHMARKS.md`; existe `tests/test_perf_15_16.py` |
| H-07 | README no lidera con diferenciadores (grafo, híbrido RRF, TTL/supersede, migradores) frente a chromadb | MEJORAR | media | 🟢 | `vantadb-python/README.md:1-30` |
| H-08 | Integraciones con frameworks como paquete instalable/testeado | DESCARTAR (cubierto por módulo `integrations` del registro) | — | — | registro fila 22; `examples/python/*` |
| H-09 | Doble identidad de import (`vantadb` vs `vantadb_py`) — decidir consolidación y timeline de deprecación | ESTRATEGIA | media | 🔴 decisión | `vantadb-python/README.md:12`, `vantadb/__init__.py`, `pyproject.toml:38` |

## Recomendaciones → filas FIND-\*/PY-\* en Backlog (según decisión Fase D)

Ver apéndice. Claims de performance en este informe: solo descriptivos de terceros
marcados donde corresponde; ningún claim propio sin benchmark (Regla 11).
