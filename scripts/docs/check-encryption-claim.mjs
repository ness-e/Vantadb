#!/usr/bin/env node
// scripts/docs/check-encryption-claim.mjs
//
// PURPOSE
//   Fail when the encryption-at-rest claim in the published docs no longer matches
//   the real state of the code. Two directions, both of them real failures:
//
//   1. NOT WIRED, but the docs claim encryption at rest  -> the P0 of the 0.9.0 audit
//      (FIND-407 / D-01): `SECURITY.md` published "AES-256-GCM at-rest encryption"
//      while the only call site of `with_cipher` in `src/` was a `#[test]`.
//   2. WIRED, but the docs still say "not encrypted at rest yet"  -> the inverse
//      claim (risk R-12 of ADR-0057): the wording would keep denying a capability
//      the product now has.
//
// WHY THIS EXISTS
//   The wording was corrected by hand in `c0e63d4e` (2026-10-20). A hand-corrected
//   security claim has no mechanical protection: the next edit that "simplifies" the
//   sentence brings the false claim back, and nobody notices until a user reads the
//   security policy. ADR-0057 (accepted 2026-10-20) makes the claim TRUE by wiring
//   the feature; this gate is what keeps the docs honest in BOTH directions.
//
// WHERE IT RUNS
//   Locally:  node scripts/docs/check-encryption-claim.mjs
//   CI:       docs-gates.yml, job `docs-index` (step "encryption claim matches the wired state")
//
// USAGE / EXIT CODES
//   node scripts/docs/check-encryption-claim.mjs [--json] [--self-test]
//   0 = the claim matches the wired state
//   1 = GATE FAILED - the claim and the code disagree
//   2 = cannot evaluate (files unreadable) - NOT a pass

import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = process.cwd();

// The files that carry the claim. They must agree with each other AND with
// `docs/user/operations/CONFIGURATION.md` (the `encryption_key` row).
const CLAIM_FILES = ['SECURITY.md', 'src/crypto.rs'];
const REFERENCE_DOC = 'docs/user/operations/CONFIGURATION.md';

// How the code says "the storage files are still plaintext". Any of these
// qualifiers makes the claim honest; none of them -> the claim is unqualified.
const NEGATIVE_QUALIFIERS = [
  'not encrypted at rest yet',
  'are not encrypted at rest',
  'not encrypted at rest',
  'remain plaintext',
  'stay plaintext',
  'still plaintext',
  'siguen en claro',
  'no estan cifrados en reposo',
  'no está cifrado en reposo',
  'not yet applied to core storage',
];

// The marker the implementation (ADR-0057, wave W6) must flip when the wiring
// is complete. Absent = not wired.
const WIRED_MARKER = 'ENCRYPTION_AT_REST_WIRED';
const WIRED_TRUE = /ENCRYPTION_AT_REST_WIRED\s*(?::\s*bool\s*)?=\s*true/;

function read(rel) {
  const p = join(ROOT, rel);
  if (!existsSync(p)) return null;
  return readFileSync(p, 'utf8');
}

/** Is the feature actually wired into the core storage write paths? */
function wiredState() {
  // The marker may live in src/crypto.rs or in the registry module next to it.
  const candidates = ['src/crypto.rs', 'src/crypto/registry.rs', 'src/crypto/mod.rs'];
  for (const rel of candidates) {
    const t = read(rel);
    if (t && WIRED_TRUE.test(t)) return { wired: true, markerIn: rel };
  }
  for (const rel of candidates) {
    const t = read(rel);
    if (t && t.includes(WIRED_MARKER)) return { wired: false, markerIn: rel };
  }
  return { wired: false, markerIn: null };
}

/** Does the text carry an honest negative qualifier about the core storage files? */
function isHonest(text) {
  const lower = text.toLowerCase();
  return NEGATIVE_QUALIFIERS.some((q) => lower.includes(q.toLowerCase()));
}

function evaluate() {
  const problems = [];
  const files = {};
  for (const rel of CLAIM_FILES) {
    const t = read(rel);
    if (t === null) {
      problems.push({ file: rel, reason: 'unreadable' });
      continue;
    }
    files[rel] = { honest: isHonest(t) };
  }
  const ref = read(REFERENCE_DOC);
  if (ref === null) {
    problems.push({ file: REFERENCE_DOC, reason: 'unreadable' });
  } else {
    files[REFERENCE_DOC] = { honest: isHonest(ref) };
  }

  const { wired, markerIn } = wiredState();

  if (problems.length) return { ok: false, wired, markerIn, files, problems, claim: 'unreadable' };

  const unqualified = Object.entries(files)
    .filter(([, v]) => !v.honest)
    .map(([k]) => k);

  if (!wired && unqualified.length) {
    return {
      ok: false,
      wired,
      markerIn,
      files,
      problems: [],
      claim: 'unqualified-while-not-wired',
      detail: unqualified,
    };
  }
  if (wired && files['SECURITY.md'].honest) {
    return {
      ok: false,
      wired,
      markerIn,
      files,
      problems: [],
      claim: 'stale-negative-while-wired',
      detail: ['SECURITY.md'],
    };
  }
  return { ok: true, wired, markerIn, files, problems: [], claim: 'consistent' };
}

function selfTest() {
  const cases = [
    { text: 'the core storage files **are not encrypted at rest yet**', expect: true },
    { text: 'WAL/HNSW/VantaFile/backend-KV/snapshots remain plaintext today', expect: true },
    { text: '- **AES-256-GCM at-rest encryption** behind the `encryption` feature flag.', expect: false },
    { text: 'the core storage files are not encrypted at rest yet (FIND-249)', expect: true },
  ];
  let fail = 0;
  for (const c of cases) {
    const got = isHonest(c.text);
    if (got !== c.expect) {
      fail++;
      console.error(`  FAIL expected honest=${c.expect} got=${got}: ${c.text.slice(0, 60)}`);
    }
  }
  console.log(`  self-test: ${cases.length - fail}/${cases.length}`);
  return fail === 0;
}

const args = process.argv.slice(2);
if (args.includes('--self-test')) {
  process.exit(selfTest() ? 0 : 1);
}

const res = evaluate();

if (args.includes('--json')) {
  console.log(JSON.stringify(res, null, 2));
  process.exit(res.ok ? 0 : 1);
}

if (res.ok) {
  console.log(
    `encryption claim: OK (${res.wired ? 'wired - la doc describe la capacidad real' : 'not wired - la doc dice que el storage sigue en claro'})`,
  );
  process.exit(0);
}

if (res.claim === 'unreadable') {
  console.error('encryption claim: GATE FAILED - no se pudo leer: ' + res.problems.map((p) => p.file).join(', '));
  process.exit(2);
}
if (res.claim === 'unqualified-while-not-wired') {
  console.error(
    'encryption claim: GATE FAILED - el cifrado en reposo NO esta cableado, pero estos ficheros\n' +
      'afirman cifrado en reposo sin el matiz que lo desmiente (P0 FIND-407):\n' +
      res.detail.map((f) => `  - ${f}`).join('\n') +
      '\n  Redaccion honesta de referencia: docs/user/operations/CONFIGURATION.md (encryption_key)',
  );
  process.exit(1);
}
console.error(
  'encryption claim: GATE FAILED - el cifrado en reposo YA esta cableado (' +
    (res.markerIn ?? 'marcador') +
    ') pero la doc sigue diciendo que no lo esta (claim falsa inversa, riesgo R-12 de ADR-0057):\n' +
    res.detail.map((f) => `  - ${f}`).join('\n') +
    '\n  Enmenda la doc hacia delante: describe la capacidad real y su alcance exacto (ADR-0057 §5.3).',
);
process.exit(1);
