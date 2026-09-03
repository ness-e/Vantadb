"""Type stubs for the ``vantadb_py`` wrapper package.

Native signatures live in ``vantadb_py.pyi`` (the compiled extension,
``vantadb_py.pyd``). This stub only adds what ``__init__.py`` defines:
``SearchRequest``, ``AsyncVantaDB`` and the re-exports — mirroring the real
``from .vantadb_py import ...`` so the package surface stays typed without
re-declaring native classes.

``import vantadb`` (the ``vantadb/`` alias package) resolves the same
``__all__`` transitively, so it needs no stub of its own.
"""

from __future__ import annotations

from typing import Any

from .vantadb_py import (
    BusyError,
    ConflictError,
    CorruptError,
    NotFoundError,
    NoVectorError,
    ResourceLimitError,
    StorageError,
    TimeoutError,
    UnsupportedError,
    VantaDB,
    VantaError,
    ValidationError,
    VantaListResult,
    VantaMemoryRecord,
    VantaSearchHit,
    VantaVector,
    __version__,
    connect,
)

__all__ = [
    "VantaDB",
    "AsyncVantaDB",
    "VantaListResult",
    "VantaMemoryRecord",
    "VantaSearchHit",
    "VantaVector",
    "SearchRequest",
    "VantaError",
    "NotFoundError",
    "ValidationError",
    "CorruptError",
    "StorageError",
    "ConflictError",
    "UnsupportedError",
    "ResourceLimitError",
    "BusyError",
    "NoVectorError",
    "TimeoutError",
    "__version__",
    "connect",
    "error_to_dict",
]


def error_to_dict(exc: BaseException) -> dict[str, Any]:
    """Serialize a VantaDB error to a plain dict mirroring TS ``toJSON()``
    (``docs/api/ERROR_HANDLING.md`` §5.2): name/code/message/retriable/hint."""
    ...


class SearchRequest:
    """Full search request for batch searches.

    Mirrors the keyword arguments of ``VantaDB.search_memory``. Pass
    instances (or equivalent dicts) to ``VantaDB.search_batch_requests``.
    """

    namespace: str
    query_vector: list[float]
    filters: dict | None
    text_query: str | None
    top_k: int
    distance_metric: str | None
    method: str | None
    explain: bool

    def __init__(
        self,
        namespace: str,
        query_vector: list[float],
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
        method: str | None = None,
        explain: bool = False,
    ) -> None: ...

    def asdict(self) -> dict:
        """Return this request as a plain dict (for non-dataclass callers)."""
        ...


class AsyncVantaDB:
    """Async wrapper around ``VantaDB``.

    Query methods run in a thread pool via ``asyncio.to_thread()``,
    releasing the GIL to the Rust engine (``py.allow_threads``).
    """

    def __init__(self, *args: Any, max_concurrency: int = 4, **kwargs: Any) -> None: ...
    async def __aenter__(self) -> AsyncVantaDB: ...
    async def __aexit__(self, *exc: Any) -> None: ...

    async def search_memory(
        self,
        namespace: str,
        query_vector: list[float],
        *,
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
        method: str | None = None,
        explain: bool = False,
        exclude_superseded: bool = False,
    ) -> list[VantaSearchHit]: ...
    async def get_memory(self, namespace: str, key: str) -> VantaMemoryRecord | None: ...
    async def list_memory(
        self,
        namespace: str,
        *,
        filters: dict | None = None,
        limit: int = 100,
        cursor: int | None = None,
        exclude_superseded: bool = False,
    ) -> VantaListResult: ...
    async def put(
        self,
        namespace: str,
        key: str,
        payload: str,
        *,
        metadata: dict | None = None,
        vector: list[float] | None = None,
        ttl_ms: int | None = None,
    ) -> VantaMemoryRecord: ...
    async def delete_memory(self, namespace: str, key: str) -> bool: ...
    async def delete_by_filter(self, namespace: str, filters: dict) -> int: ...
    async def count(self, namespace: str, filters: dict | None = None) -> int: ...
    async def similar_to_key(
        self, namespace: str, key: str, top_k: int = 10
    ) -> list[VantaSearchHit]: ...
    async def compact_wal(self) -> None: ...
    async def supersede(self, namespace: str, old_key: str, new_key: str) -> None: ...
    async def purge_expired(self) -> int: ...
    async def flush(self) -> None: ...
    async def close(self) -> None: ...
    async def insert(
        self, id: int, content: str, vector: Any, fields: dict | None = None
    ) -> None: ...
    async def put_batch(
        self,
        *,
        keys: list[str],
        vectors: Any,
        payloads: list[str] | None = None,
        metadatas: list[dict | None] | None = None,
        namespace: str | None = None,
        namespaces: list[str] | None = None,
        ttls: list[int | None] | None = None,
    ) -> list[VantaMemoryRecord]: ...
    async def put_batch_raw(
        self,
        vectors: Any,
        keys: list[str],
        *,
        payloads: list[str] | None = None,
        metadatas: list[dict | None] | None = None,
        namespaces: list[str] | None = None,
        ttls: list[int | None] | None = None,
    ) -> list[VantaMemoryRecord]: ...
    async def rebuild_index(self) -> dict: ...
    async def bulk_import(self, path: str) -> dict: ...
    async def bulk_import_bytes(self, data: bytes) -> dict: ...
    async def reindex_hnsw_from_text(
        self, namespace: str, *, page_size: int = 1000
    ) -> dict: ...
    async def export_namespace(self, path: str, namespace: str) -> dict: ...
    async def export_all(self, path: str) -> dict: ...
    async def import_file(self, path: str) -> dict: ...
    async def audit_text_index(
        self, namespace: str | None = None, deep: bool = False
    ) -> dict: ...
    async def repair_text_index(self) -> dict: ...
    async def operational_metrics(self) -> dict: ...
    async def get(self, id: int) -> dict | None: ...
    async def delete(self, id: int, reason: str = "manual deletion") -> None: ...
    async def search(self, vector: Any, top_k: int = 10) -> list[tuple[int, float]]: ...
    async def search_batch(
        self, vectors: list[Any], top_k: int = 10
    ) -> list[list[tuple[int, float]]]: ...
    async def search_batch_requests(
        self, requests: list[Any], top_k: int = 10
    ) -> list[list[VantaSearchHit]]: ...
    async def query(self, iql_query: str) -> str: ...
    async def query_structured(self, iql_query: str) -> dict: ...
    async def capabilities(self) -> dict: ...
    async def hardware_profile(self) -> dict: ...
    async def add_edge(
        self,
        source_id: int,
        target_id: int,
        label: str,
        weight: float | None = None,
        created_at_ms: int | None = None,
    ) -> None: ...
    async def graph_bfs(
        self, roots: list[int], max_depth: int = 999999, direction: str = "Forward"
    ) -> list[int]: ...
    async def graph_dfs(
        self, roots: list[int], max_depth: int = 999999, direction: str = "Forward"
    ) -> list[int]: ...
    async def graph_topological_sort(self, roots: list[int]) -> list[int]: ...
    async def graph_is_dag(self, roots: list[int]) -> bool: ...
    async def graph_page_rank(
        self,
        roots: list[int],
        max_iterations: int = 100,
        damping: float = 0.85,
        tolerance: float = 1e-6,
    ) -> dict[int, float]: ...
    async def graph_degree_centrality(
        self, roots: list[int]
    ) -> dict[int, tuple[int, int]]: ...
    async def compact_layout(self) -> int: ...
    async def list_namespaces(self) -> list[str]: ...
    async def generate_snippet(
        self,
        payload: str,
        text_query: str,
        with_highlighting: bool = False,
    ) -> str | None: ...
    async def explain_memory_search(
        self,
        namespace: str,
        query_vector: list[float],
        *,
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
    ) -> dict: ...
    def __repr__(self) -> str: ...