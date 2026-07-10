from __future__ import annotations

import uuid
from typing import Any, Dict, List, Optional

import vantadb_py as vanta

DEFAULT_NAMESPACE = "haystack"


class VantaDBDocumentStore:
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

    def write_documents(
        self,
        documents: List[dict],
        policy: str = "overwrite",
        **kwargs: Any,
    ) -> None:
        for doc in documents:
            doc_id = doc.get("id", str(uuid.uuid4()))
            content = doc.get("content", doc.get("text", ""))
            meta = dict(doc.get("meta", {}))
            self._db.put(self.namespace, doc_id, content, metadata=meta)

    def filter_documents(
        self,
        filters: Optional[Dict[str, Any]] = None,
        **kwargs: Any,
    ) -> List[Any]:
        results = self._db.list_memory(
            self.namespace,
            filters=filters or {},
            limit=kwargs.get("limit", 1000),
        )
        return [
            type("Document", (), {
                "id": r.key,
                "content": r.payload,
                "meta": dict(r.metadata),
            })()
            for r in results.records
        ]

    def delete_documents(
        self,
        filters: Optional[Dict[str, Any]] = None,
        **kwargs: Any,
    ) -> None:
        results = self._db.list_memory(
            self.namespace,
            filters=filters or {},
            limit=10000,
        )
        for rec in results.records:
            if rec.key:
                self._db.delete_memory(self.namespace, rec.key)

    def count_documents(
        self,
        filters: Optional[Dict[str, Any]] = None,
    ) -> int:
        results = self._db.list_memory(
            self.namespace,
            filters=filters or {},
            limit=1,
        )
        return len(self._db.list_memory(self.namespace, limit=10000).records)
