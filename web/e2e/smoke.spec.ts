import { test, expect } from "@playwright/test";

test("homepage loads", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/VantaDB/);
});

test("navigation links are present", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("nav.nb-nav")).toBeVisible();
});
