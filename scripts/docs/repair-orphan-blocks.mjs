// Repair the frontmatter-stamping damage left by an earlier, buggy run of
// stamp-frontmatter.mjs. 38 files have prose stranded between the frontmatter
// and the first H1; 28 of them lost real content outright (PYTHON_SDK.md lost
// 239 lines, EMBEDDED_SDK.md 257, BENCHMARKS.md 256).
//
// The current stamp-frontmatter.mjs is non-destructive (lib.mjs parseFrontmatter
// returns `body: text.slice(end)`), so the fix is to restore the committed
// version and re-apply the mechanical passes with the fixed script.
//
// Dry run by default. Pass --write to apply.
import { readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { listDocs } from './lib.mjs';

const argv = process.argv.slice(2);

const WRITE = process.argv.includes('--write');
// --exclude=<glob-ish prefix>  Skip paths another process is actively editing,
// so a concurrent agent's work is never clobbered by a restore-from-HEAD.
const EXCLUDE = argv.filter((a) => a.startsWith('--exclude=')).map((a) => a.slice('--exclude='.length));

const isProse = (l) =>
  l.trim() && !/^---$|^[A-Za-z_-]+:\s|^<!--|^\s*-->|^\|/.test(l);

const rows = [];
for (const rel of listDocs({ includeArchive: true })) {
  if (EXCLUDE.some((p) => rel.startsWith(p) || rel.includes(p))) {
    rows.push({ rel, verdict: 'SKIP excluded' });
    continue;
  }
  const lines = readFileSync(rel, 'utf8').split('\n');
  const h1 = lines.findIndex((l) => /^# \S/.test(l));
  if (h1 < 0) continue;
  if (!lines.slice(0, h1).some(isProse)) continue;

  let head;
  try {
    head = execFileSync('git', ['show', `HEAD:${rel}`], { encoding: 'utf8', maxBuffer: 1 << 26 });
  } catch {
    rows.push({ rel, verdict: 'SKIP new file (not in HEAD)' });
    continue;
  }

  const hLines = head.split('\n');
  const hH1 = hLines.findIndex((l) => /^# \S/.test(l));
  if (hH1 < 0) { rows.push({ rel, verdict: 'SKIP HEAD has no H1' }); continue; }

  const hBody = hLines.slice(hH1).map((l) => l.trimEnd());
  const nBody = lines.slice(h1).map((l) => l.trimEnd());
  const lost = hBody.length - nBody.length;
  let lcp = 0;
  while (lcp < Math.min(hBody.length, nBody.length) && hBody[lcp] === nBody[lcp]) lcp++;
  const drift = Math.min(hBody.length, nBody.length) - lcp;

  if (drift === 0 && lost === 0) { rows.push({ rel, verdict: 'ok (orphan block only)' }); continue; }

  rows.push({ rel, verdict: 'RESTORE', lost, drift });
}

const pad = (s, n) => String(s).padEnd(n);
console.log(pad('file', 62) + pad('lost', 7) + pad('drift', 7) + 'action');
console.log('-'.repeat(86));
let n = 0;
for (const r of rows) {
  const doIt = r.verdict === 'RESTORE';
  if (doIt) n++;
  console.log(pad(r.rel, 62) + pad(r.lost ?? '', 7) + pad(r.drift ?? '', 7) + (doIt ? 'RESTORE' : r.verdict));
}
console.log(`\n${rows.length} files with an orphan block, ${n} need restoring.`);

if (WRITE) {
  for (const r of rows) {
    if (r.verdict !== 'RESTORE') continue;
    writeFileSync(
      r.rel,
      execFileSync('git', ['show', `HEAD:${r.rel}`], { encoding: 'utf8', maxBuffer: 1 << 26 }),
      'utf8',
    );
  }
  console.log(`restored ${n} files from HEAD.`);
} else {
  console.log('Dry run. Re-run with --write to apply.');
}
