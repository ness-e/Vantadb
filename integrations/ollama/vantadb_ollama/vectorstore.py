from __future__ import annotations

from typing import List, Optional

import ollama

from vantadb_shared import Document, EmbeddingVectorStore

DEFAULT_NAMESPACE = "ollama"
DEFAULT_MODEL = "nomic-embed-text"

__all__ = ["VantaDBOllama", "Document", "DEFAULT_NAMESPACE", "DEFAULT_MODEL"]


class VantaDBOllama(EmbeddingVectorStore):
    """VantaDB store with Ollama embeddings (thin wrapper over vantadb_shared)."""

    def __init__(
        self,
        model: str = DEFAULT_MODEL,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
    ):
        """Initialize a VantaDB store with Ollama embeddings.

        Args:
            model: Ollama embedding model name.
                Defaults to ``"nomic-embed-text"``.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"ollama"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
        """
        self.model = model
        super().__init__(
            namespace=namespace,
            db_path=db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
        )

    def _embed(self, text: str) -> List[float]:
        if not text:
            raise ValueError("text must be a non-empty string")
        resp = ollama.embeddings(model=self.model, prompt=text)
        return list(resp["embedding"])

    def _embed_many(self, texts: List[str]) -> List[List[float]]:
        if not texts:
            return []
        resp = ollama.embed(model=self.model, input=texts)
        return [list(e) for e in resp["embeddings"]]
