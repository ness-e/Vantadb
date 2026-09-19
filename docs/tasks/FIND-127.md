# FIND-127 — triage infra CI + Dependabot

> **Plan:** `docs/plans/2026-09-19-ci-green.md` · **Wave2 última** · **Ruta:** vanta-lead
> **Branch:** develop · **Commit:** `ci: FIND-127 — ...` · **Appetite:** 2d · **Estado:** ⬜ PENDING → ⏳ IN PROGRESS

## 1. TAREA

**Objetivo:** triage infra CI + Dependabot del PR #182 para que el PR pueda ponerse verde
(o quede con rojos declarados y dueñados).

**Contrato exacto (del plan):** workflows fixed (imagen con rustc, docker aarch64 o skip
documentado, CodeQL/Vercel diagnosticados) + vulns triageadas (fix-ya: mergear PRs #180 y
familia; aceptar-riesgo: con motivo escrito por CVE) + re-runs verdes o rojos declarados.

**Acceptance criteria:**
- (a) workflows fixed mínimos por archivo;
- (b) cada vuln con veredicto fix-ya o aceptar-riesgo escrito por CVE;
- (c) re-runs verdes o rojos declarados con dueño.

**HALLAZGO-DX1 (código contradice al plan — se reporta con evidencia, no se re-deriva el resto):**
el plan diagnostica "providers sin rustc en imagen". FALSO: el log del run 35417674127 muestra
`rustc 1.98.1` instalado OK (`stable-x86_64-unknown-linux-gnu unchanged - rustc 1.98.1`);
el fallo real es `maturin develop --release` → E0432/E0433 por imports `Vanta*` pre-AST-010
en `providers/*/src/*.rs` (mismo drift que FIND-124 en desktop). Evidencia: log
`C:\Users\Eros\AppData\Local\Temp\opencode\prov-fail.log` (`error[E0432]: unresolved import
vantadb::config::VantaConfig` ×3 providers). El fix real es migrar imports (paso 2), no la imagen.

## 2. ARCHIVOS

**Clave (con línea):**
- `providers/openai/src/python.rs:6-7,303` · `providers/ollama/src/python.rs:6-7,299` ·
  `providers/litellm/src/python.rs:6-7,190` (imports `VantaConfig`/`VantaEmbedded`/
  `VantaMemoryInput`/`VantaMemoryListOptions` + `VantaMemoryMetadata::new()`)
- `providers/shared_py.rs:23-24,206-224` (`error::VantaError as CoreError`, `VantaMemoryRecord`,
  `VantaMemorySearchRequest`, `VantaValue`, `DistanceMetric` raíz — este último OK)
- `.github/workflows/release-wheels-60.yml:104-105` (`before-script-linux: yum install -y clang clang-devel`
  → exit 127: el container cross `ghcr.io/rust-cross/manylinux_2_28-cross:aarch64` es Debian-based, sin `yum`)
- `.github/workflows/sec-codeql-30.yml:1-37` (sin cambio — diagnosticado, ver paso 4)
- `.github/workflows/heavy-certification-50.yml` + `heavy-bench-nightly-51.yml` (sin cambio —
  diagnosticados hasta límite honesto, ver paso 4)
- `web/vercel.json` (`{"framework":"nextjs"}` — Vercel diagnosticado, ver paso 4)

**Relacionados:**
- Callers/callees: los 4 archivos providers solo los usa `providers-ci.yml:57-69`
  (`maturin develop` + `pytest` + `verify_pyi.py`); ningún otro workflow/crate los importa
  (crates experimentales NO miembros del workspace — `CI_POLICY.md` circuit breaker).
  `verify_pyi.py` chequea firmas `.pyi` — los renombres son solo Rust-interno, firmas Python intactas.
- PRs dependabot abiertos #162-180 + #183 (ver `gh pr list`); #180 `sharp+next` (MERGEABLE/BLOCKED)
  cubre alertas 35/34/33/31/18; #183 `rust-patch group` (7 updates, incluye lru low #1).
- `docs/operations/CI_POLICY.md` (26 workflows; wheels §9 ya documenta skip smoke aarch64;
  releases solo tag).
- 20 alertas Dependabot abiertas (ver `gh api .../dependabot/alerts`): 19 npm/web+ts, 1 Rust low (lru).

**Prohibidos (NO tocar):** bumps majors mezclados (#174 toml 0.9→1.1, #175 rocksdb — fuera de este
triage) · `reparacion.bat` · `.opencode` · `Justfile` · `ocr-*` · `completions/*` ·
`desktop/src-tauri/Cargo.lock` · stash@{0} GOV-C4 · `docs/Backlog.md` · plan file (solo recitation
al cierre) · `C:/Users/Eros/.vantadb*` · `src/` · `web/src/` · `examples/` · `vantadb-ts/src/` ·
`desktop/` (FIND-123/124/125/126 cerrados). Staging SELECTIVO solo paths propios.

## 3. DEPENDENCIAS

Wave2 última en secuencia (Wave0 ✅ FIND-124/FIND-123, Wave1 ✅ FIND-125/FIND-126; archivos
disjuntos). No bloquea a nadie salvo el verde del PR. Stop del plan: secreto/config inaccesible
desde el runner → STOP con instrucción al owner (aplica a Vercel: sin token, solo diagnóstico
local + instrucción). NextTask tras cierre: ninguna — ÚLTIMA del plan (cierre del orquestador:
P2-01 batch + progreso + archive).

## 4. REFERENCIAS

- Rules (leída completa antes de actuar): `.opencode/rules/release-ci.md`
  (Regla 5: `continue-on-error` siempre con `# CATEGORY:` — mi cambio no añade ninguno;
  Regla 2: sccache punto único — no se toca; Regla 3: Dockerfile MSRV — no se toca).
- Refs: `definition-of-done.md` (DoD v1: clippy/fmt/tests/docs/shippable) ·
  `dev-tools.md` (just/verify).
- Commands: `pipeline.md` (ejecución) · `audit.md` (verify post-tarea).
- SPEC.md raíz (sin cambios — 0 greenfield). Tabla Spec: N/A (CI/deps, sin símbolos públicos).

## 5. SKILLS (SDP Paso 0b)

`campaign_discover_skills_v2` (phase=BUILD, keywords github-actions/dependabot/vulnerabilities/
rustc/docker/codeql/vercel/triage) devolvió base+lifecycle (sin keyword-mapped). Cargadas:
- `ci-cd-and-automation` — cambios mínimos en workflows + quality gates (pasos 2-3).
- `git-workflow-and-versioning` — commit `ci:` atómico, staging selectivo, sin push (paso 6).
- `security-and-hardening` — triage vulns fix-ya vs aceptar-riesgo por severidad+explotabilidad (paso 5).
- `shipping-and-launch` — rollback plan + thresholds del release (cierre).
- `doubt-driven-development` — HALLAZGO-DX1 contradice diagnóstico del plan (verificado con logs).
- `source-driven-development` — advisories solo con URLs verificadas (GHSA/CVE).
- `incremental-implementation` — slices delgados por archivo con verify por slice.
- `test-driven-development` / `context-engineering` (lifecycle BUILD, base del SDP).
- SDP: ci-cd-and-automation · git-workflow-and-versioning · security-and-hardening ·
  shipping-and-launch · doubt-driven-development · source-driven-development ·
  incremental-implementation (+ base campaign-executor/progreso/ponytail).

## 6. HERRAMIENTAS + MCP

- `cargo check --manifest-path providers/<p>/Cargo.toml` (verify providers; `-j 2`).
- `python -m yamllint` o `actionlint` si disponible (wheels que toco); si ausente → `python -c yaml.safe_load`
  + `git diff` review (declarado).
- `gh run view --log-failed` (evidencia, ya capturada en `Temp/opencode/*.log`).
- `gh api dependabot/alerts + code-scanning/alerts` (triage).
- `campaign_verify_cmd` para verify mecánico (bug exit -1 conocido → bash directa + mención en RESULTADO).
- codegraph: N/A (workflows + renombres mecánicos con mapa verificado FIND-124; sin blast radius de código).
- Cargo workspace: N/A (providers son crates fuera del workspace; cero Rust del core).
- Internet SOLO advisories con URLs verificadas; sin red → `[cita NO VERIFICADA]` + deuda TSYS-13.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `providers-ci.yml`, `release-wheels-60.yml`, `sec-codeql-30.yml`,
  `heavy-bench-nightly-51.yml`, `heavy-certification-50.yml`, `ci-gate.yml`, `CI_POLICY.md`,
  `dependabot.yml`, `providers/openai/src/python.rs` (los otros 2 `python.rs` son clones
  estructurales — imports idénticos verificados por grep), `providers/shared_py.rs` (líneas 23-24,
  56-80, 200-230), rename-map FIND-124 (`docs/tasks/FIND-124.md` tabla + commit 797059bf).
- **Referencias hacia dentro:** providers ← solo `providers-ci.yml`; wheels ← solo `release-wheels-60.yml`.
- **Referencias entrantes:** ninguna (providers fuera del workspace; workflows standalone).
- **Veredicto:** impacto mínimo y reversible; 4 archivos Rust (renombres 1:1 verificados contra
  `src/sdk/mod.rs`, `src/error.rs:122`, `src/config.rs:572`, `src/lib.rs`) + 1 workflow (1 step).

## 7. STEPS ATÓMICOS

- [x] **Step 1 — providers renombres** (4 archivos, mapa §8). Verify: `cargo check` ×3 ✅
  (openai 20.70s, ollama 3.41s, litellm 19.05s; 10 warnings pre-existentes del lib, cero del diff).
- [x] **Step 2 — wheels aarch64** (`before-script-linux` portable yum/apt + comment FIND-127).
  Verify: `actionlint` exit 0 ✅ (docker build solo corre en CI; re-run dueñado al lead).
- [x] **Step 3 — diagnósticos sin código:**
  - CodeQL 9s fail = check `CodeQL` (app github-advanced-security) "2 configurations not found":
    baseline stale que referencia `.github/workflows/codeql.yml` (inexistente en main y en develop;
    el lane actual es `sec-codeql-30.yml`). Sin cambio de código; auto-verde post-merge a main.
  - Vercel fail = plataforma/config (sin token no hay `inspect`): `web/` compila local
    (lint ✅, `tsc --noEmit` ✅, `next build` ✅ exit 0, Node v26.8.1; sin `engines` → default Vercel).
    Dueño owner: `npx vercel inspect dpl_4XWtSU8nsn22nG9qpLga45NMEQJ4 --logs`.
  - HEAVY `startup_failure` schedule (cert ×6 domingos, bench diario ×5, fuzz-40 ×3): 0 jobs/0 logs;
    afecta SOLO callers del reusable `ci-gate.yml`; schedule corre sobre main (versiones viejas);
    `workflow_dispatch` 33814970219 SÍ crea jobs; PR lane crea jobs. Causa server-side no visible
    con estos permisos → rojo declarado, dueño lead, next: dispatch manual desde main para bisectar.
- [x] **Step 4 — triage 20 vulns** (tabla §9; digest advisories §10).
- [x] **Step 5 — verify contrato + Gate C + commit** (`ci: FIND-127 — ...`, staging selectivo,
  NO PUSH) + RESULTADO §7. Commit `a9d55c34` (9 files; pre-commit hooks verde).

## 8. MAPA RENOMBRES (fuente: FIND-124 verificado contra código)

| Viejo | Nuevo | Fuente verdad |
|---|---|---|
| `vantadb::config::VantaConfig` | `vantadb::config::Config` | `src/config.rs:572` |
| `vantadb::sdk::VantaEmbedded` | `vantadb::sdk::Embedded` | `src/sdk/mod.rs:15` |
| `vantadb::sdk::VantaMemoryInput` | `vantadb::sdk::MemoryInput` | `src/sdk/mod.rs:27` |
| `vantadb::sdk::VantaMemoryListOptions` | `vantadb::sdk::MemoryListOptions` | `src/sdk/mod.rs:28` |
| `vantadb::sdk::VantaMemoryMetadata` | `vantadb::sdk::MemoryMetadata` | `src/sdk/mod.rs` types |
| `vantadb::sdk::VantaMemoryRecord` | `vantadb::sdk::MemoryRecord` | `src/sdk/mod.rs:28` |
| `vantadb::sdk::VantaMemorySearchRequest` | `vantadb::sdk::MemorySearchRequest` | `src/sdk/mod.rs:29` |
| `vantadb::sdk::VantaValue` | `vantadb::sdk::Value` | `src/sdk/mod.rs:32` |
| `vantadb::error::VantaError` | `vantadb::error::Error` | `src/error.rs:122` |
| `vantadb::DistanceMetric` | (sin cambio — re-export raíz OK) | `src/lib.rs` |

## 9. TRIAGE 20 VULNS (fix-ya vs aceptar-riesgo por CVE, no por conteo)

### FIX-YA — mergear #180 y familia (dueño: lead)

| Alertas | Paquete | CVE/GHSA | Motivo fix-ya |
|---|---|---|---|
| #35 critical, #34 critical | next ^16.1.1→^16.3.4 | GHSA-p293-qw3h-jr36 (CVE-2026-75604), GHSA-2xp9-vwfh-vxw4 | RCE; fix en #180 (Vercel=Linux baja explotabilidad pero critical manda) |
| #33 high, #31 high, #18 high | sharp →0.35.4 | GHSA-rgj7-g3m4-5g8c (libheif RCE), GHSA-f88m-g3jw-g9cj (libvips CVE-2026-33327/33328/35590/35591/69242) | fix 0.35.4 en #180 (build-time baja explotabilidad, fix trivial) |
| #29 high | nanoid ^3.3.6→^3.3.16 | loop infinito size=0 [cita NO VERIFICADA — sin página individual; alerta Dependabot como fuente] | fix en #180 |
| #22 med, #20 high, #19 high, #16 med | postcss web 8.4.31→8.5.23 | GHSA-6g55-p6wh-862q (CVE-2026-45623), GHSA-fxqj-rqcc-2cmp (CVE-2026-69153), GHSA-r28c-9q8g-f849, GHSA-qx2v-qp2m-jg93 | fix en #180 (build-time Tailwind, CSS no atacante-controlado) |
| #26 med, #13 high | postcss vantadb-ts lock | mismos GHSA postcss | NO cubierto por #180 → FIX-YA con comando owner: `npm update postcss --prefix vantadb-ts` + `npx tsc --noEmit` + vitest |

Comandos owner (orden): `gh pr review 180 --approve && gh pr merge 180 --squash`
(está BLOCKED: requiere review + CI); luego familia rutina (patch/minor, EXCLUIR #174 toml
0.9→1.1 y #175 rocksdb 0.24→0.25 — nativos/majors fuera de este triage): #183 rust-patch,
#162/#163/#164 actions+tools, #165/#168 codeql, #166/#167/#169/#170/#171 web minors,
#173 smallvec, #176 tower-http, #177 tokenizers, #178 mach2, #179 rcgen.

### ACEPTAR-RIESGO (motivo escrito por CVE; review al renovar la herramienta madre)

| Alertas | Motivo |
|---|---|
| #36/#23/#17/#10 js-yaml (transitivo de `@eslint/eslintrc`, solo lint build-time) | YAML solo local; CVE-2026-59870 sin backport a 4.x → sin fix disponible; review al renovar eslint |
| #32/#30 vitest+`@vitest/mocker` (dev-only test runner) | exploit = test malicioso local (auto-ataque); sin path prod [URLs NO VERIFICADAS — deuda TSYS-13] |
| #15 prismjs DOM clobbering (highlighting docs estático) | contenido autor-controlado [URL NO VERIFICADA — deuda TSYS-13] |
| #1 lru low (Stacked Borrows teórico, Rust) | sin exploit conocido; #183 puede traer el bump — verificar al mergearlo |

brace-expansion (citado en el plan): 0 alerts abiertas hoy → ya cerrado, sin acción.

## 10. DIGEST ADVISORIES (URLs verificadas vía websearch 2026-09-19)

- RCE windows-hosted: https://github.com/vercel/next.js/security/advisories/GHSA-p293-qw3h-jr36
- RCE Image Optimization AVIF: https://github.com/vercel/next.js/security/advisories/GHSA-2xp9-vwfh-vxw4
  (DB: https://github.com/advisories/GHSA-2xp9-vwfh-vxw4)
- sharp libheif: https://github.com/lovell/sharp/security/advisories/GHSA-rgj7-g3m4-5g8c
  (fix 0.35.4; upstream https://github.com/strukturag/libheif/security/advisories/GHSA-g89c-p67h-r497
  y https://github.com/strukturag/libheif/security/advisories/GHSA-2jg2-4ch7-h545)
- postcss sourceMappingURL: https://github.com/advisories/GHSA-6g55-p6wh-862q (CVE-2026-45623,
  patched 8.5.12) · incomplete-fix: https://github.com/advisories/GHSA-fxqj-rqcc-2cmp (CVE-2026-69153)
- Deuda TSYS-13: verificar páginas individuales nanoid-loop, vitest-mocker-redirect, prismjs-clobbering,
  js-yaml CVE-2026-59870 antes de citarlas como URL.

## Context Save Point

DISCOVERY ✅ (2026-09-19): matriz de fallos real extraída de `gh pr checks 182` + logs
(run 35417674127 providers E0432, run 35417674112 wheel exit 127 `yum: command not found`,
check-run 105835305971 CodeQL "2 configurations not found", Vercel external fail, HEAVY schedule
`startup_failure` ×6 cert / ×5 bench con 0 jobs, 20 alerts Dependabot listadas).
Reanudar: Step 1 (edits providers) → `cargo check` ×3 → Step 2 → ... → Step 5.
Si `campaign_verify_cmd` devuelve exit -1 (bug conocido) → bash directa + mención en RESULTADO.
