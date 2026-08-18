"""VantaDB — canonical import name.

This package is a thin alias for the compiled ``vantadb_py`` bindings so that
``import vantadb`` works out of the box, matching the Rust crate and npm
package names. ``import vantadb_py`` remains fully supported and unchanged.
"""

from vantadb_py import *  # noqa: F401,F403 — re-export the public API
from vantadb_py import __version__  # noqa: F401

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