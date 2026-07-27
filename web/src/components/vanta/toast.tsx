import { toast as sonnerToast } from "sonner";
import { Check, Copy, AlertTriangle, Info } from "lucide-react";

type ToastOpts = { description?: string };

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
