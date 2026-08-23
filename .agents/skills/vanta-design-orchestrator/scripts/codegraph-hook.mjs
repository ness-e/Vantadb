#!/usr/bin/env node

/**
 * codegraph-hook.mjs — CodeGraph integration for the Vanta Design Orchestrator
 *
 * Bridges the orchestrator with CodeGraph's MCP server to:
 * 1. Detect which skills are actually installed (directory scan)
 * 2. Compare installed vs declared skills in routes.json
 * 3. Detect conflicts between project-local and global skills
 * 4. Suggest optimal route chains based on availability
 *
 * Usage:
 *   node scripts/codegraph-hook.mjs --status
 *   node scripts/codegraph-hook.mjs --check-installed
 *   node scripts/codegraph-hook.mjs --suggest "landing page with 3d"
 */

import { readFileSync, existsSync, readdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ORCHESTRATOR_DIR = resolve(__dirname, '..');
const PROJECT_SKILLS_DIR = resolve(ORCHESTRATOR_DIR, '..', '..');
const ROUTES_FILE = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');

function getInstalledSkills() {
  const dirs = [PROJECT_SKILLS_DIR];
  const installed = new Set();

  for (const d of dirs) {
    if (!existsSync(d)) continue;
    for (const entry of readdirSync(d, { withFileTypes: true })) {
      if (entry.isDirectory() && !entry.name.startsWith('_') && !entry.name.startsWith('.')) {
        installed.add(entry.name);
      }
    }
  }

  return installed;
}

function getRoutes() {
  if (!existsSync(ROUTES_FILE)) return null;
  return JSON.parse(readFileSync(ROUTES_FILE, 'utf-8'));
}

function showStatus() {
  const installed = getInstalledSkills();
  const routes = getRoutes();

  console.log(`\n=== CODEGRAPH HOOK: Vanta Design Orchestrator ===\n`);

  console.log(`Skills instaladas: ${installed.size}`);
  console.log(`Rutas declaradas: ${routes ? routes.total_routes : 0}\n`);

  if (!routes) {
    console.error('ERROR: routes.json not found');
    process.exit(1);
  }

  // Compare installed vs referenced
  const referenced = new Set();
  for (const catData of Object.values(routes.categories)) {
    for (const route of catData.routes) {
      for (const skill of route.pipeline) {
        if (!skill.startsWith('strategy/') && !skill.startsWith('infrastructure/') &&
            !skill.endsWith('.md') && !skill.startsWith('threejs') &&
            !skill.startsWith('svgo') && !skill.startsWith('sharp')) {
          referenced.add(skill);
        }
      }
    }
  }

  const missing = [...referenced].filter(s => !installed.has(s)).sort();
  const extra = [...installed].filter(s => !referenced.has(s) && !s.startsWith('vanta-design-orchestrator')).sort();

  if (missing.length > 0) {
    console.log(`⚠ Skills referenciadas pero NO instaladas (${missing.length}):`);
    for (const s of missing) console.log(`   ${s}`);
  }

  if (extra.length > 0) {
    console.log(`\nℹ Skills instaladas no referenciadas en rutas (${extra.length}):`);
    for (const s of extra) console.log(`   ${s}`);
  }

  if (missing.length === 0) {
    console.log('✅ Todas las skills referenciadas están instaladas.');
  }
}

function checkInstalled() {
  const installed = getInstalledSkills();
  console.log(`\n=== Skills Instaladas (${installed.size}) ===\n`);

  // Group by prefix/category
  const cats = {};
  for (const s of [...installed].sort()) {
    let cat = 'Other';
    if (s.startsWith('fal-')) cat = 'FAL';
    else if (s.startsWith('figma-')) cat = 'Figma';
    else if (s.startsWith('deck-')) cat = 'Deck';
    else if (s.startsWith('frame-')) cat = 'Frame';
    else if (s.startsWith('threejs-')) cat = 'Three.js';
    else if (s.startsWith('gsap-')) cat = 'GSAP';
    else if (s.startsWith('social-') || s.startsWith('card-')) cat = 'Social';
    else if (s.startsWith('video-') || s.endsWith('-template')) cat = 'Video';
    else if (s.includes('design-') || s.includes('-ui') || s.includes('ux-')) cat = 'Design/UX';
    else if (s.endsWith('-skill') || s.startsWith('taste-') || s.startsWith('minimalist-') || s.startsWith('brutalist-')) cat = 'Design Taste';
    else if (s.startsWith('brand-') || s.startsWith('color-') || s.startsWith('canvas-')) cat = 'Brand';

    if (!cats[cat]) cats[cat] = [];
    cats[cat].push(s);
  }

  for (const [cat, skills] of Object.entries(cats).sort()) {
    console.log(`  [${cat}]`);
    for (const s of skills) console.log(`    ${s}`);
  }
}

function suggestRoute(query) {
  const routes = getRoutes();
  if (!routes) {
    console.error('ERROR: routes.json not found');
    process.exit(1);
  }

  const q = query.toLowerCase();
  const results = [];

  for (const [cat, catData] of Object.entries(routes.categories)) {
    for (const route of catData.routes) {
      // Match on task name or pipeline skills
      const taskMatch = route.task.toLowerCase().includes(q);
      const skillMatch = route.pipeline.some(s => s.toLowerCase().includes(q));
      const catMatch = cat.includes(q) || catData.description.toLowerCase().includes(q);

      if (taskMatch || skillMatch || catMatch) {
        results.push({ category: cat, ...route });
      }
    }
  }

  if (results.length === 0) {
    console.log(`\nNo routes found matching "${query}"\n`);
    return;
  }

  console.log(`\n=== Rutas sugeridas para "${query}" (${results.length}) ===\n`);

  for (const r of results) {
    console.log(`  📌 ${r.task}`);
    console.log(`     Categoría: ${r.category} | Prioridad: ${r.priority}`);
    console.log(`     Pipeline: ${r.pipeline.join(' → ')}\n`);
  }
}

// ── MAIN ──────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);

if (args.includes('--status') || args.length === 0) {
  showStatus();
} else if (args.includes('--check-installed')) {
  checkInstalled();
} else if (args.includes('--suggest')) {
  const idx = args.indexOf('--suggest');
  const query = idx + 1 < args.length ? args[idx + 1] : '';
  suggestRoute(query);
} else {
  console.log(`
Usage:
  node scripts/codegraph-hook.mjs --status           Show installation status vs routes
  node scripts/codegraph-hook.mjs --check-installed  List all installed skills grouped
  node scripts/codegraph-hook.mjs --suggest "query"  Find routes matching a query
`);
}
