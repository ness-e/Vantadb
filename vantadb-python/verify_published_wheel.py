"""Post-publish verification for the vantadb-py wheel (audit ST3.3.3).

Run against a wheel installed from a package index (TestPyPI / PyPI) to confirm
the published artifact actually imports and works end-to-end, not just that the
module loads. Exercises the persistent-memory API: put, get, list, vector
search, capabilities, and durability across a reopen.

Usage:
    python verify_published_wheel.py

Environment:
    VANTADB_EXPECTED_VERSION  If set, assert ``vantadb_py.__version__`` matches.
"""

import os
import sys
import tempfile

import vantadb_py


def _check_version() -> None:
    expected = os.environ.get("VANTADB_EXPECTED_VERSION")
    actual = vantadb_py.__version__
    print(f"vantadb_py version: {actual}")
    if expected:
        assert actual == expected, (
            f"Version mismatch: got {actual!r}, expected {expected!r}"
        )
        print(f"Version matches expected {expected!r}")


def _check_functional(db_path: str) -> None:
    db = vantadb_py.VantaDB(db_path=db_path)

    record = db.put(
        "verify/main",
        "first",
        "verified",
        metadata={"category": "verify", "version": 1},
        vector=[0.1, 0.2, 0.3],
    )
    assert record is not None, "put returned no record"

    read = db.get_memory("verify/main", "first")
    assert read is not None, "get_memory returned None"
    assert read["payload"] == "verified", f"unexpected payload: {read['payload']!r}"

    page = db.list_memory("verify/main", filters={"category": "verify"})
    assert len(page["records"]) == 1, f"expected 1 record, got {len(page['records'])}"

    hits = db.search_memory("verify/main", [0.1, 0.2, 0.3], top_k=1)
    assert hits, "vector search returned no hits"
    assert hits[0].key == "first", "nearest neighbour mismatch"

    caps = db.capabilities()
    assert caps["persistence"] is True, "persistence capability not advertised"

    db.flush()
    db.close()

    # Durability: reopen and confirm the record survived.
    db2 = vantadb_py.VantaDB(db_path=db_path)
    reread = db2.get_memory("verify/main", "first")
    assert reread is not None, "record lost after reopen"
    assert reread["payload"] == "verified", "payload corrupted after reopen"
    db2.close()


def main() -> int:
    print("--- VERIFY PUBLISHED WHEEL: START ---")
    _check_version()
    with tempfile.TemporaryDirectory(prefix="vantadb_verify_") as tmp:
        _check_functional(os.path.join(tmp, "db"))
    print("--- VERIFY PUBLISHED WHEEL: PASSED ---")
    return 0


if __name__ == "__main__":
    sys.exit(main())