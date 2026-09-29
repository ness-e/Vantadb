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
// The budget counts DAMAGED FILES, not findings. One file with 200 findings is
// one document that needs one repair pass; gating on the finding count would
// make the ratchet jump around for no reason. Lower it as files are repaired,
// and to 0 once the corpus is clean.
const BUDGET = budgetArg ? Number(budgetArg.split('=')[1]) : 0;

const FENCE = /^\s*(```|~~~)/;
const HEADING = /^(#{1,6})\s+\S/;
const TABLE_ROW = /^\s*\|/;
const FRONTMATTER_KEY = /^[A-Za-z_][\w-]*\s*:/;
const HTML = /^\s*(<!--|-->)/;
// `#Fjall` -- a hash run with no space after it. GitHub renders this as a plain
// paragraph, so the document has no visible title at all. 65 files in docs/
// were written this way, and it also made every downstream H1-based check
// mis-locate the heading, which is how it stayed invisible this long.
const MALFORMED_HEADING = /^(#{1,6})(?=[^#\s])/;

/** Lines that carry no renderable content. A table row DOES carry content. */
/**
 * Lines that carry no renderable content, scanning a document BODY.
 *
 * Deliberately does NOT include the frontmatter-key rule. `Status: Accepted`,
 * `Callers: src/`, `LanceDB:` and `SDP:` are ordinary prose that a section body
 * can legitimately open with -- every ADR in docs/ has a `## Status` whose body
 * is exactly that. Treating them as blank reported 47 of 87 damaged files as
 * empty sections when they are perfectly full. The rule only makes sense where
 * YAML actually is: between the opening `---` and its close, which is what
 * `findOrphanProse` scans.
 */
const isBlankish = (l) =>
  !l.trim() ||
  /^\s*[-*_]{3,}\s*$/.test(l) ||
  HTML.test(l) ||
  /^---\s*$/.test(l);

/** As above, plus the frontmatter-key rule. Only valid above the body's start. */
const isBlankishWithKeys = (l) => isBlankish(l) || FRONTMATTER_KEY.test(l);

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
    .filter(({ text: l }) => !isBlankishWithKeys(l) && !/^\s*>/.test(l));
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
    // These four are the 47 false positives. A body opening with `Key: value`
    // is prose, not YAML, and the section is full.
    ['Status: as a body is content, not frontmatter',
      '## Status\n\nStatus: Accepted\n\n## Next\n\nb\n', []],
    ['Callers: as a body is content',
      '## Context\n\nCallers: src/\n\n## Next\n\nb\n', []],
    ['SDP: as a body is content',
      '## Note\n\nSDP: 4 in 15 min\n\n## Next\n\nb\n', []],
    ['a bare Key: line with no value is still content',
      '## Status\n\nRead-only:\n\n## Next\n\nb\n', []],
    ['heading inside a fence is not a heading',
      'a\n\n## One\n\n```\n## Fake\n```\n\nbody\n', []],
  ];
  const badHeadings = [
    ['#Fjall is malformed', '#Fjall\n', 1],
    ['# Two spaces is fine', '#  Two spaces\n', 0],
    ['######Six is malformed', '######Six\n', 1],
    ['#######Seven is not a heading', '#######Seven\n', 0],
    ['a #hashtag in prose is not a heading', 'see #hashtag here\n', 0],
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

  const countBad = (src) => {
    let fence = false;
    let n = 0;
    for (const l of src.split('\n')) {
      if (FENCE.test(l)) { fence = !fence; continue; }
      if (!fence && MALFORMED_HEADING.test(l)) n++;
    }
    return n;
  };
  for (const [name, src, want] of badHeadings) {
    const got = countBad(src);
    const ok = got === want;
    if (!ok) bad++;
    console.log(`  ${ok ? 'ok  ' : 'FAIL'}  ${name}  got=${got} want=${want}`);
  }
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
  // Malformed headings: `#Fjall` instead of `# Fjall`. Reported by line, and
  // skipped inside fences where a `#` comment is legitimate.
  {
    let fence = false;
    body.split('\n').forEach((l, i) => {
      if (FENCE.test(l)) { fence = !fence; return; }
      if (fence) return;
      const m = l.match(MALFORMED_HEADING);
      if (m) {
        findings.push({
          file: rel,
          kind: 'malformed-heading',
          line: bodyStart + i,
          detail: l.trim().slice(0, 60),
        });
      }
    });
  }
}

const byKind = findings.reduce((a, f) => ((a[f.kind] = (a[f.kind] ?? 0) + 1), a), {});

if (JSON_OUT) {
  console.log(JSON.stringify({ filesScanned: files.length, budget: BUDGET, byKind, findings }, null, 2));
} else {
  console.log(`files scanned : ${files.length}`);
  console.log(`empty sections: ${byKind['empty-section'] ?? 0}`);
  console.log(`orphan prose  : ${byKind['orphan-prose'] ?? 0}`);
  console.log(`bad headings  : ${byKind['malformed-heading'] ?? 0}`);
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
  const brokenFiles = new Set(findings.map((f) => f.file)).size;
  if (brokenFiles > BUDGET) {
    console.log(`\nOVER BUDGET: ${brokenFiles} damaged files > ${BUDGET} (${total} defects).`);
    process.exit(1);
  }
  console.log(`\nwithin budget: ${brokenFiles} of ${BUDGET} damaged files tolerated (${total} defects).`);
}

process.exit(new Set(findings.map((f) => f.file)).size > BUDGET ? 1 : 0);
