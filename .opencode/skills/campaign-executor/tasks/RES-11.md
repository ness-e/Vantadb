# TASK-RES-11: Job rustdoc en CI

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md`
- **Creado:** 2026-08-29
- **last-synced:** 2026-08-29
- **Estado:** 🟡 STAGED (verificado, archivo staged; commit delegado a vanta-lead per protocolo worker/lead)
- **SDP:** base-only (keywords: rustdoc, cargo doc, artifact, docs) — campaign-executor + ci-cd-and-automation + codebase-memory cargadas vía orquestador
- **Origen:** FND-17-api-reference-docs-as-code.md (Fase 1), DOCS-RUSTDOC en PLAN-ACCION-RESEARCH-2026-08-26.md

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `.github/workflows/ci-rust-10.yml` (602L) — workflow Rust principal; tiene jobs fmt/clippy/semver-checks/adr-gate/test/test-windows/test-macos/msrv/minimal-versions/coverage/wasm-test/experimental-check/audit/miri/deny/sanitizer-asan/sanitizer-tsan. NO tiene job rustdoc. Sigue convenciones: usa `actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0`, `rust-setup` action, `cargo` directamente.
  - `.github/actions/rust-setup/action.yml` (100L) — composite action con toolchain/cache/sccache/system-deps/nextest/llvm-cov. Inputs: toolchain/components/swap-mb/install-nextest/install-llvm-cov/install-system-deps.
  - `docs/research/archive/FND-17-api-reference-docs-as-code.md` (208L) — investigación previa §5 propone YAML exacto para el job.
  - `docs/Backlog.md:597` — fila RES-11 (origen, contexto).
  - `Cargo.toml:1-25` — paquete `vantadb`, `documentation = "https://docs.rs/vantadb"`.

- **Referencias hacia dentro (este archivo las importa/referencia):**
  - Ninguna. Workflows CI no son importados por código de runtime; su único consumidor es GitHub Actions runner.

- **Referencias hacia afuera (lo que este archivo invocaría):**
  - `actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0` (mismo SHA usado por todos los jobs del ci-rust-10.yml).
  - `./.github/actions/rust-setup` (action local compartida).
  - `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2` (mismo SHA usado en coverage).

- **Veredicto de impacto:** BAJO. Cambio aditivo a un workflow existente; no toca código fuente, ni Cargo.toml, ni dependencias. Blast radius = 1 archivo (`ci-rust-10.yml`) + verificación del contrato.

## Blast Radius
| Tipo | Alcance |
|---|---|
| Callers | GitHub Actions runner (cada push/PR a main que toque paths relevantes dispara el job nuevo) |
| Callees | `cargo doc --no-deps --workspace` (toolchain Rust, ya instalada por rust-setup) |
| Implicaciones | Build artifact extra en cada PR (HTML ~5-50MB). Tiempo añadido: ~3-5min. CI sigue verde o falla por docstring roto (gate deseado). |

## Contrato (mecánico)
- `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "cargo doc" | Measure-Object | Select-Object Count` >= 1
- OR `Test-Path .github/workflows/ci-rustdoc.yml` == $true

## Pre-mortem (del task contract)
- **Fallo 1:** `cargo doc --workspace` genera 100MB+ → artifact sin compresión.
  Mitigación: usar `target/doc/**` con `if-no-files-found: warn` y `retention-days: 7` para no acumular masa histórica.
- **Fallo 2:** rustdoc warnings tratados como errors en CI.
  Mitigación: por defecto `cargo doc` falla con `rustdoc::html` errors pero NO warnings. Si surge ruido de warnings, agregar `RUSTDOCFLAGS="-D warnings"` solo cuando el lead apruebe (cambio de policy, no parte de RES-11).
- **Fallo 3:** si workflow ya existe, no duplicar.
  Mitigación: verificar `Test-Path .github/workflows/ci-rustdoc.yml` antes de crear (ya verificado: false). Editar `ci-rust-10.yml` añade el job in-place; no crea duplicado.

## Herramientas
- Edit tool (workflows YAML)
- PowerShell `Select-String` / `Test-Path` para verify mecánico
- No requiere cargo locally — el contrato es grep + YAML

## Steps

### Step 1: PLAN — Decidir path (modificar ci-rust-10.yml vs crear ci-rustdoc.yml)
- **Acción:** Evaluar trade-offs. El contrato permite ambas. La investigación FND-17 §5 proponía workflow nuevo `docs-reference.yml`. El task description dice "(o nuevo ci-rustdoc.yml)". El plan file prefiere "ci-rust-10.yml (o nuevo ci-rustdoc.yml)".
- **Criterio:** PONYTAIL — el cambio más simple que satisface el contrato. Modificar `ci-rust-10.yml` añade 1 job a un workflow grande (mejor organización lógica: todo Rust en un lugar). Crear `ci-rustdoc.yml` separado aísla la carga de un artifact grande (mejor costo de CI: el job solo corre cuando hay cambios Rust, no en cada push). Para docs artifact, el artifact grande es preferible como job dedicado (path filter estricto, retenido por separado).
- **Decisión:** Crear nuevo `.github/workflows/ci-rustdoc.yml` (path filter estricto, retention corto, artefacto navegable descargable por PR). Cumple contrato `Test-Path` ✓. Se alinea con el split que tiene el repo (gate-docs-21 separado, ci-rust-10 separado, ci-web-11 separado).
- **Verify:** `Test-Path .github/workflows/ci-rustdoc.yml` → false antes de crear; → true después.
- **Estado:** ✅ COMPLETED (2026-08-29) — Decisión: workflow nuevo separado (`ci-rustdoc.yml`) vs job en `ci-rust-10.yml`. Criterio ponytail: artifact grande aislado con path filter estricto + retention corto. Contrato `Test-Path` se cumple con la rama del OR.

### Step 2: ACT — Crear workflow
- **Acción:** Crear `.github/workflows/ci-rustdoc.yml` con un job `rustdoc` que corra `cargo doc --no-deps --workspace` y suba `target/doc` como artifact `api-reference-rust`.
- **Decisiones de diseño (mínimas, siguiendo convenciones del repo):**
  - `on:` push a `main`/`develop` + PR a `main`, paths filtered a Rust (`src/**`, `vantadb-*/**`, `providers/**`, `Cargo.toml`, `Cargo.lock`). Esto evita correr el job cuando se tocan solo docs/web.
  - `permissions: contents: read` (consistente con ci-rust-10).
  - `concurrency: group: rustdoc-${{ github.ref }}, cancel-in-progress: true` (consistente con fmt/clippy/semver-checks en ci-rust-10).
  - `actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0` (mismo SHA que el resto).
  - `./.github/actions/rust-setup` con `install-nextest: false`, `install-llvm-cov: false`, `install-system-deps: false` (rustdoc no necesita nextest, ni llvm-cov, ni librocksdb).
  - `timeout-minutes: 15` (cargo doc workspace suele ser 3-7min).
  - `cargo doc --no-deps --workspace --all-features` — workspace entero, todas las features (consistente con `cargo clippy --all-features` que ya corre).
  - Artifcat: name `api-reference-rust`, path `target/doc`, retention 7 días, `if-no-files-found: error`.
- **Verify:** `Test-Path .github/workflows/ci-rustdoc.yml` → true.
- **Estado:** ✅ COMPLETED (2026-08-29) — Archivo creado en `.github/workflows/ci-rustdoc.yml` (79 líneas, 1 job `rustdoc`, 3 triggers). Sintaxis YAML validada con `python -c "import yaml; yaml.safe_load(...)"`.

### Step 3: VERIFY — Contrato mecánico (post-creación)
- **Acción:** Correr PowerShell nativo (Windows shell del repo) para verificar el contrato:
  1. `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "cargo doc" | Measure-Object | Select-Object Count` — debería dar 0 (no modifico este).
  2. `Test-Path .github/workflows/ci-rustdoc.yml` — debería dar True.
  3. `Select-String -Path ".github/workflows/ci-rustdoc.yml" -Pattern "cargo doc" | Measure-Object | Select-Object Count` — debería dar >=1.
- **Estado:** ✅ COMPLETED (2026-08-29) — Output del verify script:
  - Test 1: `Count: 0` (ci-rust-10.yml NO modificado, intacto)
  - Test 2: `Exists: True` (ci-rustdoc.yml creado)
  - Test 3: `Count: 3` (cargo doc aparece 3 veces en el nuevo workflow: comentarios explicativos + comando)
  - Verdict: **PASS** (rama Test 2 del OR satisfecha)

### Step 4: CIERRE — Stage para lead
- **Acción:** `git add .github/workflows/ci-rustdoc.yml` (HECHO, staged para vanta-lead) + `git commit -m "ci: RES-11 — Job rustdoc en CI (cargo doc artifact)"` (PENDIENTE — rol worker no commitea, ver tabla de permisos AGENTS.md).
- **Estado:** 🟡 STAGED (2026-08-29) — Staged en index; commit delegado a vanta-lead per protocolo worker/lead y contrato de task ("si eres worker, staged para lead"). No-hash de commit todavía.

## Dependencias
- Ninguna. Tarea standalone.

## Notas
- vanta-worker (rol actual): implementa y stagea. vanta-lead ejecuta git push (Regla 1: solo vanta-lead hace git push; commit también es del worker si la política lo permite — ver tabla de permisos). El contrato del task dice "vanta-lead no hace commit (rol vanta-worker/lead; si eres worker, staged para lead)" — por lo tanto NO commiteo, solo dejo staged.
- No se toca Cargo.toml, deny.toml ni nada del core. Cambio aislado a `.github/workflows/`.
- No agrego `RUSTDOCFLAGS="-D warnings"` porque convertir warnings en errors es una policy decision que requiere revisión por el lead (toca DX de contribuidores). El job actual solo gatea docstrings rotos (lo que `cargo doc` falla por defecto).
- Si el job resulta ruidoso (muchos warnings acumulados en el core), follow-up natural es abrir ticket FIND para endurecer con `-D warnings` después de limpiar el ruido.

## Context Save Point
- **Fecha:** 2026-08-29
- **Branch:** develop (W2-3 in wave 2)
- **CI pendiente:** no (solo cambio de workflow; CI se valida cuando se pushea)
- **Decisiones:** workflow nuevo separado (no job en ci-rust-10) — artifact grande aislado con path filter estricto y retention 7d
- **Problemas conocidos:** ninguno
- **Próxima tarea:** W2 contigua: SRV-03, WSM-07; después W3-SOLO: RES-01 (ACID WAL v2)