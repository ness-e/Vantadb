// Export helpers (VS-16): serialization of the current view to a JSONL file
// that `vanta_ingest_batch` can import back 1:1.
//
// Roundtrip contract: the bridge `IngestItem` (desktop/src-tauri connections
// types.rs) accepts exactly {id, namespace, text, embedding, metadata} — the
// same shape we emit, so `ingestBatch(lines.map(JSON.parse))` restores the
// view. TTLs (expires_at_ms) are NOT part of IngestItem and are deliberately
// omitted: importing them would silently drop them anyway.
import type { MemoryRecord } from "../../vanta";

/** One JSON object per line, no trailing newline. */
export function recordsToJsonl(records: MemoryRecord[]): string {
  return records
    .map((r) =>
      JSON.stringify({
        id: r.id,
        namespace: r.namespace,
        text: r.text,
        embedding: r.vector ?? undefined,
        metadata: r.metadata ?? undefined,
      }),
    )
    .join("\n");
}

/** Trigger a browser download of `content` as `filename` (UTF-8 text). */
export function downloadText(filename: string, content: string): void {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

/** Copy `content` to the clipboard; resolves false when unavailable. */
export async function copyText(content: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(content);
    return true;
  } catch {
    return false;
  }
}
