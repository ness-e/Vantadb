"""Mem0-shaped local memory adapter over VantaDB — signature-compatible subset.

Implements the six Mem0-style operations requested: ``add``, ``search``,
``get_all``, ``get``, ``delete``, ``reset``, plus ``close``/context-manager
support. Explicitly NOT compatible with: Mem0 v2 result envelopes, LLM-driven
extraction/dedup pipelines, ``update()``, ``history()``, ``from_config()``, or
any managed/cloud service.

Scopes (``user_id``/``agent_id``/``run_id``) are plain metadata filters —
capability identifiers, NOT credentials or an authorization boundary.

``search`` returns the engine's native score per hit — a retrieval score,
not a calibrated probability. ``reset`` deletes only records carrying the
adapter's private marker inside the adapter's namespace; it never touches
other namespaces, other records, or the database itself.

No network calls, no downloads: embeddings come from ``_LocalE5Encoder``
(local multilingual-e5-small ONNX) or a caller-supplied callable.
"""

from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path


class _LocalE5Encoder:
    """Run a pre-provisioned multilingual-e5-small export without network access."""

    def __init__(self) -> None:
        import numpy as np
        import onnxruntime as ort
        from tokenizers import Tokenizer

        model_dir = (
            Path(__file__).resolve().parents[1]
            / "embeddings"
            / "models"
            / "multilingual-e5-small"
        )
        model_path = model_dir / "onnx" / "model.onnx"
        tokenizer_path = model_dir / "tokenizer.json"
        if not model_path.is_file() or not tokenizer_path.is_file():
            raise FileNotFoundError(
                "Local multilingual-e5-small files are missing. Provision the model "
                "with embeddings/download.py before running offline, or pass an "
                "embedding_model callable. No model is downloaded by this adapter."
            )
        self._np = np
        options = ort.SessionOptions()
        options.intra_op_num_threads = 1
        self._session = ort.InferenceSession(
            str(model_path), sess_options=options, providers=["CPUExecutionProvider"]
        )
        self._tokenizer = Tokenizer.from_file(str(tokenizer_path))
        self._tokenizer.enable_truncation(max_length=512)

    def _encode(self, text: str, prefix: str) -> list[float]:
        np = self._np
        tokens = self._tokenizer.encode(prefix + text)
        feeds = {
            "input_ids": np.asarray([tokens.ids], dtype=np.int64),
            "attention_mask": np.asarray([tokens.attention_mask], dtype=np.int64),
            "token_type_ids": np.asarray([tokens.type_ids], dtype=np.int64),
        }
        inputs = {item.name: feeds[item.name] for item in self._session.get_inputs()}
        outputs = dict(
            zip(
                (item.name for item in self._session.get_outputs()),
                self._session.run(None, inputs),
                strict=True,
            )
        )
        if "sentence_embedding" in outputs:
            vector = outputs["sentence_embedding"][0]
        else:
            hidden = outputs.get("last_hidden_state")
            if hidden is None:
                raise ValueError(
                    "The local E5 export has no supported embedding output."
                )
            mask = feeds["attention_mask"][0].astype(np.float32)[:, None]
            vector = (hidden[0] * mask).sum(axis=0) / max(float(mask.sum()), 1.0)
        norm = float(np.linalg.norm(vector))
        if (
            vector.shape != (384,)
            or not np.isfinite(vector).all()
            or not np.isfinite(norm)
            or norm == 0.0
        ):
            raise ValueError(
                "The local E5 export must return a finite nonzero 384d vector."
            )
        return (vector / norm).tolist()

    def __call__(self, text: str) -> list[float]:
        return self._encode(text, "passage: ")

    def embed_query(self, text: str) -> list[float]:
        return self._encode(text, "query: ")


_MARKER = "_vanta_mem0"
_META_JSON = "_vanta_mem0_meta"
_ROLE = "_vanta_mem0_role"
_INTERNAL = {_MARKER, _META_JSON, _ROLE}
_SCOPES = ("user_id", "agent_id", "run_id")
_RESERVED = _INTERNAL | set(_SCOPES)


def _check_scope(name: str, value) -> None:
    if value is not None and (not isinstance(value, str) or not value):
        raise ValueError(f"{name} must be a non-empty string or None")


def _flat(filters, where: str) -> dict:
    """Validate flat equality filters: scalar values only, no operator keys."""
    if filters is not None and not isinstance(filters, dict):
        raise TypeError(f"{where} must be a dict or None, got {type(filters).__name__}")
    out = {}
    for k, v in (filters or {}).items():
        if not isinstance(k, str) or not k:
            raise ValueError(f"{where}: filter keys must be non-empty strings")
        if k.startswith("$") or k in _RESERVED:
            raise ValueError(f"{where}: '{k}' is a reserved key or an operator")
        if isinstance(v, (dict, list)):
            raise TypeError(
                f"{where}: only flat equality filters are supported (got {k}={v!r})"
            )
        if not isinstance(v, (str, int, float, bool)) and v is not None:
            raise TypeError(f"{where}: filter values must be scalars (got {k}={v!r})")
        if isinstance(v, float) and not math.isfinite(v):
            raise ValueError(f"{where}: filter values must be finite (got {k}={v!r})")
        out[k] = v
    return out


class VantaMemory:
    """Mem0-shaped memory over a VantaDB embedded database."""

    def __init__(
        self,
        db_path: str = "./vanta_data",
        namespace: str = "mem0/default",
        embedding_model=None,
    ) -> None:
        self._path = db_path
        self._ns = namespace
        if embedding_model is not None and not callable(embedding_model):
            raise TypeError("embedding_model must be callable (or None for local E5)")
        self._encoder = embedding_model
        self._db = None

    def _client(self):
        if self._db is None:
            from vantadb import Client

            self._db = Client(self._path)
        return self._db

    def _enc(self):
        if self._encoder is None:
            self._encoder = _LocalE5Encoder()
        return self._encoder

    def _vec(self, text: str, query: bool = False) -> list[float]:
        enc = self._enc()
        fn = getattr(enc, "embed_query", enc) if query else enc
        vec = [float(x) for x in fn(text)]
        if not vec or not all(math.isfinite(x) for x in vec):
            raise ValueError("embedding_model must return a non-empty finite vector")
        return vec

    def _key(self, scope: dict, role: str, content: str) -> str:
        canon = json.dumps(
            {"namespace": self._ns, **scope, "role": role, "content": content},
            sort_keys=True,
            ensure_ascii=False,
        )
        return hashlib.sha256(canon.encode()).hexdigest()

    def _scope(self, user_id=None, agent_id=None, run_id=None) -> dict:
        for name, value in (
            ("user_id", user_id),
            ("agent_id", agent_id),
            ("run_id", run_id),
        ):
            _check_scope(name, value)
        return {"user_id": user_id, "agent_id": agent_id, "run_id": run_id}

    def _scope_meta(self, scope: dict) -> dict:
        return {k: v for k, v in scope.items() if v is not None}

    def _own(self, record) -> bool:
        return bool(
            record
            and getattr(record, "metadata", None)
            and record.metadata.get(_MARKER) is True
        )

    def _to_item(self, record, score=None) -> dict:
        meta = dict(record.metadata or {})
        meta.pop(_MARKER, None)
        role = meta.pop(_ROLE, None)
        raw = meta.pop(_META_JSON, None)
        if raw is not None:
            try:
                original = json.loads(raw)
            except (TypeError, ValueError) as exc:
                raise ValueError(
                    f"record {record.key}: corrupt stored metadata JSON: {exc}"
                ) from exc
            if not isinstance(original, dict):
                raise ValueError(
                    f"record {record.key}: stored metadata is not an object"
                )
        else:
            original = {}
        scope = {k: meta.pop(k) for k in _SCOPES if k in meta}
        item = {
            "id": record.key,
            "memory": record.payload,
            "metadata": {**original, **meta},
            **scope,
            "created_at_ms": getattr(record, "created_at_ms", None),
            "updated_at_ms": getattr(record, "updated_at_ms", None),
        }
        if role is not None:
            item["role"] = role
        if score is not None:
            item["score"] = score
        return item

    def _validate_messages(self, messages) -> list[dict]:
        if isinstance(messages, str):
            messages = [{"role": "user", "content": messages}]
        if not isinstance(messages, list):
            raise TypeError(
                "messages must be a string or a list of {role, content} dicts"
            )
        out = []
        for i, msg in enumerate(messages):
            if not isinstance(msg, dict):
                raise TypeError(
                    f"messages[{i}] must be a dict, got {type(msg).__name__}"
                )
            role = msg.get("role")
            content = msg.get("content")
            if not isinstance(role, str) or not isinstance(content, str):
                raise TypeError(
                    f"messages[{i}] requires string 'role' and 'content' "
                    f"(got role={type(role).__name__}, content={type(content).__name__})"
                )
            out.append({"role": role, "content": content})
        return out

    @staticmethod
    def _check_limit(limit, where: str) -> None:
        if isinstance(limit, bool) or not isinstance(limit, int) or limit <= 0:
            raise ValueError(f"{where}: limit must be a positive int, got {limit!r}")

    def add(
        self,
        messages,
        user_id=None,
        agent_id=None,
        run_id=None,
        metadata=None,
        filters=None,
    ) -> list[dict]:
        parsed = self._validate_messages(messages)
        scope = self._scope(user_id, agent_id, run_id)
        if metadata is not None and not isinstance(metadata, dict):
            raise TypeError("metadata must be a dict or None")
        for k in metadata or {}:
            if k in _RESERVED:
                raise ValueError(f"metadata key '{k}' collides with a reserved key")
        original = json.dumps(metadata or {}, ensure_ascii=False, allow_nan=False)
        flat_meta = _flat(
            {
                k: v
                for k, v in (metadata or {}).items()
                if isinstance(v, (str, int, float, bool))
            },
            "metadata",
        )
        extra_filters = _flat(filters, "filters")
        conflicts = []
        for k, fv in extra_filters.items():
            if k not in (metadata or {}):
                continue
            mv = (metadata or {})[k]
            try:
                same = json.dumps(mv, sort_keys=True, allow_nan=False) == json.dumps(
                    fv, sort_keys=True, allow_nan=False
                )
            except (TypeError, ValueError):
                same = False
            if not same:
                conflicts.append(k)
        if conflicts:
            raise ValueError(
                f"filters conflict with metadata values: {sorted(conflicts)}"
            )

        db = self._client()
        out = []
        for msg in parsed:
            content = msg["content"]
            if not content:
                continue
            key = self._key(scope, msg["role"], content)
            existed = db.memory.get(self._ns, key) is not None
            db.put(
                self._ns,
                key,
                content,
                metadata={
                    _MARKER: True,
                    _META_JSON: original,
                    _ROLE: msg["role"],
                    **self._scope_meta(scope),
                    **flat_meta,
                    **extra_filters,
                },
                vector=self._vec(content),
            )
            out.append(
                {
                    "id": key,
                    "memory": content,
                    "event": "UPDATE" if existed else "ADD",
                    "metadata": metadata or {},
                }
            )
        return out

    def _and_filters(self, scope: dict, extra: dict) -> dict:
        return {_MARKER: True, **self._scope_meta(scope), **extra}

    def search(
        self,
        query,
        user_id=None,
        agent_id=None,
        run_id=None,
        limit: int = 5,
        filters=None,
    ) -> list[dict]:
        self._check_limit(limit, "search")
        extra = _flat(filters, "filters")
        scope = self._scope(user_id, agent_id, run_id)
        db = self._client()
        hits = db.search(
            self._ns,
            query_vector=self._vec(str(query), query=True),
            text_query=str(query),
            filters=self._and_filters(scope, extra),
            top_k=limit,
        )
        items = []
        for h in hits or []:
            rec = h.record if hasattr(h, "record") else h
            if self._own(rec):
                items.append(self._to_item(rec, score=getattr(h, "score", None)))
        return items

    def get_all(
        self, user_id=None, agent_id=None, run_id=None, limit: int = 100, filters=None
    ) -> list[dict]:
        self._check_limit(limit, "get_all")
        extra = _flat(filters, "filters")
        scope = self._scope(user_id, agent_id, run_id)
        page = self._client().memory.list(
            self._ns, filters=self._and_filters(scope, extra), limit=limit
        )
        records = getattr(page, "records", page) or []
        return [self._to_item(r) for r in records if self._own(r)]

    def get(self, memory_id: str):
        record = self._client().memory.get(self._ns, memory_id)
        return self._to_item(record) if self._own(record) else None

    def delete(self, memory_id: str) -> bool:
        db = self._client()
        record = db.memory.get(self._ns, memory_id)
        if not self._own(record):
            return False
        return bool(db.memory.delete(self._ns, memory_id))

    def reset(self) -> dict:
        db = self._client()
        deleted = 0
        while True:
            page = db.memory.list(self._ns, filters={_MARKER: True}, limit=100)
            raw = getattr(page, "records", page) or []
            records = [r for r in raw if self._own(r)]
            if not records:
                if raw:
                    raise RuntimeError(
                        "reset made no progress: page contains only non-adapter records"
                    )
                break
            batch = 0
            for r in records:
                if db.memory.delete(self._ns, r.key):
                    batch += 1
            if batch == 0:
                raise RuntimeError(
                    "reset made no progress: deletable records returned but none deleted"
                )
            deleted += batch
        return {"deleted": deleted}

    def flush(self) -> None:
        if self._db is not None:
            self._db.flush()

    def close(self) -> None:
        if self._db is not None:
            try:
                self._db.flush()
            finally:
                self._db.close()
                self._db = None

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()
        return False


if __name__ == "__main__":
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        print("Local demo - encoder: multilingual-e5-small (local ONNX, no network)")
        mem = VantaMemory(db_path=str(Path(tmp) / "demo"))
        try:
            mem.add("The user prefers concise answers", user_id="demo")
            mem.add(
                [
                    {"role": "user", "content": "The user stores data in SQLite"},
                    {"role": "user", "content": "The user develops agents in Python"},
                ],
                user_id="demo",
            )
            hits = mem.search("user preferences", user_id="demo")
            for h in hits:
                print(f"  hit score={h['score']:.3f} :: {h['memory']}")
            assert mem.get(hits[0]["id"]) is not None
            assert len(mem.get_all(user_id="demo")) == 3
        except (ImportError, OSError, RuntimeError, ValueError) as exc:
            print(f"Demo could not run: {exc}")
            raise SystemExit(1)
        finally:
            mem.close()

        # Reopen to prove real persistence.
        with VantaMemory(db_path=str(Path(tmp) / "demo")) as mem2:
            assert len(mem2.get_all(user_id="demo")) == 3
            print("Reopen OK - persistent records in", tmp)
