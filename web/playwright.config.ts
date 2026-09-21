// E2E-WEB (WEB-08) — config del guard E2E del flujo crítico web.
// Patrón desktop/e2e: webServer arranca el dev server, asserts por
// roles/labels visibles (sin implementation details).
//
// Comando local:
//   cd web && npx playwright test
// (requiere: npm i -D @playwright/test@1.62.1 — browsers ya instalados
//  en %LOCALAPPDATA%\ms-playwright, versión alineada con desktop).
import { defineConfig } from "@playwright/test";

// Usamos `localhost` (no 127.0.0.1): Next.js 16 bloquea por defecto los
// recursos dev cross-origin (allowedDevOrigins); con localhost el Host del
// browser coincide con el dev server → sin 403 en _next/static/chunks.
const HOST = "http://localhost:3000";

export default defineConfig({
  testDir: "e2e",
  testMatch: /\.spec\.ts$/,
  timeout: 60_000,
  workers: 1,
  fullyParallel: false,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: HOST,
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run dev",
    url: HOST,
    timeout: 120_000,
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    stderr: "pipe",
  },
});
