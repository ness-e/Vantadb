"""VantaDB — The vector-graph database that thinks.

Sync and async bindings for the embedded persistent memory engine.
"""

from __future__ import annotations

import asyncio
from functools import partial

from .vantadb_py import VantaDB, VantaListResult, VantaMemoryRecord, VantaSearchHit, VantaVector, __version__

__all__ = [
    "VantaDB",
    "AsyncVantaDB",
    "VantaListResult",
    "VantaMemoryRecord",
    "VantaSearchHit",
    "VantaVector",
    "__version__",
]


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
            return await self._run(partial(fn, *args, **kwargs))

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
        explain: bool = False,
    ):
        return         await self._run(
            self._sync.search_memory,
            namespace,
            query_vector,
            filters,
            text_query,
            top_k,
            distance_metric,
            explain,
        )

    async def get_memory(self, namespace: str, key: str):
        return         await self._run(self._sync.get_memory, namespace, key)

    async def list_memory(
        self,
        namespace: str,
        *,
        filters: dict | None = None,
        limit: int = 100,
        cursor: int | None = None,
    ):
        return         await self._run(
            self._sync.list_memory,
            namespace,
            filters,
            limit,
            cursor,
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
        return         await self._run(
            self._sync.put, namespace, key, payload, metadata, vector, ttl_ms
        )

    async def delete_memory(self, namespace: str, key: str) -> bool:
        return         await self._run(self._sync.delete_memory, namespace, key)

    async def compact_wal(self):
        return         await self._run(self._sync.compact_wal)

    async def purge_expired(self) -> int:
        return         await self._run(self._sync.purge_expired)

    async def flush(self):
        return         await self._run(self._sync.flush)

    async def close(self):
        return         await self._run(self._sync.close)

    async def insert(self, id, content, vector, fields=None):
        return await self._run(
            self._sync.insert, id, content, vector, fields
        )

    async def put_batch(self, entries):
        return await self._run(self._sync.put_batch, entries)

    async def rebuild_index(self):
        return await self._run(self._sync.rebuild_index)

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

    async def query(self, iql_query):
        return await self._run(self._sync.query, iql_query)

    async def capabilities(self):
        return await self._run(self._sync.capabilities)

    async def add_edge(self, source_id, target_id, label, weight=None):
        return await self._run(
            self._sync.add_edge, source_id, target_id, label, weight
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
        return         await self._run(
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
