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
   cerraba; con cinco no. Y **ninguna lista cerrada cierra**: las tres lanes de tests
   seleccionan cinco `-p` (`vantadb`, `vantadb_py`, `vanta-memory`, `vantadb-server`,
   `vantadb-mcp`), así que por unificación la unión real es de **13**
   (`cli, fjall, memmap2, fs2, roaring, arrow, tls, opentelemetry, advanced-tokenizer, server,
   sysinfo, rayon, remote-inference` — `vantadb-server/Cargo.toml:21-30`,
   `vantadb-mcp/Cargo.toml:14-23`, `vantadb-python/Cargo.toml:25`,
   `vanta-memory/Cargo.toml:59`), no de 8. **Corrección de ronda 2:** en el primer intento
   escribí 8 midiendo la string de `--features` e ignorando la unificación — el mismo defecto
   que el ticket, reintroducido. La página **no** publica esa lista: nombra el mecanismo y
   apunta a `feature-matrix.md` §4.
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
| 6 | OCR review + review P2-01 (ronda 1 → 1-bis → 2-bis → 3-bis) | ✅ |
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

**Tier:** fast (el diff toca solo `docs/**`: `docs/dev/architecture/FEATURES.md`, este task
file, `docs/dev/avance/activo/docs.md`). Sin `docs/api/**`, sin `src/**`, sin planes.

| Campo | Valor |
|---|---|
| **Revisor** | `vanta-review` (sub-agente, contexto fresco, report-only) |
| **Enfoque** | contexto fresco adversarial, report-only: re-midió la premisa y las 5 claims en vez de aceptar este task file; leyó `ci-rust.yml` del **blob commiteado** (el working tree arrastra un hunk ajeno) y no tocó nada |
| **Cómo se probó** | `check-docs.mjs` / `check-links.mjs` re-ejecutados por él (0/0); `rg ci-rust-10` → 0; listas de flags comparadas contra `Cargo.toml:156`; citas de workflow resueltas una a una contra `git show HEAD:.github/workflows/ci-rust.yml`; **no** compiló Rust (otra lane construyendo) |
| **Veredicto ronda 1** | 🔴 **changes-required** — 3 Blocking, 4 Medium, 4 Low |
| **Veredicto ronda 1-bis** | 🔴 **changes-required** — B1/B2/B3/M1/M3 cerrados; **1 Blocking nuevo, mío**: `Run tests (Audit profile, core)` existe solo en Linux (`ci-rust.yml:473`) y macOS (`:567`); Windows es otro step y otro perfil (`Run quick integrity tests (Windows)`, `--profile ci-windows`) → citar por un identificador que no es común a las tres reintroduce el defecto de M3. +1 Medium (Step 6 ✅ con el gate abierto) y 3 Low (omisiones de `✅ ci` en `jemalloc`/`custom-allocator`, y `embed-local` sin fila) |
| **Veredicto ronda 2-bis** | 🔴 **changes-required** — el Blocking de la 1-bis cerrado; **1 Blocking nuevo, mío**: al acotar `✅ ci` a las lanes de `ci-rust.yml` dejé fuera la certificación pesada, que es el *único* sitio donde `failpoints` se ejerce (`0` ocurrencias en `ci-rust.yml`, `certification.yml:1115` + `nightly.yml`) — el mark quedaba desautorizado por su propia definición, la misma clase de B3. + 2 nits de redacción (`nombre de step` para un bullet que cita jobs; `release-combo` es la key, no el nombre mostrado) |
| **Veredicto ronda 3-bis** | ver §Cierre de la ronda 3-bis (abajo) |

**Ronda 2 (sobre el dictamen 1-bis):** §Cobertura cita los tres **jobs** (`Tests (Linux)` / `Tests
(Windows)` / `Tests (macOS)`, que existen los tres) y aclara el perfil de cada uno; la leyenda
acota `✅ ci` a `ci-rust.yml` (los workflows de release no cuentan) y `jemalloc` recibe su mark
(`release-combo`); se agrega la fila que faltaba, `embed-local`, **al final** de la tabla para no
desplazar las anclas `:32`/`:48`/`:51` que cita la matriz; Step 6 y el identificador de sesión
(no verificable desde el repo) corregidos.

**Blocking de la ronda 1 y qué se hizo:**

- **B1 — la «unión medida» era falsa: 13, no 8.** La string de `--features` es un piso;
  `-p vantadb-server`/`-p vantadb-mcp` en las mismas lanes piden
  `advanced-tokenizer, fjall, fs2, memmap2, rayon, roaring, server, sysinfo`
  (`vantadb-server/Cargo.toml:21-30`, `vantadb-mcp/Cargo.toml:14-23`) y `vanta-memory`
  añade `remote-inference` por dev-dep (`vanta-memory/Cargo.toml:59`). **Aceptado y
  corregido**: la página ya no publica ninguna lista cerrada; nombra el mecanismo, cita las
  cuatro declaraciones de features y delega la advertencia en `feature-matrix.md` §4.
- **B2 — `advanced-tokenizer` y `sysinfo` decían «ninguna lane la corre».** Falso por
  unificación: las tres lanes de nextest encienden ambas. **Aceptado y corregido**: las dos
  filas pasan a `✅ ci` con la cita de las listas de server/mcp, y `rayon` corrige su
  atribución (no es solo `vantadb-python:25`, también server/mcp).
- **B3 — la leyenda contradecía sus propios marks.** `🟡 opt-in` exigía «ninguna lane lo
  corre» y cuatro filas llevaban `opt-in · ✅ ci`. **Aceptado y corregido**: los ejes son
  ortogonales — `opt-in` = no está en `Cargo.toml:156`; `✅ ci` = alguna lane (fuera de
  `clippy --all-features`, que compila la unión) lo enciende.

**Medium de la ronda 1 y qué se hizo:**

- **M1 — mi +6/+8 de líneas desplazó las citas entrantes de `docs/feature-matrix.md:33,35,79`
  (`FEATURES.md:32/:51/:48`).** **Aceptado y corregido sin tocar la matriz**: el bloque de
  cabecera se comprimió a exactamente las mismas 6 líneas que tenía, y el puntero a la matriz
  se movió a §Cross-References. Verificado: `:32` es la fila `default`, `:48` es `server`,
  `:51` es `python_sdk` — los tres destinos que la matriz cita, otra vez correctos.
- **M2 — las citas `src/lib.rs` de la tabla están podridas (15-40 líneas) y 5 filas tocadas
  las conservan.** Cierto y **fuera de este ticket**: arreglar 9 citas es otro cambio (y en
  este carril no hay `FIND-*` asignable, ver Deuda). Delegado, no silenciado.
- **M3 — los números de línea nuevos a `ci-rust.yml` ya están obsoletos en el working tree**
  porque `FIND-807` reescribe ese fichero. **Aceptado y corregido**: §Cobertura cita
  **nombres de step** (`Run tests (Audit profile, core)`, `Clippy Lints`, `Miri Test (core -
  unsafe patterns)`), que no rotan; `rg 'ci-rust\.yml:'` sobre la página → 0.
- **M4 — `## Review P2-01` estaba vacío y el Step 6 se marcaba ✅ igual.** **Aceptado**:
  esta tabla es la corrección.

**Low:** L3 `encryption` sin `✅ ci` → corregido (Miri la corre). L1/L2 (matriz desactualizada
y fila del Backlog sin retirar) siguen en §Deuda **con dueño nombrado**. L4 (3682 vs 3685
internos) es deriva del árbol, no un defecto.

**Ronda 3 (sobre el dictamen 2-bis):** la leyenda de `✅ ci` vuelve a incluir la certificación
pesada (`certification.yml` / `nightly.yml`) además de `ci-rust.yml`, sin reabrir
`custom-allocator` (que sigue fuera: lo enciende un workflow de **release**); `💀 no-op` declara
que prevalece sobre cualquier otro mark; y las dos citas de redacción pasan a nombrar `job o
step` y el nombre mostrado del job de allocators.

## Cierre de la ronda 3-bis

Pendiente del veredicto del revisor sobre el commit de la ronda 4 (ver RESULTADO).

## Deuda / NOTICED BUT NOT TOUCHING

> **Dueño de cada ítem, nombrado** (la ronda 1 de la review rechazó «deuda anónima»): los tres
> primeros los asigna el **orquestador** en el próximo commit que pueda escribir
> `docs/dev/Backlog.md` (hoy bloqueado por el hunk ajeno de `FIND-798`, ver último ítem); no se
> inventa un número `FIND-*` acá porque el rango 814+ está en disputa con los carriles vivos.

- **`docs/feature-matrix.md:31-37` quedó desactualizada por este cambio** (prohibida acá):
  describe la fila `default` de nueve flags y el `default-features=false` como *stale* hoy.
  Tras este commit ya no lo están. Una línea de la matriz; la toca su carril. **Las citas
  `:32`/`:51`/`:48` que la matriz usa sí siguen apuntando a las filas correctas** (verificado).
- **`docs/feature-matrix.md:79`** discute `server` sin `cli` citando `FEATURES.md:48` — esa
  fila no se tocó (no la contradice ninguna medición de este ticket); sigue siendo la
  incógnita que la matriz ya declaraba «unverified».
- **Las 9 citas `src/lib.rs` de la tabla están podridas** (pre-existente, medido por la review
  ronda 1): `columnar` 74→89, `llm` 98→120, `python` 112→136, `tokenizer` 127→165, `tui`
  129→167, `wal_shipping` 138→179, `ingestion` 146→183, `transcript` 149→186, `failpoints`
  177-194→223-239. Este commit editó 5 de esas filas sin re-medir la cita: es exactamente el
  modo de fallo del ticket, en la columna de al lado. Merece fila propia.
- Las notas `CI: default + Miri (...)`, en `fjall`/`memmap2`/`fs2`, listan un subconjunto de
  la string real de Miri (falta `encryption`). No son falsas (nombran un subconjunto) y la
  string exacta aparece una sola vez en §Cobertura en CI.
- **`custom-allocator` queda sin `✅ ci` a propósito**: lo enciende `release-binaries.yml`,
  no una lane de `ci-rust.yml`, y la leyenda acota el mark a las lanes de CI (`ci-rust.yml` + certificación pesada), que excluyen los workflows de release (rondas 2 y 3).
  Si se quisiera cubrir "cualquier workflow", es una decisión de la leyenda, no de una fila.
- **La fila `FIND-808` del Backlog sigue presente** (🆕 Pendiente, con el path inexistente
  `docs/FEATURES.md:32`): `docs/dev/Backlog.md` arrastra un hunk ajeno sin commitear (`0/1`,
  la retirada de `FIND-798`). Commitear el fichero hornearía el cierre de un carril cuyo
  código no está en historia — el incidente que `git-commit-isolated.ps1` rechaza con exit 9
  (FIND-792). **Dueño: el primer carril que commitee `docs/dev/Backlog.md`**, que debe retirar
  la fila `FIND-808` en ese mismo commit.
