from __future__ import annotations

from typing import Any, List, Optional

import openai

from vantadb_shared import Document, EmbeddingVectorStore

DEFAULT_NAMESPACE = "openai"
DEFAULT_MODEL = "text-embedding-3-small"

__all__ = ["VantaDBOpenAI", "Document", "DEFAULT_NAMESPACE", "DEFAULT_MODEL"]


class VantaDBOpenAI(EmbeddingVectorStore):
    """VantaDB store with OpenAI embeddings (thin wrapper over vantadb_shared)."""

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        client: Any = None,
    ):
        """Initialize a VantaDB store with OpenAI embeddings.

        Args:
            api_key: OpenAI API key used to create the OpenAI client.
            model: OpenAI embedding model name.
                Defaults to ``"text-embedding-3-small"``.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"openai"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            client: Optional pre-configured OpenAI client. If omitted,
                one is created from ``api_key``.
        """
        self.model = model
        self._client = client if client is not None else openai.OpenAI(api_key=api_key)
        super().__init__(
            namespace=namespace,
            db_path=db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
        )

    def _embed(self, text: str) -> List[float]:
        if not text:
            raise ValueError("text must be a non-empty string")
        resp = self._client.embeddings.create(model=self.model, input=[text])
        return list(resp.data[0].embedding)

    def _embed_many(self, texts: List[str]) -> List[List[float]]:
        if not texts:
            return []
        resp = self._client.embeddings.create(model=self.model, input=texts)
        return [list(d.embedding) for d in resp.data]
