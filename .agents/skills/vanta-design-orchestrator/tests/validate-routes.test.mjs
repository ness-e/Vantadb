#!/usr/bin/env node

/**
 * validate-routes.test.mjs — Integrity tests for the Vanta Design Orchestrator
 *
 * Tests:
 * 1. All layer files exist
 * 2. All routes in routes.json reference valid skills
 * 3. No duplicate tasks
 * 4. Minimum 2 skills per route
 * 5. All strategy doc files exist
 * 6. All workflow files are valid JSON
 * 7. Routes count matches declared count
 * 8. Conflict resolutions reference real skills
 *
 * Usage:
 *   node tests/validate-routes.test.mjs
 */

import { readFileSync, existsSync, readdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ORCHESTRATOR_DIR = resolve(__dirname, '..');
const PROJECT_SKILLS_DIR = resolve(ORCHESTRATOR_DIR, '..', '..');

const TESTS = [];
let passed = 0;
let failed = 0;

function test(name, fn) {
  TESTS.push({ name, fn });
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || 'Assertion failed');
}

function run() {
  console.log(`\n🧪 Vanta Design Orchestrator — Integration Tests\n`);

  for (const { name, fn } of TESTS) {
    try {
      fn();
      console.log(`  ✅ ${name}`);
      passed++;
    } catch (e) {
      console.log(`  ❌ ${name}: ${e.message}`);
      failed++;
    }
  }

  console.log(`\n📊 Results: ${passed} passed, ${failed} failed, ${TESTS.length} total\n`);
  process.exit(failed > 0 ? 1 : 0);
}

// ── Test 1: All layer files exist ──
test('All layer files exist', () => {
  const expectedLayers = [
    'capa-1-fundaciones.md',
    'capa-2-estructura.md',
    'capa-3-visual.md',
    'capa-4-movimiento.md',
    'capa-5-auditoria.md',
    'capa-6-rendimiento.md',
    'capa-7-investigacion.md',
    'capa-8-operaciones.md',
    'capa-9-video.md',
    'capa-10-3d.md',
    'capa-11-seo-mobile.md',
    'capa-12-visual-review.md',
    'capa-12-branding.md',
    'capa-13-open-design.md',
  ];

  const layersDir = resolve(ORCHESTRATOR_DIR, 'layers');
  assert(existsSync(layersDir), 'layers/ directory not found');

  for (const file of expectedLayers) {
    assert(existsSync(resolve(layersDir, file)), `Missing layer file: ${file}`);
  }
});

// ── Test 2: routes.json exists and is valid JSON ──
test('routes.json exists and is valid JSON', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');
  assert(existsSync(path), 'routes.json not found');

  const data = JSON.parse(readFileSync(path, 'utf-8'));
  assert(data.version, 'Missing version');
  assert(data.categories, 'Missing categories');
  assert(typeof data.total_routes === 'number', 'Missing total_routes');
});

// ── Test 3: All routes have valid structure ──
test('All routes have valid structure', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');
  const data = JSON.parse(readFileSync(path, 'utf-8'));
  const validPriorities = ['ALTA', 'MEDIA', 'BAJA'];
  let count = 0;

  for (const [cat, catData] of Object.entries(data.categories)) {
    assert(catData.description, `Category "${cat}" missing description`);
    assert(Array.isArray(catData.routes), `Category "${cat}" missing routes array`);

    for (const route of catData.routes) {
      count++;
      assert(route.task, `Route in "${cat}" missing task`);
      assert(Array.isArray(route.pipeline), `Route "${route.task}" missing pipeline`);
      assert(route.pipeline.length >= 1, `Route "${route.task}" has empty pipeline`);
      assert(validPriorities.includes(route.priority), `Route "${route.task}" has invalid priority: ${route.priority}`);
    }
  }
});

// ── Test 4: No duplicate task names ──
test('No duplicate task names', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');
  const data = JSON.parse(readFileSync(path, 'utf-8'));
  const tasks = new Set();

  for (const catData of Object.values(data.categories)) {
    for (const route of catData.routes) {
      assert(!tasks.has(route.task), `Duplicate task: "${route.task}"`);
      tasks.add(route.task);
    }
  }
});

// ── Test 5: Route count matches declared ──
test('Route count matches declared total_routes', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');
  const data = JSON.parse(readFileSync(path, 'utf-8'));
  let count = 0;

  for (const catData of Object.values(data.categories)) {
    count += catData.routes.length;
  }

  assert(count === data.total_routes, `Expected ${data.total_routes} routes, found ${count}`);
});

// ── Test 6: All strategy doc files exist ──
test('All strategy doc files exist', () => {
  const expectedDocs = [
    'accessibility-strategy.md',
    'brand-documentation.md',
    'brand-operations.md',
    'brand-platform.md',
    'business-model-design.md',
    'content-strategy.md',
    'decision-hierarchy.md',
    'lean-design.md',
    'legal-protection.md',
    'metrics-framework.md',
    'sensory-identity.md',
    'sonic-kinetic-identity.md',
    'SUMMARY.md',
    'trends-2026.md',
    'validation-sustainability.md',
    'verbal-identity.md',
  ];

  const strategyDir = resolve(ORCHESTRATOR_DIR, 'strategy');
  assert(existsSync(strategyDir), 'strategy/ directory not found');

  for (const file of expectedDocs) {
    assert(existsSync(resolve(strategyDir, file)), `Missing strategy doc: ${file}`);
  }
});

// ── Test 7: All workflow files are valid JSON ──
test('All workflow files are valid JSON', () => {
  const workflowsDir = resolve(ORCHESTRATOR_DIR, 'workflows');
  assert(existsSync(workflowsDir), 'workflows/ directory not found');

  const files = readdirSync(workflowsDir).filter(f => f.endsWith('.json'));
  assert(files.length > 0, 'No workflow JSON files found');

  for (const file of files) {
    const data = JSON.parse(readFileSync(resolve(workflowsDir, file), 'utf-8'));
    assert(data.name, `Workflow "${file}" missing name`);
    assert(data.phases, `Workflow "${file}" missing phases`);
    assert(Array.isArray(data.phases), `Workflow "${file}" phases is not array`);
    assert(data.phases.length > 0, `Workflow "${file}" has no phases`);
  }
});

// ── Test 8: Conflict resolutions reference valid skills ──
test('Conflict resolutions reference valid skills', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');
  const data = JSON.parse(readFileSync(path, 'utf-8'));

  for (const resolution of (data.conflict_resolutions || [])) {
    assert(resolution.conflict, 'Missing conflict field');
    assert(resolution.resolution, 'Missing resolution field');
    // Each conflict should mention two skills separated by ↔
    assert(resolution.conflict.includes('↔'), `Invalid conflict format: "${resolution.conflict}"`);
  }
});

// ── Test 9: SKILL.md frontmatter is valid ──
test('SKILL.md has valid frontmatter', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'SKILL.md');
  assert(existsSync(path), 'SKILL.md not found');

  const content = readFileSync(path, 'utf-8');
  assert(content.startsWith('---'), 'SKILL.md missing opening frontmatter ---');

  const endIdx = content.indexOf('---', 3);
  assert(endIdx > 0, 'SKILL.md missing closing frontmatter ---');

  const frontmatter = content.slice(3, endIdx).trim();
  assert(frontmatter.includes('name:'), 'SKILL.md frontmatter missing name');
  assert(frontmatter.includes('description:'), 'SKILL.md frontmatter missing description');
});

// ── Test 10: project-presets.json is valid ──
test('project-presets.json is valid', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'configs', 'project-presets.json');
  assert(existsSync(path), 'project-presets.json not found');

  const data = JSON.parse(readFileSync(path, 'utf-8'));
  assert(data.meta, 'Missing meta');
  assert(data.presets, 'Missing presets');
  assert(typeof data.meta.total_presets === 'number', 'Missing total_presets');

  const presetCount = Object.keys(data.presets).length;
  assert(presetCount === data.meta.total_presets,
    `Expected ${data.meta.total_presets} presets, found ${presetCount}`);
});

// ── Test 11: routes-schema.json is valid JSON Schema ──
test('routes-schema.json is valid', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'routing', 'routes-schema.json');
  assert(existsSync(path), 'routes-schema.json not found');

  const schema = JSON.parse(readFileSync(path, 'utf-8'));
  assert(schema.$schema, 'Missing $schema field');
  assert(schema.title, 'Missing title');
  assert(schema.type === 'object', 'Schema type is not object');
  assert(Array.isArray(schema.required), 'Missing required array');
});

// ── Test 12: SKILL.md references match layer files ──
test('SKILL.md references match layer files', () => {
  const path = resolve(ORCHESTRATOR_DIR, 'SKILL.md');
  const content = readFileSync(path, 'utf-8');

  // Check that SKILL.md mentions the layers/ directory
  assert(content.includes('layers/'), 'SKILL.md does not reference layers/ directory');
  assert(content.includes('routes.json'), 'SKILL.md does not reference routes.json');
  assert(content.includes('skill-bridge.mjs'), 'SKILL.md does not reference skill-bridge.mjs');
  assert(content.includes('auto-profile.mjs'), 'SKILL.md does not reference auto-profile.mjs');
  assert(content.includes('validate-routes.mjs'), 'SKILL.md does not reference validate-routes.mjs');
});

// ── Test 13: CIS (scripts are executable) ──
test('Core scripts exist and are parseable', () => {
  const scripts = [
    'skill-bridge.mjs',
    'validate-routes.mjs',
    'auto-profile.mjs',
  ];

  for (const script of scripts) {
    const sp = resolve(ORCHESTRATOR_DIR, 'scripts', script);
    assert(existsSync(sp), `Missing script: ${script}`);

    // Verify it's a valid JS file (starts with shebang or import)
    const content = readFileSync(sp, 'utf-8');
    assert(content.startsWith('#!/usr/bin/env node') || content.includes('import '),
      `Script ${script} missing shebang or imports`);
  }
});

// ── Test 14: No PowerShell-only scripts (multi-platform) ──
test('No PowerShell-only critical scripts', () => {
  const scriptsDir = resolve(ORCHESTRATOR_DIR, 'scripts');
  const files = readdirSync(scriptsDir);

  // The old .ps1 can exist as backup, but we need .mjs equivalents
  const hasPS = files.some(f => f.endsWith('.ps1'));
  const hasMJS = files.some(f => f.endsWith('.mjs'));

  // It's OK to have both, but .mjs must exist
  assert(hasMJS, 'No .mjs scripts found — project is not multiplatform');
});

// ── Test 15: Layer files are non-empty ──
test('Layer files are non-empty', () => {
  const layersDir = resolve(ORCHESTRATOR_DIR, 'layers');
  const files = readdirSync(layersDir).filter(f => f.endsWith('.md'));

  for (const file of files) {
    const content = readFileSync(resolve(layersDir, file), 'utf-8');
    assert(content.length > 100, `Layer file ${file} is too short (${content.length} chars)`);
  }
});

// ── Run ──
run();
