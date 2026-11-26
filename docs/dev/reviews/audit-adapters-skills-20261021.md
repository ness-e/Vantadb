---
title: "Auditoria de los 9 adaptadores y las skills - 2026-10-21"
kind: review
status: active
description: "Auditoria de solo lectura de la superficie publica: integrations/** (9 adaptadores de framework Python), skills/** y la paridad core <-> bindings. 10 hallazgos High, 16 Medium, 11 Low y 4 sospechas. El gap que FIND-506 declaraba queda cerrado. La clase de bug dominante es 'mismo dato, distinto resultado' entre superficies."
tags: [vantadb, audit, reviews, adapters, integrations, skills, bindings]
---

# Auditoría de los 9 adaptadores y las skills

> **Origen:** `FIND-506` (los 9 adaptadores y las skills no tuvieron pase de API pública).
> **Alcance real:** `integrations/**` (9 adaptadores de framework Python), `skills/**` (2 skills
> + hooks), los bindings (`python`/`ts`/`wasm`/`node`/`ffi-core`/`mcp`/`server`) y los 3 providers
> Rust, para la tabla de paridad.
> **Sin `cargo`/`npm`/`pip`** (había otro agente compilando): lo no ejecutado va marcado como
> *sospechado*.
> **Cierra `FIND-506`.** Hallazgos: **10 High · 16 Medium · 11 Low · 4 sospechas** → filas
> `FIND-573..584` (P0/P1 uno a uno, P2/P3 agrupados por área, según la convención del repo).

## 1. Inventario

### 1.1 Los 9 adaptadores (`integrations/`) — 7 publicados en PyPI + 2 source-only

| # | Dir | Paquete PyPI | ¿Publicado? | Exporta |
|---|---|---|---|---|
| 1 | `crewai` | `vantadb-crewai` | ✅ 0.9.0 | `VantaDBTool`, `VantaDBMemoryBackend` |
| 2 | `dspy` | `vantadb-dspy` | ✅ 0.9.0 | `VantaDBRetriever` |
| 3 | `haystack` | `vantadb-haystack` | ✅ 0.9.0 | `VantaDBDocumentStore` |
| 4 | `langchain` | `vantadb-langchain` | ✅ 0.9.0 | `VantaDBVectorStore`, `VantaDBStore`\*, `VantaDBCheckpointer`\*, `__version__` (\* = `None` sin `langgraph-checkpoint`) |
| 5 | `letta` | `vantadb-letta` | ✅ 0.9.0 | `VantaDBVectorStore` |
| 6 | `llamaindex` | `vantadb-llamaindex` | ✅ 0.9.0 | `VantaDBVectorStore` |
| 7 | `mem0` | `vantadb-mem0` | ✅ 0.9.0 | `VantaDBVectorStore`, `OutputData` |
| 8 | `ollama` | `vantadb-ollama` | ❌ source-only | `VantaDBOllama`, `Document` |
| 9 | `openai` | `vantadb-openai` | ❌ source-only | `VantaDBOpenAI`, `Document` |

Helper compartido: `integrations/vantadb_shared/__init__.py` (vendorizado en los wheels de
`ollama` y `openai` vía `force-include`; no es distribución propia).

### 1.2 Skills

| Skill | Estado |
|---|---|
| `skills/vantadb/SKILL.md` (595 líneas + `scripts/install-vantadb.sh`) | 1 API + 1 script — **el script falla siempre** y **5 ejemplos reventan** |
| `skills/vantadb-mcp/SKILL.md` (516 líneas + references + assets/hooks) | API + assets — **1 bug en el hook de OpenCode** |

## 2. Los 10 High

| # | Hallazgo | Fila |
|---|---|---|
| **H-1** | La skill documenta `put_batch` con dos firmas inexistentes (`entries=`, y `namespace=/keys=/payloads=…`); el binding real es `put_batch(records)` (array de objetos) → `TypeError` en el ejemplo que copia el agente | `FIND-573` |
| **H-2** | `metrics['mmap_resident_pages']` no existe (el campo real es `mmap_resident_bytes`) → `KeyError` | `FIND-575` |
| **H-3** | `r['record']['payload']` sobre un `SearchHit` (que solo tiene atributos) → `TypeError` en el pipeline de RAG | `FIND-576` |
| **H-4** | El instalador llama `vantadb.__version__()` (es un atributo) → `TypeError` → **el script dice "la instalación falló" cuando se instaló bien** | `FIND-574` |
| **H-5** | `skills/vantadb/scripts/install-vantadb.sh:49` fija `vantadb = "0.5.0"` (la real es 0.9.0; la propia skill dice 0.9 en la línea 48) | `FIND-582` (P2) |
| **H-6** | El MCP tiene `exclude_superseded` **hardcodeado a `false`** en 5 sitios, pero la skill lo promete ⇒ un cliente MCP no puede ocultar los supersedidos | `FIND-577` |
| **H-7** | El camino *batch* de Python pierde 6 parámetros que el camino *single* sí expone (`exclude_superseded`, `min_confidence`, `as_of_ms`, `valid_window`, `include_quarantined`, `search_profile`) y el dataclass ni los declara | `FIND-578` |
| **H-8** | El adaptador de mem0 acepta `filters` y **los ignora** (tampoco en `search_batch`) ⇒ mem0 pide "estos recuerdos" y recibe todos | `FIND-579` |
| **H-9** | Los **9** README prometen "vectors + BM25 text search out of the box" y **ninguno** lo cumple por defecto (2 lo tienen opt-in, 7 no lo pasan) | `FIND-580` |
| **H-10** | 4 quickstarts construyen una herramienta que **no busca**: listan los primeros k ignorando la query (en dspy, solo 4) | `FIND-581` |

## 3. Tabla de paridad de superficie

### 3.1 Core ↔ bindings

| Capacidad | Python | TS/WASM | TS/native | Node | MCP | HTTP |
|---|---|---|---|---|---|---|
| `put`/`put_batch` | ✅ / ✅ array-of-objects | ✅ | ✅ | ✅ | ✅ | ✅ |
| `get`/`list`/`count`/`delete`/`delete_by_filter` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `supersede` | ✅ | ✅ | ✅ | ✅ | ✅ | ❓ |
| **`exclude_superseded`** | ✅ | ✅ | ✅ | ✅ | ❌ **hardcoded `false`** | ❓ |
| `versions`/`get_version` | ❌ | ❌ | ❌ | ✅ | ✅ | ❓ |
| `search`/`search_multi`/`search_with_method`/`explain` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `search_vector` (ANN) | ✅ | ✅ | ✅ | ❌ (declarado) | ❌ | ❓ |
| `scan` | ❌ | ✅ | ✅ | ❌ | ❌ | ❓ |
| `import_records`/`export_*`/`audit_text_index`/`repair_text_index` | parcial | ✅ | ✅ | ❌ | ❌ | ❓ |
| `vacuum` | ❌ | ❌ | ✅ | ✅ | ❌ | ❓ |
| `graph_page_rank`/`graph_degree_centrality` | ✅ | ❌ | ❌ | ❌ | ❌ | ❓ |
| `graph_shortest_path` | ❌ | ❌ | ✅ | ❌ | ❌ | ❓ |
| `graphrag_search` | ✅ | ✅ | ✅ | ✅ | ❌ | ❓ |
| `thread_*`(6)/`dream_*`(5)/`wiki_*`(6) | parcial | ❌ | ❌ | parcial | ✅ | ❓ |

### 3.2 Core ↔ los 9 adaptadores (todos vía `vantadb-py`)

| Capacidad | crewai | dspy | haystack | langchain | letta | llamaindex | mem0 | ollama | openai |
|---|---|---|---|---|---|---|---|---|---|
| `put` / `memory.search` / `memory.delete` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `memory.get` / `memory.list` | ✅ / ✅ | ❌ / ✅ | ✅ / ✅ | ❌ / ❌ | ❌ / ✅ | ✅ / ✅ | ✅ / ✅ | ❌ / ❌ | ❌ / ❌ |
| `delete_by_filter` / `supersede` / `exclude_superseded` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **BM25 (`text_query`)** | ❌ | ❌ | ❌ | ⚠️ opt-in | ❌ | ⚠️ opt-in | ❌ | ❌ | ❌ |
| **`filters` aplicados** | ❌ | ❌ | ⚠️ parcial | ⚠️ `filter_key/val` | ⚠️ `list` sí | ⚠️ casteados a `str` | ❌ **ignorado** | ❌ | ❌ |
| **MMR** | ❌ | ❌ | ❌ | ⚠️ client-side | ❌ | ⚠️ client-side | ❌ | ❌ | ❌ |
| `close()`/`flush()` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

**Lectura:** los 9 adaptadores exponen **~6 de ~66** operaciones del core, de forma
**inconsistente entre sí** (filtros, MMR, BM25, score). Ninguno expone `exclude_superseded`,
`supersede` ni `delete_by_filter`.

### 3.3 La clase "mismo dato, distinto resultado"

| # | Concepto | Dónde diverge |
|---|---|---|
| D-1 | `exclude_superseded` | MCP ❌ vs los 4 bindings ✅ vs los 9 adapters ❌ (nadie lo expone) |
| D-2 | `min_confidence`/`as_of_ms`/`valid_window`/`include_quarantined` en **batch** | Python ❌ (`lib.rs:2765-2771`) |
| D-3 | `filters` EQ | Python/TS/Node ✅ · langchain ✅ · llamaindex castea a `str` ❌ · mem0 los ignora ❌ |
| D-4 | Score = **similitud** ∈ [-1,1] | bindings: crudo · crewai aplica `min_score=0.0` ⇒ **descarta la similitud negativa** · langchain devuelve crudo en un método y clampea en otro · mem0 clampea a [0,1] |
| D-5 | `distance_metric` | solo `"cosine"`/`"euclidean"` en minúscula en Python |

## 4. Los ejemplos de la skill que revientan (con el arreglo)

| # | Snippet roto | Arreglo |
|---|---|---|
| 1 | `db.put_batch(entries=entries)` / `db.put_batch(namespace=…, keys=…)` | `db.put_batch([{"namespace": "ns1", "key": "k1", "payload": "…", "metadata": {…}, "vector": [0.1, 0.2], "ttl_ms": None}])` |
| 2 | `metrics['mmap_resident_pages']` | `metrics['mmap_resident_bytes']` (puede ser `None`) |
| 3 | `"\n".join([r['record']['payload'] for r in results])` | `"\n".join([r.payload for r in results])` |
| 4 | `vantadb.__version__()` | `vantadb.__version__` |
| 5 | `pip install vantadb-llamaindex llama-index-core` | `+ llama-index-embeddings-openai` |
| 6 | `db.query('SELECT * FROM memory WHERE namespace = "agent/session-1" LIMIT 10')` | `db.query('SELECT * FROM agent_session_1 LIMIT 10')` (el namespace **es** la tabla, sanitizado) |

## 5. Lo que no se pudo verificar

1. **Nada se compiló ni se ejecutó** (`cargo`, `npm`, `tsc`, `mypy`, `pytest`): todos los hallazgos
   son source-level + introspección de librerías ya instaladas.
2. Que los `.pyi` compilen contra el módulo nativo (requiere `maturin develop`).
3. Que `vantadb-ts/dist/*.d.ts` case con `src/*.ts` (artefacto de build, sin `tsc --noEmit`).
4. La superficie HTTP de `vantadb-server` completa (`src/server/handlers.rs` no leído entero):
   las celdas ❓ de la tabla.
5. El comportamiento real de los 9 adaptadores contra el core (sus `tests/` usan el binding
   compilado).
6. Los hooks `{claude,codex,cursor}/**` (son JSON de configuración; solo el de OpenCode tiene
   lógica ejecutable).
7. Si el wheel publicado `vantadb-ollama@0.9.0` contiene `embed_batch` (requería descargarlo).

## 6. Estado de los hallazgos de la campaña del 2026-10-20

| Finding | Estado |
|---|---|
| **`FIND-506`** (9 adaptadores + skills sin pase de API pública) | **CERRADO por este informe**: el alcance declarado (los 9 `__init__.py`, los 9 `pyproject.toml`, los `tests/`, `install-vantadb.sh` y `assets/hooks/**`) queda cubierto |
| **`AUD-01`** (`integrations/**` fuera de los `paths:` de CI) | **CONFIRMADO, sigue abierto** → `FIND-582` (M-13). El ledger lo marca "FIND-449 (lote, cerrado)" pero `Backlog.md:1020` lo mantiene pendiente, y el código lo confirma: `ci-fleet.yml:13-27,38-54` y `docs-gates.yml:19-37,40-58` no incluyen `integrations/**`. El postmortem lo listó como P1 |
| **Área 5 declarada "Media (vía READMEs)"** | **CONFIRMADA**: la validación por README era la que no servía — los README prometen BM25, cosine distance y quickstarts funcionales, y el código no lo cumple (H-3, H-9, H-10, M-3) |
| **"versionado de los 9 confirmado 0.9.0"** | **CONFIRMADO y correcto**: los 9 `pyproject.toml` declaran `0.9.0` = `[workspace.package] version`. Lo que no está gateado es el **piso** de `vantadb-py` (M-15) ni la presencia de las superficies en `SURFACE_MANIFEST.md` (M-14) |
| **`FIND-412`** (`exclude_superseded` en Node) | **CORREGIDO** (ya cerrado): `vantadb-node/src/lib.rs:1503,1541` lo leen, `vantadb-ts/src/native.ts:498,541` lo propagan, con tests de regresión |
| **`FIND-482`/`FIND-483`** (WASM) | **CONFIRMADOS, siguen pendientes**; aquí se **corrige su alcance**: el problema del WASM es de **tipos** (el runtime honra el flag), pero en el MCP es peor — el flag **no existe en el wire** (H-6) |
| **`FIND-268`** (Node no expone IQL `query`) | **Exclusión declarada** (`docs/api/BINDINGS_NAMESPACES.md:127-129`) → no es un fallo |
| **`FND-06` / R-8** (fórmulas de score duplicadas) | **CORREGIDO**: el `1.0 - score/2.0` ya no está en langchain/llamaindex. Queda (a) el doc canónico apuntando a un helper no expuesto a Python (M-2) y (b) el MMR client-side, que es un **deferral documentado** (`docs/api/SEARCH_PARITY.md:19`) |
| **`FIND-277`** (`superseded_by` no se consume en el read-side L1) | **Sigue vigente**: es la raíz de por qué H-6 duele — el MCP es donde más se nota, porque no puede ni filtrar ni ver el `superseded_by` |

## 7. Resumen ejecutivo

- Los 9 adaptadores exponen **~6 de ~66** operaciones del core, de forma **inconsistente entre sí**.
- La clase de bug de mayor valor ("mismo dato, distinto resultado") está confirmada en **4
  puntos**: `exclude_superseded` (MCP ❌ vs 4 bindings ✅), el camino batch de Python (6
  parámetros perdidos), `filters` en mem0 (ignorado) y el score (crudo vs clampeado vs filtrado
  por `min_score`).
- **Promesas que no se cumplen**: 9/9 READMEs prometen BM25 y 0/9 lo cumplen; 4 quickstarts
  construyen herramientas que no buscan; 5 ejemplos de la skill `vantadb` reventan (2 con
  `TypeError`, 1 con `KeyError`, 1 con `ModuleNotFoundError`, 1 con el instalador auto-fallando).
- **Gobernanza**: los 7 paquetes publicados en PyPI **no tienen fila** en el manifest canónico de
  superficies, **no disparan** sus propios tests en un PR, y su README no pasa los gates de docs.
- **Prioridad de arreglo sugerida:** H-1, H-4, H-2, H-3 (la skill se copia literalmente → es lo
  que un agente ejecuta y falla), luego H-6, H-7, H-8, H-9 (divergencia de semántica), luego
  M-13/M-14 (gobernanza, sin coste de código).
