// DESKTOP-45 slice 2: lente ESPACIO — seed con vectores + asserts UI.
//
// Contrato: empty-state inicial, botón `⤒ proyectar`, canvas role="img"
// con aria "Scatterplot de N embeddings", lista sr-only "Puntos
// proyectados" con los seeds, sin `role="alert"` fatal.
// Solo records CON `vector` se proyectan (useProjection.ts:79) → los 12
// seeds llevan vector dim 8 determinista; el resto de namespaces no
// aporta puntos (UMAP nNeighbors se adapta: projection.worker.ts:49).
import { test, expect } from "@playwright/test";
import { APP_BASE, seedRecords } from "./helpers";

const NS = "ns-espacio";
const N = 12;
const DIM = 8;

/** Vectores 8-dim deterministas y distintos (evita matriz degenerada UMAP). */
function vec(i: number): number[] {
  return Array.from({ length: DIM }, (_, d) => ((i * 7 + d * 13) % 100) / 50 - 1);
}

test.beforeAll(async () => {
  await seedRecords(
    Array.from({ length: N }, (_, i) => ({
      namespace: NS,
      key: `s-punto-${i + 1}`,
      payload: `punto espacial numero ${i + 1}`,
      metadata: {},
      vector: vec(i),
    })),
  );
});

test("lente ESPACIO: auto-proyección 12 vectores → canvas + lista accesible + notice", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
  });

  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });

  // Surface ESPACIO (WorkspaceShell: sidebar "ESPACIO", title "Ir a ESPACIO — proyección 2D").
  await page.getByRole("button", { name: /Ir a ESPACIO/ }).click();
  await expect(page.getByRole("heading", { name: "ESPACIO" })).toBeVisible();

  // Controles: selector de namespace + botón proyectar (SpaceLens.tsx:230-250).
  await expect(page.getByLabel("Namespace a proyectar")).toBeVisible();
  const projectBtn = page.getByRole("button", { name: /⤒ proyectar/ });
  await expect(projectBtn).toBeVisible();

  // Auto-proyección al montar (SpaceLens.tsx:153-156): sin click previo el
  // worker UMAP-js ya corre → el canvas aparece solo. (El empty-state
  // "sin proyección…" solo existe en fase idle sin datos.)
  const canvas = page.getByRole("img", { name: /Scatterplot de 12 embeddings/ });
  await expect(canvas).toBeAttached({ timeout: 30_000 });

  // Re-proyectar es idempotente: click manual → mismo canvas, mismos 12 puntos.
  await projectBtn.click();
  await expect(
    page.getByRole("img", { name: /Scatterplot de 12 embeddings/ }),
  ).toBeAttached({ timeout: 30_000 });

  // Alternativa accesible (UX-08, sr-only): los seeds en la lista.
  const list = page.getByRole("list", { name: "Puntos proyectados" });
  await expect(list).toContainText(`${NS}/s-punto-1`);

  // Notice de éxito (SpaceLens.tsx:165 → región global role="alert"):
  // prueba que la proyección completó, no solo que el canvas existe.
  await expect(page.getByRole("alert")).toContainText("Proyección lista: 12 puntos");

  // Sin errores fatales (mismo filtro que flujo-critico).
  const fatal = errors.filter(
    (e) =>
      !e.includes("vanta_") && !e.includes("unsupported") && !e.includes("404 (Not Found)"),
  );
  expect(fatal).toEqual([]);
});
