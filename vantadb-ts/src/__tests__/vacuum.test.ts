import { describe, it, expect } from "vitest";
import { NativeVantaDB } from "../native.js";

/**
 * EGO-14: `NativeVantaDB.vacuum()` expone la purga HNSW del core a los
 * agentes de vida larga (recall + disco degradados por nodos fantasma).
 *
 * Contrato real (ver `src/storage/engine/maintenance.rs::vacuum` +
 * `delete.rs::apply_delete_inner`): `delete()` ya desengancha el nodo de
 * HNSW y deja el tombstone en el store — vacuum solo cosecha entradas
 * huérfanas (ventanas de crash), así que `removed_nodes` suele ser 0.
 * El contrato testeable es: reporte con la forma canónica, `success: true`,
 * e idempotencia (segundo vacuum también sucede).
 */

const NS = "vacuum-ego14";

const nativeAvailable = await import("vantadb-node").then(
  () => true,
  () => false,
);

describe("vacuum() (EGO-14)", () => {
  it.runIf(nativeAvailable)("reports the canonical shape and is idempotent", async () => {
    const db = await NativeVantaDB.connect(":memory:");
    try {
      await db.put({ namespace: NS, key: "a", payload: "alpha", vector: [0.1, 0.2, 0.3] });
      await db.put({ namespace: NS, key: "b", payload: "beta", vector: [0.2, 0.3, 0.4] });
      await db.delete({ namespace: NS, key: "a" });

      const first = await db.vacuum();
      expect(first.success).toBe(true);
      expect(first.scanned_nodes).toBeGreaterThanOrEqual(first.removed_nodes);
      expect(first.reclaimed_bytes).toBeGreaterThanOrEqual(0);
      expect(first.duration_ms).toBeGreaterThanOrEqual(0);

      const second = await db.vacuum();
      expect(second.success).toBe(true);
    } finally {
      await db.close();
    }
  });
});
