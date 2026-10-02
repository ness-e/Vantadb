"""SCH-04: confidence fields consumable from the Python SDK (ADR-046 §D2).

Every `MemoryRecord` carries `confidence_class` (``"Asserted"`` | ``"Derived"``),
`confidence` in [0, 1] (``D_a = 1.0`` for asserted), `last_validated_at_ms`
(``None`` = never re-validated) and `derived_from` (empty for asserted). This
pins the read surfaces promised by the slice: ``get`` (``Record``),
``list`` (``ListResult`` records) and ``search`` (``SearchHit``).
"""

import vantadb_py as vanta
import pytest


def _db():
    return vanta.Client(":memory:", backend="memory")


def test_get_exposes_confidence_defaults():
    """`get` returns a Record whose confidence fields match the D_a defaults."""
    db = _db()
    db.put("ns", "k1", "payload", vector=[1.0, 0.0, 0.0])

    rec = db.memory.get("ns", "k1")
    assert rec.confidence_class == "Asserted", rec.confidence_class
    assert rec.confidence == 1.0, rec.confidence
    assert rec.last_validated_at_ms is None, rec.last_validated_at_ms
    assert list(rec.derived_from) == [], rec.derived_from

    # __getitem__ mirrors the typed getters (superseded_by precedent).
    assert rec["confidence_class"] == "Asserted"
    assert rec["confidence"] == 1.0
    assert rec["last_validated_at_ms"] is None
    assert list(rec["derived_from"]) == []


def test_list_exposes_confidence_defaults():
    """`list` records expose the same confidence fields as `get`."""
    db = _db()
    db.put("ns", "k1", "payload", vector=[1.0, 0.0, 0.0])

    page = db.memory.list("ns")
    records = page["records"]
    assert len(records) == 1, f"expected 1 record, got {len(records)}"
    rec = records[0]
    assert rec.confidence_class == "Asserted", rec.confidence_class
    assert rec.confidence == 1.0, rec.confidence
    assert rec.last_validated_at_ms is None, rec.last_validated_at_ms
    assert list(rec.derived_from) == [], rec.derived_from


def test_search_hit_exposes_confidence_defaults():
    """`search` hits expose the record's confidence fields as getters."""
    db = _db()
    db.put("ns", "k1", "payload", vector=[1.0, 0.0, 0.0])

    hits = db.search("ns", [1.0, 0.0, 0.0], top_k=1)
    assert len(hits) == 1, f"expected 1 hit, got {len(hits)}"
    hit = hits[0]
    assert hit.confidence_class == "Asserted", hit.confidence_class
    assert hit.confidence == 1.0, hit.confidence
    assert hit.last_validated_at_ms is None, hit.last_validated_at_ms
    assert list(hit.derived_from) == [], hit.derived_from


def _write_v2_fixture(tmp_path):
    """Two v2 export lines: an asserted parent (1.0) and a derived child
    (0.9 = min(parents) × 0.9, ADR-046 §D4a)."""
    import json

    lines = [
        {
            "schema_version": 2,
            "namespace": "sch04",
            "key": "parent-1",
            "payload": "parent fact",
            "metadata": {},
            "vector": [1.0, 0.0],
            "created_at_ms": 1000,
            "updated_at_ms": 2000,
            "version": 1,
            "expires_at_ms": None,
            "valid_at_ms": 1000,
            "confidence_class": "Asserted",
            "confidence": 1.0,
        },
        {
            "schema_version": 2,
            "namespace": "sch04",
            "key": "child-1",
            "payload": "derived summary",
            "metadata": {},
            "vector": [1.0, 0.0],
            "created_at_ms": 1000,
            "updated_at_ms": 2000,
            "version": 1,
            "expires_at_ms": None,
            "valid_at_ms": 1000,
            "confidence_class": "Derived",
            "confidence": 0.9,
            "derived_from": ["parent-1"],
        },
    ]
    path = tmp_path / "sch04-v2.jsonl"
    path.write_text("\n".join(json.dumps(line) for line in lines) + "\n", encoding="utf-8")
    return path


def test_search_min_confidence_filter(tmp_path):
    """`min_confidence` is an opt-in filter: 0.95 drops the derived record
    (0.9) while the asserted parent (1.0) stays; the field is exposed on both."""
    db = _db()
    db.import_file(str(_write_v2_fixture(tmp_path)))

    hits = db.search("sch04", [1.0, 0.0], top_k=5)
    by_key = {h.key: h for h in hits}
    assert set(by_key) == {"parent-1", "child-1"}, f"expected both, got {set(by_key)}"
    child = by_key["child-1"]
    assert child.confidence_class == "Derived", child.confidence_class
    assert abs(child.confidence - 0.9) < 1e-6, child.confidence
    assert list(child.derived_from) == ["parent-1"], child.derived_from

    filtered = db.search("sch04", [1.0, 0.0], top_k=5, min_confidence=0.95)
    assert [h.key for h in filtered] == ["parent-1"], [h.key for h in filtered]

    # Out-of-range threshold is rejected at the boundary (never clamped).
    with pytest.raises(Exception) as exc:
        db.search("sch04", [1.0, 0.0], min_confidence=1.5)
    assert "min_confidence" in str(exc.value), str(exc.value)
