// WEB-09 — screenshots before/after para gate visual owner (recreado 2026-09-02)
// Viewport 1440×900, fullPage, chromium --no-sandbox, webServer npm run dev
import { test, expect } from "@playwright/test";

test.use({ viewport: { width: 1440, height: 900 } });

test("web09 after: home fullPage + mark detalle (refinamiento sutil+A11y)", async ({ page }) => {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  // asegurar hero+trust-bar visibles
  await expect(page.getByRole("heading", { level: 1, name: /Vanta/i })).toBeVisible();
  // fullPage screenshot hero+trust-bar+capabilities (mismo framing que before)
  await page.screenshot({ path: "web09-after-home.png", fullPage: true });
  // detalle mark: localizar el mark classic y screenshot cercano
  const mark = page.locator('[aria-label*="interactive mark"]').first();
  if (await mark.count() > 0) {
    await mark.screenshot({ path: "web09-after-mark.png" });
  } else {
    // fallback: recorte centrado donde está el mark (hero right)
    await page.screenshot({ path: "web09-after-mark.png", clip: { x: 720, y: 120, width: 600, height: 600 } });
  }
});
