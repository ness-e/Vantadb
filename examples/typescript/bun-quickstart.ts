import { Client } from "vantadb-wasm";

export async function runQuickstart() {
  const db = new Client({});
  try {
    const rec = db.put({
      namespace: "demo/quickstart",
      key: "fact_001",
      payload: "The user prefers concise technical answers.",
      metadata: { topic: "preferences", source: "demo" },
      vector: [0.1, 0.2, 0.3, 0.4],
    });
    console.log(`put: ${rec.namespace}/${rec.key}`);

    const got = db.get("demo/quickstart", "fact_001");
    console.log(`get: ${got!.payload}`);

    const hits = db.search({
      namespace: "demo/quickstart",
      query_vector: [0.15, 0.25, 0.35, 0.45],
      text_query: "concise answers",
      top_k: 5,
    });
    for (const h of hits) {
      console.log(`search: ${h.record.key} score=${h.score}`);
    }
    if (!got || got.key !== "fact_001" || hits.length < 1) {
      throw new Error("quickstart invariant failed");
    }
    return { put: rec.key, got: got.key, hits: hits.length };
  } finally {
    db.close();
  }
}

if (import.meta.main) {
  await runQuickstart();
}
