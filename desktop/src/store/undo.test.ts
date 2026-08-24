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
  ingestBatch: vi.fn<[Record<string, unknown>[]], Promise<string[]>>(),
  listAll: vi.fn<[string], Promise<MemoryRecord[]>>(),
}));

vi.mock("../vanta", () => ({
  remove: mocks.remove,
  vantaPut: mocks.vantaPut,
  ingestBatch: mocks.ingestBatch,
  listAll: mocks.listAll,
}));

async function freshStore(storage?: Storage | null) {
  vi.resetModules();
  const mod = await import("./undo");
  return storage ? new mod.UndoStore(storage) : mod.undoStore;
}

/** Storage fake en memoria (mismo helper que persisted-stores.test.ts). */
function fakeStorage(initial: Record<string, string> = {}): Storage {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (k) => map.get(k) ?? null,
    setItem: (k, v) => void map.set(k, v),
    removeItem: (k) => void map.delete(k),
    clear: () => map.clear(),
    key: () => null,
    length: 0,
  };
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
  localStorage.clear(); // jsdom comparte storage entre freshStore() (DESKTOP-30)
  mocks.remove.mockReset();
  mocks.vantaPut.mockReset();
  mocks.ingestBatch.mockReset();
  mocks.listAll.mockReset();
  mocks.remove.mockResolvedValue(undefined);
  mocks.vantaPut.mockResolvedValue({});
  mocks.ingestBatch.mockResolvedValue([]);
  mocks.listAll.mockResolvedValue([]);
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

describe("undoStore.renameNamespace / deleteNamespace (DESKTOP-32)", () => {
  it("rename copia al nuevo ns (embedding incluido), borra el viejo y un undo revierte", async () => {
    const store = await freshStore();
    const r1 = { ...rec("a", "viejo"), vector: [1, 2] };
    const r2 = rec("b", "viejo");
    mocks.listAll.mockResolvedValue([r1, r2]);
    mocks.ingestBatch.mockResolvedValue(["a", "b"]);

    const n = await store.renameNamespace("viejo", "nuevo");
    expect(n).toBe(2);
    // Copia al destino preservando payload/metadata/vector.
    expect(mocks.ingestBatch.mock.calls[0][0]).toEqual([
      { id: "a", namespace: "nuevo", text: "text-a", embedding: [1, 2], metadata: { k: "a" } },
      { id: "b", namespace: "nuevo", text: "text-b", embedding: undefined, metadata: { k: "b" } },
    ]);
    // Originales borrados del ns viejo.
    expect(mocks.remove.mock.calls.map((c) => c.slice(0, 2))).toEqual([
      ["a", "viejo"],
      ["b", "viejo"],
    ]);

    // Un Ctrl+Z: re-copia al origen y borra las copias del destino.
    mocks.remove.mockClear();
    const label = await store.undo();
    expect(label).toBe('deshecho · "nuevo" vuelve a llamarse "viejo"');
    expect(mocks.vantaPut.mock.calls[0][0]).toMatchObject({ namespace: "viejo", key: "a" });
    expect(mocks.vantaPut.mock.calls[1][0]).toMatchObject({ namespace: "viejo", key: "b" });
    expect(mocks.remove.mock.calls.map((c) => c.slice(0, 2))).toEqual([
      ["a", "nuevo"],
      ["b", "nuevo"],
    ]);
  });

  it("namespace vacío → no-op sin entrada de undo", async () => {
    const store = await freshStore();
    mocks.listAll.mockResolvedValue([]);
    expect(await store.renameNamespace("x", "y")).toBe(0);
    expect(await store.deleteNamespace("x")).toBe(0);
    expect(store.canUndo()).toBe(false);
  });

  it("deleteNamespace mueve todo a papelera y UN undo restaura el lote", async () => {
    const store = await freshStore();
    mocks.listAll.mockResolvedValue([rec("a", "ns"), rec("b", "ns")]);

    const n = await store.deleteNamespace("ns");
    expect(n).toBe(2);
    expect(store.getTrash().map((t) => t.record.id)).toEqual(["a", "b"]);
    expect(store.canUndo()).toBe(true);

    const label = await store.undo();
    expect(label).toBe("deshecho · restaurados 2");
    expect(store.getTrash()).toEqual([]);
  });

  it("si la copia del rename falla a mitad, no queda entry de undo registrado", async () => {
    const store = await freshStore();
    mocks.listAll.mockResolvedValue([rec("a", "ns")]);
    mocks.ingestBatch.mockRejectedValue(new Error("backend down"));

    await expect(store.renameNamespace("ns", "otro")).rejects.toThrow("backend down");
    expect(store.canUndo()).toBe(false);
    // Los originales siguen intactos (el borrado corre después de la copia).
    expect(mocks.remove).not.toHaveBeenCalled();
  });
});

describe("undoStore persistence (DESKTOP-30)", () => {
  const KEY = "vanta.trash.v1";

  it("softDelete persiste la papelera y un store nuevo sobre el MISMO storage hidrata (≈ reinicio)", async () => {
    const storage = fakeStorage();
    const store = await freshStore(storage);

    await store.softDelete(rec("gone"));
    expect(JSON.parse(storage.getItem(KEY) ?? "[]")[0]?.record?.id).toBe("gone");

    // Instancia nueva = app reiniciada: el tombstone sobrevive.
    const rehydrated = await freshStore(storage);
    expect(rehydrated.getTrash().map((t) => t.record.id)).toEqual(["gone"]);
  });

  it("restore saca el tombstone del storage", async () => {
    const storage = fakeStorage();
    const store = await freshStore(storage);

    await store.softDelete(rec("x"));
    await store.restore(store.getTrash()[0]!);
    expect((await freshStore(storage)).getTrash()).toEqual([]);
  });

  it("tombstone con shape inválido se descarta; storage corrupto → papelera vacía sin crashear", async () => {
    const mixed = await freshStore(
      fakeStorage({
        [KEY]: JSON.stringify([
          { record: { id: "ok", namespace: "ns" }, deletedAtMs: 1 },
          { record: { id: 42 }, deletedAtMs: 2 },
          { garbage: true },
        ]),
      }),
    );
    expect(mixed.getTrash().map((t) => t.record.id)).toEqual(["ok"]);

    const broken = await freshStore(fakeStorage({ [KEY]: "{not json" }));
    expect(broken.getTrash()).toEqual([]);
  });
});
