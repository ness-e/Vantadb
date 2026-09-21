// Inspector helpers (VS-06): metadata KV rows con tipo inferido de VantaValue
// y draft de TTL. El wire del bridge es JSON plano (desktop/src-tauri native.rs
// `from_vanta_value`): DateTime → string RFC3339, listas → arrays, objetos →
// string JSON. El editor trabaja sobre ese JSON plano y `rowsToMetadata` vuelve
// a producir valores que el bridge re-mapea a VantaValue sin pérdida.
import type { MemoryRecord } from "../../vanta";

// --- Metadata KV rows ---------------------------------------------------------

/** Tipos VantaValue expuestos en el editor (string/int/float/bool/datetime/list/null). */
export type MetaType = "str" | "int" | "flt" | "bool" | "date" | "lst" | "nil";

export interface MetaRow {
  key: string;
  type: MetaType;
  /** Valor crudo del input; se tipa en `rowsToMetadata`. */
  raw: string;
}

/** RFC3339 / ISO local (lo que serializa `from_vanta_value` para DateTime). */
const ISO_RE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/;

export function inferType(v: unknown): MetaType {
  if (v === null || v === undefined) return "nil";
  if (typeof v === "boolean") return "bool";
  if (typeof v === "number") return Number.isInteger(v) ? "int" : "flt";
  if (typeof v === "string") return ISO_RE.test(v) ? "date" : "str";
  if (Array.isArray(v)) return "lst";
  // Objetos anidados: el bridge los aplana a string JSON en lectura.
  return "str";
}

/** datetime-local value ("YYYY-MM-DDTHH:mm", hora local) desde un ISO/RFC3339. */
export function dateToLocalInput(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** Valor almacenado → string crudo del input según su tipo inferido. */
export function valueToRaw(v: unknown): string {
  if (v === null || v === undefined) return "";
  if (Array.isArray(v) || (typeof v === "object" && typeof v !== "string")) {
    return JSON.stringify(v);
  }
  const s = String(v);
  return typeof v === "string" && ISO_RE.test(s) ? dateToLocalInput(s) : s;
}

export function metadataToRows(meta: Record<string, unknown> | null | undefined): MetaRow[] {
  return Object.entries(meta ?? {}).map(([key, v]) => ({
    key,
    type: inferType(v),
    raw: valueToRaw(v),
  }));
}

/**
 * Rows → metadata JSON plano (lo que `vantaPut` espera). Valida cada fila y
 * devuelve `error` ante input inválido (evita corromper metadata con NaN/JSON
 * roto — validación en la frontera del guardado).
 */
export function rowsToMetadata(rows: MetaRow[]): {
  meta: Record<string, unknown>;
  error: string | null;
} {
  const meta: Record<string, unknown> = {};
  const seen = new Set<string>();
  for (const row of rows) {
    const key = row.key.trim();
    if (!key) return { meta: {}, error: "metadata: fila con key vacía" };
    if (seen.has(key)) return { meta: {}, error: `metadata: key duplicada "${key}"` };
    seen.add(key);
    switch (row.type) {
      case "str":
        meta[key] = row.raw;
        break;
      case "int": {
        if (row.raw.trim() === "") return { meta: {}, error: `metadata: "${key}" vacío` };
        const n = Number(row.raw);
        if (!Number.isInteger(n)) return { meta: {}, error: `metadata: "${key}" no es un entero` };
        meta[key] = n;
        break;
      }
      case "flt": {
        if (row.raw.trim() === "") return { meta: {}, error: `metadata: "${key}" vacío` };
        const n = Number(row.raw);
        if (Number.isNaN(n)) return { meta: {}, error: `metadata: "${key}" no es un número` };
        meta[key] = n;
        break;
      }
      case "bool":
        meta[key] = row.raw === "true";
        break;
      case "date": {
        const t = Date.parse(row.raw);
        if (Number.isNaN(t)) return { meta: {}, error: `metadata: "${key}" no es una fecha válida` };
        meta[key] = new Date(t).toISOString();
        break;
      }
      case "lst": {
        try {
          const parsed = JSON.parse(row.raw);
          if (!Array.isArray(parsed)) {
            return { meta: {}, error: `metadata: "${key}" debe ser una lista JSON` };
          }
          meta[key] = parsed;
        } catch {
          return { meta: {}, error: `metadata: "${key}" tiene JSON inválido` };
        }
        break;
      }
      case "nil":
        meta[key] = null;
        break;
    }
  }
  return { meta, error: null };
}

// --- TTL draft -----------------------------------------------------------------

export type TtlMode = "never" | "relative" | "absolute";

export interface TtlDraft {
  mode: TtlMode;
  /** relative: duración en minutos (el input de edición usa min). */
  relMinutes: number;
  /** absolute: valor datetime-local ("YYYY-MM-DDTHH:mm"). */
  absLocal: string;
}

export function ttlFromMs(ms: number): TtlDraft {
  return {
    mode: "absolute",
    relMinutes: 0,
    absLocal: dateToLocalInput(new Date(ms).toISOString()),
  };
}

export function ttlFromRecord(record: MemoryRecord): TtlDraft {
  return record.expires_at_ms ? ttlFromMs(record.expires_at_ms) : { mode: "never", relMinutes: 0, absLocal: "" };
}

/** Draft → expires_at_ms absoluto; null = nunca expira (vantaPut con undefined). */
export function ttlToMs(d: TtlDraft, now: number): number | null {
  if (d.mode === "never") return null;
  if (d.mode === "relative") {
    return d.relMinutes > 0 ? now + d.relMinutes * 60_000 : null;
  }
  const t = Date.parse(d.absLocal);
  return Number.isNaN(t) ? null : t;
}

// --- Formatters -----------------------------------------------------------------

export function fmtDateTime(ms: number): string {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

export function fmtRelative(ms: number, now: number): string {
  const diff = Math.max(0, now - ms);
  const m = Math.floor(diff / 60_000);
  if (m < 1) return "now";
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

export function fmtDuration(ms: number): string {
  const m = Math.max(0, Math.ceil(ms / 60_000));
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ${m % 60}m`;
  return `${Math.floor(h / 24)}d ${h % 24}h`;
}