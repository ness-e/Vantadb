import { useEffect, useRef } from "react";

interface NbModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

export function NbModal({ open, onClose, title, children }: NbModalProps) {
  const overlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => { document.body.style.overflow = ""; };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      ref={overlayRef}
      className="nb-modal-overlay nb-modal-overlay--open"
      onClick={(e) => { if (e.target === overlayRef.current) onClose(); }}
    >
      <div className="nb-modal" role="dialog" aria-modal="true" aria-label={title}>
        <div className="nb-modal-head">
          <span className="nb-modal-title">{title}</span>
          <button className="nb-modal-close" onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>
        <div className="nb-modal-body">
          {children}
        </div>
      </div>
    </div>
  );
}
