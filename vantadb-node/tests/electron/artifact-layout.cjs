"use strict";

// DIST-21 — donde deja el artefacto `electron-builder --dir`, en las 3 plataformas.
//
// `dir: true` no devuelve rutas (un directorio no es un artefacto), asi que hay
// que reconocer la forma que quedo en `dist/`. Las tres formas NO son homologas:
//
//   Windows  dist/win-unpacked/resources/app.asar
//   Linux    dist/linux-unpacked/resources/app.asar
//   macOS    dist/mac-arm64/<Producto>.app/Contents/Resources/app.asar
//
// En macOS el `.app` cuelga UN NIVEL MAS ABAJO que el directorio de plataforma:
// `dist/mac-arm64` no termina en `.app`, asi que un unico `readdir(dist)` con
// `/(-unpacked|\.app)$/` nunca lo ve. Ese fue el rojo de macOS (el `.node` carga
// bien en Electron arm64; lo que fallaba era encontrar el artefacto).
//
// Fuente de las tres formas (node_modules/app-builder-lib/out/):
//   platformPackager.js:96   `<outDir>/<platform><arch?>[-unpacked]` (macOS sin `-unpacked`)
//   platformPackager.js:321  ejecutable del artefacto `<appOutDir>/<executableName>`
//   platformPackager.js:487  recursos de macOS `<appOutDir>/<productFilename>.app/Contents/Resources`
//
// El NOMBRE del ejecutable NO es el mismo en las tres, y no es un detalle de
// estilo: son dos reglas distintas de electron-builder.
//
//   Windows  appInfo.js:57   `productFilename` = `sanitizeFileName(productName)`
//   macOS    appInfo.js:57   idem (el bundle y `Contents/MacOS/<Producto>` comparten nombre)
//   Linux    linuxPackager.js:16
//                            `executableName = this.appInfo.sanitizedName.toLowerCase()`
//            appInfo.js:123  `sanitizedName` = `sanitizeFileName(this.name)`
//            appInfo.js:115  `name` = `info.metadata.name` = el `name` del package.json
//
// O sea: en Linux el binario sale del **`name` del paquete**, no del
// `productName`, y ademas pierde las mayusculas. Con
// `name: "vantadb-electron-asar-check"` / `productName: "VantadbElectronCheck"`
// el binario real es `vantadb-electron-asar-check`, no `vantadbelectroncheck`.
// (Salvo `build.linux.executableName` / `config.executableName`, que esta app de
// prueba no define; si alguna vez lo define, se resuelve con esa clave.)
//
// La tabla SHAPES es la unica fuente de verdad del layout y `resolveArtifact`
// exige EXACTAMENTE una coincidencia completa (`app.asar` + ejecutable): una
// forma desconocida falla con diagnostico en vez de devolver el primer
// directorio que aparezca.
//
// `node artifact-layout.cjs` corre el self-check offline (fixtures de las tres
// formas): es lo que se puede verificar en un host de una sola plataforma. El
// fixture usa `name != productName`, para que una regla equivocada NO pueda
// pasar el test por coincidencia (era el defecto anterior: el fixture escribia
// el nombre que la regla vieja esperaba y se autoafirmaba).

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

// `sanitizeFileName` de electron-builder (builder-util/out/filename.js) es un
// wrapper de `sanitize-filename` sin opciones (replacement `""`). Se replica en
// vez de requerir el paquete: `builder-util`/`sanitize-filename` son
// dependencias TRANSITIVAS de electron-builder, no declaradas en
// `vantadb-node/package.json`, y este self-check corre offline sin necesitarlas.
// Regla copiada de `sanitize-filename@1.6.5` (index.js) y su wrapper
// `builder-util/out/filename.js:sanitizeFileName`.
// ponytail: sin el `truncate-utf8-bytes` a 255 B (un nombre npm no pasa de 214
// chars); si algun dia se sanitiza un productName largo, importar la lib.
const ILLEGAL_RE = /[/?<>\\:*|"]/g;
const CONTROL_RE = /[\x00-\x1f\x80-\x9f]/g;
const RESERVED_RE = /^\.+$/;
const WINDOWS_RESERVED_RE =
  /^(con|prn|aux|nul|com[0-9\u00b9\u00b2\u00b3]|lpt[0-9\u00b9\u00b2\u00b3])(\..*)?$/i;
const TRAILING_DOTS_AND_SPACES_RE = /[. ]+$/;

function sanitizeFileName(input) {
  return input
    .replace(ILLEGAL_RE, "")
    .replace(CONTROL_RE, "")
    .replace(RESERVED_RE, "")
    .replace(WINDOWS_RESERVED_RE, "")
    .replace(TRAILING_DOTS_AND_SPACES_RE, "")
    .replace(WINDOWS_RESERVED_RE, "");
}

// El binario de Linux: `sanitizedName.toLowerCase()` (linuxPackager.js:16).
function linuxExecutableName(packageName) {
  return sanitizeFileName(packageName).toLowerCase();
}

function childDirs(dir) {
  try {
    return fs
      .readdirSync(dir, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => path.join(dir, entry.name));
  } catch {
    // `dist` todavia no existe: se devuelve vacio y el error lo da el caller.
    return [];
  }
}

const SHAPES = [
  {
    id: "windows-unpacked",
    platform: "win32",
    roots: (dist) =>
      childDirs(dist).filter((dir) => /^win(-[a-z0-9]+)?-unpacked$/.test(path.basename(dir))),
    // Windows: `productFilename` = `sanitizeFileName(productName)` (appInfo.js:57).
    layout: (root, { productName }) => ({
      resources: path.join(root, "resources"),
      executable: path.join(root, `${productName}.exe`),
    }),
  },
  {
    id: "linux-unpacked",
    platform: "linux",
    roots: (dist) =>
      childDirs(dist).filter((dir) => /^linux(-[a-z0-9]+)?-unpacked$/.test(path.basename(dir))),
    // Linux NO usa el productName: `sanitizedName` = `sanitizeFileName(name)`
    // (appInfo.js:123) y el binario es eso en minusculas (linuxPackager.js:16).
    layout: (root, { packageName }) => ({
      resources: path.join(root, "resources"),
      executable: path.join(root, linuxExecutableName(packageName)),
    }),
  },
  {
    id: "mac-app",
    platform: "darwin",
    roots: (dist) =>
      childDirs(dist)
        .filter((dir) => /^mac(-[a-z0-9]+)?$/.test(path.basename(dir)))
        .flatMap((dir) => childDirs(dir))
        .filter((dir) => dir.endsWith(".app")),
    layout: (root) => ({
      resources: path.join(root, "Contents", "Resources"),
      // El bundle y su ejecutable comparten nombre: `<Producto>.app/Contents/MacOS/<Producto>`.
      executable: path.join(root, "Contents", "MacOS", path.basename(root, ".app")),
    }),
  },
];

function resolveArtifact(distDir, { productName, packageName } = {}) {
  assert.ok(productName, "resolveArtifact necesita productName (ejecutable de Windows/macOS)");
  assert.ok(packageName, "resolveArtifact necesita packageName (ejecutable de Linux)");
  const names = { productName, packageName };
  const examined = [];
  const matches = [];
  for (const shape of SHAPES) {
    for (const root of shape.roots(distDir)) {
      const layout = shape.layout(root, names);
      const hasAsar = fs.existsSync(path.join(layout.resources, "app.asar"));
      const hasExecutable = fs.existsSync(layout.executable);
      examined.push(
        `${shape.id}:${path.relative(distDir, root)} asar=${hasAsar} ejecutable=${hasExecutable}`,
      );
      if (hasAsar && hasExecutable) matches.push({ shape: shape.id, root, ...layout });
    }
  }
  if (matches.length !== 1) {
    throw new Error(
      `esperaba exactamente 1 artefacto de "electron-builder --dir" en ${distDir} y hay ` +
        `${matches.length} [formas: ${SHAPES.map((shape) => shape.id).join(", ")}] ` +
        `[visto: ${examined.length > 0 ? examined.join(" | ") : "dist vacio o inexistente"}]`,
    );
  }
  return matches[0];
}

function selfCheck() {
  // `name != productName` a proposito: en Linux el ejecutable sale del `name`
  // (`sanitizeFileName(name).toLowerCase()`), en Windows/macOS del
  // `productName`. Con nombres iguales el fixture se autoafirmaba — escribia el
  // binario que la regla vieja esperaba y el test pasaba sin medir nada.
  const PRODUCT = "VantadbElectronCheck";
  const PACKAGE = "vantadb-electron-asar-check";
  // Lo que devolvia la regla vieja (productName.toLowerCase()) — NO existe en Linux.
  const WRONG_LINUX = PRODUCT.toLowerCase();
  const app = `${PRODUCT}.app`;
  const toPosix = (relative) => relative.split(path.sep).join("/");
  const macFiles = (platformDir) => [
    `${platformDir}/${app}/Contents/Resources/app.asar`,
    `${platformDir}/${app}/Contents/Resources/app.asar.unpacked/vantadb_native.darwin-arm64.node`,
    `${platformDir}/${app}/Contents/MacOS/${PRODUCT}`,
  ];

  const cases = [
    {
      name: "windows x64",
      files: [
        "win-unpacked/resources/app.asar",
        "win-unpacked/resources/app.asar.unpacked/vantadb_native.win32-x64-msvc.node",
        `win-unpacked/${PRODUCT}.exe`,
      ],
      expect: {
        shape: "windows-unpacked",
        resources: "win-unpacked/resources",
        executable: `win-unpacked/${PRODUCT}.exe`,
      },
    },
    {
      name: "linux x64 (sin sufijo de arch): el binario sale del NAME del paquete",
      files: ["linux-unpacked/resources/app.asar", `linux-unpacked/${PACKAGE}`],
      expect: {
        shape: "linux-unpacked",
        resources: "linux-unpacked/resources",
        executable: `linux-unpacked/${PACKAGE}`,
      },
    },
    {
      name: "linux arm64 (con sufijo de arch)",
      files: ["linux-arm64-unpacked/resources/app.asar", `linux-arm64-unpacked/${PACKAGE}`],
      expect: {
        shape: "linux-unpacked",
        resources: "linux-arm64-unpacked/resources",
        executable: `linux-arm64-unpacked/${PACKAGE}`,
      },
    },
    {
      // Fija la formula completa: sanitize PRIMERO (dropea `:`), lowercase DESPUES.
      // `sanitize-filename` no toca espacios; `toLowerCase` no toca `:`.
      name: "linux: sanitiza el NAME antes de bajar a minusculas",
      packageName: "Vanta:DB Check",
      files: ["linux-unpacked/resources/app.asar", "linux-unpacked/vantadb check"],
      expect: {
        shape: "linux-unpacked",
        resources: "linux-unpacked/resources",
        executable: "linux-unpacked/vantadb check",
      },
    },
    {
      name: "macOS arm64 (el .app un nivel mas abajo)",
      files: macFiles("mac-arm64"),
      expect: {
        shape: "mac-app",
        resources: `mac-arm64/${app}/Contents/Resources`,
        executable: `mac-arm64/${app}/Contents/MacOS/${PRODUCT}`,
      },
    },
    {
      name: "macOS x64 (sin sufijo de arch)",
      files: macFiles("mac"),
      expect: {
        shape: "mac-app",
        resources: `mac/${app}/Contents/Resources`,
        executable: `mac/${app}/Contents/MacOS/${PRODUCT}`,
      },
    },
    {
      // El rojo de Linux: un binario derivado del productName NO es el artefacto.
      // Con la regla vieja este caso PASABA (devolvia match), y en CI el real
      // fallaba. Este es el caso que impide que la regla vuelva atras.
      name: "invalido: linux con el binario derivado del productName",
      files: ["linux-unpacked/resources/app.asar", `linux-unpacked/${WRONG_LINUX}`],
      expect: null,
    },
    {
      // El rojo real de macOS: el artefacto no es dist/mac-arm64, es el .app de dentro.
      name: "invalido: mac-arm64 sin el .app",
      files: ["mac-arm64/resources/app.asar", `mac-arm64/${PRODUCT}`],
      expect: null,
    },
    {
      name: "invalido: .app sin app.asar",
      files: [`mac-arm64/${app}/Contents/MacOS/${PRODUCT}`],
      expect: null,
    },
    {
      name: "invalido: directorio con forma desconocida",
      files: ["foo-unpacked/resources/app.asar", "foo-unpacked/x"],
      expect: null,
    },
    {
      name: "ambiguo: dos artefactos completos",
      files: [
        "win-unpacked/resources/app.asar",
        `win-unpacked/${PRODUCT}.exe`,
        "linux-unpacked/resources/app.asar",
        `linux-unpacked/${PACKAGE}`,
      ],
      expect: null,
    },
  ];

  for (const testCase of cases) {
    const dist = fs.mkdtempSync(path.join(os.tmpdir(), "vanta-artifact-"));
    for (const relative of testCase.files) {
      const full = path.join(dist, relative);
      fs.mkdirSync(path.dirname(full), { recursive: true });
      fs.writeFileSync(full, "");
    }
    const names = { productName: PRODUCT, packageName: testCase.packageName || PACKAGE };
    try {
      if (testCase.expect === null) {
        assert.throws(() => resolveArtifact(dist, names), /exactamente 1 artefacto/, testCase.name);
      } else {
        const found = resolveArtifact(dist, names);
        assert.deepEqual(
          {
            shape: found.shape,
            resources: toPosix(path.relative(dist, found.resources)),
            executable: toPosix(path.relative(dist, found.executable)),
          },
          testCase.expect,
          testCase.name,
        );
      }
    } finally {
      fs.rmSync(dist, { recursive: true, force: true });
    }
  }

  console.log(JSON.stringify({ ok: true, cases: cases.length }));
}

module.exports = { SHAPES, resolveArtifact, sanitizeFileName, linuxExecutableName };

if (require.main === module) selfCheck();
