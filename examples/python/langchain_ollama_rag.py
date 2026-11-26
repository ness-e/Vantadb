"""
VantaDB RAG demo: LangChain + Ollama + VantaDB.

End-to-end retrieval-augmented generation pipeline using the *real*
integration packages (not emulated modules):

    langchain_ollama.OllamaEmbeddings   -> embeddings (Ollama server)
    vantadb_langchain.VantaDBVectorStore -> vector storage + search (VantaDB)
    langchain_core.vectorstores.VectorStore -> the LangChain interface

Flow: ingest 4 documents with metadata -> semantic retrieval -> build an
LLM prompt from the top hits and print it together with the expected answer.

Usage with a real Ollama server
-------------------------------
1. Install and start Ollama, then pull the embedding model:
       ollama pull nomic-embed-text
2. Install the integration packages:
       pip install vantadb-langchain langchain-ollama
3. Run:
       python examples/python/langchain_ollama_rag.py

The script detects Ollama at startup (import + one connectivity probe).
If the `langchain_ollama` package is missing or no Ollama server is
reachable on localhost:11434, it degrades to deterministic hash-based
mock embeddings so the demo still runs end-to-end with exit code 0.

No network access is required in fallback mode.
"""

import hashlib
import shutil
import tempfile

DB_PATH = tempfile.mkdtemp(prefix="vantadb-rag-")
NAMESPACE = "langchain-ollama-demo"
MODEL = "nomic-embed-text"


class _MockEmbeddings:
    """Deterministic hash-based stand-in for Ollama embeddings (offline mode)."""

    DIM = 32

    def _vec(self, text: str) -> list[float]:
        digest = hashlib.sha256(text.encode("utf-8")).digest()
        return [
            ((digest[i % len(digest)] + i * 31) & 0xFF) / 128.0 - 1.0
            for i in range(self.DIM)
        ]

    def embed_query(self, text: str) -> list[float]:
        return self._vec(text)

    def embed_documents(self, texts: list[str]) -> list[list[float]]:
        return [self._vec(t) for t in texts]


def _make_embeddings():
    """Return (embeddings, mode). Real Ollama if reachable, else mock."""
    try:
        from langchain_ollama import OllamaEmbeddings

        embeddings = OllamaEmbeddings(model=MODEL)
        embeddings.embed_query("connectivity probe")  # raises if server is down
        return embeddings, f"ollama ({MODEL})"
    except Exception:
        return _MockEmbeddings(), "mock (Ollama not available - deterministic fallback)"


def main() -> None:
    from vantadb_langchain import VantaDBVectorStore

    embeddings, mode = _make_embeddings()
    print(f"[embedding] {mode}")
    print(f"[db] {DB_PATH} namespace={NAMESPACE}")

    store = VantaDBVectorStore(
        embeddings,
        db_path=DB_PATH,
        namespace=NAMESPACE,
        memory_limit_bytes=128_000_000,
    )

    # 1. Ingest documents with metadata
    documents = [
        (
            "VantaDB is a deeply embedded vector database written in Rust.",
            {"category": "architecture", "source": "README"},
        ),
        (
            "Using multiple databases (Vector, Graph, Relational) creates a glue-code nightmare.",
            {"category": "motivation", "source": "README"},
        ),
        (
            "By compiling the database via PyO3, Python apps query vectors with zero-copy overhead.",
            {"category": "architecture", "source": "docs"},
        ),
        (
            "Resource governance automatically shifts HNSW heaps to disk (MMAP) when RAM is low.",
            {"category": "governance", "source": "docs"},
        ),
    ]
    texts = [t for t, _ in documents]
    metadatas = [m for _, m in documents]

    print(f"\n[ingest] adding {len(documents)} documents...")
    ids = store.add_texts(texts, metadatas=metadatas)
    print(f"[ingest] stored ids: {[i[:8] for i in ids]}")

    # 2. Semantic retrieval (returns (Document, distance) pairs)
    query = "How does VantaDB avoid the glue-code problem in Python?"
    print(f"\n[retrieval] query: {query!r}")
    hits = store.similarity_search_with_score(query, k=2)

    context_chunks = []
    for i, (doc, score) in enumerate(hits, 1):
        meta = dict(doc.metadata)
        print(
            f"  [{i}] distance={score:.4f} category={meta.get('category')} : "
            f"{doc.page_content[:70]}..."
        )
        context_chunks.append(doc.page_content)

    # 3. Generation: build the RAG prompt and show the expected answer
    context = "\n".join(context_chunks)
    prompt = (
        "Answer the user's question using only the context below.\n\n"
        f"Context:\n{context}\n\nQuestion: {query}"
    )
    print("\n[generation] prompt sent to LLM:\n")
    print(prompt)

    expected = (
        "VantaDB avoids the glue-code problem by keeping all data models in a single "
        "embedded engine: vector, graph, and relational queries share one database, so "
        "no separate stores or glue code are needed."
    )
    print(f"\n[generation] expected answer:\n{expected}")

    # Cleanup (best-effort; temp dir is outside the repo)
    try:
        store._db.close()
    except Exception:
        pass
    shutil.rmtree(DB_PATH, ignore_errors=True)
    print(f"\n[done] demo completed, cleaned up {DB_PATH}")


if __name__ == "__main__":
    main()
