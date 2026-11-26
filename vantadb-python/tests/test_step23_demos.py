"""Focused tests for the Step 23 demo scripts (supersession + certified forget)."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

import pytest

EX = Path(__file__).resolve().parents[2] / "examples"


def _load(name: str):
    spec = importlib.util.spec_from_file_location(name, EX / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


class _FakeEncoder:
    def __call__(self, text: str) -> list[float]:
        return self._vec(text)

    def embed_query(self, text: str) -> list[float]:
        return self._vec(text)

    @staticmethod
    def _vec(text: str) -> list[float]:
        import math

        v = [0.0] * 384
        for i, ch in enumerate(text.encode("utf-8")):
            v[i % 384] += (ch % 53) / 53.0
        n = math.sqrt(sum(x * x for x in v)) or 1.0
        return [x / n for x in v]


@pytest.fixture
def sup():
    return _load("supersession_demo")


@pytest.fixture
def gdpr():
    return _load("gdpr_forget_demo")


def test_supersession_demo(sup) -> None:
    s = sup.run_demo(encoder=_FakeEncoder())
    assert s["new_is_current"] is True
    assert s["history_has_both"] is True
    assert s["superseded_by"] == "pref-indent-2"
    assert isinstance(s["superseded_at_ms"], int)
    assert s["dedup_unchanged"] is True
    assert s["conflict_fields"] == ["indent_spaces", "tabs_enabled"]
    assert s["graph_forward_link"] is True
    assert s["graph_inverse_link"] is True


def test_forget_demo_scope_and_isolation(gdpr) -> None:
    s = gdpr.run_demo(encoder=_FakeEncoder())
    assert s["forgotten"] == 3
    assert s["post_none"] is True
    assert s["post_hits"] == 0
    assert s["control_ok"] is True
    assert s["certs_ok"] is True


_SURFACES = (
    "store",
    "shred",
    "vector_index",
    "vector_store",
    "derived_index",
    "text_index",
    "sparse_index",
    "version_history",
    "wal",
)


def _rehash(c: dict) -> dict:
    canon = {
        k: c[k]
        for k in (
            "schema_version",
            "timestamp",
            "namespace",
            "key",
            "node_id",
            "reason",
            "status",
            "surfaces",
            "chain",
            "out_of_scope",
        )
    }
    canon["integrity"] = {"algorithm": c["integrity"]["algorithm"], "sha256": ""}
    raw = json.dumps(canon, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
    c["integrity"]["sha256"] = hashlib.sha256(raw.encode("utf-8")).hexdigest()
    return c


def _cert(ns: str = "agent/users", key: str = "k1", **kw) -> dict:
    c = {
        "schema_version": 1,
        "timestamp": "2026-11-06T00:00:00Z",
        "namespace": ns,
        "key": key,
        "node_id": "1",
        "reason": "r",
        "status": "purged",
        "surfaces": [
            {"surface": n, "action": "purged", "residues": 0, "evidence": "e"}
            for n in _SURFACES
        ],
        "chain": {
            "scheme": "s",
            "format_version": 1,
            "chained": True,
            "status": "referenced",
            "verify_command": "vanta-cli verify",
        },
        "out_of_scope": ["ssd", "backups"],
        "integrity": {"algorithm": "sha256", "sha256": ""},
    }
    c.update(kw)
    return _rehash(c)


def test_receipt_recursive_reorder_ok(gdpr) -> None:
    c = _cert()
    rev = {k: c[k] for k in reversed(list(c))}
    rev["chain"] = {k: c["chain"][k] for k in reversed(list(c["chain"]))}
    rev["integrity"] = {"sha256": c["integrity"]["sha256"], "algorithm": "sha256"}
    rev["surfaces"] = [{k: s[k] for k in reversed(list(s))} for s in c["surfaces"]]
    assert gdpr.verify_receipt(json.dumps(rev), "agent/users", "k1")["ok"] is True


def test_receipt_wrong_ns_rehash_rejected(gdpr) -> None:
    v = gdpr.verify_receipt(json.dumps(_cert(ns="other/ns")), "agent/users", "k1")
    assert v["ok"] is False and "namespace" in v["error"]


def test_receipt_wrong_key_rehash_rejected(gdpr) -> None:
    v = gdpr.verify_receipt(json.dumps(_cert(key="k2")), "agent/users", "k1")
    assert v["ok"] is False and "key" in v["error"]


def test_receipt_unknown_algo_rehash_rejected(gdpr) -> None:
    c = _cert(integrity={"algorithm": "md5", "sha256": ""})
    v = gdpr.verify_receipt(json.dumps(c), "agent/users", "k1")
    assert v["ok"] is False


def test_receipt_tampered_digest_rejected(gdpr) -> None:
    c = _cert()
    c["integrity"]["sha256"] = "0" * 64
    v = gdpr.verify_receipt(json.dumps(c), "agent/users", "k1")
    assert v["ok"] is False


def test_receipt_bad_schema_and_status(gdpr) -> None:
    for sv in (True, "1", 2):
        v = gdpr.verify_receipt(
            json.dumps(_cert(schema_version=sv)), "agent/users", "k1"
        )
        assert v["ok"] is False
    v = gdpr.verify_receipt(json.dumps(_cert(status="pending")), "agent/users", "k1")
    assert v["ok"] is False


def test_receipt_surface_anomalies_rehash_rejected(gdpr) -> None:
    nine = _cert()["surfaces"]
    for bad in (
        nine[:-1],
        nine + [nine[0]],
        nine[:-1]
        + [{"surface": "mystery", "action": "purged", "residues": 0, "evidence": "e"}],
    ):
        v = gdpr.verify_receipt(json.dumps(_cert(surfaces=bad)), "agent/users", "k1")
        assert v["ok"] is False


@pytest.mark.parametrize("residues", [1, -1, True])
def test_receipt_bad_residues_rejected(gdpr, residues) -> None:
    surf = _cert()["surfaces"]
    surf[0]["residues"] = residues
    v = gdpr.verify_receipt(json.dumps(_cert(surfaces=surf)), "agent/users", "k1")
    assert v["ok"] is False


def test_receipt_empty_out_of_scope_rejected(gdpr) -> None:
    v = gdpr.verify_receipt(json.dumps(_cert(out_of_scope=[])), "agent/users", "k1")
    assert v["ok"] is False


def test_receipt_real_surface_action_does_not_override_purged_status(gdpr) -> None:
    surfaces = _cert()["surfaces"]
    surfaces[0].update(action="deleted", residues=1)
    receipt = _cert(surfaces=surfaces)
    assert gdpr.verify_receipt(json.dumps(receipt), "agent/users", "k1")["ok"] is False


@pytest.mark.parametrize(
    "changes",
    [
        {"timestamp": "not-a-dateZ"},
        {"node_id": str(1 << 128)},
        {"node_id": "١"},
        {
            "chain": {
                "scheme": "s",
                "format_version": True,
                "chained": True,
                "status": "referenced",
                "verify_command": "vanta-cli verify",
            }
        },
        {
            "chain": {
                "scheme": "s",
                "format_version": 3,
                "chained": "true",
                "status": "referenced",
                "verify_command": "vanta-cli verify",
            }
        },
    ],
)
def test_receipt_malformed_typed_fields_rejected(gdpr, changes) -> None:
    assert (
        gdpr.verify_receipt(json.dumps(_cert(**changes)), "agent/users", "k1")["ok"]
        is False
    )


def test_receipt_malformed_json_rejected(gdpr) -> None:
    assert gdpr.verify_receipt("not json", "agent/users", "k1")["ok"] is False
    assert gdpr.verify_receipt("[1,2]", "agent/users", "k1")["ok"] is False
