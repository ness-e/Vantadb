"""Smoke test: migrate demo records from ChromaDB and LanceDB into VantaDB.

Verifies that migrate_from_chroma / migrate_from_lancedb write records that
can be recovered with get_memory() and search_memory().
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from vantadb_py import VantaDB
from vantadb_py.migrate import migrate_from_chroma, migrate_from_lancedb

CHROMA_DOCS = [
    ("c1", "VantaDB is an embedded vector database.", {"source": "docs", "page": 1}, [1.0, 0.0, 0.0]),
    ("c2", "It supports Python, TypeScript, and Rust.", {"source": "docs", "page": 2}, [0.0, 1.0, 0.0]),
    ("c3", "Graph edges connect related memories.", {"source": "blog", "page": 3}, [0.0, 0.0, 1.0]),
]

LANCE_ROWS = [
    {"id": "l1", "vector": [1.0, 0.0, 0.0], "text": "Hybrid search fuses BM25 with HNSW.", "source": "docs"},
    {"id": "l2", "vector": [0.0, 1.0, 0.0], "text": "TTL expiry removes stale records.", "source": "docs"},
    {"id": "l3", "vector": [0.0, 0.0, 1.0], "text": "Namespaces are created lazily on put.", "source": "blog"},
]


@pytest.fixture()
def chroma_source() -> Path:
    chromadb = pytest.importorskip("chromadb")
    path = Path(tempfile.mkdtemp())
    client = chromadb.PersistentClient(path=str(path))
    col = client.create_collection("my_docs")
    col.add(
        ids=[d[0] for d in CHROMA_DOCS],
        documents=[d[1] for d in CHROMA_DOCS],
        metadatas=[d[2] for d in CHROMA_DOCS],
        embeddings=[d[3] for d in CHROMA_DOCS],
    )
    return path


@pytest.fixture()
def lancedb_source() -> Path:
    lancedb = pytest.importorskip("lancedb")
    path = Path(tempfile.mkdtemp())
    db = lancedb.connect(str(path))
    db.create_table("my_table", data=LANCE_ROWS)
    return path


def _verify(target_path: Path, namespace: str, expected: list[tuple[str, list[float]]]) -> None:
    db = VantaDB(str(target_path))
    try:
        for key, vector in expected:
            record = db.get_memory(namespace, key)
            assert record is not None, f"{namespace}/{key} missing"
            assert record.payload
        hits = db.search_memory(namespace, expected[0][1], top_k=3)
        assert len(hits) >= 1, "search_memory returned no hits"
        assert hits[0].key == expected[0][0], f"top hit {hits[0].key} != {expected[0][0]}"
    finally:
        db.close()


def test_migrate_from_chroma(chroma_source: Path) -> None:
    dest = Path(tempfile.mkdtemp()) / "vanta_db"
    count = migrate_from_chroma(str(chroma_source), str(dest))
    assert count == 3
    _verify(dest, "my_docs", [(d[0], d[3]) for d in CHROMA_DOCS])


def test_migrate_from_chroma_custom_namespace(chroma_source: Path) -> None:
    dest = Path(tempfile.mkdtemp()) / "vanta_db"
    count = migrate_from_chroma(str(chroma_source), str(dest), namespace="memories")
    assert count == 3
    _verify(dest, "memories", [(d[0], d[3]) for d in CHROMA_DOCS])


def test_migrate_from_lancedb(lancedb_source: Path) -> None:
    dest = Path(tempfile.mkdtemp()) / "vanta_db"
    count = migrate_from_lancedb(str(lancedb_source), str(dest))
    assert count == 3
    _verify(dest, "my_table", [(r["id"], r["vector"]) for r in LANCE_ROWS])


def test_migrate_from_lancedb_custom_table(lancedb_source: Path) -> None:
    dest = Path(tempfile.mkdtemp()) / "vanta_db"
    count = migrate_from_lancedb(str(lancedb_source), str(dest), table_name="my_table")
    assert count == 3
    _verify(dest, "my_table", [(r["id"], r["vector"]) for r in LANCE_ROWS])
