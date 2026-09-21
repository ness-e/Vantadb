import { toast as sonnerToast } from "sonner";
import { Check, Copy, AlertTriangle, Info } from "lucide-react";
import { dictionaries, DEFAULT_LANG, type Lang } from "@/lib/dictionaries";

type ToastOpts = { description?: string };

/**
 * Current UI language outside the React tree. `document.documentElement.lang`
 * is the canonical runtime source — LanguageProvider syncs it on mount and on
 * every setLang (lib/language-provider.tsx). Falls back to the SSR default.
 */
function currentLang(): Lang {
  if (typeof document !== "undefined") {
    const l = document.documentElement.lang?.slice(0, 2);
    if (l === "es" || l === "en") return l;
  }
  return DEFAULT_LANG;
}

/** Dictionary lookup mirroring LanguageProvider's fallback chain (language-provider.tsx:63). */
function translate(key: string): string {
  const lang = currentLang();
  return dictionaries[lang]?.[key] || dictionaries.es?.[key] || key;
}

/**
 * Toast an unknown error by engine error code when available.
 * The web app never imports `vantadb` as a module (WASM runs inside the
 * sandboxed playground iframe and only plain strings cross postMessage), so
 * VantaError instances don't reach this side of the boundary — we duck-type
 * `error.code: string` instead of `instanceof`. Unknown or unmapped codes
 * fall back to `toast.error`; the raw message is shown only in dev.
 */
export function toastError(error: unknown, opts?: ToastOpts) {
  let code: string | undefined;
  // Duck-type: engine errors carry a stable `code` (VantaError::code() →
  // VANTADB_* values, see vantadb-ts ERROR_CODES).
  if (typeof error === "object" && error !== null && "code" in error && typeof error.code === "string") {
    code = error.code;
  }
  const mapped = code ? translate(`errors.${code}`) : undefined;
  const title = mapped && mapped !== `errors.${code}` ? mapped : translate("toast.error");
  const message = error instanceof Error ? error.message : code ?? (error === undefined ? "" : String(error));
  toast.error(title, {
    description: opts?.description ?? (process.env.NODE_ENV === "development" ? message : undefined),
  });
}

/**
 * Manga-styled toast helpers built on sonner.
 * Usage: import { toast } from "@/components/vanta/toast";
 */
export const toast = {
  copied: (label: string, opts?: ToastOpts) =>
    sonnerToast.success(`Copiado: ${label}`, {
      description: opts?.description,
      icon: <Check className="h-4 w-4" strokeWidth={3} />,
    }),

  copy: (label: string) =>
    sonnerToast("Copiado al portapapeles", {
      description: label,
      icon: <Copy className="h-4 w-4" strokeWidth={2.5} />,
    }),

  error: (message: string, opts?: ToastOpts) =>
    sonnerToast.error(message, {
      description: opts?.description,
      icon: <AlertTriangle className="h-4 w-4" strokeWidth={2.5} />,
    }),

  info: (message: string, opts?: ToastOpts) =>
    sonnerToast(message, {
      description: opts?.description,
      icon: <Info className="h-4 w-4" strokeWidth={2.5} />,
    }),

  raw: sonnerToast,
};
