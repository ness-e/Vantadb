"""DIST-15: GraphRAG exposed on the Python binding.

The core pipeline already exists (`Embedded::graphrag_search`); this suite
pins the Python surface and the canonical wire shape shared by every binding
(Py/TS/Node/WASM): snake_case keys, u128 ids as native ints, complete stats.

Run: ``python -m pytest tests/test_graphrag.py -q`` (after ``maturin develop``).
"""

from __future__ import annotations

import pytest

from vantadb_py import Client, ValidationError


@pytest.fixture()
def db():
    client = Client(":memory:", backend="memory")
    yield client
    client.close()


def _seed_graph(db: Client) -> list[int]:
    records = [
        ("a", "vector database for agents", [0.1, 0.2, 0.3]),
        ("b", "graph expansion uses edges", [0.2, 0.3, 0.4]),
        ("c", "BM25 lexical retrieval engine", [0.3, 0.4, 0.5]),
    ]
    ids = [db.put("graphrag", key, payload, vector=vector).node_id for key, payload, vector in records]
    db.add_edge(ids[0], ids[1], "uses")
    db.add_edge(ids[0], ids[2], "uses")
    return ids


def test_graphrag_search_returns_canonical_dict(db: Client):
    _seed_graph(db)

    result = db.graphrag_search("graphrag", query="vector database")

    assert set(result) == {"nodes", "edges", "context_text", "stats"}
    assert result["nodes"], "expected at least one node"
    node = result["nodes"][0]
    assert set(node) == {"id", "content", "score", "hop_distance"}
    assert isinstance(node["id"], int), "u128 ids are native ints on the Python wire"
    assert isinstance(node["score"], float)
    assert result["context_text"], "context_text must be non-empty"
    assert set(result["stats"]) == {
        "seeds_found",
        "nodes_expanded",
        "total_candidates",
        "expansion_hops_used",
    }
    assert result["stats"]["seeds_found"] > 0
    assert result["edges"], "expected at least one edge"
    edge = result["edges"][0]
    assert set(edge) == {"source", "target", "label"}
    assert isinstance(edge["source"], int)


def test_graphrag_search_accepts_vector_only_query(db: Client):
    _seed_graph(db)

    result = db.graphrag_search("graphrag", query_vector=[0.1, 0.2, 0.3])

    assert result["stats"]["seeds_found"] > 0
    assert result["nodes"], "expected vector seeds to expand"


def test_graphrag_search_empty_namespace_returns_empty_result(db: Client):
    result = db.graphrag_search("missing-namespace", query="anything")

    assert result["nodes"] == []
    assert result["edges"] == []
    assert result["context_text"] == ""
    assert result["stats"]["seeds_found"] == 0
    assert result["stats"]["nodes_expanded"] == 0


def test_graphrag_search_rejects_oversized_query_vector(db: Client):
    _seed_graph(db)

    with pytest.raises(ValidationError):
        db.graphrag_search("graphrag", query_vector=[0.0] * 10_001)
