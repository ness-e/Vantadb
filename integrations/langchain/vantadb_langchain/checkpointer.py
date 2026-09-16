"""LangGraph ``BaseCheckpointSaver`` adapter backed by VantaDB (INTG-01).

Checkpoints are stored inline (full ``channel_values`` in the payload —
no blob split; semantically equivalent to the reference ``InMemorySaver``).
Keys are JSON arrays so thread/ns/id round-trip without escaping:

- checkpoints: namespace ``langgraph:checkpoints``,
  key ``[thread_id, checkpoint_ns, checkpoint_id]``
- writes: namespace ``langgraph:writes``,
  key ``[thread_id, checkpoint_ns, checkpoint_id, task_id, idx]``

Thread queries use VantaDB metadata filters (``thread_id`` mirrored in
record metadata).

Sources:
- https://github.com/langchain-ai/langgraph/blob/5931a5f0/libs/checkpoint/README.md
- https://reference.langchain.com/python/langgraph.checkpoint/base/BaseCheckpointSaver
- Reference semantics: ``langgraph.checkpoint.memory.InMemorySaver``
"""

from __future__ import annotations

import base64
import json
from collections.abc import AsyncIterator, Iterator, Sequence
from typing import Any, Optional

import vantadb_py as vanta
from langchain_core.runnables import RunnableConfig
from langgraph.checkpoint.base import (
    BaseCheckpointSaver,
    ChannelVersions,
    Checkpoint,
    CheckpointMetadata,
    CheckpointTuple,
    SerializerProtocol,
    get_checkpoint_id,
    get_checkpoint_metadata,
)

CKPT_NS = "langgraph.checkpoints"
WRITES_NS = "langgraph.writes"


def _b64encode(raw: bytes) -> str:
    return base64.b64encode(raw).decode("ascii")


def _b64decode(data: str) -> bytes:
    return base64.b64decode(data.encode("ascii"))


def _ckpt_key(thread_id: str, checkpoint_ns: str, checkpoint_id: str) -> str:
    return json.dumps([thread_id, checkpoint_ns, checkpoint_id])


def _ckpt_thread_meta(thread_id: str) -> dict[str, Any]:
    return {"thread_id": thread_id}


class VantaDBCheckpointer(BaseCheckpointSaver):
    """Persistent LangGraph checkpointer backed by embedded VantaDB.

    Args:
        db_path: Filesystem path for the VantaDB database.
        serde: Optional LangGraph serializer (defaults to
            ``JsonPlusSerializer`` via the base class).
    """

    def __init__(
        self,
        db_path: str = "./vantadb_data",
        *,
        serde: Optional[SerializerProtocol] = None,
        memory_limit_bytes: Optional[int] = None,
        read_only: bool = False,
        backend: Optional[str] = None,
    ):
        super().__init__(serde=serde)
        self._db = vanta.Client(
            db_path,
            memory_limit_bytes=memory_limit_bytes,
            read_only=read_only,
            backend=backend,
        )

    # ── serde helpers ────────────────────────────────────────

    def _dump(self, obj: Any) -> dict[str, str]:
        type_tag, data = self.serde.dumps_typed(obj)
        return {"type": type_tag, "data": _b64encode(data)}

    def _load(self, blob: dict[str, str]) -> Any:
        return self.serde.loads_typed((blob["type"], _b64decode(blob["data"])))

    # ── reads ────────────────────────────────────────────────

    def _scan_ckpt_keys(self, thread_id: Optional[str] = None) -> list[str]:
        keys: list[str] = []
        cursor: Optional[int] = None
        filters = _ckpt_thread_meta(thread_id) if thread_id else None
        while True:
            page = self._db.memory.list(
                CKPT_NS, filters=filters, limit=1000, cursor=cursor
            )
            if not page or not page.records:
                break
            keys.extend(rec.key for rec in page.records if rec.key)
            cursor = page.next_cursor
            if cursor is None:
                break
        return keys

    def _read_ckpt(self, key: str) -> Optional[dict[str, Any]]:
        record = self._db.memory.get(CKPT_NS, key)
        if record is None:
            return None
        return json.loads(record.payload)

    def _writes_for(self, thread_id: str, ns: str, ckpt_id: str) -> list[Any]:
        prefix = json.dumps([thread_id, ns, ckpt_id])[:-1]  # strip trailing "]"
        pending: list[Any] = []
        cursor: Optional[int] = None
        while True:
            page = self._db.memory.list(
                WRITES_NS, filters=_ckpt_thread_meta(thread_id), limit=1000,
                cursor=cursor,
            )
            if not page or not page.records:
                break
            for rec in page.records:
                if rec.key and rec.key.startswith(prefix):
                    blob = json.loads(rec.payload)
                    pending.append(
                        (blob["task_id"], blob["channel"], self._load(blob["value"]))
                    )
            cursor = page.next_cursor
            if cursor is None:
                break
        return pending

    def _to_tuple(
        self, config: RunnableConfig, key: str, stored: dict[str, Any]
    ) -> CheckpointTuple:
        thread_id, checkpoint_ns, checkpoint_id = json.loads(key)
        checkpoint = self._load(stored["checkpoint"])
        metadata = self._load(stored["metadata"])
        parent_id = stored.get("parent_id")
        base_config: RunnableConfig = {
            "configurable": {
                "thread_id": thread_id,
                "checkpoint_ns": checkpoint_ns,
                "checkpoint_id": checkpoint_id,
            }
        }
        return CheckpointTuple(
            config=base_config,
            checkpoint=checkpoint,
            metadata=metadata,
            parent_config=(
                {
                    "configurable": {
                        "thread_id": thread_id,
                        "checkpoint_ns": checkpoint_ns,
                        "checkpoint_id": parent_id,
                    }
                }
                if parent_id
                else None
            ),
            pending_writes=self._writes_for(thread_id, checkpoint_ns, checkpoint_id),
        )

    def get_tuple(self, config: RunnableConfig) -> Optional[CheckpointTuple]:
        thread_id: str = config["configurable"]["thread_id"]
        checkpoint_ns: str = config["configurable"].get("checkpoint_ns", "")
        if checkpoint_id := get_checkpoint_id(config):
            key = _ckpt_key(thread_id, checkpoint_ns, checkpoint_id)
            stored = self._read_ckpt(key)
            return self._to_tuple(config, key, stored) if stored else None
        candidates = [
            k for k in self._scan_ckpt_keys(thread_id)
            if json.loads(k)[1] == checkpoint_ns
        ]
        if not candidates:
            return None
        key = max(candidates, key=lambda k: json.loads(k)[2])
        stored = self._read_ckpt(key)
        return self._to_tuple(config, key, stored) if stored else None

    def list(
        self,
        config: Optional[RunnableConfig],
        *,
        filter: Optional[dict[str, Any]] = None,
        before: Optional[RunnableConfig] = None,
        limit: Optional[int] = None,
    ) -> Iterator[CheckpointTuple]:
        thread_ids: Optional[tuple[str, ...]] = (
            (config["configurable"]["thread_id"],) if config else None
        )
        config_ns = config["configurable"].get("checkpoint_ns") if config else None
        config_id = get_checkpoint_id(config) if config else None
        before_id = get_checkpoint_id(before) if before else None
        keys: list[str] = []
        for tid in thread_ids or [None]:
            keys.extend(self._scan_ckpt_keys(tid))
        for key in sorted(keys, key=lambda k: json.loads(k)[2], reverse=True):
            thread_id, checkpoint_ns, checkpoint_id = json.loads(key)
            if config_ns is not None and checkpoint_ns != config_ns:
                continue
            if config_id and checkpoint_id != config_id:
                continue
            if before_id and checkpoint_id >= before_id:
                continue
            stored = self._read_ckpt(key)
            if stored is None:
                continue
            metadata = self._load(stored["metadata"])
            if filter and not all(
                v == metadata.get(k) for k, v in filter.items()
            ):
                continue
            if limit is not None and limit <= 0:
                break
            if limit is not None:
                limit -= 1
            yield self._to_tuple(config or {}, key, stored)

    # ── writes ───────────────────────────────────────────────

    def put(
        self,
        config: RunnableConfig,
        checkpoint: Checkpoint,
        metadata: CheckpointMetadata,
        new_versions: ChannelVersions,
    ) -> RunnableConfig:
        thread_id: str = config["configurable"]["thread_id"]
        checkpoint_ns: str = config["configurable"].get("checkpoint_ns", "")
        key = _ckpt_key(thread_id, checkpoint_ns, checkpoint["id"])
        payload = json.dumps(
            {
                "checkpoint": self._dump(dict(checkpoint)),
                "metadata": self._dump(
                    get_checkpoint_metadata(config, metadata)
                ),
                "parent_id": config["configurable"].get("checkpoint_id"),
            }
        )
        self._db.put(
            CKPT_NS,
            key,
            payload,
            metadata={
                "thread_id": thread_id,
                "checkpoint_ns": checkpoint_ns,
                "checkpoint_id": checkpoint["id"],
            },
        )
        return {
            "configurable": {
                "thread_id": thread_id,
                "checkpoint_ns": checkpoint_ns,
                "checkpoint_id": checkpoint["id"],
            }
        }

    def put_writes(
        self,
        config: RunnableConfig,
        writes: Sequence[tuple[str, Any]],
        task_id: str,
        task_path: str = "",
    ) -> None:
        thread_id: str = config["configurable"]["thread_id"]
        checkpoint_ns: str = config["configurable"].get("checkpoint_ns", "")
        checkpoint_id: str = config["configurable"]["checkpoint_id"]
        for idx, (channel, value) in enumerate(writes):
            key = json.dumps([thread_id, checkpoint_ns, checkpoint_id, task_id, idx])
            if self._db.memory.get(WRITES_NS, key) is not None:
                continue
            payload = json.dumps(
                {
                    "task_id": task_id,
                    "channel": channel,
                    "value": self._dump(value),
                    "task_path": task_path,
                }
            )
            self._db.put(
                WRITES_NS,
                key,
                payload,
                metadata=_ckpt_thread_meta(thread_id),
            )

    def delete_thread(self, thread_id: str) -> None:
        for ns in (CKPT_NS, WRITES_NS):
            cursor: Optional[int] = None
            while True:
                page = self._db.memory.list(
                    ns, filters=_ckpt_thread_meta(thread_id), limit=1000,
                    cursor=cursor,
                )
                if not page or not page.records:
                    break
                for rec in page.records:
                    if rec.key:
                        self._db.memory.delete(ns, rec.key)
                cursor = page.next_cursor
                if cursor is None:
                    break

    # ── async (thin wrappers — same pattern as InMemorySaver) ──

    async def aget_tuple(self, config: RunnableConfig) -> Optional[CheckpointTuple]:
        return self.get_tuple(config)

    async def alist(
        self,
        config: Optional[RunnableConfig],
        *,
        filter: Optional[dict[str, Any]] = None,
        before: Optional[RunnableConfig] = None,
        limit: Optional[int] = None,
    ) -> AsyncIterator[CheckpointTuple]:
        for item in self.list(config, filter=filter, before=before, limit=limit):
            yield item

    async def aput(
        self,
        config: RunnableConfig,
        checkpoint: Checkpoint,
        metadata: CheckpointMetadata,
        new_versions: ChannelVersions,
    ) -> RunnableConfig:
        return self.put(config, checkpoint, metadata, new_versions)

    async def aput_writes(
        self,
        config: RunnableConfig,
        writes: Sequence[tuple[str, Any]],
        task_id: str,
        task_path: str = "",
    ) -> None:
        return self.put_writes(config, writes, task_id, task_path)

    async def adelete_thread(self, thread_id: str) -> None:
        return self.delete_thread(thread_id)
