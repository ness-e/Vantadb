#!/usr/bin/env node
// scripts/docs/wikilinks-to-md.mjs
//
// PURPOSE
//   Convert Obsidian wikilinks ([[Note]]) into GitHub-compatible relative
//   Markdown links ([Label](path.md)) so that docs/ renders correctly on
//   GitHub while Obsidian's graph view, backlinks panel and unlinked mentions
//   keep working (Obsidian treats both forms as "internal links").
//
//   Reference: https://obsidian.md/help/links
//     "If interoperability is important to you, you can disable Wikilinks and
//      use Markdown links instead."
//
// WHY NOT THE OBSIDIAN PLUGIN
//   The "Better Markdown Links" plugin (mnaoumov/obsidian-better-markdown-links,
//   MIT) is the interactive equivalent and is recommended for manual use. This
//   script exists because the conversion must be reproducible and reviewable in
//   CI, and because a bulk rename of 650 links should land as one reviewable
//   diff rather than as 140 silent editor writes.
//
// SAFETY PROPERTIES
//   - Idempotent: a second run is a no-op (no [[...]] left to convert).
//   - Never guesses: an unresolvable wikilink is REPORTED and LEFT UNTOUCHED.
//     Converting it to a wrong path would be worse than leaving it broken.
//   - Ambiguity is explicit: a basename that matches N>1 files is reported, and
//     resolved via a deterministic tie-break (deepest-first, then lexicographic)
//     only if --allow-ambiguous is passed.
//   - Only rewrites files that actually change.
//   - Never touches front matter, fenced code blocks, or inline code spans
//     (via segment() from lib.mjs -- the same helper the checkers use).
//
// USAGE
//   node scripts/docs/wikilinks-to-md.mjs            # dry run, exit 1 if work pending
//   node scripts/docs/wikilinks-to-md.mjs --write    # apply
//   node scripts/docs/wikilinks-to-md.mjs --json     # machine-readable report
//   node scripts/docs/wikilinks-to-md.mjs --write --allow-ambiguous
//
// EXIT CODES
//   0 = converged (nothing left to convert, or --write applied cleanly)
//   1 = pending work (dry run found conversions) OR unresolved links remain
//   2 = bad usage

import { readdirSync, readFileSync } from 'node:fs';
import { join, dirname, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { segment, segmentsRoundTrip, normalisePath } from './lib.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const DOCS = join(ROOT, 'docs');
const WRITE = process.argv.includes('--write');
const JSON_OUT = process.argv.includes('--json');
const ALLOW_AMBIGUOUS = process.argv.includes('--allow-ambiguous');

if (process.argv.includes('--help') || process.argv.includes('-h')) {
  console.log('usage: wikilinks-to-md.mjs [--write] [--json] [--allow-ambiguous]');
  process.exit(0);
}

// ---------------------------------------------------------------- file index

/** @type {string[]} */
const allMd = [];
(function walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === '.obsidian' || entry.name === 'node_modules') continue;
    const full = join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.name.endsWith('.md')) allMd.push(full);
  }
})(DOCS);

const toRel = (abs) => relative(ROOT, abs).split(sep).join('/');

/** vault-relative POSIX path -> absolute path */
const byRelPath = new Map();
/** lowercase basename (with and without .md) -> vault-relative paths[] */
const byBase = new Map();
/** lowercase vault-relative path (no ext) -> vault-relative path */
const byNoExt = new Map();

/** Index a basename under both "x.md" and "x" so [[x]] and [[x.md]] both hit. */
function indexBase(key, value) {
  const k = key.toLowerCase();
  if (!byBase.has(k)) byBase.set(k, []);
  if (!byBase.get(k).includes(value)) byBase.get(k).push(value);
}

for (const abs of allMd) {
  const rel = toRel(abs);
  byRelPath.set(rel, abs);
  byRelPath.set(rel.toLowerCase(), abs);

  const relNoExt = rel.replace(/\.md$/i, '');
  byNoExt.set(relNoExt.toLowerCase(), rel);

  const file = rel.split('/').pop();
  indexBase(file, rel);
  indexBase(file.replace(/\.md$/i, ''), rel);
}

// ------------------------------------------------------------- link parsing

// [[target]] [[target|alias]] [[target#anchor]] [[dir/target]]
const WIKILINK = /(!?)\[\[([^\]|#]+?)(?:#([^\]|]+))?(\|[^\]]*)?\]\]/g;

function encodePath(p) {
  // Encode spaces and characters that break Markdown link targets, but keep
  // path separators and the safe filename alphabet intact.
  return p
    .split('/')
    .map((seg) => encodeURIComponent(seg).replace(/%2F/gi, '/'))
    .join('/');
}

/** Look a candidate vault-relative path up, exact then case-insensitive. */
function lookup(cand) {
  if (byRelPath.has(cand)) return byRelPath.get(cand);
  if (byNoExt.has(cand)) return byNoExt.get(cand);
  const cl = cand.toLowerCase();
  if (byRelPath.has(cl)) return byRelPath.get(cl);
  if (byNoExt.has(cl)) return byNoExt.get(cl);
  return null;
}

/** The single canonical path for a spelling, or null when it is ambiguous. */
const unique = (spelling) => {
  const s = byBase.get(String(spelling).toLowerCase());
  if (!s || s.size === 0) return null;
  if (s.size === 1) return [...s][0];
  return null;
};

/**
 * Resolve a wikilink target to a vault-relative path, or null.
 * `srcDir` is the vault-relative directory of the file containing the link.
 *
 * Resolution order:
 *   1. Explicitly file-relative ("./x.md", "../x.md") -> resolved against srcDir.
 *      Obsidian wikilinks do NOT support "../" natively, so these were already
 *      broken; the author's intent is unambiguous, so honour it.
 *   2. Sibling in the same directory. Handles the very common
 *      [[README.md]] written from a subdirectory, where 16 READMEs exist.
 *   3. Vault-relative path (with or without .md, case-insensitive).
 *   4. Unique basename anywhere in the vault.
 *   5. Ambiguous basename -> null unless --allow-ambiguous.
 */
function resolve(targetRaw, srcDir) {
  const t = targetRaw.trim().replace(/\\/g, '/');
  const base = t.split('/').pop();

  if (t.startsWith('./') || t.startsWith('../')) {
    const hit = lookup(normalisePath(`${srcDir}/${t}`));
    if (hit) return hit;
  }

  const sib = lookup(srcDir ? `${srcDir}/${t}` : t);
  if (sib) return sib;

  const vr = lookup(t);
  if (vr) return vr;

  const cands = byBase.get(base.toLowerCase());
  if (!cands || cands.length === 0) return null;
  if (cands.length === 1) return cands[0];

  const sorted = [...cands].sort(
    (a, b) => a.split('/').length - b.split('/').length || a.localeCompare(b),
  );
  return ALLOW_AMBIGUOUS ? sorted[0] : null;
}

/** Candidates for a target that could not be resolved, for the report. */
function candidatesFor(targetRaw, srcDir) {
  const t = targetRaw.trim().replace(/\\/g, '/');
  const base = t.split('/').pop().toLowerCase();
  if (t.startsWith('./') || t.startsWith('../')) {
    const norm = normalisePath(`${srcDir}/${t}`);
    return byNoExt.has(norm.toLowerCase()) ? [byNoExt.get(norm.toLowerCase())] : null;
  }
  const cands = byBase.get(base) ?? byBase.get(base.replace(/\.md$/i, '')) ?? [];
  return cands.length ? cands : null;
}

// ---------------------------------------------------------------- self-test
//
// The single runnable check this script needs. It is a check on the WRITER, not
// on the content: `segment()` must be lossless and order-preserving, because
// this script concatenates segments back into a file. A segmenter that reorders
// or drops text corrupts every document it touches while leaving every read-only
// checker perfectly green.

if (process.argv.includes('--self-test')) {
  const CASES = [
    ['plain prose only', 'a\nb\nc\n'],
    ['prose interleaved with inline code', 'plain one\nhas `code` here\nplain two\n'],
    ['code fence in the middle', 'a\n```py\n[[x]]\n```\nb\n'],
    ['frontmatter then heading', '---\ntitle: t\n---\n# H\n\nbody `c`\n'],
    ['no trailing newline', 'a\nb'],
    ['only inline code', '`a` `b`\n'],
    ['fence at end of file', 'a\n```\nb\n'],
    ['table row then inline code', '| a | b |\n|---|---|\ntext `x`\n'],
    ['CRLF frontmatter', '---\r\ntitle: t\r\n---\r\n# H\r\n\r\nbody `c`\r\n'],
    ['wikilink inside inline code', 'see `[[not a link]]` here\n'],
  ];
  let bad = 0;
  for (const [name, src] of CASES) {
    const ok = segmentsRoundTrip(src);
    if (!ok) bad++;
    console.log(`  ${ok ? 'ok  ' : 'FAIL'}  ${name}`);
  }
  // And the real property this script depends on: inline code is never rewritten.
  const guarded = 'see `[[keep]]` and [[convert]]\n';
  const kept = segment(guarded)
    .filter((s) => s.why === 'inline-code')
    .every((s) => s.text === '`[[keep]]`');
  if (!kept) bad++;
  console.log(`  ${kept ? 'ok  ' : 'FAIL'}  wikilinks inside inline code are not rewritten`);
  console.log(bad ? `\nSELF-TEST FAILED: ${bad} case(s).` : '\nself-test passed.');
  process.exit(bad ? 1 : 0);
}

// ---------------------------------------------------------------- conversion

const report = {
  filesScanned: allMd.length,
  filesChanged: 0,
  linksConverted: 0,
  skippedCodeFence: 0,
  skippedFrontmatter: 0,
  skippedInlineCode: 0,
  unresolved: [],
  ambiguous: [],
  nonPortable: [],
  segmentRoundTripFailures: [],
};

const conversions = new Map();

for (const abs of allMd) {
  const rel = toRel(abs);
  const src = readFileSync(abs, 'utf8');
  if (!src.includes('[[')) continue;

  const srcDir = dirname(rel);
  let touched = false;
  const skipKey = {
    'code-fence': 'skippedCodeFence',
    frontmatter: 'skippedFrontmatter',
    'inline-code': 'skippedInlineCode',
  };

  // Hard invariant, asserted on every real file before anything is written:
  // segment() must be lossless and order-preserving. An earlier version of
  // segment() buffered code-free lines and flushed them at the end, so every
  // file that mixed prose with inline code had its prose relocated to the
  // bottom. The checkers could not see it (proseOf() does not care about
  // order); only the write-back path reproduced it, and it destroyed 32 files.
  if (!segmentsRoundTrip(src)) {
    report.segmentRoundTripFailures.push(rel);
    continue;
  }

  const out = segment(src)
    .map((seg) => {
      if (!seg.safe) {
        const k = skipKey[seg.why];
        if (k) report[k] += (seg.text.match(/\[\[/g) ?? []).length;
        return seg.text;
      }
      if (!seg.text.includes('[[')) return seg.text;

      return seg.text.replace(WIKILINK, (match, bang, targetRaw, anchor, aliasPart) => {
        // Obsidian embeds (![[x]]) have no GitHub equivalent. There are none in
        // this repo today; if one appears, leave it and report rather than guess.
        if (bang) {
          report.nonPortable.push({ file: rel, target: targetRaw.trim(), kind: 'embed' });
          return match;
        }

        const hit = resolve(targetRaw, srcDir);
        if (!hit) {
          const cands = candidatesFor(targetRaw, srcDir);
          if (cands && cands.length > 1) {
            report.ambiguous.push({ file: rel, target: targetRaw.trim(), candidates: cands });
          } else {
            report.unresolved.push({ file: rel, target: targetRaw.trim() });
          }
          return match;
        }

        let relPath = relative(srcDir, hit).split(sep).join('/');
        if (!relPath.startsWith('.')) relPath = './' + relPath;

        // Block references (#^id) are Obsidian-only per Obsidian's own docs; keep
        // them but flag so the reviewer can decide.
        if (anchor && anchor.trim().startsWith('^')) {
          report.nonPortable.push({ file: rel, target: targetRaw.trim(), kind: 'block-ref' });
        }

        // `[[bm25\|BM25]]` inside a table cell: the backslash escapes the pipe. It
        // must be stripped from BOTH the target and the fallback label, or the
        // emitted link text becomes `bm25\`.
        const label =
          (aliasPart ?? '').replace(/^\|/, '').replace(/\\+$/, '').trim() ||
          targetRaw.trim().replace(/\\+$/, '').split('/').pop();
        const suffix = anchor ? `#${anchor.trim()}` : '';
        const replacement = `[${label}](${encodePath(relPath)}${suffix})`;

        touched = true;
        report.linksConverted++;
        return replacement;
      });
    })
    .join('');

  if (touched) conversions.set(abs, out);
}

report.filesChanged = conversions.size;

// Refuse to write anything if the round-trip invariant failed on any file. A
// partial write plus a silent corruption is worse than no write.
if (report.segmentRoundTripFailures.length) {
  console.error(
    `\nABORT: segment() is not lossless on ${report.segmentRoundTripFailures.length} file(s). ` +
      `Nothing was written. This is a bug in lib.mjs, not in the content.`,
  );
  for (const f of report.segmentRoundTripFailures.slice(0, 20)) console.error(`  ${f}`);
  process.exit(1);
}

if (WRITE) {
  for (const [abs, content] of conversions) {
    (await import('node:fs')).writeFileSync(abs, content, 'utf8');
  }
}

// -------------------------------------------------------------------- output

if (JSON_OUT) {
  console.log(JSON.stringify(report, null, 2));
} else {
  const mode = WRITE ? 'APPLIED' : 'DRY RUN';
  console.log(`[${mode}] files scanned   : ${report.filesScanned}`);
  console.log(`[${mode}] files changed   : ${report.filesChanged}`);
  console.log(`[${mode}] links converted : ${report.linksConverted}`);
  console.log(
    `[${mode}] left untouched  : ${report.skippedCodeFence} in code fences, ` +
      `${report.skippedFrontmatter} in front matter, ${report.skippedInlineCode} in inline code`,
  );

  if (report.ambiguous.length) {
    console.log(`\nAMBIGUOUS (${report.ambiguous.length}) — basename matches >1 file:`);
    const seen = new Set();
    for (const a of report.ambiguous) {
      const key = `${a.file}|${a.target}`;
      if (seen.has(key)) continue;
      seen.add(key);
      console.log(`  ${a.file}  [[${a.target}]]`);
      console.log(`      candidates: ${a.candidates.join(', ')}`);
    }
  }

  if (report.unresolved.length) {
    const uniq = [...new Map(report.unresolved.map((u) => [`${u.file}|${u.target}`, u])).values()];
    console.log(`\nUNRESOLVED (${report.unresolved.length}, ${uniq.length} unique) — left untouched:`);
    for (const u of uniq.slice(0, 40)) console.log(`  ${u.file}  [[${u.target}]]`);
    if (uniq.length > 40) console.log(`  ... and ${uniq.length - 40} more`);
  }

  if (report.nonPortable.length) {
    console.log(`\nOBSIDIAN-ONLY (${report.nonPortable.length}) — kept, not portable to GitHub:`);
    for (const n of [...new Set(report.nonPortable.map((x) => `${x.file}|${x.target}|${x.kind}`))]) {
      console.log(`  ${n.replace(/\|/g, '  ')}`);
    }
  }

  if (!WRITE && report.filesChanged) {
    console.log(`\nRe-run with --write to apply ${report.filesChanged} file change(s).`);
  }
}

const ok =
  !report.unresolved.length && !report.ambiguous.length && (WRITE || report.filesChanged === 0);
process.exit(ok ? 0 : 1);
