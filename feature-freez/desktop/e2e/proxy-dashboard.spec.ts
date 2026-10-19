// H-07 (DESKTOP-QW10): proxy dashboard — mock upstream
// Contrato: ProxyDashboard con `page.route("**/snapshot", mock)` determinista, sin proceso `vanta-proxy` real.
// Cubre: form CONECTAR cuando no hay proxy, dashboard TurnReports/sesiones/writeback/rate_limit con mock,
// visibilidad condicional del botón/surface PROXY (proxyConfigured gate) y cambiar URL que limpia.
//
// El mock es `SnapshotWire` espejo de `ProxyDashboard.tsx:49-55` (turns/sessions/writeback/rate_limit).
import { test, expect } from "@playwright/test";
import { APP_BASE } from "./helpers";

const MOCK_SNAPSHOT = {
  turns: [
    { timestamp_ms: 1_720_000_000_000, space_id: "space-a", protocol: "openai", model: "gpt-4o", status: 200, duration_ms: 123 },
    { timestamp_ms: 1_720_000_005_000, space_id: "space-b", protocol: "anthropic", model: "claude-3", status: 429, duration_ms: 456 },
  ],
  // ponytail: TTLs far-future (10m/5m) for drift-safe E2E — static Date.now() at file load must survive ~2m server startup + test queue
  sessions: [
    { key: "team-alpha", stage: "team", updated_at_ms: Date.now(), expires_at_ms: Date.now() + 600_000 },
    { key: "agent-beta", stage: "agent", updated_at_ms: Date.now(), expires_at_ms: Date.now() + 300_000 },
    { key: "task-expired", stage: "task", updated_at_ms: Date.now(), expires_at_ms: Date.now() - 1000 },
  ],
  writeback: { pending_labels: ["label-a", "label-b"], pending_count: 2 },
  rate_limit: { limit_per_minute: 60, hits_total: 5, degraded: false },
};

const MOCK_SNAPSHOT_DEGRADED = {
  ...MOCK_SNAPSHOT,
  rate_limit: { limit_per_minute: 10, hits_total: 99, degraded: true },
};

async function clearProxyStorage(page: import("@playwright/test").Page) {
  await page.evaluate(() => {
    localStorage.removeItem("vanta.proxy.url");
    // Forzar que la surface inicial no sea proxy (si el test anterior la dejó en proxy)
    try {
      const ws = JSON.parse(localStorage.getItem("vanta.workspace.v1") ?? "{}");
      if (ws.surface === "proxy") {
        ws.surface = "resumen";
        localStorage.setItem("vanta.workspace.v1", JSON.stringify(ws));
      }
    } catch {}
  });
}

async function setProxySurface(page: import("@playwright/test").Page) {
  await page.evaluate(() => {
    try {
      const ws = JSON.parse(localStorage.getItem("vanta.workspace.v1") ?? "{}");
      ws.surface = "proxy";
      localStorage.setItem("vanta.workspace.v1", JSON.stringify(ws));
    } catch {
      localStorage.setItem("vanta.workspace.v1", JSON.stringify({ surface: "proxy" }));
    }
  });
}

async function setProxyUrl(page: import("@playwright/test").Page, url: string) {
  await page.evaluate((u) => {
    if (u) localStorage.setItem("vanta.proxy.url", u);
    else localStorage.removeItem("vanta.proxy.url");
    window.dispatchEvent(new Event("vanta-proxy-url"));
  }, url);
}

test("sin proxy: botón PROXY oculto en sidebar/palette", async ({ page }) => {
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
  await clearProxyStorage(page);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  // Botón PROXY no existe en sidebar cuando proxyConfigured false
  await expect(page.getByRole("button", { name: /Ir a PROXY/ })).toHaveCount(0);

  // Palette Ctrl+K no lista PROXY cuando no configurado
  await page.keyboard.press("Control+k");
  const palette = page.getByRole("dialog", { name: "Comandos de Vanta Studio" });
  await expect(palette).toBeVisible();
  await expect(palette.getByText("PROXY", { exact: true })).toHaveCount(0);
  await page.keyboard.press("Escape");
});

test("proxy form CONECTAR vía surface proxy (mock sin upstream real)", async ({ page }) => {
  // Mock antes de navegar: el fetch a `${proxyUrl}/snapshot` será interceptado
  await page.route("**/snapshot", async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(MOCK_SNAPSHOT) });
  });

  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await clearProxyStorage(page);
  // Forzar surface proxy para mostrar el form (botón oculto, pero surface sí existe)
  await setProxySurface(page);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  // Form configurador visible (estado `!configured` de ProxyDashboard)
  await expect(page.getByText("PROXY", { exact: false }).first()).toBeVisible();
  await expect(page.getByLabel("URL base del proxy")).toBeVisible();
  await expect(page.getByRole("button", { name: "CONECTAR" })).toBeVisible();
  // Placeholder default
  await expect(page.getByPlaceholder("http://127.0.0.1:8096")).toBeVisible();

  // Rellenar y guardar → el componente dispara PROXY_URL_EVENT y pasa a `configured=true`
  await page.getByLabel("URL base del proxy").fill("http://127.0.0.1:8096");
  await page.getByRole("button", { name: "CONECTAR" }).click();

  // Tras guardar, el dashboard debe mostrar los paneles con datos mockeados (poll cada 5s, pero tick inmediato)
  await expect(page.getByText(/Turn reports/)).toBeVisible({ timeout: 10000 });
  await expect(page.getByText("space-a")).toBeVisible();
  await expect(page.getByText("gpt-4o")).toBeVisible();
  await expect(page.getByText("claude-3")).toBeVisible();

  // Botón PROXY ahora visible en sidebar (proxyConfigured true → sideButton monta)
  await expect(page.getByRole("button", { name: /Ir a PROXY/ })).toBeVisible();
  // localStorage persistido + evento disparado
  const stored = await page.evaluate(() => localStorage.getItem("vanta.proxy.url"));
  expect(stored).toBe("http://127.0.0.1:8096");

  await clearProxyStorage(page);
  await page.unroute("**/snapshot");
});

test("dashboard mock renders sesiones/writeback/rate-limit/TTL + degraded", async ({ page }) => {
  await page.route("**/snapshot", async (route) => {
    const url = route.request().url();
    // Devolver degraded cuando el test lo necesite: distinguimos por un header? Simpler: devolver fixture normal aquí,
    // el degraded se testea en el siguiente sub-test con un route distinto.
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(MOCK_SNAPSHOT) });
  });

  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await setProxyUrl(page, "http://127.0.0.1:8096");
  await setProxySurface(page);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  await expect(page.getByText(/Turn reports/)).toBeVisible({ timeout: 10000 });
  // TurnReports: 2 filas en la tabla (200 y 429, colores distintos)
  await expect(page.getByRole("table")).toBeVisible();
  await expect(page.getByText("space-a")).toBeVisible();
  await expect(page.getByText("space-b")).toBeVisible();
  await expect(page.getByText("200").first()).toBeVisible();
  await expect(page.getByText("429").first()).toBeVisible();

  // Sesiones activas: team→agent→task con glyphs y TTL legible
  await expect(page.getByText(/Sesiones activas/)).toBeVisible();
  await expect(page.getByText("team-alpha")).toBeVisible();
  await expect(page.getByText("agent-beta")).toBeVisible();
  await expect(page.getByText("task-expired")).toBeVisible();
  // TTL expirado para task-expired + TTL minutos para las no expiradas (drift-safe: ttlLabel "1m" si >=60s)
  await expect(page.getByText("expirado")).toBeVisible();
  await expect(page.locator('[title="TTL restante"]').filter({ hasText: /\d+m/ }).first()).toBeVisible();

  // Write-back pendiente
  await expect(page.getByText("Write-back pendiente")).toBeVisible();
  await expect(page.getByText("2").first()).toBeVisible(); // pending_count
  await expect(page.getByText("label-a")).toBeVisible();
  await expect(page.getByText("label-b")).toBeVisible();

  // Rate limit ok (no degraded) — hits_total 5 es único dd exacto en el panel Rate limit
  await expect(page.getByText("Rate limit")).toBeVisible();
  await expect(page.getByText("60/min")).toBeVisible();
  await expect(page.locator('dd').filter({ hasText: /^5$/ }).first()).toBeVisible();
  await expect(page.getByText("ok").first()).toBeVisible();

  await page.unroute("**/snapshot");
  await clearProxyStorage(page);
});

test("proxy degraded (fail-open) muestra estado degraded y 429 hits", async ({ page }) => {
  await page.route("**/snapshot", async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(MOCK_SNAPSHOT_DEGRADED) });
  });
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await setProxyUrl(page, "http://127.0.0.1:8096");
  await setProxySurface(page);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  await expect(page.getByText(/Turn reports/)).toBeVisible({ timeout: 10000 });
  await expect(page.getByText("degraded (fail-open)")).toBeVisible();
  await expect(page.getByText("99")).toBeVisible(); // hits_total degraded

  await page.unroute("**/snapshot");
  await clearProxyStorage(page);
});

test("cambiar URL limpia snapshot y vuelve al form CONECTAR", async ({ page }) => {
  await page.route("**/snapshot", async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(MOCK_SNAPSHOT) });
  });
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await setProxyUrl(page, "http://127.0.0.1:8096");
  await setProxySurface(page);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
  await expect(page.getByText(/Turn reports/)).toBeVisible({ timeout: 10000 });

  // Botón cambiar URL (con Pencil)
  await page.getByRole("button", { name: "cambiar URL" }).click();
  // Vuelve al form configurador
  await expect(page.getByLabel("URL base del proxy")).toBeVisible();
  await expect(page.getByRole("button", { name: "CONECTAR" })).toBeVisible();
  // localStorage limpio
  const cleared = await page.evaluate(() => localStorage.getItem("vanta.proxy.url"));
  expect(cleared).toBeNull();
  // Botón PROXY del sidebar desaparece (proxyConfigured false)
  await expect(page.getByRole("button", { name: /Ir a PROXY/ })).toHaveCount(0);

  await page.unroute("**/snapshot");
});
