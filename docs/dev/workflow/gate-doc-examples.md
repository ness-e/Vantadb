---
title: "`gate-doc-examples.yml` — GATE: Docs — code examples are real API"
kind: runbook
status: active
description: "One number: how many code examples in docs/ reference API that does not exist"
tags: [vantadb, ci, gate-docs, documentation, python]
related: [.github/workflows/gate-doc-examples.yml, RULES.md, gate-docs-links.md, TRIGGERS.md]
---

# `gate-doc-examples.yml` — GATE: Docs — code examples are real API

## What it does

One number: **how many code examples in `docs/` reference API that does not exist.**

A link checker cannot see this. `[Python SDK](https://.../docs/api/PYTHON_SDK.md)` resolves
perfectly while the page underneath documents a method that was never implemented, and
`llms.txt` — the file an agent reads to learn the API — points straight at that page. Every
other gate in this repo stayed green the whole time.

`check-doc-examples.mjs` does two jobs.

**A — extract.** Walk `docs/`, find every ` ```python ` and ` ```rust ` block, and classify it:

| Class | Meaning | Counted |
|---|---|---|
| `runnable` | Self-contained: imports what it uses, binds every name it reads, no `...` | **verified** |
| `fragment` | Illustrative: missing imports, or an object defined in the surrounding prose | reported, not gated |
| `doc-test` | A Rust block that is really a doctest (`## Testing`/`## Example`, `no_run`/`ignore`, or shaped like `assert!` / `#[test]`) | reported as coverage |

Fragments are expected and fine — 253 of 307 Python blocks are fragments, most of them
signature blocks like `db.put(namespace: str, key: str) -> Record` that are illustrations of a
call, not programs. The gate is about the `runnable` set.

**B — verify.** For every `runnable` Python block, every attribute and method it touches is
resolved against the **real installed `vantadb` package** with `getattr` on the real PyO3
class. Nothing is executed: the block is parsed with Python's `ast` module, an environment of
name → type is threaded through the statements in source order, and each access chain
(`db.memory.get`) is walked to the attribute that is missing. Offline, about a second, and it
needs no sandbox because no snippet ever runs.

One extra rule, because it is the one that was silently broken: **`from vantadb import X` is
checked even in a fragment.** An import either resolves or it does not, on its own — the rest
of the block is irrelevant. That is where the 0.6.0 class renames show up.

## What it catches

| Caught | How |
|---|---|
| A documented method that was renamed or removed (`db.search_memory`, `db.get_memory`) | attribute chain resolves to `missing` |
| A documented class that was removed (`from vantadb import VantaDB`) | import name not on the module |
| A method called on the wrong sub-client (`db.memory.get_archived`) | chain resolved through the real `MemoryClient` |
| A Rust block importing a crate that is not a workspace member | `use` roots vs `Cargo.toml` `members` |
| Silent regression of all of the above on any PR | budget; a PR that adds one fails immediately |

## What it deliberately does not catch, and why

- **Keyword arguments.** `inspect.signature` *does* work on the PyO3 methods, so
  `db.search(namespace="ns", text="x")` (the real parameter is `text_query`) is checkable and
  is not checked. Two reasons: keyword drift is a different failure class with a different fix,
  and the corpus has enough of it that folding it in here would make this gate's first run
  unactionable. Add it as its own budgeted signal when the attribute signal is drained.
- **Return types.** PyO3 exposes no return annotations, so a chain that starts at a call
  result (`hits[0].payload`) stops at the first unknown and is counted as *unresolved*, never
  as *missing*. A false positive here would be a broken gate, so the resolver refuses to guess:
  72 chains verified, 96 unresolved, 0 wrongly reported.
- **Rust method existence.** Rust has no runtime reflection, so "does this method exist" needs
  a compiled index (`rustdoc --output-format json`) that this gate does not have. Rust is
  scoped to crate-name checking plus doctest coverage. Saying so is better than a heuristic
  that invents methods.
- **Runtime behaviour, types, and results.** A block can reference only real methods and still
  be wrong. `dev-tools/validate_doc_snippets.py` is the complement: it *executes* the tutorial
  blocks against a real temp database. This gate is the wide, cheap, static sweep; that one is
  the narrow, real-execution check.
- **Blocks marked `# vanta-skip`.** The author's own declaration that a block is illustrative
  (needs a live Ollama, an API key, an interactive REPL). 18 blocks. Same rule as
  `dev-tools/validate_doc_snippets.py`.
- **Archived trees.** `archive/` and `target/` are frozen history and are excluded by default,
  the same rule `check-links.mjs` uses. `--all` includes them; see the table below.

## The budget

| Budget | Value | Why not zero | Drained in |
|---|---|---|---|
| Documented-but-nonexistent API references | **1** | Measured 2026-09-29 against `vantadb` 0.7.0. One entry, in a file that ships to readers: `docs/user/blog/ollama_vantadb_local_memory.md:45` imports `VantaDB`, a class AST-010 removed in 0.6.0. | fix the blog post, then set the budget to 0 |

A budget exists so the gate can start non-zero without being switched off, and so a PR that
**adds** drift fails on the spot. It is a ratchet, not a tolerance. Override without editing
code:

```
node scripts/docs/check-doc-examples.mjs --max-missing=0
```

With the archive included (`--all`) the count is 7, not 1 — the other six are the same
`vantadb.VantaDB(...)` breakage in `docs/dev/research/archive/Investigacion-plan.md`, which is
frozen and not repaired. That is why archive is off by default.

## When it runs

- **Push** to `main` / `develop` with changes in `docs/**`, `scripts/docs/**`,
  `vantadb-python/**`, `Cargo.toml` or `Cargo.lock`
- **Pull request** to `main` / `develop` with the same paths
- **Workflow dispatch** manual

The `vantadb-python/**` and `Cargo.*` paths matter: the gate resolves against the *built*
package, so a PR that renames a method in the binding must re-run this gate even if it touches
no documentation at all. That is the exact case this gate exists for.

No schedule. A weekly sweep would only re-report the same frozen number.

## How to run it locally

Every command below was run on 2026-09-29 and exits as described.

```bash
# The gate (what CI runs). Exit 0 while the count is within budget, 1 over.
node scripts/docs/check-doc-examples.mjs

# The gate's own logic, on synthetic snippets: 17 assertions, no framework.
# A real method must pass, an invented method must fail, a fragment must
# classify as a fragment. Exit 0 = OK, 1 = an assertion failed.
node scripts/docs/check-doc-examples.mjs --self-test

# Machine-readable.
node scripts/docs/check-doc-examples.mjs --json

# Include archived trees (raises the count from 1 to 7, so it exits 1).
node scripts/docs/check-doc-examples.mjs --all

# Prove the budget is the thing that fails, not the finding.
node scripts/docs/check-doc-examples.mjs --max-missing=0    # exits 1

# Pick an interpreter.
node scripts/docs/check-doc-examples.mjs --python=/usr/bin/python3.11
```

**The package must be importable.** If `import vantadb` fails, the gate exits **2** and prints
`NOT IMPORTABLE` — it does not degrade to "verified nothing" and pass, because a green check
that verified zero snippets is worse than no check. In CI the wheel is built with
`maturin-action` from `./vantadb-python/Cargo.toml`, the same approach as `ci-examples.yml`.
Locally, either use the repo venv or install the wheel:

```bash
pip install ./dist/vantadb_py-*.whl
```

`--allow-missing-package` downgrades exit 2 to a printout and verifies nothing. Use it only
when you deliberately want the extraction report (the fragment/doc-test counts are still
correct without the package).

## When a real API change breaks a snippet

The gate reports `file:line`, the full chain, and the attribute that is missing. Fix the
**documentation**, in this order:

1. **Was the method renamed, or did it never exist?**
   Renamed → update the snippet to the new name. Never existed → the documentation is wrong
   and the snippet must go or be rewritten against the real API. Do not add an alias to the
   package to make a doc pass; the doc is the cheaper thing to fix and the package is
   released.
2. **A class rename** (`VantaDB` → `Client`, `VantaMemoryRecord` → `Record`, …) is a global
   `find` across the whole tree, not a one-line edit. `docs/api/DEPRECATIONS.md` and
   `docs/api/BINDINGS_NAMESPACES.md` hold the mapping.
3. **Re-run the gate** and lower the budget by the number you fixed:
   `--max-missing=0` is the target; when it passes, change `MISSING_BUDGET` in the script and
   commit that with the fix. A fix that does not lower the budget is not a fix, because the
   next person cannot tell the difference.
4. **Fragments are not excused.** The gate will not verify a fragment's attribute chains
   because it cannot know the type of `db` from the surrounding prose. If a drift lands in a
   fragment, this gate will not find it — the fragment is where drift hides. When you touch a
   fragment, check it by hand.

## Invariants

- **No snippet is ever executed.** Verification is `ast` plus `getattr` on the real class. The
  gate is safe to run on any documentation, including documentation from a fork.
- **The gate cannot report a false positive on a descriptor.** `Client.memory` is a
  `getset_descriptor` on the class; resolving it against the class would make every
  `db.memory.get(...)` look broken. The resolver opens one real `Client` in a temp directory
  (closed and removed on the way out) and resolves instance-level attributes against that. A
  descriptor on a class with no live instance is reported *unresolved*, never *missing*.
- **The Rust workspace list is read from `Cargo.toml`**, not hardcoded, so adding a member does
  not require editing the gate.
- **`lib.mjs` is not edited by this gate.** Its `segment()` fence regex
  (`/^\s{0,3}(`{3,}|~{3,})(.*)$/`, no `m` flag) can only match a fence on the last line of a
  file, because in JavaScript `$` matches only at `InputLength` and `.` does not cross a
  newline. Measured: 0 code-fence segments across `docs/api/GRAPH_RAG.md`, and `proseOf()`
  returns `db.put(` as prose. So `check-doc-examples.mjs` scans fences itself — 20 lines —
  and this is the reason, written down so it is not mistaken for duplication. Fixing
  `lib.mjs` is a one-line change, and it also changes what `check-links.mjs` counts.
- **One exit code per outcome**: 0 within budget, 1 over budget or self-test failure, 2
  bad usage or no `vantadb`. A gate with a fourth outcome nobody can predict is a gate people
  stop reading.
