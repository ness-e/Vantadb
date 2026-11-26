"""Synthetic, serial agent-memory microbenchmark with isolated engine processes.

Precomputed 384d topic-clustered vectors are fixtures, not model embeddings.
Each five operations contain one write and four session-filtered hybrid reads.
The first read after each write checks read-your-write visibility by key.
Setup, corpus generation, validation and final close are outside timed calls.
Native retrieval plans, candidate budgets and durability acknowledgements differ.
An additional four-writer probe measures observed contention failures separately.
This is neither a semantic-quality evaluation nor a crash test.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import importlib.metadata
import json
import logging
import math
import os
import platform
import random
import sqlite3
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Barrier, Lock
from typing import Any

DIMENSIONS = 384
logger = logging.getLogger(__name__)
INITIAL_RECORDS = 1024
SESSIONS = 8
TOP_K = 5
CANDIDATES = TOP_K
RRF_K = 60
CONTENTION_WORKERS = 4
CONTENTION_WRITES_PER_WORKER = 25
SQLITE_BUSY_TIMEOUT_SECONDS = 5.0
TARGETS = ("vantadb", "sqlite-vec", "lancedb")
TOPICS = (
    "formatting",
    "deployment",
    "archival",
    "security",
    "migration",
    "retrieval",
    "scheduling",
    "testing",
)


class MissingDependency(ImportError):
    """A requested optional engine is not installed."""


def require_module(module: str, distribution: str) -> Any:
    try:
        return importlib.import_module(module)
    except ModuleNotFoundError as exc:
        if exc.name == module:
            raise MissingDependency(
                f"{distribution} is not installed; install it with "
                f"python -m pip install {distribution}"
            ) from exc
        raise


def percentile(values: list[float], percent: int) -> float | None:
    """Return the nearest-rank percentile, in the input unit."""
    if not values:
        return None
    ordered = sorted(values)
    return ordered[max(0, math.ceil(len(ordered) * percent / 100) - 1)]


def summarize(values: list[float], percents: tuple[int, ...]) -> dict:
    return {
        "successful_samples": len(values),
        **{f"p{p}_ms": percentile(values, p) for p in percents},
    }


def build_workload(turns: int, seed: int) -> dict:
    """Freeze one trace shared byte-for-byte by every engine worker."""
    rng = random.Random(seed)

    def record(identifier: int) -> dict:
        topic_index = rng.randrange(len(TOPICS))
        vector = [rng.gauss(0, 0.03) for _ in range(DIMENSIONS)]
        vector[topic_index] += 1.0
        norm = math.sqrt(sum(v * v for v in vector))
        role = "user" if identifier % 2 else "assistant"
        topic = TOPICS[topic_index]
        anchor = f"memoryrecord{identifier:07d}"
        return {
            "id": identifier,
            "session_id": identifier % SESSIONS,
            "role": role,
            "timestamp": 1_700_000_000_000 + identifier * 1000,
            "text": f"{role} discussed project {topic} preferences and decisions {anchor}",
            "vector": [v / norm for v in vector],
            "topic": topic,
            "anchor": anchor,
        }

    writes = turns // 5
    records = [record(i) for i in range(1, INITIAL_RECORDS + writes + 1)]
    operations = []
    available = INITIAL_RECORDS
    inserted = 0
    previous_was_write = False
    for index in range(turns):
        if index % 5 == 0 and inserted < writes:
            available += 1
            inserted += 1
            operations.append({"kind": "write", "id": available})
            previous_was_write = True
        else:
            identifier = available if previous_was_write else rng.randint(1, available)
            row = records[identifier - 1]
            operations.append(
                {
                    "kind": "read",
                    "id": identifier,
                    "text": row["anchor"] if previous_was_write else row["topic"],
                    "read_your_write": previous_was_write,
                }
            )
            previous_was_write = False
    contention_records = [
        record(INITIAL_RECORDS + writes + i + 1)
        for i in range(CONTENTION_WORKERS * CONTENTION_WRITES_PER_WORKER)
    ]
    return {
        "schema_version": 1,
        "seed": seed,
        "turns": turns,
        "initial_records": INITIAL_RECORDS,
        "dimensions": DIMENSIONS,
        "sessions": SESSIONS,
        "top_k": TOP_K,
        "records": records,
        "operations": operations,
        "contention_probe": {
            "workers": CONTENTION_WORKERS,
            "writes_per_worker": CONTENTION_WRITES_PER_WORKER,
            "records": contention_records,
        },
    }


def encode_workload(workload: dict) -> bytes:
    return json.dumps(workload, separators=(",", ":"), allow_nan=False).encode("utf-8")


def reciprocal_rank_fusion(rankings: list[list[int]], limit: int = TOP_K) -> list[int]:
    scores: dict[int, float] = {}
    for ranking in rankings:
        for rank, identifier in enumerate(ranking, 1):
            scores[identifier] = scores.get(identifier, 0.0) + 1.0 / (RRF_K + rank)
    return sorted(scores, key=lambda identifier: (-scores[identifier], identifier))[
        :limit
    ]


class VantaAdapter:
    """Native in-process BM25/vector search through the Python SDK."""

    def __init__(self, path: str, records: list[dict]) -> None:
        module = require_module("vantadb", "vantadb-py")
        self.db = module.Client(path)
        self.config = {
            "distribution": "vantadb-py",
            "version": importlib.metadata.version("vantadb-py"),
            "backend": "fjall",
            "fusion": "native engine defaults",
            "requested_distance": "cosine",
            "durability": "SDK defaults; API acknowledgement, not a per-write fsync claim",
        }
        for record in records:
            self.write(record)

    def write(self, row: dict) -> None:
        self.db.memory.put(
            "agent/benchmark",
            str(row["id"]),
            row["text"],
            vector=row["vector"],
            metadata={k: row[k] for k in ("session_id", "role", "timestamp")},
        )

    def search(self, row: dict, text: str) -> list[dict]:
        hits = self.db.memory.search(
            "agent/benchmark",
            query_vector=row["vector"],
            text_query=text,
            filters={"session_id": row["session_id"]},
            distance_metric="cosine",
            top_k=TOP_K,
        )
        return [
            {"id": int(hit.key), "session_id": hit.metadata["session_id"]}
            for hit in hits
        ]

    def close(self) -> None:
        self.db.close()


class SqliteVecAdapter:
    """vec0 cosine KNN plus FTS5 BM25, with application-side RRF."""

    def __init__(self, path: str, records: list[dict] | None) -> None:
        self.vec = require_module("sqlite_vec", "sqlite-vec")
        self.directory = path
        self.db = sqlite3.connect(
            str(Path(path) / "memory.sqlite"), timeout=SQLITE_BUSY_TIMEOUT_SECONDS
        )
        try:
            self.db.enable_load_extension(True)
            try:
                self.vec.load(self.db)
            finally:
                self.db.enable_load_extension(False)
            if records is not None:
                self.db.executescript(
                    """
                PRAGMA journal_mode=WAL;
                PRAGMA synchronous=NORMAL;
                CREATE TABLE memories (
                    id INTEGER PRIMARY KEY,
                    session_id INTEGER NOT NULL,
                    role TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    text TEXT NOT NULL
                );
                CREATE INDEX memory_session ON memories(session_id);
                CREATE VIRTUAL TABLE vectors USING vec0(
                    id INTEGER PRIMARY KEY,
                    embedding FLOAT[384] distance_metric=cosine,
                    session_id INTEGER
                );
                CREATE VIRTUAL TABLE terms USING fts5(
                    text, content='memories', content_rowid='id'
                );
                """
                )
                with self.db:
                    for record in records:
                        self._insert(record)
            else:
                self.db.execute("PRAGMA synchronous=NORMAL")
        except Exception:
            self.db.close()
            raise
        self.config = {
            "distribution": "sqlite-vec",
            "version": importlib.metadata.version("sqlite-vec"),
            "sqlite_version": sqlite3.sqlite_version,
            "journal_mode": "WAL",
            "synchronous": "NORMAL",
            "busy_timeout_seconds": SQLITE_BUSY_TIMEOUT_SECONDS,
            "distance": "cosine",
            "vector_search": "exact vec0",
            "fusion": "application RRF",
            "rrf_k": RRF_K,
            "candidates_per_branch": CANDIDATES,
            "prefilter": "session_id equality on both branches before limit",
        }

    def _insert(self, row: dict) -> None:
        self.db.execute(
            "INSERT INTO memories(id, session_id, role, timestamp, text) VALUES(?,?,?,?,?)",
            [row[k] for k in ("id", "session_id", "role", "timestamp", "text")],
        )
        self.db.execute(
            "INSERT INTO vectors(id, embedding, session_id) VALUES(?,?,?)",
            (row["id"], self.vec.serialize_float32(row["vector"]), row["session_id"]),
        )
        self.db.execute(
            "INSERT INTO terms(rowid, text) VALUES(?,?)", (row["id"], row["text"])
        )

    def write(self, row: dict) -> None:
        with self.db:
            self._insert(row)

    def search(self, row: dict, text: str) -> list[dict]:
        vector_ids = [
            item[0]
            for item in self.db.execute(
                """
                SELECT id FROM vectors
                WHERE embedding MATCH ? AND k = ? AND session_id = ?
                ORDER BY distance
                """,
                (
                    self.vec.serialize_float32(row["vector"]),
                    CANDIDATES,
                    row["session_id"],
                ),
            )
        ]
        text_ids = [
            item[0]
            for item in self.db.execute(
                """
                SELECT memories.id FROM terms
                JOIN memories ON memories.id = terms.rowid
                WHERE terms MATCH ? AND memories.session_id = ?
                ORDER BY bm25(terms), memories.id LIMIT ?
                """,
                (text, row["session_id"], CANDIDATES),
            )
        ]
        identifiers = reciprocal_rank_fusion([vector_ids, text_ids])
        if not identifiers:
            return []
        placeholders = ",".join("?" for _ in identifiers)
        rows = dict(
            self.db.execute(
                f"SELECT id, session_id FROM memories WHERE id IN ({placeholders})",
                identifiers,
            )
        )
        return [
            {"id": identifier, "session_id": rows[identifier]}
            for identifier in identifiers
        ]

    def close(self) -> None:
        self.db.close()


class LanceAdapter:
    """Native LanceDB vector/FTS query with explicit reciprocal-rank fusion."""

    def __init__(self, path: str, records: list[dict]) -> None:
        lancedb = require_module("lancedb", "lancedb")
        import pyarrow as pa
        from lancedb.index import FTS
        from lancedb.rerankers import RRFReranker

        self.connection = lancedb.connect(path)
        schema = pa.schema(
            [
                pa.field("id", pa.int64()),
                pa.field("session_id", pa.int64()),
                pa.field("role", pa.string()),
                pa.field("timestamp", pa.int64()),
                pa.field("text", pa.string()),
                pa.field("vector", pa.list_(pa.float32(), DIMENSIONS)),
            ]
        )
        self.table = self.connection.create_table(
            "memories", data=[self._data(row) for row in records], schema=schema
        )
        self.table.create_index("text", config=FTS(stem=False, remove_stop_words=False))
        self.reranker = RRFReranker(K=RRF_K)
        self.config = {
            "distribution": "lancedb",
            "version": importlib.metadata.version("lancedb"),
            "backend": "embedded local filesystem",
            "distance": "cosine",
            "vector_search": "exact flat, no ANN index",
            "text_search": "native FTS; includes default unindexed-tail handling",
            "fusion": "native query with RRFReranker",
            "rrf_k": RRF_K,
            "candidates_per_branch": TOP_K,
            "prefilter": True,
            "durability": "default table.add acknowledgement",
        }

    @staticmethod
    def _data(row: dict) -> dict:
        return {
            k: row[k]
            for k in ("id", "session_id", "role", "timestamp", "text", "vector")
        }

    def write(self, row: dict) -> None:
        self.table.add([self._data(row)])

    def search(self, row: dict, text: str) -> list[dict]:
        return (
            self.table.search(
                query_type="hybrid", vector_column_name="vector", fts_columns="text"
            )
            .vector(row["vector"])
            .text(text)
            .distance_type("cosine")
            .where(f"session_id = {int(row['session_id'])}", prefilter=True)
            .rerank(self.reranker)
            .limit(TOP_K)
            .to_list()
        )

    def close(self) -> None:
        self.table = None
        self.connection = None


ADAPTERS = {
    "vantadb": VantaAdapter,
    "sqlite-vec": SqliteVecAdapter,
    "lancedb": LanceAdapter,
}


def is_lock_error(error: Exception) -> bool:
    if isinstance(error, sqlite3.OperationalError):
        return getattr(error, "sqlite_errorcode", 0) & 255 in (
            sqlite3.SQLITE_BUSY,
            sqlite3.SQLITE_LOCKED,
        )
    message = str(error).lower()
    return any(
        phrase in message
        for phrase in (
            "database is locked",
            "database table is locked",
            "failed to acquire lock",
            "lock already held",
            "storage is busy",
        )
    )


def run_worker(target: str, workload_path: Path) -> dict:
    psutil = require_module("psutil", "psutil")
    process = psutil.Process()
    raw = workload_path.read_bytes()
    workload = json.loads(raw)
    records = {row["id"]: row for row in workload["records"]}
    latencies: dict[str, list[float]] = {"write": [], "read": []}
    errors = []
    attempted = 0
    read_your_write_checks = 0
    rss_initial = process.memory_info().rss
    started = time.perf_counter_ns()
    result = {
        "target": target,
        "status": "error",
        "workload_sha256": hashlib.sha256(raw).hexdigest(),
        "concurrent_workers": 1,
        "rss_initial_bytes": rss_initial,
    }
    adapter = None
    with tempfile.TemporaryDirectory(prefix=f"vanta-agent-{target}-") as directory:
        try:
            initial = workload["records"][: workload["initial_records"]]
            adapter = ADAPTERS[target](directory, initial)
            result["configuration"] = adapter.config
            result["setup_ms"] = (time.perf_counter_ns() - started) / 1e6
            result["rss_after_seed_bytes"] = process.memory_info().rss
            for operation in workload["operations"]:
                attempted += 1
                row = records[operation["id"]]
                began = time.perf_counter_ns()
                try:
                    if operation["kind"] == "write":
                        adapter.write(row)
                        hits = None
                    else:
                        hits = adapter.search(row, operation["text"])
                    elapsed = (time.perf_counter_ns() - began) / 1e6
                    if hits is not None:
                        if not hits:
                            raise AssertionError(
                                "Filtered hybrid query returned no results"
                            )
                        if any(hit["session_id"] != row["session_id"] for hit in hits):
                            raise AssertionError("Session isolation check failed")
                        if operation["read_your_write"]:
                            if row["id"] not in {hit["id"] for hit in hits}:
                                raise AssertionError(
                                    "Read-your-write witness was not retrieved"
                                )
                            read_your_write_checks += 1
                    latencies[operation["kind"]].append(elapsed)
                except Exception as exc:
                    logger.exception("Benchmark operation failed for %s", target)
                    locked = is_lock_error(exc)
                    errors.append(
                        {
                            "operation": attempted,
                            "kind": operation["kind"],
                            "type": type(exc).__name__,
                            "message": str(exc),
                            "lock": locked,
                        }
                    )
                    if not locked:
                        break
            rss_final = process.memory_info().rss
            lock_count = sum(error["lock"] for error in errors)
            result.update(
                status="ok"
                if not errors and attempted == workload["turns"]
                else "error",
                attempted_operations=attempted,
                write=summarize(latencies["write"], (50, 95, 99)),
                hybrid_read=summarize(latencies["read"], (50, 95)),
                latencies_ms=latencies,
                rss_final_bytes=rss_final,
                rss_delta_bytes=rss_final - rss_initial,
                lock_errors=lock_count,
                observed_lock_error_rate=lock_count / attempted if attempted else None,
                contention_exercised=False,
                read_your_write_checks=read_your_write_checks,
                errors=errors,
            )
            if result["status"] == "ok":
                result["contention_probe"] = run_contention_probe(
                    adapter, workload["contention_probe"]
                )
                result["contention_exercised"] = result["contention_probe"][
                    "overlap_observed"
                ]
                result["rss_after_contention_bytes"] = process.memory_info().rss
                if (
                    result["contention_probe"]["status"] != "completed"
                    or result["contention_probe"]["other_write_errors"]
                ):
                    result.update(
                        status="error",
                        reason="Contention probe was incomplete or had non-lock errors",
                    )
        except MissingDependency as exc:
            result.update(
                status="skipped" if target != "vantadb" else "error", reason=str(exc)
            )
        except Exception as exc:
            logger.exception("Benchmark setup or workload failed for %s", target)
            optional_setup_failure = adapter is None and target != "vantadb"
            result.update(
                status="skipped" if optional_setup_failure else "error",
                phase="setup" if adapter is None else "workload",
                reason=f"{type(exc).__name__}: {exc}",
            )
        finally:
            if adapter is not None:
                try:
                    adapter.close()
                except Exception as exc:
                    logger.exception("Benchmark close failed for %s", target)
                    result.update(
                        status="error", cleanup_error=f"{type(exc).__name__}: {exc}"
                    )
    return result


def run_contention_probe(adapter: Any, specification: dict) -> dict:
    """Run a separate write-only probe, without altering serial latency samples."""
    workers = specification["workers"]
    rows = specification["records"]
    gate = Barrier(workers)
    mutex = Lock()
    activity = {"inflight": 0, "peak": 0}

    def worker(index: int) -> dict:
        view = adapter
        owns_view = False
        result = {"attempted": 0, "successful": 0, "errors": [], "lifecycle_errors": []}
        try:
            if isinstance(adapter, SqliteVecAdapter):
                view = SqliteVecAdapter(adapter.directory, None)
                owns_view = True
            gate.wait(timeout=30)
            for row in rows[index::workers]:
                result["attempted"] += 1
                with mutex:
                    activity["inflight"] += 1
                    activity["peak"] = max(activity["peak"], activity["inflight"])
                try:
                    view.write(row)
                    result["successful"] += 1
                except Exception as exc:
                    logger.exception("Contention write failed for worker %s", index)
                    result["errors"].append(
                        {
                            "worker": index,
                            "id": row["id"],
                            "type": type(exc).__name__,
                            "message": str(exc),
                            "lock": is_lock_error(exc),
                        }
                    )
                finally:
                    with mutex:
                        activity["inflight"] -= 1
        except Exception as exc:
            logger.exception("Contention worker setup failed")
            result["lifecycle_errors"].append(f"{type(exc).__name__}: {exc}")
            gate.abort()
        finally:
            if owns_view:
                try:
                    view.close()
                except Exception as exc:
                    logger.exception("Contention connection close failed")
                    result["lifecycle_errors"].append(f"{type(exc).__name__}: {exc}")
        return result

    with ThreadPoolExecutor(max_workers=workers) as executor:
        outcomes = list(executor.map(worker, range(workers)))
    attempted = sum(outcome["attempted"] for outcome in outcomes)
    successful = sum(outcome["successful"] for outcome in outcomes)
    errors = [error for outcome in outcomes for error in outcome["errors"]]
    lifecycle_errors = [
        error for outcome in outcomes for error in outcome["lifecycle_errors"]
    ]
    lock_errors = sum(error["lock"] for error in errors)
    return {
        "status": "completed"
        if attempted == len(rows) and not lifecycle_errors
        else "incomplete",
        "profile": "separate write-only contention probe, excluded from serial percentiles",
        "workers": workers,
        "client_topology": "one SQLite connection per thread"
        if isinstance(adapter, SqliteVecAdapter)
        else "one shared engine client/table",
        "expected_writes": len(rows),
        "attempted_writes": attempted,
        "successful_writes": successful,
        "lock_errors": lock_errors,
        "observed_lock_error_rate": lock_errors / attempted if attempted else None,
        "other_write_errors": len(errors) - lock_errors,
        "max_inflight_calls": activity["peak"],
        "overlap_observed": activity["peak"] > 1,
        "errors": errors,
        "lifecycle_errors": lifecycle_errors,
    }


def turns_argument(value: str) -> int:
    turns = int(value)
    if not 1000 <= turns <= 10000:
        raise argparse.ArgumentTypeError("--turns must be between 1000 and 10000")
    return turns


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--turns", type=turns_argument, default=2000)
    parser.add_argument(
        "--output", type=Path, default=Path("benchmarks/agent_memory_results.json")
    )
    parser.add_argument("--targets", nargs="+", choices=TARGETS, default=list(TARGETS))
    parser.add_argument("--seed", type=int, default=23)
    parser.add_argument("--_worker", choices=TARGETS, help=argparse.SUPPRESS)
    parser.add_argument("--_workload", type=Path, help=argparse.SUPPRESS)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args._worker:
        report = run_worker(args._worker, args._workload)
        args.output.write_text(
            json.dumps(report, indent=2, allow_nan=False) + "\n", "utf-8"
        )
        return 0
    workload = build_workload(args.turns, args.seed)
    raw = encode_workload(workload)
    writes = sum(op["kind"] == "write" for op in workload["operations"])
    psutil = require_module("psutil", "psutil")
    report = {
        "schema_version": 1,
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "environment": {
            "platform": platform.platform(),
            "cpu": platform.processor(),
            "logical_cpus": os.cpu_count(),
            "physical_cpus": psutil.cpu_count(logical=False),
            "ram_bytes": psutil.virtual_memory().total,
            "python": platform.python_version(),
        },
        "methodology": {
            "workload": "synthetic agent-shaped operations, precomputed topic-clustered fixture vectors",
            "turns": args.turns,
            "seed": args.seed,
            "initial_records": INITIAL_RECORDS,
            "writes": writes,
            "reads": args.turns - writes,
            "write_fraction": writes / args.turns,
            "dimensions": DIMENSIONS,
            "sessions": SESSIONS,
            "top_k": TOP_K,
            "percentile_method": "nearest-rank, successful API calls only",
            "workload_sha256": hashlib.sha256(raw).hexdigest(),
            "process_isolation": "fresh subprocess and temporary on-disk database per target",
            "rss_scope": "whole worker: Python, common preloaded trace, native modules, and database",
            "timed_scope": "adapter write/search including binding conversion and result materialization",
            "not_timed": "generation, initial seeding/index build, correctness checks, final close",
            "contention_probe": {
                "profile": "separate write-only phase, excluded from serial latency samples",
                "workers": CONTENTION_WORKERS,
                "writes_per_worker": CONTENTION_WRITES_PER_WORKER,
            },
            "limitations": [
                "One serial latency worker; separate four-thread write-only contention probe",
                "Not a semantic retrieval-quality or real-agent evaluation",
                "Native ANN/flat retrieval plans, candidate budgets and tie-breaking differ",
                "Native result shapes differ; LanceDB materializes full rows including vectors",
                "SQLite uses WAL/NORMAL; engine durability acknowledgement boundaries differ",
                "No cold-cache, crash recovery, multi-process writers or statistical repeat trials",
                "RSS delta is not a standalone Rust-engine memory measurement",
            ],
        },
        "results": [],
    }
    with tempfile.TemporaryDirectory(prefix="vanta-agent-trace-") as directory:
        trace_path = Path(directory) / "workload.json"
        trace_path.write_bytes(raw)
        for target in dict.fromkeys(args.targets):
            output = Path(directory) / f"{target}.json"
            command = [
                sys.executable,
                str(Path(__file__).resolve()),
                "--_worker",
                target,
                "--_workload",
                str(trace_path),
                "--output",
                str(output),
            ]
            try:
                environment = os.environ.copy()
                environment.pop("VANTADB_BACKEND", None)
                completed = subprocess.run(
                    command,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=600,
                    env=environment,
                )
                if completed.returncode != 0 or not output.is_file():
                    item = {
                        "target": target,
                        "status": "error",
                        "reason": f"Worker exit {completed.returncode}",
                        "stderr": completed.stderr[-4000:],
                    }
                else:
                    item = json.loads(output.read_text("utf-8"))
                if completed.stderr:
                    item["worker_stderr_tail"] = completed.stderr[-8000:]
            except subprocess.TimeoutExpired:
                item = {
                    "target": target,
                    "status": "error",
                    "reason": "Worker exceeded 600 seconds",
                }
            report["results"].append(item)
            print(f"{target}: {item['status'].upper()}")
            if item["status"] == "ok":
                print(
                    f"  write p50/p95/p99 ms: {item['write']['p50_ms']:.3f} / "
                    f"{item['write']['p95_ms']:.3f} / {item['write']['p99_ms']:.3f}"
                )
                print(
                    f"  hybrid read p50/p95 ms: {item['hybrid_read']['p50_ms']:.3f} / "
                    f"{item['hybrid_read']['p95_ms']:.3f}"
                )
                print(
                    f"  RSS initial/final MiB: {item['rss_initial_bytes'] / 2**20:.2f} / "
                    f"{item['rss_final_bytes'] / 2**20:.2f}; observed lock errors: "
                    f"{item['lock_errors']}/{item['attempted_operations']} (serial, no contention)"
                )
                probe = item["contention_probe"]
                print(
                    f"  contention ({probe['workers']} writer threads): "
                    f"lock errors {probe['lock_errors']}/{probe['attempted_writes']}; "
                    f"max in-flight calls {probe['max_inflight_calls']}; "
                    f"other errors {probe['other_write_errors']}"
                )
            else:
                print(f"  {item.get('reason', item.get('errors'))}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, indent=2, allow_nan=False) + "\n", "utf-8"
    )
    print(f"Report: {args.output}")
    return int(
        any(item["status"] == "error" for item in report["results"])
        or not any(item["status"] == "ok" for item in report["results"])
    )


if __name__ == "__main__":
    raise SystemExit(main())
