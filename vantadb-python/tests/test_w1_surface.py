"""W1 (API-02) surface contract — Python side of the 4-binding parity matrix.

Canonical W1 surface (``docs/api/BINDINGS_NAMESPACES.md`` § W1 parity matrix):

- hybrid search exposes a higher-is-better ``score`` (never a distance);
- node CRUD by explicit id is ``insert_node``/``get_node``/``delete_node``;
- batch writes take an array of objects: ``put_batch([{...}])``;
- cross-namespace search is ``search_multi(namespaces, ...)``;
- ``u128`` ids cross the boundary without precision loss.

The TS/Node/WASM sides assert the same matrix in their own suites; these
tests pin the Python signatures and semantics.
"""

from __future__ import annotations

import inspect

import pytest

import vantadb as vanta
import vantadb_py.vantadb_py as _native


def _client_cls():
    """The real pyclass.

    ``tests/conftest.py`` rebinds the package-level ``Client`` to a tracking
    factory (MOD-16), so class-level assertions must use the compiled
    extension module directly.
    """
    return _native.Client


def _param_names(fn) -> list[str]:
    return [p for p in inspect.signature(fn).parameters if p != "self"]


def test_node_crud_uses_canonical_names_only():
    """Node ops are insert_node/get_node/delete_node; bare names are gone."""
    db = vanta.Client(":memory:", backend="memory")
    try:
        db.insert_node(1, "one", [1.0, 0.0])
        assert db.get_node(1)["id"] == 1
        db.delete_node(1, "w1 contract cleanup")
        assert db.get_node(1) is None
    finally:
        db.close()

    for legacy in ("insert", "get", "delete"):
        assert not hasattr(_client_cls(), legacy), (
            f"legacy node op 'Client.{legacy}' must not exist (W1/API-02)"
        )
    # db.graph exposes the same canonical names (single implementation).
    db = vanta.Client(":memory:", backend="memory")
    try:
        for name in ("insert_node", "get_node", "delete_node"):
            assert hasattr(db.graph, name), f"db.graph.{name} missing"
    finally:
        db.close()


def test_hybrid_search_hit_exposes_score_not_distance():
    """SearchHit carries a relevance score (higher is better), no distance."""
    db = vanta.Client(":memory:", backend="memory")
    try:
        db.put("ns", "k", "payload", vector=[1.0, 0.0, 0.0])
        hits = db.search("ns", [1.0, 0.0, 0.0], top_k=1)
        assert hits, "expected at least one hit"
        hit = hits[0]
        assert isinstance(hit.score, float), f"score must be float, got {type(hit.score)}"
        assert hit.score > 0.99, f"exact match must score ~1.0, got {hit.score}"
        assert not hasattr(hit, "distance"), "SearchHit must not expose distance (W1)"
    finally:
        db.close()


def test_put_batch_signature_is_array_of_objects():
    """put_batch(records) — one positional array, no legacy columnar kwargs."""
    params = _param_names(_client_cls().put_batch)
    assert params == ["records"], f"put_batch params {params} != ['records']"


def test_search_multi_signature_matches_matrix():
    """search_multi(namespaces, query_vector, ...) mirrors the other bindings."""
    params = _param_names(_client_cls().search_multi)
    assert params == [
        "namespaces",
        "query_vector",
        "filters",
        "text_query",
        "top_k",
        "distance_metric",
        "explain",
        "exclude_superseded",
        "query_sparse",  # WIRE-03: sparse parity (additive, end of signature)
        "min_confidence",  # SCH-04: confidence filter (additive, end of signature)
    ], f"search_multi params {params} drifted from the W1 matrix"


def test_u128_node_ids_roundtrip_without_precision_loss():
    """Ids above 2^53 cross the Python boundary exactly (native int)."""
    big = 2**64 + 3  # > 2^53, exercises the u128 wire
    db = vanta.Client(":memory:", backend="memory")
    try:
        db.insert_node(big, "big", [1.0, 0.0])
        node = db.get_node(big)
        assert node is not None and node["id"] == big, (
            f"u128 id must roundtrip exactly, got {node and node['id']!r}"
        )
        record = db.put("ns", "k", "payload")
        assert isinstance(record.node_id, int) and record.node_id > 0
    finally:
        db.close()


def test_async_client_mirrors_canonical_names():
    """AsyncClient mirrors the canonical W1 surface (no legacy node names)."""
    for name in ("insert_node", "get_node", "delete_node", "search_multi"):
        assert hasattr(vanta.AsyncClient, name), f"AsyncClient.{name} missing"
    for legacy in ("insert", "get", "delete"):
        assert not hasattr(vanta.AsyncClient, legacy), (
            f"AsyncClient.{legacy} must not exist (W1/API-02)"
        )
