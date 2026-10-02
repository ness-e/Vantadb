// Independent probe: feed check-secrets.mjs known-bad strings and confirm it
// actually fires. The corpus measures zero hits, which means a clean run proves
// nothing on its own -- this is the check that proves the rules work.
import { writeFileSync, mkdtempSync, rmSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { segment } from './lib.mjs';

const dir = mkdtempSync(join(tmpdir(), 'secret-probe-'));
const fixture = join(dir, 'probe.md');
const body = [
  '---',
  'title: probe',
  '---',
  '',
  '# Probe',
  '',
  'Key AKIAIOSFODNN7EXAMPLE here.',
  'Token ghp_0123456789abcdefghijklmnopqrstuvwxyz here.',
  'Key AIzaSyA1234567890abcdefghijklmnopqrstuv here.',
  'Token xoxb-123456789012-1234567890123-abcdefghijklmnopqrstuvwx here.',
  'Key sk_live_abcdefghijklmnopqrstuvwx here.',
  'Password = aB3xY7zQ1mN9pL2kJ here.',
  'Host https://wiki.corp.example here.',
  '',
  'A safe line mentioning `AKIA` inside code only.',
  '',
].join('\n');
writeFileSync(fixture, body, 'utf8');

// Diagnostic: what does segment() make of the fixture?
const segs = segment(body);
const prose = segs.filter((s) => s.why === 'prose');
console.log('segments      :', segs.length, '| prose segments:', prose.length);
console.log('round-trip ok :', segs.map((s) => s.text).join('') === body);
const awsProse = prose.filter((s) => /\b(?:AKIA|ASIA)[0-9A-Z]{16}\b/.test(s.text));
console.log('AWS shape in prose segments:', awsProse.length);

let out = '';
try {
  out = execFileSync('node', ['scripts/docs/check-secrets.mjs', '--json', `--root=${dir}`], {
    encoding: 'utf8',
    maxBuffer: 1 << 26,
  });
} catch (e) {
  out = e.stdout ?? '';
  console.log('scanner exited', e.status);
}
console.log('raw scanner output starts:', JSON.stringify(out.slice(0, 120)));

rmSync(dir, { recursive: true, force: true });

let r;
try {
  r = JSON.parse(out);
} catch {
  console.log('\nRESULT: scanner did not emit JSON — the probe cannot conclude.');
  process.exit(2);
}

const fired = new Set((r.findings ?? []).map((f) => f.rule));
const want = [
  'aws-access-key-id',
  'github-token',
  'google-api-key',
  'slack-token',
  'stripe-live-key',
  'credential-assignment',
  'internal-hostname',
];
const missing = want.filter((w) => !fired.has(w));
console.log('filesScanned  :', r.filesScanned);
console.log('rules that fired:', [...fired].sort().join(', ') || '(none)');
console.log(missing.length ? `\nMISSING: ${missing.join(', ')}` : '\nall expected rules fired.');
process.exit(missing.length ? 1 : 0);
