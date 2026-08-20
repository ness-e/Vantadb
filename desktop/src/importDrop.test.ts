// WASM-04 — Tests del import por ARCHIVO: parseVdbDump (.vdbdump: export
// VantaDB + snapshots Qdrant-style) + parseImportFile (dispatch por extensión)
// + runImport con rows de archivo. node:test puro (misma convención que
// vanta-wasm-map.test.ts): un archivo dropeado se lee con File API y su texto
// ES el mismo input del paste → el contrato "mismos rows que paste" se verifica
// comparando parseImportFile contra parseImport del mismo contenido.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  MAX_IMPORT,
  parseImport,
  parseImportFile,
  parseVdbDump,
  runImport,
  type ImportItem,
  type ParsedRow,
} from "./components/ingest/parseImport.ts";

const NS = "pruebas";

function validItems(rows: ParsedRow[]): ImportItem[] {
  return rows.filter((r) => r.item !== null).map((r) => r.item as ImportItem);
}

// Línea de export real de VantaDB (VantaMemoryExportLine): timestamps, vector
// y versión son transporte, no metadata de usuario.
const VANTA_LINE = JSON.stringify({
  schema_version: 1,
  namespace: "docs",
  key: "k1",
  payload: "record exportado",
  metadata: { tipo: "nota" },
  vector: [0.1, 0.2, 0.3],
  sparse_vector: null,
  created_at_ms: 1700000000000,
  updated_at_ms: 1700000000001,
  version: 2,
  expires_at_ms: null,
});

// Punto Qdrant-style: payload es el objeto del record.
const QDRANT_LINE = JSON.stringify({
  id: "qd-1",
  vector: [0.5, 0.5],
  payload: { content: "texto qdrant", source: "web" },
});

test("parseVdbDump: línea de export VantaDB → id/text/ns/metadata limpios (sin transporte)", () => {
  const res = parseVdbDump(VANTA_LINE, NS);
  assert.equal(res.error, undefined);
  assert.equal(res.valid, 1);
  assert.equal(res.invalid, 0);
  assert.deepEqual(validItems(res.rows), [
    { id: "k1", text: "record exportado", namespace: "docs", metadata: { tipo: "nota" } },
  ]);
});

test("parseVdbDump: punto Qdrant-style → text de payload.content, resto del payload a metadata, vector descartado", () => {
  const res = parseVdbDump(QDRANT_LINE, NS);
  assert.equal(res.valid, 1);
  assert.deepEqual(validItems(res.rows), [
    { id: "qd-1", text: "texto qdrant", namespace: NS, metadata: { source: "web" } },
  ]);
});

test("parseVdbDump: payload Qdrant sin key text-ish → payload serializado como texto", () => {
  const res = parseVdbDump(JSON.stringify({ id: "p1", vector: [], payload: { count: 3 } }), NS);
  assert.equal(res.valid, 1);
  assert.equal(validItems(res.rows)[0].text, '{"count":3}');
});

test("parseVdbDump: NDJSON mixto VantaDB + Qdrant + línea rota → error por línea, no silencioso", () => {
  const text = [VANTA_LINE, QDRANT_LINE, "no-json"].join("\n");
  const res = parseVdbDump(text, NS);
  assert.equal(res.valid, 2);
  assert.equal(res.invalid, 1);
  assert.match(res.rows[2].error ?? "", /línea 3: JSON inválido/);
});

test("parseVdbDump: array JSON completo → todos los rows", () => {
  const text = JSON.stringify([
    { key: "a", payload: "uno", metadata: { n: 1 } },
    { id: "b", text: "dos" },
  ]);
  const res = parseVdbDump(text, NS);
  assert.equal(res.valid, 2);
  assert.deepEqual(validItems(res.rows).map((i) => i.id), ["a", "b"]);
});

test("parseVdbDump: vacío → error global legible", () => {
  const res = parseVdbDump("  \n", NS);
  assert.equal(res.error, "El archivo está vacío.");
  assert.equal(res.rows.length, 0);
});

test("parseVdbDump: objeto sin text/payload → error por fila", () => {
  const res = parseVdbDump('{"key":"x"}', NS);
  assert.equal(res.valid, 0);
  assert.equal(res.invalid, 1);
  assert.match(res.rows[0].error ?? "", /falta text\/payload\/content/);
});

test("parseImportFile: extensión .vdbdump → parseVdbDump", () => {
  const res = parseImportFile("memorias.vdbdump", QDRANT_LINE, NS);
  assert.equal(res.valid, 1);
  assert.equal(validItems(res.rows)[0].text, "texto qdrant");
});

test("parseImportFile: .jsonl → mismos rows que pegar el contenido (parseImport)", () => {
  const jsonl = ['{"text":"a","key":"x1"}', '{"text":"b","key":"x2"}'].join("\n");
  const fromFile = parseImportFile("datos.jsonl", jsonl, NS);
  const fromPaste = parseImport(jsonl, NS);
  assert.deepEqual(fromFile, fromPaste);
  assert.equal(fromFile.valid, 2);
});

test("parseImportFile: .csv → mismos rows que pegar el contenido", () => {
  const csv = ["key,payload", "c1,primera", "c2,segunda"].join("\n");
  const fromFile = parseImportFile("datos.csv", csv, NS);
  const fromPaste = parseImport(csv, NS);
  assert.deepEqual(fromFile, fromPaste);
  assert.equal(fromFile.valid, 2);
});

test("parseImportFile: > MAX_IMPORT líneas → truncated y solo los primeros", () => {
  const lines: string[] = [];
  for (let i = 1; i <= MAX_IMPORT + 5; i++) lines.push(`{"key":"k${i}","text":"t${i}"}`);
  const res = parseImportFile("grande.vdbdump", lines.join("\n"), NS);
  assert.equal(res.truncated, true);
  assert.equal(res.rows.length, MAX_IMPORT);
});

test("runImport: rows de un .vdbdump → ingestBatch en chunks, reporte verde", async () => {
  const lines: string[] = [];
  for (let i = 1; i <= 60; i++) lines.push(JSON.stringify({ id: `k${i}`, text: `t${i}` }));
  const res = parseVdbDump(lines.join("\n"), NS);
  assert.equal(res.valid, 60);
  let calls = 0;
  const report = await runImport(
    res.rows.filter((r) => r.item !== null),
    async (chunk: ImportItem[]) => {
      calls++;
      return chunk.map((c) => c.id ?? "");
    },
  );
  assert.equal(report.imported, 60);
  assert.equal(report.errors.length, 0);
  assert.equal(calls, 2); // chunks de 50: 50 + 10
});