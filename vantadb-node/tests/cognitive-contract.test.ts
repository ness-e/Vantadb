import { describe, it, expect } from "vitest";
import { VantaDb, AbortHandle } from "../index.js";
import * as os from "node:os";
import * as path from "node:path";

// DIST-19 S5 (MEMG-24) + S4 (DIST-20): contract suite for the in-process
// `VantaCognitiveAPI` facade. Pins the wire shapes the desktop Ego host
// (`EgoMemoryAdapter`) consumes: stable JSON keys, null-vs-block recall,
// idle-gated consolidation, reinforce round-trip, checkpoint round-trip,
// and cooperative abort (`AbortHandle`, incl. the one-line `AbortSignal`
// wiring). A green suite blocks releases (contract gate).

function tmp(name: string): string {
  return path.join(os.tmpdir(), `vanta-cog-${name}-${process.pid}`);
}

const L1_SESSION = "cog-suite";
const L1_NS = `l1/${L1_SESSION}`;
const L1_WORDS = ["brass", "telescope", "calibration", "ledger"];

async function seedL1(db: InstanceType<typeof VantaDb>): Promise<void> {
  // L1 records are JSON-serialized `MemoryRecord`s (the L1 reader skips
  // payloads that don't deserialize — plain strings are never recalled).
  // Minimal required fields per `vanta-memory/.../abstractions/types.rs`.
  const record = {
    id: "obs-1",
    content: `The ${L1_WORDS.join(" ")} notes from yesterday`,
    type: "episodic",
    priority: 50,
    scene_name: "cog-suite",
    source_message_ids: [],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    session_key: L1_SESSION,
  };
  await db.put({ namespace: L1_NS, key: "obs-1", payload: JSON.stringify(record) });
}

describe("vantadb-node cognitive contract (DIST-19 S5)", () => {
  it("exposes the facade surface with stable signatures", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      expect(typeof AbortHandle).toBe("function");
      const handle = new AbortHandle();
      expect(typeof handle.abort).toBe("function");
      for (const method of [
        "memoryRecall",
        "contextAssemble",
        "memoryConsolidate",
        "memoryReinforce",
        "checkpointSave",
        "checkpointLoad",
      ]) {
        expect(typeof (db as unknown as Record<string, unknown>)[method]).toBe("function");
      }
      // `search` keeps working with one arg (backward compat, S4 additive).
      expect(db.search.length).toBeLessThanOrEqual(2);
    } finally {
      await db.close();
    }
  });

  it("memoryRecall returns null on an empty session, JSON after seeding", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      const empty = await db.memoryRecall({
        user_text: "anything",
        session_key: "no-such-session",
      });
      expect(empty).toBeNull();

      await seedL1(db);
      const recalled = await db.memoryRecall({
        user_text: `questions about ${L1_WORDS.slice(0, 3).join(" ")}`,
        session_key: L1_SESSION,
      });
      expect(recalled).not.toBeNull();
      // Stable wire keys (EgoMemoryAdapter surface).
      expect(Object.keys(recalled).sort()).toEqual([
        "append_system_context",
        "effective_mode",
        "governance",
        "persona",
        "prepend_context",
        "recalled_memories",
      ]);
      expect(recalled.recalled_memories.length).toBeGreaterThan(0);
      expect(typeof recalled.governance).toBe("object");
      expect(typeof recalled.effective_mode).toBe("string");
    } finally {
      await db.close();
    }
  });

  it("memoryRecall validates scope and session_key", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await expect(
        db.memoryRecall({ user_text: "x", session_key: L1_SESSION, scope: "bogus" }),
      ).rejects.toThrow(/scope/);
      // Review-S3 Optional: empty session_key is rejected, never silent.
      await expect(db.memoryRecall({ user_text: "x", session_key: "  " })).rejects.toThrow(
        /session_key/,
      );
    } finally {
      await db.close();
    }
  });

  it("contextAssemble compacts under a tight budget and validates inputs", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      const messages = Array.from({ length: 20 }, (_, i) => ({
        role: i % 2 === 0 ? "user" : "assistant",
        content: `message number ${i} with filler words to consume tokens `.repeat(10),
      }));
      const out = await db.contextAssemble({ messages, token_budget: 200 });
      expect(out.messages.length).toBeLessThan(messages.length);
      expect(Object.keys(out.report).sort()).toEqual([
        "mode",
        "msgs_before",
        "msgs_conserved",
        "tokens_after",
        "tokens_before",
      ]);
      expect(out.report.msgs_before).toBe(messages.length);
      expect(typeof out.has_boundary).toBe("boolean");
      await expect(db.contextAssemble({ messages, token_budget: 0 })).rejects.toThrow(
        /token_budget/,
      );
    } finally {
      await db.close();
    }
  });

  it("memoryConsolidate is idle-gated and reports the run", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      const now = Date.now();
      // Not idle: fail-closed, never a silent no-op.
      await expect(
        db.memoryConsolidate({ session_id: L1_SESSION, now_ms: now, last_active_at_ms: now }),
      ).rejects.toThrow(/not idle/);
      // Review-S3 Optional: empty session_id is rejected.
      await expect(
        db.memoryConsolidate({ session_id: " ", now_ms: now, last_active_at_ms: 0 }),
      ).rejects.toThrow(/session_id/);

      await seedL1(db);
      const run = await db.memoryConsolidate({
        session_id: L1_SESSION,
        now_ms: now,
        last_active_at_ms: now - 700_000,
      });
      expect(run.session_id).toBe(L1_SESSION);
      expect(typeof run.run_id).toBe("string");
      expect(run.inputs_scanned).toBeGreaterThanOrEqual(1);
    } finally {
      await db.close();
    }
  });

  it("memoryReinforce round-trips a recalled hit and validates inputs", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await seedL1(db);
      const recalled = await db.memoryRecall({
        user_text: `questions about ${L1_WORDS.slice(0, 3).join(" ")}`,
        session_key: L1_SESSION,
      });
      const hit = recalled.recalled_memories[0];
      const record = await db.memoryReinforce(hit, "used");
      expect(record.namespace).toBe(L1_NS);

      await expect(db.memoryReinforce(hit, "bogus")).rejects.toThrow(/outcome/);
      await expect(db.memoryReinforce({ nope: true }, "used")).rejects.toThrow(/hit/);
    } finally {
      await db.close();
    }
  });

  it("checkpoint save/load round-trips records", async () => {
    const dir = tmp("ckpt");
    const file = path.join(os.tmpdir(), `vanta-ckpt-${process.pid}.jsonl`);
    const db = await VantaDb.connect(dir);
    try {
      await db.put({ namespace: "ckpt", key: "a", payload: "checkpoint me" });
      const report = await db.checkpointSave(file);
      expect(report.records_exported).toBeGreaterThanOrEqual(1);
      expect(typeof report.sha256).toBe("string");
      await expect(db.checkpointSave("")).rejects.toThrow(/path/);
    } finally {
      await db.close();
    }
    const db2 = await VantaDb.connect(tmp("ckpt-restore"));
    try {
      const loaded = await db2.checkpointLoad(file, false);
      expect(loaded.inserted + loaded.updated).toBeGreaterThanOrEqual(1);
      const rec = await db2.get("ckpt", "a");
      expect(rec?.payload).toBe("checkpoint me");
    } finally {
      await db2.close();
    }
  });

  it("search with an idle AbortHandle behaves like no handle (S4 parity)", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await db.put({ namespace: "vec", key: "a", payload: "a", vector: [1, 0] });
      const handle = new AbortHandle();
      const hits = await db.search({ namespace: "vec", query_vector: [1, 0] }, handle);
      expect(hits.length).toBeGreaterThan(0);
    } finally {
      await db.close();
    }
  });

  it("aborting mid-search rejects with Cancelled and the db stays usable (S4/Regla 8)", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      // Slow op: 5000 records x 128d so the abort (same tick) wins reliably.
      const dim = 128;
      const batch = Array.from({ length: 5000 }, (_, i) => ({
        namespace: "bulk",
        key: `k${i}`,
        payload: `bulk record ${i} brass telescope filler words`,
        vector: Array.from({ length: dim }, (_, d) => ((i * 31 + d * 7) % 100) / 100),
      }));
      for (let off = 0; off < batch.length; off += 1000) {
        await db.putBatch(batch.slice(off, off + 1000));
      }

      // One-line AbortSignal wiring (the Ego pattern).
      const controller = new AbortController();
      const handle = new AbortHandle();
      controller.signal.addEventListener("abort", () => handle.abort());
      const pending = db.search(
        { namespace: "bulk", query_vector: new Array(dim).fill(0.5), top_k: 100 },
        handle,
      );
      controller.abort();
      await expect(pending).rejects.toThrow(/abort/i);

      // Evidence for Regla 8: engine untouched by the abort — writes/reads
      // work, and close() drains promptly (a hang would fail the suite).
      await db.put({
        namespace: "bulk",
        key: "after",
        payload: "ok",
        vector: new Array(dim).fill(0.1),
      });
      const rec = await db.get("bulk", "after");
      expect(rec?.payload).toBe("ok");
    } finally {
      await db.close();
    }
  }, 60_000);

  it("trust_aware recall is opt-in and byte-identical by default (S8/FIND-320)", async () => {
    const db = await VantaDb.connect(":memory:");
    try {
      await seedL1(db);
      const base = {
        user_text: `questions about ${L1_WORDS.slice(0, 3).join(" ")}`,
        session_key: L1_SESSION,
      };
      const a = await db.memoryRecall(base);
      const b = await db.memoryRecall({ ...base, trust_aware: false });
      expect(JSON.stringify(a)).toBe(JSON.stringify(b));
      const c = await db.memoryRecall({ ...base, trust_aware: true, now_ms: Date.now() });
      expect(c).not.toBeNull();
      expect(Object.keys(c).sort()).toEqual(Object.keys(a).sort());
    } finally {
      await db.close();
    }
  });
});
