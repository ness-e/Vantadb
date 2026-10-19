// Copy-as (VS-17): serialización de un MemoryRecord a JSON / markdown + copia
// al portapapeles. Módulo puro (React-free a propósito: el self-check
// `desktop/scripts/selfcheck-vs17.mjs` lo compila con tsc y valida el shape);
// type-only import de vanta como activity/logic.ts (se erasa en compilación).
import type { MemoryRecord } from "../../vanta";

/** Registro completo como JSON pretty-printed (copiar "registro completo"). */
export function recordToJson(record: MemoryRecord): string {
  return JSON.stringify(record, null, 2);
}

/** Registro como documento markdown legible (copiar "payload markdown"):
 * encabezado ns/key, payload, tabla de metadata, pie con versión/actualización. */
export function recordToMarkdown(record: MemoryRecord): string {
  const lines: string[] = [];
  lines.push(`# ${record.namespace}/${record.id}`);
  lines.push("");
  lines.push(record.text);

  const metaEntries = Object.entries(record.metadata ?? {});
  if (metaEntries.length > 0) {
    lines.push("");
    lines.push("## Metadata");
    lines.push("");
    lines.push("| key | value |");
    lines.push("| --- | --- |");
    for (const [k, v] of metaEntries) {
      lines.push(`| ${k} | ${typeof v === "string" ? v : JSON.stringify(v)} |`);
    }
  }

  const bits: string[] = [];
  if (record.version != null) bits.push(`v${record.version}`);
  if (record.updated_at_ms) bits.push(`updated ${new Date(record.updated_at_ms).toISOString()}`);
  if (record.vector?.length) bits.push(`${record.vector.length}d`);
  if (bits.length > 0) {
    lines.push("");
    lines.push(`<!-- ${bits.join(" · ")} -->`);
  }
  return lines.join("\n");
}

/** Copia texto al portapapeles. Tauri v2 expone `navigator.clipboard` (WebView2
 * / WKWebView); si no está disponible lanza un error descriptivo para el UI. */
export async function copyText(text: string): Promise<void> {
  if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error("portapapeles no disponible");
}