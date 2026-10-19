// Batch selection ops (OP-02) — selección múltiple del grid por key
// `${namespace}:${id}` (mismo keying que getRowId de TanStack). Módulo puro
// para testear toggle/select-all sin React Testing Library.
import { describe, expect, it } from "vitest";
import type { MemoryRecord } from "../vanta";
import { rowKey, selectAll, toggleId } from "./batchSelection";

function rec(id: string, namespace = "ns"): MemoryRecord {
  return { id, namespace, text: `text-${id}` };
}

describe("batchSelection", () => {
  it("rowKey es `${namespace}:${id}` — mismo keying que getRowId del grid", () => {
    expect(rowKey(rec("a"))).toBe("ns:a");
    expect(rowKey(rec("b", "other"))).toBe("other:b");
  });

  it("toggleId agrega y remueve sin mutar el Set original", () => {
    const sel = new Set(["ns:a"]);
    const added = toggleId(sel, "ns:b");
    expect(added.has("ns:b")).toBe(true);
    expect(sel.has("ns:b")).toBe(false); // original intacto (immutable)

    const removed = toggleId(added, "ns:b");
    expect(removed.has("ns:b")).toBe(false);
    expect(removed.has("ns:a")).toBe(true);
  });

  it("selectAll selecciona todo cuando falta alguno; limpia cuando ya están todos", () => {
    const ids = ["ns:a", "ns:b", "ns:c"];
    const all = selectAll(new Set(["ns:a"]), ids);
    expect([...all].sort()).toEqual([...ids].sort());

    const cleared = selectAll(all, ids);
    expect(cleared.size).toBe(0);
  });

  it("selectAll con selección vacía selecciona todos; con lista vacía limpia", () => {
    expect(selectAll(new Set(), ["ns:a"]).has("ns:a")).toBe(true);
    expect(selectAll(new Set(["ns:a"]), []).size).toBe(0);
  });
});