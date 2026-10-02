/**
 * API-01 wire test: `QueryResult::Write.node_id` (u128) must cross the WASM
 * boundary as an exact decimal string — ids > 2^53 must not round through f64.
 *
 * RED-before-GREEN: before the core `u128_serde` fix, `INSERT NODE#2^53+1`
 * came back as the number 9007199254740992 (silent precision loss).
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client } from "../src/vantadb.js";

// 2^53 + 1 — the first integer a JS `number` cannot represent exactly.
const BIG = 9007199254740993n;

describe("API-01: QueryResult::Write.node_id > 2^53 exacto (string|bigint)", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("INSERT con id > 2^53 devuelve node_id exacto (string o bigint)", () => {
    const result = db.query(`INSERT NODE#${BIG} TYPE Person {}`);
    expect(result.Write, "expected a Write result").toBeTruthy();

    const nodeId = result.Write!.node_id;
    // Representation depends on the serializer at the boundary:
    // — fresh `u128_serde` wire (human-readable serializers): decimal string
    // — prebuilt/stale WASM without the fix: bigint (still exact)
    // — an f64/number would be silent precision loss and must never appear.
    expect(typeof nodeId).not.toBe("number");
    const exact =
      typeof nodeId === "bigint" ? nodeId : BigInt(nodeId as string);
    expect(exact).toBe(BIG);
    // The id must survive a JSON round-trip (the actual wire constraint).
    const asJson = { node_id: nodeId.toString() };
    expect(BigInt(JSON.parse(JSON.stringify(asJson)).node_id)).toBe(BIG);
  });
});
