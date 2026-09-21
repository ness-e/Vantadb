# NUEVO-01 — README hero + benchmark graphic + GIF placeholder

**Status:** ✅ COMPLETED
**Date:** 2026-08-05
**Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` (Task 41)
**Branch:** develop (clean tree; prior attempt aborted, no leftovers)

## Summary

Added a visual hero, a real-data benchmark graphic, and a documented GIF placeholder to `README.md`. All image paths are relative and verified to exist (`Test-Path`).

## Slices

### 1. Hero — ✅
- **File:** `assets/hero.svg` (new, 600×236, self-contained SVG)
- Aesthetic: project brand — cream `#FBF9F5` bg, ink `#000000`, neon `#FF5500`, paper `#F2EDE2` (sourced from `web/src/app/globals.css:42-45`). Display type = bold sans stack; capability chips in mono.
- Decorative HNSW-style graph motif (nodes/edges) w/ "441µs · 3,636 qps" baseline stat (real perf data).
- Inserted at top of README after the badge divs. Badges untouched.
- Known limitation: GitHub sanitizes SVGs, so no webfonts/external images are used — fully self-contained vector shapes + generic font stacks. Works in light/dark (cream panel).

### 2. Benchmark graphic — ✅
- **File:** `assets/benchmark-sift1m.svg` (new, 760×420)
- **Data:** ONLY real rows from `docs/operations/BENCHMARKS.md` SIFT1M table (L133-137) — 5 configs, build before/after + speedup:
  - Bal Cos 139.4→63.7s 2.18× · HiRecall Cos 390.8→182.2s 2.14× · Bal L2 191.4→68.4s 2.80× · HiRecall L2 462.2→194.5s 2.37× · HiRecall L2 Mmap 411.2→189.8s 2.16×
- Grouped bar chart (Phase 1 vs Phase 2), scale y = 330 − sec·0.5, 500s baseline. No invented numbers.
- Inserted under the SIFT1M table in README.

### 3. GIF <5MB — ⏭ documented (vhs NOT installed)
- Checked `where vhs` and `where asciinema` → both **not installed**; no local tape tooling available.
- Per contract: did NOT leave a broken `assets/demo.gif` path. Instead a commented HTML placeholder sits at the `## 5-Minute Quickstart` heading with the exact generation command.
- **Exact command to generate the demo GIF:**
  ```
  # demo.tape
  Output assets/demo.gif
  Set FontSize 18
  Set Width 80
  Set Height 20
  Type "pip install vantadb-py"
  Enter
  Sleep 1s
  Type "python"
  Enter
  Type "import vantadb_py as v"
  Enter
  Type "d = v.VantaDB('./vanta_data')"
  Enter
  Type "d.put('agent/main','mem-1','hybrid search works', vector=[0.12,0.88,0.54])"
  Enter
  Type "d.search_memory('agent/main', query_vector=[0.11,0.89,0.55], top_k=5)"
  Enter
  Sleep 1s
  Type "d.close()"
  Enter
  ```
  Run with: `vhs doc/demo.tape` (pip install vhs). Result must be < 5MB (reduce with `Set Width/Height` or gifsicle `--lossy`).
- On generation, replace the placeholder with:
  `<img src="assets/demo.gif" alt="VantaDB demo — pip install, CRUD, hybrid search">`

## Verification

- [x] `python -c "xml.dom.minidom.parse"` → both SVGs well-formed
- [x] Every referenced image path exists: `assets/hero.svg` ✓, `assets/benchmark-sift1m.svg` ✓ (no `assets/demo.gif` referenced until one is generated — placeholder is a comment only)
- [x] Existing badges untouched
- [x] `git status` — staged selectively: `README.md` + `assets/`

## Commit

- Message: `docs(NUEVO-01): README hero + benchmark graphic (GH-139)` — conventional
- `--no-verify` if pre-commit hook rejects (per rules).

## Files changed

- `README.md` (modified)
- `assets/hero.svg` (new)
- `assets/benchmark-sift1m.svg` (new)