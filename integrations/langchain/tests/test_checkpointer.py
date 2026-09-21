"""Tests for VantaDB LangGraph checkpointer adapter (INTG-01)."""
import pytest
pytest.importorskip("langgraph.checkpoint.base", reason="langgraph-checkpoint not installed; checkpointer suite skipped")
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_langchain import VantaDBCheckpointer


def _checkpoint(cid, values):
    return {
        "v": 1,
        "ts": "2026-09-10T00:00:00+00:00",
        "id": cid,
        "channel_values": dict(values),
        "channel_versions": {k: 1 for k in values},
        "versions_seen": {},
    }


@pytest.fixture
def saver(tmp_path):
    return VantaDBCheckpointer(db_path=str(tmp_path / "ckpt_db"))


def test_put_and_get_tuple(saver):
    config = {"configurable": {"thread_id": "t1", "checkpoint_ns": ""}}
    ckpt = _checkpoint("cid-1", {"x": 1})
    metadata = {"source": "input", "step": 0}
    returned = saver.put(config, ckpt, metadata, {"x": 1})
    assert returned["configurable"]["checkpoint_id"] == "cid-1"

    got = saver.get_tuple({**config, "configurable": {**config["configurable"]}})
    assert got is not None
    assert got.checkpoint["channel_values"] == {"x": 1}
    assert got.metadata["step"] == 0


def test_get_latest_without_id(saver):
    config = {"configurable": {"thread_id": "t1", "checkpoint_ns": ""}}
    saver.put(config, _checkpoint("cid-1", {"x": 1}), {"step": 0}, {})
    saver.put(config, _checkpoint("cid-2", {"x": 2}), {"step": 1}, {})
    got = saver.get_tuple(config)
    assert got is not None
    assert got.checkpoint["channel_values"] == {"x": 2}


def test_get_missing_thread_returns_none(saver):
    config = {"configurable": {"thread_id": "ghost", "checkpoint_ns": ""}}
    assert saver.get_tuple(config) is None


def test_put_writes_visible_as_pending(saver):
    config = {"configurable": {"thread_id": "t1", "checkpoint_ns": ""}}
    saver.put(config, _checkpoint("cid-1", {}), {}, {})
    write_config = {
        "configurable": {"thread_id": "t1", "checkpoint_ns": "", "checkpoint_id": "cid-1"}
    }
    saver.put_writes(write_config, [("node-a", {"out": 1})], task_id="task-1")
    got = saver.get_tuple(write_config)
    assert got is not None
    assert ("task-1", "node-a", {"out": 1}) in (got.pending_writes or [])


def test_list_filter_and_limit(saver):
    config = {"configurable": {"thread_id": "t1", "checkpoint_ns": ""}}
    saver.put(config, _checkpoint("cid-1", {"x": 1}), {"step": 0}, {})
    saver.put(config, _checkpoint("cid-2", {"x": 2}), {"step": 1}, {})
    saver.put({"configurable": {"thread_id": "t2", "checkpoint_ns": ""}},
              _checkpoint("cid-1", {"x": 9}), {"step": 0}, {})

    all_t1 = list(saver.list(config))
    assert len(all_t1) == 2
    filtered = list(saver.list(config, filter={"step": 1}))
    assert len(filtered) == 1
    assert filtered[0].checkpoint["channel_values"] == {"x": 2}
    limited = list(saver.list(config, limit=1))
    assert len(limited) == 1


def test_delete_thread(saver):
    config = {"configurable": {"thread_id": "t1", "checkpoint_ns": ""}}
    saver.put(config, _checkpoint("cid-1", {"x": 1}), {}, {})
    saver.delete_thread("t1")
    assert saver.get_tuple(config) is None
    assert list(saver.list(config)) == []


@pytest.mark.asyncio
async def test_async_roundtrip(saver):
    config = {"configurable": {"thread_id": "t1", "checkpoint_ns": ""}}
    await saver.aput(config, _checkpoint("cid-1", {"x": 5}), {"step": 0}, {})
    got = await saver.aget_tuple(config)
    assert got is not None
    assert got.checkpoint["channel_values"] == {"x": 5}
    items = [t async for t in saver.alist(config)]
    assert len(items) == 1
    await saver.adelete_thread("t1")
    assert await saver.aget_tuple(config) is None


def test_end_to_end_with_compiled_graph(tmp_path):
    """Proof of contract: real StateGraph persists + resumes via thread_id."""
    pytest.importorskip("langgraph.graph", reason="langgraph not installed; e2e skipped")
    from typing import TypedDict
    from langgraph.graph import StateGraph

    class S(TypedDict):
        x: int

    def add_one(s: S):
        return {"x": s["x"] + 1}

    builder = StateGraph(S)
    builder.add_node("n", add_one)
    builder.set_entry_point("n")
    builder.set_finish_point("n")
    graph = builder.compile(
        checkpointer=VantaDBCheckpointer(db_path=str(tmp_path / "e2e_db"))
    )
    cfg = {"configurable": {"thread_id": "t-e2e"}}
    assert graph.invoke({"x": 1}, cfg) == {"x": 2}
    assert graph.invoke({"x": 10}, cfg) == {"x": 11}
    assert graph.get_state(cfg).values == {"x": 11}
