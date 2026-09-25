# Plan de Ejecución: Cierre de 14 vulnerabilidades Dependabot abiertas — 2026-09-22

> **Campaign ID:** 67262f34-0fe8-4046-9f0c-9023f081e2eb
> **Inicio:** 2026-09-22
> **Estado:** ✅ COMPLETADO (2026-09-23: 0 abiertas — 28 fixed + 1 dismissed + 1 auto_dismissed. Cierre administrativo pendiente: archive + progreso)
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

- [x] VULN-B1 · Bump `postcss` 8.5.15→≥8.5.23 SOLO en `vantadb-ts/package-lock.json`
  (único lock aún vulnerable; node/web/remotion/desktop ya ≥8.5.23). Cierra #26.
- [x] VULN-B2 · Clonar bump vitest/mocker →4.1.11 para `vantadb-ts` (gaps #40/#41;
  #196 solo cubre `vantadb-node`). Riesgo temporal aceptable (localhost default).
- [x] VULN-B3 · js-yaml node (`vantadb-node/package-lock.json:2392`, #42) + web nested
  (`web/package-lock.json:480`, #48, vía bump eslintrc u `overrides`) + nanoid desktop
  (`desktop/package-lock.json:5409`, dev). Contrato: alertas → `fixed` o motivo escrito.
- Verificación Wave B: `gh api dependabot/alerts` muestra cada alerta en `fixed`;
  `npm audit` del scope sin HIGH/MEDIUM nuevos.

## Wave C — Accept-risk documentado (no bump)

- [x] VULN-C1 · glib #38 (UB VariantStrIter, MEDIUM): bump a 0.20.0 bloqueado por Tauri2
  (pinea gtk/glib 0.18); sin PoC en VantaDB (no se usa `VariantStrIter` en código propio,
  solo transitivo webview). Acción: `cargo audit` allow con expiry + track upstream
  (gtk-rs-core#1343), NO bump manual. Contrato: justificación escrita + allow con fecha.

## Alcanzabilidad (evidencia del auditor — por qué este orden)

- fast-uri/js-yaml/nanoid: transitivas de tooling (ajv/cosmiconfig/postcss), build-time,
  no llegan a wasm/server/prod — pero HIGH+SSRF/DoS van primero igual.
- postcss/vitest: dev-only (requieren CSS atacante o dev-server expuesto) — después.
- glib: prod desktop pero UB no alcanzable sin uso propio — accept-risk al final.

## Wave D — Verificación final (orquestador, tras merges + rescans)

- [x] VULN-D1 · ✅ 2026-09-23 verificado: `gh api dependabot/alerts` → 0 abiertas (28 fixed + 1 dismissed + 1 auto_dismissed; #26/#30/#32 cerradas por re-scan tras merges). Contrato: 0 abiertas cumplido.
  #26/#30/#32/#36/#42 sigue abierta tras 24h del merge, diagnosticar: lag de re-scan vs gap
  real). Contrato: 0 abiertas o motivo escrito por restante. (Glib #38 ya dismissed como
  accept-risk; no cuenta.)

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
Próxima acción: `/pipeline run docs/dev/plans/2026-09-22-vulnerabilities.md` (Wave A primero)
Contrato: plan file existe con waves, contratos alerta→fixed y gates; task files bajo demanda
Próxima tarea: VULN-A1 (merge #195)
last-synced: 2026-09-22
=== END RECITATION ===

=== RECITATION VULN-B1-B2 ===
Campaign ID: 67262f34-0fe8-4046-9f0c-9023f081e2eb
Objetivo activo: VULN-B1-B2: bumps postcss+vitest en vantadb-ts (gaps #26/#40/#41)
Estado: completed
Última acción: S1+S2 completos + commit acb6fcb3 en fix/vuln-ts-060 + rescate worktree compartido (rama B3 cf6560f0 separada, disjunta)
Resultado: ✅
Próxima acción: Lead: git push fix/vuln-ts-060 + abrir PR; verificar alertas #26/#40/#41 en fixed
Contrato: verificacion: npm ls postcss@8.5.28 + vitest/mocker@4.1.11 OK; tsc exit 0; npm test 311/311 OK; commit acb6fcb3 (3 files) en fix/vuln-ts-060, SIN PUSH | evidencia: npm ls + tsc/test exits (tool results); git show --stat acb6fcb3 (3 files, 135+/60-) | artefactos: commit acb6fcb3; docs/dev/tasks/VULN-B1-B2.md | invariantes: cero codigo; rama no pusheada; rama B3 separada cf6560f0 | deuda: campaign_verify_cmd no corrido (BUG exit -1 conocido, bash directa usada); skill progreso omitida (scope prohibe docs/ salvo task file); plan file sin commitear (recitations compartidas B1-B2+B3, reconcilia lead) | queda_pendiente: lead pushea fix/vuln-ts-060 + abre PR (cierra #26/#40/#41 via alerta fixed)
Próxima tarea si completa: lead-push-PR-VULN-B1-B2
=== END RECITATION ===

=== RECITATION VULN-B3 ===
Campaign ID: 67262f34-0fe8-4046-9f0c-9023f081e2eb
Objetivo activo: VULN-B3 — js-yaml node/web + nanoid desktop (bumps mínimos, solo lockfiles)
Estado: completed
Última acción: 3/3 slices ✅ + commit cf6560f0 en fix/vuln-node-web-desktop (base develop, 4 files, mensaje exacto), SIN PUSH. Colisión de worktree compartido con agente VULN-B1/B2: mi commit cayó en su rama y el peer lo re-ubicó limpio sobre develop; verificado contenido idéntico + WIP ajeno intacto y excluido.
Resultado: ✅
Próxima acción: Lead: git push fix/vuln-node-web-desktop + abrir PR + review P2-01 + skill progreso (omitida aquí por scope docs/ prohibido)
Contrato: Contrato cumplido. Verificacion: npm ls js-yaml vantadb-node→4.3.2 ✅ + web nested (@eslint/eslintrc)→4.3.2 ✅ + npm ls nanoid desktop→3.3.19 (≥3.3.18) ✅; npx tsc --noEmit web exit 0 ✅; npm run lint web exit 0 ✅; campaign_verify_cmd exit 0 ✅; git diff HEAD limpio en mis 4 paths ✅. Evidencia: npm ls outputs por slice + lock windows 2401/480/5409 + tsc/lint exits + commit cf6560f0 (4 files, hooks pre-commit verdes). Artefactos: vantadb-node/package-lock.json, web/package-lock.json, desktop/package-lock.json, docs/dev/tasks/VULN-B3.md. Invariantes: vantadb-ts/src Rust/workflows/docs (salvo task file) intactos; nada publicado; SIN PUSH. Deuda: ninguna (review P2-01 diferido al PR). Queda_pendiente: lead pushea rama + abre PR + review + skill progreso.
Próxima tarea si completa: lead-push-PR-VULN-B3
=== END RECITATION ===
