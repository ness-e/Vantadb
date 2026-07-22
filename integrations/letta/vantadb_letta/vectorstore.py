from __future__ import annotations

import uuid
from typing import Any, Callable, Dict, List, Optional

import vantadb_py as vanta

DEFAULT_NAMESPACE = "letta"
DEFAULT_TOP_K = 4


class VantaDBVectorStore:
    def __init__(
        self,
        embedding: Optional[Callable[[str], List[float]]] = None,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        """Initialize a VantaDB vector store for Letta.

        Args:
            embedding: Optional callable that converts text to an
                embedding vector list. Used for semantic search.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"letta"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        self.embedding = embedding
        self.namespace = namespace
        self.path = db_path
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    def insert(
        self,
        text: str,
        source: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> str:
        """Insert a text entry into the store.

        Generates a UUID key automatically. If an embedding function
        is configured, the vector is computed and stored alongside
        the text.

        Args:
            text: The text content to insert. Must be non-empty.
            source: Optional source identifier; stored in metadata
                under the ``"source"`` key.
            metadata: Optional dictionary of additional metadata.

        Returns:
            The generated key (UUID string) for the inserted entry.

        Raises:
            ValueError: If ``text`` is empty or whitespace-only.
        """
        if not text or not text.strip():
            raise ValueError("text must be a non-empty string")
        key = str(uuid.uuid4())
        meta = dict(metadata or {})
        if source:
            meta["source"] = source
        kwargs: Dict[str, Any] = {"metadata": meta}
        if self.embedding is not None:
            kwargs["vector"] = self.embedding(text)
        self._db.put(self.namespace, key, text, **kwargs)
        return key

    def search(self, query: str, k: int = DEFAULT_TOP_K) -> List[dict]:
        """Search the store by vector similarity or listing.

        When an embedding function is configured, performs a vector
        similarity search; otherwise returns the first ``k`` records.

        Args:
            query: The search query string (used for embedding if
                available).
            k: Number of results to return. Must be positive.
                Defaults to 4.

        Returns:
            A list of dictionaries with ``"key"``, ``"text"``,
            ``"metadata"``, and ``"score"`` keys.

        Raises:
            ValueError: If ``k`` is not positive.
        """
        if k <= 0:
            raise ValueError("k must be > 0")
        if self.embedding is not None:
            vector = self.embedding(query)
            results = self._db.search_memory(self.namespace, vector, top_k=k, distance_metric="cosine")
            return [
                {"key": hit.key, "text": hit.payload, "metadata": dict(hit.metadata), "score": hit.score}
                for hit in results
            ]
        # fallback: list all
        results = self._db.list_memory(self.namespace, limit=k)
        return [
            {"key": r.key, "text": r.payload, "metadata": dict(r.metadata), "score": 1.0}
            for r in results.records
        ]

    def delete(self, key: str) -> bool:
        """Delete a single record by its key.

        Args:
            key: The key of the record to delete.

        Returns:
            ``True`` if the deletion succeeded, ``False`` otherwise.
        """
        return self._db.delete_memory(self.namespace, key)

    def list(self, limit: int = 100, filters: Optional[Dict[str, Any]] = None) -> List[dict]:
        """List records in the store, with optional metadata filtering.

        Args:
            limit: Maximum number of records to return.
                Defaults to 100. Must be positive.
            filters: Optional dictionary of metadata field-value pairs
                to filter by.

        Returns:
            A list of dictionaries with ``"key"``, ``"text"``, and
            ``"metadata"`` keys.

        Raises:
            ValueError: If ``limit`` is not positive.
        """
        if limit <= 0:
            raise ValueError("limit must be positive")
        kwargs: Dict[str, Any] = {"namespace": self.namespace, "limit": limit}
        if filters:
            kwargs["filters"] = filters
        results = self._db.list_memory(**kwargs)
        return [
            {"key": r.key, "text": r.payload, "metadata": dict(r.metadata)}
            for r in results.records
        ]

    def to_dict(self) -> Dict[str, Any]:
        """Serialize the store configuration to a dictionary.

        Returns:
            A dictionary with ``"path"`` and ``"namespace"`` keys
            for round-tripping via ``from_dict()``.
        """
        return {
            "path": self.path,
            "namespace": self.namespace,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> VantaDBVectorStore:
        """Deserialize store configuration from a dictionary.

        Args:
            data: Dictionary with ``"path"`` and optionally
                ``"namespace"`` keys, as produced by ``to_dict()``.

        Returns:
            A new ``VantaDBVectorStore`` instance.
        """
        return cls(
            db_path=data["path"],
            namespace=data.get("namespace", DEFAULT_NAMESPACE),
        )
