#!/usr/bin/env node
// scripts/docs/sync-workspace-version.mjs
//
// PURPOSE (FIND-379, FIND-230)
//   Synchronize all version-coupled manifests and documentation across the
//   monorepo to match `[workspace.package] version` from the root Cargo.toml.
//
// TRACKED TARGETS
//   1. vantadb-ts/package.json: top-level "version"
//   2. server.json: top-level "version" (preserves 2-space formatting + trailing newline)
//   3. docs/api/openapi.yaml: `  version: "<version>"` under `info:`
//   4. docs/api/MCP.md: `Current MCP implementation version: <version>`
//   5. All 9 integrations/*/pyproject.toml:
//      dependency upper bound "vantadb-py>=X.Y.Z,<0.N.0" -> "<0.{minor+1}.0"
//   6. docs/api/SURFACE_MANIFEST.md: header "workspace version: <version>"
//      (and regenerates via gen-surface-manifest.mjs --write when --write is set)
//   7. release-manifest.json: "version" and "tag" (FIND-342: single source of
//      truth for post-release verification - its own comment says to bump
//      version/tag with each core release)
//   8. vantadb-ts/package-lock.json: root "version" (2 places: root and
//      packages[""]). The lockfile ships inside the published npm tarball.
//
// USAGE / EXIT CODES
//   node scripts/docs/sync-workspace-version.mjs [--check]  (default)
//     0 = all manifests match Cargo.toml workspace version
//     1 = drift detected (lists out-of-sync files)
//   node scripts/docs/sync-workspace-version.mjs --write
//     updates all drifted manifests on disk; exits 0
//   node scripts/docs/sync-workspace-version.mjs --self-test
//     in-memory assertions on version extraction & transformation logic; exits 0
//   2 = unrecoverable error (e.g. root Cargo.toml unreadable)

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import { ROOT, abs } from './lib.mjs';

export const INTEGRATION_PACKAGES = [
  'crewai',
  'dspy',
  'haystack',
  'langchain',
  'letta',
  'llamaindex',
  'mem0',
  'ollama',
  'openai',
];

// ------------------------------------------------------------------ parsers

export function parseWorkspaceVersion(cargoToml) {
  const lines = String(cargoToml).split(/\r?\n/);
  const start = lines.findIndex((l) => l.trim() === '[workspace.package]');
  if (start === -1) return null;
  for (let i = start + 1; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.startsWith('[')) break;
    const m = line.match(/^version\s*=\s*"([^"]+)"/);
    if (m) return m[1];
  }
  return null;
}

export function parsePackageVersion(text) {
  try {
    const v = JSON.parse(text)?.version;
    return typeof v === 'string' && v.length > 0 ? v : null;
  } catch {
    return null;
  }
}

export function parseOpenApiVersion(text) {
  const m = String(text).match(/^  version:\s*"([^"]+)"/m);
  return m ? m[1] : null;
}

export function parseMcpVersion(text) {
  const m = String(text).match(/Current MCP implementation version:\s*`?v?([^\s`\r\n]+)`?/);
  return m ? m[1] : null;
}

export function parsePyprojectUpperBound(text) {
  const m = String(text).match(/['"]vantadb-py>=[^,'"]+,(<0\.\d+\.0)['"]/);
  return m ? m[1] : null;
}

export function parseSurfaceManifestVersion(text) {
  const m = String(text).match(/workspace version:\s*([^\s*)]+)/);
  return m ? m[1] : null;
}

export function getPyprojectExpectedBound(newVersion) {
  const [majorStr, minorStr] = newVersion.split('.');
  const major = parseInt(majorStr, 10);
  const minor = parseInt(minorStr, 10);
  const nextMinor = minor + 1;
  return major === 0 ? `<0.${nextMinor}.0` : `<${major + 1}.0.0`;
}

// -------------------------------------------------------------- transformers

export function transformPackageJson(text, newVersion) {
  return text.replace(/^(\s*"version":\s*)"[^"]+"/m, `$1"${newVersion}"`);
}

export function transformServerJson(text, newVersion) {
  try {
    const parsed = JSON.parse(text);
    parsed.version = newVersion;
    return JSON.stringify(parsed, null, 2) + '\n';
  } catch {
    return text.replace(/^(\s*"version":\s*)"[^"]+"/m, `$1"${newVersion}"`);
  }
}

export function transformOpenApiYaml(text, newVersion) {
  return text.replace(/^(\s*version:\s*)"[^"]*"/m, `$1"${newVersion}"`);
}

export function transformMcpMd(text, newVersion) {
  return text.replace(
    /(Current MCP implementation version:\s*)(`?v?)([\d.]+)(`?)/,
    (match, prefix, vPrefix, oldVer, backtick) => `${prefix}${vPrefix}${newVersion}${backtick}`,
  );
}

export const PROVIDER_PACKAGES = ['openai', 'ollama', 'litellm'];

export function parsePyprojectVersionAndBound(text) {
  const mVer = String(text).match(/^\[project\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  const mBound = String(text).match(/['"]vantadb-py>=[^,'"]+,(<0\.\d+\.0)['"]/);
  if (!mVer || !mBound) return null;
  return `${mVer[1]} (${mBound[1]})`;
}

export function transformPyprojectToml(text, newVersion) {
  const newBound = getPyprojectExpectedBound(newVersion);
  return String(text)
    .replace(/^(\[project\][\s\S]*?^version\s*=\s*)"[^"]+"/m, `$1"${newVersion}"`)
    .replace(/(['"]vantadb-py>=[^,'"]+,)<0\.\d+\.0(['"])/g, `$1${newBound}$2`);
}

export function parseCargoPkgVersion(text) {
  const mPkg = String(text).match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  return mPkg ? mPkg[1] : null;
}

export function transformCargoPkgToml(text, newVersion) {
  return String(text).replace(/^(\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m, `$1"${newVersion}"`);
}

export function transformSurfaceManifest(text, newVersion) {
  return text.replace(/(workspace version:\s*)[^\s*)]+/g, `$1${newVersion}`);
}

// release-manifest.json (FIND-342) es la fuente de verdad de la verificacion
// post-release: su propio comentario dice "Bump version/tag with each core
// release". Sin esto, el tag v0.9.0 llevaba un manifiesto describiendo la
// v0.8.0.
export function parseReleaseManifestVersionAndTag(text) {
  const mVer = String(text).match(/^\s*"version":\s*"([^"]+)"/m);
  const mTag = String(text).match(/^\s*"tag":\s*"([^"]+)"/m);
  if (!mVer || !mTag) return null;
  return `${mVer[1]} (${mTag[1]})`;
}

export function transformReleaseManifest(text, newVersion) {
  return String(text)
    .replace(/^(\s*"version":\s*)"[^"]+"/m, `$1"${newVersion}"`)
    .replace(/^(\s*"tag":\s*)"v[^"]+"/m, `$1"v${newVersion}"`);
}

// El lockfile de npm viaja dentro del tarball publicado: su "version" raiz
// (2 sitios: raiz y packages[""]) debe coincidir con package.json. npm ci
// tolera el desajuste y ningun gate lo comprueba, pero publicar un
// lockfile que dice 0.8.0 en un release 0.9.0 es drift.
export function parseNodeLockVersion(text) {
  const m = String(text).match(/^\s*"version":\s*"([^"]+)"/m);
  return m ? m[1] : null;
}

export function transformNodeLockJson(text, newVersion) {
  let seen = 0;
  return String(text).replace(/^(\s*)"version":\s*"[^"]+"/gm, (match, indent) => {
    seen += 1;
    // Solo los 2 primeros: raiz y packages[""]. A partir de ahi son las
    // versiones de las dependencias.
    return seen <= 2 ? `${indent}"version": "${newVersion}"` : match;
  });
}

export function parseNodeCargoVersion(text) {
  const mPkg = String(text).match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  const mDep = String(text).match(/^vanta-memory\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/m);
  if (!mPkg || !mDep) return null;
  return mPkg[1] === mDep[1] ? mPkg[1] : `${mPkg[1]} (vanta-memory=${mDep[1]})`;
}

export function transformNodeCargoToml(text, newVersion) {
  return String(text)
    .replace(/^(\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m, `$1"${newVersion}"`)
    .replace(/^(vanta-memory\s*=\s*\{[^}]*version\s*=\s*)"[^"]+"/m, `$1"${newVersion}"`);
}

// vanta-memory depende del crate raiz del workspace con un requisito de
// version literal: `vantadb = { path = "../", version = "0.9.0" }`. Ese
// requisito tiene que seguir a la version del workspace, porque el path dep
// local debe SATISFACERLO: si el workspace pasa a 0.10.0 y el requisito
// sigue en 0.9.0, cargo no resuelve el workspace entero
// ("failed to select a version for the requirement `vantadb = ^0.9.0`").
// release-plz no lo actualiza, porque vanta-memory esta en hold
// (`release = false`) y por tanto no se procesa.
export function parseVantaMemoryDepRequirement(text) {
  const m = String(text).match(/^vantadb\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/m);
  return m ? m[1] : null;
}

export function transformVantaMemoryDepRequirement(text, newVersion) {
  return String(text).replace(
    /^(vantadb\s*=\s*\{[^}]*version\s*=\s*)"[^"]+"/m,
    `$1"${newVersion}"`,
  );
}

// --------------------------------------------------------------- evaluation

export function buildTargetSpecs(workspaceVersion) {
  const expectedBound = getPyprojectExpectedBound(workspaceVersion);
  const specs = [
    {
      name: 'vantadb-ts/package.json',
      relPath: 'vantadb-ts/package.json',
      expected: workspaceVersion,
      parse: parsePackageVersion,
      transform: transformPackageJson,
    },
    {
      name: 'vantadb-node/package.json',
      relPath: 'vantadb-node/package.json',
      expected: workspaceVersion,
      parse: parsePackageVersion,
      transform: transformPackageJson,
    },
    {
      name: 'vantadb-node/Cargo.toml',
      relPath: 'vantadb-node/Cargo.toml',
      expected: workspaceVersion,
      parse: parseNodeCargoVersion,
      transform: transformNodeCargoToml,
    },
    {
      name: 'server.json',
      relPath: 'server.json',
      expected: workspaceVersion,
      parse: parsePackageVersion,
      transform: transformServerJson,
    },
    {
      name: 'docs/api/openapi.yaml',
      relPath: 'docs/api/openapi.yaml',
      expected: workspaceVersion,
      parse: parseOpenApiVersion,
      transform: transformOpenApiYaml,
    },
    {
      name: 'docs/api/MCP.md',
      relPath: 'docs/api/MCP.md',
      expected: workspaceVersion,
      parse: parseMcpVersion,
      transform: transformMcpMd,
    },
  ];

  for (const pkg of INTEGRATION_PACKAGES) {
    specs.push({
      name: `integrations/${pkg}/pyproject.toml`,
      relPath: `integrations/${pkg}/pyproject.toml`,
      expected: `${workspaceVersion} (${expectedBound})`,
      parse: parsePyprojectVersionAndBound,
      transform: transformPyprojectToml,
    });
  }

  for (const prov of PROVIDER_PACKAGES) {
    specs.push({
      name: `providers/${prov}/Cargo.toml`,
      relPath: `providers/${prov}/Cargo.toml`,
      expected: workspaceVersion,
      parse: parseCargoPkgVersion,
      transform: transformCargoPkgToml,
    });
  }

  specs.push({
    name: 'docs/api/SURFACE_MANIFEST.md',
    relPath: 'docs/api/SURFACE_MANIFEST.md',
    expected: workspaceVersion,
    parse: parseSurfaceManifestVersion,
    transform: transformSurfaceManifest,
  });

  specs.push({
    name: 'release-manifest.json',
    relPath: 'release-manifest.json',
    expected: `${workspaceVersion} (v${workspaceVersion})`,
    parse: parseReleaseManifestVersionAndTag,
    transform: transformReleaseManifest,
  });

  specs.push({
    name: 'vantadb-ts/package-lock.json',
    relPath: 'vantadb-ts/package-lock.json',
    expected: workspaceVersion,
    parse: parseNodeLockVersion,
    transform: transformNodeLockJson,
  });

  specs.push({
    name: 'vanta-memory/Cargo.toml',
    relPath: 'vanta-memory/Cargo.toml',
    expected: workspaceVersion,
    parse: parseVantaMemoryDepRequirement,
    transform: transformVantaMemoryDepRequirement,
  });

  return specs;
}

// ---------------------------------------------------------------- self-test

export function selfTest() {
  const results = [];
  const check = (name, fn) => {
    try {
      fn();
      results.push({ name, ok: true });
    } catch (e) {
      results.push({ name, ok: false, error: e.message });
    }
  };
  const assert = (cond, msg) => {
    if (!cond) throw new Error(msg);
  };

  // 1. Cargo.toml workspace version extraction
  check('parseWorkspaceVersion: extracts [workspace.package] version', () => {
    const toml = '[package]\nname = "foo"\nversion = "9.9.9"\n[workspace.package]\nversion = "0.8.0"\n';
    assert(parseWorkspaceVersion(toml) === '0.8.0', `got ${parseWorkspaceVersion(toml)}`);
  });

  // 2. package.json extraction & transform
  check('package.json: transforms version 0.8.0 -> 0.9.0', () => {
    const fixture = '{\n  "name": "vantadb",\n  "version": "0.8.0",\n  "type": "module"\n}\n';
    const out = transformPackageJson(fixture, '0.9.0');
    assert(parsePackageVersion(out) === '0.9.0', `parse failed, got ${parsePackageVersion(out)}`);
    assert(out.includes('"version": "0.9.0"'), 'missing version string');
  });

  // 3. server.json extraction & transform (preserving 2-space formatting + newline)
  check('server.json: transforms version 0.8.0 -> 0.9.0 preserving formatting', () => {
    const fixture = '{\n  "name": "vantadb",\n  "version": "0.8.0"\n}\n';
    const out = transformServerJson(fixture, '0.9.0');
    assert(parsePackageVersion(out) === '0.9.0', `parse failed: ${out}`);
    assert(out.endsWith('\n'), 'must end with newline');
    assert(out === '{\n  "name": "vantadb",\n  "version": "0.9.0"\n}\n', 'exact formatting match');
  });

  // 4. openapi.yaml extraction & transform
  check('openapi.yaml: transforms info.version 0.8.0 -> 0.9.0', () => {
    const fixture = 'openapi: "3.1.0"\ninfo:\n  title: VantaDB\n  version: "0.8.0"\n';
    const out = transformOpenApiYaml(fixture, '0.9.0');
    assert(parseOpenApiVersion(out) === '0.9.0', `got ${parseOpenApiVersion(out)}`);
    assert(out.includes('  version: "0.9.0"'), 'expected indented version');
  });

  // 5. MCP.md extraction & transform (bare version and `v...` formatted version)
  check('MCP.md: transforms implementation version 0.8.0 -> 0.9.0 (bare)', () => {
    const fixture = '# Header\n\nCurrent MCP implementation version: 0.8.0\n\nBody';
    const out = transformMcpMd(fixture, '0.9.0');
    assert(parseMcpVersion(out) === '0.9.0', `got ${parseMcpVersion(out)}`);
    assert(out.includes('Current MCP implementation version: 0.9.0'), `output mismatch: ${out}`);
  });

  check('MCP.md: transforms implementation version `v0.8.0` -> `v0.9.0`', () => {
    const fixture = '# Header\n\nCurrent MCP implementation version: `v0.8.0`\n\nBody';
    const out = transformMcpMd(fixture, '0.9.0');
    assert(parseMcpVersion(out) === '0.9.0', `got ${parseMcpVersion(out)}`);
    assert(out.includes('Current MCP implementation version: `v0.9.0`'), `output mismatch: ${out}`);
  });

  // 6. pyproject.toml upper bound transform: <0.9.0 -> <0.10.0 for 0.9.0
  check('pyproject.toml: transforms vantadb-py upper bound <0.9.0 -> <0.10.0', () => {
    const fixture = 'dependencies = [\n    "vantadb-py>=0.6.1,<0.9.0",\n    "dspy>=2.6,<3",\n]\n';
    const out = transformPyprojectToml(fixture, '0.9.0');
    assert(parsePyprojectUpperBound(out) === '<0.10.0', `got ${parsePyprojectUpperBound(out)}`);
    assert(out.includes('"vantadb-py>=0.6.1,<0.10.0"'), `output mismatch: ${out}`);
  });

  check('pyproject.toml: preserves lower bound 0.5.0 when updating upper bound', () => {
    const fixture = 'dependencies = [\n    "vantadb-py>=0.5.0,<0.9.0",\n    "openai>=1.0,<2",\n]\n';
    const out = transformPyprojectToml(fixture, '0.9.0');
    assert(parsePyprojectUpperBound(out) === '<0.10.0', `got ${parsePyprojectUpperBound(out)}`);
    assert(out.includes('"vantadb-py>=0.5.0,<0.10.0"'), `output mismatch: ${out}`);
  });

  // 7. SURFACE_MANIFEST.md header transform
  check('SURFACE_MANIFEST.md: transforms header workspace version', () => {
    const fixture = '> **workspace version: 0.8.0** (source: root `Cargo.toml`).\n';
    const out = transformSurfaceManifest(fixture, '0.9.0');
    assert(parseSurfaceManifestVersion(out) === '0.9.0', `got ${parseSurfaceManifestVersion(out)}`);
    assert(out.includes('workspace version: 0.9.0'), `output mismatch: ${out}`);
  });

  // 8. release-manifest.json: version + tag
  check('release-manifest.json: transforms version and tag together', () => {
    const fixture = '{\n  "version": "0.8.0",\n  "tag": "v0.8.0",\n  "github": {}\n}\n';
    const out = transformReleaseManifest(fixture, '0.9.0');
    assert(parseReleaseManifestVersionAndTag(out) === '0.9.0 (v0.9.0)', `got ${parseReleaseManifestVersionAndTag(out)}`);
    assert(out.includes('"tag": "v0.9.0"'), 'tag not bumped');
  });

  // 9. vantadb-ts/package-lock.json: solo raiz y packages[""], no las deps
  check('package-lock.json: bumps root version twice, leaves dependency versions', () => {
    const fixture = '{\n  "name": "vantadb",\n  "version": "0.8.0",\n  "packages": {\n    "": {\n      "name": "vantadb",\n      "version": "0.8.0"\n    },\n    "node_modules/dep": {\n      "version": "0.8.0"\n    }\n  }\n}\n';
    const out = transformNodeLockJson(fixture, '0.9.0');
    assert(parseNodeLockVersion(out) === '0.9.0', `got ${parseNodeLockVersion(out)}`);
    assert(out.includes('"version": "0.9.0",\n  "packages"'), 'root version not bumped');
    assert(out.includes('"version": "0.9.0"\n    }'), 'packages[""] version not bumped');
    assert(out.includes('"node_modules/dep": {\n      "version": "0.8.0"'), 'dependency version was touched');
  });

  // 10. vanta-memory: el requisito del path dep debe seguir al workspace
  check('vanta-memory/Cargo.toml: bumps the vantadb path-dep requirement', () => {
    const fixture = '[package]\nname = "vanta-memory"\nversion.workspace = true\n\n[dependencies]\nvantadb = { path = "../", version = "0.8.0", default-features = false }\n';
    const out = transformVantaMemoryDepRequirement(fixture, '0.9.0');
    assert(parseVantaMemoryDepRequirement(out) === '0.9.0', `got ${parseVantaMemoryDepRequirement(out)}`);
    assert(out.includes('version = "0.9.0"'), `output mismatch: ${out}`);
    assert(out.includes('path = "../"'), 'path dep lost');
  });

  const failed = results.filter((r) => !r.ok);
  for (const r of results) {
    console.log(`${r.ok ? 'ok  ' : 'FAIL'}  ${r.name}${r.ok ? '' : ` -- ${r.error}`}`);
  }
  console.log(`\n${results.length - failed.length}/${results.length} assertions passed${failed.length ? ` — ${failed.length} FAILED` : ''}`);
  return failed.length === 0;
}

// --------------------------------------------------------------------- main

const ARGS = process.argv.slice(2);

if (ARGS.includes('--help')) {
  console.log([
    'sync-workspace-version.mjs — synchronize monorepo manifests to Cargo.toml workspace version',
    '',
    '  --check       (default) verify all tracked manifests match workspace version; exit 1 on drift',
    '  --write       write synchronized versions to disk and regenerate surface manifest',
    '  --self-test   run in-memory transformation assertions and exit 0',
    '  --help        display this help text',
    '',
    'exit codes: 0 ok · 1 drift detected / assertions failed · 2 evaluation error',
  ].join('\n'));
  process.exit(0);
}

if (ARGS.includes('--self-test')) {
  process.exit(selfTest() ? 0 : 1);
}

const writeMode = ARGS.includes('--write');

let cargoToml;
try {
  cargoToml = readFileSync(abs('Cargo.toml'), 'utf8');
} catch (e) {
  console.error(`ERROR: unable to read Cargo.toml from ${ROOT}: ${e.message}`);
  process.exit(2);
}

const workspaceVersion = parseWorkspaceVersion(cargoToml);
if (!workspaceVersion) {
  console.error('ERROR: failed to extract [workspace.package] version from root Cargo.toml');
  process.exit(2);
}

console.log(`Workspace version: ${workspaceVersion} (from Cargo.toml)`);
console.log(`Mode: ${writeMode ? 'WRITE (sync to disk)' : 'CHECK (read-only audit)'}\n`);

const specs = buildTargetSpecs(workspaceVersion);
let driftedCount = 0;
let updatedCount = 0;

for (const spec of specs) {
  const fullPath = abs(spec.relPath);
  let content;
  try {
    content = readFileSync(fullPath, 'utf8');
  } catch (e) {
    console.error(`FAIL: ${spec.name}: unable to read file (${e.message})`);
    driftedCount++;
    continue;
  }

  const found = spec.parse(content);
  const match = found === spec.expected;

  if (match) {
    console.log(`ok   ${spec.name}: ${found}`);
  } else {
    driftedCount++;
    console.log(`DRIFT ${spec.name}: found '${found ?? '(unreadable)'}', expected '${spec.expected}'`);

    if (writeMode) {
      const transformed = spec.transform(content, workspaceVersion);
      try {
        writeFileSync(fullPath, transformed, 'utf8');
        console.log(`  -> updated ${spec.name} on disk`);
        updatedCount++;
      } catch (e) {
        console.error(`  -> ERROR writing ${spec.name}: ${e.message}`);
      }
    }
  }
}

if (writeMode) {
  // Regenerate SURFACE_MANIFEST.md via gen-surface-manifest.mjs --write to ensure full table sync
  const genSurfaceScript = abs('scripts/docs/gen-surface-manifest.mjs');
  if (existsSync(genSurfaceScript)) {
    try {
      execFileSync(process.execPath, [genSurfaceScript, '--write'], { stdio: 'inherit' });
      console.log('Regenerated docs/api/SURFACE_MANIFEST.md via gen-surface-manifest.mjs --write');
    } catch (e) {
      console.warn(`Warning: gen-surface-manifest.mjs --write invocation failed: ${e.message}`);
    }
  }

  // Regenerate canonical documentation indexes and llms.txt via gen-index.mjs --write
  const genIndexScript = abs('scripts/docs/gen-index.mjs');
  if (existsSync(genIndexScript)) {
    try {
      execFileSync(process.execPath, [genIndexScript, '--write'], { stdio: 'inherit' });
      console.log('Regenerated docs/index.md and llms.txt via gen-index.mjs --write');
    } catch (e) {
      console.warn(`Warning: gen-index.mjs --write invocation failed: ${e.message}`);
    }
  }

  console.log(`\nSync complete: ${updatedCount} file(s) updated.`);
  process.exit(0);
}

if (driftedCount > 0) {
  console.error(`\nFAILED: ${driftedCount} target(s) drifted from workspace version ${workspaceVersion}.`);
  console.error('Run `node scripts/docs/sync-workspace-version.mjs --write` to synchronize all manifests.');
  process.exit(1);
}

console.log(`\nOK: All ${specs.length} manifests synchronized with workspace version ${workspaceVersion}.`);
process.exit(0);
