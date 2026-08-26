// E2E-WEB (WEB-08): guard de regresión del flujo crítico landing → docs → playground.
// Patrón desktop/e2e: asserts por roles/labels visibles de la app (sin implementation details).
import { test, expect } from "@playwright/test";

test("flujo crítico: landing → /docs#quickstart → /playground (run WASM)", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
  });

  // 1. Landing — hero con comando de instalación copiable (WEB-06).
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { level: 1, name: /Vanta/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /pip install vantadb-py/ })).toBeVisible();

  // 2. CTA "5-Minute Quickstart" navega al ancla #quickstart de /docs
  //    (decisión WDA: NO existe ruta /quickstart; el quickstart vive en /docs).
  await page.getByRole("link", { name: /Quickstart/ }).click();
  await expect(page).toHaveURL(/\/docs#quickstart/);
  await expect(page.getByRole("heading", { level: 2, name: /5-Minute Quickstart/ })).toBeVisible();

  // 3. Playground — editor visible + ejecución real contra el engine WASM.
  await page.goto("/playground", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { level: 2, name: "Code Playground", exact: true })).toBeVisible();
  await page.getByRole("button", { name: /Run code/ }).click();
  await expect(page.getByText(/VantaDB WASM engine loaded/)).toBeVisible({ timeout: 15_000 });

  expect(errors).toEqual([]);
});
