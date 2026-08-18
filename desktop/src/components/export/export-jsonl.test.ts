// VS-16 JSONL export tests — pure serialization (node --test).
import { test } from "node:test";
import assert from "node:assert/strict";
import { recordsToJsonl } from "./export-jsonl.ts";

test("recordsToJsonl: one object per line, importable IngestItem shape", () => {
  const out = recordsToJsonl([
    {
      id: "a",
      namespace: "ns",
      text: "hello",
      metadata: { k: 1 },
      vector: [0.5, 0.25],
      created_at_ms: 1,
      version: 3,
      expires_at_ms: 999,
    },
    {
      id: "b",
      namespace: "ns2",
      text: "world",
    },
  ]);
  const lines = out.split("\n");
  assert.equal(lines.length, 2);
  const first = JSON.parse(lines[0]);
  // Roundtrip contract: emit exactly {id, namespace, text, embedding, metadata}.
  assert.deepEqual(first, {
    id: "a",
    namespace: "ns",
    text: "hello",
    embedding: [0.5, 0.25],
    metadata: { k: 1 },
  });
  // Non-IngestItem fields (created_at, version, ttl) are dropped, not emitted.
  assert.equal(JSON.stringify(lines[0]).includes("created_at_ms"), false);
  assert.equal(JSON.stringify(lines[0]).includes("version"), false);
  assert.equal(JSON.stringify(lines[0]).includes("expires_at_ms"), false);
});

test("recordsToJsonl: empty input", () => {
  assert.equal(recordsToJsonl([]), "");
});

test("recordsToJsonl: omits undefined vector/metadata (no nulls on wire)", () => {
  const out = recordsToJsonl([{ id: "x", namespace: "n", text: "t" }]);
  const obj = JSON.parse(out);
  assert.equal("embedding" in obj, false);
  assert.equal("metadata" in obj, false);
});