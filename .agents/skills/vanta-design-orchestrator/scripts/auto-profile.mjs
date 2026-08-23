#!/usr/bin/env node

/**
 * auto-profile.mjs — Auto-detection of project profile
 *
 * Detects the project type from the workspace root AND from
 * subdirectories (for monorepos where a web frontend lives
 * alongside a non-web root). This orchestrator is ONLY relevant
 * for web/frontend projects — if no web project is found it
 * reports that the orchestrator is not applicable.
 *
 * Usage:
 *   node scripts/auto-profile.mjs [--json]
 *   node scripts/auto-profile.mjs --recommend-layers
 */

import { readFileSync, existsSync, readdirSync, statSync } from 'fs';
import { resolve, dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ORCHESTRATOR_DIR = resolve(__dirname, '..');
const PROJECT_SKILLS_DIR = resolve(ORCHESTRATOR_DIR, '..', '..');
const WORKSPACE_ROOT = resolve(PROJECT_SKILLS_DIR, '..');

const LAYERS_DIR = resolve(ORCHESTRATOR_DIR, 'layers');

const LAYER_FILES = [
  { id: 'capa-1', file: 'capa-1-fundaciones.md', name: 'Fundaciones y Tokens', base: true },
  { id: 'capa-2', file: 'capa-2-estructura.md', name: 'Estructura y Usabilidad', base: true },
  { id: 'capa-3', file: 'capa-3-visual.md', name: 'Diseño Visual', base: true },
  { id: 'capa-4', file: 'capa-4-movimiento.md', name: 'Interacciones y Movimiento', base: true },
  { id: 'capa-5', file: 'capa-5-auditoria.md', name: 'Auditoría y Refinamiento', base: true },
  { id: 'capa-6', file: 'capa-6-rendimiento.md', name: 'Rendimiento y Optimización', base: true },
  { id: 'capa-7', file: 'capa-7-investigacion.md', name: 'Investigación', base: true },
  { id: 'capa-8', file: 'capa-8-operaciones.md', name: 'Operaciones', base: true },
  { id: 'capa-9', file: 'capa-9-video.md', name: 'Video', base: false },
  { id: 'capa-10', file: 'capa-10-3d.md', name: '3D Avanzado', base: false },
  { id: 'capa-11', file: 'capa-11-seo-mobile.md', name: 'SEO + Mobile', base: false },
  { id: 'capa-12-review', file: 'capa-12-visual-review.md', name: 'Visual Review', base: false },
  { id: 'capa-12-branding', file: 'capa-12-branding.md', name: 'Branding y Arte', base: true },
  { id: 'capa-13', file: 'capa-13-open-design.md', name: 'Open Design Skills', base: false },
];

// ── WEB FRONTEND DETECTION ──────────────────────────────────────────────────
// Look at root + one level deep for actual web projects.

const WEB_FRAMEWORK_SIGNALS = {
  next:   { dep: 'next' },
  react:  { dep: 'react' },
  vite:   { file: /^vite\.config/ },
  gatsby: { dep: 'gatsby' },
  astro:  { dep: 'astro' },
  nuxt:   { dep: 'nuxt' },
  svelte: { dep: 'svelte' },
  angular: { dep: '@angular/core' },
};

function scanPackageJson(dir) {
  const fp = resolve(dir, 'package.json');
  if (!existsSync(fp)) return null;
  try {
    return JSON.parse(readFileSync(fp, 'utf-8'));
  } catch {
    return null;
  }
}

function detectWebProject(dir) {
  const pkg = scanPackageJson(dir);
  if (!pkg) return null;

  const allDeps = { ...pkg.dependencies, ...pkg.devDependencies };
  const dirname = dir.split(/[\\/]/).pop();

  for (const [framework, signal] of Object.entries(WEB_FRAMEWORK_SIGNALS)) {
    if (signal.dep && allDeps[signal.dep]) {
      return { dir: dirname, framework, pkg };
    }
    if (signal.file) {
      try {
        const files = readdirSync(dir);
        if (files.some(f => signal.file.test(f))) {
          return { dir: dirname, framework, pkg };
        }
      } catch { /* not readable */ }
    }
  }

  // Check for index.html (static web)
  try {
    const files = readdirSync(dir);
    if (files.some(f => f === 'index.html' || f === 'index.htm')) {
      return { dir: dirname, framework: 'static-html', pkg };
    }
  } catch { /* not readable */ }

  return null;
}

function scanSubdirs() {
  const results = [];
  try {
    const entries = readdirSync(WORKSPACE_ROOT);
    for (const entry of entries) {
      const fullPath = join(WORKSPACE_ROOT, entry);
      try {
        if (statSync(fullPath).isDirectory() && !entry.startsWith('.')) {
          const web = detectWebProject(fullPath);
          if (web) results.push(web);
        }
      } catch { /* skip unreadable */ }
    }
  } catch { /* skip unreadable root */ }
  return results;
}

function detectProfile() {
  // 1. Check root first
  const rootWeb = detectWebProject(WORKSPACE_ROOT);
  const subdirWebs = scanSubdirs();

  const files = readdirSync(WORKSPACE_ROOT);
  const filesLower = files.map(f => f.toLowerCase());

  const hasCargoToml = files.includes('Cargo.toml');
  const hasPyProject = filesLower.some(f => f === 'pyproject.toml' || f === 'setup.py' || f === 'setup.cfg');
  const hasDockerfile = filesLower.some(f => f.startsWith('dockerfile') || f === 'docker-compose.yml');
  const hasTauri = filesLower.some(f => f.startsWith('tauri.conf'));

  let profile = 'generic';
  let framework = '';
  let details = [];
  let webSources = [];

  // ── WEB PROJECT (root or subdirectory) ──
  if (rootWeb) {
    webSources.push({ dir: '(root)', framework: rootWeb.framework });
    const pkg = rootWeb.pkg;

    if (pkg.dependencies?.next) {
      profile = 'nextjs';
      framework = 'Next.js';
    } else if (pkg.dependencies?.['@angular/core']) {
      profile = 'angular';
      framework = 'Angular';
    } else if (pkg.dependencies?.react) {
      // Distinguish React + Vite vs plain React
      try {
        const hasViteConfig = filesLower.some(f => f.startsWith('vite.config'));
        profile = hasViteConfig ? 'react-vite' : 'react';
        framework = hasViteConfig ? 'React + Vite' : 'React';
      } catch {
        profile = 'react';
        framework = 'React';
      }
    } else if (pkg.dependencies?.svelte) {
      profile = 'svelte';
      framework = 'Svelte';
    } else if (pkg.dependencies?.astro) {
      profile = 'astro';
      framework = 'Astro';
    } else if (pkg.dependencies?.gatsby) {
      profile = 'gatsby';
      framework = 'Gatsby';
    } else if (pkg.dependencies?.nuxt || pkg.dependencies?.nuxt3) {
      profile = 'nuxt';
      framework = 'Nuxt';
    } else {
      profile = 'node';
      framework = 'Node.js web';
    }
    details.push(`${framework} application (root)`);
  }

  // ── SUBDIRECTORY WEB PROJECTS ──
  for (const sw of subdirWebs) {
    // Don't double-count if root is already a web project
    if (rootWeb && sw.dir === files.find(f => f.toLowerCase() === sw.dir.toLowerCase())) continue;
    webSources.push(sw);
    details.push(`${sw.framework} frontend in ${sw.dir}/`);
  }

  // ── NON-WEB ROOT (Rust, Python, etc.) ──
  if (!rootWeb) {
    if (hasCargoToml) {
      profile = 'rust';
      details.push('Cargo workspace (root)');
      if (hasTauri) {
        profile = 'tauri';
        framework = 'Tauri';
        details.push('Tauri desktop app');
      }
    } else if (hasPyProject) {
      profile = 'python';
      details.push('Python project (root)');
    } else {
      details.push('No frontend detected at root');
    }

    if (hasDockerfile) details.push('Docker available');

    // If we found web sub-projects, override the profile
    if (subdirWebs.length > 0) {
      const primary = subdirWebs[0];
      profile = primary.framework === 'react' ? 'react-vite' : primary.framework;
      framework = primary.framework;
      // Keep the non-web root info in details but mark as web-detected
      details.unshift(`Web frontend detected in ${primary.dir}/`);
    }
  }

  const applicable = rootWeb !== null || subdirWebs.length > 0;

  return {
    profile,
    framework,
    details,
    hasDockerfile,
    hasTauri,
    applicable,
    webSources,
    rootIsWeb: rootWeb !== null,
  };
}

function recommendLayers(profile, applicable) {
  const baseIds = LAYER_FILES.filter(l => l.base).map(l => l.id);
  const optional = LAYER_FILES.filter(l => !l.base);

  // If no web frontend found anywhere, orchestrator is not applicable
  if (!applicable) {
    return {
      base: [],
      recommended: [],
      skipped: optional.map(l => l.id),
      message: 'This orchestrator is for web/frontend design only. No web project detected.',
    };
  }

  const recommended = [];
  const skipped = [];

  const webProfiles = ['nextjs', 'react', 'react-vite', 'vite', 'node', 'svelte', 'astro', 'gatsby', 'nuxt', 'angular', 'static-html'];

  for (const layer of optional) {
    if (webProfiles.includes(profile)) {
      // Web projects get all optional layers
      recommended.push(layer.id);
    } else {
      // Non-web root with web sub-project: recommend everything
      recommended.push(layer.id);
    }
  }

  return { base: baseIds, recommended, skipped, message: null };
}

// ── MAIN ──────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
const useJson = args.includes('--json');
const showLayers = args.includes('--recommend-layers');

const profile = detectProfile();
const layers = recommendLayers(profile.profile, profile.applicable);

if (useJson) {
  console.log(JSON.stringify({ profile, layers }, null, 2));
  process.exit(0);
}

console.log(`\n=== AUTO-PROFILE: Vanta Design Orchestrator ===\n`);

// Not applicable?
if (!profile.applicable) {
  console.log(`  ⛔ NO WEB FRONTEND DETECTED`);
  console.log(`     This orchestrator is for web/frontend design only.`);
  console.log(`     The project (${profile.profile}) has no web frontend at root or in subdirectories.\n`);
  if (profile.details.length) {
    console.log(`  ℹ️  What was found:`);
    for (const d of profile.details) console.log(`     ${d}`);
  }
  process.exit(1);
}

// Profile info
console.log(`  📦 Profile: ${profile.profile.toUpperCase()}${profile.framework ? ` (${profile.framework})` : ''}`);
for (const d of profile.details) console.log(`     ${d}`);
if (profile.hasDockerfile) console.log(`     🐳 Docker available`);
if (profile.hasTauri) console.log(`     🖥️  Tauri desktop`);

// Web sources
if (profile.webSources.length > 0) {
  console.log(`\n  🌐 Web frontend(s) found:`);
  for (const ws of profile.webSources) {
    console.log(`     ${ws.dir} → ${ws.framework}`);
  }
}

// Layers
console.log(`\n  📋 Layers:`);
console.log(`     Base (siempre): ${layers.base.join(', ')}`);
if (layers.recommended.length > 0) {
  console.log(`     Recomendadas: ${layers.recommended.join(', ')}`);
}
if (layers.skipped.length > 0) {
  console.log(`     Saltadas: ${layers.skipped.join(', ')}`);
}

if (showLayers) {
  console.log(`\n  📄 Layer files:`);
  for (const layer of LAYER_FILES) {
    const fp = resolve(LAYERS_DIR, layer.file);
    const exists = existsSync(fp);
    const status = exists ? '✅' : '❌';
    const type = layer.base ? 'base' : 'opt';
    console.log(`     ${status} [${type}] ${layer.file} — ${layer.name}`);
  }
}

// Installed layers check
let missing = 0;
for (const layer of LAYER_FILES) {
  if (!existsSync(resolve(LAYERS_DIR, layer.file))) {
    if (missing === 0) console.log(`\n  ⚠ Archivos de capa faltantes:`);
    console.log(`     ${layer.file}`);
    missing++;
  }
}

if (missing === 0) {
  console.log(`\n  ✅ Todos los archivos de capa están presentes.`);
}
