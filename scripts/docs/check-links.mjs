#!/usr/bin/env node
// scripts/docs/check-links.mjs
//
// PURPOSE
//   Offline integrity gate for docs/. Verifies that every internal Markdown link
//   resolves to a file that exists. Self-contained: no network, no lychee, no npm
//   install. Runs in ~1s on 1 700 files, so it is cheap enough for every PR.
//
//   External URLs are reported as a COUNT only and never fail the gate, because
//   external rot is expected and gating on it blocks PRs for no signal. The
//   community pattern for that is a scheduled, non-blocking sweep.
//   (see: qdrant/landing_page and lancedb/lancedb weekly lychee jobs, both
//    `fail: false`, both writing results to a tracking issue)
//
// WHAT IS CHECKED
//   - [label](relative/path.md)      relative Markdown links
//   - [label](./dir/other.md#anchor) with the #anchor fragment stripped for
//                                   existence, and the target counted
//   - [[wikilink]]                   legacy Obsidian form — reported so the
//                                   migration can be proven complete
//   - ![alt](assets/x.png)           image/attachment targets
//   - Protocol/anchor-only links     skipped
//
// NOT CHECKED
//   - Link text quality, headings, prose. This is existence, not correctness.
//   - Duplicate basenames. That is `check-docs.mjs --orphans` / the schema gate.
//
// USAGE
//   node scripts/docs/check-links.mjs              # gate, exit 1 on internal breaks
//   node scripts/docs/check-links.mjs --json       # machine-readable
//   node scripts/docs/check-links.mjs --all        # include archived trees
//   node scripts/docs/check-links.mjs --no-wikilinks  # do not gate on [[wikilinks]]
//   node scripts/docs/check-links.mjs --max-wikilinks=N  # set the tolerated count
//
// EXIT CODES
//   0 = all internal links resolve and the wikilink count is within budget
//   1 = at least one internal link is broken, or the wikilink budget is exceeded
//   2 = bad usage

import { readdirSync, readFileSync, existsSync } from 'node:fs';
import { join, dirname, relative, sep, resolve as pathResolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildTargetSet, listDocs, parseFrontmatter, proseOf, segment, WIKI_LINK, MD_LINK } from './lib.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const DOCS = join(ROOT, 'docs');

const ARGS = new Set(process.argv.slice(2));
const JSON_OUT = ARGS.has('--json');
const INCLUDE_ARCHIVE = ARGS.has('--all');
const FAIL_WIKILINKS = !ARGS.has('--no-wikilinks');

/**
 * Broken internal Markdown links, tolerated.
 *
 * 2026-10-04 (F1-T1 drained, DOCS-F1): 0. Every broken entry was triaged P0-P4
 * (docs/dev/tasks/DOCS-F1.md); the P0-P2 residue was repaired and the P3 frozen
 * trees are excluded below. A non-zero budget now means a real regression in the
 * navigable surface, so keep it at 0.
 *
 * History: 109 (2026-09-28) -> 58 (resolution index widened to the whole repo:
 * 51 were false positives) -> 44 -> 0. `repair-links.mjs` fixed only 1 of 110
 * automatically, which is the diagnostic: the rest were claims in prose about
 * documents that did not exist, links inside code spans, or frozen history.
 */
const BROKEN_MD_BUDGET = Number(
  (process.argv.find((a) => a.startsWith('--max-broken=')) ?? '').split('=')[1] || 0,
);
const WIKILINK_BUDGET = Number(
  (process.argv.find((a) => a.startsWith('--max-wikilinks=')) ?? '').split('=')[1] || 20,
);

/**
 * Wikilinks in prose, tolerated. Counted as OCCURRENCES, not files, and computed
 * over `proseOf()` -- front matter, fenced code blocks and inline code spans are
 * excluded, because a document that DOCUMENTS a pattern is not an instance of
 * it. The front matter `links:` property is excluded on the same grounds, and for
 * a stronger reason: Obsidian types that property vault-wide, so rewriting it
 * would corrupt all 1725 files.
 *
 * 2026-10-04 (DOCS-F1): 20 occurrences, down from 40 (2026-09-28), all in
 * frozen trees: 19 in tasks/** work-items + 1 in avance/historial/ snapshots.
 * The F1-T2 mojibake ([[bench]]/[[test]]/[[package]]) was drained to 0 in
 * prose, and the live surface has no prose wikilinks left. The remaining
 * `[[table]]` hits are legitimate TOML references inside code spans and are
 * not counted.
 *
 * The gate's job is to stop the count going UP. Lower it as the debt drains.
 */

// Frozen history is never repaired, so it is not part of the gate. Same
// rationale as the .markdownlint-cli2.yaml `ignores` block.
const ARCHIVE_RE = /(^|\/)(archive|target|node_modules|\.obsidian)(\/|$)/;
const SKIP_DIR = new Set(['.obsidian', 'node_modules']);

// P3 sources (F1 triage 2026-10-04, docs/dev/tasks/DOCS-F1.md): frozen by
// design, so a broken link inside them is never repaired and must not gate.
//   avance/historial/**  the historical registry (snapshots, campaign records)
//   tasks/**             frozen work-items (owner restriction: they do not move
//                        and are not edited without their own task file)
// `archive/` trees are already out of scope via ARCHIVE_RE above. The motivo for
// this exclusion lives in .github/workflows/gate-docs-links.yml.
const FROZEN_RE = /(^|\/)(avance\/historial|tasks)(\/|$)/;

// ---------------------------------------------------------------- file index

// Resolved targets come from the WHOLE repository, not from docs/ alone.
// A document may legitimately link out of the tree -- docs/api/VERSIONING.md
// -> ../../CONTRIBUTING.md is a correct GitHub link, and reporting it as broken
// is a false positive that inflates the number the gate has to tolerate. The
// list of .md files to SCAN is still docs/ only; only the resolution index is
// repo-wide. buildTargetSet() lives in lib.mjs so this cannot drift from the
// other scripts again.
const { files: allFiles, dirs: allDirs } = buildTargetSet();
const mdFiles = listDocs({ includeArchive: true });

const inScope = (rel) => INCLUDE_ARCHIVE || !ARCHIVE_RE.test(rel);

// ------------------------------------------------------------- link parsing

// Link patterns are imported from lib.mjs so that check-links, check-docs and
// wikilinks-to-md share one definition. Three scripts with three regexes for the
// same syntax is how a corpus ends up with three different totals for it.

function normalise(p) {
  const out = [];
  for (const seg of p.split(/[\\/]/)) {
    if (seg === '' || seg === '.') continue;
    if (seg === '..') out.pop();
    else out.push(seg);
  }
  return out.join('/');
}

const report = {
  filesScanned: 0,
  filesInScope: 0,
  linksChecked: 0,
  externalLinks: 0,
  wikilinksRemaining: 0,
  broken: [], // { file, line, target, kind }
};

/**
 * Line number for a character index into `proseOf(text)`.
 *
 * proseOf() concatenates only the safe segments, and every safe segment is a
 * single line (the segmenter pushes per line), so a match's line is the line of
 * the segment holding its first character. Counting newlines in the RAW text
 * with an index from the prose string reports the wrong line as soon as a code
 * span appears before the match.
 */
function proseLineLookup(text) {
  // segment()'s line numbers do not count the front matter block (the walker
  // starts after it), so add its line count back -- otherwise every reported
  // line is off by the size of the frontmatter. The boundary comes from
  // parseFrontmatter() so it cannot drift from the parser it mirrors.
  const fm = parseFrontmatter(text);
  const offset = fm.hadFrontmatter ? text.slice(0, fm.end).split('\n').length - 1 : 0;
  const bounds = [];
  let acc = 0;
  for (const s of segment(text)) {
    if (!s.safe) continue;
    acc += s.text.length;
    bounds.push({ end: acc, line: s.line + offset });
  }
  return (idx) => {
    for (const b of bounds) if (idx < b.end) return b.line;
    return 0;
  };
}

for (const rel of mdFiles) {
  report.filesScanned++;
  if (!inScope(rel)) continue;
  report.filesInScope++;

  const abs = join(ROOT, rel);
  const text = readFileSync(abs, 'utf8');
  const dir = dirname(rel);

  const prose = proseOf(text);
  const lineAt = proseLineLookup(text);

  // Scanned over proseOf(), not raw text: a `[label](path.md)` inside an inline
  // code span or a fenced block is documentation OF a link, not a clickable
  // link -- GitHub does not render it as one. Counting it made every document
  // that shows link syntax a permanent false positive (same class the wikilink
  // scan below already excludes). Measured on the clean pre-fix snapshot
  // (3bdc3fd5): 15 of 46 broken entries sat fully inside code (14-18 depending
  // on the segment-boundary convention; method in docs/dev/tasks/DOCS-F1.md).
  for (const m of prose.matchAll(MD_LINK)) {
    const raw = m[1];
    if (/^(https?:|mailto:|tel:|data:|obsidian:|#|\/\/)/i.test(raw)) {
      report.externalLinks++;
      continue;
    }
    report.linksChecked++;

    const [pathPart, anchor] = raw.split('#');
    if (!pathPart) continue; // pure #anchor

    const decoded = safeDecode(pathPart);
    const candidate = decoded.startsWith('/')
      ? normalise(decoded)
      : normalise(`${dir}/${decoded}`);

    if (!allFiles.has(candidate)) {
      // P3 frozen sources do not gate (FROZEN_RE above): the link is left as-is
      // by policy, so counting it would be permanent noise.
      if (!FROZEN_RE.test(rel)) {
        report.broken.push({ file: rel, line: lineAt(m.index), target: raw, kind: 'md' });
      }
    } else if (anchor && candidate.endsWith('.md')) {
      // Fragment is advisory: a missing heading is a softer failure than a
      // missing file, so it is tracked separately and does not fail the gate.
      report.anchors ??= [];
      report.anchors.push({ file: rel, target: candidate, anchor });
    }
  }

  for (const m of prose.matchAll(WIKI_LINK)) {
    report.wikilinksRemaining++;
    const target = m[1].trim();
    const base = target.toLowerCase();
    const hit =
      allFiles.has(target) ||
      allFiles.has(`${dir}/${target}`) ||
      [...allFiles].some((f) => f.toLowerCase().endsWith(`/${base}`));
    if (!hit) {
      report.broken.push({ file: rel, line: 0, target, kind: 'wikilink' });
    }
  }
}

function safeDecode(s) {
  try {
    return decodeURIComponent(s);
  } catch {
    return s;
  }
}

// -------------------------------------------------------------------- output

if (JSON_OUT) {
  console.log(JSON.stringify(report, null, 2));
} else {
  const brokenMdCount = report.broken.filter((b) => b.kind === 'md').length;
  console.log(`files scanned          : ${report.filesScanned} (${report.filesInScope} in scope)`);
  console.log(`internal links checked : ${report.linksChecked}`);
  console.log(`external links (skipped): ${report.externalLinks}`);
  console.log(`wikilinks remaining    : ${report.wikilinksRemaining} (budget ${WIKILINK_BUDGET})`);
  console.log(`broken markdown links  : ${report.broken.filter((b) => b.kind === 'md').length} (budget ${BROKEN_MD_BUDGET})`);

  if (report.broken.length) {
    const byFile = new Map();
    for (const b of report.broken) {
      if (!byFile.has(b.file)) byFile.set(b.file, []);
      byFile.get(b.file).push(b);
    }
    // Only Markdown links fail the gate. A broken [[wikilink]] is already covered
    // by WIKILINK_BUDGET, and reporting it twice makes the real signal harder to
    // read in a 1700-file corpus.
    const mdBroken = report.broken.filter((b) => b.kind === 'md').length;
    const wikiBroken = report.broken.filter((b) => b.kind === 'wikilink').length;
    console.log(
      `\nBROKEN (${report.broken.length} in ${byFile.size} files): ` +
        `${mdBroken} markdown (GATES) + ${wikiBroken} wikilink (budgeted, informational)`,
    );
    for (const [file, items] of [...byFile].sort((a, b) => b[1].length - a[1].length)) {
      const shown = items.slice(0, 5);
      const gating = items.filter((i) => i.kind === 'md').length;
      console.log(`  ${file}  (${items.length}${gating ? `, ${gating} gating` : ''})`);
      for (const it of shown) {
        console.log(`      L${it.line}  -> ${it.target}`);
      }
      if (items.length > shown.length) console.log(`      ... +${items.length - shown.length} more`);
    }
  } else {
    console.log('\nOK: every internal link resolves.');
  }

  if (brokenMdCount > 0) {
    console.log(
      brokenMdCount > BROKEN_MD_BUDGET
        ? `\nOVER BUDGET: ${brokenMdCount} broken markdown links > ${BROKEN_MD_BUDGET}.`
        : `\nwithin budget: ${brokenMdCount} broken markdown links of ${BROKEN_MD_BUDGET} tolerated. ` +
          'P0-P2 must be 0 (F1 triage: docs/dev/tasks/DOCS-F1.md). ' +
          'Each one is a claim in the prose about a document that does not exist: ' +
          'write the page or delete the sentence.',
    );
  }

  if (report.wikilinksRemaining) {
    const over = report.wikilinksRemaining > WIKILINK_BUDGET;
    console.log(
      `\n${over ? 'OVER BUDGET' : 'within budget'}: ${report.wikilinksRemaining} [[wikilinks]] ` +
        `of which ${WIKILINK_BUDGET} tolerated. These do not render on GitHub. ` +
        'Run: node scripts/docs/wikilinks-to-md.mjs   (budget: F1-T2 in the plan)',
    );
  }
}

const brokenMd = report.broken.filter((b) => b.kind === 'md').length;
const brokenWikilinks = report.broken.filter((b) => b.kind === 'wikilink').length;
const brokenFails = brokenMd > BROKEN_MD_BUDGET;
const wikilinksFail = FAIL_WIKILINKS && report.wikilinksRemaining > WIKILINK_BUDGET;
process.exit(brokenFails || wikilinksFail ? 1 : 0);
