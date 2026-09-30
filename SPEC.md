# Spec: MVP — VantaDB como memoria automática en agentes de código

> **Para qué es este archivo:** es el contrato del plan `docs/dev/plans/2026-09-17-mvp-memoria-agentes.md`. Dice QUÉ se va a construir, POR QUÉ (el problema que resuelve), CÓMO se verifica y QUÉ NO entra. Se lee antes de ejecutar cualquier tarea del plan y se actualiza si una decisión cambia. **Para qué se utiliza:** los sub-agentes lo usan como fuente de verdad (objetivo, límites, criterios de éxito) para no desviarse; el owner lo usa para auditar alcance.

## Objective

Que un usuario nuevo pase de ver el repo en GitHub a tener memoria persistente funcionando en su agente de código (OpenCode, Claude Code, Cursor, Codex) con UN comando y CERO configuración manual: embeddings locales reales, recall automático en cada conversación (incluidos filtros temporales tipo "ayer a las 2pm"), captura al cerrar, y proxy opcional prendido por defecto.

**El problema que resuelve** (Notion `Problema`, completo): los agentes no tienen capa que gobierne el ciclo de vida de la memoria — olvidar decisiones, repetir errores corregidos, contaminarse con información obsoleta/duplicada/alucinada (`autoenvenenamiento`), deriva por resúmenes, cascadas multiagente. La ventana de contexto NO sustituye memoria persistente (tabla ventana-vs-memoria + "Lost in the Middle"). VantaDB ataca 6 áreas: persistencia, recuperación, selección, evolución, gobernanza, seguridad; y 8 dimensiones (trabajo, episódica, semántica, procedimental, temporal, confianza, identidad, meta-memoria).

**Dónde estamos** (Notion `Propuesta`, matriz REAL/PARCIAL 2026-09-14): motor v0.5.0 REAL (persistencia WAL, híbrido BM25+HNSW+RRF, CRUD/TTL/supersession, grafos, SDKs, server, MCP 87 tools, memoria L0→L3 parcial); PROPUESTA v0.7 (governance manual: `mark_duplicate`, `detect_conflicts`, `extract_skills`) → v1.0 (automática). **Regla de interpretación:** ¿fortalece la memoria embebida gobernada? Lo demás queda fuera. Este plan NO implementa v0.7/v1.0 (programa MGR, DEFER): deja el v0.5.0 usable automáticamente.

## Target Users

Dev que usa agentes de código y quiere que recuerden entre sesiones sin leer docs ni editar configs. Éxito = del repo al primer recuerdo recuperado por sinónimo en <30 min, sin compilar ni clonar.

## Core Features

- [ ] F1: Distribución sin fricción — AC: `install.sh/ps1` + binario release CON motor (`embed-local`), ORT+modelo descargados por instalador, one-liner sin clone ni rustup (FIND-104/105)
- [ ] F2: Instalador interactivo — AC: wizard con defaults (Enter=auto), proxy default-on + opt-out, bloque MCP por cliente, regla de agente, prueba viva final (FIND-104)
- [ ] F3: Recall continuo — AC: filtros temporales en búsqueda + traductor determinista NL→ms con tests + política recall-first con umbrales (FIND-103)
- [ ] F4: Ganchos por cliente — AC: plantillas OpenCode/Claude/Cursor/Codex (SessionStart, por-mensaje, PreCompact, Stop) con tests y presupuesto de tokens (FIND-106)
- [ ] F5: Superficie MCP completa de `vanta-memory` — AC: sueños, aprobación, extracción de skills, ingesta real, programador, escenas-escritura, cada uno con tool+schema+tests+docs (FIND-107)
- [ ] F6: Robustez embeddings — AC: sin abort ante dylib incompatible (FIND-100)
- [ ] F7: Demo prueba — AC: `agent_memory_cli` recuerda entre 2 sesiones (SHOW-04)
- [ ] F8: Review automatizado completo — AC: rules OCR por path + job CI nocturno + session viewer como evidencia (FIND-108)

## Tech Stack

Rust workspace (`vantadb`, `vanta-memory`, `vantadb-mcp`, `vantadb-server`, `vanta-proxy`), ONNX (`ort` load-dynamic, nativo ≥1.27), PowerShell/sh instaladores, GitHub Releases matrix + sha256, MCP stdio JSON-RPC, hooks por cliente (OpenCode `hooks`/plugin, Claude `hooks.json`, Codex `hooks.json`), open-code-review (`ocr`, delegation sin key por defecto). Restricción: sin red en CI/tests salvo descargas versionadas con checksum.

## Commands

```powershell
# Instalar (usuario final, sin clone) — FIND-105 (URLs reales desde 2026-09-17)
iwr https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex          # Windows
curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh     # Linux/macOS
pwsh setup-embeddings.ps1 -NonInteractive    # wizard defaults
# Lanzar MCP con modelo local
.\vanta-mcp-local.ps1 -DbPath C:\data\vantadb
# Verificar (dev, por tarea)
cargo test -p <crate> -j 2
cargo clippy -p vantadb --all-targets -- -D warnings
cargo fmt --check; git diff --check
pwsh scripts/validate-docs-coverage.ps1
pwsh dev-tools/ocr-review.ps1 -Format json   # delegation, sin key
cargo audit; cargo deny check
```

## Project Structure

```
scripts/install.sh|ps1   → instalador binario+modelo+ORT (FIND-105)
setup-embeddings.ps1     → wizard interactivo (FIND-104)
vanta-mcp-local.ps1      → launcher MCP con env (existe, EMB-12)
skills/vantadb-mcp/      → skill + assets por cliente + hooks templates (FIND-103/106)
src/llm.rs               → provider local + init ORT graceful (FIND-100)
vantadb-mcp/src/         → tools (filtros temporales, recall, exposición) (FIND-103/107)
vanta-memory/src/        → fuente a exponer (FIND-107)
examples/agent_memory_cli/ → demo F7 (SHOW-04)
.opencodereview/         → rules OCR por path (FIND-108, nuevo)
```

## Code Style

Convenciones del repo (Rust: `?`+`Result`, sin `unwrap` en prod, clippy `-D warnings`; helpers MCP <20 líneas/fn estilo `dim_mismatch_guidance`; PS1 `Set-StrictMode`, secrets nunca a disco). Reglas duras por área en `.opencode/rules/` (ver Marco normativo del plan).

## Testing Strategy

`cargo test -p <crate> -j 2` (unit+integración) · cat tests semánticos con umbrales medidos (pares≥0.90, gap≥0.05, Regla 11) · smoke MCP stdio en DB temporal (`initialize`+`tools/list`=87+put→get+search) · `test-mcp.py` por perfil (full/dev/memory) · hooks con eventos simulados por cliente · `validate-docs-coverage.ps1` 0 gaps · OCR delegation sin Critical/High. Sin red en tests (mocks/tmpdir).

## Boundaries

- Always: verify mecánico antes de commit; review P2-01 por agente distinto; secrets nunca a disco; fallback avisado (`fallback:true`), nunca silencioso; dim distinta bloquea+guía, nunca auto-reindex (Q4 embeddings-auto); commits atómicos con task ID; Backlog/avance solo vía orquestador.
- Ask first: nueva dependencia; cambio de API pública MCP; bins en release matrix; tocar `~/.cargo/bin` (instalación global).
- Never: commitear WIP ajeno (`.opencode`, `Justfile`, `completions/*`, tauri lock, ocr-*/reparacion.bat); matar procesos ajenos; reescribir Backlog fuera de la campaña; claims sin fuente (Regla 11); `irm|iex` sin checksum documentado.

## Decisiones (Q1-Q5 embeddings-auto + Q1-Q5 esta campaña)

| # | Decisión | Resuelto |
|---|----------|----------|
| 1 | Modelo default e5-small, 1 modelo por DB | ✅ owner Fase 0 |
| 2 | Instalador interactivo con defaults + proxy default-on/opt-out | ✅ Q1/Q4 |
| 3 | Clientes: OpenCode+Claude+Cursor+Codex | ✅ Q3 |
| 4 | SHOW-04 dentro del MVP; SHOW-03 fuera | ✅ Q1+Gate P |
| 5 | FIND-100 dentro del MVP | ✅ Q2 |
| 6 | Set DO (8) /DEFER/SKIP confirmado | ✅ Q5 Gate P |
| 7 | Dummy solo con flag / dim bloquea+guía | ✅ Q4/Q5 embeddings-auto |
| 8 | OCR: delegation default sin key; full solo nocturno con key | ✅ research 2026-09-17 |

## Alcance cierre-mvp (plan `2026-09-18-cierre-mvp.md`, Gate P 2026-09-18)

- **IMPL-112 (ingesta real):** la fuente de verdad es la spec `docs/dev/tasks/FIND-112.md` (trait `LlmRunner` reutilizado, matriz local+ollama/openai, TOML+env secrets-solo-env, gates G0–G4, 10 tests nombrados). S1 local primero; S2 solo si S1 sale sin fricción (Gate V si friccionó).
- **S4/S6b (aprobación/programador):** diseño primero en `FIND-110-spec` / `FIND-113-spec` (cero código); ship solo con dueño defendible, si no re-DEFER honesto.
- **TUI REPL:** los mutantes IQL deben funcionar en sesión (`src/tui/repl.rs`, engine read-only hoy) o quedar el límite documentado en su help con motivo (decide FIND-117, no re-diseñar el TUI).
- **Ask-first `~/.cargo/bin` → APROBADO** para FIND-98 (reinstall parity 79→87) vía Gate P 2026-09-18; si el lock persiste → STOP sin forzar.
- **`vantadb-ts/examples/`:** referenciar desde README/QUICKSTART salvo motivo escrito para mover (decide SHOW-05-resto).

## Success Criteria (campaña MVP)

1. Usuario nuevo: 1 comando → `embed_texts` real (`fallback:false`) + recuerdo guardado y recuperado por sinónimo en su agente (<30 min, sin compilar ni clonar).
2. "Qué hice ayer a las 2pm en el módulo X" → respuesta con cita del registro (test temporal verde).
3. `cargo audit`/`deny`/clippy/fmt verdes; coverage docs 0 gaps; OCR sin Critical/High.
4. 0 regresiones: suites `mcp_tests`/`memory` verdes; binario instalado == fuente.

## North Star (producto) — formalizada 2026-09-27 (DEF-05)

> **North Star: agentes activos que recuperan una memoria con éxito en ventana de 7 días** (proxy operacional: sesiones con put+search en la misma ventana de 7 días).

Los criterios de §Success Criteria son de **campaña (MVP)** — siguen vigentes para el MVP (ver Frontera de la Adenda). Este es el criterio de **producto**: mide que el núcleo (memoria embebida gobernada — jerarquía de DEF-01) se *usa* de punta a punta, i.e. un agente que almacena y recupera en la misma semana.

**Baseline (al 2026-09-26):** 0 sesiones — sin despliegue del proxy con el loop de memoria en uso externo. El contador arranca con el primer deployment instrumentado; ICP-01 lo consume en la demo CI.

### Medición contra el proxy actual (verificada 2026-09-27)

**PUT — derivable hoy.** Todo turno proxied exitoso con sesión se auto-captura y persiste en el store del proxy (`[auth] db_path`, default `vantadb_data`):

- `server.rs:256-258` (captura en éxito, fire-and-forget) → `server.rs:606-629` (`capture_turn`) → `capture.rs:53-139`.
- Namespace `proxy-turns` (`capture.rs:20`), key `{ms}-{seq}` (`capture.rs:65`), payload JSON `{session, protocol, space, model, text}` (`capture.rs:66-73`); además registro L1 en `l1/{session}` con `created_at` (`capture.rs:80-102`).

Consulta: `memory_list` (`vantadb-mcp`, read-only — `handlers/tools.rs:200-218`) o `list` del SDK contra el store del proxy; los keys `{ms}-{seq}` llevan el timestamp → filtrar client-side `ms ≥ now − 7d` (los filters son metadata-only — patrón página+filtra en `recall-policy.md:38,52`) y contar `payload.session` distintos.

**SEARCH — instrumentado (ICP-01, 2026-09-29).** Cada search ejecutado persiste un evento **metadata-only** (`{session, kind:"search", hits}`; key `{ms}-{seq}`) en el namespace `proxy-memory-events` vía `WriteBack::track` fire-and-forget (`vanta-proxy/src/memory_tools.rs`) — el wire nunca espera el write. Con eso la North Star queda derivable como sesiones presentes en `proxy-turns` ∩ eventos `search` con `hits ≥ 1` en la misma ventana de 7 días.

**FIND (implementado en ICP-01, 2026-09-29):** la instrumentación mínima del search quedó implementada exactamente como fue diseñada — persistir un evento por search ejecutado reutilizando `WriteBack::track` en `memory_tools::search` (namespace `proxy-memory-events`, key `{ms}-{seq}`, payload `{session, kind:"search", hits}`; sin tocar el wire y sin persistir query ni contenido). Consulta: `python scripts/north_star_metric.py --db <proxy-db>` (comando, contrato del evento y queries MCP equivalentes en `docs/api/PROXY.md` §North Star; self-test offline con `--self-test`). Nota de distribución: el script usa `vanta-cli mcp-call`, que viaja en el tren 0.8.0 (hoy en `develop`; v0.7.0 es anterior).

### Guardrails (no decorativos — fuente y medición por ítem)

| Guardrail | Fuente | Medición |
|---|---|---|
| 0 hallazgos high sin parche ≤ 7 días | owner 2026-09-24 (`VISION.md` §North Star) | Revisión semanal: filas abiertas high/crítica en `docs/dev/Backlog.md` + reportes de `docs/dev/reviews/` — ninguna con antigüedad > 7 días |
| 0 regresión p99 > 15% | gate revivido HARD-06/FIND-154 (2026-09-27) | `python benchmarks/compare_baseline.py` (bloqueo >15% en familias estables; bandas por familia) · Rust: `cargo bench -p vantadb --bench canonical_p99` vs `docs/user/operations/BENCHMARKS.md` §8 |
| 100% artefactos con versión sincronizada | rails HARD-01 (`docs/api/COMPATIBILITY.md`, `docs/api/VERSIONING.md`) + release-plz | Post-release: versión publicada (crates.io `vantadb` · npm `vantadb`/`vantadb-wasm` · PyPI `vantadb-py` · GitHub Release) == head de `docs/CHANGELOG.md` |
| 0 violaciones Regla 11 en material público | `AGENTS.md` Regla 11 | Review pre-publicación: todo claim de performance cita bench reproducible + comando (`BENCHMARKS.md`) |

## Adenda 2026-09-24 — Decisiones post-investigación integral

> Fuente: `docs/dev/plans/2026-09-24-post-investigacion-integral.md` + `docs/dev/research/product-definition-gap-2026-09-24.md`.

| # | Decisión (owner 2026-09-24) | Efecto |
|---|------------------------------|--------|
| 1 | **3 tracks ICP** a profundidad: AI-IDEs vía MCP · devs local-LLM/privacidad · frameworks | BIZ-08 resuelta; filas P54 (ICP-01..03) |
| 2 | **Migración única de schema en 0.7.0**: bitemporalidad + confianza + cuarentena | Filas P53 (SCH-01..08) |
| 3 | **Harness completo + head-to-head**: LoCoMo/LongMemEval-S/BEAM-subset + write-quality/abstención/tokens/p99-CI | Filas VER-08/VER-09 (absorbe EXE-02) |
| 4 | **North Star**: agentes activos que recuperan una memoria con éxito en ventana de 7 días (medible en proxy/MCP) | DEF-05 |
| 5 | **Naming freeze** 0.7.0→1.0 (9 artefactos; ADR) | DEF-04 |
| 6 | **Jerarquía de producto (DEF-01)**: 1 núcleo (memoria embebida gobernada, "SQLite para agentes") + 3 puertas ICP; RAG = capacidad del núcleo, nunca target (opción c, owner 2026-09-27) | Alinea SPEC/README/VISION en una sola jerarquía |

**Frontera:** este SPEC gobierna el MVP de memoria automática; la frontera de superficies (core-promise vs labs) vive en `EXPERIMENTAL_FEATURES.md` §Scope Budget (DEF-02/03/07): clasificación por superficie + regla de inversión — admisión labs por defecto, promoción solo con evidencia North Star, congelamiento de labs — y la jerarquía de producto en `VISION.md` — **un núcleo (motor de memoria embebido gobernado) + tres puertas de entrada** (ICP-01 AI-IDEs vía MCP · ICP-02 local-LLM/privacidad · ICP-03 frameworks), decidida por el owner 2026-09-27 (DEF-01; RAG = capacidad del núcleo, no target). Los success criteria de campaña de abajo siguen vigentes para el MVP; los de **producto** son la North Star de DEF-05.

## Open Questions

Ninguna bloqueante del MVP original (FIND-100/106/107 cerradas en campaña — ver avance). Abiertas del cierre-mvp, para DISCOVERY por tarea: fricción del trait en S1 (decide S2, Gate V); semántica TUI (handle de escritura vs límite documentado, FIND-117); productor `submit` S4 y dueño backend S6b (specs 110/113).
