"""Native checks for the hero demo memory stage.

The encoder fixture is deterministic and test-only — it produces
fixed-shape 384-d vectors, NOT semantic embeddings.
"""

import importlib.util
import json
import sys
from pathlib import Path

EX = Path(__file__).resolve().parents[2] / "examples"


def _load(name: str):
    if str(EX) not in sys.path:
        sys.path.insert(0, str(EX))
    spec = importlib.util.spec_from_file_location(name, EX / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


class _FakeEnc:
    def __call__(self, text: str) -> list[float]:
        return [0.5, 0.5] + [0.0] * 382

    def embed_query(self, text: str) -> list[float]:
        return self(text)


def test_memory_frames_labels_and_schema(tmp_path):
    from vantadb import Client

    hd = _load("hero_demo")
    db = Client(str(tmp_path))
    try:
        fc, fs = hd._memory_frames(db, _FakeEnc())
    finally:
        db.close()

    out_cap = fc["output"]
    assert "captured L0 messages=" in out_cap
    assert "application-curated records=" in out_cap
    assert "L2" in out_cap
    assert "vector dims: 384" in out_cap

    out_sup = fs["output"]
    assert "initial hybrid RRF score=" in out_sup
    assert "current=['pref-indent-2']" in out_sup
    assert "'pref-indent'" in out_sup and "'pref-indent-2'" in out_sup
    for f in (fc, fs):
        assert set(f) == {"title", "command", "output"}
        assert f["title"] and f["command"] and f["output"]


def test_supersession_filters_and_adjacency(tmp_path):
    from vantadb import Client, memory_node_id

    hd = _load("hero_demo")
    db = Client(str(tmp_path))
    try:
        hd._memory_frames(db, _FakeEnc())
        cur = db.search(
            hd.NS,
            query_vector=[0.5, 0.5] + [0.0] * 382,
            text_query="indentation",
            filters={"scope": "formatting"},
            top_k=5,
            exclude_superseded=True,
        )
        assert {h.key for h in cur} == {"pref-indent-2"}

        old_id = memory_node_id(hd.NS, "pref-indent")
        new_id = memory_node_id(hd.NS, "pref-indent-2")
        old_labels = [
            e[1] for e in db.graph.get_node(old_id)["edges"] if e[0] == new_id
        ]
        new_labels = [
            e[1] for e in db.graph.get_node(new_id)["edges"] if e[0] == old_id
        ]
        assert "superseded_by" in old_labels
        assert "supersedes_edge" in new_labels
    finally:
        db.close()


def test_frozen_schema_contract():
    root = Path(__file__).resolve().parents[2]
    path = root / "vhs" / "hero-run.json"
    assert path.is_file(), "hero-run.json missing — run examples/hero_demo.py first"
    doc = json.loads(path.read_text(encoding="utf-8"))
    assert doc["schema_version"] == 1
    assert doc["mcp_version"] == "0.9.0"
    assert doc["disclosure"]
    assert len(doc["frames"]) == 5
    for f in doc["frames"]:
        assert set(f) == {"title", "command", "output"}
        assert f["title"] and f["command"] and f["output"]
        if "vanta-cli" in f["command"]:
            assert "<temporary demo database>" in f["command"]
    assert doc["frames"][4]["title"] == "L2 scene & engine inspection"


def test_frozen_l2_evidence():
    path = Path(__file__).resolve().parents[2] / "vhs" / "hero-evidence.json"
    assert path.is_file(), (
        "hero-evidence.json missing — run examples/hero_demo.py first"
    )
    ev = json.loads(path.read_text(encoding="utf-8"))
    assert set(ev) == {"stats", "l2_scene"}
    scene = ev["l2_scene"]
    assert scene["scene_name"] == "formatting"
    assert scene["meta"]["summary"] == "Demo formatting preference"
    assert "2-space indentation with tabs" in scene["content"]
    assert scene["deleted"] is False
    assert "scene/hero-session" in ev["stats"]["namespaces"]
