# ECO/GH-143 — limpieza hooks y sccache

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-28 — ECO-001: Eliminar hooks muertos de Claude Code

**Objetivo:** Eliminar hooks muertos de Claude Code (`.opencode/hooks/hooks.json` y `session-start.sh`) que nunca se ejecutan en OpenCode/Windows.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `ECO-001` | Remove dead Claude Code hooks (C1) | ✅ COMPLETADO | Commit `cf623e5c`. Archivos eliminados de `.opencode/hooks/`. Blast radius: cero. |

**Verificación:** `Test-Path ".opencode/hooks"` → False ✅ | `git log --oneline -1` → `cf623e5c chore: remove dead Claude Code hooks (ECO-001)` ✅

### 2026-07-28 — ECO-002: Corregir contradicción de --no-verify en AGENTS.md

**Objetivo:** Eliminar contradicción en AGENTS.md donde Regla 1 prohibía `--no-verify` pero Regla 7 lo autorizaba para cambios triviales.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `ECO-002` | Corregir contradicción de --no-verify en AGENTS.md (C3) | ✅ COMPLETADO | Regla B (línea 967 original) ya eliminada. Solo queda Regla A (`grep --no-verify .opencode/AGENTS.md` → 1 match: prohibición en línea 791). `.antigravity/AGENTS.md` idéntico. |

**Verificación:** `grep "trivial.*CI\|no-verify.*trivial" .opencode/AGENTS.md` → 0 coincidencias ✅ | `grep "trivial.*CI\|no-verify.*trivial" .antigravity/AGENTS.md` → 0 coincidencias ✅

<!-- movido a ARCHIVO_HISTORICO.md -->

### 2026-08-02 — GH-143: Acelerar CI con sccache y paralelización

**Objetivo:** CI ≥20% más rápido. Habilitar sccache para cachear compilación Rust y eliminar bottleneck de `cargo install cargo-nextest` en Windows.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `GH-143` | Acelerar CI con sccache y paralelización | ✅ COMPLETADO | Commits `44404c7d` (docs INV-017 + drift AGENTS.md), `1f9f5c41` (sccache + fix install de nextest). sccache vía `mozilla-actions/sccache-action@v0.0.11` en `.github/actions/rust-setup/action.yml`; nextest Windows vía `taiki-e/install-action` en `ci-rust-10.yml`. Run 30737269105: 15/15 jobs pasan; Tests (Windows) 14m29s → 8m35s (−40.7%). Issue #143 cerrado. |

**Verificación:** `gh run view 30737269105` → success ✅ | job test-windows 515s vs baseline 869s (−40.7%) ✅ | actionlint + pyyaml OK ✅
