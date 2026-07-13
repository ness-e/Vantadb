import type { ReactNode } from "react";
import { animate } from "motion";
import { useEffect, useRef } from "react";
import { useFocusTrap } from "@/hooks/useFocusTrap";

interface NavDrawerProps {
  open: boolean;
  onClose: () => void;
  header: ReactNode;
  body: ReactNode;
  footer: ReactNode;
}

export function NavDrawer({ open, onClose, header, body, footer }: NavDrawerProps) {
  const drawerRef = useRef<HTMLDivElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);

  useFocusTrap(drawerRef, open);

  useEffect(() => {
    if (open && bodyRef.current) {
      const children = Array.from(bodyRef.current.children);
      animate(
        children,
        { opacity: [0, 1], x: [-20, 0] },
        {
          duration: 0.18,
          delay: 0.04,
          ease: [0.25, 0.46, 0.45, 0.94],
        },
      );
    }
  }, [open]);

  useEffect(() => {
    document.body.classList.toggle("overflow-hidden", open);
    return () => {
      document.body.classList.remove("overflow-hidden");
    };
  }, [open]);

  return (
    <>
      {open && <div className="nc-nav-overlay" onClick={onClose} aria-hidden="true" />}

      <div
        className="nc-nav-drawer"
        ref={drawerRef}
        aria-hidden={!open}
        role="dialog"
        aria-modal="true"
        aria-label="Navigation menu"
      >
        <div className="nc-drawer-header">{header}</div>
        <div className="nc-drawer-body" ref={bodyRef}>
          {body}
        </div>
        <div className="nc-drawer-footer">{footer}</div>
      </div>
    </>
  );
}
