from __future__ import annotations

import math
from collections import defaultdict
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
        hybrid_mode: bool = False,
        **kwargs: Any,
    ):
        super().__init__(**kwargs)
        self._namespace = namespace
        self._db_path = db_path
        self._hybrid_mode = hybrid_mode
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

    def _hybrid_search(
        self,
        query_embedding: List[float],
        query_str: str,
        k: int,
        filters: Optional[Dict[str, Any]] = None,
    ) -> VectorStoreQueryResult:
        """Vector + text search fused via Reciprocal Rank Fusion (RRF)."""
        RRF_K = 60

        # Vector search — oversample 2x for the fusion pool
        vector_results = self._client.search_memory(
            self._namespace,
            query_embedding,
            top_k=k * 2,
            distance_metric="cosine",
            filters=filters,
        )

        # Text search — oversample 2x, vector is required positional but text_query does the work
        text_results = self._client.search_memory(
            self._namespace,
            query_embedding,
            top_k=k * 2,
            text_query=query_str,
            distance_metric="cosine",
            filters=filters,
        )

        # RRF fusion
        scores: Dict[str, float] = defaultdict(float)
        seen: Dict[str, Any] = {}

        for rank, hit in enumerate(vector_results):
            scores[hit.key] += 1.0 / (RRF_K + rank)
            seen[hit.key] = (hit, 1.0 - hit.score / 2.0)

        for rank, hit in enumerate(text_results):
            scores[hit.key] += 1.0 / (RRF_K + rank)
            if hit.key not in seen:
                seen[hit.key] = (hit, 1.0 - hit.score / 2.0)

        # Sort by combined RRF score, take top k
        ranked = sorted(scores.keys(), key=lambda key: scores[key], reverse=True)[:k]

        nodes: List[TextNode] = []
        similarities: List[float] = []
        ids: List[str] = []

        for key in ranked:
            hit, sim = seen[key]
            node = self._record_to_node(self._hit_to_dict(hit))
            nodes.append(node)
            similarities.append(sim)
            ids.append(key)

        return VectorStoreQueryResult(nodes=nodes, similarities=similarities, ids=ids)

    # ── MMR search ──────────────────────────────────────────

    def _mmr_search(
        self,
        query_embedding: List[float],
        k: int,
        fetch_k: int = 20,
        lambda_mult: float = 0.5,
        filters: Optional[Dict[str, Any]] = None,
    ) -> VectorStoreQueryResult:
        """MMR — balance relevance and diversity."""
        # 1. Fetch fetch_k candidates
        results = self._client.search_memory(
            self._namespace,
            query_embedding,
            top_k=fetch_k,
            distance_metric="cosine",
            filters=filters,
        )
        if not results:
            return VectorStoreQueryResult(nodes=[], similarities=[], ids=[])

        # 2. Load embeddings for each candidate
        cand_embs: List[List[float]] = []
        nodes: List[TextNode] = []
        similarities: List[float] = []

        for hit in results:
            node = self._record_to_node(self._hit_to_dict(hit))
            nodes.append(node)
            similarities.append(1.0 - hit.score / 2.0)

            rec = self._client.get_memory(self._namespace, hit.key)
            vec: List[float] = []
            if rec is not None:
                try:
                    v = rec.vector
                    vec = list(v) if v is not None else []
                except (ValueError, TypeError, RuntimeError):
                    vec = []
            if not vec:
                vec = query_embedding
            cand_embs.append(vec)

        # 3. Greedy MMR selection
        selected: List[int] = []
        candidates = list(range(len(nodes)))
        k = min(k, len(nodes))

        while len(selected) < k and candidates:
            best_idx = -1
            best_score = -1.0
            for i in candidates:
                mmr = lambda_mult * similarities[i]
                if selected:
                    max_sim = max(
                        self._cosine_sim(cand_embs[i], cand_embs[j])
                        for j in selected
                    )
                    mmr -= (1.0 - lambda_mult) * max_sim
                if mmr > best_score:
                    best_score = mmr
                    best_idx = i
            if best_idx < 0:
                break
            selected.append(best_idx)
            candidates.remove(best_idx)

        return VectorStoreQueryResult(
            nodes=[nodes[i] for i in selected],
            similarities=[similarities[i] for i in selected],
            ids=[nodes[i].node_id for i in selected],
        )

    @staticmethod
    def _cosine_sim(a: List[float], b: List[float]) -> float:
        dot = sum(x * y for x, y in zip(a, b))
        na = math.sqrt(sum(x * x for x in a))
        nb = math.sqrt(sum(x * x for x in b))
        if na == 0 or nb == 0:
            return 0.0
        return dot / (na * nb)

    def query(self, query: VectorStoreQuery, **kwargs: Any) -> VectorStoreQueryResult:
        query_embedding = query.query_embedding
        similarity_top_k = query.similarity_top_k or DEFAULT_TOP_K
        query_str = query.query_str

        if query_embedding is None:
            return VectorStoreQueryResult(nodes=[], similarities=[], ids=[])

        filters = self._build_vanta_filters(query.filters)

        # MMR mode — balance relevance and diversity
        if kwargs.get("mmr") or query.mode.value.lower() == "mmr":
            mmr_fetch_k = kwargs.get("mmr_fetch_k", similarity_top_k * 5)
            mmr_lambda_mult = kwargs.get("mmr_lambda_mult", 0.5)
            return self._mmr_search(
                query_embedding,
                k=similarity_top_k,
                fetch_k=mmr_fetch_k,
                lambda_mult=mmr_lambda_mult,
                filters=filters,
            )

        # Client-side RRF fusion when hybrid_mode is enabled and we have text
        if self._hybrid_mode and query_str:
            return self._hybrid_search(query_embedding, query_str, similarity_top_k, filters)

        # Server-side hybrid (VantaDB internal) or pure vector
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
