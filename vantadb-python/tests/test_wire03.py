"""WIRE-03 — sparse query/insert + text-only + filtros avanzados (Py).

Contrato (plan master Task 17):
- ``query_sparse`` expuesto con roundtrip put → search.
- text-only por la puerta principal: ``query_vector=[]`` + ``text_query``.
- filtros avanzados range/datetime por la forma canónica cross-SDK:
  ``{"campo": valor}`` (``$eq`` implícito) y ``{"campo": {"$gte": v, "$lt": v2}}``.

RED esperado antes de implementar: ``put(sparse_vector=…)`` y
``search(query_sparse=…)`` levantan ``TypeError`` (kwargs inexistentes).
"""

from __future__ import annotations

from datetime import datetime, timezone

import pytest

import vantadb_py

NS = "wire03"


def _client() -> vantadb_py.Client:
    return vantadb_py.Client(":memory:", backend="memory")


def test_sparse_vector_roundtrip_returns_record():
    """put(sparse_vector) → search(query_sparse) encuentra el registro."""
    db = _client()
    db.put(NS, "s1", "sparse doc", vector=[1.0, 0.0], sparse_vector={7: 1.5, 42: 0.75})

    hits = db.search(NS, [1.0, 0.0], query_sparse={7: 1.5}, top_k=5)

    assert [h.key for h in hits] == ["s1"]


def test_sparse_only_search_matches_postings():
    """Sparse puro (sin dense ni texto) devuelve solo los posting con overlap."""
    db = _client()
    db.put(NS, "a", "alpha", sparse_vector={1: 1.0})
    db.put(NS, "b", "beta", sparse_vector={2: 1.0})

    hits = db.search(NS, [], query_sparse={2: 1.0}, top_k=5)

    assert [h.key for h in hits] == ["b"]


def test_put_batch_accepts_sparse_vector():
    """El shape array-of-objects de put_batch acepta sparse_vector."""
    db = _client()
    db.put_batch(
        [{"namespace": NS, "key": "b1", "payload": "p", "sparse_vector": {5: 2.0}}]
    )

    hits = db.search(NS, [], query_sparse={5: 2.0}, top_k=5)

    assert [h.key for h in hits] == ["b1"]


def test_query_vector_empty_with_text_query_is_text_only():
    """``query_vector=[]`` + text_query = solo-BM25 (sin error de vector)."""
    db = _client()
    db.put(NS, "t1", "the quick brown fox jumps", vector=[1.0, 0.0])
    db.put(NS, "t2", "lazy dogs sleep", vector=[0.0, 1.0])

    hits = db.search(NS, [], text_query="quick", top_k=5)

    assert [h.key for h in hits] == ["t1"]


def test_sparse_vector_rejects_non_u32_key():
    """Las dimensiones sparse son u32; una clave no numérica falla claro."""
    db = _client()
    with pytest.raises((TypeError, ValueError)):
        db.put(NS, "bad", "p", sparse_vector={"x": 1.0})


def test_sparse_vector_rejects_f32_overflow():
    """1e39 es finito en f64 pero `inf` en f32 (N1 review) — rechazo claro."""
    db = _client()
    with pytest.raises((TypeError, ValueError)):
        db.put(NS, "overflow", "p", sparse_vector={1: 1e39})


def test_sparse_query_rejects_non_numeric_weight():
    db = _client()
    with pytest.raises((TypeError, ValueError)):
        db.search(NS, [1.0, 0.0], query_sparse={1: "heavy"})


# ── Filtros avanzados: rango + datetime (forma canónica py↔js) ──────────────


def _seed_timeline(db) -> None:
    db.put(NS, "old", "a", metadata={"when": datetime(2026, 1, 1, tzinfo=timezone.utc)})
    db.put(NS, "mid", "b", metadata={"when": datetime(2026, 6, 1, tzinfo=timezone.utc)})
    db.put(NS, "new", "c", metadata={"when": datetime(2026, 12, 1, tzinfo=timezone.utc)})


def test_advanced_filter_datetime_range_count():
    """Range datetime [jun, dic) = 1 — misma forma que el test espejo TS."""
    db = _client()
    _seed_timeline(db)
    lo = datetime(2026, 6, 1, tzinfo=timezone.utc)
    hi = datetime(2026, 12, 1, tzinfo=timezone.utc)

    assert db.count(NS, {"when": {"$gte": lo, "$lt": hi}}) == 1


def test_advanced_filter_datetime_range_delete():
    db = _client()
    _seed_timeline(db)
    lo = datetime(2026, 6, 1, tzinfo=timezone.utc)
    hi = datetime(2026, 12, 1, tzinfo=timezone.utc)

    deleted = db.delete_by_filter(NS, {"when": {"$gte": lo, "$lt": hi}})

    assert deleted == 1
    assert db.count(NS) == 2


def test_advanced_filter_flat_and_multiple_fields():
    """Objeto plano = AND implícito de igualdades (``$and`` canónico)."""
    db = _client()
    db.put(NS, "m", "x", metadata={"tier": "hot", "lang": "en"})
    db.put(NS, "n", "y", metadata={"tier": "hot", "lang": "es"})

    assert db.count(NS, {"tier": "hot", "lang": "es"}) == 1
