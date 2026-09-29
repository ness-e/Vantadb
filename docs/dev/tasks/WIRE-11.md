---
title: "WIRE-11: `llm-driver` always-on en consumidores shipped (MCP/proxy)"
kind: task
description: "VANTADBINGESTPROVIDER=ollama|openai funciona o falla con mensaje claro (nunca no-op silencioso); cargo check/test -p vantadb-mcp -p vanta-proxy verdes.\""
---

# WIRE-11: `llm-driver` always-on en consumidores shipped (MCP/proxy)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-post-investigacion-integral.md` (P56; C11/A13)
- **Fuente:** Backlog `P56` (fila WIRE-11, creada 2026-09-25)
- **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🟠 · **Tipo:** Rust config/build (`fix:`)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-25 · **last-synced:** 2026-09-25
- **Estado:** ✅ COMPLETED (P2-01 vanta-review approve; lead verify: honest-test 1/1 con feature + commit limpio 4 archivos)
- **Incógnitas (uphill):** 0 (diseño resuelto P/D 2026-09-25: always-on) · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `StandaloneLlmRunner` ← `vanta-memory/src/ingest/runner_config.rs` (2 callers); `IngestRunnerProvider` parsea ollama/openai/local |
| Callees | `vanta-memory` feature `llm-driver = ["dep:reqwest"]` (`vanta-memory/Cargo.toml:39`); sin ella `run` → `NotConfigured` (`adapters/standalone/llm_runner.rs:106-111`) |
| Implicaciones | Solo 2 líneas Cargo (`vantadb-mcp`, `vanta-proxy` → `features = ["llm-driver"]`); verificar unificación reqwest (MCP ya trae reqwest 0.12 sin defaults; proxy igual) + default-features de vanta-memory; test `NotConfigured` default intacto |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `vantadb-mcp/Cargo.toml:10-27`, `vanta-proxy/Cargo.toml:29-30`, `vanta-memory/Cargo.toml:15-45` (features), `vanta-memory/src/adapters/standalone/llm_runner.rs:100-120,190-220`, `vanta-memory/src/ingest/runner_config.rs:31-70`
- **Referencias hacia dentro:** `IngestRunnerProvider::parse` (unknown → warn + local); `LlmRunner` trait
- **Referencias entrantes:** `wiki_ingest` (MCP) → worker vanta-memory; ingest G2
- **Veredicto impacto:** bajo — 2 líneas de manifiesto + verificación de unificación de features; cero cambios de código esperados

## Contrato
"`VANTADB_INGEST_PROVIDER=ollama|openai` funciona o falla con mensaje claro (nunca no-op silencioso); `cargo check/test -p vantadb-mcp -p vanta-proxy` verdes."

## Spec (SDD — Phase 1b)
No es feature-add: flags sobre dependencia existente, cero símbolos públicos nuevos. Diseño resuelto P/D owner 2026-09-25: **always-on** (descartados forward opt-in y solo-error).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** degradación P4 documentada intacta (sin feature → NotConfigured + store-all); test `llm_free_mode_reports_not_configured` verde; sin conflicto TLS — dual-stack rustls+default-tls explícito (patrón workspace existente `Cargo.toml:94` src/llm.rs; `vanta-memory/Cargo.toml:21` sin `default-features=false`; P2-01 2026-09-26: "rustls-only" como redactado era falso, se corrige aquí).
- **Comandos de verificación:** `cargo check -p vantadb-mcp -p vanta-proxy` + suites relevantes + `cargo tree -p vantadb-mcp -i reqwest` (unificación sana)
- **Deuda pendiente:** ninguna al abrir

## Recitation
```
=== RECITATION ===
Objetivo activo: WIRE-11 — llm-driver always-on MCP/proxy
Estado: plan (desde: —)
Última acción: task file creado; diseño always-on; causa verificada (0 habilitadores)
Resultado: ⬜
Próxima acción: Step 1 — Cargo MCP + check
Contrato: ver ## Contrato
Invariantes: degradación P4 intacta; test NotConfigured verde; TLS rustls-only
Deuda: ninguna
Próxima tarea si completa: EXE-01
last-synced: 2026-09-25
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (2 líneas; elimina no-op silencioso).

## Definition of Done (3 niveles)
- **Task:** contrato + clippy/fmt + `cargo tree` sin duplicación reqwest conflictiva
- **Commit:** atómico, `fix:` + WIRE-11, verify mecánico antes
- **Release:** N/A (sin versionado)

## Herramientas necesarias
- `cargo check/test/tree`, `campaign_verify_cmd`
- **Skills cargadas (SDP):** source-driven-development (base) + systematic-debugging (causa verificada: 0 habilitadores) + test-driven-development (comportamiento honesto testeado) + incremental-implementation (1 consumidor por vez)

## Investigation Notes
- Confirmado 2026-09-25: solo `vanta-memory/Cargo.toml:39` define la feature; MCP (`:14`, features propias solo embed-local/remote-inference) y proxy (`:30` default-features=false) no la habilitan.
- `run` sin feature → `NotConfigured` (`llm_runner.rs:106-111`); test `:218` lo aserta en default.
- `runner_config.rs:39-41`: HTTP/HTTPS vía Standalone + llm-driver ("S2 wires G2").

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes | 4 steps |
| % completado | 0% |

## Fase 1 — Evidencia de Debugging (GATE Bug)
- **Repro:** `VANTADB_INGEST_PROVIDER=ollama` en binario MCP shipped → ingesta procede como local sin aviso.
- **Hipótesis:** feature nunca habilitada en consumidores → runner NotConfigured tragado por degradación. Confirmada por grep (0 habilitadores).
- **1 variable:** un consumidor por step (MCP → proxy).
- **Test:** provider remoto funciona o falla con mensaje claro (nunca silencioso).

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — N/A: sin input externo nuevo; reqwest ya es dependencia (misma versión 0.12); no relajar TLS.
- [x] **PERFORMANCE** — N/A justificado: cambio de compilación, no de hot path. (Bloat binario aceptado por decisión always-on.)

## Steps
### Step 1: MCP always-on
- **Archivos:** `vantadb-mcp/Cargo.toml:14`
- **Acción:** `features = ["llm-driver"]` en dep vanta-memory (respetando features propias existentes)
- **Verify:** `cargo check -p vantadb-mcp` ✅ (54s) + `cargo test -p vantadb-mcp --test wiki_ingest_runner` ✅ 5/5 (incl. `ingest_ollama_down_degrades` → ahora Transport honesto). Nota: `--test mcp_tests` full bajo `cargo test` tiene fallos paralelos PRE-EXISTENTES (verificado vía stash: 47 fallos sin mi cambio; binario excluido del perfil nextest default).
- **Estado:** ✅ COMPLETED

### Step 2: Proxy always-on
- **Archivos:** `vanta-proxy/Cargo.toml:30`
- **Acción:** `features = ["llm-driver"]` (mantener `default-features = false` si vanta-memory tiene defaults que no queremos)
- **Verify:** `cargo check -p vanta-proxy` ✅ + `cargo test -p vanta-proxy` lib ✅ 171/171; `api05_snapshot_auth` 2 fallos PRE-EXISTENTES (verificado vía stash sin mi cambio).
- **Estado:** ✅ COMPLETED

### Step 3: Comportamiento honesto e2e
- **Archivos:** `vanta-memory/src/adapters/standalone/llm_runner.rs` (tests) — gate `#[cfg(not(feature))]` al test default-only para matriz verde en ambos combos
- **Acción:** test nuevo `llm_driver_fails_loud_on_unreachable_endpoint` (cfg llm-driver): endpoint cerrado → Transport/Timeout con mensaje no vacío, nunca NotConfigured/Ok. Docs usuario: N/A (`VANTADB_INGEST_PROVIDER` solo existe en dev-docs).
- **Verify:** `--features llm-driver --lib llm_runner` ✅ 7/7; default `--lib llm_runner` ✅ 7/7 (P4 intacta); `cargo tree -p vantadb-mcp/-p vanta-proxy -i reqwest` → versión única v0.12.28, sin duplicación; dual TLS (rustls + default-tls) = patrón workspace existente (`Cargo.toml:94` src/llm.rs), sin conflicto → NO STOP.
- **Estado:** ✅ COMPLETED

### Step 4: Cierre
- **Archivos:** —
- **Acción:** fmt+clippy, `verify_changed.ps1`, commit `fix:`, progreso
- **Verify:** `campaign_verify_cmd` con el contrato
- **Estado:** ✅ COMPLETED (fmt scope limpio vía rustfmt single-file; clippy scope 0 warnings propios; P2-01 APPROVE registrado; commit atómico solo-scope, sin push)

## Dependencias
- Ninguna. Siguiente: EXE-01.

## Review (GATE P2-01)
- **Revisor:** vanta-review — distinto del implementador (sesión `ses_f245a6f64ffeR7gKDS6D5n77P6`, 2026-09-26)
- **Enfoque:** ¿unificación reqwest sana? ¿comportamiento honesto verificado?
- **Cómo se probó:** re-ejecutó comandos exactos — llm_runner 7/7 ambos combos, wiki_ingest_runner 5/5, tree reqwest v0.12.28 única en ambos consumidores, rustfmt limpio, clippy vanta-memory 0 warnings
- **Checklist anti-hábitos:** según plantilla
- **Veredicto:** ✅ APPROVE — sin critical/required; 2 opcionales (🟡-1 invariante TLS reescrito en este file; 🟡-2 assert endurecido a `!NotConfigured`+mensaje en test nuevo) + nota higiene (29 dirty ajenos; commit solo scope WIRE-11)

## Notas
- WIP ajeno PROHIBIDO: `src/sdk/types/`, `QueryResult` — esta tarea no los toca.
- Si `cargo tree` muestra conflicto TLS (rustls vs default), STOP y escalar al lead antes de forzar features.
