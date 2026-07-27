import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { readFileSync, writeFileSync, existsSync, mkdirSync, appendFileSync, renameSync } from "fs";
import { resolve, dirname } from "path";

const BASE_URL = "http://localhost:5173";
const REPORTS_DIR = resolve("e2e/reports");
const SCRATCH_FILE = resolve(REPORTS_DIR, ".scratch.ndjson");

const ROUTES = [
  { path: "/", name: "Home" },
  { path: "/config", name: "Setup Wizard" },
  { path: "/cost", name: "Cost Analyzer" },
  { path: "/storage", name: "Data Vault" },
  { path: "/maint", name: "Ops Log" },
  { path: "/playground", name: "Dev Console" },
  { path: "/why-vantadb", name: "Pitch Deck" },
  { path: "/use-cases", name: "Field Report" },
  { path: "/blog", name: "Reading Room" },
  { path: "/solutions/ai-agents", name: "Agent Blueprint" },
  { path: "/solutions/ai-ide-tooling", name: "IDE Spec" },
  { path: "/solutions/local-rag", name: "Local RAG Pipeline" },
  { path: "/docs/core-engine", name: "Core Engine" },
  { path: "/docs/data-model", name: "Data Model" },
  { path: "/docs/query-language", name: "Query Language" },
  { path: "/docs/indexing", name: "Indexing" },
  { path: "/docs/vector-search", name: "Vector Search" },
  { path: "/docs/graph-traversal", name: "Graph Traversal" },
  { path: "/docs/time-series", name: "Time Series" },
  { path: "/docs/schemas", name: "Schemas" },
  { path: "/docs/relationships", name: "Relationships" },
  { path: "/docs/connectors", name: "Connectors" },
  { path: "/docs/security", name: "Security" },
  { path: "/docs/clustering", name: "Clustering" },
  { path: "/docs/extending-vantadb", name: "Extending" },
  { path: "/docs/licensing", name: "Licensing" },
];

// Disk-backed results (survives Playwright test isolation between for-loop tests)
function initScratch() {
  if (!existsSync(REPORTS_DIR)) mkdirSync(REPORTS_DIR, { recursive: true });
  writeFileSync(SCRATCH_FILE, "");
}

function appendResult(r: AuditResults) {
  appendFileSync(SCRATCH_FILE, JSON.stringify(r) + "\n");
}

function readResults(): AuditResults[] {
  if (!existsSync(SCRATCH_FILE)) return [];
  const raw = readFileSync(SCRATCH_FILE, "utf-8").trim();
  if (!raw) return [];
  return raw
    .split("\n")
    .filter(Boolean)
    .map((l) => JSON.parse(l));
}

interface AuditResults {
  route: string;
  name: string;
  axeViolations: number;
  cssIssues: string[];
  consoleErrors: string[];
  passed: boolean;
}

interface PipelineReport {
  timestamp: string;
  results: AuditResults[];
  summary: {
    total: number;
    passed: number;
    failed: number;
    totalAxeViolations: number;
    totalCssIssues: number;
    totalConsoleErrors: number;
  };
}

const DESIGN_TOKENS = {
  validFonts: [
    "JetBrains Mono",
    "Space Grotesk",
    "Outfit",
    "Inter",
    "Fira Code",
    "Cascadia Code",
    "monospace",
    "sans-serif",
  ],
  validColors: [
    "rgb(30, 30, 46)",
    "rgb(49, 50, 68)",
    "rgb(69, 71, 90)",
    "rgb(88, 91, 112)",
    "rgb(108, 112, 134)",
    "rgb(166, 173, 200)",
    "rgb(205, 214, 244)",
    "rgb(245, 245, 245)",
    "rgb(255, 85, 0)",
    "rgb(230, 74, 0)",
    "rgb(255, 119, 51)",
    "rgb(255, 255, 255)",
    "rgb(0, 0, 0)",
  ],
  validBorderRadius: ["0px"],
};

test.describe("Design Audit Pipeline", () => {
  initScratch();
  for (const route of ROUTES) {
    test(`${route.name} (${route.path}) — full audit`, async ({ page }) => {
      const result: AuditResults = {
        route: route.path,
        name: route.name,
        axeViolations: 0,
        cssIssues: [],
        consoleErrors: [],
        passed: true,
      };

      const consoleLogs: string[] = [];
      page.on("console", (msg) => {
        if (msg.type() === "error") {
          consoleLogs.push(`${msg.type()}: ${msg.text()}`);
        }
      });

      page.on("pageerror", (err) => {
        consoleLogs.push(`PAGE_ERROR: ${err.message}`);
      });

      await page.goto(`${BASE_URL}${route.path}`, {
        waitUntil: "networkidle",
        timeout: 30000,
      });

      await page.waitForTimeout(1000);

      // Wait for web fonts to finish loading
      await page.evaluate(() => document.fonts.ready);
      await page.waitForTimeout(500);

      result.consoleErrors = consoleLogs;
      if (consoleLogs.length > 0) {
        result.passed = false;
      }

      const axeResults = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
        .analyze();

      result.axeViolations = axeResults.violations.length;
      if (axeResults.violations.length > 0) {
        result.passed = false;
      }

      const cssIssues = await page.evaluate((tokens) => {
        const issues: string[] = [];

        // Check Google Fonts <link> is present in the DOM
        const gflink = document.querySelector('link[href*="fonts.googleapis.com"]');
        if (!gflink) {
          issues.push("No Google Fonts <link> found");
        }

        // Check fonts are registered (parsed from @font-face, even if not yet downloaded)
        const fontsRegistered = Array.from(document.fonts).length > 0;
        if (!fontsRegistered) {
          issues.push("No custom fonts registered in document.fonts");
        }

        // Check border-radius only on structural container elements
        const structSelector = [
          "div",
          "section",
          "header",
          "footer",
          "nav",
          "main",
          "article",
          "aside",
          "ul",
          "ol",
          "li",
          "table",
          "tr",
          "td",
          "th",
        ].join(",");
        const structElements = document.querySelectorAll(structSelector);

        for (const el of structElements) {
          const br = window.getComputedStyle(el).borderRadius;
          const brVal = parseFloat(br);
          if (!isNaN(brVal) && brVal > 0 && brVal < 50 && !br.includes("%")) {
            const tag = el.tagName.toLowerCase();
            const id = el.id ? `#${el.id}` : "";
            const cls = Array.from(el.classList).join(".");
            issues.push(
              `border-radius:${br} on <${tag}${id}${cls ? `.${cls}` : ""}> — expected 0px`,
            );
          }
        }

        return issues;
      }, DESIGN_TOKENS);

      result.cssIssues = cssIssues;
      if (cssIssues.length > 0) {
        result.passed = false;
      }

      appendResult(result);

      const axeDetail =
        axeResults.violations.length > 0
          ? axeResults.violations
              .map((v) => `  • ${v.id}: ${v.description} (${v.impact}) — ${v.help}`)
              .join("\n")
          : "  ✓ No violations";

      const cssDetail =
        cssIssues.length > 0
          ? cssIssues.map((i) => `  • ${i}`).join("\n")
          : "  ✓ All tokens compliant";

      const consoleDetail =
        consoleLogs.length > 0 ? consoleLogs.map((l) => `  • ${l}`).join("\n") : "  ✓ No errors";

      test.info().annotations.push({
        type: "axe-violations",
        description: `[${route.name}] ${axeResults.violations.length} violations\n${axeDetail}`,
      });

      test.info().annotations.push({
        type: "css-issues",
        description: `[${route.name}] ${cssIssues.length} issues\n${cssDetail}`,
      });

      test.info().annotations.push({
        type: "console-errors",
        description: `[${route.name}] ${consoleLogs.length} errors\n${consoleDetail}`,
      });

      expect.soft(result.axeViolations).toBe(0);
      expect.soft(result.consoleErrors.length).toBe(0);
    });
  }
});
