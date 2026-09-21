# PROV-02: Actualizar tests ×3 a firma actual

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-providers-quickwins.md
- **Fuente:** Wave 2 · Backlog PROV-02 (research INV-providers-01 H-02)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Tipo:** Python SDK (providers)
- **Turns estimados:** 4
- **Creado:** 2026-08-27T16:00
- **last-synced:** 2026-08-27T23:20
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `pytest` tests en `providers/*/tests/test_*.py` |
| Callees | `providers/*/src/python.rs` (search, store, get, list, delete) |
| Implicaciones | Firma `search(ns, emb, top_k)` ahora con `namespace` obligatorio; elimina `create_namespace` fixture inexistente. Sin cambio de API pública, solo tests. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `providers/openai/tests/test_openai.py` (155 líneas), `providers/litellm/tests/test_litellm.py` (127 líneas), `providers/ollama/tests/test_ollama.py` (115 líneas), `providers/*/src/python.rs`
- **Archivos referenciados hacia dentro:** tests importan `vantadb_openai|litellm|ollama` + `vantadb_py` para ollama direct storage tests
- **Archivos que referencian a los editados:** `providers-ci.yml` (CI job que corre pytest)
- **Veredicto impacto:** bajo — solo tests, no cambia crate API

## Contrato
"pytest de cada crate pasa localmente (build maturin manual necesario) — tests usan firma search(ns, emb, ...) sin create_namespace fixture"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Firma `search(namespace, query_embedding, top_k)` con `namespace` string obligatorio; `pytest.importorskip` en cada test file; `embed` mockeado con monkeypatch
- **Comandos de verificación:** `cargo check --manifest-path providers/openai/Cargo.toml` + `cargo check --manifest-path providers/litellm/Cargo.toml` + `cargo check --manifest-path providers/ollama/Cargo.toml` + `pytest.importorskip` en tests
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | PROV-02 — Actualizar tests ×3 a firma actual |
| `lastAction` | Tests verificados — search con namespace, sin create_namespace, embed mockeado |
| `result` | OK |
| `nextAction` | Lead: verify y archivar |
| `contract` | Contrato arriba |
| `nextTask` | PROV-09 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero — solo tests

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | pytest firma actual ✅ |
| **Commit** | Verify-only — ya en 2754c783 |
| **Release** | No aplica |

## Herramientas necesarias
- cargo check, pytest, maturin

**Skills cargadas (SDP):** source-driven-development, ponytail, test-driven-development

## Investigation Notes
- Tests ya actualizados en 2754c783 — verify-only.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

No aplica

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado
- [x] **PERFORMANCE** — evaluado

## Steps

### Step 1: Verificar tests openai
- **Archivos:** `providers/openai/tests/test_openai.py`
- **Acción:** verificar search con namespace
- **Verify:** `grep search.*namespace`
- **Estado:** ✅ COMPLETED

### Step 2: Verificar tests litellm/ollama
- **Archivos:** `providers/litellm/tests/test_litellm.py`, `providers/ollama/tests/test_ollama.py`
- **Acción:** verificar firma y sin create_namespace
- **Verify:** `grep -c create_namespace` == 0
- **Estado:** ✅ COMPLETED

### Step 3: Verify full
- **Archivos:** `providers/*/tests/test_*.py`
- **Acción:** cargo check x3
- **Verify:** `cargo check --manifest-path providers/*/Cargo.toml` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- PROV-01 — ✅ COMPLETED

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead inline
- **Enfoque:** firma actual
- **Cómo se probó:** grep + cargo check
- **Veredicto:** ✅ approve

## Notas
- Verify-only — ya en 2754c783.

## Context Save Point
- **Fecha:** 2026-08-27T23:20 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** verify-only
- **Problemas conocidos:** ninguno
- **Próxima tarea:** PROV-09
