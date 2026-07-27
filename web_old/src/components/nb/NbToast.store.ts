type ToastVariant = "info" | "success" | "error" | "warning";

interface Toast {
  id: number;
  message: string;
  variant: ToastVariant;
  exiting: boolean;
}

let toastId = 0;
const listeners: Set<(t: Toast) => void> = new Set();

export const toast = (message: string, variant: ToastVariant = "info") => {
  const t: Toast = { id: ++toastId, message, variant, exiting: false };
  listeners.forEach((fn) => fn(t));
};

export type { Toast, ToastVariant };
export { listeners };
