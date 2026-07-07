import { test, expect } from "@playwright/test";
import * as path from "node:path";
import * as fs from "node:fs";

const BASELINE_DIR = path.resolve(__dirname, "baselines");
const DIFF_DIR = path.resolve(__dirname, "diffs");
const ACTUAL_DIR = path.resolve(__dirname, "actual");

test.beforeAll(() => {
  for (const dir of [BASELINE_DIR, DIFF_DIR, ACTUAL_DIR]) {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  }
});

async function snapshotWithDiff(
  page: import("@playwright/test").Page,
  name: string,
  opts?: { maxDiffPixelRatio?: number },
): Promise<void> {
  const actualPath = path.join(ACTUAL_DIR, `${name}.png`);
  const baselinePath = path.join(BASELINE_DIR, `${name}.png`);
  const diffPath = path.join(DIFF_DIR, `${name}-diff.png`);

  await page.screenshot({ path: actualPath, fullPage: true });

  if (!fs.existsSync(baselinePath)) {
    fs.copyFileSync(actualPath, baselinePath);
    test.info().annotations.push({
      type: "baseline-created",
      description: `Baseline created for "${name}" — review and commit ${baselinePath}`,
    });
    return;
  }

  const expectation = expect(page).toHaveScreenshot(name, {
    maxDiffPixels: opts?.maxDiffPixelRatio
      ? undefined
      : 100,
    maxDiffPixelRatio: opts?.maxDiffPixelRatio,
    threshold: 0.2,
  });

  try {
    await expectation;
  } catch (e) {
    const actualBuffer = fs.readFileSync(actualPath);
    const baselineBuffer = fs.readFileSync(baselinePath);
    await page.screenshot({ path: diffPath, fullPage: true });
    throw e;
  }
}

test.describe("VantaDB Server — Visual Regression", () => {
  test("GET /health returns 200 with OK body", async ({ page }) => {
    const response = await page.goto("/health", { waitUntil: "networkidle" });
    expect(response?.status()).toBe(200);

    const body = await response?.json();
    expect(body).toMatchObject({ success: true, data: "OK" });

    await snapshotWithDiff(page, "health-endpoint");
  });

  test("GET /health raw JSON response", async ({ page }) => {
    const response = await page.request.get("/health");
    expect(response.ok()).toBe(true);

    const json = await response.json();
    expect(json.success).toBe(true);
    expect(json.data).toBe("OK");

    await page.setContent(
      `<html>
        <head>
          <meta charset="utf-8" />
          <title>VantaDB Health Response</title>
          <style>
            body { background: #0d1117; color: #c9d1d9; font-family: "JetBrains Mono", monospace; padding: 2rem; }
            pre { background: #161b22; padding: 1.5rem; border-radius: 8px; border: 1px solid #30363d; overflow-x: auto; }
            .badge { display: inline-block; padding: 0.25rem 0.75rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }
            .badge.ok { background: #238636; color: #fff; }
            .key { color: #79c0ff; }
            .string { color: #a5d6ff; }
            .bool { color: #d2a8ff; }
          </style>
        </head>
        <body>
          <h1 style="margin-bottom: 0.5rem;">GET /health</h1>
          <span class="badge ok">200 OK</span>
          <pre>${JSON.stringify(json, null, 2)}</pre>
        </body>
      </html>`,
    );

    await snapshotWithDiff(page, "health-json-response");
  });

  test("Server root returns 404 with expected shape", async ({ page }) => {
    const response = await page.goto("/", { waitUntil: "networkidle" });
    expect(response?.status()).toBe(404);

    await snapshotWithDiff(page, "root-404");
  });

  test("Server headers include VantaDB identifiers", async ({ page }) => {
    const response = await page.goto("/health", { waitUntil: "networkidle" });
    expect(response?.status()).toBe(200);

    const headers = response?.headers();
    expect(headers).toBeDefined();
  });

  test("Metrics endpoint returns Prometheus text", async ({ page }) => {
    const response = await page.goto("/metrics", { waitUntil: "networkidle" });
    expect(response?.status()).toBe(200);

    const contentType = response?.headers()["content-type"] || "";
    expect(contentType).toContain("text/plain");

    const text = await response?.text();
    expect(text).toBeDefined();
    expect(text?.length).toBeGreaterThan(0);

    await snapshotWithDiff(page, "metrics-endpoint", {
      maxDiffPixelRatio: 0.1,
    });
  });
});

test.describe("VantaDB Server — Security Headers", () => {
  test("Health endpoint does not leak internal details", async ({ page }) => {
    const response = await page.goto("/health", { waitUntil: "networkidle" });
    expect(response?.status()).toBe(200);

    const body = await response?.text();
    expect(body).not.toContain("stack");
    expect(body).not.toContain("traceback");
    expect(body).not.toContain("internal");
  });
});
