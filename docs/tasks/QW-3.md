# TASK QW-3: llamaindex attrs privados + import — PrivateAttr + get_type_hints()

## Metadata
- **Plan file:** `docs/plans/2026-08-25-integrations-research-wins.md`
- **Fuente:** Wave 1 QW-3 (H-04 =MOD-48)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** Wave 1 — Fixes de bugs de contrato
- **Tipo:** Python
- **Turns estimados:** 5-7
- **Creado:** 2026-08-27T01:30
- **last-synced:** 2026-08-27T01:30
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `integrations/llamaindex/vantadb_llamaindex/__init__.py` (re-export VantaDBVectorStore), `integrations/llamaindex/tests/test_vectorstore.py` (23 tests usan store), docs/plans reference, ningún módulo Rust core depende del adapter (aislado) |
| Callees | `vantadb_py` (VantaDB client), `llama_index.core.bridge.pydantic.PrivateAttr`, `llama_index.core.vector_stores.types.BasePydanticVectorStore`, `llama_index.core.vector_stores.types.MetadataFilter/MetadataFilters/FilterOperator/VectorStoreQuery`, `pydantic.BaseModel` via bridge |
| Implicaciones | contrato no cambia API pública (thin wrapper), fix mantiene seriálización pydantic limpia (model_dump sin _client/_namespace), get_type_hints resuelve MetadataFilter; no afecta performance/memoria/serialización más allá de PrivateAttr; no requiere migración de datos ni re-indexación; tests existentes deben pasar (23) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `integrations/llamaindex/vantadb_llamaindex/vectorstore.py` (481 líneas completas)
  - `integrations/llamaindex/tests/test_vectorstore.py` (262 líneas completas)
  - `integrations/llamaindex/tests/conftest.py` (2 líneas)
  - `integrations/llamaindex/vantadb_llamaindex/__init__.py` (3 líneas)
  - `integrations/llamaindex/pyproject.toml` (36 líneas)
  - `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 1 QW-3)
  - `venv/Lib/site-packages/llama_index/core/vector_stores/types.py` (BasePydanticVectorStore, verifies arbitrary_types_allowed + PrivateAttr usage)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):**
  - `import vantadb_py as vanta` → `vantadb-python` PyO3 (no tocar core `vantadb/src/*` protegido)
  - `from llama_index.core.bridge.pydantic import PrivateAttr` → pydantic v2 PrivateAttr (docs: https://docs.pydantic.dev/latest/concepts/models/#private-model-attributes)
  - `from llama_index.core.vector_stores.types import BasePydanticVectorStore` → hereda `model_config = ConfigDict(arbitrary_types_allowed=True)`
  - `from llama_index.core.vector_stores.types import MetadataFilter, MetadataFilters, FilterOperator` → necesarios para que `get_type_hints()` resuelva anotaciones (from __future__ import annotations)
  - `from llama_index.core.schema import BaseNode, TextNode` etc. → node conversion helpers
- **Archivos que referencian a los editados (referencias entrantes):**
  - `tests/test_vectorstore.py` — 23 tests usan VantaDBVectorStore (add/query/delete/get_nodes/clear + 2 tests QW-3 específicos: test_method_type_hints_resolve, test_private_attrs_declared_and_serialization_clean)
  - `integrations/llamaindex/README.md` → importa VantaDBVectorStore
  - `integrations/llamaindex/.pytest_cache` → cache
  - Ningún módulo Rust core, web/, u otro adapter depende de este archivo (grep confirma aislamiento)
- **Veredicto impacto:** bajo — impacto localizado a `integrations/llamaindex/`, seguro para edit. No toca paths multi-índice/dashmap/parking_lot/Tokio → no requiere auditoría concurrencia (Regla 8). No hot path vectorial core → no requiere perf bench (Regla 9). No trust boundary nuevo → no security hardening más allá de validación pydantic.

## Spec

N/A — bug-fix con contrato mecánico (Wave 1 QW-3). No agrega símbolos públicos nuevos; solo corrige declaración de attrs privados y resolución de imports.

Problema: `_namespace`/`_client` (y `_db_path`/`_hybrid_mode`) no declarados como `PrivateAttr` → pydantic los trataba como fields (filtraban en model_dump/model_json_schema, validación extra). Además `MetadataFilter` faltaba en imports → `typing.get_type_hints(VantaDBVectorStore._build_vanta_filters)` lanzaba NameError bajo `from __future__ import annotations`.

Criterio: `_namespace`/`_client` declarados como `PrivateAttr`; anotación `Tuple[Optional[Dict[str,Any]], List[MetadataFilter]]` resuelve bajo `get_type_hints()`; test que ejercita serialización pydantic (`model_dump` sin `_client`/`_namespace`) pasa.

Alcance: `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:7-37` (imports + PrivateAttr declarations).

Decisiones: usar `PrivateAttr()` sin default (asignado en __init__, consistente con `integrations/crewai` pattern `PrivateAttr`); importar `MetadataFilter` completo (no string forward ref) para que get_type_hints resuelva sin `model_rebuild()`; mantener `Any` para `_client` (vantadb_py type opaco, arbitrary_types_allowed).

## Contrato

```
typing.get_type_hints(VantaDBVectorStore._build_vanta_filters) resuelve sin NameError y contiene 'return'
store.model_dump() no contiene '_client' ni '_namespace' (PrivateAttr no filtra)
python -m pytest integrations/llamaindex/tests -q  # 23 passed
```

Verificación mecánica:
1. `python -c "import typing; from vantadb_llamaindex.vectorstore import VantaDBVectorStore; print(typing.get_type_hints(VantaDBVectorStore._build_vanta_filters))"` ✅ contiene return
2. `python -c "from vantadb_llamaindex.vectorstore import VantaDBVectorStore; s=VantaDBVectorStore(db_path=tmp, namespace='t'); print('_client' not in s.model_dump() and '_namespace' not in s.model_dump())"` ✅ True
3. `python -m pytest integrations/llamaindex/tests -q` → 23 passed ✅
4. `python -m pytest integrations/llamaindex/tests/test_vectorstore.py::test_method_type_hints_resolve -xvs` ✅
5. `python -m pytest integrations/llamaindex/tests/test_vectorstore.py::test_private_attrs_declared_and_serialization_clean -xvs` ✅

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `model_dump()`/`model_json_schema()` nunca exponen `_client`/`_namespace`/`_db_path`/`_hybrid_mode` (son PrivateAttr); `get_type_hints` resuelve todas las anotaciones del vectorstore sin NameError; `add`/`query`/`delete`/`get_nodes`/`clear` mantienen contrato existente (no cambia API pública)
- **Comandos de verificación:** `python -m pytest integrations/llamaindex/tests -q` → 23 passed; `python -c "import typing; from vantadb_llamaindex.vectorstore import VantaDBVectorStore; assert 'return' in typing.get_type_hints(VantaDBVectorStore._build_vanta_filters)"`
- **Deuda pendiente:** ninguna — fix ya en HEAD (96e143ec), esta tarea re-verifica. Si hay follow-up: considerar default explícito en PrivateAttr (`default=...`) para reconstrucción via `model_validate` sin `__init__` (deuda menor, no bloqueante)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva. Fix reduce deuda P2 (attrs privados mal declarados) sin introducir nueva. No se toca `vantadb/src/*`.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumple: PrivateAttr + get_type_hints resuelve + model_dump limpio + 23 tests pass |
| **Commit** | No aplica en esta iteración (verify-only, fix ya commit 96e143ec) — se documenta handoff sin nuevo commit por regla "no commit" del prompt |
| **Release** | No aplica (adapter Python puro, no requiere publish en esta wave — QW-7 separado) |

Gate: task se marca COMPLETED si contrato task pasa + capa determinista (pytest) pasa.

## Herramientas necesarias

- **source-driven-development:** verificar docs oficiales pydantic PrivateAttr + llama_index BasePydanticVectorStore
- **ponytail (full):** ladder mínimo — reusar stdlib PrivateAttr pattern existente, no añadir deps, thin wrapper
- **systematic-debugging:** root cause get_type_hints fallo por import faltante MetadataFilter
- **test-driven-development:** contrato ya en tests existentes, verify rojo→verde sin nuevo código si ya verde

**SKILLS_CARGADAS (SDP):** source-driven-development, ponytail, systematic-debugging, test-driven-development (≤4 por SDP)

Lifecycle mapping BUILD (source-driven, ponytail, TDD) + VERIFY (systematic-debugging)

Grep SKILLS-MANIFEST.md por `llamaindex|pydantic|PrivateAttr|vector|serialization` → skill `vantadb` (rating 8) aporta contexto producto pero no añade patrón de código específico al fix; `python-packaging` no relevante (no publish). Discovery ≤8 skills → 4 cargadas, justificadas arriba. SDP sin candidatos adicionales más allá de `vantadb` (informativa, no cargada como skill operativa).

## Investigation Notes

- STACK DETECTED (source-driven-development Step 1):
  - `pydantic 2.12.5` (from venv)
  - `llama-index-core` via `llama_index.core.bridge.pydantic.PrivateAttr` (re-export de `pydantic.PrivateAttr`)
  - `vantadb-py 0.5.0` (from `integrations/llamaindex/pyproject.toml` dependencies `vantadb-py>=0.5.0,<0.6.0`, `llama-index-core>=0.12`)
  → Fetching official docs for PrivateAttr pattern.

- FETCH Step 2 — fuentes autoritativas citadas:
  - **Pydantic PrivateAttr** — `https://docs.pydantic.dev/latest/concepts/models/#private-model-attributes` : PrivateAttr declarados como `_attr: Type = PrivateAttr(default=...)` quedan en `__pydantic_private__`, no en `model_fields` ni `model_dump()` — usar PrivateAttr para internals no serializables (client, db_path, namespace). Impl: `class M(BaseModel): _a: Any = PrivateAttr(default='hello')`
  - **Pydantic BaseModel model_dump** — `https://docs.pydantic.dev/latest/concepts/models/#basic-model-usage` : `model_dump()` solo vuelca fields, no private attrs.
  - **llama_index BasePydanticVectorStore** — `venv/Lib/site-packages/llama_index/core/vector_stores/types.py:334-338` : `class BasePydanticVectorStore(BaseComponent, ABC): model_config = ConfigDict(arbitrary_types_allowed=True)` — permite `Any` para `_client` opaco; hereda `BaseModel` private attr support.
  - **typing.get_type_hints** con `from __future__ import annotations` — requiere imports completos en runtime globals; `MetadataFilter` debe estar importado para que `_build_vanta_filters` → `Tuple[..., List[MetadataFilter]]` resuelva (docs.python.org/3/library/typing.html#typing.get_type_hints). Fix: añadir `MetadataFilter` al import de `llama_index.core.vector_stores.types`.

- CodeGraph blast radius (Step 2): `VantaDBVectorStore` en `integrations/llamaindex` tiene 3 callers en `__init__.py` + tests; callees `PrivateAttr`, `search_memory`, `list_memory` etc.; 143 edges; implicaciones: contrato no rompe, isolated.

- Investigación confirma fix ya en HEAD commit 96e143ec (diff: `+MetadataFilter` import, `+_namespace/_db_path/_hybrid_mode/_client = PrivateAttr()`). No se requiere edición adicional salvo verify. Ponytail: no añadir abstracción, no duplicar lógica core/bindings.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado contra docs oficiales, no hay decisiones abiertas |
| Pendientes de ejecución (downhill) | 0 tras verify (4 steps) |
| % completado | 100% (verify-only, fix preexistente) |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** `python -c "import typing; from vantadb_llamaindex.vectorstore import VantaDBVectorStore; typing.get_type_hints(VantaDBVectorStore._build_vanta_filters)"` → `NameError: name 'MetadataFilter' is not defined` (antes del fix, MetadataFilter faltaba en imports bajo `from __future__ import annotations`). Además `store.model_dump()` contenía `_client` si no era PrivateAttr (pydantic lo serializaba como field).
- **Hipótesis:** `vectorstore.py:7-15` importaba `MetadataFilters` pero no `MetadataFilter` (singular) usado en firma `_build_vanta_filters(...) -> Tuple[..., List[MetadataFilter]]`; con `from __future__ import annotations` la anotación queda como string y `get_type_hints` evalúa en globals → falla si símbolo no importado. Hipótesis PrivateAttr: attrs con `_` pero sin `PrivateAttr` son tratados como fields privados con warning pero igual en `model_fields`, filtran en dump/schema.
- **1 variable controlada:** añadir `MetadataFilter` al import + declarar 4 attrs como `PrivateAttr()` (commit 96e143ec) — un único cambio de contrato, no mezclar con dedup ni otros fixes.
- **Test RED:** `test_method_type_hints_resolve` rojo antes del fix (NameError); `test_private_attrs_declared_and_serialization_clean` rojo (assert `_client not in model_dump()` falla). Ambos verdes tras fix.

Gate: evidencia poblada antes de Steps de fix. Fix ya aplicado en 96e143ec, re-verificado aquí.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No toca trust boundaries (no input usuario sin validar, no auth, no FFI nuevo, no deps nuevas). Vectorstore es thin wrapper sobre vantadb_py ya validado. Checklist `security-and-hardening` no aplica más allá de pydantic model_dump no expone credenciales (verificado: _client privado no filtra). Justificación: no hay vector de ataque nuevo.
- [x] **PERFORMANCE** — No toca hot path (vector/index/text_index/engine). No loop de search/ingestión modificado más allá de tipo hint + PrivateAttr (overhead cero). No requiere baseline bench (Regla 9 no aplica). Justificación: cambio declarativo pydantic, sin impacto runtime medible.

## Steps

### Step 1: DISCOVERY — blast radius + Regla 0 + source-driven detect
- **Archivos:** `integrations/llamaindex/vantadb_llamaindex/vectorstore.py`, `integrations/llamaindex/tests/test_vectorstore.py`, `venv/.../types.py`
- **Acción:** Confirmar fixes QW-3 (96e143ec) ya en disco: 4 PrivateAttr declarados, MetadataFilter import completo, get_type_hints resuelve. Mapear Regla 0 completa arriba. CodeGraph explore blast radius. Fetch docs oficiales pydantic PrivateAttr + BasePydanticVectorStore. Validar stack versions.
- **Verify:** `python -c "import typing; from vantadb_llamaindex.vectorstore import VantaDBVectorStore; print(typing.get_type_hints(VantaDBVectorStore._build_vanta_filters))"` → contiene return; `codegraph_explore` blast radius; docs fetched
- **Estado:** ✅ DONE (2026-08-27 — ya en HEAD; get_type_hints OK verificado, codegraph 22 símbolos, docs citados)

### Step 2: ACT — verify fix presente, no edit necesario (harden si hace falta)
- **Archivos:** `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:7-37`
- **Acción:** Revisar vectorstore.py líneas 34-37: `_namespace/_db_path/_hybrid_mode/_client` deben ser `PrivateAttr()` con type `Any`/`str`/`bool` según corresponda. Revisar import línea 12: `MetadataFilter` presente junto a `MetadataFilters`. Si todo verde, no editar (ponytail: shortest diff wins — no tocar código que ya pasa contrato). Si faltara default explícito, evaluar si añadir `default=...` mejora reconstrucción via `model_validate` — decidir no cambiar (consistente con crewai pattern, no requerido por contrato).
- **Verify:** `grep -n "PrivateAttr\|MetadataFilter" integrations/llamaindex/vantadb_llamaindex/vectorstore.py` → 4 PrivateAttr + MetadataFilter import; scripts python inline: model_dump limpio
- **Estado:** ✅ DONE (2026-08-27 — 4 PrivateAttr + MetadataFilter presentes, model_dump sin leaks, no edit requerido)

### Step 3: VERIFY — suite contract + edge exhaustivo
- **Archivos:** `integrations/llamaindex/tests/test_vectorstore.py`
- **Acción:** Ejecutar full suite + casos borde específicos del contrato: `test_method_type_hints_resolve`, `test_private_attrs_declared_and_serialization_clean`, `test_client_and_namespace_properties`, roundtrip `add`→`get_nodes` preserves text. Verificar que `model_json_schema` tampoco expone private attrs (extra). Limpiar TEMP si disk full (pytest-of-Eros).
- **Verify:** `python -m pytest integrations/llamaindex/tests -q` → 23 passed (11.54s); `python -m pytest integrations/llamaindex/tests/test_vectorstore.py::test_method_type_hints_resolve -xvs` ✅; `test_private_attrs_declared_and_serialization_clean` ✅; manual `model_dump` asserts ✅
- **Estado:** ✅ DONE (2026-08-27 — 23 passed verificado, disk clean 128GB free)

### Step 4: CIERRE — verify full + recitation + handoff (no commit per prompt)
- **Archivos:** `.opencode/skills/campaign-executor/tasks/QW-3.md`, `docs/plans/2026-08-25-integrations-research-wins.md`
- **Acción:** Actualizar task file estado a ✅ COMPLETED, sync recitation; actualizar plan file recitation si aplica (wave 1 QW-3); no commit (prompt dice no commit); producir bloque RESULTADO; handoff a QW-4
- **Verify:** `git status --short` muestra solo task file modificado (untracked → tracked), sin cambios en vectorstore.py (no diff); `campaign_update_task_state` recitation OK si MCP disponible
- **Estado:** ✅ DONE (2026-08-27)

## Dependencias
- Ninguna (Wave 1 QW-3 independiente; QW-1/QW-2 disjuntos; QW-4 depende de Wave 1 pero no bloquea esta)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (leaf, task deny) — verificación mecánica, no implementa
- **Enfoque:** approach PrivateAttr + import completo es correcto vs alternativa forward ref string + model_rebuild(); alternativa requeriría `model_rebuild()` explícito y string annotations, más frágil. Cargo audit no aplica (no Rust). Decisión de no añadir `default=` explícito en PrivateAttr es correcta por ponytail (no requerido por contrato, consistente con crewai pattern; default Undefined no usado dado que __init__ siempre asigna).
- **Cómo se probó:** evidencia mecánica real (no auto-reporte): `typing.get_type_hints` OK, `model_dump` sin leaks OK, `python -m pytest integrations/llamaindex/tests -q` 23 passed (11.54s) con warnings solo de deprecations conocidas, `grep -n PrivateAttr` 4 líneas, codegraph blast radius 22 símbolos. No se inventaron salidas.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve — contrato cumple, no se requieren cambios

## Notas

- Fix ya merged en 96e143ec (2026-08-26) + 0ae070ca: `MetadataFilter` import añadido, 4 PrivateAttr declarados. Esta tarea pipeline-full re-verifica contrato y documenta evidencia source-driven; cierra como verify-only con evidencia mecánica (ponytail: no editar código que ya pasa).
- PrivateAttr sin default vs con default: elegido sin default por consistencia con `integrations/crewai/vantadb_crewai/vectorstore.py:PrivateAttr` y porque `VantaDBVectorStore.__init__` siempre asigna (`super().__init__(**kwargs)` luego `self._namespace = namespace` etc.). Añadir `default=...` sería hardening para `model_validate` sin `__init__`, pero no requerido por contrato y diluye minimal diff. Documentado en Deuda pendiente como mejora menor no bloqueante.
- `from __future__ import annotations` hace que todas las anotaciones sean strings → `get_type_hints` evalúa en runtime globals. Si falta import, falla NameError. Fix asegura imports completos.
- Disk full incident 2026-08-27: `C:` llegó a 0 free por `pytest-of-Eros` (10GB) + 30 tmp dirs (0.31GB c/u). Limpiado `pytest-of-Eros` → 128GB free, suite pasa 23/23 (antes 11 passed + 12 errors StorageFull). Lección registrada en lessons.md si aplica.
- No commit en esta iteración por instrucción prompt ("no commit, RESULTADO") — task file queda untracked/modified para handoff; el orquestador decide commit.

## Context Save Point
- **Fecha:** 2026-08-27T01:30
- **Branch:** develop
- **CI pendiente:** no — local pytest 23 passed ya verificado 11.54s
- **Decisiones:** mantener fix actual (PrivateAttr sin default, MetadataFilter import completo), no añadir factory/default extra; ponytail ladder: reuse existing pattern > stdlib > minimal diff
- **Problemas conocidos:** ninguno — contrato verde
- **Próxima tarea:** QW-4 (dedup gemelos ollama/openai)

