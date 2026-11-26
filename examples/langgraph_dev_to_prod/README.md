# LangGraph: same graph in dev and prod (VantaDB checkpointer)

The claim: **the graph code does not change between dev and prod — only the
checkpointer does.** Dev uses LangGraph's `InMemorySaver`; prod uses
`VantaDBCheckpointer` (embedded VantaDB, no server). This gap — dev on
`InMemorySaver`, prod on `PostgresSaver` — is the one Vertical 2 of the
[go-to-market plan](../../docs/dev/strategy/GO_TO_MARKET.md) names.

`demo.py` verifies it mechanically, across **five fresh processes**:

| Check | What it proves |
|-------|----------------|
| Backend-free assert on `respond`/`build_graph` (+ fingerprint shown) | The business section names no backend or mode — the assert is the gate (a negative control fails without it); the fingerprint is a same-file display invariant across the processes, informative only |
| dev-first vs prod-first reply | Identical behavior on identical input |
| Fresh runs see the request they just wrote | Each mode actually wrote and read its own checkpoint (the precondition for the resume checks below) |
| dev resume in a new process | **Control:** `InMemorySaver` does not survive the process |
| prod resume in a new process | **Value:** VantaDB checkpoints persist across process restarts |
| Second thread in the same prod DB | Per-thread isolation (no cross-user leakage) |

Deterministic and offline: the graph node is plain Python — no LLM, no API
keys, no network. It tests the checkpointing pipeline, not model quality.

## Run it

```bash
# 1) VantaDB Python SDK — the current PyPI release, or a wheel built from this repo
python -m pip install vantadb-py

# 2) LangGraph runtime
python -m pip install "langgraph>=1,<2"

# 3) The adapter is not on PyPI yet (Alpha — the first `adapters-v*` release
#    publishes it; see integrations/README.md). --no-deps: the adapter's
#    declared pins (`vantadb-py>=0.5.0,<0.7.0`, `langchain-core>=0.3,<1`)
#    predate the current core/framework majors; the refresh ships with the
#    first adapters release. The SDK + LangGraph come from steps 1-2.
python -m pip install --no-deps ./integrations/langchain

python examples/langgraph_dev_to_prod/demo.py
```

Expected tail:

```
  [ok] all 5 processes ran the same script (fingerprint — display invariant)
  [ok] identical behavior on identical input (dev-first == prod-first)
  [ok] fresh runs see the request they just wrote
  [ok] CONTROL: dev (InMemorySaver) does not survive a fresh process
  [ok] VALUE: prod (VantaDB) resumes state written by a previous process
  [ok] two threads in one prod database stay isolated

DEMO PASSED -- ...
```

Any failure exits non-zero with the failing check named. CI runs exactly this
script on every relevant PR (`.github/workflows/ci-frameworks-demo.yml`) — a
regression in VantaDB checkpoint persistence, or a mode/backend reference
leaking into the business section, turns it red.

Track one-pager, install paths and honest limits:
[FRAMEWORKS.md](../../docs/user/FRAMEWORKS.md).
