"""Type stubs for the ``vantadb`` canonical import package.

FIND-582 M-1: declares the full re-export surface the runtime provides
(``vantadb/__init__.py`` does ``from vantadb_py import *``), including the
11-class typed error hierarchy, ``error_to_dict`` and ``memory_node_id`` —
previously undeclared, so ``from vantadb import NotFoundError`` failed
type-checking despite working at runtime.
"""

from vantadb_py import (
    AsyncClient,
    BusyError,
    Client,
    ConflictError,
    CorruptError,
    Error,
    ListResult,
    NoVectorError,
    NotFoundError,
    Record,
    ResourceLimitError,
    SearchHit,
    SearchRequest,
    StorageError,
    TimeoutError,
    UnsupportedError,
    ValidationError,
    Vector,
    __version__,
    connect,
    error_to_dict,
    memory_node_id,
)

__all__ = [
    "AsyncClient",
    "BusyError",
    "Client",
    "ConflictError",
    "CorruptError",
    "Error",
    "ListResult",
    "NoVectorError",
    "NotFoundError",
    "Record",
    "ResourceLimitError",
    "SearchHit",
    "SearchRequest",
    "StorageError",
    "TimeoutError",
    "UnsupportedError",
    "ValidationError",
    "Vector",
    "__version__",
    "connect",
    "error_to_dict",
    "memory_node_id",
]
