export function verifyClient(Client) {
  const db = Client.create();
  try {
    db.put({ namespace: "check", key: "k", payload: "smoke", vector: [0.1, 0.2] });
    const g = db.get({ namespace: "check", key: "k" });
    const hits = db.search({
      namespace: "check",
      query_vector: [0.1, 0.2],
      text_query: "smoke",
      top_k: 1,
    });
    if (g?.key !== "k") {
      throw new Error("get did not return the stored key");
    }
    if (!hits.some((h) => h.record.key === "k")) {
      throw new Error("mixed search did not return the stored key");
    }
    console.log("--check-imports verified: get=k hits>=1");
  } finally {
    db.close();
  }
}
