from __future__ import annotations

import json
import uuid
from typing import Any, Dict, List, Optional, Callable

import vantadb_py as vanta
from mem0.vector_stores.base import VectorStoreBase

DEFAULT_NAMESPACE = "mem0"
DEFAULT_TOP_K = 5


class OutputData:
    """Lightweight search/query result — id, score, payload.

    Matches the shape mem0's Memory layer expects from vector stores:
    ``.id`` (str), ``.score`` (float in [0,1], higher=better), ``.payload`` (dict).
    """

    __slots__ = ("id", "score", "payload")

    def __init__(
        self,
        id: Optional[str] = None,
        score: Optional[float] = None,
        payload: Optional[Dict[str, Any]] = None,
    ):
        """Initialize a lightweight search result container.

        Args:
            id: Optional record identifier.
            score: Optional relevance score in [0, 1].
            payload: Optional dictionary of result data.
                Defaults to an empty dict.
        """
        self.id = id
        self.score = score
        self.payload = payload or {}

    def __repr__(self) -> str:
        return f"OutputData(id={self.id!r}, score={self.score!r})"


def _normalize_score(raw: Optional[float]) -> float:
    """Return a score in [0, 1] where 1 = most similar.

    If the value is already in [0,1] it passes through unchanged.
    Otherwise it is treated as a distance and inverted.
    """
    if raw is None:
        return 0.0
    if 0.0 <= raw <= 1.0:
        return raw
    return max(0.0, 1.0 - raw)


def _build_payload(metadata, payload_text: str = "") -> Dict[str, Any]:
    """Merge VantaDB record fields into a single payload dict."""
    d = dict(metadata) if metadata else {}
    if payload_text:
        d["payload"] = payload_text
    return d


# ──────────────────────────────────────────────────────────────────────
#  VectorStoreBase implementation
# ──────────────────────────────────────────────────────────────────────


class VantaDBVectorStore(VectorStoreBase):
    """Mem0 vector-store adapter backed by VantaDB.

    Usage (direct, out-of-band)::

        store = VantaDBVectorStore(db_path="/tmp/vdb", namespace="memories")
        store.create_col("memories", vector_size=384)
        store.insert([vector], payloads=[{"data": "hello"}], ids=["k1"])
        results = store.search("hello", vector, top_k=5)

    The constructor also accepts an optional ``embedding`` callable for
    backward-compatible convenience methods (``.add()``).
    """

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
        """Initialize a Mem0 VectorStoreBase adapter backed by VantaDB.

        Args:
            embedding: Optional callable that converts text to an
                embedding vector list. Used by the convenience method
                ``.add()``.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace (collection) to operate on.
                Defaults to ``"mem0"``.
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

    # ------------------------------------------------------------------
    #  VectorStoreBase abstract methods  (11 methods)
    # ------------------------------------------------------------------

    def create_col(self, name: str, vector_size: Optional[int] = None, distance: str = "cosine") -> None:
        """Create a collection (namespace in VantaDB).

        VantaDB is schemaless — namespaces are created lazily on first
        write.  Logged for observability; actual creation happens on
        first insert.
        """
        import logging
        logging.getLogger(__name__).info(
            f"create_col({name}): VantaDB auto-creates on first insert; "
            f"vector_size={vector_size}, distance={distance} ignored"
        )

    def insert(
        self,
        vectors: List[List[float]],
        payloads: Optional[List[Dict[str, Any]]] = None,
        ids: Optional[List[str]] = None,
    ) -> None:
        """Insert a batch of vectors with optional payloads and ids."""
        for i, vec in enumerate(vectors):
            key = ids[i] if ids else str(uuid.uuid4())
            p = payloads[i] if payloads else {}
            text = p.get("data") or p.get("text") or p.get("content") or json.dumps(p)
            self._db.put(
                self.namespace,
                key,
                text,
                metadata=dict(p),
                vector=vec,
            )

    def search(
        self,
        query: str,
        vectors: List[float],
        top_k: int = DEFAULT_TOP_K,
        filters: Optional[Dict[str, Any]] = None,
    ) -> List[OutputData]:
        """Search by pre-computed vector.

        ``vectors`` is a single dense vector (list of floats).  Mem0 handles
        embeddings externally and passes the result here.
        """
        results = self._db.search_memory(
            self.namespace, vectors, top_k=top_k, distance_metric="cosine"
        )
        return [
            OutputData(
                id=hit.key,
                score=_normalize_score(hit.score),
                payload=_build_payload(hit.metadata, hit.payload),
            )
            for hit in results
        ]

    def delete(self, vector_id: str) -> None:
        """Delete a single record by its id."""
        self._db.delete_memory(self.namespace, vector_id)

    def update(
        self,
        vector_id: str,
        vector: Optional[List[float]] = None,
        payload: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Replace the vector and/or payload of an existing record.

        First verifies the record exists, then attempts an atomic
        ``update_memory`` call.  Falls back to delete + insert if the
        underlying VantaDB version does not provide ``update_memory``.
        """
        existing = self.get(vector_id)
        if existing is None:
            raise ValueError(f"Record {vector_id} not found")
        cur = dict(existing.payload)
        if payload:
            cur.update(payload)
        text = cur.get("data") or cur.get("text") or cur.get("content") or ""
        try:
            self._db.update_memory(self.namespace, vector_id, text)
        except AttributeError:
            # Fallback: delete + insert
            self._db.delete_memory(self.namespace, vector_id)
            self._db.put(
                self.namespace, vector_id, text, metadata=cur, vector=vector,
            )

    def get(self, vector_id: str) -> Optional[OutputData]:
        """Retrieve a single record by its id."""
        try:
            record = self._db.get_memory(self.namespace, vector_id)
        except Exception:
            return None
        if record is None:
            return None
        return OutputData(
            id=getattr(record, "key", vector_id),
            score=1.0,
            payload=_build_payload(getattr(record, "metadata", None), getattr(record, "payload", "")),
        )

    def list_cols(self) -> List[str]:
        """Return all known collection (namespace) names."""
        try:
            return self._db.list_namespaces()
        except Exception:
            return [self.namespace]

    def delete_col(self) -> None:
        """Remove the current collection (namespace)."""
        try:
            self._db.delete_namespace(self.namespace)
        except Exception as e:
            # Fallback: delete individual records
            try:
                records = self._db.list_memory(self.namespace, limit=10000)
                for r in records.records if hasattr(records, 'records') else records:
                    self._db.delete_memory(self.namespace, r.key)
            except Exception as e2:
                raise RuntimeError(f"Failed to delete collection {self.namespace}: {e2}") from e

    def col_info(self) -> Dict[str, Any]:
        """Return basic metadata about the collection."""
        return {
            "name": self.namespace,
            "vector_size": None,
            "distance": "cosine",
        }

    def list(
        self,
        filters: Optional[Dict[str, Any]] = None,
        top_k: int = 100,
    ) -> List[OutputData]:
        """List records in the collection, optionally filtered."""
        raw = self._db.list_memory(self.namespace, filters=filters, limit=top_k)
        return [
            OutputData(
                id=r.key,
                score=1.0,
                payload=_build_payload(r.metadata, r.payload),
            )
            for r in raw.records
        ]

    def reset(self) -> None:
        """Delete the collection so it is recreated on next use."""
        self.delete_col()

    # ------------------------------------------------------------------
    #  Backward-compatible convenience method
    # ------------------------------------------------------------------

    def add(
        self,
        text: str,
        user_id: Optional[str] = None,
        metadata: Optional[dict] = None,
        **kwargs: Any,
    ) -> str:
        """Single text insert (original flat-class API).

        If ``self.embedding`` is set, the vector is computed internally.
        Returns the assigned key.
        """
        key = str(uuid.uuid4())
        meta = dict(metadata or {})
        if user_id:
            meta["user_id"] = user_id
        if self.embedding is not None:
            self._db.put(self.namespace, key, text, metadata=meta, vector=self.embedding(text))
        else:
            self._db.put(self.namespace, key, text, metadata=meta)
        return key
