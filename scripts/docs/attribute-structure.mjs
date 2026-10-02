// Attribute each structural defect to its origin: caused by the segment() bug in
// this working tree, or already present in the last commit.
//
// A file is "inherited" when HEAD already has prose above its first H1 or an
// empty section at the same place. Those are pre-existing, committed defects and
// are a different (older) problem from the segmenter regression.
//
//   node scripts/docs/attribute-structure.mjs
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { listDocs } from './lib.mjs';

const argv = process.argv.slice(2);
const SELF_TEST = argv.includes('--self-test');

/** Prose between frontmatter and the first H1, plus empty sections. Cheap, self-contained. */
function defects(text) {
  const nl = text[3] === '\r' ? '\r\n' : '\n';
  const close = text.startsWith('---') ? text.indexOf(nl + '---' + nl, 3) : -1;
  const bodyStart = close === -1 ? 0 : close + (nl + '---' + nl).length;
  const body = text.slice(bodyStart);
  const lines = body.split('\n');

  const blankish = (l) =>
    !l.trim() || /^\s*[-*_]{3,}\s*$/.test(l) || /^\s*(<!--|-->)/.test(l)
    || /^[A-Za-z_][\w-]*\s*:/.test(l) || /^---\s*$/.test(l);

  const h1rel = lines.findIndex((l) => /^# \S/.test(l));
  let orphan = 0;
  if (h1rel > 0) orphan = lines.slice(0, h1rel).filter((l) => !blankish(l)).length;

  let empty = 0;
  let fence = false;
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i];
    if (/^\s{0,3}(```|~~~)/.test(l)) { fence = !fence; continue; }
    if (fence || !/^#{1,6}\s+\S/.test(l)) continue;
    const lvl = l.match(/^(#{1,6})\s/)[1].length;
    let j = i + 1;
    while (j < lines.length && blankish(lines[j])) j++;
    if (j >= lines.length) { empty++; continue; }
    if (/^#{1,6}\s+\S/.test(lines[j]) && lines[j].match(/^(#{1,6})\s/)[1].length <= lvl) empty++;
  }
  return { orphan, empty };
}

if (SELF_TEST) {
  const a = defects('---\ntitle: t\n---\n# H\n\nbody\n');
  const b = defects('---\ntitle: t\n---\nprose above\n\n# H\n\n## S\n\nbody\n');
  const ok = a.orphan === 0 && a.empty === 0 && b.orphan === 1;
  console.log(`  ${ok ? 'ok  ' : 'FAIL'}  clean file -> 0 defects, prose-above-H1 -> 1 orphan`);
  process.exit(ok ? 0 : 1);
}

const rows = [];
for (const rel of listDocs({ includeArchive: true })) {
  const now = defects(readFileSync(rel, 'utf8'));
  if (now.orphan === 0 && now.empty === 0) continue;
  let head = null;
  try {
    head = defects(execFileSync('git', ['show', `HEAD:${rel}`], { encoding: 'utf8', maxBuffer: 1 << 26 }));
  } catch {
    rows.push({ rel, now, head: null });
    continue;
  }
  rows.push({ rel, now, head });
}

const caused = rows.filter((r) => r.head && (r.now.orphan > r.head.orphan || r.now.empty > r.head.empty));
const inherited = rows.filter((r) => r.head && r.now.orphan <= r.head.orphan && r.now.empty <= r.head.empty);
const fresh = rows.filter((r) => !r.head);

console.log(`files with defects now : ${rows.length}`);
console.log(`  caused by this tree  : ${caused.length}`);
console.log(`  inherited from HEAD  : ${inherited.length}`);
console.log(`  new files            : ${fresh.length}`);

if (caused.length) {
  console.log('\nCAUSED BY THIS WORKING TREE (regression):');
  for (const r of caused.sort((a, b) => b.now.orphan - a.now.orphan)) {
    console.log(`  ${r.rel}  orphan ${r.head.orphan}->${r.now.orphan}  empty ${r.head.empty}->${r.now.empty}`);
  }
}
if (inherited.length) {
  console.log('\nINHERITED FROM HEAD (pre-existing):');
  for (const r of inherited.sort((a, b) => b.now.orphan - a.now.orphan).slice(0, 30)) {
    console.log(`  ${r.rel}  orphan ${r.now.orphan}  empty ${r.now.empty}`);
  }
  if (inherited.length > 30) console.log(`  ... and ${inherited.length - 30} more`);
}
if (fresh.length) {
  console.log('\nNEW FILES (no HEAD baseline):');
  for (const r of fresh.slice(0, 20)) console.log(`  ${r.rel}  orphan ${r.now.orphan}  empty ${r.now.empty}`);
}
