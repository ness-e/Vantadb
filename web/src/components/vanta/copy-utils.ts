/**
 * copyToClipboard — robust copy with fallback for browsers without Clipboard API.
 * Uses navigator.clipboard.writeText when available, falls back to a hidden
 * textarea + execCommand('copy') for older browsers / insecure contexts.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  // Modern path: Clipboard API (requires secure context)
  if (
    typeof navigator !== "undefined" &&
    navigator.clipboard &&
    typeof navigator.clipboard.writeText === "function" &&
    window.isSecureContext
  ) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // fall through to legacy path
    }
  }

  // Legacy fallback: hidden textarea + execCommand
  if (typeof document === "undefined") return false;
  try {
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.top = "-9999px";
    textarea.style.left = "-9999px";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(textarea);
    return ok;
  } catch {
    return false;
  }
}
