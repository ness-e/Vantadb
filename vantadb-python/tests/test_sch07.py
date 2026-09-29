"""SCH-07: v2 fields + temporal/quarantine params consumable from Python.

The 10 v2 record fields (bitemporal + confidence + quarantine) are exposed as
typed getters on both ``Record`` and ``SearchHit``; the query params
(``as_of_ms``/``valid_window``/``include_quarantined`` + ``min_confidence``)
cross ``search``/``search_multi``/``list`` with the same wire names as the
SDK. ADR-046 §D2/§D3/§D5.
"""

import json

import vantadb_py as vanta
import pytest


def _db():
    return vanta.Client(":memory:", backend="memory")


def test_record_exposes_v2_bitemporal_and_quarantine_fields():
    """`get`/`list`/`search` expose the v2 fields with normalized defaults."""
    db = _db()
    db.put("ns", "k1", "payload", vector=[1.0, 0.0, 0.0])
    rec = db.memory.get("ns", "k1")

    # v1 normalization: valid_at := created_at (ADR-046 §D7), open window.
    assert rec.valid_at_ms == rec.created_at_ms, (rec.valid_at_ms, rec.created_at_ms)
    assert rec.invalid_at_ms is None
    # Active record: no quarantine state.
    assert rec.quarantined_at_ms is None
    assert rec.quarantine_reason is None
    assert rec.quarantined_by is None
    assert rec.quarantine_review_due_ms is None

    # __getitem__ mirrors the typed getters (SCH-04 precedent).
    assert rec["valid_at_ms"] == rec.created_at_ms
    assert rec["invalid_at_ms"] is None
    assert rec["quarantined_at_ms"] is None
    assert rec["quarantine_reason"] is None
    assert rec["quarantined_by"] is None
    assert rec["quarantine_review_due_ms"] is None

    # SearchHit mirrors the same flat view.
    hit = db.search("ns", [1.0, 0.0, 0.0], top_k=1)[0]
    assert hit.valid_at_ms == rec.valid_at_ms
    assert hit.invalid_at_ms is None
    assert hit.quarantined_at_ms is None
    assert hit.quarantine_reason is None

    page = db.memory.list("ns")
    rec_listed = page["records"][0]
    assert rec_listed.valid_at_ms == rec.created_at_ms
    assert rec_listed.quarantined_at_ms is None


def _write_temporal_fixture(tmp_path):
    """Two v2 records with disjoint validity windows: early [1000, 2000),
    late [2000, 3000); both vectors are adjacent to the probe [1.0, 0.0]."""
    base = {
        "schema_version": 2,
        "metadata": {},
        "vector": [1.0, 0.0],
        "created_at_ms": 1000,
        "updated_at_ms": 1000,
        "version": 1,
        "confidence_class": "Asserted",
        "confidence": 1.0,
    }
    lines = [
        {**base, "namespace": "tt", "key": "early", "payload": "alpha temporal",
         "valid_at_ms": 1000, "invalid_at_ms": 2000},
        {**base, "namespace": "tt", "key": "late", "payload": "alpha temporal",
         "valid_at_ms": 2000, "invalid_at_ms": 3000},
    ]
    path = tmp_path / "sch07-temporal.jsonl"
    path.write_text("\n".join(json.dumps(line) for line in lines) + "\n", encoding="utf-8")
    return path


def test_search_as_of_and_valid_window(tmp_path):
    """`as_of_ms`/`valid_window` filter by valid time with the ADR-046 §D3
    predicate (start inclusive, end exclusive; window overlap half-open)."""
    db = _db()
    db.import_file(str(_write_temporal_fixture(tmp_path)))

    def keys(**kwargs):
        return sorted(h.key for h in db.search("tt", [1.0, 0.0], top_k=10, **kwargs))

    assert keys() == ["early", "late"], "default view unchanged"
    assert keys(as_of_ms=999) == []
    assert keys(as_of_ms=1000) == ["early"], "start is inclusive"
    assert keys(as_of_ms=1999) == ["early"]
    assert keys(as_of_ms=2000) == ["late"], "end is exclusive"
    assert keys(as_of_ms=2999) == ["late"]
    assert keys(as_of_ms=3000) == []

    assert keys(valid_window={"from_ms": 1000, "to_ms": 2000}) == ["early"]
    assert keys(valid_window={"from_ms": 1000, "to_ms": 2001}) == ["early", "late"]
    # Uniform window includes both; none matches an empty range (rejected).
    assert keys(valid_window={"from_ms": 0, "to_ms": 10_000}) == ["early", "late"]

    # Inverted window is rejected at the core boundary (never swapped).
    with pytest.raises(Exception) as exc:
        db.search("tt", [1.0, 0.0], valid_window={"from_ms": 10, "to_ms": 1})
    assert "valid_window" in str(exc.value), str(exc.value)

    # Shape errors from the Python frontier raise ValueError.
    with pytest.raises(ValueError):
        db.search("tt", [1.0, 0.0], valid_window={"from_ms": 1})
    with pytest.raises(ValueError):
        db.search("tt", [1.0, 0.0], valid_window={"from_ms": "soon", "to_ms": 5})

    # `list` carries the same temporal params.
    page = db.memory.list("tt", as_of_ms=1000)
    assert [r.key for r in page["records"]] == ["early"], page["records"]


def _write_quarantine_fixture(tmp_path):
    """One active record + one quarantined record imported with visible state
    (unreviewed_import, T1c — import preserves the quarantine roundtrip)."""
    base = {
        "schema_version": 2,
        "metadata": {},
        "vector": [1.0, 0.0],
        "created_at_ms": 1000,
        "updated_at_ms": 1000,
        "version": 1,
        "valid_at_ms": 1000,
        "confidence_class": "Asserted",
        "confidence": 1.0,
    }
    lines = [
        {**base, "namespace": "q", "key": "safe", "payload": "alpha shared"},
        {**base, "namespace": "q", "key": "suspect", "payload": "alpha shared",
         "quarantined_at_ms": 1500, "quarantine_reason": "unreviewed_import",
         "quarantined_by": "system:import"},
    ]
    path = tmp_path / "sch07-quarantine.jsonl"
    path.write_text("\n".join(json.dumps(line) for line in lines) + "\n", encoding="utf-8")
    return path


def test_include_quarantined_and_list_min_confidence(tmp_path):
    """Quarantined records are excluded by default; opt-in shows them.
    `list` carries the same opt-in plus `min_confidence` (ADR-046 §D2)."""
    db = _db()
    db.import_file(str(_write_quarantine_fixture(tmp_path)))

    hits = db.search("q", [1.0, 0.0], top_k=10)
    assert [h.key for h in hits] == ["safe"], [h.key for h in hits]

    hits = db.search("q", [1.0, 0.0], top_k=10, include_quarantined=True)
    assert sorted(h.key for h in hits) == ["safe", "suspect"], [h.key for h in hits]

    # get() returns the quarantined record with visible state (never 404).
    rec = db.memory.get("q", "suspect")
    assert rec is not None
    assert rec.quarantined_at_ms == 1500, rec.quarantined_at_ms
    assert rec.quarantine_reason == "unreviewed_import", rec.quarantine_reason
    assert rec.quarantined_by == "system:import", rec.quarantined_by

    page = db.memory.list("q")
    assert [r.key for r in page["records"]] == ["safe"], page["records"]

    page = db.memory.list("q", include_quarantined=True)
    assert sorted(r.key for r in page["records"]) == ["safe", "suspect"]

    # min_confidence is opt-in on list (SCH-07, ADR-046 §D2).
    page = db.memory.list("q", min_confidence=0.5)
    assert [r.key for r in page["records"]] == ["safe"], page["records"]

    with pytest.raises(Exception) as exc:
        db.memory.list("q", min_confidence=1.5)
    assert "min_confidence" in str(exc.value), str(exc.value)

    # `search_multi` accepts the same params (W1/API-02 surface).
    hits = db.search_multi(["q"], [1.0, 0.0], top_k=10, include_quarantined=True)
    assert sorted(h.key for h in hits) == ["safe", "suspect"], [h.key for h in hits]
