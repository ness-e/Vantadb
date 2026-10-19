// E2E-VISUAL (UX-19 + DAUD-01) — config del guard E2E del flujo crítico desktop.
//
// Corre contra el web build `embedded` servido por el binario real vanta-cli
// (e2e/serve.mjs) — sin Tauri, como el smoke original. El webServer rebuilda
// dist-web y arranca el server con un DB temp fresco por run (determinista).
//
// Alternativa dev: E2E_BASE_URL=http://localhost:1420/ npx playwright test
// (vite dev proxya /api → vanta-cli server en :8090, ver vite.config.ts).
//
// Source: playwright.dev/docs/test-webserver (command/url/timeout/
// reuseExistingServer verificado contra docs oficiales).
import { defineConfig } from "@playwright/test";

const PORT = Number(process.env.E2E_PORT ?? 8091);
const HOST = `http://127.0.0.1:${PORT}`;

export default defineConfig({
  testDir: "e2e",
  testMatch: /\.spec\.ts$/,
  timeout: 60_000,
  // DB temp compartida + papelera es por-sesión de página → serial para
  // determinismo (3 tests corren en <1 min).
  workers: 1,
  fullyParallel: false,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: process.env.E2E_BASE_URL ?? `${HOST}/dashboard/`,
    trace: "retain-on-failure",
  },
  webServer: {
    command: `node e2e/serve.mjs --port ${PORT}`,
    url: `${HOST}/api/v2/health`,
    timeout: 120_000, // incluye el rebuild de dist-web
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    stderr: "pipe",
  },
});