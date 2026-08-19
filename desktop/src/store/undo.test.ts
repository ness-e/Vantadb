// ESPACIO-02: softDeleteBatch (selección → borrar → undo restaura) — VS-08.
//
// El store es un singleton → cada test importa un módulo fresco
// (vi.resetModules + vi.mock hoisted). Los mocks del bridge `../vanta` se
// comparten vía vi.hoisted para que implementations seteadas en-test
// sobrevivan al re-import.
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { MemoryRecord } from "../vanta";

const mocks = vi.hoisted(() => ({
  remove: vi.fn<[string, string?], Promise<void>>(),
  vantaPut: vi.fn<[Record<string, unknown>], Promise<unknown>>(),
}));

vi.mock("../vanta", () => ({
  remove: mocks.remove,
  vantaPut: mocks.vantaPut,
}));

async function freshStore() {
  vi.resetModules();
  const mod = await import("./undo");
  return mod.undoStore;
}

function rec(id: string, namespace = "ns"): MemoryRecord {
  return {
    id,
    namespace,
    text: `text-${id}`,
    metadata: { k: id },
    expires_at_ms: 1234,
  };
}

beforeEach(() => {
  mocks.remove.mockReset();
  mocks.vantaPut.mockReset();
  mocks.remove.mockResolvedValue(undefined);
  mocks.vantaPut.mockResolvedValue({});
});

describe("undoStore.softDeleteBatch", () => {
  it("borra cada key, registra tombstones y UN undo restaura todo el lote", async () => {
    const store = await freshStore();
    const records = [rec("a"), rec("b"), rec("c")];

    await store.softDeleteBatch(records);

    // Backend: cada key removida con su namespace.
    expect(mocks.remove.mock.calls.map((c) => c.slice(0, 2))).toEqual([
      ["a", "ns"],
      ["b", "ns"],
      ["c", "ns"],
    ]);
    // Papelera: 3 tombstones con el snapshot completo.
    expect(store.getTrash().map((t) => t.record.id)).toEqual(["a", "b", "c"]);
    expect(store.canUndo()).toBe(true);

    // Un solo Ctrl+Z deshace el lote completo: re-put con payload/metadata/ttl.
    const label = await store.undo();
    expect(label).toBe("deshecho · restaurados 3");
    expect(mocks.vantaPut.mock.calls).toHaveLength(3);
    expect(mocks.vantaPut.mock.calls[0][0]).toEqual({
      namespace: "ns",
      key: "a",
      payload: "text-a",
      metadata: { k: "a" },
      expires_at_ms: 1234,
    });
    expect(store.getTrash()).toEqual([]);
    expect(store.canUndo()).toBe(false);
  });

  it("restaura el snapshot previo de la papelera al deshacer (tombstone pre-existente sobrevive)", async () => {
    const store = await freshStore();
    await store.softDelete(rec("keep"));
    const trashBefore = store.getTrash();

    await store.softDeleteBatch([rec("x"), rec("y")]);
    expect(store.getTrash()).toHaveLength(3);

    await store.undo();
    // El tombstone previo (keep) vuelve a estar solo — el lote se restauró.
    expect(store.getTrash()).toEqual(trashBefore);
  });

  it("si un remove del backend falla, NO registra entrada ni tombstones", async () => {
    const store = await freshStore();
    mocks.remove
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("backend down"));

    await expect(store.softDeleteBatch([rec("a"), rec("b")])).rejects.toThrow(
      "backend down",
    );
    expect(store.getTrash()).toEqual([]);
    expect(store.canUndo()).toBe(false);
  });
});
