> **ENTRY POINT — Audit Command (router)**
> El agente DEBE leer este archivo cuando el usuario envía un mensaje que empieza con `/audit`.
>
> **CONSOLIDACIÓN 2026-08-25:** `/audit` ya NO tiene implementación propia de fases.
> Toda la lógica de auditoría vive en la skill `unified-review` (`.opencode/skills/unified-review/`,
> perfil `profiles/vantadb.yml`). Este comando es un ROUTER: mapea el modo, invoca la
> skill, y aplica el contrato de salida (reporte + INDEX + backlog). Antes había aquí una
> implementación paralela de 9 fases — eliminada por divergir de unified-review.

Cargá las skills `progreso`, `ponytail` (full), luego `unified-review`.

## Router: modo según el argumento

| Invocación | unified-review | Uso |
|------------|----------------|-----|
| `/audit quick` | `--mode quick --profile vantadb` | Gate mecánico (~2min): fmt/clippy/test/deny |
| `/audit certify` | `--mode certify --profile vantadb` | Pre-push/merge gate secuencial, hard stop al primer error |
| `/audit review` | `--mode review --profile vantadb` | Deep review + code review sin CLI pesado |
| `/audit` o `/audit full` | `--mode full --profile vantadb` | Pipeline completo con scoring ISO |

Si el usuario pasa `--profile <nombre>`, úsalo en lugar de `vantadb`.

## Ejecución

1. Cargá la skill `unified-review` y ejecutá su flujo para el modo elegido
   (la skill define las fases, waves y sub-agentes — este archivo no los duplica).
2. Pre-check estándar: si no hay diff (`git diff --name-only HEAD` vacío), usá
   `git diff --name-only HEAD~1` como scope del review.
3. En modo `certify`: ejecución secuencial con hard stop al primer fallo
   ("❌ LAYER N FAILED — abortando" / "✅ CERTIFY PASSED — safe to push").

## Contrato de salida (post-ejecución, obligatorio)

1. **Reporte:** `docs/reviews/audit-<modo>-<YYYYMMDD>-<HHMMSS>.md` (naming
   zero-padded, igual que unified-review) con scoreboard por fase, findings
   priorizados (Critical/Important/Suggestion con file:line + fix), FODA y veredicto.
2. **Estado:** `docs/last-audit-state.json`:
   `{timestamp, mode, veredicto: PASS|FAIL, findings_critical, report_file}`
3. **INDEX:** fila en `docs/reports/INDEX.md`; si un audit previo del mismo modo
   queda superado, marcá el anterior `superado`.
4. **Backlog:** hallazgos ≥ medium → filas **FIND-\*** en `docs/Backlog.md`
   sección `## Hallazgos pendientes de reportes` (esquema único, fuente canónica
   `prompts/findings.md`; NO crear prefijos AUD-/REVIEW- nuevos). Cada fila con
   `Origen: <este reporte>`.

## Mensaje final

- PASS: "✅ AUDIT PASSED (<modo>)" + resumen de findings
- FAIL: "❌ AUDIT FAILED — fix errors above before shipping"
