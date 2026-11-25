---
title: "MEM-72: runner LLM degradado visible, no silencioso"
kind: task
status: in-progress
description: "`capabilities` (MCP) y la superficie de observabilidad de vanta-memory exponen runner en uso, resultado del último pass y profundidad de cola; con runner=None y cola no vacía se emite un warn UNA sola vez"
---

# MEM-72: runner LLM degradado visible, no silencioso

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§9.b, ola MCP/MEM)
- **Fuente:** Backlog `MEM-72` — dependencia bloqueante de `MCP-43/44/47`
- **Esfuerzo:** 🟡 2-3h
- **Tipo:** Rust (feature-add: símbolos públicos nuevos en `vanta-memory`, campo aditivo en respuesta MCP)
- **Estado:** ⏳ IN PROGRESS
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory/src/services/scheduler.rs` (`run_pass` ← 6 callers: server WIRE-16, `tests/scheduler.rs`, `tests/scheduler_crash.rs`), `vantadb-server/src/scheduler.rs:108-115` (`ingest_runner_factory` → `build_ingest_runner`), `vantadb-mcp/src/handlers/tools.rs:3151` (`capabilities`) y `vantadb-mcp/src/wiki.rs:293` (wiki ingest runner) |
| Callees | `vanta-memory/src/utils/local_backend.rs` (`queue_depth`, `pending_count`), `core::conversation::now_ms` |
| Implicaciones | Solo aditivo. `run_pass` no cambia firma ni comportamiento (una llamada extra de telemetría al final). `build_runner` mantiene su contrato de configuración (misma salida, sin I/O) y añade UNA nota de observabilidad deliberada — es el único mapeo provider→runner, y "last build wins". La respuesta de `capabilities` gana la clave `ingest`; ninguna clave existente cambia de forma. Sin migración de datos. Sin impacto de memoria (struct fijo, sin crecimiento dependiente del dataset — memory-budget R4). El snapshot es **proceso-local** (la cola vive en `LocalStateBackend`, ADR-0054). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona editada):** `vanta-memory/src/ingest/runner_config.rs` (L1-59, 60-179, 256-293), `vanta-memory/src/ingest/mod.rs` (237L), `vanta-memory/src/services/scheduler.rs` (L1-325), `vanta-memory/src/utils/local_backend.rs` (L186-365), `vantadb-server/src/scheduler.rs` (192L), `vantadb-mcp/src/handlers/tools.rs` (L3130-3199), `vantadb-mcp/tests/embedding_visibility.rs` (L1-60), `docs/api/VANTA_MEMORY.md` (§ scheduler, L330-357), `docs/api/MCP.md` (L415-440).
- **Referencias hacia dentro:** `runner_config::{build_ingest_runner, ConcreteRunner, IngestRunnerCfg, IngestRunnerProvider}` ← `vantadb-server/src/scheduler.rs:37`, `vantadb-mcp/src/wiki.rs:36`, `vantadb-mcp/tests/wiki_ingest_runner.rs:21`; `services::scheduler::{run_pass, spawn_memory_scheduler*}` ← `vantadb-server/src/scheduler.rs:39`, `tests/scheduler.rs`, `tests/scheduler_crash.rs`.
- **Referencias entrantes de lo editado:** `capabilities` ← clientes MCP (contrato de superficie, `docs/api/MCP.md:426`) y `vantadb-mcp/tests/embedding_visibility.rs`.
- **Veredicto impacto:** BAJO-MEDIO. Un módulo nuevo (`ingest::observability`), 3 ediciones mínimas (2 llamadas en scheduler, 1 en `build_runner`), 1 bloque aditivo en `capabilities`. Pre-mortem: (a) warn por pass → mitigado con flag de episodio; (b) confundir "pass saltado" con "runner ausente" → el snapshot separa `runner`, `scheduler_running`, `last_pass.skipped` y `queue`; (c) campos no aditivos → solo se inserta `out["ingest"]`, ningún campo existente se toca (test lo verifica).

## Contrato

```
cargo nextest run --profile audit -p vanta-memory -E 'test(scheduler) or test(runner) or test(pass)' --build-jobs 2
cargo nextest run --profile audit -p vantadb-mcp -E 'test(capabilities)' --build-jobs 2
```
más un test que demuestre la distinción: con runner `None` y cola no vacía, `capabilities` reporta el estado degradado de forma legible y el aviso se emite **una sola vez**.

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde vive el estado | A) registro proceso-global `OnceLock<Mutex<..>>` en `ingest::observability` (lo lee el mismo proceso que hospeda el scheduler; MCP-43 lo usará) / B) `Arc` pasado desde el host (MCP no tiene handle del scheduler del server) | ✅ A — evidencia: ADR-0054 "one loop per process; read from the same process" |
| 2 | Quién conoce el provider | A) `IngestRunnerCfg::build_runner()` anota el runner construido (único punto que mapea provider→`ConcreteRunner`) / B) el loop del scheduler (genérico sobre `R: LlmRunner`, no puede distinguir Ollama de OpenAI) | ✅ A — evidencia: `runner_config.rs:259-270` es el único mapeo |
| 3 | Qué significa "degradado" | A) `runner == none` (no extrae nunca) / B) `last_pass.failed > 0` (fallo transitorio) | ✅ A + cola: `none` con cola 0 es normal; con cola > 0 es pipeline roto |
| 4 | Frecuencia del aviso | A) una vez por **episodio** (runner `none` && cola > 0), reseteado al drenar la cola o al configurar runner / B) una vez por pass | ✅ A — pre-mortem (a): un warn por pass inunda el log de cada host |
| 5 | Contador observable del aviso | A) `degraded_warnings: u32` en el snapshot (verificable sin subscriber de `tracing`) / B) solo el log (test requiere dev-dep `tracing-subscriber`) | ✅ A — evidencia: `vanta-memory/Cargo.toml` no tiene `tracing-subscriber` y añadirlo sale del scope de escritura |
| 6 | Forma en `capabilities` | A) clave nueva `ingest` (aditiva) / B) reestructurar `Capabilities` del core | ✅ A — `src/sdk/**` es fuera de scope y el contrato exige campos aditivos |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | `ingest::observability` (estado + `note_runner` / `note_pass` / `note_skipped_pass` / `status` / `notice`) | ✅ |
| 2 | Wiring: `build_runner` anota runner; `run_pass` anota pass; loop anota `scheduler_running` + skip degradado | ✅ |
| 3 | `capabilities` (MCP) expone `ingest` aditivo | ✅ |
| 4 | Tests: `vanta-memory/tests/ingest_observability.rs` + `vantadb-mcp/tests/capabilities_ingest.rs` | ✅ |
| 5 | Docs: `docs/api/VANTA_MEMORY.md` + `docs/api/MCP.md` | ✅ |
| 6 | Verify contrato + OCR review + review P2-01 + commit local | ✅ |

## Verification

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vanta-memory -E 'test(scheduler) or test(runner) or test(pass)' --build-jobs 2` | ✅ 44/44 pass (704 skipped) |
| `cargo nextest run --profile audit -p vantadb-mcp -E 'test(capabilities)' --build-jobs 2` | ✅ 2/2 pass (190 skipped) |
| `cargo clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings` | ✅ limpio |
| `rustfmt --check` (archivos de la tarea) | ✅ limpio |
| `node scripts/docs/check-links.mjs` / `check-docs.mjs` | ✅ 0 broken links; sin violaciones nuevas |
| `pwsh dev-tools/ocr-review.ps1` + `ocr delegate rule <4 archivos .rs>` | ✅ aplicado (sin Critical/High propios) |

## Resultado

El estado degradado dejó de ser invisible: `ingest::observability::status()` publica
`runner` (`unknown|none|ollama|openai`), `degraded`, `scheduler_running`,
`queue{depth,claimed}`, `last_pass{skipped,processed,failed,timers_fired}`,
`degraded_warnings` y un `notice` legible; el tool MCP `capabilities` lo embebe
bajo la clave aditiva `ingest`. La distinción es **estado + cola**: `none` con
`depth 0` es un pipeline ocioso (`is_broken() == false`), `none` con `depth > 0`
es uno roto, y en ese caso se emite **un** `warn!` por episodio (nunca por pass) —
`degraded_warnings` lo cuenta, que es lo que verifica el test.


## Invariantes de dominio (handoff — MUST)

- **Principio 4:** el pass degradado sigue sin tocar la cola (nada se pierde, nada se quema). La telemetría no altera el flujo.
- **Una vez por episodio:** el aviso no puede repetirse por pass.
- **Aditivo:** la respuesta `capabilities` solo gana `ingest`; ninguna clave previa cambia.
- **`unwrap`/`expect` prohibido** en código nuevo (`Anyhow`/`Result`); el mutex se recupera del poison como `local_backend.rs:52-58`.
- **Sin dependencias nuevas.**

## Deuda

- El contador de avisos es la evidencia mecánica del "una vez"; la captura del nivel de log exigiría `tracing-subscriber` como dev-dep de `vanta-memory` (fuera del scope de escritura). Rebajar el `warn!` a `debug!` dejaría el test verde — deuda declarada, no silenciada.
- `note_skipped_pass` infiere la causa del efecto (un pass saltado ⇒ no hay runner) y lo escribe en el estado; hoy es cierto para todos los hosts (el factory de `vantadb-server` sólo devuelve `None` cuando no hay engine), pero un host con `|| None` propio vería el aviso genérico "the host built no runner".
- El proceso MCP aún no hospeda el scheduler (`MCP-43`): hasta entonces `capabilities` reportará `scheduler_running: false` y `last_pass: null` — estado honesto, no degradación. La cola del writer no es observable desde el proceso MCP (cola in-process).

## Review (P2-01, adversarial — `vanta-review`, contexto fresco)

Veredicto inicial `changes-required`; 1 bloqueante + 3 Medium, todos corregidos en esta ronda:

| Hallazgo | Fix |
|---|---|
| **[High]** `scheduler_running` era write-once-true (un loop parado seguía reportándose vivo) | `note_scheduler_stopped()` + llamada al salir del loop y en `Drop`; el test cubre el ciclo start→stop |
| **[Medium]** El test decía "uno solo" y declaraba dos (raza real bajo `cargo test` no aislado) | Un único `#[test]` con 6 fases (episodio, drenaje, runner real, ciclo del loop, pass real) |
| **[Medium]** `build_runner` documentado como "pure config mapping" con efecto lateral | Doc comment declara el efecto, el porqué y el "last build wins" (wiki ingest per-call) |
| **[Medium]** El shape MCP no explica que la cola es proceso-local | Una frase en `VANTA_MEMORY.md` y en `MCP.md`; el ejemplo ya no sugiere ver la cola del writer |

