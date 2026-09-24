# VULN-B1-B2 — postcss + vitest bumps en `vantadb-ts` (cierra #26/#40/#41)

> Plan: `docs/dev/plans/2026-09-22-vulnerabilities.md` (Wave B) · Rama: `fix/vuln-ts-060` desde `develop`
> Estado: ⬜ PENDING → ⏳ IN PROGRESS → ✅/❌ · Push: SOLO vanta-lead

## Objetivo

Cerrar gaps #26 (postcss) y #40/#41 (vitest/@vitest/mocker) en `vantadb-ts` con bumps mínimos.
Solo manifiesto/lock — cero cambios de código.

## Contrato (ley — si no se cumple, no está completa)

- (a) `vantadb-ts/package-lock.json` con postcss ≥8.5.23 y vitest/@vitest/mocker ≥4.1.11 (`npm ls` lo prueba)
- (b) `npx tsc --noEmit` exit 0 + `npm test` verde en `vantadb-ts/`
- (c) cero cambios de código (solo manifiesto/lock)
- (d) commit en rama nueva `fix/vuln-ts-060`, SIN PUSH

## Archivos permitidos (scope discipline)

- `vantadb-ts/package.json` (1 línea: vitest `^4.1.10`→`^4.1.11`)
- `vantadb-ts/package-lock.json` (regenerado por npm)
- Este task file. Plan file: solo recitation.

## PROHIBIDOS

`vantadb-ts/src/`, `web/`, `vantadb-node/`, `desktop/`, `src/` Rust, `.github/workflows/`,
`docs/` salvo task file, plan file (edición), `docs/dev/Backlog.md`, secretos, WIP ajeno, publicar.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vantadb-ts/package.json` (73L: vitest `^4.1.10` devDep directo;
  postcss NO es dep directa — llega transitiva), `vantadb-node/package.json` (precedente #196:
  vitest ya en `^4.1.11`), `.opencode/rules/js-ecosystem.md` (completo), plan file Wave B.
- **Lock actual (evidencia `rg package-lock.json`):** `node_modules/postcss` 8.5.15 (L2641),
  `node_modules/vitest` 4.1.10 (L3102), `node_modules/@vitest/mocker` 4.1.10 (L1472),
  `node_modules/@vitest/coverage-v8` 4.1.10 (L1423, fuera de gaps — no se toca salvo que npm lo exija).
- **Hacia dentro (quién trae cada versión — evidencia `npm ls`):**
  postcss@8.5.15 ← vite@6.4.3 ← vite-plugin-wasm@3.6.0 (devDep directo);
  vitest@4.1.10 ← devDep directo `^4.1.10`; mocker@4.1.10 ← vitest.
  (postcss@8.5.25 bajo vantadb-node→vite@8.2.0 ya ≥floor — no es nuestro scope.)
- **Entrantes:** ningún `src/` importa postcss/vitest en runtime (build/test tooling dev-only).
  Codegraph blast radius: solo tipos `vantadb-node` no afectados (disjunto).
- **Veredicto:** impacto mínimo — 2 archivos manifiesto/lock, sin símbolos públicos nuevos,
  sin hot path, sin trust boundary. Rollback: `git revert` del commit único.

## SDP

`campaign_discover_skills_v2` phase=BUILD keywords=[npm, lockfile, postcss, vitest, bump] →
8: campaign-executor, source-driven-development, incremental-implementation,
test-driven-development, context-engineering, doubt-driven-development,
frontend-ui-engineering, api-and-interface-design.
Cargadas: source-driven-development, doubt-driven-development, incremental-implementation.
No cargadas (justificado): frontend-ui-engineering + api-and-interface-design (cero código/UI/API);
test-driven-development + context-engineering (cubiertas por worker lifecycle §1b/§3a);
campaign-executor (base auto-MCP).

## Slices (incremental, 1 path vertical)

- [x] S1 · B2+B1: bump vitest spec + `npm update postcss` + `npm install` → `npm ls` prueba contrato (a)
  ✅ postcss@8.5.28 (≥8.5.23) en vite@6.4.3 y vite@8.3.0; vitest/mocker@4.1.11.
  Colateral: nanoid 3.3.14→3.3.19 (vía postcss ^3.3.7, == blessed Wave-A #199).
- [x] S2 · Verify contrato (b): `npx tsc --noEmit` exit 0 + `npm test` 311/311 (12 files) → commit selectivo (d) + cierre

## Gates

- D: no disparado — bump mecánico, blast radius 2 archivos lock/manifiesto, sin símbolos
  públicos nuevos, contrato no ambiguo. (Sin tool `question` disponible → motivo registrado aquí.)
- V: si verify falla 2× mismo-error → STOP + INCOMPLETO (sin `question` → registra motivo).
- C: commit selectivo solo `vantadb-ts/package.json` + `package-lock.json`; `git status` sin
  ajenos antes de `git add`. Colaterales (ej. coverage-v8) → documentar, no expandir scope.

## Fuentes

GHSA ya verificados en plan file (postcss GHSA-fxqj-rqcc-2cmp CVE-2026-69153;
vitest GHSA-82fw-gwwq-j7x9 CVE-2026-84373). Internet N/A en esta tarea.
