---
id: "INT-02"
name: "Publish LlamaIndex adapter to PyPI"
created: "2026-07-14"
module: "integrations/llamaindex"
status: "ready"
estimate: "1 turn"
---

## Contract
`python -m build integrations/llamaindex/` produces .tar.gz + .whl;
CI workflow `release-adapters-62.yml` covers llamaindex in matrix; 5/5 adapter tests pass.

## Atomic Steps
1. Verify package builds → ✅
2. Verify tests pass → 5/5 ✅
3. Verify CI workflow covers llamaindex → ✅ (in matrix)
4. Verify PyPI name available → 404 ✅
5. Verify dependency `vantadb-py>=0.2` → ✅

## Skills
ponytail full

## Checks
python -m build integrations/llamaindex/

## Blast Radius
- `integrations/llamaindex/` — pure Python package
- `vantadb-py` (external dependency, already on PyPI)
- `.github/workflows/release-adapters-62.yml` — CI pipeline covers llamaindex

## Investigation Notes
- Package name `vantadb-llamaindex` available on PyPI (404)
- Dependency `vantadb-py>=0.2` resolved by v0.2.0
- Publish trigger: tag `adapters-v0.3.0` on CI, covers all 9 adapters
- No code changes needed — CI workflow already handles it
