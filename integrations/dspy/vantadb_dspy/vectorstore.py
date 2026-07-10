from __future__ import annotations

from typing import Any, List, Optional

import vantadb_py as vanta

DEFAULT_NAMESPACE = "dspy"
DEFAULT_TOP_K = 4


class VantaDBRetriever:
    def __init__(
        self,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        k: int = DEFAULT_TOP_K,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        self.k = k
        self.namespace = namespace
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    def forward(self, query: str, **kwargs: Any) -> List[str]:
        results = self._db.list_memory(self.namespace, limit=self.k)
        hits = []
        for rec in results.records:
            if query.lower() in rec.payload.lower():
                hits.append(rec.payload)
        return hits[:self.k]

    def __call__(self, query: str, **kwargs: Any) -> List[str]:
        return self.forward(query, **kwargs)

    def _add(self, text: str, key: str) -> None:
        self._db.put(self.namespace, key, text)
