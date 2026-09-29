#!/usr/bin/env node
// scripts/docs/stamp-frontmatter.mjs
//
// PURPOSE
//   One-time backfill: give every .md under docs/ a valid frontmatter block.
//
//   The measured state on 2026-09-28 was 1250 of 1721 files with NO frontmatter
//   at all. That is not untidiness, it is a blocker: without `kind` nothing can
//   be indexed, and without an index every file is an orphan. This script is the
//   prerequisite for gen-index.mjs doing anything useful.
//
//   It NEVER invents content. Every value it writes is derived mechanically:
//     title       <- frontmatter, else the H1, else the filename
//     kind        <- derived from the path (the only trustworthy source)
//     status      <- 'archived' if under an archive/ directory, else null
//     description <- the first prose line, trimmed, capped
//     tags        <- NEVER written (must be a human choice from a closed list)
//
//   Keys it does not know are preserved, so a hand-written key is never lost.
//   Keys the schema retired are reported, not deleted, unless --prune is given.
//
// USAGE
//   node scripts/docs/stamp-frontmatter.mjs            # dry run
//   node scripts/docs/stamp-frontmatter.mjs --write
//   node scripts/docs/stamp-frontmatter.mjs --write --prune       # drop retired keys
//   node scripts/docs/stamp-frontmatter.mjs --write --fix-kind    # correct a wrong kind
//   node scripts/docs/stamp-frontmatter.mjs --only docs/api      # scope to a subtree
//   node scripts/docs/stamp-frontmatter.mjs --write --refresh-description
//   node scripts/docs/stamp-frontmatter.mjs --json

import {
  listDocs, parseFrontmatter, serialiseFrontmatter, deriveKind, firstH1, firstProseLine,
  readDoc, writeDoc, isArchive,
} from './lib.mjs';

const WRITE = process.argv.includes('--write');
const PRUNE = process.argv.includes('--prune');
const FIX_KIND = process.argv.includes('--fix-kind');
const REFRESH_DESCRIPTION = process.argv.includes('--refresh-description');
const JSON_OUT = process.argv.includes('--json');
const ONLY = (() => {
  const i = process.argv.indexOf('--only');
  return i !== -1 ? process.argv[i + 1] : null;
})();

// Keys the 2026-09-28 schema retired. Reported always; removed only with --prune.
const RETIRED = ['type', 'last_reviewed', 'related', 'language', 'cssclasses', 'guid', 'up', 'breadcrumb', 'icon'];

/** Truncate on a word boundary so a clipped value never ends mid-word. */
function clip(s, max) {
  const t = String(s).trim();
  if (t.length <= max) return t;
  const head = t.slice(0, max - 3);
  const cut = head.lastIndexOf(' ');
  return (cut > max * 0.6 ? head.slice(0, cut) : head).replace(/[.,;:)\]]+$/, '') + '...';
}

/**
 * Normalise a tag to the one shape Obsidian can type safely: lowercase ASCII with
 * hyphens. The corpus had `búsqueda`, `híbrida`, `métricas`, `io_uring`, and
 * `2026-06` — a mix of accents, underscores and date-like strings. Obsidian
 * derives a property's type from its name vault-wide, so a tag that is
 * sometimes a date and sometimes a word is exactly the failure mode to remove.
 */
function normaliseTag(t) {
  return String(t)
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')      // strip combining accents
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')          // underscores, spaces, dots, anything else
    .replace(/^-+|-+$/g, '')
    .slice(0, 40);
}

const docs = listDocs({ includeArchive: true }).filter((d) => !ONLY || d.startsWith(ONLY));

const report = {
  scanned: docs.length,
  stamped: 0, // frontmatter added where there was none
  updated: 0, // frontmatter added where there was some (missing kind/description)
  unchanged: 0,
  kindFixed: 0,
  retiredFound: {},
  retiredRemoved: 0,
  files: [],
};

const edits = new Map();

for (const rel of docs) {
  const text = readDoc(rel);
  const fm = parseFrontmatter(text);
  const before = fm.data;
  const had = fm.hadFrontmatter;

  const next = { ...before };
  let changed = false;

  // title: trust frontmatter, then H1, then filename.
  // Truncated to the schema's maxLength, because a derived value that violates
  // the schema it is stamped under is the script's bug, not a gate finding.
  if (!next.title) {
    const h1 = firstH1(fm.body);
    const fromName = rel.split('/').pop().replace(/\.md$/i, '').replace(/[-_]+/g, ' ');
    next.title = clip(h1 ?? fromName, 120);
    changed = true;
  } else if (String(next.title).length > 120) {
    next.title = clip(String(next.title), 120);
    changed = true;
  }

  // kind: the path is the only trustworthy source
  const derived = deriveKind(rel);
  if (!next.kind || (FIX_KIND && next.kind !== derived)) {
    if (next.kind && next.kind !== derived) report.kindFixed++;
    next.kind = derived;
    changed = true;
  }

  // status: archived is a fact about the path
  if (isArchive(rel) && !next.status) {
    next.status = 'archived';
    changed = true;
  }

  // description: first prose line, links flattened to their text, capped on a
  // word boundary. A description that ends mid-URL is worse than none.
  if (!next.description || REFRESH_DESCRIPTION) {
    const prose = firstProseLine(fm.body);
    if (prose) {
      const flat = prose
        .replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1') // [HNSW](../x.md) -> HNSW
        .replace(/^\s*["'`*_]+/, '')                // stray quote or emphasis left by the source line
        .replace(/[*`_]/g, '')
        .replace(/\s+/g, ' ')
        .trim()
        .replace(/[.:;]+$/, '');
      if (flat.length > 3) {
        next.description = clip(flat, 240);
        changed = true;
      }
    }
  }

  for (const k of RETIRED) {
    if (k in before) {
      report.retiredFound[k] = (report.retiredFound[k] ?? 0) + 1;
      if (PRUNE) {
        delete next[k];
        report.retiredRemoved++;
        changed = true;
      }
    }
  }

  // tags: normalise shape. Never add a tag, never drop one -- only rewrite the
  // spelling of an existing one, and drop it only if normalisation empties it.
  if (next.tags !== undefined && next.tags !== null) {
    const before2 = Array.isArray(next.tags) ? next.tags : [next.tags];
    const after = [...new Set(before2.map(normaliseTag).filter(Boolean))];
    if (after.length !== before2.length || after.some((t, i) => t !== before2[i])) {
      if (after.length) next.tags = after;
      else delete next.tags;
      report.tagsNormalised = (report.tagsNormalised ?? 0) + 1;
      changed = true;
    }
  }

  if (!changed) {
    report.unchanged++;
    continue;
  }

  const rebuilt = serialiseFrontmatter(next) + '\n' + fm.body.replace(/^\n+/, '');
  if (rebuilt === text) {
    report.unchanged++;
    continue;
  }

  if (!had) report.stamped++;
  else report.updated++;
  report.files.push(rel);
  edits.set(rel, rebuilt);
}

if (WRITE) for (const [rel, text] of edits) writeDoc(rel, text);

if (JSON_OUT) {
  console.log(JSON.stringify({ ...report, files: report.files.length }, null, 2));
} else {
  const mode = WRITE ? 'APPLIED' : 'DRY RUN';
  console.log(`[${mode}] scanned        : ${report.scanned}`);
  console.log(`[${mode}] frontmatter added (was absent) : ${report.stamped}`);
  console.log(`[${mode}] frontmatter completed          : ${report.updated}`);
  console.log(`[${mode}] unchanged                     : ${report.unchanged}`);
  if (FIX_KIND) console.log(`[${mode}] kind corrected                : ${report.kindFixed}`);

  const rf = Object.entries(report.retiredFound);
  if (rf.length) {
    console.log(`\nRETIRED KEYS FOUND ${PRUNE ? '(removed)' : '(not removed — pass --prune)'}: ` +
      rf.map(([k, v]) => `${k}=${v}`).join(', '));
    if (!PRUNE) {
      console.log('  Rationale: type was never a taxonomy; last_reviewed is a self-reported');
      console.log('  date, not a measurement (freshness comes from git); related was a');
      console.log('  hand-maintained backlink list that gen-index.mjs now generates.');
    }
  }
  if (!WRITE && report.files.length) {
    console.log(`\nRe-run with --write to rewrite ${report.files.length} file(s).`);
  }
}
