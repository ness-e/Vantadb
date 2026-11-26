from importlib.metadata import version as _version

from vantadb_langchain.vectorstore import VantaDBVectorStore

try:
    from vantadb_langchain.store import VantaDBStore
except ImportError:  # langgraph-checkpoint not installed — VectorStore-only env
    VantaDBStore = None  # type: ignore[assignment]

try:
    from vantadb_langchain.checkpointer import VantaDBCheckpointer
except ImportError:  # langgraph-checkpoint not installed — VectorStore-only env
    VantaDBCheckpointer = None  # type: ignore[assignment]

try:
    __version__ = _version("vantadb-langchain")
except Exception:
    __version__ = "0.0.0"

__all__ = [
    "VantaDBVectorStore",
    "VantaDBCheckpointer",
    "VantaDBStore",
    "__version__",
]
