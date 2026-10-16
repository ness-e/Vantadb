// selfcheck-wasm-e2e.ts (WASM-03): E2E smoke de la consola standalone 100%
// browser — `vite build --mode wasm` produce `dist-wasm/` estático que corre
// contra WASM + OPFS sin NINGÚN server. Este script sirve dist-wasm con un
// server estático node:http (127.0.0.1 = secure context → OPFS disponible),
// navega con Playwright (Edge real, como WASM-01), ingesta un record por la
// UI, verifica que aparece en el grid, hace RELOAD real y verifica que el
// record persiste (OPFS vía connect_persistent + save()).
//
// Corre con node 24 (type-stripping nativo, sin tsx) desde desktop/:
//
//   node scripts/selfcheck-wasm-e2e.ts
//
// Requiere:
//   - `npm run build:wasm` ya corrido (desktop/dist-wasm/)
//   - `vantadb-wasm/pkg` regenerado (wasm-pack build vantadb-wasm — git-ignored)
//   - devDep `playwright` con Edge instalado (channel msedge; el chromium
//     bundled de Playwright no expone navigator.storage.getDirectory — WASM-01)
//
// Exit 0 solo si todo pasa. La OPFS del origin 127.0.0.1 acumula datos entre
// runs: cada run usa un id de record único (`e2e-<timestamp>`).
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { existsSync } from "node:fs";
import { extname, join, normalize, resolve } from "node:path";
import { chromium } from "playwright";

const SCRIPT_DIR = import.meta.dirname;
const DIST_WASM = resolve(SCRIPT_DIR, "..", "dist-wasm");
const MIME: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
};

let failures = 0;
function check(cond: boolean, msg: string): void {
  if (cond) console.log(`  ok: ${msg}`);
  else {
    failures += 1;
    console.error(`  FAIL: ${msg}`);
  }
}

async function main(): Promise<void> {
  if (!existsSync(DIST_WASM)) {
    console.error(`dist-wasm no existe: ${DIST_WASM} — corré antes: npm run build:wasm`);
    process.exit(1);
  }

  const key = `e2e-${Date.now()}`;
  const payload = `payload standalone ${key}`;

  // Servidor estático (SIN server VantaDB — solo sirve los assets del build).
  const server = createServer(async (req, res) => {
    const url = new URL(req.url ?? "/", "http://localhost");
    console.log(`[e2e] req ${req.method} ${url.pathname}`);
    try {
      let p = normalize(join(DIST_WASM, decodeURIComponent(url.pathname)));
      if (!p.startsWith(DIST_WASM)) throw new Error("path escape");
      const isDir = (await stat(p).then((s) => s.isDirectory()).catch(() => false)) || url.pathname.endsWith("/");
      if (isDir) p = join(p, "index.html");
      const data = await readFile(p);
      res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
      res.end(data);
    } catch {
      res.writeHead(404);
      res.end("not found");
    }
  });
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  const port = (server.address() as { port: number }).port;
  const base = `http://127.0.0.1:${port}`;
  console.log(`[e2e] sirviendo dist-wasm en ${base}/ …`);

  const browser = await chromium.launch({ channel: "msedge", headless: true });
  const page = await browser.newPage();
  page.setDefaultTimeout(20_000);
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
  });
  // WASM-04: los modales de import confirman con window.confirm — Playwright
  // los auto-descarta (devuelve false) por defecto; aceptar para no cancelar.
  page.on("dialog", (d) => d.accept());

  try {
    // --- Fase 1: boot + HOME + health (instantiation WASM real) ------------
    await page.goto(base + "/", { waitUntil: "domcontentloaded" });
    await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
    check(true, "boot: brand 'Vanta Studio' visible (app montada)");
    // health() pasa por el WasmBackend → el glue wasm + OPFS instancian OK.
    const healthBadge = page.getByText("BM25 · HNSW · RRF");
    await healthBadge.waitFor({ state: "visible" });
    check(true, "health: badge 'BM25 · HNSW · RRF' (vanta_health vía WASM)");

    // --- Fase 2: CRUD por la UI (MEMORIAS → IngestForm) ---------------------
    await page.getByRole("button", { name: /MEMORIAS/ }).click();
    await page.getByRole("button", { name: "Add record" }).waitFor({ state: "visible" });
    await page.getByLabel("Record id").fill(key);
    await page.getByLabel("Text content").fill(payload);
    // Namespace vacío (default del form) — el grid browse lista `db.list("")`,
    // y el WASM filtra por namespace exacto: un record en "e2e" jamás aparece
    // en el grid (WASM-03 verify: bug del E2E original del sub-agente).
    await page.getByRole("button", { name: "Add record" }).click();
    await page.getByRole("alert").getByText(/Stored 1 record/).waitFor({ state: "visible" });
    check(true, "ingest: 'Stored 1 record(s).' via UI (put + persist OPFS)");

    // El grid browse monta en mount y no se refresca tras un ingest individual
    // (solo ImportPaste remonta vía gridKey++) — re-navegar a MEMORIAS fuerza
    // el remount y el fetchFirst() que trae la fila (mismo comportamiento nativo).
    await page.getByRole("button", { name: /RESUMEN/ }).click();
    await page.getByRole("button", { name: /MEMORIAS/ }).click();

    const grid = page.getByRole("region", { name: "Memorias" });
    await grid.waitFor({ state: "visible" });
    const row = grid.getByRole("row").filter({ hasText: key });
    await row.waitFor({ state: "visible" });
    check(true, "grid MEMORIAS: fila e2e-<ts> visible");

    // --- Fase 2.5: import por ARCHIVO (WASM-04) — drop de un .jsonl real ---
    // setInputFiles con payload en memoria (sin tocar disco): el archivo se lee
    // con File API (`file.text()`), se parsea con el parser de OP-01 y el
    // reporte + gridKey++ remontan el grid con los records importados.
    await page.getByRole("button", { name: /IMPORT ARCHIVO/ }).click();
    await page
      .getByRole("dialog", { name: /Importar archivo/ })
      .waitFor({ state: "visible" });
    // El modal por defecto usa el nombre de la conexión implícita ("embedded")
    // como namespace; fijar "default" para que el grid (list("")) lo vea.
    // Scope al dialog: la surface MEMORIAS tiene su propio input Namespace
    // (IngestForm) — getByLabel sin scope sería ambiguo (strict mode).
    const dropDialog = page.getByRole("dialog", { name: /Importar archivo/ });
    await dropDialog.getByLabel("namespace").fill("default");
    const dropKey = `e2e-drop-${Date.now()}`;
    await page.setInputFiles('input[type="file"]', {
      name: "memorias.jsonl",
      mimeType: "application/x-ndjson",
      buffer: Buffer.from(
        [
          JSON.stringify({ key: dropKey, text: `payload drop ${dropKey}` }),
          JSON.stringify({ key: `${dropKey}-2`, text: `segundo drop ${dropKey}` }),
        ].join("\n"),
      ),
    });
    // Preview del archivo parseado (key del primer record visible en la tabla).
    await page.getByText(dropKey).first().waitFor({ state: "visible" });
    check(true, "drop: preview con filas del .jsonl parseado");
    await dropDialog.getByRole("button", { name: /IMPORTAR/ }).click();
    // Reporte (aria-live) + notice del shell (role=alert) tras onImported.
    await page.getByRole("alert").getByText(/Importados 2 registros/).waitFor({ state: "visible" });
    check(true, "drop: reporte '✓ 2 importados' + notice del shell");
    // El botón de cierre expone aria-label="Cerrar" (el texto "✕ CERRAR" queda
    // anulado por el label accesible) — matchear por el nombre accesible.
    await dropDialog.getByRole("button", { name: "Cerrar" }).click();
    // onImported remonta el grid (gridKey++) — la fila del drop aparece sin
    // re-navegar (contrato: "drop file real → records en grid"). El archivo
    // importa 2 records; el hasText del key corto matchearía ambos (el segundo
    // id es `${dropKey}-2`) → anclar al nombre accesible de la fila exacta.
    const dropRow = grid.getByRole("row", {
      name: new RegExp(`^Seleccionar ${dropKey} `),
    });
    await dropRow.waitFor({ state: "visible" });
    check(true, "grid MEMORIAS: fila del archivo dropeado visible tras import");

    // --- Fase 3: RELOAD real → persistencia OPFS ----------------------------
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
    await healthBadge.waitFor({ state: "visible" });
    await page.getByRole("button", { name: /MEMORIAS/ }).click();
    await grid.waitFor({ state: "visible" });
    const rowAfter = grid.getByRole("row").filter({ hasText: key });
    await rowAfter.waitFor({ state: "visible" });
    check(true, "persistencia: tras reload el record e2e-<ts> sigue en el grid (OPFS)");

    // Sin errores fatales inesperados (los degradados esperados son info/warn;
    // un error real de instanciación/IDB rompería el check).
    const fatal = errors.filter((e) => !e.includes("vanta_") && !e.includes("unsupported"));
    check(
      fatal.length === 0,
      `sin pageerror/console.error inesperados${fatal.length ? `: ${fatal.slice(0, 3).join(" | ")}` : ""}`,
    );
  } catch (err) {
    failures += 1;
    console.error(`[e2e] ERROR:`, err instanceof Error ? err.message : err);
    console.error(`[e2e] pageerror/console.error acumulados:\n${errors.slice(-10).join("\n")}`);
    await page.screenshot({ path: resolve(SCRIPT_DIR, "..", "e2e-wasm-fail.png") }).catch(() => {});
  } finally {
    await browser.close().catch(() => {});
    server.close();
  }

  console.log(failures === 0 ? "PASS" : `FAILED (${failures})`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((err) => {
  console.error(`[e2e] ERROR:`, err instanceof Error ? err.message : err);
  process.exit(1);
});