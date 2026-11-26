import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { Client } from "vantadb";

export async function runQuickstart() {
  const db = Client.create();
  try {
    const rec = db.put({
      namespace: "demo/quickstart",
      key: "fact_001",
      payload: "The user prefers concise technical answers.",
      metadata: { topic: "preferences", source: "demo" },
      vector: [0.1, 0.2, 0.3, 0.4],
    });
    console.log(`put: ${rec.namespace}/${rec.key}`);

    const got = db.get({ namespace: "demo/quickstart", key: "fact_001" });
    console.log(`get: ${got.payload}`);

    const hits = db.search({
      namespace: "demo/quickstart",
      query_vector: [0.15, 0.25, 0.35, 0.45],
      text_query: "concise answers",
      top_k: 5,
    });
    for (const h of hits) {
      console.log(`search: ${h.record.key} score=${h.score}`);
    }
    return { put: rec.key, got: got.key, hits: hits.length };
  } finally {
    db.close();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  await runQuickstart();
}
