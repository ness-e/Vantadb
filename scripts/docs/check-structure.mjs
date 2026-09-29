// Detect the two structural corruptions that an earlier, buggy run of
// stamp-frontmatter.mjs left behind in docs/:
//
//   1. EMPTY SECTION  -- a heading immediately followed by another heading, a
//      fence, or EOF. A heading with no body means the body was moved elsewhere
//      (usually stranded above the H1) or deleted.
//   2. ORPHAN PROSE   -- real prose between the frontmatter and the first H1.
//      That text belongs under some heading; if it is above the H1 it is stranded.
//
// Both are silent on GitHub: the file renders, the link check passes, and the
// content is simply gone from where a reader would look for it.
//
//   node scripts/docs/check-structure.mjs
//   node scripts/docs/check-structure.mjs --json
//   node scripts/docs/check-structure.mjs --self-test
//
// Exits non-zero when any finding exceeds --budget (default 0: this is a defect
// class, not a metric to be tolerated).
import { readFileSync } from 'node:fs';
import { listDocs } from './lib.mjs';

const argv = process.argv.slice(2);
const JSON_OUT = argv.includes('--json');
const SELF_TEST = argv.includes('--self-test');
const budgetArg = argv.find((a) => a.startsWith('--budget='));
const BUDGET = budgetArg ? Number(budgetArg.split('=')[1]) : 0;

const FENCE = /^\s*(```|~~~)/;
const HEADING = /^(#{1,6})\s+\S/;
const TABLE_ROW = /^\s*\|/;
const FRONTMATTER_KEY = /^[A-Za-z_][\w-]*\s*:/;
const HTML = /^\s*(<!--|-->)/;

/** Lines that carry no renderable content. A table row DOES carry content. */
const isBlankish = (l) =>
  !l.trim() ||
  /^\s*[-*_]{3,}\s*$/.test(l) ||
  HTML.test(l) ||
  FRONTMATTER_KEY.test(l) ||
  /^---\s*$/.test(l);

const levelOf = (l) => (HEADING.test(l) ? l.match(/^(#{1,6})\s/)[1].length : 0);

/**
 * A heading is EMPTY when the next thing after it is a heading of the SAME OR
 * SHALLOWER level, or end-of-file. Two deliberate exclusions keep the signal
 * clean enough to act on:
 *
 *   - a deeper next heading (`##` then `###`) is a container delegating to its
 *     children, not an empty section -- this is what CHANGELOG.md is full of;
 *   - a fenced code block, a table, a list, or a blockquote all count as body.
 *
 * The defect being caught is narrow and specific: the body of a section was
 * moved out of it (usually stranded above the file's H1) or deleted, leaving a
 * heading whose content is gone. That renders silently on GitHub.
 */
function findEmptySections(body, startLine) {
  const out = [];
  const lines = body.split('\n');
  let inFence = false;
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i];
    if (FENCE.test(l)) { inFence = !inFence; continue; }
    if (inFence || !HEADING.test(l)) continue;

    const level = levelOf(l);
    let j = i + 1;
    while (j < lines.length && isBlankish(lines[j])) j++;

    if (j >= lines.length) { out.push({ line: startLine + i, heading: l.trim() }); continue; }
    const n = lines[j];
    if (HEADING.test(n) && levelOf(n) <= level) {
      out.push({ line: startLine + i, heading: l.trim() });
    }
  }
  return out;
}

/**
 * Real prose stranded between the end of the frontmatter and the file's first
 * H1. That text belongs under some heading further down; above the H1 it is
 * orphaned, and on GitHub it renders as a preface with no owner.
 */
function findOrphanProse(text) {
  const nl = text[3] === '\r' ? '\r\n' : '\n';
  const close = text.startsWith('---') ? text.indexOf(nl + '---' + nl, 3) : -1;
  const bodyStart = close === -1 ? 0 : close + (nl + '---' + nl).length;
  const body = text.slice(bodyStart);
  const rel = body.split('\n').findIndex((l) => /^# \S/.test(l));
  if (rel <= 0) return [];
  const pre = body.split('\n').slice(0, rel);
  const base = text.slice(0, bodyStart).split('\n').length - 1;
  return pre
    .map((text2, i) => ({ line: base + i + 1, text: text2 }))
    .filter(({ text: l }) => !isBlankish(l) && !/^\s*>/.test(l));
}

// ------------------------------------------------------------------ self-test
if (SELF_TEST) {
  const cases = [
    ['empty section between same-level headings',
      'a\n\n## One\n\n## Two\n\nb\n', ['One']],
    ['empty section at end of file',
      'a\n\n## One\n\nprose\n\n## Two\n\n', ['Two']],
    ['heading then fence is NOT empty',
      'a\n\n## One\n\n```py\ncode\n```\n', []],
    ['heading with prose is NOT empty',
      'a\n\n## One\n\nprose here\n', []],
    ['heading with only a table IS content',
      'a\n\n## One\n\n| a | b |\n|---|---|\n', []],
    ['heading with a list IS content',
      'a\n\n## One\n\n- x\n- y\n', []],
    ['heading with a blockquote IS content',
      'a\n\n## One\n\n> quoted\n', []],
    ['container H2 delegating to H3 is NOT empty',
      'a\n\n## One\n\n### Sub\n\nbody\n', []],
    ['thematic break does not fill a section',
      'a\n\n## One\n\n---\n\n## Two\n\nb\n', ['One']],
    ['heading inside a fence is not a heading',
      'a\n\n## One\n\n```\n## Fake\n```\n\nbody\n', []],
  ];
  let bad = 0;
  for (const [name, src, want] of cases) {
    const got = findEmptySections(src, 0).map((e) => e.heading.replace(/^#+\s*/, ''));
    const ok = JSON.stringify(got) === JSON.stringify(want);
    if (!ok) bad++;
    console.log(`  ${ok ? 'ok  ' : 'FAIL'} ${name}  got=${JSON.stringify(got)} want=${JSON.stringify(want)}`);
  }
  const o1 = findOrphanProse('---\ntitle: x\n---\n\nstranded prose\n\n# H\n\n## S\n\nreal\n');
  const o0 = findOrphanProse('---\ntitle: x\n---\n\n# H\n\n## S\n\nreal\n');
  const okO = o1.length === 1 && o0.length === 0;
  if (!okO) bad++;
  console.log(`  ${okO ? 'ok  ' : 'FAIL'} orphan prose detection  got=${o1.length}/${o0.length} want=1/0`);
  console.log(bad ? `\nSELF-TEST FAILED: ${bad} case(s).` : '\nself-test passed.');
  process.exit(bad ? 1 : 0);
}

// ---------------------------------------------------------------------- scan
const files = listDocs({ includeArchive: true });
const findings = [];

for (const rel of files) {
  const text = readFileSync(rel, 'utf8');
  const nl = text.startsWith('---') && text[3] === '\r' ? '\r\n' : '\n';
  const close = text.startsWith('---') ? text.indexOf(nl + '---' + nl, 3) : -1;
  const body = close === -1 ? text : text.slice(close + (nl + '---' + nl).length);
  const bodyStart = close === -1 ? 1 : text.slice(0, close + (nl + '---' + nl).length).split('\n').length;

  for (const s of findEmptySections(body, bodyStart)) {
    findings.push({ file: rel, kind: 'empty-section', line: s.line, detail: s.heading });
  }
  for (const o of findOrphanProse(text)) {
    findings.push({ file: rel, kind: 'orphan-prose', line: o.line, detail: o.text.trim().slice(0, 80) });
  }
}

const byKind = findings.reduce((a, f) => ((a[f.kind] = (a[f.kind] ?? 0) + 1), a), {});

if (JSON_OUT) {
  console.log(JSON.stringify({ filesScanned: files.length, budget: BUDGET, byKind, findings }, null, 2));
} else {
  console.log(`files scanned : ${files.length}`);
  console.log(`empty sections: ${byKind['empty-section'] ?? 0}`);
  console.log(`orphan prose  : ${byKind['orphan-prose'] ?? 0}`);
  if (findings.length) {
    const byFile = new Map();
    for (const f of findings) {
      if (!byFile.has(f.file)) byFile.set(f.file, []);
      byFile.get(f.file).push(f);
    }
    for (const [file, fs] of byFile) {
      console.log(`\n  ${file}  (${fs.length})`);
      for (const f of fs.slice(0, 8)) {
        console.log(`    L${f.line}  ${f.kind.padEnd(14)} ${f.detail}`);
      }
      if (fs.length > 8) console.log(`    ... and ${fs.length - 8} more`);
    }
  }
  const total = findings.length;
  if (total > BUDGET) {
    console.log(`\nOVER BUDGET: ${total} structural defects > ${BUDGET}.`);
    process.exit(1);
  }
  console.log(`\nwithin budget: ${total} of ${BUDGET} tolerated.`);
}

process.exit(findings.length > BUDGET ? 1 : 0);
