# FIND-82: assert de conteo en test-mcp.py (gate que no gatea)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-15-find-correcciones.md (Task 14, Wave4)
- **Fuente:** docs/dev/Backlog.md fila FIND-82 + plan Task 14
- **Esfuerzo:** 🟡 1d (appetite 4h)
- **Prioridad:** 🔴 Alta
- **Tipo:** Test (Python script, MCP handshake)
- **Turns estimados:** 8
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 (causa raíz confirmada en DISCOVERY)
- **Pendientes (downhill):** 2 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno en código (script standalone invocado manualmente/CI); SKILL.md §Testing lo referencia como comando (`python scripts/test-mcp.py`) |
| Callees | binario MCP bajo test (`vanta-cli server --mcp` → delega a `vantadb-server --mcp` → `vantadb_mcp::run_stdio_server_auto`); `VANTADB_MCP_PROFILE` env (full/dev/memory); `vantadb-mcp/src/handlers/tools.rs::handle_tools_list` + `profile_allowed_tools`; `resources.rs::handle_resources_list` (2); `prompts.rs::handle_prompts_list` (4) |
| Implicaciones | contrato de test cambia (script ahora falla si drift); comportamiento público MCP NO cambia (solo el harness); sin impacto en performance/memoria/serialización; sin migración; tests Rust existentes no afectados |

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0).

- **Archivos leídos (completos):** `skills/vantadb-mcp/scripts/test-mcp.py` (187L); `vantadb-mcp/src/config.rs` (142L, perfiles); `vantadb-mcp/src/handlers/resources.rs` (162L, 2 recursos); `vantadb-mcp/src/handlers/prompts.rs` (101L, 4 prompts); `skills/vantadb-mcp/SKILL.md` (§Testing :76-96); `skills/vantadb-mcp/references/configuration.md`; `skills/vantadb-mcp/references/mcp-protocol.md` (:7 dice 60 tools — stale, FUERA de scope FIND-83); `skills/vantadb-mcp/scripts/setup-vantadb.sh` (build `cargo install --manifest-path Cargo.toml --bin vanta-cli`); `src/cli_handlers/server.rs:253-349` (`vanta-cli server --mcp` delega a binario `vantadb-server --mcp` hijo); `vantadb-server/src/main.rs:62-65` (`run_stdio_server_auto`); `vantadb-mcp/tests/mcp_tests.rs:4459-4686` (contrato canónico: Full==79 exacto, Dev 30..38, Memory 15..22 + presencia por perfil)
- **Archivos referenciados hacia dentro (imports/dependencias):** test-mcp.py solo usa stdlib (`json/os/shutil/subprocess/sys/tempfile/threading`); lee env `VANTADB_MCP_BIN`; el servidor bajo test lee `VANTADB_MCP_PROFILE` + `VANTADB_STORAGE_PATH`
- **Archivos que referencian a los editados (referencias entrantes):** `skills/vantadb-mcp/SKILL.md:80-82,92-96` (documenta el comando, NO el conteo — no requiere sync); ningún `.py`/workflow lo importa
- **Veredicto impacto:** BAJO — 1 archivo `.py` standalone; el cambio solo endurece el harness (exit 1 si drift). Riesgo de falsos rojos mitigado con rangos Rust-canónicos para dev/memory + exacto solo en full.

## Contrato

`python skills/vantadb-mcp/scripts/test-mcp.py <binario-desde-fuente>` → handshake 4/4 verde con asserts por perfil; el mismo script contra binario stale (56 tools) → exit 1 con mensaje de drift; `git diff --check` limpio; `python -m py_compile` OK.

## Spec (SDD — Phase 1b)

NO es feature-add: la solución NO agrega símbolos públicos (`pub fn`/tool/endpoint/binding). Solo endurece un script `.py` existente con asserts + docstring. Sin sección Spec (gate mecánico no aplica; Gate D no dispara: blast radius 1 archivo, sin hot path/API pública, contrato no ambiguo).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** handshake MCP intacto (initialize→tools/list→resources/list→prompts/list, mismo orden y params); resolución de binario intacta (argv > env > PATH > target/); archivos prohibidos intactos (`.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, `docs/pipeline-state.json`, `tools.rs`, skills sync FIND-83, Rust core fuera de scope); WIP ajeno de Wave4 (FIND-77/83) no tocado.
- **Comandos de verificación:** `python skills/vantadb-mcp/scripts/test-mcp.py target/debug/vantadb-server.exe` (4/4 + asserts); `python -m py_compile skills/vantadb-mcp/scripts/test-mcp.py`; `git diff --check`
- **Deuda pendiente:** ninguna (nit conocido: `mcp-protocol.md:7` dice 60 tools stale → FIND-83, no este task)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Fuente |
|------------------------|--------|
| `activeGoal` | FIND-82 — assert de conteo en test-mcp.py |
| `lastAction` | DISCOVERY completo + causa raíz 56-vs-79 confirmada (vanta-cli delega a vantadb-server stale en PATH) |
| `result` | PARTIAL (task file creado, fix pendiente) |
| `nextAction` | Step 1: editar test-mcp.py (asserts por perfil + docstring build) |
| `contract` | ver §Contrato + §Invariantes |
| `nextTask` | FIND-83 (Wave4, archivo disjunto) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — el cambio es solo test-harness, no introduce deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato cumple + `py_compile` OK + tamper-test (drift → exit 1) + script verde con binario desde fuente |
| **Commit** | Commit atómico `test:` (solo `test-mcp.py` + este task file), `git diff --check` limpio |
| **Release** | N/A (harness, sin release; justificar en Notas) |

## Herramientas necesarias

- python (script + py_compile)
- cargo build (binario desde fuente para verificación)
- codegraph_explore (blast radius — hecho, no aportó: símbolos fuera del índice py)

**Skills cargadas (SDP):** campaign-executor (base task-system); source-driven-development (base MCP server); incremental-implementation (lifecycle BUILD, slices delgados); test-driven-development (lifecycle BUILD, RED=0 asserts→GREEN); context-engineering (lifecycle BUILD, context pack por slice); doubt-driven-development (lifecycle BUILD, stakes altos — gate decorativo); api-and-interface-design (lifecycle BUILD, boundary del harness). `frontend-ui-engineering` devuelta por SDP descartada (sin `web/` en scope — justificado). SDP keywords: mcp-handshake, test-asserts, profile-count, python-test. `systematic-debugging` aplica como bug (gate que no gatea): repro=script exit 0 con 56 tools + 0 asserts; hipótesis=sin asserts + binario stale enmascara drift (confirmada).

## Investigation Notes

- **Causa raíz (confirmada con evidencia, no hipótesis):** `vanta-cli server --mcp` NO implementa MCP — `src/cli_handlers/server.rs:253-349` spawnea `vantadb-server --mcp` hijo. `target/debug/vantadb-server.exe` NO existe; la resolución del script prefiere `vanta-cli` en PATH (`C:/Users/Eros/.cargo/bin/vanta-cli.exe`) que delega al `vantadb-server` instalado (stale). Ambos reportan **56 tools** (medido hoy contra los dos binarios), mientras la fuente tiene **79** (`mcp_tests.rs:4468-4473` Full==79 exacto). El script tiene **0 asserts** (`Select-String assert` = 0) → 4/4 verde con 56 tools = gate decorativo. Drift 79-vs-56 enmascarado exactamente como describe el contrato.
- **23 tools faltantes en el binario stale:** `memory_versions`, `memory_supersede`, `search_with_method`, `search_multi`, `memory_search`, `memory_recall`, `embed_texts` (7) + `remove_edge`, `write_axiom`, `delete_axiom`, `vacuum`, `snapshot_create`, `snapshot_restore`, `context_assemble`, `scene_*`×3 (10) + `thread_*`×6 (6) = 23. 56+23=79 ✓.
- **Conteos canónicos por perfil (fuente Rust, `tools.rs:1028-1190` + `mcp_tests.rs:4566-4686`):** Full==79 exacto (22 memory + 16 dev + 41 extended); Dev==38 actual con contrato rango 30..38; Memory==22 actual con contrato rango 15..22; resources==2 (`metrics://`, `schema://`); prompts==4 (`search_memory`, `analyze_namespace`, `summarize_context`, `query_builder`). Perfil vía `VANTADB_MCP_PROFILE` (full/dev/memory, default full — `config.rs:126-129`).
- **Build desde fuente (documentar en script):** `cargo build -p vantadb-server` (binario real MCP) o `cargo build --bin vanta-cli` + `cargo build --bin vantadb-server` (el wrapper `server --mcp` necesita al hijo en PATH o junto al exe). Instalación: `cargo install --manifest-path Cargo.toml --bin vanta-cli` (setup-vantadb.sh:26). El script ya acepta binario explícito por argv/`VANTADB_MCP_BIN` — el contrato es usarlo con binario recién compilado.
- **Web research:** no requerida — 0 ambigüedad externa (todo verificable en repo: conteos en `mcp_tests.rs`, perfiles en `config.rs`, delegación en `server.rs`). Sin URLs citadas → gate TSYS-13 N/A.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 2 (Step 1 implementar+verificar, Step 2 review+commit+cierre) |
| % completado | 100% (Steps 1-2 DONE + review approve) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A justificado: el cambio toca solo un harness de test local (stdio, single-user, sin trust boundary nuevo, sin input de usuario red, sin dependencias). No se carga `security-and-hardening` (sin superficie nueva).
- [x] **PERFORMANCE** — N/A justificado: sin hot path (script secuencial 4 requests, sin loops calientes). No se carga `performance-optimization`.

## Steps

### Step 1: Asserts por perfil + docstring build en test-mcp.py
- **Archivos:** `skills/vantadb-mcp/scripts/test-mcp.py`
- **Acción:** (a) docstring: build desde fuente (`cargo build -p vantadb-server`, wrapper necesita hijo, uso argv/`VANTADB_MCP_BIN`, perfiles `VANTADB_MCP_PROFILE`); (b) tabla `EXPECTED_TOOLS = {"full": 79 exacto, "dev": (30,38), "memory": (15,22)}` + `EXPECTED_RESOURCES=2` + `EXPECTED_PROMPTS=4` con fuentes citadas; (c) asserts tras cada `tools/resources/prompts-list` que fallan con mensaje de drift (esperado vs real + perfil + hint de rebuild); (d) `passed` solo si asserts OK; resumen final incluye perfil+conteos esperados.
- **Verify:** `python -m py_compile skills/vantadb-mcp/scripts/test-mcp.py` ✅ + `python skills/vantadb-mcp/scripts/test-mcp.py target/debug/vantadb-server.exe` → 4/4 exit 0 (79 tools full) ✅ + tamper-test stale (56 → 3/4 exit 1 drift) ✅ + dev 38/memory 22 4/4 ✅ + `git diff --check` ✅
- **Estado:** ✅ DONE

### Step 2: Review P2-01 + commit + cierre
- **Archivos:** `docs/dev/tasks/FIND-82.md` (este file), Backlog (vía progreso, no manual)
- **Acción:** review por agente distinto (vanta-review) → `git add` solo propios → `git commit -m "test: FIND-82 — ..."` → `campaign_memory_write` lesson → `campaign_update_task_state completed` → RESULTADO
- **Verify:** `git status --short` (solo 2 archivos propios) + commit hash
- **Estado:** ✅ DONE (review approve ses_f5a35e200ffeTO54udJT73r5ld)

## Dependencias

- Wave3 DONE (FIND-87/73/71 completadas — sin colisión de archivos).
- Wave4 paralela: FIND-77 (`tools.rs:1090` comentario) y FIND-83 (skills sync) — archivos DISJUNTOS, sin dependencia mutua. Orden con FIND-90 (Wave0): no aplica (FIND-90 tocó `resources.rs:139-145`+`tools.rs:1821,2847-2857` dispatch, ya mergeado 26e6ebc0; este task NO toca Rust).
- Next: Wave5 (FIND-67/68/81).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sub-agente distinto, task ses_f5a35e200ffeTO54udJT73r5ld)
- **Enfoque:** rangos Rust-canónicos por perfil correctos vs exactos globales (el Rust test usa exacto solo en full); `continue`-sin-`passed++` es el fail correcto (preserva handshake + resumen 3/4); no rebuildear en el fix, solo docstring (ponytail).
- **Cómo se probó:** revisor re-ejecutó `py_compile` exit 0 + `git diff --check` exit 0; ramas verificadas por lógica (56∉[79,79], 38∈[30,38], 22∈[15,22]) + evidencia del implementador (fresco 79 4/4 exit 0, stale 56 3/4 exit 1, dev/memory 4/4). Resources/prompts verificados contra fuente (resources.rs:13-29, prompts.rs:9-46). Perfil default full coincide con config.rs:16-17.
- **Checklist anti-hábitos tóxicos:** ✅ verificado por revisor (contrato mecánico, causa raíz con evidencia, sin scope creep, Release N/A justificado, deuda cero).
- **Veredicto:** ✅ approve (nits no bloqueantes: id JSON-RPC duplicado tras continue — inofensivo; docstring path Windows — aceptable).

## Notas

- DoD nivel Release N/A: harness sin artefacto publicable; la verificación mecánica del task (py_compile + handshake + tamper) es el gate aplicable.
- `mcp-protocol.md:7` (60 tools) y `tools.rs:25,1090`/`config.rs:11-22` (76 tools) stale → scope FIND-77/FIND-83, explícitamente NO tocados aquí.
- Pre-mortem del plan respetado: conteo parametrizado por perfil (no mágico global); binario stale cubierto con doc build-desde-fuente + tamper-test.
- Branch: `develop`. Commit prefijo `test:` + ID solo propios. Push vía vanta-lead (worker no pushea).
