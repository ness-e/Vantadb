from importlib.metadata import version as _version

from vantadb_langchain.vectorstore import VantaDBVectorStore

try:
    __version__ = _version("vantadb-langchain")
except Exception:
    __version__ = "0.0.0"

__all__ = ["VantaDBVectorStore", "__version__"]
