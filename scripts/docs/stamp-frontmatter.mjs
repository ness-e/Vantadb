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
//   node scripts/docs/stamp-frontmatter.mjs --self-test   # writer assertions, no scan

import {
  listDocs,
  parseFrontmatter,
  serialiseFrontmatter,
  deriveKind,
  firstH1,
  firstProseLine,
  readDoc,
  writeDoc,
  isArchive,
} from "./lib.mjs";

const WRITE = process.argv.includes("--write");
const PRUNE = process.argv.includes("--prune");
const FIX_KIND = process.argv.includes("--fix-kind");
const REFRESH_DESCRIPTION = process.argv.includes("--refresh-description");
const JSON_OUT = process.argv.includes("--json");
const ONLY = (() => {
  const i = process.argv.indexOf("--only");
  return i !== -1 ? process.argv[i + 1] : null;
})();

// Keys the 2026-09-28 schema retired. Reported always; removed only with --prune.
const RETIRED = [
  "type",
  "last_reviewed",
  "related",
  "language",
  "cssclasses",
  "guid",
  "up",
  "breadcrumb",
  "icon",
];

/** Truncate on a word boundary so a clipped value never ends mid-word. */
function clip(s, max) {
  const t = String(s).trim();
  if (t.length <= max) return t;
  const head = t.slice(0, max - 3);
  const cut = head.lastIndexOf(" ");
  return (cut > max * 0.6 ? head.slice(0, cut) : head).replace(/[.,;:)\]]+$/, "") + "...";
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
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "") // strip combining accents
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-") // underscores, spaces, dots, anything else
    .replace(/^-+|-+$/g, "")
    .slice(0, 40);
}

/** `title` of last resort: the filename, de-slugged ("FIND-591" -> "FIND 591"). */
const nameFrom = (rel) =>
  rel
    .split("/")
    .pop()
    .replace(/\.md$/i, "")
    .replace(/[-_]+/g, " ");

/**
 * Frontmatter block + body.
 *
 * Two invariants, both learned on 2026-11-23 when this line damaged 33 files:
 *   - the block uses the file's OWN newline. An LF block on a CRLF body is a
 *     mixed file, and one newline after `---` is one blank line, not a stray one.
 *   - leading blank lines are consumed with `/^(?:\r?\n)+/`. `^\n+` leaves the
 *     `\r` behind, so every pass printed one more blank line before the H1.
 */
function compose(fm, data, text) {
  const nl = text.includes("\r\n") ? "\r\n" : "\n";
  const block = serialiseFrontmatter(data);
  const body = fm.body.replace(/^(?:\r?\n)+/, "");
  if (!body) return block; // nothing to separate: no blank line at all
  // The block already ends with one newline, so `nl` here IS the blank line
  // between frontmatter and body: exactly one, never two.
  return (nl === "\r\n" ? block.replace(/\n/g, "\r\n") : block) + nl + body;
}

/**
 * Rebuild one document's frontmatter. Pure: same input, same output, no fs.
 *
 * The whole rewrite lives here so `--self-test` exercises the REAL path. A
 * self-test that re-implements the regex stays green while the writer corrupts
 * files, which is exactly how the 2026-11-23 damage shipped.
 *
 * @returns {{ text: string, changed: boolean, had: boolean,
 *             kindFixed: number, tagsNormalised: number,
 *             retiredFound: Record<string, number>, retiredRemoved?: number }}
 */
function stampDocument(rel, text, { prune = false, fixKind = false, refreshDescription = false } = {}) {
  const fm = parseFrontmatter(text);
  const before = fm.data;
  const next = { ...before };
  const stats = { kindFixed: 0, tagsNormalised: 0, retiredFound: {} };
  let changed = false;

  // title: trust frontmatter, then H1, then filename. The H1 beats the filename
  // because a filename is a slug ("FIND-591"), not a title.
  // Truncated to the schema's maxLength, because a derived value that violates
  // the schema it is stamped under is the script's bug, not a gate finding.
  if (!next.title) {
    next.title = clip(firstH1(fm.body) ?? nameFrom(rel), 120);
    changed = true;
  } else if (String(next.title).length > 120) {
    next.title = clip(String(next.title), 120);
    changed = true;
  }

  // kind: the path is the only trustworthy source
  const derived = deriveKind(rel);
  if (!next.kind || (fixKind && next.kind !== derived)) {
    if (next.kind && next.kind !== derived) stats.kindFixed++;
    next.kind = derived;
    changed = true;
  }

  // status: archived is a fact about the path
  if (isArchive(rel) && !next.status) {
    next.status = "archived";
    changed = true;
  }

  // description: first prose line, links flattened to their text, capped on a
  // word boundary. A description that ends mid-URL is worse than none.
  if (!next.description || refreshDescription) {
    const prose = firstProseLine(fm.body);
    if (prose) {
      const flat = prose
        .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1") // [HNSW](../x.md) -> HNSW
        .replace(/^\s*["'`*_]+/, "") // stray quote or emphasis left by the source line
        .replace(/[*`_]/g, "")
        .replace(/\s+/g, " ")
        .trim()
        .replace(/[.:;]+$/, "");
      if (flat.length > 3) {
        next.description = clip(flat, 240);
        changed = true;
      }
    }
  }

  for (const k of RETIRED) {
    if (k in before) {
      stats.retiredFound[k] = 1;
      if (prune) {
        delete next[k];
        stats.retiredRemoved = (stats.retiredRemoved ?? 0) + 1;
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
      stats.tagsNormalised = 1;
      changed = true;
    }
  }

  if (!changed) return { text, changed: false, had: fm.hadFrontmatter, ...stats };

  const rebuilt = compose(fm, next, text);
  if (rebuilt === text) return { text, changed: false, had: fm.hadFrontmatter, ...stats };
  return { text: rebuilt, changed: true, had: fm.hadFrontmatter, ...stats };
}

// ---------------------------------------------------------------- self-test
//
// The three ways this script damaged 33 files on 2026-11-23, as fixtures. Each
// case asserts the writer's CONTRACT (one frontmatter block, one blank line
// before the body, the H1 as title) and that a second pass changes nothing.

if (process.argv.includes("--self-test")) {
  /**
   * Why not "count the `---` lines": a horizontal rule in the body is also a
   * `---` line, so that assertion fails for the wrong reason. The contract is
   * "the parser sees exactly one block and the body is not another one".
   */
  const oneBlock = (s) => {
    const fm = parseFrontmatter(s);
    return fm.hadFrontmatter && !/^---[ \t]*\r?\n/.test(fm.body.replace(/^\uFEFF/, ""));
  };
  /** Blank lines between the closing `---` and the first body line. */
  const blanksAfterFrontmatter = (s) => {
    const ls = s.split(/\r?\n/);
    const close = ls.findIndex((l, i) => i > 0 && l.trim() === "---");
    if (close === -1) return -1;
    let n = 0;
    while (close + 1 + n < ls.length && ls[close + 1 + n] === "") n++;
    // A trailing newline at EOF is not a blank line.
    return ls[close + 1 + n] === undefined ? n - 1 : n;
  };
  const REL = "docs/dev/tasks/FIND-591.md"; // the filename must never win over the H1

  const CASES = [
    {
      name: "BOM + frontmatter: exactly one block",
      doc: "\uFEFF---\ntitle: Real title\n---\n# Real title\n\nBody.\n",
      want: { title: "Real title", blankAfterFm: 1 },
    },
    {
      name: "BOM + H1, no frontmatter: title comes from the H1",
      doc: "\uFEFF# Real title\n\nBody.\n",
      want: { title: "Real title", blankAfterFm: 1 },
    },
    {
      name: "CRLF: no blank line accumulates before the H1",
      doc: "---\r\ntitle: Real title\r\n---\r\n\r\n# Real title\r\n\r\nBody.\r\n",
      want: { title: "Real title", blankAfterFm: 1, blockEol: "\r\n" },
    },
    {
      name: "no H1: title falls back to the filename",
      doc: "Just prose, no heading.\n",
      want: { title: "FIND 591", blankAfterFm: 1 },
    },
    {
      name: "plain LF document is stamped once",
      doc: "# Real title\n\nBody.\n",
      want: { title: "Real title", blankAfterFm: 1 },
    },
    {
      // Closing `---` as the last thing in the file (no trailing newline): the
      // parser used to report "no frontmatter" and the writer stacked a second
      // block on top, deriving `description: title: ...` from the orphan YAML.
      name: "closing --- at EOF: no second block",
      doc: "---\ntitle: Real title\n---",
      want: { title: "Real title", blankAfterFm: 0 },
    },
  ];

  let bad = 0;
  for (const c of CASES) {
    const first = stampDocument(REL, c.doc);
    const second = stampDocument(REL, first.text);
    const got = {
      title: parseFrontmatter(first.text).data.title,
      oneBlock: oneBlock(first.text),
      blankAfterFm: blanksAfterFrontmatter(first.text),
      blockEol: first.text.startsWith("---\r\n") ? "\r\n" : "\n",
    };
    const problems = [];
    if (got.title !== c.want.title) problems.push(`title "${got.title}" != "${c.want.title}"`);
    if (!got.oneBlock) problems.push("not exactly one frontmatter block");
    if (first.text.startsWith("\uFEFF")) problems.push("BOM re-emitted");
    if (got.blankAfterFm !== c.want.blankAfterFm)
      problems.push(
        `${got.blankAfterFm} blank line(s) after frontmatter, want ${c.want.blankAfterFm}`,
      );
    // The block must use the file's own newline; an LF block in a CRLF document
    // is a mixed file, and no other case in this list would notice.
    if (got.blockEol !== (c.want.blockEol ?? "\n"))
      problems.push(`block newline ${JSON.stringify(got.blockEol)} != ${JSON.stringify(c.want.blockEol ?? "\n")}`);
    if (second.changed || second.text !== first.text) problems.push("second pass is not a no-op");
    if (problems.length) bad++;
    console.log(
      `  ${problems.length ? "FAIL" : "ok  "}  ${c.name}${problems.length ? ` — ${problems.join("; ")}` : ""}`,
    );
  }
  console.log(bad ? `\nSELF-TEST FAILED: ${bad} case(s).` : "\nself-test passed.");
  process.exit(bad ? 1 : 0);
}

// --self-test exits above, before the tree is walked: the assertions need no
// corpus, and a test that reads the working tree goes red for the wrong reason.
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
  const { text: rebuilt, changed, had, kindFixed, tagsNormalised, retiredFound, retiredRemoved } =
    stampDocument(rel, text, {
      prune: PRUNE,
      fixKind: FIX_KIND,
      refreshDescription: REFRESH_DESCRIPTION,
    });

  report.kindFixed += kindFixed;
  report.retiredRemoved += retiredRemoved ?? 0;
  if (tagsNormalised) report.tagsNormalised = (report.tagsNormalised ?? 0) + tagsNormalised;
  for (const [k, n] of Object.entries(retiredFound))
    report.retiredFound[k] = (report.retiredFound[k] ?? 0) + n;

  if (!changed) {
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
  const mode = WRITE ? "APPLIED" : "DRY RUN";
  console.log(`[${mode}] scanned        : ${report.scanned}`);
  console.log(`[${mode}] frontmatter added (was absent) : ${report.stamped}`);
  console.log(`[${mode}] frontmatter completed          : ${report.updated}`);
  console.log(`[${mode}] unchanged                     : ${report.unchanged}`);
  if (FIX_KIND) console.log(`[${mode}] kind corrected                : ${report.kindFixed}`);

  const rf = Object.entries(report.retiredFound);
  if (rf.length) {
    console.log(
      `\nRETIRED KEYS FOUND ${PRUNE ? "(removed)" : "(not removed — pass --prune)"}: ` +
        rf.map(([k, v]) => `${k}=${v}`).join(", "),
    );
    if (!PRUNE) {
      console.log("  Rationale: type was never a taxonomy; last_reviewed is a self-reported");
      console.log("  date, not a measurement (freshness comes from git); related was a");
      console.log("  hand-maintained backlink list that gen-index.mjs now generates.");
    }
  }
  if (!WRITE && report.files.length) {
    console.log(`\nRe-run with --write to rewrite ${report.files.length} file(s).`);
  }
}
