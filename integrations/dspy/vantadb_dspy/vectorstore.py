from __future__ import annotations

from typing import Any, Callable, List, Optional

import vantadb_py as vanta

try:
    import dspy
except ImportError:
    dspy = None  # type: ignore[assignment]

try:
    from dspy import Retrieve as DSPyRetrieve
except ImportError:
    DSPyRetrieve = object  # fallback si dspy no está instalado

DEFAULT_NAMESPACE = "dspy"
DEFAULT_TOP_K = 4


class VantaDBRetriever(DSPyRetrieve):
    def __init__(
        self,
        embedding: Optional[Callable[[str], List[float]]] = None,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        k: int = DEFAULT_TOP_K,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        """Initialize a VantaDB-backed DSPy retriever.

        Implements the ``dspy.Retrieve`` protocol for use in DSPy
        programs. When ``embedding`` is provided, retrieval uses vector
        similarity; otherwise it falls back to text-based filtering.

        Args:
            embedding: Optional callable that converts a text string to
                an embedding vector list.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"dspy"``.
            k: Default number of results to retrieve.
                Defaults to 4.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        super().__init__(k=k)
        self.embedding = embedding
        self.k = k
        self.namespace = namespace
        self.db_path = db_path
        self.memory_limit_bytes = memory_limit_bytes
        self.read_only = read_only
        self.backend = backend
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    def forward(self, query: str, **kwargs: Any) -> List[str]:
        """Execute retrieval and return results as a DSPy Prediction.

        Implements the DSPy ``Retrieve.forward()`` protocol. When an
        embedding function is configured, performs vector similarity search;
        otherwise filters all records by substring match.

        Args:
            query: The search query string.
            **kwargs: Additional keyword arguments. Supports ``k`` (int)
                to override the default top-K result count.

        Returns:
            A ``dspy.Prediction`` with a ``passages`` attribute
            containing the result text list, or a plain list if DSPy
            is not installed.
        """
        k = kwargs.get("k", self.k)
        if self.embedding is not None:
            vector = self.embedding(query)
            results = self._db.search_memory(
                self.namespace, vector, top_k=k, distance_metric="cosine"
            )
            passages = [hit.payload for hit in results]
        else:
            # fallback: text-based filter over all records
            results = self._db.list_memory(self.namespace, limit=k)
            passages = [
                r.payload for r in results.records
                if r.payload and query.lower() in r.payload.lower()
            ]
        if dspy is not None:
            return dspy.Prediction(passages=passages)
        return passages

    def __call__(self, query: str, **kwargs: Any) -> Any:
        return self.forward(query, **kwargs)

    def dump_state(self):
        """Serialize the retriever state for DSPy checkpointing.

        Extends the parent ``dump_state()`` with VantaDB-specific
        configuration fields.

        Returns:
            A dictionary containing the serialised state, including
            ``namespace``, ``db_path``, ``k``, and ``backend``.
        """
        state = super().dump_state() if hasattr(super(), "dump_state") else {}
        state.update(
            {
                "namespace": self.namespace,
                "db_path": self.db_path,
                "k": self.k,
                "backend": self.backend,
            }
        )
        return state

    def load_state(self, state):
        """Restore the retriever state from a DSPy checkpoint.

        Args:
            state: A dictionary with serialised state, as produced by
                ``dump_state()``. Expected keys include ``namespace``,
                ``db_path``, ``k``, and ``backend``.
        """
        if hasattr(super(), "load_state"):
            super().load_state(state)
        self.namespace = state.get("namespace", self.namespace)
        self.db_path = state.get("db_path", self.db_path)
        self.k = state.get("k", self.k)
        self.backend = state.get("backend", self.backend)

    def _add(self, text: str, key: str, metadata: Optional[dict] = None) -> None:
        vector = self.embedding(text) if self.embedding else None
        self._db.put(self.namespace, key, text, metadata=metadata, vector=vector)
