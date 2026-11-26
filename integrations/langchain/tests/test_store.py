"""Tests for VantaDB LangGraph BaseStore adapter (INTG-01)."""
import pytest
pytest.importorskip("langgraph.store.base", reason="langgraph-checkpoint not installed; store suite skipped")
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from langgraph.store.base import GetOp, Item, PutOp

from vantadb_langchain import VantaDBStore


class FakeEmbeddings:
    def embed_query(self, text: str):
        return [0.1] * 4

    def embed_documents(self, texts):
        return [[0.1] * 4 for _ in texts]


@pytest.fixture
def store(tmp_path):
    return VantaDBStore(db_path=str(tmp_path / "store_db"), embeddings=FakeEmbeddings())


@pytest.mark.asyncio
async def test_put_and_get(store):
    await store.aput(("users", "mem"), "k1", {"val": 1})
    item = await store.aget(("users", "mem"), "k1")
    assert isinstance(item, Item)
    assert item.key == "k1"
    assert tuple(item.namespace) == ("users", "mem")
    assert item.value == {"val": 1}


@pytest.mark.asyncio
async def test_get_missing_returns_none(store):
    assert await store.aget(("users", "mem"), "nope") is None


@pytest.mark.asyncio
async def test_delete(store):
    await store.aput(("users", "mem"), "k1", {"val": 1})
    await store.adelete(("users", "mem"), "k1")
    assert await store.aget(("users", "mem"), "k1") is None


@pytest.mark.asyncio
async def test_search_prefix(store):
    await store.aput(("u", "a"), "m1", {"text": "likes pizza"})
    await store.aput(("u", "b"), "m2", {"text": "likes pasta"})
    both = await store.asearch(("u",))
    assert {r.key for r in both} == {"m1", "m2"}
    only_a = await store.asearch(("u", "a"))
    assert [r.key for r in only_a] == ["m1"]


@pytest.mark.asyncio
async def test_search_filter(store):
    await store.aput(("docs",), "r1", {"kind": "animal", "name": "cat"})
    await store.aput(("docs",), "r2", {"kind": "vehicle", "name": "car"})
    results = await store.asearch(("docs",), filter={"kind": "animal"})
    assert [r.key for r in results] == ["r1"]
    assert results[0].value["name"] == "cat"


@pytest.mark.asyncio
async def test_search_limit_offset(store):
    for i in range(3):
        await store.aput(("docs",), f"r{i}", {"i": i})
    page1 = await store.asearch(("docs",), limit=2)
    page2 = await store.asearch(("docs",), limit=2, offset=2)
    assert len(page1) == 2
    assert len(page2) == 1


@pytest.mark.asyncio
async def test_search_query_semantic(store):
    await store.aput(("docs",), "r1", {"text": "the cat sits"})
    results = await store.asearch(("docs",), query="cat")
    assert len(results) >= 1
    assert results[0].score is not None


@pytest.mark.asyncio
async def test_query_without_embeddings_raises(tmp_path):
    bare = VantaDBStore(db_path=str(tmp_path / "bare_db"))
    await bare.aput(("docs",), "r1", {"text": "hello"})
    with pytest.raises(ValueError, match="embeddings"):
        await bare.asearch(("docs",), query="hello")


@pytest.mark.asyncio
async def test_invalid_namespace_raises(store):
    with pytest.raises(Exception, match="namespace"):
        await store.aput(("bad/ns",), "k1", {"v": 1})


@pytest.mark.asyncio
async def test_list_namespaces(store):
    await store.aput(("u", "a"), "m1", {"v": 1})
    await store.aput(("u", "b"), "m2", {"v": 2})
    nss = await store.alist_namespaces(prefix=("u",))
    assert ("u", "a") in nss
    assert ("u", "b") in nss


@pytest.mark.asyncio
async def test_batch_mixed_ops(store):
    results = await store.abatch(
        [PutOp(("b",), "k1", {"v": 7}), GetOp(("b",), "k1")]
    )
    assert results[1] is not None
    assert results[1].value == {"v": 7}


def test_sync_put_get_delete(store):
    store.put(("s",), "k1", {"v": 42})
    item = store.get(("s",), "k1")
    assert item is not None and item.value == {"v": 42}
    store.delete(("s",), "k1")
    assert store.get(("s",), "k1") is None
