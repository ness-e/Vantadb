// DESKTOP-40 (slice 1): catálogo ES/EN mínimo + tt() con fallback.
// DESKTOP-40-slice2: shell chrome (titlebar/splash/help/ns/shell/app).
import { describe, expect, it } from "vitest";
import { dictionaries, tp, tt } from "./index";

describe("i18n slice 1", () => {
  it("resuelve claves ES de Settings", () => {
    expect(tt("es", "settings.connections", "FALLBACK")).toBe("Conexiones guardadas");
  });

  it("resuelve claves EN de Settings", () => {
    expect(tt("en", "settings.connections", "FALLBACK")).toBe("Saved connections");
  });

  it("fallback cuando la clave falta en el catálogo", () => {
    expect(tt("es", "missing.key", "Texto directo")).toBe("Texto directo");
    expect(tt("en", "missing.key", "Texto directo")).toBe("Texto directo");
  });

  it("catálogos ES/EN simétricos (mismas claves)", () => {
    expect(Object.keys(dictionaries.en).sort()).toEqual(Object.keys(dictionaries.es).sort());
  });
});

describe("i18n slice 2 (shell chrome)", () => {
  it("resuelve claves ES del shell", () => {
    expect(tt("es", "titlebar.minimize", "FALLBACK")).toBe("Minimizar");
    expect(tt("es", "splash.skipIntro", "FALLBACK")).toBe("Saltar la introducción");
    expect(tt("es", "help.title", "FALLBACK")).toBe("Guía rápida");
    expect(tt("es", "ns.create", "FALLBACK")).toBe("crear");
    expect(tt("es", "shell.sidebarTitle", "FALLBACK")).toBe("Panel lateral");
    expect(tt("es", "app.opFailed", "FALLBACK")).toBe("La operación falló.");
  });

  it("resuelve claves EN del shell", () => {
    expect(tt("en", "titlebar.minimize", "FALLBACK")).toBe("Minimize");
    expect(tt("en", "splash.skipIntro", "FALLBACK")).toBe("Skip the intro");
    expect(tt("en", "help.title", "FALLBACK")).toBe("Quick guide");
    expect(tt("en", "ns.create", "FALLBACK")).toBe("create");
    expect(tt("en", "shell.sidebarTitle", "FALLBACK")).toBe("Sidebar");
    expect(tt("en", "app.opFailed", "FALLBACK")).toBe("The operation failed.");
  });

  it("tp() interpola notices del shell", () => {
    expect(tp("es", "ns.renameTitle", 'Renombrar "{name}"', { name: "docs" })).toBe('Renombrar "docs"');
    expect(tp("en", "ns.renameTitle", 'Rename "{name}"', { name: "docs" })).toBe('Rename "docs"');
    expect(tp("es", "shell.nsRenamed", "FALLBACK", { from: "a", to: "b", n: "3" })).toContain('"a" renombrado a "b" (3 registros)');
  });

  it("claves shell agregadas post-discovery (viewer/import/menu)", () => {    expect(tt("es", "shell.loadingViewer", "FALLBACK")).toBe("cargando visor…");
    expect(tt("en", "shell.loadingViewer", "FALLBACK")).toBe("loading viewer…");
    expect(tt("en", "shell.menuSettings", "FALLBACK")).toBe("Go to Settings");
    expect(tt("es", "shell.menuThemeLight", "FALLBACK")).toBe("Tema claro");
    expect(tp("es", "shell.imported", "FALLBACK", { n: "5" })).toBe("Importados 5 registros.");
    expect(tp("en", "shell.imported", "FALLBACK", { n: "5" })).toBe("Imported 5 records.");
  });
});

describe("i18n slice 3 (lentes/paneles)", () => {
  it("resuelve claves ES de lentes", () => {
    expect(tt("es", "trash.aria", "FALLBACK")).toBe("Papelera");
    expect(tt("es", "space.project", "FALLBACK")).toBe("⤒ proyectar");
    expect(tt("es", "graph.labels", "FALLBACK")).toBe("labels");
    expect(tt("es", "retrieval.explain", "FALLBACK")).toBe("▸ EXPLICAR");
    expect(tt("es", "consolidate.detect", "FALLBACK")).toBe("⛃ Detectar candidatos");
    expect(tt("es", "indices.vecTitle", "FALLBACK")).toBe("Índice vectorial · HNSW");
    expect(tt("es", "inspector.saveBtn", "FALLBACK")).toBe("GUARDAR");
    expect(tt("es", "palette.navGroup", "FALLBACK")).toBe("Navegación");
    expect(tt("es", "home.title", "FALLBACK")).toBe("VISTA GENERAL");
    expect(tt("es", "data.title", "FALLBACK")).toBe("Memorias");
    expect(tt("es", "ingest.title", "FALLBACK")).toBe("Ingestar");
    expect(tt("es", "panels.conn.title", "FALLBACK")).toBe("Conexiones");
  });

  it("resuelve claves EN de lentes", () => {
    expect(tt("en", "trash.aria", "FALLBACK")).toBe("Trash");
    expect(tt("en", "space.project", "FALLBACK")).toBe("⤒ project");
    expect(tt("en", "retrieval.explain", "FALLBACK")).toBe("▸ EXPLAIN");
    expect(tt("en", "consolidate.detect", "FALLBACK")).toBe("⛃ Detect candidates");
    expect(tt("en", "indices.vecTitle", "FALLBACK")).toBe("Vector index · HNSW");
    expect(tt("en", "inspector.saveBtn", "FALLBACK")).toBe("SAVE");
    expect(tt("en", "home.title", "FALLBACK")).toBe("OVERVIEW");
    expect(tt("en", "data.title", "FALLBACK")).toBe("Memories");
    expect(tt("en", "panels.conn.title", "FALLBACK")).toBe("Connections");
  });

  it("tp() interpola notices de lentes", () => {
    expect(tp("es", "trash.restored", "FALLBACK", { id: "k1" })).toBe("restaurado k1");
    expect(tp("en", "trash.restored", "FALLBACK", { id: "k1" })).toBe("restored k1");
    expect(tp("es", "space.projDone", "FALLBACK", { n: "42" })).toBe("Proyección lista: 42 puntos (seed fijo)");
    expect(tp("es", "consolidate.foundPairs", "FALLBACK", { n: "3" })).toBe("Consolidación: 3 par(es) candidato(s) detectado(s)");
    expect(tp("es", "graph.console.outcomeRead", "FALLBACK", { n: "5", m: "4" })).toBe("✓ 5 registros (4 resaltados)");
    expect(tp("es", "export.copiedRecords", "FALLBACK", { n: "7" })).toBe("copiados 7 registros (JSONL)");
    expect(tt("es", "space.noVectors", "FALLBACK")).toBe("sin registros con vector para proyectar");
    expect(tt("en", "space.noVectors", "FALLBACK")).toBe("no records with vectors to project");
  });
});
