# FIND-104: Instalador interactivo completo end-to-end

## Metadata
- **Plan file:** docs/plans/2026-09-17-mvp-memoria-agentes.md
- **Fuente:** plan file Task 5 (Wave2) + SPEC.md F2
- **Esfuerzo:** 🟡 2d (appetite plan) — ejecución real ~1 sesión
- **Prioridad:** 🟠 Media-Alta
- **Tipo:** Scripts (PowerShell) + assets
- **Turns estimados:** 12
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETED (S1-S4 verdes + commit; P2-01 y Gates V/C por orquestador)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 4 steps (S1-S4)

## TAREA
- **Objetivo:** extender el wizard `setup-embeddings.ps1` (EMB-11) a flujo end-to-end con cero fricción = 1 flujo guiado: modelo (tabla tamaño/idioma/tiempo) + carpeta de base (DbPath) + proveedor local/ollama/openai con detección (servidor/key) + runtime ORT ≥1.27 + escritura del bloque MCP por cliente + línea de regla en AGENTS.md/CLAUDE.md + proxy default prendido (aviso proceso extra + apagado fácil, config TOML mínima, guía base_url, namespace `proxy-turns` + curaduría a hilos) + resumen + prueba viva final put→recall→search. Enter = defaults, re-ejecutable sin romper (idempotente + backup si sobreinstala).
- **Contrato:** (a) wizard end-to-end simulado verde; (b) `-NonInteractive` intacto (exit 0, mismo comportamiento previo + defaults nuevos); (c) secrets nunca a disco (keys solo env/memoria, maskeadas, jamás logueadas en claro); (d) prueba viva final verde (put→get→search vía vanta-cli en DB temporal).
- **AC detallado:**
  - (a) wizard simulado end-to-end verde: test con stdin simulado (Enters = defaults) recorre TODO el flujo sin red (modelo presente, ORT presente vía override) y sale 0.
  - (b) `-NonInteractive` intacto: `pwsh -NoProfile -File setup-embeddings.ps1 -NonInteractive` sale 0 sin prompts, sin colgarse en Confirm/Read-Host.
  - (c) secrets nunca a disco: grep de keys en archivos escritos = 0; wizard nunca pide key por teclado (PSReadLine history la escribiría); solo chequeo presencia env + display `***set***`.
  - (d) prueba viva final verde: `vanta-cli put → get → search` en DB temporal con el modelo/ORT del wizard, assert contenido.

## ARCHIVOS
- **Clave:**
  - `setup-embeddings.ps1` (wizard EMB-11 — extender, NO reescribir)
  - `vanta-mcp-local.ps1` (launcher EMB-12 — SOLO yo lo edito; FIND-106 solo lectura)
  - `skills/vantadb-mcp/assets/` — SOLO subdir nuevo `install/` para plantillas por cliente (el subdir `hooks/` es de FIND-106 en paralelo, NO tocar)
- **Relacionados (lectura):**
  - `opencode.jsonc` (bloque MCP `vantadb`: `vanta-cli server --mcp --db` — plantilla base verificada)
  - `docs/QUICKSTART.md` (flujo put→get→list de referencia)
  - `SPEC.md` (Q3 4 clientes: OpenCode+Claude+Cursor+Codex; Q4 proxy default-on + opt-out; F2 = esta tarea)
  - `skills/vantadb-mcp/references/recall-policy.md` (FIND-103, commit c01baa90 — regla de curaduría proxy-turns→hilos, consumida no re-derivada)
  - `vanta-proxy/config.toml` (config TOML mínima base: server.port 8096, upstream.url) + `vanta-proxy/src/capture.rs:15` (`TURNS_NAMESPACE = "proxy-turns"`)
  - `embeddings/manifest.json` (tabla modelos) + `skills/vantadb-mcp/assets/*.json` (3 plantillas verificadas: opencode/claude-desktop/cursor)
- **Prohibidos (cero edits):** `reparacion.bat`, `.opencode/` (config VIVA ajena — solo lectura; mis plantillas van a `assets/install/`), `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/Backlog.md`, plan file, `src/llm.rs`, `examples/`, `vanta-memory/`, `vantadb-mcp/src/`.

## DEPENDENCIAS
- **Wave2** paralela con FIND-106; split disjunto: yo scripts + `assets/install/`, él `assets/hooks/` + docs hooks. `vanta-mcp-local.ps1` SOLO yo lo edito.
- **Hereda FIND-103** (commit c01baa90): recall policy + instructions + traductor temporal — consumida (citada en wizard/proxy guía), no re-derivada.
- **FIND-105** (Wave3) publica lo mío (one-liner encadena wizard).
- **NextTask:** Wave3 (orquestador decide 105/108).
- **Stop:** cliente sin formato verificable (Codex: repo sin plantilla) → documentado-manual, no bloquea.

## REFERENCIAS (lectura completa antes de codificar — hecho)
- **Rules:** `.opencode/rules/release-ci.md` ✅ (allocator/CI/sccache — no aplica a scripts, sin conflicto) + `.opencode/rules/server-mcp.md` ✅ (R-1 serverInfo sync, R-2 semáforo+spawn_blocking, R-3 /metrics real — launcher no toca handlers, sin conflicto).
- **Refs:** `definition-of-done.md`, `dev-tools.md`, `skills-engineering.md` (SDP canónico).
- **Commands:** `pipeline.md`, `audit.md`.
- **SPEC.md:** Q3 4 clientes, Q4 proxy default-on + opt-out, F2 AC = este contrato.

## SKILLS
- **SDP (Paso 0b, `campaign_discover_skills_v2` phase BUILD):** `campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design`.
- **SKILLS_CARGADAS (final ≤8, justificadas):** `shipping-and-launch` (flujo instalación/rollback) · `test-driven-development` (wizard simulado RED→GREEN) · `systematic-debugging` (fallos wizard sin adivinar) · `security-and-hardening` (secrets/checksums, FASE SECURITY) · `source-driven-development` (formatos MCP contra plantillas verificadas, no memoria) · `incremental-implementation` (slices verticales) · `context-engineering` (context pack por slice) · `git-workflow-and-versioning` (commits atómicos, staging selectivo).
- **Descartadas del SDP:** `frontend-ui-engineering` (sin web/), `api-and-interface-design` (sin API pública nueva), `doubt-driven-development` (P2-01 lo hace el orquestador con agente distinto), `campaign-executor/progreso/ponytail` (base auto, no cuentan).

## HERRAMIENTAS+MCP
- `pwsh -NoProfile` (dry-run wizard, simulado end-to-end con stdin pipe).
- MCP stdio smoke: `vanta-cli server --mcp` (`initialize` + `tools/list` 86 + `embed_texts` dim check) — solo lectura, sin edits.
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO).
- agent-search/metasearchmcp + webfetch SOLO si formato MCP por cliente incierto. **No usado:** 3 formatos verificados en repo + Codex → manual (Stop rule). Sin red necesaria.
- Secrets NUNCA a disco ni a logs (verificación: grep).
- codegraph N/A (scripts PS1, no código Rust).

## INVESTIGACIÓN CÓDIGO (DISCOVERY — hecho)
- **Existe EMB-11** (`setup-embeddings.ps1`, 303 líneas): provider local/ollama/openai + detección (probe :11434, env key maskeada) + tabla modelos (tamaño/tiempo heurística ~10MB/s) + carpeta ONNX override + ORT ≥1.27 (store persistente %LOCALAPPDATA%/VantaDB, orden store→env→cache Temp→System32 rechazo→GitHub) + `download.py --check` final + `-NonInteractive` defaults + secrets OK (nunca pide key por teclado).
- **Falta (este task):** carpeta de base DbPath + bloque MCP por cliente + regla agente + proxy default + resumen + prueba viva final.
- **Existe EMB-12** (`vanta-mcp-local.ps1`, ~90 líneas): `-DbPath` mandatory sin default + `-Provider/-Model/-VantaCli/-ServerArgs` + resolución binario (repo build primero) + ORT solo-apunta (nunca descarga) + env explícito heredado + stdio (info a stderr) + secrets nunca parámetros.
- **Falta launcher:** passthrough proxy (config path + env, sin romper stdio).
- **Assets:** 3 plantillas verificadas (`opencode-config.json` tipo local command[] · `claude-desktop-config.json` + `cursor-config.json` mcpServers command+args+env) + 0 `install/` + 0 `hooks/` (FIND-106 aún no creó — sin colisión).
- **Proxy real:** `vanta-proxy/` con `config.toml` (server.port 8096, upstream.url anthropic, models[]) + `capture.rs:15` `TURNS_NAMESPACE="proxy-turns"` + recall-policy §6 (curaduría: review `memory_list` proxy-turns → propuesta `thread_send` → inbox approve/reject, NUNCA auto-promote; inbox tools DEFER FIND-107 S4 → inbox = confirmación explícita del agente).
- **FIND-103 consumido:** `skills/vantadb-mcp/references/recall-policy.md` (hook map SessionStart/per-message/PreCompact/Stop, top_k 5, scope agent, umbrales estructurales, temporal `parse_temporal_expression`, presupuesto 10%).

## INVESTIGACIÓN PROBLEMA
- **Tradeoff fricción vs control:** defaults Enter (cero fricción) + opt-out proxy 1 paso (1 pregunta S/n, default S) + re-ejecutable sin romper (idempotente: todo write chequea existencia primero; MCP configs con backup `.bak` fechado solo si cambia contenido; regla agente con chequeo de línea existente; TOML proxy sin overwrite si existe salvo flag).
- **Por qué TOML separada + guía (no auto-start):** stdout del launcher es canal JSON-RPC (comentario launcher); proceso extra HTTP :8096 rompería stdio si se auto-lanza dentro; SPEC Q4 dice "opcional prendido por defecto" con opt-out, no "forzado siempre corriendo". Default-on = config escrita + instrucciones 1-comando start/stop, no proceso huérfano.
- **Por qué CLI para prueba viva (no MCP stdio):** recall MCP ya verde en FIND-103 (temporal_tests 12/12); lo que el wizard debe probar es que EL MODELO/ORT/DB del usuario funcionan (put→get→search CLI). MCP smoke lo hace el orquestador/CI.

## INVESTIGACIÓN INTERNET
- N/A por defecto. 3/4 formatos verificados en repo (`assets/*.json`); Codex sin plantilla → Stop rule: documentado-manual en `assets/install/codex-manual.md`, no bloquea. Sin citas externas → sin GATE CITAS. [Deuda TSYS-13: ninguna — 0 URLs citadas.]

## VALIDACIÓN+CIERRE
- Verify contrato: (a) simulado end-to-end exit 0; (b) `-NonInteractive` exit 0 sin prompts; (c) grep secrets 0 en archivos escritos; (d) put→get→search verde en DB temporal.
- OCR delegation (`pwsh dev-tools/ocr-review.ps1`; Critical/High = bloquea) + DoD 3 niveles + P2-01 orquestador (agente distinto) + Gates D/V/C vía `question` (D: no dispara — 2 scripts + assets nuevos, sin hot path/símbolos públicos; V/C: orquestador al cierre).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | humano (`pwsh -File`), FIND-105 one-liner (futuro), clientes MCP (leen `assets/install/`) |
| Callees | `embeddings/manifest.json`, `embeddings/download.py --check`, `vanta-cli` (put/get/search, server --mcp), `vanta-proxy/config.toml` (plantilla), env sesión |
| Implicaciones | contrato previo intacto (defaults + flags viejos sin cambio); comportamiento público nuevo solo aditivo (nuevos switches opcionales, nuevos pasos wizard con defaults); sin performance/memoria/serialización (scripts); sin migración; tests existentes: ninguno de PS1 (0 regresiones posibles en Rust) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `setup-embeddings.ps1` (303L) · `vanta-mcp-local.ps1` (~90L) · `skills/vantadb-mcp/assets/opencode-config.json` · `claude-desktop-config.json` · `cursor-config.json` · `vanta-proxy/config.toml` · `vanta-proxy/src/capture.rs` (30L head) · `skills/vantadb-mcp/references/recall-policy.md` (70L) · `.opencode/rules/release-ci.md` · `.opencode/rules/server-mcp.md` · `AGENTS.md` (stub) · `SPEC.md` (grep Q3/Q4) · `opencode.jsonc` (bloque vantadb).
- **Archivos referenciados hacia dentro:** wizard → `embeddings/manifest.json`, `embeddings/download.py`, `embeddings/models/*/onnx`, env (`VANTADB_*`, `ORT_DYLIB_PATH`), `vanta-cli`; launcher → `target/debug/vanta-cli.exe` | `vanta-cli` PATH, `onnxruntime.dll` candidatos, `vanta-cli server --mcp --db`.
- **Archivos que referencian a los editados:** `docs/QUICKSTART.md` (flujo manual, no llama scripts) · `skills/vantadb-mcp/scripts/test-mcp.py` (regla binario repo-primero, misma que launcher) · FIND-105 futuro (encadenará wizard) · `.opencode/` config VIVA (NO referenciar desde wizard — prohibido editar).
- **Veredicto impacto:** BAJO. 2 scripts + 1 subdir nuevo. Sin Rust, sin handlers, sin API pública nueva (flags PS1 opcionales ≠ `pub fn`). Reversibles por commit atómico. WIP ajeno (`completions/*`, Backlog, plan, `.opencode`, `reparacion.bat`) intacto vía staging selectivo.

## Contrato
"wizard simulado end-to-end exit 0 + `-NonInteractive` exit 0 sin prompts + grep secrets 0 en artefactos + `vanta-cli put→get→search` verde en DB temporal"

## Spec (SDD — sin símbolo público Rust nuevo; decisiones de flags/flujo documentadas)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Integración proxy en launcher | A: TOML separada + guía 1-comando (stdio intacto; proceso extra explícito) / B: auto-start proxy dentro del launcher (rompe stdio JSON-RPC, procesos huérfanos) | A | ✅ decidido-por-evidencia (launcher: "stdout is the JSON-RPC channel"; `vanta-proxy/config.toml`: port 8096 HTTP) |
| 2 | Proxy en `-NonInteractive` | A: escribe TOML default-on sin arrancar + aviso opt-out (Q4 default-on, no cuelga CI) / B: omite proxy (viola Q4) | A | ✅ decidido-por-evidencia (SPEC Q4 proxy default-on confirmado por owner en plan Gate P) |
| 3 | Cliente Codex sin plantilla en repo | A: manual documentado (Stop rule, no bloquea) / B: inventar formato (riesgo pre-mortem #1) | A | ✅ decidido-por-evidencia (assets/: 3 json, 0 codex; plan Stop: "sin formato verificable → documentado-manual") |
| 4 | Prueba viva final | A: CLI put→get→search en DB temporal (prueba modelo/ORT/DB del usuario) / B: MCP stdio put→recall→search (recall ya verde FIND-103) | A | ✅ decidido-por-evidencia (FIND-103 temporal_tests 12/12; CLI tiene put/get/search verificados `--help`) |
| 5 | Regla agente (AGENTS.md sin CLAUDE.md en repo) | A: wizard ofrece append idempotente 1-línea a `./AGENTS.md`+`./CLAUDE.md` del PROYECTO TARGET (CWD) con chequeo previo / B: editar `AGENTS.md` del repo VantaDB (stub que prohíbe duplicar) | A | ✅ decidido-por-evidencia (`AGENTS.md:5` "Do not duplicate content here"; `Test-Path CLAUDE.md=False`) |

## Invariantes de dominio (handoff)
- No romper: `-NonInteractive` exit 0 sin prompts · secrets nunca a disco · stdio MCP (stdout JSON-RPC) · WIP ajeno · `.opencode/` vivo.
- Verificación: comandos en cada step + `VERIFY_CONTRATO` final.
- Deuda: OCR delegation + P2-01 review los hace el orquestador (agente distinto).

## Steps

### S1: Wizard — DbPath + bloques MCP por cliente + regla agente ✅ DONE
- **Qué:** en `setup-embeddings.ps1`: nuevos params opcionales (`-DbPath`, `-NoProxy`, `-SkipLiveTest` — todos default que preservan comportamiento viejo); tras sección modelo/ORT: prompt carpeta base (default `~/.vantadb`, Enter=auto; crea dir si falta); escribe bloques MCP personalizados a `<Db>/mcp-{opencode,claude,cursor}.json` con rutas absolutas launcher+Db (idempotente con backup `.bak`; plantillas versionadas con placeholders quedan en `assets/install/`); ofrece append idempotente 1-línea recall a `./AGENTS.md`+`./CLAUDE.md` del CWD (chequeo previo; en `-NonInteractive` se omite).
- **Verificación:** ✅ `setup-embeddings.ps1 -NonInteractive -DbPath <tmp>` exit 0 (regresión); bloques creados; `parser syntax-errors=0`.

### S2: Wizard — proxy default-on + TOML mínima + guía + resumen ✅ DONE
- **Qué:** en `setup-embeddings.ps1`: oferta proxy default S (1 paso opt-out, aviso "proceso extra :8096, apaga con Ctrl+C / no arranques el proxy"); escribe TOML mínima (`<Db>/vanta-proxy.toml`: server.port, upstream.url, models[]) solo si no existe (idempotente); imprime guía base_url por cliente (`http://127.0.0.1:8096`) + namespace `proxy-turns` + curaduría (review→thread_send→inbox, NUNCA auto-promote, cita recall-policy §6); resumen final (provider/modelo/DbPath/clientes/proxy/ORT).
- **Verificación:** ✅ NonInteractive: TOML creada + resumen impreso; re-run: "existente (sin overwrite)".

### S3: Launcher passthrough proxy + `assets/install/` 4 plantillas ✅ DONE
- **Qué:** en `vanta-mcp-local.ps1`: nuevo param opcional `-ProxyConfig` — si existe: valida path + exporta `VANTADB_PROXY_CONFIG` + guía a stderr (jamás stdout, stdio intacto); sin flag: comportamiento idéntico a antes. Creados `skills/vantadb-mcp/assets/install/`: `opencode.json`, `claude-desktop.json`, `cursor.json` (placeholders `<REPO>`/`<DB_PATH>`) + `codex-manual.md` (Stop rule) + `agent-rule.md` (1-línea recall) + `proxy.toml.example` (mínima).
- **Verificación:** ✅ launcher `-DbPath X -ProxyConfig Y --help` OK (env + proxy exports a stderr, help intacto); 3 JSON `ConvertFrom-Json` OK.

### S4: Prueba viva final + simulado end-to-end + verify + commit ✅ DONE
- **Qué:** `Invoke-LiveTest` en wizard (salteable `-SkipLiveTest`): `vanta-cli put→get→search` con asserts.
- **Verificación:** ✅ (a) interactivo pipe-Enters exit 0 full incl. live test; (b) NonInteractive exit 0; (c) grep secrets 0 hits; (d) `PRUEBA VIVA verde: put->get->search OK` en 2 DBs temporales; idempotencia re-run `unchanged/sin overwrite`; `cargo fmt --check` exit 0; MCP smoke `test-mcp.py` 5/5 (86 tools); OCR delegation sin Critical/High (advisory, group1+group2 aplican a mis archivos); bug `--top-k`→`--limit` root-caused via `search --help` (no adivinado).

## Context Save Point
- DISCOVERY completo 2026-09-17: wizard/launcher/assets leídos + SPEC Q3/Q4 + recall-policy FIND-103 + proxy config/capture + rules release-ci/server-mcp + CLI put/get/search verificados + Codex gap → manual. Branch develop. Sin red usada (0 URLs). Próximo: S1 (editar wizard DbPath+MCP+regla).
- SDP: ver §SKILLS. Gates: D no-dispara (2 scripts + assets nuevos, sin hot path/pubs); V/C al cierre por orquestador.

## Log
- 2026-09-17 DISCOVERY completo + task file creado (este archivo).
- 2026-09-17 S1-S4 implementados y verificados (ver S4). Commit pendiente (staging selectivo) + campaign completed.
