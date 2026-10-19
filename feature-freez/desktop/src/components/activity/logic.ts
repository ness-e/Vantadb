// Lógica pura de la superficie ACTIVITY (VS-15): encoding de ops, grouping
// temporal y parseo del JSONL de audit. React-free a propósito: el self-check
// (`desktop/scripts/selfcheck-vs15.mjs`) la compila con tsc y la valida contra
// un fixture JSONL con la misma forma que escribe el core (`src/audit.rs`).
import type { AuditEvent } from "../../vanta";

/** Parse one JSONL line into an AuditEvent; null when malformed or missing a
 * required field. Mirrors the core/VS-12 behavior of skipping bad lines. */
export function parseAuditLine(line: string): AuditEvent | null {
  const trimmed = line.trim();
  if (!trimmed) return null;
  try {
    const v = JSON.parse(trimmed) as Partial<AuditEvent>;
    if (
      typeof v.timestamp !== "string" ||
      typeof v.op !== "string" ||
      typeof v.namespace !== "string" ||
      typeof v.key !== "string" ||
      typeof v.outcome !== "string"
    ) {
      return null;
    }
    return v as AuditEvent;
  } catch {
    return null;
  }
}

/** Encoding redundante por op: tone (color) + icon (glifo) + label (texto).
 * Mapea las ops reales que escribe el core (src/audit.rs + api.rs/impl_export.rs);
 * ops futuras (p.ej. `expire`) caen en el fallback `unknown`. */
export type OpTone = "neutral" | "batch" | "danger" | "transfer" | "unknown";

export interface OpMeta {
  label: string;
  icon: string;
  tone: OpTone;
}

const OP_META: Record<string, OpMeta> = {
  put: { label: "PUT", icon: "✎", tone: "neutral" },
  put_batch: { label: "BATCH", icon: "▤", tone: "batch" },
  delete: { label: "DEL", icon: "✕", tone: "danger" },
  delete_by_filter: { label: "DEL-F", icon: "▦", tone: "danger" },
  export_namespace: { label: "EXPORT", icon: "⤓", tone: "transfer" },
  export_all: { label: "EXPORT", icon: "⤓", tone: "transfer" },
  import_file: { label: "IMPORT", icon: "⤒", tone: "transfer" },
};

const UNKNOWN_OP: OpMeta = { label: "OP", icon: "◈", tone: "unknown" };

/** Metadata for a single op string, with a stable fallback for unknown ops. */
export function opMeta(op: string): OpMeta {
  const known = OP_META[op];
  if (known) return known;
  return { ...UNKNOWN_OP, label: op.toUpperCase().slice(0, 8) };
}

/** Outcome encoding: icon + text (redundant with the neon color at render). */
export interface OutcomeMeta {
  icon: string;
  label: string;
  err: boolean;
}

export function outcomeMeta(outcome: string): OutcomeMeta {
  return outcome === "ok"
    ? { icon: "✓", label: "ok", err: false }
    : { icon: "✕", label: "err", err: true };
}

/** Family used by the Timeline's summary counts (writes / deletes / transfers).
 * The core emits no `expire` op yet; future ops land in `transfer`. */
export type OpFamily = "write" | "delete" | "transfer";

export function opFamily(op: string): OpFamily {
  if (op === "put" || op === "put_batch") return "write";
  if (op === "delete" || op === "delete_by_filter") return "delete";
  return "transfer";
}

export type BucketGranularity = "hour" | "day";

/** Stable bucket key for an ISO timestamp at the given granularity, in LOCAL
 * time (display buckets; the log timestamps are UTC instants). */
export function bucketKey(ts: string, granularity: BucketGranularity): string {
  const d = new Date(ts);
  if (Number.isNaN(d.getTime())) return "sin-timestamp";
  const p = (n: number) => String(n).padStart(2, "0");
  const date = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  return granularity === "day" ? date : `${date}T${p(d.getHours())}:00`;
}

/** Human label for a bucket key, built from the key parts (not a Date
 * round-trip: `new Date("2026-08-18")` parses as UTC and can shift a day in
 * negative-offset zones). */
export function bucketLabel(key: string, granularity: BucketGranularity): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:T(\d{2}):00)?$/.exec(key);
  if (!m) return key;
  const [, y, mo, d, h] = m;
  const local = new Date(Number(y), Number(mo) - 1, Number(d), h ? Number(h) : 0);
  if (granularity === "day") {
    return local.toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" });
  }
  return local.toLocaleString(undefined, { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
}

/** Group events newest-first into ordered buckets of the given granularity. */
export interface TimeBucket {
  key: string;
  label: string;
  events: AuditEvent[];
}

export function groupByBucket(
  events: AuditEvent[],
  granularity: BucketGranularity,
): TimeBucket[] {
  const order: string[] = [];
  const buckets = new Map<string, AuditEvent[]>();
  for (const e of events) {
    const key = bucketKey(e.timestamp, granularity);
    const existing = buckets.get(key);
    if (existing) {
      existing.push(e);
    } else {
      buckets.set(key, [e]);
      order.push(key);
    }
  }
  return order.map((key) => ({ key, label: bucketLabel(key, granularity), events: buckets.get(key)! }));
}

/** Short local clock time for a table row ("14:03:05"). */
export function eventClock(ts: string): string {
  const d = new Date(ts);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}