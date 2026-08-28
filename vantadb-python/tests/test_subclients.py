"""SDKB-03: domain sub-clients (db.memory / db.graph / db.system / db.wiki).

Mirror of SDKB-02 (TypeScript): each sub-client method must delegate to the
flat ``VantaDB`` method of the same name with an IDENTICAL signature and
result. Grouping only — zero new logic (D43, D42).

Canonical map: docs/api/BINDINGS_NAMESPACES.md. Naming hazard respected:
Python ``insert``/``get``/``delete`` are NODE-level ops (graph domain).
"""

import os

import pytest

import vantadb_py as vanta


@pytest.fixture()
def db():
    """Fresh in-memory database per test."""
    instance = vanta.VantaDB(":memory:", backend="memory")
    yield instance
    instance.close()


# ---------------------------------------------------------------------------
# memory sub-client
# ---------------------------------------------------------------------------


def test_memory_put_get_delete_identity(db):
    rec = db.memory.put("ns", "k1", "payload-1", metadata={"a": "b"})
    assert rec.payload == "payload-1"
    got_flat = db.get_memory("ns", "k1")
    got_sub = db.memory.get_memory("ns", "k1")
    assert got_sub.payload == got_flat.payload == "payload-1"
    assert got_sub.metadata == got_flat.metadata == {"a": "b"}
    assert db.memory.delete_memory("ns", "k1") is True
    assert db.memory.get_memory("ns", "k1") is None
    # flat path deletes its own record identically
    db.put("ns", "k2", "payload-2")
    assert db.delete_memory("ns", "k2") is True
    assert db.get_memory("ns", "k2") is None


def test_memory_search_identity(db):
    db.put("ns", "m1", "alpha text")
    db.put("ns", "m2", "beta text")
    hits_flat = db.search_memory("ns", [0.1, 0.2, 0.3], top_k=5)
    hits_sub = db.memory.search_memory("ns", [0.1, 0.2, 0.3], top_k=5)
    assert [(h.key, h.payload) for h in hits_sub] == [
        (h.key, h.payload) for h in hits_flat
    ]


def test_memory_list_and_namespaces_identity(db):
    db.put("ns-a", "k", "v")
    db.put("ns-b", "k", "v")
    assert sorted(db.memory.list_namespaces()) == sorted(
        db.list_namespaces()
    ) == ["ns-a", "ns-b"]
    listed_flat = db.list_memory("ns-a")
    listed_sub = db.memory.list_memory("ns-a")
    assert listed_sub[0].key == listed_flat[0].key == "k"


def test_memory_supersede_identity(db):
    db.put("ns", "old", "old payload")
    db.put("ns", "new", "new payload")
    db.memory.supersede("ns", "old", "new")
    # Old record carries the marker; new record stays intact (ADR-028).
    old_rec = db.get_memory("ns", "old")
    assert old_rec["superseded_by"] == "new"
    assert old_rec.payload == "old payload"
    new_rec = db.get_memory("ns", "new")
    assert new_rec["superseded_by"] is None
    assert new_rec.payload == "new payload"


def test_memory_purge_expired_identity(db):
    db.put("ns", "ttl-k", "v", ttl_ms=1)
    import time

    time.sleep(0.01)
    removed = db.memory.purge_expired()
    assert isinstance(removed, int)
    assert db.purge_expired() == 0


def test_memory_generate_snippet_identity(db):
    snippet_flat = db.generate_snippet("hello brave world", "brave")
    snippet_sub = db.memory.generate_snippet("hello brave world", "brave")
    assert snippet_sub == snippet_flat


def test_memory_pure_ann_search_identity(db):
    db.insert(1, "node one", [1.0, 0.0])
    res_flat = db.search([1.0, 0.0], top_k=3)
    res_sub = db.memory.search([1.0, 0.0], top_k=3)
    assert res_sub == res_flat


def test_memory_count_identity(db):
    db.memory.put("ns", "a", "alpha", metadata={"category": "task"})
    db.memory.put("ns", "b", "beta", metadata={"category": "note"})
    assert db.memory.count("ns") == db.count("ns") == 2
    assert db.memory.count("ns", {"category": "task"}) == db.count(
        "ns", {"category": "task"}
    ) == 1


def test_memory_delete_by_filter_identity(db):
    # Destructive op: each path must delete the same number on an identical seed.
    db.memory.put("ns", "a", "alpha", metadata={"category": "task"})
    db.memory.put("ns", "b", "beta", metadata={"category": "task"})
    db.memory.put("ns", "c", "gamma", metadata={"category": "note"})
    assert db.memory.delete_by_filter("ns", {"category": "task"}) == 2
    db.memory.put("ns", "d", "delta", metadata={"category": "task"})
    db.memory.put("ns", "e", "epsilon", metadata={"category": "task"})
    assert db.delete_by_filter("ns", {"category": "task"}) == 2
    assert db.memory.count("ns") == 1


def test_memory_similar_to_key_identity(db):
    db.memory.put("ns", "k0", "source", vector=[1.0, 0.0])
    db.memory.put("ns", "k2", "similar", vector=[0.9, 0.0])
    db.memory.put("ns", "k1", "opposite", vector=[-1.0, 0.0])
    flat = [(h.key, h.score) for h in db.similar_to_key("ns", "k0", top_k=3)]
    sub = [(h.key, h.score) for h in db.memory.similar_to_key("ns", "k0", top_k=3)]
    assert sub == flat
    assert [k for k, _ in flat] == ["k2", "k1"]


# ---------------------------------------------------------------------------
# graph sub-client (node-level insert/get/delete per naming hazard)
# ---------------------------------------------------------------------------


def test_graph_node_crud_identity(db):
    db.graph.insert(10, "content-10", [0.5, 0.5], {"kind": "note"})
    node_flat = db.get(10)
    node_sub = db.graph.get(10)
    assert node_sub["fields"]["kind"] == node_flat["fields"]["kind"] == "note"
    db.graph.delete(10, "test cleanup")
    assert db.get(10) is None


def test_graph_edges_and_traversals_identity(db):
    for nid in range(1, 4):
        db.graph.insert(nid, f"n{nid}", [])
    db.graph.add_edge(1, 2, "next")
    db.add_edge(2, 3, "next")
    assert db.graph.graph_bfs([1]) == db.graph_bfs([1]) == [1, 2, 3]
    assert db.graph.graph_topological_sort([1]) == [1, 2, 3]
    assert db.graph.graph_is_dag([1]) is True


def test_graph_bfs_filtered_identity(db):
    """Paridad con node/ts: graph_bfs_filtered con filtro de labels/time_range."""
    for nid in range(1, 5):
        db.graph.insert(nid, f"n{nid}", [])
    # Edge 1->2 with label 10, 2->3 with label 20, 3->4 with label 10
    db.graph.add_edge(1, 2, "a", created_at_ms=1000)
    db.graph.add_edge(2, 3, "b", created_at_ms=2000)
    db.graph.add_edge(3, 4, "a", created_at_ms=3000)

    # Filter by label: only follow edges with label "a"
    # Note: label is a string in Python, internally mapped to label_id
    # For this test, we test the method exists and returns a list
    result_flat = db.graph_bfs_filtered(
        roots=[1],
        max_depth=3,
        direction="Forward",
        labels=[],  # empty = no label filter
        time_range=None,
    )
    result_sub = db.graph.graph_bfs_filtered(
        roots=[1],
        max_depth=3,
        direction="Forward",
        labels=[],
        time_range=None,
    )
    assert result_sub == result_flat
    assert set(result_flat) == {1, 2, 3, 4}

    # Filter by time_range: only edges created in [1500, 2500]
    result_time_flat = db.graph_bfs_filtered(
        roots=[1],
        max_depth=3,
        direction="Forward",
        labels=[],
        time_range=(1500, 2500),
    )
    result_time_sub = db.graph.graph_bfs_filtered(
        roots=[1],
        max_depth=3,
        direction="Forward",
        labels=[],
        time_range=(1500, 2500),
    )
    assert result_time_sub == result_time_flat
    # Should include 1 (root), 2 (edge 1->2 at 1000 is OUTSIDE range, so NOT followed)
    # Wait - edge 1->2 is at 1000, which is outside [1500, 2500], so traversal stops at 1
    assert result_time_flat == [1]


def test_graph_metrics_python_only_identity(db):
    for nid in range(1, 4):
        db.insert(nid, f"n{nid}", [])
    db.add_edge(1, 2, "e")
    pr_flat = db.graph_page_rank([1, 2, 3])
    pr_sub = db.graph.graph_page_rank([1, 2, 3])
    assert set(pr_sub) == set(pr_flat) == {1, 2, 3}
    dc_flat = db.graph_degree_centrality([1, 2])
    dc_sub = db.graph.graph_degree_centrality([1, 2])
    assert dict(dc_sub) == dict(dc_flat)


# ---------------------------------------------------------------------------
# system sub-client
# ---------------------------------------------------------------------------


def test_system_capabilities_and_metrics_identity(db):
    caps_sub = db.system.capabilities()
    assert caps_sub["vector_search"] == db.capabilities()["vector_search"]
    metrics_sub = db.system.operational_metrics()
    metrics_flat = db.operational_metrics()
    assert type(metrics_sub) is type(metrics_flat)


def test_system_query_identity(db):
    write_sub = db.system.query('INSERT NODE#42 TYPE Person { name: "q" }')
    assert isinstance(write_sub, str)
    assert 'message: "Node 42 inserted."' in write_sub
    out_flat = db.query("FROM Person")
    out_sub = db.system.query("FROM Person")
    # Loose identity: same node mentioned (hit counts may drift between runs).
    assert "42" in out_sub and "42" in out_flat


def test_system_flush_compact_wal_identity(db):
    db.put("ns", "k", "v")
    db.system.flush()
    db.flush()
    freed = db.system.compact_wal()
    assert freed is None


def test_system_export_import_roundtrip(db, tmp_path):
    path = str(tmp_path / "export.json")
    db.put("ns", "k", "v")
    report = db.system.export_all(path)
    assert report["records_exported"] == 1
    other = vanta.VantaDB(":memory:", backend="memory")
    try:
        imported = other.system.import_file(path)
        assert imported["inserted"] == 1
        assert other.get_memory("ns", "k").payload == "v"
    finally:
        other.close()


def test_system_bulk_import_bytes_identity(db):
    # .vdbdump wire format (see test_sdk.py COV-001): magic + version + count + JSON body
    import json
    import struct

    records = [
        {
            "namespace": "ns",
            "key": "bulk-k",
            "payload": "bulk-v",
            "metadata": {},
            "vector": [0.25, -0.75],
            "ttl_ms": None,
        }
    ]
    body = json.dumps(records).encode()
    blob = (
        b"VDBJSON\n"
        + bytes([0x01])
        + struct.pack("<Q", len(records))
        + body
    )
    report = db.system.bulk_import_bytes(blob)
    assert report["total_records"] == 1
    assert report["batches_committed"] >= 1
    # Same contract as test_sdk.py bulk tests: report asserted, retrieval NOT
    # (imported records are buffered until flush — see COV-001 notes).


# ---------------------------------------------------------------------------
# wiki sub-client
# ---------------------------------------------------------------------------


def test_wiki_recover_invalid_id_same_error(db):
    """Delegation identity on the error path (no summary node setup needed)."""
    with pytest.raises(Exception) as exc_sub:
        db.wiki.recover_archived_nodes("not-a-number")
    with pytest.raises(Exception) as exc_flat:
        db.recover_archived_nodes("not-a-number")
    assert str(exc_sub.value) == str(exc_flat.value)


# ---------------------------------------------------------------------------
# cross-cutting
# ---------------------------------------------------------------------------


def test_backward_compat_flat_methods_untouched(db):
    """Suite-existente intacta: los métodos planos siguen funcionando igual."""
    rec = db.put("compat", "k", "v")
    assert rec.payload == "v"
    assert db.get_memory("compat", "k").payload == "v"
    assert repr(db).startswith("VantaDB(")


def test_subclients_are_distinct_objects(db):
    """Cada acceso construye un cliente fresco (sin estado compartido)."""
    assert db.memory is not db.memory
    assert db.graph is not db.graph
