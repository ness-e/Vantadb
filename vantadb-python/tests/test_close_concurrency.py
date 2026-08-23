"""MOD-17: concurrent close() must not deadlock the interpreter.

Root cause (pre-fix): ``VantaDB.close()`` called ``OpGate::drain()`` while
holding the GIL. The drain waits on a condvar until the in-flight op count
reaches zero, but every in-flight op returning from its own ``py.detach``
must re-acquire the GIL before it can drop its ``OpGuard`` — so the count
never reaches zero and both threads block forever.

Fix: drain runs inside ``py.detach`` (PyO3 0.29 parallelism guide: always
detach when blocked work needs the GIL back).
"""

import faulthandler
import threading
import time

import pytest

import vantadb_py

CLOSE_TIMEOUT_SECONDS = 30


def _worker_loop(db, stop, errors, i):
    """Spin put/get until the stop event; bail on the first real error."""
    n = 0
    while not stop.is_set():
        key = f"k{i}-{n}"
        try:
            db.put("stress", key, "x" * 256)
            db.get_memory("stress", key)
        except Exception as exc:
            # Races against the durability barrier are expected once
            # close() starts; anything else is a real failure.
            if "closing" not in str(exc).lower():
                errors.append(exc)
            return
        n += 1


def test_close_concurrent_stress(tmp_path):
    # The GIL deadlock freezes the WHOLE interpreter: even the main thread's
    # own timeout assert cannot run (it needs the GIL back to raise). The
    # faulthandler watchdog runs at C level, needs no GIL, and hard-exits
    # with all-thread tracebacks — turning the hang into a fast CI failure.
    faulthandler.dump_traceback_later(CLOSE_TIMEOUT_SECONDS, exit=True)
    db = vantadb_py.VantaDB(str(tmp_path / "mod17"))
    stop = threading.Event()
    closed = threading.Event()
    errors: list[Exception] = []

    workers = [
        threading.Thread(
            target=_worker_loop, args=(db, stop, errors, i), daemon=True
        )
        for i in range(4)
    ]
    for w in workers:
        w.start()

    time.sleep(0.1)  # guarantee ops are mid-flight when close() lands

    def closer():
        try:
            db.close()
        except Exception as exc:  # pragma: no cover - only on real breakage
            errors.append(exc)
        finally:
            closed.set()

    closer_thread = threading.Thread(target=closer, daemon=True)
    closer_thread.start()

    # Hard timeout: with the GIL deadlock this event never fires and the
    # assert fails instead of hanging the suite. All threads are daemons,
    # so pytest can still exit after the failure.
    assert closed.wait(CLOSE_TIMEOUT_SECONDS), (
        "close() did not return within "
        f"{CLOSE_TIMEOUT_SECONDS}s — OpGate::drain() deadlock with GIL held (MOD-17)"
    )
    stop.set()
    for w in workers:
        w.join(timeout=5)
    faulthandler.cancel_dump_traceback_later()
    assert not errors, errors[:5]


def test_close_is_rejecting_new_ops_after_close(tmp_path):
    """Sanity: post-close contract unchanged (closing flag rejects new ops)."""
    db = vantadb_py.VantaDB(str(tmp_path / "mod17b"))
    db.put("ns", "k", "v")
    db.close()
    with pytest.raises(RuntimeError, match="closing"):
        db.put("ns", "k2", "v2")
