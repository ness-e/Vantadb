from __future__ import annotations

from typing import Any, Dict, List, Optional, Sequence

import vantadb_py as vanta
from llama_index.core.bridge.pydantic import PrivateAttr
from llama_index.core.schema import BaseNode, MetadataMode, TextNode, NodeRelationship, RelatedNodeInfo
from llama_index.core.vector_stores.types import (
    BasePydanticVectorStore,
    MetadataFilters,
    FilterOperator,
    VectorStoreQuery,
    VectorStoreQueryResult,
)
from llama_index.core.vector_stores.utils import (
    metadata_dict_to_node,
    node_to_metadata_dict,
)

DEFAULT_NAMESPACE = "llamaindex"
DEFAULT_TOP_K = 4


class VantaDBVectorStore(BasePydanticVectorStore):
    stores_text: bool = True
    flat_metadata: bool = False
    is_embedding_query: bool = True

    def __init__(
        self,
        db_path: str = "./vantadb_data",
        namespace: str = DEFAULT_NAMESPACE,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
        **kwargs: Any,
    ):
        super().__init__(**kwargs)
        self._namespace = namespace
        self._db_path = db_path
        self._client = vanta.VantaDB(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    @property
    def client(self) -> vanta.VantaDB:
        return self._client

    @property
    def namespace(self) -> str:
        return self._namespace

    def _node_to_key(self, node: BaseNode) -> str:
        return node.node_id

    @staticmethod
    def _hit_to_dict(hit: vanta.VantaSearchHit) -> dict:
        return {
            "key": hit.key,
            "node_id": hit.id,
            "payload": hit.payload,
            "metadata": dict(hit.metadata),
            "created_at_ms": hit.created_at_ms,
            "updated_at_ms": hit.updated_at_ms,
            "version": hit.version,
        }

    @staticmethod
    def _record_to_dict(record: vanta.VantaMemoryRecord) -> dict:
        try:
            vec = record.vector
            if vec is not None:
                vec = list(vec)
        except (ValueError, TypeError, RuntimeError):
            vec = None
        return {
            "key": record.key,
            "payload": record.payload,
            "metadata": dict(record.metadata),
            "vector": vec,
            "created_at_ms": record.created_at_ms,
            "updated_at_ms": record.updated_at_ms,
            "version": record.version,
            "node_id": record.node_id,
        }

    def _record_to_node(self, record: dict) -> TextNode:
        metadata = dict(record.get("metadata", {}))
        node_id = record.get("key", "")
        text = record.get("payload", "")
        embedding = record.get("vector")

        node = metadata_dict_to_node(metadata, text=text) if metadata else TextNode(
            text=text,
            id_=node_id,
        )
        if embedding:
            node.embedding = embedding
        return node

    # ── Required abstract methods ────────────────────────────

    def add(self, nodes: Sequence[BaseNode], **kwargs: Any) -> List[str]:
        ids: List[str] = []
        entries: List[tuple] = []

        for node in nodes:
            node_id = self._node_to_key(node)
            text = node.get_content(MetadataMode.NONE)
            embedding = node.get_embedding()
            metadata = node_to_metadata_dict(
                node, remove_text=True, flat_metadata=self.flat_metadata
            )

            entries.append((self._namespace, node_id, text, metadata, embedding, None))
            ids.append(node_id)

        if entries:
            self._client.put_batch(entries)
        return ids

    def delete(self, ref_doc_id: str, **delete_kwargs: Any) -> None:
        page = self._client.list_memory(
            self._namespace,
            filters={"ref_doc_id": ref_doc_id},
            limit=10000,
        )
        for rec in page.records:
            key = rec.key
            if key:
                self._client.delete_memory(self._namespace, key)

    def query(self, query: VectorStoreQuery, **kwargs: Any) -> VectorStoreQueryResult:
        query_embedding = query.query_embedding
        similarity_top_k = query.similarity_top_k or DEFAULT_TOP_K
        query_str = query.query_str

        if query_embedding is None:
            return VectorStoreQueryResult(nodes=[], similarities=[], ids=[])

        filters = self._build_vanta_filters(query.filters)

        if query.mode.value == "hybrid" or (query_str and query_embedding):
            results = self._client.search_memory(
                self._namespace,
                query_embedding,
                top_k=similarity_top_k,
                text_query=query_str,
                distance_metric="cosine",
                filters=filters,
            )
        else:
            results = self._client.search_memory(
                self._namespace,
                query_embedding,
                top_k=similarity_top_k,
                distance_metric="cosine",
                filters=filters,
            )

        nodes: List[TextNode] = []
        similarities: List[float] = []
        ids: List[str] = []

        for hit in results:
            node = self._record_to_node(self._hit_to_dict(hit))
            nodes.append(node)
            similarities.append(1.0 - hit.score / 2.0)
            ids.append(hit.key)

        return VectorStoreQueryResult(nodes=nodes, similarities=similarities, ids=ids)

    def _build_vanta_filters(self, filters: Optional[MetadataFilters]) -> Optional[Dict[str, Any]]:
        if filters is None or not filters.filters:
            return None

        result: Dict[str, Any] = {}
        for f in filters.filters:
            if hasattr(f, "key") and hasattr(f, "value"):
                result[f.key] = f.value
        return result if result else None

    # ── Optional methods ─────────────────────────────────────

    def get_nodes(
        self,
        node_ids: Optional[List[str]] = None,
        filters: Optional[MetadataFilters] = None,
    ) -> List[BaseNode]:
        nodes: List[BaseNode] = []
        if node_ids:
            for node_id in node_ids:
                record = self._client.get_memory(self._namespace, node_id)
                if record:
                    nodes.append(self._record_to_node(self._record_to_dict(record)))
        return nodes

    def delete_nodes(
        self,
        node_ids: Optional[List[str]] = None,
        filters: Optional[MetadataFilters] = None,
        **delete_kwargs: Any,
    ) -> None:
        if node_ids:
            for node_id in node_ids:
                self._client.delete_memory(self._namespace, node_id)

    def clear(self) -> None:
        all_records = self._client.list_memory(self._namespace, limit=10000)
        for rec in all_records.records:
            key = rec.key
            if key:
                self._client.delete_memory(self._namespace, key)
