// scripts/docs/lib.mjs
//
// Shared helpers for the docs/ tooling. Zero dependencies, Node >= 18.
//
// Everything the documentation gates need to agree on lives here: how a file is
// found, how frontmatter is read and written, and how a document's `kind` is
// derived from its path. Keeping the derivation in one place is what makes
// "kind decides the path" enforceable instead of aspirational.

import { readdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join, dirname, relative, sep, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
export const DOCS = join(ROOT, 'docs');
export const SCHEMA = join(DOCS, '_schema', 'frontmatter.schema.json');
export const TAGS = join(DOCS, '_schema', 'tags.txt');

const SKIP_DIR = new Set(['.obsidian', 'node_modules', '_site', 'dist']);
const ARCHIVE_RE = /(^|\/)(archive|target)(\/|$)/;

// ------------------------------------------------------------------- walking

/** @returns {string[]} repo-relative POSIX paths of every .md under docs/ */
/** Repo-relative paths of git-ignored files under docs/ (empty when git is unavailable). */
function ignoredDocs() {
  try {
    const raw = execFileSync(
      'git',
      ['ls-files', '--others', '--ignored', '--exclude-standard', '-z', '--', 'docs'],
      { cwd: ROOT, encoding: 'utf8' },
    );
    return new Set(raw.split('\0').filter(Boolean));
  } catch {
    return new Set();
  }
}

export function listDocs({ includeArchive = true } = {}) {
  const out = [];
  (function walk(dir) {
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      if (SKIP_DIR.has(e.name)) continue;
      const full = join(dir, e.name);
      if (e.isDirectory()) walk(full);
      else if (e.name.endsWith('.md')) {
        const rel = toPosix(relative(ROOT, full));
        if (includeArchive || !ARCHIVE_RE.test(rel)) out.push(rel);
      }
    }
  })(DOCS);
  // CI parity: git-ignored files exist locally but never in the repo, so indexing
  // them makes the generated indexes drift from CI (which regenerates from a clean
  // checkout) — e.g. `docs/user/discord/todo.md` (.gitignore `todo.md`). Untracked
  // but NOT ignored files stay: a new doc must enter the index before it is committed.
  const ignored = ignoredDocs();
  return out.filter((p) => !ignored.has(p)).sort();
}

export const toPosix = (p) => p.split(sep).join('/');
export const abs = (rel) => join(ROOT, rel);
export const readDoc = (rel) => readFileSync(abs(rel), 'utf8');
export const writeDoc = (rel, text) => writeFileSync(abs(rel), text, 'utf8');

export const isArchive = (rel) => ARCHIVE_RE.test(rel);

/**
 * Every repo-relative path a Markdown link may resolve to.
 *
 * Scope is the WHOLE repository, not just docs/. Documents legitimately link
 * out of the tree: `docs/api/MCP.md` -> `../../vantadb-mcp/src/error.rs` is a
 * correct GitHub link. Indexing only docs/ would report those as broken.
 */
export function buildTargetSet() {
  const files = new Set();
  const dirs = new Set();
  const skip = new Set(['.git', 'node_modules', 'target', '.obsidian', '_site', 'dist', '.next', 'vendor']);
  (function walk(dir, depth) {
    if (depth > 8) return; // bound the walk; docs never nest this deep
    const rel = toPosix(relative(ROOT, dir));
    dirs.add(rel);
    let entries;
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      return; // unreadable directory (permissions) - not our problem
    }
    for (const e of entries) {
      if (skip.has(e.name)) continue;
      const full = join(dir, e.name);
      const r = toPosix(relative(ROOT, full));
      files.add(r);
      if (e.isDirectory()) walk(full, depth + 1);
      else if (e.name.endsWith('.md')) files.add(r.replace(/\.md$/i, ''));
    }
  })(ROOT, 0);
  return { files, dirs };
}

// --------------------------------------------------------------- frontmatter

/**
 * Parse the leading YAML frontmatter block without a YAML dependency.
 * Handles the subset that is legal in this vault: scalars, inline arrays,
 * block arrays of scalars. Returns { raw, data, body, hadFrontmatter }.
 */
export function parseFrontmatter(text) {
  if (!/^---\r?\n/.test(text)) {
    return { raw: null, data: {}, body: text, hadFrontmatter: false, end: 0 };
  }
  const nl = text[3] === '\r' ? '\r\n' : '\n';
  const close = text.indexOf(nl + '---' + nl, 3);
  if (close === -1) {
    return { raw: null, data: {}, body: text, hadFrontmatter: false, end: 0 };
  }
  const raw = text.slice(text.indexOf(nl) + nl.length, close);
  const end = close + (nl + '---' + nl).length;
  return { raw, data: parseYaml(raw), body: text.slice(end), hadFrontmatter: true, end, nl };
}

/** Minimal YAML subset parser: `key: value`, `key: [a, b]`, `- item` blocks. */
function parseYaml(raw) {
  const out = {};
  const lines = raw.split(/\r?\n/);
  let currentKey = null;
  for (const line of lines) {
    if (!line.trim() || line.trimStart().startsWith('#')) continue;
    const m = line.match(/^([A-Za-z0-9_.-]+):\s*(.*)$/);
    if (m) {
      const [, k, v] = m;
      currentKey = k;
      if (v === '') out[k] = null;
      else if (v.startsWith('[') && v.endsWith(']')) {
        out[k] = v
          .slice(1, -1)
          .split(',')
          .map((s) => unquote(s.trim()))
          .filter((s) => s.length);
      } else out[k] = unquote(v.trim());
      continue;
    }
    const li = line.match(/^\s*-\s+(.*)$/);
    if (li && currentKey) {
      if (!Array.isArray(out[currentKey])) out[currentKey] = out[currentKey] == null ? [] : [out[currentKey]];
      out[currentKey].push(unquote(li[1].trim()));
    }
  }
  return out;
}

const unquote = (s) => s.replace(/^["'](.*)["']$/, '$1');

/**
 * Serialise frontmatter in the canonical key order. Comments are not preserved;
 * a file rewritten by stamp-frontmatter loses any hand-written frontmatter
 * comments. That is accepted: the schema is the documentation, not the comments.
 */
export function serialiseFrontmatter(data) {
  const ORDER = [
    'title',
    'kind',
    'status',
    'description',
    'aliases',
    'tags',
    'supersedes',
    'superseded_by',
  ];
  const lines = ['---'];
  for (const k of ORDER) {
    if (!(k in data)) continue;
    const v = data[k];
    if (v === null || v === undefined || v === '') continue;
    if (Array.isArray(v)) {
      if (!v.length) continue;
      lines.push(`${k}: [${v.map((x) => quoteIfNeeded(String(x))).join(', ')}]`);
    } else {
      lines.push(`${k}: ${quoteIfNeeded(String(v))}`);
    }
  }
  // preserve unknown keys at the end so nothing is silently dropped
  for (const [k, v] of Object.entries(data)) {
    if (ORDER.includes(k)) continue;
    if (v === null || v === undefined || v === '') continue;
    if (Array.isArray(v)) lines.push(`${k}: [${v.map((x) => quoteIfNeeded(String(x))).join(', ')}]`);
    else lines.push(`${k}: ${quoteIfNeeded(String(v))}`);
  }
  lines.push('---', '');
  return lines.join('\n');
}

const needsQuote = (s) =>
  /^[\d]/.test(s) ||
  /[:#\[\]{}&*!|>'"%@`,]/.test(s) ||
  /^(true|false|null|yes|no|on|off)$/i.test(s) ||
  s !== s.trim();

const quoteIfNeeded = (s) => (needsQuote(s) ? `"${s.replace(/"/g, '\\"')}"` : s);

// ---------------------------------------------------------------- kind / path

/**
 * Derive a document's `kind` from its path. This is the single source of truth
 * for classification; frontmatter `kind:` must agree with it (rule 6 in
 * docs/dev/workflow/gate-docs.md).
 *
 * Order matters: the most specific pattern wins.
 */
export function deriveKind(rel) {
  const p = '/' + rel;
  const name = basename(rel).toLowerCase();

  // An index file is an index wherever it lives. Checked first: a generated
  // `docs/api/index.md` would otherwise be classified as a reference page.
  if (name === 'readme.md' || name === 'index.md') return 'index';

  if (/\/adr\/|\/architecture\/adr\//.test(p)) return 'adr';
  if (/^changelog\.md$/i.test(name) || /\/changelog/.test(name)) return 'changelog';
  if (/\/glosario\/|\/glossary\//.test(p)) return 'glossary';
  if (/\/tasks?\//.test(p)) return 'task';
  if (/\/plans?\//.test(p)) return 'plan';
  if (/\/research\//.test(p)) return 'research';
  if (/\/reviews?\//.test(p)) return 'review';
  if (/\/reports?\//.test(p)) return 'report';
  if (/\/(operations|runbooks?|workflow)\//.test(p)) return 'runbook';
  if (/\/avance\//.test(p)) return 'review';
  if (/\/api\//.test(p)) return 'reference';
  if (/\/(concepts?|architecture|strategy|vision)\//.test(p)) return 'concept';
  if (/\/tutorials?\//.test(p)) return 'tutorial';
  if (/\/(guides?|howto)\//.test(p)) return 'howto';
  if (/\/benchmarks?\//.test(p)) return 'report';
  if (/\/(user|desktop|discord)\//.test(p)) return 'howto';
  if (/\/dev\//.test(p)) return 'research';
  return 'concept';
}

/** Directories whose own README acts as the section index. */
export function isIndexCandidate(rel) {
  const n = basename(rel).toLowerCase();
  return n === 'readme.md' || n === 'index.md';
}

/** First H1 in the body, or null. */
export function firstH1(body) {
  const m = body.match(/^#\s+(.+)$/m);
  return m ? m[1].trim() : null;
}

/**
 * First sentence-like line of prose, for use as a `description`.
 *
 * Skips more than a naive filter would, because a derived description that reads
 * like "- **Plan file:** docs/dev/plans/x.md" is worse than no description at all.
 * Skipped: headings, blockquotes, lists (bullet, numbered, task, and bold-key
 * metadata rows), tables, code, images, rules, and the body of a `## Metadata`
 * section, which in this repo is a list of bold-key rows rather than prose.
 */
export function firstProseLine(body) {
  let inFence = false;
  let inMetadataSection = false;
  for (const line of body.split(/\r?\n/)) {
    const t = line.trim();
    if (/^\s*(```|~~~)/.test(t)) { inFence = !inFence; continue; }
    if (inFence) continue;
    if (/^#{1,6}\s/.test(t)) {
      inMetadataSection = /^#{1,6}\s*(metadata|metadatos|meta)\b/i.test(t);
      continue;
    }
    if (inMetadataSection) continue;
    if (!t) continue;
    if (t.startsWith('>') || t.startsWith('|')) continue;
    if (/^[-*+]\s/.test(t)) continue;
    if (/^\d+[.)]\s/.test(t)) continue;
    if (/^\[[ xX]\]/.test(t)) continue;
    if (/^(!\[|\[!)/.test(t)) continue;
    if (/^(---|===|\*\*\*|___)/.test(t)) continue;
    if (/^<!--/.test(t)) continue;
    if (/^\{/.test(t)) continue; // JSON/YAML body
    // The generated-index banner is a multi-line HTML comment. Without this, the
    // derived description of a generated index becomes "Run: node
    // scripts/docs/gen-index.mjs --write", which is a command, not a summary.
    if (/^Run: node /.test(t)) continue;
    return t;
  }
  return null;
}

// -------------------------------------------------------------------- schema

export function loadSchema() {
  return JSON.parse(readFileSync(SCHEMA, 'utf8'));
}

export function loadTagVocabulary() {
  if (!existsSync(TAGS)) return null;
  return new Set(
    readFileSync(TAGS, 'utf8')
      .split(/\r?\n/)
      .map((s) => s.trim())
      .filter((s) => s && !s.startsWith('#')),
  );
}

/**
 * Validate one frontmatter object against the schema.
 *
 * Returns { errors, warnings }.
 *   errors   - structural problems: missing required key, wrong type, bad enum,
 *              bad tag format, unresolvable supersedes target. These gate.
 *   warnings - unknown keys. Reported, not gated: the corpus carried ~57 ad-hoc
 *              keys on 2026-09-28 and 1000+ of the files are task files whose
 *              frontmatter the campaign system may read. A gate that starts red
 *              gets switched off, so unknown keys get drained first and the
 *              schema stays strict. Migration path: rename to `x-<name>`.
 *
 * Deliberately not a full JSON Schema implementation: it covers exactly the
 * keywords the schema uses, so it cannot silently drift from the schema file
 * without also being edited here.
 */
export function validateFrontmatter(data, schema, tagVocab) {
  const problems = [];
  const warnings = [];
  const props = schema.properties;

  for (const req of schema.required ?? []) {
    if (data[req] === undefined || data[req] === null || data[req] === '') {
      problems.push(`missing required key: ${req}`);
    }
  }

  for (const k of Object.keys(data)) {
    if (props[k]) continue;
    if (k.startsWith('x-')) continue; // documented escape hatch
    warnings.push(`unknown key: ${k} (rename to x-${k} to acknowledge it deliberately)`);
  }

  for (const [k, v] of Object.entries(data)) {
    const spec = props[k];
    if (!spec) continue;
    if (v === null || v === '') continue;
    if (spec.enum && !spec.enum.includes(String(v))) {
      // A non-enum `status` is a warning, not an error. The enum exists to make
      // the vocabulary visible; the gate's job is to catch a missing or mistyped
      // required key and a kind/path disagreement. 48 files carried ad-hoc
      // status values on 2026-09-28 and an unknown value simply sorts as
      // "unknown" in the generated index. Gate it after the corpus is drained.
      const msg = `${k}: "${String(v).slice(0, 60)}" not in [${spec.enum.join(', ')}]`;
      if (k === 'status') warnings.push(msg);
      else problems.push(msg);
    }
    if (spec.type === 'array' && !Array.isArray(v)) {
      problems.push(`${k}: must be a list, got ${typeof v} (list vs scalar changes the type Obsidian shows)`);
    }
    if (spec.type === 'string' && Array.isArray(v)) {
      problems.push(`${k}: must be a scalar, got a list`);
    }
    if (spec.maxLength && String(v).length > spec.maxLength) {
      problems.push(`${k}: ${String(v).length} chars exceeds maxLength ${spec.maxLength}`);
    }
    if (k === 'tags') {
      // Only the FORMAT is enforced. Obsidian resolves a property's type from its
      // name vault-wide, so a stray value ("2026-06", "Híbrida", a date-like
      // string) corrupts the display of every file using `tags`. Membership in
      // tags.txt is advisory: the corpus used ~430 distinct tags on 2026-09-28,
      // and a 430-entry list is not a vocabulary. Rationalising tags is a human
      // decision, tracked in docs/dev/plans/, not a gate.
      const list = Array.isArray(v) ? v : [v];
      for (const t of list) {
        if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(String(t))) {
          problems.push(
            `tags: "${t}" must be lowercase-hyphenated (Obsidian derives the ` +
              `property type from the name vault-wide)`,
          );
        }
      }
    }
    if ((k === 'supersedes' || k === 'superseded_by') && String(v).length) {
      // Only validate something that is actually shaped like a repo path. A bare
      // date or a bare number in this field is a human mistake worth reporting,
      // not a missing file worth failing the build over.
      const val = String(v);
      if (!/[/\\]|\.md$/i.test(val)) {
        warnings.push(`${k}: "${val.slice(0, 40)}" is not a path (expected docs/... or a .md file)`);
      } else if (!existsSync(abs(val))) {
        problems.push(`${k}: target does not exist: ${val}`);
      }
    }
  }
  return { errors: problems, warnings };
}

// ---------------------------------------------------------------- link parsing

// [[wikilink]] and [[target|alias]]
export const WIKI_LINK = /!?\[\[([^\]|#]+)(?:#[^\]|]+)?(?:\|[^\]]*)?\]\]/g;
// [label](target), also catching ![alt](target) for images
export const MD_LINK = /!?\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;

/**
 * Split a Markdown document into segments that are safe to rewrite and segments
 * that must be left byte-identical.
 *
 * Protected segments:
 *   - YAML front matter. Obsidian uses `links:`/`aliases:` there as wikilinks, and
 *     that property's type is vault-wide, so rewriting it corrupts 1725 files.
 *   - fenced code blocks (``` and ~~~), including nested longer fences
 *   - inline code spans (`...`) inside prose lines
 *
 * This matters twice over. A migration that rewrites a wikilink inside a code
 * block corrupts the sample; and a document that DOCUMENTS a pattern -- e.g. a
 * plan explaining that `[[bench]]` is mojibake -- would otherwise be counted as
 * containing that pattern, and the count would never drain.
 */
export function segment(src) {
  const segs = [];
  let i = 0;
  const n = src.length;

  // 1. front matter
  if (src.startsWith('---\n') || src.startsWith('---\r\n')) {
    const nl = src[3] === '\r' ? '\r\n' : '\n';
    const end = src.indexOf(nl + '---' + nl, 3);
    const stop = end === -1 ? -1 : end + (nl + '---' + nl).length;
    if (stop !== -1) {
      segs.push({ text: src.slice(0, stop), safe: false, why: 'frontmatter' });
      i = stop;
    }
  }

  // 2. walk the body line by line tracking fence state.
  //    Loop condition is `pos < n`, and every branch leaves pos > where it
  //    started. An earlier version used `pos <= n` and only broke in the prose
  //    branch, so a file whose LAST line is a fence marker spun forever.
  let pos = i;
  let fence = null; // { marker, len }
  let lineNo = 1;

  const push = (text, safe, why) => {
    if (text) segs.push({ text, safe, why, line: lineNo - 1 });
  };

  while (pos < n) {
    const nlIdx = src.indexOf('\n', pos);
    const lineEnd = nlIdx === -1 ? n : nlIdx + 1;
    const line = src.slice(pos, lineEnd);
    const thisLine = lineNo;
    pos = lineEnd;
    lineNo++;

    // Match against the line WITHOUT its newline. In JavaScript `$` does not
    // match before a trailing newline (unlike Python), so matching the raw slice
    // meant the fence pattern only ever fired on a file's LAST line. Every
    // consumer of segment() -- proseOf(), check-links, check-secrets,
    // wikilinks-to-md -- was therefore reading code-block contents as prose.
    const bare = line.replace(/\r?\n$/, '');
    const fenceMatch = bare.match(/^\s{0,3}(`{3,}|~{3,})(.*)$/);
    if (fenceMatch) {
      const marker = fenceMatch[1][0];
      const len = fenceMatch[1].length;
      if (fence === null) {
        fence = { marker, len };
        push(line, false, 'code-fence');
      } else {
        if (marker === fence.marker && len >= fence.len && fenceMatch[2].trim() === '') fence = null;
        push(line, false, 'code-fence');
      }
      continue;
    }

    if (fence !== null) {
      push(line, false, 'code-fence');
      continue;
    }

    // Prose line: split out inline code spans. Segments MUST be emitted in
    // positional order -- an earlier version buffered plain lines and flushed
    // the buffer at the end, which silently relocated every code-free line in
    // the file to the bottom. Checkers never noticed because proseOf() only
    // concatenates safe segments and does not care about order, but any script
    // that writes segments back (wikilinks-to-md.mjs) reproduced the damage.
    let cursor = 0;
    let sawCode = false;
    // `lineNo` is already this line's number: it was captured as `thisLine` and
    // incremented above. Resetting it here (an earlier version did) pinned every
    // segment to line 1, so a consumer indexing by `seg.line` saw one long file.
    for (const m of line.matchAll(/(`+)([^`]|[^`][\s\S]*?)\1/g)) {
      if (m.index > cursor) push(line.slice(cursor, m.index), true, 'prose');
      push(m[0], false, 'inline-code');
      cursor = m.index + m[0].length;
      sawCode = true;
    }
    if (!sawCode) push(line, true, 'prose');
    else if (cursor < line.length) push(line.slice(cursor), true, 'prose');
  }

  return segs;
}

/**
 * The invariant that must hold for `segment()`: concatenating every segment
 * reproduces the input byte for byte. When this fails, any script that writes
 * segments back is about to corrupt a file.
 */
export const segmentsRoundTrip = (src) => segment(src).map((s) => s.text).join('') === src;

/** Concatenate only the segments a link scanner should look at. */
export function proseOf(src) {
  return segment(src)
    .filter((s) => s.safe)
    .map((s) => s.text)
    .join('');
}

export const normalisePath = (p) => {
  const out = [];
  for (const seg of String(p).split(/[\\/]/)) {
    if (seg === '' || seg === '.') continue;
    if (seg === '..') out.pop();
    else out.push(seg);
  }
  return out.join('/');
};

export const linkFrom = (fromRel, toRel) => {
  const p = toPosix(relative(dirname(fromRel), toRel));
  return p.startsWith('.') ? p : './' + p;
};

export const encodePath = (p) =>
  p.split('/').map((s) => encodeURIComponent(s).replace(/%2F/gi, '/')).join('/');

export const mdLink = (fromRel, toRel, label) =>
  `[${label}](${encodePath(linkFrom(fromRel, toRel))})`;

/** Canonical wikilink / filename forms that refer to the same document. */
export const nameVariants = (rel) => {
  const n = basename(rel);
  const stem = n.replace(/\.md$/i, '');
  return new Set([n.toLowerCase(), stem.toLowerCase(), rel.toLowerCase()]);
};
