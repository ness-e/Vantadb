# Propuesta de Investigación — INV-017: sccache en CI

> **ID:** `INV-017`
> **Categoría:** Devops (CI/CD)
> **Fecha:** 2026-08-02
> **Estado:** ✅ Investigación Completada — Alimenta GH-143
> **Fuente:** docs/Backlog.md:217

---

## 1. Contexto y Objetivos

GH-143 pide acelerar el CI de Rust. Esta investigación evalúa integrar
[sccache](https://github.com/mozilla/sccache) al pipeline actual (que ya usa
`Swatinem/rust-cache`) y documenta el hallazgo de un drift documental en
`.opencode/AGENTS.md`.

---

## 2. Estado real del CI (verificado)

> **Nota de estado 2026-08-04:** este análisis corresponde al 2026-08-02. **El plan de este doc se aplicó literalmente y está COMPLETO** (ver §5 → implementado en `.github/actions/rust-setup/action.yml:72-81`; fix nextest en `ci-rust-10.yml:136-139`; drift de AGENTS.md corregido en `.opencode/AGENTS.md:333-335`; Backlog: INV-017 ✅ y GH-143 ✅). Las afirmaciones de abajo eran ciertas al 08-02, no hoy.

- `rg -rln sccache .github/` → **0 resultados** (al 2026-08-02). sccache NO estaba implementado en ningún workflow.
- `.opencode/AGENTS.md` (sección "CI: sccache") afirmaba: *"Agregado al workflow `ci-rust-10.yml` mediante `mozilla-actions/sccache-action`"* — **DRIFT documental**: la integración nunca existió.
- Baseline medible: run `30731463004` de `ci-rust-10.yml` → el job más lento es **Tests (Windows) 14m29s**, dominado por `cargo install cargo-nextest --locked` (`ci-rust-10.yml:138` en ese momento; hoy esa línea usa `taiki-e/install-action`) en vez de `taiki-e/install-action` (que Linux/macOS ya usan vía `rust-setup:79-83`).

---

## 3. Compatibilidad sccache + GitHub Actions

**SÍ — con configuración mínima.** Fuente: README de `mozilla-actions/sccache-action`.

- Step: `uses: mozilla-actions/sccache-action@v0.0.11`
- La acción instala el binario y setea `SCCACHE_PATH` automáticamente (válido en Linux/macOS/Windows).
- Para Rust se requieren 2 env vars:
  ```yaml
  env:
    SCCACHE_GHA_ENABLED: "true"
    RUSTC_WRAPPER: "sccache"
  ```
- Backend = **GHA cache automático**: usa `ACTIONS_CACHE_URL`/`ACTIONS_RUNTIME_TOKEN` que GitHub inyecta solos. Sin infraestructura adicional (sin Redis/S3/almacenamiento propio).
- El action crea automáticamente un step `Post Run sccache-cache` que sube el cache al final del job.
- Versión sccache: pinneable vía input `version` (mínimo v0.11.0; recomendado `v0.16.0`).
- **Nota sobre composite actions**: no existe `env` a nivel `runs` para acciones compuestas (solo Docker actions lo tienen). El mecanismo documentado es escribir las env vars a `$GITHUB_ENV` en un step `run` — aplica a todos los steps posteriores del job y a todos los runners.

---

## 4. Complementariedad vs redundancia con Swatinem/rust-cache

Ambos coexisten; **no son redundantes** — atacan capas distintas:

| Aspecto | `Swatinem/rust-cache` | sccache |
|---|---|---|
| Granularidad | Cachea `target/` entero (blob grande) | Cachea objetos compilados individuales (`.rlib`) |
| Restore | Lento (descomprimir blob grande) | Rápido, lazy por crate |
| Invalida | Todo al cambiar `Cargo.lock` | Solo los crates afectados |
| Compartición entre jobs | Por job (keys por OS/ref) | Comparte hits entre jobs paralelos del mismo run (clippy/test/coverage compilan los mismos crates) |
| Límite | Compiten por el mismo cache GHA (10GB, eviction LRU 7 días) | idem |

**Riesgo:** ambos consumen del mismo bucket de cache de GHA. Con un workspace de ~17 crates esto es aceptable; monitorear si se acerca al límite de 10GB.

---

## 5. Diseño de integración mínima (punto único en rust-setup)

Integrar en `.github/actions/rust-setup/action.yml` beneficia a **todos** los jobs
que la usan (fmt, clippy, test Linux/macOS, msrv, minimal-versions, coverage,
experimental-check, audit, deny) sin tocar cada workflow:

```yaml
    - name: Set up sccache
      uses: mozilla-actions/sccache-action@v0.0.11
      with:
        version: v0.16.0

    - name: Enable sccache for cargo
      shell: bash
      run: |
        echo "SCCACHE_GHA_ENABLED=true" >> "$GITHUB_ENV"
        echo "RUSTC_WRAPPER=sccache" >> "$GITHUB_ENV"
```

- Ubicación: después de `Rust cache` (Swatinem) y **antes del primer comando cargo** del job.
- `RUSTC_WRAPPER=sccache` NO interfiere con `taiki-e/install-action` (descarga binarios precompilados, no invoca cargo) ni con el fix del nextest de Windows.
- **Fuera de alcance:** `test-windows` y `miri` no usan rust-setup → no reciben sccache (no cambiarlos).

---

## 6. Impacto estimado

| Cambio | Ahorro estimado |
|---|---|
| Fix `cargo install cargo-nextest` → `taiki-e/install-action` en Windows | **~4-5 min** del bottleneck (14m29s → ~9-10m, ~30%) — el mayor ROI |
| sccache en rust-setup | 0-15% wall-clock sobre jobs ya cacheados (cache fría: sin beneficio; cache caliente: reduce rebuilds) |
| **Total combinado** | Cumple DoD de GH-143: ≥20% más rápido (≤11m36s) |

---

## 7. Hallazgo del drift en `.opencode/AGENTS.md`

- La sección "CI: sccache" afirmaba una implementación que nunca existió (0 hits de sccache en `.github/`).
- **Acción:** reescribir la sección para reflejar la realidad — integración planeada/implementada según GH-143, con acción exacta, env vars y ubicación (`.github/actions/rust-setup/action.yml`).

---

## 8. Veredicto ROI

sccache es **barato de integrar** (2 steps en rust-setup, sin infra) pero el ahorro
de wall-clock sobre el CI ya cacheado es **modesto (0-15%)**. El fix del nextest de
Windows es el cambio de mayor ROI (~30% del bottleneck). **Recomendación: hacer ambos.**

---

## 9. Referencias

- [mozilla-actions/sccache-action — README](https://github.com/mozilla-actions/sccache-action)
- [GitHub Docs — Metadata syntax (composite actions)](https://docs.github.com/en/actions/creating-actions/metadata-syntax-for-github-actions#runs-for-composite-actions)
- [mozilla/sccache — GitHub Actions cache backend](https://github.com/mozilla/sccache)
