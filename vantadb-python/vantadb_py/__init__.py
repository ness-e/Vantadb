"""VantaDB — The vector-graph database that thinks.

Sync and async bindings for the embedded persistent memory engine.
"""

from __future__ import annotations

import asyncio
from functools import partial

from .vantadb_native import VantaDB, __version__

__all__ = ["VantaDB", "AsyncVantaDB", "__version__"]


def _to_thread(fn, *args, **kwargs):
    return asyncio.to_thread(partial(fn, *args, **kwargs))


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

    def __init__(self, *args, **kwargs):
        self._sync = VantaDB(*args, **kwargs)

    async def __aenter__(self):
        return self

    async def __aexit__(self, *exc):
        self._sync.close()

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
        return await _to_thread(
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
        return await _to_thread(self._sync.get_memory, namespace, key)

    async def list_memory(
        self,
        namespace: str,
        *,
        filters: dict | None = None,
        limit: int = 100,
        cursor: int | None = None,
    ):
        return await _to_thread(
            self._sync.list_memory, namespace, filters, limit, cursor
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
    ):
        return await _to_thread(
            self._sync.put, namespace, key, payload, metadata, vector
        )

    async def delete_memory(self, namespace: str, key: str) -> bool:
        return await _to_thread(self._sync.delete_memory, namespace, key)

    async def flush(self):
        return await _to_thread(self._sync.flush)

    async def close(self):
        return await _to_thread(self._sync.close)

    # ── Passthrough for sync methods that are instant ──

    @property
    def hardware_profile(self):
        return self._sync.hardware_profile()

    def __repr__(self):
        return f"AsyncVantaDB(sync={self._sync!r})"
