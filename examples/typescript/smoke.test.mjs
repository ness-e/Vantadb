import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { cpSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { Client } from "vantadb";

import { runQuickstart } from "./node-quickstart.mjs";

const NS = "test/smoke";
const HERE = fileURLToPath(new URL(".", import.meta.url));

test("CRUD lifecycle on the real SDK", () => {
  const db = Client.create();
  try {
    db.put({
      namespace: NS,
      key: "a",
      payload: "alpha",
      metadata: { n: 1 },
      vector: [1, 0, 0],
    });
    db.put({ namespace: NS, key: "b", payload: "beta", vector: [0, 1, 0] });

    const got = db.get({ namespace: NS, key: "a" });
    assert.equal(got.payload, "alpha");
    assert.deepEqual(got.metadata.n, { Int: 1 });
    assert.equal(db.get({ namespace: NS, key: "missing" }), null);

    assert.equal(db.delete({ namespace: NS, key: "b" }), true);
    assert.equal(db.get({ namespace: NS, key: "b" }), null);

    const page = db.list({ namespace: NS });
    assert.equal(page.records.length, 1);
  } finally {
    db.close();
  }
});

test("hybrid search returns the stored record", () => {
  const db = Client.create();
  try {
    db.put({ namespace: NS, key: "v1", payload: "vector hit", vector: [1, 0, 0] });
    db.put({ namespace: NS, key: "t1", payload: "exact phrase token" });

    const vec = db.search({ namespace: NS, query_vector: [0.9, 0.1, 0.0], top_k: 5 });
    assert.ok(vec.some((h) => h.record.key === "v1"));

    const txt = db.search({ namespace: NS, text_query: "phrase", top_k: 5 });
    assert.ok(txt.some((h) => h.record.key === "t1"));

    const mixed = db.search({
      namespace: NS,
      query_vector: [0.9, 0.1, 0.0],
      text_query: "phrase",
      top_k: 5,
    });
    assert.ok(mixed.length >= 1);
    assert.ok(mixed.every((h) => h.record && typeof h.record.key === "string"));
  } finally {
    db.close();
  }
});

test("closed client lifecycle", () => {
  const db = Client.create();
  db.put({ namespace: NS, key: "x", payload: "y" });
  db.close();
  assert.throws(() => db.get({ namespace: NS, key: "x" }));
});

test("quickstart example returns a valid summary", async () => {
  const r = await runQuickstart();
  assert.equal(r.put, "fact_001");
  assert.equal(r.got, "fact_001");
  assert.ok(r.hits >= 1);
});

test("quickstart CLI runs via relative path from repo root", () => {
  const out = execFileSync(process.execPath, ["examples/typescript/node-quickstart.mjs"], {
    cwd: resolve(HERE, "../.."),
    encoding: "utf8",
  });
  assert.match(out, /put: demo\/quickstart\/fact_001/);
  assert.match(out, /get: /);
  assert.match(out, /search: fact_001/);
});

test("quickstart CLI runs via absolute path with spaces", (t) => {
  const scratch = mkdtempSync(resolve(HERE, ".scratch dir "));
  t.after(() => rmSync(scratch, { recursive: true, force: true }));
  const copy = resolve(scratch, "node-quickstart.mjs");
  cpSync(resolve(HERE, "node-quickstart.mjs"), copy);
  assert.ok(copy.includes(" "), "scratch path must contain spaces");
  const out = execFileSync(process.execPath, [copy], { encoding: "utf8" });
  assert.match(out, /put: demo\/quickstart\/fact_001/);
});

test("existing TS examples use canonical Client import, no VantaDB import", () => {
  const files = [
    "vercel-ai-memory.mjs",
    "langchain-rag.mjs",
    "llamaindex-rag.mjs",
    "vercel-ai/index.ts",
    "langchain/index.ts",
    "llamaindex/index.ts",
  ];
  const base = resolve(HERE, "../../vantadb-ts/examples");
  for (const f of files) {
    const src = readFileSync(resolve(base, f), "utf8");
    assert.match(src, /import \{ Client \} from "vantadb"/, `${f}: missing canonical import`);
    assert.doesNotMatch(src, /import \{ VantaDB \}/, `${f}: stale VantaDB import`);
  }
});
