// DESKTOP-40 (slice 1): catálogo ES/EN mínimo + tt() con fallback.
import { describe, expect, it } from "vitest";
import { dictionaries, tt } from "./index";

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
