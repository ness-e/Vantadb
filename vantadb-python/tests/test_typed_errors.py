"""MOD-20: typed VantaError hierarchy + structured query results.

The binding exposes a `VantaError` hierarchy (inheriting from `RuntimeError`
so `except RuntimeError` keeps working) mapping the core `VantaError` variants
to specific subclasses, plus `query_structured()` which returns a structured
dict instead of the formatted string of `query()`.
"""

import os
import shutil

import pytest

import vantadb_py as vanta


TEST_DB_PATH = "./test_typed_errors_db"


@pytest.fixture(autouse=True)
def cleanup():
    def _clean():
        for path in glob_like():
            if os.path.exists(path):
                shutil.rmtree(path, ignore_errors=True)

    yield
    _clean()


def glob_like():
    import glob

    return glob.glob(f"{TEST_DB_PATH}_*")


def _db():
    return vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)


def _unique_path():
    import uuid

    return f"{TEST_DB_PATH}_{uuid.uuid4().hex[:8]}"


# ── Hierarchy shape ──────────────────────────────────────────────────────────


def test_hierarchy_base_is_runtimeerror_and_exception():
    """VantaError must be catchable as RuntimeError and Exception (backward compat)."""
    assert issubclass(vanta.VantaError, RuntimeError)
    assert issubclass(vanta.VantaError, Exception)
    for name in (
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
    ):
        cls = getattr(vanta, name)
        assert issubclass(cls, vanta.VantaError), f"{name} must subclass VantaError"


# ── Typed error mapping ──────────────────────────────────────────────────────


def test_supersede_missing_key_raises_not_found():
    """Missing keys on supersede map to NotFoundError (and are VantaError/RuntimeError)."""
    db = _db()
    try:
        db.put("ns", "k", "payload")
        with pytest.raises(vanta.NotFoundError) as exc:
            db.supersede("ns", "ghost", "k")
        assert isinstance(exc.value, vanta.VantaError)
        assert isinstance(exc.value, RuntimeError)
    finally:
        db.close()


def test_supersede_same_key_raises_validation():
    """old == new on supersede maps to ValidationError."""
    db = _db()
    try:
        db.put("ns", "k", "payload")
        with pytest.raises(vanta.ValidationError):
            db.supersede("ns", "k", "k")
    finally:
        db.close()


def test_catch_all_vanta_error_catches_all():
    """A single `except VantaError` should catch both mapped families."""
    db = _db()
    try:
        db.put("ns", "k", "payload")
        for trigger in ("missing", "same"):
            try:
                if trigger == "missing":
                    db.supersede("ns", "ghost", "k")
                else:
                    db.supersede("ns", "k", "k")
                assert False, f"{trigger} should have raised"
            except vanta.VantaError:
                pass  # expected — both families are VantaError
    finally:
        db.close()


def test_legacy_runtimeerror_except_still_works():
    """Old `except RuntimeError` callers still catch VantaDB errors."""
    db = _db()
    try:
        db.put("ns", "k", "payload")
        try:
            db.supersede("ns", "ghost", "k")
            assert False, "should have raised"
        except RuntimeError:
            pass  # backward compat preserved
    finally:
        db.close()


# ── query_structured() ───────────────────────────────────────────────────────


def test_query_structured_write_and_read():
    """query_structured returns a dict with a `kind` discriminator."""
    db = _db()
    try:
        write = db.query_structured('INSERT NODE#42 TYPE Person { name: "queryable" }')
        assert isinstance(write, dict)
        assert write["kind"] == "write"
        assert write["affected_nodes"] >= 1
        assert isinstance(write["node_id"], str)

        result = db.query_structured("FROM Person")
        assert isinstance(result, dict)
        assert result["kind"] == "read"
        assert isinstance(result["nodes"], list)
        ids = [n["id"] for n in result["nodes"]]
        assert "42" in ids, f"query result should include node 42, got {ids}"
    finally:
        db.close()


def test_query_still_returns_string():
    """The legacy query() is unchanged — additive, not breaking."""
    db = _db()
    try:
        write = db.query('INSERT NODE#42 TYPE Person { name: "q" }')
        assert isinstance(write, str)
        result = db.query("FROM Person")
        assert isinstance(result, str)
        assert "42" in result
    finally:
        db.close()
