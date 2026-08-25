"""VantaDB — The vector-graph database that thinks.

Sync and async bindings for the embedded persistent memory engine.
"""

from __future__ import annotations

import asyncio
from dataclasses import asdict, dataclass
from functools import partial

from .vantadb_py import (
    VantaDB,
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
    "__version__",
    "connect",
]


@dataclass
class SearchRequest:
    """Full search request for batch searches.

    Mirrors the keyword arguments of ``VantaDB.search_memory``. Pass instances
    (or equivalent dicts) to ``VantaDB.search_batch_requests``, which runs them
    in parallel in the Rust engine with GIL released.

    Args:
        namespace: Namespace to search within.
        query_vector: Query embedding vector (list of floats or NumPy array).
            Empty skips dense vector search.
        filters: Optional dict of metadata field values to filter on.
        text_query: Optional full-text query for BM25 lexical search.
        top_k: Maximum number of hits to return (default 10).
        distance_metric: ``"cosine"`` (default) or ``"euclidean"``.
        method: Optional index backend override: ``"ivf"``, ``"scann"``,
            ``"hnsw"`` or ``"flat"``. Defaults to the engine's configured
            routing.
        explain: Whether to include search explanations (default False).

    Example::

        from vantadb_py import VantaDB, SearchRequest

        db = VantaDB(":memory:")
        requests = [
            SearchRequest("ns", [1.0, 0.0, 0.0], text_query="memory", top_k=5),
            SearchRequest("ns", [0.0, 1.0, 0.0], filters={"kind": "task"}, top_k=5),
        ]
        results = db.search_batch_requests(requests)

    A plain dict with the same keys is also accepted (e.g. ``asdict(request)``).
    """

    namespace: str
    query_vector: list[float]
    filters: dict | None = None
    text_query: str | None = None
    top_k: int = 10
    distance_metric: str | None = None
    method: str | None = None
    explain: bool = False

    def asdict(self):
        """Return this request as a plain dict (for non-dataclass callers)."""
        return asdict(self)


class AsyncVantaDB:
    """Async wrapper around VantaDB.

    Query methods (search_memory, get_memory, list_memory) run
    in a thread pool via ``asyncio.to_thread()``, releasing the GIL
    to the Rust engine which already uses ``py.allow_threads()``.

    Usage::

        async with AsyncVantaDB("./my_brain") as db:
            record = await db.get_memory("ns", "key")
            results = await db.search_memory("ns", [1.0, 0.0, 0.0], top_k=5)
    """

    def __init__(self, *args, max_concurrency: int = 4, **kwargs):
        self._sync = VantaDB(*args, **kwargs)
        self._sem = asyncio.Semaphore(max_concurrency)

    async def _run(self, fn, *args, **kwargs):
        async with self._sem:
            return await asyncio.to_thread(partial(fn, *args, **kwargs))

    async def __aenter__(self):
        return self

    async def __aexit__(self, *exc):
        # Use to_thread to release the GIL so the Rust engine can close
        # without blocking the asyncio event loop. The close call itself
        # already uses py.allow_threads() on the Rust side.
        await self._run(self._sync.close)

    # ── Query methods (async via to_thread) ──

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
    ):
        return await self._run(
            self._sync.search_memory,
            namespace,
            query_vector,
            filters,
            text_query,
            top_k,
            distance_metric,
            method,
            explain,
            exclude_superseded,
        )

    async def get_memory(self, namespace: str, key: str):
        return await self._run(self._sync.get_memory, namespace, key)

    async def list_memory(
        self,
        namespace: str,
        *,
        filters: dict | None = None,
        limit: int = 100,
        cursor: int | None = None,
        exclude_superseded: bool = False,
    ):
        return await self._run(
            self._sync.list_memory,
            namespace,
            filters,
            limit,
            cursor,
            exclude_superseded,
        )

    # ── Mutations (sync wrappers for completeness) ──

    async def put(
        self,
        namespace: str,
        key: str,
        payload: str,
        *,
        metadata: dict | None = None,
        vector: list[float] | None = None,
        ttl_ms: int | None = None,
    ):
        return await self._run(
            self._sync.put, namespace, key, payload, metadata, vector, ttl_ms
        )

    async def delete_memory(self, namespace: str, key: str) -> bool:
        return await self._run(self._sync.delete_memory, namespace, key)

    async def delete_by_filter(self, namespace: str, filters: dict) -> int:
        return await self._run(self._sync.delete_by_filter, namespace, filters)

    async def count(self, namespace: str, filters: dict | None = None) -> int:
        return await self._run(self._sync.count, namespace, filters)

    async def similar_to_key(
        self, namespace: str, key: str, top_k: int = 10
    ) -> list:
        return await self._run(self._sync.similar_to_key, namespace, key, top_k)

    async def compact_wal(self):
        return await self._run(self._sync.compact_wal)

    async def supersede(self, namespace: str, old_key: str, new_key: str):
        return await self._run(self._sync.supersede, namespace, old_key, new_key)

    async def purge_expired(self) -> int:
        return await self._run(self._sync.purge_expired)

    async def flush(self):
        return await self._run(self._sync.flush)

    async def close(self):
        return await self._run(self._sync.close)

    async def insert(self, id, content, vector, fields=None):
        return await self._run(
            self._sync.insert, id, content, vector, fields
        )

    async def put_batch(
        self,
        entries=None,
        *,
        keys=None,
        vectors=None,
        payloads=None,
        metadatas=None,
        namespace=None,
        namespaces=None,
        ttls=None,
    ):
        return await self._run(
            self._sync.put_batch,
            entries,
            keys,
            vectors,
            payloads,
            metadatas,
            namespace,
            namespaces,
            ttls,
        )

    async def put_batch_raw(
        self,
        vectors,
        keys,
        *,
        payloads=None,
        metadatas=None,
        namespaces=None,
        ttls=None,
    ):
        """Insert multiple records using a 2D numpy array as vectors (zero-copy).

        Args:
            vectors: 2D numpy float32 array of shape ``[N, D]``, or any buffer-protocol object.
            keys: List of N string keys.
            payloads: Optional list of N string payloads.
            metadatas: Optional list of N dicts or None.
            namespaces: Optional list of N namespace strings (default "default").
            ttls: Optional list of N optional TTL values in ms.

        Returns a list of ``VantaMemoryRecord`` dicts in input order.
        """
        return await self._run(
            self._sync.put_batch_raw,
            vectors,
            keys,
            payloads,
            metadatas,
            namespaces,
            ttls,
        )

    async def rebuild_index(self):
        return await self._run(self._sync.rebuild_index)

    async def bulk_import(self, path: str):
        return await self._run(self._sync.bulk_import, path)

    async def bulk_import_bytes(self, data: bytes):
        return await self._run(self._sync.bulk_import_bytes, data)

    async def reindex_hnsw_from_text(self, namespace: str, *, page_size: int = 1000):
        return await self._run(
            self._sync.reindex_hnsw_from_text, namespace, page_size
        )

    async def export_namespace(self, path, namespace):
        return await self._run(
            self._sync.export_namespace, path, namespace
        )

    async def export_all(self, path):
        return await self._run(self._sync.export_all, path)

    async def import_file(self, path):
        return await self._run(self._sync.import_file, path)

    async def audit_text_index(self, namespace=None, deep=False):
        return await self._run(
            self._sync.audit_text_index, namespace, deep
        )

    async def repair_text_index(self):
        return await self._run(self._sync.repair_text_index)

    async def operational_metrics(self):
        return await self._run(self._sync.operational_metrics)

    async def get(self, id):
        return await self._run(self._sync.get, id)

    async def delete(self, id, reason="manual deletion"):
        return await self._run(self._sync.delete, id, reason)

    async def search(self, vector, top_k=10):
        return await self._run(self._sync.search, vector, top_k)

    async def search_batch(self, vectors, top_k=10):
        return await self._run(
            self._sync.search_batch, vectors, top_k
        )

    async def search_batch_requests(self, requests, top_k=10):
        return await self._run(
            self._sync.search_batch_requests, requests, top_k
        )

    async def query(self, iql_query):
        return await self._run(self._sync.query, iql_query)

    async def capabilities(self):
        return await self._run(self._sync.capabilities)

    async def add_edge(
        self, source_id, target_id, label, weight=None, created_at_ms=None
    ):
        return await self._run(
            self._sync.add_edge,
            source_id,
            target_id,
            label,
            weight,
            created_at_ms,
        )

    async def graph_bfs(self, roots, max_depth=999999):
        return await self._run(
            self._sync.graph_bfs, roots, max_depth
        )

    async def graph_dfs(self, roots, max_depth=999999):
        return await self._run(
            self._sync.graph_dfs, roots, max_depth
        )

    async def graph_topological_sort(self, roots):
        return await self._run(
            self._sync.graph_topological_sort, roots
        )

    async def graph_is_dag(self, roots):
        return await self._run(self._sync.graph_is_dag, roots)

    async def graph_page_rank(
        self, roots, max_iterations=100, damping=0.85, tolerance=1e-6
    ):
        return await self._run(
            self._sync.graph_page_rank,
            roots,
            max_iterations,
            damping,
            tolerance,
        )

    async def graph_degree_centrality(self, roots):
        return await self._run(self._sync.graph_degree_centrality, roots)

    async def compact_layout(self):
        return await self._run(self._sync.compact_layout)

    async def list_namespaces(self):
        return await self._run(self._sync.list_namespaces)

    async def generate_snippet(
        self, payload, text_query, with_highlighting=False
    ):
        return await self._run(
            self._sync.generate_snippet,
            payload,
            text_query,
            with_highlighting,
        )

    async def explain_memory_search(
        self,
        namespace: str,
        query_vector: list[float],
        *,
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
    ):
        return await self._run(
            self._sync.explain_memory_search,
            namespace,
            query_vector,
            filters,
            text_query,
            top_k,
            distance_metric,
        )

    # ── Passthrough for sync methods ──

    async def hardware_profile(self):
        return await self._run(self._sync.hardware_profile)

    def __repr__(self):
        return f"AsyncVantaDB(sync={self._sync!r})"
