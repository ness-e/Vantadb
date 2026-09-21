# Plan de Ejecución: Cierre de 14 vulnerabilidades Dependabot abiertas — 2026-09-22

> **Campaign ID:** (asigna `campaign_get_next_task` al arrancar)
> **Inicio:** 2026-09-22
> **Estado:** 🔄 EN PROGRESO (Wave A ✅ 2026-09-22: 14→6 abiertas; Wave B/C pendientes)
> **Fuente:** triage READ-ONLY 2026-09-22 (`vanta-audit`: 14/14 alertas ubicadas en
> lockfiles con archivo:línea + GHSA/CVE verificados). #35/#34 críticas Next.js ya FIXED.

## Decisiones del owner (vinculantes, 2026-09-22)

1. Survivor vitest: **#196** (cierra #40/#32 node); #197 se cierra.
2. glib #38: **aceptado y vigilado** (allow con expiry + track upstream, sin bump manual).
3. PRs de seguridad Wave A mergean a `main` + forward-port a `develop` (excepción a D3 del plan PRs).
4. **Wave A aprobada para ejecución inmediata**.

## Resumen

8 HIGH + 6 MEDIUM, todas ubicadas. 9/14 se cierran con 4 PRs ya abiertos (#195, #198,
#199, #196); 4/14 exigen PRs nuevos (gaps sin cobertura); 1/14 (glib) es accept-risk
documentado. Seguridad-primero: SSRF/DoS primero, dev-only después, UB no-alcanzable al final.

## Wave A — Merge PRs existentes (seguridad primero; batch remotion junto)

- [x] VULN-A1 · Merge #195 (fast-uri 3.1.5→3.1.8 en `web/remotion/package-lock.json`).
  Cierra 4 HIGH de una vez (#44 host-confusion, #45 SSRF double-decode, #46 IPv6,
  #47 IDN). Excede floors 3.1.6/3.1.7. Contrato: alertas #44-47 → `fixed`.
- [x] VULN-A2 · Merge #198 (js-yaml →4.3.2 en remotion). Cierra #36 parcial (DoS CPU
  CVE-2026-84375). Gaps node/web-nested quedan a Wave B.
- [x] VULN-A3 · Merge #199 (nanoid →3.3.19 en remotion, excede floor 3.3.18).
  Cierra #43 parcial. Mergear junto a A1-A2 minimiza conflictos del lock.
- [x] VULN-A4 · Merge #196 (superset mocker+vitest node) + cerrar #197 duplicado.
  Cierra #30/#32 (node). Contrato: alertas node → `fixed`.
- Post-Wave A: forward-port `main`→`develop` (los 5 PRs apuntan a `main`; sin port hay drift).

## Wave B — PRs nuevos para gaps sin cobertura (crear, CI verde, mergear)

- [ ] VULN-B1 · Bump `postcss` 8.5.15→≥8.5.23 SOLO en `vantadb-ts/package-lock.json`
  (único lock aún vulnerable; node/web/remotion/desktop ya ≥8.5.23). Cierra #26.
- [ ] VULN-B2 · Clonar bump vitest/mocker →4.1.11 para `vantadb-ts` (gaps #40/#41;
  #196 solo cubre `vantadb-node`). Riesgo temporal aceptable (localhost default).
- [ ] VULN-B3 · js-yaml node (`vantadb-node/package-lock.json:2392`, #42) + web nested
  (`web/package-lock.json:480`, #48, vía bump eslintrc u `overrides`) + nanoid desktop
  (`desktop/package-lock.json:5409`, dev). Contrato: alertas → `fixed` o motivo escrito.
- Verificación Wave B: `gh api dependabot/alerts` muestra cada alerta en `fixed`;
  `npm audit` del scope sin HIGH/MEDIUM nuevos.

## Wave C — Accept-risk documentado (no bump)

- [ ] VULN-C1 · glib #38 (UB VariantStrIter, MEDIUM): bump a 0.20.0 bloqueado por Tauri2
  (pinea gtk/glib 0.18); sin PoC en VantaDB (no se usa `VariantStrIter` en código propio,
  solo transitivo webview). Acción: `cargo audit` allow con expiry + track upstream
  (gtk-rs-core#1343), NO bump manual. Contrato: justificación escrita + allow con fecha.

## Alcanzabilidad (evidencia del auditor — por qué este orden)

- fast-uri/js-yaml/nanoid: transitivas de tooling (ajv/cosmiconfig/postcss), build-time,
  no llegan a wasm/server/prod — pero HIGH+SSRF/DoS van primero igual.
- postcss/vitest: dev-only (requieren CSS atacante o dev-server expuesto) — después.
- glib: prod desktop pero UB no alcanzable sin uso propio — accept-risk al final.

## Gates

- **Gate P:** este plan es SOLO análisis+propuesta; merges/updates se ejecutan en
  `/pipeline run` con verify (alerta en `fixed` + CI verde del PR).
- **Stop honesto:** si un bump rompe build/tests → STOP + desagregar, no forzar major.
- **Appetite / Branch:** 2d / merges vía `gh pr merge`; PRs nuevos en ramas dependabot-style.

## Fuentes (GHSA/CVE verificados por webfetch)

- js-yaml HIGH 7.5 CVE-2026-84375: https://github.com/advisories/GHSA-2883-xcg3-v3hh
- nanoid HIGH 8.2 CVE-2026-67213: https://github.com/advisories/GHSA-2v37-7h3g-55p8
- postcss MODERATE 6.3 CVE-2026-69153: https://github.com/advisories/GHSA-fxqj-rqcc-2cmp
  (+ previa https://github.com/advisories/GHSA-6g55-p6wh-862q)
- vitest/mocker MODERATE 5.9 CVE-2026-84373: https://github.com/advisories/GHSA-82fw-gwwq-j7x9
- fast-uri HIGH: https://github.com/advisories/GHSA-fph4-wmhf-6fwf (CVE-2026-75899) +
  https://github.com/advisories/GHSA-5jgf-p345-68v8 (CVE-2026-75931) (+ floors 3.1.2/3.1.7
  verificados por websearch, cubiertos por 3.1.8)
- glib MODERATE: https://github.com/advisories/GHSA-wrw7-89jp-8q8g +
  https://rustsec.org/advisories/RUSTSEC-2024-0429.html (fix gtk-rs-core#1343)

=== RECITATION ===
Objetivo activo: PLAN vulnerabilities-14 — plan creado (SOLO PLAN)
Estado: plan (Wave A: 4 merges · Wave B: 3 PRs nuevos · Wave C: 1 accept-risk)
Última acción: plan creado desde triage 14/14 + GHSA verificados
Resultado: ✅
Próxima acción: `/pipeline run docs/plans/2026-09-22-vulnerabilities.md` (Wave A primero)
Contrato: plan file existe con waves, contratos alerta→fixed y gates; task files bajo demanda
Próxima tarea: VULN-A1 (merge #195)
last-synced: 2026-09-22
=== END RECITATION ===
