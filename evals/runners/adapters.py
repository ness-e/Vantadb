#!/usr/bin/env python3
"""evals/runners/adapters.py — VER-09 external memory-system runners.

Maps third-party memory SDKs onto the SAME store contract the VER-08 harness
(`evals/memory_harness.py`) already consumes, so `run_harness` computes IDENTICAL
metrics (recall_all@k / recall_any@k session-level, p50/p99, ingest QPS,
write-quality, abstention proxy, token-economy, isolation) for every system.

Store contract (defined by `evals/memory_harness.py:174-357` — reused, never forked):
    name: str
    put(ns, key, text) -> None
    get(ns, key) -> {"payload", "namespace", "confidence"} | None
    search(ns, query, top_k) -> [{"key", "score", "confidence", "namespace", "payload"}]
    close() -> None

Systems and pinned versions (protocol §Systems — verified against official docs
2026-09-30; see `evals/runners/README.md`):

  * mem0 OSS (`mem0ai==2.2.1`, Apache-2.0) — raw mode (`infer=False`) runs with
    ZERO LLM calls: local fastembed embeddings + embedded qdrant. Native mode
    (LLM extraction) needs a provider key (skipped when absent).
  * Zep (`zep-cloud==3.30.0` client -> Zep Cloud API) — needs ZEP_API_KEY +
    credits; messages carry metadata that projects onto episodes, so session-level
    evidence recall maps 1:1. Skipped without a key (no self-host here: Graphiti
    needs Neo4j/FalkorDB + LLM keys).
  * Letta (`letta==0.33.8`) — an agent harness (agent-managed memory: MemFS +
    dreaming), not a put/search store; the contract does not map. Skipped with a
    documented rationale (QA-level protocol deferred, FIND).

Skip policy (plan stop condition): a runner that cannot be pinned/installed or
whose LLM cost exceeds budget is reported as a DOCUMENTED SKIP with its reason —
never silently dropped, never fabricated.
"""
from __future__ import annotations

import importlib
import importlib.metadata
import importlib.util
import os
import pathlib
import tempfile
import time

# ---------------------------------------------------------------------------
# Pins + constants (single source for the protocol/report metadata)
# ---------------------------------------------------------------------------
PINNED = {
    "mem0": "mem0ai==2.2.1 (fastembed==0.8.1, qdrant-client==1.19.1)",
    "zep": "zep-cloud==3.30.0 (Zep Cloud API)",
    "letta": "letta==0.33.8 (agent harness)",
}
MEM0_EMBED_MODEL = "BAAI/bge-small-en-v1.5"  # 384d, local ONNX (fastembed)
MEM0_EMBED_DIMS = 384
ZEP_MAX_MSG_CHARS = 4000  # API hard limit is 4,096 chars/message — leave margin
ZEP_MAX_MSGS_PER_CALL = 30  # API hard limit
SYSTEMS = ("vantadb", "mem0", "zep", "letta")

_import = importlib.import_module  # patch point for the offline self-test


def _has_module(name: str) -> bool:
    """Best-effort module presence check (patchable in the offline self-test)."""
    try:
        return importlib.util.find_spec(name) is not None
    except (ImportError, ValueError):
        return False


class RunnerSkipped(RuntimeError):
    """A runner that cannot run in this environment — documented skip, not failure."""

    def __init__(self, system: str, reason: str) -> None:
        super().__init__(f"{system}: {reason}")
        self.system = system
        self.reason = reason


def _pkg_version(name: str) -> str:
    try:
        return importlib.metadata.version(name)
    except Exception:
        return "unknown"


def chunk_text(text: str, limit: int) -> list[str]:
    """Split `text` into <= limit-char chunks on line boundaries (lossless).

    Only used where an API imposes a HARD message-size limit (Zep 4,096 chars);
    never as an optional optimization — the protocol ingests the same session
    text everywhere the API allows it.
    """
    if len(text) <= limit:
        return [text]
    chunks: list[str] = []
    rest = text
    while len(rest) > limit:
        cut = rest.rfind("\n", 0, limit)
        if cut <= 0:
            cut = limit
        chunks.append(rest[:cut])
        rest = rest[cut:]
    if rest:
        chunks.append(rest)
    return chunks


def _md_first(metadata, name: str):
    """Read a metadata value that may be scalar, list or 'projected' list."""
    if not isinstance(metadata, dict):
        return None
    value = metadata.get(name)
    if isinstance(value, (list, tuple)):
        return value[0] if value else None
    return value


# ---------------------------------------------------------------------------
# mem0 OSS
# ---------------------------------------------------------------------------
class Mem0Store:
    """mem0 OSS adapter.

    mode="raw"    → `add(..., infer=False)`: stores the payload as provided;
                    zero LLM calls (declared protocol config for the VER-09 run).
    mode="native" → default product pipeline (LLM fact extraction, pinned
                    gpt-4o-mini); requires an OpenAI key and budget.
    """

    name = "mem0"
    supports_write_quality = True  # raw mode: exact payload retrievable by key

    def __init__(self, mode: str = "raw", data_dir: str | None = None) -> None:
        if mode not in ("raw", "native"):
            raise ValueError(f"mem0 mode must be raw|native, got {mode!r}")
        if mode == "native" and not os.environ.get("OPENAI_API_KEY"):
            raise RunnerSkipped(
                "mem0",
                "native mode needs an LLM provider key (OPENAI_API_KEY); "
                "use --mem0-mode raw (zero LLM calls)",
            )
        self.mode = mode
        self.data_dir = pathlib.Path(data_dir or tempfile.mkdtemp(prefix="mem0_h2h_"))
        # Best-effort hygiene: telemetry off + keep mem0 state under the run dir.
        os.environ.setdefault("MEM0_TELEMETRY", "False")
        os.environ.setdefault("HF_HUB_DISABLE_SYMLINKS_WARNING", "1")
        os.environ.setdefault("ANONYMIZED_TELEMETRY", "False")
        os.environ.setdefault("MEM0_DIR", str(self.data_dir))
        try:
            mem0 = _import("mem0")
        except ImportError as exc:  # noqa: TRY003
            raise RunnerSkipped(
                "mem0", "pip install mem0ai==2.2.1 fastembed==0.8.1 (not importable)"
            ) from exc
        self.version = _pkg_version("mem0ai")
        config = {
            "llm": {
                "provider": "openai",
                "config": {
                    "model": "gpt-4o-mini",
                    # Raw mode never calls the LLM; native mode uses the real key.
                    "api_key": os.environ.get("OPENAI_API_KEY", "sk-raw-mode-never-called"),
                },
            },
            "embedder": {
                "provider": "fastembed",
                "config": {"model": MEM0_EMBED_MODEL},
            },
            "vector_store": {
                "provider": "qdrant",
                "config": {
                    # Embedded, in-memory: no disk residue, no fsync noise in the
                    # ingest/query latencies (declared in the protocol §Limits).
                    "path": ":memory:",
                    "collection_name": "h2h_mem0",
                    "embedding_model_dims": MEM0_EMBED_DIMS,
                },
            },
        }
        try:
            self.memory = mem0.Memory.from_config(config)
        except ImportError as exc:
            raise RunnerSkipped(
                "mem0",
                f"mem0 dependency missing at from_config ({exc}); "
                "pip install mem0ai==2.2.1 fastembed==0.8.1",
            ) from exc
        self._all_cache: dict[str, list] = {}

    # -- contract -----------------------------------------------------------
    def put(self, ns: str, key: str, text: str) -> None:
        self.memory.add(
            [{"role": "user", "content": text}],
            user_id=ns,
            metadata={"session_id": key},
            infer=self.mode == "native",
        )

    def get(self, ns: str, key: str) -> dict | None:
        for record in self._get_all(ns):
            if _md_first(record.get("metadata"), "session_id") == key:
                return {
                    "payload": record.get("memory") or "",
                    "namespace": ns,
                    "confidence": 1.0,
                }
        return None

    def search(self, ns: str, query: str, top_k: int) -> list[dict]:
        # mem0 2.2.1 signature: search(query, *, top_k=20, filters=..., threshold=0.1)
        # The system's default similarity threshold (0.1) is kept — declared.
        result = self.memory.search(query, top_k=top_k, filters={"user_id": ns})
        items = result.get("results", result) if isinstance(result, dict) else result
        hits = []
        for record in items or []:
            score = float(record.get("score") or 0.0)
            hits.append(
                {
                    "key": _md_first(record.get("metadata"), "session_id")
                    or record.get("id", "?"),
                    "score": score,
                    "confidence": score,  # raw score; ECE not reported for externals
                    "namespace": ns,
                    "payload": record.get("memory") or "",
                }
            )
        return hits[:top_k]

    def close(self) -> None:
        for target in (
            getattr(self.memory, "close", None),
            getattr(getattr(self.memory, "vector_store", None), "client", None),
        ):
            if target is None:
                continue
            try:
                target()
            except TypeError:
                try:
                    target.close()
                except Exception:
                    pass
            except Exception:
                pass

    # -- internals ----------------------------------------------------------
    def _get_all(self, ns: str) -> list:
        # ponytail: unbounded per-namespace cache — fine for one eval run
        # (the harness sweeps ns by ns); cap/evict if this ever runs long-lived.
        if ns not in self._all_cache:
            # get_all() defaults to top_k=20 — request far above the per-ns
            # session count so the full namespace is returned.
            result = self.memory.get_all(filters={"user_id": ns}, top_k=10_000)
            self._all_cache[ns] = (
                result.get("results", result) if isinstance(result, dict) else result
            ) or []
        return self._all_cache[ns]

    def config_summary(self) -> dict:
        return {
            "system": "mem0",
            "mode": self.mode,
            "pins": PINNED["mem0"],
            "embedder": f"fastembed {MEM0_EMBED_MODEL} ({MEM0_EMBED_DIMS}d)",
            "vector_store": "qdrant-client embedded (in-memory ':memory:')",
            "search": "search(top_k=k, filters={user_id}) — default similarity threshold 0.1 kept",
            "llm_calls": "none (raw: infer=False)" if self.mode == "raw" else "gpt-4o-mini (native)",
            "config_note": (
                "raw mode = extraction disabled; NOT mem0's default product pipeline"
                if self.mode == "raw"
                else "native = default product pipeline (LLM fact extraction)"
            ),
        }


# ---------------------------------------------------------------------------
# Zep Cloud
# ---------------------------------------------------------------------------
class ZepStore:
    """Zep Cloud adapter (zep-cloud SDK).

    put    → thread.create(thread_id=ns, user_id=ns) + thread.add_messages with
             metadata {"session_id": key} (messages are chunked to the API's
             4,096-char hard limit — declared adaptation; metadata is preserved
             and projections make episode search session-level comparable).
    search → graph.search(user_id=ns, query, scope="episodes", limit=top_k).
    get    → not supported (extraction-based service, no exact-key retrieval):
             write-quality is N/A for this system (declared).
    """

    name = "zep"
    supports_write_quality = False

    def __init__(self) -> None:
        key = os.environ.get("ZEP_API_KEY")
        if not key:
            raise RunnerSkipped(
                "zep",
                "needs ZEP_API_KEY (Zep Cloud; paid credits). Self-host alternative "
                "= Graphiti + Neo4j/FalkorDB + LLM keys — also unavailable here",
            )
        try:
            client_mod = _import("zep_cloud.client")
            types_mod = _import("zep_cloud.types")
        except ImportError as exc:  # noqa: TRY003
            raise RunnerSkipped("zep", "pip install zep-cloud==3.30.0 (not importable)") from exc
        self._Message = types_mod.Message
        self.client = client_mod.Zep(api_key=key)
        self.version = _pkg_version("zep-cloud")

    # -- contract -----------------------------------------------------------
    def put(self, ns: str, key: str, text: str) -> None:
        try:
            self.client.thread.create(thread_id=ns, user_id=ns)
        except Exception:
            pass  # thread already exists — the SDK raises on duplicate ids
        chunks = chunk_text(text, ZEP_MAX_MSG_CHARS)
        for start in range(0, len(chunks), ZEP_MAX_MSGS_PER_CALL):
            batch = [
                self._Message(
                    name="session",
                    role="user",
                    content=piece,
                    metadata={"session_id": key, "chunk": start + offset},
                )
                for offset, piece in enumerate(chunks[start : start + ZEP_MAX_MSGS_PER_CALL])
            ]
            self.client.thread.add_messages(thread_id=ns, messages=batch)

    def get(self, ns: str, key: str) -> dict | None:
        return None  # exact-key retrieval not supported; write-quality N/A

    def search(self, ns: str, query: str, top_k: int) -> list[dict]:
        result = self.client.graph.search(
            user_id=ns,
            query=query[:400],  # API hard limit: 400-char queries
            scope="episodes",
            limit=top_k,
        )
        hits = []
        for episode in getattr(result, "episodes", None) or []:
            score = float(getattr(episode, "score", None) or 0.0)
            hits.append(
                {
                    "key": _md_first(getattr(episode, "metadata", None), "session_id")
                    or getattr(episode, "uuid_", "?"),  # zep_cloud Episode field is `uuid_`
                    "score": score,
                    "confidence": score,
                    "namespace": ns,
                    "payload": getattr(episode, "content", "") or "",
                }
            )
        return hits[:top_k]

    def close(self) -> None:
        pass

    def config_summary(self) -> dict:
        return {
            "system": "zep",
            "pins": PINNED["zep"],
            "ingest": "thread.create + add_messages (chunked ≤4000 chars, metadata session_id)",
            "search": "graph.search(scope=episodes, limit=top_k)",
            "llm_calls": "Zep Cloud (credits) — extraction managed server-side",
        }


# ---------------------------------------------------------------------------
# Letta
# ---------------------------------------------------------------------------
class LettaStore:
    """Letta is an agent harness, not a put/search store — documented skip.

    Letta 0.33.x (self-hosted `letta --backend local` or Letta Cloud) manages
    memory INSIDE the agent loop (memory blocks, git-versioned MemFS, dreaming)
    behind a model provider. There is no ingest-text → retrieve-by-similarity
    primitive to map onto the harness contract, and no provider is available in
    this environment. A QA-level (LLM-in-the-loop) protocol is deferred (FIND).
    """

    name = "letta"
    supports_write_quality = False

    def __init__(self) -> None:
        raise RunnerSkipped(
            "letta",
            "agent harness (agent-managed memory: MemFS/dreaming), not a put/search "
            "store — contract does not map; needs a model provider (none here). "
            "QA-level protocol deferred (FIND)",
        )


# ---------------------------------------------------------------------------
# Registry / availability
# ---------------------------------------------------------------------------
def check_availability(mem0_mode: str = "raw") -> list[dict]:
    """Report, without running anything, whether each system can execute here."""
    out = []
    for name in SYSTEMS:
        if name == "vantadb":
            try:
                mod = _import("vantadb")
                out.append({
                    "system": name,
                    "status": "ok",
                    "reason": "",
                    "version": getattr(mod, "__version__", _pkg_version("vantadb")),
                })
            except ImportError:
                out.append({"system": name, "status": "skip", "reason": "vantadb bindings not importable", "version": ""})
        elif name == "mem0":
            try:
                _import("mem0")
            except ImportError:
                out.append({"system": name, "status": "skip", "reason": "pip install mem0ai==2.2.1 fastembed==0.8.1", "version": ""})
                continue
            if not _has_module("fastembed"):
                out.append({"system": name, "status": "skip", "reason": "fastembed not importable (pip install fastembed==0.8.1)", "version": _pkg_version("mem0ai")})
            elif mem0_mode == "native" and not os.environ.get("OPENAI_API_KEY"):
                out.append({"system": name, "status": "skip", "reason": "native mode needs OPENAI_API_KEY; raw mode needs none", "version": _pkg_version("mem0ai")})
            else:
                out.append({"system": name, "status": "ok", "reason": f"mode={mem0_mode}", "version": _pkg_version("mem0ai")})
        elif name == "zep":
            if not os.environ.get("ZEP_API_KEY"):
                out.append({"system": name, "status": "skip", "reason": "needs ZEP_API_KEY (Zep Cloud; credits)", "version": _pkg_version("zep-cloud")})
            else:
                out.append({"system": name, "status": "ok", "reason": "", "version": _pkg_version("zep-cloud")})
        elif name == "letta":
            out.append({"system": name, "status": "skip", "reason": "agent harness — store contract does not map; needs provider (see README)", "version": _pkg_version("letta")})
    return out


def build_store(system: str, mem0_mode: str = "raw", data_dir: str | None = None):
    """Build a store for `system`, or raise RunnerSkipped with the reason."""
    if system == "vantadb":
        # Reuse the VER-08 adapter verbatim (no fork).
        harness = _import("memory_harness")
        return harness.VantaStore(backend="memory")
    if system == "mem0":
        return Mem0Store(mode=mem0_mode, data_dir=data_dir)
    if system == "zep":
        return ZepStore()
    if system == "letta":
        return LettaStore()
    raise ValueError(f"unknown system {system!r} (known: {', '.join(SYSTEMS)})")


def utc_now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
