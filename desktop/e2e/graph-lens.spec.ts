// DESKTOP-45 slice 1: lente GRAFO (surface IQL) — seed REST + asserts UI.
//
// Contrato: header GRAFO + meta namespace/nodos, toolbar (iql/fit/reset/
// labels), canvas role="img" con aria "Grafo de N nodos", lista sr-only
// "Nodos visibles del grafo" con los seeds, consola IQL visible + toggle.
// Seed 7 records en `ns-grafo` (supera los ≤5 de `default` en runs
// completos → el seed del grafo elige ns-grafo; ver useGraphData.ts:3-7).
import { test, expect } from "@playwright/test";
import { APP_BASE, seedRecords } from "./helpers";

const NS = "ns-grafo";
const KEYS = Array.from({ length: 7 }, (_, i) => `g-nodo-${i + 1}`);

test.beforeAll(async () => {
  await seedRecords(
    KEYS.map((key, i) => ({
      namespace: NS,
      key,
      payload: `nodo grafo numero ${i + 1} conectado al hub`,
      metadata: {},
    })),
  );
});

test("lente GRAFO: header + toolbar + canvas + lista accesible + consola IQL", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
  });

  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  // Surface IQL (WorkspaceShell: sidebar "IQL", title "Ir a IQL — consola de queries sobre grafo").
  await page.getByRole("button", { name: /Ir a IQL/ }).click();
  await expect(page.getByRole("heading", { name: "GRAFO" })).toBeVisible();
  // Meta del seed: "ns-grafo · 7/500 nodos · 0 aristas" (hubs sin aristas).
  await expect(page.getByText(/ns-grafo · 7\/500 nodos/)).toBeVisible();

  // Toolbar: consola toggle + fit + reset + labels (titles de GraphLens.tsx).
  await expect(
    page.getByRole("button", { name: "⌨ iql" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: /⛶ fit/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /↺ reset/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /labels/ })).toBeVisible();

  // Canvas 3D: role="img" con aria "Grafo de 7 nodos y 0 aristas…".
  const canvas = page.getByRole("img", { name: /Grafo de 7 nodos/ });
  await expect(canvas).toBeAttached();

  // Alternativa accesible (UX-08, sr-only): los 7 seeds en la lista.
  const list = page.getByRole("list", { name: "Nodos visibles del grafo" });
  await expect(list).toContainText("nodo grafo numero 1 conectado al hub");

  // Consola IQL embebida (GRAFO-03) visible; el toggle la oculta y la restaura.
  const runBtn = page.getByTitle("Ejecutar (Ctrl+Enter)");
  await expect(runBtn).toBeVisible();
  await page.getByRole("button", { name: "⌨ iql" }).click();
  await expect(runBtn).toHaveCount(0);
  await expect(page.getByText("ctrl+enter en la consola = ejecutar iql")).toHaveCount(0);
  await page.getByRole("button", { name: "⌨ iql" }).click();
  await expect(runBtn).toBeVisible();

  // Sin errores fatales (mismo filtro que flujo-critico: degradados vanta_*/unsupported + 404 del probe).
  const fatal = errors.filter(
    (e) =>
      !e.includes("vanta_") && !e.includes("unsupported") && !e.includes("404 (Not Found)"),
  );
  expect(fatal).toEqual([]);
});
