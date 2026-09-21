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
    } catch (e) {
      // Degradación intencional: permisos/secure-context pueden denegar la
      // Clipboard API; el caller NO debe ver un fallo, recién fallaría si
      // también falla el fallback legacy abajo. Se loggea para diagnóstico.
      console.warn("copyToClipboard: Clipboard API unavailable, using legacy fallback", e);
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
  } catch (e) {
    // Ambos paths fallaron: se retorna false y el caller muestra el toast
    // ("No se pudo copiar"). Se loggea la causa para no silenciar el error.
    console.warn("copyToClipboard: legacy execCommand fallback failed", e);
    return false;
  }
}
