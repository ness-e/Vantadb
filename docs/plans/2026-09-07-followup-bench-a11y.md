# Plan de Ejecución: Follow-up Bench + A11y + Node 2026-09-07

> **Campaign ID:** 5ae0f404-9af7-4ffc-8a6e-117290842c97
> **Inicio:** 2026-09-07
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md (post-campaña 2026-09-07-backlog-triage 8/8)
> **Autonomous:** false
> **SPEC:** no existe SPEC.md — DO set sin feature-add greenfield (INTG/BND-08/TS-10
> diferidos); no se genera SPEC. Si un DO revela feature nueva → pausa + mini-spec.
> **SDP:** triage read-only; skills base: campaign-executor, brainstorming,
> writing-plans, planning-and-task-breakdown, progreso, ponytail (full),
> spec-driven-development.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 4 |
| 🟡 DEFER | resto (splits, PRX/FUT/INTG/STABLE-05/09, GOV-TK, MEM, BLOG-CTA, DESKTOP, PROV, TS-10/11/12/13, WSM-14, WEB-09, FIND-20/21) |
| ❌ SKIP | 4 (UX-17, UX-13, MOD-05, FIND-47 — evidencia abajo) |
| 🔴 BLOQUEADO | 6 (MKT-18f humano, MKT-18i upstream, AUD-042 upstream, DESKTOP-41/43/44 humanos/firma, PRX-10→PRX-03, TS-12→BND-08) |

Status: ⬆️ uphill = 1 (PERF-BENCH-01 metodología A/B) · ⬇️ downhill = 4

## Gate P — confirmación usuario (2026-09-07)

Set confirmado vía `question` → "Aprobar plan (Recomendado)".

## Verificación real global (Paso 0)

- `index.d.ts:349,359,364,366` declara `Promise<number>` vs `lib.rs:529,554`
  `-> napi::Result<u64>` (napi mapea u64→BigInt; BND-12 ya lo sufrió en tests) →
  FIND-BND12-01 ✅ real (nuevo ID para el colateral registrado en BND-12).
- `bench-abi.mjs` existe en `vantadb-node/bench/` + `vantadb-ts/bench/bench.mjs`
  + §15 con medianas → PERF-BENCH-01 ✅ desbloqueado (era "condicionado a números").
- `desktop/e2e/` tiene flujo-critico/multi-perfil/proxy-dashboard + SMOKE-MANUAL.md;
  el smoke ingest→teclado→borrar→papelera→restore→paleta no tiene spec → UX-19 ✅ real.
- `release-npm-node.yml` existe (matrix 7 targets) + BND-12 34/34 → STABLE-07 ✅ real.
- `IngestForm.tsx:65 onRefresh?.()` + `WorkspaceShell.tsx:945,955 onRefresh={setGridKey+1}` →
  UX-17 ✅ ya implementado → SKIP.
- `ActivityPanel.tsx:149-170` banner redactado + `<details>` técnico → UX-13 ✅ ya
  implementado → SKIP.
- `rg InMemoryEngine src/` → 0 hits → MOD-05 ✅ stale → SKIP.
- FIND-47 se auto-declara "no hotspot algorítmico" → SKIP (no-evidencia de problema).

## Tasks

### Task 1: FIND-BND12-01 — index.d.ts u64 → BigInt (4 decls)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1-2h
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-node/index.d.ts:349,359,364,366`, `vantadb-node/src/lib.rs:529,554`
- **Verificación real:** ✅ CÓDIGO-REAL — `Promise<number>` vs `napi::Result<u64>`;
  napi-rs serializa u64 como BigInt (BND-12 lo probó: `0n`/`2n`).
- **Gate Justificación:** contrato TS miente (type-lie) en 4 métodos; consumidores
  que hacen aritmética number rompen en runtime; fix 4 líneas + test assert.
- **Gate Result:** ✅ DO
- **Contrato:** `grep -c "Promise<number>" vantadb-node/index.d.ts` → 0 en esas 4
  decls (bigint) + `npm test` en `vantadb-node/` 34 passed + nuevo test que aserta
  `typeof await db.count(...) === "bigint"`
- **Pre-mortem:**
  - **Fallo probable 1:** `index.d.ts` es auto-generado por napi (header "auto-generated")
    y se sobrescribe en cada build → fix real en `#[napi]` attrs de `lib.rs` o
    documentar regeneración; verificar cómo se generó.
  - **Fallo probable 2:** `compactLayout`/`deleteByFilter` podrían devolver number
    en algún path → verificar cada firma Rust antes de cambiar su decl.
  - **Fallo probable 3:** consumidores TS existentes asumen number → breaking
    menor documentado en NODE_SDK.md (ya tiene sección BigInt truth).
- **Stop conditions:** si el .d.ts se regenera y pisa el fix → resolver en `lib.rs`
  attrs o tooling, no re-editar a mano dos veces (2ª vez → DEFER con hallazgo).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | d.ts auto-generado pisa fix | fix en fuente (lib.rs) | DISCOVERY: leer header + build |
  | 🟢×🟡 | firma mixta number/bigint | una por una contra lib.rs | primer step |
  | 🟢×🟢 | breaking consumidores | nota en NODE_SDK.md | review |
- **Cynefin:** 🟨 complicado — decidir dónde vive el fix (d.ts vs attrs vs tooling).
- **Top 3 riesgos:** regeneración; firmas mixtas; breaking.
- **Uphill/Downhill:** ⬆️ uphill: 1 (fuente de verdad del .d.ts) · resto ⬇️.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí (type-lie runtime) / sí (≤1d) / sí (cierra colateral BND-12).
- **Task file:** `docs/tasks/FIND-BND12-01.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 66ce130f

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 2: PERF-BENCH-01 — A/B vantadb-node nativo vs vantadb-ts WASM

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `vantadb-node/bench/bench-abi.mjs`, `vantadb-ts/bench/bench.mjs`, `docs/operations/BENCHMARKS.md` (nueva §)
- **Verificación real:** ✅ CÓDIGO-REAL — ambos harnesses existen; decisión
  "native primario en Node condicionado a números" pendiente de números.
- **Gate Justificación:** decide posicionamiento (Regla 9); prerrequisito TS-09/BND-12
  cumplidos; solo números propios (D1, sin head-to-head externo).
- **Gate Result:** ✅ DO
- **Contrato:** nueva § BENCHMARKS con tabla insert/search p50/p99 + tamaño
  binario (`.node` vs wasm pkg) + entorno + comandos ×3 corridas mediana;
  0 comparativas externas; 0 adjetivos sin número
- **Pre-mortem:**
  - **Fallo probable 1:** comparar persistente (fjall+fsync) vs in-memory es
    injusto → fairness caveat explícito (precedente NODE_SDK.md), misma métrica,
    dos columnas honestas.
  - **Fallo probable 2:** varianza GC domina (ya visto en §15) → mediana ×3 + rango.
  - **Fallo probable 3:** tamaño binario depende de toolchain → reportar
    artefactos medidos + comando, no absolutos universales.
- **Stop conditions:** varianza >50% sin causa tras ×5 → DEFER con evidencia.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | comparación injusta | fairness caveat + columnas | review Regla 11 |
  | 🟡×🟡 | varianza | mediana ×3 + rango | primera corrida |
  | 🟢×🟢 | binario toolchain | comando + artefacto | review |
- **Cynefin:** 🟨 complicado (metodología; Cynefin+Top3 por 🔴/ambigua no aplica
  en prioridad pero sí por ambigüedad método → se clasifica igual).
- **Top 3 riesgos:** fairness; varianza; binario.
- **Uphill/Downhill:** ⬆️ uphill: 1 (diseño A/B justo) · resto ⬇️.
- **DoD task:** contrato ✅ · sync · recitation. Regla 9/11.
- **Shape Up:** sí (posicionamiento) / sí (≤3d) / sí (harnesses listos).
- **Task file:** `docs/tasks/PERF-BENCH-01.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 63e6a0e5

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:** requiere DISCOVERY (vanta-research si metodología ambigua); si el
  diseñador del bench lo ve 🟧 complejo → probe-sense-respond, no plan total upfront.

### Task 3: UX-19 — Smoke manual → spec Playwright permanente

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟡
- **Archivos clave:** `desktop/e2e/` (nuevo spec), `desktop/e2e/SMOKE-MANUAL.md` (pasos),
  `desktop/playwright.config.ts`
- **Verificación real:** ✅ CÓDIGO-REAL — e2e infra existe (3 specs + config con
  webServer workers:1); SMOKE-MANUAL.md documenta el recorrido sin spec.
- **Gate Justificación:** flujo crítico (ingest→teclado→borrar→papelera→restore→paleta)
  hoy depende de QA manual; guard de regresión permanente.
- **Gate Result:** ✅ DO
- **Contrato:** nuevo `desktop/e2e/smoke-critico.spec.ts` verde
  (`npx playwright test smoke-critico` passed) + suite e2e existente intacta
  (flujo-critico + daud01-temas + multi-perfil + proxy-dashboard verdes)
- **Pre-mortem:**
  - **Fallo probable 1:** requiere server real :8090 + seed (como el smoke manual) →
    reutilizar `serve.mjs`/helpers existentes; si necesita backend vivo documentar
    setup en el spec.
  - **Fallo probable 2:** flaky teclado/foco en CI → locators por roles/labels
    (patrón e2e existente), timeout 60s, workers:1.
  - **Fallo probable 3:** solapa con flujo-critico.spec.ts → leerlo primero y
    cubrir solo el delta (papelera/restore/paleta si falta).
- **Stop conditions:** si requiere app empaquetada (no dev-server) → DEFER (infra mayor).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟡 | setup backend vivo | reutilizar serve.mjs/helpers | DISCOVERY |
  | 🟢×🟡 | flaky foco | roles/labels + timeouts | primera corrida |
  | 🟢×🟢 | duplicación flujo-critico | leer + delta | DISCOVERY |
- **Cynefin:** 🟦 obvio (patrón e2e existente ×3).
- **Top 3 riesgos:** setup; flaky; duplicación.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí (regresión flujo crítico).
- **Task file:** `docs/tasks/UX-19.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** c61642d9

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 4: STABLE-07 — Validar vantadb-node matrix 7 targets

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `.github/workflows/release-npm-node.yml`, `vantadb-node/package.json`,
  `vantadb-node/src/lib.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — workflow existe con matrix; BND-12 dejó
  suite 34/34; falta medición formal 3 corridas + `npm pack` incluye `*.node`.
- **Gate Justificación:** gate npm Fast Gate para futuro default + pre-requisito
  de cualquier publish (BND-08/TS-12); sin `continue-on-error`.
- **Gate Result:** ✅ DO
- **Contrato:** `npm ci && npm run build && npm test` (34 passed) + `npm pack`
  incluye `*.node` + workflow sin `continue-on-error` + tiempo matrix medido
  y documentado en la task (Fast <5min o Heavy justificado)
- **Pre-mortem:**
  - **Fallo probable 1:** solo 1 target compilable en este runner (win x64) →
    validar local + auditar YAML del resto (no inventar builds cruzados).
  - **Fallo probable 2:** `npm pack` no incluye `.node` por files[] → fix package.json.
  - **Fallo probable 3:** matrix lenta >5min → documentar como Heavy (P47 gate 9).
- **Stop conditions:** toolchain napi roto en runner limpio → BLOQUEADO con evidencia.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟢 | 1 solo target local | YAML audit resto | DISCOVERY |
  | 🟢×🟡 | pack sin .node | files[] fix | verify |
  | 🟢×🟢 | matrix lenta | etiqueta Heavy | medición |
- **Cynefin:** 🟦 obvio.
- **Top 3 riesgos:** targets; pack; tiempo.
- **Uphill/Downhill:** ⬇️ downhill.
- **DoD task:** contrato ✅ · sync · recitation.
- **Shape Up:** sí/sí/sí.
- **Task file:** `docs/tasks/STABLE-07.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** (Wave1 commit, validación read-only)

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

## SKIP con evidencia (no re-proponer sin evidencia nueva)

| ID | Evidencia de SKIP |
|----|-------------------|
| UX-17 | `onRefresh?.()` en `IngestForm.tsx:65` + wiring `WorkspaceShell.tsx:945,955` + `key={gridKey}` — remount ya implementado |
| UX-13 | Banner redactado + `<details>` técnico (`ActivityPanel.tsx:149-170`) |
| MOD-05 | `rg InMemoryEngine src/` → 0 hits — clase ya eliminada |
| FIND-47 | El propio reporte declara "no hotspot algorítmico" — sin evidencia de problema |

## DEFER (resumen — detalle en Backlog.md)

- Splits: REVIEW-10, FIND-48/49/50 · Proxy PRX-02..13 · Futuro FUT-02..14, OLD-01
- Estrategia: BND-08, TS-10/11/12/13, WSM-14, INTG-01/02 (feature-add → mini-spec),
  STABLE-05/09, GOV-TK2/5/8, BLOG-CTA, MEM-66/68/69/70, MCP-41, SRV-06, PROV-11/12
- Desktop: DESKTOP-40/42/43/45, FIND-20/21, resto UX (06/08/09/10/11/12/15)
- Web: WEB-09 (input owner) · CI-01 residual

## BLOQUEADO (con causa)

| ID | Causa |
|----|-------|
| MKT-18f | acción humana PyPI |
| MKT-18i | upstream AnythingLLM |
| AUD-042 | upstream tantivy ≥0.27 |
| DESKTOP-41/44 | sesión humana / VM |
| DESKTOP-43 | firma (wontfix DEVOPS-10) |
| PRX-10 | requiere PRX-03 |
| TS-12 | requiere BND-08 |

## Grafo de dependencias (orden)

```
FIND-BND12-01 (independiente) ┐
PERF-BENCH-01 (independiente) ┤→ Wave0 ×3 + STABLE-07 en Wave1 (tras BND-12 base, ya done)
UX-19 (independiente) ┘
STABLE-07 (Wave1: usa BND-12 suite estable)
```

Waves (`FAIL_MODE=parallel`, MAX 3): Wave0 {FIND-BND12-01, PERF-BENCH-01, UX-19} · Wave1 {STABLE-07}.

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| index.d.ts auto-generado pisa fix | resolver en fuente lib.rs, no re-editar |
| A/B injusto persistente vs in-memory | fairness caveat obligatorio (Regla 11) |
| E2E requiere backend vivo | reutilizar serve.mjs/helpers |

=== RECITATION FIND-BND12-01 ===
Campaign ID: 5ae0f404-9af7-4ffc-8a6e-117290842c97
Objetivo activo: FIND-BND12-01: index.d.ts u64 → BigInt (4 decls)
Estado: completed
Última acción: Steps 1-3 ✅ + verify mecánico completo (grep 0/4, cargo check, npm test 35/35)
Resultado: ✅
Próxima acción: vanta-lead: git add 4 archivos + commit fix: node typings u64 as bigint (FIND-BND12-01) en develop
Contrato: verificacion: Select-String Promise<number>=0/Promise<bigint>=4 + cargo check ok + npm test 35 passed; evidencia: claim 4 decls mienten — evidencia: index.d.ts:349,359,364,366 ahora bigint — confianza: alta; claim runtime u64→BigInt — evidencia: tests/api.test.ts 0n/2n + nuevo test typeof — confianza: alta; claim attrs fijan regeneración — evidencia: lib.rs ts_return_type ×4 + napi types-overwrite docs citados en dts-header — confianza: alta; artefactos: vantadb-node/src/lib.rs, vantadb-node/index.d.ts, vantadb-node/tests/api.test.ts, docs/tasks/FIND-BND12-01.md; invariantes: plan file intacto, sin commit (vanta-lead), Wave0 sin dependencias; deuda: ninguna; queda_pendiente: commit vía vanta-lead
Próxima tarea si completa: PERF-BENCH-01
=== END RECITATION ===

=== RECITATION UX-19 ===
Campaign ID: 5ae0f404-9af7-4ffc-8a6e-117290842c97
Objetivo activo: UX-19: guard Playwright permanente del flujo critico
Estado: completed
Última acción: Fix locator multi-perfil:51 + corrida final full suite 12/12 verde; task file sync; SIN commit por orden runner
Resultado: OK
Próxima acción: Orquestador: commit via vanta-lead + skill progreso; proxima tarea Wave0 segun loop externo
Contrato: verificacion: npx playwright test en desktop/ = 12 passed (36.1s) | evidencia: corrida full suite 12/12 verde; flujo-critico.spec.ts:38 cubre ingest-teclado-borrar-papelera-restore-paleta | artefactos: docs/tasks/UX-19.md (nuevo), desktop/e2e/multi-perfil.spec.ts:45-51 (fix locator, sin commit) | invariantes: plan file intacto; sin cambios en app/src; suite e2e verde | deuda: Gate D nombre smoke-critico vs flujo-critico pendiente de owner (renombrar si exige literal); skill progreso no ejecutada (la corre orquestador) | queda_pendiente: commit por vanta-lead (test: permanent smoke-critico E2E guard UX-19)
Próxima tarea si completa: ninguno (Wave0 paralelo: FIND-BND12-01, PERF-BENCH-01 van por separado)
=== END RECITATION ===

=== RECITATION PERF-BENCH-01 ===
Campaign ID: 5ae0f404-9af7-4ffc-8a6e-117290842c97
Objetivo activo: PERF-BENCH-01: A/B native vs WASM numbers + BENCHMARKS §16
Estado: completed
Última acción: S2 ×3 corridas A/B + S3 §16 escrita + S4 verify full verde (fmt/clippy/nextest 2945/validate-docs 0 gaps). SIN commit por orden.
Resultado: OK
Próxima acción: vanta-lead: git add docs/operations/BENCHMARKS.md docs/tasks/PERF-BENCH-01.md + commit + skill progreso + plan file Task 2 → ✅
Contrato: Contrato: § BENCHMARKS tabla insert/search p50/p99 + binarios + entorno + cmds ×3 mediana; 0 externas; 0 adjetivos sin número | verificacion: cargo fmt --check exit 0 + cargo clippy --workspace --all-targets --all-features 0 warnings + cargo nextest --profile audit 2945 passed/1 skipped + validate-docs-coverage.ps1 0 gaps (pwsh7) | evidencia: [{claim: 'tabla lado-a-lado medianas ×3 2000x384x200', evidencia: 'docs/operations/BENCHMARKS.md §16 + $env:TEMP/opencode/bench_ab_run{A,B,C}.log', confianza: 'alta'}, {claim: 'ratios p50 wasm/native 2.64x/1.33x/1.57x, search_vector rangos solapados', evidencia: 'BENCHMARKS.md §16 tabla crudas', confianza: 'alta'}, {claim: 'binarios .node 5.13MiB vs wasm 2.40MiB medidos', evidencia: '(Get-Item).Length 5379072/2512705', confianza: 'alta'}] | artefactos: docs/operations/BENCHMARKS.md (§16), docs/tasks/PERF-BENCH-01.md | invariantes: no tocados index.d.ts/src/lib.rs/api.test.ts (FIND-BND12-01), plan file, sin commit | deuda: skill progreso no ejecutada (movería Backlog/plan en Wave0 paralelo — lo hace el lead) | queda_pendiente: commit 'bench: node native vs wasm A/B numbers (PERF-BENCH-01)' lo hace vanta-lead; rama browser OPFS + shape canónico 100k quedan no-medidos (documentados en §16)
Próxima tarea si completa: UX-19
=== END RECITATION ===

=== RECITATION 4 ===
Campaign ID: 5ae0f404-9af7-4ffc-8a6e-117290842c97
Objetivo activo: STABLE-07: Validar vantadb-node matrix 7 targets
Estado: in-progress
Última acción: DISCOVERY: plan+workflow+package.json+lib.rs+Cargo.toml+tests leidos; Gate D/P evaluados; task file STABLE-07.md creado
Resultado: PARTIAL
Próxima acción: S1: auditar YAML matrix 7 targets + continue-on-error; luego npm test local
Contrato: verificacion: DISCOVERY completo (workflow+package.json+lib.rs+tests leidos); evidencia pendiente: npm test local + npm pack + YAML audit | artefactos: docs/tasks/STABLE-07.md (nuevo) | invariantes: no tocar plan file ni archivos Wave0 (66ce130f,63e6a0e5,c61642d9); sin commit (orden runner) | deuda: ninguna aun | queda_pendiente: S1-S4 validacion + cierre verify full
Próxima tarea si completa: ninguno (Wave1 ultima)
=== END RECITATION ===

=== RECITATION STABLE-07 ===
Campaign ID: 5ae0f404-9af7-4ffc-8a6e-117290842c97
Objetivo activo: STABLE-07: Validar vantadb-node matrix 7 targets (gate npm + pre-requisito publish)
Estado: completed
Última acción: S5 cierre: re-verify S1-S4 + fmt exit 0 + medición Heavy documentada; task file sync; SIN commit
Resultado: ✅
Próxima acción: Orquestador/vanta-lead: git add docs/tasks/STABLE-07.md + commit + skill progreso + plan Task 4 → ✅
Contrato: verificacion: npm test 35/35 (10.18s) + npm pack --dry-run 6 files con .node 5.4MB + YAML 7/7 targets + 1xCOE CATEGORY:INFORMATIONAL + cargo fmt --check exit 0; evidencia: claim matrix 7 targets — evidencia: release-npm-node.yml:38-58 vs package.json:36-44 (7/7 match) — confianza: alta; claim suite verde — evidencia: npm test 35/35 vitest 4.1.10 — confianza: alta; claim pack incluye .node — evidencia: npm pack dry-run vantadb_native.win32-x64-msvc.node 5.4MB — confianza: alta; claim Heavy justificado — evidencia: timeouts 20/10/15 + 7 jobs LTO 3 OS — confianza: media; artefactos: docs/tasks/STABLE-07.md (único archivo escrito); invariantes: plan file intacto, archivos Wave0 intactos (66ce130f,63e6a0e5,c61642d9), sin commit (orden runner); deuda: ninguna; queda_pendiente: commit vía vanta-lead + skill progreso + plan Task 4 → ✅
Próxima tarea si completa: ninguno (Wave1 última)
=== END RECITATION ===
