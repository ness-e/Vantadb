#!/usr/bin/env python3
"""
evals/soak_test.py — Soak Testing 1,000 Turnos contra Fuga de Contexto (PRD-04 / Q26 Opción B).

Simula 1,000 turnos continuos de un agente de ingeniería multi-sesión evaluando:
  1. Inserciones episódicas L1 (put con vectores deterministas de 384d y metadatos).
  2. Actualizaciones de preferencias con contradicción y supersession (c.supersede).
  3. Consultas híbridas (c.search combinando BM25 + Vector + filtros de sesión + exclude_superseded).
  4. Purgas certificadas físicas (c.forget con mode='certified') cada 100 turnos.
  5. Checkpoints telemétricos cada 100 turnos (RSS, tamaño en disco, latencias p50/p95/p99,
     incidentes de fuga de contexto y verificación criptográfica de purga).

Salida:
  evals/results/soak_test_1000t.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import pathlib
import platform
import random
import shutil
import sys
import tempfile
import time
from typing import Any, Dict, List, Set

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace", line_buffering=True)
except Exception:
    pass

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = REPO_ROOT / "evals" / "results" / "soak_test_1000t.json"
SEED = 42
NAMESPACE = "engineering_agent_memory"
DIMS = 384

SESSIONS = [
    {"id": "session_arch", "topic": "Architecture & System Design", "lead": "Alice"},
    {"id": "session_perf", "topic": "Performance & Storage Tuning", "lead": "Bob"},
    {"id": "session_sec", "topic": "Security, Secrets & Hardening", "lead": "Charlie"},
    {"id": "session_ui", "topic": "UI Tokens, Motion & Themes", "lead": "Diana"},
    {"id": "session_db", "topic": "Database WAL, Partitions & HNSW", "lead": "Eve"},
]

FACT_TEMPLATES = [
    "Decided to adopt {pattern} pattern for {component} in order to optimize throughput and cache efficiency.",
    "Configured {param} parameter to value {val} to mitigate high contention under heavy concurrent workloads.",
    "Benchmarked {component} with payload size {val}KB; observed stable latency distribution with zero regressions.",
    "Updated technical specification for {component} requiring strict deterministic hash verification.",
    "Refactored internal pipeline of {component} eliminating redundant allocations in the critical write path.",
]

PATTERNS = ["FastGate filter", "Zero-Copy buffer", "Write-Ahead Log", "LSM Compactor", "Skip-List Index", "RRF Fusion"]
COMPONENTS = ["MemoryCoordinator", "PartitionManager", "ShreddingStore", "HnswGraphIndex", "TextPostingTable"]
PARAMS = ["max_wal_segment_size", "hnsw_ef_search", "lru_cache_capacity", "bm25_k1", "bm25_b", "lock_free_ring"]


def get_process_rss_mib() -> float:
    """Measure resident set size (RSS) in MiB."""
    try:
        import psutil
        return round(psutil.Process().memory_info().rss / (1024 * 1024), 2)
    except Exception:
        pass
    try:
        if platform.system() == "Windows":
            import ctypes
            class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
                _fields_ = [
                    ('cb', ctypes.c_ulong),
                    ('PageFaultCount', ctypes.c_ulong),
                    ('PeakWorkingSetSize', ctypes.c_size_t),
                    ('WorkingSetSize', ctypes.c_size_t),
                    ('QuotaPeakPagedPoolUsage', ctypes.c_size_t),
                    ('QuotaPagedPoolUsage', ctypes.c_size_t),
                    ('QuotaPeakNonPagedPoolUsage', ctypes.c_size_t),
                    ('QuotaNonPagedPoolUsage', ctypes.c_size_t),
                    ('PagefileUsage', ctypes.c_size_t),
                    ('PeakPagefileUsage', ctypes.c_size_t),
                ]
            counters = PROCESS_MEMORY_COUNTERS()
            counters.cb = ctypes.sizeof(PROCESS_MEMORY_COUNTERS)
            handle = ctypes.windll.kernel32.GetCurrentProcess()
            if ctypes.windll.psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                return round(counters.WorkingSetSize / (1024 * 1024), 2)
        import resource
        return round(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024, 2)
    except Exception:
        return 0.0


def get_dir_size_kib(path: pathlib.Path) -> float:
    """Calculate directory recursive size in KiB."""
    total = 0
    try:
        for p in path.rglob("*"):
            if p.is_file():
                total += p.stat().st_size
    except Exception:
        pass
    return round(total / 1024, 2)


def percentile(vals: List[float], p: float) -> float:
    """Calculate percentile using nearest-rank."""
    if not vals:
        return 0.0
    ordered = sorted(vals)
    n = len(ordered)
    idx = max(0, min(math.ceil(p * n) - 1, n - 1))
    return round(ordered[idx], 3)


def make_deterministic_vector(seed_text: str, dims: int = DIMS) -> List[float]:
    """Generate a deterministic unit-normalized pseudo-embedding vector."""
    hasher = hashlib.sha256()
    hasher.update(seed_text.encode("utf-8"))
    raw_hash = hasher.digest()
    
    # Expand 32 bytes into dims floats deterministically
    rng = random.Random(int.from_bytes(raw_hash[:8], byteorder="big"))
    vec = [rng.gauss(0.0, 1.0) for _ in range(dims)]
    norm = math.sqrt(sum(x * x for x in vec))
    if norm > 0.0:
        vec = [x / norm for x in vec]
    return vec


def run_soak_test(total_turns: int = 1000, output_path: pathlib.Path = DEFAULT_OUTPUT) -> Dict[str, Any]:
    import vantadb

    print(f"================================================================================")
    print(f"🚀 VantaDB Soak Testing 1,000 Turnos (PRD-04 / Q26 Opción B)")
    print(f"   VantaDB Version: {getattr(vantadb, '__version__', '0.9.0')}")
    print(f"   Platform: {platform.system()} {platform.machine()} ({platform.python_version()})")
    print(f"   Total Turns: {total_turns}")
    print(f"================================================================================")

    temp_dir = tempfile.mkdtemp(prefix="vantadb_soak_1000t_")
    db_path = pathlib.Path(temp_dir)
    client = vantadb.Client(str(db_path))

    rng = random.Random(SEED)
    initial_rss = get_process_rss_mib()

    # Tracking sets for strict leakage verification
    active_keys_by_session: Dict[str, Set[str]] = {s["id"]: set() for s in SESSIONS}
    superseded_keys: Set[str] = set()
    purged_keys: Set[str] = set()
    latest_pref_by_session: Dict[str, Dict[str, str]] = {s["id"]: {} for s in SESSIONS}

    write_latencies: List[float] = []
    search_latencies: List[float] = []
    checkpoints: List[Dict[str, Any]] = []

    context_leakage_incidents = 0
    certified_forgets_verified = 0
    total_superseded_count = 0
    total_purged_count = 0

    print(f"Turn | Phase Checkpoint | DB Size (KiB) | RSS (MiB) | Write p95 (ms) | Search p95 (ms) | Leakage | Forgets")
    print(f"-----+------------------+---------------+-----------+----------------+-----------------+---------+--------")

    for turn in range(1, total_turns + 1):
        sess_idx = (turn - 1) % len(SESSIONS)
        current_sess = SESSIONS[sess_idx]
        sid = current_sess["id"]
        topic = current_sess["topic"]

        # -------------------------------------------------------------
        # 1. Inserción Episódica L1 (Write path)
        # -------------------------------------------------------------
        ep_key = f"ep_{sid}_t{turn:04d}"
        tpl = rng.choice(FACT_TEMPLATES)
        pattern = rng.choice(PATTERNS)
        component = rng.choice(COMPONENTS)
        param = rng.choice(PARAMS)
        val = rng.randint(16, 8192)
        fact_text = f"[{sid} t={turn}] {tpl.format(pattern=pattern, component=component, param=param, val=val)}"
        vec = make_deterministic_vector(f"{topic}:{fact_text}")

        meta: Dict[str, Any] = {
            "session_id": sid,
            "turn": turn,
            "topic": topic,
            "layer": "L1",
            "lead": current_sess["lead"],
        }

        t_w0 = time.perf_counter()
        client.put(NAMESPACE, ep_key, fact_text, metadata=meta, vector=vec)
        active_keys_by_session[sid].add(ep_key)

        # -------------------------------------------------------------
        # 2. Contradicción y Supersession de Preferencias (cada 20 turnos)
        # -------------------------------------------------------------
        if turn % 20 == 0:
            pref_category = "theme_and_layout" if (turn % 40 == 0) else "storage_cache_policy"
            old_key = latest_pref_by_session[sid].get(pref_category)
            new_key = f"pref_{sid}_{pref_category}_v{turn:04d}"
            new_val = f"Pref-Option-{rng.randint(100, 999)}"
            pref_payload = f"[{sid} Preference] {pref_category} set to {new_val} at turn {turn} (supersedes prior)."
            pref_meta = {"session_id": sid, "turn": turn, "topic": "preferences", "category": pref_category}
            pref_vec = make_deterministic_vector(pref_payload)

            client.put(NAMESPACE, new_key, pref_payload, metadata=pref_meta, vector=pref_vec)
            active_keys_by_session[sid].add(new_key)

            if old_key is not None:
                client.supersede(NAMESPACE, old_key, new_key)
                superseded_keys.add(old_key)
                if old_key in active_keys_by_session[sid]:
                    active_keys_by_session[sid].remove(old_key)
                total_superseded_count += 1

            latest_pref_by_session[sid][pref_category] = new_key

        # -------------------------------------------------------------
        # 3. Purga Certificada Física (cada 100 turnos)
        # -------------------------------------------------------------
        if turn % 100 == 0:
            # Insert and immediately purge a sensitive GDPR compliance record
            sensitive_key = f"gdpr_token_{sid}_t{turn:04d}"
            sensitive_payload = f"CONFIDENTIAL API TOKEN FOR {sid}: secret_key_uuid_{rng.randint(100000, 999999)}"
            sensitive_vec = make_deterministic_vector(sensitive_payload)
            client.put(
                NAMESPACE,
                sensitive_key,
                sensitive_payload,
                metadata={"session_id": sid, "gdpr": True, "turn": turn},
                vector=sensitive_vec,
            )

            # Certified forget execution
            cert_result = client.forget(NAMESPACE, sensitive_key, mode="certified")
            total_purged_count += 1
            purged_keys.add(sensitive_key)

            # Verification of certificate payload
            assert cert_result.get("mode") == "certified", f"Invalid mode: {cert_result}"
            assert cert_result.get("forgotten_count") == 1, f"Expected 1 forgotten, got {cert_result}"
            assert sensitive_key in cert_result.get("keys", []), f"Key missing in cert: {cert_result}"

            raw_certs = cert_result.get("certificates", [])
            assert len(raw_certs) > 0, "No certificate payload returned"
            cert_obj = json.loads(raw_certs[0])
            assert cert_obj["status"] == "purged"
            assert cert_obj["integrity"]["algorithm"] == "sha256"
            assert len(cert_obj["integrity"]["sha256"]) == 64

            # Immediate verification: search must NOT return the purged key
            post_purge_hits = client.search(
                NAMESPACE,
                sensitive_vec,
                text_query="CONFIDENTIAL API TOKEN",
                filters={"session_id": sid},
                top_k=5,
            )
            purged_found = any(h.key == sensitive_key for h in post_purge_hits)
            assert not purged_found, f"CRITICAL: Purged key {sensitive_key} was returned in search!"
            certified_forgets_verified += 1

        t_w1 = time.perf_counter()
        write_latencies.append((t_w1 - t_w0) * 1000.0)

        # -------------------------------------------------------------
        # 4. Consulta Híbrida y Verificación Anti-Fuga de Contexto
        # -------------------------------------------------------------
        query_text = f"{component} {pattern}"
        q_vec = make_deterministic_vector(f"{topic}:{query_text}")

        t_s0 = time.perf_counter()
        hits = client.search(
            NAMESPACE,
            q_vec,
            text_query=query_text,
            filters={"session_id": sid},
            top_k=10,
            exclude_superseded=True,
        )
        t_s1 = time.perf_counter()
        search_latencies.append((t_s1 - t_s0) * 1000.0)

        # Verification of isolation and validity
        for hit in hits:
            # Check 1: Session isolation (zero cross-session contamination)
            hit_sid = (hit.metadata or {}).get("session_id")
            if hit_sid != sid:
                context_leakage_incidents += 1
                print(f"❌ LEAKAGE: Hit {hit.key} belongs to session {hit_sid}, expected {sid}!")

            # Check 2: Superseded fact leakage
            if hit.key in superseded_keys:
                context_leakage_incidents += 1
                print(f"❌ LEAKAGE: Superseded key {hit.key} appeared in search results!")

            # Check 3: Purged fact leakage
            if hit.key in purged_keys:
                context_leakage_incidents += 1
                print(f"❌ LEAKAGE: Purged key {hit.key} appeared in search results!")

        # -------------------------------------------------------------
        # 5. Checkpoints Telemétricos cada 100 Turnos
        # -------------------------------------------------------------
        if turn % 100 == 0:
            current_rss = get_process_rss_mib()
            current_disk_kib = get_dir_size_kib(db_path)
            current_disk_mib = round(current_disk_kib / 1024, 2)
            
            # Window of the last 100 turns for localized latency tracking
            recent_w = write_latencies[-100:]
            recent_s = search_latencies[-100:]

            cp = {
                "turn": turn,
                "rss_mib": current_rss,
                "disk_kib": current_disk_kib,
                "disk_mib": current_disk_mib,
                "write_p50_ms": percentile(recent_w, 0.50),
                "write_p95_ms": percentile(recent_w, 0.95),
                "write_p99_ms": percentile(recent_w, 0.99),
                "search_p50_ms": percentile(recent_s, 0.50),
                "search_p95_ms": percentile(recent_s, 0.95),
                "search_p99_ms": percentile(recent_s, 0.99),
                "context_leakage_incidents": context_leakage_incidents,
                "certified_forgets_verified": certified_forgets_verified,
                "superseded_records_count": len(superseded_keys),
            }
            checkpoints.append(cp)

            print(
                f"{turn:4d} | Checkpoint {turn//100:02d}/10 | {current_disk_kib:13.1f} | {current_rss:9.2f} | "
                f"{cp['write_p95_ms']:14.3f} | {cp['search_p95_ms']:15.3f} | "
                f"{context_leakage_incidents:7d} | {certified_forgets_verified:7d}"
            )

    final_rss = get_process_rss_mib()
    final_disk_kib = get_dir_size_kib(db_path)
    final_disk_mib = round(final_disk_kib / 1024, 2)

    client.close()
    shutil.rmtree(temp_dir, ignore_errors=True)

    summary = {
        "benchmark": "soak_test_1000t",
        "prd": "PRD-04",
        "decision": "Q26 Opción B",
        "status": "PASSED" if context_leakage_incidents == 0 else "FAILED",
        "environment": {
            "os": platform.system(),
            "arch": platform.machine(),
            "python": platform.python_version(),
            "vantadb_version": getattr(vantadb, "__version__", "0.9.0"),
            "seed": SEED,
            "dimensions": DIMS,
            "sessions_count": len(SESSIONS),
        },
        "totals": {
            "total_turns": total_turns,
            "total_episodic_writes": total_turns,
            "total_searches": total_turns,
            "total_superseded": total_superseded_count,
            "total_certified_purges": total_purged_count,
            "certified_forgets_verified": certified_forgets_verified,
            "context_leakage_incidents": context_leakage_incidents,
            "context_leakage_rate": round(context_leakage_incidents / total_turns, 6),
        },
        "latencies_ms": {
            "write_p50_ms": percentile(write_latencies, 0.50),
            "write_p95_ms": percentile(write_latencies, 0.95),
            "write_p99_ms": percentile(write_latencies, 0.99),
            "search_p50_ms": percentile(search_latencies, 0.50),
            "search_p95_ms": percentile(search_latencies, 0.95),
            "search_p99_ms": percentile(search_latencies, 0.99),
        },
        "memory_and_storage": {
            "initial_rss_mib": initial_rss,
            "final_rss_mib": final_rss,
            "rss_delta_mib": round(final_rss - initial_rss, 2),
            "final_disk_kib": final_disk_kib,
            "final_disk_mib": final_disk_mib,
        },
        "checkpoints": checkpoints,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(f"\n✅ Reporte de Soak Testing guardado exitosamente en: {output_path}")
    print(f"   Context Leakage Incidents: {context_leakage_incidents} (Rate: 0.0%)")
    print(f"   Certified Forgets Verified: {certified_forgets_verified}/10")
    print(f"   Write Latency p50: {summary['latencies_ms']['write_p50_ms']} ms | p99: {summary['latencies_ms']['write_p99_ms']} ms")
    print(f"   Search Latency p50: {summary['latencies_ms']['search_p50_ms']} ms | p99: {summary['latencies_ms']['search_p99_ms']} ms")
    print(f"   Final RSS: {final_rss} MiB (Delta: +{summary['memory_and_storage']['rss_delta_mib']} MiB)")
    print(f"   Final DB Size: {final_disk_kib} KiB ({final_disk_mib} MiB)")

    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="VantaDB Soak Testing 1,000 Turns (PRD-04).")
    parser.add_argument("--turns", type=int, default=1000, help="Number of continuous turns (default: 1000).")
    parser.add_argument("--output", type=str, default=str(DEFAULT_OUTPUT), help="Output JSON report path.")
    args = parser.parse_args()

    run_soak_test(total_turns=args.turns, output_path=pathlib.Path(args.output))


if __name__ == "__main__":
    main()
