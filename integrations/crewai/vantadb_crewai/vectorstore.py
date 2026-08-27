from __future__ import annotations

from typing import Any, Callable, List, Optional

import uuid

import vantadb_py as vanta

try:
    from crewai.tools import BaseTool as CrewAIBaseTool
except ImportError:
    CrewAIBaseTool = object  # fallback si crewai no está instalado

try:
    from pydantic import PrivateAttr
except ImportError:
    PrivateAttr = None  # fallback si pydantic no está como dependencia directa

DEFAULT_NAMESPACE = "crewai"
DEFAULT_TOP_K = 4


class VantaDBTool(CrewAIBaseTool):
    # Declared as Pydantic fields so normal assignment works with extra='forbid'.
    namespace: str = DEFAULT_NAMESPACE
    embedding: Optional[Any] = None
    db_path: str = "./vantadb_data"
    top_k: int = DEFAULT_TOP_K
    if PrivateAttr is not None:
        _db: Any = PrivateAttr()

    def __init__(
        self,
        embedding: Optional[Callable[[str], List[float]]] = None,
        name: str = "VantaDB Search",
        description: str = "Search documents stored in VantaDB",
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        top_k: int = DEFAULT_TOP_K,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        """Initialize a VantaDB-powered CrewAI tool.

        Args:
            embedding: Optional callable that converts text to a vector
                embedding list. Used for semantic search.
            name: Name for the CrewAI tool. Defaults to "VantaDB Search".
            description: Description for the CrewAI tool. Defaults to
                "Search documents stored in VantaDB".
            db_path: Filesystem path for the VantaDB database.
                Defaults to "./vantadb_data".
            namespace: VantaDB namespace to operate on.
                Defaults to "crewai".
            top_k: Default number of results to return. Defaults to 4.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        super().__init__(name=name, description=description)
        self.namespace = namespace
        self.embedding = embedding
        self.db_path = db_path
        self.top_k = top_k
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    def _run(self, query: str, **kwargs: Any) -> str:
        """Execute a search query against the VantaDB store.

        Implements the CrewAI ``BaseTool._run`` protocol. When an
        embedding function is configured, performs vector similarity search;
        otherwise falls back to listing all records.

        Args:
            query: The search query string.
            **kwargs: Additional keyword arguments. Supports ``k`` (int)
                to override the default top-K result count.

        Returns:
            A newline-separated string of result passages, or
            ``"No results found."`` if the store is empty.
        """
        if not query or not query.strip():
            return "No query provided."

        k = (
            kwargs.get("k", self.top_k)
            if hasattr(self, "top_k")
            else kwargs.get("k", DEFAULT_TOP_K)
        )

        if self.embedding:
            embedding = self.embedding(query)
            results = self._db.search_memory(
                self.namespace,
                embedding,
                top_k=k,
                distance_metric="cosine",
            )
            passages = (
                [hit.payload for hit in results]
                if hasattr(results, "__iter__")
                else []
            )
        else:
            # Fallback: list all
            results = self._db.list_memory(namespace=self.namespace, limit=k)
            records = (
                results.records
                if hasattr(results, "records")
                else list(results)
            )
            passages = [
                getattr(r, "payload", None) or str(r)
                for r in records
            ]

        return "\n".join(passages) if passages else "No results found."

    def _put(self, text: str, metadata: Optional[dict] = None) -> None:
        """Store a text entry with optional metadata in VantaDB.

        Implements the CrewAI ``BaseTool`` storage protocol. Generates a
        UUID key automatically. If an embedding function is configured, the
        vector is computed and stored alongside the text.

        Args:
            text: The text content to store. Must be non-empty.
            metadata: Optional dictionary of metadata to attach.

        Raises:
            ValueError: If ``text`` is empty or whitespace-only.
        """
        if not text or not text.strip():
            raise ValueError("Text cannot be empty")

        vector = None
        if self.embedding:
            vector = self.embedding(text)
        self._db.put(
            self.namespace,
            str(uuid.uuid4()),
            text,
            metadata=metadata or {},
            vector=vector,
        )

    def delete(self, key: str) -> bool:
        """Delete a record by key from the store.

        Args:
            key: The unique key of the record to delete.

        Returns:
            True if the deletion succeeded.
        """
        self._db.delete_memory(self.namespace, key)
        return True

    def list(self, limit: int = 100, cursor: Optional[str] = None) -> dict:
        """List records with optional pagination.

        Args:
            limit: Maximum number of records to return. Default 100.
            cursor: Optional pagination cursor from a previous ``list`` call.

        Returns:
            A dict with ``records`` and, if more are available, a ``cursor``.
        """
        # dspy pattern: cursor arrives as str from serialized pages; list_memory expects int.
        if cursor is None or (isinstance(cursor, str) and cursor == ""):
            cursor_int: Optional[int] = None
        else:
            try:
                cursor_int = int(cursor)  # type: ignore[arg-type]
            except (ValueError, TypeError) as exc:
                raise ValueError(
                    f"Invalid cursor value {cursor!r}: must be int or int-like string"
                ) from exc
        results = self._db.list_memory(
            namespace=self.namespace, limit=limit, cursor=cursor_int,
        )
        records = (
            results.records
            if hasattr(results, "records")
            else list(results)
        )
        # VantaListResult uses `next_cursor`; keep `cursor` fallback for compat
        next_cursor = getattr(results, "next_cursor", None)
        if next_cursor is None:
            next_cursor = getattr(results, "cursor", None)
        out: dict[str, Any] = {"records": records}
        if next_cursor is not None:
            out["cursor"] = next_cursor
        return out

    def to_dict(self) -> dict:
        """Serialize tool configuration to a dict.

        Returns:
            A dict with ``db_path``, ``namespace``, ``k``, and
            ``embedding_model`` keys.
        """
        return {
            "db_path": self.db_path,
            "namespace": self.namespace,
            "k": self.top_k,
            "embedding_model": (
                str(type(self.embedding).__name__)
                if self.embedding is not None
                else None
            ),
        }

    @classmethod
    def from_dict(cls, data: dict) -> VantaDBTool:
        """Create a VantaDBTool from a configuration dict.

        Args:
            data: Dict with tool configuration. Keys match ``to_dict``:
                ``db_path``, ``namespace``, ``k``, ``embedding_model``.

        Returns:
            A new ``VantaDBTool`` instance.

        Note:
            ``embedding_model`` is only a type-name string and cannot be
            reconstructed; it is intentionally ignored. Pass the embedding
            callable explicitly via ``embedding`` when you need semantic
            search after a roundtrip — otherwise the tool falls back to
            listing records.
        """
        # ponytail: minimal reconstruct — if caller supplies callable via `embedding`, use it; else fallback
        embedding = data.get("embedding")
        if not callable(embedding):
            maybe = data.get("embedding_model")
            if callable(maybe):
                embedding = maybe
            else:
                embedding = None
        return cls(
            embedding=embedding,
            db_path=data.get("db_path", "./vantadb_data"),
            namespace=data.get("namespace", DEFAULT_NAMESPACE),
            top_k=data.get("k", data.get("top_k", DEFAULT_TOP_K)),
        )

    def __call__(self, *args: Any, **kwargs: Any) -> str:
        return self._run(*args, **kwargs)
