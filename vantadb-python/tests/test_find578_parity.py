"""FIND-578 — Python binding parity gaps closed in one PyO3 cycle.

Absorbs FIND-199 (``confidence`` on ``put``/``put_batch``), FIND-256
(``runtime_profile`` PascalCase + ``profile`` alias in ``capabilities()``),
FIND-258 (``map_recall_error`` preserves the typed-error contract), and
FIND-582 M-1 (canonical ``vantadb`` stub/package surface) — plus the six
``SearchRequest`` batch fields ``search()`` already exposed
(``exclude_superseded``, ``min_confidence``, ``as_of_ms``, ``valid_window``,
``include_quarantined``, ``search_profile``).
"""

from __future__ import annotations

import ast
import json
from pathlib import Path

import pytest
import vantadb_py as vanta

PKG = Path(__file__).resolve().parents[1]


def _db():
    return vanta.Client(":memory:", backend="memory")


# ── FIND-578: SearchRequest batch parity ────────────────────────────────────


def _write_temporal_fixture(tmp_path):
    """Two v2 records with disjoint validity windows (same shape as
    ``test_sch07._write_temporal_fixture``): early [1000, 2000), late
    [2000, 3000); both vectors adjacent to the probe [1.0, 0.0]."""
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
        {
            **base,
            "namespace": "tt",
            "key": "early",
            "payload": "alpha temporal",
            "valid_at_ms": 1000,
            "invalid_at_ms": 2000,
        },
        {
            **base,
            "namespace": "tt",
            "key": "late",
            "payload": "alpha temporal",
            "valid_at_ms": 2000,
            "invalid_at_ms": 3000,
        },
    ]
    path = tmp_path / "find578-temporal.jsonl"
    path.write_text(
        "\n".join(json.dumps(line) for line in lines) + "\n", encoding="utf-8"
    )
    return path


def _write_quarantine_fixture(tmp_path):
    """One active + one quarantined record (same shape as
    ``test_sch07._write_quarantine_fixture``)."""
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
        {
            **base,
            "namespace": "q",
            "key": "suspect",
            "payload": "alpha shared",
            "quarantined_at_ms": 1500,
            "quarantine_reason": "unreviewed_import",
            "quarantined_by": "system:import",
        },
    ]
    path = tmp_path / "find578-quarantine.jsonl"
    path.write_text(
        "\n".join(json.dumps(line) for line in lines) + "\n", encoding="utf-8"
    )
    return path


def test_batch_request_carries_temporal_params(tmp_path):
    """`as_of_ms`/`valid_window` on batch requests return the same rows as
    `search` — the batch path is no longer a different view of the corpus."""
    db = _db()
    db.import_file(str(_write_temporal_fixture(tmp_path)))

    for kwargs in ({"as_of_ms": 1000}, {"as_of_ms": 2000}, {"valid_window": {"from_ms": 1000, "to_ms": 2000}}):
        sequential = sorted(
            h.key for h in db.search("tt", [1.0, 0.0], top_k=10, **kwargs)
        )
        batch = sorted(
            h.key
            for h in db.search_batch_requests(
                [vanta.SearchRequest(namespace="tt", query_vector=[1.0, 0.0], **kwargs)]
            )[0]
        )
        assert batch == sequential, f"{kwargs}: batch {batch} != search {sequential}"


def test_batch_request_carries_quarantine_view(tmp_path):
    """`include_quarantined` on batch requests mirrors `search` (opt-in)."""
    db = _db()
    db.import_file(str(_write_quarantine_fixture(tmp_path)))

    # Default: quarantined records stay hidden — same as `search`.
    batch = db.search_batch_requests(
        [vanta.SearchRequest(namespace="q", query_vector=[1.0, 0.0])]
    )[0]
    assert [h.key for h in batch] == ["safe"], [h.key for h in batch]

    batch = db.search_batch_requests(
        [
            vanta.SearchRequest(
                namespace="q", query_vector=[1.0, 0.0], include_quarantined=True
            )
        ]
    )[0]
    assert sorted(h.key for h in batch) == ["safe", "suspect"]


def test_batch_request_carries_exclude_superseded():
    """`exclude_superseded` on batch requests hides superseded records."""
    db = _db()
    db.put("ns", "old", "old fact", vector=[1.0, 0.0, 0.0])
    db.put("ns", "new", "new fact", vector=[0.9, 0.1, 0.0])
    db.supersede("ns", "old", "new")

    def keys(req_kwargs):
        return sorted(
            h.key
            for h in db.search_batch_requests(
                [vanta.SearchRequest(namespace="ns", query_vector=[1.0, 0.0, 0.0], **req_kwargs)]
            )[0]
        )

    assert keys({}) == ["new", "old"], "default keeps superseded visible"
    assert keys({"exclude_superseded": True}) == ["new"], (
        "batch must hide the superseded record like search does"
    )


def test_batch_request_carries_min_confidence():
    """`min_confidence` on batch requests filters below-threshold hits."""
    db = _db()
    db.put("ns", "strong", "alpha shared", vector=[1.0, 0.0, 0.0], confidence=1.0)
    db.put("ns", "weak", "alpha shared", vector=[0.9, 0.1, 0.0], confidence=0.3)

    def keys(req_kwargs):
        return sorted(
            h.key
            for h in db.search_batch_requests(
                [vanta.SearchRequest(namespace="ns", query_vector=[1.0, 0.0, 0.0], **req_kwargs)]
            )[0]
        )

    assert keys({}) == ["strong", "weak"]
    assert keys({"min_confidence": 0.5}) == ["strong"]
    # Out-of-range is rejected at the core boundary (same as `search`).
    with pytest.raises(vanta.ValidationError):
        keys({"min_confidence": 1.5})


def test_batch_request_carries_search_profile():
    """`search_profile` dict reaches the core (MEM-01); bad shape/mode/key
    are rejected with Python errors."""
    db = _db()
    db.put("ns", "a", "rust memory engine", vector=[1.0, 0.0, 0.0])

    hits = db.search_batch_requests(
        [
            vanta.SearchRequest(
                namespace="ns",
                text_query="rust",
                search_profile={"mode": "keyword"},
            )
        ]
    )[0]
    assert [h.key for h in hits] == ["a"], hits

    hits = db.search_batch_requests(
        [
            vanta.SearchRequest(
                namespace="ns",
                query_vector=[1.0, 0.0, 0.0],
                search_profile={"mode": "vector"},
            )
        ]
    )[0]
    assert [h.key for h in hits] == ["a"], hits

    with pytest.raises(ValueError, match="Unknown search_profile mode"):
        db.search_batch_requests(
            [vanta.SearchRequest(namespace="ns", search_profile={"mode": "quantum"})]
        )
    with pytest.raises(ValueError, match="Unknown search_profile key"):
        db.search_batch_requests(
            [vanta.SearchRequest(namespace="ns", search_profile={"bogus": 1})]
        )
    with pytest.raises(TypeError, match="must be a dict"):
        db.search_batch_requests(
            [vanta.SearchRequest(namespace="ns", search_profile="hybrid")]
        )


def test_batch_request_dict_form_carries_new_fields():
    """Plain dicts get the same new fields (asdict parity)."""
    db = _db()
    db.put("ns", "a", "alpha", vector=[1.0, 0.0, 0.0])
    db.put("ns", "b", "beta", vector=[0.0, 1.0, 0.0], confidence=0.2)

    req = vanta.SearchRequest(
        namespace="ns", query_vector=[1.0, 0.0, 0.0], min_confidence=0.5
    ).asdict()
    hits = db.search_batch_requests([req])[0]
    assert [h.key for h in hits] == ["a"], hits


# ── FIND-199: confidence on put / put_batch ─────────────────────────────────


def test_put_accepts_declared_confidence():
    """`put(..., confidence=x)` writes the declared score (ADR-046 §D4)."""
    db = _db()
    rec = db.put("ns", "k1", "payload", confidence=0.7)
    assert rec.confidence == pytest.approx(0.7), rec.confidence
    assert rec.confidence_class == "Asserted", rec.confidence_class
    # Read surfaces agree.
    assert db.memory.get("ns", "k1").confidence == pytest.approx(0.7)


def test_put_confidence_out_of_range_rejected():
    """confidence outside [0,1] is a ValidationError — never clamped."""
    db = _db()
    for bad in (1.5, -0.1, float("nan"), float("inf")):
        with pytest.raises(vanta.ValidationError, match="confidence"):
            db.put("ns", "k1", "payload", confidence=bad)


def test_put_batch_confidence_per_record_and_batch_default():
    """Per-record `"confidence"` wins over the batch-level default."""
    db = _db()
    db.put_batch(
        [
            {"namespace": "ns", "key": "own", "payload": "p", "confidence": 0.2},
            {"namespace": "ns", "key": "defaulted", "payload": "p"},
        ],
        confidence=0.8,
    )
    assert db.memory.get("ns", "own").confidence == pytest.approx(0.2)
    assert db.memory.get("ns", "defaulted").confidence == pytest.approx(0.8)


def test_put_batch_confidence_out_of_range_rejected():
    db = _db()
    with pytest.raises(vanta.ValidationError, match="confidence"):
        db.put_batch([{"namespace": "ns", "key": "k", "payload": "p", "confidence": 2.0}])
    with pytest.raises(vanta.ValidationError, match="confidence"):
        db.put_batch([{"namespace": "ns", "key": "k", "payload": "p"}], confidence=9.9)


# ── FIND-256: capabilities() cross-binding shape ────────────────────────────


def test_capabilities_runtime_profile_pascalcase_with_profile_alias():
    """`runtime_profile` is the canonical key (core serde labels, same as
    WASM/TS/Node); `profile` stays as a backward-compatible alias."""
    db = _db()
    caps = db.capabilities()
    assert caps["runtime_profile"] in {"Enterprise", "Performance", "LowResource"}, (
        caps["runtime_profile"]
    )
    assert caps["profile"] == caps["runtime_profile"], (
        f"alias drifted: {caps['profile']!r} != {caps['runtime_profile']!r}"
    )
    # The merged telemetry view carries the same keys.
    hw = db.hardware_profile()
    assert hw["runtime_profile"] == caps["runtime_profile"]
    assert hw["profile"] == caps["runtime_profile"]


# ── FIND-258: RecallError typed mapping ──────────────────────────────────────


def test_recall_malformed_persona_raises_typed_corrupt_error():
    """A persisted persona record that fails to deserialize must surface as
    ``CorruptError`` (data corruption), not collapse to a bare RuntimeError —
    with canonical ``.code``/``.retriable``/``.hint`` metadata and valid
    ``error_to_dict()``, and still catchable through ``vanta.Error``."""
    db = _db()
    db.put("persona/sess-1", "persona.md", "{{{not json")
    with pytest.raises(vanta.CorruptError) as exc:
        db.memory_recall("hello", "sess-1")
    assert exc.value.code == "VANTADB_CORRUPT"
    assert exc.value.retriable is False
    assert isinstance(exc.value.hint, str) and exc.value.hint
    d = vanta.error_to_dict(exc.value)
    assert d["name"] == "CorruptError"
    assert d["code"] == "VANTADB_CORRUPT"
    assert d["retriable"] is False
    assert d["hint"] == exc.value.hint
    # Same path catchable via the hierarchy base (backward compat).
    with pytest.raises(vanta.Error) as base_exc:
        db.memory_recall("hello", "sess-1")
    assert base_exc.value.code == "VANTADB_CORRUPT"


def test_l0_malformed_cursor_raises_typed_corrupt_error():
    """A persisted L0 cursor that fails to deserialize must surface as
    ``CorruptError`` with canonical ``.code``/``.retriable``/``.hint`` metadata
    and valid ``error_to_dict()``."""
    db = _db()
    db.put("l0_cursor/sess-1", "__cursor", "{{{not json")
    with pytest.raises(vanta.CorruptError) as exc:
        db.memory_capture("sess-1", [{"role": "user", "content": "hello"}])
    assert exc.value.code == "VANTADB_CORRUPT"
    assert exc.value.retriable is False
    assert isinstance(exc.value.hint, str) and exc.value.hint
    d = vanta.error_to_dict(exc.value)
    assert d["name"] == "CorruptError"
    assert d["code"] == "VANTADB_CORRUPT"
    assert d["retriable"] is False
    assert d["hint"] == exc.value.hint


# ── FIND-582 M-1: canonical package + stub surface ──────────────────────────


def test_pyi_stubs_parse_cleanly():
    """Every shipped stub must be valid Python syntax (`ast.parse`)."""
    for stub in (
        PKG / "vantadb_py" / "vantadb_py.pyi",
        PKG / "vantadb_py" / "__init__.pyi",
        PKG / "vantadb" / "__init__.pyi",
    ):
        ast.parse(stub.read_text(encoding="utf-8"), filename=str(stub))


def test_canonical_package_all_parity_with_stub():
    """`vantadb/__init__.py` `__all__` must equal `vantadb/__init__.pyi`'s —
    and cover the error hierarchy the runtime re-exports."""

    def _all_of(path: Path) -> list[str]:
        for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
            if isinstance(node, ast.Assign):
                for t in node.targets:
                    if isinstance(t, ast.Name) and t.id == "__all__":
                        return [e.value for e in node.value.elts]
        return []

    py_all = _all_of(PKG / "vantadb" / "__init__.py")
    pyi_all = _all_of(PKG / "vantadb" / "__init__.pyi")
    assert py_all == pyi_all, f"__all__ drift: py={py_all} pyi={pyi_all}"
    for name in (
        "Error",
        "NotFoundError",
        "ValidationError",
        "CorruptError",
        "StorageError",
        "ConflictError",
        "UnsupportedError",
        "ResourceLimitError",
        "BusyError",
        "NoVectorError",
        "TimeoutError",
        "error_to_dict",
        "memory_node_id",
    ):
        assert name in py_all, f"{name} missing from canonical __all__"


def test_canonical_package_resolves_error_surface():
    """Runtime check: `import vantadb` exposes the declared error classes."""
    import vantadb as canonical

    for name in canonical.__all__:
        assert hasattr(canonical, name), f"vantadb.{name} missing at runtime"
    assert issubclass(canonical.NotFoundError, canonical.Error)
