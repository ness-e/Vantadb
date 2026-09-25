# DESKTOP-45 — Bench app + specs E2E (recortado)

> **Plan:** `docs/dev/plans/2026-09-10-code.md` (Task 12, Wave3) · **Estado:** ⏳ IN PROGRESS
> **Ruta:** vanta-worker · **Appetite:** max 1d · **Tipo:** docs (e2e specs + bench doc)
> **Contrato:** specs E2E proxy/graph/space lenses (+ Playwright) + intento bench documentado (si app no corre: evidencia + DEFER H-15)
> **SDP:** playwright-cli, incremental-implementation, test-driven-development, context-engineering (+ base campaign-executor, progreso) · keywords: desktop/e2e/benchmarks · discover v2 BUILD 8 skills (frontend-ui-engineering, api-and-interface-design, source-driven-development, doubt-driven-development scored — solo se cargan las 4 aplicables al slice)

## DISCOVERY (2026-09-10)

- `desktop/e2e/` tiene 4 specs: `flujo-critico`, `proxy-dashboard` (H-07 ✅ mock upstream, 210L), `daud01-temas`, `multi-perfil` + `helpers.ts` (seed vía `POST /api/v2/records/batch`, `APP_BASE` en `:8091/dashboard/`) + `serve.mjs` (rebuild dist-web + server con DB temp) + `SMOKE-MANUAL.md`.
- `playwright.config.ts`: serial (`workers: 1`), `webServer: node e2e/serve.mjs`, timeout 60s/test, 120s server (incluye rebuild).
- Proxy lens YA cubierta por `proxy-dashboard.spec.ts` (H-07: form CONECTAR, dashboard con mock `**/snapshot`, gate `proxyConfigured`, cambio URL limpia) → el contrato "proxy/graph/space" se cumple con **graph + space nuevas** (proxy = pre-existente verificada, no duplicar).
- `GraphLens.tsx`: surface `iql` (WorkspaceShell:606 `surface === "iql"`; sidebar `IQL`, title "Ir a IQL — consola de queries sobre grafo"). Seed: namespace con más registros → `graphDegree` hubs sin aristas (`useGraphData.ts:3-7`). UI estable para asserts: header `GRAFO`, toolbar `⌨ iql / ⛶ fit / ↺ reset / labels`, canvas `role="img"` con aria `Grafo de N nodos y M aristas`, lista sr-only `Nodos visibles del grafo`, consola IQL colapsable, pista de interacción.
- `SpaceLens.tsx`: surface `espacio` (sidebar `ESPACIO`, title "Ir a ESPACIO — proyección 2D"). Solo records **con `vector`** se proyectan (`useProjection.ts:79`); resto skip. UMAP `nNeighbors: min(15, max(2, n-1))` (`projection.worker.ts:49`) → funciona con pocos puntos. UI: header `ESPACIO`, `select[aria-label="Namespace a proyectar"]`, botón `⤒ proyectar`, empty `sin proyección — click en «⤒ proyectar»`, canvas `role="img"` `Scatterplot de N embeddings…`, lista sr-only `Puntos proyectados`, `SelectionBar`.
- `release-plz.toml:23-32` = nota H-11 ✅ (desktop excluido de release-plz, no es bench; H-11 real era el § de bench en BENCHMARKS.md — el plan la da por hecha, no se toca).
- Bench app (H-15): exige app viva (Tauri o embedded server). Stop condition del plan: si no arranca en CI → specs + evidencia, DEFER bench.
- **Gate D (question-gates):** NO dispara — 2 archivos nuevos en `desktop/e2e/` + 1 nota bench, sin símbolos públicos, sin hot path, sin API pública. Motivo: specs aditivas disjuntas Wave3.
- **Pre-mortem del plan:** Playwright sin sesión dedicada → specs autocontenidas (seed REST + asserts roles/labels existentes, patrón `flujo-critico.spec.ts`) + DEFER corrida si el runner no levanta `serve.mjs`.

## Spec (contrato mecánico — qué/when/then por slice)

| Slice | Archivo | Given | When | Then |
|-------|---------|-------|------|------|
| 1 | `desktop/e2e/graph-lens.spec.ts` | 3 records seed en `ns-grafo` | Ir a IQL | header GRAFO + meta namespace/nodos; toolbar iql/fit/reset/labels; canvas `Grafo de 3 nodos`; lista sr-only con 3 items; consola IQL visible + toggle la oculta |
| 2 | `desktop/e2e/space-lens.spec.ts` | 12 records **con vector dim 8** en `ns-espacio` | Ir a ESPACIO → `⤒ proyectar` | empty-state inicial; tras proyectar canvas `Scatterplot de 12 embeddings`; lista sr-only con puntos; sin `role="alert"` fatal |
| 3 | bench | app viva o no | intento `test:e2e` scoped o doc | specs listan OK; si serve.mjs no levanta → evidencia + DEFER H-15 en task file |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/e2e/helpers.ts` (46L), `flujo-critico.spec.ts` (115L), `proxy-dashboard.spec.ts` (1-60), `playwright.config.ts` (39L), `desktop/package.json` (scripts `test:e2e`), `GraphLens.tsx` (151L), `useGraphData.ts` (1-80), `SpaceLens.tsx` (200-317), `useProjection.ts` (1-90), `projection.worker.ts` (nNeighbors), `WorkspaceShell.tsx` (líneas 604-606, 1055-1062), `BENCHMARKS.md` (1-80), `release-plz.toml` (1-40).
- **Referencias hacia dentro:** specs nuevas solo importan `./helpers` + `@playwright/test`; config `testMatch: /\.spec\.ts$/` las recoge sin cambios; `serve.mjs` sin cambios.
- **Referencias entrantes:** nadie importa `desktop/e2e/*` (solo playwright). Nota bench: doc nuevo, sin referencias.
- **Veredicto:** impacto cero en app/Rust/CI. Archivos nuevos aditivos, rollback = borrarlos. Riesgo residual: `serve.mjs` rebuild dist-web lento en este runner (120s timeout) → slice 3 decide correr o DEFER con evidencia.

## Steps

- [x] 1. `desktop/e2e/graph-lens.spec.ts`: surface IQL + toolbar + canvas + lista sr-only + consola toggle
- [x] 2. `desktop/e2e/space-lens.spec.ts`: surface ESPACIO + auto-proyección 12 vectores + canvas + notice éxito
- [x] 3. Verify mecánico: `tsc --noEmit` + `playwright test --list` + corrida scoped 2/2 verde
- [x] 4. Commit solo-propios en develop + recitation + RESULTADO

## Verify (2026-09-10)

- `npx tsc --noEmit` (desktop) ✅ exit 0
- `npx playwright test --list` ✅ 14 tests / 6 files (2 nuevos listados)
- `npx playwright test graph-lens.spec.ts space-lens.spec.ts` ✅ **2 passed (32.1s)** — server embedded real (`serve.mjs`, DB temp), sin mocks
  - graph-lens 1.2m primera corrida (incluye rebuild dist-web); space-lens 29.5s → 9.9s
  - TDD real: space-lens falló 2 veces antes del verde — (1) empty-state idle no existe (SpaceLens auto-proyecta al montar, `SpaceLens.tsx:153-156`); (2) `role="alert"` contiene el notice de éxito `Proyección lista: 12 puntos` (`SpaceLens.tsx:165`), convertido en assert positivo
- Proxy lens: pre-cubierta por `proxy-dashboard.spec.ts` (H-07, 5 tests mock upstream) — no duplicada
- **Bench (H-15):** app embedded SÍ corre en este runner (evidencia: corridas E2E reales 32.1s). Lo que NO existe es un harness perf dedicado para la app desktop (timings por-spec como única métrica: graph ~1.2m con rebuild, space ~10s). → **H-15 DEFER** con evidencia: sin harness no hay números Regla-11 citables; los timings E2E no son bench.
- WIP ajeno intacto: `desktop/src-tauri/Cargo.lock` (M pre-existente), `opencode.jsonc`, `.opencode`, `Investigacion-plan.md` — NO stageados.
