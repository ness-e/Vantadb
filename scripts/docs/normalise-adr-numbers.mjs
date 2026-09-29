// Normalise ADR numbers in prose to the four-digit form used by the filenames.
//
// After the 2026-09-29 rename every ADR file is `ADR-NNNN-kebab-case.md`, but
// 182 prose references still said `ADR-046` while 1159 said `ADR-0046`. One ADR
// written two ways in one corpus is the same ambiguity class that produced the
// 041 collision: a reader cannot tell whether `ADR-041` and `ADR-0041` are the
// same document.
//
// Only the NUMBER is rewritten, and only where it is a standalone token -- a
// bare `ADR-046`, not part of a path (`ADR-0046-schema-v2.md` already correct),
// not part of a longer identifier, and not inside a fenced code block.
//
//   node scripts/docs/normalise-adr-numbers.mjs --self-test
//   node scripts/docs/normalise-adr-numbers.mjs [--write]
import { readFileSync, writeFileSync } from 'node:fs';
import { listDocs } from './lib.mjs';

const WRITE = process.argv.includes('--write');
const FENCE = /^\s*(```|~~~)/;

// A three-digit ADR number standing alone. `\b` on both sides keeps it out of
// `ADR-0046` (there is no word boundary between `0` and `0` after the third
// digit... actually it is excluded by requiring exactly three digits then a
// non-digit) and out of `ADR-0460`.
const THREE = /\bADR-(\d{3})\b/g;

const fix = (line) => line.replace(THREE, (_m, n) => `ADR-0${n}`);

if (process.argv.includes('--self-test')) {
  const cases = [
    ['ADR-046 alone', 'Ver ADR-046 para el detalle.', 'Ver ADR-0046 para el detalle.'],
    ['inside a path already 4-digit', 'docs/adr/ADR-0046-x.md', 'docs/adr/ADR-0046-x.md'],
    ['a 4-digit number is untouched', 'ADR-0046 y ADR-0047', 'ADR-0046 y ADR-0047'],
    ['five digits untouched', 'ADR-00460', 'ADR-00460'],
    ['not an ADR identifier', 'ADR046 sin guion', 'ADR046 sin guion'],
    ['two on one line', 'ADR-030 y ADR-045', 'ADR-0030 y ADR-0045'],
  ];
  let bad = 0;
  for (const [name, src, want] of cases) {
    const got = fix(src);
    const ok = got === want;
    if (!ok) bad++;
    console.log(`  ${ok ? 'ok  ' : 'FAIL'}  ${name}  got="${got}" want="${want}"`);
  }
  console.log(bad ? `\nSELF-TEST FAILED: ${bad} case(s).` : '\nself-test passed.');
  process.exit(bad ? 1 : 0);
}

let files = 0;
let lines = 0;
let fenceSkipped = 0;

for (const rel of listDocs({ includeArchive: true })) {
  const src = readFileSync(rel, 'utf8');
  if (!/\bADR-\d{3}\b/.test(src)) continue;

  const out = [];
  let fence = false;
  let touched = 0;
  for (const line of src.split('\n')) {
    if (FENCE.test(line)) { fence = !fence; out.push(line); continue; }
    if (fence) {
      if (/\bADR-\d{3}\b/.test(line)) fenceSkipped++;
      out.push(line);
      continue;
    }
    const next = fix(line);
    if (next !== line) touched++;
    out.push(next);
  }
  if (!touched) continue;
  if (WRITE) writeFileSync(rel, out.join('\n'), 'utf8');
  files++;
  lines += touched;
}

console.log(`${WRITE ? 'NORMALISED' : 'DRY RUN'}  files: ${files}  lines: ${lines}`);
if (fenceSkipped) console.log(`  left alone inside code fences: ${fenceSkipped}`);
if (!WRITE) console.log('  re-run with --write to apply');
