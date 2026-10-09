"""DIST-17 — Python producer for the cross-language conformance scenario.

Runs the canonical scenario from ``tests/parity/scenario.json`` (shared by the
Py/Node/WASM producers) and writes the Python binding's projection to
``target/bindings-parity/python.json``. The cross-language comparison itself is
``dev-tools/parity-compare.mjs`` (canonical hash + diff); this test only emits
its own artifact and sanity-checks the projection so a silently-empty binding
fails here instead of "passing" the comparator.

Wall-clock fields (``created_at_ms``, ``updated_at_ms``, ``valid_at_ms``,
``last_accessed``) are intentionally NOT part of the projection — they are not
deterministic by design. Node ids and scores are; ids travel as decimal strings
(u128 > 2^53, API-01).
"""

from __future__ import annotations

import json
from pathlib import Path

import vantadb

_ROOT = Path(__file__).resolve().parents[2]
_SCENARIO_PATH = _ROOT / "tests" / "parity" / "scenario.json"
_ARTIFACT_PATH = _ROOT / "target" / "bindings-parity" / "python.json"


def _load_scenario() -> dict:
    return json.loads(_SCENARIO_PATH.read_text(encoding="utf-8"))


def _run_scenario(scenario: dict) -> dict:
    db = vantadb.Client(":memory:", backend="memory")
    try:
        ids: dict[str, str] = {}
        put_rows = []
        for record in scenario["records"]:
            stored = db.put(
                scenario["namespace"],
                record["key"],
                record["payload"],
                vector=record["vector"],
            )
            ids[record["key"]] = str(stored.node_id)
            put_rows.append({"key": record["key"], "node_id": str(stored.node_id)})

        search = scenario["search"]
        hits = db.search(
            scenario["namespace"],
            search["query_vector"],
            text_query=search["text_query"],
            top_k=search["top_k"],
        )
        search_rows = [{"id": str(hit.node_id), "score": float(hit.score)} for hit in hits]

        graph = scenario["graph"]
        for edge in graph["edges"]:
            db.add_edge(int(ids[edge["source_key"]]), int(ids[edge["target_key"]]), edge["label"])
        bfs = [
            str(node_id)
            for node_id in db.graph_bfs(
                [int(ids[graph["root_key"]])], graph["max_depth"], graph["direction"]
            )
        ]

        iql = scenario["iql"]
        write = db.query_structured(iql["write"])
        read = db.query_structured(iql["read"])
        iql_row = {
            "write_node_id": str(write["node_id"]),
            "read_ids": sorted(str(node["id"]) for node in read["nodes"]),
        }
    finally:
        db.close()

    return {
        "binding": "python",
        "scenario": scenario["scenario"],
        "excluded": [],
        "steps": {
            "put": put_rows,
            "search": search_rows,
            "graph_bfs": bfs,
            "iql": iql_row,
        },
    }


def test_canonical_scenario_produces_binding_artifact():
    """Runs put/search/graph/IQL and emits the canonical Python artifact."""
    scenario = _load_scenario()
    artifact = _run_scenario(scenario)

    # Sanity: the projection must be complete before it is written — an empty
    # or partial projection would otherwise pass the comparator vacuously.
    put_rows = artifact["steps"]["put"]
    search_rows = artifact["steps"]["search"]
    bfs = artifact["steps"]["graph_bfs"]
    expected_ids = {row["node_id"] for row in put_rows}

    assert len(put_rows) == len(scenario["records"])
    assert all(row["node_id"].isdigit() for row in put_rows)
    assert len(expected_ids) == len(put_rows), "node ids must be distinct"

    assert len(search_rows) >= 2
    assert all(row["id"] in expected_ids and row["score"] > 0 for row in search_rows)

    assert bfs[0] == artifact["steps"]["put"][0]["node_id"], "BFS must start at the root"
    assert set(bfs) == expected_ids, f"BFS must visit every record node, got {bfs}"

    assert artifact["steps"]["iql"]["read_ids"] == ["42424242"]

    _ARTIFACT_PATH.parent.mkdir(parents=True, exist_ok=True)
    _ARTIFACT_PATH.write_text(json.dumps(artifact, indent=2, sort_keys=True) + "\n", encoding="utf-8")
