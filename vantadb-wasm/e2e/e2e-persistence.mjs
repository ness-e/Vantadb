// E2E: VantaDB WASM browser persistence (OPFS + IndexedDB fallback) with real reload.
//
// Prerequisites:
//   - wasm-pack build vantadb-wasm --target no-modules --out-dir e2e/pkg-nomodules
//     (the web-target pkg imports the wasm as an ES module, which stable
//     Chrome/Edge reject — verified 2026-08-19; the no-modules variant loads
//     the wasm via classic script + fetch and works everywhere)
//   - playwright installed in desktop/node_modules
//   - a real browser (msedge/chrome) — Playwright's bundled chromium build
//     does not expose navigator.storage.getDirectory (OPFS), so it cannot
//     run the OPFS phase (verified 2026-08-19)
//
// Run:
//   node vantadb-wasm/e2e/e2e-persistence.mjs
//
// Flow per storage backend:
//   seed   -> open persist.html?storage=<opfs|idb>, put 10 records, save()
//   reload -> real page.reload(), reconnect, expect all 10 records back
// Failure exits non-zero. Pass output: "ALL PASS".
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const { chromium } = require("../../desktop/node_modules/playwright");

const root = normalize(fileURLToPath(new URL("..", import.meta.url))); // vantadb-wasm/
const MIME = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".css": "text/css",
};

const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url, "http://localhost");
    let p = normalize(join(root, decodeURIComponent(url.pathname)));
    if (!p.startsWith(root)) throw new Error("path escape");
    if ((await stat(p).then((s) => s.isDirectory()).catch(() => false))) p = join(p, "index.html");
    const data = await readFile(p);
    res.writeHead(200, { "content-type": MIME[extname(p)] || "application/octet-stream" });
    res.end(data);
  } catch {
    res.writeHead(404);
    res.end("not found");
  }
});

await new Promise((r) => server.listen(0, "127.0.0.1", r));
const port = server.address().port;

async function runStorage(browser, storage) {
  const ctx = await browser.newContext();
  const page = await ctx.newPage();
  page.on("console", (m) => console.log(`[${storage}][console]`, m.text()));
  page.on("pageerror", (e) => console.log(`[${storage}][pageerror]`, e.message));
  const url = `http://127.0.0.1:${port}/e2e/persist.html?storage=${storage}`;

  await page.goto(url);
  await page.waitForFunction(() => document.body.dataset.done === "1");
  const seeded = await page.evaluate(() => document.body.dataset.result);
  if (seeded !== "SEEDED") {
    console.log(`[${storage}] page log:\n${await page.evaluate(() => document.getElementById("log").textContent)}`);
    throw new Error(`[${storage}] seed failed: ${seeded}`);
  }
  console.log(`[${storage}] seed OK: 10 records put + saved`);

  await page.reload(); // real browser reload, sessionStorage carries phase -> verify
  await page.waitForFunction(() => document.body.dataset.done === "1");
  const result = await page.evaluate(() => document.body.dataset.result);
  console.log(`[${storage}] after reload: ${result}`);

  await ctx.close();
  if (result !== "PASS:10") throw new Error(`[${storage}] persistence FAILED: ${result}`);
}

const browser = await chromium.launch({ channel: "msedge" });
try {
  await runStorage(browser, "opfs");
  await runStorage(browser, "idb");
  console.log("ALL PASS");
} finally {
  await browser.close();
  server.close();
}