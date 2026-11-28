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
//   LinuxPackager.js:16      `executableName = sanitizedName.toLowerCase()` — el binario
//                            de Linux pierde las mayusculas del productName.
//
// La tabla SHAPES es la unica fuente de verdad del layout y `resolveArtifact`
// exige EXACTAMENTE una coincidencia completa (`app.asar` + ejecutable): una
// forma desconocida falla con diagnostico en vez de devolver el primer
// directorio que aparezca.
//
// `node artifact-layout.cjs` corre el self-check offline (fixtures de las tres
// formas): es lo que se puede verificar en un host de una sola plataforma.

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

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
    layout: (root, productName) => ({
      resources: path.join(root, "resources"),
      executable: path.join(root, `${productName}.exe`),
    }),
  },
  {
    id: "linux-unpacked",
    platform: "linux",
    roots: (dist) =>
      childDirs(dist).filter((dir) => /^linux(-[a-z0-9]+)?-unpacked$/.test(path.basename(dir))),
    layout: (root, productName) => ({
      resources: path.join(root, "resources"),
      // sanitizedName.toLowerCase(); el productName de este check no lleva
      // caracteres que `sanitizeFileName` reescriba.
      executable: path.join(root, productName.toLowerCase()),
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

function resolveArtifact(distDir, { productName } = {}) {
  assert.ok(productName, "resolveArtifact necesita productName");
  const examined = [];
  const matches = [];
  for (const shape of SHAPES) {
    for (const root of shape.roots(distDir)) {
      const layout = shape.layout(root, productName);
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
  const PRODUCT = "VantadbElectronCheck";
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
      name: "linux x64 (sin sufijo de arch)",
      files: ["linux-unpacked/resources/app.asar", "linux-unpacked/vantadbelectroncheck"],
      expect: {
        shape: "linux-unpacked",
        resources: "linux-unpacked/resources",
        executable: "linux-unpacked/vantadbelectroncheck",
      },
    },
    {
      name: "linux arm64 (con sufijo de arch)",
      files: [
        "linux-arm64-unpacked/resources/app.asar",
        "linux-arm64-unpacked/vantadbelectroncheck",
      ],
      expect: {
        shape: "linux-unpacked",
        resources: "linux-arm64-unpacked/resources",
        executable: "linux-arm64-unpacked/vantadbelectroncheck",
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
        "linux-unpacked/vantadbelectroncheck",
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
    try {
      if (testCase.expect === null) {
        assert.throws(
          () => resolveArtifact(dist, { productName: PRODUCT }),
          /exactamente 1 artefacto/,
          testCase.name,
        );
      } else {
        const found = resolveArtifact(dist, { productName: PRODUCT });
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

module.exports = { SHAPES, resolveArtifact };

if (require.main === module) selfCheck();
