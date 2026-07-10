from __future__ import annotations

import uuid
from typing import Any, List, Optional

import vantadb_py as vanta

DEFAULT_NAMESPACE = "mem0"
DEFAULT_TOP_K = 4


class VantaDBVectorStore:
    def __init__(
        self,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        self.namespace = namespace
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    def add(
        self,
        text: str,
        user_id: Optional[str] = None,
        metadata: Optional[dict] = None,
        **kwargs: Any,
    ) -> str:
        key = str(uuid.uuid4())
        meta = dict(metadata or {})
        if user_id:
            meta["user_id"] = user_id
        self._db.put(self.namespace, key, text, metadata=meta)
        return key

    def search(self, query: str, k: int = DEFAULT_TOP_K, **kwargs: Any) -> List[dict]:
        results = self._db.list_memory(self.namespace, limit=k)
        hits = []
        for rec in results.records:
            if query.lower() in rec.payload.lower():
                hits.append({
                    "key": rec.key,
                    "payload": rec.payload,
                    "metadata": dict(rec.metadata),
                    "score": 1.0,
                })
        return hits[:k]

    def delete(self, key: str) -> bool:
        return self._db.delete_memory(self.namespace, key)

    def list(self, user_id: Optional[str] = None, limit: int = 100) -> List[dict]:
        filters = {"user_id": user_id} if user_id else None
        results = self._db.list_memory(self.namespace, filters=filters, limit=limit)
        return [
            {"key": r.key, "payload": r.payload, "metadata": dict(r.metadata)}
            for r in results.records
        ]
