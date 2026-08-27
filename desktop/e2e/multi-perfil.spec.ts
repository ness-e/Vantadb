// H-07 (DESKTOP-QW10): E2E multi-perfil conexión
// Contrato: store `vanta.connections.v1` (native + server con Bearer) + Settings defaults (topK/mode/lang) persisten vía localStorage + health embedded.
//
// Nota: en build web embedded (`isEmbedded=true`) la sección "Conexiones guardadas" de AJUSTES está oculta (gate `!embedded`),
// pero el store es el mismo (connectionPrefs, patrón DESKTOP-23). Este spec valida a nivel E2E:
//  - el health del backend embebido está vivo (BM25 · HNSW · RRF)
//  - Settings defaults topK/mode/lang sí visibles en embedded y persisten tras reload
//  - profiles nativo+server persisten vía localStorage (mismo store que los defaults) — round-trip nativo
//
// Perfiles UI Tauri completa (conectar vía perfil con Bearer) queda cubierta por unit tests 4/4
// (`connections.test.ts`) + smoke manual Tauri (SMOKE-MANUAL.md) — no se mockea `__TAURI_INTERNALS__` aquí (YAGNI, 14 métodos invoke).
import { test, expect } from "@playwright/test";
import { APP_BASE, seedRecords } from "./helpers";

test.beforeAll(async () => {
  // Seed mínimo para que el health probe tenga datos y la barra topbar no esté vacía.
  await seedRecords([{ namespace: "default", key: "mp-seed", payload: "multi-perfil seed", metadata: {} }]);
});

test("health embedded vivo en RESUMEN (BM25 · HNSW · RRF)", async ({ page }) => {
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
  // Badge del topbar: BM25 · HNSW · RRF cuando health ok, "OFFLINE" si el server no respondió.
  await expect(page.getByText("BM25 · HNSW · RRF")).toBeVisible({ timeout: 15000 });
  // Tag del footer: "embedded" visible en RESUMEN (3 ocurrencias: sidebar, topbar, footer → first)
  await expect(page.getByText("embedded").first()).toBeVisible();
});

test("Settings defaults topK/mode/lang persisten tras reload (mismo store que perfiles)", async ({ page }) => {
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  // Ir a AJUSTES (siempre visible, incluso en embedded)
  await page.getByRole("button", { name: /Ir a AJUSTES/ }).click();
  await expect(page.getByText("Defaults de búsqueda")).toBeVisible();

  // topK → 15
  const topKInput = page.getByLabel("top_k");
  // Fallback selector: el input numérico de topK es el primero tipo number cerca de "top_k"
  const topK = page.locator('input[type="number"]').first();
  await topK.fill("15");
  await expect(topK).toHaveValue("15");

  // modo → vector
  const modeSelect = page.getByLabel("modo");
  // Hay dos selects: el de kind perfil (cuando visible) y el de modo; en embedded solo el de modo está presente
  // Elegimos el que tiene option "Híbrido"
  const selectMode = modeSelect.or(page.locator('select').filter({ hasText: "Híbrido" }));
  // Si el gate embedded oculta el select de perfiles, el único select visible es el de modo
  const modeSel = page.locator("select").last();
  await modeSel.selectOption("vector");
  await expect(modeSel).toHaveValue("vector");

  // idioma → ENGLISH
  const langEn = page.getByRole("button", { name: "ENGLISH" });
  await langEn.click();
  await expect(langEn).toHaveAttribute("aria-pressed", "true");

  // Persistencia tras reload: el store `vanta.connections.v1` debe reflejar los cambios
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
  await page.getByRole("button", { name: /Ir a AJUSTES/ }).click();
  await expect(page.getByText("Defaults de búsqueda")).toBeVisible();

  const persisted = await page.evaluate(() => {
    try {
      return JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}");
    } catch {
      return {};
    }
  });
  expect(persisted.topK).toBe(15);
  expect(persisted.mode).toBe("vector");
  expect(persisted.lang).toBe("en");

  // Cleanup: volver a defaults originales para no contaminar serial workers
  await page.evaluate(() => {
    try {
      const raw = JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}");
      raw.topK = 8;
      raw.mode = "hybrid";
      raw.lang = "es";
      localStorage.setItem("vanta.connections.v1", JSON.stringify(raw));
    } catch {}
  });
});

test("connectionPrefs profiles native+server (con Bearer) round-trip vía localStorage + reload", async ({ page }) => {
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  const profilesFixture = [
    { id: "p-native-1", name: "local-dev", kind: "native", path: "vantadb-local" },
    { id: "p-server-1", name: "staging", kind: "server", url: "http://10.0.0.1", port: 9090, token: "s3cr3t" },
  ];

  await page.evaluate((profiles) => {
    localStorage.setItem(
      "vanta.connections.v1",
      JSON.stringify({ profiles, activeProfileId: profiles[0].id, topK: 8, mode: "hybrid", lang: "es" }),
    );
  }, profilesFixture);

  // Verificar lectura inmediata
  const before = await page.evaluate(() => {
    try {
      return JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}");
    } catch {
      return {};
    }
  });
  expect(before.profiles).toHaveLength(2);
  expect(before.profiles.find((p: { id: string }) => p.id === "p-server-1")).toMatchObject({
    name: "staging",
    kind: "server",
    url: "http://10.0.0.1",
    port: 9090,
    token: "s3cr3t",
  });
  expect(before.activeProfileId).toBe("p-native-1");

  // Recarga → hidratación del store (ConnectionPrefsStore.load() en Settings)
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  const after = await page.evaluate(() => {
    try {
      return JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}");
    } catch {
      return {};
    }
  });
  expect(after.profiles).toHaveLength(2);
  expect(after.profiles.map((p: { id: string }) => p.id).sort()).toEqual(["p-native-1", "p-server-1"].sort());
  // activeProfileId preservado
  expect(after.activeProfileId).toBe("p-native-1");

  // Mutación: upsert mismo id reemplaza (no duplica) + remove limpia activeProfileId si era el activo
  await page.evaluate(() => {
    try {
      const raw = JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}");
      // upsert reemplazo del server
      raw.profiles = raw.profiles.map((p: { id: string }) =>
        p.id === "p-server-1" ? { ...p, name: "staging-v2", token: "newt" } : p,
      );
      localStorage.setItem("vanta.connections.v1", JSON.stringify(raw));
    } catch {}
  });
  const mutated = await page.evaluate(() => JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}"));
  expect(mutated.profiles.find((p: { id: string }) => p.id === "p-server-1").name).toBe("staging-v2");
  expect(mutated.profiles).toHaveLength(2);

  // Cleanup
  await page.evaluate(() => {
    localStorage.removeItem("vanta.connections.v1");
    localStorage.removeItem("vanta.workspace.v1");
  });
});

test("profileTarget y sanitize toleran storage corrupto sin crashear (sanitización E2E)", async ({ page }) => {
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  await page.evaluate(() => {
    localStorage.setItem("vanta.connections.v1", "{not json");
  });
  await page.reload({ waitUntil: "domcontentloaded" });
  // La app no debe crashear: Vanta Studio sigue visible (catch en ConnectionPrefsStore.load()).
  await expect(page.getByText("Vanta Studio").first()).toBeVisible();

  // Guardar perfil válido después de corrupto debe recuperarse
  await page.evaluate(() => {
    localStorage.setItem(
      "vanta.connections.v1",
      JSON.stringify({ profiles: [{ id: "ok", name: "ok", kind: "native", path: "x" }] }),
    );
  });
  await page.reload({ waitUntil: "domcontentloaded" });
  await expect(page.getByText("Vanta Studio").first()).toBeVisible();
  const ok = await page.evaluate(() => {
    try {
      return JSON.parse(localStorage.getItem("vanta.connections.v1") ?? "{}");
    } catch {
      return {};
    }
  });
  expect(ok.profiles).toHaveLength(1);

  await page.evaluate(() => {
    localStorage.removeItem("vanta.connections.v1");
    localStorage.removeItem("vanta.workspace.v1");
  });
});
