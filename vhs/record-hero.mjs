import { createServer } from "node:http";
import { constants, copyFileSync, existsSync, mkdtempSync, readFileSync, statSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { tmpdir, homedir } from "node:os";
import { fileURLToPath, pathToFileURL } from "node:url";

const HERE = fileURLToPath(new URL(".", import.meta.url));
const ALLOWED = new Map([
  ["/hero-player.html", "hero-player.html"],
  ["/hero-run.json", "hero-run.json"],
]);
const PW_MODULE = process.env.PLAYWRIGHT_MODULE;
const OUT = process.argv[2] || join(homedir(), "paso23-evidence", "hero-browser-raw.webm");
const CHROME = process.env.CHROME_BIN || undefined;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function main() {
  if (!PW_MODULE || !existsSync(PW_MODULE)) {
    throw new Error(
      "PLAYWRIGHT_MODULE env must be an absolute path to an installed playwright-core index.mjs",
    );
  }
  if (existsSync(OUT)) throw new Error(`refusing to clobber existing ${OUT}`);
  if (!existsSync(dirname(OUT))) throw new Error(`output dir missing: ${dirname(OUT)}`);
  const doc = JSON.parse(readFileSync(join(HERE, "hero-run.json"), "utf8"));
  if (doc.schema_version !== 1 || doc.frames?.length !== 5)
    throw new Error("hero-run.json invalid — run examples/hero_demo.py first");

  const { chromium } = await import(pathToFileURL(resolve(PW_MODULE)).href);

  const server = createServer((req, res) => {
    if (req.method !== "GET" && req.method !== "HEAD") {
      res.writeHead(405);
      res.end();
      return;
    }
    const file = ALLOWED.get(req.url.split("?")[0]);
    if (!file) {
      res.writeHead(404);
      res.end();
      return;
    }
    try {
      const body = readFileSync(join(HERE, file));
      res.writeHead(200, {
        "Content-Type": file.endsWith(".html") ? "text/html" : "application/json",
      });
      res.end(req.method === "GET" ? body : undefined);
    } catch {
      res.writeHead(500);
      res.end();
    }
  });
  await new Promise((r, j) => {
    server.on("error", j);
    server.listen(0, "127.0.0.1", r);
  });
  const url = `http://127.0.0.1:${server.address().port}/hero-player.html`;

  const videoDir = mkdtempSync(join(tmpdir(), "vanta-hero-video-"));
  let browser, ctx;
  try {
    browser = await chromium.launch({
      headless: true,
      executablePath: CHROME,
    });
    console.log(`browser: ${browser.version()}`);
    ctx = await browser.newContext({
      viewport: { width: 1280, height: 800 },
      reducedMotion: "no-preference",
      recordVideo: { dir: videoDir, size: { width: 1280, height: 800 } },
    });
    const page = await ctx.newPage();
    await page.goto(url);
    await page.waitForFunction("window.__demoReady === true", null, { timeout: 10000 });
    await page.evaluate("window.startDemo()");
    await page.waitForFunction("window.__demoDone === true", null, { timeout: 70000 });
    await sleep(500);
    await ctx.close();
    const video = page.video();
    const raw = await video.path();
    copyFileSync(raw, OUT, constants.COPYFILE_EXCL);
    if (!statSync(OUT, { throwIfNoEntry: false })?.size) throw new Error("empty/missing capture");
    console.log(`captured ${OUT} (from ${raw})`);
  } finally {
    try {
      if (ctx) await ctx.close();
    } catch {}
    try {
      if (browser) await browser.close();
    } catch {}
    server.close();
    console.log(`raw video dir preserved: ${videoDir}`);
  }
}

main().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
