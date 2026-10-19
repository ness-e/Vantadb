// E2E-VISUAL (DAUD-01): verificación visual runtime del FIX-D1.
//
// FIX-D1 (App.css/index.css): body padding 24px→0 + todos los colores vía
// tokens theme-flipping (--background, --border, --ink-shadow). Antes: body
// quedaba CREMA alrededor del shell en dark y la sombra dura #000 era
// invisible sobre fondo negro.
//
// Verificación programática (computed styles) en ambos temas + screenshots de
// evidencia en e2e/screenshots/. Targets:
//   - sin marco crema en dark  → body bg = #0a0a0a + padding 0px + sidebar bg
//   - inputs desnudos voltean  → regla @layer base `input { background:
//     var(--background) }` (App.css:32-47) en checkbox del grid + radios TTL
//     del Inspector + textarea de ImportPaste
//   - sombras duras visibles    → botones usan var(--ink-shadow): #000 light /
//     #FBF9F5 dark
import { test, expect } from "@playwright/test";
import {
  APP_BASE,
  CREAM,
  ensureScreenshotsDir,
  SCREENSHOTS_DIR,
  seedRecords,
  VANTA_BLACK,
} from "./helpers";
import { join } from "node:path";

test.beforeAll(async () => {
  // Un registro para abrir el Inspector (radios TTL desnudos).
  await seedRecords([
    { namespace: "default", key: "k-vector", payload: "mascota gato feliz jugando", metadata: {} },
  ]);
});

/** Computed styles del shell que FIX-D1 tocó (body, sidebar, input, botón). */
function shellStyles(page: import("@playwright/test").Page) {
  return page.evaluate(() => {
    const bg = (el: Element | null) => (el ? getComputedStyle(el).backgroundColor : null);
    const shadow = (el: Element | null) => (el ? getComputedStyle(el).boxShadow : null);
    return {
      bodyBg: bg(document.body),
      bodyPadding: getComputedStyle(document.body).padding,
      sidebarBg: bg(document.querySelector('aside[aria-label="Panel lateral"]')),
      inputBg: bg(document.querySelector('input[aria-label="Búsqueda global"]')),
      themeBtnShadow: shadow(
        document.querySelector('button[aria-label="Cambiar tema claro/oscuro"]'),
      ),
    };
  });
}

test("light: fondo crema, sin marco, inputs voltean a crema, sombra dura negra", async ({
  page,
}) => {
  ensureScreenshotsDir();
  // Tema forzado light ANTES del mount (main.tsx lee localStorage).
  await page.addInitScript(() => localStorage.setItem("vanta-theme", "light"));
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  const s = await shellStyles(page);
  expect(s.bodyBg).toBe(CREAM);
  expect(s.sidebarBg).toBe(CREAM);
  expect(s.inputBg).toBe(CREAM);
  expect(s.bodyPadding).toBe("0px"); // FIX-D1: padding 24px→0
  expect(s.themeBtnShadow).toContain("0, 0, 0"); // --ink-shadow negro en light

  // Input "desnudo" del grid (checkbox select-all sin clase bg): Tailwind
  // preflight (base layer) pisa el fallback @layer base de App.css → computa
  // transparent; el fondo del tema se ve detrás (voltea con --background).
  // El guard anti-regresión real: NUNCA crema en light no aplica; en light
  // transparent es correcto (body crema detrás).
  await page.getByRole("button", { name: /Ir a MEMORIAS/ }).click();
  const grid = page.getByRole("region", { name: "Memorias" });
  const bareCheckbox = grid.getByRole("checkbox", {
    name: "Seleccionar todas las filas cargadas",
  });
  await expect(bareCheckbox).toBeVisible();
  const cbBg = await bareCheckbox.evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(cbBg === "rgba(0, 0, 0, 0)" || cbBg === CREAM).toBe(true);

  await page.screenshot({ path: join(SCREENSHOTS_DIR, "daud01-light.png"), fullPage: true });
});

test("dark: sin marco crema, inputs desnudos voltean, sombra dura crema", async ({ page }) => {
  ensureScreenshotsDir();
  await page.addInitScript(() => localStorage.setItem("vanta-theme", "dark"));
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  const s = await shellStyles(page);
  expect(s.bodyBg).toBe(VANTA_BLACK); // ← el marco crema ya no existe en dark
  expect(s.sidebarBg).toBe(VANTA_BLACK);
  expect(s.inputBg).toBe(VANTA_BLACK);
  expect(s.bodyPadding).toBe("0px");
  expect(s.themeBtnShadow).toContain("251, 249, 245"); // sombra crema visible en dark

  // Input "desnudo" del grid: NUNCA crema en dark (transparent → el fondo
  // #0a0a0a del tema se ve detrás; o el propio token si el fallback gana).
  await page.getByRole("button", { name: /Ir a MEMORIAS/ }).click();
  const grid = page.getByRole("region", { name: "Memorias" });
  const bareCheckbox = grid.getByRole("checkbox", {
    name: "Seleccionar todas las filas cargadas",
  });
  await expect(bareCheckbox).toBeVisible();
  const cbBg = await bareCheckbox.evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(cbBg === "rgba(0, 0, 0, 0)" || cbBg === VANTA_BLACK).toBe(true);

  // ImportPaste: textarea voltea a negro en dark; cerrar el modal (✕ CERRAR).
  await page.getByRole("button", { name: /IMPORT CSV\/JSON/ }).click();
  const pasteTextarea = page.locator("textarea").last();
  await expect(pasteTextarea).toBeVisible();
  expect(await pasteTextarea.evaluate((el) => getComputedStyle(el).backgroundColor)).toBe(
    VANTA_BLACK,
  );
  const cerrarBtn = page.getByRole("button", { name: "Cerrar", exact: true });
  await cerrarBtn.click();
  await expect(cerrarBtn).toHaveCount(0);

  // Inspector: aside bg-card voltea a negro y los radios desnudos del TTL
  // NO pintan crema en dark (preflight transparent → panel bg-background detrás).
  await grid.getByRole("row").filter({ hasText: "k-vector" }).locator("code").first().click();
  const inspector = page.getByRole("complementary", { name: "Inspector de registro" });
  await expect(inspector).toBeVisible();
  expect(await inspector.evaluate((el) => getComputedStyle(el).backgroundColor)).toBe(
    VANTA_BLACK,
  );
  await inspector.getByRole("button", { name: "editar" }).click();
  const radio = inspector.locator('input[type="radio"]').first();
  await expect(radio).toBeVisible();
  const radioBg = await radio.evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(radioBg === "rgba(0, 0, 0, 0)" || radioBg === VANTA_BLACK).toBe(true);

  await page.screenshot({ path: join(SCREENSHOTS_DIR, "daud01-dark.png"), fullPage: true });
});