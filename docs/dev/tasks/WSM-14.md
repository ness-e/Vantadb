---
title: "WSM-14: Plan de adopción npm (README + demo Transformers.js + keywords honestas)"
kind: task
description: "Posicionamiento 'browser AI agent memory' en las 2 superficies npm + demo Transformers.js enlazada + keywords en Cargo.toml/package.json + comparativa re-medida y honesta (gap de bundle size declarado)"
---

# WSM-14: Plan de adopción npm (README + demo Transformers.js + keywords honestas)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 26)
- **Fuente:** Backlog `WSM-14` (INV-vantadb-wasm 2026-08-25 → H-21, estrategia aprobada) + master plan Task 26 (expandido F0)
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Tipo:** Docs / npm metadata (TypeScript SDK + WASM crate)
- **Turns estimados:** 15-30
- **Creado:** 2026-10-04
- **last-synced:** 2026-10-04
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas (mecanismo del README publicado resuelto en DISCOVERY — ver Impacto Regla 0)
- **Pendientes (downhill):** 8 steps de ejecución restantes

## SDP (Skill Discovery Protocol v3)
`campaign_discover_skills_v2` phase=BUILD (taskId 26, taskType "TypeScript SDK") → base: **campaign-executor · progreso · source-driven-development** · pinned: **security-and-hardening** (policy trust boundary) · lifecycle BUILD: **incremental-implementation · test-driven-development · context-engineering · doubt-driven-development** · keyword-mapped: shipping-and-launch · git-workflow-and-versioning.

**Cargadas:** `documentation-skill` · `writing-guidelines` · `api-and-interface-design` · `source-driven-development` · `security-and-hardening` (pinned). Base campaign-executor/progreso/ponytail auto-cargadas vía harness.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers (quién consume) | Publish npm `release-npm-61.yml` → `wasm-pack build` (copia README + keywords desde los tracked); npm tarball `vantadb` (`files: README.md`); `docs/api/TS_SDK.md:828` y `BINDINGS_NAMESPACES.md:15` (links a anclas del TS README); `vantadb-wasm/demo/README.md:54` (link `../README.md`); `scripts/docs/check-npm-versions.mjs` (version de `vantadb-ts/package.json`); `vantadb-ts/scripts/smoke-pack.mjs` + `benchmarks/wasm_bench.mjs` (leen `vantadb-wasm/pkg`) |
| Callees (de qué dependen) | `vantadb-wasm/Cargo.toml` (keywords + description → `pkg/package.json`), `vantadb-wasm/README.md` (→ `pkg/README.md`), `vantadb-ts/package.json` (→ tarball npm) |
| Implicaciones | Sin cambios de código. Cambia la **superficie pública npm** (README/keywords/description) — no rompe contratos de API. Anclas existentes de los READMEs preservadas. No toca tests de bindings ni workflows de paridad (DIST-17 en vuelo). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/README.md` (321L), `vantadb-ts/README.md` (393L), `vantadb-wasm/Cargo.toml`, `vantadb-ts/package.json`, `vantadb-wasm/demo/README.md`, `dev-tools/build-wasm-types.mjs`, `dev-tools/fix-wasm-pkg-files.mjs`, `.github/workflows/release-npm-61.yml` (335L), `vantadb-ts/scripts/smoke-pack.mjs`, `vantadb-wasm/pkg/package.json` + `pkg/README.md` (generados).
- **Mecanismo del README publicado (verificado contra fuente oficial de wasm-pack, repo `wasm-bindgen/wasm-pack`):**
  - `src/readme.rs` → *"Copy the crate's README into the `pkg` directory"*: en cada `wasm-pack build`, `vantadb-wasm/README.md` (tracked) se copia a `pkg/README.md`, que es lo que `npm publish` publica como README de `vantadb-wasm`.
  - `src/manifest/mod.rs` → `let keywords = if pkg.keywords.len() > 0 { Some(pkg.keywords.clone()) } else { None };` — las `keywords` de `[package]` en `Cargo.toml` se copian al `pkg/package.json` generado.
  - Evidencia en repo: `pkg/README.md` == `vantadb-wasm/README.md` (SHA256 idéntico, 15567 B) y `pkg/package.json` actual **sin** keywords (Cargo.toml no las tiene) — consistente con la fuente.
  - **→ El mecanismo ES editable desde el repo (tracked README + Cargo.toml); la stop-condition del plan NO dispara.**
  - El `dev-tools/build-wasm.ps1` que el README cita **no existe** (verificado): los scripts reales son `wasm-pack build --release` (CI `publish-wasm`) + `dev-tools/build-wasm-types.mjs` (.d.ts) + `dev-tools/fix-wasm-pkg-files.mjs` (snippets).
- **Referencias entrantes:** `vantadb-ts/README.md` → `../vantadb-wasm/README.md` (§1 y §4); `vantadb-wasm/README.md:130,311` → `vantadb-ts/README.md`; `docs/api/TS_SDK.md:828` → `vantadb-ts/README.md#vantadb-vs-vantadb-node-npm`; `docs/api/BINDINGS_NAMESPACES.md:15` → `vantadb-ts/README.md#domain-sub-clients`; `vantadb-wasm/demo/README.md:54` → `../README.md`; `release-npm-61.yml:80-219` + scripts leen/parchean `pkg/`.
- **Veredicto impacto:** **bajo** — docs + metadata npm; cero código; únicos efectos: README/keywords/description de las 2 superficies npm + anclas preservadas. `check-npm-versions` no se afecta (no se toca `version`).

## Contrato
"README npm con posicionamiento 'browser AI agent memory' + demo Transformers.js enlazada + keywords/comparativa honesta (incluye el gap de bundle size, sin ocultarlo); **sin claims de performance sin benchmark** (Regla 11); smoke del paquete (npm pack / carga en Node+script) verde; `check-links`/`check-docs` exit 0."

## Spec (decisiones — docs/metadata; no agrega símbolos públicos)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Superficie del README publicado | A) editar `pkg/` (gitignored — se pierde en el próximo build) / B) editar el tracked + Cargo.toml y dejar que wasm-pack propague | ✅ **B** — evidencia: `wasm-pack/src/readme.rs` + `src/manifest/mod.rs` + hash idéntico `pkg/README.md`==tracked |
| 2 | Link del demo (debe renderizar en npmjs.com) | A) relativo `../vantadb-wasm/demo/` (npmjs puede dejarlo roto; no verificable — 403 al scrape) / B) absoluto GitHub `tree/main/vantadb-wasm/demo` | ✅ **B** — robusto en GitHub y npm; `origin/main` contiene `vantadb-wasm/demo` (verificado con `git ls-tree`) |
| 3 | Keywords `vantadb-wasm` (Cargo.toml → npm) | set ≤5 (límite crates.io, aunque el crate es `release=false`) | ✅ `agent-memory`, `browser`, `wasm`, `vector-database`, `local-first` |
| 4 | Keywords/description `vantadb` (TS) | ampliar el set actual (8) con posicionamiento | ✅ + `browser`, `agent-memory`, `local-first`, `hybrid-search`; description con "browser AI agent memory" |
| 5 | Números del bundle size | A) mantener medición fechada 2026-09-15 (no coincide con el `pkg/` actual en disco: 1.86 MB) / B) re-medir con el rebuild del smoke y fechar (Regla 11) | ✅ **B** — re-medición con fecha + comando repro; el gap se declara, no se oculta |
| 6 | Referencias stale a `dev-tools/build-wasm.ps1` | fix dentro del scope (archivo inexistente citado 2×) | ✅ corregir al mecanismo real (`wasm-pack build --release` + scripts) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  - `vantadb-wasm/pkg/` es **generado** — nunca hand-edit (excepción documentada: `node dev-tools/build-wasm-types.mjs` para el `.d.ts`).
  - No tocar `vantadb-ts/src/__tests__/**`, `vantadb-python/tests/**`, `vantadb-node/tests/**` ni workflows de paridad (DIST-17 en vuelo).
  - Anclas de `vantadb-ts/README.md` usadas por docs/api: `#vantadb-vs-vantadb-node-npm` y `#domain-sub-clients` deben seguir existiendo.
  - Sin claims de performance/adopción sin fecha + fuente (Regla 11).
  - Commit **LOCAL**; nunca push.
- **Comandos de verificación:** smoke (Step 7) + `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs` + `node scripts/docs/check-npm-versions.mjs` + `pwsh dev-tools/verify_changed.ps1` (+ `verify.ps1` fast tier al cierre).
- **Deuda pendiente:** ninguna al cierre (si el re-measure revela drift relevante, queda documentado en el README).

## Steps

### Step 1: DISCOVERY (mecanismo publish + blast radius + SDP + task file)
- **Archivos:** (lectura) ver Impacto Regla 0
- **Acción:** localizar el mecanismo real del README publicado; mapear referencias; SDP; crear task file.
- **Verify:** mecanismo verificado contra fuente oficial wasm-pack + hash `pkg/README.md`==tracked; `check_index_coverage` sin issues.
- **Estado:** ✅ COMPLETED

### Step 2: Rebuild `pkg/` + re-medición (precondición del smoke)
- **Archivos:** `vantadb-wasm/pkg/` (generado, gitignored)
- **Acción:** `wasm-pack build --release` desde `vantadb-wasm/`; medir raw + gzip de `vantadb_wasm_bg.wasm` / `_bg.js` / `vantadb_wasm.js` (método del README §1); correr `node dev-tools/build-wasm-types.mjs` + `node dev-tools/fix-wasm-pkg-files.mjs`.
- **Verify:** números nuevos capturados (raw 1,856,638 / gz 745,152; total transfer ~739 KB); `pkg/` no ensucia tracked.
- **Estado:** ✅ COMPLETED (rebuild inicial + post-edits + post-review-fixes; hash final verificado en Step 7)

### Step 3: Editar `vantadb-wasm/README.md`
- **Archivos:** `vantadb-wasm/README.md`
- **Acción:** (a) posicionamiento "browser AI agent memory" + demo Transformers.js enlazado (link absoluto) arriba; (b) corregir referencias stale a `dev-tools/build-wasm.ps1` (2×) al mecanismo real; (c) documentar que `pkg/README.md` + keywords se generan desde tracked; (d) actualizar §1 (tabla + staleness note) y §4 (fila VantaDB: versión + números) con la re-medición fechada; (e) footer last reviewed.
- **Verify:** `git diff` solo ese archivo; anchors existentes intactos; sin claims sin fecha/fuente. ✅ (+ fixes R1: 671KB→739KB, 1.58→1.77 MB ×2, repro `Get-Item`; + R2: anchor `#-5-impact-…` con leading dash)
- **Estado:** ✅ COMPLETED

### Step 4: Editar `vantadb-wasm/Cargo.toml`
- **Archivos:** `vantadb-wasm/Cargo.toml`
- **Acción:** `keywords = ["agent-memory", "browser", "wasm", "vector-database", "local-first"]` + description con posicionamiento.
- **Verify:** `cargo metadata`/parse OK; próximo build copia keywords a `pkg/package.json`. ✅ verificado: `["agent-memory","browser","wasm","vector-database","local-first"]` en `pkg/package.json`.
- **Estado:** ✅ COMPLETED

### Step 5: Editar `vantadb-ts/README.md`
- **Archivos:** `vantadb-ts/README.md`
- **Acción:** (a) posicionamiento "browser AI agent memory" + demo enlazado (link absoluto) en el header; (b) corregir "~1.3 MB" → medición fechada; (c) fix del anchor roto `#1-bundle-sizes-measured-2026-08-30` → heading real; (d) tabla de comparación: fila VantaDB (versión + números) y ratio; (e) preservar anclas `#vantadb-vs-vantadb-node-npm` y `#domain-sub-clients`.
- **Verify:** `git diff` solo ese archivo; `docs/api/TS_SDK.md` y `BINDINGS_NAMESPACES.md` siguen resolviendo (check-links). ✅ (+ fix R1 tagline OPFS/`js-ecosystem` R-1; + cita p99 BENCHMARKS)
- **Estado:** ✅ COMPLETED

### Step 6: Editar `vantadb-ts/package.json`
- **Archivos:** `vantadb-ts/package.json`
- **Acción:** description con "browser AI agent memory"; keywords + `browser`, `agent-memory`, `local-first`, `hybrid-search`; **no tocar** `version`/`engines`/`dependencies`.
- **Verify:** JSON válido (`node -e "require(...)"`); `check-npm-versions.mjs` exit 0. ✅ (12 keywords; version 0.8.0 intacta)
- **Estado:** ✅ COMPLETED

### Step 7: Smoke del artefacto publicado
- **Archivos:** (generados) `vantadb-wasm/pkg/`
- **Acción:** rebuild tras edits → verificar `pkg/README.md` == tracked (SHA256) + `pkg/package.json` con keywords/description → `npm pack --dry-run` en `pkg/` (README en tarball) → `npm run build` + `node scripts/smoke-pack.mjs` en `vantadb-ts` (pack + install + quickstart Node).
- **Verify:** hash idéntico + keywords presentes + smoke TS-07 PASSED. ✅ (`pkg/README.md` == tracked; keywords; `[smoke] PASSED (vantadb-0.8.0.tgz)`; `npm pack` incluye README)
- **Estado:** ✅ COMPLETED

### Step 8: Gates mecánicos
- **Archivos:** —
- **Acción:** `node scripts/docs/check-links.mjs` && `check-docs.mjs` && `check-npm-versions.mjs`; `npx tsc --noEmit` (TS); `pwsh dev-tools/verify_changed.ps1`; fast tier `pwsh dev-tools/verify.ps1` (ALL PASS).
- **Verify:** todos exit 0. ✅ check-links 0 broken · check-docs all clear · check-npm-versions OK · tsc 0 · vitest 342/342 · verify_changed 4/4 · verify.ps1 ALL 10 PASS
- **Estado:** ✅ COMPLETED

### Step 9: Cierre (OCR + review P2-01 + commit + campaign)
- **Archivos:** `docs/dev/tasks/WSM-14.md`
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory); fork a `vanta-review` (Fast tier — paths docs/metadata); commit LOCAL `docs(npm): WSM-14 — ...`; `campaign_update_task_state taskId 26 completed` con recitation + review payload.
- **Verify:** OCR sin Critical/High (rules-only, 2 reviewables: package.json + Cargo.toml, sin findings); veredicto review registrado (R3 APPROVE); commit hash capturado.
- **Estado:** ✅ COMPLETED — commit `0c3e465e` (4 archivos, path-scoped; WIP ajeno excluido)

## Review (GATE — agente distinto, P2-01)

> Tier: **Fast** (paths del diff: `vantadb-wasm/README.md`, `vantadb-wasm/Cargo.toml`, `vantadb-ts/README.md`, `vantadb-ts/package.json` — ninguno matchea globs adversariales).

- **Revisor:** ✅ `vanta-review` — segunda opinión P2-01 en contexto fresco (sesión distinta al implementador, sin participación en los edits).
- **Enfoque:** verificación adversarial del changeset docs/metadata npm: (a) mecanismo de propagación wasm-pack contra fuente upstream (`master` de `wasm-bindgen/wasm-pack`); (b) re-medición independiente de números con el comando documentado; (c) re-ejecución de smoke TS-07 + gates; (d) anclas/enlaces; (e) superficie pública real (registry npm + API del SDK); (f) regla de área `js-ecosystem` R-1; (g) scope/prohibidos.
- **Cómo se probó:** `Get-FileHash` → `pkg/README.md` == tracked (`BE469D14…785B8`) ✓ · `cargo read-manifest` → description+keywords de `Cargo.toml` == `pkg/package.json` (match exacto) ✓ · `npm pack --dry-run` en `pkg/` incluye `README.md` (16 547 B) ✓ · upstream wasm-pack `src/readme.rs` + `src/manifest/mod.rs` (rama `master`) confirman copia de README/keywords/description ✓ · `wasm-pack 0.15.0` ✓ · re-medición con el comando del README: gzip 745 152 / 11 265 / 156 ✓ (idéntico a la tabla), raw `Get-Item` 1 856 638 / 58 519 / 245 ✓ · `node scripts/smoke-pack.mjs` → `[smoke] PASSED (vantadb-0.8.0.tgz)` exit 0 ✓ · `check-links` 0 broken exit 0 ✓ · `check-docs` gating all clear exit 0 ✓ · `check-npm-versions` exit 0 ✓ · registry: `vantadb@0.8.0` (latest) + `vantadb-wasm@0.8.0` ✓ · anchor `#1-bundle-sizes-measured-2026-10-04` (×2) correcto, `#vantadb-vs-vantadb-node-npm`/`#domain-sub-clients` preservadas, 0 refs al anchor viejo ✓. No re-ejecutados (diff sin código): `tsc`/`vitest`/`verify_changed.ps1` — fmt/check/clippy corren sobre `-p vantadb` y no consumen metadata de wasm; manifest validado aparte con `cargo read-manifest`.
- **Veredicto:** 🔴 **REQUEST CHANGES** — 0 Critical · 4 Required:
  1. `vantadb-wasm/README.md:254` — "671 KB gzipped" stale vs ~739 KB de la misma §4 (`:244`/`:250`). Fix: 739 KB.
  2. `vantadb-wasm/README.md:309` y `:313` — "1.58 MB" stale vs 1.77 MB re-medido. Fix: 1.77 MB (×2).
  3. `vantadb-wasm/README.md:48` — comando repro raw roto: `Get-ChildItem -Filter` no acepta array (verificado: `No se puede convertir "System.Object[]" al tipo "System.String"`). Fix: `Get-Item <3 paths> | Select-Object Name, Length`. Línea pre-existente, pero la sección fue re-certificada por este cambio y el contrato cita "comando repro".
  4. `vantadb-ts/README.md:3` — tagline nuevo "with OPFS persistence" sin método habilitante: viola `js-ecosystem` R-1 (MUST) y no es alcanzable desde la API pública del SDK (`Client.create/connect/open`; `connect_persistent/idb/worker` solo existen en el binding raw `vantadb-wasm`). Fix: calificar (p. ej. "OPFS via the raw `vantadb-wasm` binding — `Client.connect_persistent()`") o quitar.
  Optional: cita de fuente para "sub-ms at 100K" (`vantadb-wasm/README.md:257`, `vantadb-ts/README.md:167` — BENCHMARKS §5 lo respalda: 441.2 µs p50 balanced @100K; falta el link) · footer `vantadb-wasm/README.md:335` ("size only changes when Cargo.toml features/Cargo.lock deps change") contradice la nota de provenance "toolchain + code" · `vantadb-wasm/demo/README.md:38-39,56-59` números stale (pre-existente, follow-up FIND-75).
  Revisión delta tras fixes: re-chequear los 4 puntos; si se toca el README de `vantadb-wasm`, re-build/re-hash `pkg/` (propaga a la superficie npm).
- **R2 — revisión delta (post-fixes):** Required 1-4 ✅ cerrados: (1) `:254` → `~739 KB gzipped transfer`; (2) §6 → `1.77 MB` ×2; (3) repro → `Get-Item` re-ejecutado (1 856 638 / 58 519 / 245 exacto); (4) tagline sin OPFS + demo line califica `connect_persistent()` desde el raw binding (verificado `demo/app.js:150`; el SDK no lo expone). Optional: sub-ms ahora cita BENCHMARKS §5 (p99 441 µs — correcto: la columna §5 es **p99**, no p50), footer alineado con "code, toolchain, and Cargo.toml/Cargo.lock". Mecánica: `pkg/README.md` == tracked (`4B1F2964C70BEEA5CF5AD566A86A6B25A6A5E8A1868765E5EAA74050C1E8B805`), keywords intactas, greps stale 0/0, `check-links`/`check-docs` exit 0, repo `visibility=public`.
  **Veredicto R2:** 🔴 **REQUEST CHANGES** — 1 Required nuevo: anchor roto en ambos READMEs. El link cita `#5-impact-of-loop-and-hnsw-distance-optimization-phase-2`, pero el heading real es `## 🚀 5. Impact…` → anchor GitHub `#-5-impact-of-loop-and-hnsw-distance-optimization-phase-2` (**leading dash**, verificado en el HTML renderizado del blob `main` y consistente con `docs/user/COMPARISON.md:60,92` que usa `#-1-…`, `#-6-…`). Fix: agregar el `-` inicial en `vantadb-wasm/README.md:257` y `vantadb-ts/README.md:167`. (`check-links` no lo detecta: links absolutos → contados como externos y skipped.)
- **R3 — confirmación final (post-fix anchor):** ✅ verificado — `#-5-impact-of-loop-and-hnsw…` presente en `:257` y `:167` (2 hits), 0 hits de `BENCHMARKS.md#5-impact` sin dash; `pkg/README.md` == tracked == `23B913607B2D43B39B82D4388A83EB45EF944EEB7CED82619534ECBE1B940A31`; keywords intactas. **Veredicto final: ✅ APPROVE** (0 Critical / 0 Required abiertos; Optionals aplicados salvo `demo/README.md` stale — follow-up FIND-75 fuera de scope).

## Context Save Point

- **Iteración 1 (2026-10-04):** DISCOVERY completo. Mecanismo del README publicado resuelto (wasm-pack copia README + keywords desde tracked; `build-wasm.ps1` no existe). Task file creado. Steps 2-8 ejecutados: rebuild + re-medición (1.77 MB raw / ~739 KB transfer), edits en 4 archivos, smoke TS-07 PASSED, gates todos verdes, `verify.ps1` ALL 10 PASS.
- **Iteración 2 (2026-10-04):** review P2-01 (vanta-review, contexto fresco) → R1 REQUEST CHANGES (4 Required + Optional) → fixes aplicados → R2 REQUEST CHANGES (1 Required: anchor con leading dash por emoji del heading) → fix aplicado → R3 (confirmación) → commit + cierre.

## RESULTADO §7 (a completar en el cierre)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 9/9 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 0c3e465e (docs(npm): WSM-14)
ARCHIVOS: vantadb-wasm/README.md, vantadb-wasm/Cargo.toml, vantadb-ts/README.md, vantadb-ts/package.json, docs/dev/tasks/WSM-14.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no (plan expandido, contrato claro) D:no (docs/metadata, spec por evidencia) V:no (verify verde, review en rondas) C:no (sin colaterales; WIP ajeno excluido)
SKILLS_CARGADAS: documentation-skill, writing-guidelines, api-and-interface-design, source-driven-development, security-and-hardening (pinned)
```
