import { readFileSync, existsSync, writeFileSync, mkdirSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { execSync } from "child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const REPORTS_DIR = resolve(ROOT, "e2e", "reports");

function runCommand(cmd, label) {
  console.log(`\n── ${label} ──`);
  console.log(`$ ${cmd}`);
  try {
    const output = execSync(cmd, {
      cwd: ROOT,
      encoding: "utf-8",
      timeout: 300000,
      stdio: "pipe",
      maxBuffer: 50 * 1024 * 1024,
    });
    console.log(output.slice(0, 1000));
    return { success: true, output };
  } catch (err) {
    const output = err.stdout || err.stderr || err.message;
    console.log(output?.toString().slice(0, 1000));
    return { success: false, output: output?.toString() || err.message };
  }
}

function readJsonReport() {
  const path = resolve(REPORTS_DIR, "pipeline-report.json");
  if (!existsSync(path)) {
    return null;
  }
  return JSON.parse(readFileSync(path, "utf-8"));
}

function parsePlaywrightResults(jsonOutput) {
  try {
    const parsed = JSON.parse(jsonOutput);
    const specs = parsed?.suites?.[0]?.suites?.[0]?.specs || [];
    return specs.map((spec) => {
      const test = spec.tests?.[0];
      const annotations = test?.annotations || [];
      const axeNote = annotations.find((a) => a.type === "axe-violations");
      const cssNote = annotations.find((a) => a.type === "css-issues");
      const consoleNote = annotations.find((a) => a.type === "console-errors");
      const name = spec.title.replace(/ — full audit$/, "");
      const route = name.match(/\(([^)]+)\)$/)?.[1] || name;
      const shortName = name.replace(/\s*\([^)]+\)\s*$/, "").trim();
      const axeMatch = axeNote?.description?.match(/^\[[^\]]+\]\s*(\d+)\s*violation/);
      const cssMatch = cssNote?.description?.match(/^\[[^\]]+\]\s*(\d+)\s*issues/);
      const consoleMatch = consoleNote?.description?.match(/^\[[^\]]+\]\s*(\d+)\s*errors/);
      const axeViolations = axeMatch ? parseInt(axeMatch[1]) : 0;
      const cssIssues = cssMatch ? parseInt(cssMatch[1]) : 0;
      const consoleErrors = consoleMatch ? parseInt(consoleMatch[1]) : 0;
      return {
        route,
        name: shortName,
        axeViolations,
        cssIssues: cssIssues > 0 ? [cssNote.description] : [],
        consoleErrors: consoleErrors > 0 ? [consoleNote.description] : [],
        passed: axeViolations === 0 && cssIssues === 0 && consoleErrors === 0,
      };
    });
  } catch {
    return [];
  }
}

function generateFullReport() {
  const layers = [];

  layers.push(runCommand('npx stylelint "src/**/*.css"', "Capa 1 — Tokens & CSS (stylelint)"));

  layers.push(
    runCommand('npx htmlhint "index.html" "src/**/*.html"', "Capa 2 — HTML Semántico (HTMLHint)"),
  );

  layers.push(runCommand("npx @biomejs/biome check src/", "Capa 3 — Linter Unificado (Biome)"));

  const playwrightLayer = runCommand(
    "npx playwright test e2e/design-audit-pipeline.spec.ts --reporter=json --workers=1",
    "Capa 4+5 — WCAG + CSS Computado (Playwright + axe-core)",
  );
  layers.push(playwrightLayer);

  // Parse Playwright JSON output to build route-level report
  const pwResults = playwrightLayer?.output ? parsePlaywrightResults(playwrightLayer.output) : [];

  // Write pipeline-report.json from parsed results
  const pipelineReport = {
    timestamp: new Date().toISOString(),
    results: pwResults,
    summary: {
      total: pwResults.length,
      passed: pwResults.filter((r) => r.passed).length,
      failed: pwResults.filter((r) => !r.passed).length,
      totalAxeViolations: pwResults.reduce((s, r) => s + r.axeViolations, 0),
      totalCssIssues: pwResults.reduce(
        (s, r) => s + (Array.isArray(r.cssIssues) ? r.cssIssues.length : 0),
        0,
      ),
      totalConsoleErrors: pwResults.reduce(
        (s, r) => s + (Array.isArray(r.consoleErrors) ? r.consoleErrors.length : 0),
        0,
      ),
    },
  };

  if (!existsSync(REPORTS_DIR)) mkdirSync(REPORTS_DIR, { recursive: true });
  writeFileSync(
    resolve(REPORTS_DIR, "pipeline-report.json"),
    JSON.stringify(pipelineReport, null, 2),
  );

  const stylelintResult = layers[0];
  const htmlhintResult = layers[1];
  const biomeResult = layers[2];
  const playwrightResult = layers[3];

  let summary = "";
  if (pipelineReport) {
    const s = pipelineReport.summary;
    summary = `
<h2>Resumen Ejecutivo</h2>
<div class="summary-grid">
  <div class="card"><strong>${s.passed}</strong> / ${s.total} rutas pasaron</div>
  <div class="card"><strong>${s.totalAxeViolations}</strong> violaciones WCAG</div>
  <div class="card"><strong>${s.totalCssIssues}</strong> issues CSS</div>
  <div class="card"><strong>${s.totalConsoleErrors}</strong> errores console</div>
</div>`;
  }

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Design Audit — Full Report</title>
<style>
  :root {
    --base: #1e1e2e; --surface0: #45475a; --text: #a6adc8;
    --amber: #ff5500; --white: #ffffff;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: "JetBrains Mono", monospace;
    background: var(--base);
    color: var(--text);
    padding: 2rem;
    line-height: 1.6;
  }
  h1, h2, h3 { font-family: "Space Grotesk", sans-serif; color: var(--white); }
  h1 { font-size: 2rem; margin-bottom: 0.25rem; }
  .subtitle { color: var(--amber); font-size: 0.85rem; margin-bottom: 2rem; }
  .summary-grid { display: flex; gap: 1rem; flex-wrap: wrap; margin: 1rem 0 2rem; }
  .card {
    border: 2px solid var(--surface0);
    padding: 0.75rem 1rem;
    font-size: 0.9rem;
  }
  .card strong { color: var(--amber); }
  .layer {
    border: 1px solid var(--surface0);
    margin-bottom: 1rem;
  }
  .layer-header {
    background: var(--surface0);
    padding: 0.5rem 1rem;
    font-weight: bold;
    color: var(--white);
    cursor: default;
  }
  .layer-header .status {
    float: right;
    color: ${stylelintResult?.success ? "#00c853" : "var(--amber)"};
  }
  .layer-body {
    padding: 1rem;
    font-size: 0.8rem;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 300px;
    overflow-y: auto;
  }
  .layer-body::-webkit-scrollbar { width: 6px; }
  .layer-body::-webkit-scrollbar-thumb { background: var(--surface0); }
  footer { margin-top: 3rem; padding-top: 1rem; border-top: 1px solid var(--surface0); font-size: 0.75rem; }
</style>
</head>
<body>
<h1>🖥 Design Audit — Consolidated Report</h1>
<p class="subtitle">${new Date().toISOString()} · 7-layer pipeline · all open-source</p>

${summary}

<div class="layer">
  <div class="layer-header">
    Layer 1 — CSS Tokens & Style Enforcement
    <span class="status">${stylelintResult?.success ? "✓" : "✗"}</span>
  </div>
  <div class="layer-body">${
    stylelintResult ? escapeHtml(stylelintResult.output.slice(0, 2000)) : "Error running stylelint"
  }</div>
</div>

<div class="layer">
  <div class="layer-header">
    Layer 2 — HTML Semantic Validation
    <span class="status">${htmlhintResult?.success ? "✓" : "✗"}</span>
  </div>
  <div class="layer-body">${
    htmlhintResult ? escapeHtml(htmlhintResult.output.slice(0, 2000)) : "Error running HTMLHint"
  }</div>
</div>

<div class="layer">
  <div class="layer-header">
    Layer 3 — Unified Linter (Biome)
    <span class="status">${biomeResult?.success ? "✓" : "✗"}</span>
  </div>
  <div class="layer-body">${
    biomeResult ? escapeHtml(biomeResult.output.slice(0, 2000)) : "Error running Biome"
  }</div>
</div>

<div class="layer">
  <div class="layer-header">
    Layer 4+5 — WCAG Accessibility + CSS Computed
    <span class="status">${playwrightResult?.success ? "✓" : "✗"}</span>
  </div>
  <div class="layer-body">${
    playwrightResult
      ? escapeHtml(playwrightResult.output.slice(0, 2000))
      : "Error running Playwright audit"
  }</div>
</div>

<div class="layer">
  <div class="layer-header">
    Layer 6 — Visual Regression
    <span class="status">Run on demand</span>
  </div>
  <div class="layer-body">Run <code>npm run audit:visual</code> in web/ to execute visual regression tests against stored baselines.</div>
</div>

<div class="layer">
  <div class="layer-header">
    Layer 7 — This Report
    <span class="status">✓</span>
  </div>
  <div class="layer-body">Report generated at ${new Date().toISOString()}</div>
</div>

<footer>
  <p>VantaDB Design Audit Orchestrator · Open-source tools: stylelint, HTMLHint, Biome, @axe-core/playwright, pixelmatch, Playwright</p>
</footer>
</body>
</html>`;

  const htmlPath = resolve(REPORTS_DIR, "consolidated-report.html");
  if (!existsSync(REPORTS_DIR)) {
    mkdirSync(REPORTS_DIR, { recursive: true });
  }
  writeFileSync(htmlPath, html);
  console.log(`\n📄 Reporte consolidado: ${htmlPath}`);
  return htmlPath;
}

function escapeHtml(str) {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

generateFullReport();
