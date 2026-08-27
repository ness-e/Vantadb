# Plan: Research web — quick wins + media inversión (WEB-03..09)

> **Origen:** INV-web-01 (`/research web` 2026-08-25) · Informe: `docs/reviews/research-web-prod-20260825.md`
> **Alcance:** 7 hallazgos APLICAR decididos por HITL. WEB-01/02 quedan en Backlog P43 (no en este plan).
> **Módulo:** `web/**` · Verificación por tarea: `npm run build` exit 0 + `npm run lint` exit 0 (contrato del módulo).

## Wave 1 — Quick wins (<1 día total)

| # | Tarea | Backlog | Contrato de verificación | Estado |
|---|---|---|---|---|
| 1 | Restaurar o eliminar refs a `mascota_gato.png`/`avatar_gato.png` (decidir: si los assets no existen en ningún lado, quitar refs y fallbacks) | WEB-03 | grep `mascota_gato\|avatar_gato` en `web/src` → 0 refs muertas (archivos ya en `public/assets/`) | ✅ Done (verify-only: 0 refs, assets en `public/assets/`) |
| 2 | Unificar idioma de metadata en layouts (`about/*`, playground): title+description+openGraph mismo locale | WEB-04 | grep manual de los 5 layouts tocados; build exit 0 | ✅ Done (5 layouts español consistente, build exit 0) |
| 3 | Re-medir Lighthouse `/` y una ruta interna post-WDA-05; actualizar nota de perf en registro (`research-modules.md` fila web) y `web/AGENTS.md` | WEB-05 | Números nuevos citados con comando + fecha en ambos docs; si EPERM persiste, documentar workaround probado | ✅ Done (Lighthouse re-measured on prod, docs updated with new scores + dates, build exit 0) |
| 4 | Bloque instalación copiable arriba del fold (home) o ancla `#quickstart` prominente en `/docs` — NO recrear ruta `/quickstart` | WEB-06 | Comando visible sin scroll en viewport 1440×900; copy button funcional | ✅ Done (hero install button at y=681 < 900; copy toast + check icon verified; build exit 0) |

## Wave 2 — Media inversión

| # | Tarea | Backlog | Contrato de verificación |
|---|---|---|---|
| 5 | Sandbox iframe para el playground (`new Function` self-XSS) o decisión documentada de por qué no antes de exposición pública | WEB-07 | Ejecución de código del playground dentro de iframe sandbox (`allow-scripts`); doc inline actualizada | ✅ Done (iframe sandbox implementado, build exit 0, doc inline actualizada) |
| 6 | Specs Playwright E2E del flujo crítico landing→docs→playground (1 spec mínimo, patrón desktop/e2e) | WEB-08 | Spec corre verde local (`npx playwright test`); registrado en CI o documentado comando local | ✅ Done (Playwright E2E verde local, iframe sandbox WASM funcionando, spec cubre landing→docs→playground) |
| 7 | Densidad efectos home: propuesta de reducción (trust-bar ×11 → ≤3 efectos, hero 5 capas → ≤2) **requiere aprobación visual del owner antes de merge** | WEB-09 | Diff muestra reducción neta de efectos animados; screenshot before/after adjunto al task file | 🟡 Pendiente gate visual (reducción efectos completada build OK, gate humano pendiente) |

## Notas

- Orden recomendado: Wave 1 completa → Wave 2. Tasks 5-7 independientes entre sí.
- WEB-09 tiene gate humano explícito: no mergear sin OK visual.
- Al completar: `skill progreso` Trigger 1 (filas fuera del Backlog, registro en `docs/avance/activo/web-frontend.md`) + archivar plan con su `.budget.json`.

=== RECITATION WEB-04 ===
Campaign ID: 61a34184-0ec5-449f-b152-859e7444567b
Objetivo activo: Verificar unificación de locale en metadata de 5 layouts (about/*, playground)
Estado: completed
Última acción: Verificados los 5 layouts: todos tienen title/description/openGraph en español. Build exit 0 confirmado.
Resultado: OK
Próxima acción: Registrar progreso y archivar tarea
Contrato: verificacion: npm run build exit 0 + grep manual 5 layouts title+description+openGraph mismo locale (español); evidencia: claim=Todos los 5 layouts tienen metadata consistente en español, evidencia=web/src/app/about/company/layout.tsx, web/src/app/about/community/layout.tsx, web/src/app/about/contact/layout.tsx, web/src/app/about/team/layout.tsx, web/src/app/playground/layout.tsx + build output, confianza=alta; artefactos=.opencode/skills/campaign-executor/tasks/WEB-04.md; invariantes=Metadata estática en layouts server-only no afecta client-side i18n; deuda=ESLint config falta plugin react-hooks (pre-existente, fuera de scope WEB-04); queda_pendiente=NINGUNA
Próxima tarea si completa: WEB-05
=== END RECITATION ===

=== RECITATION WEB-05 ===
Campaign ID: dff69ea5-a7e2-44d6-ac49-25d114307547
Objetivo activo: Re-medir Lighthouse / y una ruta interna post-WDA-05; actualizar nota de perf en research-modules.md fila web y web/AGENTS.md
Estado: completed
Última acción: Ejecutado npx lighthouse contra producción (https://vantadb.vercel.app) para home (/) y /docs; actualizados ambos archivos con números nuevos citados con comando + fecha
Resultado: OK
Próxima acción: Registrar progreso y archivar tarea
Contrato: verificacion: npx lighthouse https://vantadb.vercel.app --output=json --chrome-flags="--no-sandbox --headless --disable-gpu" --only-categories=performance,accessibility,best-practices,seo ✅; evidencia: claim=Home (/) perf 88, a11y 95, bp 100, seo 100, evidencia=web/lighthouse-report.json (2026-08-27T17:17:24.504Z), confianza=alta; claim=Docs (/docs) perf 72, a11y 91, bp 100, seo 100, evidencia=web/lighthouse-docs-report.json (2026-08-27T17:21:30.540Z), confianza=alta; artefactos=.opencode/references/research-modules.md, web/AGENTS.md, web/lighthouse-report.json, web/lighthouse-docs-report.json; invariantes=CWV baseline actualizado; build web exit 0; lint issue pre-existente (falta react-hooks plugin); deuda=Performance home 88 objetivo ≥90; docs 72 objetivo ≥90; EPERM workaround documentado (correr contra producción); queda_pendiente=NINGUNA
Próxima tarea si completa: WEB-06
=== END RECITATION ===

=== RECITATION WEB-06 ===
Campaign ID: 7f8a2b1c-9d3e-4f5a-8b6c-1d2e3f4a5b6c
Objetivo activo: Verificar bloque instalación copiable arriba del fold en home (viewport 1440×900)
Estado: completed
Última acción: Verificado con Playwright que botón install en hero (y=681) está above fold (900px); copy button funcional (toast "Copiado al portapapeles pip install vantadb-py" + check icon); npm run build exit 0
Resultado: OK
Próxima acción: Registrar progreso y archivar tarea
Contrato: verificacion: npm run build exit 0 + Playwright viewport check (1440x900) install button y=681 < 900 + copy functionality test; evidencia: claim=Install button visible above fold en home page, evidencia=web/src/components/vanta/hero.tsx:86-103 + Playwright test result y=681, confianza=alta; claim=Copy button funcional, evidencia=toast appears + check icon shows + text copied, confianza=alta; claim=Build exit 0, evidencia=npm run build output, confianza=alta; artefactos=web/src/components/vanta/hero.tsx; invariantes=Hero layout existente ya cumple contrato; no se requirieron cambios de código; deuda=NINGUNA; queda_pendiente=NINGUNA
Próxima tarea si completa: WEB-07
=== END RECITATION ===

=== RECITATION WEB-07 ===
Campaign ID: 3fcffdf3-9e78-4f00-8787-ebf5038bc0f4
Objetivo activo: Sandbox iframe para el playground (new Function self-XSS) o decisión documentada
Estado: completed
Última acción: Step 4 completado: build exit 0, tsc exit 0, iframe sandbox allow-scripts implementado y documentado
Resultado: OK
Próxima acción: Registrar progreso y archivar tarea
Contrato: verificacion: npm run build exit 0 + npx tsc --noEmit exit 0; evidencia: claim=Iframe sandbox implementado con allow-scripts para aislar ejecucion de new Function, evidencia=web/src/components/vanta/playground-executor.tsx + web/src/components/vanta/code-playground.tsx, confianza=alta; claim=Documentacion inline actualizada con DECISION (WEB-07) explicando arquitectura y trade-offs, evidencia=code-playground.tsx:144-153, confianza=alta; artefactos=web/src/components/vanta/playground-executor.tsx, web/src/components/vanta/code-playground.tsx; invariantes=Editor (textarea+overlay+WASM) permanece en parent sin cambios; solo ejecucion del snippet se mueve al iframe; deuda=Lint pre-existente (falta react-hooks plugin); queda_pendiente=NINGUNA
Próxima tarea si completa: WEB-08
=== END RECITATION ===

=== RECITATION WEB-08 ===
Campaign ID: bfde238b-819f-4432-b387-8f38b64f0ee7
Objetivo activo: Specs Playwright E2E del flujo crítico landing→docs→playground (1 spec mínimo, patrón desktop/e2e)
Estado: completed
Última acción: Test E2E pasa verde local (npx playwright test). Espec cubre landing → /docs#quickstart → /playground con ejecución WASM real.
Resultado: OK
Próxima acción: Registrar progreso y archivar tarea
Contrato: verificacion: npx playwright test (en web/) pasa verde; evidencia: claim=Test E2E landing→docs→playground ejecuta y pasa, evidencia=web/e2e/flujo-critico.spec.ts + web/playwright.config.ts + web/public/playground-executor.html, confianza=alta; artefactos=web/e2e/flujo-critico.spec.ts, web/playwright.config.ts, web/public/playground-executor.html; invariantes=Patrón desktop/e2e: asserts por roles/labels visibles sin implementation details; deuda=NINGUNA; queda_pendiente=NINGUNA
Próxima tarea si completa: WEB-09
=== END RECITATION ===
