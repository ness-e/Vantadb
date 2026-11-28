---
title: "DIST-21 — Carga nativa del .node en un proceso Electron real + asarUnpack, en 3 OSes"
kind: task
status: completed
description: "El .node de vantadb-node nunca se ha cargado dentro de un Electron real y nadie verificaba que asarUnpack lo saque del asar: dos fallos (ABI drift y ruta virtual dlopen) invisibles en CI y visibles en el cliente. Se entrega test de proceso Electron real sobre disco + verificacion del empaquetado + workflow de 3 OSes."
id: DIST-21
tags: [vantadb, electron, node, bindings, ci, asar]
---

# DIST-21 — Electron real carga el `.node`, y el asar lo deja fuera

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `DIST-21`, 🟠 Alta / 🔴 P0)
- **Origen:** Ego Readiness Plan 2026-10-07 · **Ver:** `DIST-19` (workflow `electron-node.yml`, ya no existe)
- **Esfuerzo:** 🟢 1-2 d
- **Estado:** ✅ COMPLETED (local, sin push — Regla 7)

## Premisa (re-medida al abrir — no re-derivada)

La fila afirma 0 % implementado. Medido en HEAD el 2026-11-28:

```
Get-ChildItem .github/workflows -Filter *.yml   -> 20 workflows
rg -il "electron" .github/workflows             -> 0 archivos
rg -il "asar"     vantadb-node/tests            -> 0
```

**Confirmada.** El `.node` existe (`vantadb_native.win32-x64-msvc.node`, 8,4 MB, 27-nov 23:42) pero
ninguna prueba lo carga dentro de Electron y ninguna comprueba el `asarUnpack`.

## Blast Radius

- **Paths tocados:** `vantadb-node/package.json`, `vantadb-node/package-lock.json`,
  `vantadb-node/tests/electron/**` (nuevo), `.github/workflows/ci-electron.yml` (nuevo),
  `docs/dev/workflow/README.md` + `TRIGGERS.md` (índice de workflows), `docs/dev/Backlog.md` (fila).
- **Sin tocar:** `src/**`, `vanta-memory/**`, `vantadb-{python,wasm,ts}/src/**`, `vantadb-mcp/**`,
  `dev-tools/**`, `.opencode/**`, `docs/index.md`, `Cargo.toml`, cualquier otro workflow.
- **Carril ajeno (SKIP declarado):** `src/storage/**`, `src/attestation.rs`, `tests/certified_delete.rs`
  los tiene tomados otro carril Rust; no se tocan ni se interpretan sus fallos de gate.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-node/package.json`, `vantadb-node/vitest.config.ts`,
  `vantadb-node/index.d.ts` (§API: `VantaDb.connect`/`put`/`get`/`close`, `MemoryInput`, `MemoryRecord`),
  `docs/dev/workflow/README.md`, `docs/dev/workflow/TRIGGERS.md`, `.github/workflows/ci-fleet.yml`
  (cabecera + `on:` + `env`/`permissions`/`concurrency`), `.github/workflows/ci-rust.yml` (cabecera),
  `.github/workflows/release-npm-node.yml` (build matrix + steps de `napi build`).
- **Leídos parcial (generado):** `vantadb-node/package-lock.json` — lo reescribe `npm install`;
  solo se lee la cabecera (`lockfileVersion`) para confirmar que el bump es mecánico.
- **Referencias hacia dentro:** `vantadb-node/index.cjs` carga el `.node` con `require('./vantadb_native.<platform>.node')`;
  `tests/*.test.ts` (12 ficheros) importan `../index.js` — el runner es **Node**, no Electron.
- **Referencias entrantes:** `rg -l 'ci-fleet' docs/` → `docs/dev/workflow/README.md`,
  `docs/dev/workflow/TRIGGERS.md`, `docs/dev/operations/CI_POLICY.md`, `docs/index.md`
  (**`docs/index.md` está fuera de zona** → se anota, no se toca).
  `release-npm-node.yml` publica `vantadb-node` pero no valida carga en Electron.
- **Veredicto impacto:** **bajo en código** (no hay lógica de producto: un test nuevo, devDeps y un
  workflow). **Alto en consecuencia** si falta: un `.node` con ABI incompatible o dentro del asar
  rompe a todo consumidor Electron y ningún gate actual lo ve.

## Contrato

1. Un test corre **dentro de un proceso Electron real** (no el runner de Node), hace
   `VantaDb.connect()` sobre **disco**, escribe, lee, cierra, **reapertura** y vuelve a leer.
   (`NativeVantaDB` no existe en este binding: la clase exportada es `VantaDb` — corrección de
   nombre de la fila, verificada en `index.cjs:tail` y `index.d.ts:453`.)
2. Carga el `.node` **que produce `npm run build`** en `vantadb-node/` (el artefacto real, no un mock).
3. El test **falla** si el `.node` no está o no carga — demostrado con el fichero renombrado.
4. `electron` como **devDependency con versión exacta** (sin `^`) + script `npm run test:electron`.
   Nunca en `dependencies`.
5. `asarUnpack: ["**/*.node"]` **verificado**: el `.node` aparece en `app.asar.unpacked/` y **no**
   dentro de `app.asar`; el app empaquetado se ejecuta y lee de disco.
6. `.github/workflows/ci-electron.yml`, matriz de 3 OSes, estilo copiado de `ci-fleet.yml` /
   `ci-rust.yml`, `permissions: contents: read`, **sin** `pull_request_target`, timeout explícito.
   Añadido al índice (`docs/dev/workflow/README.md` + `TRIGGERS.md`).

## Spec (SDD — decisiones técnicas)

| Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|
| Versión de Electron | **`45.5.0` exacta** (`latest` de `45-x-y`, medido con `npm view electron dist-tags`) | `^45.5.0` (rango) | La fila exige pin exacto: el punto del test es congelar el ABI contra el que se valida. Un rango re-valida un ABI distinto en cada `npm ci`. |
| Proceso Electron | `electron tests/electron/main.cjs` (CJS) | ESM + `--experimental` | `index.cjs` ya exporta `VantaDb`; CJS evita el flag y funciona en las 3 versiones de Node que trae Electron. |
| Base de datos | `VantaDb.connect(mkdtemp)` + reapertura | memoria | La fila exige **disco**: el fallo típico (`dlopen`) aparece antes, pero la reapertura prueba que el motor escribió de verdad. |
| Aserción de que no es vacuo | renombrar el `.node` y ver rojo | confiar en el verde | Un test que pasa porque nunca conectó vale menos que ninguno. |
| Empaquetado | `electron-builder --dir` con `build.asarUnpack` en la app de fixture | `@electron/asar` a mano | `--dir` produce el **mismo** `app.asar`/`app.asar.unpacked` que el instalador y honra la clave real `asarUnpack`; `@electron/asar` solo probaría el glob de la herramienta, no nuestra config. Si `--dir` no cabe, se degrada a `@electron/asar` y se declara PARCIAL. |
| Linux sin display | `xvfb-run -a` en el workflow | `--headless` / `--ozone-platform=headless` | **Medido**: `xvfb` está en la imagen oficial `ubuntu-latest` (`xvfb 2:21.1.12-1ubuntu1.9` en `Ubuntu2404-Readme.md` de `actions/runner-images`, image version 20261122.23.1) → `xvfb-run` funciona sin instalar nada; el step además instala defensivamente si el runner migra a 26.04 (anunciado para nov-2026). `--headless`/`--ozone-platform=headless` **no** se usan: no hay host Linux aquí y la doc de Electron no los garantiza para una app. |

## Invariantes de dominio (handoff — MUST)

- El `.node` publicado sigue siendo el de `napi build --release`; el test **no** recompila el crate
  (otro carril tiene el lock de `target/`).
- `electron` **jamás** en `dependencies` (el paquete publicado no debe arrastrar 250 MB).
- Ninguna modificación a workflows ajenos, ni a `src/**`.
- El workflow no lleva `pull_request_target` ni `continue-on-error` (Regla 2).

## Steps

1. ✅ **Electron pinneado + herramientas.** `electron@45.5.0` exacto, `@electron/asar@4.3.4`,
   `electron-builder@26.18.0` (devDeps). Disco: 11,26 GB → 10,51 GB tras instalar (~750 MB: el
   postinstall del zip + `dist/`), por encima del umbral de 6 GB declarado en el encargo.
2. ✅ **`tests/electron/main.cjs`** — proceso principal de Electron, `VantaDb.connect()` sobre
   `mkdtemp`, put + get + close + **reapertura** + get. (`NativeVantaDB` no existe: la clase
   exportada por `index.cjs` es `VantaDb`; se usa la real.)
3. ✅ **Windows verde + rojo demostrado dos veces.** Verde: `{"ok":true,"electron":"45.5.0",…,`
   `"filesOnDisk":8}`, exit 0. Rojo A (renombrado): `FAIL DIST-21: no hay ningun .node…`, exit 1.
   Rojo B (corrupto, 2 MB de ceros que sí pasa el pre-check): falla en `require()`, `main.cjs:59`,
   exit 1 — el rojo es del `dlopen`, no de un `existsSync`. Restaurado (8.393.728 B) → verde.
4. ✅ **`tests/electron/asar-check.cjs`** — `electron-builder --dir` sobre una app temporal con
   `build.asarUnpack: ["**/*.node"]`; aserciones: `.node` en `app.asar.unpacked/`, entrada del asar
   **sin `offset`** y `unpacked: true` con tamaño coincidente, y **el ejecutable empaquetado corre y
   lee de disco**. → `asarUnpack` **verificado**, no parcial.
5. ✅ **`ci-electron.yml`** (3 OSes, `contents: read`, sin `pull_request_target`, `timeout 45`,
   `paths` acotado) + filas en `docs/dev/workflow/README.md` (20 → 21) y `TRIGGERS.md`.
6. ✅ **Verify + commit local**: `npm test` 12/82 (sin regresión), prettier en `.cjs`/`.json`,
   `actionlint` exit 0, `check-docs.mjs`/`check-links.mjs` exit 0.

## Resultado

- **Electron:** `45.5.0` exacto. Corre Node 24.22.3 / NAPI 10 / MODULE_VERSION 150.
- **3 OSes:** Windows x64 **ejecutado aquí** (verde + dos rojos). macOS y Linux: diseño en el
  workflow (misma secuencia), **no ejecutados desde este host** → el primer run de CI es su
  verificación. Linux sin display: `xvfb-run -a`, y el `xvfb` **medido** en la imagen oficial
  `ubuntu-latest` (`xvfb 2:21.1.12-1ubuntu1.9` en `actions/runner-images`
  `Ubuntu2404-Readme.md`, image version 20261122.23.1), con instalación defensiva si el label
  migra a 26.04 (anunciado para nov-2026). `--headless`/`--ozone-platform=headless` quedan **sin
  medir** y por eso no se usan.
- **`asarUnpack`:** verificado de punta a punta (empaquetado + ejecución del binario empaquetado).
- **Deuda:** (1) la ruta Linux/macOS se confirma en CI; (2) `docs/index.md` sin regenerar (fuera de
  zona, lo regenera `gen-index.mjs` desde el `title` del README); (3) el commit arrastra 2 líneas de
  `FIND-826`/`FIND-903` del carril Rust en el `Backlog.md` compartido (clase FIND-792, no revertidas).

## Deuda técnica (Regla 6)

- Linux se ejecuta con `xvfb-run`; el primer run de CI es la verificación real (no medible aquí).
- `electron-builder` añade ~40 MB de devDeps; si `--dir` no resulta estable, se degrada a parcial
  con dueño declarado.

## Definition of Done

- [ ] `npm run test:electron` verde en Windows, con salida pegada.
- [ ] Rojo demostrado con el `.node` renombrado, y restaurado.
- [ ] `asarUnpack` verificado (o PARCIAL declarado con su dueño).
- [ ] `ci-electron.yml` válido (`actionlint`), 3 OSes, `permissions` mínimos.
- [ ] Índice de workflows actualizado; fila del Backlog actualizada.
