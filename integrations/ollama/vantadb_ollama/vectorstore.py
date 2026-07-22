from __future__ import annotations

import uuid
from typing import Any, Iterable, List, Optional

import ollama
import vantadb_py as vanta

DEFAULT_NAMESPACE = "ollama"
DEFAULT_TOP_K = 4
DEFAULT_MODEL = "nomic-embed-text"


class VantaDBOllama:
    def __init__(
        self,
        model: str = DEFAULT_MODEL,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
    ):
        """Initialize a VantaDB store with Ollama embeddings.

        Args:
            model: Ollama embedding model name.
                Defaults to ``"nomic-embed-text"``.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"ollama"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
        """
        self.model = model
        self.namespace = namespace
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
        )

    def _embed(self, text: str) -> List[float]:
        if not text:
            raise ValueError("text must be a non-empty string")
        resp = ollama.embeddings(model=self.model, prompt=text)
        return list(resp["embedding"])

    def _embed_many(self, texts: List[str]) -> List[List[float]]:
        if not texts:
            return []
        resp = ollama.embed(model=self.model, input=texts)
        return [list(e) for e in resp["embeddings"]]

    def add_texts(
        self,
        texts: List[str],
        metadatas: Optional[List[dict]] = None,
        ids: Optional[List[str]] = None,
    ) -> List[str]:
        """Add texts with Ollama embeddings to the store.

        Embeds all texts in a single batch via the Ollama API, then
        stores each text with its vector and optional metadata.

        Args:
            texts: List of text strings to add.
            metadatas: Optional list of metadata dicts, one per text.
            ids: Optional list of IDs, one per text. UUIDs are
                generated for entries without an ID.

        Returns:
            A list of assigned IDs, one per input text.

        Raises:
            ValueError: If ``texts`` is empty, or if ``metadatas``
                or ``ids`` length does not match ``texts`` length.
        """
        if not texts:
            return []
        if metadatas is not None and len(metadatas) != len(texts):
            raise ValueError(
                f"metadatas length ({len(metadatas)}) must match texts length ({len(texts)})"
            )
        if ids is not None and len(ids) != len(texts):
            raise ValueError(
                f"ids length ({len(ids)}) must match texts length ({len(texts)})"
            )
        vectors = self._embed_many(texts)
        result_ids: List[str] = []
        for i, text in enumerate(texts):
            key = ids[i] if ids else str(uuid.uuid4())
            meta = metadatas[i] if metadatas else {}
            self._db.put(self.namespace, key, text, metadata=meta, vector=vectors[i])
            result_ids.append(key)
        return result_ids

    async def aadd_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[List[dict]] = None,
        **kwargs: Any,
    ) -> List[str]:
        """Async version of ``add_texts``.

        Delegates to the synchronous implementation. Provided for
        compatibility with async framework pipelines.

        Args:
            texts: Iterable of text strings to add.
            metadatas: Optional list of metadata dicts, one per text.
            **kwargs: Passed through to ``add_texts``.

        Returns:
            A list of assigned IDs, one per input text.
        """
        return self.add_texts(texts, metadatas, **kwargs)

    def similarity_search(self, query: str, k: int = DEFAULT_TOP_K) -> List[Any]:
        """Search for documents similar to the query text.

        Embeds the query via the Ollama API, then performs a vector
        similarity search in VantaDB. Returns lightweight document-like
        objects with ``page_content`` and ``metadata`` attributes.

        Args:
            query: The search query string. Must be non-empty.
            k: Number of results to return. Defaults to 4.

        Returns:
            A list of document-like objects, each with ``page_content``
            and ``metadata`` attributes.

        Raises:
            ValueError: If ``query`` is empty or ``k`` is not positive.
        """
        if not query:
            raise ValueError("query must be a non-empty string")
        if k <= 0:
            k = 4
        vector = self._embed(query)
        results = self._db.search_memory(self.namespace, vector, top_k=k, distance_metric="cosine")
        hits = []
        for hit in results:
            hits.append(type("Document", (), {
                "page_content": hit.payload,
                "metadata": dict(hit.metadata),
            })())
        return hits[:k]

    async def asimilarity_search(self, query: str, k: int = 4, **kwargs: Any) -> List[Any]:
        """Async version of ``similarity_search``.

        Delegates to the synchronous implementation. Provided for
        compatibility with async framework pipelines.

        Args:
            query: The search query string.
            k: Number of results to return. Defaults to 4.
            **kwargs: Passed through to ``similarity_search``.

        Returns:
            A list of document-like objects, each with ``page_content``
            and ``metadata`` attributes.
        """
        return self.similarity_search(query, k, **kwargs)

    def delete(self, ids: Optional[List[str]] = None) -> bool:
        """Delete documents by their IDs.

        Args:
            ids: Optional list of document IDs to delete. If
                ``None`` or empty, no-op and returns ``True``.

        Returns:
            ``True`` if the operation completed.
        """
        if ids is None:
            return True
        if not ids:
            return True
        for key in ids:
            self._db.delete_memory(self.namespace, key)
        return True

    async def adelete(self, ids: Optional[List[str]] = None, **kwargs: Any) -> Optional[bool]:
        """Async version of ``delete``.

        Delegates to the synchronous implementation. Provided for
        compatibility with async framework pipelines.

        Args:
            ids: Optional list of document IDs to delete.
            **kwargs: Passed through to ``delete``.

        Returns:
            ``True`` if the operation completed.
        """
        return self.delete(ids, **kwargs)
