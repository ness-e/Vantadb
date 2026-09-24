import { useEffect, useRef } from "react";
import { tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

/**
 * AppContextMenu (FIND-21) — menú contextual propio in-app: reemplaza el menú
 * nativo del WebView en right-click. Posición fija + clamp a viewport; Esc o
 * click-fuera lo cierra. Sin deps nativas (el shortcut global Tauri queda
 * DEFER — ver docs/user/desktop/GUIDE.md).
 */

export interface ContextMenuItem {
  id: string;
  label: string;
  /** Atajo visible (ej. "Ctrl+K") — solo etiqueta, el binding vive en el shell. */
  hint: string;
  onSelect: () => void;
}

export function AppContextMenu({
  x,
  y,
  items,
  onClose,
  lang = connectionPrefs.get().lang ?? "es",
}: {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
  lang?: DesktopLang;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onDocClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    }
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [onClose]);

  // Clamp: el menú nunca se sale del viewport.
  const left = Math.max(4, Math.min(x, window.innerWidth - 260));
  const top = Math.max(4, Math.min(y, window.innerHeight - items.length * 36 - 16));

  return (
    <div
      ref={ref}
      role="menu"
      aria-label={tt(lang, "ctxmenu.aria", "Menú contextual")}
      tabIndex={-1}
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
      }}
      className="fixed z-[60] w-60 border-2 border-foreground bg-background shadow-[4px_4px_0_0_#000]"
      style={{ left, top }}
    >
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          role="menuitem"
          onClick={() => {
            item.onSelect();
            onClose();
          }}
          className="flex w-full items-center gap-3 px-3 py-2 text-left text-sm text-foreground hover:bg-foreground hover:text-background"
        >
          <span className="flex-1">{item.label}</span>
          <kbd className="font-tech text-[11px] opacity-70">{item.hint}</kbd>
        </button>
      ))}
    </div>
  );
}
