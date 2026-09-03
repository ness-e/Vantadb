"""Type stubs for the Rust-native ``vantadb_py`` extension module.

Single source of truth for the signatures of the compiled ``vantadb_py``
extension (built from ``src/lib.rs`` + ``src/types.rs``). The wrapper package
``vantadb_py/__init__.py`` re-exports from this module; its stub
(``__init__.pyi``) must NOT re-declare these classes.

Package structure (``pyproject.toml [tool.maturin] module-name = "vantadb_py"``):

- ``vantadb_py.pyd`` — the compiled extension, stubbed by THIS file.
- ``vantadb_py/__init__.py`` — pure-Python wrapper (``SearchRequest``,
  ``AsyncVantaDB``, re-exports); stubbed by ``__init__.pyi``.
- ``vantadb/__init__.py`` — canonical ``import vantadb`` alias (re-exports
  ``vantadb_py``; typed transitively, no stub of its own).

``MemoryClient``/``GraphClient``/``SystemClient``/``WikiClient`` are not
module-level names at runtime (they are returned by the ``VantaDB`` getters
``db.memory|graph|system|wiki``), so they are declared here only as types.

Anti-drift: ``tests/test_stub_drift.py`` asserts this file matches the
compiled module (method set, parameter names and requiredness via
``inspect.signature``).
"""

from __future__ import annotations

from typing import Any


class VantaError(RuntimeError):
    """Base class for every VantaDB error raised by this binding.

    Inherits from ``RuntimeError`` so existing ``except RuntimeError`` /
    ``except Exception`` callers keep working. Raise/catch the specific
    subclasses below for typed error handling.

    ERR-PY-01: every raised instance carries the canonical metadata from
    ``docs/api/ERROR_HANDLING.md`` §5.1. ``to_dict()`` is not a method
    (``create_exception`` types cannot carry them) — use
    ``vantadb.error_to_dict(exc)`` for the §5.2 plain dict.
    """

    code: str
    """Canonical ``VANTADB_*`` code (exact wire value, §1.1)."""
    retriable: bool
    """Mirrors Rust ``VantaError::is_retriable()``."""
    hint: str | None
    """Recovery hint from ``recovery_hint()``; ``None`` when absent."""


class NotFoundError(VantaError):
    """A requested node/record/namespace was not found."""


class ValidationError(VantaError):
    """Input validation failed (bad dimensions, invalid IQL, schema, etc.)."""


class CorruptError(VantaError):
    """Persisted data is corrupt or uses an incompatible format."""


class StorageError(VantaError):
    """An I/O or storage-backend error occurred."""


class ConflictError(VantaError):
    """An execution conflict or graph cycle was detected."""


class UnsupportedError(VantaError):
    """An unsupported operation was attempted."""


class ResourceLimitError(VantaError):
    """A resource limit (e.g. memory) was exceeded."""


class BusyError(VantaError):
    """The database is busy or not initialized."""


class NoVectorError(VantaError):
    """A record exists but carries no vector."""


class TimeoutError(VantaError):
    """An operation exceeded its time budget (VantaDB's, not the builtin)."""


class VantaVector:
    """Read-only view over a ``f32`` vector exposed by search hits."""

    def __len__(self) -> int: ...
    def __getitem__(self, idx: int) -> float: ...
    def __iter__(self) -> VantaVectorIter: ...
    def __repr__(self) -> str: ...


class VantaVectorIter:
    def __iter__(self) -> VantaVectorIter: ...
    def __next__(self) -> float: ...


class VantaSearchHit:
    """A single search result."""

    namespace: str
    key: str
    payload: str
    metadata: dict
    vector: VantaVector | None
    score: float
    id: int
    created_at_ms: int
    updated_at_ms: int
    version: int
    node_id: int
    expires_at_ms: int | None

    def __repr__(self) -> str: ...


class VantaMemoryRecord:
    """A memory record with typed property access."""

    namespace: str
    key: str
    payload: str
    metadata: dict
    vector: Any | None
    created_at_ms: int
    updated_at_ms: int
    version: int
    node_id: int
    expires_at_ms: int | None

    def __getitem__(self, key: str) -> Any: ...
    def __repr__(self) -> str: ...


class VantaListResult:
    """A page of memory records with pagination info."""

    records: list[VantaMemoryRecord]
    total_count: int
    next_cursor: int | None

    def __len__(self) -> int: ...
    def __getitem__(self, key: int | str) -> Any: ...
    def __iter__(self) -> VantaListResultIter: ...
    def __repr__(self) -> str: ...


class VantaListResultIter:
    def __iter__(self) -> VantaListResultIter: ...
    def __next__(self) -> VantaMemoryRecord: ...


class VantaDB:
    """Create or open a VantaDB database.

    Args:
        db_path: Path to the database directory. Pass ``":memory:"`` (with
            ``backend="memory"``) for an in-memory database.
        memory_limit_bytes: Optional memory budget in bytes for the Rust
            engine. If None, uses hardware detection or the
            ``VANTADB_MEMORY_LIMIT`` env var.
        read_only: If True, opens the DB in read-only mode.
        backend: Storage backend — ``"memory"``, ``"rocksdb"``, or None
            (None selects the default persistent backend, fjall).
    """

    def __init__(
        self,
        db_path: str,
        memory_limit_bytes: int | None = None,
        read_only: bool = False,
        backend: str | None = None,
    ) -> None: ...

    # ── Memory records (also grouped under db.memory.*) ────────────────────

    def put(
        self,
        namespace: str,
        key: str,
        payload: str,
        metadata: dict | None = None,
        vector: Any | None = None,
        ttl_ms: int | None = None,
    ) -> VantaMemoryRecord: ...
    def put_batch(
        self,
        keys: list[str],
        vectors: list[list[float]],
        payloads: list[str] | None = None,
        metadatas: list[dict | None] | None = None,
        namespace: str | None = None,
        namespaces: list[str] | None = None,
        ttls: list[int | None] | None = None,
    ) -> list[VantaMemoryRecord]: ...
    def put_batch_raw(
        self,
        vectors: Any,
        keys: list[str],
        payloads: list[str] | None = None,
        metadatas: list[dict | None] | None = None,
        namespaces: list[str] | None = None,
        ttls: list[int | None] | None = None,
    ) -> list[VantaMemoryRecord]: ...
    def get_memory(self, namespace: str, key: str) -> VantaMemoryRecord | None: ...
    def delete_memory(self, namespace: str, key: str) -> bool: ...
    def delete_by_filter(self, namespace: str, filters: dict) -> int: ...
    def count(self, namespace: str, filters: dict | None = None) -> int: ...
    def similar_to_key(
        self, namespace: str, key: str, top_k: int = 10
    ) -> list[VantaSearchHit]: ...
    def list_memory(
        self,
        namespace: str,
        filters: dict | None = None,
        limit: int = 100,
        cursor: int | None = None,
        exclude_superseded: bool = False,
    ) -> VantaListResult: ...
    def search_memory(
        self,
        namespace: str,
        query_vector: Any,
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
        method: str | None = None,
        explain: bool = False,
        exclude_superseded: bool = False,
    ) -> list[VantaSearchHit]: ...
    def search(self, vector: Any, top_k: int = 10) -> list[tuple[int, float]]: ...
    def search_batch(
        self, vectors: list[Any], top_k: int = 10
    ) -> list[list[tuple[int, float]]]: ...
    def search_batch_requests(
        self, requests: list[Any], top_k: int = 10
    ) -> list[list[VantaSearchHit]]: ...
    def explain_memory_search(
        self,
        namespace: str,
        query_vector: list[float],
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
    ) -> dict: ...
    def supersede(self, namespace: str, old_key: str, new_key: str) -> None: ...
    def generate_snippet(
        self,
        payload: str,
        text_query: str,
        with_highlighting: bool = False,
    ) -> str | None: ...
    def purge_expired(self) -> int: ...
    def list_namespaces(self) -> list[str]: ...

    # ── Graph nodes and edges (also grouped under db.graph.*) ──────────────

    def insert(
        self, id: int, content: str, vector: Any, fields: dict | None = None
    ) -> None: ...
    def get(self, id: int) -> dict | None: ...
    def delete(self, id: int, reason: str = "manual deletion") -> None: ...
    def add_edge(
        self,
        source_id: int,
        target_id: int,
        label: str,
        weight: float | None = None,
        created_at_ms: int | None = None,
    ) -> None: ...
    def graph_bfs(
        self, roots: list[int], max_depth: int = 999999, direction: str = "Forward"
    ) -> list[int]: ...
    def graph_bfs_filtered(
        self,
        roots: list[int],
        max_depth: int = 999999,
        direction: str = "Forward",
        labels: list[int] | None = None,
        time_range: tuple[int, int] | None = None,
    ) -> list[int]: ...
    def graph_dfs(
        self, roots: list[int], max_depth: int = 999999, direction: str = "Forward"
    ) -> list[int]: ...
    def graph_topological_sort(self, roots: list[int]) -> list[int]: ...
    def graph_is_dag(self, roots: list[int]) -> bool: ...
    def graph_page_rank(
        self,
        roots: list[int],
        max_iterations: int = 100,
        damping: float = 0.85,
        tolerance: float = 1e-6,
    ) -> dict[int, float]: ...
    def graph_degree_centrality(
        self, roots: list[int]
    ) -> dict[int, tuple[int, int]]: ...

    # ── System / lifecycle (also grouped under db.system.*) ────────────────

    def capabilities(self) -> dict: ...
    def hardware_profile(self) -> dict: ...
    def operational_metrics(self) -> dict: ...
    def query(self, iql_query: str) -> str: ...
    def query_structured(self, iql_query: str) -> dict: ...
    def flush(self) -> None: ...
    def compact_wal(self) -> None: ...
    def compact_layout(self) -> int: ...
    def rebuild_index(self) -> dict: ...
    def reindex_hnsw_from_text(self, namespace: str, page_size: int = 1000) -> dict: ...
    def repair_text_index(self) -> dict: ...
    def audit_text_index(self, namespace: str | None = None, deep: bool = False) -> dict: ...
    def export_namespace(self, path: str, namespace: str) -> dict: ...
    def export_all(self, path: str) -> dict: ...
    def import_file(self, path: str) -> dict: ...
    def bulk_import(self, path: str) -> dict: ...
    def bulk_import_bytes(self, data: bytes) -> dict: ...
    def close(self) -> None: ...

    # ── Wiki summary-archive recovery (also grouped under db.wiki.*) ───────

    def recover_archived_nodes(self, summary_id: str) -> list[Any]: ...

    # ── Domain sub-clients (SDKB-03) ───────────────────────────────────────

    @property
    def memory(self) -> MemoryClient: ...
    @property
    def graph(self) -> GraphClient: ...
    @property
    def system(self) -> SystemClient: ...
    @property
    def wiki(self) -> WikiClient: ...


class MemoryClient:
    """Grouped view over ``VantaDB`` memory-record methods (``db.memory.*``).

    Native forwarder: every method delegates to the same-named flat method on
    ``VantaDB`` (``forward_to_db!`` macro) — same signature, same result.
    """

    def put(
        self,
        namespace: str,
        key: str,
        payload: str,
        metadata: dict | None = None,
        vector: Any | None = None,
        ttl_ms: int | None = None,
    ) -> VantaMemoryRecord: ...
    def put_batch(
        self,
        keys: list[str],
        vectors: list[list[float]],
        payloads: list[str] | None = None,
        metadatas: list[dict | None] | None = None,
        namespace: str | None = None,
        namespaces: list[str] | None = None,
        ttls: list[int | None] | None = None,
    ) -> list[VantaMemoryRecord]: ...
    def put_batch_raw(
        self,
        vectors: Any,
        keys: list[str],
        payloads: list[str] | None = None,
        metadatas: list[dict | None] | None = None,
        namespaces: list[str] | None = None,
        ttls: list[int | None] | None = None,
    ) -> list[VantaMemoryRecord]: ...
    def get_memory(self, namespace: str, key: str) -> VantaMemoryRecord | None: ...
    def delete_memory(self, namespace: str, key: str) -> bool: ...
    def delete_by_filter(self, namespace: str, filters: dict) -> int: ...
    def count(self, namespace: str, filters: dict | None = None) -> int: ...
    def similar_to_key(
        self, namespace: str, key: str, top_k: int = 10
    ) -> list[VantaSearchHit]: ...
    def list_memory(
        self,
        namespace: str,
        filters: dict | None = None,
        limit: int = 100,
        cursor: int | None = None,
        exclude_superseded: bool = False,
    ) -> VantaListResult: ...
    def search_memory(
        self,
        namespace: str,
        query_vector: Any,
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
        method: str | None = None,
        explain: bool = False,
        exclude_superseded: bool = False,
    ) -> list[VantaSearchHit]: ...
    def search(self, vector: Any, top_k: int = 10) -> list[tuple[int, float]]: ...
    def search_batch(
        self, vectors: list[Any], top_k: int = 10
    ) -> list[list[tuple[int, float]]]: ...
    def search_batch_requests(
        self, requests: list[Any], top_k: int = 10
    ) -> list[list[VantaSearchHit]]: ...
    def explain_memory_search(
        self,
        namespace: str,
        query_vector: list[float],
        filters: dict | None = None,
        text_query: str | None = None,
        top_k: int = 10,
        distance_metric: str | None = None,
    ) -> dict: ...
    def supersede(self, namespace: str, old_key: str, new_key: str) -> None: ...
    def generate_snippet(
        self,
        payload: str,
        text_query: str,
        with_highlighting: bool = False,
    ) -> str | None: ...
    def purge_expired(self) -> int: ...
    def list_namespaces(self) -> list[str]: ...


class GraphClient:
    """Grouped view over ``VantaDB`` graph methods (``db.graph.*``).

    Naming note: ``insert``/``get``/``delete`` are NODE-level ops
    (``id: u128``) in Python.
    """

    def insert(
        self, id: int, content: str, vector: Any, fields: dict | None = None
    ) -> None: ...
    def get(self, id: int) -> dict | None: ...
    def delete(self, id: int, reason: str = "manual deletion") -> None: ...
    def add_edge(
        self,
        source_id: int,
        target_id: int,
        label: str,
        weight: float | None = None,
        created_at_ms: int | None = None,
    ) -> None: ...
    def graph_bfs(
        self, roots: list[int], max_depth: int = 999999, direction: str = "Forward"
    ) -> list[int]: ...
    def graph_bfs_filtered(
        self,
        roots: list[int],
        max_depth: int = 999999,
        direction: str = "Forward",
        labels: list[int] | None = None,
        time_range: tuple[int, int] | None = None,
    ) -> list[int]: ...
    def graph_dfs(
        self, roots: list[int], max_depth: int = 999999, direction: str = "Forward"
    ) -> list[int]: ...
    def graph_topological_sort(self, roots: list[int]) -> list[int]: ...
    def graph_is_dag(self, roots: list[int]) -> bool: ...
    def graph_page_rank(
        self,
        roots: list[int],
        max_iterations: int = 100,
        damping: float = 0.85,
        tolerance: float = 1e-6,
    ) -> dict[int, float]: ...
    def graph_degree_centrality(
        self, roots: list[int]
    ) -> dict[int, tuple[int, int]]: ...


class SystemClient:
    """Catch-all grouped view over ``VantaDB`` system methods (``db.system.*``)."""

    def capabilities(self) -> dict: ...
    def hardware_profile(self) -> dict: ...
    def operational_metrics(self) -> dict: ...
    def query(self, iql_query: str) -> str: ...
    def query_structured(self, iql_query: str) -> dict: ...
    def flush(self) -> None: ...
    def compact_wal(self) -> None: ...
    def compact_layout(self) -> int: ...
    def rebuild_index(self) -> dict: ...
    def reindex_hnsw_from_text(self, namespace: str, page_size: int = 1000) -> dict: ...
    def repair_text_index(self) -> dict: ...
    def audit_text_index(self, namespace: str | None = None, deep: bool = False) -> dict: ...
    def export_namespace(self, path: str, namespace: str) -> dict: ...
    def export_all(self, path: str) -> dict: ...
    def import_file(self, path: str) -> dict: ...
    def bulk_import(self, path: str) -> dict: ...
    def bulk_import_bytes(self, data: bytes) -> dict: ...
    def close(self) -> None: ...


class WikiClient:
    """Wiki summary-archive recovery (``db.wiki.*``)."""

    def recover_archived_nodes(self, summary_id: str) -> list[Any]: ...


def connect(
    path: str,
    memory_limit: int | None = None,
    read_only: bool = False,
    backend: str | None = None,
) -> VantaDB: ...

__version__: str