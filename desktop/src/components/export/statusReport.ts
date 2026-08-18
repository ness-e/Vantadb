// Status report (VS-16): readable markdown summary of the current view.
// Pure functions — no DOM, testable with node --test.
//
// `namespace_stats` is not exposed on the desktop bridge (per VS-16 contract),
// so the report derives counts from the records the UI already has (the
// filtered view) or a `list({limit})` fetch done by the caller.
import type { MemoryRecord } from "../../vanta.ts";
import { inferMetaFields } from "../search/filters-core.ts";

export interface StatusReportOptions {
  /** ISO-ish label for the report title, e.g. new Date().toISOString(). */
  generatedAt: string;
  /** When true, records with a future expiry are listed in an "upcoming TTLs" section. */
  includeUpcomingTtls?: boolean;
}

function fmtMs(ms: number | null | undefined): string {
  if (!ms) return "—";
  return new Date(ms).toISOString();
}

function fmtDuration(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  const d = Math.floor(s / 86_400);
  const h = Math.floor((s % 86_400) / 3_600);
  const m = Math.floor((s % 3_600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

/** Build the markdown report from the given records (pure). */
export function buildStatusReport(
  records: MemoryRecord[],
  opts: StatusReportOptions,
): string {
  const byNs = new Map<string, MemoryRecord[]>();
  for (const r of records) {
    const list = byNs.get(r.namespace) ?? [];
    list.push(r);
    byNs.set(r.namespace, list);
  }

  const metaFields = inferMetaFields(records);

  const lines: string[] = [];
  lines.push(`# VantaDB status report`);
  lines.push("");
  lines.push(`Generated: ${opts.generatedAt}`);
  lines.push(`Records in view: ${records.length}`);
  lines.push("");
  lines.push(`## Namespaces`);
  lines.push("");
  lines.push(`| Namespace | Records |`);
  lines.push(`| --- | --- |`);
  for (const [ns, list] of [...byNs.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
    lines.push(`| \`${ns}\` | ${list.length} |`);
  }
  lines.push("");

  lines.push(`## Metadata fields`);
  lines.push("");
  if (metaFields.length === 0) {
    lines.push("No metadata fields present in the current view.");
  } else {
    lines.push(`| Field | Type |`);
    lines.push(`| --- | --- |`);
    for (const f of metaFields) {
      lines.push(`| \`${f.name}\` | \`${f.type}\` |`);
    }
  }
  lines.push("");

  if (opts.includeUpcomingTtls) {
    const now = Date.now();
    const expiring = records
      .filter((r) => r.expires_at_ms != null && r.expires_at_ms > now)
      .sort((a, b) => (a.expires_at_ms ?? 0) - (b.expires_at_ms ?? 0));
    lines.push(`## Upcoming expirations`);
    lines.push("");
    if (expiring.length === 0) {
      lines.push("No records expire in the current view.");
    } else {
      lines.push(`| Key | Namespace | Expires | In |`);
      lines.push(`| --- | --- | --- | --- |`);
      for (const r of expiring) {
        lines.push(
          `| \`${r.id}\` | \`${r.namespace}\` | ${fmtMs(r.expires_at_ms)} | ${fmtDuration(r.expires_at_ms! - now)} |`,
        );
      }
    }
    lines.push("");
  }

  return lines.join("\n");
}
