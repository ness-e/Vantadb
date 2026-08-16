# FND-17 — API reference automatizada (docs-as-code)

- **Plan:** `docs/plans/2026-08-16-wave-r2-r7-fnd.md` (Task 6, Wave 2)
- **Tipo:** ANÁLISIS (docs + devops) — entregable: análisis + plan, NO implementación forzada
- **Contrato:** "análisis + plan o pipeline entregado (no implementación forzada)"
- **Gate Justificación:** 🟡 prio; análisis read-only; lo primero que evalúa un dev
- **Estado:** ⬜ PENDING

## Pasos

- [x] S1 DISCOVERY — estado actual de generación de API reference en CI (grep workflows, tooling disponible)
- [x] S2 Análisis — qué genera rustdoc (sin deps) vs typedoc/pydoc/mkdocstrings (deps nuevas); estado de docstrings en SDKs
- [x] S3 Decisión — plan de bajo costo o defer justificado; análisis escrito en `docs/Investigaciones/FND-17-api-reference-docs-as-code.md`
- [x] S4 VERIFY — archivo existe, citas archivo:línea verificadas (gate-docs-21.yml:30/62, ci-rust-10.yml:154, Cargo.toml:11, pyproject.toml:41-42, vantadb_py.pyi:1), URLs resuelven (typedoc.org, pdoc.dev, doc.rust-lang.org/rustdoc — GATE CITAS ✅), decisión explícita (§6: plan Fase 1 + defer TS/Python/site)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-python/pyproject.toml` (53L), `vantadb-ts/package.json` (61L), `scripts/validate-docs-coverage.ps1` (187L), `docs/README.md` (52L), `docs/Investigaciones/INV-017-sccache-ci.md` (formato), `.github/workflows/` (grep dirigido — 17 workflows)
- **Referencias hacia dentro:** ninguna — archivo nuevo
- **Referencias entrantes:** ninguna — archivo nuevo en `docs/Investigaciones/`
- **Veredicto:** creación de archivo nuevo, cero impacto en código/workflows. NO se tocan workflows ni Backlog (reglas estrictas de la task).

## Verificación

- Archivo `docs/Investigaciones/FND-17-api-reference-docs-as-code.md` existe
- Citas con `archivo:línea` verificadas contra contenido real
- URLs citadas resueltas (GATE CITAS)
- Decisión explícita: plan (bajo costo) o defer (justificado)