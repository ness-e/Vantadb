// Fix `#Fjall` -> `# Fjall`: a hash run with no space after it.
//
// GitHub renders `#Fjall` as a plain paragraph, so those 66 documents have no
// visible title. It also breaks every heading-anchored check, which is how the
// defect stayed invisible while check-structure.mjs reported the surrounding
// content as "prose stranded above the H1" instead of naming the real cause.
//
// Only the heading line is touched, and only outside fences (a `#` comment
// inside a code block is legitimate and must be left alone).
//
//   node scripts/docs/fix-bad-headings.mjs            # dry run
//   node scripts/docs/fix-bad-headings.mjs --write
import { readFileSync, writeFileSync } from 'node:fs';
import { listDocs, segmentsRoundTrip } from './lib.mjs';

const WRITE = process.argv.includes('--write');
const FENCE = /^\s*(```|~~~)/;
const BAD = /^(#{1,6})(?=[^#\s])/;
// Whole-file prefilter. The `m` flag is load-bearing: without it `^` anchors to
// the start of the STRING, so this tested only line 1 of every file and skipped
// all 66 of them. The per-line loop below is where matching actually happens,
// which is why the prefilter must stay an optimisation and never the gate.
const BAD_ANYWHERE = /^(#{1,6})(?=[^#\s])/m;

if (process.argv.includes('--self-test')) {
  const countIn = (src) => {
    let fence = false;
    let n = 0;
    for (const l of src.split('\n')) {
      if (FENCE.test(l)) { fence = !fence; continue; }
      if (!fence && BAD.test(l)) n++;
    }
    return n;
  };
  const cases = [
    ['#Fjall on line 5, not line 1', '---\nt: x\n---\n\n#Fjall\n', 1],
    ['two malformed headings', '#A\n#B\n', 2],
    ['well-formed headings untouched', '# A\n## B\n', 0],
    ['a hash comment inside a fence is not a heading', '#A\n\n```sh\n#comment\n```\n', 1],
  ];
  let bad = 0;
  for (const [name, src, want] of cases) {
    const got = countIn(src);
    if (got !== want) bad++;
    console.log(`  ${got === want ? 'ok  ' : 'FAIL'}  ${name}  got=${got} want=${want}`);
  }
  const pre = BAD_ANYWHERE.test('---\nt: x\n---\n\n#Fjall\n');
  if (!pre) bad++;
  console.log(`  ${pre ? 'ok  ' : 'FAIL'}  prefilter matches a heading on a later line`);
  console.log(bad ? `\nSELF-TEST FAILED: ${bad} case(s).` : '\nself-test passed.');
  process.exit(bad ? 1 : 0);
}

let filesChanged = 0;
let linesChanged = 0;
let inFenceHits = 0;

for (const rel of listDocs({ includeArchive: true })) {
  const src = readFileSync(rel, 'utf8');
  if (!BAD_ANYWHERE.test(src)) continue;

  const out = [];
  let fence = false;
  let touched = 0;
  for (const line of src.split('\n')) {
    if (FENCE.test(line)) { fence = !fence; out.push(line); continue; }
    if (fence) {
      if (BAD.test(line)) inFenceHits++;
      out.push(line);
      continue;
    }
    const m = line.match(BAD);
    if (m) { out.push(`${m[1]} ${line.slice(m[1].length)}`); touched++; continue; }
    out.push(line);
  }

  if (!touched) continue;

  // The invariant, checked per line rather than by regex over the whole file:
  // an earlier version compared `next.replace(/^(#{1,6}) /gm, '$1')` against the
  // source, which also strips the space from headings that were ALREADY correct
  // -- so the comparison could never hold and every file was refused.
  const orig = src.split('\n');
  const now = out.join('\n').split('\n');
  let violated = null;
  if (orig.length !== now.length) violated = 'line count changed';
  else {
    for (let i = 0; i < orig.length; i++) {
      if (orig[i] === now[i]) continue;
      // Every changed line must be its original with exactly one space inserted
      // after the hash run, and nothing else moved.
      if (now[i].replace(/^(#{1,6}) /, '$1') !== orig[i]) { violated = `line ${i + 1}`; break; }
    }
  }
  if (violated) {
    console.error(`  REFUSING ${rel}: ${violated}`);
    process.exitCode = 2;
    continue;
  }

  if (WRITE) writeFileSync(rel, now.join('\n'), 'utf8');
  filesChanged++;
  linesChanged += touched;
}

console.log(`${WRITE ? 'FIXED' : 'DRY RUN'}  files: ${filesChanged}  headings: ${linesChanged}`);
if (inFenceHits) console.log(`  left alone inside code fences: ${inFenceHits}`);
if (!WRITE) console.log('  re-run with --write to apply');
