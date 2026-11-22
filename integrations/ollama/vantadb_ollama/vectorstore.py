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

    @staticmethod
    def _extract_embeddings(resp: Any, *, plural: bool) -> Any:
        attr = "embeddings" if plural else "embedding"
        if hasattr(resp, attr):
            return getattr(resp, attr)
        return resp[attr]

    def _embed(self, text: str) -> List[float]:
        if not text:
            raise ValueError("text must be a non-empty string")
        return self._embed_many([text])[0]

    def _embed_many(self, texts: List[str]) -> List[List[float]]:
        if not texts:
            return []
        if hasattr(ollama, "embed"):
            resp = ollama.embed(model=self.model, input=texts)
            return [list(e) for e in self._extract_embeddings(resp, plural=True)]
        return [
            list(self._extract_embeddings(ollama.embeddings(model=self.model, prompt=t), plural=False))
            for t in texts
        ]
