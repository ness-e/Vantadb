#!/usr/bin/env node
// scripts/docs/check-docs.mjs
//
// PURPOSE
//   Single CI entry point for documentation governance. Replaces the ad-hoc bash
//   loop in gate-docs.yml that only checked "does frontmatter contain a title:".
//
//   The baseline measured on 2026-09-28 that motivated this file:
//     1717 files | 467 (27%) had frontmatter | 69 duplicate-basename groups
//     1481 orphans (86%) | 659 of 1397 internal links broken (47%)
//   Those numbers are now mechanically checkable, which is the only thing that
//   makes them improve.
//
// CHECKS
//   1. frontmatter present and valid against docs/_schema/frontmatter.schema.json
//   2. frontmatter `kind` agrees with the kind derived from the path
//   3. no duplicate basenames outside the allowlist (69 groups on 2026-09-28)
//   4. every non-index, non-archived document is reachable from an index
//      (the orphan gate -- this is the one that was failing at 86%)
//   5. no [[wikilinks]] in prose (they render as literal text on GitHub)
//
// MODES
//   default        report everything, exit 1 only on the checks that are already
//                  clean enough to gate (1, 2, 5). Checks 3 and 4 report only,
//                  because turning them red on day one is how a gate gets
//                  switched off. Promote them with --strict once drained.
//   --strict       all checks gate
//   --json         machine-readable
//   --fix-hints    print the exact command that reduces the count for each finding
//
// EXIT CODES
//   0 = all gating checks pass
//   1 = a gating check failed
//   2 = bad usage

import { listDocs, parseFrontmatter, deriveKind, isArchive, isIndexCandidate, loadSchema, loadTagVocabulary, validateFrontmatter, buildTargetSet, normalisePath, toPosix, linkFrom, encodePath, abs, readDoc, proseOf } from './lib.mjs';
import { readdirSync } from 'node:fs';
import { join, dirname, relative, basename, sep } from 'node:path';

const STRICT = process.argv.includes('--strict');
const JSON_OUT = process.argv.includes('--json');
const HINTS = process.argv.includes('--fix-hints');

const schema = loadSchema();
const vocab = loadTagVocabulary();
const docs = listDocs({ includeArchive: true });
const docsNoArchive = listDocs({ includeArchive: false });

// An empty string here means "duplicates are reported, not gated". A non-empty
// list means those basenames are permanently exempt.
const ALLOWED_DUPLICATE_BASENAMES = new Set([]);

// Wikilinks in prose, tolerated. Same population and same rationale as
// scripts/docs/check-links.mjs -- that script counts occurrences (40), this one
// counts files (33), because the two answer different questions: "is the
// Markdown link graph intact" and "does any file still carry legacy syntax".
// 2026-09-28: ~30 are mojibake from an earlier encoding pass (`[[bench]]`,
// `[[test]]`, `[[package]]`, `[[bin]]`), tracked as F1-T2 in
// docs/dev/plans/2026-09-28-docs-consolidation.md; the rest are dead links to
// renamed files (F1-T1). Lower as it drains.
const WIKILINK_FILE_BUDGET = 33;

const findings = {
  missingFrontmatter: [],
  schema: [],
  unknownKeys: [],
  kindMismatch: [],
  duplicateBasenames: [],
  orphans: [],
  wikilinks: [],
};

const records = new Map();
const recordsNoArchive = new Map();

// ------------------------------------------------------------- 1, 2: metadata

for (const rel of docs) {
  const text = readDoc(rel);
  const fm = parseFrontmatter(text);
  const rec = { rel, fm, kind: fm.data.kind ?? null, derived: deriveKind(rel) };

  if (!fm.hadFrontmatter) {
    findings.missingFrontmatter.push(rel);
  } else {
    const { errors, warnings } = validateFrontmatter(fm.data, schema, vocab);
    for (const p of errors) findings.schema.push({ file: rel, problem: p });
    for (const w of warnings) findings.unknownKeys.push({ file: rel, problem: w });
    if (fm.data.kind && fm.data.kind !== rec.derived) {
      findings.kindMismatch.push({ file: rel, declared: fm.data.kind, derived: rec.derived });
    }
  }
  records.set(rel, rec);
  if (!isArchive(rel)) recordsNoArchive.set(rel, rec);
}

// ------------------------------------------------------------ 3: duplicate names

{
  const groups = new Map();
  for (const rel of docs) {
    const n = basename(rel).toLowerCase();
    if (!groups.has(n)) groups.set(n, []);
    groups.get(n).push(rel);
  }
  for (const [n, files] of groups) {
    if (files.length < 2) continue;
    if (ALLOWED_DUPLICATE_BASENAMES.has(n)) continue;
    findings.duplicateBasenames.push({ basename: n, count: files.length, files });
  }
  findings.duplicateBasenames.sort((a, b) => b.count - a.count);
}

// ------------------------------------------------------------ 4: orphan gate

// Reachability is computed from the LINK GRAPH, not from the generated index:
// a file is reachable if some other file links to it, or if a generated index
// lists it. This mirrors the "orphan != bad" distinction: a leaf reference page
// reached only from a section index is fine.
{
  const { files } = buildTargetSet();
  const inbound = new Map();
  for (const rel of docs) inbound.set(rel, 0);

  const MD_LINK = /!?\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
  for (const rel of docs) {
    const text = readDoc(rel);
    const dir = dirname(rel);
    for (const m of text.matchAll(MD_LINK)) {
      const raw = m[1];
      if (/^(https?:|mailto:|tel:|data:|obsidian:|#|\/\/)/i.test(raw)) continue;
      const [pathPart] = raw.split('#');
      if (!pathPart) continue;
      let dec = pathPart;
      try { dec = decodeURIComponent(pathPart); } catch { /* keep raw */ }
      const cand = dec.startsWith('/') ? normalisePath(dec) : normalisePath(`${dir}/${dec}`);
      if (inbound.has(cand)) inbound.set(cand, inbound.get(cand) + 1);
    }
  }

  for (const rel of docsNoArchive) {
    if (isIndexCandidate(rel)) continue;
    if (inbound.get(rel) === 0) findings.orphans.push(rel);
  }
  findings.orphanTotal = docsNoArchive.filter((r) => !isIndexCandidate(r)).length;
}

// ------------------------------------------------------------ 5: wikilinks

{
  for (const rel of docs) {
    // proseOf() excludes front matter, fenced code blocks and inline code spans.
    // The front matter `links:` property is not counted because it is CORRECT
    // there: Obsidian types that property vault-wide, so rewriting it would
    // corrupt every file using it. Code is excluded because a document that
    // DOCUMENTS a pattern is not an instance of it -- this very plan names
    // `[[bench]]` in order to explain that it is mojibake.
    const n = (proseOf(readDoc(rel)).match(/\[\[[^\]]+\]\]/g) ?? []).length;
    if (n) findings.wikilinks.push({ file: rel, count: n });
  }
}

// ------------------------------------------------------------------- output

const gating = {
  missingFrontmatter: findings.missingFrontmatter.length,
  schema: findings.schema.length,
  kindMismatch: findings.kindMismatch.length,
  wikilinks: findings.wikilinks.length > WIKILINK_FILE_BUDGET ? findings.wikilinks.length - WIKILINK_FILE_BUDGET : 0,
  duplicateBasenames: STRICT ? findings.duplicateBasenames.length : 0,
  orphans: STRICT ? findings.orphans.length : 0,
};
const failed = Object.entries(gating).filter(([, v]) => v > 0);

if (JSON_OUT) {
  console.log(JSON.stringify({ ...findings, gating, failed: failed.map(([k]) => k) }, null, 2));
} else {
  console.log(`documents            : ${docs.length} (${docsNoArchive.length} non-archived)`);
  console.log(`GATING (fail -> 1)   : ${failed.length ? failed.map(([k, v]) => `${k}=${v}`).join(', ') : 'all clear'}`);
  console.log(`reporting only       : wikilinks=${findings.wikilinks.length}/${WIKILINK_FILE_BUDGET} files, unknownKeys=${findings.unknownKeys.length} files, duplicateBasenames=${findings.duplicateBasenames.length} groups, orphans=${findings.orphans.length}/${findings.orphanTotal}`);
  console.log('');

  const show = (label, items, fmt, cap = 12) => {
    if (!items.length) return;
    console.log(`${label} (${items.length}):`);
    for (const it of items.slice(0, cap)) console.log('  ' + fmt(it));
    if (items.length > cap) console.log(`  ... and ${items.length - cap} more`);
    console.log('');
  };

  show('MISSING FRONTMATTER', findings.missingFrontmatter, (f) => f);
  show('SCHEMA VIOLATIONS (gating)', findings.schema, (s) => `${s.file}  ${s.problem}`);
  show('UNKNOWN KEYS (reported, not gated — rename to x-<name>)', findings.unknownKeys,
    (s) => `${s.file}  ${s.problem}`);
  show('KIND MISMATCH (path vs frontmatter)', findings.kindMismatch,
    (k) => `${k.file}  declared=${k.declared} derived=${k.derived}`);
  show('DUPLICATE BASENAMES', findings.duplicateBasenames,
    (d) => `${d.basename} x${d.count}  e.g. ${d.files.slice(0, 2).join(', ')}`);
  show('ORPHANS (no inbound link, not an index, not archived)', findings.orphans, (f) => f);
  show('WIKILINKS IN PROSE (reported; gates only above budget)', findings.wikilinks, (w) => `${w.file}  (${w.count})`);

  if (HINTS) {
    console.log('HOW TO REDUCE EACH COUNT');
    console.log(`  ${findings.missingFrontmatter.length} missing frontmatter : node scripts/docs/stamp-frontmatter.mjs --write`);
    console.log(`  ${findings.schema.length} schema violations       : fix by hand, or run the above then the schema`);
    console.log(`  ${findings.unknownKeys.length} unknown keys          : rename to x-<name> in the same PR (reported, not gated)`);
    console.log(`  ${findings.kindMismatch.length} kind mismatches       : node scripts/docs/stamp-frontmatter.mjs --write --fix-kind`);
    console.log(`  ${findings.duplicateBasenames.length} duplicate names     : rename to kebab-case, one README.md per directory`);
    console.log(`  ${findings.orphans.length} orphans                 : node scripts/docs/gen-index.mjs --write  (indexes, not renames)`);
    console.log(`  ${findings.wikilinks.length} files with wikilinks  : node scripts/docs/wikilinks-to-md.mjs --write   (budget ${WIKILINK_FILE_BUDGET})`);
    console.log(`  broken internal links            : node scripts/docs/repair-links.mjs --write`);
    console.log(`  then re-run: node scripts/docs/check-docs.mjs`);
  }
}

process.exit(failed.length ? 1 : 0);
