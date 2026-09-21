// E2E-VISUAL (UX-19): guard permanente del flujo crítico desktop que antes
// dependía de QA manual (smoke E2E con datos reales, evidencia smoke-0*.png).
//
// Flujo: ingest por UI → grid → Inspector por TECLADO (Enter) → borrado 2
// pasos → papelera → RESTORE → paleta Ctrl+K → AJUSTES.
// Seed vía REST (idempotente por key) + asserts de estados UI (roles/labels
// existentes de la app — sin implementation details).
import { test, expect } from "@playwright/test";
import { APP_BASE, seedRecords } from "./helpers";

const K_VEC = "k-vector";
const K_EDIT = "k-edit";
const K_DEL = "k-del";

test.beforeAll(async () => {
  await seedRecords([
    {
      namespace: "default",
      key: K_VEC,
      payload: "mascota gato feliz jugando",
      metadata: { tipo: { String: "memoria" } },
    },
    {
      namespace: "default",
      key: K_EDIT,
      payload: "registro que se abre con Enter",
      metadata: {},
    },
    {
      namespace: "default",
      key: K_DEL,
      payload: "registro que se borra del grid",
      metadata: { origen: { String: "e2e" } },
    },
  ]);
});

test("flujo crítico: ingest → grid → Inspector por Enter → borrado → papelera → RESTORE → paleta → AJUSTES", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
  });

  // 1. Carga del web build embedded + HOME con backend vivo.
  await page.goto(APP_BASE, { waitUntil: "domcontentloaded" });
  await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
  await expect(page.getByText("BM25 · HNSW · RRF")).toBeVisible();

  // 2. MEMORIAS: grid muestra los registros sembrados.
  await page.getByRole("button", { name: /Ir a MEMORIAS/ }).click();
  const grid = page.getByRole("region", { name: "Memorias" });
  await expect(grid).toBeVisible();
  await expect(grid.getByText("3 cargados")).toBeVisible();
  await expect(grid.getByRole("row").filter({ hasText: K_VEC })).toBeVisible();

  // 3. Ingest por UI (labels visibles) → notice + grid refresca (UX-17).
  //    FIND-23: el mapping HTTP de vanta_ingest rechaza namespace omitido
  //    (vanta-http-map.ts:93 manda "") → el test explicita "default" (workaround
  //    de test; el bug va como FIND-23 en Backlog, no se toca la app).
  await page.getByLabel("ID (opcional)").fill("e2e-ui");
  await page.getByLabel("Contenido de texto").fill("registro ingerido por la ui e2e");
  await page.getByPlaceholder("por omisión 'default'").fill("default");
  await page.getByRole("button", { name: "Agregar registro" }).click();
  await expect(page.getByRole("alert")).toContainText("Guardados 1 registro(s)");
  await expect(grid.getByText("4 cargados")).toBeVisible();
  await expect(grid.getByRole("row").filter({ hasText: "e2e-ui" })).toBeVisible();

  // 4. Inspector por TECLADO: foco en fila → Enter abre el Inspector (UX-02).
  const rowEdit = grid.getByRole("row").filter({ hasText: K_EDIT });
  await rowEdit.focus();
  await page.keyboard.press("Enter");
  const inspector = page.getByRole("complementary", { name: "Inspector de registro" });
  await expect(inspector).toBeVisible();
  await page.getByRole("button", { name: "Cerrar inspector" }).click();

  // 5. Borrado 2 pasos: papelera → BORRAR → fila fuera del grid.
  const rowDel = grid.getByRole("row").filter({ hasText: K_DEL });
  await rowDel.getByRole("button", { name: `Mover ${K_DEL} a papelera` }).click();
  await rowDel.getByRole("button", { name: "BORRAR" }).click();
  await expect(rowDel).toHaveCount(0);

  // 6. PAPELERA: tombstone visible → RESTORE.
  await page.getByRole("button", { name: /Ir a PAPELERA/ }).click();
  const trash = page.getByRole("region", { name: "Papelera" });
  await expect(trash.getByText("1 tombstone")).toBeVisible();
  await trash.getByRole("button", { name: /RESTORE/ }).click();
  await expect(page.getByRole("alert")).toContainText("restaurado k-del");

  // 7. MEMORIAS de nuevo: el registro restaurado vuelve al grid.
  await page.getByRole("button", { name: /Ir a MEMORIAS/ }).click();
  await expect(
    page.getByRole("region", { name: "Memorias" }).getByRole("row").filter({ hasText: K_DEL }),
  ).toBeVisible();

  // 8. Paleta Ctrl+K → AJUSTES (Settings con input "Nombre del perfil").
  await page.keyboard.press("Control+k");
  const palette = page.getByRole("dialog", { name: "Comandos de Vanta Studio" });
  await expect(palette).toBeVisible();
  await palette.getByPlaceholder("Escribí un comando o buscá una key…").fill("ajustes");
  await palette.getByText("AJUSTES", { exact: true }).click();
  // Settings embebido (WEB-05) oculta perfiles → assert en sección que sí
  // renderiza: "Defaults de búsqueda" (h2 del Section).
  await expect(page.getByText("Defaults de búsqueda")).toBeVisible();

  // 9. Sin errores fatales (degradados esperados: vanta_*/unsupported; y el 404
  //    del pre-check get() de IngestForm sobre key nueva — probe deliberado).
  const fatal = errors.filter(
    (e) =>
      !e.includes("vanta_") && !e.includes("unsupported") && !e.includes("404 (Not Found)"),
  );
  expect(fatal).toEqual([]);
});