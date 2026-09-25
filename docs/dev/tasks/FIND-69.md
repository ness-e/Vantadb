# FIND-69 — dspy sin framework (fallback object explota)

> Campaign: `6ab26f3f-cf16-4416-9255-c18cca0bcaf0` · Plan: `docs/dev/plans/2026-09-15-find-correcciones.md`
> Estado: ⬜ PENDING → IN PROGRESS (Wave6, disjunto de FIND-66 Formula y FIND-75 wasm)
> Appetite: 2h · Esfuerzo: 🟢 · Prioridad: 🟡 · Ruta: vanta-worker
> Branch: `develop` · Commit previsto: `fix: FIND-69 — ...` (solo propios)
> nextTask: Wave7 (FIND-84/85/80, orquestador decide)
> Sin símbolos nuevos → sin Spec (fix mínimo sobre comportamiento existente).

## SDP

`campaign_discover_skills_v2 archivosClave="integrations/dspy/vantadb_dspy/vectorstore.py:19-24,62, integrations/dspy/tests/" phase="BUILD" contractKeywords=["python-fallback","dspy-adapter","pytest-matrix","optional-dependency"]` →
base: campaign-executor, progreso +
lifecycle: incremental-implementation, test-driven-development, context-engineering,
source-driven-development, doubt-driven-development (+ frontend-ui-engineering,
api-and-interface-design descartadas por inaplicables — sin web/, sin API nueva).
Cargada: systematic-debugging (bug → inline, fases 1-4 abajo).
`SDP: systematic-debugging (beyond base; frontend-ui/api descartadas)`

## Gate D (question-gates.md)

Blast radius = 1 `__init__` + suite `integrations/dspy/tests/` (8 tests). Sin símbolos
públicos nuevos, sin hot path Rust, sin API pública, contrato no ambiguo
(matriz ambos modos + forward shape), fix (no feature-add) → Gate D **no disparado**,
sin `question`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (slice):** `integrations/dspy/vantadb_dspy/vectorstore.py`
  (179 líneas, entero), `integrations/dspy/tests/test_vectorstore.py` (109 líneas, entero),
  `integrations/dspy/tests/conftest.py` (2 líneas), `integrations/dspy/vantadb_dspy/__init__.py`
  (3 líneas); `codegraph_explore "dspy vectorstore fallback VantaDBRetriever forward"`.
- **Referencias hacia dentro (lo que `__init__` toca):** `DSPyRetrieve` (base:
  `dspy.Retrieve` real o `object` fallback `:24`), `vanta.VantaDB(...)` (constructor
  persistente), `DEFAULT_NAMESPACE`/`DEFAULT_TOP_K`; `forward()` lee `self.k` (`:95`),
  `dump_state`/`load_state` leen/escriben `self.k` (`:129,:147`).
- **Referencias entrantes:** `VantaDBRetriever` tiene 7 callers (tests ×6 + ejemplo
  `examples/python/dspy_retriever.py` — clase homónima local, NO hereda de esta);
  `forward` 1 caller (`__call__`); `DSPyRetrieve` 1 caller (declaración de clase).
- **Veredicto:** impacto = `__init__` + atributo `self.k` + suite dspy. Prohibidos
  intactos: `.opencode/`, `completions/`, tauri lock, stash@{0}, FIND-66 (Formula/),
  FIND-75 (wasm/), otros adapters `integrations/`.

## Root cause (systematic-debugging Fase 1)

Repro determinista sin dspy (runner: dspy NO instalado → fallback activo):
`VantaDBRetriever(db_path=..., namespace=...)` →
`vectorstore.py:62 super().__init__(k=k)` → `object.__init__()` no acepta `k` →
`TypeError: object.__init__() takes exactly one argument` — garantizado siempre
sin framework. Evidencia: `python -c` repro (mismo traceback).
Hipótesis única: tolerar el `super().__init__(k=k)` cuando la base es `object`
(try/except TypeError) + fijar `self.k = k` explícito (hoy `k` solo lo guarda el
padre real; en fallback `forward`/`dump_state`/`load_state` leerían un atributo
inexistente). Path CON framework intacto: mismo `k`, `super()` se ejecuta igual.

## Steps

- [x] **Step 1 (único, ~5 líneas):** PLAN→ACT→VERIFY — try/except TypeError en
  `super().__init__(k=k)` + `self.k = k` explícito; matriz SIN dspy verde parcial +
  CON dspy documentado (sin red, no instalable) + `forward()` shape intacto. ✅
  - Repro pre-fix: `TypeError: object.__init__() takes exactly one argument` ✅ (gate confirmado)
  - Fallback aislado post-fix (DB stub): INIT OK k=3, forward `_Prediction.passages`
    (`['hello world']` match / `[]` nomatch / k-kwarg OK), empty → `passages=[]`,
    `dump_state` + `load_state` OK ✅ (shape contrato probado SIN dspy)
  - `python -m py_compile` (4 archivos) → exit 0 ✅
  - `git diff --check` → limpio ✅
  - `pytest integrations/dspy/tests/` → 5 failed + 3 errors, TODOS en
    `vanta.VantaDB` AttributeError (drift SDK pre-existente, 9 adapters) → `FIND-94` en Backlog ❌ (no regresión: pre-fix fallaba antes, con TypeError)
  - CON dspy: no instalado + sin red → documentado, no verificado (deuda explícita, permite el contrato)
  - Review P2-01: vanta-review **approve** ✅ (nit no bloqueante: `except TypeError`
    amplio podría enmascarar TypeError interno del padre con-framework; se mantiene
    por robustez ante firmas Retrieve distintas — documentado, no churn)
  - DoD: gate TypeError cerrado; sin deuda neta; WIP ajeno intacto.

## Contrato

- `pytest integrations/dspy/tests/` verde SIN dspy + CON dspy (si hay instalado;
  si no, documentar) + preserva `forward()` shape. `py_compile` 0.

## Save Point

- Pre-fix: repro `TypeError object.__init__()` confirmado; `dspy` NO instalado;
  `self.k` nunca asignado en fallback (latente, mismo fix lo cubre).
- WIP ajeno intacto (no tocar): `.opencode/`, `completions/`, tauri lock,
  stash@{0}, FIND-66, FIND-75, otros adapters.
