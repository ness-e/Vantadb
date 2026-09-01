# Plan: Research web — quick wins + media inversión (WEB-03..09)

> **Origen:** INV-web-01 (`/research web` 2026-08-25) · Informe: `docs/reviews/research-web-prod-20260825.md`
> **Alcance:** 7 hallazgos APLICAR decididos por HITL. WEB-01/02 quedan en Backlog P43 (no en este plan).
> **Módulo:** `web/**` · Verificación por tarea: `npm run build` exit 0 + `npm run lint` exit 0 (contrato del módulo).

## Wave 1 — Quick wins (<1 día total)

| # | Tarea | Backlog | Contrato de verificación | Estado |
|---|---|---|---|---|
| 1 | Restaurar o eliminar refs a `mascota_gato.png`/`avatar_gato.png` (decidir: si los assets no existen en ningún lado, quitar refs y fallbacks) | WEB-03 | grep `mascota_gato\|avatar_gato` en `web/src` → 0 refs muertas (archivos ya en `public/assets/`) | ✅ Done (verify-only: 0 refs, assets en `public/assets/`) |
| 2 | Unificar idioma de metadata en layouts (`about/*`, playground): title+description+openGraph mismo locale | WEB-04 | grep manual de los 5 layouts tocados; build exit 0 | ✅ Done (5 layouts ES consistente — title === openGraph.title, description === openGraph.description ES; `npm run build` 36/36 ✅, `npm run lint` 0 errors 7 warnings ✅ — 2026-09-01) |
| 3 | Re-medir Lighthouse `/` y una ruta interna post-WDA-05; actualizar nota de perf en registro (`research-modules.md` fila web) y `web/AGENTS.md` | WEB-05 | Números nuevos citados con comando + fecha en ambos docs; si EPERM persiste, documentar workaround probado | ✅ Done (2026-09-01 — `/` perf 99/a11y96/bp96/seo100 LCP1800 CLS0 TBT0, `/docs` perf 98/a11y94/bp96/seo100 LCP1680 — lighthouse 13.4.1, Chrome152, `npx lighthouse http://localhost:3000 --chrome-flags="--no-sandbox --headless --disable-gpu"` + EPERM no reprodujo, workaround --no-sandbox probado) |
| 4 | Bloque instalación copiable arriba del fold (home) o ancla `#quickstart` prominente en `/docs` — NO recrear ruta `/quickstart` | WEB-06 | Comando visible sin scroll en viewport 1440×900; copy button funcional | ✅ Done (2026-09-01 — hero `id="quickstart"` y=534 <900 + `quick-install.tsx` 112L + header docs `quickstart-header` bar + copy funcional + `npm run build` 36/36 ✅ `npm run lint` 0e7w ✅, 404 /quickstart, Playwright 1440×900 PASS) |

## Wave 2 — Media inversión

| # | Tarea | Backlog | Contrato de verificación | Estado |
|---|---|---|---|---|
| 5 | Sandbox iframe para el playground (`new Function` self-XSS) o decisión documentada de por qué no antes de exposición pública | WEB-07 | Ejecución de código del playground dentro de iframe sandbox (`allow-scripts`); doc inline actualizada | ✅ Done (2026-08-27 — `playground-executor.tsx` + `public/playground-executor.html` iframe `allow-scripts allow-same-origin`, `postMessage` aislado) |
| 6 | Specs Playwright E2E del flujo crítico landing→docs→playground (1 spec mínimo, patrón desktop/e2e) | WEB-08 | Spec corre verde local (`npx playwright test`); registrado en CI o documentado comando local | ✅ Done (2026-09-01 — `e2e/flujo-critico.spec.ts` 32L + `playwright.config.ts` 36L, `npx playwright test` 1 test 3.1s ✅, `npm run build` 36/36 ✅, `npx tsc --noEmit` ✅, comando documentado en `web/AGENTS.md` §E2E Guard + `playwright.config.ts` header; CI web build+lint, e2e local) |
| 7 | Densidad efectos home: propuesta de reducción (trust-bar ×11 → ≤3 efectos, hero 5 capas → ≤2) **requiere aprobación visual del owner antes de merge** | WEB-09 | Diff muestra reducción neta de efectos animados; screenshot before/after adjunto al task file | ⬜ Pending (gate visual owner) |

## Notas

- Orden recomendado: Wave 1 completa → Wave 2. Tasks 5-7 independientes entre sí.
- WEB-09 tiene gate humano explícito: no mergear sin OK visual.
- Al completar: `skill progreso` Trigger 1 (filas fuera del Backlog, registro en `docs/avance/activo/web-frontend.md`) + archivar plan con su `.budget.json`.

=== RECITATION WEB-05 ===
Campaign ID: 7adc9e28-c942-4641-bf09-752310a70b80
Objetivo activo: Re-medir Lighthouse `/` y `/docs` post-WDA-05; actualizar web/AGENTS.md y research-modules.md fila web con números + comando + fecha
Estado: completed
Última acción: Lighthouse local 2026-09-01 — `/` perf99/a11y96/bp96/seo100, `/docs` perf98/a11y94/bp96/seo100 — ambos docs actualizados + build/lint verdes + plan file Done
Resultado: ✅
Próxima acción: ninguno — WEB-05 completo, siguiente WEB-06 si aplica
Contrato: Números nuevos citados con comando + fecha en ambos docs; si EPERM persiste documentar workaround | verificacion: web/AGENTS.md tabla 99/98 + comando + fecha 2026-09-01 + entorno Chrome152/lighthouse13.4.1; research-modules.md fila web perf99/98; build 36/36 exit0; lint 0e7w exit0; EPERM no reprodujo --no-sandbox OK fallback prod 200 | artefactos: web/AGENTS.md, .opencode/references/research-modules.md, web/lighthouse-report.json, web/lighthouse-report-docs.json | invariantes: no tocar WEB-09 sin gate visual | deuda: ninguna
Próxima tarea si completa: WEB-06
=== END RECITATION ===
=== RECITATION WEB-06 ===
Campaign ID: 7adc9e28-c942-4641-bf09-752310a70b80
Objetivo activo: Bloque instalación copiable arriba del fold (home) o ancla #quickstart prominente en /docs — NO recrear ruta /quickstart
Estado: completed
Última acción: quick-install.tsx 112L + hero id="quickstart" y=534 + docs header bar quickstart-header + build/lint verdes + viewport 1440×900 PASS
Resultado: ✅
Próxima acción: ninguno — WEB-06 completo, siguiente WEB-07 (Wave 2) si aplica
Contrato: Comando visible sin scroll en viewport 1440×900; copy button funcional | verificacion: hero [data-testid="quick-install"] y=534 h=33 bottom 567 <900 PASS (Playwright 1440×900 chromium --no-sandbox); copy button visible+click ok; docs #quickstart exists 1 + header bar 1; GET /quickstart 404 PASS; build 36/36 exit0; lint 0e7w exit0 | artefactos: web/src/components/vanta/quick-install.tsx, web/src/components/vanta/hero.tsx (id quickstart), web/src/components/vanta/docs-view.tsx (QuickInstall bar), web/src/app/quickstart NO existe | invariantes: no recrear /quickstart, no tocar WEB-09 sin gate visual, R-FE-4 light-only, R-FE-5 touch 44px | deuda: ninguna
Próxima tarea si completa: WEB-07
=== END RECITATION ===
=== RECITATION WEB-08 ===
Campaign ID: 7adc9e28-c942-4641-bf09-752310a70b80
Objetivo activo: Specs Playwright E2E del flujo crítico landing→docs→playground (1 spec mínimo, patrón desktop/e2e)
Estado: completed
Última acción: fix playground-executor.html fetch+eval wasm_bindgen (let→window) + playground-executor.tsx ping retry + code-playground.tsx wait ready 5s; npx playwright test 1 test 3.1s PASS; build 36/36, tsc, lint 0e4w
Resultado: ✅
Próxima acción: ninguno — WEB-08 completo, siguiente WEB-09 (gate visual owner) si aplica
Contrato: Spec corre verde local (npx playwright test); registrado en CI o documentado comando local | verificacion: e2e/flujo-critico.spec.ts 32L 1 test PASS 3.1s (chromium, webServer npm run dev localhost:3000); playwright.config.ts 36L testDir e2e workers1 localhost; build 36/36 exit0; tsc --noEmit exit0; lint 0e4w exit0; comando documentado web/AGENTS.md §E2E Guard + playwright.config.ts header | artefactos: web/e2e/flujo-critico.spec.ts, web/playwright.config.ts, web/public/playground-executor.html (fix fetch+eval), web/src/components/vanta/playground-executor.tsx (ping+onLoad), web/src/components/vanta/code-playground.tsx (wait ready), web/AGENTS.md §E2E Guard | invariantes: R-FE-4 light-only, no tocar WEB-09 sin gate visual, allow-same-origin techo documentado | deuda: e2e no registrado en CI web (solo local documentado) — opcional añadir job e2e con playwright install en ci-web-11.yml
Próxima tarea si completa: WEB-09
=== END RECITATION ===
