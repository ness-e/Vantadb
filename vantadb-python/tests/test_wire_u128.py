"""
API-01 wire tests: u128 ids > 2^53 must cross the Python boundary intact.

Covers the W0 change in the core SDK: ``QueryResult::Write.node_id`` now
serializes as a decimal string (``u128_serde``), so ids above 2^53 survive
any JSON bridge. Plus the memory-record ``node_id`` (u128 ``XxHash3_128``)
exactness guarantee on the Python side.

RED-before-GREEN note: ``test_query_write_node_id_is_decimal_string`` fails
on a core where ``Write.node_id`` is serialized as a JSON number — f64 would
round ``9007199254740993`` to ``9007199254740992`` (or lose the string type).
"""

import vantadb_py as vanta

BIG = 2**53 + 1  # 9007199254740993 — first integer an f64 cannot represent


def _client(tmp_path, name):
    return vanta.Client(str(tmp_path / name), memory_limit_bytes=128 * 1024 * 1024)


def test_memory_record_node_id_is_exact_u128(tmp_path):
    db = _client(tmp_path, "api01_mem")
    try:
        record = db.put("api01", "big-id", "payload")
        node_id = record["node_id"]
        assert node_id > 2**53, f"xxhash128 node id must exceed 2^53, got {node_id}"

        fetched = db.memory.get("api01", "big-id")
        assert fetched is not None
        assert fetched["node_id"] == node_id, "u128 id must round-trip exactly"
    finally:
        db.close()


def test_query_write_node_id_is_decimal_string(tmp_path):
    db = _client(tmp_path, "api01_iql")
    try:
        result = db.query_structured(f"INSERT NODE#{BIG} TYPE Person {{}}")
        assert result["kind"] == "write", f"expected a write result, got {result!r}"

        node_id = result["node_id"]
        assert isinstance(node_id, str), (
            "u128 node_id must cross as a decimal string, "
            f"got {type(node_id).__name__}"
        )
        assert node_id == str(BIG), (
            f"f64 would round {BIG} -> 9007199254740992; got {node_id!r}"
        )
    finally:
        db.close()
