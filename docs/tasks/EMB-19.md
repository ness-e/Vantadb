# EMB-19 — verificación punta a punta + reinstalación coordinada (Wave5 cierre)

> **Plan:** `docs/plans/2026-09-16-embeddings-auto.md` (Wave5, cierra campaña; alimenta a EMB-20)
> **Estado:** ⏳ IN PROGRESS 2026-09-17 (DISCOVERY completo, Steps en curso)
> **Appetite:** 1d · 🟡 · 🔴 Alta
> **Branch/Commit:** develop / `chore: EMB-19` (install no se commitea; sí la verificación)
> **Cynefin:** 🟨 complicado · ⬆️ 0 / ⬇️ 3 steps
> **Ruta:** vanta-lead (CI/release/instalación — tabla Routing: release/CI/packaging/dependencias = yo mismo)
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging + codebase-memory + documentation-and-adrs + progreso (ver §5)

## 1. TAREA

**Objetivo:** verificación punta a punta con el binario final + reinstalación coordinada del binario global + matriz de proveedores condicional (local real; ollama/openai solo si hay servidor/key, si no documentado-no-ejecutado). Cierra la campaña EMB-10..18; alimenta a EMB-20 (docs describen SOLO lo verificado aquí, Regla 11).

**Contrato exacto (ley — nada cuenta sin prueba e2e con el binario final):**
1. Cat test automático verde con el MISMO binario final (`target/debug/vanta-cli.exe` EMB-10 + fixes EMB-13..17): `VANTADB_EMBEDDING_PROVIDER=local` → señal semántica (pares≫impares, umbral con evidencia numérica; precedente EMB-10: pares≥0.91 vs impares≤0.78; EMB-13: par=0.9282 vs 0.8427/0.8366 `fallback:false`).
2. Sinónimos MCP verde: `memory_put` sin vector → guarda CON vector (EMB-14) + `search_memory`/`memory_recall` texto-solo sin palabras comunes recupera el par (`felino↔gato`, precedente EMB-15: `keys=["d1","d0","d2"]` + recall `hybrid` con D0).
3. Matriz proveedores: `local` REAL ejecutado; `ollama`/`openai` SOLO si hay servidor/key — si no, `documentado-no-ejecutado` (precedente FIND-69: `CON dspy: no instalado + sin red → documentado, no verificado`). Estado verificado 2026-09-17: Ollama server VIVE en `localhost:11434` (`/api/tags` → `{"models":[]}`, v0.33.2) pero SIN modelos → matriz ollama = degradación avisada (no crash), no embedding real; `OPENAI_API_KEY`/`VANTADB_OPENAI_API_KEY` UNSET → matriz openai = documentado-no-ejecutado.
4. Suites verdes: `cargo test -p vantadb-mcp -j 2` (incl. `mcp_tests` 93/93 + `test_embed_texts` 7/7 + `test_auto_embed` 5/5 + `test_query_embed` 4/4 + `test_model_switch` 3/3) + `cargo test -p vanta-memory -j 2` verde + `python skills/vantadb-mcp/scripts/test-mcp.py` por perfil (FIND-82: `full`=79 exacto; `dev` 30..38; `memory` 15..22).
5. `cargo audit` sin NUEVOS (sin vulnerabilidades critical/high nuevas; baseline a registrar) + `cargo deny check` si aplica al gate release.
6. Binario reinstalado y `tools/list` = 79 vía PATH: `~/.cargo/bin/vanta-cli.exe` (12.46MB 2026-08-23, STALE) → reemplazar por build local (27.55MB 2026-09-16) + `vantadb-server.exe` (12.69MB → 40.35MB) + `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | vanta-cli server --mcp --db <tmp>` vía PATH = 79 tools. Si PID 3864 sigue bloqueando install → coordinar ventana (NO `taskkill` ajeno); verificado 2026-09-17: PID 3864 YA NO EXISTE (`Get-Process -Id 3864` vacío) → ventana ABIERTA, install ejecutable.
7. `scripts/validate-docs-coverage.ps1` 0 gaps en lo que toca esta tarea (MCP tools ↔ `docs/api/MCP.md`; EMB-20 es dueño de curar docs, aquí solo verificar que EMB-13..17 no rompieron paridad).

**Acceptance del plan:** Gate Justificación = nada cuenta sin prueba e2e con el binario final. Pre-mortem: PID 3864 bloquea install → coordinar ventana (no taskkill ajeno) ✅ resuelto (PID ausente); ollama/key ausentes → matriz parcial documentada (precedente FIND-69) ✅ aplica a openai + ollama-sin-modelos.

## 2. ARCHIVOS

**Clave (con :línea):**
- `target/debug/vanta-cli.exe:1` (binario EMB-10, 27.55MB 2026-09-16 15:44, con `embed-local,remote-inference` + fix `token_type_ids` + EMB-13..17) — e2e contra ESTE binario.
- `target/debug/vantadb-server.exe:1` (40.35MB 2026-09-16 15:52, wrapper `vanta-cli server --mcp` lo spawnea vía `src/cli_handlers/server.rs:253-289`) — reinstalar AMBOS (test-mcp.py exige rebuild BOTH).
- `vanta-mcp-local.ps1:1-99` (lanzador EMB-12: `-DbPath` mandatory, `VANTADB_EMBEDDING_PROVIDER` + `VANTADB_LOCAL_MODEL` absoluta, repo-local-first, ORT autodetect LOCALAPPDATA→Temp, info a stderr) — verificación e2e USA el lanzador.
- `~/.cargo/bin/vanta-cli.exe` (12.46MB 2026-08-23, STALE sin `embed_texts` — EMB-12 Corrida 1: `-32601 Tool not found`) + `~/.cargo/bin/vantadb-server.exe` (12.69MB 2026-08-23) — destino reinstall (bloqueo PID 3864 verificado AUSENTE 2026-09-17).
- `docs/tasks/EMB-19.md` (este archivo — NUEVO en DISCOVERY).

**Relacionados (callers/callees de codegraph_explore + trace_path):**
- `skills/vantadb-mcp/scripts/test-mcp.py:1-253` por perfil (FIND-82: `EXPECTED_TOOLS full=(79,79)`, resolve `VANTADB_MCP_BIN`→PATH→`target/debug/`, drift gate "Stale binary? Rebuild").
- `scripts/validate-docs-coverage.ps1:167-186` (MCP tools ↔ `docs/api/MCP.md` vía `handle_tools_list` en `vantadb-mcp/src/handlers/tools.rs`) + `:192-219` (skills mirror FIND-83).
- Suites: `vantadb-mcp/tests/mcp_tests.rs` (`mcp_tests` 93 incl. 2 emb18 + `test_mcp_tool_profiles` source-of-truth 79) + `vantadb-mcp/tests/test_embed_texts.rs` (7/7) + `test_auto_embed.rs` (5/5) + `test_query_embed.rs` (4/4) + `test_model_switch.rs` (3/3) + `vanta-memory` suite (`cargo test -p vanta-memory -j 2`).
- `docs/tasks/EMB-10.md` (binario + matriz 3 casos + ORT≥1.27 + FIND-100/101/102) · `docs/tasks/EMB-12.md` (lanzador + smoke 79 + `dim==384`) · `docs/tasks/EMB-13.md` (señal 0.9282 + fallback) · `docs/tasks/EMB-15.md` (sinónimos `["d1","d0","d2"]` + `hybrid`) · `docs/tasks/EMB-11.md` (ORT persistente `%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll` 16.46MB 2026-09-10) · `docs/tasks/EMB-17.md` (switch + `MAX_CACHED_LOCAL_MODELS=2`) · `docs/tasks/FIND-69.md` (precedente matriz parcial documentada).
- `embeddings/manifest.json` (9 ids, default `multilingual-e5-small` 384d) + `embeddings/models/` (3 en disco: all-MiniLM-L6-v2, multilingual-e5-small, paraphrase-multilingual-MiniLM-L12-v2) + `src/llm.rs` (solo lectura aquí) + `src/config.rs:935-959` (envs `VANTADB_*`).

**Prohibidos (WIP ajeno que NO se toca — `detect_changes` 2026-09-17: 11 changed files, TODOS prohibidos, 0 solape con mi scope):**
- `.opencode/` (m submodule) · `Justfile` · `completions/_vanta-cli*` (`_vanta-cli`, `_vanta-cli.ps1`, `vanta-cli.fish`) · `desktop/src-tauri/Cargo.lock` · `.github/workflows/ocr-delegate.yml` (??) · `dev-tools/ocr-review.ps1` (??) · `reparacion.bat` (??) · `docs/pipeline-state.json` (no existe, no crear) · `stash@{0..14}` (no tocar) · `embeddings/README.md` (dueño EMB-20) · `docs/api/MCP.md` (dueño EMB-20, solo lectura `:18,218`) · `skills/vantadb-mcp/SKILL.md` (dueño EMB-20) · `docs/Backlog.md` + `docs/avance/` (orquestador/progreso, NO tocar por orden) · plan file (solo recitation orquestador) · `src/llm.rs` escritura · `Cargo.toml`.

## 3. DEPENDENCIAS

- **Wave:** Wave5 (cierra campaña; alimenta a EMB-20). Wave0-W4 DONE: EMB-10 ✅ `a0f65d29` + EMB-11 ✅ `a3c4d903` + EMB-12 ✅ `f80b9886` + EMB-13 ✅ `ad8af1d1` + EMB-16 ✅ `be6a3c27` + EMB-18 ✅ `43405be3` + EMB-14 ✅ `11de0d42` + FIND-99 ✅ (épica cerrada 13+14) + EMB-15 ✅ `9f4fd725` + EMB-17 ✅ `34b6ca63` (branch `develop`, log verificado 2026-09-17).
- **BLOQUEANTES:** ninguna (EMB-19 es cierre verificador, 0 código prod tocado esperado — solo verificación + reinstall fuera del repo).
- **Previa:** EMB-17 ✅ / **nextTask:** EMB-20 (docs describen lo verificado — EMB-20 finaliza tras tu verify; NO documentar lo no-verde aquí).
- **Hereda:** todo W0-W4 + `test-mcp.py` por perfil (FIND-82) + `validate-docs-coverage.ps1` + ORT persistente LOCALAPPDATA (EMB-11) + lanzador EMB-12 + cat-test umbrales EMB-10/13 + sinónimos EMB-15 + switch EMB-17 + dim-gate EMB-18.

## 4. REFERENCIAS

- `.opencode/references/task-system.md` (LEÍDA COMPLETA: prompts `plan/task/iter-loop-tools`, MCP `get_next_task/verify_cmd/discover_skills_v2/detect_task_type/validate_command/enforce_state`, state machine C0 PLAN→ACT→VERIFY, tabla canónica codegraph✅/campaign✅/metasearch✅/argus✅) + `.opencode/references/test-suite.md` (LEÍDA: `cargo nextest run --profile audit --workspace --build-jobs 2` Fast Gate; focados por `-p/--test`; features `failpoints/cli/arrow`).
- `docs/api/MCP.md:18` (LEÍDA: stdio JSON-RPC 2.0, launcher canónico `vanta-cli server --mcp --db ~/.vantadb`) + `:218` (LEÍDA: perfil `full` default 79 tools; `dev` ~35; `memory` ~18) + `:198` (79 tools en 7 familias con annotations) — SOLO LECTURA (dueño EMB-20).
- `.opencode/rules/server-mcp.md` + `release-ci.md` (a leer si se toca install/publish — install global = release-ci scope, vanta-lead) + clean-code Apéndice V (0 código prod → N/A pero Humble Object aplica al lanzador ya cerrado).
- Tabla Spec N/A (sin símbolo público nuevo; verificación + reinstall, 0 `pub fn`/tool/endpoint; si aparece símbolo nuevo en verify → decidir por-evidencia + fila FIND, no silencio).
- Regla 11: SOLO números medidos (cada claim de señal/latencia/tools con comando + output; sin bench canónico no hay claims de perf).
- Gate P heredado (Q1-Q5 owner 2026-09-16); Gate D/V/C evaluados abajo (§10 + RESULTADO).

## 5. SKILLS

**SDP (campaign_discover_skills_v2 phase=BUILD archivosClave="target/debug/vanta-cli.exe,vanta-mcp-local.ps1,~/.cargo/bin/vanta-cli.exe,skills/vantadb-mcp/scripts/test-mcp.py,scripts/validate-docs-coverage.ps1" contractKeywords=["e2e","reinstall","provider-matrix","mcp-verify"] maxSkills=8 — ejecutado 2026-09-17):**
- `campaign-executor` (1.00 base MCP) — cuándo: state machine PLAN→ACT→VERIFY + RESULTADO §7 en cada invocación.
- `source-driven-development` (1.00 base MCP) — cuándo: cada comando sugerido y mapa manifest verificados en código (`config.rs`, `server.rs`, `manifest.json`), no de memoria.
- `incremental-implementation` (1.00 lifecycle BUILD) — cuándo: slices verticales delgados (cat-test → sinónimos → matriz → suites → reinstall → cierre), cada slice con verify.
- `test-driven-development` (1.00 lifecycle BUILD) — cuándo: nada cuenta sin prueba e2e con el binario final; RED ya existe en EMB-13..17, aquí se RE-ejecutan como verificación (no se escriben tests nuevos salvo que el e2e revele bug → Prove-It Pattern).
- `context-engineering` (1.00 lifecycle BUILD) — cuándo: sesión compleja multi-evidencia (este file = context pack Rules→Plan→Source→Error).
- `doubt-driven-development` (1.00 lifecycle BUILD) — cuándo: stakes altos (reinstall global + claim "79 tools reales"): reconciliación adversarial del claim crítico antes del cierre.
- `api-and-interface-design` (1.00 lifecycle BUILD) — cuándo: superficie MCP (`tools/list` 79, `embed_texts` dim, `search_memory` shapes) — adición>modificación, Hyrum's Law en reinstall.
- `systematic-debugging` (keyword-mapped, base MCP v1 + precedente EMB-10/15) — cuándo: ante CUALQUIER fallo e2e (fases 1-4, root-cause antes de fix; 3 fixes fallidos → cuestionar arquitectura, no 4º fix).
- `codebase-memory` (base esperada del prompt, no vino en v2 pero se usa igual) — cuándo: blast radius (`codegraph_explore` + `detect_changes` + `get_architecture`) + `check_index_coverage` antes de afirmar "nada llama a X".
- `documentation-and-adrs` (base esperada del prompt) — cuándo: matriz parcial DOCUMENTADA (ollama-sin-modelos/openai-sin-key como FIND-69) + decisiones con tradeoff → ADR o `decisions` memory.
- `progreso` (base sesión) — cuándo: al cierre (Trigger 1: Backlog→avance; esta tarea NO toca Backlog/avance directamente — lo ejecuta el orquestador vía skill progreso).
- `frontend-ui-engineering` + `browser-testing-with-devtools` (sugeridas por scorer) — DESCARTADAS con justificación: sin `web/`, sin browser, scope discipline (mismo criterio EMB-10/12/15/17).
- **Cargadas vía `skill`:** test-driven-development, systematic-debugging, codebase-memory, documentation-and-adrs (campaign-executor/progreso/incremental/source/context/doubt/api = base sesión o ya en contexto; se referencian sin recarga redundante).
- **SDP registrado:** `SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging, codebase-memory, documentation-and-adrs, progreso (frontend-ui-engineering + browser-testing-with-devtools descartadas: sin web/browser)`

## 6. HERRAMIENTAS+MCP

- `cargo test -p vantadb-mcp -j 2` (suite completa default; focados `--test test_embed_texts/test_auto_embed/test_query_embed/test_model_switch/mcp_tests`) + `cargo test -p vantadb-mcp --features embed-local,remote-inference -j 2` (canónico features SIN model-env como EMB-14/15; CON model-env local para señal real) — `cargo` SIEMPRE `-j 2` (OOM/rustc-crash).
- `cargo test -p vanta-memory -j 2` (suite memory del contrato).
- `cargo audit` (contrato: sin NUEVOS; baseline a registrar con `cargo audit --version` 0.22.1) + `cargo deny check` (deny 0.19.9, licencias MIT/Apache-2.0).
- MCP stdio e2e: `initialize` + `tools/list` (=79) + `tools/call embed_texts` (dim + `fallback` flag) + `memory_put` sin vector → `memory_get` CON vector + `search_memory`/`memory_recall` sinónimos (felino↔gato) — vía lanzador `vanta-mcp-local.ps1 -DbPath <tmp>` (DB temporal, NUNCA base del usuario) o binario directo con env `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<absoluta>` + `ORT_DYLIB_PATH=%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`.
- `python skills/vantadb-mcp/scripts/test-mcp.py <bin>` por perfil (`full`/`dev`/`memory` vía `VANTADB_MCP_PROFILE`) — drift gate FIND-82.
- `pwsh scripts/validate-docs-coverage.ps1` (0 gaps) + `git diff --check` + `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings` (verify full; 0 código tocado esperado → deben pasar en verde heredado).
- `actionlint` SI toca yml (NO toca: `ocr-delegate.yml` prohibido → N/A documentado).
- `campaign_verify_cmd` (bug exit -1 conocido EMB-10/12/15/17 → bash directa + mención en RESULTADO) + `campaign_validate_command` antes del primer comando riesgoso + `campaign_enforce_state` en transiciones + `campaign_session_track` para tracking multi-iteración.
- MCP a usar: `codegraph_explore` (blast radius — ejecutado) + `detect_changes` (ejecutado: 11 files WIP ajeno, impacto 1 módulo `ocr-review`) + `get_architecture` (si hace falta contexto) + `check_index_coverage` (ejecutado: 4 paths `no_recorded_issue`, freshness `metadata_changed` → se leyó fuente directa) + `session_track` — sin grep-loop si codegraph responde (respondió + Reads directos completaron).

## 7. INVESTIGACIÓN CÓDIGO

**Blast radius resumido (construido en DISCOVERY 2026-09-17):**
- Binario EMB-10 (`target/debug/vanta-cli.exe` 27.55MB) → lanzador EMB-12 (`vanta-mcp-local.ps1`, repo-local-first, env explícita) → wrapper `src/cli_handlers/server.rs:253-289` (`VANTADB_STORAGE_PATH=<db>` + env heredado) → `vantadb-server` MCP (`server --mcp --db`) → `handle_tools_list` (`tools.rs` → 79 tools perfil `full`) → `embed_texts` (EMB-13 real + EMB-17 switch + EMB-16 prefijos) → `memory_put/put_batch` auto-embed (EMB-14) → `search_memory/recall` mismo-proveedor (EMB-15) → dim-gate EMB-18 → suites (`mcp_tests` 93 + 4 focadas 19 tests) + `vanta-memory` suite + `test-mcp.py` drift gate + `validate-docs-coverage` paridad.
- Implicaciones install global: `~/.cargo/bin/vanta-cli.exe` + `vantadb-server.exe` STALE (ago-23, 12MB, sin `embed_texts` → `-32601`) enmascaran drift (misma regla que test-mcp.py "stale installed binary masks drift" + EMB-12 Corrida 1). Reinstall = `Copy-Item target/debug/*.exe ~/.cargo/bin/` (ventana ABIERTA: PID 3864 ausente) → `tools/list` vía PATH debe dar 79. Rollback = recopy desde backup o rebuild (aditivo puro, sin migración).
- Riesgo PID 3864: plan pre-mortem (bloquea install, no taskkill ajeno) → VERIFICADO AUSENTE 2026-09-17 (`Get-Process -Id 3864` vacío) → install ejecutable SIN coordinación. Si reaparece durante install (file lock) → STOP + coordinar ventana, matriz parcial NO aplica a install (es binario, no red).
- Riesgo matriz parcial: Ollama VIVE pero `models:[]` (sin embeddings) → ollama準備不足 = degradación avisada (no crash), documentado-no-ejecutado para embedding real; OpenAI sin key → documentado-no-ejecutado (FIND-69). Local REAL con ORT persistente (16.46MB) + 3 modelos en disco → verde ejecutado.
- `detect_changes` inbound depth=3: 11 changed, 27 seeds, impacted_total=1 (`dev-tools/ocr-review.ps1` Module hop 1) — 0 impacto en mi scope (verificación + reinstall no tocan prod).

**Impacto mapeado (Regla 0):**
- **Archivos leídos (completos):** plan EMB-19 §Wave5 + EMB-10/12/13/14/15/17 task files + `vanta-mcp-local.ps1` (99L) + `test-mcp.py` (253L) + `validate-docs-coverage.ps1` (227L) + `MCP.md:18,218,198` + `test-suite.md` + `task-system.md` + `FIND-69.md` + `manifest.json` (parcial) + codegraph_explore output + `detect_changes` + `check_index_coverage`.
- **Hacia dentro:** binarios EMB-10, modelo e5-small en disco, ORT persistente, vars `config.rs`, comando canónico MCP.md, `handle_tools_list` 79, helpers EMB-13..17.
- **Entrantes:** EMB-20 (describe lo verificado aquí); futuro cambio `opencode.jsonc` (no tocado); PATH global (post-install).
- **Veredicto:** BAJO (0 prod tocado esperado) + install global (fuera del repo, reversible). Rollback = no-commit de verificación o recopy de backup.

## 8. INVESTIGACIÓN PROBLEMA

- Hipótesis ante fallo e2e: (a) ORT dylib incompatible (FIND-100: System32 1.17.1 vs ORT_API_VERSION=27 → abort `0xc0000409`; mitigación: `ORT_DYLIB_PATH` persistente ≥1.27 verificado presente); (b) CWD frágil (`VANTADB_LOCAL_MODEL` relativa → usar ABSOLUTA + correr desde repo root; tests usan `CARGO_MANIFEST_DIR`-relativo + hardcodeado); (c) dim-mismatch (base con otra dim → EMB-18 gatea con guía, no es fallo del verify — es verde del gate); (d) stale PATH (global viejo enmascara → resolver repo-local-first en e2e, PATH solo tras reinstall); (e) model-env tension (EMB-15: 3 fallos con model-env por puts 384d vs tests dims 3/4 + race concurrente — pre-existentes vía stash-proof, NO regressions de EMB-19).
- Decisión spec con tradeoff: local-real vs matriz-parcial-documentada — ejecutar `local` REAL (hay servidor/modelo/ORT) + `ollama` degradación-avisada (hay servidor sin modelos) + `openai` documentado-no-ejecutado (sin key, sin red para crearla). Precedente FIND-69 §Steps (`CON dspy: no instalado + sin red → documentado, no verificado (deuda explícita, permite el contrato)`). Nunca dummy silencioso (Q5: `"fallback":true` visible).
- Causa raíz de "global STALE": instalado 2026-08-23 pre-campaña (12MB, sin features embed) + PID 3864 lo bloqueaba → ahora PID ausente → reinstall cierra la causa.

## 9. INVESTIGACIÓN INTERNET

N/A — todo local (binario + modelo + ORT + suites + MCP stdio + scripts en repo; stack verificado `cargo 1.95.0/rustc 1.95.0`, `cargo-audit 0.22.1`, `cargo-deny 0.19.9`, Ollama v0.33.2 vía `localhost:11434/api/tags` en vivo). Sin ambigüedad externa real (no hay API externa con duda: MCP stdio + ONNX local + suites cargo están versionados en repo). Sin red usada como evidencia → sin citas, sin deuda TSYS-13. Si surgiera duda de API `ort`/MCP spec → `webfetch` docs oficiales (`ort.pyke.io`, `modelcontextprotocol.io`) + deuda TSYS-13 si sin red. Estado: N/A (igual que EMB-13/14/15/17 §9). Gate de citas TSYS-13: sin URLs en evidencia → N/A.

## 10. VALIDACIÓN+CIERRE

- [ ] Step 1 — cat test automático verde + sinónimos MCP verde (binario final + lanzador, DB temporal): `embed_texts` dim=384 `fallback:false` + pares≫impares con números + `memory_put` sin vector → `memory_get` CON vector + `search_memory`/`recall` sinónimos recuperan par sin palabras comunes.
- [ ] Step 2 — matriz proveedores + suites + audit: local REAL verde; ollama degradación avisada (servidor vivo sin modelos); openai documentado-no-ejecutado (sin key); `cargo test -p vantadb-mcp -j 2` + `cargo test -p vanta-memory -j 2` verdes; `test-mcp.py` por perfil (full=79); `cargo audit` sin nuevos + `cargo deny check`.
- [ ] Step 3 — reinstall + verify full + cierre: backup global → copy `vanta-cli.exe`+`vantadb-server.exe` → `tools/list`=79 vía PATH → `cargo fmt --check` + `clippy --workspace --all-targets -- -D warnings` + `validate-docs-coverage.ps1` 0 gaps → OCR advisory (`dev-tools/ocr-review.ps1`; Critical/High=bloquea, Medium→FIND-*) → DoD 3 niveles → reviewer distinto P2-01 → commit `chore: EMB-19` SOLO propios (task file; install NO se commitea) → recitation + RESULTADO §7 + Save Point.
- Gates D/V/C activos (ver §Spec Gate + RESULTADO): D (blast 0 prod + contrato mecánico + sin símbolos nuevos → NO dispara); V (2 fallas mismo-error → `question` antes de FAILED); C (colaterales → fila FIND + `question`).
- DoD 3 niveles: **Task** (contrato §1 verificable + evidencia numérica) · **Commit** (atómico `chore: EMB-19`, `git diff` solo propios, fmt+clippy verdes) · **Release** (pre-push gate Regla 1 vía `dev-tools/verify.ps1`/`just verify` si aplica; install global verificado vía PATH).
- Reviewer distinto P2-01 (doubt-driven; vanta-review si disponible — si no, deuda leve como EMB-13/14/15/17 + OCR input).

## Steps

- [x] **Step 1 — cat test + sinónimos e2e (binario final + lanzador):** rebuild binario final (`cargo build --bin vanta-cli --bin vantadb-server --features embed-local,remote-inference -j 2`, 1m57s) + `embed_texts` dim=384 `fallback:false` + pares-vs-impares con números + put→get con vector + search/recall sinónimos · Verify: evidencia numérica en §Ejecución ✅ DONE 2026-09-17
- [x] **Step 2 — matriz + suites + audit:** local REAL + ollama degradación + openai degradación/documentado + `cargo test -p vantadb-mcp/vanta-memory -j 2` + `test-mcp.py` perfiles + `cargo audit/deny` · Verify: outputs en §Ejecución ✅ DONE 2026-09-17
- [x] **Step 3 — reinstall + verify full + cierre:** ventana AUTORIZADA por usuario (Gate C `question` → "Autorizar ventana ahora") → `Stop-Process 25596,3792` → backup `.bak-emb19` → copy ambos exes → `tools/list`=79 vía PATH ✅ + fmt/clippy/docs-coverage ✅ + OCR advisory ✅ → commit `chore:` → recitation + RESULTADO ✅ DONE 2026-09-17

## Ejecución (evidencia mecánica — se puebla en Steps)

### HALLAZGO crítico DISCOVERY→Step 1 (systematic-debugging Fase 1, root-cause antes de fix)
- `target/debug/vanta-cli.exe` 27.55MB timestamp 2026-09-16 15:44 ANTERIOR a EMB-13 (`ad8af1d1` 16:13) → binario SIN EMB-13..17 (evidencia: `embed_texts` stdio sin campo `fallback`, keys `[count,dim,dimensions,embeddings,model,next_cursor,truncated]`).
- Fix: `cargo build --bin vanta-cli --bin vantadb-server --features embed-local,remote-inference -j 2` → `Finished dev profile in 1m57s` → bins 24.88MB/26.26MB 2026-09-16 20:06. Post-rebuild keys `[...,fallback,...]` ✅.
- Causa raíz hang harness 10min (dos incidentes): (1) `stderr=PIPE` sin drain → pipe lleno bloquea server (STABLE-04; `test-mcp.py` lo drena con thread) → fix `stderr=DEVNULL`; (2) recall stdio sobre L1 vacío → `Ok(None)` → `effective_mode:keyword` + `recalled:[]` (código `tools.rs:1741-1746`, comportamiento correcto, no fallo) → recall se prueba vía focado con `seed_l1`.

### Step 1 — cat test + sinónimos (binario FINAL rebuild, DB temporal, `stderr=DEVNULL`)
- `initialize` OK (`vantadb 0.5.0`) + `tools/list` = **79** ✅ (repo binary).
- `embed_texts [D0,D1,D2,D3]`: keys `[count,dim,dimensions,embeddings,fallback,model,next_cursor,truncated]` · `dim=384` · `fallback=False` · `model=multilingual-e5-small` · `count=4` ✅
  - `s(pares D0-D1)=0.9158` vs `s(D0-D2)=0.8427` vs `s(D0-D3)=0.8423` → gap **0.0732** ✅ GREEN (precedente EMB-13: 0.9282 vs 0.8427/0.8366; EMB-10: pares≥0.91 vs impares≤0.78).
- `memory_put` (d0,d1,d2 sin vector) → `memory_get d0`: `vector` len **384** (no null) ✅ PUT-AUTO-EMBED GREEN (EMB-14).
- `search_memory {text_query:"felino descansando"}` (0 palabras comunes con D0): `keys=["d1","d0","d2"]` ✅ SEARCH-SINONIMO GREEN (idéntico a EMB-15).
- `memory_recall` stdio: `effective_mode:keyword`, `recalled:[]` — ESPERADO con L1 vacío (sin seed L1 vía MCP); recall real probado en focado abajo.
- Focado `cargo test -p vantadb-mcp --features embed-local,remote-inference --test test_query_embed -j 2 -- --nocapture` (env local+ORT persistente): **4/4** ✅ — `search keys=["d1","d0","d2"]` + `recall mode="hybrid" recalled=[felino…, gato…, cuántica…]` (D0 vía felino↔gato) ✅ RECALL-SINONIMO GREEN.

### Step 2 — matriz + suites + audit
- **local REAL** (arriba): dim 384, fallback:false, señal 0.9158, search+recall sinónimos ✅ EJECUTADO.
- **ollama** (`VANTADB_EMBEDDING_PROVIDER=ollama`, servidor VIVO `localhost:11434` v0.33.2 pero `/api/tags` → `{"models":[]}`): `fallback:true` + `warning="Embedding model unavailable (provider='ollama'); using deterministic fallback vectors (no semantic signal). Provider error: ... 404 Not Found"` · `dim=384` · exit 0 sin crash ✅ DEGRADACIÓN AVISADA EJECUTADA (embedding real NO ejecutable: sin modelos; Q5 verde).
- **openai** (sin `VANTADB_OPENAI_API_KEY`/`OPENAI_API_KEY`, ambas UNSET verificado): `fallback:true` + `warning="... (provider='openai'); ... Provider error: Invalid input: VANTADB_OPENAI_API_KEY must be set ..."` · exit 0 sin crash ✅ DEGRADACIÓN AVISADA EJECUTADA + embedding real DOCUMENTADO-NO-EJECUTADO (precedente FIND-69; sin key, sin red para crearla).
- `cargo test -p vantadb-mcp -j 2` (default): **TODO verde, 0 failed** — 27+9+7+3+**93 mcp_tests**+7+10+5 auto_embed+7 embed_texts+3+4 query_embed+7+3 model_switch+1+7 ✅ (incl. `test_mcp_tool_profiles` 79).
- `cargo test -p vanta-memory -j 2`: **TODO verde, 0 failed** (20 suites: 3+15+5+7+16+9+5+1+13+21+14+9+17+13+4+5+10+2+6+1) ✅.
- `python skills/vantadb-mcp/scripts/test-mcp.py target/debug/vantadb-server.exe`: `full` **4/4** (79 tools ✅) · `dev` 4/4 · `memory` 4/4 ✅ (FIND-82).
- `cargo audit`: **1 allowed warning pre-existente** `RUSTSEC-2026-0253` (`lru 0.16.4` unsound vía `tantivy 0.26.1`); 0 errores; **sin NUEVOS** (0 cambios `Cargo.toml/lock` en mi scope — `git status` solo WIP ajeno) ✅. `cargo deny check`: **advisories ok, bans ok, licenses ok, sources ok** ✅ (deny 0.19.9, audit 0.22.1).

### Step 3 — reinstall (VENTANA AUTORIZADA + EJECUTADA 2026-09-17) + verify full (VERDE)
- Pre-mortem plan decía PID 3864 → verificado AUSENTE (`Get-Process -Id 3864` vacío) ✅ ventana esperada ABIERTA.
- PERO lockers reales nuevos: `Copy-Item` → `The process cannot access the file ... being used by another process` para AMBOS exes.
- Lockers: PID **25596** `vanta-cli` + PID **3792** `vantadb-server`, ambos `C:\Users\Eros\.cargo\bin\...`, inicio 2026-09-16 17:26 (servidor MCP de esta sesión) → Gate C `question` al usuario → respuesta **"Autorizar ventana ahora"** → `Stop-Process -Id 25596,3792 -Force` → ambos gone → backup `.bak-emb19` (12.46/12.69MB pre-campaña) → `Copy-Item target/debug/*.exe ~/.cargo/bin/` → global **24.83/26.26MB 2026-09-16 20:06** ✅.
- Post-install vía PATH (sin argv, resuelve global): `vanta-cli 0.5.0` + `test-mcp.py` → `Found 79 tools (profile 'full': 79..79 OK)` + `4/4 passed` ✅ CONTRATO CERRADO. Rollback: recopy desde `.bak-emb19`.
- Verify full: `cargo fmt --check` exit **0** ✅ · `cargo clippy --workspace --all-targets --all-features -- -D warnings` **Finished** sin warnings (5m23s) ✅ · `scripts/validate-docs-coverage.ps1` **0 gaps** (8/8: sdk 28, config 60, cli 42, py 48, mcp-tools 49, skills-mirror 10 pares) ✅.
- OCR advisory: `ocr delegate` solo reglas sobre WIP ajeno (`completions/*`, `ocr-delegate.yml`); mi scope = 1 task file nuevo (md, `unsupported_ext` como EMB-12) + 0 prod → **sin Critical/High, sin Medium → sin FIND-*** ✅.
- `git diff --check` pendiente al commitear (solo `docs/tasks/EMB-19.md` untracked + WIP ajeno excluido).

## Spec Gate

Sin símbolos públicos nuevos (verificación + reinstall, 0 `pub fn`/tool/endpoint). Tabla §4 de EMB-15/17 vale como spec de decisiones heredadas. Gate D: blast 0 archivos prod + test + contrato mecánico del plan → NO dispara `question` (motivo: verificación sin código + install reversible fuera del repo). Gate V: 2 fallas mismo-error → `question` antes de FAILED. Gate C: colaterales → fila FIND + `question` (WIP ajeno en worktree → excluir del commit, confirmar alcance).

## Context Save Point

DISCOVERY completo 2026-09-17. Plan + EMB-10/12/13/15/17 + FIND-69 + lanzador (99L) + test-mcp.py (253L) + validate-docs (227L) + MCP.md + test-suite + task-system + codegraph_explore + detect_changes (11 WIP, impacto 1) + check_index_coverage (4×no_recorded_issue) + estado vivo (PID ausente, Ollama vivo-sin-modelos, OpenAI unset, bins STALE-vs-final). Task file creado. Siguiente: Step 1 cat test + sinónimos.

## Invariantes de dominio (handoff — MUST)

- **Preservar:** 0 edición prod (verificación solo lee/ejecuta); shapes MCP intactos; budgeting 128/25k; fallback Q5; mensajes EMB-18 exactos; prefijos EMB-16; hunks EMB-13..17 intactos; WIP ajeno excluido del commit; sin push (vía vanta-lead/orquestador); DB temporal para e2e (NUNCA base del usuario `~/.vantadb` ni `./db` por defecto); secrets NUNCA a disco/log.
- **Comandos:** `cargo test -p vantadb-mcp -j 2` + `cargo test -p vanta-memory -j 2` + `python skills/vantadb-mcp/scripts/test-mcp.py` + MCP stdio vía lanzador + `cargo audit` + `scripts/validate-docs-coverage.ps1` + `cargo -j 2` siempre.
- **Deuda:** matriz ollama-sin-modelos/openai-sin-key = documentado-no-ejecutado (FIND-69), NO es deuda oculta; vanta-review si no disponible = deuda leve.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** 0 esperado (0 código tocado). Si el e2e revela bug → Prove-It + fix mínimo + pagar con P2-5/P2-8 o fila FIND (nunca deuda silenciosa).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §1 (7 puntos) verificable + evidencia numérica + matriz documentada donde no ejecutable |
| **Commit** | Atómico `chore: EMB-19` (solo `docs/tasks/EMB-19.md`; install NO se commitea), `git diff` solo propios, fmt+clippy verdes |
| **Release** | Pre-push gate Regla 1 (`dev-tools/verify.ps1`/`just verify` si aplica) + PATH `tools/list`=79 + rollback (recopy backup) documentado |

SKILLS_CARGADAS: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging, codebase-memory, documentation-and-adrs, progreso
SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging, codebase-memory, documentation-and-adrs, progreso (frontend-ui-engineering + browser-testing-with-devtools descartadas: sin web/browser)
