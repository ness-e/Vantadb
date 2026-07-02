#!/usr/bin/env node
/**
 * Visual Review Pipeline
 *
 * Uso:
 *   node .agent/skills/visual-review/scripts/visual-review-pipeline.mjs \
 *     --url http://localhost:3000 \
 *     --routes /,/about,/pricing,/blog,/docs,/download
 *
 * Flags:
 *   --url       Base URL (default: http://localhost:3000)
 *   --routes    Rutas separadas por coma (default: /)
 *   --viewports Viewports a testear (default: 1440x900,768x1024,390x844)
 *   --output    Directorio de output (default: screenshots/)
 *   --report    Generar reporte HTML (default: true)
 */

import { execSync } from "child_process";
import { existsSync, mkdirSync, writeFileSync } from "fs";
import { resolve } from "path";

const argv = process.argv.slice(2);
const args = {};
for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  if (a.startsWith("--")) {
    const eqIdx = a.indexOf("=");
    if (eqIdx !== -1) {
      const k = a.slice(2, eqIdx);
      const v = a.slice(eqIdx + 1);
      args[k] = v;
    } else {
      const k = a.slice(2);
      const next = argv[i + 1];
      if (next && !next.startsWith("--")) {
        args[k] = next;
        i++;
      } else {
        args[k] = true;
      }
    }
  }
}

const URL = (args.url || "http://localhost:3000").replace(/\/$/, "");
const ROUTES = (args.routes || "/").split(",");
const VIEWPORTS = (args.viewports || "1440x900,768x1024,390x844").split(",");
const OUTPUT = resolve(args.output || "screenshots");
const GEN_REPORT = args.report !== "false";

if (!existsSync(OUTPUT)) mkdirSync(OUTPUT, { recursive: true });

function run(cmd) {
  try {
    return execSync(cmd, { encoding: "utf-8", timeout: 30000 });
  } catch (e) {
    return e.stdout || e.message || "";
  }
}

function vpLabel(vp) {
  const map = { "1440x900": "desktop", "768x1024": "tablet", "390x844": "mobile" };
  return map[vp] || vp;
}

const results = [];

// ── Step 1: Open browser ──
console.log("Opening browser...");
run(`playwright-cli open ${URL}`);

for (const route of ROUTES) {
  console.log(`\n=== ${route} ===`);

  // Navigate
  run(`playwright-cli goto ${URL}${route}`);

  for (const vp of VIEWPORTS) {
    const label = vpLabel(vp);
    const [w, h] = vp.split("x");

    // Resize
    run(`playwright-cli resize ${w} ${h}`);
    run(`playwright-cli eval "new Promise(r => setTimeout(r, 500))"`);

    // Screenshot
    const filename = `${route === "/" ? "index" : route.slice(1)}-${label}.png`;
    const filepath = resolve(OUTPUT, filename);
    run(`playwright-cli screenshot --full-page --filename="${filepath}"`);

    // Console errors
    const consoleLog = run(`playwright-cli console`);
    const errorCount = (consoleLog.match(/\[ERROR\]/g) || []).length;

    // CSS audit via eval
    const auditRaw = run(`playwright-cli eval "
      (function() {
        var s = getComputedStyle.bind(window);
        var issues = [];
        var els = {};

        // Font-size check
        ['p','label','span','a','li','button','td','th','figcaption','small','.tl-tag','.fg-label','.framebar-label','.hero-stat-label','.term-cell-tag','.sp-step-tag','.tl-legend','.sp-sidebar-num','.sp-sidebar-icon','.footer-col-title','.section-eyebrow'].forEach(function(sel) {
          document.querySelectorAll(sel).forEach(function(el) {
            var fs = parseFloat(s(el).fontSize);
            if (fs > 0 && fs < 11) {
              var tag = el.tagName.toLowerCase();
              var cls = el.className.slice(0,40);
              issues.push({severity:'warn',category:'font-size',selector:sel,tag:tag,class:cls,value:fs+'px',msg:'Texto menor a 11px'});
            }
          });
        });

        // Contrast check (simplified — checks against parent bg assumptions)
        // Only flag elements with opacity < 0.3 on text
        document.querySelectorAll('*').forEach(function(el) {
          var fs = parseFloat(s(el).fontSize);
          if (fs > 0 && fs < 20) {
            var op = parseFloat(s(el).opacity);
            if (op > 0 && op < 0.3) {
              issues.push({severity:'warn',category:'contrast',selector:el.tagName+'.'+(el.className||'').slice(0,30),value:op,msg:'Opacidad < 0.3 en texto ('+fs+'px)'});
            }
          }
        });

        // Heading hierarchy check
        var headings = document.querySelectorAll('h1,h2,h3,h4,h5,h6');
        var lastLevel = 0;
        headings.forEach(function(h) {
          var level = parseInt(h.tagName[1]);
          if (level - lastLevel > 1 && lastLevel > 0) {
            issues.push({severity:'warn',category:'heading',selector:h.tagName,value:h.textContent.slice(0,50),msg:'Skip de nivel heading: h'+lastLevel+' -> '+h.tagName});
          }
          lastLevel = level;
        });

        // Console errors check
        issues.push({severity:'error',category:'console',value:"+errorCount+",msg:'Errores en consola: '+"+errorCount+",count:"+errorCount+"});

        return JSON.stringify(issues, null, 2);
      })()
    "`);

    const entry = { route, viewport: label, filepath, errorCount };
    results.push(entry);

    console.log(`  ${label} (${w}x${h}): ${errorCount} console errors → ${filename}`);
  }
}

// ── Close browser ──
run(`playwright-cli close`);

// ── Step 2: Generate report ──
if (GEN_REPORT) {
  const reportPath = resolve(OUTPUT, "visual-review-report.html");

  let rows = "";
  const totalErrors = results.reduce((s, r) => s + r.errorCount, 0);

  for (const r of results) {
    const filename = r.filepath.split("\\").pop() || r.filepath.split("/").pop();
    const rel = r.filepath.replace(process.cwd() + "\\", "").replace(process.cwd() + "/", "");
    rows += `<tr>
      <td>${r.route}</td>
      <td>${r.viewport}</td>
      <td class="${r.errorCount > 0 ? 'error' : 'ok'}">${r.errorCount}</td>
      <td><a href="${rel}" target="_blank">📸</a></td>
    </tr>`;
  }

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Visual Review Report — VantaDB</title>
<style>
  * { margin:0; padding:0; box-sizing:border-box; }
  body { font-family: system-ui, sans-serif; background: #0c0c0e; color: #e0ddd5; padding: 2rem; }
  h1 { font-size: 1.5rem; color: #ff6a00; margin-bottom: 0.5rem; }
  .sub { color: #7a7a7a; margin-bottom: 2rem; }
  table { width: 100%; border-collapse: collapse; }
  th, td { padding: 0.6rem 1rem; text-align: left; border-bottom: 1px solid rgba(255,255,255,0.05); }
  th { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.1em; color: #7a7a7a; }
  td { font-size: 0.85rem; }
  .error { color: #ff5f56; font-weight: 600; }
  .ok { color: #28c840; }
  .summary { display: flex; gap: 1.5rem; margin-bottom: 2rem; }
  .stat { background: rgba(255,255,255,0.02); border-radius: 8px; padding: 1rem 1.5rem; border: 1px solid rgba(255,255,255,0.04); }
  .stat-num { font-size: 2rem; font-weight: 700; color: #fff; }
  .stat-label { font-size: 0.7rem; color: #7a7a7a; margin-top: 0.25rem; }
  a { color: #ff6a00; text-decoration: none; }
  a:hover { text-decoration: underline; }
</style>
</head>
<body>
  <h1>Visual Review Report</h1>
  <p class="sub">${new Date().toISOString().slice(0,19).replace('T',' ')} — ${results.length} captures × ${ROUTES.length} routes</p>

  <div class="summary">
    <div class="stat">
      <div class="stat-num">${totalErrors}</div>
      <div class="stat-label">Total Console Errors</div>
    </div>
    <div class="stat">
      <div class="stat-num">${VIEWPORTS.length}</div>
      <div class="stat-label">Viewports</div>
    </div>
    <div class="stat">
      <div class="stat-num">${ROUTES.length}</div>
      <div class="stat-label">Routes</div>
    </div>
  </div>

  <table>
    <thead><tr><th>Route</th><th>Viewport</th><th>Errors</th><th>Screenshot</th></tr></thead>
    <tbody>${rows}</tbody>
  </table>
</body>
</html>`;

  writeFileSync(reportPath, html, "utf-8");
  console.log(`\n📋 Report generated: ${reportPath}`);
}

console.log("\n✅ Pipeline complete");
