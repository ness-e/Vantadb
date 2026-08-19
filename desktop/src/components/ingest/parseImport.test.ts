// OP-01 — Tests del parser de import (parseImport/runImport).
// Contrato del plan: "pegar CSV de 10 filas → preview 10 → import → grid
// muestra 10; fila inválida se marca sin romper el resto". El parser es puro
// (sin imports del bridge) → corre en vitest sin Tauri.
import { describe, expect, it, vi } from "vitest";
import {
  EXAMPLE_CSV,
  MAX_IMPORT,
  parseImport,
  runImport,
  type ImportItem,
  type ParsedRow,
} from "./parseImport";

const NS = "pruebas";

function validItems(rows: ParsedRow[]): ImportItem[] {
  return rows.filter((r) => r.item !== null).map((r) => r.item as ImportItem);
}

function csv10(): string {
  const lines = ["key,payload,metadata_json"];
  for (let i = 1; i <= 10; i++) {
    lines.push(`r${i},registro numero ${i},"{""i"":${i}}"`);
  }
  return lines.join("\n");
}

describe("parseImport — CSV", () => {
  it("10 filas CSV → 10 válidas con key/text/metadata y namespace default", () => {
    const res = parseImport(csv10(), NS);
    expect(res.error).toBeUndefined();
    expect(res.valid).toBe(10);
    expect(res.invalid).toBe(0);
    expect(res.truncated).toBe(false);
    const items = validItems(res.rows);
    expect(items[0]).toEqual({ id: "r1", text: "registro numero 1", namespace: NS, metadata: { i: 1 } });
    expect(items[9].id).toBe("r10");
  });

  it("fila inválida (text vacío) se marca sin romper el resto", () => {
    const csv = ["key,payload,metadata_json", "ok-1,primera,{}", "rota,,{}", "ok-2,tercera,{}"].join("\n");
    const res = parseImport(csv, NS);
    expect(res.valid).toBe(2);
    expect(res.invalid).toBe(1);
    const broken = res.rows.find((r) => r.item === null)!;
    expect(broken.index).toBe(3);
    expect(broken.error).toContain("falta text/payload/content");
    // Las filas válidas siguen intactas
    expect(validItems(res.rows).map((i) => i.id)).toEqual(["ok-1", "ok-2"]);
  });

  it("metadata JSON inválido → error por fila (con índice)", () => {
    const csv = ["key,payload,metadata_json", "a,hola,{no-json", "b,mundo,{}"].join("\n");
    const res = parseImport(csv, NS);
    expect(res.valid).toBe(1);
    expect(res.invalid).toBe(1);
    expect(res.rows[0].error).toContain("metadata no es JSON válido");
    expect(res.rows[0].index).toBe(2);
  });

  it("delimitador tab (pegado de planilla) y columna namespace override", () => {
    const csv = ["key\tpayload\tnamespace", "a\ttexto a\totro-ns"].join("\n");
    const res = parseImport(csv, NS);
    expect(res.valid).toBe(1);
    expect(validItems(res.rows)[0].namespace).toBe("otro-ns");
  });

  it("CSV sin cabecera reconocible → error global legible", () => {
    const res = parseImport("k1,hola,{}", NS);
    expect(res.error).toContain("CSV sin columna de texto");
    expect(res.rows).toHaveLength(0);
  });
});

describe("parseImport — JSON", () => {
  it("array de objetos: key/text/metadata + keys no reconocidas → metadata", () => {
    const json = JSON.stringify([
      { key: "k1", text: "primera", metadata: { tipo: "nota" }, extra: 7 },
      { id: "k2", payload: "segunda" },
    ]);
    const res = parseImport(json, NS);
    expect(res.valid).toBe(2);
    expect(validItems(res.rows)[0]).toEqual({
      id: "k1",
      text: "primera",
      namespace: NS,
      metadata: { tipo: "nota", extra: 7 },
    });
    expect(validItems(res.rows)[1].id).toBe("k2");
  });

  it("objeto único → 1 válido", () => {
    const res = parseImport('{"text": "solo", "id": 42}', NS);
    expect(res.valid).toBe(1);
    expect(validItems(res.rows)[0]).toEqual({ id: "42", text: "solo", namespace: NS });
  });

  it("NDJSON (una línea = un objeto), líneas vacías salteadas", () => {
    const ndjson = ['{"text":"a","key":"x1"}', "", '{"text":"b","key":"x2"}'].join("\n");
    const res = parseImport(ndjson, NS);
    expect(res.valid).toBe(2);
    expect(validItems(res.rows).map((i) => i.id)).toEqual(["x1", "x2"]);
  });

  it("línea NDJSON inválida → error por línea sin romper el resto", () => {
    const ndjson = ['{"text":"a"}', "esto-no-es-json", '{"text":"b"}'].join("\n");
    const res = parseImport(ndjson, NS);
    expect(res.valid).toBe(2);
    expect(res.invalid).toBe(1);
    expect(res.rows[1].error).toContain("línea 2: JSON inválido");
  });

  it("objeto sin text/payload/content → error por fila", () => {
    const res = parseImport('[{"key":"k","meta":1}]', NS);
    expect(res.valid).toBe(0);
    expect(res.invalid).toBe(1);
    expect(res.rows[0].error).toContain("falta text/payload/content");
  });
});

describe("parseImport — límites y errores globales", () => {
  it("> 1000 registros → truncated y solo los primeros 1000", () => {
    const lines = ["key,payload"];
    for (let i = 1; i <= MAX_IMPORT + 5; i++) lines.push(`k${i},texto ${i}`);
    const res = parseImport(lines.join("\n"), NS);
    expect(res.truncated).toBe(true);
    expect(res.rows).toHaveLength(MAX_IMPORT);
    expect(res.valid).toBe(MAX_IMPORT);
  });

  it("texto vacío → error global", () => {
    const res = parseImport("   \n  ", NS);
    expect(res.error).toContain("Pegá CSV o JSON primero");
    expect(res.rows).toHaveLength(0);
  });

  it("JSON roto de una línea → error global legible", () => {
    const res = parseImport('{"text": "a"', NS);
    expect(res.error).toContain("JSON inválido");
    expect(res.rows).toHaveLength(0);
  });

  it("NDJSON todo inválido → error global, sin ruido por línea", () => {
    const res = parseImport(["{oops}", '{"tambien": rot}'].join("\n"), NS);
    expect(res.error).toContain("JSON inválido");
    expect(res.rows).toHaveLength(0);
  });
});

describe("runImport — chunking de ingestBatch", () => {
  function rows(n: number): ParsedRow[] {
    return Array.from({ length: n }, (_, i) => ({
      index: i + 1,
      item: { id: `k${i + 1}`, text: `t${i + 1}`, namespace: NS },
    }));
  }

  it("3 chunks de 50 → imported 150, sin errores", async () => {
    const ingestFn = vi.fn(async (chunk: ImportItem[]) => chunk.map((c) => c.id ?? ""));
    const report = await runImport(rows(150), ingestFn);
    expect(report.imported).toBe(150);
    expect(report.errors).toHaveLength(0);
    expect(ingestFn).toHaveBeenCalledTimes(3);
  });

  it("chunk 2 falla → error con rango de filas originales", async () => {
    const ingestFn = vi.fn(async (chunk: ImportItem[]) => {
      if (chunk[0].id === "k51") throw new Error("put_batch falló");
      return chunk.map((c) => c.id ?? "");
    });
    const report = await runImport(rows(120), ingestFn);
    expect(report.imported).toBe(70); // chunks 1 (50) y 3 (20) ok
    expect(report.errors).toHaveLength(1);
    expect(report.errors[0].rows).toBe("51-100");
    expect(report.errors[0].message).toBe("put_batch falló");
  });

  it("error message no-Error se serializa a String", async () => {
    const ingestFn = vi.fn(async () => {
      throw { message: "error plano" };
    });
    const report = await runImport(rows(1), ingestFn);
    expect(report.errors[0].message).toBe("[object Object]");
  });
});

describe("EXAMPLE_CSV", () => {
  it("el ejemplo del botón parsea a 3 válidos", () => {
    const res = parseImport(EXAMPLE_CSV, NS);
    expect(res.valid).toBe(3);
    expect(res.invalid).toBe(0);
  });
});