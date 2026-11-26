"""Certified forget demo - erase one user's records with integrity receipts.

Uses the local E5 encoder from examples/migrate_from_mem0.py (local ONNX
assets, no downloads). Fictitious PII only. Scores are native retrieval
scores, not probabilities.
"""

from __future__ import annotations

import json
import sys
import tempfile
from datetime import datetime, timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from migrate_from_mem0 import _LocalE5Encoder

NS = "agent/users"
_SURFACES = {
    "store",
    "shred",
    "vector_index",
    "vector_store",
    "derived_index",
    "text_index",
    "sparse_index",
    "version_history",
    "wal",
}


def verify_receipt(certificate: str, namespace: str, key: str) -> dict:
    """Check a PurgeCertificate's structure and self-integrity hash.

    This detects modifications that were not accompanied by a recomputed
    hash. It is not a signature: it does not authenticate the issuer, does
    not prove erasure of SSD sectors or backups, and is not a legal
    certification. Returns {"ok": bool, "error": str | None, ...}.
    """
    import hashlib
    import hmac

    def fail(msg: str) -> dict:
        return {"ok": False, "error": msg}

    try:
        cert = json.loads(certificate)
    except (TypeError, ValueError):
        return fail("certificate is not valid JSON")
    if not isinstance(cert, dict):
        return fail("certificate is not a JSON object")

    sv = cert.get("schema_version")
    if not isinstance(sv, int) or isinstance(sv, bool) or sv != 1:
        return fail("schema_version must be integer 1")
    if cert.get("namespace") != namespace:
        return fail(f"namespace mismatch: {cert.get('namespace')!r}")
    if cert.get("key") != key:
        return fail(f"key mismatch: {cert.get('key')!r}")
    if cert.get("status") != "purged":
        return fail(f"status must be 'purged', got {cert.get('status')!r}")

    ts = cert.get("timestamp")
    if not isinstance(ts, str) or not ts.endswith(("Z", "+00:00")):
        return fail("timestamp must be a UTC string")
    try:
        if datetime.fromisoformat(ts).utcoffset() != timedelta(0):
            return fail("timestamp must represent UTC")
    except ValueError:
        return fail("timestamp must be ISO 8601")
    nid = cert.get("node_id")
    if not isinstance(nid, str) or not nid.isascii() or not nid.isdigit():
        return fail("node_id must be an ASCII decimal string")
    if int(nid) >= 1 << 128:
        return fail("node_id exceeds u128")
    if not isinstance(cert.get("reason"), str):
        return fail("reason must be a string")

    integrity = cert.get("integrity")
    if not isinstance(integrity, dict):
        return fail("integrity must be an object")
    if integrity.get("algorithm") != "sha256":
        return fail("integrity.algorithm must be 'sha256'")
    declared = integrity.get("sha256")
    if (
        not isinstance(declared, str)
        or len(declared) != 64
        or declared != declared.lower()
        or any(c not in "0123456789abcdef" for c in declared)
    ):
        return fail("integrity.sha256 must be 64 lowercase hex chars")

    surfaces = cert.get("surfaces")
    if not isinstance(surfaces, list):
        return fail("surfaces must be a list")
    names = []
    for s in surfaces:
        if not isinstance(s, dict):
            return fail("surface entries must be objects")
        name = s.get("surface")
        if not isinstance(name, str) or name not in _SURFACES:
            return fail(f"unknown surface {name!r}")
        residues = s.get("residues")
        if not isinstance(residues, int) or isinstance(residues, bool):
            return fail(f"surface {name}: residues must be integer")
        if residues < 0:
            return fail(f"surface {name}: residues must be >= 0")
        if residues != 0:
            return fail(f"surface {name}: purged certificate requires residues=0")
        if not all(isinstance(s.get(k), str) and s[k] for k in ("action", "evidence")):
            return fail(
                f"surface {name}: action and evidence must be non-empty strings"
            )
        names.append(name)
    if len(names) != 9 or set(names) != _SURFACES or len(set(names)) != len(names):
        return fail("surfaces must contain each of the 9 surfaces exactly once")

    oos = cert.get("out_of_scope")
    if (
        not isinstance(oos, list)
        or not oos
        or not all(isinstance(x, str) and x for x in oos)
    ):
        return fail("out_of_scope must be a non-empty list of strings")

    chain = cert.get("chain")
    if not isinstance(chain, dict):
        return fail("chain must be an object")
    if (
        type(chain.get("format_version")) is not int
        or not 0 <= chain["format_version"] < 1 << 32
    ):
        return fail("chain.format_version must be a u32")
    if type(chain.get("chained")) is not bool:
        return fail("chain.chained must be a boolean")
    if not all(
        isinstance(chain.get(k), str) and chain[k]
        for k in ("scheme", "status", "verify_command")
    ):
        return fail("chain string fields must be non-empty")

    try:
        canon = {
            "schema_version": cert["schema_version"],
            "timestamp": cert["timestamp"],
            "namespace": cert["namespace"],
            "key": cert["key"],
            "node_id": cert["node_id"],
            "reason": cert["reason"],
            "status": cert["status"],
            "surfaces": [
                {
                    "surface": s["surface"],
                    "action": s["action"],
                    "residues": s["residues"],
                    "evidence": s["evidence"],
                }
                for s in cert["surfaces"]
            ],
            "chain": {
                "scheme": chain["scheme"],
                "format_version": chain["format_version"],
                "chained": chain["chained"],
                "status": chain["status"],
                "verify_command": chain["verify_command"],
            },
            "out_of_scope": cert["out_of_scope"],
            "integrity": {
                "algorithm": integrity["algorithm"],
                "sha256": "",
            },
        }
    except KeyError as exc:
        return fail(f"missing canonical field: {exc}")

    raw = json.dumps(
        canon, ensure_ascii=False, separators=(",", ":"), allow_nan=False
    ).encode("utf-8")
    digest = hashlib.sha256(raw).hexdigest()
    if not hmac.compare_digest(digest, declared):
        return fail("integrity digest mismatch")
    return {"ok": True, "error": None, "digest": digest}


def run_demo(encoder=None) -> dict:
    """Forget one user's records; verify receipts; prove scope isolation."""
    enc = encoder or _LocalE5Encoder()
    user = "user_gdpr_42"
    keys = ["pii-name", "pii-email", "pii-phone"]
    tmp = tempfile.TemporaryDirectory()
    try:
        from vantadb import Client

        db = Client(tmp.name)
        try:
            facts = {
                "pii-name": "User name is Alice Doe",
                "pii-email": "User email is alice@example.test",
                "pii-phone": "User phone is +00-555-0100",
            }
            for k in keys:
                db.put(
                    NS,
                    k,
                    facts[k],
                    vector=enc(facts[k]),
                    metadata={"user_id": user},
                )
            db.put(
                NS,
                "ctrl",
                "User likes cats",
                vector=enc("User likes cats"),
                metadata={"user_id": "other_user"},
            )
            assert all(db.memory.get(NS, k) is not None for k in keys), (
                "pre-forget records missing"
            )
            pre = db.search(
                NS,
                query_vector=enc.embed_query("user contact details"),
                text_query="user",
                filters={"user_id": user},
                top_k=10,
            )
            print(f"Pre-forget scoped hits: {len(pre)}")
            assert len(pre) == 3

            result = db.memory.forget(NS, {"user_id": user}, mode="certified")
            certs = result["certificates"]
            parsed = {}
            for c in certs:
                cert = json.loads(c)
                parsed[cert["key"]] = c
            if (
                len(certs) != len(keys)
                or len(parsed) != len(certs)
                or set(parsed) != set(keys)
            ):
                raise RuntimeError(
                    f"certificate keys {sorted(parsed)} != {sorted(keys)}"
                )
            if result["forgotten_count"] != 3:
                raise RuntimeError(f"forgotten_count={result['forgotten_count']} != 3")

            print("First receipt (pretty JSON):")
            print(json.dumps(json.loads(parsed[keys[0]]), indent=2))
            for k in keys:
                v = verify_receipt(parsed[k], NS, k)
                if not v["ok"]:
                    raise RuntimeError(
                        f"receipt verification failed for {k}: {v['error']}"
                    )
                print(f"{k}: integrity=True sha256={v['digest']}")
            chain = json.loads(parsed[keys[0]])["chain"]
            print(
                "Limits: hash detects unpaired modification only; not a "
                "signature, not SSD/backup erasure, not legal GDPR proof; "
                "chain.status = referenced (verify_command is an "
                "instruction, not proof of a verified chain)."
            )
            print(f"chain.verify_command: {chain['verify_command']}")

            post = [db.memory.get(NS, k) for k in keys]
            vec_only = db.search(
                NS,
                query_vector=enc.embed_query("user contact details"),
                filters={"user_id": user},
                top_k=10,
            )
            text_only = db.search(
                NS, text_query="Alice", filters={"user_id": user}, top_k=10
            )
            mixed = db.search(
                NS,
                query_vector=enc.embed_query("user contact details"),
                text_query="Alice",
                filters={"user_id": user},
                top_k=10,
            )
            ctrl = db.memory.get(NS, "ctrl")
            assert all(r is None for r in post), "records still readable"
            assert len(vec_only) == 0 and len(text_only) == 0 and len(mixed) == 0
            assert ctrl is not None and ctrl.key == "ctrl"
            summary = {
                "forgotten": result["forgotten_count"],
                "post_none": all(r is None for r in post),
                "post_hits": len(vec_only) + len(text_only) + len(mixed),
                "control_ok": ctrl is not None,
                "certs_ok": True,
            }
            print(json.dumps(summary, indent=2))
            return summary
        finally:
            db.close()
    finally:
        tmp.cleanup()


if __name__ == "__main__":
    run_demo()
