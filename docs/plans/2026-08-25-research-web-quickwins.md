# Plan: Research web — quick wins + media inversión (WEB-03..09)

> **Origen:** INV-web-01 (`/research web` 2026-08-25) · Informe: `docs/reviews/research-web-prod-20260825.md`
> **Alcance:** 7 hallazgos APLICAR decididos por HITL. WEB-01/02 quedan en Backlog P43 (no en este plan).
> **Módulo:** `web/**` · Verificación por tarea: `npm run build` exit 0 + `npm run lint` exit 0 (contrato del módulo).

## Wave 1 — Quick wins (<1 día total)

| # | Tarea | Backlog | Contrato de verificación | Estado |
|---|---|---|---|---|
| 1 | Restaurar o eliminar refs a `mascota_gato.png`/`avatar_gato.png` (decidir: si los assets no existen en ningún lado, quitar refs y fallbacks) | WEB-03 | grep `mascota_gato\|avatar_gato` en `web/src` → 0 refs muertas (archivos ya en `public/assets/`) | ✅ Done (verify-only: 0 refs, assets en `public/assets/`) |
| 2 | Unificar idioma de metadata en layouts (`about/*`, playground): title+description+openGraph mismo locale | WEB-04 | grep manual de los 5 layouts tocados; build exit 0 | ✅ Done (5 layouts ES consistente — title === openGraph.title, description === openGraph.description ES; `npm run build` 36/36 ✅, `npm run lint` 0 errors 7 warnings ✅ — 2026-09-01) |
| 3 | Re-medir Lighthouse `/` y una ruta interna post-WDA-05; actualizar nota de perf en registro (`research-modules.md` fila web) y `web/AGENTS.md` | WEB-05 | Números nuevos citados con comando + fecha en ambos docs; si EPERM persiste, documentar workaround probado | ⬜ Pendiente |
| 4 | Bloque instalación copiable arriba del fold (home) o ancla `#quickstart` prominente en `/docs` — NO recrear ruta `/quickstart` | WEB-06 | Comando visible sin scroll en viewport 1440×900; copy button funcional | ⬜ Pendiente |

## Wave 2 — Media inversión

| # | Tarea | Backlog | Contrato de verificación |
|---|---|---|---|
| 5 | Sandbox iframe para el playground (`new Function` self-XSS) o decisión documentada de por qué no antes de exposición pública | WEB-07 | Ejecución de código del playground dentro de iframe sandbox (`allow-scripts`); doc inline actualizada |
| 6 | Specs Playwright E2E del flujo crítico landing→docs→playground (1 spec mínimo, patrón desktop/e2e) | WEB-08 | Spec corre verde local (`npx playwright test`); registrado en CI o documentado comando local |
| 7 | Densidad efectos home: propuesta de reducción (trust-bar ×11 → ≤3 efectos, hero 5 capas → ≤2) **requiere aprobación visual del owner antes de merge** | WEB-09 | Diff muestra reducción neta de efectos animados; screenshot before/after adjunto al task file |

## Notas

- Orden recomendado: Wave 1 completa → Wave 2. Tasks 5-7 independientes entre sí.
- WEB-09 tiene gate humano explícito: no mergear sin OK visual.
- Al completar: `skill progreso` Trigger 1 (filas fuera del Backlog, registro en `docs/avance/activo/web-frontend.md`) + archivar plan con su `.budget.json`.
