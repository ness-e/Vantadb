// E2E-VISUAL (UX-19 + DAUD-01): web server helper para Playwright.
// Sirve el web build `embedded` de la consola (dist-web) a través del binario
// real `vanta-cli server` — el mismo patrón que el smoke original
// (scripts/selfcheck-web-e2e.ts) y que el plan: "NO requiere Tauri".
//
// Lifecycle: Playwright (webServer en playwright.config.ts) spawnea este
// proceso y espera el health URL; al terminar los tests lo mata (process
// group). Este script limpia el hijo vanta-cli y el DB temp en señales.
//
// Env:
//   VANTA_CLI_BIN  — ruta explícita al binario (default: target/debug|release)
//   E2E_SKIP_BUILD  — "1" saltea el rebuild de dist-web (iteración rápida)
//   --port N        — puerto HTTP del server (default 8091)
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const ROOT = resolve(import.meta.dirname, ".."); // desktop/
const REPO = resolve(ROOT, "..");
const DIST_WEB = join(ROOT, "dist-web");
const portArg = process.argv.indexOf("--port");
const PORT = Number(portArg >= 0 ? process.argv[portArg + 1] : 8091);

function resolveBinary() {
  if (process.env.VANTA_CLI_BIN) return process.env.VANTA_CLI_BIN;
  for (const cand of ["target/debug/vanta-cli.exe", "target/release/vanta-cli.exe"]) {
    const p = resolve(REPO, cand);
    if (existsSync(p)) return p;
  }
  return "vanta-cli"; // PATH fallback
}

// Rebuild determinístico: el guard no puede correr contra un dist-web stale
// (verificado 2026-08-25: dist-web 19/08 vs src 24/08). E2E_SKIP_BUILD=1 para
// iterar sin rebuild.
if (process.env.E2E_SKIP_BUILD !== "1") {
  console.log("[e2e-serve] build: npx vite build --mode web (dist-web)");
  const r = spawnSync("npx", ["vite", "build", "--mode", "web"], {
    cwd: ROOT,
    stdio: "inherit",
    shell: true, // Windows: npx/vite son .cmd
  });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

if (!existsSync(join(DIST_WEB, "index.html"))) {
  console.error(`[e2e-serve] falta dist-web/index.html tras el build`);
  process.exit(1);
}

const bin = resolveBinary();
const dbDir = mkdtempSync(join(tmpdir(), "vantadb-e2e-"));
console.log(`[e2e-serve] binary: ${bin}`);
console.log(`[e2e-serve] db: ${dbDir}`);
console.log(`[e2e-serve] port: ${PORT}`);

const child = spawn(
  bin,
  ["server", "--http", "-p", String(PORT), "-d", dbDir, "--dashboard-dir", DIST_WEB],
  { windowsHide: true, stdio: ["ignore", "pipe", "pipe"] },
);
child.stdout?.on("data", (d) => process.stdout.write(d));
child.stderr?.on("data", (d) => process.stderr.write(d));

function shutdown() {
  try {
    child.kill();
  } catch {
    /* ya murió */
  }
  try {
    rmSync(dbDir, { recursive: true, force: true });
  } catch {
    /* best effort */
  }
  process.exit(0);
}

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
child.on("exit", (code) => {
  console.error(`[e2e-serve] vanta-cli exited code ${code}`);
  process.exit(code ?? 0);
});