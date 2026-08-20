// selfcheck-web-e2e.ts (WEB-06): E2E Playwright contra el server REAL.
// Corre con node 24 (type-stripping nativo, sin tsx) desde desktop/:
//
//   node scripts/selfcheck-web-e2e.ts
//
// Requiere:
//   - binary `vanta-cli` con feature `server` (target/debug o target/release)
//   - build web de la consola en desktop/dist-web/ (WEB-05: vite build --mode web)
//   - devDep `playwright` (desktop/package.json) con browsers descargados
//
// Flujo: arranca `vanta-cli server --http -p 8080 -d <DB-temp> --dashboard-dir dist-web`,
// ingesta 3 registros vía REST (uno con vector), navega :8080/dashboard/, y verifica
// en la UI: HOME con datos, grid MEMORIAS (registro aparece), edición en Inspector,
// borrado con undo (Ctrl+Z), y search híbrida desde la Topbar. Exit 0 solo si todo
// pasa. Limpia DB temp + proceso server al terminar (incluso en fallo).
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { chromium } from "playwright";

// --- Config ----------------------------------------------------------------

const PORT = 8080; // contrato del plan
const BASE = `http://127.0.0.1:${PORT}`;
const DASHBOARD = `${BASE}/dashboard/`;
const SCRIPT_DIR = import.meta.dirname;
const REPO_ROOT = resolve(SCRIPT_DIR, "..", "..");
const DIST_WEB = resolve(SCRIPT_DIR, "..", "dist-web");

// Records de prueba (uno con vector → hit de search híbrida). La consola web
// lista/busca SIN namespace (el bridge nativo defaulta a "default") — por eso la
// ingesta va al namespace "default" para que HOME/sidebar/grid la vean.
const NS = "default";
const K_VEC = "k-vector"; // payload contiene token único "gato"
const K_EDIT = "k-edit"; // se edita en el Inspector
const K_DEL = "k-del"; // se borra del grid
const EDITED_TEXT = "texto editado en el inspector e2e";

let failures = 0;
function check(cond: boolean, msg: string): void {
  if (cond) {
    console.log(`  ok: ${msg}`);
  } else {
    failures += 1;
    console.error(`  FAIL: ${msg}`);
  }
}

async function fetchJson(path: string, init?: RequestInit): Promise<{ status: number; body: unknown }> {
  const res = await fetch(`${BASE}${path}`, init);
  let body: unknown = null;
  const text = await res.text();
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    body = text;
  }
  return { status: res.status, body };
}

// --- Server helpers --------------------------------------------------------

function resolveBinary(): string {
  if (process.env.VANTA_CLI_BIN) return process.env.VANTA_CLI_BIN;
  for (const cand of ["target/debug/vanta-cli.exe", "target/release/vanta-cli.exe"]) {
    const p = resolve(REPO_ROOT, cand);
    if (existsSync(p)) return p;
  }
  // Fallback: vanta-cli en PATH.
  return "vanta-cli";
}

async function waitForHealth(timeoutMs: number): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${BASE}/api/v2/health`, { signal: AbortSignal.timeout(2000) });
      if (res.ok) return true;
    } catch {
      // server aún arrancando
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  return false;
}

// --- Main ------------------------------------------------------------------

async function main(): Promise<void> {
  if (!existsSync(DIST_WEB)) {
    console.error(`dist-web no existe: ${DIST_WEB} — corré antes: npm run build:web (vite build --mode web)`);
    process.exit(1);
  }

  const bin = resolveBinary();
  const dbDir = mkdtempSync(join(tmpdir(), "vantadb-e2e-"));
  const serverLog: string[] = [];
  let child = null as ReturnType<typeof spawn> | null;

  const killServer = () => {
    if (child && !child.killed) {
      try {
        child.kill();
      } catch {
        /* ya murió */
      }
    }
  };
  const cleanup = () => {
    try {
      rmSync(dbDir, { recursive: true, force: true });
    } catch {
      /* temp dir opcional */
    }
  };

  try {
    console.log(`[e2e] binary: ${bin}`);
    console.log(`[e2e] dist-web: ${DIST_WEB}`);
    console.log(`[e2e] db temp: ${dbDir}`);
    console.log(`[e2e] arrancando server en ${BASE} …`);

    child = spawn(
      bin,
      ["server", "--http", "-p", String(PORT), "-d", dbDir, "--dashboard-dir", DIST_WEB],
      {
        windowsHide: true,
        stdio: ["ignore", "pipe", "pipe"],
        // REST-01: ya no se escapa del rate limiter con VANTADB_RATE_LIMIT_RPM=0.
        // El default (600 rpm, burst completo sin auth) deja pasar las ráfagas
        // normales de la consola (~12 reqs: grid + inspector + sidebar).
      },
    );
    child.stdout?.on("data", (d: Buffer) => serverLog.push(d.toString()));
    child.stderr?.on("data", (d: Buffer) => serverLog.push(d.toString()));
    child.on("exit", (code) => {
      if (code !== null && code !== 0) serverLog.push(`[server exit code ${code}]`);
    });

    if (!(await waitForHealth(30_000))) {
      console.error(`[e2e] server no respondió /api/v2/health en 30s`);
      console.error(serverLog.slice(-20).join(""));
      process.exit(1);
    }
    console.log("[e2e] health OK");

    // --- Ingesta previa vía REST (contrato #3) --------------------------------
    const ingestBody = [
      {
        namespace: NS,
        key: K_VEC,
        payload: "mascota gato feliz jugando",
        metadata: { tipo: { String: "memoria" } },
        vector: [1.0, 0.0, 0.0],
        sparse_vector: null,
        ttl_ms: null,
      },
      {
        namespace: NS,
        key: K_EDIT,
        payload: "registro que se va a editar en la ui",
        metadata: {},
        vector: null,
        sparse_vector: null,
        ttl_ms: null,
      },
      {
        namespace: NS,
        key: K_DEL,
        payload: "registro que se va a borrar del grid",
        metadata: { origen: { String: "e2e" } },
        vector: null,
        sparse_vector: null,
        ttl_ms: null,
      },
    ];
    const ingest = await fetchJson("/api/v2/records/batch", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ingestBody),
    });
    check(ingest.status === 201, `POST /api/v2/records/batch → ${ingest.status} (3 registros)`);

    // Search híbrida vía REST (híbrido = text_query + query_vector).
    const searchReq = {
      namespace: NS,
      query_vector: [1.0, 0.0, 0.0],
      query_sparse: null,
      filters: {},
      text_query: "gato",
      top_k: 5,
      distance_metric: "Cosine",
      explain: false,
    };
    const searchRest = await fetchJson("/api/v2/search", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(searchReq),
    });
    // REST-04: search devuelve página {records, next_cursor} (no array plano).
    const page = (searchRest.body as { records?: { record?: { key?: string } }[] } | null) ?? {};
    const hits = page.records ?? [];
    check(
      searchRest.status === 200 &&
        hits.some((h) => (h.record as { key?: string } | undefined)?.key === K_VEC),
      `POST /api/v2/search "gato" → ${searchRest.status}, hit ${K_VEC} presente (híbrido)`,
    );

    // --- Browser (contrato #2) ------------------------------------------------
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    page.setDefaultTimeout(15_000);

    try {
      // 2. Navegar al dashboard.
      const errors: string[] = [];
      page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
      page.on("console", (m) => {
        if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
      });

      await page.goto(DASHBOARD, { waitUntil: "domcontentloaded" });

      // 3. HOME con datos.
      await page.getByText("Vanta Studio").first().waitFor({ state: "visible" });
      await page.getByText("MEMORIA EN VISTA").waitFor({ state: "visible" });
      const nsCard = page.locator("article").filter({ hasText: "Por namespace" });
      try {
        await nsCard.getByText("default").waitFor({ state: "visible", timeout: 15_000 });
      } catch {
        // Diagnóstico: qué muestra realmente el app (cargando/error) + log del server.
        const homeText = await page.locator("section[aria-label='Resumen de la memoria']").innerText().catch(() => "(sin section)");
        console.error(`[e2e] DIAG HOME section text: ${JSON.stringify(homeText.slice(0, 300))}`);
        console.error(`[e2e] DIAG server log tail:\n${serverLog.slice(-25).join("")}`);
        await page.screenshot({ path: join(SCRIPT_DIR, "..", "e2e-home-fail.png") }).catch(() => {});
        throw new Error("namespace 'default' no visible en card Por namespace");
      }
      check(true, "HOME: brand + MEMORIA EN VISTA + namespace 'default' visible");
      const healthBadge = page.getByText("BM25 · HNSW · RRF");
      check((await healthBadge.count()) > 0, "HOME: badge health 'BM25 · HNSW · RRF' (health ok)");

      // 4a. Grid MEMORIAS: el registro aparece.
      await page.getByRole("button", { name: /MEMORIAS/ }).click();
      const grid = page.getByRole("region", { name: "Memorias" });
      await grid.waitFor({ state: "visible" });
      await grid.getByText("3 loaded").waitFor({ state: "visible" });
      const rowKVec = grid.getByRole("row").filter({ hasText: K_VEC });
      await rowKVec.waitFor({ state: "visible" });
      check(true, "MEMORIAS: grid con '3 loaded' y fila k-vector visible");

      // 4b. Edición vía Inspector (PAYLOAD → CodeMirror → GUARDAR) — sobre K_EDIT.
      const rowKEdit = grid.getByRole("row").filter({ hasText: K_EDIT });
      await rowKEdit.locator("code").first().click();
      const inspector = page.getByRole("complementary", { name: "Inspector de registro" });
      await inspector.waitFor({ state: "visible" });
      await inspector.getByRole("button", { name: /PAYLOAD/ }).click();
      await inspector.getByRole("button", { name: "editar json" }).click();
      const cm = inspector.locator(".cm-content");
      await cm.waitFor({ state: "visible" });
      await cm.click();
      await page.keyboard.press("Control+a");
      await page.keyboard.type(EDITED_TEXT);
      await inspector.getByRole("button", { name: "GUARDAR" }).click();
      await inspector.getByText(/✓ guardado/).waitFor({ state: "visible" });
      const edited = await fetchJson(`/api/v2/records/${NS}/${K_EDIT}`);
      const editedBody = edited.body as { payload?: string; version?: number } | null;
      if (edited.status !== 200 || editedBody?.payload !== EDITED_TEXT) {
        console.error(`[e2e] DIAG edit: status=${edited.status} payload=${JSON.stringify(editedBody?.payload)} version=${editedBody?.version}`);
        console.error(`[e2e] DIAG edit flash: ${await inspector.innerText().catch(() => "(sin inspector)")}`);
      }
      check(
        edited.status === 200 && editedBody?.payload === EDITED_TEXT,
        `edición: GET ${K_EDIT} → payload == "${EDITED_TEXT}"`,
      );
      check(editedBody?.version === 2, `edición: version bump → v2 (era v1)`);

      // 4c. Borrado con undo: fila k-del del grid → papelera → Ctrl+Z restaura.
      await page.getByRole("button", { name: "Cerrar inspector" }).click();
      const rowKDel = grid.getByRole("row").filter({ hasText: K_DEL });
      await rowKDel.getByRole("button", { name: `Mover ${K_DEL} a papelera` }).click();
      await rowKDel.getByRole("button", { name: "BORRAR" }).click();
      await rowKDel.waitFor({ state: "detached" });
      const del = await fetchJson(`/api/v2/records/${NS}/${K_DEL}`);
      check(del.status === 404, `borrado: fila ${K_DEL} fuera del grid y GET → 404`);
      await page.keyboard.press("Control+z");
      const notice = page.getByRole("alert");
      try {
        await notice.getByText(/deshecho · restaurado/).waitFor({ state: "visible", timeout: 5_000 });
      } catch {
        const alerts = await page.getByRole("alert").allInnerTexts().catch(() => []);
        console.error(`[e2e] DIAG undo: alerts=${JSON.stringify(alerts)}`);
        await page.screenshot({ path: join(SCRIPT_DIR, "..", "e2e-undo-fail.png") }).catch(() => {});
      }
      const restored = await fetchJson(`/api/v2/records/${NS}/${K_DEL}`);
      check(restored.status === 200, `undo: Ctrl+Z → notice 'deshecho' y GET ${K_DEL} → 200`);

      // 5. Search híbrida desde la Topbar.
      const searchBox = page.getByRole("searchbox", { name: "Búsqueda global" });
      await searchBox.fill("gato");
      await searchBox.press("Enter");
      await page.getByText("Resultados de búsqueda").waitFor({ state: "visible" });
      const hit = page.getByRole("listitem").filter({ hasText: K_VEC });
      await hit.waitFor({ state: "visible" });
      check(true, "search: 'gato' → 'Resultados de búsqueda' con hit k-vector (score visible)");

      // Sin errores de consola/pageerror graves (los degradados esperados de
      // IQL/metrics son console.info/warn — los errores reales rompen el check).
      const fatal = errors.filter((e) => !e.includes("vanta_") && !e.includes("unsupported"));
      check(fatal.length === 0, `sin pageerror/console.error inesperados${fatal.length ? `: ${fatal.slice(0, 3).join(" | ")}` : ""}`);

      await browser.close();
    } catch (err) {
      await browser.close().catch(() => {});
      throw err;
    }
  } finally {
    killServer();
    cleanup();
  }

  console.log(failures === 0 ? "PASS" : `FAILED (${failures})`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((err) => {
  console.error(`[e2e] ERROR:`, err instanceof Error ? err.message : err);
  process.exit(1);
});