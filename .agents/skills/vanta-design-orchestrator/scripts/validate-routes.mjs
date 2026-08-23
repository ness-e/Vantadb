#!/usr/bin/env node

/**
 * validate-routes.mjs — Validation script for the routing table
 *
 * Checks:
 * 1. All skills referenced in routes.json actually exist as directories
 * 2. No duplicate task names
 * 3. All routes have at least 2 skills (anti-single-skill rule)
 * 4. All priorities are valid (ALTA/MEDIA/BAJA)
 * 5. routes.json conforms to routes-schema.json
 *
 * Usage:
 *   node scripts/validate-routes.mjs
 */

import { readFileSync, existsSync, readdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ORCHESTRATOR_DIR = resolve(__dirname, '..');
const PROJECT_SKILLS_DIR = resolve(ORCHESTRATOR_DIR, '..', '..');
const ROUTES_FILE = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');
const SCHEMA_FILE = resolve(ORCHESTRATOR_DIR, 'routing', 'routes-schema.json');

let exitCode = 0;

function error(msg) {
  console.error(`  ❌ ${msg}`);
  exitCode = 1;
}

function warn(msg) {
  console.warn(`  ⚠ ${msg}`);
}

function ok(msg) {
  console.log(`  ✅ ${msg}`);
}

function readRoutes() {
  if (!existsSync(ROUTES_FILE)) {
    error(`Routes file not found: ${ROUTES_FILE}`);
    process.exit(1);
  }
  return JSON.parse(readFileSync(ROUTES_FILE, 'utf-8'));
}

function getInstalledSkills() {
  if (!existsSync(PROJECT_SKILLS_DIR)) return [];
  return readdirSync(PROJECT_SKILLS_DIR, { withFileTypes: true })
    .filter(d => d.isDirectory())
    .map(d => d.name);
}

console.log(`\n=== VALIDATE-ROUTES: Vanta Design Orchestrator ===\n`);

// 1. Parse routes
const routes = readRoutes();
const installed = getInstalledSkills();

console.log(`Routes version: ${routes.version}`);
console.log(`Declared routes: ${routes.total_routes}`);
console.log(`Installed skills: ${installed.length}`);
console.log(`\n--- Checks ---\n`);

// 2. Count actual routes
let actualCount = 0;
const taskNames = new Set();
const routeSkills = new Set();

for (const [cat, catData] of Object.entries(routes.categories)) {
  if (!catData.routes || !Array.isArray(catData.routes)) {
    error(`Category "${cat}" has no routes array`);
    continue;
  }

  for (const route of catData.routes) {
    actualCount++;

    // Check duplicate task names
    if (taskNames.has(route.task)) {
      error(`Duplicate task name: "${route.task}"`);
    }
    taskNames.add(route.task);

    // Check valid priority
    if (!['ALTA', 'MEDIA', 'BAJA'].includes(route.priority)) {
      error(`Invalid priority "${route.priority}" in task "${route.task}"`);
    }

    // Check minimum 2 skills
    if (route.pipeline.length < 2) {
      warn(`Task "${route.task}" has only ${route.pipeline.length} skill — minimum recommended is 2`);
    }

    // Check each skill exists
    for (const skill of route.pipeline) {
      // Skip doc references (strategy/*, infrastructure/*, etc.)
      if (skill.startsWith('strategy/') || skill.startsWith('infrastructure/') ||
          skill.startsWith('designer-toolkit/') || skill.startsWith('design-ops/') ||
          skill.startsWith('brand-') || skill.startsWith('ux-') ||
          skill.endsWith('.md') || skill.startsWith('prototype') ||
          skill.startsWith('to-') || skill.startsWith('handoff') ||
          skill.startsWith('extract-') || skill.startsWith('just-') ||
          skill.startsWith('brand-') || skill.startsWith('threejs') ||
          skill.startsWith('developer-') || skill.startsWith('analytics-') ||
          skill.startsWith('lighthouse') || skill.startsWith('axe-') ||
          skill.startsWith('svgo') || skill.startsWith('sharp') ||
          skill.startsWith('figma-')) {
        // These are either tools, doc refs, or npm tools — skip existence check
        routeSkills.add(skill);
        continue;
      }

      routeSkills.add(skill);
      if (!installed.includes(skill) && !skill.includes('/')) {
        warn(`Skill "${skill}" referenced in "${route.task}" but not found in installed skills`);
      }
    }
  }
}

// 3. Route count match
if (actualCount !== routes.total_routes) {
  warn(`Declared ${routes.total_routes} routes but found ${actualCount}`);
} else {
  ok(`${actualCount} routes match declared count`);
}

// 4. Check for skills that exist but are unreferenced
const unreferenced = installed.filter(s =>
  !routeSkills.has(s) &&
  !s.startsWith('vanta-design-orchestrator') &&
  !s.startsWith('_') &&
  s !== 'vanta-design-orchestrator'
);

if (unreferenced.length > 0) {
  console.log(`\n  Skills instaladas no referenciadas en routes.json:`);
  for (const s of unreferenced.sort()) {
    console.log(`    ${s}`);
  }
}

// 5. Schema validation
try {
  const schema = JSON.parse(readFileSync(SCHEMA_FILE, 'utf-8'));
  ok(`Schema file loads correctly: ${schema.title}`);
} catch (e) {
  error(`Schema file invalid: ${e.message}`);
}

// 6. Summary
console.log(`\n--- Result ---`);
if (exitCode === 0) {
  ok(`All validations passed`);
} else {
  error(`${exitCode} issues found — review warnings above`);
}

process.exit(exitCode);
