# FND-16 — Multi-target CI (wheels + WASM por PR)

**Fecha:** 2026-08-16 · **Wave:** P20c · **Prio:** 🟢 · **Tipo:** análisis (sin implementación)
**Owner:** vanta-lead · **Estado:** análisis completo, decisión propuesta (pendiente aprobación del lead para implementar)

---

## 1. Estado actual — triggers y matrix por workflow

### 1.1 `release-wheels-60.yml` — Wheels Python (maturin)

| Aspecto | Valor | Cita |
|---|---|---|
| Trigger PR | `pull_request` branches `main`, paths `src/**`, `vantadb-python/**`, `Cargo.toml`, `Cargo.lock`, el propio workflow | `release-wheels-60.yml:10-17` |
| Trigger push | tags `v*.*.*` | `release-wheels-60.yml:18-19` |
| Trigger manual | `workflow_dispatch` con input `publish_testpypi` | `release-wheels-60.yml:4-9` |
| Job build | `build-wheels` — timeout 45 min | `release-wheels-60.yml:33-37` |
| Matrix | `os: [ubuntu-latest, macos-latest, windows-latest]` | `release-wheels-60.yml:39-42` |
| Build | maturin-action, `--release`, manylinux `2_28`, musllinux `1_2` | `release-wheels-60.yml:82-90` |
| Smoke test | por OS: import + `pytest vantadb-python/tests/test_sdk.py` en venv | `release-wheels-60.yml:92-114` |
| Publish | solo en tag (`publish-pypi`) o `workflow_dispatch` testpypi | `release-wheels-60.yml:124-155` |

**Hallazgo clave: los wheels YA se compilan en cada PR** contra `main` (3 OS + smoke tests), no solo en release. El publish sigue siendo solo-tag.

### 1.2 `release-npm-61.yml` — WASM (wasm-pack) + TS

| Aspecto | Valor | Cita |
|---|---|---|
| Trigger PR | **NO existe** | — |
| Trigger push | tags `v*.*.*` | `release-npm-61.yml:16-17` |
| Trigger manual | `workflow_dispatch` con inputs `package` / `dry_run` | `release-npm-61.yml:4-15` |
| Job tests | `tests` — setup Node 22, wasm-pack, `rustup target add wasm32-unknown-unknown`, `wasm-pack build --release`, `npm install`, `npm run build`, `npm test` | `release-npm-61.yml:31-71` |
| Publish | `publish-wasm` / `publish-ts` — solo tag o dispatch | `release-npm-61.yml:73-199` |

**Hallazgo clave: WASM NO se compila en PR.** El job `tests` (que incluye build wasm32 + tests TS) solo corre en release (tag) o manual.

### 1.3 `ci-rust-10.yml` — Fast Gate

| Aspecto | Valor | Cita |
|---|---|---|
| Trigger PR | `pull_request` branches `main`, paths amplios incluyendo `vantadb-*/**`, `integrations/**` | `ci-rust-10.yml:20-35` |
| Job `wasm-test` | `wasm-pack test --chrome --headless` — **solo `push` a main**, `continue-on-error: true` (BEST-EFFORT), condición `contains(github.event.head_commit.modified, 'vantadb-wasm/')` | `ci-rust-10.yml:388-395` |
| Job `experimental-check` | `cargo check -p vantadb-server -p vantadb-mcp -p vantadb-wasm` — compila los 3 crates **para el host (linux x86_64)**, NO para el target `wasm32-unknown-unknown` | `ci-rust-10.yml:409-425` |
| Job `clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` — incluye vantadb-wasm como crate, pero para host | `ci-rust-10.yml:83-86` |

**Hallazgo clave:** el Fast Gate compila el crate `vantadb-wasm` para **host** (`cargo check`/`clippy`), pero **nadie compila el target `wasm32-unknown-unknown` en PR**. El único job que toca wasm32 (`wasm-test`) es push-only, non-blocking, y depende de `head_commit.modified` (el último commit del push, no del PR).

### 1.4 `release.yml` — release-plz

- Solo `push` a `main` (branches: `[main]`) con jobs `release-plz-release` / `release-plz-pr` (`release.yml:6-8,11,33`). No compila nada — orquesta bump/PR/tag.

---

## 2. Análisis de costo y gap real

### 2.1 ¿Qué se valida ya en PR hoy?

| Artefacto | ¿Se compila en PR? | Dónde | Bloquea? |
|---|---|---|---|
| Crate `vantadb` core | ✅ | `ci-rust-10` clippy/test/coverage | ✅ |
| Crate `vantadb-wasm` (host) | ✅ | `ci-rust-10` experimental-check + clippy | ⚠️ experimental-check es non-blocking; clippy sí |
| **WASM target `wasm32-unknown-unknown`** | ❌ | — | — |
| Wheels Linux/Mac/Windows (maturin) | ✅ | `release-wheels-60` build-wheels + smoke | ✅ (por PR a main) |
| TS SDK (npm build + test) | ❌ | solo en `release-npm-61` tag/dispatch | — |

### 2.2 El gap real

**Un PR que rompe el build WASM (target wasm32) o el build TS pasa CI sin detectarse.** Se detecta recién en release (tag → `release-npm-61` job `tests`), donde falla *después* de haber taggeado, o manualmente vía `workflow_dispatch`.

Los wheels no tienen este problema: `release-wheels-60` ya corre en PR (3 OS + smoke tests).

Costo actual del gap:
- Cada release WASM roto = release bloqueado + fix post-tag + re-tag. El fix es barato (WASM se detecta en minutos), pero el ciclo release se interrumpe.
- `wasm-test` existente es insuficiente: solo push a main, `continue-on-error`, y el guard de `head_commit.modified` no captura PRs multi-commit.

### 2.3 Costo de cubrir el gap en PR

| Opción | Jobs/OS | Minutos estimados | Costo |
|---|---|---|---|
| WASM build en PR (solo wasm-pack build + npm test, 1 runner ubuntu) | 1 runner × ~8-15 min | ~10 min por PR de `vantadb-wasm/**` o `vantadb-ts/**` | **Bajo** — reutiliza el job `tests` existente |
| Wheels en PR | ya existente: 3 OS × ~15-30 min | ~45-90 min por PR | **Ya se paga** — no es incremental |
| Full multi-target (wheels + wasm + TS) en PR | 4 runners | ~55-100 min por PR | Alto — la mayor parte es wheels, que ya existe |

**Conclusión de costo:** el incremental real es ~1 runner ubuntu (~10 min) SOLO cuando el PR toca `vantadb-wasm/**` / `vantadb-ts/**` (paths filter). El Fast Gate no se contamina: sigue <5 min en el 90% de PRs que no tocan bindings.

---

## 3. Decisión

**PLAN — no defer.** El gap es real (build wasm32/TS roto pasa CI y solo falla en release), y el costo de cerrarlo es bajo porque el job `tests` ya existe en `release-npm-61.yml` — solo falta el trigger de PR con paths filter.

### Plan propuesto (para aprobación del lead — NO implementado aquí)

**Archivo a modificar:** `.github/workflows/release-npm-61.yml` (único cambio)

1. **Agregar trigger `pull_request`** al `on:` (junto a tags y dispatch):
   ```yaml
   pull_request:
     branches: ["main"]
     paths:
       - "vantadb-wasm/**"
       - "vantadb-ts/**"
       - "Cargo.toml"
       - "Cargo.lock"
       - ".github/workflows/release-npm-61.yml"
   ```
2. **El job `tests` existente** (release-npm-61.yml:31-71) ya hace exactamente lo que falta: setup Node, wasm-pack, `wasm-pack build --release` (target wasm32), `npm run build`, `npm test`. Cero jobs nuevos — solo trigger.
3. **No tocar `publish-wasm`/`publish-ts`**: sus condiciones (`startsWith(github.ref, 'refs/tags/v')` || `workflow_dispatch`) ya excluyen PRs → no hay riesgo de publicar desde PR.
4. **Path filter** garantiza que PRs core (Rust sin bindings) no paguen el job → Fast Gate intacto.

**Alternativa descartada:** agregar wasm32 a `ci-rust-10` `experimental-check` (agregar `rustup target add wasm32-unknown-unknown` + `wasm-pack build`). Descartada porque: (a) compila para host hoy, no para wasm32; (b) `experimental-check` es `continue-on-error` por diseño (circuit breaker, CI_POLICY.md:96-98) — no daría gate real; (c) duplicaría el job `tests` que ya existe en release-npm-61.

### Criterios de aceptación del plan (para el PR de implementación)

- PR que toca `vantadb-wasm/**` → job `tests` corre y bloquea si `wasm-pack build` o `npm test` fallan.
- PR core (solo `src/**`) → job NO corre (path filter).
- Release tag → comportamiento idéntico al actual (build + publish).
- Fast Gate `ci-rust-10` sin cambios.

---

## 4. Conclusión

- **Wheels multi-OS por PR: ya cubierto** (`release-wheels-60.yml:10-17`) — no hay nada que hacer.
- **WASM (wasm32) + TS por PR: gap real** — el único build wasm32 es push-only + non-blocking (`ci-rust-10.yml:388-395`).
- **Plan:** un solo cambio YAML en `release-npm-61.yml` — agregar trigger `pull_request` con paths filter; el job `tests` existente cubre el build. Costo: ~10 min por PR de bindings, 0 min para PRs core.
- **Defer no justificado:** el costo es bajo y el job ya existe; diferir mantiene el riesgo de release roto post-tag sin beneficio.

**Decisión final propuesta:** ✅ implementar el plan (1 change en release-npm-61.yml) — sujeto a aprobación del lead antes de editar workflows.