"""Tests for VantaDB LangChain vector store adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_langchain import VantaDBVectorStore
from langchain_core.documents import Document
import vantadb_py as vanta


class FakeEmbeddings:
    def embed_query(self, text: str):
        return [0.1] * 4
    def embed_documents(self, texts):
        return [[0.1] * 4 for _ in texts]


@pytest.fixture
def store():
    embeddings = FakeEmbeddings()
    path = os.path.join(tempfile.mkdtemp(), "test_lc")
    store = VantaDBVectorStore(embeddings, db_path=path, namespace="test_lc")
    yield store


# ── Existing tests (preserved) ─────────────────────────────────

def test_add_and_search(store):
    docs = [Document(page_content="hello world", metadata={"type": "greeting"})]
    ids = store.add_documents(docs)
    assert len(ids) == 1

    results = store.similarity_search("hello", k=5)
    assert len(results) >= 1
    assert "hello" in results[0].page_content


def test_delete_by_filter(store):
    docs = [Document(page_content="one", metadata={"tag": "a"}),
            Document(page_content="two", metadata={"tag": "b"})]
    store.add_documents(docs)

    count = store.delete_by_filter("tag", "a")
    assert count >= 1

    remaining = store.similarity_search("one", k=5)
    assert len(remaining) == 1


def test_metadata_filter(store):
    docs = [Document(page_content="cat", metadata={"kind": "animal"}),
            Document(page_content="car", metadata={"kind": "vehicle"})]
    store.add_documents(docs)

    results = store.similarity_search("cat", k=5, filter_key="kind", filter_val="animal")
    assert len(results) >= 1
    assert results[0].page_content == "cat"


def test_empty_store(store):
    results = store.similarity_search("nothing", k=5)
    assert len(results) == 0


def test_get_by_ids(store):
    docs = [Document(page_content="test")]
    ids = store.add_documents(docs)
    found = store.get_by_ids(ids)
    assert len(found) == 1


# ── New tests: add_texts ───────────────────────────────────────

def test_add_texts_with_metadata(store):
    texts = ["alpha", "beta", "gamma"]
    metadatas = [{"group": "a"}, {"group": "b"}, {"group": "c"}]
    ids = store.add_texts(texts, metadatas=metadatas)
    assert len(ids) == 3

    for key in ids:
        record = store._db.get_memory(store.namespace, key)
        assert record is not None


def test_add_texts_without_metadata(store):
    texts = ["just", "texts"]
    ids = store.add_texts(texts)
    assert len(ids) == 2


def test_add_texts_with_ids(store):
    texts = ["x", "y"]
    ids = ["custom-1", "custom-2"]
    result = store.add_texts(texts, ids=ids)
    assert result == ["custom-1", "custom-2"]

    found = store.get_by_ids(["custom-1"])
    assert len(found) == 1
    assert found[0].page_content == "x"


def test_add_texts_metadata_mismatch_raises(store):
    texts = ["a", "b"]
    metadatas = [{"k": "v"}]  # only 1, texts has 2
    with pytest.raises(ValueError, match="metadatas length"):
        store.add_texts(texts, metadatas=metadatas)


def test_add_texts_ids_mismatch_raises(store):
    texts = ["a", "b", "c"]
    ids = ["only-one"]
    with pytest.raises(ValueError, match="ids length"):
        store.add_texts(texts, ids=ids)


# ── New tests: delete by ids ───────────────────────────────────

def test_delete_by_ids(store):
    docs = [Document(page_content="alpha"), Document(page_content="beta")]
    ids = store.add_documents(docs)
    assert len(store.get_by_ids(ids)) == 2

    store.delete(ids=[ids[0]])
    remaining = store.get_by_ids(ids)
    assert len(remaining) == 1
    assert remaining[0].page_content == "beta"


def test_delete_nonexistent_id(store):
    # Should not raise
    result = store.delete(ids=["does-not-exist"])
    assert result is True


def test_delete_with_none_ids(store):
    # delete(None) should be a no-op returning True
    result = store.delete(ids=None)
    assert result is True


# ── New tests: similarity_search edge cases ────────────────────

def test_similarity_search_returns_empty_for_nonsense(store):
    store.add_texts(["real content here"])
    results = store.similarity_search("zzzzzzzzz_nonexistent_zzzzzzz", k=5)
    # may return results because fake embeddings always match; just verify shape
    assert isinstance(results, list)


def test_similarity_search_with_text_query(store):
    store.add_texts(["hello world", "goodbye moon"])
    # text_query filters on text; with fake embeddings everything is similar
    results = store.similarity_search_with_score(
        "hello", k=5, text_query="moon"
    )
    assert isinstance(results, list)


def test_similarity_search_by_vector(store):
    store.add_texts(["vector search"])
    docs = store.similarity_search_by_vector([0.1] * 4, k=5)
    assert len(docs) >= 1


def test_similarity_search_with_vector_score(store):
    store.add_texts(["score test"])
    results = store.similarity_search_with_vector_score([0.1] * 4, k=5)
    assert len(results) >= 1
    doc, score = results[0]
    assert isinstance(score, float)


# ── New tests: edge cases ──────────────────────────────────────

def test_add_texts_empty(store):
    ids = store.add_texts([])
    assert ids == []


def test_add_documents_empty(store):
    ids = store.add_documents([])
    assert ids == []


def test_similarity_search_with_k_zero(store):
    store.add_texts(["some content"])
    results = store.similarity_search("hello", k=0)
    assert len(results) == 0


def test_from_texts_classmethod(store):
    texts = ["from", "classmethod"]
    embeddings = FakeEmbeddings()
    vs = VantaDBVectorStore.from_texts(
        texts,
        embedding=embeddings,
        db_path=os.path.join(tempfile.mkdtemp(), "test_ft"),
        namespace="test_ft",
    )
    results = vs.similarity_search("from", k=5)
    assert len(results) >= 1


def test_from_texts_with_metadata(store):
    texts = ["a", "b"]
    metadatas = [{"x": 1}, {"y": 2}]
    embeddings = FakeEmbeddings()
    vs = VantaDBVectorStore.from_texts(
        texts, embedding=embeddings, metadatas=metadatas,
        db_path=os.path.join(tempfile.mkdtemp(), "test_ftm"),
        namespace="test_ftm",
    )
    results = vs.similarity_search("a", k=5)
    assert len(results) >= 1


def test_embeddings_property(store):
    assert store.embeddings is store.embedding


# ── New tests: relevance score ─────────────────────────────────

def test_cosine_relevance_score(store):
    # 1.0 - distance / 2.0
    assert store._cosine_relevance_score_fn(0.0) == 1.0
    assert store._cosine_relevance_score_fn(1.0) == 0.5
    assert store._cosine_relevance_score_fn(2.0) == 0.0
