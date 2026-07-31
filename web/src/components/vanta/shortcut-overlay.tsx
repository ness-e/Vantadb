"use client";

import { useEffect, useState, useRef } from "react";
import { X, Command, ArrowUp, ArrowDown, CornerDownLeft } from "lucide-react";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useLanguage } from "@/lib/language-provider";

const SHORTCUTS = [
  {
    group: "shortcuts.navigation",
    items: [
      { keys: ["⌘/Ctrl", "K"], label: "shortcuts.openQuickSearch" },
      { keys: ["?"], label: "shortcuts.showShortcuts" },
      { keys: ["Esc"], label: "shortcuts.closePanel" },
      { keys: ["G", "H"], label: "shortcuts.goToIndex" },
      { keys: ["G", "B"], label: "shortcuts.goToBenchmarks" },
      { keys: ["G", "D"], label: "shortcuts.goToQuickstart" },
    ],
  },
  {
    group: "shortcuts.quickSearch",
    items: [
      { keys: ["↑"], label: "shortcuts.prevResult" },
      { keys: ["↓"], label: "shortcuts.nextResult" },
      { keys: ["↵"], label: "shortcuts.selectResult" },
    ],
  },
];

export function ShortcutOverlay({
  onNavigate,
}: {
  onNavigate: (v: "home" | "benchmarks" | "docs") => void;
}) {
  const { t } = useLanguage();
  const [open, setOpen] = useState(false);
  const overlayRef = useRef<HTMLDivElement>(null);
  useFocusTrap(overlayRef, open);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Open on "?"
      if (
        e.key === "?" &&
        !(
          e.target instanceof HTMLInputElement ||
          e.target instanceof HTMLTextAreaElement
        )
      ) {
        e.preventDefault();
        setOpen((o) => !o);
      } else if (e.key === "Escape" && open) {
        setOpen(false);
      }

      // G + letter navigation (sequential shortcut)
      if (
        e.key.toLowerCase() === "g" &&
        !(
          e.target instanceof HTMLInputElement ||
          e.target instanceof HTMLTextAreaElement
        ) &&
        !open
      ) {
        const onSecond = (ev: KeyboardEvent) => {
          window.removeEventListener("keydown", onSecond);
          const k = ev.key.toLowerCase();
          if (k === "h") onNavigate("home");
          else if (k === "b") onNavigate("benchmarks");
          else if (k === "d") onNavigate("docs");
        };
        window.addEventListener("keydown", onSecond, { once: true });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onNavigate]);

  if (!open) return null;

  return (
    <div
      ref={overlayRef}
      tabIndex={-1}
      className="fixed inset-0 z-[70] flex items-center justify-center p-4 outline-none"
      role="dialog"
      aria-modal="true"
      aria-label={t("shortcuts.ariaLabel")}
    >
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-[2px]"
        onClick={() => setOpen(false)}
      />

      <div className="relative w-full max-w-lg border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]   ">
        <div className="flex items-center justify-between border-b-4 border-black px-4 py-3 ">
          <h2 className="flex items-center gap-2 font-display text-xl uppercase text-black ">
            <Command className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
            {t("shortcuts.title")}
          </h2>
          <button
            onClick={() => setOpen(false)}
            className="inline-flex h-7 w-7 items-center justify-center border-2 border-black bg-[#F2EDE2] text-black transition-colors hover:bg-[#FF5500]   "
            aria-label={t("shortcuts.close")}
          >
            <X className="h-3.5 w-3.5" strokeWidth={3} />
          </button>
        </div>

        <div className="space-y-5 p-5">
          {SHORTCUTS.map((group) => (
            <div key={group.group}>
              <p className="mb-2 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black/40 ">
                {t(group.group)}
              </p>
              <div className="space-y-1.5">
                {group.items.map((item) => (
                  <div
                    key={item.label}
                    className="flex items-center justify-between gap-3 py-1"
                  >
                    <span className="font-tech text-xs text-black ">
                      {t(item.label)}
                    </span>
                    <div className="flex items-center gap-1">
                      {item.keys.map((k, i) => (
                        <kbd
                          key={i}
                          className="inline-flex h-6 min-w-6 items-center justify-center border-2 border-black bg-black px-1.5 font-mono text-[10px] font-bold text-[#FF5500]   "
                        >
                          {k}
                        </kbd>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div className="flex items-center justify-between border-t-2 border-black/15 px-4 py-2 ">
          <span className="font-tech text-[9px] uppercase tracking-wider text-black/40 ">
            <span className="font-tech text-[9px] uppercase tracking-wider text-black/40 ">
              {t("shortcuts.pressToClose")}
            </span>
          </span>
          <span className="font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
            VantaDB
          </span>
        </div>
      </div>
    </div>
  );
}

/**
 * ShortcutHintButton — small "?" button for the navbar that opens the overlay.
 */
export function ShortcutHintButton({ onClick }: { onClick: () => void }) {
  const { t } = useLanguage();
  return (
    <button
      onClick={onClick}
      className="hidden h-9 items-center justify-center border-4 border-black bg-[#FBF9F5] px-2 font-mono text-sm font-bold text-black shadow-[4px_4px_0_0_#000] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[2px_2px_0_0_#000]      sm:inline-flex"
      aria-label={t("shortcuts.showKeyboard")}
      title={t("shortcuts.keyboardTitle")}
    >
      ?
    </button>
  );
}
