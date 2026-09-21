"""VantaDB storage backend for CrewAI unified Memory (INTG-02).

Implements the ``StorageBackend`` protocol (``crewai.memory.storage.backend``):
``save/search/delete/update/get_record/list_records/get_scope_info/list_scopes/
list_categories/count/reset`` + async ``asave/asearch/adelete``.

Mapping (mini-spec en ``docs/tasks/INTG-02.md``): un solo namespace VantaDB por
backend; ``MemoryRecord`` <-> payload=content, key=record.id, sistema en
metadata ``__mem_*``, vector=embedding. Scopes ``/a/b`` con prefix matching en
Python. Sin crewai instalado opera duck-typed (shim local ``ScopeInfo``).
"""
from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Optional

import vantadb_py as vanta

try:
    from crewai.memory.storage.backend import StorageBackend as _Protocol
except ImportError:
    _Protocol = object  # type: ignore[assignment,misc]

try:
    from crewai.memory.types import ScopeInfo as _ScopeInfo
except ImportError:
    _ScopeInfo = None  # type: ignore[assignment]

DEFAULT_NAMESPACE = "crewai_memory"

# Reserved metadata keys for MemoryRecord system fields (user metadata intact).
K_SCOPE = "__mem_scope"
K_CATEGORIES = "__mem_categories"
K_IMPORTANCE = "__mem_importance"
K_CREATED = "__mem_created_at"
K_ACCESSED = "__mem_last_accessed"
K_SOURCE = "__mem_source"
K_PRIVATE = "__mem_private"


@dataclass
class _ShimScopeInfo:
    """Fallback when crewai is not installed (same field names as ScopeInfo)."""

    path: str
    record_count: int = 0
    categories: list = field(default_factory=list)
    oldest_record: Optional[datetime] = None
    newest_record: Optional[datetime] = None
    child_scopes: list = field(default_factory=list)


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _parse_dt(value: Any) -> Optional[datetime]:
    if value is None:
        return None
    if isinstance(value, datetime):
        return value
    try:
        return datetime.fromisoformat(str(value))
    except (ValueError, TypeError):
        return None


class _Rec:
    """Duck-typed MemoryRecord view (works with crewai record or shim)."""

    __slots__ = ("id", "content", "scope", "categories", "metadata",
                 "importance", "created_at", "last_accessed", "embedding",
                 "source", "private")

    def __init__(self, **kw: Any):
        now = datetime.now(timezone.utc)
        self.id = kw.get("id")
        self.content = kw.get("content", "")
        self.scope = kw.get("scope") or "/"
        self.categories = list(kw.get("categories") or [])
        self.metadata = dict(kw.get("metadata") or {})
        self.importance = float(kw.get("importance", 0.5))
        self.created_at = kw.get("created_at") or now
        self.last_accessed = kw.get("last_accessed") or now
        self.embedding = kw.get("embedding")
        self.source = kw.get("source")
        self.private = bool(kw.get("private", False))


class VantaDBMemoryBackend(_Protocol):
    """``StorageBackend`` backed by embedded VantaDB (local-first, sin servidor)."""

    def __init__(
        self,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        self.db_path = db_path
        self.namespace = namespace
        self._db = vanta.Client(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    # -- mapping helpers ---------------------------------------------------
    @staticmethod
    def _to_meta(record: Any) -> dict:
        meta = dict(getattr(record, "metadata", None) or {})
        meta[K_SCOPE] = getattr(record, "scope", None) or "/"
        meta[K_CATEGORIES] = list(getattr(record, "categories", None) or [])
        meta[K_IMPORTANCE] = float(getattr(record, "importance", 0.5))
        created = getattr(record, "created_at", None)
        meta[K_CREATED] = created.isoformat() if isinstance(created, datetime) else _now_iso()
        accessed = getattr(record, "last_accessed", None)
        meta[K_ACCESSED] = accessed.isoformat() if isinstance(accessed, datetime) else _now_iso()
        if getattr(record, "source", None) is not None:
            meta[K_SOURCE] = record.source
        meta[K_PRIVATE] = bool(getattr(record, "private", False))
        return meta

    @staticmethod
    def _from_stored(key: str, payload: str, meta: dict, vector: Any) -> _Rec:
        user_meta = {k: v for k, v in meta.items() if not k.startswith("__mem_")}
        return _Rec(
            id=key,
            content=payload,
            scope=meta.get(K_SCOPE, "/"),
            categories=meta.get(K_CATEGORIES, []),
            metadata=user_meta,
            importance=meta.get(K_IMPORTANCE, 0.5),
            created_at=_parse_dt(meta.get(K_CREATED)),
            last_accessed=_parse_dt(meta.get(K_ACCESSED)),
            embedding=vector,
            source=meta.get(K_SOURCE),
            private=meta.get(K_PRIVATE, False),
        )

    def _iter_all(self, scope_prefix: Optional[str] = None):
        """Yield stored records (newest last) filtered by scope prefix."""
        cursor = None
        while True:
            page = self._db.memory.list(namespace=self.namespace, limit=200, cursor=cursor)
            records = page.records if hasattr(page, "records") else list(page)
            if not records:
                break
            for r in records:
                meta = dict(getattr(r, "metadata", None) or {})
                scope = meta.get(K_SCOPE, "/")
                if scope_prefix and not (scope == scope_prefix or scope.startswith(scope_prefix.rstrip("/") + "/")):
                    continue
                vec = getattr(r, "vector", None)
                yield self._from_stored(r.key, r.payload, meta,
                                        list(vec) if vec is not None else None)
            cursor = getattr(page, "next_cursor", None)
            if cursor is None:
                break

    # -- core protocol -----------------------------------------------------
    def save(self, records: list) -> None:
        for record in records:
            self._db.put(
                self.namespace,
                str(record.id),
                record.content,
                metadata=self._to_meta(record),
                vector=list(record.embedding) if record.embedding is not None else None,
            )

    def get_record(self, record_id: str) -> Optional[_Rec]:
        try:
            r = self._db.memory.get(self.namespace, record_id)
        except Exception:
            return None
        if r is None:
            return None
        meta = dict(getattr(r, "metadata", None) or {})
        vec = getattr(r, "vector", None)
        return self._from_stored(r.key, r.payload, meta,
                                 list(vec) if vec is not None else None)

    def update(self, record: Any) -> None:
        self.save([record])  # put() es upsert nativo

    def search(
        self,
        query_embedding: list,
        scope_prefix: Optional[str] = None,
        categories: Optional[list] = None,
        metadata_filter: Optional[dict] = None,
        limit: int = 10,
        min_score: float = 0.0,
    ) -> list:
        if not query_embedding:
            return []
        fetch = limit * 2 + 10  # oversample: el filtro por scope/categoría recorta
        try:
            hits = self._db.memory.search(self.namespace, list(query_embedding), top_k=fetch)
            hits = list(hits)
        except Exception:
            return []
        cats = set(categories or [])
        out = []
        for h in hits:
            meta = dict(getattr(h, "metadata", None) or {})
            scope = meta.get(K_SCOPE, "/")
            if scope_prefix and not (scope == scope_prefix or scope.startswith(scope_prefix.rstrip("/") + "/")):
                continue
            rec_cats = meta.get(K_CATEGORIES, [])
            if cats and not (cats & set(rec_cats)):
                continue
            if metadata_filter and any(meta.get(k) != v for k, v in metadata_filter.items()):
                continue
            score = float(getattr(h, "score", 0.0) or 0.0)
            if score < min_score:
                continue
            vec = getattr(h, "vector", None)
            out.append((self._from_stored(h.key, h.payload, meta,
                                          list(vec) if vec is not None else None), score))
        out.sort(key=lambda t: t[1], reverse=True)
        return out[:limit]

    def delete(
        self,
        scope_prefix: Optional[str] = None,
        categories: Optional[list] = None,
        record_ids: Optional[list] = None,
        older_than: Optional[datetime] = None,
        metadata_filter: Optional[dict] = None,
    ) -> int:
        ids = set(record_ids or [])
        cats = set(categories or [])
        n = 0
        for rec in self._iter_all(scope_prefix):
            if ids and rec.id not in ids:
                continue
            if cats and not (cats & set(rec.categories)):
                continue
            if older_than and rec.created_at and rec.created_at >= older_than:
                continue
            if metadata_filter and any(rec.metadata.get(k) != v for k, v in metadata_filter.items()):
                continue
            self._db.memory.delete(self.namespace, rec.id)
            n += 1
        return n

    # -- discovery / maintenance -------------------------------------------
    def list_records(
        self,
        scope_prefix: Optional[str] = None,
        limit: int = 200,
        offset: int = 0,
    ) -> list:
        """Records newest-first (ordered by ``created_at`` desc)."""
        recs = sorted(
            self._iter_all(scope_prefix),
            key=lambda r: (r.created_at is not None, r.created_at),
            reverse=True,
        )
        return recs[offset:offset + limit]

    def get_scope_info(self, scope: str):
        recs = [r for r in self._iter_all(scope)]
        cats: list = []
        oldest = newest = None
        children: set = set()
        for r in recs:
            for c in r.categories:
                if c not in cats:
                    cats.append(c)
            if r.created_at and (oldest is None or r.created_at < oldest):
                oldest = r.created_at
            if r.created_at and (newest is None or r.created_at > newest):
                newest = r.created_at
            if r.scope != scope:
                parts = [p for p in r.scope.split("/") if p]
                base = [p for p in scope.split("/") if p]
                if parts[:len(base)] == base and len(parts) > len(base):
                    children.add("/" + "/".join(parts[:len(base) + 1]))
        cls = _ScopeInfo if _ScopeInfo is not None else _ShimScopeInfo
        return cls(
            path=scope,
            record_count=len(recs),
            categories=cats,
            oldest_record=oldest,
            newest_record=newest,
            child_scopes=sorted(children),
        )

    def list_scopes(self, parent: str = "/") -> list:
        children: set = set()
        base = [p for p in parent.split("/") if p]
        for r in self._iter_all():
            rp = [p for p in (r.scope or "/").split("/") if p]
            if rp[:len(base)] == base and len(rp) > len(base):
                children.add("/" + "/".join(rp[:len(base) + 1]))
        return sorted(children)

    def list_categories(self, scope_prefix: Optional[str] = None) -> dict:
        counts: dict = {}
        for r in self._iter_all(scope_prefix):
            for c in r.categories:
                counts[c] = counts.get(c, 0) + 1
        return counts

    def count(self, scope_prefix: Optional[str] = None) -> int:
        return sum(1 for _ in self._iter_all(scope_prefix))

    def reset(self, scope_prefix: Optional[str] = None) -> None:
        self.delete(scope_prefix=scope_prefix)

    # -- async (thin wrappers; VantaDB SDK es sync) -------------------------
    async def asave(self, records: list) -> None:
        self.save(records)

    async def asearch(self, query_embedding: list, **kw: Any) -> list:
        return self.search(query_embedding, **kw)

    async def adelete(self, **kw: Any) -> int:
        return self.delete(**kw)
