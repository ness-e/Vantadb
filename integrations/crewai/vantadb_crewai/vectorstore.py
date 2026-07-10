from __future__ import annotations

from typing import Any, Optional

import vantadb_py as vanta

DEFAULT_NAMESPACE = "crewai"


class VantaDBTool:
    def __init__(
        self,
        name: str = "VantaDB Search",
        description: str = "Search documents stored in VantaDB",
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        self.name = name
        self.description = description
        self.namespace = namespace
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    def _run(self, query: str, **kwargs: Any) -> str:
        results = self._db.list_memory(self.namespace, limit=10)
        hits = []
        for rec in results.records:
            if query.lower() in rec.payload.lower():
                hits.append(rec.payload)
        return "\n".join(hits[:5]) if hits else "No results found."

    def _put(self, text: str, metadata: Optional[dict] = None) -> None:
        import uuid
        self._db.put(self.namespace, str(uuid.uuid4()), text, metadata=metadata or {})

    def categorize(self, text: str) -> str:
        if not text.strip():
            return "empty"
        return "informational"

    def __call__(self, *args: Any, **kwargs: Any) -> str:
        return self._run(*args, **kwargs)
