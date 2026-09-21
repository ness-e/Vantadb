// DESKTOP-26 Step 2: stores persistidos (VS-17). FavoritesStore y
// SearchHistory hidratan desde storage inyectable y persisten cada mutación.
// Storage fake en memoria → round-trips deterministas sin tocar localStorage.
// DESKTOP-23: WorkspacePrefsStore (surface + filtros) mismo patrón.
import { describe, expect, it } from "vitest";
import { EMPTY_QUERY } from "../components/search/filters-core";
import { WorkspacePrefsStore } from "./preferences";
import { FavoritesStore } from "./favorites";
import { SearchHistory } from "./search-history";

const KEY = { fav: "vanta.favorites.v1", hist: "vanta.search-history.v1" };

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

describe("WorkspacePrefsStore", () => {
  const QUERY = { ...EMPTY_QUERY, combinator: "or" as const };

  it("round-trip: set persiste y un store nuevo sobre el MISMO storage hidrata", () => {
    const storage = fakeStorage();
    const p = new WorkspacePrefsStore(storage);

    p.set({ surface: "retrieval", showFilters: true, ruleGroup: QUERY });

    // Storage crudo verificado + hidratación en instancia nueva (≈ reinicio).
    expect(JSON.parse(storage.getItem("vanta.workspace.v1") ?? "{}")).toMatchObject({
      surface: "retrieval",
      showFilters: true,
    });
    const rehydrated = new WorkspacePrefsStore(storage).get();
    expect(rehydrated.surface).toBe("retrieval");
    expect(rehydrated.showFilters).toBe(true);
    expect(rehydrated.ruleGroup?.combinator).toBe("or");
  });

  it("merge parcial: set de un campo no borra los demás", () => {
    const storage = fakeStorage();
    const p = new WorkspacePrefsStore(storage);
    p.set({ surface: "memorias", showFilters: true });
    p.set({ surface: "iql" });
    expect(new WorkspacePrefsStore(storage).get()).toMatchObject({
      surface: "iql",
      showFilters: true,
    });
  });

  it("storage corrupto o campos con tipo incorrecto → defaults limpios", () => {
    expect(new WorkspacePrefsStore(fakeStorage({ "vanta.workspace.v1": "{not json" })).get()).toEqual({});
    // ruleGroup sin `rules` array se descarta; los campos válidos se conservan.
    const mixed = new WorkspacePrefsStore(
      fakeStorage({ "vanta.workspace.v1": '{"surface":"espacio","ruleGroup":{"x":1}}' }),
    );
    expect(mixed.get()).toEqual({ surface: "espacio" });
  });
});

describe("FavoritesStore", () => {
  it("hidrata del storage, toggle persiste y un store nuevo lo ve", () => {
    // Hidratación: favorito precargado es visible.
    const storage = fakeStorage({ [KEY.fav]: '[{"namespace":"ns","key":null}]' });
    const s = new FavoritesStore(storage);
    expect(s.isFavorite("ns", null)).toBe(true);

    // Toggle de key: agrega → persiste → segundo toggle quita.
    expect(s.toggle("ns", "k1")).toBe(true);
    expect(s.toggle("ns", "k1")).toBe(false);

    // Round-trip: instancia nueva sobre el MISMO storage ve el estado final
    // (solo quedó el favorito de namespace; k1 se quitó).
    const rehydrated = new FavoritesStore(storage);
    expect(rehydrated.isFavorite("ns", null)).toBe(true);
    expect(rehydrated.isFavorite("ns", "k1")).toBe(false);
  });

  it("storage corrupto → arranca limpio sin crashear", () => {
    const broken = new FavoritesStore(fakeStorage({ [KEY.fav]: "{not json" }));
    expect(broken.getFavorites()).toEqual([]);
  });
});

describe("SearchHistory", () => {
  it("round-trip completo: add → persiste → nuevo store hidrata; dedup mueve al frente", () => {
    const storage = fakeStorage();
    const h = new SearchHistory(storage);

    h.add("  q1  "); // trim
    h.add("q2");
    h.add("q1"); // dedup no-consecutivo → q1 vuelve al frente
    expect(h.get()).toEqual(["q1", "q2"]);

    // Persistencia verificada leyendo el storage crudo.
    expect(JSON.parse(storage.getItem(KEY.hist) ?? "[]")).toEqual(["q1", "q2"]);

    // Hidratación en instancia nueva con el MISMO storage.
    expect(new SearchHistory(storage).get()).toEqual(["q1", "q2"]);

    h.clear();
    expect(new SearchHistory(storage).get()).toEqual([]);
  });

  it("query vacía no se registra; cap de 10 entradas", () => {
    const h = new SearchHistory(fakeStorage());
    h.add("   ");
    expect(h.get()).toEqual([]);
    for (let i = 0; i < 12; i++) h.add(`q${i}`);
    expect(h.get()).toHaveLength(10);
    expect(h.get()[0]).toBe("q11"); // newest-first
  });
});
