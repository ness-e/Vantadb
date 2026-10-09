---
title: "TASK FIND-231: Cubrir el combo release (server+allocator, -D warnings) en CI"
kind: task
description: "Job release-combo en ci-rust.yml: compila en perfil release el combo exacto que quemó el primer run de release-binaries 0.8.0 (server+allocator, RUSTFLAGS=-D warnings) — verde con el árbol actual, rojo con el fix 9004c43f revertido (repro local documentada)"
---

# TASK FIND-231: Cubrir el combo release (server+allocator, -D warnings) en CI

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 6, Wave 2 — última tarea de ejecución)
- **Fuente:** `docs/dev/Backlog.md` (FIND-231) + plan post-release 0.8.0
- **Esfuerzo:** 🟡 3-4h | **Appetite:** 1d
- **Prioridad:** 🟠 Media
- **Tipo:** CI/CD-DevOps (job nuevo en `ci-rust.yml`) — blast radius: 1 workflow + 1 fila de policy + task file + 2 índices regenerados (`docs/index.md`, `llms.txt` — artefactos generados); cero código de producción
- **Turns estimados:** 15-30
- **Creado:** 2026-10-03 | **last-synced:** 2026-10-03
- **Estado:** ✅ COMPLETED (commit local de cierre; hash en recitation)
- **Campaign ID:** post-release-0.8.0-20261002
- **Incógnitas (uphill):** 3 → 0 (check vs build resuelto por medición; matriz OS decidida por evidencia; `-D warnings` verificado contra action source)
- **Pendientes (downhill):** 0 (Steps 1-3 ✅; post-push tramo 2 del contrato queda en orquestador/owner)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/ci-rust.yml` — workflow que corre en push a `main`, PR a `main` y `workflow_dispatch` (triggers ya existentes, sin cambio). Consumidor nuevo: el job `release-combo`. Ningún otro workflow/job lo invoca. |
| Callees | `./.github/actions/rust-setup` (dtolnay/rust-toolchain pinned + Swatinem/rust-cache + sccache v0.16.0) → `cargo check --release` de `vantadb --bin vanta-cli` (features `server,jemalloc`) y `vantadb-server` (feature `jemalloc`). Replica los comandos de `release-binaries.yml:121-122`. |
| Implicaciones | Aditivo puro: 1 job nuevo, 0 cambios de comportamiento en jobs existentes; +2 artefactos generados (`docs/index.md`, `llms.txt`) regenerados por `gen-index` (DoD de documentación; incluyen la entrada FIND-227 que faltaba). Sube el wall-time del Fast Gate (medido localmente; ver §Medición) — jobs corren en paralelo, el job queda fuera del critical path actual. No toca API pública, wire format, performance ni datos. `opencode.jsonc` (WIP ajeno) fuera del commit. |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/ci-rust.yml` (734L — 20 jobs inventariados; el job nuevo se inserta después de `experimental-check`, antes de `audit`), `.github/workflows/release-binaries.yml` (156L — referencia L109-122: comandos exactos + selector de allocator por OS), `.github/actions/rust-setup/action.yml` (100L — NO setea `RUSTFLAGS`, ver §Evidencia), `docs/dev/operations/CI_POLICY.md` (§1 Fast Gate — tabla de jobs + presupuesto `<5 min`), `.opencode/rules/release-ci.md` (42L — Reglas 1/2/5), `docs/dev/tasks/FIND-227.md` (formato de task file CI), `.opencode/task-system/prompts/task.md` (formato canónico).
- **Referencias entrantes (grep):** `ci-rust` aparece en 11 docs (`ci-cd-guide.md`, `ci-rust-10.md`, `CI_POLICY.md`, `FAQ.md`, `README.md`, `RULES.md`, `RUNBOOK.md`, `TRIGGERS.md`, `TEST_MAP.md`, `gate-api-docs.md`, `release-ci.md`); `TRIGGERS.md` y `workflow/README.md` son workflow-level (sin inventario de jobs → sin cambio); `CI_POLICY.md` §1 lista jobs del Fast Gate → se agrega 1 fila. `ci-rust-10.md` (runbook) ya está stale (dice "13 jobs"; hoy hay 20) — drift pre-existente, fuera de scope (ver §Notas).
- **Referencias salientes de los editados:** `ci-rust.yml` → actions pinneadas por SHA, `rust-setup`, `.config/nextest.toml`; `release-binaries.yml` → solo lectura (referencia).
- **Veredicto impacto:** **BAJO / aditivo.** `git revert` de 1 commit restaura el estado previo. No se toca ningún job existente (ni `sanitizer-asan`/`sanitizer-tsan` — cerrados en FIND-226/227 —, ni `perf-bench.yml`/`benchmarks/**` — FIND-232 —, ni los 15 workflows de FIND-228, ni `gate-docs*` — FIND-230).

## Evidencia dura

### 1. El run rojo original (release v0.8.0) y su causa

- **Run:** `37045939672` (evento `release`, tag `v0.8.0`, 2026-10-02) — **5/5 jobs de build fallaron** (+ job Docker, retirado después) + Tests OK. Job x86_64-linux: `110971572674`; Windows: `110971572698`; aarch64-linux: `110971572852`; macOS: `110971572561`/`110971572871`.
- **Error (idéntico en los 5 OS):** en `src/sdk/search/debug_ops.rs` — `unused imports: validate_metadata and validate_namespace` (L2), `unused import: super::super::types::*` (L5), `super::debug` (L6), `crate::backend::BackendPartition` (L7), `Error and Result` (L10); y en `src/sdk/search/fusion.rs:72` — `function fuse_rrf is never used`. Todos con `note: -D unused-imports implied by -D warnings`.
- **Mecánica:** `debug_ops` tiene TODOS sus métodos gated por `#[cfg(debug_assertions)]`; en perfil release (debug_assertions=off) el módulo queda sin items que usen sus imports → unused-imports → error por `-D warnings`. `fuse_rrf` queda unreachable (dead_code). Los tests corren en debug → **ningún job compilaba el perfil release**.
- **Fix:** `9004c43f` (2026-10-02, 2 líneas): `#[cfg(debug_assertions)]` en `src/sdk/search/mod.rs:15` (`mod debug_ops`) y `src/sdk/search/fusion.rs:72` (`fuse_rrf`). Verificado en el árbol actual: `git show 9004c43f` presente.
- **Run verde post-fix:** `37082059543` (dispatch backfill, 5/5 targets success; x86_64-linux job total **463s** — build full SIN sccache, dato de costo).
- **Nota de precisión (E0080):** el E0080 mencionado en el pedido corresponde a FIND-184 (`d4d7961a`, assertion compile-time en `vanta-memory/tests/smoke.rs` bajo feature-unification de tests) — clase adyacente ya resuelta en otra lane; NO es reproducible revirtiendo `9004c43f` y no es alcanzable por este job (checks de binarios, no de test targets). La repro de este task captura la clase del release-binaries (unused-imports + dead_code).

### 2. Ningún job actual compila el combo release **bajo `-D warnings`**

- `rg -n "release" .github/workflows/ci-rust.yml` → solo comentarios (semver-checks, sanitizers); todos los `cargo check`/`nextest` de `ci-rust.yml` corren en debug y sin allocator.
- **Corrección de precisión (review ronda 1, F1):** `desktop.yml` SÍ compila el combo exacto — `:78-79` (Windows, `custom-allocator`) y `:150-151`/`:216-217` (macOS/Linux, `jemalloc`) — pero vía `./rust-setup` (dtolnay, `:59/134/194`) → **sin `RUSTFLAGS`** → sin `-D warnings`. Por eso el 0.8.0 no falló ahí: el gate de warnings del release lane no estaba presente. El diferenciador real es **release profile + `-D warnings`** (los tests van en debug; clippy va en debug con `--all-features` y por eso no ve el módulo gated).
- Los comandos exactos del release (`release-binaries.yml:109-122`): `cargo build --release --target ${{ matrix.target }} --features "server,$ALLOC_FEATURES" --package vantadb --bin vanta-cli` + `cargo build --release --target ${{ matrix.target }} --features "$ALLOC_FEATURES" --package vantadb-server`, con `ALLOC_FEATURES`: Windows=`custom-allocator` (mimalloc), Linux/macOS=`jemalloc`.

### 3. De dónde sale `-D warnings` (validado contra action source, 2026-10-03)

- `release-binaries.yml:80` usa `actions-rust-lang/setup-rust-toolchain@166cdcfd…` → su `action.yml` (pin) declara `rustflags: default: "-D warnings"` → setea `RUSTFLAGS`. **De ahí venía el gate del run rojo.**
- `./.github/actions/rust-setup/action.yml:61-65` usa `dtolnay/rust-toolchain@fa04a145…` → su `action.yml` (pin) **NO setea `RUSTFLAGS`** (solo CARGO_INCREMENTAL/TERM_COLOR/sparse). **Consecuencia de diseño:** el job nuevo debe declarar `RUSTFLAGS: "-D warnings"` explícito para replicar el gate.

### 4. Repro RED (fix revertido en scratch local) — captura cruda

**Método:** edit temporal de las 2 líneas de `9004c43f` (removidas `#[cfg(debug_assertions)]` de `src/sdk/search/mod.rs:14` y `src/sdk/search/fusion.rs:71` en working tree) → comandos exactos del job (features `custom-allocator` local; CI usa `jemalloc`) → restauración con `git checkout -- src/sdk/search/mod.rs src/sdk/search/fusion.rs`.

**Resultado (log: `%TEMP%\opencode\find231\repro-red.log`):**

| Comando (réplica exacta del job) | Exit | Errores capturados |
|---|---|---|
| `cargo check --release --features "server,custom-allocator" --package vantadb --bin vanta-cli` | **101** | 6 errores: unused-imports ×5 en `src/sdk/search/debug_ops.rs` (L2 `validate_metadata`+`validate_namespace`; L5 `super::super::types::*`; L6 `super::debug`; L7 `crate::backend::BackendPartition`; L10 `Error`+`Result`) + `function fuse_rrf is never used` (`fusion.rs:72`) |
| `cargo check --release --features "custom-allocator" --package vantadb-server` | **101** | idénticos (lib `vantadb` compilada como dependencia) |

**Match con el run original:** los 6 errores + la nota `` `-D unused-imports` implied by `-D warnings` `` son idénticos a los del run `37045939672` (jobs 110971572674 / 110971572698 / 110971572852 / 110971572561 / 110971572871) → la repro reproduce **la clase exacta** que quemó el release 0.8.0. (E0080 no aparece: pertenece a FIND-184/`d4d7961a` — otra clase, otra lane; §Evidencia 1.)

**Restauración:** `git checkout --` → `git status --short` sin modificaciones en `src/` ✅; el re-run en verde (§5, warm) confirma el árbol restaurado.

**Nota de validez:** `cargo check --release` reproduce la clase porque el perfil release apaga `debug_assertions` — prueba empírica de que el slice elegido cubre el fallo (no requiere full build para el diagnóstico).

### 5. Medición local (dimensionar el job)

**Host:** Windows 11 Pro 10.0.26200 · Intel i5-1235U (10C/12T) · 31.8 GB RAM · cargo/rustc 1.95.0 · `RUSTFLAGS=-D warnings` · features `custom-allocator` (Windows local; CI usa `jemalloc` — la clase es de perfil, no de allocator). Logs: `%TEMP%\opencode\find231\{measure-green,warm-green}.log`.

| # | Comando (réplica del job, features del release) | Resultado | Tiempo |
|---|--------------------------------------------------|-----------|--------|
| 1 | `cargo check --release --features "server,custom-allocator" --package vantadb --bin vanta-cli` | exit 0 | **123.8s** (cold) |
| 2 | `cargo check --release --features "custom-allocator" --package vantadb-server` | exit 0 | **80.8s** (delta; deps compartidas) |
| — | **CHECK total (job candidato, cold)** | **exit 0** | **204.6s = 3.4 min** |
| 3 | `cargo build --release --features "server,custom-allocator" --package vantadb --bin vanta-cli` | exit 0 | **503.7s = 8.4 min** (cold; thin LTO + codegen-units=1) |
| 4 | build server | interrumpido (no aporta: [3] solo ya excede el presupuesto) | — |
| — | **CHECK re-run warm post-restore** (ambos) | exit 0 | **24.2s + 24.9s** |

**Contexto CI (evidencia):** run post-fix `37082059543` — job x86_64-linux `463s` (build full + package, rust-cache, **sin sccache**). Critical path actual del workflow (run `37082038794`): Tests (Windows) **2655s** → el job nuevo no extiende el wall-time (jobs en paralelo); suma runner-minutes (~3-4 min cold / menos warm con sccache).

**Decisión (stop-condition del plan):** **`cargo check --release`** (no full build). Rationale: (a) local cold 3.4 min ≤ presupuesto — CI warm ≤ cold, dentro de la meta 2-4 min; (b) el build full local del primer binario = 8.4 min y en CI = 463s job sin sccache → >5 min warm por diseño; (c) la clase de FIND-229 es de **diagnóstico** (lints bajo `-D warnings` con debug_assertions off) — `cargo check` ejecuta exactamente esos diagnósticos y la repro RED (§4) lo prueba empíricamente. El build full real sigue siendo responsabilidad de `release-binaries.yml` en release-time; este job es el pre-gate barato.

## Contrato (del plan)

> **el job nuevo verde en CI con el árbol actual Y rojo si se revierte el fix** (repro local documentada en este task file).

| Tramo | Condición | Verificación |
|-------|-----------|--------------|
| Local (pre-commit) | (a) `actionlint` exit 0; (b) comando(s) del job **verde** con el árbol actual; (c) repro **roja** con el fix revertido en scratch + restauración limpia (`git status` sin residuos); (d) tiempo medido | comandos en §Steps |
| Post-push (orquestador/owner) | job `release-combo` **verde** en el próximo run de ci-rust | `gh run list --workflow=ci-rust.yml --limit 1` → `gh run view --job <release-combo-job-id>` → conclusion `success` (comandos exactos en §Steps) |

## Spec (SDD — decisiones técnicas con evidencia)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | `cargo check` vs `cargo build` | build full (cubre link; costo ~463s CI sin sccache) / check release (cubre la clase de diagnóstico; menor costo) | medir local y elegir con stop-condition del plan (>5 min warm → check) | ✅ decidido-por-evidencia: **check** — local cold 3.4 min ≤ presupuesto; build CI 463s job (sin sccache) → riesgo >5 min warm; repro RED prueba que check reproduce la clase (§Medición + §4) |
| 2 | Matriz OS | ubuntu (el que falló; barato) / ubuntu+windows (cubre mimalloc) / 5 targets | ubuntu | ✅ decidido-por-evidencia: la clase es de **perfil** (release + cfg), no de OS — idéntica en los 5 targets del run rojo; Windows/mimalloc queda cubierto por `release-binaries` mismo + `test-windows` smoke |
| 3 | `-D warnings` | explícito `RUSTFLAGS` / confiar en el default del action | explícito | ✅ decidido-por-evidencia: `rust-setup` (dtolnay) NO lo setea (action.yml pin, §Evidencia 3) |
| 4 | `needs:` | ninguno / `[fmt, clippy]` | `[fmt, clippy]` | ✅ decidido-por-convención: todos los jobs pesados de `ci-rust.yml` (test/coverage/miri/sanitizers) usan `needs: [fmt, clippy]` |
| 5 | Scope features | `server,jemalloc` + `jemalloc` (exactos del release) / ampliar | exactos | ✅ decidido-por-contrato: "replicar EXACTAMENTE los features del release" |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) no tocar jobs existentes de `ci-rust.yml` (en particular `sanitizer-asan`/`sanitizer-tsan`); (2) no tocar `release-binaries.yml` (solo referencia); (3) no tocar `opencode.jsonc` (WIP ajeno), plan file ni `docs/pipeline-state.json`; (4) el job NUNCA lleva `continue-on-error` (es gate — Regla 2); (5) commit LOCAL, sin push (política owner); (6) no alterar triggers de `ci-rust.yml`.
- **Comandos de verificación:** `actionlint .github/workflows/ci-rust.yml` (exit 0) · comando(s) del job en verde local (ver §Steps) · repro RED capturada y restaurada · `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs` (exit 0).
- **Deuda pendiente:** verificación post-push del job en CI (tramo 2 del contrato — orquestador/owner); drift pre-existente de `ci-rust-10.md` (fuera de scope, §Notas).

## Deuda técnica (Regla 6)

**Saldo neto de deuda por PR:** Sin deuda (0 dependencias nuevas; +1 job +1 fila de docs).

> El job REDUCE deuda: cierra el gap que quemó el ciclo de release 0.8.0 (FIND-229) con un gate mecánico de 2-4 min.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato local completo: actionlint ✅ + job verde local ✅ + repro RED documentada ✅ + medición ✅ |
| **Commit** | `ci: FIND-231 — …` atómico, verify mecánico local, commit LOCAL sin push |
| **Release** | n/a (job de CI; el gate de release real sigue siendo `release-binaries.yml` post-fix) |

## Herramientas necesarias

- `cargo`/`rustc` 1.95.0 local (medición + repro), `actionlint` 1.7.12, `gh` CLI (evidencia de runs), `node scripts/docs/*.mjs` (docs), OCR delegation (`dev-tools/ocr-review.ps1`), PowerShell (captura de logs en `%TEMP%\opencode\find231\`).

**Skills cargadas (SDP):** `campaign-executor` + `progreso` (base, auto), `ci-cd-and-automation` (pinned CI), `git-workflow-and-versioning` (pinned CI), `doubt-driven-development` (base type devops), `incremental-implementation` + `test-driven-development` + `context-engineering` (lifecycle BUILD), `documentation-skill` (obligatoria por crear/editar `.md` bajo `docs/`), `coordinated-web-search` (router — usado para validar los defaults de las actions contra su source pinneado).

## Investigation Notes

- **Web research (liviano, inline):** defaults de `-D warnings` verificados contra el `action.yml` **en el SHA pinneado** de cada action (raw.githubusercontent.com) — no de memoria: `actions-rust-lang/setup-rust-toolchain@166cdcfd` → `rustflags: default: "-D warnings"`; `dtolnay/rust-toolchain@fa04a145` → sin RUSTFLAGS. Sin red no habría sido posible fijar el diseño del `env:` — ambas URLs resueltas 2026-10-03.
- **Semántica `cfg(debug_assertions)`:** `cargo check --release` selecciona el perfil release (debug-assertions=off) → la repro RED con el comando exacto del job es la prueba empírica de que el slice elegido cubre la clase (no requiere full build para diagnosticar).
- **Costo CI:** run post-fix `37082059543` job x86_64-linux = 463s (build full, rust-cache, sin sccache). El job nuevo usa sccache + rust-cache vía `rust-setup` → warm ≤ ese número.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas: defaults de actions (§Evidencia 3), semántica check --release (§Notas), costo CI/build (§Evidencia 1, §Medición) |
| Pendientes de ejecución (downhill) | 1 — Step 3 (cierre) |
| % completado | ~90% (Steps 1-2 ✅; falta cierre) |

## Steps

### Step 1: Medición local + repro RED (scratch)
- **Archivos:** `src/sdk/search/mod.rs`, `src/sdk/search/fusion.rs` (edit temporal + restauración), logs en `%TEMP%\opencode\find231\`
- **Acción:** (a) medir en verde `cargo check --release` del combo exacto (features `server,custom-allocator` local + `custom-allocator`, `RUSTFLAGS=-D warnings`) y decidir check vs build con la stop-condition; (b) revertir temporalmente las 2 líneas `#[cfg(debug_assertions)]` de `9004c43f`, correr el comando exacto del job → capturar errores; (c) restaurar (`git checkout --`) y confirmar `git status` sin residuos.
- **Verify:** log de medición + captura RED en `%TEMP%\opencode\find231\`; `git status --short` → solo `opencode.jsonc` (WIP ajeno) + archivos del task.
- **Resultado:** ✅ check cold 204.6s exit 0 (build 503.7s → decisión check); repro RED exit 101 ×2 con los 6 errores idénticos al run `37045939672`; restauración limpia + warm 24.2s/24.9s exit 0. Detalle: §Evidencia 4-5.
- **Estado:** ✅

### Step 2: Job `release-combo` en `ci-rust.yml` (+ fila en CI_POLICY.md)
- **Archivos:** `.github/workflows/ci-rust.yml` (job nuevo), `docs/dev/operations/CI_POLICY.md` (§1 tabla de jobs)
- **Acción:** insertar el job con los comandos exactos del release (decisión Step 1), `RUSTFLAGS: "-D warnings"`, `needs: [fmt, clippy]`, timeout acorde; comentario FIND-231 explicando la clase. Agregar 1 fila a la tabla de jobs del Fast Gate.
- **Verify:** `actionlint .github/workflows/ci-rust.yml` exit 0; `node scripts/docs/check-links.mjs` + `check-docs.mjs` exit 0.
- **Resultado:** ✅ job insertado (id `release-combo`, ubuntu, timeout 15m, `RUSTFLAGS` explícito, swap 4096) + fila CI_POLICY; actionlint 0; check-links 0; check-docs 0 (GATING all clear); gen-index regenerado (`docs/index.md` + `llms.txt`, incluye FIND-227 pendiente + FIND-231) y `--check` pasa.
- **Estado:** ✅

### Step 3: Cierre (OCR + review P2-01 + commit LOCAL)
- **Archivos:** `docs/dev/tasks/FIND-231.md` (evidencia final), commit
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (Critical/High bloquean); review P2-01 por agente distinto (tier Fast: verify mecánico + veredicto); commit LOCAL `ci: FIND-231 — …` (sin push).
- **Verify:** OCR sin Critical/High; veredicto registrado en §Review; `git show --stat` del commit sin `opencode.jsonc` ni plan file.
- **Resultado:** ✅ OCR delegado (1 archivo revisable + rule group aplicado → 0 Critical/High); review P2-01 ronda 1 changes-required → fixes F1/F3/F5/F6 → ronda 2 approve; commit local `ci: FIND-231` (5 archivos, hash en recitation).
- **Estado:** ✅

## Dependencias

- FIND-226 (`7ea9ab3e`) y FIND-227 (`a63c9d00`): base del `ci-rust.yml` actual (jobs ASan/TSan cerrados — NO se tocan). ✅
- FIND-228 (`0e5c9e9d`): triggers ya deduplicados — este job hereda los triggers existentes de `ci-rust.yml` sin cambios. ✅
- FIND-232 / FIND-230 / gate-docs*: sin dependencia (paths disjuntos). ✅

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sub-agente fresco; sesión `ses_effdd66e2ffeB0vt00Bj8hwK0T`; contexto ≠ autor).
- **Paths del diff:** `.github/workflows/ci-rust.yml`, `docs/dev/operations/CI_POLICY.md`, `docs/index.md`, `llms.txt`, `docs/dev/tasks/FIND-231.md` → **Tier Fast** (CI + docs; sin paths adversariales).
- **Gate requerido:** verify fast mecánico (actionlint + docs checks + `verify_changed.ps1`) + veredicto registrado.
- **Ronda 1 — ❌ changes-required** (1 Medium + 3 Low + 2 Nits; re-derivación independiente del revisor: actionlint, logs de evidencia, action sources pinneadas, run original):
  - **F1 (Medium):** el claim "ningún job compila el perfil release del combo" es **falsable** — `desktop.yml:78-79/150-151/216-217` compila el combo exacto, pero sin `RUSTFLAGS` (rust-setup/dtolnay) → sin `-D warnings`. → **Fix:** claim corregido a "bajo `-D warnings`" en el comentario del job + §Evidencia 2 (+ evidencia desktop.yml file:line). Verificado por el autor contra `desktop.yml`.
  - **F2 (Low):** el verde local usó `custom-allocator` (Windows); el comando CI exacto (`jemalloc`) solo se prueba en el tramo 2 (CI) — aceptado y documentado (§Notas).
  - **F3 (Low):** sin guard de drift para el combo copiado → **Fix:** comentario "keep in sync with release-binaries.yml:109-122" en el job.
  - **F4 (Low):** evidencia OCR = input de delegación (preview+rules), no resultado → documentado: `ocr-review.json` (7 files, 1 revisable = `ci-rust.yml`) + rule-group aplicado manualmente → 0 Critical/High; docs excluidos por `unsupported_ext` del tool.
  - **F5 (Nit):** edit accidental de un comentario ajeno (`experimental-check`: em dash → hyphen) → **Fix:** em dash restaurado; diff del job ahora estrictamente aditivo (`--numstat` = 33+/0-).
  - **F6 (Nit):** Blast Radius subdeclaraba los índices regenerados → **Fix:** Metadata + tabla Blast Radius actualizadas.
- **Ronda 2 — ✅ approve** (sin findings funcionales): F1/F3/F5/F6 verificados corregidos; diff `33+/0-` insertion-only; em dash byte-idéntico a HEAD; §Evidencia 2 con counter-evidence exacta (`desktop.yml:78-79/150-151/216-217` + `:59/134/194`); §Review ronda 1 fiel a los hallazgos. Nit residual F2-pointer (resuelto: nota añadida a §Notas) + tally cosmético (corregido).
- **Veredicto final:** **✅ approve** — `vanta-review`, sesión `ses_effdd66e2ffeB0vt00Bj8hwK0T` (contexto fresco ≠ autor; P2-01 satisfecho). Commit-ready: incluir solo `ci-rust.yml` + `CI_POLICY.md` + `FIND-231.md` + `docs/index.md` + `llms.txt`; excluir `opencode.jsonc` y plan file (WIP de otros actores).

## Notas

- **Decisiones de diseño:** (1) matriz mínima ubuntu — la clase es de perfil, no de OS (§Spec 2); (2) `-D warnings` explícito (§Evidencia 3); (3) job en el batch existente con sccache ya configurado (pre-mortem del plan: costo CI); (4) sin `continue-on-error` (gate real).
- **Drift colateral detectado (fuera de scope):** `docs/dev/workflow/ci-rust-10.md` está stale ("13 jobs" vs 20 actuales; coverage ">=59%" vs 80% ADR-0018). No se corrige inline (scope discipline); candidato a FIND en Backlog si el owner lo prioriza. `CI_POLICY.md` §1 también tiene imprecisiones pre-existentes (p.ej. `experimental-check` listado como continue-on-error cuando no lo es) — solo se agrega la fila del job nuevo.
- **Fases SECURITY/PERFORMANCE:** SECURITY — n/a: no toca trust boundaries, input de usuario, auth, datos ni agrega dependencias (solo un job que compila lo existente). PERFORMANCE — n/a: no toca hot paths; el costo es de CI (medido en §Medición).
- **Regla 6:** sin deuda nueva; el job paga deuda de proceso (previene el ciclo quemado de FIND-229).
- **Residual F2 (review ronda 1):** el verde local de los comandos usó `custom-allocator` (Windows); el comando CI exacto (`jemalloc`) queda probado solo en el tramo 2 (CI post-push). La clase es de perfil, no de allocator: el run original (jemalloc + release + `-D warnings`) produjo exactamente los mismos 6 diagnósticos, y la repro local los reproduce con el mismo mecanismo.
- **Medición build local:** `cargo build --release` del primer binario completó en 503.7s (exit 0) antes de detener la medición; el build del server quedó sin medir (no aporta: el primer binario ya excede el presupuesto). Dato CI equivalente: job `463s` sin sccache (run `37082059543`).

- **Post-push (2026-10-03, run `37102066714`):** ✅ **Release Combo (server + allocator, -D warnings) = SUCCESS** en CI — tramo 2 cerrado; el residual F2 (comando jemalloc exacto probado solo en CI) quedó resuelto: pasa en el runner.
