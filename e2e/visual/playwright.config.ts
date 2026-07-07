import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI
    ? [["html", { outputFolder: "playwright-report" }], ["list"]]
    : "html",
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    viewport: { width: 1280, height: 720 },
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
  webServer: {
    command:
      "cargo run --manifest-path ../../vantadb-server/Cargo.toml --release -- --port 4173 --db-dir ./test-data",
    port: 4173,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
    cwd: process.cwd(),
  },
});
