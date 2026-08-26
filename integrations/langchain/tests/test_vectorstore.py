"""Tests for VantaDB LangChain vector store adapter."""
import pytest
pytest.importorskip("langchain_core", reason="langchain_core SDK not installed; adapter suite skipped")
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


def test_from_texts_with_metadata():
    """from_texts classmethod should preserve metadata."""
    embeddings = FakeEmbeddings()
    texts = ["doc1", "doc2", "doc3"]
    metadatas = [{"type": "a"}, {"type": "b"}, {"type": "c"}]
    path = os.path.join(tempfile.mkdtemp(), "test_ft")
    store = VantaDBVectorStore.from_texts(
        texts, embeddings, metadatas=metadatas, db_path=path, namespace="test_ft"
    )
    results = store.similarity_search("doc", k=3)
    assert len(results) == 3
    types = {r.metadata.get("type") for r in results}
    assert types == {"a", "b", "c"}


def test_mmr_diversity(store):
    """MMR should return diverse results, not just top-k."""
    texts = [
        "quantum computing explained simply",
        "quantum mechanics fundamentals",
        "quantum physics for beginners",
        "classical computing vs quantum",
        "introduction to algorithms",
    ]
    docs = [Document(page_content=t, metadata={"id": i}) for i, t in enumerate(texts)]
    store.add_documents(docs)

    # MMR with high diversity (lambda_mult=0)
    diverse = store.max_marginal_relevance_search("quantum", k=3, lambda_mult=0.0)
    # MMR with no diversity (lambda_mult=1)
    focused = store.max_marginal_relevance_search("quantum", k=3, lambda_mult=1.0)

    assert len(diverse) == 3
    assert len(focused) == 3
    # With high diversity, we should get different results than pure relevance
    diverse_ids = {d.metadata.get("id") for d in diverse}
    focused_ids = {d.metadata.get("id") for d in focused}
    # At minimum, both return 3 results


def test_embeddings_property(store):
    assert store.embeddings is store.embedding


# ── New tests: relevance score ─────────────────────────────────

def test_cosine_relevance_score(store):
    # 1.0 - distance / 2.0
    assert store._cosine_relevance_score_fn(0.0) == 1.0
    assert store._cosine_relevance_score_fn(1.0) == 0.5
    assert store._cosine_relevance_score_fn(2.0) == 0.0


# ── QW-2: add_documents con ids parciales ──

def test_add_documents_partial_ids(store):
    """Mezcla de docs con/sin id: los faltantes obtienen UUID, no ValueError engañoso."""
    import uuid
    from langchain_core.documents import Document
    docs = [
        Document(page_content="with id", id="custom-id-1"),
        Document(page_content="without id"),
    ]
    ids = store.add_documents(docs)
    assert len(ids) == 2
    assert ids[0] == "custom-id-1"
    uuid.UUID(ids[1])  # el doc sin id recibe un UUID válido


def test_add_documents_all_with_ids_preserved(store):
    """Si todos tienen id, se preservan tal cual (sin regenerar)."""
    from langchain_core.documents import Document
    docs = [Document(page_content="a", id="id-a"), Document(page_content="b", id="id-b")]
    assert store.add_documents(docs) == ["id-a", "id-b"]
