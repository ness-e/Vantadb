import { useCallback, useEffect, useState } from "react";
import { type Toast, type ToastVariant, listeners } from "./NbToast.store";

const ICONS: Record<ToastVariant, string> = {
  info: "i",
  success: "\u2713",
  error: "\u2715",
  warning: "!",
};

export function NbToastContainer() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((t: Toast) => {
    setToasts((prev) => [...prev, t]);
    setTimeout(() => {
      setToasts((prev) => prev.map((x) => (x.id === t.id ? { ...x, exiting: true } : x)));
      setTimeout(() => {
        setToasts((prev) => prev.filter((x) => x.id !== t.id));
      }, 200);
    }, 3200);
  }, []);

  useEffect(() => {
    listeners.add(addToast);
    return () => {
      listeners.delete(addToast);
    };
  }, [addToast]);

  return (
    <div className="nb-toast-container" aria-live="polite">
      {toasts.map((t) => (
        <div key={t.id} className={`nb-toast nb-toast--enter${t.exiting ? " nb-toast--exit" : ""}`}>
          <span className={`nb-toast-icon nb-toast-icon--${t.variant}`}>{ICONS[t.variant]}</span>
          <span className="nb-toast-msg">{t.message}</span>
        </div>
      ))}
    </div>
  );
}
