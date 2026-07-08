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
        store = cls(embedding=embedding, **kwargs)
        store.add_texts(texts, metadatas=metadatas, ids=ids)
        return store

    # ── Search methods ───────────────────────────────────────

    def similarity_search_with_score(
        self, query: str, k: int = DEFAULT_TOP_K, **kwargs: Any
    ) -> List[Tuple[Document, float]]:
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
        docs_with_scores = self.similarity_search_with_vector_score(
            embedding, k=k, **kwargs
        )
        return [doc for doc, _ in docs_with_scores]

    def similarity_search_with_vector_score(
        self, embedding: List[float], k: int = DEFAULT_TOP_K, **kwargs: Any
    ) -> List[Tuple[Document, float]]:
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
        texts_list = list(texts)
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
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        ids = [doc.id for doc in documents if doc.id] or None
        return self.add_texts(texts, metadatas=metadatas, ids=ids, **kwargs)

    def delete(
        self, ids: Optional[List[str]] = None, **kwargs: Any
    ) -> Optional[bool]:
        if ids is None:
            return True

        for key in ids:
            self._db.delete_memory(self.namespace, key)
        return True

    def delete_by_filter(self, filter_key: str, filter_val: Any) -> int:
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
