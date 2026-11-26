from __future__ import annotations

import json
import logging
import math
import uuid
from typing import Any, Dict, List, Optional, Tuple, Callable

import vantadb_py as vanta
from mem0.vector_stores.base import VectorStoreBase

logger = logging.getLogger(__name__)

DEFAULT_NAMESPACE = "mem0"
DEFAULT_TOP_K = 5

# FIND-579: candidate over-fetch factor used when a mem0 `filters` dict
# contains operators that cannot be pushed down to the engine (they are
# evaluated in Python instead). Bounded so a filtered search never returns
# fewer than `top_k` hits unless the corpus genuinely lacks matches.
_POST_FILTER_OVERFETCH = 4
_POST_FILTER_MIN_CANDIDATES = 25

# M-8: hard bound for `delete_col()` — each iteration deletes up to one page
# of 1000 records; the cap only triggers on a pathological loop (records
# re-created faster than deleted, or a backend that never reports progress).
_DELETE_COL_MAX_ITERATIONS = 10_000

# Filter operators evaluated in the Python post-filter (mirrors the wire
# format `py_dict_to_filter_ops` accepts in the SDK: $eq/$neq/$gt/$gte/$lt/$lte,
# plus mem0-style $in/$nin for parity with other vector stores).
_KNOWN_OPS = frozenset({"$eq", "$neq", "$ne", "$gt", "$gte", "$lt", "$lte", "$in", "$nin"})
_LOGICAL_KEY_MAP = {
    "AND": "$and",
    "OR": "$or",
    "NOT": "$not",
    "$and": "$and",
    "$or": "$or",
    "$not": "$not",
}


class OutputData:
    """Lightweight search/query result — id, score, payload.

    Matches the shape mem0's Memory layer expects from vector stores:
    ``.id`` (str), ``.score`` (float in [0,1], higher=better), ``.payload`` (dict).
    """

    __slots__ = ("id", "score", "payload")

    def __init__(
        self,
        id: Optional[str] = None,
        score: Optional[float] = None,
        payload: Optional[Dict[str, Any]] = None,
    ):
        """Initialize a lightweight search result container.

        Args:
            id: Optional record identifier.
            score: Optional relevance score in [0, 1].
            payload: Optional dictionary of result data.
                Defaults to an empty dict.
        """
        self.id = id
        self.score = score
        self.payload = payload or {}

    def __repr__(self) -> str:
        return f"OutputData(id={self.id!r}, score={self.score!r})"


def _normalize_score(raw: Optional[float]) -> float:
    """Map a raw VantaDB score to a similarity in [0, 1].

    FIND-94: backend emits cosine similarity (higher = better,
    identical -> 1.0), so values in ``[0, 1]`` pass through unchanged.
    The ``1 - d`` branch below only handles legacy distance inputs
    (>1); it never triggers with the current backend and is kept
    for backward-compatible callers (pinned by
    ``test_normalize_score_exact_semantics``).

    Exact semantics (pinned by ``test_normalize_score_exact_semantics``):

    - ``None`` -> ``0.0``.
    - Value already in ``[0, 1]`` -> returned unchanged (treated as a
      pre-normalized similarity; VantaDB cosine never emits these, but
      callers may pass scores from other sources).
    - Any other value is treated as a distance ``d`` and inverted:
      ``score = clamp(1 - d, 0, 1)``. Cosine distance lives in ``[0, 2]``
      where 0 = identical, so ``d = 0`` -> ``1.0`` and ``d >= 1`` -> ``0.0``.
      Invalid negatives are clamped to ``0.0``, never above ``1.0``.
    """
    if raw is None:
        return 0.0
    if 0.0 <= raw <= 1.0:
        return float(raw)
    if raw < 0.0:
        return 0.0
    return min(1.0, max(0.0, 1.0 - raw))


def _build_payload(metadata, payload_text: str = "") -> Dict[str, Any]:
    """Merge VantaDB record fields into a single payload dict."""
    d = dict(metadata) if metadata else {}
    if payload_text:
        d["payload"] = payload_text
    return d


# ──────────────────────────────────────────────────────────────────────
#  Filter translation (FIND-579)
# ──────────────────────────────────────────────────────────────────────


def _is_native_scalar(value: Any) -> bool:
    """True when ``value`` is a scalar supported by ``py_dict_to_metadata``."""
    if value is None or isinstance(value, (bool, int, str)):
        return True
    if isinstance(value, float):
        return math.isfinite(value)
    return False


def _translate_filters(
    filters: Optional[Dict[str, Any]],
) -> Tuple[Dict[str, Any], List[Tuple[str, str, Any]]]:
    """Split a mem0-style ``filters`` dict into engine push-down + residual ops.

    VantaDB's ``memory.search``/``memory.list`` ``filters`` parameter only
    accepts a flat ``{field: scalar}`` map (metadata equality). Nested
    ``{"$op": value}`` specs, logical combinators (``AND``/``OR``/``NOT``),
    and non-scalar values (lists, dicts, non-finite floats, objects) are
    routed to Python residual evaluation so they never trigger PyO3
    ``TypeError`` or silently match nothing.

    Returns ``(native_eq, residual)``:

    - ``native_eq``: flat ``{field: value}`` equality dict, safe to pass to
      the binding (``None``-equivalent when empty).
    - ``residual``: ``[(field, op, value)]`` evaluated in Python by
      :func:`_record_matches`. Scalar ``$eq`` goes push-down; every other
      operator, logical combinator, or non-scalar spec is residual. Unknown
      ``$op`` keys and malformed logical shapes fail closed (the record is
      excluded) — a filter that cannot be applied must never silently
      broaden the result set.
    """
    native: Dict[str, Any] = {}
    residual: List[Tuple[str, str, Any]] = []
    seen_logical: set[str] = set()
    for field, spec in (filters or {}).items():
        if field in _LOGICAL_KEY_MAP:
            norm_op = _LOGICAL_KEY_MAP[field]
            if norm_op not in seen_logical:
                seen_logical.add(norm_op)
                residual.append((field, norm_op, spec))
            continue
        if isinstance(spec, dict):
            if not spec:
                residual.append((field, "$eq", spec))
                continue
            for op, val in spec.items():
                if op == "$eq":
                    if _is_native_scalar(val):
                        native[field] = val
                    else:
                        residual.append((field, "$eq", val))
                elif op in _KNOWN_OPS:
                    residual.append((field, op, val))
                else:
                    logger.warning(
                        "vantadb_mem0: unsupported filter operator %r on field %r; "
                        "failing closed (hits excluded)",
                        op,
                        field,
                    )
                    residual.append((field, op, val))
        elif _is_native_scalar(spec):
            # Flat scalar -> implicit $eq, the canonical wire form.
            native[field] = spec
        else:
            residual.append((field, "$eq", spec))
    return native, residual


def _value_matches(actual: Any, op: str, expected: Any) -> bool:
    """Evaluate one (field, op, expected) triple against a stored value."""
    if op in ("$eq",):
        return actual == expected
    if op in ("$neq", "$ne"):
        return actual != expected
    if op == "$in":
        return isinstance(expected, (list, tuple, set)) and actual in expected
    if op == "$nin":
        return isinstance(expected, (list, tuple, set)) and actual not in expected
    try:
        if op == "$gt":
            return actual > expected
        if op == "$gte":
            return actual >= expected
        if op == "$lt":
            return actual < expected
        if op == "$lte":
            return actual <= expected
    except TypeError:
        # Incomparable types (e.g. str vs int) cannot satisfy the predicate.
        return False
    # Unknown operator: fail closed — never broaden a filtered query.
    return False


def _eval_subfilter(payload: Dict[str, Any], sub: Any) -> bool:
    """Evaluate a nested filter dict against ``payload`` (fail-closed)."""
    if not isinstance(sub, dict):
        return False
    sub_native, sub_residual = _translate_filters(sub)
    for field, expected in sub_native.items():
        if not _value_matches(payload.get(field), "$eq", expected):
            return False
    return _record_matches(payload, sub_residual)


def _eval_logical(payload: Dict[str, Any], op: str, spec: Any) -> bool:
    """Evaluate compound ``$and`` / ``$or`` / ``$not`` filter predicates."""
    if op == "$and":
        if not isinstance(spec, list) or not all(isinstance(item, dict) for item in spec):
            return False
        return all(_eval_subfilter(payload, item) for item in spec)
    if op == "$or":
        if not isinstance(spec, list) or not all(isinstance(item, dict) for item in spec):
            return False
        return any(_eval_subfilter(payload, item) for item in spec)
    if op == "$not":
        if isinstance(spec, dict):
            return not _eval_subfilter(payload, spec)
        if isinstance(spec, list) and all(isinstance(item, dict) for item in spec):
            return not any(_eval_subfilter(payload, item) for item in spec)
        return False
    return False


def _record_matches(payload: Dict[str, Any], residual: List[Tuple[str, str, Any]]) -> bool:
    """True when ``payload`` satisfies every residual filter predicate."""
    for field, op, expected in residual:
        if op in ("$and", "$or", "$not"):
            if not _eval_logical(payload, op, expected):
                return False
        elif not _value_matches(payload.get(field), op, expected):
            return False
    return True


# ──────────────────────────────────────────────────────────────────────
#  VectorStoreBase implementation
# ──────────────────────────────────────────────────────────────────────


class VantaDBVectorStore(VectorStoreBase):
    """Mem0 vector-store adapter backed by VantaDB.

    Usage (direct, out-of-band)::

        store = VantaDBVectorStore(db_path="/tmp/vdb", namespace="memories")
        store.create_col("memories", vector_size=384)
        store.insert([vector], payloads=[{"data": "hello"}], ids=["k1"])
        results = store.search("hello", vector, top_k=5)

    The constructor also accepts an optional ``embedding`` callable for
    backward-compatible convenience methods (``.add()``).
    """

    def __init__(
        self,
        embedding: Optional[Callable[[str], List[float]]] = None,
        *,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        """Initialize a Mem0 VectorStoreBase adapter backed by VantaDB.

        Args:
            embedding: Optional callable that converts text to an
                embedding vector list. Used by the convenience method
                ``.add()``.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace (collection) to operate on.
                Defaults to ``"mem0"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        self.embedding = embedding
        self.namespace = namespace
        self._db = vanta.Client(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    # ------------------------------------------------------------------
    #  VectorStoreBase abstract methods  (11 methods)
    # ------------------------------------------------------------------

    def create_col(self, name: str, vector_size: Optional[int] = None, distance: str = "cosine") -> None:
        """Create a collection (namespace in VantaDB).

        VantaDB is schemaless — namespaces are created lazily on first
        write.  Logged for observability; actual creation happens on
        first insert.
        """
        logger.info(
            f"create_col({name}): VantaDB auto-creates on first insert; "
            f"vector_size={vector_size}, distance={distance} ignored"
        )

    def insert(
        self,
        vectors: List[List[float]],
        payloads: Optional[List[Dict[str, Any]]] = None,
        ids: Optional[List[str]] = None,
    ) -> None:
        """Insert a batch of vectors with optional payloads and ids."""
        for i, vec in enumerate(vectors):
            key = ids[i] if ids else str(uuid.uuid4())
            p = payloads[i] if payloads else {}
            text = p.get("data") or p.get("text") or p.get("content") or json.dumps(p)
            self._db.put(
                self.namespace,
                key,
                text,
                metadata=dict(p),
                vector=vec,
            )

    def search(
        self,
        query: str,
        vectors: Optional[List[float]] = None,
        top_k: int = DEFAULT_TOP_K,
        filters: Optional[Dict[str, Any]] = None,
        *,
        text_query: Optional[str] = None,
    ) -> List[OutputData]:
        """Search by pre-computed vector, embedded query, or BM25 text.

        ``vectors`` is a dense query vector (or ``None`` when relying on
        ``self.embedding`` or BM25 lexical search). Mem0 normally supplies
        ``vectors`` externally; omitting it triggers ``self.embedding(query)``
        when configured, or falls back to core BM25 search via ``text_query``.

        ``filters`` (FIND-579): flat ``{field: value}`` pairs and ``$eq``
        specs are pushed down to the engine's metadata filter; any other
        operator spec is evaluated as an exact Python post-filter (with
        candidate over-fetch). A filter is never silently dropped — unknown
        operators fail closed.
        """
        if top_k <= 0:
            return []
        native_eq, residual = _translate_filters(filters)
        fetch_k = top_k
        if residual:
            fetch_k = max(top_k * _POST_FILTER_OVERFETCH, _POST_FILTER_MIN_CANDIDATES)
        query_vec = vectors
        if (query_vec is None or len(query_vec) == 0) and self.embedding is not None and query:
            query_vec = self.embedding(query)
        if query_vec is None or len(query_vec) == 0:
            lexical = text_query or (query.strip() if query else "")
            if not lexical:
                return []
            results = self._db.memory.search(
                self.namespace,
                None,
                top_k=fetch_k,
                text_query=lexical,
                filters=native_eq or None,
            )
        else:
            search_kwargs: Dict[str, Any] = {
                "top_k": fetch_k,
                "distance_metric": "cosine",
                "filters": native_eq or None,
            }
            if text_query:
                search_kwargs["text_query"] = text_query
            results = self._db.memory.search(
                self.namespace,
                query_vec,
                **search_kwargs,
            )
        out: List[OutputData] = []
        for hit in results:
            payload = _build_payload(hit.metadata, hit.payload)
            if residual and not _record_matches(payload, residual):
                continue
            out.append(
                OutputData(
                    id=hit.key,
                    score=_normalize_score(hit.score),
                    payload=payload,
                )
            )
            if len(out) >= top_k:
                break
        return out

    def delete(self, vector_id: str) -> None:
        """Delete a single record by its id."""
        self._db.memory.delete(self.namespace, vector_id)

    def update(
        self,
        vector_id: str,
        vector: Optional[List[float]] = None,
        payload: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Replace the vector and/or payload of an existing record.

        ``put()`` is native upsert: fetch the stored vector so a call
        without ``vector`` preserves it (FIND-94: no ``update_memory``).
        """
        existing = self.get(vector_id)
        if existing is None:
            raise ValueError(f"Record {vector_id} not found")
        cur = dict(existing.payload)
        if payload:
            cur.update(payload)
        rec = self._db.memory.get(self.namespace, vector_id)
        text = (
            cur.get("data")
            or cur.get("text")
            or cur.get("content")
            or cur.get("payload")
            or getattr(rec, "payload", "")
            or ""
        )
        old_vector: Optional[List[float]] = None
        if rec is not None and getattr(rec, "vector", None) is not None:
            try:
                old_vector = list(rec.vector)
            except (ValueError, TypeError, RuntimeError):
                old_vector = None
        self._db.put(
            self.namespace, vector_id, text, metadata=cur,
            vector=vector if vector is not None else old_vector,
        )

    def get(self, vector_id: str) -> Optional[OutputData]:
        """Retrieve a single record by its id."""
        try:
            record = self._db.memory.get(self.namespace, vector_id)
        except Exception:
            return None
        if record is None:
            return None
        return OutputData(
            id=getattr(record, "key", vector_id),
            score=1.0,
            payload=_build_payload(getattr(record, "metadata", None), getattr(record, "payload", "")),
        )

    def list_cols(self) -> List[str]:
        """Return all known collection (namespace) names."""
        try:
            return self._db.list_namespaces()
        except Exception:
            return [self.namespace]

    def delete_col(self) -> None:
        """Remove the current collection (namespace).

        FIND-94: no ``delete_namespace`` in SDK 0.5.0 — loop
        ``memory.list`` + ``memory.delete``. Re-list from the start
        until empty (no advancing cursor while deleting: offsets
        shift under mutation and would skip records past page 1).

        M-8 strict progress guard: the loop exits only on an empty page.
        Three non-progress signals raise instead of spinning forever:

        - a page identical to the previous one (every key survived a
          full delete pass);
        - two consecutive passes in which ``memory.delete`` reported
          ``False`` for every listed key (a read-only or failing
          backend). One zero-delete pass is tolerated so a concurrent
          deleter cannot false-positive the guard — if that actor
          finished, the next page is empty and the loop exits cleanly.
        - an iteration cap, for the pathological case where records are
          re-created faster than they are deleted.

        ``delete`` returning ``None`` (older bindings) is tolerated as a
        success signal — the identical-page check above still guards
        that case; only an explicit ``False`` counts as a failed delete.
        """
        prev_keys: Optional[frozenset] = None
        zero_delete_passes = 0
        for _ in range(_DELETE_COL_MAX_ITERATIONS):
            try:
                records = self._db.memory.list(self.namespace, limit=1000)
            except Exception as e:
                raise RuntimeError(f"Failed to delete collection {self.namespace}: {e}") from e
            page = records.records if hasattr(records, 'records') else records
            if not page:
                return
            keys = frozenset(r.key for r in page)
            if keys == prev_keys:
                raise RuntimeError(
                    f"delete_col({self.namespace}): no progress — the same "
                    f"{len(keys)} keys survived a full delete pass"
                )
            prev_keys = keys
            deleted = 0
            for r in page:
                if self._db.memory.delete(self.namespace, r.key) is not False:
                    deleted += 1
            if deleted == 0:
                zero_delete_passes += 1
                if zero_delete_passes >= 2:
                    raise RuntimeError(
                        f"delete_col({self.namespace}): memory.delete "
                        "reported failure for every key in two "
                        "consecutive passes — backend is not making "
                        "progress"
                    )
            else:
                zero_delete_passes = 0
        raise RuntimeError(
            f"delete_col({self.namespace}): exceeded "
            f"{_DELETE_COL_MAX_ITERATIONS} iterations without reaching an "
            "empty page"
        )

    def col_info(self) -> Dict[str, Any]:
        """Return basic metadata about the collection."""
        return {
            "name": self.namespace,
            "vector_size": None,
            "distance": "cosine",
        }

    def list(
        self,
        filters: Optional[Dict[str, Any]] = None,
        top_k: int = 100,
    ) -> List[OutputData]:
        """List records in the collection, optionally filtered.

        Same filter contract as :meth:`search` (FIND-579): equality specs
        are pushed down, other operators are evaluated in Python.
        """
        if top_k <= 0:
            return []
        native_eq, residual = _translate_filters(filters)
        fetch_k = top_k
        if residual:
            fetch_k = max(top_k * _POST_FILTER_OVERFETCH, _POST_FILTER_MIN_CANDIDATES)
        raw = self._db.memory.list(
            self.namespace, filters=native_eq or None, limit=fetch_k
        )
        out: List[OutputData] = []
        for r in raw.records:
            payload = _build_payload(r.metadata, r.payload)
            if residual and not _record_matches(payload, residual):
                continue
            out.append(
                OutputData(
                    id=r.key,
                    score=1.0,
                    payload=payload,
                )
            )
            if len(out) >= top_k:
                break
        return out

    def reset(self) -> None:
        """Delete the collection so it is recreated on next use."""
        self.delete_col()

    # ------------------------------------------------------------------
    #  Backward-compatible convenience method
    # ------------------------------------------------------------------

    def add(
        self,
        text: str,
        user_id: Optional[str] = None,
        metadata: Optional[dict] = None,
        **kwargs: Any,
    ) -> str:
        """Single text insert (original flat-class API).

        If ``self.embedding`` is set, the vector is computed internally.
        Returns the assigned key.
        """
        key = str(uuid.uuid4())
        meta = dict(metadata or {})
        if user_id:
            meta["user_id"] = user_id
        if self.embedding is not None:
            self._db.put(self.namespace, key, text, metadata=meta, vector=self.embedding(text))
        else:
            self._db.put(self.namespace, key, text, metadata=meta)
        return key

    # ── Lifecycle management ──────────────────────────────────

    def flush(self) -> None:
        """Flush pending writes to durable storage."""
        if hasattr(self._db, "flush"):
            self._db.flush()

    def close(self) -> None:
        """Close the underlying VantaDB client."""
        if hasattr(self._db, "close"):
            self._db.close()

    def __enter__(self) -> VantaDBVectorStore:
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()
