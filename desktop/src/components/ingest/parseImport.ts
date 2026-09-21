// OP-01 — Parser de import pegado: CSV (con cabecera), JSON array, objeto único
// o NDJSON → filas validables para `ingestBatch`. 100% puro (sin imports del
// bridge) para ser testeable en vitest sin Tauri.
//
// Reglas del contrato:
// - CSV: primera fila = cabecera; columnas reconocidas (case-insensitive):
//   key|id|node_id → id · text|payload|content|message → text ·
//   metadata|metadata_json|meta → metadata (JSON) · namespace → override.
//   Columnas desconocidas se ignoran. Delimitador "," o "\t" (si no hay comas).
// - JSON: array de objetos, objeto único, o NDJSON (una línea = un objeto).
//   Keys restantes no reconocidas → metadata (nunca se pierde data).
// - Filas sin text/payload/content → error POR FILA (con índice 1-based del
//   paste original). Error de formato global → `error` legible.
// - Máx 1000 registros por paste → `truncated: true` (la UI avisa).
//
// `runImport` ejecuta `ingestBatch` en chunks de 50 (el bridge `put_batch` es
// atómico por llamada → una falla de chunk se reporta con el rango de filas
// originales; los errores exactos por fila los captura el parse).

export const MAX_IMPORT = 1000;
export const CHUNK_SIZE = 50;

/** Item mínimo, estructuralmente compatible con `IngestItem` de vanta.ts. */
export interface ImportItem {
  id?: string;
  text: string;
  namespace?: string;
  metadata?: Record<string, unknown>;
}

export interface ParsedRow {
  /** Índice 1-based en el texto pegado original (para reportar errores). */
  index: number;
  item: ImportItem | null;
  error?: string;
}

export interface ParseResult {
  rows: ParsedRow[];
  valid: number;
  invalid: number;
  /** true si el paste excede `limit` registros — solo se conservan los primeros. */
  truncated: boolean;
  /** Error global legible (input no parseable). */
  error?: string;
}

export interface ImportError {
  /** Referencia a filas: "3", "12-15" o "1-50" (rango de chunk). */
  rows: string;
  message: string;
}

export interface ImportReport {
  imported: number;
  errors: ImportError[];
}

const ID_KEYS = ["key", "id", "node_id"];
const TEXT_KEYS = ["text", "payload", "content", "message"];
const META_KEYS = ["metadata", "metadata_json", "meta"];
const NS_KEYS = ["namespace"];

/** CSV de muestra para el botón "EJEMPLO" del modal. */
export const EXAMPLE_CSV = `key,payload,metadata_json
nota-1,Primera memoria importada,"{""tipo"":""nota"",""prioridad"":1}"
nota-2,Segunda memoria importada,"{""tipo"":""nota"",""prioridad"":2}"
nota-3,Tercera memoria importada,"{""tipo"":""nota"",""prioridad"":3}"`;

function norm(s: string): string {
  return s.trim().toLowerCase();
}

/** Split de una línea CSV con soporte mínimo de comillas (RFC 4180). Sin
 * campos multilínea (limitación documentada: los pastes reales no los usan). */
function splitCsvLine(line: string, delim: string): string[] {
  const cells: string[] = [];
  let cur = "";
  let inQ = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (inQ) {
      if (ch === '"') {
        if (line[i + 1] === '"') {
          cur += '"';
          i++;
        } else {
          inQ = false;
        }
      } else {
        cur += ch;
      }
    } else if (ch === '"') {
      inQ = true;
    } else if (ch === delim) {
      cells.push(cur);
      cur = "";
    } else {
      cur += ch;
    }
  }
  cells.push(cur);
  return cells;
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return v !== null && typeof v === "object" && !Array.isArray(v);
}

function asId(v: unknown): string | undefined {
  if (typeof v === "string" && v.trim()) return v.trim();
  if (typeof v === "number" && Number.isFinite(v)) return String(v);
  return undefined;
}

function textOf(v: unknown): string {
  return typeof v === "string" ? v : JSON.stringify(v);
}

/** Fila desde un objeto JSON (array / NDJSON / objeto único). */
function jsonRow(index: number, obj: unknown, ns: string): ParsedRow {
  if (!isRecord(obj)) {
    return { index, item: null, error: `fila ${index}: se esperaba un objeto JSON` };
  }
  const item: ImportItem = { text: "", namespace: ns };
  const meta: Record<string, unknown> = {};
  let textSet = false;

  for (const [rawK, v] of Object.entries(obj)) {
    const k = norm(rawK);
    if (ID_KEYS.includes(k)) {
      const id = asId(v);
      if (id) item.id = id;
    } else if (TEXT_KEYS.includes(k)) {
      item.text = textOf(v);
      textSet = true;
    } else if (NS_KEYS.includes(k)) {
      if (typeof v === "string" && v.trim()) item.namespace = v.trim();
    } else if (META_KEYS.includes(k)) {
      if (isRecord(v)) {
        Object.assign(meta, v);
      } else if (typeof v === "string") {
        try {
          const parsed: unknown = JSON.parse(v);
          if (isRecord(parsed)) Object.assign(meta, parsed);
        } catch {
          return { index, item: null, error: `fila ${index}: metadata no es JSON válido` };
        }
      }
    } else {
      // Keys no reconocidas → metadata (nunca se pierde data).
      meta[rawK] = v;
    }
  }

  if (!textSet || !item.text.trim()) {
    return { index, item: null, error: `fila ${index}: falta text/payload/content` };
  }
  if (Object.keys(meta).length > 0) item.metadata = meta;
  return { index, item };
}

/** Fila desde una línea CSV (mapeo por rol de columna). */
function csvRow(
  index: number,
  cells: string[],
  colRole: Map<number, "id" | "text" | "meta" | "ns">,
  ns: string,
): ParsedRow {
  const item: ImportItem = { text: "", namespace: ns };
  const meta: Record<string, unknown> = {};
  let textSet = false;

  for (const [i, role] of colRole) {
    const v = cells[i] ?? "";
    const trimmed = v.trim();
    switch (role) {
      case "id":
        if (trimmed) item.id = trimmed;
        break;
      case "text":
        item.text = v;
        textSet = true;
        break;
      case "ns":
        if (trimmed) item.namespace = trimmed;
        break;
      case "meta": {
        if (!trimmed) break;
        try {
          const parsed: unknown = JSON.parse(trimmed);
          if (isRecord(parsed)) Object.assign(meta, parsed);
        } catch {
          return { index, item: null, error: `fila ${index}: metadata no es JSON válido` };
        }
        break;
      }
    }
  }

  if (!textSet || !item.text.trim()) {
    return { index, item: null, error: `fila ${index}: falta text/payload/content` };
  }
  if (Object.keys(meta).length > 0) item.metadata = meta;
  return { index, item };
}

/** Devuelve un error GLOBAL legible si nada pudo parsearse (JSON roto de una
 * sola línea, NDJSON todo inválido, etc.). Con parse mixto (algunas líneas ok)
 * devuelve undefined y los errores quedan por línea. */
function parseJson(
  input: string,
  rows: ParsedRow[],
  ns: string,
  limit: number,
  onTruncate: () => void,
): string | undefined {
  // JSON.parse del input completo: array de objetos u objeto único.
  try {
    const parsed: unknown = JSON.parse(input);
    const arr = Array.isArray(parsed) ? parsed : [parsed];
    for (const obj of arr) {
      if (rows.length >= limit) {
        onTruncate();
        break;
      }
      rows.push(jsonRow(rows.length + 1, obj, ns));
    }
    return undefined;
  } catch (wholeErr) {
    // No es JSON completo → intentar NDJSON (una línea = un objeto).
    let lineNo = 0;
    let parsedAny = false;
    let lastErr: unknown = wholeErr;
    for (const line of input.split(/\r?\n/)) {
      const t = line.trim();
      if (!t) continue;
      lineNo++;
      if (rows.length >= limit) {
        onTruncate();
        break;
      }
      try {
        rows.push(jsonRow(lineNo, JSON.parse(t), ns));
        parsedAny = true;
      } catch (e) {
        rows.push({
          index: lineNo,
          item: null,
          error: `línea ${lineNo}: JSON inválido — ${(e as Error).message}`,
        });
        lastErr = e;
      }
    }
    if (!parsedAny) {
      const msg = lastErr instanceof Error ? lastErr.message : String(lastErr);
      return `JSON inválido — ${msg}`;
    }
    return undefined;
  }
}

function parseCsv(
  input: string,
  rows: ParsedRow[],
  ns: string,
  limit: number,
  onTruncate: () => void,
): string | undefined {
  const lines = input.split(/\r?\n/);
  const headerIdx = lines.findIndex((l) => l.trim().length > 0);
  if (headerIdx === -1) return "No hay texto para importar.";

  const headerLine = lines[headerIdx].trim();
  const delim = headerLine.includes(",") ? "," : "\t";
  const cols = splitCsvLine(headerLine, delim).map(norm);

  const colRole = new Map<number, "id" | "text" | "meta" | "ns">();
  let textCol = -1;
  cols.forEach((c, i) => {
    if (ID_KEYS.includes(c)) colRole.set(i, "id");
    else if (TEXT_KEYS.includes(c)) {
      colRole.set(i, "text");
      textCol = i;
    } else if (META_KEYS.includes(c)) colRole.set(i, "meta");
    else if (NS_KEYS.includes(c)) colRole.set(i, "ns");
  });

  if (textCol === -1) {
    return `CSV sin columna de texto reconocida (cabecera: ${cols.join(", ") || "vacía"}). Se espera una cabecera con al menos una de: ${TEXT_KEYS.join(", ")}.`;
  }

  for (let li = headerIdx + 1; li < lines.length; li++) {
    const t = lines[li].trim();
    if (!t) continue;
    if (rows.length >= limit) {
      onTruncate();
      break;
    }
    rows.push(csvRow(li + 1, splitCsvLine(t, delim), colRole, ns));
  }
  return undefined;
}

/** Parse principal. `namespace` es el default para filas sin override. */
export function parseImport(text: string, namespace: string, limit = MAX_IMPORT): ParseResult {
  const trimmed = text.trim();
  if (!trimmed) {
    return { rows: [], valid: 0, invalid: 0, truncated: false, error: "Pegá CSV o JSON primero." };
  }

  const rows: ParsedRow[] = [];
  let truncated = false;
  const onTruncate = () => {
    truncated = true;
  };

  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    const err = parseJson(trimmed, rows, namespace, limit, onTruncate);
    if (err) {
      return { rows: [], valid: 0, invalid: 0, truncated, error: err };
    }
  } else {
    const err = parseCsv(trimmed, rows, namespace, limit, onTruncate);
    if (err) {
      return { rows: [], valid: 0, invalid: 0, truncated, error: err };
    }
  }

  const valid = rows.filter((r) => r.item !== null).length;
  return { rows, valid, invalid: rows.length - valid, truncated, error: undefined };
}

// ── .vdbdump (WASM-04) ─────────────────────────────────────────────────────
// Un `.vdbdump` es NDJSON de dos familias:
//   - Export real de VantaDB (`VantaMemoryExportLine`): {schema_version,
//     namespace, key, payload, metadata, vector, created_at_ms, ...} — los
//     campos de transporte (timestamps, version, vector) NO son metadata de
//     usuario y se descartan.
//   - Snapshot Qdrant-style: {id, vector, payload} donde `payload` es el
//     objeto del record — su key text-ish (content/text/...) es el texto y el
//     resto del payload pasa a metadata (nunca se pierde data).
// `parseVdbDump` acepta array JSON, objeto único o NDJSON (igual que
// `parseJson`), pero con el mapper de dump en vez de `jsonRow`.

/** Campos de transporte de un export VantaDB que no son metadata de usuario. */
const VDB_SKIP_KEYS = new Set([
  "schema_version",
  "vector",
  "sparse_vector",
  "created_at_ms",
  "updated_at_ms",
  "version",
  "node_id",
  "expires_at_ms",
]);

/** Fila desde un objeto de dump (export VantaDB o punto Qdrant-style). */
function vdbRow(index: number, obj: unknown, ns: string): ParsedRow {
  if (!isRecord(obj)) {
    return { index, item: null, error: `fila ${index}: se esperaba un objeto JSON` };
  }
  const item: ImportItem = { text: "", namespace: ns };
  const meta: Record<string, unknown> = {};
  let textSet = false;

  for (const [rawK, v] of Object.entries(obj)) {
    const k = norm(rawK);
    if (VDB_SKIP_KEYS.has(k)) continue; // transporte, no metadata de usuario
    if (ID_KEYS.includes(k)) {
      const id = asId(v);
      if (id) item.id = id;
    } else if (k === "payload" && isRecord(v)) {
      // Qdrant-style: payload es el objeto del record → su key text-ish es el
      // texto; el resto del payload va a metadata.
      let found = false;
      for (const [pk, pv] of Object.entries(v)) {
        if (TEXT_KEYS.includes(norm(pk))) {
          item.text = textOf(pv);
          textSet = true;
          found = true;
        } else {
          meta[pk] = pv;
        }
      }
      // Payload sin key text-ish → serializar el payload entero como texto.
      if (!found) {
        item.text = textOf(v);
        textSet = true;
      }
    } else if (TEXT_KEYS.includes(k)) {
      item.text = textOf(v);
      textSet = true;
    } else if (NS_KEYS.includes(k)) {
      if (typeof v === "string" && v.trim()) item.namespace = v.trim();
    } else if (META_KEYS.includes(k)) {
      if (isRecord(v)) {
        Object.assign(meta, v);
      } else if (typeof v === "string") {
        try {
          const parsed: unknown = JSON.parse(v);
          if (isRecord(parsed)) Object.assign(meta, parsed);
        } catch {
          return { index, item: null, error: `fila ${index}: metadata no es JSON válido` };
        }
      }
    } else {
      // Keys no reconocidas → metadata (nunca se pierde data).
      meta[rawK] = v;
    }
  }

  if (!textSet || !item.text.trim()) {
    return { index, item: null, error: `fila ${index}: falta text/payload/content` };
  }
  if (Object.keys(meta).length > 0) item.metadata = meta;
  return { index, item };
}

/** Parse de un `.vdbdump` (export VantaDB o snapshot Qdrant-style): array JSON,
 * objeto único o NDJSON. Misma firma/contrato que `parseImport`. */
export function parseVdbDump(
  text: string,
  namespace: string,
  limit = MAX_IMPORT,
): ParseResult {
  const trimmed = text.trim();
  if (!trimmed) {
    return { rows: [], valid: 0, invalid: 0, truncated: false, error: "El archivo está vacío." };
  }

  const rows: ParsedRow[] = [];
  let truncated = false;
  const onTruncate = () => {
    truncated = true;
  };

  // JSON.parse del input completo: array de objetos u objeto único.
  try {
    const parsed: unknown = JSON.parse(trimmed);
    const arr = Array.isArray(parsed) ? parsed : [parsed];
    for (const obj of arr) {
      if (rows.length >= limit) {
        onTruncate();
        break;
      }
      rows.push(vdbRow(rows.length + 1, obj, namespace));
    }
  } catch (wholeErr) {
    // No es JSON completo → NDJSON (una línea = un objeto).
    let lineNo = 0;
    let parsedAny = false;
    let lastErr: unknown = wholeErr;
    for (const line of trimmed.split(/\r?\n/)) {
      const t = line.trim();
      if (!t) continue;
      lineNo++;
      if (rows.length >= limit) {
        onTruncate();
        break;
      }
      try {
        rows.push(vdbRow(lineNo, JSON.parse(t), namespace));
        parsedAny = true;
      } catch (e) {
        rows.push({
          index: lineNo,
          item: null,
          error: `línea ${lineNo}: JSON inválido — ${(e as Error).message}`,
        });
        lastErr = e;
      }
    }
    if (!parsedAny) {
      const msg = lastErr instanceof Error ? lastErr.message : String(lastErr);
      return { rows: [], valid: 0, invalid: 0, truncated, error: `JSON inválido — ${msg}` };
    }
  }

  const valid = rows.filter((r) => r.item !== null).length;
  return { rows, valid, invalid: rows.length - valid, truncated, error: undefined };
}

/** Dispatch por extensión de archivo (ImportDrop, WASM-04): `.vdbdump` →
 * `parseVdbDump` (export VantaDB + Qdrant-style); el resto (`.json`/`.jsonl`/
 * `.csv`) → `parseImport` de OP-01. El texto del archivo ES el mismo input del
 * paste, así que el preview/reporte del drop es idéntico al del modal Pegar. */
export function parseImportFile(
  fileName: string,
  text: string,
  namespace: string,
  limit = MAX_IMPORT,
): ParseResult {
  const ext = fileName.toLowerCase().split(".").pop() ?? "";
  return ext === "vdbdump" ? parseVdbDump(text, namespace, limit) : parseImport(text, namespace, limit);
}

/** Importa las filas válidas en chunks (bridge `put_batch` atómico por llamada).
 * `ingestFn` inyectable (en la app: `ingestBatch` de vanta.ts). Las fallas de
 * chunk se reportan con el rango de filas ORIGINALES del paste. */
export async function runImport(
  validRows: ParsedRow[],
  ingestFn: (chunk: ImportItem[]) => Promise<string[]>,
  chunkSize = CHUNK_SIZE,
): Promise<ImportReport> {
  const items = validRows.filter((r) => r.item !== null);
  const report: ImportReport = { imported: 0, errors: [] };

  for (let i = 0; i < items.length; i += chunkSize) {
    const chunk = items.slice(i, i + chunkSize);
    const first = chunk[0].index;
    const last = chunk[chunk.length - 1].index;
    try {
      const ids = await ingestFn(chunk.map((r) => r.item as ImportItem));
      report.imported += ids.length;
    } catch (err) {
      report.errors.push({
        rows: first === last ? String(first) : `${first}-${last}`,
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }
  return report;
}