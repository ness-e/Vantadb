// DESKTOP-31: ConnectionPrefsStore — perfiles conexión, defaults búsqueda,
// idioma. Storage fake en memoria → round-trips deterministas (mismo patrón
// que persisted-stores.test.ts).
import { describe, expect, it } from "vitest";
import { ConnectionPrefsStore } from "./connections";

const KEY = "vanta.connections.v1";

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

describe("ConnectionPrefsStore", () => {
  it("upsertProfile persiste y un store nuevo sobre el MISMO storage hidrata", () => {
    const storage = fakeStorage();
    const s = new ConnectionPrefsStore(storage);

    const p = s.upsertProfile({ name: "local", kind: "native", path: "vantadb-local" });
    expect(p.id).toBeTruthy();

    // Storage crudo verificado + hidratación en instancia nueva (≈ reinicio).
    const raw = JSON.parse(storage.getItem(KEY) ?? "{}");
    expect(raw.profiles).toHaveLength(1);
    expect(raw.profiles[0]).toMatchObject({ name: "local", kind: "native", path: "vantadb-local" });
    expect(new ConnectionPrefsStore(storage).get().profiles?.[0].name).toBe("local");
  });

  it("upsert con mismo id reemplaza (no duplica); remove limpia activeProfileId", () => {
    const storage = fakeStorage();
    const s = new ConnectionPrefsStore(storage);

    const a = s.upsertProfile({ name: "srv", kind: "server", url: "http://127.0.0.1", port: 8080 });
    s.set({ activeProfileId: a.id });
    s.upsertProfile({ id: a.id, name: "srv2", kind: "server", url: "http://10.0.0.1", port: 9090, token: "t" });

    let prefs = new ConnectionPrefsStore(storage).get();
    expect(prefs.profiles).toHaveLength(1);
    expect(prefs.profiles?.[0]).toMatchObject({ name: "srv2", token: "t" });
    expect(prefs.activeProfileId).toBe(a.id);

    s.removeProfile(a.id);
    prefs = new ConnectionPrefsStore(storage).get();
    expect(prefs.profiles).toHaveLength(0);
    expect(prefs.activeProfileId).toBeNull();
  });

  it("defaults de búsqueda + idioma: round-trip y sanitización", () => {
    const storage = fakeStorage();
    const s = new ConnectionPrefsStore(storage);
    s.set({ topK: 12, mode: "vector", lang: "en" });
    expect(new ConnectionPrefsStore(storage).get()).toMatchObject({ topK: 12, mode: "vector", lang: "en" });

    // Campos con tipo incorrecto o corruptos se descartan sin crashear.
    const mixed = new ConnectionPrefsStore(
      fakeStorage({ [KEY]: '{"topK":-3,"mode":"bm25","lang":"fr","profiles":[{"id":"x"}]}' }),
    );
    expect(mixed.get().topK).toBeUndefined();
    expect(mixed.get().mode).toBeUndefined();
    expect(mixed.get().lang).toBeUndefined();
  });

  it("storage corrupto → arranca limpio sin crashear", () => {
    const broken = new ConnectionPrefsStore(fakeStorage({ [KEY]: "{not json" }));
    expect(broken.get()).toEqual({});
  });
});
