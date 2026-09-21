# SHOW-04 — demo agente-con-memoria (prueba de aceptación viva del MVP)

> **Plan:** `docs/plans/2026-09-17-mvp-memoria-agentes.md` (Wave0, Task 3) · **Campaign:** b2ece025-e9d3-4f8b-835d-1d0143a86b66
> **Ruta:** vanta-worker · **Branch:** develop · **Commit:** `feat: SHOW-04 — ...` (NO PUSH) · **Appetite:** 3d
> **Estado:** ⬜ PENDING → ⏳ IN PROGRESS (esta sesión)

## 1. TAREA: objetivo + contrato + AC

**Objetivo:** demo agente-con-memoria = prueba de aceptación viva del MVP. Un agente guarda un dato en la sesión 1 y lo recuerda en la sesión 2, con assert mecánico (no timing), 1 comando, 100% local, cero credenciales.

**Contrato (ley — si no se cumple, no está completa):**
1. 2 corridas de motor (sesión 1 guarda → close/flush → sesión 2 reabre el mismo DB dir y recuerda) con **assert mecánico de contenido** (el payload recuperado contiene el dato guardado; `search` devuelve el hit esperado).
2. **1 comando** sin credenciales: `python examples/agent_memory_cli/agent_memory_demo.py` (orquesta ambas sesiones internamente) + `python -m pytest examples/agent_memory_cli/test_agent_memory_cli.py` como prueba e2e.
3. **Determinista:** vectores explícitos fijos (sin proveedor de embeddings, sin red, sin ONNX), asserts de contenido exacto, `tempfile` aislado en tests. Nada flaky.

**AC:**
- (a) corrida 1 guarda dato; corrida 2 lo recuerda (assert mecánico de contenido, no de timing).
- (b) 1 comando, 100% local, cero credenciales.
- (c) determinista (asserts de contenido, no flaky timing).

**Diseño elegido (fricción mínima):** demo **Python** (`import vantadb` canónico, `Client`, vectores explícitos 3-d fijos). Reusa el patrón probado de `examples/python/agent_memory.py` (put + search + metadata) y de `test_sdk.py::test_put_get_list_search` + persistencia por reopen. NO usa scenes/`inject_context` como API directa (requieren captura previa de escenas / thread numérico y sumarían fricción + flakiness) — scenes + `inject_context` quedan como **base conceptual** (el motor de memoria es el mismo que exponen vía MCP) y se citan en el README como siguiente paso. Si el revisor exige scenes literal, el fallback es una variante `scene_*` como slice 2 (no en este commit).

## 2. ARCHIVOS: clave + relacionados + prohibidos

**Clave (tocar/crear — SOLO esto):**
- `examples/agent_memory_cli/` (NUEVO dir; `examples/` hoy tiene `demo/python/rust/colab` pero NO `agent_memory_cli` — verificado 2026-09-17 vía Read):
  - `agent_memory_demo.py` (demo 1-comando, ~90 líneas)
  - `test_agent_memory_cli.py` (pytest e2e, ~60 líneas)
  - `README.md` (uso + contrato + local-first)

**Relacionados (lectura, NO modificación):**
- `examples/python/agent_memory.py` (plantilla de estructura put/search/flush/close)
- `examples/README.md` (tabla de ejemplos; NO se edita en este commit — scope discipline)
- `vantadb-python/tests/test_sdk.py:232-271` (patrón `db.put` + `db.memory.get` + `db.search`)
- `vantadb-python/vantadb/__init__.py` (import canónico `Client`)
- `vanta-memory` scenes + `vantadb-mcp/src/handlers/tools.rs:1919` (`inject_context`) + `vantadb-mcp/src/scenes.rs` (base conceptual)

**PROHIBIDOS (no tocar bajo ningún concepto):**
- `reparacion.bat` (untracked ajeno), `.opencode` (submodule modificado ajeno), `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`, `completitions/*` (`completions/_vanta-cli*` modificados ajenos), `desktop/src-tauri/Cargo.lock`, `stash@{0}` GOV-C4
- `src/llm.rs` (FIND-100 en paralelo), `vanta-memory/` + `vantadb-mcp/src/handlers/` (FIND-107 en paralelo)
- WIP ajeno visible en `git status` (`docs/Backlog.md`, `docs/plans/...`, `completions/*`) — commit SOLO `examples/agent_memory_cli/` + `docs/tasks/SHOW-04.md`

## Impacto mapeado (Regla 0) — poblado ANTES del primer edit

- **Archivos leídos completos:** `examples/README.md`, `examples/python/agent_memory.py`, `vantadb-python/vantadb/__init__.py`, `vantadb-python/vantadb_py/__init__.py` (parcial: API Client), `test_sdk.py:1-120 + 232-271`, `.opencode/rules/server-mcp.md`, `.opencode/rules/python-bindings.md`, `.opencode/references/test-suite.md`, `.opencode/references/definition-of-done.md`
- **Referencias hacia dentro (qué importa el nuevo código):** solo `vantadb` (binding instalado 0.5.0, sistema) + stdlib (`argparse`, `shutil`, `sys`, `tempfile`). Nada del workspace core (demo corre contra el binding instalado — cero acoplamiento de build).
- **Referencias entrantes (quién importa lo nuevo):** nadie (ejemplo aislado; `examples/README.md` NO se edita en este commit). Sin callers → blast radius de escritura = 0 archivos existentes.
- **Blast radius lectura (codegraph/grep):** `inject_context` (handlers/tools.rs:1919, thread_id numérico), `scene_*` (scenes.rs:3 tools, requieren captura previa), `db.put`/`memory.get`/`search` (patrón estable en docenas de tests). Sin hot paths (vector/, engine.rs, WAL, storage) tocados.
- **Veredicto:** ADITIVO puro (3 archivos nuevos, 0 modificados). Revertible con `git rm`. Sin símbolos públicos nuevos (ejemplo, no librería → sin tabla Spec). Gate D: **no disparado** (≤3 archivos, sin API pública, sin hot path, contrato no ambiguo).

## 3. DEPENDENCIAS

Wave0 sin dependencias (paralela FIND-100 + FIND-107, archivos disjuntos). Solo necesita motor actual (binding `vantadb` 0.5.0 ya instalado en `C:\Python314\python.exe` + `pytest 9.1.1` — verificado en vivo). `target/audit-venv` NO existe en esta máquina → se usa el python del sistema (documentado en README + RESULTADO). Stop: dependencia externa inevitable → documentar requisito, no forzar (no ocurrió). NextTask: FIND-103 (orquestador).

## 4. REFERENCIAS

- **Rules (lectura completa):** `.opencode/rules/server-mcp.md` (R-1/R-2/R-3 no aplican a ejemplo Python — sin server/handlers; se cita por marco normativo del plan) + `.opencode/rules/python-bindings.md` (R-1 batch/GIL no aplica — puts individuales; R-2 closure/GIL no aplica — sin callbacks; se respeta uso canónico `Client` + vectores explícitos).
- **Refs:** `testing-patterns.md` (AAA, DAMP, state-not-interactions), `test-suite.md` (pytest para Python), `definition-of-done.md` (standing checklist + DoD v1), `clean-code-clean-architecture.md` Ap. V (ejemplo simple, sin capas).
- **Commands:** `pipeline.md` (ejecución).
- **SPEC.md raíz:** éxito medible (demo recuerda entre sesiones).
- **Sin símbolo público nuevo → sin tabla Spec** (ejemplo, no API).

## 5. SKILLS (SDP real Paso 0b)

`campaign_discover_skills_v2` (phase BUILD, keywords demo/2-corridas/determinista/local/scenes-inject_context, maxSkills 8) → 8 devueltas.
- **Cargadas y aplicadas:** `test-driven-development` (demo como test e2e: RED test inexistente → GREEN demo), `incremental-implementation` (1 slice vertical delgado: save→reopen→recall), `context-engineering` (context pack: rules → plan → source del slice + ejemplo patrón), `source-driven-development` (APIs verificadas en `test_sdk.py` + `__init__.py`, no de memoria).
- **Descartadas con motivo:** `doubt-driven-development` (stakes bajos: ejemplo aislado, sin prod/seguridad), `frontend-ui-engineering` (sin web/), `api-and-interface-design` (sin API nueva), `campaign-executor` (base, ya activa vía harness).
- `systematic-debugging` (sugerida por plan): on-standby, no cargada — sin bug que diagnosticar; se carga si VERIFY falla.
- `documentation-and-adrs`: cubierta por el README del slice (sin ADR: ejemplo sin tradeoff arquitectónico).
- **SDP:** test-driven-development, incremental-implementation, context-engineering, source-driven-development (+ base campaign-executor/progreso/ponytail)

## 6. HERRAMIENTAS + MCP

- `python examples/agent_memory_cli/agent_memory_demo.py` (1 comando, 2 sesiones internas con assert) + `python -m pytest examples/agent_memory_cli/test_agent_memory_cli.py -v` (e2e).
- `campaign_verify_cmd` para el contrato (si bug exit -1 → bash directa + mención en RESULTADO).
- codegraph/grep SOLO lectura (scenes/`inject_context` localizados, no modificados). Internet N/A (todo local; Regla de Validación satisfecha con fuente interna: tests + binding instalado).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY — hecha, no re-derivar)

- `examples/` tiene `demo/python/rust/colab`, NO `agent_memory_cli` ✅ (hueco real).
- `examples/python/agent_memory.py`: single-run con cleanup final (`shutil.rmtree`) → NO prueba persistencia entre sesiones (lo reutilizamos como plantilla, invertimos el cleanup).
- Patrón persistencia probado: `db.put(ns,key,payload,metadata,vector)` → `flush/close` → reopen mismo path → `db.memory.get` / `db.search` (test_sdk.py:232-271 + persist 345-350).
- Import canónico: `from vantadb import Client` (sin DeprecationWarning; `vantadb_py` legacy advierte).
- `inject_context`: handler `tools.rs:1919`, `thread_id` numérico obligatorio (AUD-050), no-idempotente. `scene_*`: `scenes.rs` 3 tools read-only, requieren captura previa de escenas. Ambos sobredimensionados para el contrato → base conceptual, no API directa (decisión documentada §1).

## 8. INVESTIGACIÓN PROBLEMA

- **Determinismo:** vectores fijos 3-d (`[1.0,0,0]` / `[0,1.0,0]`), payloads exactos con asserts `in`/`==` sobre contenido, DB temporal única por corrida (`tempfile.mkdtemp`), sin sleeps/retries/timing. Sin embedding provider → sin nondeterminismo de modelo.
- **Local-first:** vectores explícitos (el motor nunca llama a proveedor externo), sin env vars, sin credenciales, sin red. `grep` de `os.environ`/http en el slice → 0 hits (verificable).

## 9. INVESTIGACIÓN INTERNET

N/A (todo local — por diseño del contrato).

## 10. STEPS ATÓMICOS (~100 líneas c/u)

- [x] **Step 1 — RED (TDD):** escribir `test_agent_memory_cli.py` (e2e: save→close→reopen→recall con asserts de contenido) y confirmar que FALLA (demo aún no existe). Contratao: `python -m pytest examples/agent_memory_cli/ -v` → 2 failed (import error).
- [x] **Step 2 — GREEN (slice vertical):** implementar `agent_memory_demo.py` (`sesion_1_guardar` + `sesion_2_recordar` + `main` 1-comando con asserts) + `README.md`. Contrato: demo imprime `✅ SESION 1/2 OK` + `MEMORIA VERIFICADA`.
- [x] **Step 3 — VERIFY + CIERRE:** 2 corridas mecánicas (demo directa + pytest) + `campaign_verify_cmd` (o bash directa si exit -1) + `ocr-review.ps1` advisory + commit `feat:` (SOLO 4 paths: 3 de examples + este task file) + `campaign_update_task_state(completed)` + RESULTADO §7.

## 11. VALIDACIÓN + CIERRE

- Verify contrato: demo 1-comando (2 sesiones internas, assert contenido) + pytest e2e verde.
- Full según stack Python: `python -m pytest examples/agent_memory_cli/ -v` (+ `py_compile`; sin fmt/clippy — no hay Rust; `cargo` no se toca).
- OCR delegation advisory (`pwsh dev-tools/ocr-review.ps1`) — ejemplo Python nuevo, sin trust boundaries nuevos.
- DoD 3 niveles: Correctness (AC a+b+c verificados en runtime) + Quality (nombres intencionales, sin duplicación, scope puro) + Ship (docs README en el mismo commit, rollback `git rm`, sin deuda silenciosa).
- P2-01 orquestador + Gates D/V/C vía `question` (D: no disparado — ver §Impacto; V: solo si 2 fallas mismo-error; C: colaterales vía findings.md).

## Context Save Point

- **Hecho:** plan + rules + refs + código base leídos; entorno verificado (`vantadb` 0.5.0 + pytest 9.1.1 en system python; sin audit-venv); WIP ajeno mapeado (intocable); task file creado con DISCOVERY completo.
- **Próximo:** Step 1 RED → crear `examples/agent_memory_cli/test_agent_memory_cli.py`, correr pytest, confirmar FAIL por razón correcta (módulo demo inexistente).
- **Para reanudar:** leer este file + `git status --short` (verificar que el WIP ajeno sigue intacto) y continuar en el primer step ⬜.
