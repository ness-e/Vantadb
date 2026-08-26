#!/usr/bin/env node
// TS-07: smoke-test the packed npm tarball end-to-end.
//
// 1. `npm pack` the SDK into a clean temp dir
// 2. if the manifest declares `vantadb-wasm: "file:..."`, rewrite it to the
//    registry version (same as release-npm-61.yml "Rewrite dependency for
//    publish") so the tarball is installable from a clean tree — the repo is
//    never touched
// 3. install the fixed tgz there and run a minimal quickstart
//    (create + put + get + close) against the installed package
// 4. clean up
//
// Wired into `.github/workflows/release-npm-61.yml` (publish-ts) after build,
// before publish. Run locally: `node scripts/smoke-pack.mjs` from vantadb-ts/.
import { execSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const tmp = mkdtempSync(join(tmpdir(), "vantadb-smoke-"));

let failed = false;
try {
  // 1. Pack.
  const out = execSync(`npm pack --pack-destination "${tmp}"`, {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });
  const tgzName = out.trim().split(/\r?\n/).pop();
  if (!tgzName?.endsWith(".tgz")) {
    throw new Error(`unexpected npm pack output: ${JSON.stringify(out)}`);
  }
  console.log(`[smoke] packed: ${tgzName}`);

  // 2. Rewrite file: dependencies on the extracted copy only.
  const pkgDir = join(tmp, "package");
  execSync(`tar -xzf "${join(tmp, tgzName)}" -C "${tmp}"`);
  const manifestPath = join(pkgDir, "package.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  const wasmDep = manifest.dependencies?.["vantadb-wasm"];
  if (typeof wasmDep === "string" && wasmDep.startsWith("file:")) {
    const localPkg = join(root, "..", "vantadb-wasm", "pkg", "package.json");
    if (!existsSync(localPkg)) {
      throw new Error(
        `vantadb-wasm dep is "${wasmDep}" but no local pkg build exists to derive a version from`,
      );
    }
    const wasmVer = JSON.parse(readFileSync(localPkg, "utf8")).version;
    manifest.dependencies["vantadb-wasm"] = `^${wasmVer}`;
    writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
    console.log(`[smoke] rewrote vantadb-wasm dep -> ^${wasmVer}`);
  }

  // --pack-destination overwrites the original tgz in tmp with the fixed one.
  execSync(`npm pack --pack-destination "${tmp}"`, { cwd: pkgDir, stdio: "ignore" });
  const fixedTgz = `${manifest.name.replace("/", "-")}-${manifest.version}.tgz`;

  // 3. Install in the clean temp dir + minimal quickstart.
  const app = mkdtempSync(join(tmpdir(), "vantadb-smoke-app-"));
  try {
    writeFileSync(join(app, "package.json"), JSON.stringify({ private: true }));
    execSync(
      `npm install "${join(tmp, fixedTgz)}" --no-audit --no-fund --loglevel=error`,
      { cwd: app, stdio: "inherit" },
    );
    console.log("[smoke] tarball installed cleanly");

    writeFileSync(
      join(app, "quickstart.mjs"),
      [
        'import { VantaDB } from "vantadb";',
        "",
        "const db = VantaDB.create();",
        'const rec = await db.put({ namespace: "smoke", key: "k", payload: "hello" });',
        'if (rec.payload !== "hello") throw new Error("put returned wrong payload");',
        'const got = await db.get("smoke", "k");',
        'if (!got || got.payload !== "hello") throw new Error("get did not return the record");',
        "db.close();",
        'console.log("SMOKE OK");',
        "",
      ].join("\n"),
    );
    execSync("node quickstart.mjs", { cwd: app, stdio: "inherit" });
  } finally {
    rmSync(app, { recursive: true, force: true });
  }

  console.log(`[smoke] PASSED (${fixedTgz})`);
} catch (e) {
  failed = true;
  console.error(`[smoke] FAILED: ${e.message}`);
} finally {
  rmSync(tmp, { recursive: true, force: true }); // 4. Clean up.
}
process.exitCode = failed ? 1 : 0;
