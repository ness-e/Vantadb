"""Shared fixtures for the vantadb_py test suite (MOD-16).

Two things happen here:

1. Every ``VantaDB`` opened during a test is tracked and closed at teardown
   (MOD-16 contract). PyO3 instances are neither gc-visible nor weakref-able,
   so tracking hooks the package attribute (conftest imports before any test
   module; both ``import vantadb_py as vanta`` and ``from vantadb_py import
   VantaDB`` styles resolve it, including ``AsyncVantaDB``'s inner sync
   handle). Reopening a file-backed path closes the previous tracked handle
   first (Fjall takes an exclusive lock per path), mirroring the
   refcount-release timing helpers like ``migrate_from_*`` rely on mid-test.

2. Explicit ``memory_limit_bytes`` values are floored to
   ``_MIN_TEST_MEMORY_LIMIT``. The engine's backpressure guard compares
   PROCESS-WIDE RSS against this limit (FND-01-F1:
   ``src/storage/engine/stats.rs::check_memory_pressure`` reads process RSS,
   not per-engine bytes), so a "small" 128 MiB budget is really a
   whole-process budget. Once heavy optional test deps (chromadb/lancedb add
   ~110 MiB) push the interpreter baseline past 128 MiB x 0.80 default
   threshold, every write rejects with ``ResourceLimit`` and cascades into
   dozens of failures across files, even though each DB holds only a few
   records. The floor restores the tests' evident intent ("a small DB with
   plenty of headroom") under the engine's actual semantics.
"""

import os

import pytest

import vantadb_py

_MIN_TEST_MEMORY_LIMIT = 1024 * 1024 * 1024  # 1 GiB — interpreter + test deps baseline + headroom

_REGISTRY = []  # VantaDB instances created during the current test
_BY_PATH = {}   # abspath -> instance, for file-backed handles only


def _close_quietly(db):
    try:
        db.close()  # idempotent on the Rust side; teardown must never mask test results
    except Exception:
        pass


def _drain():
    for db in _REGISTRY:
        _close_quietly(db)
    _REGISTRY.clear()
    _BY_PATH.clear()


_original_vantadb = vantadb_py.VantaDB


def _tracking_vantadb(*args, **kwargs):
    path = args[0] if args else kwargs.get("db_path", "")
    key = os.path.abspath(path) if path and path != ":memory:" else None
    if key is not None:
        prev = _BY_PATH.pop(key, None)
        if prev is not None:
            _close_quietly(prev)  # release the Fjall lock BEFORE reopening the path
            try:
                _REGISTRY.remove(prev)
            except ValueError:
                pass
    args = list(args)
    if len(args) >= 2 and isinstance(args[1], int):
        args[1] = max(args[1], _MIN_TEST_MEMORY_LIMIT)
    if isinstance(kwargs.get("memory_limit_bytes"), int):
        kwargs["memory_limit_bytes"] = max(kwargs["memory_limit_bytes"], _MIN_TEST_MEMORY_LIMIT)
    db = _original_vantadb(*args, **kwargs)
    if key is not None:
        _BY_PATH[key] = db
    _REGISTRY.append(db)
    return db


# Rebind the package global so every construction path is tracked.
vantadb_py.VantaDB = _tracking_vantadb


@pytest.fixture(autouse=True)
def close_all_vanta_dbs():
    """Close every VantaDB opened by the finished test (MOD-16)."""
    _drain()  # defensive: nothing should survive the previous teardown
    yield
    _drain()
