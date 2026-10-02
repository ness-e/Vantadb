#!/usr/bin/env node
// scripts/docs/repair-links.mjs
//
// PURPOSE
//   Repair broken internal Markdown links in docs/ by re-resolving the target
//   against what is actually on disk. The measured baseline on 2026-09-28 was
//   659 broken links out of 1397 (47%), caused by moved files, case drift, ADR
//   renames and a `../..`-style wikilink that Obsidian never resolved.
//
// DESIGN
//   Only repairs a link when there is a UNIQUE, HIGH-CONFIDENCE match. Every
//   repair is reported; every unresolvable link is reported and left alone.
//   A wrong link is worse than a broken one: it silently misdirects the reader
//   and the agent, and passes every existence check downstream.
//
// CONFIDENCE LADDER (all must be unique, otherwise no repair)
//   1. exact relative path
//   2. case-insensitive relative path
//   3. ADR convention swap:  001_x.md  <->  ADR-001-x.md
//   4. separator normalise:  a_b-c.md <->  a-b-c.md
//   5. unique basename anywhere in docs/
//   6. unique stem (extension-insensitive) anywhere in docs/
//
// USAGE
//   node scripts/docs/repair-links.mjs            # dry run, exit 1 if repairs pending
//   node scripts/docs/repair-links.mjs --write    # apply
//   node scripts/docs/repair-links.mjs --json
//   node scripts/docs/repair-links.mjs --write --include-archive

import { listDocs, buildTargetSet, normalisePath, encodePath, abs, readDoc, writeDoc, toPosix } from './lib.mjs';
import { readFileSync } from 'node:fs';
import { join, dirname, relative, sep } from 'node:path';

const WRITE = process.argv.includes('--write');
const JSON_OUT = process.argv.includes('--json');
const INCLUDE_ARCHIVE = process.argv.includes('--include-archive');

const MD_LINK = /(!?\[[^\]]*\]\()([^)\s]+)(\s+"[^"]*")?(\))/g;
const WIKI_LINK = /(!?)\[\[([^\]|#]+)(?:#([^\]|]+))?(\|[^\]]*)?\]\]/g;

const { files } = buildTargetSet();
const docs = listDocs({ includeArchive: INCLUDE_ARCHIVE });

// index: several spellings -> canonical doc path
const bySpelling = new Map(); // spelling -> Set<canonical>
const addSpelling = (spelling, canonical) => {
  if (!bySpelling.has(spelling)) bySpelling.set(spelling, new Set());
  bySpelling.get(spelling).add(canonical);
};
const addVariants = (rel) => {
  const n = rel.split('/').pop();
  const stem = n.replace(/\.md$/i, '');
  addSpelling(n.toLowerCase(), rel);
  addSpelling(stem.toLowerCase(), rel);
  addSpelling(rel.toLowerCase(), rel);
  addSpelling(rel.toLowerCase().replace(/\.md$/i, ''), rel);
  // ADR convention swap and separator normalisation
  const adr = rel.match(/ADR-(\d+)[-_](.+)\.md$/i);
  if (adr) {
    addSpelling(`${adr[1]}_${adr[2]}.md`.toLowerCase(), rel);
    addSpelling(`${adr[1]}-${adr[2]}.md`.toLowerCase(), rel);
    addSpelling(`${adr[1]}_${adr[2]}`.toLowerCase(), rel);
    addSpelling(`${adr[1]}-${adr[2]}`.toLowerCase(), rel);
  }
  const bare = rel.match(/(?:^|\/)(\d+)[-_](.+)\.md$/);
  if (bare) {
    addSpelling(`ADR-${bare[1]}-${bare[2]}.md`.toLowerCase(), rel);
    addSpelling(`adr-${bare[1]}-${bare[2]}.md`.toLowerCase(), rel);
  }
};
for (const d of docs) addVariants(d);

const unique = (spelling) => {
  const s = bySpelling.get(String(spelling).toLowerCase());
  if (!s || s.size === 0) return null;
  if (s.size === 1) return [...s][0];
  return null; // ambiguous -> refuse
};

const report = { scanned: 0, filesChanged: 0, repaired: 0, repairedWikilinks: 0, unresolved: [], skippedAmbiguous: [] };
const changes = new Map();

for (const rel of docs) {
  report.scanned++;
  const text = readDoc(rel);
  if (!/]\(|\[\[/.test(text)) continue;
  const dir = dirname(rel);
  let touched = false;

  const fixTarget = (raw) => {
    // `[[target\|Alias]]` inside a table cell leaves a trailing backslash on the
    // target. Strip it before resolving, or every such link is unrepairable.
    const [pathPart, frag] = raw.replace(/\\+$/, '').split('#');
    if (!pathPart) return null;
    let decoded = pathPart;
    try { decoded = decodeURIComponent(pathPart); } catch { /* keep raw */ }
    const candidate = decoded.startsWith('/') ? normalisePath(decoded) : normalisePath(`${dir}/${decoded}`);
    if (files.has(candidate)) return null; // already fine
    if (files.has(candidate + '.md')) return null;

    // Ladder. A sibling wins over any global basename match -- this mirrors
    // Obsidian's `newLinkFormat: "relative"` semantics and is what makes
    // `[[README.md]]` inside a subdirectory resolve to that subdirectory's
    // README instead of one of the 15 others in the vault.
    const tries = [
      candidate,
      candidate.replace(/\.md$/i, ''),
      `${dir}/${candidate.split('/').pop()}`,
      `${dir}/${candidate.split('/').pop().replace(/\.md$/i, '')}`,
      candidate.split('/').pop(),
      candidate.split('/').pop().replace(/\.md$/i, ''),
    ];
    for (const t of tries) {
      const hit = unique(t);
      if (hit) return hit;
    }
    // ambiguous spellings
    for (const t of tries) {
      const s = bySpelling.get(String(t).toLowerCase());
      if (s && s.size > 1) {
        report.skippedAmbiguous.push({ file: rel, target: raw, candidates: [...s] });
        return null;
      }
    }
    report.unresolved.push({ file: rel, target: raw });
    return null;
  };

  let out = text.replace(MD_LINK, (m, open, target, title, close) => {
    if (/^(https?:|mailto:|tel:|data:|obsidian:|#|\/\/)/i.test(target)) return m;
    const hit = fixTarget(target);
    if (!hit) return m;
    const frag = target.includes('#') ? '#' + target.split('#').slice(1).join('#') : '';
    let p = toPosix(relative(dir, hit));
    if (!p.startsWith('.')) p = './' + p;
    touched = true;
    report.repaired++;
    return `${open}${encodePath(p)}${frag}${title ?? ''}${close}`;
  });

  out = out.replace(WIKI_LINK, (m, bang, target, anchor) => {
    if (bang) return m;
    const hit = fixTarget(target);
    if (!hit) return m;
    let p = toPosix(relative(dir, hit));
    if (!p.startsWith('.')) p = './' + p;
    const suffix = anchor ? '#' + anchor : '';
    touched = true;
    report.repairedWikilinks++;
    return `[${target.trim().split('/').pop()}](${encodePath(p)}${suffix})`;
  });

  if (touched) changes.set(rel, out);
}

report.filesChanged = changes.size;
if (WRITE) for (const [rel, text] of changes) writeDoc(rel, text);

if (JSON_OUT) {
  console.log(JSON.stringify(report, null, 2));
} else {
  const mode = WRITE ? 'APPLIED' : 'DRY RUN';
  console.log(`[${mode}] files scanned  : ${report.scanned}`);
  console.log(`[${mode}] files changed  : ${report.filesChanged}`);
  console.log(`[${mode}] md links fixed : ${report.repaired}`);
  console.log(`[${mode}] wikilinks fixed: ${report.repairedWikilinks}`);

  const uniq = [...new Map(report.unresolved.map((u) => [`${u.file}|${u.target}`, u])).values()];
  if (uniq.length) {
    console.log(`\nUNRESOLVED (${report.unresolved.length}, ${uniq.length} unique) — no safe repair:`);
    for (const u of uniq.slice(0, 30)) console.log(`  ${u.file}  -> ${u.target}`);
    if (uniq.length > 30) console.log(`  ... and ${uniq.length - 30} more`);
  }
  const amb = [...new Map(report.skippedAmbiguous.map((u) => [`${u.file}|${u.target}`, u])).values()];
  if (amb.length) {
    console.log(`\nAMBIGUOUS (${amb.length}) — manual decision needed:`);
    for (const a of amb.slice(0, 15)) {
      console.log(`  ${a.file}  -> ${a.target}`);
      console.log(`      ${a.candidates.slice(0, 4).join(', ')}${a.candidates.length > 4 ? ', ...' : ''}`);
    }
    if (amb.length > 15) console.log(`  ... and ${amb.length - 15} more`);
  }
  if (!WRITE && report.filesChanged) console.log(`\nRe-run with --write to apply ${report.filesChanged} file(s).`);
}

process.exit(!WRITE && (report.filesChanged || report.unresolved.length) ? 1 : 0);
