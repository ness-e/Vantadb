"""DIST-02 — cognitive layer: ``memory_capture`` / ``memory_recall``.

The Python binding re-exports the ``vanta-memory`` hooks: ``memory_capture``
persists conversation turns into L0 (LLM-free, idempotent per cursor) and
``memory_recall`` reads L1 memories + persona + scene navigation for the
current turn. All storage runs against an in-memory VantaDB; no LLM involved
(the pipeline degrades per P4 — recall reports ``effective_mode``).

Canonical map: ``docs/api/PYTHON_SDK.md`` § Cognitive Layer.
"""

from __future__ import annotations

import asyncio
import json

import pytest

import vantadb as vanta


@pytest.fixture()
def db():
    """Fresh in-memory database per test."""
    instance = vanta.Client(":memory:", backend="memory")
    yield instance
    instance.close()


def _l1_record(content: str, session_key: str = "sess-1", record_id: str = "m1") -> dict:
    """Minimal L1 ``MemoryRecord`` wire shape (serde defaults fill the rest)."""
    return {
        "id": record_id,
        "content": content,
        "type": "persona",
        "priority": 80,
        "scene_name": "ui-setup",
        "source_message_ids": [],
        "created_at": "2026-08-20T10:00:00Z",
        "updated_at": "2026-08-20T10:00:00Z",
        "session_key": session_key,
        "session_id": "",
    }


def _seed_l1(db, content: str, session_key: str = "sess-1", record_id: str = "m1") -> None:
    """Persist an L1 record exactly as the recall reader expects it."""
    db.put(
        "l1/" + session_key,
        record_id,
        json.dumps(_l1_record(content, session_key, record_id)),
    )


# ── capture ──────────────────────────────────────────────────────────────────


def test_memory_capture_records_turns_and_reports_counts(db):
    result = db.memory_capture(
        "sess-1",
        [
            {"role": "user", "content": "I love coffee", "timestamp_ms": 1000},
            {"role": "assistant", "content": "Noted!", "timestamp_ms": 1001},
            {"role": "system", "content": "filtered by role", "timestamp_ms": 1002},
        ],
    )
    assert result["recorded_count"] == 2
    assert result["filtered_messages"] == 1  # system role filtered
    assert result["cursor_ms"] == 1001
    # L0 records are visible through the normal memory API.
    stored = db.memory.list("l0/sess-1")
    assert sorted(r.payload for r in stored) == ["I love coffee", "Noted!"]


def test_memory_capture_replay_is_idempotent(db):
    messages = [{"role": "user", "content": "hello", "id": "m1", "timestamp_ms": 2000}]
    first = db.memory_capture("sess-1", messages)
    second = db.memory_capture("sess-1", messages)
    assert first["recorded_count"] == 1
    assert second["recorded_count"] == 0, "cursor: a replayed turn must not duplicate"
    assert len(db.memory.list("l0/sess-1")) == 1


def test_memory_capture_strips_code_blocks_from_assistant(db):
    result = db.memory_capture(
        "sess-1",
        [
            {
                "role": "assistant",
                "content": "answer:\n```python\nx = 1\n```\ndone",
                "timestamp_ms": 3000,
            }
        ],
    )
    assert result["recorded_count"] == 1
    payload = db.memory.list("l0/sess-1")[0].payload
    assert "x = 1" not in payload
    assert "answer:" in payload and "done" in payload


def test_memory_capture_rejects_malformed_messages(db):
    with pytest.raises(TypeError, match=r"messages\[0\] must be a dict"):
        db.memory_capture("sess-1", ["not-a-dict"])
    with pytest.raises(ValueError, match=r"messages\[0\] is missing 'content'"):
        db.memory_capture("sess-1", [{"role": "user"}])
    with pytest.raises(TypeError, match=r"messages\[0\]\['role'\] must be a str"):
        db.memory_capture("sess-1", [{"role": 1, "content": "x"}])


# ── recall ───────────────────────────────────────────────────────────────────


def test_memory_recall_returns_none_when_nothing_to_inject(db):
    assert db.memory_recall("anything", "sess-empty") is None


def test_memory_recall_returns_matching_l1_memory(db):
    _seed_l1(db, "user prefers dark mode")
    result = db.memory_recall("what does the user prefer about dark mode?", "sess-1")
    assert result is not None
    # No embedding hook attached: hybrid degrades to keyword (D38).
    assert result["effective_mode"] == "keyword"
    memories = result["recalled_memories"]
    assert [m["content"] for m in memories] == ["user prefers dark mode"]
    assert memories[0]["score"] >= 1
    assert memories[0]["type"] == "persona"
    assert memories[0]["source_namespace"] == "l1/sess-1"
    assert memories[0]["source_key"] == "m1"
    # Dynamic memories prepend; the stable tools guide appends.
    assert result["prepend_context"] is not None
    assert "<relevant-memories>" in result["prepend_context"]
    assert result["persona"] is None


def test_memory_recall_respects_max_results(db):
    for i in range(3):
        _seed_l1(db, f"topic alpha note {i}", record_id=f"m{i}")
    result = db.memory_recall("alpha", "sess-1", max_results=2)
    assert result is not None
    assert len(result["recalled_memories"]) == 2


def test_memory_recall_accepts_all_scopes_and_rejects_unknown(db):
    for scope in ("session", "agent", "team"):
        assert db.memory_recall("text", "sess-1", scope=scope) is None  # empty pool
    with pytest.raises(ValueError, match="Unknown recall scope"):
        db.memory_recall("text", "sess-1", scope="galaxy")


def test_memory_capture_then_recall_roundtrip(db):
    """The vertical slice: capture persists, recall reads the same store."""
    db.memory_capture(
        "sess-1",
        [{"role": "user", "content": "the launch code is orion", "timestamp_ms": 5000}],
    )
    _seed_l1(db, "launch code is orion", record_id="m-orion")
    result = db.memory_recall("what is the launch code?", "sess-1")
    assert result is not None
    assert result["recalled_memories"][0]["content"] == "launch code is orion"


# ── async surface (AsyncClient wrapper) ──────────────────────────────────────


def test_async_memory_capture_and_recall_roundtrip(db):
    async def scenario():
        async with vanta.AsyncClient(":memory:", backend="memory") as adb:
            captured = await adb.memory_capture(
                "sess-async",
                [{"role": "user", "content": "async note", "timestamp_ms": 7000}],
            )
            assert captured["recorded_count"] == 1
            assert await adb.memory_recall("note", "sess-async") is None
            return captured

    assert asyncio.run(scenario())["recorded_count"] == 1
