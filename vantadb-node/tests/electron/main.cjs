"use strict";

// DIST-21 — el .node nativo, cargado DENTRO de un proceso Electron real.
//
// Lo que prueba y que ningun test actual cubre:
//   1. el binding que carga Electron es el `.node` que produce `npm run build`
//      (mismo fichero, misma ruta), no el runner de Node ni un mock;
//   2. el proceso donde corre es el **principal** de Electron (`process.type
//      === 'browser'`), no un Node suelto;
//   3. `VantaDb.connect()` sobre **disco**, escribe, lee, cierra y **reabre**:
//      si el motor no persistio, la segunda lectura no encuentra nada.
//
// Si el `.node` falta o no carga, este script sale 1: el verde no puede ser
// vacuo (verificado renombrando el fichero, ver DIST-21.md §Steps).
//
// Se ejecuta igual suelto (`electron tests/electron/main.cjs`) que empaquetado
// dentro de `app.asar` (lo copia `asar-check.mjs` a la raiz de la app): la raiz
// se busca subiendo hasta encontrar `index.cjs`, y en asar `fs`/`require`
// siguen resolviendo (Electron redirige a `app.asar.unpacked/` si el binario se
// descomprimio).

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const NAMESPACE = "electron-main";
const KEY = "dist-21";
const PAYLOAD = "hola desde el proceso principal de electron";

function fail(message, error) {
  console.error(`FAIL DIST-21: ${message}`);
  if (error) console.error(error && error.stack ? error.stack : error);
  process.exit(1);
}

function findPackageRoot(start) {
  let dir = start;
  for (let i = 0; i < 4; i += 1) {
    if (fs.existsSync(path.join(dir, "index.cjs"))) return dir;
    dir = path.dirname(dir);
  }
  fail(`no encuentro index.cjs subiendo desde ${start}`);
}

async function main() {
  // 1. Corremos dentro de Electron, en el proceso principal.
  assert.ok(process.versions.electron, `process.versions.electron ausente: no es Electron`);
  assert.equal(
    process.type,
    "browser",
    `process.type es ${process.type}: no es el proceso principal`,
  );

  // 2. El binding es el .node real del paquete.
  const pkgRoot = findPackageRoot(__dirname);
  const bindings = fs.readdirSync(pkgRoot).filter((name) => name.endsWith(".node"));
  assert.ok(bindings.length > 0, `no hay ningun .node en ${pkgRoot}: corre npm run build`);
  const binding = path.join(pkgRoot, bindings[0]);
  const bindingStat = fs.statSync(binding);
  assert.ok(
    bindingStat.size > 1_000_000,
    `${binding} pesa ${bindingStat.size} B: no parece el binding real`,
  );

  const { VantaDb } = require(path.join(pkgRoot, "index.cjs"));
  assert.equal(typeof VantaDb.connect, "function", "VantaDb.connect no es una funcion");

  // 3. Disco, no memoria: write -> read -> close -> reopen -> read.
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vanta-electron-"));
  const db = await VantaDb.connect(dir);
  await db.put({
    namespace: NAMESPACE,
    key: KEY,
    payload: PAYLOAD,
    metadata: { source: "electron-main" },
    vector: [1, 0, 0, 0],
  });
  const written = await db.get(NAMESPACE, KEY);
  assert.ok(written, `get() devolvio null tras put() en ${dir}`);
  assert.equal(written.payload, PAYLOAD, "el payload leido no coincide con el escrito");
  await db.close();

  const onDisk = fs.readdirSync(dir);
  assert.ok(onDisk.length > 0, `connect() no escribio nada en ${dir}`);

  const reopened = await VantaDb.connect(dir);
  const readBack = await reopened.get(NAMESPACE, KEY);
  assert.ok(readBack, `la reapertura de ${dir} no encontro el registro: no hubo persistencia real`);
  assert.equal(readBack.payload, PAYLOAD, "el payload tras reabrir no coincide");
  assert.equal(readBack.namespace, NAMESPACE);
  assert.equal(readBack.key, KEY);
  await reopened.close();

  console.log(
    JSON.stringify({
      ok: true,
      electron: process.versions.electron,
      node: process.versions.node,
      napi: process.versions.napi,
      moduleVersion: process.versions.modules,
      binding: path.basename(binding),
      bindingBytes: bindingStat.size,
      dbDir: dir,
      filesOnDisk: onDisk.length,
    }),
  );
  process.exit(0);
}

main().catch((error) => fail("excepcion no controlada", error));
