"""VantaDB — canonical import name.

This package is a thin alias for the compiled ``vantadb_py`` bindings so that
``import vantadb`` works out of the box, matching the Rust crate and npm
package names. ``import vantadb_py`` remains available but deprecated
(see ``vantadb_py/__init__.py``, removal in 0.6.0).
"""

# PY-03: suppress the alias DeprecationWarning that vantadb_py/__init__.py
# emits — it must fire only for users importing vantadb_py directly.
import warnings

with warnings.catch_warnings():
    warnings.simplefilter("ignore", DeprecationWarning)
    from vantadb_py import *
    from vantadb_py import __version__

# FIND-582 M-1: the canonical package re-exports the full ``vantadb_py``
# surface — including the typed error hierarchy (11 classes), ``error_to_dict``
# and ``memory_node_id`` — so ``from vantadb import X`` matches what the
# runtime actually provides. Kept in sync with ``__init__.pyi``.
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
