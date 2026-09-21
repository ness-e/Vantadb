// DESKTOP-26 Step 4: round-trip del bridge vanta.ts ↔ Tauri con invoke mockeado.
// Valida que cada wrapper emita el nombre de comando Tauri correcto (los nombres
// están cableados en src-tauri/src/lib.rs — renombrarlos rompe el backend) y
// que los args llegan con la forma snake_case que serde espera.
import { beforeEach, describe, expect, it, vi } from "vitest";
import { get, listPage, remove, search, type MemoryRecord } from "./vanta";
import { TauriBackend, transport } from "./transport";

const mocks = vi.hoisted(() => {
  // getTransport() corre al cargar transport.ts: sin esta marca el entorno
  // jsdom resuelve HttpBackend en vez de TauriBackend.
  const marker = {};
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = marker;
  const w = (globalThis as { window?: Record<string, unknown> }).window;
  if (w && w !== globalThis) w.__TAURI_INTERNALS__ = marker;
  return { invoke: vi.fn() };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

function record(id: string): MemoryRecord {
  return { id, namespace: "ns", text: `text-${id}`, version: 1 };
}

beforeEach(() => {
  mocks.invoke.mockReset();
});

describe("bridge ↔ Tauri (invoke mock)", () => {
  it("put → get → delete: comandos y args exactos en el wire", async () => {
    mocks.invoke
      .mockResolvedValueOnce(record("k1")) // vanta_put devuelve el stored record
      .mockResolvedValueOnce(record("k1")) // vanta_get
      .mockResolvedValueOnce(undefined); // vanta_delete

    const put = await import("./vanta").then((m) =>
      m.vantaPut({ namespace: "ns", key: "k1", payload: "hello", metadata: { a: 1 } }),
    );
    expect(put.id).toBe("k1");
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_put", {
      namespace: "ns",
      key: "k1",
      payload: "hello",
      metadata: { a: 1 },
      expires_at_ms: undefined,
    });

    await get("k1", "ns");
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_get", {
      key: "k1",
      namespace: "ns",
    });

    await remove("k1", "ns");
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_delete", {
      key: "k1",
      namespace: "ns",
    });
  });

  it("search envuelve la query; listPage pasa cursor/limit", async () => {
    mocks.invoke
      .mockResolvedValueOnce([record("hit")])
      .mockResolvedValueOnce({ records: [record("r")], next_cursor: null });

    const hits = await search({ query: "foo", top_k: 5, explain: true });
    expect(hits).toHaveLength(1);
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_search", {
      query: { query: "foo", top_k: 5, explain: true },
    });

    const page = await listPage({ limit: 10, cursor: 20 });
    expect(page.records).toHaveLength(1);
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_list", {
      namespace: undefined,
      limit: 10,
      cursor: 20,
    });
  });

  it("TauriBackend delega directo en invoke; errores propagan sin envolver", async () => {
    const backend = new TauriBackend();
    // El singleton del módulo también es Tauri gracias a la marca hoisted.
    expect(transport).toBeInstanceOf(TauriBackend);
    mocks.invoke.mockRejectedValueOnce({ Native: "namespace not found" });
    await expect(backend.call("vanta_get", { key: "x" })).rejects.toEqual({
      Native: "namespace not found",
    });
    expect(mocks.invoke).toHaveBeenCalledWith("vanta_get", { key: "x" });
  });
});

// DESKTOP-36: wrappers read-only de vanta-memory. Valida nombre de comando
// (cableado en src-tauri/src/lib.rs) y forma exacta de los args en el wire:
// comandos con `rename_all = "snake_case"` reciben claves snake_case; el resto
// usa el camelCase default del IPC de Tauri v2. El lado Rust se prueba contra
// el seed REAL de vanta-seed (`import_seed_str`) en
// desktop/src-tauri/src/commands/memory.rs — acá solo contrato de serialización.
describe("memory observability bridge (DESKTOP-36)", () => {
  beforeEach(async () => {
    mocks.invoke.mockReset();
    await import("./vanta");
  });

  it("scene_list/persona_get/skill_list usan camelCase default del IPC", async () => {
    mocks.invoke
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce([]);

    const m = await import("./vanta");
    await m.memorySceneList("sess-1");
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_scenes_list", {
      sessionKey: "sess-1",
    });

    await m.memoryPersonaGet("sess-1");
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_persona_get", {
      sessionKey: "sess-1",
    });
    expect(await m.memoryPersonaGet("none")).toBeNull();

    await m.memorySkillList();
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_skills_list", undefined);
  });

  it("scene_read/query + genlog_query emiten claves snake_case", async () => {
    mocks.invoke
      .mockResolvedValueOnce({ scene_name: "s1", meta: {}, content: "", deleted: false })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([]);

    const m = await import("./vanta");
    await m.memorySceneRead("sess-1", "deploy-runbook");
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_scene_read", {
      session_key: "sess-1",
      scene_name: "deploy-runbook",
    });

    await m.memorySceneQuery("sess-1", "deploy runner", 3);
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_scene_query", {
      session_key: "sess-1",
      keyword: "deploy runner",
      top_k: 3,
    });

    await m.memoryGenlogQuery("sess-1", "l2", 10);
    expect(mocks.invoke).toHaveBeenLastCalledWith("vanta_genlog_query", {
      session_key: "sess-1",
      layer: "l2",
      limit: 10,
    });
  });
});
