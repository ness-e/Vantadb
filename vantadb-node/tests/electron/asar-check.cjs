"use strict";

// DIST-21 — `asarUnpack: ["**/*.node"]` verificado sobre un empaquetado REAL.
//
// Electron no puede hacer `dlopen` sobre una ruta virtual (`app.asar`): si el
// `.node` queda dentro del archivo, el paquete publicado arranca y muere. Este
// script empaqueta el binding con `electron-builder --dir` (el MISMO
// `app.asar`/`app.asar.unpacked` que produce el instalador, honrando la clave
// `build.asarUnpack` de la app) y comprueba las dos mitades:
//
//   1. el `.node` aparece en `app.asar.unpacked/**` (existe fuera del archivo);
//   2. el `.node` NO aparece en el listado de `app.asar` (no esta dentro);
//   3. el ejecutable empaquetado arranca y lee de disco (exit 0). Es su propio
//      control negativo por construccion: si el binario estuviera dentro del
//      asar, ese step fallaria al cargar.
//
// La app de prueba se genera en un temporal copiando los artefactos de
// `npm run build` (`index.cjs`, `index.js`, `*.node`): no hay copias en git que
// puedan quedar stale.
//
// Donde queda el artefacto lo resuelve `artifact-layout.cjs`: en Windows y Linux
// es el directorio de plataforma (`win-unpacked`/`linux-unpacked`), en macOS es
// el `.app` que cuelga un nivel mas abajo (`dist/mac-arm64/<Producto>.app`). El
// asar y su `app.asar.unpacked/` cuelgan de `resources/` en las tres, asi que
// las aserciones de abajo son identicas en macOS (nada se relaja).

const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const { resolveArtifact } = require("./artifact-layout.cjs");

const pkgRoot = path.resolve(__dirname, "../..");
const pkgJson = JSON.parse(fs.readFileSync(path.join(pkgRoot, "package.json"), "utf8"));
const electronVersion = pkgJson.devDependencies.electron;
const PRODUCT_NAME = "VantadbElectronCheck";

function stageApp() {
  const nodeFiles = fs.readdirSync(pkgRoot).filter((name) => name.endsWith(".node"));
  assert.ok(nodeFiles.length > 0, `no hay .node en ${pkgRoot}: corre npm run build`);

  const appDir = fs.mkdtempSync(path.join(os.tmpdir(), "vanta-asar-app-"));
  for (const name of ["index.cjs", "index.js", "index.d.ts", ...nodeFiles]) {
    fs.copyFileSync(path.join(pkgRoot, name), path.join(appDir, name));
  }
  fs.copyFileSync(path.join(__dirname, "main.cjs"), path.join(appDir, "main.cjs"));

  fs.writeFileSync(
    path.join(appDir, "package.json"),
    JSON.stringify(
      {
        name: "vantadb-electron-asar-check",
        version: "0.0.0",
        private: true,
        main: "main.cjs",
        devDependencies: { electron: electronVersion },
        build: {
          asar: true,
          asarUnpack: ["**/*.node"],
        },
      },
      null,
      2,
    ),
  );
  return { appDir, nodeFiles };
}

function collectNodes(dir) {
  const found = [];
  const walk = (current) => {
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith(".node")) found.push(path.relative(dir, full));
    }
  };
  walk(dir);
  return found;
}

async function main() {
  const { appDir, nodeFiles } = stageApp();
  const { build } = require("electron-builder");

  await build({
    projectDir: appDir,
    dir: true,
    config: {
      appId: "com.vantadb.electron.check",
      productName: PRODUCT_NAME,
      directories: { output: "dist" },
      electronVersion,
      // Sin esto electron-builder baja un Electron propio (~100 MB): usamos el
      // del paquete, asi se empaqueta exactamente el ABI que instala `npm ci`.
      electronDist: path.join(pkgRoot, "node_modules", "electron", "dist"),
      npmRebuild: false,
      files: ["**/*"],
      mac: { identity: null },
    },
  });

  // `dir: true` no devuelve rutas de artefacto (el directorio no lo es): la
  // forma se reconoce con la tabla de `artifact-layout.cjs`, que cubre las tres
  // plataformas y exige una unica coincidencia completa.
  const distDir = path.join(appDir, "dist");
  const { shape, resources, executable } = resolveArtifact(distDir, { productName: PRODUCT_NAME });

  const asarPath = path.join(resources, "app.asar");
  const unpackedRoot = path.join(resources, "app.asar.unpacked");
  assert.ok(fs.existsSync(asarPath), `no existe ${asarPath}`);
  assert.ok(fs.existsSync(unpackedRoot), `no existe ${unpackedRoot}: asarUnpack no actuo`);

  // 1. El .node vive en app.asar.unpacked/.
  const unpackedNodes = collectNodes(unpackedRoot);
  assert.ok(
    unpackedNodes.length > 0,
    `app.asar.unpacked no contiene ningun .node (esperaba ${nodeFiles.join(", ")})`,
  );

  // 2. El .node NO tiene contenido DENTRO de app.asar. En un asar, `unpacked:
  //    true` + ausencia de `offset` significa que el archivo solo tiene una
  //    entrada de indice en el archivo: sus bytes viven en app.asar.unpacked/.
  //    (Un `.node` con `offset` es un `.node` que Electron no podra dlopear.)
  const asar = require("@electron/asar");
  const header = asar.getRawHeader(asarPath).header;
  const entries = [];
  const collect = (node, prefix) => {
    for (const [name, meta] of Object.entries(node.files || {})) {
      const full = prefix ? `${prefix}/${name}` : name;
      if (meta.files) collect(meta, full);
      else entries.push({ name: full, meta });
    }
  };
  collect(header, "");
  const packedNodes = entries.filter(
    (entry) => entry.name.endsWith(".node") && entry.meta.offset !== undefined,
  );
  assert.equal(
    packedNodes.length,
    0,
    `el .node sigue DENTRO de app.asar (con offset): ${packedNodes
      .map((entry) => entry.name)
      .join(", ")} — asarUnpack no lo saco`,
  );
  const indexedNodes = entries.filter((entry) => entry.name.endsWith(".node"));
  assert.ok(
    indexedNodes.length > 0,
    `app.asar no tiene ninguna entrada .node: asarUnpack la excluyo del indice`,
  );
  for (const entry of indexedNodes) {
    assert.equal(
      entry.meta.unpacked,
      true,
      `${entry.name} esta en el indice pero no marcado unpacked`,
    );
    const onDisk = path.join(unpackedRoot, entry.name);
    assert.ok(
      fs.existsSync(onDisk),
      `la entrada ${entry.name} dice unpacked pero no existe en ${onDisk}`,
    );
    assert.equal(
      fs.statSync(onDisk).size,
      entry.meta.size,
      `${entry.name} en disco no mide lo que declara el indice`,
    );
  }

  // 3. El ejecutable empaquetado arranca y lee de disco.
  assert.ok(fs.existsSync(executable), `no existe el ejecutable ${executable}`);
  const output = execFileSync(executable, [], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 180000,
  });
  const line = output.split(/\r?\n/).find((row) => row.startsWith('{"ok":true'));
  assert.ok(line, `el ejecutable empaquetado no reporto ok (salida: ${output.slice(-400)})`);

  // 3.5. El temporal lleva una copia completa de Electron (~250 MB): se borra
  // solo en verde. En rojo se conserva para poder inspeccionarlo.
  fs.rmSync(appDir, { recursive: true, force: true });

  console.log(
    JSON.stringify({
      ok: true,
      artifactShape: shape,
      asar: path.basename(asarPath),
      unpackedNodes,
      packedNodeEntries: packedNodes.length,
      executable: path.basename(executable),
      packedRun: JSON.parse(line),
    }),
  );
}

main().catch((error) => {
  console.error(`FAIL DIST-21 asar: ${error && error.message ? error.message : error}`);
  console.error(error && error.stack);
  process.exit(1);
});
