from __future__ import annotations

import uuid
from typing import Any, List, Optional

import ollama
import vantadb_py as vanta

DEFAULT_NAMESPACE = "ollama"
DEFAULT_TOP_K = 4
DEFAULT_MODEL = "nomic-embed-text"


class VantaDBOllama:
    def __init__(
        self,
        model: str = DEFAULT_MODEL,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
    ):
        self.model = model
        self.namespace = namespace
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
        )

    def _embed(self, text: str) -> List[float]:
        resp = ollama.embeddings(model=self.model, prompt=text)
        return list(resp["embedding"])

    def _embed_many(self, texts: List[str]) -> List[List[float]]:
        return [self._embed(t) for t in texts]

    def add_texts(
        self,
        texts: List[str],
        metadatas: Optional[List[dict]] = None,
        ids: Optional[List[str]] = None,
    ) -> List[str]:
        vectors = self._embed_many(texts)
        result_ids: List[str] = []
        for i, text in enumerate(texts):
            key = ids[i] if ids else str(uuid.uuid4())
            meta = metadatas[i] if metadatas else {}
            self._db.put(self.namespace, key, text, metadata=meta, vector=vectors[i])
            result_ids.append(key)
        return result_ids

    def similarity_search(self, query: str, k: int = DEFAULT_TOP_K) -> List[Any]:
        results = self._db.list_memory(self.namespace, limit=k)
        hits = []
        for rec in results.records:
            if query.lower() in rec.payload.lower():
                hits.append(type("Document", (), {
                    "page_content": rec.payload,
                    "metadata": dict(rec.metadata),
                })())
        return hits[:k]

    def delete(self, ids: Optional[List[str]] = None) -> bool:
        if ids is None:
            return True
        for key in ids:
            self._db.delete_memory(self.namespace, key)
        return True
