# FIND-98 — reinstalar `vanta-cli` (parity 79→87)

> **Estado:** 🟡 STOP — lock Windows persistente (2/2 intentos), re-DEFER propuesto al orquestador
> **Fecha ejecución:** 2026-09-18 · **Ruta:** vanta-worker · **Branch:** develop
> **Plan:** `docs/plans/2026-09-18-cierre-mvp.md` (Wave0, primera en secuencia)
> **SDP:** `campaign-executor` · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `systematic-debugging` · `shipping-and-launch` (8 total; `frontend-ui-engineering`/`api-and-interface-design` del scoring descartadas por irrelevantes al slice — distribución binaria, sin UI ni API nueva)

## 1. TAREA

**Objetivo:** el binario que usa el usuario (`C:/Users/Eros/.cargo/bin/vanta-cli.exe`) miente 8 tools (sirve 79 vs 87 de la fuente); cero fricción exige parity. Reinstalar desde fuente.

**Contrato exacto (plan §Task 1):** binario instalado sirve **87 tools** (`tools/list`) + smoke `initialize` OK + `cargo build` solo si el binario existente está bloqueado y rebuild es la vía; si el lock persiste → STOP con motivo (no forzar), re-DEFER válido.

**Acceptance criteria + veredicto:**

| AC | Resultado |
|----|-----------|
| (a) instalado == 87 (`tools/list`) | ❌ NO CUMPLE — instalado sirve **79** (medido en vivo, drift gate `test-mcp.py` 4/5). STOP por lock, sin cambios al binario |
| (b) smoke `initialize` OK | ✅ en ambos: instalado (`vantadb 0.5.0`, protocol 2026-06-18) e fuente (idem) — el binario viejo responde, solo está stale |
| (c) lock → STOP sin forzar | ✅ — 2/2 intentos `LOCKED`, PIDs vivos 28612/6460, cero kills, cero writes al binario |

**Stop ejecutado (plan §Task 1):** lock persistente tras 2 intentos → re-DEFER con evidencia (este file). Matar procesos MCP vivos es decisión del owner.

## 2. ARCHIVOS

**Clave:**
- `C:/Users/Eros/.cargo/bin/vanta-cli.exe` (fuera del repo) — 24.831.488 bytes, 2026-09-16 8:06, sirve 79
- `C:/Users/Eros/.cargo/bin/vantadb-server.exe` (fuera del repo) — 26.263.552 bytes, 2026-09-16 8:06, sirve 79 (hermano obligatorio — ver §8)
- `target/debug/vanta-cli.exe` — 16.449.536 bytes, 2026-09-17 8:27, sirve **87** (fuente lista, sin rebuild)
- `target/debug/vantadb-server.exe` — 22.724.608 bytes, 2026-09-17 7:49, sirve **87**

**Relacionados (lectura):**
- `src/bin/vanta-cli.rs:1-28` (entry thin + allocators `jemalloc`/`mimalloc` por plataforma — release-ci Regla 1)
- `src/cli_handlers/server.rs:253-328` (wrapper `server --mcp`: PATH-primero vía `Command::new("vantadb-server.exe")` → en Windows resuelve dir-del-padre antes que PATH; fallback sibling `current_exe.set_file_name` — `server.rs:294-296`)
- `Cargo.toml:2-5` (workspace `vantadb`, versión workspace) · ambos binarios reportan `vanta-cli 0.5.0` (versión OK, solo drift de tools)
- `skills/vantadb-mcp/scripts/test-mcp.py:1-60` (smoke canónico: `initialize`→`tools/list`→`resources/list`→`prompts/list`→recall, drift gate full = 87 exacto, DB temporal — no toca datos vivos)

**Prohibidos (respetados — `git status` verificado §6, staging selectivo solo este file):** forzar unlock matando procesos ajenos · `reparacion.bat` · `.opencode` · `Justfile` (inexistente, no tocado) · `ocr-*` · `completions/*` · `desktop/src-tauri/Cargo.lock` · `stash@{0}` GOV-C4 · `docs/Backlog.md` (fila 98 NO tocada — re-DEFER vía orquestador) · plan file (sin recitation propia — el handoff es el RESULTADO) · `C:/Users/Eros/.vantadb*` (smokes solo en DB temporal del script) · files de FIND-110-spec/113-spec (paralelas de wave, disjuntas).

**Impacto mapeado (Regla 0):** único write = creación de este file (nuevo, cero referencias entrantes). Cero ediciones a archivos existentes. Cero código del repo modificado → blast radius nulo en el repo; el blast radius real (binarios fuera del repo) se deja intacto por el STOP.

## 3. DEPENDENCIAS

- **Wave0 primera en secuencia** (plan §Grafo): FIND-110-spec + FIND-113-spec después por rate-limit; archivos disjuntos (fuera-del-repo · docs/tasks/110 · docs/tasks/113) — esta STOP no las bloquea.
- **Sin bloqueantes** para el resto del plan (el binario stale no impide S1/S2/117/119/SHOW-05; solo el dogfooding vía agente sigue en 79 hasta el reinstall).
- **NextTask (orquestador):** FIND-110-spec.

## 4. REFERENCIAS

- **Rules** `.opencode/rules/release-ci.md` — lectura completa (42L). Aplica: Regla 1 (binarios de producción con `custom-allocator`/`jemalloc` — el reinstall futuro debería ser `cargo install --path` release, no copia de debug, salvo decisión owner; la copia debug es funcional pero cambia perfil perf/tamaño) y Regla 4 (versión workspace 0.5.0 — verificada igual en ambos binarios).
- **Refs:** `definition-of-done.md` (standing checklist + DoD v1 — ver §10) · `dev-tools.md` (test-mcp.py como smoke canónico, `cargo build --bin ... -j 2` si hiciera falta rebuild).
- **Commands:** `pipeline.md` (este run) · `audit.md` (gates VERIFY — full suite N/A sin código, §10).
- **SPEC.md raíz §Alcance cierre-mvp (2026-09-18):** Ask-first `~/.cargo/bin` → APROBADO para FIND-98 vía Gate P; lock persistente → STOP sin forzar. Success Criteria §4 (binario instalado == fuente) queda pendiente por este STOP.
- **Tabla Spec:** N/A — distribución, sin símbolos públicos nuevos (Gate D no disparado, §10).

## 5. SKILLS (SDP Paso 0b real)

`campaign_discover_skills_v2` (phase BUILD, 8 keywords contrato) devolvió 8 con scoring 1.00; ajuste ponytail: `frontend-ui-engineering` + `api-and-interface-design` descartadas (scoring ruidoso — slice de distribución binaria, sin UI ni API nueva); añadidas `systematic-debugging` (lock/repro — keyword-mapped por el propio discover) · `shipping-and-launch` (distribución — sugerida del plan) · `progreso` (cierre). Cargadas vía `skill`: systematic-debugging, incremental-implementation, shipping-and-launch, progreso (+ base sesión campaign-executor/ponytail-full/brainstorming/writing-plans/planning-and-task-breakdown según plan §Notas).
**SKILLS_CARGADAS:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, systematic-debugging, shipping-and-launch (+ progreso en cierre).

## 6. HERRAMIENTAS+MCP

| Comando | Resultado |
|---------|-----------|
| `python skills/vantadb-mcp/scripts/test-mcp.py C:/Users/Eros/.cargo/bin/vanta-cli.exe` (DB temporal) | 4/5 — `initialize` ✅ (`vantadb 0.5.0`), `tools/list` **79** (drift gate dispara: espera 87) |
| `... test-mcp.py C:/Users/Eros/.cargo/bin/vantadb-server.exe` | 79 directo — el drift vive en el server, el wrapper solo lo spawnea |
| `... test-mcp.py target/debug/vanta-cli.exe` | **5/5 — 87 tools** ✅ (fuente lista) |
| `... test-mcp.py target/debug/vantadb-server.exe` | **87 directo** ✅ |
| Exclusive-open (intento 1 + intento 2, ambos bins) | `LOCKED` 2/2 — PIDs 28612 (`vanta-cli`) + 6460 (`vantadb-server`), iniciados 2026-09-17 6:05 (MCP vivos; casi seguro backends de esta sesión — matarlos = suicidar mis propias tools) |
| `cargo build -p vantadb --bin vanta-cli -j 2` | NO ejecutado — innecesario: el par debug ya sirve 87 (rebuild solo si el owner prefiere release por Regla 1) |
| `campaign_verify_cmd("git diff --check")` | bug exit -1 (conocido, plan §Riesgos) → fallback bash directa ✅ OK + mención (esta fila + RESULTADO) |
| `git diff --check` + `git status --short` (bash) | OK; WIP ajeno intacto (`.opencode`, `SPEC.md`, `completions/*`, `docs/Backlog.md`, `reparacion.bat`, plan untracked) |

Cargo siempre `-j 2` (no se llegó a compilar). Internet N/A (cero ambigüedad externa).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY)

- **Entry:** `src/bin/vanta-cli.rs:1-28` thin (parse clap → `cli_handlers`); allocators por plataforma (Regla 1).
- **Wrapper:** `src/cli_handlers/server.rs:253-328` — `server --mcp` NO sirve tools in-process: spawnea `vantadb-server --mcp` con stdio heredado. Orden: `Command::new("vantadb-server.exe")` (en Windows resuelve primero el dir del padre → sibling) y solo si `NotFound`, fallback al sibling explícito (`:294-296`). Consecuencia: el par instalado se auto-consume (79+79) y el par debug se auto-consume (87+87) — verificado en vivo, sin asumir.
- **Versión:** workspace 0.5.0; `--version` idéntico en ambos → el drift es solo de superficie MCP, no de versión.
- **Lock:** handle exclusivo denegado en ambos instalados; dueños PIDs vivos con `StartTime` 2026-09-17 (el PID 3864 del plan murió; el lock lo heredaron los servidores actuales).

## 8. INVESTIGACIÓN PROBLEMA

- **Causa del drift (verificada, no asumida):** instalado 2026-09-16 8:06 precede a los 4 commits que añaden tools — `472526dc` S1 scene (81, 09-17 11:40) · `325b1237` S2 dream (84) · `74c225a0` S3 dream (86) · `234f0627` FIND-111 skill_extract (**87**, 09-17 19:51). Gap 8 = scene_write/edit + dream×5 + skill_extract (coincide con el plan).
- **Vía de reinstall (decisión lista para el owner):** copia directa del PAR debug (`vanta-cli.exe` + `vantadb-server.exe` — ambos obligatorios por §7; copiar solo el CLI deja 79) en cuanto los PIDs liberen; o `cargo install --path` release si se exige Regla 1 (build frío con `-j 2`, sin matar el build). Rebuild hoy innecesario (debug ya en 87).
- **Por qué STOP y no workaround:** los PIDs son MCP vivos (probablemente los que sirven mis propias tools `vantadb_*` de esta sesión); matarlos viola la prohibición explícita + SPEC Boundaries ("matar procesos ajenos" en Never) y puede tumbar la sesión. Reintentar cuando mueran naturalmente o con decisión explícita del owner de bajar el MCP.

## 9. INVESTIGACIÓN INTERNET

N/A (cero ambigüedad de APIs externas; todo verificado contra fuente local + git log).

## 10. VALIDACIÓN + CIERRE

- **Verify contrato:** (a) ❌ 79≠87 por lock — STOP válido por contrato; (b) ✅ initialize en ambos; (c) ✅ STOP sin forzar, evidencia arriba. `VERIFY_CONTRATO: falla` (binario intacto a propósito).
- **OCR delegation:** N/A-justificado — cero código modificado salvo este task file (docs); nada que delegar a review de código.
- **DoD 3 niveles:**
  - N1 contrato: (b)+(c) ✅, (a) bloqueado con evidencia → STOP contractual, no FAILED técnico.
  - N2 standing (adaptado, sin código): correctness = mediciones en vivo con DB temporal (no auto-reporte) ✅ · quality = scope quirúrgico (1 file nuevo, staging selectivo) ✅ · integration = wrapper/server entendidos como par ✅ · docs = este file ✅ · ship-readiness = rollback trivial (re-DEFER: no se tocó nada; reinstall futuro = copia reversible) ✅.
  - N3 proyecto: compila/tests/linters N/A (sin código — correr `audit --workspace` no añade señal y quema budget); `git diff --check` ✅; CHANGELOG no (sin cambio user-visible shippado); avance/Backlog solo vía orquestador (prohibido aquí).
- **P2-01:** pendiente orquestador (`vanta-review`, batch al cierre por área según plan — sin código que revisar, solo este file).
- **Gates:** P: no (heredado Gate P 2026-09-18, Ask-first aprobado) · D: no (distribución, 0 archivos repo tocados, 0 símbolos públicos — motivo: sin-código) · V: no (cero fallos verify; smokes de investigación pasaron como medición, no como gate) · C: disparado (colateral §7-par-obligatorio incorporado a ESTE file, no requiere fila FIND nueva; staging confirmado selectivo).
- **Commit:** `docs: FIND-98 — ...` solo este file, NO PUSH (ejecuta el lead).

```
RESULTADO: 🟡 INCOMPLETO
STEPS_OK: 2/3 (discovery+evidencia ✅ · reinstall ⬜ bloqueado por lock · cierre-docs ✅)
PROXIMO_STEP: reintentar copia del PAR debug (87 verificado) cuando PIDs 28612/6460 liberen, o decisión owner (bajar MCP / cargo install release) — luego smoke 87 + cerrar
COMMIT_HASH: (al commitear este file)
ARCHIVOS: docs/tasks/FIND-98.md
VERIFY_CONTRATO: falla (instalado 79≠87 — STOP contractual, binario intacto)
BLOQUEO: lock Windows persistente 2/2 en ambos instalados (PIDs vivos 28612/6460) — matar prohibido → re-DEFER válido
GATES_EVALUADOS: P:no|heredado-Gate-P-2026-09-18 D:no|distribucion-sin-codigo V:no|cero-fallos-verify C:disparado|par-obligatorio-en-file
SKILLS_CARGADAS: campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, systematic-debugging, shipping-and-launch (+ progreso)
```
