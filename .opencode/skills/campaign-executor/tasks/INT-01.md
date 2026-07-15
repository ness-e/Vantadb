---
id: "INT-01"
name: "Publish LangChain adapter to PyPI"
created: "2026-07-14"
module: "integrations/langchain"
status: "ready"
estimate: "1 turn"
---

## Contract
`python -m build integrations/langchain/` produces .tar.gz + .whl;
CI workflow `release-adapters-62.yml` can publish on tag `adapters-v*`; 5/5 adapter tests pass.

## Atomic Steps
1. Verify package builds: `python -m build integrations/langchain/` → ✅
2. Verify tests pass: `python -m pytest integrations/langchain/tests/ -v` → 5/5 ✅
3. Verify CI workflow exists: `.github/workflows/release-adapters-62.yml` → ✅
4. Verify PyPI name available: `GET /pypi/vantadb-langchain/json` → 404 ✅
5. Verify dependency on PyPI: `vantadb-py>=0.2` → v0.2.0 published ✅

## Skills
ponytail full

## Checks
python -m build integrations/langchain/

## Blast Radius
- `integrations/langchain/` — pure Python package
- `vantadb-py` (external PyPI dependency) — already at v0.2.0
- `.github/workflows/release-adapters-62.yml` — CI pipeline (exists, no changes needed)

## Investigation Notes
- Package name `vantadb-langchain` is available on PyPI
- Dependency `vantadb-py>=0.2` resolved by v0.2.0 → all existing releases satisfy it
- 9 adapters in `integrations/`: langchain, llamaindex, mem0, crewai, dspy, haystack, letta, openai, ollama
- CI workflow has OIDC trusted publishing via `pypa/gh-action-pypi-publish`
- Publish trigger: tag `adapters-v*.*.*` (production) or manual dispatch (TestPyPI)
