# Antigravity Adapted VantaDB Skills

> ⚠️ **Note:** The legacy skills `vantadb-certify`, `vantadb-audit`, and `vantadb-full-review`
> are now **deprecated**. They are replaced by `unified-review` (`.opencode/skills/unified-review/`).
> See `.opencode/skills/unified-review/SKILL.md` for the replacement.

## 1. unified-review — Review, Audit & Certification (replaces legacy)
* **Ubicación:** `.opencode/skills/unified-review/SKILL.md` (1084 líneas)
* **Propósito:** Review, audit y certificación unificados. Reemplaza las 3 skills legacy. Modos: quick, certify, review, full. Perfiles YAML (default + vantadb).

## 2. vantadb-certify (DEPRECATED — Pre-push Gate)
* **Ubicación:** `.antigravity/skills/vantadb-certify/SKILL.md`
* **Estado:** ⛔ DEPRECATED — reemplazado por `unified-review --mode certify --profile vantadb`
* **Propósito (histórico):** Pre-flight gate con verificación de 8 capas.

## 3. vantadb-audit (DEPRECATED — Auditoría)
* **Ubicación:** `.antigravity/skills/vantadb-audit/SKILL.md`
* **Estado:** ⛔ DEPRECATED — reemplazado por `unified-review --mode full --profile vantadb`
* **Propósito (histórico):** Auditoría multi-fase en paralelo.

## 4. vantadb-full-review (DEPRECATED — Revisión Integral)
* **Ubicación:** `.antigravity/skills/vantadb-full-review/SKILL.md`
* **Estado:** ⛔ DEPRECATED — reemplazado por `unified-review --profile vantadb`
* **Propósito (histórico):** Revisión completa de las 8 capas del proyecto.

## 5. review-deep (Revisión por Módulo)
* **Ubicación:** `.antigravity/skills/review-deep/SKILL.md`
* **Propósito:** Análisis estático detallado por módulo usando CodeGraph y web research.

## 5. progreso (Sincronización de Backlog)
* **Ubicación:** `.antigravity/skills/progreso/SKILL.md`
* **Propósito:** Mantiene la sincronización entre `docs/Backlog.md` y `docs/progreso/README.md`.
