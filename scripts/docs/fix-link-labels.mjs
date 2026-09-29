#!/usr/bin/env node
// scripts/docs/fix-link-labels.mjs
//
// PURPOSE
//   One-off repair for labels mangled during the wikilink migration.
//
//   `[[bm25\|BM25]]` is the Obsidian spelling for a link inside a Markdown table
//   cell, where the pipe must be escaped. When wikilinks-to-md.mjs converted the
//   target it stripped the escaping backslash, but the fallback label kept it,
//   producing `[bm25\](../glosario/bm25.md)` -- a label that renders as `bm25\`.
//
//   wikilinks-to-md.mjs is fixed for future runs; this drains what it already
//   wrote. Idempotent: a second run is a no-op.
//
// USAGE
//   node scripts/docs/fix-link-labels.mjs [--write] [--json]

import { listDocs, readDoc, writeDoc } from './lib.mjs';

const WRITE = process.argv.includes('--write');
const JSON_OUT = process.argv.includes('--json');

const docs = listDocs({ includeArchive: true });
// A label ending in a backslash, immediately before ](  —  and not an escaped
// bracket, which is legitimate Markdown.
const BROKEN = /\[([^\]\n]*?)\\+\]\(/g;

let files = 0;
let labels = 0;
const report = [];

for (const rel of docs) {
  const text = readDoc(rel);
  if (!/\\\]\(/.test(text)) continue;
  let hits = 0;
  const out = text.replace(BROKEN, (m, label) => {
    hits++;
    return `[${label.replace(/\\+$/, '')}](`;
  });
  if (hits > 0) {
    files++;
    labels += hits;
    report.push({ file: rel, labels: hits });
    if (WRITE) writeDoc(rel, out);
  }
}

if (JSON_OUT) {
  console.log(JSON.stringify({ files, labels, report }, null, 2));
} else {
  console.log(`[${WRITE ? 'APPLIED' : 'DRY RUN'}] files affected : ${files}`);
  console.log(`[${WRITE ? 'APPLIED' : 'DRY RUN'}] labels fixed    : ${labels}`);
  for (const r of report.slice(0, 20)) console.log(`  ${r.file}  (${r.labels})`);
  if (report.length > 20) console.log(`  ... and ${report.length - 20} more files`);
}
