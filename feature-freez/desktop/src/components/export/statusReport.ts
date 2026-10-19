// Status report (VS-16): readable markdown summary of the current view.
// Pure functions — no DOM, testable with node --test.
// H-05: output in ES — the whole UI is Spanish; an EN report was inconsistent.
//
// The report describes the *current view*, so it derives counts from the
// records the UI already has (the filtered view) or a `list({limit})` fetch
// done by the caller — per-namespace totals for the whole store come from
// `namespaceStats()` (VS-CORE-02), not here.
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
  lines.push(`# Reporte de estado VantaDB`);
  lines.push("");
  lines.push(`Generado: ${opts.generatedAt}`);
  lines.push(`Registros en vista: ${records.length}`);
  lines.push("");
  lines.push(`## Namespaces`);
  lines.push("");
  lines.push(`| Namespace | Registros |`);
  lines.push(`| --- | --- |`);
  for (const [ns, list] of [...byNs.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
    lines.push(`| \`${ns}\` | ${list.length} |`);
  }
  lines.push("");

  lines.push(`## Campos de metadata`);
  lines.push("");
  if (metaFields.length === 0) {
    lines.push("Sin campos de metadata en la vista actual.");
  } else {
    lines.push(`| Campo | Tipo |`);
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
    lines.push(`## Expiraciones próximas`);
    lines.push("");
    if (expiring.length === 0) {
      lines.push("Ningún registro expira en la vista actual.");
    } else {
      lines.push(`| Key | Namespace | Expira | En |`);
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
