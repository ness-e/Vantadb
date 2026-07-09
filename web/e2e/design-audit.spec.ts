import { test, expect } from "@playwright/test";

const ROUTES = [
  { path: "/", name: "Home / Landing" },
  { path: "/engine", name: "Engine" },
  { path: "/architecture", name: "Architecture" },
  { path: "/pricing", name: "Pricing" },
  { path: "/security", name: "Security" },
  { path: "/latency", name: "Latency" },
  { path: "/changelog", name: "Changelog" },
  { path: "/benchmarks", name: "Benchmarks" },
  { path: "/docs", name: "Docs" },
  { path: "/integrations", name: "Integrations" },
  { path: "/config", name: "Config" },
  { path: "/cost", name: "Cost" },
  { path: "/storage", name: "Storage" },
  { path: "/maint", name: "Maintenance" },
  { path: "/playground", name: "Playground" },
  { path: "/why-vantadb", name: "Why VantaDB" },
  { path: "/use-cases", name: "Use Cases" },
  { path: "/blog", name: "Blog" },
  { path: "/about", name: "About" },
  { path: "/about/team", name: "Team" },
  { path: "/about/contact", name: "Contact" },
  { path: "/about/company", name: "Company" },
  { path: "/about/community", name: "Community" },
  { path: "/solutions/ai-agents", name: "AI Agents" },
  { path: "/solutions/ai-ide-tooling", name: "AI IDE Tooling" },
  { path: "/solutions/local-rag", name: "Local RAG" },
];

const ALLOWED_BG = [
  /rgb\(0,\s*0,\s*0\)/,
  /rgb\(8,\s*8,\s*8\)/,
  /rgb\(10,\s*10,\s*10\)/,
  /rgb\(13,\s*13,\s*13\)/,
  /rgb\(15,\s*15,\s*15\)/,
  /rgb\(20,\s*20,\s*20\)/,
  /rgb\(22,\s*22,\s*22\)/,
  /rgb\(24,\s*24,\s*24\)/,
  /rgb\(26,\s*26,\s*26\)/,
  /rgb\(30,\s*30,\s*30\)/,
  /rgb\(36,\s*36,\s*36\)/,
  /rgb\(40,\s*40,\s*40\)/,
  /rgb\(255,\s*255,\s*255\)/,
  /rgb\(245,\s*245,\s*245\)/,
  /rgb\(240,\s*240,\s*240\)/,
  /rgba\(0,\s*0,\s*0,\s*0\)/,
  /transparent/i,
  /#ff5500/i,
  /rgb\(255,\s*85,\s*0\)/,
];

const CONTAINER_TAGS = new Set([
  "div",
  "section",
  "article",
  "header",
  "footer",
  "main",
  "aside",
  "nav",
  "form",
  "fieldset",
  "table",
  "figure",
  "details",
  "dialog",
]);

const IGNORE_BORDER_TAGS = new Set([
  "span",
  "a",
  "strong",
  "em",
  "b",
  "i",
  "u",
  "small",
  "sub",
  "sup",
  "label",
  "input",
  "textarea",
  "select",
  "option",
  "button",
  "path",
  "svg",
  "img",
  "br",
  "hr",
  "wbr",
]);

function parseBlur(boxShadow: string): number {
  if (!boxShadow || boxShadow === "none") return 0;
  const layer = boxShadow.split(",")[0].trim();
  const tokens = layer.split(/\s+/).filter((t) => t.endsWith("px"));
  if (tokens.length >= 3) {
    return parseInt(tokens[2]) || 0;
  }
  return 0;
}

interface Violation {
  route: string;
  selector: string;
  property: string;
  value: string;
  expected: string;
}

test.describe("Swiss Brutalism Design Audit", () => {
  const allViolations: Violation[] = [];
  let routeCount = 0;

  test.afterAll(() => {
    const byRoute: Record<string, Violation[]> = {};
    for (const v of allViolations) {
      if (!byRoute[v.route]) byRoute[v.route] = [];
      byRoute[v.route].push(v);
    }
    console.log(`\n=== SWISS BRUTALISM DESIGN AUDIT REPORT ===`);
    console.log(`Routes checked: ${routeCount}`);
    console.log(`Total violations: ${allViolations.length}`);
    if (allViolations.length > 0) {
      for (const [route, violations] of Object.entries(byRoute)) {
        console.log(`\n${route} (${violations.length}):`);
        for (const v of violations) {
          console.log(`  [${v.property}] ${v.selector} = "${v.value}" — expected ${v.expected}`);
        }
      }
    } else {
      console.log("  All design tokens compliant!");
    }
    console.log(`\n=== END REPORT ===\n`);
  });

  for (const route of ROUTES) {
    test(`${route.name} (${route.path}) — design token compliance`, async ({ page }) => {
      routeCount++;
      const violations: Violation[] = [];
      const push = (sel: string, prop: string, val: string, exp: string) => {
        violations.push({
          route: route.path,
          selector: sel,
          property: prop,
          value: val,
          expected: exp,
        });
      };

      await page.goto(route.path);
      await page.waitForLoadState("networkidle");
      await page.waitForTimeout(500);

      const title = await page.title();
      expect(title).toContain("VantaDB");

      const elements = await page.locator("[class*='nc-']").all();
      const sampled = elements.slice(0, 80);

      for (const el of sampled) {
        const tag = (await el.evaluate((node) => node.tagName.toLowerCase())) as string;
        const cls = (await el.getAttribute("class")) || "";
        const selStr = `${tag}.${cls
          .split(/\s+/)
          .filter((c) => c.startsWith("nc-"))
          .join(".")}`;

        const styles = await el.evaluate((node) => {
          const s = getComputedStyle(node);
          return {
            borderRadius: s.borderRadius,
            boxShadow: s.boxShadow,
            borderTopWidth: s.borderTopWidth,
            borderRightWidth: s.borderRightWidth,
            borderBottomWidth: s.borderBottomWidth,
            borderLeftWidth: s.borderLeftWidth,
            fontFamily: s.fontFamily,
            color: s.color,
            backgroundColor: s.backgroundColor,
          };
        });

        if (CONTAINER_TAGS.has(tag) && styles.borderRadius && styles.borderRadius !== "0px") {
          push(selStr, "border-radius", styles.borderRadius, "0px");
        }

        if (CONTAINER_TAGS.has(tag)) {
          const blur = parseBlur(styles.boxShadow);
          if (blur > 1) {
            push(selStr, "box-shadow blur", `${blur}px`, "≤1px (hard shadow)");
          }
        }

        if (CONTAINER_TAGS.has(tag)) {
          const widths = [
            parseFloat(styles.borderTopWidth) || 0,
            parseFloat(styles.borderRightWidth) || 0,
            parseFloat(styles.borderBottomWidth) || 0,
            parseFloat(styles.borderLeftWidth) || 0,
          ];
          const maxW = Math.max(...widths);
          if (maxW > 0 && maxW < 2) {
            push(selStr, "border-width (thin)", `${maxW}px`, "≥2px or 0 for non-bordered elements");
          }
        }

        const fgMatch = styles.color.match(/rgb\(\d+,\s*\d+,\s*\d+\)/);
        if (tag !== "svg" && tag !== "path" && tag !== "img" && fgMatch) {
          const rgb = fgMatch[0].match(/\d+/g)?.map(Number);
          if (rgb) {
            const [r, g, b] = rgb;
            if (r > 200 && g > 200 && b > 200) {
              // known light color
            } else if (r < 40 && g < 40 && b < 40) {
              // known dark color
            } else if (r === 255 && g === 85 && b === 0) {
              // known accent color
            } else if (r < 80 && g < 80 && b < 80) {
              // known dark color
            } else if (r === 0 && g === 200 && b === 83) {
              // known accent color
            } else if (r === 255 && g === 23 && b === 68) {
              // known accent color
            } else if (r > 80 && g > 80 && b > 80 && r < 150 && g < 150 && b < 150) {
              // known mid color
            } else if (r > 150 && g > 150 && b > 150 && r < 200 && g < 200 && b < 200) {
              // known light color
            } else {
              push(
                selStr,
                "color (unexpected)",
                styles.color,
                "neutral, white, black, amber, success, danger",
              );
            }
          }
        }

        if (
          styles.backgroundColor &&
          styles.backgroundColor !== "rgba(0, 0, 0, 0)" &&
          styles.backgroundColor !== "transparent"
        ) {
          const isAllowed = ALLOWED_BG.some((re) => re.test(styles.backgroundColor));
          if (!isAllowed) {
            push(
              selStr,
              "background-color",
              styles.backgroundColor,
              "dark/light neutral or amber accent",
            );
          }
        }

        const VALID_FONTS = [
          /jetbrains/i,
          /space grotesk/i,
          /outfit/i,
          /monospace/i,
          /sans-serif/i,
          /system-ui/i,
        ];
        const fontOk = VALID_FONTS.some((re) => re.test(styles.fontFamily));
        if (!fontOk && tag !== "svg" && tag !== "path") {
          push(selStr, "font-family", styles.fontFamily, "JetBrains Mono / Space Grotesk / Outfit");
        }
      }

      const reducedMotion = await page.evaluate(() => {
        const sheets = [...document.styleSheets].flatMap((s) => {
          try {
            return [...(s.cssRules || [])];
          } catch {
            return [];
          }
        });
        return sheets.some(
          (r) =>
            r.type === CSSRule.MEDIA_RULE &&
            (r as CSSMediaRule).media.mediaText.includes("prefers-reduced-motion"),
        );
      });

      if (!reducedMotion) {
        console.warn(`  ⚠ ${route.path}: No @media (prefers-reduced-motion) rule found`);
      }

      allViolations.push(...violations);

      if (violations.length > 0) {
        const msg = violations
          .map((v) => `  [${v.property}] ${v.selector}: "${v.value}" (expected: ${v.expected})`)
          .join("\n");
        console.log(`\n${route.path} — ${violations.length} violation(s):\n${msg}`);
      }

      expect(violations).toEqual([]);
    });
  }
});
