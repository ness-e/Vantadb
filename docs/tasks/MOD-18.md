# MOD-18: Consolidar stubs `.pyi` duplicados + test anti-drift firma↔stub

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 6)
- **Fuente:** backlog — stubs `.pyi` duplicados y desactualizados (put_batch type, métodos faltantes)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Python SDK
- **Turns estimados:** 20
- **Creado:** 2026-08-25T14:30
- **last-synced:** 2026-08-25T15:10
- **Estado:** ✅ COMPLETED (implementación; commit lo ejecuta el lead)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes (verify completo; commit = lead)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | type checkers / editores (mypy, pyright, IDE) que consumen la wheel; `integrations/dspy/vantadb_dspy/vectorstore.py`, `integrations/llamaindex/*` (importan `vanta.VantaDB`, `db.search_memory`, `db.list_memory`, `db.put`, `db.delete_memory`) |
| Callees | `vantadb-python/src/lib.rs` + `src/types.rs` (módulo nativo compilado `vantadb_py.pyd` — fuente de verdad de firmas) |
| Implicaciones | No rompe contratos runtime: los `.pyi` no se ejecutan. Cambios de firma son ADITIVOS (params/métodos faltantes) + corrección de return types. No cambia API pública core. No toca performance ni requiere migración. Tests existentes no dependen de los stubs. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-python/vantadb_py/__init__.pyi` (469L), `vantadb-python/vantadb_py/vantadb_py.pyi` (194L), `vantadb-python/vantadb_py/__init__.py` (417L), `vantadb-python/vantadb/__init__.py` (21L), `vantadb-python/pyproject.toml` (53L), `vantadb-python/tests/conftest.py` (88L); secciones clave de `vantadb-python/src/lib.rs` (subclients 220-379, put_batch 482-661, getters 2128-2172, pymodule 2315-2328).
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `pyproject.toml [tool.maturin] include = ["vantadb_py/py.typed", "vantadb_py/*.pyi", "vantadb/__init__.py"]` → ambos `.pyi` viajan en la wheel. `__init__.py` hace `from .vantadb_py import (VantaDB, VantaListResult, VantaMemoryRecord, VantaSearchHit, VantaVector, __version__, connect)` → el stub del módulo nativo debe tipar esas clases.
- **Archivos que referencian a los editados (referencias entrantes):** grep de imports de `vantadb_py`/`vanta`: `integrations/dspy/vantadb_dspy/vectorstore.py`, `integrations/llamaindex/vantadb_llamaindex/vectorstore.py`, tests del SDK, conftest.py. Ninguno importa los `.pyi` directamente (no son ejecutables).
- **Veredicto impacto:** BAJO para runtime (los `.pyi` solo los consume el type checker; ningún import runtime los referencia). Los 2 stubs NO son duplicados idénticos: `vantadb_py.pyi` stubea el módulo nativo compilado (`vantadb_py.pyd`, sub-módulo de `vantadb_py.vantadb_py`) y `__init__.pyi` stubea el wrapper (`SearchRequest`, `AsyncVantaDB`, re-exports). **Consolidación = fuente única de verdad de firmas nativas en `vantadb_py.pyi` + re-export en `__init__.pyi`** (elimina duplicación de declaraciones de clases nativas), sin romper paths de import (ambos archivos quedan, pyproject include intacto).

## Contrato
"`python -m pytest tests/` verde (118+new pasando, 4 slow deselected); test anti-drift `tests/test_stub_drift.py` pasa contra el módulo compilado; `vantadb_py.pyi` declara TODOS los métodos nativos reales (verificado por el test); `put_batch`/`put_batch_raw` tipados `list[VantaMemoryRecord]` (no `list[dict]`); mypy si está instalado (no lo está → documentado); `cargo check -p vantadb` OK (sin cambios Rust)"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** estructura de import del paquete intacta — `from .vantadb_py import ...` en `__init__.py` (el módulo compilado se llama `vantadb_py.pyd`, stubeado por `vantadb_py.pyi`); `vantadb/__init__.py` alias (`import vantadb` = re-export de `vantadb_py`, tipado vía `__init__.pyi` sin stub propio); pyproject `module-name="vantadb_py"` + include `vantadb_py/*.pyi` sin cambios; firmas de métodos ya correctas no cambian (solo se agregan params/métodos faltantes y se corrigen return types).
- **Comandos de verificación:** `python -m pytest tests/test_stub_drift.py -q` ✅; `python -m pytest tests/ -q` ✅ (118 + nuevos); `python -c "import ast; [ast.parse(open(f, encoding='utf-8').read()) for f in ['vantadb_py/vantadb_py.pyi','vantadb_py/__init__.pyi']]"` ✅ (stubs parsean).
- **Deuda pendiente:** mypy no instalado en el entorno → verificación mypy documentada como NO CORRIDA (pendiente de instalar o CI). Nada más.

## Recitation (canónico — estructura única)
- `activeGoal`: MOD-18 — consolidar stubs `.pyi` + test anti-drift firma↔stub
- `lastAction`: DISCOVERY completo (codegraph + lectura de 7 archivos + inspect del módulo compilado) — drift mapeado en 6 categorías
- `result`: PARTIAL (en progreso)
- `nextAction`: Step 1 — escribir `tests/test_stub_drift.py` (RED) y verificar que falla contra stubs actuales
- `contract`:
  - verificacion: `python -m pytest tests/test_stub_drift.py -q` + `python -m pytest tests/ -q` (118 baseline + nuevos) + parse de stubs con ast
  - evidencia:
    - claim: put_batch nativo retorna `Vec<VantaPyMemoryRecord>` (stub decía `list[dict]`)
      evidencia: `vantadb-python/src/lib.rs:494` + `inspect.signature(vantadb_py.VantaDB.put_batch)` → `(self, /, entries, keys=None, ...)`
      confianza: alta
    - claim: VantaDB nativo expone 6 métodos ausentes del stub (`bulk_import`, `bulk_import_bytes`, `reindex_hnsw_from_text`, `search_batch_requests`, `supersede`, `recover_archived_nodes`) y 4 params ausentes (`exclude_superseded` en search_memory/list_memory, `direction` en graph_bfs/graph_dfs, `created_at_ms` en add_edge)
      evidencia: `dir(vantadb_py.VantaDB)` + `inspect.signature` (módulo compilado 0.5.0)
      confianza: alta
    - claim: los 2 stubs son capas distintas (nativo vs wrapper), ambos necesarios — consolidación = re-export, no borrado
      evidencia: `pyproject.toml` include + `__init__.py` import de `.vantadb_py` + `vantadb/__init__.py` alias
      confianza: alta
  - artefactos: `.opencode/skills/campaign-executor/tasks/MOD-18.md`
  - invariantes: estructura de import intacta (3 capas: `vantadb_py.pyd` → `vantadb_py/__init__.py` → `vantadb/__init__.py`), pyproject sin cambios, métodos correctos no se tocan
  - deuda: mypy no instalado — verify mypy pendiente (NO CORRIDO)
  - queda_pendiente: el lead verifica mecánico y commitea (sub-agentes NO commitean)
- `nextTask`: MOD-20 (Wave 3, misma área Python — coordinar archivos si solapa)

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva. El cambio reduce deuda de DX (stubs desactualizados).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| Task | Contrato verificable: test anti-drift GREEN contra módulo compilado; suite pytest completa verde; stubs parsean |
| Commit | Lo ejecuta el LEAD (sub-agente NO commitea): commit atómico convencional (`fix(python): ...`) |
| Release | No aplica (no toca release) — justificado: solo `.pyi` + test |

## Herramientas necesarias
- Terminal Python (python 3.11.9, pytest 9.1.1, numpy 2.4.6)
- codegraph_explore (blast radius) — usado

## Investigation Notes
- Estructura real del paquete (3 capas): (1) `vantadb_py.pyd` — módulo compilado PyO3, stubeado por `vantadb_py.pyi`; (2) `vantadb_py/__init__.py` — wrapper Python puro (SearchRequest, AsyncVantaDB, re-exports), stubeado por `__init__.pyi`; (3) `vantadb/__init__.py` — alias puro (`from vantadb_py import *`), tipado transitivamente vía `__init__.pyi` (mypy resuelve star-import con `__all__` tipado → NO necesita stub propio, decisión ponytail).
- Subclients nativos (`MemoryClient`/`GraphClient`/`SystemClient`/`WikiClient`) NO son names module-level (`dir(vantadb_py.vantadb_py)` no los muestra) pero los getters `db.memory|graph|system|wiki` los devuelven → se declaran en `vantadb_py.pyi` como tipos internos. Verificado: `type(db.memory).__name__ == 'MemoryClient'` con exactamente los métodos del macro `forward_to_db!` (lib.rs:282-338).
- Drift completo mapeado (nativo = verdad, vía `inspect.signature` del módulo compilado):
  1. `put_batch`/`put_batch_raw` → retorno real `list[VantaMemoryRecord]` (stub: `list[dict]`) ❌
  2. `put_batch.entries` → REAL requerido posicional (puede ser None); stub lo daba opcional con default ❌
  3. VantaDB stub sin 6 métodos: `bulk_import`, `bulk_import_bytes`, `reindex_hnsw_from_text`, `search_batch_requests`, `supersede`, `recover_archived_nodes` ❌
  4. Params faltantes: `exclude_superseded` (search_memory/list_memory), `direction` (graph_bfs/graph_dfs), `created_at_ms` (add_edge) ❌
  5. AsyncVantaDB stub sin 5 métodos (`supersede`, `search_batch_requests`, `bulk_import`, `bulk_import_bytes`, `reindex_hnsw_from_text`) + `max_concurrency` en `__init__` + mismos params faltantes ❌
  6. `vantadb_py.pyi` sin subclients ni VantaVectorIter/VantaListResultIter ❌
- FASE SECURITY: no aplica (no toca trust boundaries — los `.pyi` no se ejecutan; no agrega/quita dependencias).
- FASE PERFORMANCE: no aplica (no toca hot paths; stubs no se ejecutan).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — estructura real y firmas verificadas contra módulo compilado |
| Pendientes de ejecución (downhill) | 4 steps |
| % completado | 10% |

## Steps

### Step 1: Test anti-drift RED
- **Archivos:** `vantadb-python/tests/test_stub_drift.py` (nuevo)
- **Acción:** escribir test anti-drift con `ast` (parse de stubs) + `inspect` (firmas reales del módulo compilado): (1) paridad de métodos y props de VantaDB vs stub; (2) paridad de params (nombres + requeridos) por método compartido; (3) paridad de subclients memory/graph/system/wiki; (4) return type de put_batch/put_batch_raw contiene `VantaMemoryRecord`; (5) AsyncVantaDB (wrapper puro, no requiere módulo nativo): paridad de métodos + params; (6) `__all__` de `__init__.py` == `__init__.pyi`. Skip limpio con `importorskip` si el módulo compilado no está (CI sin maturin develop).
- **Verify:** `python -m pytest tests/test_stub_drift.py -q` → FALLA (RED) contra stubs actuales
- **Estado:** ✅ DONE — RED confirmado: 6/7 fallan contra stubs viejos (exactamente los drift mapeados); 7/7 GREEN tras fix de stubs + 1 bug del propio test (dunder filter en `_assert_method_parity`, `_stub_required_params` excluye `self`)

### Step 2: Reescritura `vantadb_py.pyi` (fuente única de verdad nativa)
- **Archivos:** `vantadb-python/vantadb_py/vantadb_py.pyi`
- **Acción:** declarar módulo nativo completo con firmas REALES: VantaVector, VantaVectorIter, VantaSearchHit, VantaMemoryRecord, VantaListResult, VantaListResultIter, VantaDB (48 métodos + 4 getters con params corregidos: exclude_superseded, direction, created_at_ms, entries requerido, returns list[VantaMemoryRecord]), MemoryClient (18), GraphClient (10), SystemClient (17), WikiClient (1), connect, __version__. Docstring documentando la estructura de 3 capas.
- **Verify:** `python -c "import ast; ast.parse(open('vantadb_py/vantadb_py.pyi', encoding='utf-8').read())"` + test anti-drift GREEN
- **Estado:** ✅ DONE — rewrite completo (383 líneas), subclients tipados como tipos internos (no son names module-level)

### Step 3: Reescritura `__init__.pyi` (wrapper re-export)
- **Archivos:** `vantadb-python/vantadb_py/__init__.pyi`
- **Acción:** re-export de `.vantadb_py` (sin re-declarar clases nativas), SearchRequest (dataclass real), AsyncVantaDB completo (5 métodos faltantes + max_concurrency + params corregidos), `__all__` espejo de `__init__.py`.
- **Verify:** `python -c "import ast; ast.parse(...)"` + `python -m pytest tests/test_stub_drift.py -q` GREEN
- **Estado:** ✅ DONE — re-export + SearchRequest + AsyncVantaDB completo (48 métodos)

### Step 4: Verify full
- **Archivos:** ninguno (solo comandos)
- **Acción:** suite completa + checks de contrato
- **Verify:** `python -m pytest tests/ -q` (118+ pasando, 4 deselected); `cargo check -p vantadb` (sin cambios Rust, confirmar que nada se rompió); mypy → documentar NO INSTALADO
- **Estado:** ✅ DONE — `python -m pytest tests/ -q` → **125 passed, 4 deselected** en 160s; `ast.parse` de ambos stubs OK; `cargo check -p vantadb` Finished dev OK; mypy/pyright NO instalados → verify mypy NO CORRIDO (documentado, deuda pendiente)

## Context Save Point (2026-08-25T15:10)
- **Trabajo hecho:** 3 archivos tocados — `vantadb_py.pyi` (nativo, fuente única de verdad), `__init__.pyi` (wrapper re-export), `tests/test_stub_drift.py` (7 tests anti-drift). Suite 125 passed. Sin commits (regla: el lead commitea).
- **Próximo paso para el LEAD:** verificar mecánico (pytest + cargo check ya corridos), ejecutar `git add vantadb-python/vantadb_py/__init__.pyi vantadb-python/vantadb_py/vantadb_py.pyi vantadb-python/tests/test_stub_drift.py` y commitear `fix(python): consolidate vantadb_py stubs + anti-drift test (MOD-18)`. El task file también va (`.opencode/skills/campaign-executor/tasks/MOD-18.md`).
- **Riesgos residuales:** ninguno — los `.pyi` no afectan runtime; el test anti-drift es la barrera futura. Deuda: mypy/pyright no instalados (verify mypy pendiente, opcional).

## Dependencias
- Ninguna (Wave 3 independiente; MOD-20 misma área Python pero el lead coordina commits)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** pendiente — el lead decide (vanta-review para approach; el gate se cumple en el flujo del lead antes de marcar COMPLETED definitivo)
- **Enfoque:** pendiente
- **Cómo se probó:** evidencia = test anti-drift GREEN contra módulo compilado (no auto-reporte)
- **Checklist anti-hábitos tóxicos:** no aplica aún (implementación en curso)
- **Veredicto:** pendiente

## Notas
- Decisión de consolidación: NO borrar ninguno de los 2 stubs — son capas distintas (módulo nativo vs wrapper). "1 stub consolidado" se interpreta como: firmas nativas declaradas UNA vez (en `vantadb_py.pyi`), wrapper re-exporta. Alternativa evaluada y descartada: borrar `vantadb_py.pyi` y declarar todo en `__init__.pyi` → rompería el typing del sub-módulo `vantadb_py.vantadb_py` que `__init__.py` importa (mypy necesita el stub del módulo compilado en su path de resolución).
- Gate D: blast radius 3 archivos, sin hot path, sin API pública core, contrato claro → NO requiere question al usuario.
- El test anti-drift es la barrera: si mañana un método PyO3 cambia de firma sin actualizar el stub, el test falla.