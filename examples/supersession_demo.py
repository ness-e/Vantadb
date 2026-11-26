"""Supersession demo - update a user preference while keeping its history.

Uses the local E5 encoder from examples/migrate_from_mem0.py (local ONNX
assets, no downloads) for vector search. Deterministic, no LLM calls.
"""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from migrate_from_mem0 import _LocalE5Encoder

NS = "agent/prefs"


def _canon_pref(payload: str, metadata: dict) -> tuple:
    return payload, tuple(sorted(metadata.items()))


def _conflicts(old_meta: dict, new_meta: dict, fields: tuple) -> list:
    if old_meta.get("entity") != new_meta.get("entity") or old_meta.get(
        "scope"
    ) != new_meta.get("scope"):
        return []
    return [f for f in fields if old_meta.get(f) != new_meta.get(f)]


def run_demo(encoder=None) -> dict:
    """Run the supersession workflow and return a machine-readable summary."""
    enc = encoder or _LocalE5Encoder()
    tmp = tempfile.TemporaryDirectory()
    try:
        from vantadb import Client, memory_node_id

        db = Client(tmp.name)
        try:
            meta_v1 = {
                "category": "user_pref",
                "scope": "formatting",
                "entity": "demo-user",
                "indent_spaces": 4,
                "tabs_enabled": False,
            }
            print("[1/6] Put preference v1 (4 spaces, no tabs)")
            db.put(
                NS,
                "pref-indent",
                "User prefers 4-space indentation, no tabs",
                vector=enc("User prefers 4-space indentation"),
                metadata=meta_v1,
            )

            hits = db.search(
                NS,
                query_vector=enc.embed_query("indentation style preference"),
                text_query="indentation",
                top_k=5,
            )
            print("[2/6] Initial search (native score, not a probability):")
            for h in hits:
                print(f"      {h.key} score={h.score:.4f}")
            if [hit.key for hit in hits] != ["pref-indent"]:
                raise RuntimeError("Initial preference was not retrieved")

            print("[3/6] Application-level dedup check (not engine semantic dedup)")
            existing = db.memory.get(NS, "pref-indent")
            incoming = _canon_pref("User prefers 4-space indentation, no tabs", meta_v1)
            current = _canon_pref(existing.payload, existing.metadata)
            before_n = len(db.memory.list(NS).records)
            before_version = existing.version
            skipped = incoming == current
            if skipped:
                print("      identical canonical preference -> write skipped")
            after_rec = db.memory.get(NS, "pref-indent")
            after_n = len(db.memory.list(NS).records)
            dedup_ok = (
                skipped
                and before_n == after_n
                and before_version == after_rec.version
                and after_rec.metadata == existing.metadata
                and list(after_rec.vector) == list(existing.vector)
            )
            print(
                f"      records {before_n} -> {after_n}; version unchanged: {dedup_ok}"
            )

            print("[4/6] Application classifier: conflicting preference")
            new_meta = dict(meta_v1)
            new_meta.update({"indent_spaces": 2, "tabs_enabled": True})
            conflicts = _conflicts(
                existing.metadata, new_meta, ("indent_spaces", "tabs_enabled")
            )
            print(f"      conflict fields: {conflicts}")
            db.put(
                NS,
                "pref-indent-2",
                "User prefers 2-space indentation with tabs",
                vector=enc("User prefers 2-space indentation with tabs"),
                metadata=new_meta,
            )

            print("[5/6] Supersede old key with new key")
            db.memory.supersede(NS, "pref-indent", "pref-indent-2")

            print("[6/6] Search with exclude_superseded True vs False")
            cur = db.search(
                NS,
                query_vector=enc.embed_query("indentation"),
                exclude_superseded=True,
                top_k=10,
            )
            hist = db.search(
                NS,
                query_vector=enc.embed_query("indentation"),
                exclude_superseded=False,
                top_k=10,
            )
            cur_keys = [h.key for h in cur]
            hist_keys = [h.key for h in hist]
            old_rec = db.memory.get(NS, "pref-indent")
            print(f"      current: {cur_keys}")
            print(f"      history: {hist_keys}")
            print(
                f"      pref-indent superseded_by={old_rec.superseded_by} "
                f"superseded_at_ms={old_rec.superseded_at_ms}"
            )
            old_id = memory_node_id(NS, "pref-indent")
            new_id = memory_node_id(NS, "pref-indent-2")
            old_node = db.graph.get_node(old_id)
            new_node = db.graph.get_node(new_id)
            forward_labels = [
                edge[1] for edge in old_node["edges"] if edge[0] == new_id
            ]
            inverse_labels = [
                edge[1] for edge in new_node["edges"] if edge[0] == old_id
            ]
            print(f"      old -> new graph labels: {forward_labels}")
            print(f"      new -> old graph labels: {inverse_labels}")
            summary = {
                "current_keys": cur_keys,
                "history_keys": hist_keys,
                "superseded_by": old_rec.superseded_by,
                "superseded_at_ms": old_rec.superseded_at_ms,
                "dedup_unchanged": dedup_ok,
                "conflict_fields": conflicts,
                "new_is_current": cur_keys == ["pref-indent-2"],
                "history_has_both": set(hist_keys) == {"pref-indent", "pref-indent-2"},
                "graph_forward_link": "superseded_by" in forward_labels,
                "graph_inverse_link": "supersedes_edge" in inverse_labels,
                "inverse_graph_labels": inverse_labels,
            }
            if not (
                dedup_ok
                and conflicts == ["indent_spaces", "tabs_enabled"]
                and summary["new_is_current"]
                and summary["history_has_both"]
                and summary["superseded_by"] == "pref-indent-2"
                and isinstance(summary["superseded_at_ms"], int)
                and summary["graph_forward_link"]
                and summary["graph_inverse_link"]
            ):
                raise RuntimeError("Supersession demonstration invariant failed")
            print(json.dumps(summary, indent=2))
            return summary
        finally:
            db.close()
    finally:
        tmp.cleanup()


if __name__ == "__main__":
    run_demo()
