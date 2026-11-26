/**
 * D5d: DTO id simétrico `number | bigint` en get/delete/addEdge/removeEdge.
 *
 * Background (verificado 2026-09-12 contra `vantadb-wasm/src/lib.rs:1714-1762`
 * y `:2290-2296`):
 *   - El wire recibe `id: &str` y parsea `u128` (`parse_node_id`) — el wire
 *     NO impone el límite 2^53. `String(bigint)` es exacto.
 *   - `insertNode` ya acepta `number | bigint` + guard safe-integer, pero
 *     `getNode`/`deleteNode`/`addEdge`/`removeEdge` solo aceptaban `number`:
 *     IDs >2^53 se perdían en lectura (o ni compilaban con bigint).
 *
 * Contract: este archivo usa bigint >2^53 en los 4 métodos (+ delegados
 * `db.graph.*`). Antes del fix, `tsc` falla TS2345 (bigint no asignable a
 * number); después del fix, todo verde. Los tests de número inseguro exigen
 * el mismo guard `INVALID_ARGUMENT` que `insertNode`.
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { Client, DbError } from "../src/vantadb.js";

// 2^53 + 1 — el primer entero que un `number` JS no puede representar.
const BIG = 9007199254740993n;
// u128 grande (cabe en el wire `parse_node_id` → u128).
const HUGE = 123456789012345678901234567890n;
// D5d intencional: primer entero no representable como `number` — ejerce el
// guard safe-integer (el literal pierde precisión en parseo, eso es el punto).
// eslint-disable-next-line no-loss-of-precision
const UNSAFE_NUMBER = 9007199254740993;

describe("D5d: bigint IDs simétricos en lectura/escritura de grafo", () => {
  let db: Client;

  beforeAll(() => {
    db = Client.create();
  });

  afterAll(() => {
    db.close();
  });

  it("getNode(bigint >2^53) round-trip tras insertNode(bigint)", () => {
    db.insertNode(BIG, "big-node");
    const node = db.getNode(BIG);
    expect(node).not.toBeNull();
  });

  it("addEdge(bigint, bigint) + getNode expone target bigint exacto", () => {
    db.insertNode(HUGE, "huge-node");
    db.addEdge(BIG, HUGE, "links", 0.5);
    const node = db.getNode(BIG);
    expect(node).not.toBeNull();
    expect(node!.edges.map((e) => e.target)).toContain(HUGE);
  });

  it("removeEdge(bigint, bigint) elimina la arista", () => {
    db.removeEdge(BIG, HUGE, "links");
    const node = db.getNode(BIG);
    expect(node).not.toBeNull();
    expect(node!.edges.map((e) => e.target)).not.toContain(HUGE);
  });

  it("deleteNode(bigint) elimina el nodo", () => {
    db.deleteNode(HUGE, "cleanup");
    expect(db.getNode(HUGE)).toBeNull();
  });

  it("delegados db.graph.* aceptan bigint", () => {
    const G = 9007199254740995n;
    db.graph.insertNode(G, "via-subclient");
    expect(db.graph.getNode(G)).not.toBeNull();
    db.graph.addEdge(BIG, G, "sub");
    db.graph.removeEdge(BIG, G, "sub");
    db.graph.deleteNode(G, "cleanup");
    expect(db.graph.getNode(G)).toBeNull();
  });

  it("getNode(number inseguro) lanza INVALID_ARGUMENT (como insertNode)", () => {
    expect(() => db.getNode(UNSAFE_NUMBER)).toThrow(DbError);
    expect(() => db.getNode(UNSAFE_NUMBER)).toThrow(/safe integer/i);
  });

  it("deleteNode/addEdge con number inseguro lanzan INVALID_ARGUMENT", () => {
    expect(() => db.deleteNode(UNSAFE_NUMBER, "x")).toThrow(/safe integer/i);
    expect(() => db.addEdge(UNSAFE_NUMBER, 1, "x")).toThrow(/safe integer/i);
    expect(() => db.addEdge(1, UNSAFE_NUMBER, "x")).toThrow(/safe integer/i);
    expect(() => db.removeEdge(UNSAFE_NUMBER, 1, "x")).toThrow(/safe integer/i);
  });
});
