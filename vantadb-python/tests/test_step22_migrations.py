"""Step-22 focused tests: Chroma migrator extension, CLI wrapper, Mem0 adapter.

Unit sections use strict stubs (no network, no native engine). E2E sections use
the real installed vantadb-py 0.9.0 plus ChromaDB persistent stores in fresh
temp directories only.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
CLI = REPO_ROOT / "scripts" / "migrate_from_chroma.py"
ADAPTER = REPO_ROOT / "examples" / "migrate_from_mem0.py"


def _run_cli(*argv: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(CLI), *argv],
        check=False,
        capture_output=True,
        text=True,
        timeout=120,
    )


def test_cli_help_without_sdk_or_chroma() -> None:
    """--help must work in -S mode (no site-packages at all)."""
    res = subprocess.run(
        [sys.executable, "-S", str(CLI), "--help"],
        check=False,
        capture_output=True,
        text=True,
        timeout=120,
    )
    assert res.returncode == 0
    for flag in (
        "--chroma-path",
        "--vanta-path",
        "--collection",
        "--namespace-prefix",
        "--batch-size",
    ):
        assert flag in res.stdout
    assert "chroma/" in res.stdout


def test_cli_missing_chroma_friendly_message(tmp_path: Path) -> None:
    src = tmp_path / "chroma_src"
    src.mkdir()
    try:
        (src / "chroma.sqlite3").write_bytes(b"")
        res = subprocess.run(
            [
                sys.executable,
                "-c",
                (
                    "import sys, runpy; sys.modules['chromadb']=None; "
                    "sys.argv=['migrate_from_chroma.py']+sys.argv[1:];"
                    f"runpy.run_path({str(CLI)!r}, run_name='__main__')"
                ),
                "--chroma-path",
                str(src),
                "--vanta-path",
                str(tmp_path / "vanta_dst"),
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=120,
        )
        err = res.stderr
        assert res.returncode == 1
        assert "ChromaDB is not installed. Run: pip install chromadb" in err
        assert "Traceback" not in err
    finally:
        shutil.rmtree(src, ignore_errors=True)


def test_cli_rejects_nonpositive_batch_size() -> None:
    for bad in ("0", "-5"):
        res = _run_cli("--batch-size", bad)
        assert res.returncode != 0


def test_cli_rejects_missing_source(tmp_path: Path) -> None:
    res = _run_cli("--chroma-path", str(tmp_path / "nope"))
    assert res.returncode == 1
    assert "chroma.sqlite3" in res.stderr or "does not exist" in res.stderr


def test_cli_rejects_same_and_nested_paths(tmp_path: Path) -> None:
    src = tmp_path / "chroma_data"
    src.mkdir()
    (src / "chroma.sqlite3").write_bytes(b"")
    res = _run_cli("--chroma-path", str(src), "--vanta-path", str(src))
    assert res.returncode == 1
    nested = src / "out"
    res = _run_cli("--chroma-path", str(src), "--vanta-path", str(nested))
    assert res.returncode == 1
    outer = tmp_path / "vanta_data"
    outer.mkdir()
    res = _run_cli("--chroma-path", str(outer / "sub"), "--vanta-path", str(outer))
    assert res.returncode == 1


class _StubCollection:
    def __init__(self, name, pages):
        self.name = name
        self._pages = pages

    def get(self, *, limit, offset, include):
        remaining = sum(len(p["ids"]) for p in self._pages)
        if offset >= remaining:
            return {"ids": []}
        pos = 0
        for p in self._pages:
            if offset < pos + len(p["ids"]):
                return p
            pos += len(p["ids"])
        return {"ids": []}


class _StubDB:
    def __init__(self):
        self.records = []
        self.flushed = False
        self.closed = False
        self.fail = False

    def put_batch(self, records):
        if self.fail:
            raise RuntimeError("boom")
        for r in records:
            assert isinstance(r, dict), "put_batch requires array-of-objects"
            for field in ("namespace", "key", "payload"):
                assert field in r
        self.records.extend(records)
        return []

    def flush(self):
        self.flushed = True

    def close(self):
        self.closed = True


class _ChromaStub:
    def __init__(self, collections):
        self.Settings = lambda **kw: kw
        self._collections = collections

    def PersistentClient(self, path=None, settings=None):
        cols = {c.name: c for c in self._collections}
        return type(
            "C",
            (),
            {
                "list_collections": lambda s: list(cols.values()),
                "get_collection": lambda s, n, **k: cols[n],
            },
        )()


def _patch_env(monkeypatch, collections, db=None):
    """Stub chromadb module + vantadb.Client for one migrator call."""
    monkeypatch.setitem(sys.modules, "chromadb", _ChromaStub(collections))
    import vantadb

    stub = db or _StubDB()
    monkeypatch.setattr(vantadb, "Client", lambda path: stub)
    return stub


def test_collection_names_objects_and_strings_sorted() -> None:
    from vantadb_py.migrate import chroma as mod

    class Obj:
        def __init__(self, n):
            self.name = n

    c = type("C", (), {"list_collections": lambda s: [Obj("b"), "a", Obj("c")]})()
    assert mod._collection_names(c, None) == ["a", "b", "c"]


def test_rejects_nonpositive_batch_size() -> None:
    from vantadb_py.migrate import migrate_from_chroma

    with pytest.raises(ValueError, match="batch_size"):
        migrate_from_chroma("x", "y", batch_size=0)


def test_uneven_arrays_rejected_no_repo_writes(tmp_path, monkeypatch) -> None:
    np = pytest.importorskip("numpy")
    from vantadb_py.migrate import migrate_from_chroma

    col = _StubCollection(
        "c",
        [
            {
                "ids": ["a", "b"],
                "documents": ["d1"],
                "metadatas": [{}, {}],
                "embeddings": np.zeros((2, 3)),
            }
        ],
    )
    _patch_env(monkeypatch, [col])
    with pytest.raises(ValueError, match="length"):
        migrate_from_chroma(str(tmp_path / "src"), str(tmp_path / "dst"))
    assert not (Path.cwd() / "dst").exists()


def test_prefix_empty_collection_and_callback(tmp_path, monkeypatch) -> None:
    from vantadb_py.migrate import migrate_from_chroma

    empty = _StubCollection("empty", [{"ids": []}])
    full = _StubCollection(
        "full",
        [
            {
                "ids": ["k1"],
                "documents": ["doc"],
                "metadatas": [{"a": 1}],
                "embeddings": [[0.1, 0.2]],
            }
        ],
    )
    db = _patch_env(monkeypatch, [full, empty])
    captured = []
    total = migrate_from_chroma(
        str(tmp_path / "s"),
        str(tmp_path / "d"),
        namespace_prefix="chroma/",
        on_collection=lambda n, c: captured.append((n, c)),
    )
    assert total == 1
    assert captured == [("empty", 0), ("full", 1)]
    assert db.records[0]["namespace"] == "chroma/full"
    assert db.flushed and db.closed


def test_close_on_failure(tmp_path, monkeypatch) -> None:
    from vantadb_py.migrate import migrate_from_chroma

    col = _StubCollection(
        "c",
        [{"ids": ["a"], "documents": ["d"], "metadatas": [{}], "embeddings": [[1.0]]}],
    )
    db = _StubDB()
    db.fail = True
    _patch_env(monkeypatch, [col], db=db)
    with pytest.raises(RuntimeError):
        migrate_from_chroma(str(tmp_path / "s"), str(tmp_path / "d"))
    assert db.closed is True


@pytest.fixture()
def chroma_two_collections() -> Path:
    chromadb = pytest.importorskip("chromadb")
    path = Path(tempfile.mkdtemp())
    try:
        client = chromadb.PersistentClient(
            path=str(path),
            settings=chromadb.Settings(anonymized_telemetry=False),
        )
        c1 = client.create_collection("alpha")
        c1.add(
            ids=["shared", "a1", "a2"],
            documents=["alpha one", "alpha two", "alpha three"],
            metadatas=[
                {"flag": True, "n": 1},
                {"flag": False, "n": 2},
                {"flag": True, "n": 3},
            ],
            embeddings=[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        )
        c2 = client.create_collection("beta")
        c2.add(
            ids=["shared", "b1"],
            documents=["beta one", "beta two"],
            metadatas=[{"flag": True, "n": 4}, {"flag": False, "n": 5}],
            embeddings=[[0.5, 0.5, 0.0], [0.25, 0.75, 0.0]],
        )
        yield path
    finally:
        shutil.rmtree(path, ignore_errors=True)


def test_e2e_chroma_migration(chroma_two_collections: Path, tmp_path: Path) -> None:
    from vantadb import Client
    from vantadb_py.migrate import migrate_from_chroma

    dest = tmp_path / "vanta_db"
    seen = []
    total = migrate_from_chroma(
        str(chroma_two_collections),
        str(dest),
        batch_size=2,
        namespace_prefix="chroma/",
        on_collection=lambda n, c: seen.append((n, c)),
    )
    assert total == 5
    assert sorted(seen) == [("alpha", 3), ("beta", 2)]

    db = Client(str(dest))
    try:
        assert db.memory.get("chroma/alpha", "shared").payload == "alpha one"
        assert db.memory.get("chroma/beta", "shared").payload == "beta one"
        hits = db.search("chroma/alpha", [1.0, 0.0, 0.0], top_k=2)
        assert hits and hits[0].key == "shared"
    finally:
        db.close()

    total2 = migrate_from_chroma(
        str(chroma_two_collections), str(dest), namespace_prefix="chroma/"
    )
    assert total2 == 5
    db2 = Client(str(dest))
    try:
        assert len(db2.memory.list("chroma/alpha").records) == 3
        assert len(db2.memory.list("chroma/beta").records) == 2
        r = db2.memory.get("chroma/alpha", "a1")
        assert r.metadata["flag"] is False and r.metadata["n"] == 2
        r2 = db2.memory.get("chroma/beta", "b1")
        assert r2.metadata["flag"] is False and r2.metadata["n"] == 5
        hits = db2.search("chroma/beta", [0.5, 0.5, 0.0], top_k=2)
        assert hits and hits[0].key == "shared"
    finally:
        db2.close()


def test_cli_nonexistent_collection_exit1(
    chroma_two_collections: Path, tmp_path: Path
) -> None:
    res = _run_cli(
        "--chroma-path",
        str(chroma_two_collections),
        "--vanta-path",
        str(tmp_path / "cli_bad"),
        "--collection",
        "does_not_exist",
    )
    assert res.returncode == 1
    assert "Traceback" not in res.stderr
    assert "partial" in res.stderr


def test_cli_e2e_default_prefix(chroma_two_collections: Path, tmp_path: Path) -> None:
    res = _run_cli(
        "--chroma-path",
        str(chroma_two_collections),
        "--vanta-path",
        str(tmp_path / "cli_db"),
        "--batch-size",
        "2",
    )
    assert res.returncode == 0, res.stderr
    assert "5 record(s)" in res.stdout
    from vantadb import Client

    db = Client(str(tmp_path / "cli_db"))
    try:
        assert db.memory.get("chroma/alpha", "shared") is not None
    finally:
        db.close()


def test_cli_e2e_without_vantadb_py_importable(
    chroma_two_collections: Path, tmp_path: Path
) -> None:
    """CLI must migrate via the checkout helper even when vantadb_py cannot import."""
    res = subprocess.run(
        [
            sys.executable,
            "-c",
            (
                "import sys, runpy; sys.modules['vantadb_py.migrate']=None; "
                "sys.argv=['migrate_from_chroma.py']+sys.argv[1:];"
                f"runpy.run_path({str(CLI)!r}, run_name='__main__')"
            ),
            "--chroma-path",
            str(chroma_two_collections),
            "--vanta-path",
            str(tmp_path / "cli_db2"),
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=180,
    )
    assert res.returncode == 0, res.stderr
    assert "5 record(s)" in res.stdout
    from vantadb import Client

    db = Client(str(tmp_path / "cli_db2"))
    try:
        assert db.memory.get("chroma/beta", "b1") is not None
    finally:
        db.close()


def _load_adapter():
    import importlib.util

    spec = importlib.util.spec_from_file_location("migrate_from_mem0", ADAPTER)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _fake_encoder(dim=8):
    """Deterministic test-only encoder (NOT semantic)."""

    def enc(text: str):
        vec = [0.0] * dim
        for i, ch in enumerate(text.encode()[:dim]):
            vec[i % dim] = (ch % 7) / 7.0 + 0.01
        return vec

    enc.embed_query = enc
    return enc


@pytest.fixture()
def mem(tmp_path):
    mod = _load_adapter()
    m = mod.VantaMemory(
        db_path=str(tmp_path / "db"),
        namespace="mem0/test",
        embedding_model=_fake_encoder(),
    )
    yield m
    m.close()


def test_mem0_add_get_search(mem) -> None:
    out = mem.add([{"role": "user", "content": "sky is blue"}], user_id="u1")
    assert isinstance(out, list) and out[0]["event"] == "ADD"
    rid = out[0]["id"]
    got = mem.get(rid)
    assert got is not None and got["memory"] == "sky is blue"
    assert got["role"] == "user"
    hits = mem.search("blue sky", user_id="u1", limit=3)
    assert hits and hits[0]["id"] == rid
    assert isinstance(hits[0]["score"], (int, float))


def test_mem0_string_message_and_scopes(mem) -> None:
    out = mem.add("plain fact", user_id="u2", agent_id="a9", run_id="r7")
    assert out[0]["id"]
    allu = mem.get_all(user_id="u2")
    assert len(allu) == 1 and allu[0]["agent_id"] == "a9" and allu[0]["run_id"] == "r7"
    assert mem.get_all(user_id="uX") == []


def test_mem0_scopes_isolation_and_id_stable(mem) -> None:
    a = mem.add("same", user_id="uA")[0]["id"]
    b = mem.add("same", user_id="uB")[0]["id"]
    assert a != b, "ids must be scoped to avoid cross-user collisions"
    again = mem.add("same", user_id="uA")
    assert again[0]["id"] == a and again[0]["event"] == "UPDATE"


def test_mem0_nested_metadata_readback_exact(mem) -> None:
    meta = {"prefs": {"lang": "es", "depth": {"x": [1, 2]}}, "topic": "t"}
    rid = mem.add("nested", user_id="u1", metadata=meta)[0]["id"]
    got = mem.get(rid)
    assert got["metadata"]["prefs"] == meta["prefs"]
    assert got["metadata"]["topic"] == "t"
    hits = mem.get_all(user_id="u1", filters={"topic": "t"})
    assert len(hits) == 1


def test_mem0_filter_conflict_nested_null_and_bool_vs_int(mem) -> None:
    with pytest.raises(ValueError):
        mem.add(
            "x", user_id="u1", metadata={"prefs": {"x": 1}}, filters={"prefs": "new"}
        )
    with pytest.raises(ValueError):
        mem.add("x", user_id="u1", metadata={"tag": None}, filters={"tag": "new"})
    with pytest.raises(ValueError):
        mem.add("x", user_id="u1", metadata={"flag": True}, filters={"flag": 1})
    ok = mem.add("x", user_id="u1", metadata={"flag": True}, filters={"flag": True})
    assert ok[0]["id"]
    assert (
        mem.get_all(user_id="u1", filters={"flag": True})[0]["metadata"]["flag"] is True
    )


def test_mem0_flat_rejects_nan_inf_and_non_dict(mem) -> None:
    with pytest.raises(ValueError):
        mem.search("q", user_id="u1", filters={"score": float("nan")})
    with pytest.raises(ValueError):
        mem.search("q", user_id="u1", filters={"score": float("inf")})
    with pytest.raises(TypeError):
        mem.search("q", user_id="u1", filters=["topic", "x"])
    with pytest.raises(TypeError):
        mem.add("x", user_id="u1", filters={"topic": object()})


def test_mem0_role_from_user_metadata_cannot_override(mem) -> None:
    rid = mem.add("r", user_id="u1", metadata={"role": "fake"})[0]["id"]
    assert mem.get(rid)["role"] == "user"


def test_mem0_literal_dollar_value_allowed(mem) -> None:
    mem.add("v", user_id="u1", filters={"price": "$100"})
    hits = mem.get_all(user_id="u1", filters={"price": "$100"})
    assert len(hits) == 1


def test_mem0_filter_rejects_operators_and_dicts(mem) -> None:
    with pytest.raises(ValueError):
        mem.search("q", user_id="u1", filters={"$eq": "x"})
    with pytest.raises(TypeError):
        mem.search("q", user_id="u1", filters={"topic": {"nested": "dict"}})
    with pytest.raises(TypeError):
        mem.search("q", user_id="u1", filters={"topic": ["a"]})


def test_mem0_scope_conflict_and_reserved_reject(mem) -> None:
    with pytest.raises(ValueError):
        mem.add("f", user_id="u1", metadata={"user_id": "evil"})
    with pytest.raises(ValueError):
        mem.add("f", user_id="u1", metadata={"topic": "a"}, filters={"topic": "b"})
    ok = mem.add("f", user_id="u1", metadata={"topic": "a"}, filters={"topic": "a"})
    assert ok[0]["id"]


def test_mem0_invalid_messages_rejected_before_writes(mem) -> None:
    with pytest.raises(TypeError):
        mem.add({"bad": "type"}, user_id="u1")
    with pytest.raises(TypeError):
        mem.add([{"role": "user", "content": 42}], user_id="u1")
    with pytest.raises(TypeError):
        mem.add([{"role": 7, "content": "x"}], user_id="u1")
    assert mem.get_all(user_id="u1") == []
    with pytest.raises(ValueError):
        mem.add("x", user_id="")
    with pytest.raises(ValueError):
        mem.search("q", user_id="u1", limit=True)
    with pytest.raises(ValueError):
        mem.search("q", user_id="u1", limit=0)


def test_mem0_encoder_nan_inf_rejected(tmp_path) -> None:
    mod = _load_adapter()
    for bad_vec in ([float("nan")] * 8, [float("inf")] + [0.0] * 7, []):
        m = mod.VantaMemory(
            db_path=str(tmp_path / f"db_{len(bad_vec)}"),
            namespace="n",
            embedding_model=lambda t, v=bad_vec: v,
        )
        with pytest.raises(ValueError, match="finite|non-empty"):
            m.add("x", user_id="u1")
        m.close()


def test_mem0_delete_and_notfound(mem) -> None:
    rid = mem.add("del me", user_id="u1")[0]["id"]
    assert mem.delete(rid) is True
    assert mem.get(rid) is None
    assert mem.delete(rid) is False
    assert mem.delete("nonexistent-key") is False
    assert mem.get("nonexistent-key") is None


def test_mem0_limit_respected(mem) -> None:
    for i in range(7):
        mem.add(f"n{i}", user_id="u1")
    assert len(mem.get_all(user_id="u1", limit=3)) == 3


def test_mem0_foreign_records_not_exposed(tmp_path) -> None:
    mod = _load_adapter()
    m1 = mod.VantaMemory(
        db_path=str(tmp_path / "db"),
        namespace="shared/ns",
        embedding_model=_fake_encoder(),
    )
    m1.add("mine", user_id="u1")
    m1._client().put("shared/ns", "foreign-key", "not mine", metadata={"user_id": "u1"})
    ids = [r["id"] for r in m1.get_all(user_id="u1")]
    assert "foreign-key" not in ids
    assert m1.get("foreign-key") is None
    assert m1.delete("foreign-key") is False
    m1.close()


def test_mem0_reset_paginates_only_own(tmp_path) -> None:
    mod = _load_adapter()
    m = mod.VantaMemory(
        db_path=str(tmp_path / "db2"),
        namespace="mem0/reset",
        embedding_model=_fake_encoder(),
    )
    for i in range(120):
        m.add(f"fact-{i}", user_id="uR")
    db = m._client()
    db.put("mem0/reset", "foreign", "keep", metadata={})
    db.put("other/ns", "other", "keep", metadata={})
    res = m.reset()
    assert res["deleted"] == 120
    m.close()
    from vantadb import Client

    db2 = Client(str(tmp_path / "db2"))
    assert db2.memory.get("mem0/reset", "foreign") is not None
    assert db2.memory.get("other/ns", "other") is not None
    db2.close()


def test_mem0_reset_no_progress_raises(monkeypatch, tmp_path) -> None:
    mod = _load_adapter()
    record = type(
        "R",
        (),
        {
            "key": "k1",
            "payload": "x",
            "metadata": {mod._MARKER: True},
            "created_at_ms": 0,
            "updated_at_ms": 0,
        },
    )()

    class Memory:
        calls = 0

        def list(self, ns, filters=None, limit=None):
            Memory.calls += 1
            if Memory.calls > 2:
                raise AssertionError("reset looped without progress")
            return type("P", (), {"records": [record]})()

        def delete(self, ns, key):
            return False

        def get(self, ns, key):
            return None

    class StubClient:
        def __init__(self, path):
            self.memory = Memory()

        def flush(self):
            pass

        def close(self):
            pass

    import vantadb

    monkeypatch.setattr(vantadb, "Client", StubClient)
    m = mod.VantaMemory(
        db_path=str(tmp_path / "db_np"),
        namespace="n",
        embedding_model=_fake_encoder(),
    )
    with pytest.raises(RuntimeError, match="no progress"):
        m.reset()
    assert Memory.calls == 1


def test_mem0_close_reopen_persists(tmp_path) -> None:
    mod = _load_adapter()
    p = str(tmp_path / "persist")
    m = mod.VantaMemory(db_path=p, namespace="mem0/p", embedding_model=_fake_encoder())
    rid = m.add("durable", user_id="u1")[0]["id"]
    m.close()
    m2 = mod.VantaMemory(db_path=p, namespace="mem0/p", embedding_model=_fake_encoder())
    assert m2.get(rid)["memory"] == "durable"
    m2.close()
