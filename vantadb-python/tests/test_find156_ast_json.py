"""FIND-156 — the Python binding exposes the core's AST JSON.

The projection belongs to the core: ``vantadb::parser::parse_statement`` +
``serde::Serialize`` (shape pinned by
``src/parser/mod.rs::tests::test_ast_json_projection_shape`` and documented in
``docs/api/IQL.md`` §AST JSON). This test pins the *binding* to that same
snapshot — a divergence means ``Client.parse`` re-serialized the AST by hand
instead of reusing the core projection.
"""

import json

import pytest
import vantadb_py as vanta

# Same statement the core test uses, so both snapshots describe one shape.
QUERY = "FROM Person p WHERE edad == 28 FETCH name"

# Verbatim from docs/api/IQL.md §AST JSON — same query, same shape.
EXPECTED = {
    "Query": {
        "from_entity": "Person",
        "traversal": None,
        "target_alias": "p",
        "where_clause": [{"Relational": ["edad", "Eq", {"Int": 28}]}],
        "fetch": ["name"],
        "rank_by": None,
        "temperature": None,
        "owner_role": None,
        "search_profile": None,
        "as_of_ms": None,
        "limit": None,
        "offset": None,
    }
}


def _db():
    return vanta.Client(":memory:", backend="memory")


def test_parse_matches_the_core_ast_json_snapshot():
    """Exact equality with the documented / core-pinned projection."""
    assert _db().parse(QUERY) == EXPECTED


def test_parse_pins_the_same_fields_the_core_test_pins():
    """The four assertions of ``test_ast_json_projection_shape``, from Python."""
    ast = _db().parse(QUERY)
    assert ast["Query"]["from_entity"] == "Person"
    assert ast["Query"]["target_alias"] == "p"
    assert ast["Query"]["where_clause"][0]["Relational"][0] == "edad"
    assert ast["Query"]["where_clause"][0]["Relational"][1] == "Eq"
    assert ast["Query"]["where_clause"][0]["Relational"][2]["Int"] == 28
    assert ast["Query"]["fetch"] == ["name"]
    assert ast["Query"]["search_profile"] is None
    assert ast["Query"]["limit"] is None
    assert ast["Query"]["offset"] is None


def test_parse_exposes_additive_pagination_fields():
    """WIRE-12: ``limit``/``offset`` serialize additively (null when absent)."""
    ast = _db().parse("FROM Doc LIMIT 5 OFFSET 2")
    assert ast["Query"]["limit"] == 5
    assert ast["Query"]["offset"] == 2


def test_parse_result_is_json_compatible():
    """The returned value is plain JSON data, not a binding-specific wrapper."""
    ast = _db().parse(QUERY)
    assert json.loads(json.dumps(ast)) == EXPECTED


def test_parse_rejects_invalid_iql_as_validation_error():
    """Parse failures keep the SDK taxonomy and its canonical error code."""
    # NB: `SELECT ... FROM ...` is VALID IQL (Statement::Select) — not a parse error.
    with pytest.raises(vanta.ValidationError, match="IQL parse error") as exc:
        _db().parse("FROM")
    assert exc.value.code == "VANTADB_VALIDATION_ERROR"


def test_parse_rejects_trailing_input():
    """Leftover input after a complete statement is an error, as in ``query``."""
    with pytest.raises(vanta.ValidationError, match="IQL parse error"):
        _db().parse("FROM Person p extra_stuff")


def test_parse_handles_non_query_variants():
    """The AST is the whole statement enum, not just ``Query``."""
    ast = _db().parse('INSERT MESSAGE SYSTEM "hi" TO THREAD#1')
    assert "InsertMessage" in ast


def test_parse_is_pure_and_works_on_a_closed_client():
    """No ``op_gate`` and no engine access: same projection after ``close()``."""
    db = _db()
    db.close()
    assert db.parse(QUERY) == EXPECTED
