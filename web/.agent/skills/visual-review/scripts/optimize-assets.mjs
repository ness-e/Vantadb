#!/usr/bin/env node
/**
 * Optimize image assets for production.
 * Uses: ImageMagick, cwebp, squoosh-cli, sharp-cli
 *
 * Uso:
 *   node .agent/skills/visual-review/scripts/optimize-assets.mjs --input dist/assets
 *
 * Flags:
 *   --input    Directorio de assets (default: dist/assets)
 *   --quality  Calidad para WebP/AVIF (default: 80)
 *   --png      Optimizar PNGs con ImageMagick (default: true)
 *   --webp     Convertir a WebP (default: true)
 *   --avif     Convertir a AVIF (default: false)
 */

import { execSync } from "child_process";
import { existsSync, readdirSync } from "fs";
import { extname, join, resolve } from "path";
import { cwd } from "process";

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

const INPUT = resolve(args.input || join(cwd(), "dist", "assets"));
const QUALITY = parseInt(args.quality || "80");
const PNG = args.png !== "false";
const WEBP = args.webp !== "false";
const AVIF = args.avif === "true";

if (!existsSync(INPUT)) {
  console.error("Input directory not found:", INPUT);
  process.exit(1);
}

function run(cmd) {
  try {
    const out = execSync(cmd, { encoding: "utf-8", timeout: 60000 });
    return out.trim();
  } catch (e) {
    return e.message || "";
  }
}

function humanSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / 1024 / 1024).toFixed(1) + " MB";
}

const files = readdirSync(INPUT);
let totalSaved = 0;
let processed = 0;

console.log("Optimizing assets in:", INPUT);
console.log("Quality:", QUALITY + "%");
console.log("");

for (const file of files) {
  const ext = extname(file).toLowerCase();
  const fullPath = join(INPUT, file);
  const originalSize = existsSync(fullPath) ? require("fs").statSync(fullPath).size : 0;

  // PNG optimization via ImageMagick
  if (PNG && ext === ".png") {
    const outPath = join(INPUT, file.replace(/\.png$/, "-optimized.png"));
    run(`magick convert "${fullPath}" -strip -quality ${QUALITY} "${outPath}"`);
    if (existsSync(outPath)) {
      const newSize = require("fs").statSync(outPath).size;
      const saved = originalSize - newSize;
      if (saved > 0) {
        run(`move /Y "${outPath}" "${fullPath}"`);
        totalSaved += saved;
        processed++;
        console.log(`  ✓ ${file}: ${humanSize(originalSize)} → ${humanSize(newSize)} (${(saved/originalSize*100).toFixed(0)}% saved)`);
      } else {
        require("fs").unlinkSync(outPath);
      }
    }
  }

  // WebP conversion
  if (WEBP && (ext === ".png" || ext === ".jpg" || ext === ".jpeg")) {
    const baseName = file.replace(/\.[^.]+$/, "");
    const webpPath = join(INPUT, baseName + ".webp");
    run(`cwebp -q ${QUALITY} "${fullPath}" -o "${webpPath}"`);
    if (existsSync(webpPath)) {
      const webpSize = require("fs").statSync(webpPath).size;
      if (webpSize < originalSize * 0.95) {
        totalSaved += originalSize - webpSize;
        processed++;
        console.log(`  ✓ ${file} → ${baseName}.webp: ${humanSize(originalSize)} → ${humanSize(webpSize)} (${((originalSize - webpSize)/originalSize*100).toFixed(0)}% saved)`);
      } else {
        require("fs").unlinkSync(webpPath);
      }
    }
  }
}

console.log("");
console.log(`Done. ${processed} files processed, ${humanSize(totalSaved)} total savings.`);
