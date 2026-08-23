#!/usr/bin/env node

/**
 * Vanta Design Orchestrator — Skill Bridge CLI
 * Multiplatform (Node.js) version of the original PowerShell skill-bridge.ps1
 *
 * Usage:
 *   node scripts/skill-bridge.mjs --list-skills
 *   node scripts/skill-bridge.mjs --list-categories
 *   node scripts/skill-bridge.mjs --list-presets
 *   node scripts/skill-bridge.mjs --route landing-page
 *   node scripts/skill-bridge.mjs --preset landing-page
 *   node scripts/skill-bridge.mjs --analyze-conflicts
 *   node scripts/skill-bridge.mjs --validate-routes
 *   node scripts/skill-bridge.mjs --auto-profile
 */

import { readFileSync, existsSync, readdirSync, statSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ORCHESTRATOR_DIR = resolve(__dirname, '..');
const PROJECT_SKILLS_DIR = resolve(ORCHESTRATOR_DIR, '..', '..');
const PRESETS_FILE = resolve(ORCHESTRATOR_DIR, 'configs', 'project-presets.json');
const ROUTES_FILE = resolve(ORCHESTRATOR_DIR, 'routing', 'routes.json');

function getAllSkillDirs(baseDir) {
  if (!existsSync(baseDir)) return [];
  return readdirSync(baseDir, { withFileTypes: true })
    .filter(d => d.isDirectory())
    .map(d => d.name);
}

function getAllSkills() {
  const project = getAllSkillDirs(PROJECT_SKILLS_DIR);
  return [...new Set([...project])].sort();
}

function getSkillCategory(name) {
  const categories = {
    'fal-generate': 'FAL', 'fal-image-edit': 'FAL', 'fal-3d': 'FAL',
    'fal-kling-o3': 'FAL', 'fal-upscale': 'FAL', 'fal-video-edit': 'FAL',
    'fal-lip-sync': 'FAL', 'fal-vision': 'FAL',
    'figma-generate-design': 'Figma', 'figma-create-new-file': 'Figma',
    'figma-generate-library': 'Figma', 'figma-code-connect-components': 'Figma',
    'figma-implement-design': 'Figma', 'figma-use': 'Figma',
    'figma-create-design-system-rules': 'Figma',
    'deck-swiss-international': 'Deck', 'deck-open-slide-canvas': 'Deck',
    'deck-guizang-editorial': 'Deck',
    'ppt-keynote': 'Deck', 'pptx': 'Deck', 'pptx-html-fidelity-audit': 'Deck',
    'frame-glitch-title': 'Frame', 'frame-light-leak-cinema': 'Frame',
    'frame-liquid-bg-hero': 'Frame', 'frame-logo-outro': 'Frame',
    'frame-data-chart-nyt': 'Frame', 'frame-flowchart-sticky': 'Frame',
    'social-x-post-card': 'Social', 'social-reddit-card': 'Social',
    'social-spotify-card': 'Social', 'card-twitter': 'Social', 'card-xiaohongshu': 'Social',
    'video-hyperframes': 'Video', 'remotion': 'Video',
    'docx': 'Docs', 'pdf': 'Docs', 'doc-kami-parchment': 'Docs',
    'brandkit': 'Brand', 'canvas-design': 'Brand', 'algorithmic-art': 'Brand',
    'theme-factory': 'Brand', 'color-expert': 'Brand', 'creative-director': 'Brand',
    'design-brief': 'Brand', 'design-md': 'Brand',
    'shadcn-ui': 'UI', 'platform-design': 'UI', 'login-flow': 'UI', 'faq-page': 'UI',
    'd3-visualization': 'Visual', 'mockup-device-3d': 'Visual',
    'shader-dev': 'Visual', 'hand-drawn-diagrams': 'Visual',
    'imagegen-frontend-web': 'Image', 'imagegen-frontend-mobile': 'Image',
    'poster-hero': 'Image', 'screenshots-marketing': 'Image',
    'speech': 'Audio',
    'copywriting': 'Marketing', 'competitive-ads-extractor': 'Marketing',
    'motion': 'Animation', 'animejs': 'Animation', 'design-motion-principles': 'Animation',
    'impeccable': 'Audit', 'impeccable-design-polish': 'Audit',
    'ai-seo': 'SEO', 'seo': 'SEO', 'roier-seo': 'SEO', 'seo-audit': 'SEO',
    'hyperframes': 'Video', 'hyperframes-animation': 'Video',
    'remotion-best-practices': 'Video', 'threejs': '3D',
    'threejs-fundamentals': '3D', 'threejs-geometry': '3D',
    'threejs-materials': '3D', 'threejs-interaction': '3D',
    'threejs-animation': '3D', 'threejs-shaders': '3D',
    'export-download-debugging': 'Utility', 'pr-feedback-quality-gate': 'Utility',
    'release-notes-one-pager': 'Utility', 'reference-design-contract': 'Utility',
    'web-artifacts-builder': 'Utility', 'research-decision-room': 'Utility',
    'plan-design-review': 'Utility', 'brainstorming': 'Process',
    'writing-plans': 'Process', 'systematic-debugging': 'Process',
  };

  if (categories[name]) return categories[name];
  if (/^fal-/.test(name)) return 'FAL';
  if (/^figma-/.test(name)) return 'Figma';
  if (/^deck-/.test(name)) return 'Deck';
  if (/^frame-/.test(name)) return 'Frame';
  if (/^social-/.test(name)) return 'Social';
  if (/^card-/.test(name)) return 'Social';
  if (/^video-/.test(name)) return 'Video';
  if (/^threejs-/.test(name)) return '3D';
  if (/^gsap-/.test(name)) return 'Animation';
  if (/template$/.test(name)) return 'Template';
  return 'Other';
}

function showHelp() {
  console.log(`
VANTA DESIGN ORCHESTRATOR — SKILL BRIDGE CLI
=============================================

USOS:
  --list-skills         Listar todas las skills disponibles
  --list-categories     Listar skills agrupadas por categoría
  --list-presets        Listar presets de proyecto disponibles
  --route <name>        Mostrar el pipeline de skills para un preset
  --preset <name>       Cargar instrucciones detalladas del preset
  --analyze-conflicts   Detectar skills duplicadas y conflictos
  --validate-routes     Validar que routes.json referencia skills existentes
  --auto-profile        Detectar perfil del proyecto actual
  --help                Mostrar esta ayuda
`);
}

function showAllSkills() {
  const all = getAllSkills();
  console.log(`\n=== SKILLS DISPONIBLES: ${all.length} ===\n`);

  const cats = {};
  for (const s of all) {
    const c = getSkillCategory(s);
    if (!cats[c]) cats[c] = [];
    cats[c].push(s);
  }

  for (const [cat, skills] of Object.entries(cats).sort()) {
    console.log(`  [${cat}] (${skills.length})`);
    for (const s of skills.sort()) console.log(`    ${s}`);
  }
}

function showCategories() {
  const all = getAllSkills();
  console.log(`\n=== SKILLS POR CATEGORÍA ===\n`);

  const cats = {};
  for (const s of all) {
    const c = getSkillCategory(s);
    if (!cats[c]) cats[c] = [];
    cats[c].push(s);
  }

  let total = 0;
  for (const [cat, skills] of Object.entries(cats).sort()) {
    const count = skills.length;
    total += count;
    console.log(`  ${cat.padEnd(20)} ${count} skills`);
  }
  console.log(`  ${'---'.padEnd(20)} ---`);
  console.log(`  ${'TOTAL'.padEnd(20)} ${total} skills`);
}

function showPresets() {
  if (!existsSync(PRESETS_FILE)) {
    console.error(`ERROR: Presets file not found at ${PRESETS_FILE}`);
    return;
  }
  const presets = JSON.parse(readFileSync(PRESETS_FILE, 'utf-8'));
  const count = Object.keys(presets.presets).length;
  console.log(`\n=== PRESETS DE PROYECTO: ${count} ===\n`);

  for (const [key, p] of Object.entries(presets.presets).sort()) {
    const phases = Object.keys(p.phases).join(' → ');
    console.log(`  ${key}`);
    console.log(`    ${p.name}`);
    console.log(`    Fases: ${phases}\n`);
  }
}

function showRoute(presetName) {
  if (!existsSync(PRESETS_FILE)) {
    console.error('ERROR: Presets file not found');
    return;
  }
  const presets = JSON.parse(readFileSync(PRESETS_FILE, 'utf-8'));
  const p = presets.presets[presetName];
  if (!p) {
    console.error(`ERROR: Preset '${presetName}' not found.`);
    console.error(`Available: ${Object.keys(presets.presets).join(', ')}`);
    return;
  }

  console.log(`\n=== RUTA: ${presetName} (${p.name}) ===\n`);
  for (const [phase, skills] of Object.entries(p.phases)) {
    console.log(`  ╔══ ${phase} ══╗`);
    for (const s of skills) console.log(`  ║ ${s}`);
    console.log(`  ╚════════════════╝\n`);
  }

  if (p.recommended_skills) {
    console.log(`  + Recomendadas:`);
    for (const s of p.recommended_skills) console.log(`    ${s}`);
  }
}

function showAnalyzeConflicts() {
  console.log(`\n=== ANÁLISIS DE CONFLICTOS ===\n`);

  const project = getAllSkillDirs(PROJECT_SKILLS_DIR).sort();

  const pairs = [
    ['impeccable', 'impeccable-design-polish'],
    ['emil-design-eng', 'emilkowalski-motion'],
    ['design-taste-frontend', 'taste-skill'],
    ['redesign-existing-projects', 'redesign-skill'],
    ['minimalist-ui', 'minimalist-skill'],
    ['industrial-brutalist-ui', 'brutalist-skill'],
  ];

  console.log(`  Versiones en conflicto (similar name):`);
  for (const [a, b] of pairs) {
    const found = [a, b].filter(s => project.includes(s));
    if (found.length > 0) {
      console.log(`    ${a} ↔ ${b}`);
      console.log(`      → Tienes: ${found.join(', ')}`);
      console.log(`      → Recomendación: USAR ${b} (open-design)`);
    }
  }
}

function validateRoutes() {
  if (!existsSync(ROUTES_FILE)) {
    console.error(`ERROR: Routes file not found at ${ROUTES_FILE}`);
    return;
  }

  const routes = JSON.parse(readFileSync(ROUTES_FILE, 'utf-8'));
  const allSkills = getAllSkills();
  const routeSkills = new Set();
  let errors = 0;
  let warnings = 0;

  // Collect all skill references from routes
  for (const [cat, catData] of Object.entries(routes.categories)) {
    for (const route of catData.routes) {
      for (const s of route.pipeline) {
        // strategy/* and infra/* are docs, not skill dirs — skip those
        if (s.startsWith('strategy/') || s.startsWith('infrastructure/') || s.startsWith('designer-toolkit/') || s.startsWith('design-ops/')) {
          continue;
        }
        routeSkills.add(s);
      }
    }
  }

  console.log(`\n=== VALIDACIÓN DE RUTAS ===\n`);
  console.log(`  Skills referenciadas en routes.json: ${routeSkills.size}`);
  console.log(`  Skills instaladas: ${allSkills.length}\n`);

  for (const s of [...routeSkills].sort()) {
    if (!allSkills.includes(s)) {
      console.warn(`  ⚠ SKILL NO ENCONTRADA: ${s}`);
      warnings++;
    }
  }

  if (warnings > 0) {
    console.log(`\n  ${warnings} warnings — skills referenciadas pero no instaladas.`);
  } else {
    console.log(`  ✅ Todas las skills referenciadas están instaladas.`);
  }

  // Check for dead skills (installed but not referenced)
  const unreferenced = allSkills.filter(s => !routeSkills.has(s) && !s.startsWith('vanta-design-orchestrator'));
  if (unreferenced.length > 0) {
    console.log(`\n  Skills instaladas no referenciadas en rutas:`);
    for (const s of unreferenced.sort()) {
      console.log(`    ${s}`);
    }
  }

  console.log(`\n  Total: ${errors} errors, ${warnings} warnings`);
}

function autoProfile() {
  console.log(`\n=== AUTO-PROFILE (Detección de Perfil de Proyecto) ===\n`);

  const projectRoot = resolve(PROJECT_SKILLS_DIR, '..');
  const files = readdirSync(projectRoot);

  const hasPackageJson = files.includes('package.json');
  const hasCargoToml = files.includes('Cargo.toml');
  const hasPyProject = files.includes('pyproject.toml') || files.includes('setup.py');
  const hasDockerfile = files.includes('Dockerfile') || files.includes('docker-compose.yml');
  const hasNextConfig = files.some(f => f.startsWith('next.config'));
  const hasVite = files.some(f => f.startsWith('vite.config'));
  const hasReact = hasPackageJson && (() => {
    try {
      const pkg = JSON.parse(readFileSync(resolve(projectRoot, 'package.json'), 'utf-8'));
      return !!(pkg.dependencies?.react || pkg.devDependencies?.react);
    } catch { return false; }
  })();

  if (hasCargoToml) {
    console.log(`  📦 Perfil detectado: RUST`);
    console.log(`  → Cargo.toml encontrado`);
    if (hasReact) console.log(`  → + React frontend (Tauri?)`);
    console.log(`\n  Capas recomendadas:`);
    console.log(`    1-8 (completas) | 12 (branding) | 13 (extras)`);
    console.log(`  Saltar: 9-11 (video/3D/SEO a menos que se pida explícitamente)`);
  } else if (hasPyProject) {
    console.log(`  📦 Perfil detectado: PYTHON`);
    console.log(`  → pyproject.toml o setup.py encontrado`);
    console.log(`\n  Capas recomendadas:`);
    console.log(`    1-8 (completas) | 12 (branding) | 13 (extras)`);
    console.log(`  Saltar: 9-11 (video/3D/SEO a menos que se pida explícitamente)`);
  } else if (hasPackageJson) {
    if (hasNextConfig) {
      console.log(`  📦 Perfil detectado: NEXT.JS`);
    } else if (hasVite) {
      console.log(`  📦 Perfil detectado: VITE + ${hasReact ? 'REACT' : 'VANILLA'}`);
    } else if (hasReact) {
      console.log(`  📦 Perfil detectado: REACT (SPA)`);
    } else {
      console.log(`  📦 Perfil detectado: NODE.JS`);
    }
    console.log(`\n  Capas recomendadas:`);
    console.log(`    1-8 (completas) | 9 (video, opcional) | 10 (3D, opcional)`);
    console.log(`    11 (SEO, recomendado) | 12 (visual review, recomendado) | 13 (extras)`);
  } else {
    console.log(`  📦 Perfil detectado: GENÉRICO`);
    console.log(`  → No se detectó package.json, Cargo.toml o pyproject.toml`);
    console.log(`\n  Capas recomendadas:`);
    console.log(`    1-8 (completas) | 12 (branding) | 13 (extras)`);
    console.log(`  Preguntar al usuario qué capas opcionales activar.`);
  }

  if (hasDockerfile) {
    console.log(`\n  🐳 Docker detectado — posible despliegue containerizado.`);
  }
}

// ── MAIN ──────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
  showHelp();
  process.exit(0);
}

const flags = {
  '--list-skills': showAllSkills,
  '--list-categories': showCategories,
  '--list-presets': showPresets,
  '--analyze-conflicts': showAnalyzeConflicts,
  '--validate-routes': validateRoutes,
  '--auto-profile': autoProfile,
};

let hasAction = false;
for (const [flag, fn] of Object.entries(flags)) {
  if (args.includes(flag)) {
    fn();
    hasAction = true;
  }
}

// --route and --preset with argument
const routeIdx = args.indexOf('--route');
if (routeIdx !== -1 && routeIdx + 1 < args.length) {
  showRoute(args[routeIdx + 1]);
  hasAction = true;
}
const presetIdx = args.indexOf('--preset');
if (presetIdx !== -1 && presetIdx + 1 < args.length) {
  showRoute(args[presetIdx + 1]);
  hasAction = true;
}

if (!hasAction) {
  showHelp();
  console.log(`\nSistema detectado: ${getAllSkills().length} skills disponibles.`);
}
