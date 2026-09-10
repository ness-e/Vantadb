"""LangGraph ``BaseStore`` adapter backed by VantaDB (INTG-01).

Hierarchical LangGraph namespaces (tuples of strings) map to flat VantaDB
namespaces by joining parts with ``"/"`` — prefix search and
``list_namespaces`` reconstruct the hierarchy by splitting on ``"/"``.
Namespace parts must be non-empty strings without ``"/"``.

Sources:
- https://docs.langchain.com/oss/python/langgraph/stores ("Build a custom store")
- ``langgraph.store.base`` (``langgraph-checkpoint>=2,<5`` — pinned in pyproject)
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from typing import Any, Iterable, Optional

import vantadb_py as vanta
from langgraph.store.base import (
    BaseStore,
    GetOp,
    InvalidNamespaceError,
    Item,
    ListNamespacesOp,
    PutOp,
    SearchItem,
    SearchOp,
    tokenize_path,
)

_SEP = "/"


def _ms_to_iso(ms: int) -> str:
    return datetime.fromtimestamp(ms / 1000, tz=timezone.utc).isoformat()


def encode_namespace(namespace: tuple[str, ...] | list[str]) -> str:
    """Map a hierarchical namespace tuple to a flat VantaDB namespace."""
    parts = tuple(namespace)
    for part in parts:
        if not isinstance(part, str) or not part or _SEP in part:
            raise InvalidNamespaceError(
                f"namespace parts must be non-empty strings without { _SEP!r}: {parts!r}"
            )
    return _SEP.join(parts)


def decode_namespace(ns: str) -> tuple[str, ...]:
    """Split a flat VantaDB namespace back into a hierarchy tuple."""
    return tuple(ns.split(_SEP)) if ns else ()


def _norm_path(path: Any) -> tuple[str, ...]:
    if isinstance(path, str):
        return tuple(tokenize_path(path))
    return tuple(path)


def _matches_prefix(ns: tuple[str, ...], prefix: tuple[str, ...]) -> bool:
    return ns[: len(prefix)] == prefix


class VantaDBStore(BaseStore):
    """Persistent LangGraph ``BaseStore`` KV backed by embedded VantaDB.

    Args:
        db_path: Filesystem path for the VantaDB database.
        embeddings: Optional LangChain ``Embeddings`` used only when
            ``search`` receives a ``query`` (semantic search via
            VantaDB ``search_memory``). Without it, ``query`` raises
            ``ValueError``.
        namespace: VantaDB namespace is derived per-operation from the
            LangGraph namespace tuple (see module docstring).
    """

    def __init__(
        self,
        db_path: str = "./vantadb_data",
        *,
        embeddings: Any = None,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        self._embeddings = embeddings
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    # ── batch core (BaseStore abstract surface) ──────────────

    def batch(self, ops: Iterable[Any]) -> list[Any]:
        results: list[Any] = []
        for op in ops:
            if isinstance(op, GetOp):
                results.append(self._get(op.namespace, op.key))
            elif isinstance(op, PutOp):
                if op.value is None:
                    self._delete(op.namespace, op.key)
                    results.append(None)
                else:
                    if op.ttl is not None:
                        raise NotImplementedError(
                            "TTL is not supported by VantaDBStore"
                        )
                    self._put(op.namespace, op.key, op.value)
                    results.append(None)
            elif isinstance(op, SearchOp):
                results.append(
                    self._search(
                        op.namespace_prefix,
                        filter=op.filter,
                        limit=op.limit,
                        offset=op.offset,
                        query=op.query,
                    )
                )
            elif isinstance(op, ListNamespacesOp):
                results.append(
                    self._list_namespaces(
                        op.match_conditions,
                        max_depth=op.max_depth,
                        limit=op.limit,
                        offset=op.offset,
                    )
                )
            else:  # pragma: no cover — defensive
                raise ValueError(f"unsupported op: {type(op).__name__}")
        return results

    async def abatch(self, ops: Iterable[Any]) -> list[Any]:
        # VantaDB core is synchronous embedded — same pattern as
        # langgraph InMemorySaver (sync core + thin async wrappers).
        return self.batch(ops)

    # ── primitives ───────────────────────────────────────────

    def _put(self, namespace: Any, key: str, value: dict[str, Any]) -> None:
        if not isinstance(value, dict):
            raise TypeError(f"value must be a dict, got {type(value).__name__}")
        ns = encode_namespace(tuple(namespace))
        if not ns:
            raise InvalidNamespaceError("namespace must not be empty for put")
        payload = json.dumps(value)  # TypeError if not JSON-serializable
        # Mirror scalar value fields into VantaDB metadata so list_memory
        # filters implement BaseStore filter semantics (exact match on
        # value keys). Nested dicts/lists are payload-only.
        metadata = {
            k: v for k, v in value.items() if isinstance(v, (str, int, float, bool))
        }
        vector = None
        if self._embeddings is not None:
            vector = self._embeddings.embed_documents([payload])[0]
        self._db.put(ns, str(key), payload, metadata=metadata, vector=vector)

    def _get(self, namespace: Any, key: str) -> Optional[Item]:
        ns = encode_namespace(tuple(namespace))
        if not ns:
            return None
        record = self._db.get_memory(ns, str(key))
        if record is None:
            return None
        return self._to_item(tuple(namespace), record)

    def _delete(self, namespace: Any, key: str) -> None:
        ns = encode_namespace(tuple(namespace))
        if not ns:
            return
        self._db.delete_memory(ns, str(key))

    @staticmethod
    def _to_item(ns_tuple: tuple[str, ...], record: Any) -> Item:
        return Item(
            namespace=ns_tuple,
            key=record.key,
            value=json.loads(record.payload),
            created_at=_ms_to_iso(record.created_at_ms),
            updated_at=_ms_to_iso(record.updated_at_ms),
        )

    def _all_namespaces(self) -> list[tuple[str, ...]]:
        return [decode_namespace(ns) for ns in self._db.list_namespaces()]

    def _search(
        self,
        namespace_prefix: Any,
        *,
        filter: Optional[dict[str, Any]] = None,
        limit: int = 10,
        offset: int = 0,
        query: Optional[str] = None,
    ) -> list[SearchItem]:
        prefix = tuple(namespace_prefix)
        namespaces = [
            ns for ns in self._all_namespaces() if _matches_prefix(ns, prefix)
        ]
        if query is not None:
            return self._search_semantic(namespaces, query, filter, limit, offset)
        items: list[SearchItem] = []
        for ns in sorted(namespaces):
            ns_str = encode_namespace(ns)
            cursor: Optional[int] = None
            while True:
                page = self._db.list_memory(
                    ns_str, filters=filter, limit=1000, cursor=cursor
                )
                if not page or not page.records:
                    break
                for rec in page.records:
                    item = self._to_item(ns, rec)
                    items.append(
                        SearchItem(
                            namespace=item.namespace,
                            key=item.key,
                            value=item.value,
                            created_at=item.created_at,
                            updated_at=item.updated_at,
                            score=None,
                        )
                    )
                cursor = page.next_cursor
                if cursor is None:
                    break
        return items[offset : offset + limit]

    def _search_semantic(
        self,
        namespaces: list[tuple[str, ...]],
        query: str,
        filter: Optional[dict[str, Any]],
        limit: int,
        offset: int,
    ) -> list[SearchItem]:
        if self._embeddings is None:
            raise ValueError(
                "semantic search needs `embeddings` (VantaDBStore(embeddings=...))"
            )
        qvec = self._embeddings.embed_query(query)
        scored: list[tuple[float, SearchItem]] = []
        for ns in sorted(namespaces):
            hits = self._db.search_memory(
                encode_namespace(ns),
                qvec,
                filters=filter,
                top_k=limit + offset,
                distance_metric="cosine",
            )
            for hit in hits:
                # VantaDB score = cosine distance (lower = closer);
                # LangGraph SearchItem.score ranks higher = more similar.
                score = 1.0 - hit.score / 2.0
                scored.append(
                    (
                        score,
                        SearchItem(
                            namespace=ns,
                            key=hit.key,
                            value=json.loads(hit.payload),
                            created_at=_ms_to_iso(hit.created_at_ms),
                            updated_at=_ms_to_iso(hit.updated_at_ms),
                            score=score,
                        ),
                    )
                )
        scored.sort(key=lambda t: t[0], reverse=True)
        return [item for _, item in scored[offset : offset + limit]]

    def _list_namespaces(
        self,
        match_conditions: Any,
        *,
        max_depth: Optional[int] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[tuple[str, ...]]:
        namespaces = self._all_namespaces()
        for cond in match_conditions or ():
            path = _norm_path(cond.path)
            if cond.match_type == "prefix":
                namespaces = [ns for ns in namespaces if _matches_prefix(ns, path)]
            elif cond.match_type == "suffix":
                namespaces = [
                    ns for ns in namespaces if ns[len(ns) - len(path) :] == path
                ] if path else namespaces
            else:  # pragma: no cover — defensive
                raise ValueError(f"unknown match_type: {cond.match_type}")
        if max_depth is not None:
            namespaces = sorted({ns[:max_depth] for ns in namespaces})
        else:
            namespaces = sorted(namespaces)
        return namespaces[offset : offset + limit]
