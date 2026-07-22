"""Haystack 2.x DocumentStore adapter for VantaDB."""
from __future__ import annotations

import uuid
from typing import Any, Callable, Dict, List, Optional

import vantadb_py as vanta

from haystack.dataclasses import Document
from haystack.document_stores.types import DuplicatePolicy

DEFAULT_NAMESPACE = "haystack"
DEFAULT_TOP_K = 4
_MAX_LIST_LIMIT = 1_000_000


def _post_filter_documents(
    docs: List[Document],
    filters: Dict[str, Any],
) -> List[Document]:
    """Apply Haystack filters client-side for operators VantaDB can't handle.

    Handles OR, NOT, and comparison operators (``!=``, ``>``, ``<``, etc.)
    by filtering the in-memory document list.
    """
    if not filters:
        return docs

    # Simple field filter with comparison
    if "field" in filters:
        op = filters.get("operator", "==")
        field = filters["field"]
        value = filters["value"]

        def _match(doc: Document) -> bool:
            # Resolve the field value from the document
            # Fields like "meta.key" look in meta dict
            if field.startswith("meta."):
                doc_val = doc.meta.get(field[5:])
            elif field == "id":
                doc_val = doc.id
            elif field == "content":
                doc_val = doc.content
            else:
                doc_val = doc.meta.get(field)

            if op == "==":
                return doc_val == value
            elif op == "!=":
                return doc_val != value
            elif op == ">":
                return doc_val is not None and doc_val > value
            elif op == ">=":
                return doc_val is not None and doc_val >= value
            elif op == "<":
                return doc_val is not None and doc_val < value
            elif op == "<=":
                return doc_val is not None and doc_val <= value
            elif op == "in":
                return doc_val in value if hasattr(value, "__contains__") else False
            elif op == "not in":
                return doc_val not in value if hasattr(value, "__contains__") else True
            return True

        return [d for d in docs if _match(d)]

    # Compound filter
    if "operator" in filters and "conditions" in filters:
        op = filters["operator"]
        conditions = filters["conditions"]

        if op == "AND":
            result = docs
            for cond in conditions:
                result = _post_filter_documents(result, cond)
            return result
        elif op == "OR":
            matched: List[Document] = []
            seen: set = set()
            for cond in conditions:
                for d in _post_filter_documents(docs, cond):
                    if d.id not in seen:
                        matched.append(d)
                        seen.add(d.id)
            return matched
        elif op == "NOT":
            excluded = _post_filter_documents(docs, conditions[0])
            excluded_ids = {d.id for d in excluded}
            return [d for d in docs if d.id not in excluded_ids]

    return docs


class VantaDBDocumentStore:
    """Haystack DocumentStore backed by VantaDB.

    Implements the ``DocumentStore`` protocol — use it in Haystack pipelines.
    The embedding function is optional; when set, ``write_documents()`` stores
    vectors and ``search()`` uses vector similarity.
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
        """Initialize a Haystack DocumentStore backed by VantaDB.

        Args:
            embedding: Optional callable that converts text to an
                embedding vector list. When set, ``write_documents()``
                stores vectors and ``search()`` uses vector similarity.
            db_path: Filesystem path for the VantaDB database.
                Defaults to ``"./vantadb_data"``.
            namespace: VantaDB namespace to operate on.
                Defaults to ``"haystack"``.
            memory_limit_bytes: Optional maximum memory usage in bytes.
            read_only: If True, open the database in read-only mode.
                Defaults to False.
            backend: Optional backend identifier for VantaDB.
        """
        self.embedding = embedding
        self.namespace = namespace
        self._db_path = db_path
        self._memory_limit_bytes = memory_limit_bytes
        self._read_only = read_only
        self._backend = backend
        self._db = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    # ── Internal helpers ──────────────────────────────────

    def _embed(self, text: str) -> Optional[List[float]]:
        """Embed a single text string if an embedding function is configured."""
        if self.embedding is None:
            return None
        return self.embedding(text)

    @staticmethod
    def _record_to_document(record: vanta.VantaMemoryRecord) -> Document:
        return Document(
            id=record.key,
            content=record.payload,
            meta=dict(record.metadata or {}),
        )

    @staticmethod
    def _hit_to_document(hit: vanta.VantaSearchHit) -> Document:
        return Document(
            id=hit.key,
            content=hit.payload,
            meta=dict(hit.metadata or {}),
        )

    @staticmethod
    def _normalize(
        doc: Any,
    ) -> tuple[str, str, dict[str, Any]]:
        """Return ``(id, content, meta)`` from a ``Document`` or a plain ``dict``."""
        if isinstance(doc, Document):
            return (
                doc.id or str(uuid.uuid4()),
                doc.content or "",
                dict(doc.meta or {}),
            )
        # dict fallback for backward compat
        return (
            doc.get("id", str(uuid.uuid4())),
            doc.get("content", doc.get("text", "")),
            dict(doc.get("meta", {})),
        )

    @staticmethod
    def _haystack_filter_to_vanta(
        filters: Optional[Dict[str, Any]],
    ) -> Dict[str, Any]:
        """Translate Haystack filter syntax to VantaDB flat metadata filters.

        Handles the Haystack filter DSL commonly seen in Haystack pipelines:

        * Flat dicts (already VantaDB format) pass through unchanged.
        * ``{"field": k, "operator": "==", "value": v}`` → ``{k: v}``
        * ``{"operator": "AND", "conditions": [...]}`` flattens nested
          equality conditions into a single dict.
        * OR, NOT, and non-``==`` operators are handled by returning an empty
          filter and letting the post-filter in ``filter_documents()`` do the
          full filtering.
        * The ``meta.`` prefix is stripped — VantaDB stores metadata as flat
          dict keys, so ``meta.type`` becomes ``type``.

        Example:
            >>> VantaDBDocumentStore._haystack_filter_to_vanta(
            ...     {"field": "meta.type", "operator": "==", "value": "pdf"})
            {"type": "pdf"}

            >>> VantaDBDocumentStore._haystack_filter_to_vanta(
            ...     {"operator": "AND", "conditions": [
            ...         {"field": "meta.a", "operator": "==", "value": 1},
            ...         {"field": "meta.b", "operator": "==", "value": 2},
            ...     ]})
            {"a": 1, "b": 2}
        """
        if not filters:
            return {}

        # Already a flat dict — no Haystack nesting detected
        if "field" not in filters and "operator" not in filters:
            return filters

        # Simple field filter with equality
        if filters.get("operator") == "==" and "field" in filters:
            key = filters["field"]
            # Strip "meta." prefix — VantaDB metadata is stored as flat keys
            if key.startswith("meta."):
                key = key[5:]
            return {key: filters["value"]}

        # Compound AND — flatten all equality conditions
        if filters.get("operator") == "AND" and "conditions" in filters:
            result: Dict[str, Any] = {}
            for cond in filters["conditions"]:
                flat = VantaDBDocumentStore._haystack_filter_to_vanta(cond)
                # If a sub-condition couldn't be flattened, skip it
                # (it will be handled by post-filtering in filter_documents)
                if "field" not in flat and "operator" not in flat:
                    result.update(flat)
            return result

        # OR, NOT, or non-== operators — VantaDB can't handle these natively,
        # so return empty filter (all docs) and let post-filter do the work.
        return {}

    # ── DocumentStore protocol ────────────────────────────

    def write_documents(
        self,
        documents: List[Document],
        policy: DuplicatePolicy = DuplicatePolicy.NONE,
    ) -> int:
        """Write documents into the VantaDB store.

        Implements the Haystack ``DocumentStore.write_documents``
        protocol. Supports ``NONE``, ``FAIL``, ``SKIP``, and
        ``OVERWRITE`` duplicate policies.

        Args:
            documents: List of Haystack ``Document`` objects to write.
            policy: Duplicate handling policy. Defaults to ``NONE``.

        Returns:
            The number of documents successfully written.

        Raises:
            ValueError: If ``policy`` is ``FAIL`` and a document with
                the same ID already exists.
        """
        if isinstance(policy, str):
            policy = DuplicatePolicy(policy)
        count = 0
        for doc in documents:
            doc_id, content, meta = self._normalize(doc)

            if policy == DuplicatePolicy.FAIL:
                existing = self._db.get_memory(self.namespace, doc_id)
                if existing is not None:
                    raise ValueError(
                        f"Document with id '{doc_id}' already exists"
                    )
            elif policy == DuplicatePolicy.SKIP:
                existing = self._db.get_memory(self.namespace, doc_id)
                if existing is not None:
                    continue
            elif policy == DuplicatePolicy.OVERWRITE:
                # Delete existing record first so the new one replaces it
                # with fresh metadata, content, and vector.
                self._db.delete_memory(self.namespace, doc_id)

            vector = self._embed(content)
            self._db.put(
                self.namespace, doc_id, content,
                metadata=meta, vector=vector,
            )
            count += 1
        return count

    def filter_documents(
        self,
        filters: Optional[Dict[str, Any]] = None,
    ) -> List[Document]:
        """Retrieve documents matching the given filters.

        Implements the Haystack ``DocumentStore.filter_documents``
        protocol. Translates the Haystack filter DSL to VantaDB
        metadata filters; complex conditions (OR, NOT, comparisons)
        are handled via client-side post-filtering.

        Args:
            filters: Optional Haystack filter dictionary. Pass
                ``None`` to return all documents.

        Returns:
            A list of matching ``Document`` objects.
        """
        vanta_filters = self._haystack_filter_to_vanta(filters)
        results = self._db.list_memory(
            self.namespace,
            filters=vanta_filters or {},
            limit=_MAX_LIST_LIMIT,
        )
        docs = [self._record_to_document(r) for r in results.records]

        # Client-side post-filtering for filters VantaDB can't handle
        # natively (AND, OR, NOT, non-== operators, or compound conditions
        # that couldn't be fully flattened).
        if filters and (
            "field" in filters
            or filters.get("operator") in ("AND", "OR", "NOT")
        ):
            docs = _post_filter_documents(docs, filters)

        return docs

    def delete_documents(
        self,
        document_ids: Optional[List[str]] = None,
        **kwargs: Any,
    ) -> None:
        """Delete documents from the VantaDB store.

        Implements the Haystack ``DocumentStore.delete_documents``
        protocol. Supports deletion by explicit ID list or by
        filter dictionary (legacy).

        Args:
            document_ids: Optional list of document IDs to delete.
            **kwargs: Legacy support for ``filters`` dict with
                an ``"id"`` key.
        """
        # Protocol path: delete_documents(["id1", "id2"])
        if document_ids is not None:
            for doc_id in document_ids:
                self._db.delete_memory(self.namespace, doc_id)
            return

        # Backward compat: delete_documents(filters={"id": "x"})
        filters = kwargs.get("filters", {})
        if isinstance(filters, dict):
            doc_id = filters.pop("id", None)
            if doc_id:
                self._db.delete_memory(self.namespace, str(doc_id))
                return
        results = self._db.list_memory(
            self.namespace, filters=filters or {}, limit=_MAX_LIST_LIMIT,
        )
        for rec in results.records:
            if rec.key:
                self._db.delete_memory(self.namespace, rec.key)

    def count_documents(self) -> int:
        """Return the total number of documents in the store.

        Implements the Haystack ``DocumentStore.count_documents``
        protocol.

        Returns:
            The document count as an integer.
        """
        results = self._db.list_memory(
            self.namespace, filters={}, limit=_MAX_LIST_LIMIT,
        )
        return len(results.records)

    def to_dict(self) -> Dict[str, Any]:
        """Serialize the store configuration to a dictionary.

        Implements the Haystack ``DocumentStore.to_dict`` protocol
        for serialisation and YAML pipeline export.

        Returns:
            A dictionary with ``"type"`` and ``"init_parameters"``
            keys suitable for ``from_dict()`` round-tripping.
        """
        params: Dict[str, Any] = {
            "db_path": self._db_path,
            "namespace": self.namespace,
        }
        if self._memory_limit_bytes is not None:
            params["memory_limit_bytes"] = self._memory_limit_bytes
        if self._read_only:
            params["read_only"] = self._read_only
        if self._backend is not None:
            params["backend"] = self._backend
        return {
            "type": "VantaDBDocumentStore",
            "init_parameters": params,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "VantaDBDocumentStore":
        """Deserialize store configuration from a dictionary.

        Implements the Haystack ``DocumentStore.from_dict`` protocol
        for pipeline YAML deserialisation.

        Args:
            data: Dictionary with an ``"init_parameters"`` key, as
                produced by ``to_dict()``.

        Returns:
            A new ``VantaDBDocumentStore`` instance.
        """
        return cls(**data.get("init_parameters", {}))

    # ── Vector search (extra — not in protocol) ───────────

    def search(self, query: str, k: int = DEFAULT_TOP_K) -> List[Document]:
        """Perform a vector or keyword search over stored documents.

        Extra method not part of the core Haystack ``DocumentStore``
        protocol. When an embedding function is configured, results are
        ranked by cosine similarity; otherwise all documents are listed
        and the first ``k`` are returned.

        Args:
            query: The search query string.
            k: Number of results to return. Defaults to 4.

        Returns:
            A list of ``Document`` objects matching the query.
        """
        if self.embedding is None:
            results = self._db.list_memory(
                self.namespace, filters={}, limit=k,
            )
            return [self._record_to_document(r) for r in results.records]
        vector = self.embedding(query)
        results = self._db.search_memory(
            self.namespace, vector,
            top_k=k, distance_metric="cosine",
        )
        return [self._hit_to_document(hit) for hit in results]
