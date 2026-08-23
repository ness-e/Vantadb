# Serie REVIEW — release engineering (2026-08-06)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-08-06 — REVIEW-01: Fix subcomando `cargo semver-checks check` → `check-release`
- **Fuente:** Backlog
- **Fecha:** 2026-08-06
- **Objetivo:** Corregir el subcomando inexistente `check` en gate pre-publish L1
- **Resultado:** ✅ `check-release` aplicado en `unified-review/profiles/vantadb.yml` + `VANTADB-OPERATING-MANUAL.md`
- **Ids:** `REVIEW-01`

### 2026-08-06 — REVIEW-02: Clear stale `--ignore RUSTSEC-2026-0176/0177` audit flags
- **Fuente:** Backlog
- **Fecha:** 2026-08-06
- **Objetivo:** Quitar ignores muertos (advisories ya remediados con pyo3 0.29)
- **Resultado:** ✅ — removido de certify SKILL, unified-review profile (comando + quality gate), pre-push.ps1, docs/ci-rust-10.md. `cargo audit` = 0 advisories activos
- **Ids:** `REVIEW-02`

### 2026-08-06 — REVIEW-03: Verificar política `continue-on-error` en CI
- **Fuente:** Backlog
- **Fecha:** 2026-08-06
- **Objetivo:** Validar los 7 `continue-on-error` en 4 workflows
- **Resultado:** ✅ — los 7 ya tienen `# CATEGORY:` explícito (EXPERIMENTAL ×1, BEST-EFFORT ×3, INFORMATIONAL ×1, NON-CRITICAL ×2) alineado con CI_POLICY y Regla 2. Split best-effort documentado — no requiere gates duros
- **Ids:** `REVIEW-03`

### 2026-08-06 — REVIEW-05: Deps muertas en web/ eliminadas (prismjs + sharp)
- **Fuente:** Backlog
- **Fecha:** 2026-08-06
- **Objetivo:** Limpiar vulnerabilidades npm residuales de `web/`
- **Resultado:** ✅ — `npm audit fix` (6→4) + eliminadas deps muertas `react-syntax-highlighter` (prismjs) y `sharp` (0.34.5). **`npm audit` = 0 vulnerabilities**. `tsc --noEmit` pasa
- **Ids:** `REVIEW-05`

--
