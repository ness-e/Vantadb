// VS-16 status report tests — pure markdown builder (node --test).
import { test } from "node:test";
import assert from "node:assert/strict";
import { buildStatusReport } from "./statusReport.ts";

const FIXED = "2026-08-18T12:00:00.000Z";

test("buildStatusReport: counts per namespace + metadata types", () => {
  const md = buildStatusReport(
    [
      { id: "a", namespace: "ns1", text: "x", metadata: { kind: "note", n: 2 } },
      { id: "b", namespace: "ns1", text: "y", metadata: { kind: "todo" } },
      { id: "c", namespace: "ns2", text: "z", metadata: { flag: true } },
    ],
    { generatedAt: FIXED },
  );
  assert.match(md, /Records in view: 3/);
  assert.match(md, /\| `ns1` \| 2 \|/);
  assert.match(md, /\| `ns2` \| 1 \|/);
  assert.match(md, /\| `kind` \| `string` \|/);
  assert.match(md, /\| `n` \| `int` \|/);
  assert.match(md, /\| `flag` \| `bool` \|/);
});

test("buildStatusReport: upcoming TTLs sorted, only future ones", () => {
  const now = Date.now();
  const md = buildStatusReport(
    [
      { id: "soon", namespace: "n", text: "x", expires_at_ms: now + 60_000 },
      { id: "later", namespace: "n", text: "y", expires_at_ms: now + 3_600_000 },
      { id: "past", namespace: "n", text: "z", expires_at_ms: now - 5_000 },
      { id: "none", namespace: "n", text: "w" },
    ],
    { generatedAt: FIXED, includeUpcomingTtls: true },
  );
  const soonIdx = md.indexOf("`soon`");
  const laterIdx = md.indexOf("`later`");
  assert.ok(soonIdx >= 0 && laterIdx >= 0 && soonIdx < laterIdx, "sorted by expiry");
  assert.match(md, /\| `soon` \| `n` \| .* \| 1m \|/);
  assert.equal(md.includes("`past`"), false);
  assert.equal(md.includes("`none`"), false);
});

test("buildStatusReport: empty view", () => {
  const md = buildStatusReport([], { generatedAt: FIXED });
  assert.match(md, /Records in view: 0/);
  assert.match(md, /No metadata fields present/);
  assert.equal(md.includes("Upcoming expirations"), false);
});