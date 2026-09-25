# EMB-12 — script lanzador (env + arranque MCP)

> **Plan:** `docs/dev/plans/2026-09-16-embeddings-auto.md` (Wave1, instalador)
> **Estado:** ⏳ IN PROGRESS (DISCOVERY completo 2026-09-16)
> **Appetite:** 4h · 🟢 · 🟠
> **Branch/Commit:** develop / `feat: EMB-12`
> **Cynefin:** 🟦 obvio · ⬆️ 0 / ⬇️ 2 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, security-and-hardening (api-and-interface-design y frontend-ui-engineering sugeridas por scorer pero DESCARTADAS: script sin superficie API nueva ni `web/` — scope discipline, mismo criterio que EMB-10/11)

## 1. TAREA

**Objetivo:** script `vanta-mcp-local.ps1` (raíz del repo) que deja las env vars de embeddings explícitas en la sesión y arranca `vanta-cli server --mcp --db <arg>` para que el proceso MCP nazca con el modelo correcto — verificable vía `dim` en la respuesta de `embed_texts`.

**Contrato exacto (ley):**
1. El script setea `VANTADB_EMBEDDING_PROVIDER` + `VANTADB_LOCAL_MODEL` (ABSOLUTA) en la sesión y arranca `server --mcp --db <arg>`.
2. `--db` EXPLÍCITO y obligatorio: sin default que pueda pisar la base del usuario (el CLI trae default `./db` + fallback `VANTADB_STORAGE_PATH` — el lanzador NO hereda ese default silencioso; exige el arg).
3. `embed_texts` responde con la `dim` del modelo activo (384 para `multilingual-e5-small`).
4. Secrets (openai key) SOLO de env de sesión, jamás a disco ni al script (sin parámetro de key, sin writes, sin echo de valores).

**Acceptance del plan:** Gate Justificación — `opencode.jsonc:77-88` no tiene ejemplo `environment` (verificado: bloque `vantadb` solo con `command`+`args`, sin `env`) → env por script es el camino cierto (no asumir config). Flujo usuario: doble clic/comando → server arriba con modelo correcto. Flujo caso de uso: OpenCode/otro agente apunta su MCP al lanzador y listo.

## 2. ARCHIVOS

**Clave (leer completo antes de ACT — Regla 0):**
- `vanta-mcp-local.ps1` (NUEVO, raíz; NO existe — verificado `Test-Path` False + RED §Ejecución) — el entregable.

**Relacionados:**
- `opencode.jsonc:77-88` (LEÍDO COMPLETO: bloque `vantadb` = `command: [vanta-cli, server, --mcp, --db, C:/Users/Eros/.vantadb]`, SIN `environment` → a documentar/actualizar SOLO si el lanzador cambia el comando — tocar mínimo; este task NO lo modifica, solo verifica el camino cierto).
- `src/config.rs:935-940` (LEÍDO: `VANTADB_LOCAL_MODEL`, default relativo `embeddings/models/multilingual-e5-small/onnx`) y `:954-959` (`VANTADB_EMBEDDING_PROVIDER`, default `ollama` — sin servidor local = degradación; por eso el lanzador fija `local`).
- `src/cli.rs:13-22` (LEÍDO: `--db` global con `env = VANTADB_STORAGE_PATH`, default `./db` — el default que el lanzador NO debe heredar en silencio) y `:323-354` (`Server { http/mcp/port/host/... }` — para MCP solo importan `--mcp` + `--db`).
- `src/cli_handlers/server.rs:253-289` (LEÍDO COMPLETO: `cmd_server_mcp` spawnea `vantadb-server --mcp` con `VANTADB_STORAGE_PATH=<db_path>` + stdio heredado + **env del padre heredado** (`Command` hereda env por defecto) → las vars que setea el script SÍ llegan al proceso MCP; verificado por lectura, a probar en vivo).
- `docs/api/MCP.md:18` (LEÍDO: stdio JSON-RPC 2.0, launcher canónico `vanta-cli server --mcp --db ~/.vantadb`) y `:218` (perfil `full` default, 79 tools — el lanzador no fija perfil; fuera de scope).
- `docs/dev/tasks/EMB-10.md` (binario con motor: `target/debug/vanta-cli.exe` 24.8MB + `vantadb-server.exe` 40MB presentes; ORT_API_VERSION=27; ORT 1.30 en `Temp/opencode/ort130` efímero; `ORT_DYLIB_PATH` soportado por ort).
- `docs/dev/tasks/EMB-11.md` (LEÍDO COMPLETO 2026-09-16: paralela disjunta — 0 archivos compartidos; su config a consumir: default `multilingual-e5-small` 384d, ORT persistente futuro `LOCALAPPDATA\VantaDB\onnxruntime`, Temp como cache oportunista; EMB-11 está IN PROGRESS — leo sin bloquearme, defaults propios si falta).

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, archivos de EMB-11 (`setup-embeddings.ps1` — paralela disjunta), `Cargo.toml`, código Rust, `~/.cargo/bin` (instalación = EMB-19), `docs/dev/Backlog.md` y `docs/dev/avance/` (NO tocar por orden), `opencode.jsonc` (solo lectura en este task).

## 3. DEPENDENCIAS

- **Wave1**, paralela con EMB-11 (disjunta: 0 archivos compartidos; leo su task file/config si existe, sin bloquearme — defaults propios si no: provider `local`, modelo `multilingual-e5-small` absoluto, ORT autodetect Temp→LOCALAPPDATA→warn).
- **Hereda (Wave0 EMB-10 ✅ `a0f65d29`):** binario con motor en `target/debug/` (NO en PATH — `Get-Command vanta-cli` vacío → el lanzador resuelve PATH→repo-local), modelo e5-small en disco (onnx 470MB + tokenizer 17MB), ORT 1.30 en Temp.
- **Sin bloqueantes.** Next: Wave2 (EMB-13/16/18). Sirve a EMB-19 (verificación e2e usa el lanzador).

## 4. REFERENCIAS

- `.opencode/references/clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: script = borde exterior/composición → Humble Object, cero lógica de negocio; SLAP; stuttering; severidades /cleanCA — script nuevo: sin deuda 🟡 si ≤20 líneas/fn).
- `.opencode/rules/server-mcp.md` (LEÍDA COMPLETA: R-1 versiones coherentes — el lanzador no declara versiones, N/A; R-2/R-3 handler/metrics — lado servidor, no tocado).
- `security-and-hardening` (CARGADA: checklist §Secrets Management — sin parámetro de key, solo lee env de sesión, mask en output, 0 writes; threat model: trust boundary = env de sesión heredado al hijo; abuse case = key persistida a disco o logueada).
- `docs/api/MCP.md:18,218` (LEÍDAS: stdio + perfil full).
- Notion (Paso 0c, 4 páginas LEÍDAS COMPLETAS vía fetch): Problema (dimensión semántica garbage-in=garbage-out → justifica embeddings; memoria como superficie de seguridad → justifica no-loguear-secrets) + Propuesta (retrieval híbrido REAL; `extract_skills`/gobernanza PROPUESTA — fuera de scope). Nuevas features / Plan de accion: índices sin mapeo a EMB-12 → no aplican (filtro VantaDB).
- **Spec (scripts: N/A símbolos públicos Rust — tabla de decisiones, vale como spec §Gate mecánico):**

| Decisión | Opción | Evidencia |
|---|---|---|
| Ubicación | raíz `vanta-mcp-local.ps1` | plan; raíz más descubrible, par de `setup-embeddings.ps1` (EMB-11) |
| `--db` obligatorio, sin default | param `mandatory` — ni `./db` del CLI ni base del usuario por omisión | pre-mortem plan (default pisa base); `cli.rs:19` default `./db` existe → hay que bloquearlo |
| Provider default `local` | `VANTADB_EMBEDDING_PROVIDER=local` salvo `-Provider` explícito | `config.rs:956` default `ollama` sin servidor en esta máquina → default del server no sirve |
| Modelo default absoluto | `$PSScriptRoot/embeddings/models/multilingual-e5-small/onnx` resuelto a absoluta | `config.rs:937` default relativo frágil por CWD; `manifest.json:3` default id |
| Binario | repo-local `target/debug/vanta-cli.exe` → PATH → error claro (`-VantaCli` override) | orden INVERTIDO tras evidencia viva: el global `~/.cargo/bin` está STALE (sin `embed_texts`, error -32601) mientras el build EMB-10 del repo responde 79 tools; misma regla que `test-mcp.py` ("stale installed binary masks drift"); install global al día = EMB-19 |
| ORT nativo | autodetect: `$env:ORT_DYLIB_PATH` → `LOCALAPPDATA\VantaDB\onnxruntime` (EMB-11) → `Temp/opencode/ort130` (EMB-10) → warn y seguir | ORT_API_VERSION=27 (EMB-10); System32 1.17.1 ABORTA (FIND-100); setear solo si existe, nunca descargar (eso es EMB-11) |
| Secrets | sin parámetro de key; la key hereda del env de sesión si existe; output nunca muestra valores | contrato NUNCA-a-disco; security-and-hardening (secret commiteado se rota, no se borra) |
| Passthrough | `ValueFromRemainingArguments` al server | 1 línea, future-proof (`--help` del server, flags HTTP futuros) sin mantenimiento |
| Sin símbolos públicos nuevos | N/A — script PS1, 0 Rust tocado | Gate D no dispara `question` (ver §Spec Gate) |

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords launcher-script/env-config/mcp-stdio/opencode-config, ≤8):**
campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design (+ security-and-hardening por contrato secrets; frontend-ui-engineering y api-and-interface-design sugeridas por scorer pero DESCARTADAS con justificación: sin `web/` y sin superficie API nueva — script Humble Object, scope discipline, mismo criterio EMB-10/11).
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** security-and-hardening, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development.
**doubt-driven:** claim crítico = "el script nunca persiste ni loguea secrets y `--db` nunca defaultea a la base del usuario" → reconciliación adversarial en §Ejecución (single-model; cross-model skipped: non-interactive + sin CLI externo autorizado).

## 6. HERRAMIENTAS+MCP

- RED: `.\vanta-mcp-local.ps1 -Db ...` (antes de crear: "no se reconoce..." ✅ registrado §Ejecución).
- GREEN por slice: `[System.Management.Automation.Language.Parser]::ParseFile` (parse) + corrida real con DB temporal (`$env:TEMP\vanta-emb12-test` — NUNCA la base del usuario).
- Arranque real + `initialize` + `tools/call embed_texts` vía stdio (evidencia viva, como verificación MVP 2026-09-16): pipe JSON-RPC al proceso lanzado por el script, assert `dim == 384` del modelo activo.
- `git diff --check` + `git status --short` (alcance: solo `vanta-mcp-local.ps1` + este task file).
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO; 0 Rust tocado → fmt/clippy/nextest N/A, se documenta).
- OCR: `pwsh dev-tools/ocr-review.ps1` (advisory) al cierre.
- Sin red → investigación internet N/A + TSYS-13: ninguna cita URL como evidencia.

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY)

**Blast radius launcher→env→wrapper→server:**
- `opencode.jsonc:77-88` verificado SIN `environment` → el bloque MCP actual no puede fijar provider/modelo → env por script es el camino cierto (no asumir config). Este task NO modifica `opencode.jsonc` (tocar mínimo; el cambio de comando, si aplica, es decisión posterior con el lanzador ya probado).
- `cmd_server_mcp` (`server.rs:253-289`): `vanta-cli server --mcp --db X` setea `VANTADB_STORAGE_PATH=X` y spawnea `vantadb-server --mcp` heredando **todo el env del padre** → `VANTADB_EMBEDDING_PROVIDER`/`VANTADB_LOCAL_MODEL` seteadas por el script llegan al MCP sin tocar Rust. `Command` hereda env por defecto (std) — lectura verificada, prueba viva pendiente (smoke stdio).
- `Config::default` (`config.rs:935-959`): `VANTADB_LOCAL_MODEL` acepta absoluta; `VANTADB_EMBEDDING_PROVIDER` default `ollama` (sin servidor aquí → el lanzador debe fijar `local`).
- `Cli.db` (`cli.rs:13-22`): `--db` global, `env=VANTADB_STORAGE_PATH`, default `./db` — el default silencioso que el lanzador bloquea exigiendo `-Db` mandatory. (Nota: como `--db` es global, `server --mcp --db X` y `server --db X --mcp` equivalentes; el script usa el orden canónico de MCP.md: `server --mcp --db X`.)
- Binarios: `target/debug/vanta-cli.exe` (24.8MB) + `target/debug/vantadb-server.exe` (40MB) presentes (EMB-10); `vanta-cli` NO en PATH → fallback repo-local obligatorio en el script.
- `codegraph_explore` devolvió el wrapper + ruido TS (clientes, sin relación) — blast radius real = 1 archivo nuevo + 0 callers (script standalone invocado por humano/MCP-client). Cobertura CBM: `no_recorded_issue` en los 5 paths (best-effort, fuente leída directa de todas formas).
- `detect_changes` (scope files vs develop): 12 archivos cambiados, todos WIP ajeno/prohibidos (`.opencode`, `Justfile`, `completions/*`, `Cargo.lock` tauri, ocr-*, budget.json, `EMB-11.md` untracked, `reparacion.bat`) → NO tocar, commit SOLO propios.

**Impacto mapeado (Regla 0):**
- Archivos leídos completos: `opencode.jsonc`, `src/cli.rs` (Server + db global), `src/cli_handlers/server.rs`, `src/config.rs:925-984`, `docs/api/MCP.md` (§Getting Started + §profiles), `docs/dev/tasks/EMB-10.md`, `docs/dev/tasks/EMB-11.md`, `embeddings/manifest.json` (parcial 60L: default + dims), plan EMB-12 (§Wave1), clean-code Apéndice V, `server-mcp.md`, Notion ×4.
- Referencias hacia dentro (EMB-12 depende de): binarios EMB-10, modelo en disco, nombres de vars `config.rs`, comando canónico MCP.md, ORT Temp (cache oportunista) / LOCALAPPDATA (EMB-11 futuro).
- Referencias entrantes (dependen de EMB-12): EMB-19 (verificación e2e usa el lanzador); futuro cambio de comando en `opencode.jsonc`.
- Veredicto: 1 archivo NUEVO, 0 modificaciones; impacto = aditivo puro, rollback = borrar el .ps1. Riesgo: `--db` por defecto (mitigado: mandatory) + secreto a disco (mitigado: sin-parámetro + review).

## 8. INVESTIGACIÓN PROBLEMA

- Env que no llega al proceso MCP = modelo equivocado en silencio: el default del server es `ollama` (`config.rs:956`); sin servidor Ollama en esta máquina el provider degrada/falla mientras el usuario cree usar `local` → el lanzador lo hace explícito (escribe las vars en consola, maskeando secrets) + verificable (`embed_texts` devuelve `dim` del modelo activo; 384 = e5-small/default/fallback — el cableado real a vectores con señal es EMB-13, este task garantiza que el PROCESO nazca con el env correcto).
- Path relativo frágil por CWD (`config.rs:937`): el MCP puede spawnearse desde cualquier CWD (OpenCode, otro agente) → el lanzador resuelve ABSOLUTA desde `$PSScriptRoot`.
- `opencode.jsonc` sin `environment` (verificado): no hay camino config para el env → script (cierto) vs config (no verificado). Decidido por evidencia, no por gusto.

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: vars en `config.rs`, wrapper en `server.rs`, perfiles en MCP.md, binarios+modelo+ORT en disco). Sin red usada en esta tarea → sin citas. Estado: N/A + TSYS-13: ninguna cita URL presentada como evidencia; si surgiera duda de JSON-RPC MCP stdio → docs oficiales `modelcontextprotocol.io` + deuda TSYS-13 si sin red.

## 10. VALIDACIÓN+CIERRE

- [x] RED registrado (script inexistente → pwsh "no se reconoce") ✅ (DISCOVERY)
- [x] Slice 1 — params (`-DbPath` mandatory, `-Provider`/`-Model` con defaults, `-VantaCli` auto) + env sesión + `server --mcp --db` + parse limpio ✅
- [x] Slice 2 — ORT autodetect + warn, help, disciplina secrets (0 writes, 0 echo valores) ✅
- [x] Smoke vivo: script arranca MCP en DB temporal → `initialize` OK → `tools/list` 79 + `embed_texts` → `tools/call embed_texts` responde `dim == 384` ✅
- [x] `git diff --check` limpio (exit 0) + alcance solo propios (`vanta-mcp-local.ps1` + este task file, ambos untracked) ✅
- [x] `campaign_verify_cmd` intentado (bug exit -1) → bash directa + N/A Rust documentado (0 código tocado) ✅
- [x] OCR advisory ✅ (ver §Ejecución)
- [x] DoD 3 niveles ✅ (ver §Ejecución)
- [x] Reviewer P2-01: script nuevo sin prod code → OCR + doubt-driven single-model; cross-model skipped (non-interactive) ✅
- [x] Gates D/V/C + RESULTADO §7 + Save Point ✅ (ver cierre)
- [x] Commit `feat:` SOLO propios. Backlog→avance NO tocar. Push vía vanta-lead (este worker NO pushea). ✅

## Steps

- [x] **Step 0 — DISCOVERY + task file:** contexto, archivos, Notion, skills, gates, RED ✅ COMPLETO
- [x] **Step 1 — Slice 1 launcher mínimo:** params + env + arranque + parse ✅ COMPLETO (parse 0 errores; `-DbPath` mandatory verificado; FIX: `-Db` colisionaba con alias de `-Debug` → renombrado `-DbPath`)
- [x] **Step 2 — Slice 2 ORT + help + secrets:** autodetect, warn, help, audit ✅ COMPLETO (FIX stdio: `Write-Host` contaminaba el canal JSON-RPC → todo info a stderr vía `[Console]::Error.WriteLine`; stdout queda puro)
- [x] **Step 3 — smoke vivo + cierre:** stdio `initialize`+`embed_texts` (dim==384), diff-check, OCR, commit, recitation, RESULTADO ✅ COMPLETO

## Ejecución (evidencia mecánica)

### RED (2026-09-16, antes de crear el script)
- `Test-Path .\vanta-mcp-local.ps1` → `False`.
- `.\vanta-mcp-local.ps1 -Db $env:TEMP\vanta-emb12-red` → `El término ".\vanta-mcp-local.ps1" no se reconoce como nombre de un cmdlet...` ✅ falla por razón correcta (archivo no existe).

### GREEN por slice
- **Slice 1 (2026-09-16):** `emb12-check.ps1` → `PARSE-ERRORS=0`, params `DbPath,Provider,Model,VantaCli,ServerArgs(+comunes)`, synopsis OK. Sin `-DbPath` → `faltan uno o varios parámetros obligatorios: DbPath` exit 1 ✅. Incidente útil: `-Db` colisiona con alias de `-Debug` (common parameter) → renombrado `-DbPath` (inequívoco), RED previo actualizado.
- **Slice 2 (2026-09-16):** ORT autodetect verificado — primera corrida encontró `LOCALAPPDATA\VantaDB\onnxruntime\onnxruntime.dll` (¡EMB-11 paralela ya dejó el persistente! prioridad correcta sobre Temp). Secrets audit: `Select-String ... KEY|TOKEN|SECRET|...` → único match = la palabra "Secrets" en el comment-header (documenta la disciplina); 0 params de key, 0 writes, 0 echos de valores ✅.
- **Slice 3 smoke vivo (2026-09-16, `Temp/opencode/emb12-smoke.py`, harness en TEMP no commiteado):**
  - Corrida 1 (PATH-first): `initialize` OK pero `tools/call embed_texts` → `-32601 Tool not found` — el global `~/.cargo/bin/vanta-cli.exe` está STALE. HALLAZGO, no verde → flip de resolución a repo-local-first.
  - Corrida 2 (repo-local-first): `INIT-OK vantadb 0.5.0` + `TOOLS: 79 (embed_texts: True)` + `EMBED model=multilingual-e5-small dim=384 count=1` ✅ + stdout residual vacío (sin polución) ✅ + `DB-DIR-ENTRIES: 7` (`--db` llegó al server; base del usuario intacta) ✅ → **SMOKE-GREEN**.
  - Nota honesta: `dim==384` coincide con el modelo activo e5-small; la señal semántica real (vectores con significado vs hash) es EMB-13/FIND-99 — este task prueba que el PROCESO nace con el env correcto, no el cableado del provider.

### Secrets audit (doubt-driven CLAIM: "el script nunca persiste ni loguea secrets y `--db` nunca defaultea a la base del usuario")
- Reconciliación adversarial (single-model; cross-model skipped: non-interactive, ver §10):
  - "¿Acepta key por parámetro?" → params: DbPath/Provider/Model/VantaCli/ServerArgs. NO ✅
  - "¿Escribe a disco?" → grep `Set-Content|Add-Content|Out-File` = 0 hits ✅
  - "¿Loguea valores sensibles?" → solo escribe Provider/Model/DbPath/BIN/ORT-path (ninguno secreto); las vars `*KEY*`/`*TOKEN*` ni se leen (herencia implícita intacta) ✅
  - "¿`--db` puede caer al default `./db` o a la base del usuario?" → `-DbPath` mandatory sin default; el server recibe `--db $DbPath` siempre explícito ✅
  - "¿El info a stderr rompe el canal MCP?" → verificado: stdout residual vacío tras initialize+list+call ✅
- Veredicto: CLAIM SOSTENIDO. Sin hallazgos accionables (1 ciclo, triviales: ninguna).

### Secrets audit (doubt-driven CLAIM: "el script nunca persiste ni loguea secrets y `--db` nunca defaultea a la base del usuario")
- Diseño: sin parámetro de key; sin `Set-Content`/`Add-Content`/`Out-File`/`>`/`>>` en todo el script; output de vars maskea `*KEY*`/`*TOKEN*`/`*SECRET*`; `-Db` mandatory sin default.
- (reconciliación adversarial al cerrar Step 2)

## Spec Gate

Scripts: N/A símbolos públicos Rust — solo launcher (tabla §4). Gate D: blast radius 1 archivo nuevo + 0 ediciones, sin hot path, sin API pública, contrato mecánico + decisiones owner Q1-Q5 (Gate P del plan) → NO dispara `question`. Motivo: owner ya decidió + alcance aditivo mínimo.

### Cierre (2026-09-16)
- `git diff --check` → exit 0 (warning LF/CRLF solo de WIP ajeno `completions/_vanta-cli.ps1`, no mío). Alcance: `?? vanta-mcp-local.ps1` + `?? docs/dev/tasks/EMB-12.md` — 0 archivos ajenos.
- `campaign_verify_cmd command="git diff --check"` → `passed:false exitCode:-1` vacío (bug conocido, 4º+ caso) → fallback bash directa (exit 0) + mención en RESULTADO. 0 Rust tocado → fmt/clippy/nextest N/A documentado.
- OCR: preview workspace (8 reviewable/11; EMB-12.md excluido `unsupported_ext`) + `ocr delegate rule vanta-mcp-local.ps1` → Rule Group system/default SIN hallazgos (sin Critical/High → no bloquea; sin Medium → sin fila FIND-*). Auto-revisión contra la rúbrica: Correctness (smoke GREEN + boundaries: mandatory/warn/throw) ✅ · Security (sin params/writes/echoes de secrets; sin `Invoke-Expression`, args como array) ✅ · Performance (trivial, un spawn) ✅ · Maintainability (~100L, header con contrato) ✅ · Coverage (smoke + 4 boundaries) ✅.
- Reviewer P2-01: cambio no-crítico (script nuevo, 0 prod code) → OCR + doubt-driven single-model bastan; cross-model skipped: non-interactive + sin CLI externo autorizado (anunciado, no silenciado).
- DoD 3 niveles: (1) correctness+quality — parse 0, mandatory enforced, smoke GREEN, secrets audit ✅ · (2) integration+docs — task file completo, funciona con build EMB-10 repo-local, `opencode.jsonc` NO tocado (ejemplo de comando en header), EMB-11 consumida sin bloqueo ✅ · (3) ship-readiness — rollback = borrar 2 untracked, sin migración, commit `feat:` solo propios, sin push (vanta-lead) ✅.
- Gates: P heredado (Q1-Q5 owner, sin preguntas nuevas) · D no disparado (1 archivo nuevo, 0 símbolos públicos, contrato mecánico) · V no disparado (verify pasó; los 2 bugs — `-Db`/Debug y Write-Host/stdio — fueron de implementación, cazados por tests propios, fixeados y re-verificados, no mismo-error×2) · C evaluado (alcance limpio, colaterales: ninguno).
- learnings: `campaign_memory_write lessons` (ver recitation).

## Context Save Point

CERRADO 2026-09-16. Smoke GREEN (dim==384). Commit `feat:` propio sin push. Siguiente: Wave2 (EMB-13/16/18). Deuda TSYS-13: ninguna cita URL como evidencia.
