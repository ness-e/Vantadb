#!/usr/bin/env node
// scripts/docs/check-doc-examples.mjs
//
// PURPOSE
//   Make the code examples in docs/ executable claims instead of prose. A link
//   checker cannot catch documentation drift, because the drift this gate exists
//   for is not a bad path -- it is a METHOD THAT DOES NOT EXIST. The file an AI
//   agent reads to learn the API (llms.txt, generated from docs/api/PYTHON_SDK.md)
//   documented `db.search_memory()`; `Client` has no such method. Every gate in
//   this repo passed while that was true.
//
//   Two jobs:
//     A. EXTRACT  walk docs/, find ```python and ```rust blocks, classify each as
//                 `runnable` (self-contained), `fragment` (illustrative; cannot
//                 stand alone) or `doc-test` (a Rust block that is really a
//                 doctest).
//     B. VERIFY   for every `runnable` Python block, resolve every attribute and
//                 method the snippet TOUCHES against the real installed `vantadb`
//                 package. Nothing is executed: the snippet is parsed with
//                 Python's `ast` module and each access chain is resolved with
//                 dir()/getattr. Offline, no sandbox needed, a few seconds.
//
//   A missing method is a hard failure. That is the entire point.
//
//   Rust, honestly scoped: `doc-test` blocks are counted and reported as coverage
//   (running them is `cargo test --doc`, not this script's job), and `runnable`
//   Rust blocks are checked for crate names that are not workspace members.
//   Rust has no reflection, so "does this method exist" cannot be answered
//   statically without a compiled index. That ceiling is stated in the runbook
//   rather than papered over with a heuristic that would produce false positives.
//
// HOUSE STYLE
//   Reuses lib.mjs for path walking (`listDocs`), file reading (`readDoc`) and
//   frontmatter (`parseFrontmatter`) so this script cannot disagree with
//   check-links.mjs / check-docs.mjs about what a document is. `proseOf()` and
//   `segment()` are the link scanner's helpers and are not used here: this gate
//   wants the code blocks, not the prose around them, and `segment()`'s fence
//   regex does not fire (see the note on `extractBlocks`).
//
// USAGE
//   node scripts/docs/check-doc-examples.mjs              # gate
//   node scripts/docs/check-doc-examples.mjs --json       # machine-readable
//   node scripts/docs/check-doc-examples.mjs --all        # include archived trees
//   node scripts/docs/check-doc-examples.mjs --self-test  # built-in assertions
//   node scripts/docs/check-doc-examples.mjs --max-missing=N   # set the budget
//   node scripts/docs/check-doc-examples.mjs --python=<exe>    # pick an interpreter
//
// EXIT CODES
//   0 = no runnable snippet references API that does not exist, and the
//       missing-API count is within budget
//   1 = over budget, or --self-test failed
//   2 = bad usage, or the `vantadb` package is not importable (see below)
//
//   The vantadb import is a HARD requirement, not a soft one. A gate that
//   silently degrades to "verified nothing" is worse than no gate: it goes green
//   and the drift continues. Install the wheel (see
//   docs/dev/workflow/gate-doc-examples.md) or pass --allow-missing-package to
//   get the extraction-only report.

import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { listDocs, parseFrontmatter, readDoc, ROOT } from './lib.mjs';

const ARGS = new Set(process.argv.slice(2));
const JSON_OUT = ARGS.has('--json');
const INCLUDE_ARCHIVE = ARGS.has('--all');
const SELF_TEST = ARGS.has('--self-test');
const ALLOW_MISSING_PKG = ARGS.has('--allow-missing-package');

/**
 * Documented-but-nonexistent API surfaces, tolerated.
 *
 * The gate is new and the corpus has never been checked, so it starts with the
 * measured count as its budget: the job of a budget is to hold the line at
 * today's damage while the count goes DOWN, and to fail a PR that ADDS drift
 * immediately. Measured 2026-09-29 against vantadb 0.7.0: 1, the single entry
 * `docs/user/blog/ollama_vantadb_local_memory.md:45` -> `from vantadb import
 * VantaDB` (the class AST-010 removed in 0.6.0).
 *
 * It is NOT a tolerance for "documentation is roughly right". Lower it to 0 the
 * day that blog post is fixed; at 0 the budget is a no-op and every
 * documented-but-nonexistent surface is a hard failure.
 */
const MISSING_BUDGET = Number(
  (process.argv.find((a) => a.startsWith('--max-missing=')) ?? '').split('=')[1] || 1,
);

const PYTHON_FLAG = process.argv.find((a) => a.startsWith('--python='));
const PYTHON_BIN = PYTHON_FLAG ? PYTHON_FLAG.split('=')[1] : null;

// Frozen history is never repaired, so it is not part of the gate. Same
// rationale as check-links.mjs: a count that can never drain is noise.
const ARCHIVE_RE = /(^|\/)(archive|target)(\/|$)/;

// ---------------------------------------------------------------- extraction

/**
 * Every fenced code block in a Markdown document, with its language, its 1-based
 * line number, and the heading it sits under.
 *
 * ponytail: this scans fences itself instead of reusing `segment()` from lib.mjs,
 * because `segment()`'s fence regex is `/^\s{0,3}(`{3,}|~{3,})(.*)$/` with no `m`
 * flag. In JavaScript `$` matches only at InputLength, and `.` does not cross a
 * newline, so that regex can only ever match a fence on the LAST line of a file
 * (measured: 0 code-fence segments across docs/api/GRAPH_RAG.md, and
 * `proseOf()` returns `db.put(` as prose). Fixing it is a one-line change to
 * lib.mjs -- drop the `$` or add the `m` flag -- but lib.mjs is not this task's
 * file to edit, and `check-links.mjs` depends on the current behaviour. Until
 * that is fixed upstream, this is the smallest correct thing: a 20-line line
 * scan, and the reason for it is written down so it is not mistaken for
 * duplication.
 *
 * @returns {{lang:string, info:string, line:number, section:string|null, code:string}[]}
 */
export function extractBlocks(src) {
  const { body, end } = parseFrontmatter(src);
  const baseLine = (src.slice(0, end).match(/\n/g) ?? []).length + 1;
  const out = [];
  let section = null;
  let open = null;

  const lines = body.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/^\s{0,3}(`{3,}|~{3,})(.*)$/);
    if (m) {
      const marker = m[1][0];
      const len = m[1].length;
      const rest = m[2].trim();
      if (!open) {
        const lang = (rest.split(/[\s,{}]+/)[0] ?? '').toLowerCase();
        if (lang) open = { marker, len, lang, info: rest, line: baseLine + i, section, buf: [] };
      } else if (marker === open.marker && len >= open.len && rest === '') {
        out.push({ lang: open.lang, info: open.info, line: open.line, section: open.section, code: open.buf.join('\n') });
        open = null;
      }
      continue;
    }
    if (open) {
      open.buf.push(lines[i]);
      continue;
    }
    const h = lines[i].match(/^(#{1,6})\s+(.*\S)\s*$/);
    if (h) section = h[2];
  }
  // An unterminated fence at EOF is still reported, so a broken document shows up
  // as a block rather than silently vanishing from the count.
  if (open) {
    out.push({ lang: open.lang, info: open.info, line: open.line, section: open.section, code: open.buf.join('\n') });
  }
  return out;
}

/** Markers a block's own author used to say "this does not run". */
const SKIP_MARKER = /#\s*vanta-skip\b|<!--\s*vanta-skip/;

// ------------------------------------------------- rust: workspace + doctests

/** Crate names declared in the workspace `Cargo.toml` `members` + root package. */
function workspaceCrates() {
  const toml = readFileSync(join(ROOT, 'Cargo.toml'), 'utf8');
  const names = new Set();
  const root = toml.match(/^name\s*=\s*"([^"]+)"/m);
  if (root) names.add(root[1].replace(/-/g, '_'));
  const members = toml.match(/^members\s*=\s*\[([\s\S]*?)\]/m);
  if (members) {
    for (const m of members[1].matchAll(/"([^"]+)"/g)) {
      names.add(m[1].replace(/-/g, '_'));
      names.add(m[1].split('/').pop().replace(/-/g, '_'));
    }
  }
  return names;
}

const DOCTEST_SECTION = /^##\s+(testing|test|tests|example|examples|usage)\b/i;
const DOCTEST_HINT = /(^|,)no_run($|,)|(^|,)ignore($|,)|doctest/i;

/**
 * Classify a Rust block without a compiler.
 *
 * `doc-test` first: a block under a Testing/Example section, tagged no_run/ignore,
 * or shaped like a doctest (assert / #[test] / fn main) is a doctest, and the
 * honest thing to do with it is count it. Then `fragment` if it has unresolved
 * ellipsis or a placeholder. Then `runnable` if it names a crate: every crate it
 * references must be a workspace member, which is checkable.
 */
function classifyRust(b, crates) {
  const base = { ...b, lang: 'rust' };
  if (DOCTEST_SECTION.test(b.section ?? '') || DOCTEST_HINT.test(b.info)) {
    return { ...base, kind: 'doc-test', reason: 'section-or-flag' };
  }
  if (/^\s*(#\[test\]|assert(_eq|_ne|_matches)?!|fn main)/m.test(b.code)) {
    return { ...base, kind: 'doc-test', reason: 'assert-or-test-fn' };
  }
  if (/\.\.\.|<[a-z_]+>|\bTODO\b|\bplaceholder\b|\bunimplemented\b/i.test(b.code)) {
    return { ...base, kind: 'fragment', reason: 'placeholder' };
  }
  // Crate roots only come from `use` / `extern crate`: a path like
  // `Embedded::new()` names a TYPE, not a crate, and guessing at which is which
  // is how a checker starts inventing crates that do not exist.
  const used = new Set();
  for (const m of b.code.matchAll(/\b(?:use|extern\s+crate)\s+([a-z][a-z0-9_]*)/gi)) {
    used.add(m[1].replace(/-/g, '_'));
  }
  if (!used.size) return { ...base, kind: 'fragment', reason: 'no-crate-path' };
  const unknown = [...used].filter((c) => !crates.has(c) && !STDLIB.has(c));
  if (unknown.length) {
    return { ...base, kind: 'fragment', reason: `unknown-crate:${unknown.join(',')}` };
  }
  return { ...base, kind: 'runnable', reason: 'workspace-crate' };
}

// Crate roots that are legal in a `use` without being workspace members.
const STDLIB = new Set(['std', 'core', 'alloc', 'crate', 'self', 'super']);

// ------------------------------------------------------------ python analysis
//
// The analysis runs in Python, not JavaScript, on purpose: the task is "does
// this attribute exist on the real class", and the only honest answer comes from
// `getattr` on the real PyO3 class. Python's `ast` also already knows the
// difference between a name that is bound and a name that is merely used, which
// is exactly the runnable/fragment boundary. So the .mjs extracts and reports,
// and this helper resolves.

const PY_HELPER = String.raw`
"""Static API verifier for check-doc-examples.mjs. Reads JSON on stdin."""
import ast, builtins, json, os, shutil, sys, tempfile, types, warnings

warnings.simplefilter("ignore")

SDK = {"vantadb", "vantadb_py"}
BUILTINS = set(dir(builtins))
OUT = {"pkg": {"available": False}, "results": []}


def load_sdk():
    try:
        import vantadb
    except Exception as e:
        OUT["pkg"] = {"available": False, "error": "%s: %s" % (type(e).__name__, e)}
        return None
    info = {"available": True, "version": getattr(vantadb, "__version__", "?"),
            "module": vantadb.__name__}
    # One live Client so the domain sub-clients (db.memory / db.graph / ...) can be
    # resolved. "memory" is a getset_descriptor on the class, so Client.memory is a
    # descriptor, not a MemoryClient; only an instance yields the real type.
    # A Client preallocates ~320 MB, so this directory is not a scratch file:
    # without the rmtree every gate run leaks 320 MB into TMPDIR. That is
    # enough to fill a CI runner and enough to make git fail to write its index
    # on a developer machine. Cleanup is unconditional, including on the error
    # paths -- that is the only reason it is a finally and not a tail call.
    tmp = tempfile.mkdtemp(prefix="vanta-doc-gate-")
    try:
        try:
            info["probe"] = vantadb.Client(os.path.join(tmp, "db"))
        except Exception as e:
            info["probe_error"] = "%s: %s" % (type(e).__name__, e)
            info["probe"] = None
        info["subclients"] = {}
        if info.get("probe") is not None:
            for sub in ("memory", "graph", "system", "wiki"):
                try:
                    s = getattr(info["probe"], sub)
                    info["subclients"][sub] = type(s).__name__
                except Exception:
                    pass
        OUT["pkg"] = info
        return vantadb
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


SDK_MOD = load_sdk()


def bound_names(tree):
    out = set()
    for n in ast.walk(tree):
        if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Store):
            out.add(n.id)
        elif isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            out.add(n.name)
        elif isinstance(n, ast.arg):
            out.add(n.arg)
        elif isinstance(n, ast.Import):
            for a in n.names:
                out.add((a.asname or a.name).split(".")[0])
        elif isinstance(n, ast.ImportFrom):
            for a in n.names:
                out.add(a.asname or a.name)
        elif isinstance(n, ast.ExceptHandler) and n.name:
            out.add(n.name)
        elif isinstance(n, (ast.Global, ast.Nonlocal)):
            for x in n.names:
                out.add(x)
    return out


def classify(src):
    try:
        tree = ast.parse(src)
    except SyntaxError as e:
        # Signature-only blocks ("db.put(namespace: str, ...) -> Record") and
        # prose-in-a-fence land here. They are illustrations, not programs.
        return "fragment", "unparseable:" + (e.msg or "SyntaxError"), None
    for n in ast.walk(tree):
        if isinstance(n, ast.Expr) and isinstance(n.value, ast.Constant) and n.value.value is Ellipsis:
            return "fragment", "ellipsis", tree
    bound = bound_names(tree)
    loaded = {n.id for n in ast.walk(tree) if isinstance(n, ast.Name) and isinstance(n.ctx, ast.Load)}
    missing = sorted(loaded - bound - BUILTINS)
    if missing:
        return "fragment", "undefined:" + ",".join(missing[:4]), tree
    return "runnable", "", tree


def chain_of(node):
    """db.memory.get -> ['db','memory','get']; None if the root is not a Name."""
    parts = []
    cur = node
    while isinstance(cur, ast.Attribute):
        parts.append(cur.attr)
        cur = cur.value
    if not isinstance(cur, ast.Name):
        return None
    parts.append(cur.id)
    parts.reverse()
    return parts


class Resolver:
    """name -> python object whose attributes can be inspected."""

    def __init__(self, sdk):
        self.env = {}
        self.sdk = sdk
        self.probe = OUT["pkg"].get("probe")

    def holder(self, obj):
        # getattr on a class returns the descriptor, not the value. Use the live
        # instance when we have one, so db.memory resolves to a MemoryClient.
        if isinstance(obj, type) and self.probe is not None and obj is type(self.probe):
            return self.probe
        return obj

    def resolve(self, parts):
        """-> (status, attr) with status in ok | missing | unresolved."""
        cur = self.env.get(parts[0])
        if cur is None:
            return "unresolved", None
        for attr in parts[1:]:
            cur = self.holder(cur)
            # A descriptor on a class we hold no instance for: the tail of this
            # chain is unverifiable. Report unresolved, never missing -- a false
            # positive here would be a broken gate.
            if isinstance(cur, type) and hasattr(type(cur), attr):
                raw = inspect_getattr_static(cur, attr)
                if isinstance(raw, (types.GetSetDescriptorType, types.MemberDescriptorType)):
                    return "unresolved", None
            if not hasattr(cur, attr):
                return "missing", attr
            cur = getattr(cur, attr)
            if isinstance(cur, (types.GetSetDescriptorType, types.MemberDescriptorType)):
                return "unresolved", None
        return "ok", None

    def infer(self, node):
        """Best-effort type of an assignment RHS. None when unknown."""
        if isinstance(node, ast.Call):
            fn = node.func
            if isinstance(fn, ast.Name):
                t = self.env.get(fn.id)
                return t if isinstance(t, type) else None
            if isinstance(fn, ast.Attribute) and self.sdk is not None:
                fparts = chain_of(fn)
                if not fparts:
                    return None
                st, _ = self.resolve(fparts)
                if st == "ok":
                    v = getattr(self.holder(self.env[fparts[0]]), fn.attr, None)
                    if isinstance(v, type):
                        return v
                    if fn.attr == "connect":
                        return getattr(self.sdk, "Client", None)
            return None
        if isinstance(node, ast.Attribute):
            parts = chain_of(node)
            if parts and self.resolve(parts)[0] == "ok":
                return getattr(self.holder(self.env[parts[0]]), node.attr, None)
        return None


def inspect_getattr_static(obj, attr):
    for klass in getattr(obj, "__mro__", [obj]):
        if attr in getattr(klass, "__dict__", {}):
            return klass.__dict__[attr]
    return None


def verify(src, res):
    """Walk the tree in source order, resolving chains against env."""
    tree = ast.parse(src)
    found, checked, unresolved = [], 0, 0
    seen = set()
    verified = set()

    def visit(node, env):
        nonlocal checked, unresolved
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            mod = getattr(node, "module", None) or ""
            root = (node.names[0].name if isinstance(node, ast.Import) else mod).split(".")[0]
            if isinstance(node, ast.Import) and root not in SDK:
                pass
            elif root in SDK:
                for a in node.names:
                    local = a.asname or (a.name if isinstance(node, ast.Import) else a.name)
                    if isinstance(node, ast.Import):
                        env[local] = res.sdk
                    else:
                        env[local] = getattr(res.sdk, a.name, None)
            else:
                for a in node.names:
                    env[a.asname or a.name.split(".")[0]] = None
        elif isinstance(node, (ast.Assign, ast.AnnAssign, ast.AugAssign)):
            value = node.value
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            if value is not None:
                inferred = res.infer(value)
                for t in targets:
                    if isinstance(t, ast.Name) and inferred is not None:
                        env[t.id] = inferred
        elif isinstance(node, ast.With):
            for item in node.items:
                if item.optional_vars is not None and isinstance(item.context_expr, ast.Call):
                    inferred = res.infer(item.context_expr)
                    if isinstance(item.optional_vars, ast.Name) and inferred is not None:
                        env[item.optional_vars.id] = inferred
        elif isinstance(node, ast.Attribute):
            parts = chain_of(node)
            if parts and tuple(parts) not in seen:
                seen.add(tuple(parts))
                st, attr = res.resolve(parts)
                if st == "missing":
                    found.append({"chain": ".".join(parts), "attr": attr, "line": node.lineno})
                elif st == "ok":
                    checked += 1
                    verified.add(parts[-1] if len(parts) == 2 else parts[-1])
                else:
                    unresolved += 1
        for child in ast.iter_child_nodes(node):
            visit(child, env)

    visit(tree, res.env)
    return found, checked, unresolved, sorted(verified)


def check_imports(tree, res):
    """from vantadb import X is a complete claim on its own.

    It raises ImportError or it does not, regardless of what the rest of the
    block does -- so it is checked even in a "fragment". A fragment is not
    runnable, but the name it imports from the SDK still has to exist, and this
    is where documented-but-removed classes show up: the 0.6.0 rename deleted
    "VantaDB", "VantaMemoryRecord" and friends, and a doc that still says
    "from vantadb import VantaDB" sends every reader to an ImportError.
    """
    found = []
    if tree is None or res.sdk is None:
        return found
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and (node.module or "").split(".")[0] in SDK:
            for a in node.names:
                if a.name == "*":
                    continue
                if not hasattr(res.sdk, a.name):
                    found.append({"chain": "%s.%s" % (node.module, a.name), "attr": a.name,
                                  "line": node.lineno, "via": "import"})
        elif isinstance(node, ast.Import):
            for a in node.names:
                if a.name.split(".")[0] not in SDK:
                    continue
                if a.name not in (SDK - {"vantadb"}):
                    try:
                        __import__(a.name)
                    except Exception:
                        found.append({"chain": a.name, "attr": a.name,
                                      "line": node.lineno, "via": "import"})
    return found


def run(job):
    for item in job["snippets"]:
        rid = item.get("id", "")
        src = item.get("source", "")
        kind, reason, tree = classify(src)
        entry = {"id": rid, "kind": kind, "reason": reason}
        res = Resolver(SDK_MOD) if SDK_MOD is not None else None
        entry["missing"] = check_imports(tree, res)
        entry["checked"] = 0
        entry["unresolved"] = 0
        if kind == "runnable" and SDK_MOD is not None:
            missing, checked, unresolved, verified = verify(src, res)
            entry["missing"] = entry["missing"] + missing
            entry["checked"] = checked
            entry["unresolved"] = unresolved
            entry["verified"] = verified
        elif kind == "runnable":
            entry["skipped"] = "package-not-importable"
        OUT["results"].append(entry)


try:
    _job = json.load(sys.stdin)
    run(_job)
finally:
    if OUT["pkg"].get("probe") is not None:
        try:
            OUT["pkg"]["probe"].close()
        except Exception:
            pass
    OUT["pkg"].pop("probe", None)
    sys.stdout.write(json.dumps(OUT))
`;

// A backtick inside PY_HELPER silently ends the template literal and turns the
// Python source into a JavaScript SyntaxError pointing at a comment. Say so.
if (PY_HELPER.includes('`')) {
  throw new Error('PY_HELPER contains a backtick; it lives inside a JS template literal');
}

/** Locate a Python interpreter. python3, then python, then py -3. */function findPython() {
  const candidates = PYTHON_BIN
    ? [PYTHON_BIN]
    : process.platform === 'win32'
      ? ['python', 'py -3', 'python3']
      : ['python3', 'python'];
  for (const c of candidates) {
    const [exe, ...args] = c.split(' ');
    const r = spawnSync(exe, [...args, '-c', 'import sys; print(sys.version_info[0])'], {
      encoding: 'utf8',
    });
    if (r.status === 0) return { exe, args };
  }
  return null;
}

/** Run the Python analyzer over every Python block in one process. */
function analyzePython(blocks) {
  const py = findPython();
  if (!py) {
    return {
      pkg: { available: false, error: 'no python3/python interpreter on PATH' },
      results: [],
    };
  }
  const dir = mkdtempSync(join(tmpdir(), 'vanta-doc-gate-'));
  const script = join(dir, 'analyze.py');
  try {
    writeFileSync(script, PY_HELPER, 'utf8');
    const r = spawnSync(
      py.exe,
      [...py.args, script],
      { input: JSON.stringify({ snippets: blocks }), encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
    );
    if (r.status !== 0) {
      return {
        pkg: { available: false, error: `analyzer exited ${r.status}: ${(r.stderr || '').trim().slice(0, 1500)}` },
        results: [],
      };
    }
    return JSON.parse(r.stdout);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ------------------------------------------------------------------ pipeline

function scan() {
  const crates = workspaceCrates();
  const files = listDocs({ includeArchive: INCLUDE_ARCHIVE });
  const report = {
    filesScanned: 0,
    filesWithCode: 0,
    blocks: 0,
    byLang: {},
    python: { runnable: 0, fragment: 0 },
    rust: { runnable: 0, fragment: 0, 'doc-test': 0 },
    pkg: { available: false },
    chainsChecked: 0,
    chainsUnresolved: 0,
    analyzed: 0, // python blocks that reached the analyzer
    verified: {}, // attribute name -> number of snippets that resolved it
    missing: [], // { file, line, chain, attr, via, kind }
    fragmentReasons: {},
    docTestFiles: [],
  };

  const pyBlocks = [];
  for (const rel of files) {
    if (!INCLUDE_ARCHIVE && ARCHIVE_RE.test(rel)) continue;
    report.filesScanned++;
    const src = readDoc(rel);
    const blocks = extractBlocks(src);
    const relevant = blocks.filter((b) => b.lang === 'python' || b.lang === 'rust');
    if (!relevant.length) continue;
    report.filesWithCode++;
    for (const b of relevant) {
      report.blocks++;
      report.byLang[b.lang] = (report.byLang[b.lang] ?? 0) + 1;
      if (b.lang === 'rust') {
        const r = classifyRust(b, crates);
        report.rust[r.kind]++;
        if (r.kind === 'doc-test' && !report.docTestFiles.includes(rel)) report.docTestFiles.push(rel);
        if (r.kind === 'fragment') {
          const key = r.reason.split(':')[0];
          report.fragmentReasons[`rust/${key}`] = (report.fragmentReasons[`rust/${key}`] ?? 0) + 1;
        }
        continue;
      }
      // Python: the skip marker is the author's own declaration that the block
      // is illustrative. Same rule as dev-tools/validate_doc_snippets.py.
      if (SKIP_MARKER.test(b.code)) {
        report.python.fragment++;
        report.fragmentReasons['python/skip-marker'] =
          (report.fragmentReasons['python/skip-marker'] ?? 0) + 1;
        continue;
      }
      pyBlocks.push({ id: `${rel}:${b.line}`, source: b.code, file: rel, line: b.line });
    }
  }

  if (pyBlocks.length) {
    const analysis = analyzePython(pyBlocks);
    report.pkg = analysis.pkg;
    const byId = new Map(pyBlocks.map((b) => [b.id, b]));
    for (const r of analysis.results) {
      const meta = byId.get(r.id);
      report.analyzed++;
      if (r.kind === 'fragment') {
        report.python.fragment++;
        const key = `python/${r.reason.split(':')[0]}`;
        report.fragmentReasons[key] = (report.fragmentReasons[key] ?? 0) + 1;
      } else {
        report.python.runnable++;
        report.chainsChecked += r.checked ?? 0;
        report.chainsUnresolved += r.unresolved ?? 0;
        for (const name of r.verified ?? []) {
          report.verified[name] = (report.verified[name] ?? 0) + 1;
        }
      }
      // ast line numbers are 1-based within the block; the block's fence sits on
      // blockLine, so the block's first code line is blockLine + 1.
      for (const m of r.missing ?? []) {
        report.missing.push({
          file: meta.file,
          line: meta.line + (m.line ?? 1),
          blockLine: meta.line,
          chain: m.chain,
          attr: m.attr,
          via: m.via ?? 'attribute',
          kind: r.kind,
        });
      }
    }
  }

  return report;
}

// ----------------------------------------------------------------- self-test

/**
 * Built-in assertions. No framework: three checks that must hold, printed as
 * PASS/FAIL, non-zero exit on any failure. These are the runnable check that the
 * logic works -- a gate with no test is a gate whose first real finding nobody
 * can trust.
 */
function selfTest() {
  let failed = 0;
  const ok = (name, cond, detail = '') => {
    console.log(`${cond ? 'PASS' : 'FAIL'}  ${name}${detail ? '  ' + detail : ''}`);
    if (!cond) failed++;
  };

  // 1. extraction: the info string, line number and heading come through.
  const md = [
    '## Testing',
    '',
    '```python',
    'import vantadb',
    '```',
    '',
    '```rust',
    'use vantadb::Embedded;',
    '```',
  ].join('\n');
  const blocks = extractBlocks(md);
  ok('extract: two tagged blocks', blocks.length === 2, `got ${blocks.length}`);
  ok('extract: python lang + line', blocks[0]?.lang === 'python' && blocks[0]?.line === 3, JSON.stringify(blocks[0] && { l: blocks[0].lang, n: blocks[0].line }));
  ok('extract: rust lang + line', blocks[1]?.lang === 'rust' && blocks[1]?.line === 7, JSON.stringify(blocks[1] && { l: blocks[1].lang, n: blocks[1].line }));
  ok('extract: heading captured', blocks[0]?.section === 'Testing', String(blocks[0]?.section));

  // 2. rust classification: doctest section wins over runnable.
  const crates = workspaceCrates();
  const cratesHave = crates.has('vantadb');
  const dt = classifyRust({ info: '', section: 'Testing', code: 'use vantadb::Embedded;\nassert_eq!(1, 1);' }, crates);
  ok('rust: block under ## Testing is a doc-test', dt.kind === 'doc-test', dt.kind);
  const rt = classifyRust({ info: '', section: 'Api', code: 'use vantadb::Embedded;\nfn main() {}\nassert!(true);' }, crates);
  ok('rust: assert-shaped block is a doc-test', rt.kind === 'doc-test', rt.kind);
  const frag = classifyRust({ info: '', section: 'Api', code: 'use vantadb::...;\nlet x = 1;' }, crates);
  ok('rust: ellipsis is a fragment', frag.kind === 'fragment', frag.kind);
  if (cratesHave) {
    const good = classifyRust({ info: '', section: 'Api', code: 'use vantadb::Embedded;\nlet db = Embedded::new();' }, crates);
    ok('rust: workspace crate is runnable', good.kind === 'runnable', good.kind);
    const bad = classifyRust({ info: '', section: 'Api', code: 'use vanta_core::Embedded;' }, crates);
    ok('rust: non-member crate is a fragment', bad.kind === 'fragment', bad.kind);
  }

  // 3. python classification, offline: fragment vs runnable.
  const pyCases = [
    { name: 'fragment: object defined in surrounding prose', source: 'hits = db.search("ns", [], text_query="x")\nprint(hits[0].payload)' },
    { name: 'fragment: signature-only block', source: 'db.put(\n    namespace: str,\n    key: str,\n) -> Record' },
    { name: 'fragment: unresolved ellipsis', source: 'import vantadb\ndb.put(...)' },
    { name: 'runnable: imports and defines what it uses', source: 'import vantadb\n\ndb = vantadb.Client(":memory:")\ndb.put(namespace="ns", key="k", payload="p")' },
  ];
  const py = analyzePython(pyCases);
  if (!py.pkg.available) {
    console.log(`SKIP  python API resolution (vantadb not importable: ${py.pkg.error})`);
    console.log('      classification-only assertions still ran; run this in the CI job that installs the wheel.');
  }
  const pyCasesIdx = 'python: ';
  for (let i = 0; i < pyCases.length; i++) {
    const r = py.results[i];
    const want = pyCases[i].name.startsWith('runnable') ? 'runnable' : 'fragment';
    ok(`${pyCasesIdx}${pyCases[i].name}`, r?.kind === want, `got ${r?.kind} (${r?.reason})`);
  }

  if (py.pkg.available) {
    // 4. the two assertions that matter: a real method passes, an invented one fails.
    const real = analyzePython([
      { id: 'real', source: 'from vantadb import Client\n\ndb = Client(":memory:")\ndb.put(namespace="ns", key="k", payload="p")\ndb.memory.search(namespace="ns", query_vector=[0.1])' },
    ]);
    const r0 = real.results[0];
    ok('python: real methods resolve (put, memory.search)', r0.kind === 'runnable' && r0.missing.length === 0, JSON.stringify(r0.missing ?? r0.reason));
    ok('python: real snippet checked at least 2 chains', (r0.checked ?? 0) >= 2, `checked=${r0.checked}`);

    const invented = analyzePython([
      { id: 'invented', source: 'from vantadb import Client\n\ndb = Client(":memory:")\nhits = db.search_memory("ns", [0.1], top_k=5)' },
    ]);
    const r1 = invented.results[0];
    const names = (r1.missing ?? []).map((m) => m.attr);
    ok('python: invented method is a hard failure', r1.kind === 'runnable' && r1.missing.length === 1, JSON.stringify(r1.missing ?? r1.reason));
    ok('python: failure names search_memory', names.includes('search_memory'), JSON.stringify(names));

    const inventedMod = analyzePython([
      { id: 'mod', source: 'import vantadb\n\nprint(vantadb.VantaDB("./x"))' },
    ]);
    ok('python: invented module class is a hard failure', (inventedMod.results[0].missing ?? []).some((m) => m.attr === 'VantaDB'), JSON.stringify(inventedMod.results[0].missing));
  }

  console.log(`\nself-test: ${failed === 0 ? 'OK' : `${failed} FAILED`}`);
  return failed;
}

// -------------------------------------------------------------------- output

if (SELF_TEST) {
  process.exit(selfTest() === 0 ? 0 : 1);
}

const report = scan();

if (JSON_OUT) {
  console.log(JSON.stringify(report, null, 2));
} else {
  console.log(`files scanned          : ${report.filesScanned} (${report.filesWithCode} with python/rust code)`);
  console.log(`code blocks           : ${report.blocks}  (python ${report.byLang.python ?? 0}, rust ${report.byLang.rust ?? 0})`);
  console.log(
    `python                 : ${report.python.runnable} runnable, ${report.python.fragment} fragment ` +
      `(${report.python.runnable + report.python.fragment} of ${report.byLang.python ?? 0} blocks, ` +
      `${report.analyzed} analysed)`,
  );
  console.log(
    `rust                   : ${report.rust['doc-test']} doc-test, ${report.rust.runnable} runnable, ` +
      `${report.rust.fragment} fragment (of ${report.byLang.rust ?? 0})`,
  );
  console.log(`rust doc-test coverage : ${report.docTestFiles.length} files`);
  if (report.pkg.available) {
    console.log(`vantadb package        : ${report.pkg.version} (importable)`);
    console.log(`attribute chains checked: ${report.chainsChecked}  (unresolved ${report.chainsUnresolved})`);
  } else {
    console.log(`vantadb package        : NOT IMPORTABLE -- ${report.pkg.error}`);
    console.log('                          no snippet was verified. Install the wheel; see');
    console.log('                          docs/dev/workflow/gate-doc-examples.md');
  }

  const top = Object.entries(report.fragmentReasons)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 6)
    .map(([k, v]) => `${k}=${v}`)
    .join('  ');
  if (top) console.log(`fragment reasons       : ${top}`);

  const verified = Object.entries(report.verified).sort((a, b) => b[1] - a[1]);
  if (verified.length) {
    console.log(`verified against 0.7.0 : ${verified.length} distinct attributes, e.g. ` + verified.slice(0, 12).map(([k]) => `\`${k}\``).join(' '));
  }

  if (report.missing.length) {
    const viaImport = report.missing.filter((m) => m.via === 'import').length;
    console.log(
      `\nDOCUMENTED BUT NOT EXISTENT (${report.missing.length}, budget ${MISSING_BUDGET}; ` +
        `${viaImport} via import, ${report.missing.length - viaImport} via attribute):`,
    );
    for (const m of report.missing) {
      const via = m.via === 'import' ? 'name' : 'attribute';
      console.log(`  ${m.file}:${m.line}  ${m.chain}   ->  no ${via} '${m.attr}' on vantadb [${m.kind} block]`);
    }
    console.log(
      '\nEach one is a snippet an agent can copy and that will raise ImportError or\n' +
        'AttributeError. See docs/dev/workflow/gate-doc-examples.md for the fix procedure.',
    );
  } else if (report.pkg.available) {
    console.log('\nOK: every attribute a runnable snippet touches exists on the real package.');
  } else {
    // Never print "OK" when nothing was verified: a green line over an
    // unverifiable corpus is how a gate gets trusted after it stopped working.
    console.log('\nNOT VERIFIED: no snippet was checked, because `vantadb` is not importable.');
  }

  if (report.missing.length) {
    console.log(
      report.missing.length > MISSING_BUDGET
        ? `\nOVER BUDGET: ${report.missing.length} > ${MISSING_BUDGET}.`
        : `\nwithin budget: ${report.missing.length} of ${MISSING_BUDGET} tolerated. Lower the budget as entries are fixed.`,
    );
  }
}

if (!report.pkg.available && !ALLOW_MISSING_PKG) {
  process.exit(2);
}
process.exit(report.missing.length > MISSING_BUDGET ? 1 : 0);
