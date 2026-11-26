---
title: "FIND-769 - el gate de rustdoc falla en CI y no en local: la superficie unix"
kind: task
status: completed
description: "El step `Deny rustdoc warnings` de `docs-gates.yml` falla en su primer run (run 45804272736, exit 101) con `private_intra_doc_links` mientras la orden identica da 0 en local. Causa discriminada: no es toolchain (1.95.0 en ambos) ni feature set: el unico sitio marcado esta en la doc de un `pub fn` bajo `#[cfg(unix)]`, que en Windows no existe y por eso el instrumento local no puede verlo."
tags: [vantadb, docs, rustdoc, gate, ci]
---

# FIND-769 - el gate de rustdoc falla en CI y no en local

## Contexto

`FIND-763` cableó el gate (`Deny rustdoc warnings (doc gate)`, `RUSTDOCFLAGS:
"-D warnings"` + `cargo doc --no-deps --workspace --keep-going`) sobre la
medición en verde de `FIND-755` / `FIND-761`. En su **primer run real** (run
`45804272736`, SHA `ab3dae37f`) el step salió **101**:

```
error: public documentation for `create_snapshot` links to private item `is_mutable_after_snapshot`
  --> src/storage/engine/mod.rs:1588:11
note: `-D rustdoc::private-intra-doc-links` implied by `-D warnings`
error: could not document `vantadb`
```

La misma orden, medida en local dos veces (`FIND-755` y su reviewer), daba
`exit 0 / 0`. La contradicción es el hallazgo: hay que discriminar la causa
antes de tocar un link.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `.github/workflows/docs-gates.yml` (689 líneas — job
  `rustdoc` `:527-599`, su `rust-setup` `:544-551`, el step del gate `:563-566`,
  el step del artefacto `:574-575`) · `.github/actions/rust-setup/action.yml`
  (136 líneas — input `toolchain` `:4-7`, export `RUSTUP_TOOLCHAIN` `:85-92`) ·
  `rust-toolchain.toml` (18 líneas, `channel = "1.95.0"`) ·
  `src/storage/engine/mod.rs` (`:742-791` el helper privado, `:1560-1614` el
  `pub fn create_snapshot` unix, `:1616-1620` el fallback, `:1774-1801` los dos
  `pub fn snapshot_restore*`) · `docs/dev/tasks/FIND-763.md` (208 líneas) ·
  `docs/dev/avance/activo/docs.md` (§`FIND-755` y §`FIND-761`).
- **Referencias hacia dentro:** `create_snapshot` es API pública de
  `StorageEngine`; el único consumidor del nombre es el índice de API
  (`scripts/docs/check-api-docs.mjs`), que no lee la prosa de los doc comments.
- **Referencias salientes:** el link `[`is_mutable_after_snapshot`]` apunta a
  una `fn` **privada** del mismo módulo (`:761`), compilada en **todas** las
  plataformas.
- **Veredicto de impacto:** el arreglo es **una línea de doc comment** (link →
  code span). Cero símbolos, cero firmas, cero comportamiento. La superficie de
  API no cambia en la superficie pública, que es la que el gate mide: ahí el
  link nunca era resoluble. (El tarball del artefacto se genera con
  `--document-private-items` — `docs-gates.yml`, step siguiente — así que **sí**
  renderizaba un hipervínculo interno válido: aplanar lo quita de ese artefacto.
  Aceptado y declarado: un ítem privado no se enlaza desde la superficie pública,
  y el artefacto no justifica conservar un link que el gate prohíbe.)

## Discriminación de causa (las tres hipótesis, en orden)

| # | Hipótesis | Veredicto | Evidencia |
|---|---|---|---|
| 1 | toolchain | **descartada** | El job fija `toolchain: 1.95.0` (`docs-gates.yml:546`), que el composite exporta como `RUSTUP_TOOLCHAIN` (`action.yml:85-92`), y el log del run lo imprime en el `env:` del step (`RUSTUP_TOOLCHAIN: 1.95.0`). Local: `rustc 1.95.0 (59807616e 2026-04-14)`. **Mismo compilador: no hay versión que pueda cambiar la severidad del lint.** |
| 2 | feature set | **descartada como causa del fallo** | El `env:` del job es `CARGO_TERM_COLOR`+`CARGO_INCREMENTAL` (`:535-537`) y el del step sólo añade `RUSTDOCFLAGS` (`:564-565`). No hay `--all-features` (vive en el step del artefacto, `:575`). Sí hay **unificación de features** del `--workspace` — ver "segundo instrumento" abajo. |
| 3 | estado del árbol / superficie | **CONFIRMADA — con un discriminador que el ticket no listaba: la cfg de plataforma** | El único sitio marcado (`mod.rs:1588`) está en la doc de un `pub fn create_snapshot` **`#[cfg(unix)]`** (`:1611`). En Windows ese ítem **no existe**, así que rustdoc no procesa su doc: el instrumento local mide **otra superficie**. |

**No faltaban ~21 sitios: el lint marca 1.** Y no es que `-D warnings` corte en
el primero: la cara roja de la prueba de dos caras añadió **dos** links privados
deliberados y rustdoc reportó **los dos** en la misma pasada (y siguen los
`could not document` del final). El "1 error" del run es el universo de sitios
privados de la superficie pública del crate `vantadb`; los **otros 7 crates**
(`vantadb-ffi-core`, `vanta-memory`, `vantadb-wasm`, `vanta-proxy`,
`vantadb_py`, `vantadb-mcp`, `vantadb-server`) documentaron en ese mismo run sin
un solo diagnóstico.

### El instrumento local no puede predecir este step (dos razones medidas)

1. **Plataforma.** La superficie de rustdoc depende de la cfg del host: los
   `pub` bajo `#[cfg(unix)]` no se documentan en Windows. Un verde local en
   Windows **no cubre** ningún ítem unix-only.
2. **Unificación de features.** El step corre `--workspace`, y `vanta-proxy`
   declara `vantadb = { workspace = true, features = ["encryption"] }`
   (`vanta-proxy/Cargo.toml:33`), así que en el doc del workspace `encryption`
   está **on**. Un `-p vantadb` (features default) la deja **off** y produce dos
   `unresolved link` espurios en `src/wal.rs:2115,2117` — verificado: `-p
   vantadb` → exit 101 con esos dos; `--workspace` → no aparecen.

El cross-doc al target linux desde Windows (`--target
x86_64-unknown-linux-gnu`, target instalado) **no es viable**: `croaring-sys`
(feature default `roaring`) exige `x86_64-linux-gnu-gcc`. Queda como techo
declarado del instrumento: **el único entorno que vale para la superficie unix
es el runner de CI**, y lo que se puede hacer en local es duplicar el comando
exacto (workspace, no `-p`) y no extrapolar su verde a los ítems unix-only.

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Contar los sitios que marca el lint bajo la superficie de CI (log del run) | ✅ 1 |
| 2 | Discriminar causa (toolchain / feature set / cfg) | ✅ cfg de plataforma |
| 3 | Prueba de dos caras con el comando exacto del step | ✅ |
| 4 | Aplanar el link a code span (patrón `FIND-755`/`FIND-761`) | ✅ 1 línea |
| 5 | Comentario en el step: la superficie no es la de tu máquina, y por qué | ✅ 5 líneas + 1 en blanco |
| 6 | Cierre: task file, fila del Backlog, entrada en avance, commit local | ✅ |

## Verificación

Todo con el toolchain del job (`RUSTUP_TOOLCHAIN=1.95.0`) y el comando exacto
del step.

| Comprobación | Comando | Resultado |
|---|---|---|
| enumeración de sitios (superficie linux de CI) | `gh run view 45804272736 --log-failed` | **1** lint: `create_snapshot` → `is_mutable_after_snapshot`, `mod.rs:1588:11` |
| **cara roja** (2 links privados deliberados en `snapshot_restore` y `snapshot_restore_with_backend`) | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --keep-going` | **exit 101**, 3 `^error` = 2 `links to private item` + `could not document vantadb` (86-100 s, warm) |
| **cara verde** (links revertidos + sitio real aplanado) | la misma orden | **exit 0**, `0` líneas `^error`, `0` `links to private`/`unresolved link` |
| el `-p` no es el step | `cargo doc --no-deps -p vantadb` | exit 101 con 2 `unresolved link` en `wal.rs` (feature `encryption` off) → **`-p` no predice el gate** |
| diff doc-only | `git diff -U0 -- src/` filtrado a no-doc | **vacío** (1 línea: `/// [`x`]` → `` /// `x` ``) |
| sin `#[allow]`/`continue-on-error` | `rg 'allow\(rustdoc' src/` | **0** |
| formato | `cargo fmt --all -- --check` | **exit 0** |
| docs | `check-links.mjs` · `check-docs.mjs` | **exit 0** · **exit 0** (orphans informativos: sin `gen-index`, instrucción de la wave) |
| workflow | `actionlint .github/workflows/docs-gates.yml` | **exit 0** (el cambio es un comentario) |

## Review (P2-01)

Tier **adversarial** por glob (`src/storage/**` matchea la tabla, aunque el diff
de código sea 1 línea). **`vanta-review`, contexto fresco, report-only** —
reprodujo por su cuenta: leyó el run con `gh` (no aceptó la cita), verificó el
pin de toolchain en el workflow y en el action, confirmó `#[cfg(unix)]:1611` y
la privacidad del helper, **re-corrió la cara verde** (1m28s forzando
re-documentación: `exit 0`, 0 `^error`; descartó un primer intento de 1,51 s por
cache hit), reprodujo el `-p` con los 2 `unresolved link` de `wal.rs`, y aportó
un **control positivo** que no estaba en el task file: en
`target/doc/vantadb/storage/struct.StorageEngine.html` **no** aparecen ni la doc
unix ni el nombre del helper → la doc del ítem unix nunca llega a la superficie
Windows.

**Ronda 1: `changes-required` — 0 Blocking en el código** ("el arreglo de código
está aprobado tal cual… causa raíz correcta"; resistió `#[allow]`,
`continue-on-error` y `--document-private-items` sobre el gate), con hallazgos
de registro:

| ID | Hallazgo | Disposición |
|---|---|---|
| **B1** | `gen-index --check` exit 1 (+2 documentos): el check required `Generated indexes are current` rojea hasta la landing | **Deferral de la wave, no del cambio**: el encargo prohíbe explícitamente correr `gen-index.mjs` (mismo criterio que `FIND-755`/`FIND-761`/`FIND-763`); los +2 son `FIND-754.md` (lane hermana, untracked) y este task file. Se cierra en el `--write` de fin de wave. |
| **M1** | "otros 7 crates" vs "6 crates restantes" en la misma entrada | **corregido**: 7 (8 miembros en el workspace, 1 falló) |
| **M2** | `status: completed` con §Review vacía | **corregido**: esta sección |
| **M3** | `git diff docs/dev/Backlog.md` tenía **dos** borrados: la sección `FIND-769` y la fila `FIND-754` (otra lane, árbol compartido) | **resuelto solo**: `FIND-754` aterrizó (`c7d125fa9`) antes de este commit, así que el diff de `Backlog.md` ya es **sólo** esta sección (0 inserciones / 6 borrados). La primera tentativa de commit falló por esa misma carrera (`cannot lock ref 'HEAD': is at c7d125fa9 but expected 2f8ea4d76`) — el helper se reintentó sobre el HEAD nuevo. |
| **M4** | La frase de la cara verde sugería "local verde ⇒ sitio arreglado" | **corregido**: la cara verde es la superficie Windows y no ve el sitio unix; eso sólo lo dice el próximo run |
| **L1** | "4 líneas de comentario" (son 5) | **corregido** en task file y avance |
| **L2** | El bloque de comentario quedaba **después** del step y pegado al del artefacto | **corregido**: movido encima del step |
| **L3** | El artefacto (`--document-private-items`) sí renderizaba el link al privado | **corregido** en §Impacto: se distingue superficie pública de tarball, y el aplanado del artefacto se acepta y declara |
| **L4** | Los 3 `^error` de la cara roja son autor-reportados, no re-medidos por el reviewer | **declarado**: la propiedad que importa (rustdoc emite **varios** diagnósticos en una pasada) la verificó el reviewer con su propia corrida `-p`; el verde sí lo re-midió él. |

## Deuda / pendiente

- **`gen-index` sin correr** (B1): `docs/index.md` y `llms.txt` quedan stale
  hasta el `--write` de fin de wave; `FIND-754.md` y este task file figuran como
  ORPHAN informativos mientras tanto.
- La superficie **unix** sólo la mide CI: en Windows no hay cross-doc posible
  (`croaring-sys` pide `x86_64-linux-gnu-gcc`). El próximo run de
  `docs-gates.yml` sobre `develop` es la verificación de cierre de esta fila.
- `FIND-719`: el job `rustdoc` sigue sin ser required en ningún ruleset.
