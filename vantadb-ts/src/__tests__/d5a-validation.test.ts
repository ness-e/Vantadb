import { describe, it, expect } from "vitest";
import { normalizeValue } from "../metadata.js";
import { buildSearchRequestBase } from "../guards.js";
import { DbError, ERROR_CODES } from "../errors.js";

import type { SearchRequest } from "../types.js";

/**
 * D5a — validación en frontera TS.
 *
 * `normalizeValue` aceptaba `unknown` sin validar la forma taggeada
 * (casteo `as Value` en la rama default) y `buildSearchRequestBase`
 * defaulteaba `top_k ?? 10` sin validar `namespace`/`query_vector`/`top_k`.
 * Patrón: `native.ts:94-104` (lanza `DbError` con `VALIDATION_ERROR`).
 * PROHIBIDO `any` en este archivo y en la implementación.
 */
describe("D5a: normalizeValue estricta", () => {
  it("mapea valores planos por tipo JS", () => {
    expect(normalizeValue("x")).toEqual({ String: "x" });
    expect(normalizeValue(true)).toEqual({ Bool: true });
    expect(normalizeValue(3)).toEqual({ Int: 3 });
    expect(normalizeValue(0.5)).toEqual({ Float: 0.5 });
    expect(normalizeValue(null)).toEqual({ Null: null });
  });

  it("acepta formas taggeadas válidas (backward compat)", () => {
    expect(normalizeValue({ String: "x" })).toEqual({ String: "x" });
    expect(normalizeValue({ Int: 1 })).toEqual({ Int: 1 });
    expect(normalizeValue({ Float: 2.5 })).toEqual({ Float: 2.5 });
    expect(normalizeValue({ Bool: false })).toEqual({ Bool: false });
    expect(normalizeValue({ Null: null })).toEqual({ Null: null });
    expect(normalizeValue({ ListString: ["a"] })).toEqual({
      ListString: ["a"],
    });
  });

  it("rechaza NaN / Infinity planos", () => {
    for (const bad of [NaN, Infinity, -Infinity]) {
      let caught: unknown;
      try {
        normalizeValue(bad);
      } catch (e) {
        caught = e;
      }
      expect(caught).toBeInstanceOf(DbError);
      expect((caught as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
    }
  });

  it("rechaza undefined / símbolos / funciones", () => {
    for (const bad of [undefined, Symbol("s"), () => 1]) {
      expect(() => normalizeValue(bad)).toThrow(DbError);
    }
  });

  it("rechaza tag desconocido", () => {
    let caught: unknown;
    try {
      normalizeValue({ Bogus: 1 });
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(DbError);
    expect((caught as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
  });

  it("rechaza payload con tipo incorrecto", () => {
    expect(() => normalizeValue({ String: 123 })).toThrow(DbError);
    expect(() => normalizeValue({ Int: "1" })).toThrow(DbError);
    expect(() => normalizeValue({ Bool: 1 })).toThrow(DbError);
    expect(() => normalizeValue({ Null: 1 })).toThrow(DbError);
    expect(() => normalizeValue({ Int: 1.5 })).toThrow(DbError);
    expect(() => normalizeValue({ Float: NaN })).toThrow(DbError);
    expect(() => normalizeValue({ ListString: [1] })).toThrow(DbError);
  });

  it("rechaza objeto vacío, multi-clave y arrays", () => {
    expect(() => normalizeValue({})).toThrow(DbError);
    expect(() => normalizeValue({ String: "a", Int: 1 })).toThrow(DbError);
    expect(() => normalizeValue(["x"])).toThrow(DbError);
  });
});

describe("D5a: buildSearchRequestBase valida", () => {
  const base: SearchRequest = { namespace: "ns", query_vector: [1, 0] };

  it("defaultea top_k=10, Cosine, explain=false", () => {
    expect(buildSearchRequestBase(base)).toEqual({
      namespace: "ns",
      query_vector: [1, 0],
      top_k: 10,
      distance_metric: "Cosine",
      explain: false,
    });
  });

  it("preserva valores explícitos", () => {
    expect(
      buildSearchRequestBase({
        ...base,
        top_k: 3,
        distance_metric: "Euclidean",
        explain: true,
      }),
    ).toMatchObject({ top_k: 3, distance_metric: "Euclidean", explain: true });
  });

  it("acepta top_k = 0", () => {
    expect(buildSearchRequestBase({ ...base, top_k: 0 }).top_k).toBe(0);
  });

  it("rechaza request nulo", () => {
    expect(() =>
      buildSearchRequestBase(null as unknown as SearchRequest),
    ).toThrow(DbError);
  });

  it("rechaza namespace vacío o no-string", () => {
    for (const ns of ["", 123, null]) {
      let caught: unknown;
      try {
        buildSearchRequestBase({
          ...base,
          namespace: ns as unknown as string,
        });
      } catch (e) {
        caught = e;
      }
      expect(caught).toBeInstanceOf(DbError);
      expect((caught as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
    }
  });

  it("rechaza query_vector vacío o con elementos no finitos", () => {
    for (const qv of [[], [NaN], [1, Infinity], ["1"]]) {
      expect(() =>
        buildSearchRequestBase({
          ...base,
          query_vector: qv as unknown as number[],
        }),
      ).toThrow(DbError);
    }
  });

  it("rechaza top_k negativo, fraccionario, NaN o no-numérico", () => {
    for (const top_k of [-1, 1.5, NaN, Infinity, "5"]) {
      let caught: unknown;
      try {
        buildSearchRequestBase({
          ...base,
          top_k: top_k as unknown as number,
        });
      } catch (e) {
        caught = e;
      }
      expect(caught).toBeInstanceOf(DbError);
      expect((caught as DbError).code).toBe(ERROR_CODES.VALIDATION_ERROR);
    }
  });

  it("rechaza distance_metric desconocida", () => {
    expect(() =>
      buildSearchRequestBase({
        ...base,
        distance_metric: "Manhattan" as unknown as "Cosine",
      }),
    ).toThrow(DbError);
  });

  it("rechaza explain no-booleano", () => {
    expect(() =>
      buildSearchRequestBase({
        ...base,
        explain: "yes" as unknown as boolean,
      }),
    ).toThrow(DbError);
  });
});
