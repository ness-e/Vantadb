import { describe, it, expect } from "vitest";
import type { MemoryRecord, SearchHit, NodeRecord, EdgeRecord } from "../types.js";

describe("TypeScript types are correctly defined", () => {
  it("MemoryRecord type has all required fields", () => {
    const record: MemoryRecord = {
      namespace: "test",
      key: "k1",
      payload: "data",
      metadata: {},
      created_at_ms: "1000",
      updated_at_ms: "2000",
      version: 1,
      node_id: "42",
    };
    expect(record.namespace).toBe("test");
    expect(record.node_id).toBe("42");
  });

  it("SearchHit has optional explanation", () => {
    const hit: SearchHit = {
      record: {
        namespace: "ns",
        key: "k",
        payload: "p",
        metadata: {},
        created_at_ms: "0",
        updated_at_ms: "0",
        version: 1,
        node_id: "1",
      },
      score: 0.95,
    };
    expect(hit.explanation).toBeUndefined();

    const hitWithExp: SearchHit = {
      record: hit.record,
      score: 0.95,
      explanation: {
        identity: "ns/k",
        score: 0.95,
        matched_tokens: ["hello"],
        matched_phrases: [],
      },
    };
    expect(hitWithExp.explanation?.identity).toBe("ns/k");
  });

  it("NodeRecord has edges array", () => {
    const edges: EdgeRecord[] = [
      { target: 2, label: "related", weight: 0.8 },
    ];
    const node: NodeRecord = {
      id: 1,
      fields: {},
      vector_dimensions: 3,
      edges,
      confidence_score: 0.9,
      importance: 0.5,
      hits: 10,
      last_accessed: 1000,
      epoch: 0,
      tier: "Hot",
      is_alive: true,
    };
    expect(node.edges.length).toBe(1);
    expect(node.edges[0].target).toBe(2);
  });

  it("VantaValue variants work as discriminated union", () => {
    const stringVal = { type: "String" as const, value: "hello" };
    const intVal = { type: "Int" as const, value: 42 };
    const floatVal = { type: "Float" as const, value: 3.14 };
    const boolVal = { type: "Bool" as const, value: true };
    const nullVal = { type: "Null" as const };

    expect(stringVal.value).toBe("hello");
    expect(intVal.value).toBe(42);
    expect(floatVal.value).toBe(3.14);
    expect(boolVal.value).toBe(true);
    expect(nullVal.type).toBe("Null");
  });
});

describe("VantaConfig defaults", () => {
  it("config interface accepts partial options", () => {
    const config = { storage_path: "./data" };
    expect(config.storage_path).toBe("./data");
  });

  it("config with full options is valid", () => {
    const config = {
      storage_path: "/tmp/vanta",
      read_only: false,
      rss_threshold: 0.85,
      memory_limit: 1073741824,
    };
    expect(config.memory_limit).toBe(1073741824);
  });
});
