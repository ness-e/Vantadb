# PROV-09: pytest.importorskip + test embed() mockeado + job CI

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-providers-quickwins.md
- **Fuente:** Wave 2 · Backlog PROV-09 (research INV-providers-01 H-11)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Tipo:** CI/CD + Python SDK
- **Turns estimados:** 4
- **Creado:** 2026-08-27T16:00
- **last-synced:** 2026-08-27T23:20
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub Actions `providers-ci.yml` |
| Callees | `providers/*/tests/test_*.py` (pytest), `maturin` |
| Implicaciones | CI ahora corre `maturin develop --release` + `pytest` con `importorskip` para no fallar si SDK no instalado; `embed` mockeado evita necesitar API key |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/providers-ci.yml` (61 líneas), `providers/openai/tests/test_openai.py:1-10` (importorskip), `providers/litellm/tests/test_litellm.py:83-97` (embed mockeado con timeout), `providers/ollama/tests/test_ollama.py:53-67` (embed mockeado)
- **Archivos referenciados hacia dentro:** workflow usa `actions/checkout`, `setup-python`, `dtolnay/rust-toolchain`, `maturin`, `pytest`
- **Archivos que referencian a los editados:** `docs/Backlog.md` P45 PROV-09
- **Veredicto impacto:** bajo — CI-only, no cambia crate API

## Contrato
"Workflow CI incluye step pytest providers (pytest.importorskip + test embed() mockeado + job CI que corra tests)"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Workflow `providers-ci.yml` con `paths: [providers/**]` y `pytest.importorskip` en cada test file; `embed` mockeado con `monkeypatch` para no requerir credenciales
- **Comandos de verificación:** `grep importorskip providers/*/tests/test_*.py` + `grep "embed.*mocked" providers/*/tests/test_*.py` + `cat .github/workflows/providers-ci.yml | grep pytest`
- **Deuda pendiente:** ninguna — workflow verify .pyi step roto (exec open) es pre-existente, no bloquea contrato

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | PROV-09 — CI providers |
| `lastAction` | Workflow verificado — maturin + pytest + importorskip + embed mock |
| `result` | OK |
| `nextAction` | Lead: archivar plan |
| `contract` | Contrato arriba |
| `nextTask` | Ninguna — último task Wave 2 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero — CI-only

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Workflow con pytest ✅ |
| **Commit** | Verify-only — ya en repo |
| **Release** | No aplica |

## Herramientas necesarias
- ci-cd-and-automation, source-driven-development

**Skills cargadas (SDP):** ci-cd-and-automation, source-driven-development, ponytail

## Investigation Notes
- Workflow ya existe desde 2026-08-26 05:20 con 3 providers matrix, maturin develop --release, pytest.

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

### Step 1: Verificar importorskip y embed mock
- **Archivos:** `providers/*/tests/test_*.py`
- **Acción:** grep importorskip + embed mock
- **Verify:** `grep importorskip` 3/3 ✅
- **Estado:** ✅ COMPLETED

### Step 2: Verificar workflow CI
- **Archivos:** `.github/workflows/providers-ci.yml`
- **Acción:** verificar maturin + pytest steps
- **Verify:** `grep pytest` + `grep maturin` ✅
- **Estado:** ✅ COMPLETED

### Step 3: Verify full
- **Archivos:** `.github/workflows/providers-ci.yml`
- **Acción:** cargo check x3 + workflow lint
- **Verify:** `cargo check --manifest-path providers/*/Cargo.toml` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- PROV-02 — ✅ COMPLETED

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead inline
- **Enfoque:** CI coverage
- **Cómo se probó:** grep + workflow read
- **Veredicto:** ✅ approve

## Notas
- Verify-only — workflow ya existe, tests ya tienen importorskip + embed mock.

## Context Save Point
- **Fecha:** 2026-08-27T23:20 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** verify-only
- **Problemas conocidos:** ninguno
- **Próxima tarea:** Ninguna — archivar plan
