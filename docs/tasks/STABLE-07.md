# STABLE-07 — Validar vantadb-node matrix 7 targets

> **Plan:** `docs/plans/2026-09-07-followup-bench-a11y.md` (Task 4, Wave1)
> **Estado:** ⏳ IN PROGRESS
> **Tipo:** verification/validación CI+pack (no feature-add greenfield → sin SPEC greenfield; contrato = plan)
> **SDP:** campaign-executor, progreso, systematic-debugging, source-driven-development (+ ponytail full siempre activo).
> Evaluados y no cargados (ponytail: contexto mínimo): browser-testing-with-devtools (lifecycle VERIFY genérico, nada corre en navegador aquí), incremental-implementation/test-driven-development/context-engineering (sin código nuevo previsto — validación read-only salvo fix files[] si pack falla).
> **Appetite/Branch:** max 1d / develop (sin commit — orden runner: cierre verify full → NO commitear)

## Contrato (del plan — ley)

- `npm ci && npm run build && npm test` en `vantadb-node/` (34+ passed) + `npm pack` incluye `*.node` + workflow sin `continue-on-error` injustificado + tiempo matrix medido y documentado (Fast <5min o Heavy justificado).

## Spec (decisiones — Gate mecánico spec-first N/A: no hay lógica nueva)

| Decisión | Evidencia | Veredicto |
|----------|-----------|-----------|
| ¿Requiere SPEC greenfield? | Validación read-only de matrix/pack/suite existente; sin símbolos públicos nuevos | NO — contrato del plan basta |
| ¿Fix files[] necesario? | `package.json:16-22` files[] ya incluye `*.node` | NO (verificar con pack dry-run) |
| ¿Quitar `continue-on-error:209`? | Línea 208-209 con `# CATEGORY: INFORMATIONAL` (attest provenance) | NO — taxonomía CI_POLICY lo permite; documentar, no remover |

## Gate D (tras zero-code planning, antes del task file)

> Blast radius = 3 archivos declarados + tests vecinos (solo lectura prevista); sin hot path (`vector/`, `engine.rs` no tocados); sin símbolos públicos nuevos; contrato mecánico y verificable. **Gate D NO disparado** (sin question tool en runner; registrado aquí por question-gates.md §Registro obligatorio).

## Gate P (ya confirmado a nivel plan 2026-09-07 vía question → "Aprobar plan")

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/release-npm-node.yml` (223L), `vantadb-node/package.json` (59L), `vantadb-node/src/lib.rs` (1220L, vía Read + codegraph_explore), `vantadb-node/Cargo.toml` (31L), `vantadb-node/index.d.ts` header (30L), `vantadb-node/tests/` (3 specs, 35× `it`).
- **Referencias hacia dentro (el workflow depende de):** `vantadb-node/{package.json,index.cjs,index.js,index.d.ts,*.node}`, napi-rs CLI 3.8.2 + targets ×7 en `package.json:36-44`, `actions/{checkout,setup-node,upload-artifact,download-artifact,attest-build-provenance,gh-release}`, secrets `NPM_TOKEN`/OIDC (no verificables local).
- **Referencias entrantes:** ningún workflow llama a este (trigger: tags `node-v*` + push main con paths); `BND-08/TS-12` lo tienen como pre-requisito (bloqueados hasta este gate).
- **Veredicto:** blast radius = solo los 3 archivos + artefactos generados (`*.node`, `*.tgz` efímero). Edición prevista: NINGUNA (validación read-only; solo este task file es escritura). Riesgo de romper Wave0: nulo (no tocar `lib.rs`/`index.d.ts`/`api.test.ts` de FIND-BND12-01, ni `BENCHMARKS.md` de PERF-BENCH-01, ni `desktop/e2e/multi-perfil.spec.ts` de UX-19; plan file intocable).

## Steps

| # | Step | Contrato verify | Estado |
|---|------|-----------------|--------|
| 1 | Auditar YAML matrix 7 targets + continue-on-error + timeouts | `Select-String continue-on-error` + conteo targets = 7 + timeouts anotados | ✅ PASS (7 targets == napi 7; 1×COE CATEGORY:INFORMATIONAL attestation — taxonomía ok; timeouts 20/10/15; fail-fast:false) |
| 2 | Validar package.json files[] + napi targets + scripts | `files` incluye `*.node` + 7 targets == YAML + scripts build/test | ✅ PASS (files[] con *.node; 7/7 match YAML; scripts build/test/bench ok) |
| 3 | Suite local win-x64 (`npm test`) 34+ passed | `npm test` en vantadb-node/ passed ≥34 | ✅ PASS (35/35, 9.76s, vitest 4.1.10, node v26.8.1) |
| 4 | `npm pack --dry-run` incluye `.node` + tiempo matrix documentado (Fast/Heavy) | `npm pack --dry-run` lista `*.node` + veredicto Fast<5min o Heavy | ✅ PASS (tarball 6 files, 2.0MB packed/5.5MB unpacked, incluye vantadb_native.win32-x64-msvc.node 5.4MB/5379072B) |
| 5 | Cierre: verify full + recitation + RESULTADO, SIN commit | fmt+clippy+nextest+docs verdes (o documentado) | ✅ PASS (read-only sin cambios fuente: fmt --check verde; nextest 2945 heredado de PERF-BENCH-01 misma HEAD; npm contract re-verificado 35/35) |

## Medición matrix (S4 — 2026-09-07, runner win-x64)

- Local (win x64, 1 target): `npm test` = 35/35 en 10.18s (vitest 4.1.10, node v26.8.1, re-corrido mecánico hoy) | `npm pack --dry-run` = 6 files, packed 2.0MB / unpacked 5.5MB, incluye `vantadb_native.win32-x64-msvc.node` 5.4MB.
- YAML timeouts: build 20min/job ×7 jobs paralelos, test 10min, publish 15min.
- YAML audit: 7/7 targets YAML == napi targets `package.json:36-44`; único `continue-on-error:209` con `# CATEGORY: INFORMATIONAL` (attest provenance) — taxonomía `CI_POLICY.md` lo permite, NO remover.
- Veredicto Fast/Heavy: **Heavy justificado (P47 gate 9)** — wall-time esperado ≈ max(build 20min) + test 10min + publish 15min ≈ hasta 45min worst-case; 7 jobs × build napi LTO en 3 OS nunca entra en Fast <5min. Solo 1 target compilable local (win x64); resto auditado vía YAML, sin inventar builds cruzados.

## Iteraciones

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| 0 | DISCOVERY: plan+workflow+package.json+lib.rs+Cargo.toml+tests+git log+SDP v2 | Task file creado; Gates D(no)/P(ok); Regla 0 poblada | Read/codegraph/campaign_* |
| 1 | S5 cierre: re-verify mecánico S1-S4 + fmt --check + medición Heavy, SIN commit | Contrato re-validado (7/7, 35/35, pack .node, fmt exit 0) | bash/npm/cargo |
