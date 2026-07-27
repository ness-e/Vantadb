import { chromium } from "playwright";
import { writeFileSync, mkdirSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT = resolve(__dirname, "..", "..", "screenshots", "eval");
mkdirSync(OUT, { recursive: true });

const ROUTES = [
  "/",
  "/docs",
  "/pricing",
  "/architecture",
  "/engine",
  "/solutions/ai-agents",
  "/blog",
  "/use-cases",
  "/changelog",
];
const VIEWPORTS = [
  { name: "desktop", w: 1440, h: 900 },
  { name: "tablet", w: 768, h: 1024 },
  { name: "mobile", w: 390, h: 844 },
];

const browser = await chromium.launch({ headless: true, channel: "chrome" });
const results = [];

for (const route of ROUTES) {
  for (const vp of VIEWPORTS) {
    const context = await browser.newContext({ viewport: { width: vp.w, height: vp.h } });
    const page = await context.newPage();
    const url = `http://localhost:5173${route}`;
    const errors = [];

    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });
    page.on("pageerror", (err) => errors.push(err.message));

    try {
      await page.goto(url, { waitUntil: "networkidle", timeout: 15000 });
      await page.waitForTimeout(500);

      const safePath = route === "/" ? "index" : route.replace(/\//g, "-").replace(/^-/, "");
      const filename = `${safePath}-${vp.name}.png`;
      const filepath = resolve(OUT, filename);
      await page.screenshot({ path: filepath, fullPage: true });

      // CSS audit via JS
      const audit = await page.evaluate(() => {
        const issues = [];
        const doc = document;

        // Contrast check on body text
        const body = doc.querySelector("p, .swiss-hero-desc");
        if (body) {
          const style = getComputedStyle(body);
          const color = style.color;
          const bg = style.backgroundColor;
          issues.push({ check: "body-text-color", value: color });
          issues.push({ check: "body-bg", value: bg });
          issues.push({ check: "body-font-size", value: style.fontSize });
          issues.push({ check: "body-line-height", value: style.lineHeight });
          issues.push({ check: "body-max-width", value: style.maxWidth });
        }

        // Check h1
        const h1 = doc.querySelector("h1");
        if (h1) {
          const s = getComputedStyle(h1);
          issues.push({ check: "h1-font-size", value: s.fontSize });
          issues.push({ check: "h1-font-family", value: s.fontFamily });
          issues.push({ check: "h1-font-weight", value: s.fontWeight });
          issues.push({ check: "h1-letter-spacing", value: s.letterSpacing });
          issues.push({ check: "h1-line-height", value: s.lineHeight });
        }

        // Headings structure
        const headings = [];
        doc.querySelectorAll("h1,h2,h3,h4,h5,h6").forEach((h) => headings.push(h.tagName));
        issues.push({ check: "heading-hierarchy", value: headings.join(" → ") });

        // Count images without alt
        const imgsNoAlt = doc.querySelectorAll("img:not([alt])").length;
        issues.push({ check: "images-without-alt", value: imgsNoAlt });

        // Nav items
        const navLinks = doc.querySelectorAll(".nav-link, .vanta-logo").length;
        issues.push({ check: "nav-links-count", value: navLinks });

        // Links
        const allLinks = doc.querySelectorAll("a").length;
        issues.push({ check: "total-links", value: allLinks });

        // Check for skip-link
        const skipLink = doc.querySelector('.skip-link, [href="#main-content"]');
        issues.push({ check: "skip-link-present", value: !!skipLink });

        return issues;
      });

      results.push({
        route,
        viewport: vp.name,
        file: filename,
        errors: errors.length,
        errorMsgs: errors.slice(0, 3),
        audit,
      });
    } catch (err) {
      results.push({ route, viewport: vp.name, error: err.message });
    } finally {
      await context.close();
    }
  }
}

await browser.close();

// Write results
writeFileSync(resolve(OUT, "..", "eval-results.json"), JSON.stringify(results, null, 2));

// Summary
console.log(`\n📸 Screenshots + CSS audit complete`);
console.log(`   Output: ${OUT}`);
for (const r of results) {
  if (r.error) {
    console.log(`   ❌ ${r.route} @ ${r.viewport}: ${r.error}`);
  } else {
    const issues = r.audit.filter((a) => a.check === "images-without-alt" && a.value > 0);
    const altIssues = issues.length > 0 ? `, ${issues[0].value} imgs w/o alt` : "";
    console.log(`   ✅ ${r.route} @ ${r.viewport}: ${r.errors} console errors${altIssues}`);
  }
}
