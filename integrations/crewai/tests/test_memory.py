"""Tests for VantaDB CrewAI unified-Memory storage backend (INTG-02)."""
import asyncio
import os
import sys
from datetime import datetime, timedelta, timezone
from types import SimpleNamespace

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_crewai.memory import (  # noqa: E402 — RED: module does not exist yet
    VantaDBMemoryBackend,
)


def make_record(content="hello world", scope="/project/alpha", **kw):
    now = datetime.now(timezone.utc)
    base = dict(
        id="rec-1",
        content=content,
        scope=scope,
        categories=["decisions"],
        metadata={"topic": "db"},
        importance=0.7,
        created_at=now,
        last_accessed=now,
        embedding=[1.0, 0.0, 0.0],
        source="user:alice",
        private=False,
    )
    base.update(kw)
    return SimpleNamespace(**base)


@pytest.fixture
def backend(tmp_path):
    path = str(tmp_path / "test_mem")
    return VantaDBMemoryBackend(db_path=path, namespace="crewai_memory")


def test_save_and_get_record_roundtrip(backend):
    rec = make_record()
    backend.save([rec])
    got = backend.get_record("rec-1")
    assert got is not None
    assert got.content == "hello world"
    assert got.scope == "/project/alpha"
    assert got.categories == ["decisions"]
    assert got.metadata == {"topic": "db"}
    assert got.importance == pytest.approx(0.7)
    assert got.source == "user:alice"
    assert got.private is False


def test_search_returns_scored_match(backend):
    backend.save([make_record()])
    hits = backend.search([1.0, 0.0, 0.0], limit=5)
    assert len(hits) == 1
    record, score = hits[0]
    assert record.content == "hello world"
    assert score > 0.5


def test_search_scope_prefix_filters(backend):
    backend.save([
        make_record(id="a", scope="/project/alpha"),
        make_record(id="b", content="other", scope="/project/beta"),
    ])
    hits = backend.search([1.0, 0.0, 0.0], scope_prefix="/project/alpha", limit=5)
    assert [r.id for r, _ in hits] == ["a"]


def test_search_excludes_records_without_embedding(backend):
    backend.save([make_record(embedding=None)])
    assert backend.search([1.0, 0.0, 0.0], limit=5) == []


def test_update_replaces_content(backend):
    backend.save([make_record()])
    backend.update(make_record(content="updated content"))
    assert backend.get_record("rec-1").content == "updated content"


def test_delete_by_record_ids_returns_count(backend):
    backend.save([
        make_record(id="a", scope="/s"),
        make_record(id="b", content="x", scope="/s"),
    ])
    assert backend.delete(record_ids=["a"]) == 1
    assert backend.get_record("a") is None
    assert backend.get_record("b") is not None


def test_list_records_newest_first_with_pagination(backend):
    t0 = datetime.now(timezone.utc)
    backend.save([
        make_record(id="old", content="old", scope="/s",
                    created_at=t0 - timedelta(days=2)),
        make_record(id="new", content="new", scope="/s", created_at=t0),
    ])
    page = backend.list_records(scope_prefix="/s", limit=1, offset=0)
    assert [r.id for r in page] == ["new"]
    page2 = backend.list_records(scope_prefix="/s", limit=1, offset=1)
    assert [r.id for r in page2] == ["old"]


def test_scope_info_categories_and_children(backend):
    backend.save([
        make_record(id="a", scope="/project/alpha", categories=["x"]),
        make_record(id="b", content="y", scope="/project/alpha/deep",
                    categories=["x", "z"]),
    ])
    info = backend.get_scope_info("/project/alpha")
    assert info.record_count == 2
    assert sorted(info.categories) == ["x", "z"]
    assert info.child_scopes == ["/project/alpha/deep"]


def test_list_scopes_categories_count_reset(backend):
    backend.save([
        make_record(id="a", scope="/project/alpha", categories=["x"]),
        make_record(id="b", content="y", scope="/other", categories=["x"]),
    ])
    assert backend.list_scopes("/") == ["/other", "/project"]
    assert backend.list_categories() == {"x": 2}
    assert backend.count() == 2
    assert backend.count(scope_prefix="/other") == 1
    backend.reset(scope_prefix="/other")
    assert backend.count() == 1
    backend.reset()
    assert backend.count() == 0


def test_delete_with_older_than_and_metadata_filter(backend):
    t0 = datetime.now(timezone.utc)
    backend.save([
        make_record(id="old", scope="/s", created_at=t0 - timedelta(days=60)),
        make_record(id="new", content="n", scope="/s", created_at=t0),
    ])
    assert backend.delete(older_than=t0 - timedelta(days=30)) == 1
    assert backend.delete(metadata_filter={"topic": "db"}) == 1
    assert backend.count() == 0


def test_async_variants(backend):
    async def go():
        await backend.asave([make_record()])
        hits = await backend.asearch([1.0, 0.0, 0.0], limit=5)
        assert len(hits) == 1
        assert await backend.adelete(record_ids=["rec-1"]) == 1
    asyncio.run(go())


def test_storage_backend_protocol_conformance(backend):
    crewai_backend = pytest.importorskip(
        "crewai.memory.storage.backend",
        reason="crewai no instalado; conformancia real diferida",
    )
    assert isinstance(backend, crewai_backend.StorageBackend)
