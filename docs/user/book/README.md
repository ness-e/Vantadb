---
title: The mdBook layer (retired 2026-09-29)
kind: index
status: archived
description: "An mdBook project whose src/ held 75 files: 69 of them were 6-line"
tags: [vantadb, documentation, architecture, decision]
related: [book.toml, SUMMARY.md, ../../dev/plans/2026-09-28-docs-consolidation.md]
---

# The mdBook layer (retired 2026-09-29)

> **Status: retired.** `docs/user/book/src/` and `docs/user/book/book/` were deleted on
> 2026-09-29. `book.toml` and `SUMMARY.md` are kept as the record of what it was.
> Rationale: `docs/dev/plans/2026-09-28-docs-consolidation.md` Task 17.

## What it was

An mdBook project whose `src/` held 75 files: **69 of them were 6-line
`{{#include <path>}}` stubs** and 6 were hand-written (`SUMMARY.md`, two index files,
three archived-case-study tombstones).

**It was never a copy.** There was zero duplicated prose — the design was correct, and
`docs/dev/avance/historial/campanas/engineering-health-waves.md:32` records the intent at
the time: *"73 `{{#include}}` stubs … Cero duplicación de contenido existente."*

The design rotted instead of failing loudly:

| What | Count at deletion |
|---|---|
| Include targets that resolved | 37 |
| **Include targets broken** | **32** |
| mdBook chapters absent from `SUMMARY.md` | ~30 of ~40 |
| `SUMMARY.md` links pointing at a real stub vs. a path that does not exist | divergent, independently |
| CI workflows building the book | **0** |

Every one of the 32 breaks was a `docs/` restructure that nobody reapointed: the stubs
still pointed at `docs/architecture/`, `docs/user/strategy/`, `docs/user/operations/`,
`docs/progreso/`, and the pre-rename `001_*.md` ADR filenames.

## Why it was deleted rather than repaired

1. **No build, no consumer.** `mdbook` appears in no workflow, no script and no manifest.
   The only recorded invocation is manual. The book had not been built since the
   `docs/` restructuring and nothing noticed.
2. **It caused measurable harm while not existing.** `scripts/docs/gen-index.mjs` walks
   `docs/` with no `book` exclusion, so the stubs leaked into the generated indexes:
   `docs/user/index.md` and `llms.txt` were shipping ~75 rows of literal
   `{{#include ../../../../../architecture/adr/…}}` as visible GitHub content.
3. **Nothing validates `{{#include}}` targets.** `check-links.mjs` parses Markdown links;
   `{{#include}}` is not one. That is the actual root cause of the rot — without a
   validator, repairing it just buys time.
4. **Five new moving parts to regenerate it:** a Rust tool install, a stub generator, an
   include-target validator, a CI job, and a Pages deploy — to produce static HTML for a
   project whose declared doc surface is GitHub-rendered Markdown plus `llms.txt`.

## If you bring it back

1. Move the book root **out of `docs/`** (repo root, or `docs/_book/`). That removes it
   from the docs walk and kills the index pollution with no script change. This is the
   single highest-value step.
2. Generate the stubs from the filesystem at build time, using the `title` and `kind`
   that `docs/_schema/frontmatter.schema.json` already guarantees.
3. Add include-target validation to `check-links.mjs`. Without it you are back here.
4. Keep `SUMMARY.md` hand-written. Ordering is an editorial judgement, not a filesystem
   fact; a generated one needs a hand-authored policy anyway.

`SUMMARY.md` in this directory is the last version of the table of contents. It is kept
for the record — do not build from it, most of its targets no longer exist.

## Related

- `docs/dev/plans/2026-09-28-docs-consolidation.md` Task 17 — the decision
- `docs/dev/Backlog.md` FIND-171 — the book build was already recorded as failing
- `.opencode/task-system/memory/lessons.md` — same failure, recorded twice
