/**
 * FIND-156 — `Client.parse()` exposes the core's AST JSON.
 *
 * The projection belongs to the core: `vantadb::parser::parse_statement` +
 * `serde::Serialize` (shape pinned by
 * `src/parser/mod.rs::tests::test_ast_json_projection_shape`, documented in
 * `docs/api/IQL.md` §AST JSON). This test pins the *binding* to that same
 * snapshot — a divergence means the TS/WASM layer re-serialized the AST by
 * hand instead of reusing the core projection.
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client } from "../vantadb.js";

// Same statement the core test uses, so both snapshots describe one shape.
const QUERY = "FROM Person p WHERE edad == 28 FETCH name";

// Verbatim from docs/api/IQL.md §AST JSON — same query, same shape.
const EXPECTED = {
  Query: {
    from_entity: "Person",
    traversal: null,
    target_alias: "p",
    where_clause: [{ Relational: ["edad", "Eq", { Int: 28 }] }],
    fetch: ["name"],
    rank_by: null,
    temperature: null,
    owner_role: null,
    search_profile: null,
    as_of_ms: null,
    limit: null,
    offset: null,
  },
};

/** Shape of the fields the core test asserts (narrowing only; not a mirror). */
interface QueryNode {
  from_entity: string;
  target_alias: string;
  where_clause: { Relational: [string, string, { Int: number }] }[];
  fetch: string[];
  search_profile: unknown;
  limit: unknown;
  offset: unknown;
}

describe("FIND-156: parse() = the core AST JSON", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("matches the documented snapshot exactly", () => {
    expect(db.parse(QUERY)).toEqual(EXPECTED);
  });

  it("pins the same fields as test_ast_json_projection_shape", () => {
    const q = db.parse(QUERY).Query as unknown as QueryNode;
    expect(q.from_entity).toBe("Person");
    expect(q.target_alias).toBe("p");
    expect(q.where_clause[0].Relational[0]).toBe("edad");
    expect(q.where_clause[0].Relational[1]).toBe("Eq");
    expect(q.where_clause[0].Relational[2].Int).toBe(28);
    expect(q.fetch).toEqual(["name"]);
    expect(q.search_profile).toBeNull();
    expect(q.limit).toBeNull();
    expect(q.offset).toBeNull();
  });

  it("exposes the additive pagination fields (WIRE-12)", () => {
    const q = db.parse("FROM Doc LIMIT 5 OFFSET 2").Query as unknown as QueryNode;
    expect(q.limit).toBe(5);
    expect(q.offset).toBe(2);
  });

  it("returns plain JSON data, not a Map or class instance", () => {
    const ast = db.parse(QUERY);
    expect(ast).not.toBeInstanceOf(Map);
    expect(JSON.parse(JSON.stringify(ast))).toEqual(EXPECTED);
  });

  it("parses non-Query variants too", () => {
    const ast = db.parse('INSERT MESSAGE SYSTEM "hi" TO THREAD#1');
    expect(Object.keys(ast)).toContain("InsertMessage");
  });

  it("rejects invalid IQL", () => {
    // NB: `SELECT ... FROM ...` is VALID IQL (Statement::Select) — not an error.
    expect(() => db.parse("FROM")).toThrow();
  });

  it("is pure: parses on a closed client (no op-gate)", () => {
    const closed = Client.create();
    closed.close();
    expect(closed.parse(QUERY)).toEqual(EXPECTED);
  });

  it("pins the documented >2^53 precision limitation", () => {
    // FIND-156 review, Critical-1: the value crosses into JS via JSON.parse, so
    // every number is an IEEE-754 double. This test LOCKS that known limitation
    // (Python, decoding with json.loads, stays exact). When the core starts
    // serializing u128/i64 as strings, this assertion must flip to `.toBe(...)`
    // on the exact decimal string AND the docs' caveat must be removed.
    const ast = db.parse("FROM Person p WHERE edad == 9007199254740993");
    const where = ast.Query as unknown as QueryNode;
    const lit = where.where_clause[0].Relational[2].Int;
    expect(typeof lit).toBe("number");
    expect(Number.isSafeInteger(lit)).toBe(false);
    expect(lit).toBe(9007199254740992); // off by one: 2^53 + 1 is not representable
  });
});
