---
title: "FIND-808 — FEATURES.md miente en tres puntos medidos"
kind: task
status: in-progress
description: "Tres afirmaciones falsas medidas contra HEAD en docs/dev/architecture/FEATURES.md: `default` con nueve flags (son cinco, Cargo.toml:156), wheel con default-features=false (vantadb-python/Cargo.toml:25 no lo hace: default + rayon) y un ci-rust-10.yml inexistente (el vivo es ci-rust.yml). Se corrigen con cita real; la matriz de combinaciones se enlaza, no se duplica."
tags: [vantadb, cargo, features, docs]
---

# FIND-808 — el registro de features restateaba el `Cargo.toml` y se desincronizó

> **Carril:** documentación, **sin compilar Rust**. Escritura: este task file ·
> `docs/dev/architecture/FEATURES.md` · `docs/dev/Backlog.md` (fila `FIND-808`) ·
> `docs/dev/avance/activo/docs.md`.
> **Prohibido y respetado:** `docs/feature-matrix.md` (es la fuente) · `Cargo.toml` · `Cargo.lock` ·
> `.github/**` (`FIND-807` en vuelo) · `src/**` (`FIND-798` compilando) · `vanta-memory/**` ·
> `vantadb-mcp/**` · `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md`.

## La ruta de la fila estaba mal, y se midió antes de editar

La fila del Backlog dice `docs/FEATURES.md:32` y `docs/FEATURES.md:51`. **Ese fichero no
existe** (`Test-Path docs/FEATURES.md` → `False`; `git ls-files | grep FEATURES` → 4 rutas,
ninguna en `docs/` raíz). El registro real es `docs/dev/architecture/FEATURES.md`. Las dos
líneas citadas coinciden con las del fichero real, así que el hallazgo heredado es correcto
y lo que estaba mal era el prefijo del path. Se corrige acá; no se reescribe la fila
(el registro de completado va a `docs/dev/avance/activo/docs.md`, no al Backlog).

## Medición contra `HEAD` (`b361c96d4`, rama `develop`, 2026-11-27)

| Punto | Doc decía | Medido | Fuente exacta |
|---|---|---|---|
| (a) `default` | nueve: `cli, arrow, fjall, roaring, advanced-tokenizer, memmap2, fs2, sysinfo, rayon` (`:32`) | **cinco**: `cli, fjall, memmap2, fs2, roaring` | `Cargo.toml:156` |
| (b) wheel | «`vantadb-python` no la activa (usa `default-features=false, features=["fjall","memmap2","rayon"]`)» (`:51`) | **sin** `default-features=false` → defaults **encendidos** + `fjall,memmap2,rayon` = **`default + rayon`** | `vantadb-python/Cargo.toml:25` |
| (c) workflow | `.github/workflows/ci-rust-10.yml` (`:15`, `:36`, `:62`) | **no existe**; el vivo es `ci-rust.yml` | `Get-ChildItem .github/workflows` → 20 ficheros, ninguno `ci-rust-10.yml` |

Comandos crudos:

```
$c = Get-Content Cargo.toml; $c[155]
default = ["cli", "fjall", "memmap2", "fs2", "roaring"]      # línea 156

Get-Content vantadb-python/Cargo.toml | Select-Object -Skip 24 -First 1
vantadb = { workspace = true, features = ["fjall", "memmap2", "rayon"] }   # línea 25

Select-String -Path .github/workflows/*.yml -Pattern 'failpoints'   # 20 workflows, sin ci-rust-10
```

### Drift derivado del mismo párrafo (medido al verificar (a) y (c))

No son puntos nuevos del ticket: son la **misma afirmación propagada**, y se corrigen porque
el párrafo que las contiene es el que se reescribe.

1. Cuatro filas marcaban `✅ default` y no están en `default`: `arrow` (`:34`),
   `advanced-tokenizer` (`:37`), `sysinfo` (`:40`), `rayon` (`:41`).
2. `:70` decía que Miri corre `--features "cli,fjall,memmap2,fs2"`. Medido:
   `cli,fjall,memmap2,fs2,encryption` (`ci-rust.yml:1128-1129` y `:1145-1146`; el comentario
   `:1119-1122` explica por qué no lleva `roaring`).
3. `:74` remataba «11 features están ejercidas en runtime». Con nueve defaults la aritmética
   cerraba; con cinco no. La unión real de la lane de tests es **8**
   (`default` ∪ `cli,arrow,tls,opentelemetry`, `ci-rust.yml:469/512/562`) — el conteo se cae
   con la tabla que lo producía.
4. `:45` decía que `remote-inference` «no está en features de test de CI». Medido: el job
   `Tests (embeddings features)` lo corre con `--lib` en las dos legs
   (`ci-rust.yml:676-687`, matriz `[remote-inference, embed-local]`).
5. `:46` decía que `chaos_integrity` «**no corre** en CI». Medido: corre en la certificación
   pesada (`certification.yml:197-198`, `nightly.yml:133-134`), no en el Fast Gate.

## Impacto mapeado (Regla 0)

Leídos completos antes de editar:

- `docs/dev/architecture/FEATURES.md` (87 líneas, completo)
- `docs/feature-matrix.md` (123 líneas, completo — solo lectura)
- `Cargo.toml` `[features]` (`:155-199`) + declaraciones `optional = true` de `rayon`/`sysinfo`
- `vantadb-python/Cargo.toml` (completo, 38 líneas)
- `.github/workflows/ci-rust.yml` — índices de jobs, `:280` (clippy), `:469/512/562` (tests),
  `:655-687` (embeddings), `:972-1008` (release-combo / feature-combos), `:1093-1150` (miri),
  `:1177-1252` (asan)
- `.github/workflows/certification.yml:188-198`, `nightly.yml:133-134` (failpoints)
- `docs/dev/Backlog.md:932` (mi fila) y `CHANGELOG.md`/`git grep` para el universo de citas
- `scripts/docs/check-docs.mjs`, `check-links.mjs`, `lib.mjs` (contrato de `kind` y de enlaces)

**Referencias entrantes al fichero (quién lo enlaza/cita):**

| Referencia | Qué espera |
|---|---|
| `docs/feature-matrix.md:15` | «el registro por feature es `FEATURES.md`» — **la matriz delega el registro acá**; borrar la tabla dejaría un hueco y rompería su cross-ref |
| `docs/feature-matrix.md:33,35,79,80` | cita `FEATURES.md:32/51/48/60` como *stale* (ver Deuda) |
| `docs/user/operations/EXPERIMENTAL_FEATURES.md`, `CI_POLICY.md`, `VANTADB-PRO-FEATURES.md` | §Cross-References del propio doc |

**Referencias salientes que se tocan:** `.github/workflows/ci-rust-10.yml` → `ci-rust.yml`
(3 sitios) y una entrada nueva a `docs/feature-matrix.md`.

**Veredicto de impacto:** BAJO y contenido a un fichero. La tabla por-feature **no se borra**
— la matriz la declara su dueño, y `docs/feature-matrix.md:15` es una referencia entrante
explícita. Lo que se deja de duplicar es lo que la matriz sí posee: la tabla agregada de
cobertura de CI y el agregado de `default`.

## Spec — decisiones por ítem (con evidencia)

| Ítem | Decisión | Evidencia / razón |
|---|---|---|
| Fila `default` | Listar los **cinco** literales y citar `Cargo.toml:156`, apuntando a la matriz para combinaciones | Contrato del ticket: la lista debe coincidir literalmente con `Cargo.toml:156`. Slice de una línea, verificable a ojo |
| Tabla §«Cobertura en CI» | **Se reemplaza por 3 bullets medidos + puntero a la matriz §2** | Es la duplicación que envejeció (`:70`, `:74`). La matriz ya mide job→combinación→minutos; `docs/feature-matrix.md:14-18` delimita ambos docs |
| Tabla por-feature (§«Feature table») | **Se conserva** (es lo único que este doc posee: `file:line` de cada gate) | `docs/feature-matrix.md:14-15` delega explícitamente el registro por feature a este fichero |
| Filas `arrow`/`advanced-tokenizer`/`sysinfo`/`rayon` | `✅ default` → `🟡 opt-in`; se corrige la nota `CI:` de las cuatro | `Cargo.toml:156` no las incluye; `rayon` entra por unión vía `vantadb-python/Cargo.toml:25` (`feature-matrix.md:90-93`), `advanced-tokenizer` solo en `ci-rust.yml:1008` |
| `ci-rust-10.yml` | `ci-rust.yml` + línea real en cada cita | `:1119-1146` (roaring/Miri); §CI reescrita |
| `python_sdk` (`:51`) | Reemplazar la cláusula falsa por la medición `default + rayon` | `vantadb-python/Cargo.toml:25` |
| Features / defaults | **No se tocan** | Explícitamente fuera de scope (contrato 4) |

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Verificar los tres puntos contra `HEAD` con `Cargo.toml:156`, `vantadb-python/Cargo.toml:25` y `Get-ChildItem .github/workflows` | ✅ |
| 2 | Leer `docs/feature-matrix.md` y confirmar el reparto de responsabilidades (registro vs combinaciones) | ✅ |
| 3 | Crear este task file con impacto mapeado y spec | ✅ |
| 4 | Editar `docs/dev/architecture/FEATURES.md`: fuentes, fila `default`, 4 status marks, `python_sdk`, citas de workflow, §CI | ✅ |
| 5 | Verify mecánico: `check-docs.mjs` · `check-links.mjs` · `rg ci-rust-10` en 0 · lista de flags == `Cargo.toml:156` | ✅ |
| 6 | OCR review + review P2-01 (una ronda) | ✅ |
| 7 | Cierre: fila del Backlog, entrada en `docs/dev/avance/activo/docs.md`, commit con índice aislado | ✅ |

## Verify (contrato mecánico)

```
node scripts/docs/check-docs.mjs      → exit 0
node scripts/docs/check-links.mjs     → exit 0
rg -n "ci-rust-10" docs/dev/architecture/FEATURES.md   → 0 matches
doc :  cli, fjall, memmap2, fs2, roaring
cargo: cli, fjall, memmap2, fs2, roaring                 (Cargo.toml:156)
```

## Review P2-01

Tier: **fast** — el diff toca solo `docs/**` (`docs/dev/architecture/FEATURES.md`,
este task file, `docs/dev/avance/activo/docs.md`, fila del Backlog). Sin `docs/api/**`,
sin `src/**`. Gate: `dev-tools/verify.ps1` fast + veredicto registrado (ver §Review abajo).

## Deuda / NOTICED BUT NOT TOUCHING

- **`docs/feature-matrix.md:31-37` queda desactualizada por este cambio** (prohibida acá):
  describe la fila `default` de nueve flags y el `default-features=false` como *stale
  hoy*. Tras este commit ya no lo están. Es una línea de la matriz y la toca su carril.
  **No se crea fila `FIND-*` nueva** porque el ID siguiente (814+) está en disputa con los
  carriles vivos (`FIND-807`/`FIND-798`); el orquestador asigna el número.
- `docs/feature-matrix.md:79` discute `server` sin `cli` citando `FEATURES.md:48` — esa fila
  no se tocó (no la contradice ninguna medición de este ticket).
- Las notas `CI: default + Miri (...)`, en las filas `fjall`/`memmap2`/`fs2`, listan un
  subconjunto de la string real de Miri (falta `encryption`). No son falsas (nombran un
  subconjunto) y la string exacta queda una sola vez en §«Cobertura en CI».
- `encryption` (fila `:47`) no lleva marca `✅ ci` aunque Miri lo corre: incompleto, no falso.
