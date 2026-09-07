# BND-09 — targets linux musl en napi

> **Campaign:** a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
> **Plan:** `docs/plans/2026-09-04-durability-release-readiness.md` (Task 10, Wave 3)
> **Estado:** ✅ COMPLETO (2026-09-06, verificación sin código — contrato ya cumplido en HEAD)
> **Gated por:** BND-08 (`e9843100` pipeline verificado ✅)

## Resumen

El contrato de BND-09 (targets musl presentes en `napi.targets` + matriz CI los incluye) ya
está cumplido en HEAD desde los commits previos de BND-08. Esta tarea **no introduce
cambios productivos** — solo valida y documenta.

## Contrato

> *"targets musl presentes en config + matriz CI los incluye; si el build cross falla por
> toolchain, documentar requisito y cerrar parcial con evidencia."*
> *"Si la toolchain NO está disponible localmente (sin docker/podman/cross): validar
> matriz CI + documentar; sin código forzado."*

## Verificación de Contrato

### S1 — `napi.targets` contiene los musl requeridos ✅

Archivo: `vantadb-node/package.json` líneas 36-44

```json
"targets": [
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",       ← musl x86_64 ✅
  "aarch64-unknown-linux-gnu",
  "aarch64-unknown-linux-musl",      ← musl aarch64 ✅
  "aarch64-apple-darwin",
  "x86_64-apple-darwin"
]
```

| Target | Estado |
|--------|--------|
| `x86_64-unknown-linux-musl` | ✅ presente |
| `aarch64-unknown-linux-musl` | ✅ presente |

Origen: commit `ed75cb0b` ("feat(node): add musl targets + npm release workflow (BND-08/09)").

### S2 — Matriz CI incluye los jobs musl ✅

Archivo: `.github/workflows/release-npm-node.yml` líneas 44-52

```yaml
- target: x86_64-unknown-linux-musl
  runs-on: ubuntu-latest
  rust-target: x86_64-unknown-linux-musl
- target: aarch64-unknown-linux-musl
  runs-on: ubuntu-latest
  rust-target: aarch64-unknown-linux-musl
```

Cada job musl usa `ubuntu-latest` (runner con toolchain musl disponible vía `rustup target add`)
y la CI step `dtolnay/rust-toolchain@stable` con `target: ${{ matrix.rust-target }}` resuelve
la toolchain automáticamente.

### S3 — Toolchain local (DISCOVERY, fuera del contrato estricto)

Estado del entorno local (Windows, dev-tools sin cross):

```
$ where docker   → not found
$ where podman   → not found
$ where cross    → not found
$ rustup target list --installed:
  aarch64-apple-darwin
  wasm32-unknown-unknown
  x86_64-apple-darwin
  x86_64-pc-windows-msvc
  x86_64-unknown-linux-gnu
  (NO musl targets installed locally)
```

**Interpretación:** sin `docker`/`podman`/`cross` y sin targets musl instalados, no es posible
ejecutar el build cross-compile en este runner. Esto NO invalida el contrato porque:

1. La CI (`ubuntu-latest`) tiene `rustup target add` automático para cada job musl (ver
   `release-npm-node.yml:75-77`).
2. El contrato explícitamente acepta este modo: *"sin código forzado"*.
3. El gate BND-08 (`e9843100`) ya validó el pipeline npm en dry-run.

### S4 — Sin código forzado (Scope discipline)

- **0 archivos productivos modificados** en esta tarea.
- **0 nuevos tests necesarios** — el contrato pide presencia en config/CI, no nuevos tests
  de compilación (CI provee la verificación).
- **0 docs nuevos obligatorios** — `release-npm-node.yml` es self-documenting.

## Steps Atómicos Ejecutados

| # | Step | Estado |
|---|------|--------|
| S1 | Localizar `vantadb-node/package.json` napi.targets | ✅ verificado (líneas 36-44) |
| S2 | Localizar `.github/workflows/release-npm-node.yml` matrix | ✅ verificado (líneas 44-52) |
| S3 | Confirmar BND-08 gate cumplido | ✅ commit `e9843100` en HEAD |
| S4 | Validar toolchain local | ✅ documentado (sin docker/cross/musl) |
| S5 | Decidir cierre: validación sin código (cumple contrato) | ✅ |
| S6 | Actualizar plan file Task 10 → COMPLETO (sin stagear) | ✅ |

## Evidencia (por claim)

| Claim | Evidencia | Confianza |
|-------|-----------|-----------|
| `x86_64-unknown-linux-musl` en napi.targets | `vantadb-node/package.json:39` | alta |
| `aarch64-unknown-linux-musl` en napi.targets | `vantadb-node/package.json:41` | alta |
| Matrix CI incluye musl x86_64 job | `release-npm-node.yml:44-46` | alta |
| Matrix CI incluye musl aarch64 job | `release-npm-node.yml:50-52` | alta |
| Musl targets usan ubuntu-latest (toolchain auto) | `release-npm-node.yml:45,51` + `rust-toolchain@v1` step | alta |
| Origen de los cambios (pre-existente, no nuevos) | `git show ed75cb0b` — "feat(node): add musl targets + npm release workflow (BND-08/09)" | alta |
| Gate BND-08 cumplido | `git log --oneline` muestra `e9843100` ✅ | alta |
| Toolchain musl ausente en runner local | `rustup target list --installed` no muestra musl; `where docker/podman/cross` → not found | alta |

## Invariantes

- **No** se modifica `vantadb-node/package.json` (ya correcto en HEAD).
- **No** se modifica `.github/workflows/release-npm-node.yml` (ya correcto en HEAD).
- **No** se stagean archivos ajenos al scope de esta tarea (worktree puede contener cambios
  de otras sesiones — respetados).
- **No** se introducen nuevos tests (CI provee verificación cross-compile; contrato lo permite).

## Deuda

**Ninguna.** El contrato se cumple con la configuración existente en HEAD. La validación
cross-compile real ocurre en CI cuando se dispara el workflow (push a `main`/`develop` en
`vantadb-node/**` o tag `node-v*.*.*`).

## Artefactos

- Task file: este archivo (`docs/tasks/BND-09.md`)
- Plan file update: `docs/plans/2026-09-04-durability-release-readiness.md` (Task 10 → ✅
  COMPLETO, **sin stagear** — el orquestador decide commitearlo junto con otros sync de Wave 3)
- **Sin commit nuevo** (contrato cumplido sin diff productivo)

## Lecciones Aprendidas

1. **Contrato cumplido por pre-existencia** — verificar git history antes de generar diff
   nuevo. `ed75cb0b` añadió musl cuando BND-08 se ejecutó; BND-09 solo documenta.
2. **Sin toolchain local ≠ sin cobertura** — CI runner `ubuntu-latest` resuelve toolchain
   musl on-demand; el contrato lo acepta explícitamente.
3. **Documentación > código forzado** — el "sin código forzado" del contrato evita un anti-patrón
   (e.g., añadir `cargo:rustc-link-arguments` o toolchain installer que no resuelve nada).
