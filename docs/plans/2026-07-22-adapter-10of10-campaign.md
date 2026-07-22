# Plan: Adapter & Provider — 10/10 Campaign

> **Goal:** Llevar los 9 adapters Python + 3 providers Rust a clasificación 10/10 (producción-ready)
> **Inicio:** 2026-07-22
> **Estado:** ✅ COMPLETED (2026-07-22, commits accbfa8 + 404f388 + 65c37bf + b519111)
> **Fuente:** Investigación directa de código

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 20 | 0     | 0    | 0         |

## Hallazgos Clave por Adapter/Provider

### Rust Providers — Issues encontrados
- **openai**: `cast::<PyDict>()` falla en pydantic Embedding (🔴), falta `get()`/`list()` (🔴), `search()` sin text_query/filters (🟡)
- **ollama**: falta `get()`/`list()` (🔴), `search()` sin hybrid (🟡), sin tests (🟡)
- **litellm**: `embed` vs `embedding` nombre incorrecto (🔴), `PyModule::import` en cada llamada (🔴), falta `get()`/`list()` (🔴)
- **Todos**: sin tests (🟡), sin docstrings (🟢), sin timeout configurable (🟢)

### Python Adapters — Issues encontrados
- **mem0**: NO implementa `VectorStoreBase` (🔴), firmas incompatibles (🔴), faltan 8/11 métodos abstractos (🔴)
- **haystack**: filter syntax no parseada (🔴), `count_documents()` limit 10K (🔴), falta OVERWRITE (🔴)
- **dspy**: `forward()` devuelve `list[str]` en vez de `Prediction(passages=...)` (🔴)
- **letta**: dependencia `letta>=0.1.0` incorrecta (es `letta-client`) (🔴), falta integración con Letta SDK (🔴)
- **crewai**: `categorize()` es stub (🔴), error path tipo iterable (🔴)
- **Todos**: sin tests adecuados, sin docstrings, sin validación de edge cases

---

### Wave 1: Rust Providers — Bugs + features core
Ejecución paralela (3 sub-agentes)

#### Task PRV-01: OpenAI — fix critical bugs + get/list
- **Archivos clave:** `providers/openai/src/python.rs`
- **Gate Justificación:** Bug pydantic Embedding rompe embed() en producción + falta get/list para 10/10
- **Contrato:** `cargo check -p vantadb-openai` pasa + `get()` y `list()` expuestos
- **Task file:** `tasks/PRV-01.md`
- **Estado:** ✅ COMPLETED

#### Task PRV-02: LiteLLM — fix embed→embedding + cache + get/list
- **Archivos clave:** `providers/litellm/src/python.rs`
- **Gate Justificación:** Nombre de función incorrecto + import en cada llamada + falta get/list
- **Contrato:** `cargo check -p vantadb-litellm` pasa + `get()` y `list()` expuestos
- **Task file:** `tasks/PRV-02.md`
- **Estado:** ✅ COMPLETED

#### Task PRV-03: Ollama — add get/list
- **Archivos clave:** `providers/ollama/src/python.rs`
- **Gate Justificación:** Falta get/list para feature parity con otros providers
- **Contrato:** `cargo check -p vantadb-ollama` pasa + `get()` y `list()` expuestos
- **Task file:** `tasks/PRV-03.md`
- **Estado:** ✅ COMPLETED

### Wave 2: Python Adapters — Bugs críticos
Ejecución paralela (5 sub-agentes)

#### Task ADP-01: mem0 — rewrite to VectorStoreBase
- **Archivos clave:** `integrations/mem0/vantadb_mem0/vectorstore.py`, `integrations/mem0/pyproject.toml`
- **Gate Justificación:** No hereda de VectorStoreBase, firmas incompatibles, faltan 8/11 métodos
- **Contrato:** Python syntax check pasa + implementa 11 métodos de VectorStoreBase
- **Task file:** `tasks/ADP-01.md`
- **Estado:** ✅ COMPLETED

#### Task ADP-02: haystack — filter parsing + count limit + OVERWRITE + serialization
- **Archivos clave:** `integrations/haystack/vantadb_haystack/vectorstore.py`
- **Gate Justificación:** Filter syntax no se traduce, count limit 10K bug, falta OVERWRITE
- **Contrato:** Python syntax check pasa + tests existentes siguen pasando
- **Task file:** `tasks/ADP-02.md`
- **Estado:** ✅ COMPLETED

#### Task ADP-03: dspy — fix forward return type + dump_state + metadata
- **Archivos clave:** `integrations/dspy/vantadb_dspy/vectorstore.py`
- **Gate Justificación:** `forward()` devuelve tipo incorrecto (list[str] vs Prediction)
- **Contrato:** Python syntax check pasa + dspy.Prediction usado correctamente
- **Task file:** `tasks/ADP-03.md`
- **Estado:** ✅ COMPLETED

#### Task ADP-04: letta — fix dependency + validation + serialization
- **Archivos clave:** `integrations/letta/vantadb_letta/vectorstore.py`, `integrations/letta/pyproject.toml`
- **Gate Justificación:** Dependencia incorrecta (letta vs letta-client), falta to_dict/from_dict
- **Contrato:** Python syntax check pasa + pyproject.toml con letta-client correcto
- **Task file:** `tasks/ADP-04.md`
- **Estado:** ✅ COMPLETED

#### Task ADP-05: crewai — fix error path + categorize + validation
- **Archivos clave:** `integrations/crewai/vantadb_crewai/vectorstore.py`
- **Gate Justificación:** categorize() stub, error path tipo inconsistente
- **Contrato:** Python syntax check pasa + tests existentes pasan
- **Task file:** `tasks/ADP-05.md`
- **Estado:** ✅ COMPLETED

### Wave 3: Rust Providers — Feature parity
Ejecución en paralelo (3 sub-agentes)

#### Task PRV-04: All 3 — search(text_query, filters, distance_metric)
- **Archivos clave:** `providers/*/src/python.rs`
- **Gate Justificación:** Búsqueda híbrida y filtros necesarios para feature parity
- **Contrato:** `cargo check --workspace` pasa
- **Task file:** `tasks/PRV-04.md`
- **Estado:** ✅ COMPLETED

#### Task PRV-05: All 3 — timeout configurable + counter fix + list_namespaces
- **Archivos clave:** `providers/*/src/python.rs`
- **Gate Justificación:** Polish necesario para 10/10: timeout para evitar hangs, counter UUID
- **Contrato:** `cargo check --workspace` pasa
- **Task file:** `tasks/PRV-05.md`
- **Estado:** ✅ COMPLETED

#### Task PRV-06: All 3 — docstrings + error handling
- **Archivos clave:** `providers/*/src/python.rs`
- **Gate Justificación:** Código público sin documentación interna
- **Contrato:** `cargo check --workspace` pasa
- **Task file:** `tasks/PRV-06.md`
- **Estado:** ✅ COMPLETED

### Wave 4: Python Adapters — Tests
Ejecución paralela (4 sub-agentes)

#### Task TST-01: Tests for openai + ollama Python adapters
- **Archivos clave:** `integrations/openai/`, `integrations/ollama/`
- **Gate Justificación:** Adapters con más uso potencial, sin tests
- **Contrato:** `python -m pytest integrations/openai/tests/` pasa
- **Task file:** `tasks/TST-01.md`
- **Estado:** ✅ COMPLETED

#### Task TST-02: Tests for langchain + llamaindex adapters
- **Archivos clave:** `integrations/langchain/`, `packages/llamaindex/`
- **Gate Justificación:** Adapters clave del ecosistema, tests incompletos
- **Contrato:** `python -m pytest` en ambos pasa
- **Task file:** `tasks/TST-02.md`
- **Estado:** ✅ COMPLETED

#### Task TST-03: Tests for crewai + dspy + haystack adapters
- **Archivos clave:** `integrations/crewai/`, `integrations/dspy/`, `integrations/haystack/`
- **Gate Justificación:** Tests no cubren embedding ni edge cases
- **Contrato:** Tests pasan con embedding mockeado
- **Task file:** `tasks/TST-03.md`
- **Estado:** ✅ COMPLETED

#### Task TST-04: Tests for letta + mem0 adapters
- **Archivos clave:** `integrations/letta/`, `integrations/mem0/`
- **Gate Justificación:** Adapters recientemente reescritos, tests ausentes
- **Contrato:** Tests básicos de insert/search/delete/list pasan
- **Task file:** `tasks/TST-04.md`
- **Estado:** ✅ COMPLETED

### Wave 5: Python Adapters — Feature parity
Ejecución paralela (3 sub-agentes)

#### Task FTR-01: Async methods for openai + ollama
- **Archivos clave:** `integrations/openai/`, `integrations/ollama/`
- **Gate Justificación:** Frameworks esperan add_texts async
- **Contrato:** Python syntax check pasa
- **Task file:** `tasks/FTR-01.md`
- **Estado:** ✅ COMPLETED

#### Task FTR-02: MMR for langchain + llamaindex
- **Archivos clave:** `integrations/langchain/`, `packages/llamaindex/`
- **Gate Justificación:** Max Marginal Relevance Search es feature esperada
- **Contrato:** Método `max_marginal_relevance_search` implementado en ambos
- **Task file:** `tasks/FTR-02.md`
- **Estado:** ✅ COMPLETED

#### Task FTR-03: LlamaIndex hybrid mode
- **Archivos clave:** `packages/llamaindex/vantadb_llamaindex/`
- **Gate Justificación:** hybrid_mode stub no implementado
- **Contrato:** hybrid_mode=True hace búsqueda híbrida real
- **Task file:** `tasks/FTR-03.md`
- **Estado:** ✅ COMPLETED

### Wave 6: Docstrings + Edge Cases
Ejecución paralela (2 sub-agentes)

#### Task DOC-01: Docstrings for all Python adapters
- **Archivos clave:** `integrations/*/vantadb_*/vectorstore.py`
- **Gate Justificación:** Código público sin documentación
- **Contrato:** Todos los métodos públicos tienen docstring
- **Task file:** `tasks/DOC-01.md`
- **Estado:** ✅ COMPLETED

#### Task DOC-02: Edge case hardening for all adapters
- **Archivos clave:** Todos los adapters
- **Gate Justificación:** Validación de inputs faltante en todos
- **Contrato:** Empty inputs, None values manejados
- **Task file:** `tasks/DOC-02.md`
- **Estado:** ✅ COMPLETED

### Wave 7: Verification + Close

#### Task VERIFY: Full verification pass
- **Archivos clave:** N/A
- **Gate Justificación:** Asegurar que todo funciona junta
- **Contrato:** `cargo check --workspace` + Python syntax all adapters + tests
- **Task file:** `tasks/VERIFY-01.md`
- **Estado:** ✅ COMPLETED (commit 404f388)

