"""AsyncVantaDB smoke test (COV-001).

Exercises the async wrapper paths the coverage plan flagged as under-exercised:
``flush``, ``purge_expired``, ``query`` (IQL), the graph_* traversal/algorithms
(``add_edge``, ``graph_bfs``, ``graph_dfs``, ``graph_topological_sort``,
``graph_is_dag``, ``graph_page_rank``, ``graph_degree_centrality``), ``put``,
``delete`` / ``delete_memory``, and ``export_*`` (``export_namespace`` /
``export_all``).

Additive only — it does NOT modify the public ``AsyncVantaDB`` API.
"""

import asyncio
import os
import shutil
import tempfile

import vantadb_py as vanta


def _tmp_db():
    return tempfile.mkdtemp(prefix="cov001_")


def _rm(path):
    shutil.rmtree(path, ignore_errors=True)


def test_async_smoke_crud_flush_purge():
    """put/get_memory/search_memory, flush, ttl purge, and delete_memory."""
    path = _tmp_db()
    try:
        async def run():
            async with vanta.AsyncVantaDB(
                path, memory_limit_bytes=128 * 1024 * 1024
            ) as db:
                await db.put("ns", "k", "hello", metadata={"tag": "smoke"})
                rec = await db.get_memory("ns", "k")
                assert rec is not None, "get_memory should return the put record"
                assert rec["payload"] == "hello", f"expected 'hello', got {rec['payload']}"

                hits = await db.search_memory("ns", [1.0, 0.0, 0.0], top_k=5)
                assert isinstance(hits, list), "search_memory should return a list"

                await db.flush()

                await db.put("ns", "exp", "gone", ttl_ms=1)
                for _ in range(80):
                    if await db.get_memory("ns", "exp") is None:
                        break
                    await asyncio.sleep(0.02)
                purged = await db.purge_expired()
                assert purged >= 1, f"expected >=1 purged, got {purged}"

                deleted = await db.delete_memory("ns", "k")
                assert deleted is True, "delete_memory should return True"

        asyncio.run(run())
    finally:
        _rm(path)


def test_async_smoke_query_graph():
    """query (IQL), graph traversal/algorithms, and low-level delete by id."""
    path = _tmp_db()
    try:
        async def run():
            async with vanta.AsyncVantaDB(
                path, memory_limit_bytes=128 * 1024 * 1024
            ) as db:
                write = await db.query('INSERT NODE#7 TYPE Person { name: "smoke" }')
                assert isinstance(write, str), "INSERT should return a str"

                result = await db.query("FROM Person")
                assert isinstance(result, str) and "7" in result, \
                    f"query result should mention node 7, got {result!r}"

                await db.insert(1, "A", [])
                await db.insert(2, "B", [])
                await db.add_edge(1, 2, "next", weight=0.5)

                bfs = await db.graph_bfs([1])
                assert 2 in bfs, f"expected target 2 in BFS, got {bfs}"
                dfs = await db.graph_dfs([1])
                assert 2 in dfs, f"expected target 2 in DFS, got {dfs}"

                order = await db.graph_topological_sort([1])
                assert isinstance(order, list) and set(order) == {1, 2}, \
                    f"expected topo order over {{1,2}}, got {order}"

                assert await db.graph_is_dag([1]) is True, "expected DAG"

                ranks = await db.graph_page_rank([1], max_iterations=20)
                assert 1 in ranks and 2 in ranks, f"expected ranks for 1,2, got {ranks.keys()}"

                cent = await db.graph_degree_centrality([1, 2])
                assert 1 in cent and 2 in cent, f"expected centrality for 1,2, got {cent}"

                await db.delete(1, "smoke cleanup")
                assert await db.get(1) is None, "node 1 should be None after delete"

        asyncio.run(run())
    finally:
        _rm(path)


def test_async_smoke_export():
    """export_namespace and export_all round-trip to disk."""
    path = _tmp_db()
    try:
        with tempfile.TemporaryDirectory() as tmp:
            async def run():
                async with vanta.AsyncVantaDB(
                    path, memory_limit_bytes=128 * 1024 * 1024
                ) as db:
                    await db.put(
                        "agent/main", "exp", "portable",
                        metadata={"c": "note"}, vector=[1.0, 0.0, 0.0],
                    )
                    await db.flush()

                    ns_path = os.path.join(tmp, "ns.jsonl")
                    all_path = os.path.join(tmp, "all.jsonl")

                    exp = await db.export_namespace(ns_path, "agent/main")
                    assert exp["records_exported"] == 1, f"expected 1 exported, got {exp}"
                    assert os.path.exists(ns_path), f"export file missing: {ns_path}"

                    all_e = await db.export_all(all_path)
                    assert all_e["records_exported"] == 1, f"expected 1, got {all_e['records_exported']}"

            asyncio.run(run())
    finally:
        _rm(path)
