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

  it("claves shell agregadas post-discovery (viewer/import/menu)", () => {
    expect(tt("es", "shell.loadingViewer", "FALLBACK")).toBe("cargando visor…");
    expect(tt("en", "shell.loadingViewer", "FALLBACK")).toBe("loading viewer…");
    expect(tt("en", "shell.menuSettings", "FALLBACK")).toBe("Go to Settings");
    expect(tt("es", "shell.menuThemeLight", "FALLBACK")).toBe("Tema claro");
    expect(tp("es", "shell.imported", "FALLBACK", { n: "5" })).toBe("Importados 5 registros.");
    expect(tp("en", "shell.imported", "FALLBACK", { n: "5" })).toBe("Imported 5 records.");
  });
});
