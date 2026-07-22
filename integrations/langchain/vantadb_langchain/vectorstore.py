from __future__ import annotations

import json
import math
import uuid
from typing import Any, Callable, Iterable, List, Optional, Sequence, Tuple

import vantadb_py as vanta
from langchain_core.documents import Document
from langchain_core.embeddings import Embeddings
from langchain_core.vectorstores import VectorStore

DEFAULT_NAMESPACE = "langchain"
DEFAULT_TOP_K = 4


class VantaDBVectorStore(VectorStore):
    def __init__(
        self,
        embedding: Embeddings,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        """Initialize a LangChain VectorStore backed by VantaDB.

        Args:
            embedding: A LangChain ``Embeddings`` instance used to
                compute vectors for all text operations.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"langchain"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        self.embedding = embedding
        self.namespace = namespace
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    @property
    def embeddings(self) -> Embeddings:
        """Return the ``Embeddings`` instance used by this store.

        Implements the LangChain ``VectorStore.embeddings`` property.

        Returns:
            The ``Embeddings`` object passed at initialisation.
        """
        return self.embedding

    @staticmethod
    def _hit_to_dict(hit: vanta.VantaSearchHit) -> dict:
        return {
            "key": hit.key,
            "node_id": hit.id,
            "payload": hit.payload,
            "metadata": dict(hit.metadata),
            "created_at_ms": hit.created_at_ms,
            "updated_at_ms": hit.updated_at_ms,
            "version": hit.version,
        }

    def _to_document(self, record: dict) -> Document:
        metadata = dict(record.get("metadata", {}))
        metadata["_key"] = record.get("key", "")
        metadata["_node_id"] = record.get("node_id", 0)
        metadata["_created_at_ms"] = record.get("created_at_ms", 0)
        metadata["_updated_at_ms"] = record.get("updated_at_ms", 0)
        metadata["_version"] = record.get("version", 0)
        payload = record.get("payload", "")
        return Document(page_content=payload, metadata=metadata)

    def _build_key(self, text: str, index: int) -> str:
        return str(uuid.uuid5(uuid.NAMESPACE_DNS, f"{text}:{index}"))

    # ── Required abstract methods ────────────────────────────

    def similarity_search(
        self, query: str, k: int = DEFAULT_TOP_K, **kwargs: Any
    ) -> List[Document]:
        """Search for documents similar to the query text.

        Implements the LangChain ``VectorStore.similarity_search``
        abstract method. Embeds the query and delegates to
        ``similarity_search_by_vector``.

        Args:
            query: The search query string.
            k: Number of results to return. Defaults to 4.
            **kwargs: Passed through to
                ``similarity_search_by_vector``.

        Returns:
            A list of ``Document`` objects ranked by relevance.
        """
        if not query:
            raise ValueError("query must be a non-empty string")
        if k <= 0:
            raise ValueError(f"k must be positive, got {k}")
        if self.embedding is None:
            raise ValueError("embedding function is not set")
        embedding_vector = self.embedding.embed_query(query)
        return self.similarity_search_by_vector(embedding_vector, k=k, **kwargs)

    @classmethod
    def from_texts(
        cls,
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[dict]] = None,
        *,
        ids: Optional[List[str]] = None,
        **kwargs: Any,
    ) -> VantaDBVectorStore:
        """Create a VantaDBVectorStore from a list of texts.

        Implements the LangChain ``VectorStore.from_texts`` class
        method. Creates the store, adds the texts, and returns it.

        Args:
            texts: List of text strings to add.
            embedding: A LangChain ``Embeddings`` instance.
            metadatas: Optional list of metadata dicts, one per text.
            ids: Optional list of IDs, one per text.
            **kwargs: Additional keyword arguments forwarded to the
                ``VantaDBVectorStore`` constructor.

        Returns:
            A new ``VantaDBVectorStore`` instance containing the
            provided texts.
        """
        store = cls(embedding=embedding, **kwargs)
        store.add_texts(texts, metadatas=metadatas, ids=ids)
        return store

    # ── MMR search ──────────────────────────────────────────

    def max_marginal_relevance_search(
        self,
        query: str,
        k: int = 4,
        fetch_k: int = 20,
        lambda_mult: float = 0.5,
        **kwargs: Any,
    ) -> List[Document]:
        """MMR search — balance relevance and diversity.

        Args:
            query: Search query string.
            k: Number of results to return.
            fetch_k: Number of results to fetch initially.
            lambda_mult: 0 = only diversity, 1 = only relevance.
        """
        if not query:
            raise ValueError("query must be a non-empty string")
        if k <= 0:
            raise ValueError(f"k must be positive, got {k}")
        if self.embedding is None:
            raise ValueError("embedding function is not set")
        embedded_query = self.embedding.embed_query(query)
        return self.max_marginal_relevance_search_by_vector(
            embedded_query, k=k, fetch_k=fetch_k, lambda_mult=lambda_mult, **kwargs
        )

    def max_marginal_relevance_search_by_vector(
        self,
        embedding: List[float],
        k: int = 4,
        fetch_k: int = 20,
        lambda_mult: float = 0.5,
        **kwargs: Any,
    ) -> List[Document]:
        """MMR by embedding vector."""
        if not embedding:
            raise ValueError("embedding vector must be a non-empty list")
        if k <= 0:
            raise ValueError(f"k must be positive, got {k}")
        # 1. Fetch fetch_k candidates
        docs_with_scores = self.similarity_search_with_vector_score(
            embedding, k=fetch_k, **kwargs
        )
        if not docs_with_scores:
            return []

        # 2. Load embeddings for each candidate (needed for pairwise diversity)
        cand_embs: List[List[float]] = []
        for doc, _score in docs_with_scores:
            key = doc.metadata.get("_key", "")
            rec = self._db.get_memory(self.namespace, key) if key else None
            vec: List[float] = []
            if rec is not None:
                try:
                    v = rec.vector
                    vec = list(v) if v is not None else []
                except (ValueError, TypeError, RuntimeError):
                    vec = []
            if not vec:
                vec = embedding  # diversity-neutral fallback
            cand_embs.append(vec)

        # 3. Relevance scores (cosine → [0,1])
        relevance = [1.0 - s / 2.0 for _, s in docs_with_scores]

        # 4. Greedy MMR selection
        selected: List[int] = []
        candidates = list(range(len(docs_with_scores)))

        while len(selected) < k and candidates:
            best_idx: int = -1
            best_score: float = -1.0
            for i in candidates:
                mmr = lambda_mult * relevance[i]
                if selected:
                    max_sim = max(
                        self._cosine_sim(cand_embs[i], cand_embs[j])
                        for j in selected
                    )
                    mmr -= (1.0 - lambda_mult) * max_sim
                if mmr > best_score:
                    best_score = mmr
                    best_idx = i
            if best_idx < 0:
                break
            selected.append(best_idx)
            candidates.remove(best_idx)

        return [docs_with_scores[i][0] for i in selected]

    @staticmethod
    def _cosine_sim(a: List[float], b: List[float]) -> float:
        dot = sum(x * y for x, y in zip(a, b))
        na = math.sqrt(sum(x * x for x in a))
        nb = math.sqrt(sum(x * x for x in b))
        if na == 0 or nb == 0:
            return 0.0
        return dot / (na * nb)

    # ── Search methods ───────────────────────────────────────

    def similarity_search_with_score(
        self, query: str, k: int = DEFAULT_TOP_K, **kwargs: Any
    ) -> List[Tuple[Document, float]]:
        """Search for documents and return them with relevance scores.

        Args:
            query: The search query string.
            k: Number of results to return. Defaults to 4.
            **kwargs: Supports ``filter_key`` and ``filter_val`` for
                metadata filtering, and ``text_query`` for hybrid
                text-and-vector search.

        Returns:
            A list of ``(Document, score)`` tuples where score is a
            distance value from VantaDB (lower = more similar).
        """
        if not query:
            raise ValueError("query must be a non-empty string")
        if k <= 0:
            raise ValueError(f"k must be positive, got {k}")
        if self.embedding is None:
            raise ValueError("embedding function is not set")
        embedding_vector = self.embedding.embed_query(query)
        filter_key = kwargs.get("filter_key")
        filter_val = kwargs.get("filter_val")
        text_query = kwargs.get("text_query")

        filters = {filter_key: filter_val} if filter_key is not None and filter_val is not None else None

        if text_query:
            results = self._db.search_memory(
                self.namespace,
                embedding_vector,
                top_k=k,
                text_query=text_query,
                distance_metric="cosine",
                filters=filters,
            )
        else:
            results = self._db.search_memory(
                self.namespace,
                embedding_vector,
                top_k=k,
                distance_metric="cosine",
                filters=filters,
            )

        docs_with_scores: List[Tuple[Document, float]] = []
        for hit in results:
            doc = self._to_document(self._hit_to_dict(hit))
            docs_with_scores.append((doc, hit.score))
        return docs_with_scores

    def similarity_search_by_vector(
        self, embedding: List[float], k: int = DEFAULT_TOP_K, **kwargs: Any
    ) -> List[Document]:
        """Search for documents by embedding vector.

        Implements the LangChain ``VectorStore.similarity_search_by_vector``
        abstract method.

        Args:
            embedding: The query embedding vector.
            k: Number of results to return. Defaults to 4.
            **kwargs: Passed through to
                ``similarity_search_with_vector_score``.

        Returns:
            A list of ``Document`` objects ranked by relevance.
        """
        if not embedding:
            raise ValueError("embedding vector must be a non-empty list")
        if k <= 0:
            raise ValueError(f"k must be positive, got {k}")
        docs_with_scores = self.similarity_search_with_vector_score(
            embedding, k=k, **kwargs
        )
        return [doc for doc, _ in docs_with_scores]

    def similarity_search_with_vector_score(
        self, embedding: List[float], k: int = DEFAULT_TOP_K, **kwargs: Any
    ) -> List[Tuple[Document, float]]:
        """Search for documents by embedding vector and return scores.

        Args:
            embedding: The query embedding vector.
            k: Number of results to return. Defaults to 4.
            **kwargs: Supports ``filter_key`` and ``filter_val`` for
                metadata filtering, and ``text_query`` for hybrid
                text-and-vector search.

        Returns:
            A list of ``(Document, score)`` tuples where score is a
            distance value from VantaDB (lower = more similar).
        """
        if not embedding:
            raise ValueError("embedding vector must be a non-empty list")
        if k <= 0:
            raise ValueError(f"k must be positive, got {k}")
        text_query = kwargs.get("text_query")
        filter_key = kwargs.get("filter_key")
        filter_val = kwargs.get("filter_val")

        filters = {filter_key: filter_val} if filter_key is not None and filter_val is not None else None

        if text_query:
            results = self._db.search_memory(
                self.namespace,
                embedding,
                top_k=k,
                text_query=text_query,
                distance_metric="cosine",
                filters=filters,
            )
        else:
            results = self._db.search_memory(
                self.namespace,
                embedding,
                top_k=k,
                distance_metric="cosine",
                filters=filters,
            )

        docs_with_scores: List[Tuple[Document, float]] = []
        for hit in results:
            doc = self._to_document(self._hit_to_dict(hit))
            docs_with_scores.append((doc, hit.score))
        return docs_with_scores

    # ── Write methods ────────────────────────────────────────

    def add_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[List[dict]] = None,
        *,
        ids: Optional[List[str]] = None,
        **kwargs: Any,
    ) -> List[str]:
        """Add texts to the store with embeddings.

        Implements the LangChain ``VectorStore.add_texts`` abstract
        method. Embeds all texts in a single batch, then stores each
        one with its vector and optional metadata.

        Args:
            texts: Iterable of text strings to add.
            metadatas: Optional list of metadata dicts, one per text.
                Must match ``texts`` length if provided.
            ids: Optional list of IDs, one per text. Generated
                deterministically from content if omitted.
            **kwargs: Ignored; for compatibility with LangChain
                call patterns.

        Returns:
            A list of assigned IDs, one per input text.

        Raises:
            ValueError: If ``metadatas`` or ``ids`` length does not
                match ``texts`` length.
        """
        texts_list = list(texts)
        if not texts_list:
            return []
        if self.embedding is None:
            raise ValueError("embedding function is not set")
        if metadatas and len(metadatas) != len(texts_list):
            raise ValueError(
                f"metadatas length ({len(metadatas)}) must match texts length ({len(texts_list)})"
            )

        if ids and len(ids) != len(texts_list):
            raise ValueError(
                f"ids length ({len(ids)}) must match texts length ({len(texts_list)})"
            )

        embeddings = self.embedding.embed_documents(texts_list)
        result_ids: List[str] = []

        for i, text in enumerate(texts_list):
            key = ids[i] if ids else self._build_key(text, i)
            metadata = metadatas[i] if metadatas else {}
            vector = embeddings[i]
            self._db.put(
                self.namespace,
                key,
                text,
                metadata=metadata,
                vector=vector,
            )
            result_ids.append(key)

        return result_ids

    def add_documents(
        self, documents: List[Document], **kwargs: Any
    ) -> List[str]:
        """Add LangChain ``Document`` objects to the store.

        Implements the LangChain ``VectorStore.add_documents`` method.
        Extracts text, metadata, and optional IDs from each document
        and delegates to ``add_texts``.

        Args:
            documents: List of ``Document`` objects to add.
            **kwargs: Passed through to ``add_texts``.

        Returns:
            A list of assigned IDs, one per document.
        """
        if not documents:
            raise ValueError("documents must be a non-empty list")
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        ids = [doc.id for doc in documents if doc.id is not None] or None
        return self.add_texts(texts, metadatas=metadatas, ids=ids, **kwargs)

    def delete(
        self, ids: Optional[List[str]] = None, **kwargs: Any
    ) -> Optional[bool]:
        """Delete documents by their IDs.

        Implements the LangChain ``VectorStore.delete`` method.

        Args:
            ids: Optional list of document IDs to delete. If
                ``None``, no-op and returns ``True``.
            **kwargs: Ignored; for compatibility.

        Returns:
            ``True`` if the operation completed (even if no IDs
            were provided).
        """
        if ids is None:
            return True
        if not ids:
            return True

        for key in ids:
            self._db.delete_memory(self.namespace, key)
        return True

    def delete_by_filter(self, filter_key: str, filter_val: Any) -> int:
        """Delete all documents matching a metadata filter.

        Args:
            filter_key: The metadata field name to match.
            filter_val: The value that the field must equal.

        Returns:
            The number of deleted documents.
        """
        if not filter_key:
            raise ValueError("filter_key must be a non-empty string")
        page = self._db.list_memory(self.namespace, filters={filter_key: filter_val}, limit=10000)
        count = 0
        for rec in page.records:
            key = rec.key
            if key:
                self._db.delete_memory(self.namespace, key)
                count += 1
        return count

    @staticmethod
    def _record_to_dict(record: vanta.VantaMemoryRecord) -> dict:
        return {
            "key": record.key,
            "payload": record.payload,
            "metadata": dict(record.metadata),
            "created_at_ms": record.created_at_ms,
            "updated_at_ms": record.updated_at_ms,
            "version": record.version,
            "node_id": record.node_id,
        }

    def get_by_ids(self, ids: Sequence[str], /) -> List[Document]:
        """Retrieve documents by their IDs.

        Args:
            ids: A sequence of document IDs to look up.

        Returns:
            A list of ``Document`` objects. Only IDs that exist
            in the store are included.
        """
        if not ids:
            return []
        documents: List[Document] = []
        for key in ids:
            record = self._db.get_memory(self.namespace, key)
            if record:
                documents.append(self._to_document(self._record_to_dict(record)))
        return documents

    # ── Relevance score normalization ────────────────────────

    def _select_relevance_score_fn(self) -> Callable[[float], float]:
        return self._cosine_relevance_score_fn

    @staticmethod
    def _cosine_relevance_score_fn(distance: float) -> float:
        return 1.0 - distance / 2.0
