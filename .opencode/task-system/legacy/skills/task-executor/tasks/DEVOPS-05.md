---
id: "DEVOPS-05"
name: "Unified CI pipeline to publish all adapters to PyPI"
created: "2026-07-14"
module: ".github/workflows"
status: "ready"
estimate: "1 turn"
---

## Contract
All 9 adapters in `integrations/` build, test, and publish via a single CI workflow.
CI workflow `release-adapters-62.yml` exists and covers the full pipeline.

## Atomic Steps
1. Verify CI workflow exists → `.github/workflows/release-adapters-62.yml` ✅
2. Verify all 9 adapters build → all pass ✅
3. Verify workflow tests all adapters → matrix covers all 9 ✅
4. Verify publish jobs (TestPyPI + PyPI production) → both configured with OIDC ✅

## Skills
ponytail full

## Checks
python -m build integrations/*/

## Blast Radius
- `.github/workflows/release-adapters-62.yml` — CI pipeline
- `integrations/*/` — all 9 pure Python adapter packages

## Investigation Notes
- Backlog says "10 adapters" → only 9 exist in `integrations/`. The Rust-based crates (`vantadb-*/`) have `publish = false`.
- Pipeline already complete: test → build → TestPyPI (dispatch) → PyPI (tag `adapters-v*`)
- No code changes needed — the Backlog was stale
